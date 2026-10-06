//! Clutter Havok moves: the placed objects whose models carry a moving body
//! (`preview::cell::DynamicBody`), simulated by `physics::rigid` and drawn
//! where it has them.
//!
//! - Loaded places hand their bodies over as they're spawned ([`arrive`]);
//!   one that Havok moved before comes back where it came to rest
//!   (`GameState::havok_moved`, the game's "Havok moved" reference change).
//! - Each body's triangles are kept in the collider under its reference's
//!   form ID (walkers run into it, shots strike it) and moved with it.
//! - Shots striking a body push it as `Projectile::ApplyImpactForce` (Xbox
//!   PDB) does (`physics::impulses::projectile_impulse`, from
//!   `hiteffects::HitReports::shot_on_world`); explosions push the bodies in
//!   their sphere (`physics::impulses::explosion_push`, from
//!   `explosives`); the player, and people on their character controllers
//!   (`ai::move_body`, [`walkers`]), walking into one push it.
//! - Where a moved body is goes into the game state every frame it moves,
//!   so a save, or the place loading again, keeps it.
//!
//! Not done: grabbing with
//! the Z key (`0095f930`/`00960520`, its `fZKey…` settings traced but not
//! implemented), damage from flying objects (`fPhysicsDamage…`,
//! `0062be90`), models with more than one moving body (left solid where
//! placed), destructible objects' stages and debris.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use bevy::prelude::*;
use cellview::space;
use esm::FormId;
use physics::impulses::{self, ImpulseSettings};
use physics::rigid::{Mover, Pose, RigidWorld};
use preview::cell::DynamicBody;

use crate::dialogue::DialogueState;
use crate::scripts::PlacedRef;
use crate::walk::{CellCollision, Player};
use crate::GameFiles;

/// A push waiting for the simulation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Push {
    /// A shot by `weapon` along `dir` striking the triangles of `owner` at
    /// `point`.
    Shot {
        weapon: FormId,
        shooter: FormId,
        dir: [f32; 3],
        point: [f32; 3],
        owner: u32,
    },
    /// An explosion of record `explosion` at `at`, reaching `radius`.
    Blast {
        explosion: world::explosions::ExplosionRecord,
        at: [f32; 3],
        radius: f32,
    },
}

/// Bodies of places just spawned, and pushes, for the next update (shared
/// as the throw queue in `explosives` is: no system parameters change).
static ARRIVED: Mutex<Vec<DynamicBody>> = Mutex::new(Vec::new());
static PUSHES: Mutex<Vec<Push>> = Mutex::new(Vec::new());
/// The people (not the player) walking this frame, as their character
/// controllers want to move (`ai::move_body`): they push what they walk
/// into as the player does.
static WALKERS: Mutex<Vec<Mover>> = Mutex::new(Vec::new());

/// The people's walkers for the next update (replacing the last list).
pub(crate) fn walkers(list: Vec<Mover>) {
    if let Ok(mut q) = WALKERS.lock() {
        *q = list;
    }
}

/// A spawned place's bodies, for the simulation.
pub(crate) fn arrive(bodies: &[DynamicBody]) {
    if bodies.is_empty() {
        return;
    }
    if let Ok(mut q) = ARRIVED.lock() {
        q.extend(bodies.iter().cloned());
    }
}

/// A shot striking the collider's triangle owned by `owner` (0: none).
pub(crate) fn shot(
    weapon: FormId,
    shooter: FormId,
    (eye, dir): ([f32; 3], [f32; 3]),
    distance: f32,
    owner: u32,
) {
    let point = [0, 1, 2].map(|k| eye[k] + dir[k] * distance);
    if owner == 0 {
        return;
    }
    if let Ok(mut q) = PUSHES.lock() {
        q.push(Push::Shot {
            weapon,
            shooter,
            dir,
            point,
            owner,
        });
    }
}

/// An explosion going off.
pub(crate) fn blast(explosion: &world::explosions::ExplosionRecord, at: [f32; 3], radius: f32) {
    if let Ok(mut q) = PUSHES.lock() {
        q.push(Push::Blast {
            explosion: explosion.clone(),
            at,
            radius,
        });
    }
}

/// The simulation, and what it needs to know about each body.
#[derive(Resource, Default)]
pub struct Clutter {
    pub world: RigidWorld,
    /// Each body's base's editor ID and its shapes' Havok materials.
    info: HashMap<u32, (String, Vec<u32>)>,
    /// Bodies whose drawing has been seen (one that's gone since was
    /// unloaded with its place).
    drawn: HashSet<u32>,
    settings: Option<ImpulseSettings>,
    /// Scripts' enable state when the bodies' triangles were last switched.
    disabled_seen: world::Disabled,
}

/// A drawn piece of a simulated object: its transform where the object
/// was put.
#[derive(Component)]
pub struct Simulated {
    reference: u32,
    rest: Mat4,
}

pub struct ClutterPlugin;

impl Plugin for ClutterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Clutter>().add_systems(
            Update,
            (simulate, draw)
                .chain()
                .after(crate::combat::player_attack)
                .after(crate::explosives::fly_thrown)
                .after(crate::walk::walk),
        );
    }
}

/// Takes arrivals and pushes, steps the bodies and keeps the collider and
/// the state up with them.
#[allow(clippy::too_many_arguments)]
fn simulate(
    time: Res<Time>,
    game: Res<GameFiles>,
    mut state: ResMut<DialogueState>,
    mut collision: ResMut<CellCollision>,
    player: Res<Player>,
    mut clutter: ResMut<Clutter>,
    drawn: Query<&Simulated>,
) {
    let order = &game.0.order;
    let state = &mut state.0;
    let clutter = &mut *clutter;
    let settings = *clutter.settings.get_or_insert_with(|| {
        ImpulseSettings::read(|name| world::scripting::game_setting(order, name))
    });
    // New bodies (or ones whose place loaded again): where the state has
    // them, else where they're placed.
    let arrived: Vec<DynamicBody> = ARRIVED
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for body in arrived {
        let reference = body.reference.0;
        if let Some(old) = clutter.world.find(reference) {
            clutter.world.bodies.remove(old);
        }
        clutter.drawn.remove(&reference);
        collision.0.set_hidden(reference, false);
        let i = clutter.world.add(body.setup, body.pose);
        if let Some(&pose) = state.havok_moved.get(&body.reference) {
            clutter.world.place(i, pose, true);
        }
        if body.settle {
            // Havok settles it as its place loads (placed a little sunk
            // into what holds it, or left in the air by a save: its speed
            // isn't kept).
            clutter.world.wake(i);
        }
        if collision.0.owns(reference) {
            let (r, t) = clutter.world.delta(i);
            collision.0.move_owner(reference, &r, t);
        }
        clutter
            .info
            .insert(reference, (body.name.clone(), body.materials.clone()));
    }
    // Bodies whose drawing went (their place unloaded) leave.
    let shown: HashSet<u32> = drawn.iter().map(|s| s.reference).collect();
    let gone: Vec<u32> = clutter
        .drawn
        .iter()
        .filter(|r| !shown.contains(r))
        .copied()
        .collect();
    for r in gone {
        clutter.drawn.remove(&r);
        clutter.world.remove(r);
        clutter.info.remove(&r);
    }
    clutter.drawn.extend(shown);
    if clutter.world.bodies.is_empty() {
        return;
    }
    // Each body's triangles in the collider (a new collider after the
    // loaded squares change has none), switched off while disabled (looked
    // at again when scripts enable or disable something).
    let recheck = clutter.disabled_seen != state.disabled;
    for i in 0..clutter.world.bodies.len() {
        let b = &clutter.world.bodies[i];
        let reference = b.setup.reference;
        let mut check = recheck;
        if !collision.0.owns(reference) {
            check = true;
            let (r0, t0) = b.rest();
            let materials = clutter
                .info
                .get(&reference)
                .map(|(_, m)| m.clone())
                .unwrap_or_default();
            let surface = physics::Surface {
                friction: b.setup.friction,
                restitution: b.setup.restitution,
            };
            for (k, shape) in b.setup.shapes.iter().enumerate() {
                let (v, t, shell) = shape.triangles();
                let placed: Vec<[f32; 3]> = v
                    .iter()
                    .map(|&p| {
                        let r = [0, 1, 2]
                            .map(|row| r0[row][0] * p[0] + r0[row][1] * p[1] + r0[row][2] * p[2]);
                        [r[0] + t0[0], r[1] + t0[1], r[2] + t0[2]]
                    })
                    .collect();
                let material = materials.get(k).copied().unwrap_or(physics::NO_MATERIAL);
                collision.0.add_solid_surface(
                    &placed,
                    &t,
                    (shell, reference, material),
                    Some(surface),
                );
            }
            let (r, t) = clutter.world.delta(i);
            collision.0.move_owner(reference, &r, t);
        }
        if check {
            let enabled = world::enabled_now(order, FormId(reference), &state.disabled);
            if collision.0.is_hidden(reference) == enabled {
                collision.0.set_hidden(reference, !enabled);
            }
        }
    }
    clutter.disabled_seen = state.disabled.clone();
    // The player pushes what they walk into.
    let c = &player.character;
    let shape = physics::CharacterShape::PLAYER;
    let mut movers = if player.walking && player.ready {
        vec![Mover {
            feet: c.feet,
            radius: shape.radius,
            height: shape.height,
            velocity: [c.horizontal[0], c.horizontal[1], c.vertical_speed],
        }]
    } else {
        Vec::new()
    };
    // And the people walking about (`ai::move_body`).
    if let Ok(q) = WALKERS.lock() {
        movers.extend(q.iter().copied());
    }
    clutter.world.set_movers(movers);
    // Shots and blasts.
    let pushes: Vec<Push> = PUSHES
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for push in pushes {
        apply(clutter, order, state, &settings, push);
    }
    // The step, and what moved.
    let awake: Vec<usize> = (0..clutter.world.bodies.len())
        .filter(|&i| !clutter.world.bodies[i].asleep)
        .collect();
    clutter.world.update(&collision.0, time.delta_secs());
    let woken = (0..clutter.world.bodies.len()).filter(|&i| !clutter.world.bodies[i].asleep);
    let moving: std::collections::BTreeSet<usize> = awake.into_iter().chain(woken).collect();
    for i in moving {
        let b = &clutter.world.bodies[i];
        let reference = b.setup.reference;
        let (r, t) = clutter.world.delta(i);
        collision.0.move_owner(reference, &r, t);
        if b.moved {
            state.havok_moved.insert(FormId(reference), b.pose());
        }
        if b.asleep {
            let name = clutter.info.get(&reference).map_or("", |(n, _)| n.as_str());
            let at = b.pose().1;
            println!(
                "{:.1} s: {} ({name}) comes to rest at ({:.1}, {:.1}, {:.1}).",
                time.elapsed_secs(),
                FormId(reference),
                at[0],
                at[1],
                at[2]
            );
        }
    }
}

/// The projectile a weapon's shot is: its ammunition's (`AMMO` `DAT2` form
/// at 4) when it names one, else the weapon's own (as `world::vats` reads
/// it).
fn fired_projectile(
    order: &esm::LoadOrder,
    state: &world::scripting::GameState,
    shooter: FormId,
    weapon: &world::combat::Weapon,
) -> Option<FormId> {
    let from_ammo = weapon
        .ammo_in_use(order, state, shooter)
        .and_then(|a| order.get(a))
        .and_then(|rr| {
            let d = rr
                .record()
                .ok()?
                .get(esm::FourCC::new(b"DAT2"))?
                .data
                .clone();
            (d.len() >= 8)
                .then(|| {
                    rr.plugin
                        .to_global(FormId(u32::from_le_bytes([d[4], d[5], d[6], d[7]])))
                })
                .filter(|f| f.0 != 0)
        });
    from_ammo.or(weapon.projectile)
}

/// One push on the bodies.
fn apply(
    clutter: &mut Clutter,
    order: &esm::LoadOrder,
    state: &mut world::scripting::GameState,
    settings: &ImpulseSettings,
    push: Push,
) {
    match push {
        Push::Shot {
            weapon,
            shooter,
            dir,
            point,
            owner,
        } => {
            let Some(i) = clutter.world.find(owner) else {
                return;
            };
            let name = clutter.info.get(&owner).map_or("", |(n, _)| n.as_str());
            let Some(w) = world::combat::Weapon::load(order, weapon) else {
                println!(
                    "  the shot strikes {} ({name}): no weapon record",
                    FormId(owner)
                );
                return;
            };
            let Some(projectile) = fired_projectile(order, state, shooter, &w)
                .and_then(|p| world::explosions::ProjectileRecord::load(order, p))
            else {
                println!(
                    "  the shot strikes {} ({name}): no projectile",
                    FormId(owner)
                );
                return;
            };
            // `Projectile::ProcessImpacts` (`009c1b70`) leaves the push to
            // the explosion for projectiles that go off on impact.
            if projectile.explodes() && !projectile.alt_trigger() {
                return;
            }
            let b = &clutter.world.bodies[i];
            if !impulses::moves(b.setup.motion) {
                return;
            }
            let Some(j) = impulses::projectile_impulse(
                projectile.impact_force,
                dir,
                b.setup.layer,
                b.setup.mass,
                settings,
            ) else {
                println!(
                    "  the shot strikes {} ({name}): {} has no impact force",
                    FormId(owner),
                    projectile.form_id
                );
                return;
            };
            println!(
                "  the shot pushes {} ({}): impact force {} → impulse {:.1} (layer {}, mass {})",
                FormId(owner),
                clutter.info.get(&owner).map_or("", |(n, _)| n.as_str()),
                projectile.impact_force,
                (j[0] * j[0] + j[1] * j[1] + j[2] * j[2]).sqrt(),
                b.setup.layer,
                b.setup.mass
            );
            clutter.world.apply_point_impulse(i, j, point);
        }
        Push::Blast {
            explosion,
            at,
            radius,
        } => {
            let push_source_only =
                explosion.flags & world::explosions::expl_flags::PUSH_SOURCE_ONLY != 0;
            for i in 0..clutter.world.bodies.len() {
                let b = &clutter.world.bodies[i];
                // In the explosion's sphere (`Explosion::InitHavok`, a phantom
                // of its radius): taken here as the body's centre within the
                // radius and its reach.
                let d = {
                    let c = b.center();
                    ((c[0] - at[0]).powi(2) + (c[1] - at[1]).powi(2) + (c[2] - at[2]).powi(2))
                        .sqrt()
                };
                if d > radius {
                    continue;
                }
                // The explosion's source reference (`+0xcc`) isn't tracked
                // here: no body counts as the source's.
                if !impulses::explosion_pushes(
                    b.setup.motion,
                    b.setup.layer,
                    push_source_only,
                    false,
                ) {
                    continue;
                }
                let mut random = || {
                    // −1..1 from the state's dice (`00476b70`).
                    (state.roll() % 2_000_001) as f32 / 1_000_000.0 - 1.0
                };
                let Some((linear, angular)) = impulses::explosion_push(
                    explosion.force,
                    at,
                    b.center(),
                    b.setup.layer,
                    b.setup.mass,
                    false,
                    settings,
                    &mut random,
                ) else {
                    continue;
                };
                clutter.world.apply_linear_impulse(i, linear);
                clutter.world.apply_angular_impulse(i, angular);
            }
        }
    }
}

/// A game-space rigid move as one in Bevy's space (meters, y up).
fn bevy_delta((r, t): Pose) -> Mat4 {
    let mut m = [0.0f32; 16];
    for col in 0..3 {
        for row in 0..3 {
            m[col * 4 + row] = r[row][col];
        }
    }
    m[12] = t[0];
    m[13] = t[1];
    m[14] = t[2];
    m[15] = 1.0;
    let identity = Mat4::IDENTITY.to_cols_array();
    let to_view = Mat4::from_cols_array(&space::matrix(&identity));
    to_view * Mat4::from_cols_array(&m) * to_view.inverse()
}

/// A drawn piece just spawned: its entity, reference and transform.
type NewPiece = (Entity, &'static PlacedRef, &'static Transform);

/// Draws simulated objects where their bodies are.
fn draw(
    mut commands: Commands,
    clutter: Res<Clutter>,
    new: Query<NewPiece, (Added<PlacedRef>, Without<Simulated>)>,
    mut pieces: Query<(&Simulated, &mut Transform)>,
) {
    for (entity, placed, transform) in &new {
        if clutter.world.find(placed.0).is_some() {
            commands.entity(entity).insert(Simulated {
                reference: placed.0,
                rest: transform.compute_matrix(),
            });
        }
    }
    for (piece, mut transform) in &mut pieces {
        let Some(i) = clutter.world.find(piece.reference) else {
            continue;
        };
        let wanted = Transform::from_matrix(bevy_delta(clutter.world.delta(i)) * piece.rest);
        if *transform != wanted {
            *transform = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deltas_move_drawings_as_they_move_bodies() {
        // A quarter turn about the game's z and a move east and up: a point
        // drawn at the game's (10, 0, 0) goes to (0, 10, 0) + (5, 0, 2).
        let r = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let m = bevy_delta((r, [5.0, 0.0, 2.0]));
        let drawn = Vec3::from(space::point([10.0, 0.0, 0.0]));
        let moved = m.transform_point3(drawn);
        let want = Vec3::from(space::point([5.0, 10.0, 2.0]));
        assert!((moved - want).length() < 1e-5, "{moved} vs {want}");
    }

    #[test]
    fn shots_are_queued_only_for_owned_triangles() {
        let before = PUSHES.lock().map(|q| q.len()).unwrap_or(0);
        shot(
            FormId(1),
            FormId(0x14),
            ([0.0; 3], [1.0, 0.0, 0.0]),
            50.0,
            0,
        );
        assert_eq!(PUSHES.lock().map(|q| q.len()).unwrap_or(0), before);
        shot(
            FormId(1),
            FormId(0x14),
            ([0.0; 3], [1.0, 0.0, 0.0]),
            50.0,
            0x77,
        );
        let q = PUSHES.lock().unwrap();
        assert!(q.iter().any(|p| matches!(
            p,
            Push::Shot { owner: 0x77, point, .. } if point[0] == 50.0
        )));
    }
}

//! Grenades, dynamite and their explosions (`world::explosions`, read from
//! the game's code; evidence in `docs/EXPLOSIVES.md`). The player's attack
//! (`combat`) and people's (`fighting`) throw a weapon of animation type
//! 10–13 here instead of shooting it: the throw uses up one of the weapon,
//! and its projectile flies under gravity through the cell's collision
//! until its fuse runs out (dynamite: 2.5 s) or, for impact grenades, it
//! strikes something. Its explosion hurts everyone alive within its radius
//! and line of sight by its damage × the falloff, through the hit path
//! (`OnHit`, armour, health, `OnDeath`, fighting back), and plays its two
//! sounds; the hits are reported to `hiteffects`.
//!
//! People throw at the point their projectile's arc lands on the target
//! (its feet, for splash damage), low first, else high at
//! `fGrenadeHighArcSpeedPercentage` of the speed when the low arc meets
//! something before `fGrenadeThrowHitFractionThreshold` of the way; with
//! neither clear they don't throw.
//!
//! Measured here (guesses where the game's way isn't traced): the player
//! throws from the eye along the view, people from 60 units above their
//! feet (the hand node isn't posed); the throw leaves at once (not at the
//! throw animation's release); bodies' line-of-sight offsets use the
//! people's controller radius; targets are aimed at as 128 units tall. Not
//! drawn: the projectile's and the explosion's models, its light, image
//! space, decals, camera shake; not done: the limbs the blast reaches, the
//! force it pushes bodies and objects with, knockdowns, mines' proximity,
//! hits on objects (destructibles).

use std::sync::Mutex;

use bevy::prelude::*;
use esm::FormId;
use world::combat::Weapon;
use world::dialogue::PLAYER_REF;
use world::explosions::{
    self, ExplosionRecord, Flight, FlightEvent, FlightSettings, ProjectileRecord,
};
use world::scripting::Runner;

use crate::dialogue::{DialogueState, Talkers};
use crate::scripts::Scripts;
use crate::sounds::SoundRequests;
use crate::walk::{CellCollision, Player};
use crate::GameFiles;

/// How high above the feet people throw from (see the module notes).
pub(crate) const THROW_HEIGHT: f32 = 60.0;

/// How tall a target is taken to be for the AI's aim (see the notes).
const TARGET_HEIGHT: f32 = 128.0;

/// Where a throw goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Aim {
    /// Along a direction (unit): the player's view.
    Along([f32; 3]),
    /// To land at a target standing at this point (the AI).
    At([f32; 3]),
}

/// A throw asked for by an attack.
#[derive(Debug, Clone)]
pub(crate) struct Launch {
    pub thrower: FormId,
    pub weapon: Weapon,
    pub origin: [f32; 3],
    pub aim: Aim,
}

/// Throws asked for this frame. The attacks are in systems that can't take
/// more parameters (`fighting` runs inside `ai`'s), so they queue here.
static QUEUE: Mutex<Vec<Launch>> = Mutex::new(Vec::new());

/// Queues a throw for [`fly_thrown`].
pub(crate) fn throw(launch: Launch) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push(launch);
    }
}

/// A projectile in flight, and who threw it with what.
struct InFlight {
    thrower: FormId,
    weapon: Weapon,
    flight: Flight,
    settings: FlightSettings,
}

/// What's in flight.
#[derive(Resource, Default)]
pub struct Thrown(Vec<InFlight>);

pub struct ExplosivesPlugin;

impl Plugin for ExplosivesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Thrown>().add_systems(
            Update,
            fly_thrown
                .after(crate::combat::player_attack)
                .after(crate::ai::move_actors)
                .before(crate::hiteffects::play_hits),
        );
    }
}

/// A surface's unit normal, facing against `dir`.
fn facing_normal(collider: &physics::Collider, tri: u32, dir: [f32; 3]) -> [f32; 3] {
    let [a, b, c] = collider.triangle(tri);
    let (u, v) = (
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    );
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
    let n = n.map(|x| x / l);
    if n[0] * dir[0] + n[1] * dir[1] + n[2] * dir[2] > 0.0 {
        n.map(|x| -x)
    } else {
        n
    }
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (l > 1e-6).then(|| v.map(|x| x / l))
}

/// The direction and speed for an AI's throw at `target` (see the module
/// notes): the low arc, else the high one, if either is clear.
fn ai_throw(
    order: &esm::LoadOrder,
    collider: &physics::Collider,
    projectile: &ProjectileRecord,
    explosion: Option<&ExplosionRecord>,
    (origin, target): ([f32; 3], [f32; 3]),
    speed: f32,
) -> Option<([f32; 3], f32)> {
    let s = |n: &str, d: f32| world::scripting::game_setting(order, n).unwrap_or(d);
    let g = world::combat_ai::WORLD_GRAVITY * projectile.fall_gravity();
    let splash = (
        s("fCombatSplashDamageMaxSpeed", 3000.0),
        s("fCombatSplashDamageMinRadius", 50.0),
        s("fCombatSplashDamageMinDamage", 20.0),
    );
    let blast = explosion.map(|e| (e.radius_units(s("fBSUnitsPerFoot", 22.0)), e.damage));
    let segments = s("iBallisticProjectilePathPickSegments", 4.0) as u32;
    let threshold = s("fGrenadeThrowHitFractionThreshold", 0.8);
    let mut cast =
        |from: [f32; 3], dir: [f32; 3], len: f32| collider.raycast(from, dir, len).map(|(d, _)| d);
    for high in [false, true] {
        let mode = explosions::aim_mode(projectile, blast, high, splash);
        let point = [
            target[0],
            target[1],
            target[2] + explosions::aim_height(mode, TARGET_HEIGHT),
        ];
        let v = if high {
            speed * s(explosions::HIGH_ARC_SETTING, 0.67)
        } else {
            speed
        };
        let aim = explosions::aim_point(origin, point, v, g, high);
        let Some(dir) = normalize([aim[0] - origin[0], aim[1] - origin[1], aim[2] - origin[2]])
        else {
            continue;
        };
        if explosions::arc_clear(origin, dir, (v, g), point, (segments, threshold), &mut cast) {
            return Some((dir, v));
        }
    }
    None
}

/// One of the weapon used up by a throw; the last one leaves the hand.
fn use_up(order: &esm::LoadOrder, state: &mut world::scripting::GameState, who: FormId, w: FormId) {
    state.stock(order, who);
    let n = state.items.entry((who, w)).or_insert(0);
    *n = (*n - 1).max(0);
    if *n == 0 {
        state.unequip(who, w);
    }
}

/// Launches queued throws, flies what's in flight, and sets off what goes
/// off.
#[allow(clippy::too_many_arguments)]
pub fn fly_thrown(
    time: Res<Time>,
    game: Res<GameFiles>,
    scripts: Res<Scripts>,
    mut state: ResMut<DialogueState>,
    (collision, talkers, player): (Res<CellCollision>, Res<Talkers>, Res<Player>),
    mut sounds: ResMut<SoundRequests>,
    mut hits: ResMut<crate::hiteffects::HitReports>,
    mut thrown: ResMut<Thrown>,
) {
    let order = &game.0.order;
    let state = &mut state.0;
    let collider = &collision.0;
    let launches: Vec<Launch> = QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for l in launches {
        let Some(projectile) = l
            .weapon
            .projectile
            .and_then(|p| ProjectileRecord::load(order, p))
        else {
            continue;
        };
        let explosion = projectile
            .explosion
            .and_then(|e| ExplosionRecord::load(order, e));
        let speed = explosions::launch_speed(order, state, l.thrower, &l.weapon, &projectile, 1.0);
        let (dir, speed) = match l.aim {
            Aim::Along(d) => (d, speed),
            Aim::At(target) => {
                match ai_throw(
                    order,
                    collider,
                    &projectile,
                    explosion.as_ref(),
                    (l.origin, target),
                    speed,
                ) {
                    Some(t) => t,
                    None => {
                        println!(
                            "{:.1} s: {} holds the {}: no clear arc.",
                            time.elapsed_secs(),
                            l.thrower,
                            l.weapon.name
                        );
                        continue;
                    }
                }
            }
        };
        use_up(order, state, l.thrower, l.weapon.form_id);
        println!(
            "{:.1} s: {} throws the {} at {speed:.0} units a second ({:.1} s fuse).",
            time.elapsed_secs(),
            l.thrower,
            l.weapon.name,
            projectile.timer
        );
        let settings = FlightSettings::read(order, &projectile);
        thrown.0.push(InFlight {
            thrower: l.thrower,
            weapon: l.weapon,
            flight: Flight::launch(projectile, l.origin, dir, speed),
            settings,
        });
    }
    if thrown.0.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    let mut cast = |from: [f32; 3], dir: [f32; 3], len: f32| {
        collider
            .raycast(from, dir, len)
            .map(|(d, tri)| (d, facing_normal(collider, tri, dir)))
    };
    let mut going_off = Vec::new();
    thrown
        .0
        .retain_mut(|f| match f.flight.step(dt, &f.settings, &mut cast) {
            FlightEvent::Flying => true,
            FlightEvent::Expired => false,
            FlightEvent::Explode { at } => {
                going_off.push((f.thrower, f.weapon.clone(), f.flight.projectile.clone(), at));
                false
            }
        });
    for (thrower, weapon, projectile, at) in going_off {
        let Some(e) = projectile
            .explosion
            .and_then(|e| ExplosionRecord::load(order, e))
        else {
            continue;
        };
        let feet = player.character.feet;
        explode(
            order,
            &scripts.0,
            state,
            collider,
            &talkers,
            feet,
            (thrower, &weapon, &e, at),
            &mut sounds,
            &mut hits,
        );
    }
}

/// An explosion at `at`: its sounds, and its hits on everyone it reaches.
#[allow(clippy::too_many_arguments)]
fn explode(
    order: &esm::LoadOrder,
    scripts: &world::scripting::ScriptCache,
    state: &mut world::scripting::GameState,
    collider: &physics::Collider,
    talkers: &Talkers,
    player_feet: [f32; 3],
    (thrower, weapon, e, at): (FormId, &Weapon, &ExplosionRecord, [f32; 3]),
    sounds: &mut SoundRequests,
    hits: &mut crate::hiteffects::HitReports,
) {
    sounds.0.extend(e.sound);
    sounds.0.extend(e.sound2);
    let radius = explosions::blast_radius(order, state, Some(thrower), Some(weapon.form_id), e);
    let damage = explosions::base_damage(order, state, Some(thrower), Some(weapon), e);
    // It pushes the clutter in its sphere (`clutter`).
    crate::clutter::blast(e, at, radius);
    println!(
        "{} goes off at ({:.0}, {:.0}, {:.0}): {damage:.1} damage, {radius:.0} units.",
        e.form_id, at[0], at[1], at[2]
    );
    let mut candidates: Vec<(FormId, [f32; 3])> = talkers
        .0
        .iter()
        .filter(|t| !state.dead.contains(&t.reference))
        .filter(|t| !world::more_functions::is_ghost(state, t.reference))
        .map(|t| (t.reference, t.position))
        .collect();
    if !state.dead.contains(&PLAYER_REF) {
        candidates.push((PLAYER_REF, player_feet));
    }
    let buffer = world::scripting::game_setting(order, "fExplosionLOSBuffer").unwrap_or(6.0);
    let mut cast =
        |from: [f32; 3], dir: [f32; 3], len: f32| collider.raycast(from, dir, len).map(|(d, _)| d);
    for t in explosions::blast_targets(at, radius, &candidates) {
        let Some(&(_, position)) = candidates.iter().find(|c| c.0 == t.reference) else {
            continue;
        };
        let sees = e.flags & explosions::expl_flags::IGNORE_LOS != 0
            || explosions::los_clear(
                at,
                position,
                Some(world::combat_ai::PERSON_RADIUS),
                buffer,
                &mut cast,
            );
        if !sees {
            println!("  {} is shielded from it.", t.reference);
            continue;
        }
        let Some(hit) = Runner::new(order, scripts, state).explosion_hit(
            Some(thrower),
            t.reference,
            Some(weapon),
            damage * t.share,
        ) else {
            continue;
        };
        let killed = state.dead.contains(&t.reference);
        let left = world::combat::health(order, state, t.reference)
            .unwrap_or(0.0)
            .max(0.0);
        println!(
            "  {} at {:.0} units takes {:.1} ({left:.1} left){}.",
            t.reference,
            t.distance,
            hit.dealt,
            if killed { ", killed" } else { "" }
        );
        hits.0.push(crate::hiteffects::HitReport {
            attacker: thrower,
            target: Some(t.reference),
            weapon: Some(weapon.form_id),
            point: position,
            havok: None,
            damage: hit.dealt,
            killed,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normals_face_the_ray() {
        let mut c = physics::Collider::new();
        c.add(
            &[
                [-100.0, -100.0, 0.0],
                [100.0, -100.0, 0.0],
                [0.0, 100.0, 0.0],
            ],
            &[[0, 1, 2]],
        );
        let (_, tri) = c
            .raycast([0.0, 0.0, 50.0], [0.0, 0.0, -1.0], 100.0)
            .unwrap();
        assert_eq!(facing_normal(&c, tri, [0.0, 0.0, -1.0]), [0.0, 0.0, 1.0]);
        assert_eq!(facing_normal(&c, tri, [0.0, 0.0, 1.0]), [0.0, 0.0, -1.0]);
    }

    #[test]
    fn queued_throws_are_taken_once() {
        let w = Weapon {
            form_id: FormId(1),
            name: "Dynamite".into(),
            damage: 1.0,
            clip: 0,
            health: 0,
            animation: 10,
            ammo: Vec::new(),
            ammo_use: 1,
            min_spread: 0.0,
            spread: 0.0,
            projectile: None,
            projectiles: 1,
            min_range: 0.0,
            max_range: 0.0,
            shots_per_second: 1.0,
            reload_time: 1.0,
            skill: 33,
            crit_damage: 0.0,
            crit_mult: 1.0,
            sound: None,
            attack_animation: 0,
            reload_animation: 0,
            kill_impulse: 0.0,
            impulse_distance: 0.0,
            reach: 0.0,
            limb_damage_mult: 1.0,
            flags1: 0,
            flags2: 0,
            fire_rate: 0.0,
            attack_mult: 1.0,
            aim_arc: 0.0,
            semi_auto_delay: (0.0, 0.0),
            speed: 1.0,
            cone_mult: 1.0,
            crit_effect: None,
            crit_on_death: false,
            resist: None,
        };
        throw(Launch {
            thrower: PLAYER_REF,
            weapon: w,
            origin: [0.0; 3],
            aim: Aim::Along([0.0, 1.0, 0.0]),
        });
        let taken: Vec<Launch> = std::mem::take(&mut *QUEUE.lock().unwrap());
        assert_eq!(taken.len(), 1);
        assert!(QUEUE.lock().unwrap().is_empty());
    }
}

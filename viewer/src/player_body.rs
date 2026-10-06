//! The player's body in third person: built as people are built
//! (`world::actor::player_look`: the player's record for race, face and
//! hair, the game state for sex, worn clothes and the weapon in hand) on
//! the third-person skeleton, and animated through the same path as
//! everyone (`actors::ActorRig`, driven by `actors::animate_actors`): the
//! idle; the walk and run in the direction the keys move the player (the
//! mover's flags, `00895110` picking `Forward` … `FastRight`, the
//! back/left/right files from the walk's folder); the weapon's aim and
//! attacks while it's out; sitting down, the seated loop and getting up
//! from the furniture procedure (`world::furniture::Sitter`).
//!
//! Shown while the game shows the third-person body
//! (`PlayerCamera::actually_third`, `00951a10`); the first-person view
//! (`viewmodel`) shows otherwise.
//!
//! Not yet (left out, not substituted): sneaking's and jumping's groups
//! (`sneakmt*.kf`, `mtjump*.kf`; how the game picks them isn't traced),
//! the weapon kind's own movement groups while it's out (back, left and
//! right then play nothing), turning in place, the face's blinking and lip
//! movement, the body fading when the camera is inside it, the dead
//! player's ragdoll.

use std::sync::Arc;

use bevy::prelude::*;
use cellview::space;
use esm::FormId;
use world::animation::{group, MoveFlags};
use world::dialogue::PLAYER_REF;

use crate::actors::ActorRig;
use crate::walk::Player;
use crate::{FlyCamera, GameFiles, Spawner};

/// The movement groups' files beside the walk (`mtforward.kf`) in its
/// folder (`Characters\_Male\locomotion\male\`): the game loads every
/// `.kf` of the skeleton's `locomotion` folders for an actor (`00447330`,
/// see `preview::actor::turn_path`) and picks them by group.
const MOVES: [(u8, &str); 7] = [
    (group::BACKWARD, "mtbackward.kf"),
    (group::LEFT, "mtleft.kf"),
    (group::RIGHT, "mtright.kf"),
    (group::FAST_FORWARD, "mtfastforward.kf"),
    (group::FAST_BACKWARD, "mtfastbackward.kf"),
    (group::FAST_LEFT, "mtfastleft.kf"),
    (group::FAST_RIGHT, "mtfastright.kf"),
];

/// A file beside the walk animation.
fn beside(walk: &str, file: &str) -> String {
    let folder = walk.rfind(['\\', '/']).map_or("", |i| &walk[..=i]);
    format!("{folder}{file}")
}

/// What the body is built from, to know when to rebuild it.
#[derive(Clone, PartialEq)]
struct Built {
    weapon: Option<FormId>,
    worn: Vec<FormId>,
    female: bool,
    lighting: u64,
}

/// The player's third-person body on screen.
#[derive(Resource, Default)]
pub struct PlayerBody {
    built: Option<Built>,
    /// The entity placing it where the player stands, and the skeleton's
    /// root (which carries the `ActorRig`).
    holder: Option<Entity>,
    root: Option<Entity>,
    scale: f32,
    /// The movement groups besides the walk and run, loaded.
    moves: Vec<(u8, Arc<nif::Sequence>)>,
}

/// The holder's placement: at the feet, turned to the heading (clockwise
/// from north), at the actor's scale.
fn placement(feet: [f32; 3], heading: f32, scale: f32) -> Transform {
    let (sh, ch) = heading.sin_cos();
    let m = [
        ch * scale,
        -sh * scale,
        0.0,
        0.0,
        sh * scale,
        ch * scale,
        0.0,
        0.0,
        0.0,
        0.0,
        scale,
        0.0,
        feet[0],
        feet[1],
        feet[2],
        1.0,
    ];
    Transform::from_matrix(Mat4::from_cols_array(&space::matrix(&m)))
}

/// Keeps the body built for what the player is and wears, placed and told
/// what to play; `actors::animate_actors` poses it after.
#[allow(clippy::too_many_arguments)]
pub fn update_player_body(
    mut commands: Commands,
    game: Res<GameFiles>,
    state: Res<crate::dialogue::DialogueState>,
    player: Res<Player>,
    view: Res<crate::player_camera::PlayerView>,
    attack: Res<crate::combat::PlayerAttack>,
    mut seats: ResMut<crate::sitting::Seats>,
    mut body: ResMut<PlayerBody>,
    mut spawner: Spawner,
    cameras: Query<&FlyCamera>,
    mut placed: Query<(&mut Transform, &mut Visibility)>,
    mut rigs: Query<&mut ActorRig>,
) {
    let order = &game.0.order;
    let st = &state.0;
    let Some(lighting) = spawner.place_lighting.get() else {
        return;
    };
    let weapon = world::combat::weapon_in_hand(order, st, PLAYER_REF);
    let worn: Vec<FormId> = st
        .equipped
        .get(&PLAYER_REF)
        .into_iter()
        .flatten()
        .copied()
        .filter(|&i| {
            order
                .get(i)
                .is_some_and(|r| r.entry.header.kind.as_bytes() == b"ARMO")
        })
        .collect();
    let wanted = Built {
        weapon: weapon.as_ref().map(|w| w.form_id),
        worn,
        female: st.player_female.unwrap_or(false),
        lighting: spawner.place_lighting.changes(),
    };
    if body.built.as_ref() != Some(&wanted) {
        if let Some(old) = body.holder.take() {
            if let Ok(mut e) = commands.get_entity(old) {
                e.despawn();
            }
        }
        body.root = None;
        let look = world::actor::player_look(order, wanted.female, &wanted.worn, wanted.weapon);
        body.built = Some(wanted);
        let Some(look) = look else {
            return;
        };
        let scene = game.0.actor_scene(&look);
        let holder = commands
            .spawn((Transform::IDENTITY, Visibility::Hidden))
            .id();
        body.holder = Some(holder);
        let Some((root, joints, _)) = spawner.spawn_lone_actor_on(&scene, lighting, holder, 0)
        else {
            return;
        };
        let skeleton = scene.actors[0].skeleton.clone();
        body.scale = look.scale;
        let mut rig = ActorRig::new(skeleton, look.scale, 0.0);
        rig.joints = joints;
        body.moves = MOVES
            .iter()
            .filter_map(|&(g, file)| {
                crate::viewmodel::sequence(&game.0, &beside(&look.walk, file))
                    .map(|s| (g, Arc::new(s)))
            })
            .collect();
        commands.entity(root).insert(rig);
        body.root = Some(root);
        return;
    }
    let (Some(holder), Some(root)) = (body.holder, body.root) else {
        return;
    };
    let shown = view.camera.actually_third && player.ready;
    let sitter = st.sitters.get(&PLAYER_REF);
    let heading = match (sitter, cameras.single()) {
        (Some(s), _) => s.heading,
        (None, Ok(fly)) => (-fly.yaw).rem_euclid(std::f32::consts::TAU),
        (None, Err(_)) => 0.0,
    };
    if let Ok((mut transform, mut visibility)) = placed.get_mut(holder) {
        let want = if shown {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        *transform = placement(player.character.feet, heading, body.scale);
    }
    let Ok(mut rig) = rigs.get_mut(root) else {
        return;
    };
    // The keys' movement; with the weapon out, its kind's movement groups
    // (not loaded) would play, so only forward (the walk and run) does.
    let moving = player.speed > 0.0 && sitter.is_none();
    rig.walking = moving;
    rig.running = moving && player.moving.running;
    rig.speed = if moving {
        player.speed * body.scale
    } else {
        0.0
    };
    rig.direction = moving.then_some(MoveFlags {
        turn_left: false,
        turn_right: false,
        ..player.moving
    });
    rig.moves = if attack.out {
        Vec::new()
    } else {
        body.moves.clone()
    };
    rig.fighting = attack.out;
    rig.attack_at = attack.fired_at;
    // Furniture: the entry or exit over everything, the seat's loop
    // under it (as `sitting::furniture_frame` gives people).
    match sitter {
        Some(s) => {
            rig.dynamic_idle = s
                .dynamic_idle
                .as_ref()
                .and_then(|(_, model)| seats.sequence(&game.0, model));
            rig.overlay = s
                .playing
                .as_ref()
                .and_then(|p| Some((seats.sequence(&game.0, &p.model)?, p.elapsed)));
        }
        None => {
            rig.dynamic_idle = None;
            rig.overlay = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_movement_files_sit_beside_the_walk() {
        assert_eq!(
            beside(
                "Characters\\_Male\\locomotion\\female\\mtforward.kf",
                "mtbackward.kf"
            ),
            "Characters\\_Male\\locomotion\\female\\mtbackward.kf"
        );
    }

    #[test]
    fn the_body_stands_at_the_feet_facing_the_heading() {
        // Heading east: the body's forward (its y) points east.
        let t = placement([100.0, 200.0, 10.0], std::f32::consts::FRAC_PI_2, 1.0);
        let forward = t.rotation * Vec3::Y;
        let east = Vec3::from(space::direction([1.0, 0.0, 0.0]));
        assert!(forward.abs_diff_eq(east, 1e-5), "{forward} vs {east}");
        let at = Vec3::from(space::point([100.0, 200.0, 10.0]));
        assert!(t.translation.abs_diff_eq(at, 1e-5));
    }
}

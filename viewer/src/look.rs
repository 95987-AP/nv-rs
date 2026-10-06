//! Native head and eye tracking on placed people (`world::look_ik`): the
//! head bone turned toward whom they look at after the animation is
//! sampled, before skinning, as the game's `bhkRagdollController` does
//! in `DoRagdollAnim` (00c7d630).
//!
//! Set up as 0087e130 does: the body part with IK data and the head
//! tracking flag (`DefaultBodyPartData`'s head) names the head bone
//! (`BPNI`) and its maximum angle; people (not creatures) also track with
//! their eyes.
//!
//! Unresolved, see docs/OPENING_LOOK_IK.md: the target choice of the
//! process update (008a3100, 008a3ed0) is not ported, so the target is
//! the reference `ai` looks at (its own trace of 008a3100) or the
//! dialogue speaker's listener; only the player as a target is supported
//! (the camera point, 00952ff0); the model frame is the skeleton root's;
//! the eye heading and pitch are computed but not yet shown on the eyes.

use bevy::prelude::*;
use world::look_ik::{havok_scale, LookIk, Pose, Qs, Settings};

use crate::actors::ActorRig;

/// The look-IK state of one placed person.
#[derive(Component)]
pub struct HeadTrack {
    pub ik: LookIk,
    /// The tracked head bone (skeleton index).
    pub head: Option<usize>,
}

/// The executable's LookIK settings (no INI overrides in the shipped
/// files).
#[derive(Resource, Default)]
pub struct LookSettings(pub Settings);


/// Gives each placed person their head tracking once their skeleton is
/// up (0087e130).
pub fn set_up(
    mut commands: Commands,
    game: Res<crate::GameFiles>,
    rigs: Query<(Entity, &ActorRig, &crate::ai::Walker), Without<HeadTrack>>,
) {
    let order = &game.0.order;
    for (entity, rig, walker) in &rigs {
        let bones = &rig.skeleton.bones;
        let mut track = HeadTrack {
            ik: LookIk::default(),
            head: None,
        };
        let ordered = bones
            .iter()
            .enumerate()
            .all(|(i, b)| b.parent.is_none_or(|p| p < i));
        let part = world::body_parts::BodyPartData::of(order, walker.reference)
            .and_then(|d| d.head_tracking_part().cloned());
        if let (Some(part), true) = (part, ordered) {
            let head = part
                .ik_start
                .as_deref()
                .and_then(|n| bones.iter().position(|b| b.name.eq_ignore_ascii_case(n)));
            let mut pose = bind_pose(bones);
            let creature = world::body_parts::is_creature(order, walker.reference);
            track
                .ik
                .init(&mut pose, head, creature, part.tracking_max_angle);
            track.head = head.filter(|_| track.ik.init);
        }
        commands.entity(entity).insert(track);
    }
}

/// The skeleton's own pose (the scene graph before animation), from
/// which 00c79340 takes the bones' forward directions.
fn bind_pose(bones: &[nif::Bone]) -> Pose {
    let parents = bones
        .iter()
        .map(|b| b.parent.map_or(-1, |p| p as i16))
        .collect();
    let locals = bones.iter().map(|b| to_qs(&b.local)).collect();
    Pose::from_locals(parents, locals)
}

/// A game transform as Havok's (translation in Havok units).
fn to_qs(t: &nif::Transform) -> Qs {
    let q = world::animation::quat_from_matrix(&t.rotation);
    let h = havok_scale();
    Qs {
        t: [
            t.translation[0] * h,
            t.translation[1] * h,
            t.translation[2] * h,
            0.0,
        ],
        q: [q[1], q[2], q[3], q[0]],
        s: [t.scale; 4],
    }
}

/// Havok's transform back as the game's.
fn from_qs(t: &Qs) -> nif::Transform {
    let [x, y, z, w] = t.q;
    let h = 1.0 / havok_scale();
    nif::Transform {
        rotation: [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - z * w),
                2.0 * (x * z + y * w),
            ],
            [
                2.0 * (x * y + z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - x * w),
            ],
            [
                2.0 * (x * z - y * w),
                2.0 * (y * z + x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ],
        translation: [t.t[0] * h, t.t[1] * h, t.t[2] * h],
        scale: t.s[0],
    }
}

/// Runs one update of head tracking on a posed skeleton: `pose` holds the
/// bones' transforms in the skeleton root's space (as `ActorRig::pose_now`
/// gives them) and is changed for the head and everything under it.
pub fn track(
    track: &mut HeadTrack,
    settings: &Settings,
    bones: &[nif::Bone],
    pose: &mut [nif::Transform],
    placement: &nif::Transform,
    target: Option<[f32; 3]>,
) {
    let Some(head) = track.head else {
        return;
    };
    // The bones' local transforms from the posed ones.
    let locals: Vec<nif::Transform> = bones
        .iter()
        .enumerate()
        .map(|(i, b)| match b.parent {
            Some(p) => pose[p].inverse().then_child(&pose[i]),
            None => pose[i],
        })
        .collect();
    let world_head = placement.apply_point(pose[head].translation);
    track.ik.update_target(settings, world_head, target);
    if !(track.ik.active && settings.look_ik && settings.ragdoll_anim) {
        return;
    }
    let parents = bones
        .iter()
        .map(|b| b.parent.map_or(-1, |p| p as i16))
        .collect();
    let mut sg = Pose::from_locals(parents, locals.iter().map(to_qs).collect());
    let world_from_model = to_qs(placement);
    track.ik.do_look_at_ik(settings, &mut sg, &world_from_model);
    // The head's new local transform; everything under it keeps its own.
    let mut locals = locals;
    locals[head] = from_qs(&sg.local(head));
    let mut below = vec![false; bones.len()];
    below[head] = true;
    for i in head..bones.len() {
        if i != head && !bones[i].parent.is_some_and(|p| below[p]) {
            continue;
        }
        below[i] = true;
        pose[i] = match bones[i].parent {
            Some(p) => pose[p].then_child(&locals[i]),
            None => locals[i],
        };
    }
}

/// Where someone looks at the player: the camera, in first person
/// (`PlayerCharacter::GetLookingAtLocation`, 00952ff0).
pub fn player_look_point(camera: &GlobalTransform) -> [f32; 3] {
    crate::walk::game_point(camera.translation())
}


//! Walking through the cell as the player, and going through load doors.
//!
//! Movement uses the game's own numbers where they're known (see
//! `MovementSettings`); the collision is the models' Havok shapes
//! (`cellview::ViewerScene::collision`) and the capsule is the `physics`
//! crate's character.

use bevy::prelude::*;
use cellview::{space, DoorData, ACTIVATE_REACH, EYE_HEIGHT};
use physics::{Character, CharacterShape, Collider};

use crate::exterior::{door_start, PendingExterior};
use crate::{FlyCamera, GameFiles, PendingScene};

/// The game's movement settings, from its code and `FalloutNV.esm`
/// (`GMST`). The game works out a speed as `SpeedMult (100) × 0.01 ×
/// fMoveBaseSpeed`, times `fMoveRunMult` when running and
/// `fMoveSneakMult` when sneaking (`00647d10` / `00647f00` in
/// FalloutNV.exe), and × 0.85 / 0.75 with one or both legs crippled
/// (`world::body_parts::leg_speed_mult`; armour penalties left out).
struct MovementSettings;

impl MovementSettings {
    /// `fMoveBaseSpeed`: 77 in FalloutNV.esm (85 built in).
    const BASE_SPEED: f32 = 77.0;
    /// `fMoveRunMult`: 4 in FalloutNV.esm.
    const RUN_MULT: f32 = 4.0;
    /// `fMoveSneakMult`: 0.57 in FalloutNV.esm.
    const SNEAK_MULT: f32 = 0.57;
    /// `fJumpHeightMin`: 64 (built in, not changed by the game's files).
    const JUMP_HEIGHT: f32 = 64.0;
}

/// The current cell's solid surfaces.
#[derive(Resource)]
pub struct CellCollision(pub Collider);

/// The current cell's load doors.
#[derive(Resource)]
pub struct Doors(pub Vec<DoorData>);

/// The line under the crosshair (a door's destination).
#[derive(Component)]
pub struct Prompt;

#[derive(Resource)]
pub struct Player {
    pub walking: bool,
    pub character: Character,
    /// Where the cell started the player, for R.
    start: [f32; 3],
    /// False while the ground under the player is still loading (outdoors).
    pub ready: bool,
    /// Dropped from the free camera (F): the landing does no damage.
    from_camera: bool,
}

impl Player {
    pub fn new(walking: bool) -> Self {
        Self {
            walking,
            character: Character::new([0.0; 3]),
            start: [0.0; 3],
            ready: true,
            from_camera: false,
        }
    }

    /// Puts the player at a cell's arrival point (feet, game units).
    pub fn arrive(&mut self, feet: [f32; 3]) {
        self.start = feet;
        self.character = Character::new(feet);
        self.ready = true;
    }

    /// The physical player's feet for a view at `eye` (game units).
    ///
    /// While walking, scripts and other systems may move the camera without
    /// moving the character. Reports and saves must keep using the character
    /// position in that case. Free-camera mode retains its legacy convention
    /// of treating the camera as the player and subtracting eye height.
    pub fn position_for_view(&self, eye: [f32; 3]) -> [f32; 3] {
        if self.walking {
            self.character.feet
        } else {
            [eye[0], eye[1], eye[2] - EYE_HEIGHT]
        }
    }
}

/// F switches between walking and flying.
pub fn toggle_walking(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    cameras: Query<&Transform, With<FlyCamera>>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    player.walking = !player.walking;
    if player.walking {
        // Land wherever the camera is.
        if let Ok(transform) = cameras.single() {
            let [x, y, z] = game_point(transform.translation);
            player.character = Character::new([x, y, z - EYE_HEIGHT]);
            player.from_camera = true;
        }
    }
}

/// A point from Bevy's space (meters, y up) back to the game's (units, z
/// up).
pub fn game_point(p: Vec3) -> [f32; 3] {
    let s = 1.0 / space::METERS_PER_UNIT;
    [p.x * s, -p.z * s, p.y * s]
}

/// A direction from Bevy's space back to the game's.
fn game_direction(d: Vec3) -> [f32; 3] {
    [d.x, -d.z, d.y]
}

/// The people the player runs into: everyone alive here, upright
/// cylinders where they stand. People's controllers all have the game's
/// one size (`physics::CharacterShape::PLAYER`, 128 tall); a creature's
/// radius comes from its skeleton (`fighting::Kit`), its height taken as
/// 128 × its scale (a guess: the game sizes it from the skeleton's
/// `BSBound`). The dead don't block (their bodies are on the `DEADBIP`
/// layer, which the player's controller passes).
fn people(
    walkers: &Query<&crate::ai::Walker>,
    state: &world::scripting::GameState,
) -> Vec<physics::Person> {
    walkers
        .iter()
        .filter(|w| !state.dead.contains(&w.reference))
        .map(|w| {
            let creature = w.kit.as_ref().is_some_and(|k| k.creature.is_some());
            physics::Person {
                feet: w.position,
                radius: w
                    .kit
                    .as_ref()
                    .map_or(CharacterShape::PLAYER.radius, |k| k.radius),
                height: CharacterShape::PLAYER.height * if creature { w.scale } else { 1.0 },
            }
        })
        .collect()
}

/// Walking: the keys set the wanted speed, the character moves through the
/// cell's collision and around the people in it, and the camera sits at eye
/// height. Not while scripts have turned movement off
/// (`DisablePlayerControls`). Landing from a fall hurts as the game's falls
/// do (`world::combat::land`).
#[allow(clippy::too_many_arguments)]
pub fn walk(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut collision: ResMut<CellCollision>,
    game: Res<GameFiles>,
    mut state: ResMut<crate::dialogue::DialogueState>,
    mut player: ResMut<Player>,
    mut cameras: Query<(&mut Transform, &FlyCamera)>,
    walkers: Query<&crate::ai::Walker>,
) {
    if !player.walking || !player.ready {
        return;
    }
    collision.0.set_people(people(&walkers, &state.0));
    let locked = state.0.controls_off[world::scripting::controls::MOVEMENT]
        || state.0.dead.contains(&world::dialogue::PLAYER_REF);
    let Ok((mut transform, camera)) = cameras.single_mut() else {
        return;
    };
    // Home: back to where the place started the player (R reloads).
    if keys.just_pressed(KeyCode::Home) {
        let start = player.start;
        player.character = Character::new(start);
    }
    // Forward and right on the ground, in game space: yaw 0 looks north.
    let forward = [-camera.yaw.sin(), camera.yaw.cos()];
    let right = [camera.yaw.cos(), camera.yaw.sin()];
    let mut wish = [0.0f32; 2];
    for (key, dir, sign) in [
        (KeyCode::KeyW, forward, 1.0),
        (KeyCode::KeyS, forward, -1.0),
        (KeyCode::KeyD, right, 1.0),
        (KeyCode::KeyA, right, -1.0),
    ] {
        if keys.pressed(key) && !locked {
            wish[0] += dir[0] * sign;
            wish[1] += dir[1] * sign;
        }
    }
    let len = (wish[0] * wish[0] + wish[1] * wish[1]).sqrt();
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let sneak = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::KeyC);
    // Running is the game's default; Shift walks. Crippled legs and the
    // perks' "Modify Run Speed" (Travel Light) scale the whole speed.
    let mut speed = MovementSettings::BASE_SPEED
        * world::body_parts::leg_speed_mult(&game.0.order, &state.0, world::dialogue::PLAYER_REF)
        * world::combat::movement_speed_mult(&game.0.order, &state.0, world::dialogue::PLAYER_REF);
    if !shift {
        speed *= MovementSettings::RUN_MULT;
    }
    if sneak {
        speed *= MovementSettings::SNEAK_MULT;
    }
    let velocity = if len > 0.0 {
        [wish[0] / len * speed, wish[1] / len * speed]
    } else {
        [0.0, 0.0]
    };
    // How the player moves, for who notices them (`world::detection`).
    state.0.player_moving = len > 0.0;
    state.0.player_running = len > 0.0 && !shift && !sneak;
    state.0.player_sneaking = sneak;
    // The same as the game's movement flags, for `IsMoving`, `IsRunning`
    // and `IsSneaking` (`world::more_functions::movement`).
    {
        use world::more_functions::movement as m;
        let mut flags = 0;
        for (key, flag) in [
            (KeyCode::KeyW, m::FORWARD),
            (KeyCode::KeyS, m::BACK),
            (KeyCode::KeyA, m::LEFT),
            (KeyCode::KeyD, m::RIGHT),
        ] {
            if keys.pressed(key) && !locked {
                flags |= flag;
            }
        }
        if state.0.player_running {
            flags |= m::RUNNING;
        }
        if sneak {
            flags |= m::SNEAKING;
        }
        world::more_functions::report(
            &mut state.0,
            world::dialogue::PLAYER_REF,
            world::more_functions::Seen {
                movement: flags,
                ..Default::default()
            },
        );
    }
    let shape = CharacterShape::PLAYER;
    let jump = (keys.just_pressed(KeyCode::Space) && !locked)
        .then(|| (2.0 * shape.gravity * MovementSettings::JUMP_HEIGHT).sqrt());
    let dt = time.delta_secs();
    player
        .character
        .update_with_jump(&collision.0, &shape, velocity, jump, dt);
    if let Some(fell) = player.character.fell.take() {
        if !std::mem::take(&mut player.from_camera) {
            let hurt = world::combat::land(
                &game.0.order,
                &mut state.0,
                world::dialogue::PLAYER_REF,
                fell,
            );
            if hurt > 0.0 {
                println!("Fell {fell:.0} units: {hurt:.0} damage.");
            }
        }
    }
    let [x, y, z] = player.character.feet;
    transform.translation = Vec3::from(space::point([x, y, z + EYE_HEIGHT]));
}

/// The load door the view is on, within reach and not behind a wall.
pub(crate) fn door_in_view<'a>(
    doors: &'a [DoorData],
    collision: &Collider,
    eye: [f32; 3],
    dir: [f32; 3],
) -> Option<&'a DoorData> {
    let mut best: Option<(f32, &DoorData)> = None;
    for door in doors {
        // Slightly bigger than the model, so its frame counts.
        let lo = door.lo.map(|c| c - 4.0);
        let hi = door.hi.map(|c| c + 4.0);
        if let Some(t) = ray_box(eye, dir, lo, hi) {
            if t <= ACTIVATE_REACH && best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, door));
            }
        }
    }
    let (t, door) = best?;
    // A wall nearer than the door hides it (the door's own collision is
    // about as near as its box).
    if let Some((wall, _)) = collision.raycast(eye, dir, t) {
        if wall < t - 24.0 {
            return None;
        }
    }
    Some(door)
}

/// A door that opens where it stands (not a load door) under the crosshair
/// within reach. Its leaf's collision belongs to it
/// (`preview::cell::CellScene::collider`) and swings with it
/// (`doors::update_doors`), so the ray meets the leaf wherever it is; a
/// wall nearer than the door hides it.
pub(crate) fn opening_door_in_view(
    collision: &Collider,
    eye: [f32; 3],
    dir: [f32; 3],
) -> Option<esm::FormId> {
    let (_, t) = collision.raycast_including_hidden(eye, dir, ACTIVATE_REACH)?;
    let owner = collision.owner(t);
    (owner != 0).then_some(esm::FormId(owner))
}

/// Where a ray enters an axis-aligned box, if it does.
fn ray_box(origin: [f32; 3], dir: [f32; 3], lo: [f32; 3], hi: [f32; 3]) -> Option<f32> {
    let mut near = 0.0f32;
    let mut far = f32::INFINITY;
    for k in 0..3 {
        if dir[k].abs() < 1e-9 {
            if origin[k] < lo[k] || origin[k] > hi[k] {
                return None;
            }
            continue;
        }
        let a = (lo[k] - origin[k]) / dir[k];
        let b = (hi[k] - origin[k]) / dir[k];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    Some(near)
}

/// Load doors: the crosshair line names where the door in view leads, and
/// E goes through it, into the next interior or out to a worldspace.
#[allow(clippy::too_many_arguments)]
pub fn doors(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<GameFiles>,
    doors: Res<Doors>,
    collision: Res<CellCollision>,
    mut pending: ResMut<PendingScene>,
    mut pending_exterior: ResMut<PendingExterior>,
    cameras: Query<&Transform, With<FlyCamera>>,
    mut prompt: Query<&mut Text, With<Prompt>>,
    talk_target: Res<crate::dialogue::TalkTarget>,
    conversation: Res<crate::dialogue::Conversation>,
    activatable: Res<crate::scripts::Activatable>,
    mut activate: ResMut<crate::scripts::ActivateRequest>,
    mut state: ResMut<crate::dialogue::DialogueState>,
    scripts: Res<crate::scripts::Scripts>,
    mut sounds: ResMut<crate::sounds::SoundRequests>,
    (mut lockpicking, menus): (
        ResMut<crate::lockpick::Lockpicking>,
        Res<crate::menus::Menus>,
    ),
) {
    // A lock just picked: the player uses the door or container, as the
    // game has them do after the lockpicking menu (`00573170`): E again.
    let again = lockpicking.again.take();
    // Someone to talk to (or talking) takes the prompt and E.
    if talk_target.0.is_some() || conversation.0.is_some() {
        return;
    }
    let Ok(transform) = cameras.single() else {
        return;
    };
    let eye = game_point(transform.translation);
    let dir = game_direction(transform.forward().as_vec3());
    let door = door_in_view(&doors.0, &collision.0, eye, dir);
    // A door that swings open where it stands, when no load door is in view.
    let swing = door
        .is_none()
        .then(|| opening_door_in_view(&collision.0, eye, dir))
        .flatten();
    let line = match (door, swing, &activatable.0) {
        // Nothing while the lockpicking menu or one of the game's menus is up
        // (the roll-over is the HUD's, which their masks hide: ui::hud::parts_for_menu).
        _ if lockpicking.is_open() || menus.game_open => String::new(),
        (Some(d), _, _) => format!("E) Open door to {}", d.cell_label),
        (None, Some(d), _) => crate::doors::prompt(&game.0.order, &state.0, d).to_string(),
        // A scripted object (a machine, a switch) when no door is in view.
        (None, None, Some((_, name))) => format!("E) {name}"),
        (None, None, None) => String::new(),
    };
    for mut text in &mut prompt {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
    if let Some(r) = again {
        // A container (or a door no longer in view) is used as E on it.
        let in_view = door.is_some_and(|d| d.reference == r.0) || swing == Some(r);
        if !in_view {
            activate.0 = Some(r);
            return;
        }
    }
    // The game's menus have E while they're open (game_menus).
    let pressed = !menus.game_open && (keys.just_pressed(KeyCode::KeyE) || again.is_some());
    let Some(door) = door else {
        if !pressed {
            return;
        }
        if let Some(reference) = swing {
            // Opening or closing runs the door's script, as the load
            // doors' do; then the game's door rules (`world::doors`): the
            // swing, its sound, and nothing while it's still swinging.
            if crate::scripts::door_opens(
                &game.0.order,
                &scripts.0,
                &mut state.0,
                reference,
                &mut lockpicking.request,
            ) {
                let player = world::dialogue::PLAYER_REF;
                match crate::doors::activate(
                    &game.0.order,
                    &mut state.0,
                    &mut sounds,
                    reference,
                    Some(player),
                ) {
                    world::doors::Activated::Opening => println!("The door opens."),
                    world::doors::Activated::Closing => println!("The door closes."),
                    world::doors::Activated::Busy => {}
                }
            }
        } else if let Some((reference, _)) = &activatable.0 {
            activate.0 = Some(*reference);
        }
        return;
    };
    if !pressed {
        return;
    }
    let reference = esm::FormId(door.reference);
    if !crate::scripts::door_opens(
        &game.0.order,
        &scripts.0,
        &mut state.0,
        reference,
        &mut lockpicking.request,
    ) {
        return;
    }
    // The door's opening sound (`SNAM` on its base).
    if let Some(base) = world::scripting::base_of(&game.0.order, reference) {
        if let (Some(open), _) = world::sound::door_sounds(&game.0.order, base) {
            sounds.0.push(open);
        }
    }
    let Some(cell) = door.cell else {
        return;
    };
    // Going through a door moves the player: fast travel a script turned
    // off comes back (unless it asked to keep it off).
    world::script_functions::player_moved(&mut state.0);
    if !door.interior && door.world.is_some() {
        match door_start(&game.0, door) {
            Ok(start) => pending_exterior.0 = Some(start),
            Err(e) => println!("Couldn't go out to {}: {e}", door.cell_label),
        }
        return;
    }
    println!("Loading {} ...", door.cell_label);
    let started = std::time::Instant::now();
    match game.0.load_cell_now(esm::FormId(cell), &state.0.disabled) {
        Ok(mut scene) => {
            let [x, y, z] = door.arrive;
            scene.start = cellview::Start {
                eye: [x, y, z + EYE_HEIGHT],
                heading: door.arrive_heading,
                via: "the door",
            };
            for note in &scene.notes {
                println!("  {note}");
            }
            println!(
                "Loaded {} in {:.1} s.",
                scene.cell,
                started.elapsed().as_secs_f32()
            );
            pending.0 = Some(scene);
        }
        Err(e) => println!("Couldn't load {}: {e}", door.cell_label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rays_enter_boxes_in_front_only() {
        let lo = [10.0, -1.0, -1.0];
        let hi = [12.0, 1.0, 1.0];
        assert_eq!(ray_box([0.0; 3], [1.0, 0.0, 0.0], lo, hi), Some(10.0));
        assert_eq!(ray_box([0.0; 3], [-1.0, 0.0, 0.0], lo, hi), None);
        assert_eq!(ray_box([0.0, 5.0, 0.0], [1.0, 0.0, 0.0], lo, hi), None);
    }

    #[test]
    fn finds_the_door_in_view_open_or_shut() {
        // A door's leaf (owned by the door, 0x904) 100 units north, and a
        // wall (nobody's) 50 units east.
        let quad = |c: &mut Collider, v: [[f32; 3]; 4], owner: u32| {
            c.add_solid(&v, &[[0, 1, 2], [0, 2, 3]], 0.0, owner)
        };
        let mut c = Collider::new();
        quad(
            &mut c,
            [
                [-40.0, 100.0, 0.0],
                [40.0, 100.0, 0.0],
                [40.0, 100.0, 200.0],
                [-40.0, 100.0, 200.0],
            ],
            0x904,
        );
        quad(
            &mut c,
            [
                [50.0, -40.0, 0.0],
                [50.0, 40.0, 0.0],
                [50.0, 40.0, 200.0],
                [50.0, -40.0, 200.0],
            ],
            0,
        );
        let eye = [0.0, 0.0, 120.0];
        let north = [0.0, 1.0, 0.0];
        assert_eq!(
            opening_door_in_view(&c, eye, north),
            Some(esm::FormId(0x904))
        );
        // Swung aside (a quarter turn about its left edge), it's found
        // where it now is.
        let turn = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        c.move_owner(0x904, &turn, [-40.0 + 100.0, 100.0 + 40.0, 0.0]);
        assert_eq!(opening_door_in_view(&c, eye, north), None);
        assert_eq!(
            opening_door_in_view(&c, [-60.0, 100.0, 120.0], [1.0, 0.0, 0.0]),
            Some(esm::FormId(0x904))
        );
        // A wall isn't a door; and out of reach, nothing.
        assert_eq!(opening_door_in_view(&c, eye, [1.0, 0.0, 0.0]), None);
        assert_eq!(opening_door_in_view(&c, [0.0, -100.0, 120.0], north), None);
    }

    #[test]
    fn game_and_bevy_space_round_trip() {
        let p = [100.0, -200.0, 300.0];
        let back = game_point(Vec3::from(space::point(p)));
        assert!(back.iter().zip(p).all(|(a, b)| (a - b).abs() < 1e-3));
        let d = [0.0, 1.0, 0.0];
        assert_eq!(game_direction(Vec3::from(space::direction(d))), d);
    }
}

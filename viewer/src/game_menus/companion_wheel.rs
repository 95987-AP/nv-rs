//! The companion wheel (`ui::menus::companion_wheel`): the player using a
//! teammate (`world::living::pickpocket::Use::Wheel`,
//! `menus::Menu::CompanionWheel`). Its orders are `world::companions`':
//! a topic said and its result scripts run at once, the Stimpak, a script
//! variable; talking starts the conversation as `StartConversation`
//! would; their things open through `FollowersTrade`'s own
//! `OpenTeammateContainer`. The line said shows on the wheel's subtitle
//! while its voice plays (or for its length × `fNoticeTextTimePerCharacter`
//! without one).

use std::sync::Arc;

use bevy::audio::{AudioPlayer, AudioSource};
use bevy::prelude::*;
use cellview::Game;
use esm::{FormId, LoadOrder};
use ui::menus::companion_wheel::{CompanionWheelMenu, Facts, Request, FILE};
use world::dialogue::{Info, PLAYER_REF};
use world::scripting::{GameState, ScriptCache};

use super::{OpenMenu, Screen};
use crate::menus::Menu;

/// The wheel on screen and the companion.
pub struct WheelScreen {
    pub menu: CompanionWheelMenu,
    pub who: FormId,
}

/// The companion's voice for the wheel's lines: what to play, and the one
/// playing (its subtitle stays until it ends).
#[derive(Resource, Default)]
pub struct WheelVoices {
    pending: Vec<(String, FormId)>,
    playing: Option<Entity>,
    /// The voice playing has ended.
    ended: bool,
}

/// Whether this module shows a request.
pub fn takes(menu: &Menu) -> bool {
    matches!(menu, Menu::CompanionWheel(..))
}

/// What the context line shows (`00755dc0`).
fn facts(order: &LoadOrder, state: &GameState, who: FormId) -> Facts {
    let health = world::combat::health(order, state, who).unwrap_or(0.0);
    let full = world::combat::max_health(order, state, who).unwrap_or(0.0);
    let stimpaks =
        world::companions::stimpak(order).map_or(0, |s| state.item_count(order, PLAYER_REF, s));
    let (carried, most) = world::items::carry(order, state, who);
    // Their weapon (vtable +0x3BC(6): what's worn in the weapon slot).
    let weapon = world::combat::weapon_in_hand(order, state, who)
        .map(|w| w.name)
        .filter(|n| !n.is_empty());
    Facts {
        health: health as i32,
        max_health: full as i32,
        stimpaks,
        carried: carried as i32,
        carry_limit: most as i32,
        weapon,
    }
}

/// Opens the wheel for a request.
pub fn open(
    screen: &mut Screen,
    game: &Game,
    scripts: &ScriptCache,
    state: &mut GameState,
    request: Menu,
) {
    let Menu::CompanionWheel(who) = request else {
        return;
    };
    let order = &game.order;
    let mut menu = CompanionWheelMenu::new(0);
    let tile = match screen.load(game, FILE, &mut menu) {
        Ok(t) => t,
        Err(e) => {
            println!("The companion wheel can't be shown: {e}");
            return;
        }
    };
    menu.menu = tile;
    state.stock(order, PLAYER_REF);
    state.stock(order, who);
    let switches = world::companions::switches(order, scripts, state, who);
    if !menu.open(&mut screen.ui, switches, facts(order, state, who)) {
        println!(
            "MENUS: Companion Wheel Menu Creation Failed... Are your menu and art resources up to date?"
        );
        screen.ui.detach(tile);
        return;
    }
    println!("Companion wheel: {who} ({switches:?}).");
    screen
        .open
        .push(OpenMenu::CompanionWheel(Box::new(WheelScreen {
            menu,
            who,
        })));
}

/// A line said on the wheel: its subtitle, its voice.
fn said(
    menu: &mut CompanionWheelMenu,
    ui: &mut ui::Ui,
    order: &LoadOrder,
    voices: &mut WheelVoices,
    who: FormId,
    info: &Info,
    now_ms: f64,
) {
    let Some(response) = info.responses.first() else {
        return;
    };
    println!("Companion wheel: {who} says \"{}\".", response.text);
    let voice = world::scripting::base_of(order, who)
        .and_then(|b| world::dialogue::Speaker::load(order, who, b))
        .and_then(|s| s.voice)
        .and_then(|v| world::dialogue::voice_path(order, info, response, v));
    let until = match voice {
        Some(path) => {
            voices.pending.push((path, who));
            f64::INFINITY
        }
        None => {
            let per = world::scripting::game_setting(order, "fNoticeTextTimePerCharacter")
                .unwrap_or(ui::menus::companion_wheel::TIME_PER_CHARACTER);
            now_ms + CompanionWheelMenu::line_time(&response.text, per)
        }
    };
    menu.say(ui, &response.text, until);
}

/// Each frame: the subtitle emptied once its time is up or its voice has
/// ended.
pub fn update(screen: &mut Screen, voices: &mut WheelVoices, now_ms: f64) {
    let ended = std::mem::take(&mut voices.ended);
    for m in screen.open.iter_mut() {
        if let OpenMenu::CompanionWheel(w) = m {
            if ended && w.menu.subtitle_until == Some(f64::INFINITY) {
                w.menu.subtitle_until = Some(now_ms);
            }
            w.menu.update(&mut screen.ui, now_ms);
        }
    }
}

/// What the wheel asked for, carried out. Returns sounds to play.
pub fn after(
    screen: &mut Screen,
    game: &Game,
    scripts: &ScriptCache,
    state: &mut GameState,
    voices: &mut WheelVoices,
    now_ms: f64,
) -> Vec<FormId> {
    let order = &game.order;
    let mut sounds = Vec::new();
    for m in screen.open.iter_mut() {
        let OpenMenu::CompanionWheel(w) = m else {
            continue;
        };
        for name in w.menu.sounds.drain(..) {
            if let Some(id) = order.form_by_editor_id(&name) {
                sounds.push(id);
            }
        }
        let who = w.who;
        let requests: Vec<Request> = std::mem::take(&mut w.menu.requests);
        let mut changed = false;
        for request in requests {
            match request {
                Request::Topic(topic) => {
                    match world::companions::say_topic(order, scripts, state, who, topic, true) {
                        Some(info) => {
                            said(
                                &mut w.menu,
                                &mut screen.ui,
                                order,
                                voices,
                                who,
                                &info,
                                now_ms,
                            );
                        }
                        None => println!("Companion wheel: {who} has no line for {topic}."),
                    }
                    changed = true;
                }
                Request::Variable(name, value) => {
                    world::companions::set_variable(order, scripts, state, who, name, value);
                }
                Request::Stimpak => {
                    if world::companions::heal_with_stimpak(order, state, who) {
                        println!("Companion wheel: a Stimpak on {who}.");
                        // `Regenerating`, said only (`008d5910(13)`).
                        if let Some(info) = world::companions::say_topic(
                            order,
                            scripts,
                            state,
                            who,
                            "Regenerating",
                            false,
                        ) {
                            said(
                                &mut w.menu,
                                &mut screen.ui,
                                order,
                                voices,
                                who,
                                &info,
                                now_ms,
                            );
                        }
                        changed = true;
                    }
                }
                Request::BackUp => {
                    // `008a7760`: a default package (0x27) moving them away
                    // from the player; packages like it aren't here.
                    println!("Companion wheel: {who} would step back (not carried out here).");
                }
                Request::Talk => {
                    // As the player using them would (`00756980`).
                    state.events.push(world::scripting::Event::Talk {
                        speaker: who,
                        to: PLAYER_REF,
                        topic: None,
                        conversation: true,
                    });
                }
                Request::Close => {}
            }
        }
        if changed && !w.menu.closed {
            let f = facts(order, state, who);
            w.menu.set_facts(&mut screen.ui, f);
        }
    }
    sounds
}

/// Plays the wheel's voices, and notices when one has ended.
pub fn play_voices(
    mut commands: Commands,
    mut audio: ResMut<Assets<AudioSource>>,
    game: Res<crate::GameFiles>,
    mut voices: ResMut<WheelVoices>,
    players: Query<(), With<AudioPlayer<AudioSource>>>,
) {
    if let Some(e) = voices.playing {
        if players.get(e).is_err() {
            voices.playing = None;
            voices.ended = true;
        }
    }
    for (path, who) in std::mem::take(&mut voices.pending) {
        let Some(bytes) = game.0.assets.read(&path).ok().flatten() else {
            println!("Companion wheel: no voice file {path}.");
            voices.ended = true;
            continue;
        };
        if let Some(e) = voices.playing.take() {
            commands.entity(e).despawn();
        }
        let source = audio.add(AudioSource {
            bytes: Arc::from(bytes.into_boxed_slice()),
        });
        let (settings, voice) = crate::faces::voice_playback(&game.0, &path, who);
        voices.playing = Some(
            commands
                .spawn((AudioPlayer::new(source), settings, voice))
                .id(),
        );
    }
}

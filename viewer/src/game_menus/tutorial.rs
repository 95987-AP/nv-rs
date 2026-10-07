//! The once-only tutorial messages (`world::tutorial` the manager,
//! `ui::menus::tutorial` the menu): every frame the manager's update runs
//! over the menus on screen (`007182e0`, from the interface manager's
//! update `0070c4a0`) and a message it picks opens in the tutorial menu
//! and is marked shown; Caravan and the crafting menu open theirs directly
//! ([`show`]).
//!
//! The manager is held while the start menu is up as the pause menu
//! ([`held`]); the name entry and "outside the game" (the main menu) don't
//! arise here. A menu's fade-in (`Menu` +0x24, `007024e0`) isn't kept:
//! menus here are shown at once, so the top menu counts as shown.

use cellview::Game;
use esm::FormId;
use ui::menus::tutorial::{Message, TutorialMenu, FILE};
use world::scripting::GameState;

use super::{OpenMenu, Screen};

/// The top menu, for the manager (`00702450`, `007024e0`).
struct Top(Option<i32>);

impl world::tutorial::Screen for Top {
    fn is_menu_open(&self, class: i32) -> bool {
        self.0 == Some(class)
    }
    fn top_menu_shown(&self) -> bool {
        self.0.is_some()
    }
}

/// A message record's title and text (`FULL`, `DESC`).
fn message(game: &Game, form: FormId) -> Option<Message> {
    let record = game.order.get(form)?.record().ok()?;
    let text = record
        .get(esm::FourCC::new(b"DESC"))
        .map(|s| s.zstring())
        .unwrap_or_default();
    Some(Message {
        title: record.full_name().unwrap_or_default(),
        text,
    })
}

/// Opens the tutorial menu with a message (`TutorialMenu::Create`,
/// `007e8890`): one already open is closed first. False when it can't be
/// (no record, no menu file), as the game's `Create` answers.
pub fn show(screen: &mut Screen, game: &Game, id: u8, form: FormId) -> bool {
    let Some(m) = message(game, form) else {
        return false;
    };
    for open in screen.open.iter_mut() {
        if let OpenMenu::Tutorial(t) = open {
            t.close(&mut screen.ui);
        }
    }
    let mut menu = TutorialMenu::new(0, id);
    match screen.load(game, FILE, &mut menu) {
        Ok(tile) => menu.menu = tile,
        Err(e) => {
            println!("The tutorial menu can't be opened: {e}");
            return false;
        }
    }
    menu.open(&mut screen.ui, &m);
    println!("Tutorial: {} ({form}).", m.title);
    screen.open.push(OpenMenu::Tutorial(Box::new(menu)));
    true
}

/// Shows a message a menu opens itself if it hasn't been shown (Caravan's
/// `00741060` / `00741500`, the crafting menu's `00726ff0`): the record by
/// editor ID, marked shown once the menu has opened. True when the caller
/// is to wait for it (the menu's own flag, Caravan's `+0xe78`).
pub fn show_once(
    screen: &mut Screen,
    game: &Game,
    state: &mut GameState,
    id: u8,
    name: &str,
) -> bool {
    if state.tutorials.is_shown(id) {
        return false;
    }
    let Some(form) = game.order.form_by_editor_id(name) else {
        return false;
    };
    let shown = show(screen, game, id, form);
    if shown {
        state.tutorials.mark_shown(id);
    }
    shown
}

/// Whether the manager waits (`007182e0`'s first test, `004a4040`): the
/// start menu is up with its pause flag (`+0x1a8` bit 1, the pause menu
/// over the game).
pub fn held(open: &[OpenMenu]) -> bool {
    open.iter().any(|m| match m {
        OpenMenu::Start(s) => s.menu.flags & ui::menus::start::flag::PAUSE != 0,
        _ => false,
    })
}

/// The class of the menu on top: the game's menus here, else the
/// lockpicking menu when it's up (it isn't one of them in the viewer).
pub fn top_class(screen: &mut Screen, lockpicking: bool) -> Option<i32> {
    match screen.open.last_mut() {
        Some(m) => Some(m.code().class()),
        None => lockpicking.then_some(world::tutorial::menu::LOCKPICK),
    }
}

/// The manager's update (`007182e0`): a message it picks is shown and
/// marked (or the game's error line printed).
pub fn update(
    screen: &mut Screen,
    game: &Game,
    state: &mut GameState,
    now_ms: f64,
    lockpicking: bool,
) {
    let held = held(&screen.open);
    let top = Top(top_class(screen, lockpicking));
    let Some(id) = state.tutorials.update(now_ms as u32, true, held, &top) else {
        return;
    };
    let form = world::tutorial::message_form(id);
    if show(screen, game, id, form) && state.tutorials.mark_shown(id) {
        return;
    }
    println!("Error occurred while trying to display tutorial message");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::menus::start::{flag, StartMenu};

    /// The start menu holds the manager only as the pause menu (its flag
    /// 1), as `004a4040` asks.
    #[test]
    fn the_pause_menu_holds_the_tutorials() {
        let ui = ui::Ui::new(
            ui::Screen {
                width_px: 1920,
                height_px: 1080,
                safe_x: 15.0,
                safe_y: 15.0,
            },
            ui::SystemColors::new(None, None),
            Box::new(|_| None),
        );
        let start = |flags: u32| {
            let mut menu = StartMenu::new(0, &ui);
            menu.flags = flags;
            OpenMenu::Start(Box::new(super::super::start::StartScreen::new(menu, false)))
        };
        assert!(!held(&[]));
        assert!(!held(&[start(0)]));
        assert!(held(&[start(flag::PAUSE)]));
        assert!(held(&[start(flag::PAUSE | flag::SAVE_MODE)]));
    }
}

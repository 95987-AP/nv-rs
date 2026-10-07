//! The once-only tutorial messages (`world::tutorial` the manager,
//! `ui::menus::tutorial` the menu): every frame the manager's update runs
//! over the menus on screen (`007182e0`, from the interface manager's
//! update `0070c4a0`) and a message it picks opens in the tutorial menu
//! and is marked shown; Caravan and the crafting menu open theirs directly
//! ([`show`]).
//!
//! The manager's "held" (the start menu up, the name entry) and "outside
//! the game" don't arise here: the viewer has neither the start menu nor
//! the main menu. A menu's fade-in (`Menu` +0x24, `007024e0`) isn't kept:
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
    let top = Top(top_class(screen, lockpicking));
    let Some(id) = state.tutorials.update(now_ms as u32, true, false, &top) else {
        return;
    };
    let form = world::tutorial::message_form(id);
    if show(screen, game, id, form) && state.tutorials.mark_shown(id) {
        return;
    }
    println!("Error occurred while trying to display tutorial message");
}

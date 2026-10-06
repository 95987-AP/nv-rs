//! The Pip-Boy's "How many?" when dropping (ITEMS' Drop, `00780140` case
//! 7): more than `iInventoryAskQuantityAt` of the item asks with the
//! quantity menu (`007aba00`, `ui::menus::quantity`, up to all of them)
//! and its answer is dropped (the callback `00780c50`: the player drops
//! that many, `GameState::drop_item`); Cancel (0) drops nothing.

use cellview::Game;
use esm::FormId;
use world::dialogue::PLAYER_REF;
use world::scripting::GameState;

use super::{OpenMenu, Screen};
use crate::menus::Menu;

pub fn takes(m: &Menu) -> bool {
    matches!(m, Menu::PipboyDrop { .. })
}

/// Asks how many of `item` (up to `most`).
pub fn open(screen: &mut Screen, game: &Game, request: Menu) {
    let Menu::PipboyDrop { item, most } = request else {
        return;
    };
    super::container::open_quantity(screen, game, most);
    screen.pipboy_drop = Some(item);
}

/// The answer, dropped. Only a quantity menu with no menu under it is the
/// drop's (the others ask for the container, barter and Caravan menus
/// below them). Returns sounds.
pub fn after(screen: &mut Screen, game: &Game, state: &mut GameState) -> Vec<FormId> {
    let Some(item) = screen.pipboy_drop else {
        return Vec::new();
    };
    let mut sounds = Vec::new();
    match screen.open.first_mut() {
        Some(OpenMenu::Quantity(q)) => {
            for name in q.sounds.drain(..) {
                sounds.extend(game.order.form_by_editor_id(&name));
            }
            if let Some(n) = q.answer.take() {
                screen.pipboy_drop = None;
                if n > 0 && state.drop_item(&game.order, PLAYER_REF, item, n).is_some() {
                    println!("Dropped {n} of {item}.");
                }
            }
        }
        _ => screen.pipboy_drop = None,
    }
    sounds
}

//! Containers and bodies (`ui::menus::container`, mode 1): opened when the
//! player uses a container or a dead person (`menus::Menu::Container`);
//! the items moved are moved in the world's state (`GameState::move_item`)
//! with the game's sounds, the theft rules (`world::crime`) and the
//! container's `OnClose` block when it closes. "How many?" is the quantity
//! menu (`ui::menus::quantity`) over it.

use cellview::Game;
use esm::FormId;
use ui::menus::container::{ContainerMenu, Item, Request, Side, FILE};
use ui::menus::quantity::{self, QuantityMenu};
use world::dialogue::PLAYER_REF;
use world::scripting::{Facts, GameState, Runner};

use super::{OpenMenu, Screen};
use crate::menus::Menu;

/// Carry weight's actor value number.
const CARRY_WEIGHT: u16 = 13;

/// The container menu on screen and the reference it shows.
pub struct ContainerScreen {
    pub menu: ContainerMenu,
    pub reference: FormId,
}

/// Whether this module shows a request.
pub fn takes(menu: &Menu) -> bool {
    matches!(menu, Menu::Container(..))
}

/// A holder's things as the menu reads them (`world::items`).
fn items(order: &esm::LoadOrder, state: &GameState, holder: FormId) -> Vec<Item> {
    world::items::inventory_lines(order, state, holder)
        .into_iter()
        .map(|l| Item {
            form: l.item.0,
            name: l.name,
            count: l.count,
            form_type: l.form_type,
            equipped: l.equipped,
            quest_item: l.quest_item,
            playable: l.playable,
            regenerating_ammo: l.regenerating_ammo,
            // Weapon mods aren't kept in the world's state.
            modded: false,
            weightless: l.weightless,
            is_caps: l.item.0 == 0xF,
            icon: l.icon,
        })
        .collect()
}

/// The weight line's numbers: what the player carries and can carry.
fn weights(order: &esm::LoadOrder, state: &GameState) -> (f32, f32) {
    let carried = state.inventory_weight(order, PLAYER_REF);
    let most = Facts {
        order,
        state,
        speaker: None,
    }
    .current_actor_value(PLAYER_REF, CARRY_WEIGHT)
    .unwrap_or(0.0) as f32;
    (carried, most)
}

/// The base form a reference stands for.
fn base(order: &esm::LoadOrder, reference: FormId) -> Option<FormId> {
    world::scripting::base_of(order, reference)
}

/// Opens the container menu for a request.
pub fn open(screen: &mut Screen, game: &Game, state: &mut GameState, request: Menu) -> Vec<FormId> {
    let Menu::Container(reference, name) = request else {
        return Vec::new();
    };
    let order = &game.order;
    let mut menu = ContainerMenu::new(0);
    let tile = match screen.load(game, FILE, &mut menu) {
        Ok(t) => t,
        Err(e) => {
            println!("The container menu can't be shown: {e}");
            return Vec::new();
        }
    };
    menu.menu = tile;
    menu.name = name;
    // `0075e380`: titles in upper case when the game's language is
    // English (`[General] sLanguage`, default "ENGLISH").
    menu.english = game
        .settings
        .get("General", "sLanguage")
        .is_none_or(|l| l.trim() == "ENGLISH");
    menu.ask_quantity_at = world::scripting::game_setting(order, "iInventoryAskQuantityAt")
        .map_or(ui::menus::container::ASK_QUANTITY_AT, |v| v as i32);
    state.stock(order, reference);
    menu.weights = weights(order, state);
    let player = items(order, state, PLAYER_REF);
    let container = items(order, state, reference);
    if !menu.open(&mut screen.ui, player, container) {
        println!("MENUS: Container Menu Creation Failed.");
        screen.ui.detach(tile);
        return Vec::new();
    }
    println!("Container menu: {}.", menu.name);
    let mut sounds = Vec::new();
    if let Some(s) =
        base(order, reference).and_then(|b| world::sound::container_sound(order, b, true))
    {
        sounds.push(s);
    }
    screen
        .open
        .push(OpenMenu::Container(Box::new(ContainerScreen {
            menu,
            reference,
        })));
    sounds
}

/// What the menu asked for, carried out: items moved (with the item's
/// sound and the theft rules), "how many?" asked, the menu closed (its
/// sound and the container's `OnClose`). Returns sounds to play.
pub fn after(
    screen: &mut Screen,
    game: &Game,
    scripts: &world::scripting::ScriptCache,
    state: &mut GameState,
) -> Vec<FormId> {
    let order = &game.order;
    let mut sounds = Vec::new();
    // "How many?" answered: the container below takes the number.
    let mut answer = None;
    let mut below = false;
    for m in screen.open.iter_mut() {
        match m {
            OpenMenu::Container(_) => below = true,
            OpenMenu::Barter(_) => below = false,
            OpenMenu::Quantity(q) => {
                if below {
                    if let Some(n) = q.answer.take() {
                        answer = Some(n);
                    }
                }
                for name in q.sounds.drain(..) {
                    if let Some(id) = order.form_by_editor_id(&name) {
                        sounds.push(id);
                    }
                }
            }
            _ => {}
        }
    }
    let mut ask: Option<i32> = None;
    for m in screen.open.iter_mut() {
        let OpenMenu::Container(c) = m else {
            continue;
        };
        if let Some(n) = answer {
            c.menu.quantity_chosen(&mut screen.ui, n);
        }
        for name in c.menu.sounds.drain(..) {
            if let Some(id) = order.form_by_editor_id(&name) {
                sounds.push(id);
            }
        }
        let reference = c.reference;
        let requests: Vec<Request> = std::mem::take(&mut c.menu.requests);
        for request in requests {
            match request {
                Request::Move { from, form, count } => {
                    let item = FormId(form);
                    let (giver, taker) = match from {
                        Side::Player => (PLAYER_REF, reference),
                        Side::Container => (reference, PLAYER_REF),
                    };
                    let equipped = state.is_equipped(giver, item);
                    let moved = state.move_item(order, giver, taker, item, count);
                    if moved == 0 {
                        continue;
                    }
                    if equipped
                        && !state
                            .inventory(order, giver)
                            .iter()
                            .any(|(i, _)| *i == item)
                    {
                        state.unequip(giver, item);
                    }
                    // Taking from someone else's container is stealing
                    // (`world::crime`).
                    if from == Side::Container {
                        let owner = world::crime::owner_of(order, state, reference)
                            .filter(|&o| !world::crime::may_take(order, state, Some(o)));
                        if let Some(o) = owner {
                            world::crime::steal(order, state, reference, o);
                        }
                    }
                    if !c.menu.closed {
                        // `0075dc80`: the item's pick-up sound.
                        if let Some(s) = world::sound::item_sound(order, item, true) {
                            sounds.push(s);
                        }
                        c.menu.weights = weights(order, state);
                        let player = items(order, state, PLAYER_REF);
                        let container = items(order, state, reference);
                        c.menu.refresh_item(&mut screen.ui, form, player, container);
                    }
                }
                Request::AskQuantity { most, .. } => ask = Some(most),
                Request::Close => {
                    if let Some(s) = base(order, reference)
                        .and_then(|b| world::sound::container_sound(order, b, false))
                    {
                        sounds.push(s);
                    }
                    Runner::new(order, scripts, state).run_event(reference, "onclose", PLAYER_REF);
                }
            }
        }
    }
    if let Some(most) = ask {
        open_quantity(screen, game, most);
    }
    sounds
}

/// "How many?" over the container (`007aba00`: up to `most`, starting at
/// all of it; the pop-up background since a menu is open).
pub fn open_quantity(screen: &mut Screen, game: &Game, most: i32) {
    let mut q = QuantityMenu::new(0);
    match screen.load(game, quantity::FILE, &mut q) {
        Ok(tile) => {
            q.menu = tile;
            let popup = game
                .settings
                .float("Interface", "fPopUpBackgroundOpacity")
                .unwrap_or(ui::game::POPUP_BACKGROUND_OPACITY);
            q.open(&mut screen.ui, most, i32::MAX, Some(popup));
            screen.open.push(OpenMenu::Quantity(q));
        }
        Err(e) => println!("The quantity menu can't be shown: {e}"),
    }
}

/// Every frame: the quantity menu's ticks.
pub fn update(screen: &mut Screen) {
    for m in screen.open.iter_mut() {
        if let OpenMenu::Quantity(q) = m {
            q.update(&mut screen.ui);
        }
    }
}

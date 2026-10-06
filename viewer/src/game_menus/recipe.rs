//! Crafting (`ui::menus::recipe`): a script's `ShowRecipeMenu`
//! (`menus::Menu::Recipes`, from a workbench, a campfire, a reloading bench
//! or a Sierra Madre vending machine). What the player can make and what
//! making does are `world::crafting`'s.

use cellview::Game;
use esm::{FormId, FourCC, LoadOrder};
use ui::menus::recipe::{Filter, Ingredient, RecipeLine, RecipeMenu, Request, FILE};
use world::dialogue::PLAYER_REF;
use world::scripting::GameState;

use super::{OpenMenu, Screen};
use crate::menus::Menu;

/// The recipe menu on screen and the category it lists (`None`: every one).
pub struct RecipeScreen {
    pub menu: RecipeMenu,
    pub category: Option<FormId>,
}

/// Whether this module shows a request.
pub fn takes(menu: &Menu) -> bool {
    matches!(menu, Menu::Recipes(..))
}

fn item_name(order: &LoadOrder, item: FormId) -> String {
    order
        .get(item)
        .and_then(|r| r.record().ok())
        .and_then(|r| r.full_name())
        .unwrap_or_default()
}

fn icon_of(order: &LoadOrder, item: FormId) -> String {
    order
        .get(item)
        .and_then(|r| r.record().ok())
        .and_then(|r| r.get(FourCC::new(b"ICON")).map(|s| s.zstring()))
        .unwrap_or_default()
}

/// What the player is offered now, as the menu's lines, and the
/// sub-categories they fall in.
fn lines(
    order: &LoadOrder,
    state: &GameState,
    category: Option<FormId>,
) -> (Vec<RecipeLine>, Vec<Filter>) {
    let offers = world::crafting::offers(order, state, category);
    let filters = world::crafting::sub_categories(&offers)
        .into_iter()
        .map(|f| Filter {
            form: f.0,
            name: world::crafting::category_name(order, f),
        })
        .collect();
    let lines = offers
        .into_iter()
        .map(|o| {
            let r = o.recipe;
            RecipeLine {
                form: r.id.0,
                sub: r.sub_category.0,
                makeable: o.can_make,
                skill: r.skill.map_or_else(String::new, |av| {
                    format!(
                        "{} {}",
                        world::chargen::actor_value_name(order, av),
                        r.level
                    )
                }),
                ingredients: r
                    .ingredients
                    .iter()
                    .map(|&(item, need)| Ingredient {
                        name: item_name(order, item),
                        have: state.item_count(order, PLAYER_REF, item),
                        need,
                    })
                    .collect(),
                icon: r
                    .outputs
                    .first()
                    .map(|&(item, _)| icon_of(order, item))
                    .unwrap_or_default(),
                name: r.name,
            }
        })
        .collect();
    (lines, filters)
}

/// Opens the recipe menu for a request.
pub fn open(screen: &mut Screen, game: &Game, state: &GameState, request: Menu) -> Vec<FormId> {
    let Menu::Recipes(category) = request else {
        return Vec::new();
    };
    let order = &game.order;
    let mut menu = RecipeMenu::new(0);
    let tile = match screen.load(game, FILE, &mut menu) {
        Ok(t) => t,
        Err(e) => {
            println!("The recipe menu can't be shown: {e}");
            return Vec::new();
        }
    };
    menu.menu = tile;
    let made_at = category.map_or_else(String::new, |c| world::crafting::category_name(order, c));
    let (list, filters) = lines(order, state, category);
    let count = list.len();
    if !menu.open(&mut screen.ui, &made_at, list, filters) {
        println!("MENUS: Recipe Menu Creation Failed.");
        screen.ui.detach(tile);
        return Vec::new();
    }
    println!("Recipe menu: {made_at}, {count} recipes.");
    let sounds = menu
        .sounds
        .drain(..)
        .filter_map(|s| order.form_by_editor_id(&s))
        .collect();
    screen
        .open
        .push(OpenMenu::Recipe(Box::new(RecipeScreen { menu, category })));
    sounds
}

/// What the menu asked for, carried out: a recipe made. Returns sounds to
/// play.
pub fn after(screen: &mut Screen, game: &Game, state: &mut GameState) -> Vec<FormId> {
    let order = &game.order;
    let mut sounds = Vec::new();
    for m in screen.open.iter_mut() {
        let OpenMenu::Recipe(r) = m else {
            continue;
        };
        for name in r.menu.sounds.drain(..) {
            if let Some(id) = order.form_by_editor_id(&name) {
                sounds.push(id);
            }
        }
        let requests: Vec<Request> = std::mem::take(&mut r.menu.requests);
        for request in requests {
            let Request::Make(form) = request;
            match world::crafting::make(order, state, FormId(form)) {
                Ok(made) => {
                    let names: Vec<String> = made
                        .iter()
                        .map(|&(item, n)| format!("{n} {}", item_name(order, item)))
                        .collect();
                    println!("Crafted {}.", names.join(", "));
                    let (list, _) = lines(order, state, r.category);
                    r.menu.refreshed(&mut screen.ui, list);
                }
                Err(why) => println!("Couldn't craft: {why:?}."),
            }
        }
    }
    sounds
}

//! Crafting on a generated world (`testdata::crafting`).

use esm::{ActivePlugins, FormId, LoadOrder};
use testdata::crafting::ids::*;
use world::crafting::{self, Component, Recipe, ITEMS_CRAFTED, MOST};
use world::dialogue::PLAYER_REF;
use world::scripting::GameState;

fn order(tag: &str) -> (testdata::TempData, LoadOrder) {
    let data = testdata::crafting::crafting(tag);
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    (data, order)
}

fn give(order: &LoadOrder, state: &mut GameState, item: u32, n: i32) {
    state.stock(order, PLAYER_REF);
    *state.items.entry((PLAYER_REF, FormId(item))).or_insert(0) += n;
}

fn count(order: &LoadOrder, state: &GameState, item: u32) -> i32 {
    state.item_count(order, PLAYER_REF, FormId(item))
}

#[test]
fn recipes_read_their_data_and_components() {
    let (_data, order) = order("crafting-load");
    let r = Recipe::load(&order, FormId(R_STIMPAK)).unwrap();
    assert_eq!(r.name, "Stimpak");
    assert_eq!(r.skill, Some(37));
    assert_eq!(r.skill_level, 50);
    assert_eq!(r.category, Some(FormId(WORKBENCH)));
    assert_eq!(r.subcategory, Some(FormId(AID)));
    assert_eq!(
        r.ingredients,
        vec![
            Component {
                item: FormId(FLOWER),
                count: 2
            },
            Component {
                item: FormId(SCRAP),
                count: 1
            }
        ]
    );
    assert_eq!(
        r.outputs,
        vec![Component {
            item: FormId(STIMPAK),
            count: 1
        }]
    );
    // No skill: -1.
    assert_eq!(Recipe::load(&order, FormId(R_ROUNDS)).unwrap().skill, None);
    // An ingredient without RCQY is never added (TESRecipe::Load).
    let bloom = Recipe::load(&order, FormId(R_COUNTLESS)).unwrap();
    assert_eq!(
        bloom.ingredients,
        vec![Component {
            item: FormId(SCRAP),
            count: 1
        }]
    );
    assert_eq!(crafting::recipes(&order).len(), 7);
}

#[test]
fn how_many_can_be_made_follows_skill_ingredients_and_category() {
    let (_data, order) = order("crafting-can-make");
    let mut state = GameState::new(&order);
    let stimpak = Recipe::load(&order, FormId(R_STIMPAK)).unwrap();
    let workbench = FormId(WORKBENCH);
    give(&order, &mut state, FLOWER, 5);
    give(&order, &mut state, SCRAP, 3);
    // Medicine below 50: none.
    state.actor_values.insert((PLAYER_REF, 37), 49.0);
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &stimpak, workbench),
        0
    );
    // At 50: flowers 5 / 2 = 2, scrap 3 / 1 = 3: the fewer.
    state.actor_values.insert((PLAYER_REF, 37), 50.0);
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &stimpak, workbench),
        2
    );
    // From another category's menu: none.
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &stimpak, FormId(CAMPFIRE)),
        0
    );
    // A short ingredient: none.
    give(&order, &mut state, FLOWER, -4);
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &stimpak, workbench),
        0
    );
    // At most 100, however much there is.
    let rounds = Recipe::load(&order, FormId(R_ROUNDS)).unwrap();
    give(&order, &mut state, SCRAP, 500);
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &rounds, workbench),
        MOST
    );
    // No subcategory: never.
    let loose = Recipe::load(&order, FormId(R_LOOSE)).unwrap();
    assert_eq!(
        crafting::can_make(&order, &state, PLAYER_REF, &loose, workbench),
        0
    );
}

#[test]
fn the_list_shows_passing_recipes_makeable_first_then_by_name() {
    let (_data, order) = order("crafting-list");
    let mut state = GameState::new(&order);
    give(&order, &mut state, SCRAP, 2);
    give(&order, &mut state, CHIPS, 1);
    let workbench = FormId(WORKBENCH);
    let list = crafting::listing(&order, &state, PLAYER_REF, workbench, None);
    let names: Vec<(&str, u32)> = list
        .lines
        .iter()
        .map(|l| (l.text.as_str(), l.makeable))
        .collect();
    // The gun's condition (TestUnlocked = 1) fails: not listed. The
    // campfire's recipe is in another category. Makeable first (Bloom,
    // Grenade, Rounds), then the rest (Loose, Stimpak), each by name; the
    // rounds come ten at a time.
    assert_eq!(
        names,
        vec![
            ("Bloom", 2),
            ("Grenade", 1),
            ("Rounds (10)", 2),
            ("Loose (2)", 0),
            ("Stimpak", 0)
        ]
    );
    assert_eq!(list.makeable, 3);
    // The filter: the listed recipes' subcategories, by name.
    assert_eq!(list.subcategories, vec![FormId(AID), FormId(AMMO)]);
    // With the global set the gun is listed; narrowed to Ammo, only rounds.
    state.globals.insert(FormId(UNLOCKED), 1.0);
    let list = crafting::listing(&order, &state, PLAYER_REF, workbench, None);
    assert!(list.lines.iter().any(|l| l.recipe == FormId(R_GUN)));
    let ammo = crafting::listing(&order, &state, PLAYER_REF, workbench, Some(FormId(AMMO)));
    assert_eq!(ammo.lines.len(), 1);
    assert_eq!(ammo.lines[0].recipe, FormId(R_ROUNDS));
    assert_eq!(ammo.subcategories, vec![FormId(AID), FormId(AMMO)]);
}

#[test]
fn making_takes_ingredients_gives_outputs_and_counts_it() {
    let (_data, order) = order("crafting-make");
    let mut state = GameState::new(&order);
    give(&order, &mut state, SCRAP, 10);
    give(&order, &mut state, FLOWER, 4);
    state.actor_values.insert((PLAYER_REF, 37), 60.0);
    let stimpak = Recipe::load(&order, FormId(R_STIMPAK)).unwrap();
    let made = crafting::make(&order, &mut state, PLAYER_REF, &stimpak, 2).unwrap();
    assert_eq!(count(&order, &state, FLOWER), 0);
    assert_eq!(count(&order, &state, SCRAP), 8);
    assert_eq!(count(&order, &state, STIMPAK), 2);
    assert_eq!(made.notices, vec!["2 Stimpaks added".to_string()]);
    assert_eq!(state.misc_stats.get(&ITEMS_CRAFTED), Some(&2));

    // One output: "name added".
    let rounds = Recipe::load(&order, FormId(R_ROUNDS)).unwrap();
    let made = crafting::make(&order, &mut state, PLAYER_REF, &rounds, 1).unwrap();
    assert_eq!(made.notices, vec!["10 Rounds added".to_string()]);
    assert_eq!(count(&order, &state, ROUND), 10);
    assert_eq!(state.misc_stats.get(&ITEMS_CRAFTED), Some(&3));

    // Chips first: not counted as crafted. A grenade (thrown) comes whole.
    give(&order, &mut state, CHIPS, 1);
    let grenade = Recipe::load(&order, FormId(R_GRENADE)).unwrap();
    let made = crafting::make(&order, &mut state, PLAYER_REF, &grenade, 1).unwrap();
    assert_eq!(made.notices, vec!["Grenade added".to_string()]);
    assert_eq!(state.misc_stats.get(&ITEMS_CRAFTED), Some(&3));
    assert_eq!(
        state.weapon_health.get(&(PLAYER_REF, FormId(GRENADE))),
        None
    );

    // A gun comes at 80 % condition.
    let gun = Recipe::load(&order, FormId(R_GUN)).unwrap();
    crafting::make(&order, &mut state, PLAYER_REF, &gun, 1).unwrap();
    assert_eq!(
        state.weapon_health.get(&(PLAYER_REF, FormId(GUN))),
        Some(&0.8)
    );
    assert!(crafting::make(&order, &mut state, PLAYER_REF, &gun, 0).is_none());
}

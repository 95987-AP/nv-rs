//! Crafting (`world::crafting`) over `testdata::crafting`'s recipes. What
//! is a guess is said in `docs/DEAD_MONEY.md` "Crafting".

use esm::{ActivePlugins, FormId, LoadOrder};
use testdata::crafting::ids::*;
use world::crafting::{self, Refusal};
use world::dialogue::PLAYER_REF;
use world::scripting::{GameState, Runner, ScriptCache};

fn order(tag: &str) -> (testdata::TempData, LoadOrder) {
    let data = testdata::crafting::crafting(tag);
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    (data, order)
}

fn give(order: &LoadOrder, state: &mut GameState, item: u32, n: i32) {
    state.stock(order, PLAYER_REF);
    *state.items.entry((PLAYER_REF, FormId(item))).or_insert(0) += n;
}

fn have(order: &LoadOrder, state: &GameState, item: u32) -> i32 {
    state.item_count(order, PLAYER_REF, FormId(item))
}

fn run(order: &LoadOrder, state: &mut GameState, source: &str) {
    let scripts = ScriptCache::default();
    Runner::new(order, &scripts, state).run_source(source, None, None);
}

#[test]
fn recipes_read_from_their_records() {
    let (_d, order) = order("craft-read");
    let r = crafting::Recipe::load(&order, FormId(BULLET_RECIPE)).unwrap();
    assert_eq!(r.name, "Bullets");
    assert_eq!(r.skill, Some(40));
    assert_eq!(r.level, 40);
    assert_eq!((r.category, r.sub_category), (FormId(BENCH), FormId(AMMO)));
    assert_eq!(r.ingredients, vec![(FormId(SCRAP), 1)]);
    assert_eq!(r.outputs, vec![(FormId(BULLET), 3)]);
    let stew = crafting::Recipe::load(&order, FormId(STEW_RECIPE)).unwrap();
    assert_eq!(stew.skill, None);
    assert_eq!(
        stew.ingredients,
        vec![(FormId(SCRAP), 2), (FormId(WATER), 1)]
    );
    assert_eq!(crafting::category_name(&order, FormId(BENCH)), "Workbench");
    assert!(crafting::Recipe::load(&order, FormId(SCRAP)).is_none());
    assert_eq!(
        world::stats::NAMES[usize::from(crafting::ITEMS_CRAFTED)],
        "Items Crafted"
    );
}

#[test]
fn the_menu_lists_a_categorys_known_recipes_by_name() {
    let (_d, order) = order("craft-list");
    let mut state = GameState::new(&order);
    let names = |state: &GameState, c: Option<u32>| -> Vec<String> {
        crafting::offers(&order, state, c.map(FormId))
            .into_iter()
            .map(|o| o.recipe.name)
            .collect()
    };
    // The secret recipe waits for its note.
    assert_eq!(names(&state, Some(BENCH)), ["Bullets", "Stew"]);
    assert_eq!(names(&state, Some(FIRE)), ["Boiled Water"]);
    assert_eq!(names(&state, None).len(), 3);
    run(
        &order,
        &mut state,
        &format!("player.AddNote {}", "TestRecipeNote"),
    );
    assert_eq!(
        names(&state, Some(BENCH)),
        ["Bullets", "Secret Stew", "Stew"]
    );
    // Sub-categories in the order they first appear.
    let offers = crafting::offers(&order, &state, Some(FormId(BENCH)));
    assert_eq!(
        crafting::sub_categories(&offers),
        [FormId(AMMO), FormId(AID)]
    );
}

#[test]
fn making_asks_for_the_skill_and_every_ingredient() {
    let (_d, order) = order("craft-make");
    let mut state = GameState::new(&order);
    let make = |state: &mut GameState, r: u32| crafting::make(&order, state, FormId(r));
    // Nothing yet.
    assert_eq!(make(&mut state, STEW_RECIPE), Err(Refusal::Ingredients));
    give(&order, &mut state, SCRAP, 2);
    assert_eq!(make(&mut state, STEW_RECIPE), Err(Refusal::Ingredients));
    give(&order, &mut state, WATER, 1);
    let offer = &crafting::offers(&order, &state, Some(FormId(BENCH)))[1];
    assert!(offer.can_make);
    assert_eq!(make(&mut state, STEW_RECIPE), Ok(vec![(FormId(STEW), 1)]));
    assert_eq!(have(&order, &state, SCRAP), 0);
    assert_eq!(have(&order, &state, WATER), 0);
    assert_eq!(have(&order, &state, STEW), 1);
    assert_eq!(state.misc_stats.get(&crafting::ITEMS_CRAFTED), Some(&1));

    // Bullets want Science 40, however much scrap there is.
    give(&order, &mut state, SCRAP, 5);
    run(&order, &mut state, "player.ForceActorValue Science 39");
    assert_eq!(make(&mut state, BULLET_RECIPE), Err(Refusal::Skill));
    assert!(!crafting::offers(&order, &state, Some(FormId(BENCH)))[0].can_make);
    run(&order, &mut state, "player.ForceActorValue Science 40");
    assert_eq!(
        make(&mut state, BULLET_RECIPE),
        Ok(vec![(FormId(BULLET), 3)])
    );
    assert_eq!(have(&order, &state, BULLET), 3);
    assert_eq!(have(&order, &state, SCRAP), 4);
    assert_eq!(state.misc_stats.get(&crafting::ITEMS_CRAFTED), Some(&2));
}

#[test]
fn unknown_recipes_and_other_forms_are_refused() {
    let (_d, order) = order("craft-refuse");
    let mut state = GameState::new(&order);
    give(&order, &mut state, WATER, 1);
    assert_eq!(
        crafting::make(&order, &mut state, FormId(SECRET_RECIPE)),
        Err(Refusal::Unknown)
    );
    assert_eq!(have(&order, &state, WATER), 1);
    assert_eq!(
        crafting::make(&order, &mut state, FormId(WATER)),
        Err(Refusal::NotARecipe)
    );
    run(&order, &mut state, "player.AddNote TestRecipeNote");
    assert!(crafting::make(&order, &mut state, FormId(SECRET_RECIPE)).is_ok());
    assert_eq!(have(&order, &state, STEW), 1);
}

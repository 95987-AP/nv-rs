//! Companions (`OpenTeammateContainer`, trading things with them).

use esm::{ActivePlugins, FormId, LoadOrder};
use testdata::quest_ids::*;
use world::scripting::{Event, GameState, Runner, ScriptCache};

fn order(tag: &str) -> (testdata::TempData, LoadOrder) {
    let data = testdata::quests(tag);
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    (data, order)
}

/// `OpenTeammateContainer` (`005d9430`): on a teammate, or anyone with a
/// number other than 0; a companion's room (`0075dc80`, `008a0c20`,
/// `00577250`).
#[test]
fn trading_with_a_companion() {
    let (_data, order) = order("companions-trade");
    let scripts = ScriptCache::default();
    let mut state = GameState::new(&order);
    let doc = FormId(DOC_REF);
    let run = |state: &mut GameState, line: &str| {
        state.events.clear();
        Runner::new(&order, &scripts, state).run_source(line, None, None);
        assert!(state.unhandled.is_empty(), "{:?}", state.unhandled_first);
        state.events.clone()
    };
    assert!(run(&mut state, "DocRef.OpenTeammateContainer").is_empty());
    assert_eq!(
        run(&mut state, "DocRef.OpenTeammateContainer 1"),
        vec![Event::TeammateContainer(doc)]
    );
    state.teammates.insert(doc);
    assert_eq!(
        run(&mut state, "DocRef.OpenTeammateContainer"),
        vec![Event::TeammateContainer(doc)]
    );
    // Not on something that isn't a person.
    assert!(run(&mut state, "ChestRef.OpenTeammateContainer 1").is_empty());

    // Carry Weight 200 (the test world has no `fAVDCarryWeights…`).
    state.actor_values.insert((doc, 13), 200.0);
    let (carried, most) = world::items::carry(&order, &state, doc);
    assert!(most > 0.0 && carried <= most);
    // The rifle weighs 6.
    let room = ((most - carried) / 6.0).floor() as i32;
    assert!(world::items::has_room(
        &order,
        &state,
        doc,
        FormId(RIFLE),
        room
    ));
    assert!(!world::items::has_room(
        &order,
        &state,
        doc,
        FormId(RIFLE),
        room + 1
    ));
}

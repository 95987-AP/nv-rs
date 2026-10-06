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

/// The companion wheel (`CompanionWheelMenu`): who it comes up for
/// (`005fa330`, `00754d90`), its switches (`00754de0`), a button's topic
/// said with both result scripts run at once (`007573d0`, `007575b0`), the
/// Stimpak (`00756490`).
#[test]
fn the_companion_wheel() {
    use world::companions;
    use world::living::pickpocket::{use_person, Use};
    let (_data, order) = order("companions-wheel");
    let scripts = ScriptCache::default();
    let mut state = GameState::new(&order);
    let doc = FormId(DOC_REF);
    assert_eq!(use_person(&order, &state, doc, false), Use::Talk);
    state.teammates.insert(doc);
    // Unaggressive: the wheel; frenzied (he'd attack the player): nothing.
    state.actor_values.insert((doc, 0), 0.0);
    assert_eq!(use_person(&order, &state, doc, false), Use::Wheel);
    state.actor_values.insert((doc, 0), 3.0);
    assert_eq!(use_person(&order, &state, doc, false), Use::Nothing);
    state.actor_values.insert((doc, 0), 0.0);
    // Sneaking doesn't pick a teammate's pockets.
    state.player_sneaking = true;
    assert_eq!(use_person(&order, &state, doc, false), Use::Wheel);
    state.player_sneaking = false;

    // His script's variables, at 0 before it has run.
    let switches = companions::switches(&order, &scripts, &state, doc);
    assert_eq!(switches, companions::Switches::default());
    assert_eq!(
        companions::variable(&order, &scripts, &state, doc, "NoSuchThing"),
        -1.0
    );
    assert!(companions::set_variable(
        &order,
        &scripts,
        &mut state,
        doc,
        "FollowerSwitchAggressive",
        1.0
    ));
    assert!(companions::switches(&order, &scripts, &state, doc).aggressive);

    // "Wait Here": his line, then its begin and end scripts.
    let info = companions::say_topic(&order, &scripts, &mut state, doc, "FollowersWait", true)
        .expect("his line");
    assert_eq!(info.responses[0].text, "I'll sit tight.");
    assert!(companions::switches(&order, &scripts, &state, doc).waiting);
    assert_eq!(state.globals.get(&FormId(GLOBAL)), Some(&7.0));

    // The Stimpak: none, then one; not hurt, nothing.
    assert!(!companions::heal_with_stimpak(&order, &mut state, doc));
    state
        .items
        .insert((world::dialogue::PLAYER_REF, FormId(STIMPAK)), 1);
    assert!(!companions::heal_with_stimpak(&order, &mut state, doc));
    // Hurt: healed, the Stimpak used.
    state.damage.insert(doc, 50.0);
    assert!(companions::heal_with_stimpak(&order, &mut state, doc));
    assert_eq!(
        state.item_count(&order, world::dialogue::PLAYER_REF, FormId(STIMPAK)),
        0
    );
    assert!(state.damage.get(&doc).copied().unwrap_or(0.0) < 50.0);
    // A crippled leg, at full health: restored, and the Stimpak used.
    state.damage.remove(&doc);
    state
        .items
        .insert((world::dialogue::PLAYER_REF, FormId(STIMPAK)), 1);
    state.value_damage.insert((doc, 29), 100.0);
    assert!(companions::heal_with_stimpak(&order, &mut state, doc));
    assert!(!state.value_damage.contains_key(&(doc, 29)));
}

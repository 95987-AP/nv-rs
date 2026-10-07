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
    // Not through the dialogue's own result-script call (`0061f170`): the
    // line isn't marked said, nor he talked to.
    assert!(!state.said.contains(&FormId(WAIT_LINE)));
    assert!(!state.talked_to.contains(&doc));

    // Back Up (`008a7760`): the hook for the AI's default package.
    state.events.clear();
    companions::back_up(&mut state, doc);
    assert_eq!(state.events, vec![Event::BackUp(doc)]);

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

/// Nerve (`00644ce0`, `009b5170`, `009b5a30`): a teammate's damage, and
/// their resistance and threshold when hit, × 1 + 0.05 × the player's
/// Charisma (kept within 1 to 10).
#[test]
fn nerve() {
    use world::combat;
    use world::companions;
    let (_data, order) = order("companions-nerve");
    let scripts = ScriptCache::default();
    let mut state = GameState::new(&order);
    let doc = FormId(DOC_REF);
    let player = world::dialogue::PLAYER_REF;
    state.actor_values.insert((player, 8), 6.0);
    assert_eq!(companions::nerve(&order, &state, doc), 1.0);
    let rifle = combat::Weapon::load(&order, FormId(RIFLE)).unwrap();
    let plain = combat::weapon_damage(&order, &state, doc, Some(&rifle), false);
    let plain_armour = combat::through_armour(&order, &state, 100.0, doc, None);
    let mut fists = state.clone();
    let plain_punch = Runner::new(&order, &scripts, &mut fists)
        .hit(doc, FormId(GECKO_REF), None)
        .unwrap();

    state.teammates.insert(doc);
    assert!((companions::nerve(&order, &state, doc) - 1.3).abs() < 1e-6);
    // The player isn't their own teammate.
    assert_eq!(companions::nerve(&order, &state, player), 1.0);
    let nervy = combat::weapon_damage(&order, &state, doc, Some(&rifle), false);
    assert!((nervy - plain * 1.3).abs() < 1e-3, "{plain} {nervy}");
    // Fists: `00644ce0`'s Nerve and `009b5170`'s (the same dice).
    let mut fists = state.clone();
    let punch = Runner::new(&order, &scripts, &mut fists)
        .hit(doc, FormId(GECKO_REF), None)
        .unwrap();
    assert!(
        (punch - plain_punch * 1.3 * 1.3).abs() < 1e-3,
        "{plain_punch} {punch}"
    );

    // Hit: resistance 20 and threshold 4 become 26 and 5.2.
    state.actor_values.insert((doc, 18), 20.0);
    state.actor_values.insert((doc, 76), 4.0);
    let hit = combat::through_armour(&order, &state, 100.0, doc, None);
    assert!((hit - (100.0 * (1.0 - 0.26) - 5.2)).abs() < 1e-3, "{hit}");
    state.teammates.remove(&doc);
    let hit = combat::through_armour(&order, &state, 100.0, doc, None);
    assert!((hit - 76.0).abs() < 1e-3, "{hit} {plain_armour}");

    // Charisma within 1 to 10.
    state.teammates.insert(doc);
    state.actor_values.insert((player, 8), 0.0);
    assert!((companions::nerve(&order, &state, doc) - 1.05).abs() < 1e-6);
    state.actor_values.insert((player, 8), 14.0);
    assert!((companions::nerve(&order, &state, doc) - 1.5).abs() < 1e-6);
}

//! Sitting, idles and sandboxing read from a plugin built from scratch
//! (`testdata::sitting`): the idle tree walked as the game walks it, the
//! sit procedure asking it for each step with `GetSitting` reported to
//! scripts, the marker settings, and a sandbox package's scan and choice.

use esm::{ActivePlugins, FormId, LoadOrder};
use testdata::sitting_ids::*;
use world::furniture::{self, Animations, MarkerSettings, SitState, Sitter, Step};
use world::idles::{IdleAsker, IdleMarker, IdleQuestion, IdleTree};
use world::sandbox::{self, activities, Kind, Nearby, Sandbox};
use world::scripting::{Facts, GameState};

fn order(tag: &str) -> (testdata::TempData, LoadOrder) {
    let data = testdata::sitting(tag);
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    (data, order)
}

const SKELETON: &str = "Characters\\_Male\\Skeleton.nif";

/// The idle the tree gives with the sit values given.
fn pick(
    order: &LoadOrder,
    state: &GameState,
    tree: &IdleTree,
    (sitting, sleeping, marker): (u8, u8, u8),
) -> Option<String> {
    let facts = Facts {
        order,
        state,
        speaker: None,
    };
    let about = IdleQuestion {
        sitting,
        sleeping,
        marker,
        procedure: world::idles::procedures::NONE,
        ..IdleQuestion::default()
    };
    let asker = IdleAsker::new(FormId(SITTER_REF), about, Some(&facts), 7);
    let roots = tree.roots_for(SKELETON);
    tree.evaluate(&roots, &|i| asker.passes(i), &|_| false)
        .map(|i| i.model.rsplit('\\').next().unwrap().to_string())
}

#[test]
fn the_idle_tree_gives_each_step_of_sitting_its_animation() {
    let (_data, order) = order("sitting-tree");
    let tree = IdleTree::load(&order);
    // The loose idle isn't a root.
    assert_eq!(
        tree.roots_for(SKELETON),
        [FormId(FURNITURE_IDLES), FormId(GENERAL_IDLES)]
    );
    let state = GameState::new(&order);
    let at = |sitting: u8| pick(&order, &state, &tree, (sitting, 0, 14));
    assert_eq!(at(1).as_deref(), Some("DynamicIdle_ChairSit.kf"));
    assert_eq!(at(2).as_deref(), Some("Chair_ForwardEnter.kf"));
    assert_eq!(at(3).as_deref(), Some("SitChairRelaxA.kf"));
    assert_eq!(at(4).as_deref(), Some("Chair_ForwardExit.kf"));
    // Standing: the general idles.
    assert_eq!(at(0).as_deref(), Some("Shrug.kf"));
    // Another marker number: the blocking `Sitting` answers "nothing",
    // never the standing idles.
    assert_eq!(pick(&order, &state, &tree, (3, 0, 11)), None);
    let relax = tree.get(FormId(RELAX)).unwrap();
    assert_eq!(
        (relax.group(), relax.loops(), relax.replay_delay()),
        (7, (1, 3), 40)
    );
    assert!(tree.get(FormId(SITTING)).unwrap().is_blocking());
    assert!(tree.get(FormId(WAVE)).unwrap().is_loose());
}

/// Entries take 2 s and walk 55 forward; exits 1.5 s and 55 back.
struct Anims;

impl Animations for Anims {
    fn length(&mut self, model: &str) -> Option<f32> {
        Some(if model.contains("Enter") { 2.0 } else { 1.5 })
    }
    fn root_offset(&mut self, model: &str, time: f32) -> [f32; 3] {
        if model.contains("Enter") {
            [0.0, 55.0 * (time / 2.0).min(1.0), 0.0]
        } else {
            [0.0, -55.0 * (time / 1.5).min(1.0), 0.0]
        }
    }
}

#[test]
fn the_sit_procedure_asks_the_tree_and_scripts_see_get_sitting() {
    let (_data, order) = order("sitting-procedure");
    let tree = IdleTree::load(&order);
    let mut state = GameState::new(&order);
    let me = FormId(SITTER_REF);
    let chair = FormId(CHAIR_REF);
    // The chair's front marker (number 14) facing south into it.
    let marker = nif::FurnitureMarker {
        offset: [0.0, 60.0, -30.0],
        heading: 3141.0 / 1000.0,
        marker: 14,
    };
    let flags = furniture::marker_flags(&order, FormId(CHAIR));
    assert_eq!(flags, 0x4000_0001);
    let placed = furniture::place_markers(&[marker], [100.0, 0.0, 0.0], 0.0, 1.0);
    let m = furniture::nearest_free(&placed, flags, |_| false, [0.0; 3]).unwrap();
    let settings = MarkerSettings::read(&order, 14);
    assert_eq!(settings.delta, [2.4809, 57.3572, -28.948]);
    assert!((settings.heading_delta - std::f32::consts::PI).abs() < 1e-6);
    // Missing settings are 0, as the exe's defaults.
    assert_eq!(MarkerSettings::read(&order, 11), MarkerSettings::default());

    let get = |state: &GameState, function: u16| {
        Facts {
            order: &order,
            state,
            speaker: None,
        }
        .value(function, Some(me), &[])
        .unwrap()
    };
    const GET_SITTING: u16 = 159;
    const MARKER_ID: u16 = 160;
    // Heading for it: the furniture is theirs (`IsCurrentFurnitureRef`),
    // GetSitting still 0.
    state.furniture.insert(me, chair);
    state
        .sitters
        .insert(me, Sitter::new(chair, m, settings, m.position, 0.0));
    assert_eq!(get(&state, GET_SITTING), 0.0);
    assert_eq!(get(&state, MARKER_ID), 14.0);
    let step = |state: &mut GameState, dt: f32, seated_idle: bool| {
        let mut sitter = state.sitters.remove(&me).unwrap();
        let snapshot = state.clone();
        let mut ask = |sitting: u8, sleeping: u8, number: u8| {
            let model = pick(&order, &snapshot, &tree, (sitting, sleeping, number))?;
            Some((FormId(0), model))
        };
        let s = sitter.update(dt, seated_idle, &mut ask, &mut Anims);
        let out = (s, sitter.playing.as_ref().map(|p| p.model.clone()));
        state.sitters.insert(me, sitter);
        out
    };
    let (s, playing) = step(&mut state, 0.0, false);
    assert_eq!(
        (s, playing.as_deref()),
        (Step::Busy, Some("Chair_ForwardEnter.kf"))
    );
    assert_eq!(get(&state, GET_SITTING), 2.0);
    assert_eq!(
        state.sitters[&me].dynamic_idle.as_ref().unwrap().1,
        "DynamicIdle_ChairSit.kf"
    );
    assert_eq!(step(&mut state, 2.5, false).0, Step::Settled);
    assert_eq!(get(&state, GET_SITTING), 3.0);
    // Getting up waits for a seated idle, then plays the exit.
    state.sitters.get_mut(&me).unwrap().stand_up();
    step(&mut state, 0.1, true);
    assert_eq!(state.sitters[&me].state, SitState::Sitting);
    let (_, playing) = step(&mut state, 0.1, false);
    assert_eq!(playing.as_deref(), Some("Chair_ForwardExit.kf"));
    assert_eq!(get(&state, GET_SITTING), 4.0);
    assert_eq!(step(&mut state, 2.0, false).0, Step::Released);
    state.stand(me);
    assert_eq!(get(&state, GET_SITTING), 0.0);
    assert!(!state.furniture.contains_key(&me));
}

#[test]
fn idle_markers_and_settings_are_read_from_their_records() {
    let (_data, order) = order("sitting-records");
    let m = IdleMarker::load(&order, FormId(IDLE_MARKER_REF)).unwrap();
    assert!(m.in_sequence());
    assert_eq!((m.timer, m.idles.clone()), (16.0, vec![FormId(WAVE)]));
    let tree = IdleTree::load(&order);
    assert!(tree.passes_with_parents(FormId(WAVE), &|_| true));
    assert!(!tree.passes_with_parents(FormId(RELAX), &|i| i.form_id != FormId(SITTING)));
    let settings = sandbox::Settings::read(&order);
    // `FalloutNV.esm`'s furniture multiplier; the rest the exe's defaults.
    assert_eq!(settings.duration_mult[0], 3.0);
    assert_eq!(settings.duration_mult[2], 0.75);
    assert_eq!(settings.search_radius, 6000.0);
    assert_eq!(
        sandbox::package_flags(&order, FormId(SANDBOX)),
        sandbox::flags::NO_EATING
    );
    assert_eq!(sandbox::energy(&order, FormId(SITTER_REF)), 50);
}

#[test]
fn a_sandbox_finds_what_its_package_and_the_owners_allow() {
    let (_data, order) = order("sitting-sandbox");
    let mut state = GameState::new(&order);
    let me = FormId(SITTER_REF);
    let package = world::ai::current_package(&order, &state, me).unwrap();
    assert_eq!(package.kind, world::ai::kinds::SANDBOX);
    let settings = sandbox::Settings::read(&order);
    let radius = package.location.unwrap().radius;
    let mut sb = Sandbox::new(
        package.form_id,
        [0.0; 3],
        radius,
        sandbox::package_flags(&order, package.form_id),
        sandbox::energy(&order, me),
        &settings,
    );
    assert_eq!(sb.radius, 512.0);
    // What's in the cell, as the scan sees it.
    let cell = world::find_cells(&order, "TestSittingCell").unwrap()[0];
    let nearby: Vec<Nearby> = order
        .references_in_cell(cell)
        .into_iter()
        .filter_map(|rr| {
            let r = rr.form_id;
            if sandbox::ignored(&order, r) {
                return None;
            }
            let kind = sandbox::kind_of(&order, r)?;
            let (_, _, position, _) = state.place(&order, r)?;
            Some(Nearby {
                reference: r,
                kind,
                position,
                allowed: sandbox::may_use(&order, &state, me, r),
                usable: true,
            })
        })
        .collect();
    let kind = |r: u32| nearby.iter().find(|n| n.reference == FormId(r)).unwrap();
    assert_eq!(kind(CHAIR_REF).kind, Kind::Furniture(0x4000_0001));
    assert_eq!(kind(BED_REF).kind, Kind::Furniture(0x8000_0001));
    assert_eq!(kind(FOOD_REF).kind, Kind::Food);
    assert_eq!(kind(IDLE_MARKER_REF).kind, Kind::IdleMarker);
    assert_eq!(kind(OWNER_REF).kind, Kind::Actor);
    // The owner's chair isn't theirs to use; the marked one is left out.
    assert!(kind(CHAIR_REF).allowed && !kind(OWNED_CHAIR_REF).allowed);
    assert!(sandbox::ignored(&order, FormId(IGNORED_CHAIR_REF)));
    assert!(!nearby
        .iter()
        .any(|n| n.reference == FormId(IGNORED_CHAIR_REF)));
    // Children need furniture flagged for them.
    assert!(!sandbox::child_can_use(&order, FormId(CHAIR_REF)));
    assert!(sandbox::child_can_use(&order, FormId(FOOD_REF)));
    sb.scan(me, &nearby, false);
    let found: Vec<(u8, Option<FormId>)> = sb
        .candidates
        .iter()
        .map(|c| (c.activity, c.target))
        .collect();
    // No eating (the package says so), not the owned chair, not the owner
    // (2000 away), not themselves; the bed (sleeping is allowed), the
    // marker and wandering.
    assert_eq!(
        found,
        [
            (activities::SIT, Some(FormId(CHAIR_REF))),
            (activities::SLEEP, Some(FormId(BED_REF))),
            (activities::IDLE_MARKER, Some(FormId(IDLE_MARKER_REF))),
            (activities::WANDER, None),
        ]
    );
    assert!(sb.chair_found && sb.bed_found && !sb.food_found);
    // At 15:00 (no meal, no sleep) they sit, use the marker or wander,
    // each for its game minutes; never sleep by day.
    let mut dice = 99u64;
    let mut roll = || {
        dice ^= dice << 13;
        dice ^= dice >> 7;
        dice ^= dice << 17;
        dice
    };
    let hour = state.global(&order, "GameHour").unwrap();
    let mut seen = std::collections::HashSet::new();
    for k in 0..40 {
        let here = |c: &sandbox::Candidate| {
            c.target
                .map_or(Some([0.0; 3]), |t| state.place(&order, t).map(|p| p.2))
        };
        let c = sb
            .choose(
                me,
                &settings,
                100.0 * k as f32,
                (0, hour),
                &mut roll,
                None,
                &here,
            )
            .unwrap();
        assert_ne!(c.activity, activities::SLEEP);
        assert!(c.duration > 0.0);
        seen.insert(c.activity);
    }
    assert!(seen.contains(&activities::SIT) && seen.contains(&activities::WANDER));
    state.stand(me);
}

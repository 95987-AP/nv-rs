//! Companions' wheel (`CompanionWheelMenu`, class 1075, vtable `01071d0c`
//! in FalloutNV.exe; notes in `docs/COMPANIONS.md`): what the menu reads
//! from the companion and what its buttons do to them.
//!
//! - Using a teammate (the player using them, `005fa330`, `00607990`):
//!   the wheel when they can take orders (`00754d90`: alive and up, and not
//!   about to attack the player); else nothing.
//! - Its four switches read the companion's script variables
//!   (`00754de0`, `008c1740`; -1 for one their script lacks):
//!   `FollowerSwitchAggressive` 1, `IsFollowingLong` 1, `CombatStyleRanged`
//!   1, `Waiting` not 0.
//! - A button said as a topic (`007573d0`: the companion's line for it,
//!   `0061a720`) and done (`007575b0`: the line's begin and end result
//!   scripts run on them at once, `005ac1e0`); the aggressive switch also
//!   sets `FollowerSwitchAggressive` itself (`008c16c0`).
//! - A Stimpak (`00756490`): with one in the player's things (`00576260`),
//!   a companion hurt (health below full: `00893590`, current over base
//!   Health, 1 when the base is 0) or with a crippled limb (a condition,
//!   actor values 25 to 30, at 0) gets the Stimpak's effects when hurt
//!   (`008c1f80`), the player loses one (vtable +0x17C), every condition
//!   (25 to 31, the brain's too) is restored by 1000 (`0088b740`), and they
//!   say their `Regenerating` line (`008d5910(13)`, `007573d0`).

use esm::{FormId, LoadOrder};

use crate::dialogue::{Info, Speaker, PLAYER_REF};
use script::interp::Locals;

use crate::scripting::{base_of, script_of, GameState, Runner, ScriptCache};

/// The menu's class.
pub const CLASS: i32 = 1075;

/// A companion's script variable by name (`008c1740`): -1 when their
/// script has none. A script that hasn't run yet has its variables at 0,
/// as the game's are from the start.
pub fn variable(
    order: &LoadOrder,
    scripts: &ScriptCache,
    state: &GameState,
    who: FormId,
    name: &str,
) -> f32 {
    let name = name.to_ascii_lowercase();
    let value = match state.variables.get(&who) {
        Some(l) => l.get(&name),
        None => new_locals(order, scripts, who).and_then(|l| l.get(&name)),
    };
    value.map_or(-1.0, |v| v as f32)
}

/// Sets one (`008c16c0`), when their script has it.
pub fn set_variable(
    order: &LoadOrder,
    scripts: &ScriptCache,
    state: &mut GameState,
    who: FormId,
    name: &str,
    value: f32,
) -> bool {
    let locals = match state.variables.entry(who) {
        std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
        std::collections::hash_map::Entry::Vacant(e) => {
            let Some(l) = new_locals(order, scripts, who) else {
                return false;
            };
            e.insert(l)
        }
    };
    locals.set(&name.to_ascii_lowercase(), f64::from(value))
}

fn new_locals(order: &LoadOrder, scripts: &ScriptCache, who: FormId) -> Option<Locals> {
    let script = scripts.script(order, script_of(order, who)?)?;
    Some(Locals::new(&script))
}

/// The wheel's four switches as the companion's variables have them
/// (`00754de0`: the menu's bytes +0x6D, +0x6F, +0x6C, +0x6E; buttons 0, 2,
/// 5 and 7 show their other picture, `user11`, when on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switches {
    pub aggressive: bool,
    pub waiting: bool,
    pub keep_distance: bool,
    pub ranged: bool,
}

pub fn switches(
    order: &LoadOrder,
    scripts: &ScriptCache,
    state: &GameState,
    who: FormId,
) -> Switches {
    let v = |name: &str| variable(order, scripts, state, who, name);
    Switches {
        aggressive: v("FollowerSwitchAggressive") == 1.0,
        // The menu keeps "following" (`Waiting` 0); the button shows the
        // other.
        waiting: v("Waiting") != 0.0,
        keep_distance: v("IsFollowingLong") == 1.0,
        ranged: v("CombatStyleRanged") == 1.0,
    }
}

/// Whether using a teammate brings up the wheel (`00754d90`): alive and
/// up (`00884480`: not dead or unconscious), and they wouldn't attack the
/// player (`008b06d0`, `GetShouldAttack`: [`crate::factions::attacks_on_sight`]).
pub fn wheel_allowed(order: &LoadOrder, state: &GameState, who: FormId) -> bool {
    if state.dead.contains(&who) || state.unconscious.contains(&who) {
        return false;
    }
    !crate::factions::attacks_on_sight(order, state, who, PLAYER_REF)
}

/// A companion says their line for a topic (`007573d0`) and, with
/// `run_scripts`, its result scripts run on them at once (`007575b0`: the
/// begin script, then the end script). The line said, if they have one.
pub fn say_topic(
    order: &LoadOrder,
    scripts: &ScriptCache,
    state: &mut GameState,
    who: FormId,
    topic: &str,
    run_scripts: bool,
) -> Option<Info> {
    let topic = order.form_by_editor_id(topic)?;
    let base = base_of(order, who)?;
    let speaker = Speaker::load(order, who, base)?;
    let info = crate::dialogue::pick(order, topic, &speaker, state)?;
    crate::dialogue::line_begins(state, &info, who);
    if run_scripts {
        for source in [info.begin_script.as_deref(), info.end_script.as_deref()]
            .into_iter()
            .flatten()
        {
            Runner::new(order, scripts, state).run_source(source, Some(who), Some(who));
        }
    }
    Some(info)
}

/// The Stimpak (`DOBJ` default object 0).
pub fn stimpak(order: &LoadOrder) -> Option<FormId> {
    crate::items::default_object(order, 0)
}

/// The limbs' conditions (actor values 25 to 30), looked at for a cripple.
const LIMBS: [u16; 6] = [25, 26, 27, 28, 29, 30];
/// Those a Stimpak from the wheel restores: the brain's (31) too.
const RESTORED: [u16; 7] = [31, 29, 30, 27, 28, 26, 25];

/// The wheel's Stimpak (`00756490`): true when one was used.
pub fn heal_with_stimpak(order: &LoadOrder, state: &mut GameState, who: FormId) -> bool {
    let Some(stim) = stimpak(order) else {
        return false;
    };
    if state.item_count(order, PLAYER_REF, stim) <= 0 {
        return false;
    }
    let facts = crate::scripting::Facts {
        order,
        state,
        speaker: None,
    };
    let crippled = LIMBS
        .iter()
        .any(|&av| facts.current_actor_value(who, av).unwrap_or(100.0) == 0.0);
    let health = crate::combat::health(order, state, who).unwrap_or(0.0);
    let full = crate::combat::max_health(order, state, who).unwrap_or(0.0);
    let hurt = full > 0.0 && health / full < 1.0;
    if !hurt && !crippled {
        return false;
    }
    if hurt {
        crate::magic::apply(order, state, who, stim, PLAYER_REF, false);
    }
    state.stock(order, PLAYER_REF);
    if let Some(n) = state.items.get_mut(&(PLAYER_REF, stim)) {
        *n = (*n - 1).max(0);
    }
    for av in RESTORED {
        crate::magic::change(order, state, who, av, 1000.0, PLAYER_REF);
    }
    true
}

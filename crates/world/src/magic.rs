//! Effects on people and creatures: what aid items, spells and abilities do
//! to their actor values, and for how long.
//!
//! An item or spell lists its effects (`EFID` + `EFIT`, read by
//! [`crate::items::effects`]); each names a magic effect (`MGEF`) whose
//! `DATA` says how it works. Read from the records and checked against
//! what the game's items are known to do:
//!
//! - **Recover** (flag 0x02): the value comes back when the effect ends,
//!   so it's a modifier while it lasts. Buffout's "Increase Strength"
//!   (0x72) has it: +2 Strength for 240 s, then back.
//! - Without it the change stays: at once with no duration, else the
//!   magnitude every second for the duration ("Restore Health" 0x70: a
//!   Nuka-Cola's 2 for 25 s heals 50 over 25 s).
//! - **Detrimental** (flag 0x04) lowers the value; the resisting actor
//!   value (at 16) takes its percentage off ("Damage Rads" is resisted by
//!   Rad Resistance, 20).
//! - **Script** effects (archetype 1) run their script (the associated
//!   item, at 8) on whoever they affect: `ScriptEffectStart` once,
//!   `ScriptEffectUpdate` while they last, `ScriptEffectFinish` at the end.
//!   Nuka-Cola's cap effect is `additem caps001 1` at the start.
//!
//! Rads, dehydration, hunger and sleep deprivation count up as they're
//! "damaged": the game's own scripts read `player.getav RadiationRads >=
//! 200` (`CGTutorialSCRIPT`) and cure with `player.RestoreAV
//! RadiationRads 1000` (the doctors' dialogue).
//!
//! Guesses: changes over time are applied smoothly rather than once a
//! second; resistance scales a detrimental change by (1 − resistance /
//! 100); a script effect with no duration runs start, update and finish
//! in one go; abilities and diseases last while held, other spells given
//! with `AddSpell` are cast once; nothing ends an effect early on death
//! but the death itself.

use esm::{FormId, LoadOrder};
use script::interp::Locals;

use crate::combat::av;
use crate::scripting::{Facts, GameState, Runner};

/// Magic effect archetypes (`MGEF` `DATA` u32 at 64) this module tells
/// apart (the game's list: 0 value modifier, 1 script, 2 dispel, 3 cure
/// disease, 24 paralysis, 33 concussion, 34 value and parts, 35 limb
/// condition, 36 turbo, …).
pub mod archetype {
    pub const VALUE_MODIFIER: u32 = 0;
    pub const SCRIPT: u32 = 1;
    pub const VALUE_AND_PARTS: u32 = 34;
}

/// Actor values that count up as they're damaged: radiation (54), and
/// hardcore mode's dehydration, hunger and sleep deprivation (73–75).
pub const COUNTERS: [u16; 4] = [54, 73, 74, 75];

/// Spell types (`SPIT` u32 at 0) that last as long as they're held:
/// disease 1, ability 4, addiction 10.
const HELD_SPELL_TYPES: [u32; 3] = [1, 4, 10];

/// An effect working on someone.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveEffect {
    pub target: FormId,
    /// The item or spell it came from.
    pub source: FormId,
    /// The magic effect (`MGEF`).
    pub effect: FormId,
    pub actor_value: i32,
    pub magnitude: f32,
    pub detrimental: bool,
    pub recover: bool,
    pub archetype: u32,
    pub resist: i32,
    pub script: Option<FormId>,
    /// Seconds left; infinite while an ability is held.
    pub remaining: f32,
    /// Script effects: whether `ScriptEffectStart` has run, and the
    /// script's variables.
    pub started: bool,
    pub locals: Locals,
}

impl ActiveEffect {
    fn changes_value(&self) -> bool {
        matches!(
            self.archetype,
            archetype::VALUE_MODIFIER | archetype::VALUE_AND_PARTS
        ) && self.actor_value >= 0
    }

    fn sign(&self) -> f64 {
        if self.detrimental {
            -1.0
        } else {
            1.0
        }
    }
}

/// What an effect's recover modifiers add to an actor value now (Buffout's
/// +2 Strength while it lasts). `held_only`: only those that last while a
/// spell is held (abilities), for the "permanent" value.
pub fn modifier(state: &GameState, who: FormId, value: u16, held_only: bool) -> f64 {
    state
        .active_effects
        .iter()
        .filter(|e| {
            e.target == who
                && e.recover
                && e.changes_value()
                && e.actor_value == i32::from(value)
                && (!held_only || e.remaining.is_infinite())
        })
        .map(|e| e.sign() * f64::from(e.magnitude))
        .sum()
}

/// What's working on someone, for the player's screen: one line per effect
/// on a value, with its item or spell and the time left ("Buffout: +2
/// Strength, 230 s left").
pub fn summary(order: &LoadOrder, state: &GameState, who: FormId) -> Vec<String> {
    let name = |f: FormId| {
        order
            .get(f)
            .and_then(|r| r.record().ok())
            .and_then(|r| {
                r.full_name()
                    .filter(|n| !n.is_empty())
                    .or_else(|| r.editor_id())
            })
            .unwrap_or_else(|| f.to_string())
    };
    state
        .active_effects
        .iter()
        .filter(|e| e.target == who && e.changes_value())
        .map(|e| {
            let v = e.actor_value as u16;
            let up = !e.detrimental != COUNTERS.contains(&v);
            let per = if e.recover { "" } else { " a second" };
            let left = if e.remaining.is_finite() {
                format!(", {:.0} s left", e.remaining.ceil())
            } else {
                String::new()
            };
            format!(
                "{}: {}{} {}{per}{left}",
                name(e.source),
                if up { "+" } else { "-" },
                e.magnitude,
                crate::chargen::actor_value_name(order, v)
            )
        })
        .collect()
}

/// Whether someone is under a magic effect (`HasMagicEffect`).
pub fn has_effect(state: &GameState, who: FormId, effect: FormId) -> bool {
    state
        .active_effects
        .iter()
        .any(|e| e.target == who && e.effect == effect)
}

/// Whether someone is under an item's or spell's effects (`IsSpellTarget`).
pub fn is_target_of(state: &GameState, who: FormId, source: FormId) -> bool {
    state
        .active_effects
        .iter()
        .any(|e| e.target == who && e.source == source)
}

/// A value changes for good: positive restores, negative damages. Health
/// goes through [`crate::combat`] (damage can kill); the counters
/// ([`COUNTERS`]) go up when damaged; other values are lowered by damage
/// and restored up to what they were. A counter's change then goes through
/// its stages (radiation sickness, hardcore's needs:
/// `world::living::needs::changed`, the game's actor-value callbacks).
/// Whether it killed them.
pub fn change(
    order: &LoadOrder,
    state: &mut GameState,
    who: FormId,
    value: u16,
    amount: f64,
    by: FormId,
) -> bool {
    if value == av::HEALTH {
        if amount < 0.0 {
            return crate::combat::hurt(order, state, who, -amount, by);
        }
        let lost = state.damage.entry(who).or_insert(0.0);
        *lost = (*lost - amount).max(0.0);
        return false;
    }
    let counter = COUNTERS.contains(&value);
    let before = counter.then(|| crate::living::needs::value(order, state, who, value));
    let damage = state.value_damage.entry((who, value)).or_insert(0.0);
    *damage = (*damage - amount).max(0.0);
    if *damage == 0.0 {
        state.value_damage.remove(&(who, value));
    }
    if let Some(old) = before {
        let new = crate::living::needs::value(order, state, who, value);
        if new != old {
            crate::living::needs::changed(order, state, who, value, old, new);
        }
    }
    false
}

/// An effect changes its value by `amount`; a "value and parts" one
/// (archetype 34, the Stimpak's "Restore Health & Conditions") also every
/// body part condition (actor values 25–31) by `amount` ×
/// `fMagicVACNoPartTargetedMult` (0.1, the exe's default; `0082b970`,
/// when it isn't aimed at one part from the Pip-Boy, which isn't done).
/// Whether it killed them.
fn change_with_parts(
    order: &LoadOrder,
    state: &mut GameState,
    effect: &ActiveEffect,
    amount: f64,
    by: FormId,
) -> bool {
    let died = change(
        order,
        state,
        effect.target,
        effect.actor_value as u16,
        amount,
        by,
    );
    if effect.archetype == archetype::VALUE_AND_PARTS {
        let share = f64::from(
            crate::scripting::game_setting(order, "fMagicVACNoPartTargetedMult").unwrap_or(0.1),
        );
        for part in crate::body_parts::av::FIRST_CONDITION..=crate::body_parts::av::LAST_CONDITION {
            change(order, state, effect.target, part, amount * share, by);
        }
    }
    died
}

/// Applies an item's or spell's effects to `target` (cast by `caster`):
/// each effect whose conditions pass. `held`: an ability's, lasting as long
/// as it's held. Says what each did, for messages ("+2 Strength for
/// 240 s").
pub fn apply(
    order: &LoadOrder,
    state: &mut GameState,
    target: FormId,
    source: FormId,
    caster: FormId,
    held: bool,
) -> Vec<String> {
    let mut done = Vec::new();
    let ingestible = crate::items::ingestible_flags(order, source);
    let addiction = spell_type(order, source) == Some(ADDICTION_SPELL_TYPE);
    for e in crate::items::effects(order, source) {
        let applies = Facts {
            order,
            state,
            speaker: None,
        }
        .conditions_pass(&e.conditions, target, caster);
        if !applies {
            continue;
        }
        let magnitude = ingestible_magnitude(order, state, target, ingestible, &e);
        let mut duration = e.duration.max(0) as f32;
        if addiction {
            // An addiction's withdrawal lasts as the target's perks'
            // "Modify Addiction Duration" (entry point 24) say
            // (`00823210`; no perk in the game's data uses it).
            duration = crate::perks::apply_for(
                order,
                state,
                target,
                crate::perks::entry::MODIFY_ADDICTION_DURATION,
                duration,
                &[],
            );
        } else if ingestible.is_some() && !e.hostile {
            // A food's or chem's benefit lasts as the target's perks'
            // "Modify Positive Chem Duration" (25: Chemist × 2, Day
            // Tripper × 1.33) say; hostile effects (`MGEF` flag 0x01) as
            // long as ever (`00823210`).
            duration = crate::perks::apply_for(
                order,
                state,
                target,
                crate::perks::entry::MODIFY_POSITIVE_CHEM_DURATION,
                duration,
                &[],
            );
        }
        let mut active = ActiveEffect {
            target,
            source,
            effect: e.effect,
            actor_value: e.actor_value,
            magnitude,
            detrimental: e.harmful,
            recover: e.recover,
            archetype: e.archetype,
            resist: e.resist,
            script: e.script,
            remaining: if held { f32::INFINITY } else { duration },
            started: false,
            locals: Locals::default(),
        };
        if active.detrimental && active.resist >= 0 {
            let resisted = Facts {
                order,
                state,
                speaker: None,
            }
            .current_actor_value(target, active.resist as u16)
            .unwrap_or(0.0);
            active.magnitude *= (1.0 - (resisted as f32 / 100.0)).clamp(0.0, 1.0);
        }
        let value_name = (active.actor_value >= 0)
            .then(|| crate::chargen::actor_value_name(order, active.actor_value as u16));
        // Rads go up when damaged.
        let counter = active.actor_value >= 0 && COUNTERS.contains(&(active.actor_value as u16));
        let amount = format!(
            "{}{}",
            if active.detrimental != counter {
                "-"
            } else {
                "+"
            },
            active.magnitude
        );
        match (active.changes_value(), value_name) {
            // At once.
            (true, Some(name)) if active.remaining == 0.0 => {
                if !active.recover {
                    let amount_now = active.sign() * f64::from(active.magnitude);
                    change_with_parts(order, state, &active, amount_now, caster);
                    done.push(format!("{amount} {name}"));
                }
                continue;
            }
            (true, Some(name)) => {
                let per = if active.recover { "" } else { " a second" };
                if active.remaining.is_finite() {
                    done.push(format!("{amount} {name}{per} for {} s", active.remaining));
                } else {
                    done.push(format!("{amount} {name}{per}"));
                }
            }
            _ => {}
        }
        state.active_effects.push(active);
    }
    done
}

/// The spell type an addiction is (`SPIT` u32 at 0).
const ADDICTION_SPELL_TYPE: u32 = 10;

/// A spell's type (`SPIT` u32 at 0: 0 actor effect, 1 disease, 4 ability,
/// 10 addiction); `None` for anything but a spell.
fn spell_type(order: &LoadOrder, spell: FormId) -> Option<u32> {
    let rr = order
        .get(spell)
        .filter(|r| r.entry.header.kind.as_bytes() == b"SPEL")?;
    let d = rr
        .record()
        .ok()?
        .get(esm::FourCC::new(b"SPIT"))
        .filter(|s| s.data.len() >= 4)?
        .data
        .clone();
    Some(crate::cell::le_u32(&d, 0))
}

/// An aid item's effect magnitude as the game sets it up (`00815d00`,
/// read from the code): a **medicine** (`ENIT` flag 0x04, the Stimpak)
/// restores its magnitude × (`fMagicMedicineSkillBase` + the player's
/// Medicine ÷ 100 × `fMagicMedicineSkillMult` 2) on every effect; a
/// **food** (0x02) its magnitude × (`fMagicSurvivalSkillBase` + Survival
/// ÷ 100 × `fMagicSurvivalSkillMult` 2) on every effect but radiation
/// (actor value 54), whose magnitude goes through the eater's perks'
/// "Modify Radiation Consumed" (entry point 44: Lead Belly × 0.5) instead;
/// either kind's Health effects (16) also through the perks' "Modify
/// Recovered Health" (12: Better Healing × 1.2) when the one healed is the
/// player. Neither flag: the magnitude as written. The skill is the
/// player's whoever eats (`0066ef50`).
fn ingestible_magnitude(
    order: &LoadOrder,
    state: &GameState,
    target: FormId,
    flags: Option<u8>,
    e: &crate::items::ItemEffect,
) -> f32 {
    use crate::items::ingestible::{FOOD, MEDICINE};
    let base = e.magnitude as f32;
    let Some(flags) = flags else {
        return base;
    };
    let setting = |n: &str, d: f32| crate::scripting::game_setting(order, n).unwrap_or(d);
    let player_skill = |skill: u16| {
        Facts {
            order,
            state,
            speaker: None,
        }
        .current_actor_value(crate::dialogue::PLAYER_REF, skill)
        .unwrap_or(0.0) as f32
    };
    let recovered_health = |m: f32| {
        if target == crate::dialogue::PLAYER_REF && e.actor_value == i32::from(av::HEALTH) {
            crate::perks::apply_for(
                order,
                state,
                target,
                crate::perks::entry::MODIFY_RECOVERED_HEALTH,
                m,
                &[],
            )
        } else {
            m
        }
    };
    let mut magnitude = base;
    if flags & MEDICINE != 0 {
        let medicine = setting("fMagicMedicineSkillBase", MEDICINE_SKILL_BASE)
            + player_skill(37) / 100.0 * setting("fMagicMedicineSkillMult", 2.0);
        magnitude = recovered_health(base * medicine);
    }
    if flags & FOOD != 0 {
        if e.actor_value == i32::from(RADIATION) {
            magnitude = base
                * crate::perks::apply_for(
                    order,
                    state,
                    target,
                    crate::perks::entry::MODIFY_RADIATION_CONSUMED,
                    1.0,
                    &[],
                );
        } else {
            let survival = setting("fMagicSurvivalSkillBase", SURVIVAL_SKILL_BASE)
                + player_skill(44) / 100.0 * setting("fMagicSurvivalSkillMult", 2.0);
            magnitude = recovered_health(base * survival);
        }
    }
    magnitude
}

/// The radiation actor value (`RadiationRads`).
const RADIATION: u16 = 54;

/// The exe's defaults for `fMagicMedicineSkillBase` and
/// `fMagicSurvivalSkillBase` (the data doesn't set them).
const MEDICINE_SKILL_BASE: f32 = 1.0;
const SURVIVAL_SKILL_BASE: f32 = 1.0;

/// A spell is cast on someone (`CastImmediateOnSelf`, `Cast`) or given to
/// them (`AddSpell`): abilities, diseases and addictions given last while
/// held; anything else takes effect as cast.
pub fn add_spell(
    order: &LoadOrder,
    state: &mut GameState,
    target: FormId,
    spell: FormId,
    caster: FormId,
    given: bool,
) -> Vec<String> {
    let held = given
        && order
            .get(spell)
            .and_then(|r| r.record().ok())
            .and_then(|r| r.get(esm::FourCC::new(b"SPIT")).map(|s| s.data.clone()))
            .is_some_and(|d| {
                d.len() >= 4 && HELD_SPELL_TYPES.contains(&crate::cell::le_u32(&d, 0))
            });
    if held && is_target_of(state, target, spell) {
        return Vec::new();
    }
    apply(order, state, target, spell, caster, held)
}

/// Takes an item's or spell's effects off someone (`RemoveSpell`,
/// `Dispel`): what was taken off (script effects still to finish).
pub fn remove(state: &mut GameState, target: FormId, source: FormId) -> Vec<ActiveEffect> {
    let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut state.active_effects)
        .into_iter()
        .partition(|e| e.target == target && e.source == source);
    state.active_effects = kept;
    gone
}

/// Time passes for every effect: changes over time are applied, script
/// effects run, and what has run its course ends. The dead lose theirs.
pub fn tick(runner: &mut Runner, seconds: f32) {
    let order = runner.order;
    let active = std::mem::take(&mut runner.state.active_effects);
    let mut kept = Vec::with_capacity(active.len());
    for mut e in active {
        if runner.state.dead.contains(&e.target) {
            continue;
        }
        let step = seconds.min(e.remaining.max(0.0));
        if e.changes_value() && !e.recover && step > 0.0 {
            let amount = e.sign() * f64::from(e.magnitude) * f64::from(step);
            if change_with_parts(order, runner.state, &e, amount, e.target) {
                runner.run_event(e.target, "ondeath", e.target);
                continue;
            }
        }
        if let Some(script) = e.script {
            if !e.started {
                e.started = true;
                runner.run_effect_script(script, &mut e, "scripteffectstart", seconds);
            }
            runner.run_effect_script(script, &mut e, "scripteffectupdate", seconds);
        }
        e.remaining -= seconds;
        if e.remaining > 0.0 {
            kept.push(e);
        } else if let Some(script) = e.script {
            runner.run_effect_script(script, &mut e, "scripteffectfinish", seconds);
        }
    }
    // Effects scripts added meanwhile come after.
    kept.append(&mut runner.state.active_effects);
    runner.state.active_effects = kept;
}

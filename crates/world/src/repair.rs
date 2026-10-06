//! Repairing and modding the player's items, as the Pip-Boy's Repair menu
//! (`RepairMenu`, `repair_menu.xml`) and item mod menu (`ItemModMenu`,
//! `item_mod_menu.xml`) do it in FalloutNV.exe 1.4.0.525.
//!
//! - What an item can be mended with: another of the same item, or an item
//!   on its repair list (`REPL`, a form list): `004d4bd0`.
//! - The condition a repair gives (`00648090`): from the Repair skill and
//!   the two items' conditions, with `fRepairMin`, `fRepairMax`,
//!   `fRepairScavengeMult`, `fRepairSkillBase` and `fRepairSkillMax`.
//! - Repairing (`007b5d80`, `RepairMenu::RepairItem` (Xbox PDB)): the
//!   broken item takes that condition (at most full), one of the mending
//!   items goes, "Items Repaired" goes up.
//! - Weapon mods (`00783af0`, `ItemModMenu::ItemModItem` (Xbox PDB)): a
//!   weapon's three mod slots (flags 1, 2, 4) each take one item mod
//!   (`IMOD`, the weapon's `WMI1` .. `WMI3`); fitting one sets its flag,
//!   takes the mod item, and raises "Weapon Modifications".
//!
//! The world keeps one condition per carried weapon (and none for apparel,
//! which stays whole): copies of a weapon share it, so "another of the same
//! item" mends with that same condition.

use esm::{FormId, FourCC, LoadOrder};

use crate::dialogue::PLAYER_REF;
use crate::scripting::{game_setting, GameState};

/// The Repair skill (actor value 39, `0066ef20(0x27)`).
pub const REPAIR_SKILL: u16 = 0x27;

/// "Weapon Modifications" and "Items Repaired" (misc statistics 0x21 and
/// 0x22, `004d5c60`).
pub const STAT_WEAPON_MODS: u8 = 0x21;
pub const STAT_ITEMS_REPAIRED: u8 = 0x22;

/// The repair settings, the plugins' or the exe's defaults (`00f67ba0`,
/// `00f67c00`, `00f67c30` and their neighbours).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairSettings {
    pub min: f32,
    pub max: f32,
    pub scavenge_mult: f32,
    pub skill_base: f32,
    pub skill_max: f32,
}

impl Default for RepairSettings {
    fn default() -> Self {
        RepairSettings {
            min: 0.5,
            max: 2.0,
            scavenge_mult: 0.05,
            skill_base: 4.0,
            skill_max: 9.0,
        }
    }
}

impl RepairSettings {
    pub fn load(order: &LoadOrder) -> RepairSettings {
        let d = RepairSettings::default();
        let get = |name: &str, v: f32| game_setting(order, name).unwrap_or(v);
        RepairSettings {
            min: get("fRepairMin", d.min),
            max: get("fRepairMax", d.max),
            scavenge_mult: get("fRepairScavengeMult", d.scavenge_mult),
            skill_base: get("fRepairSkillBase", d.skill_base),
            skill_max: get("fRepairSkillMax", d.skill_max),
        }
    }
}

/// `004bd510(v, 1)`: the whole part, plus one when what's left is at least
/// a half.
fn round_half_up(v: f32) -> i32 {
    let whole = v.trunc();
    whole as i32 + i32::from(v - whole >= 0.5)
}

/// The condition (percent) mending an item in condition `a` with one in
/// condition `b` (both percent) gives, before it's held to 100; and, when
/// that's no better than the better of the two, the Repair skill it would
/// take (the menu's "%d%s REPAIR SKILL NEEDED").
// Translated from 00648090 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn repaired_condition(s: &RepairSettings, skill: i32, a: f32, b: f32) -> (f32, Option<i32>) {
    let (a, b) = (a / 10.0, b / 10.0);
    let better = a.max(b);
    let worse = a.min(b);
    let skill_term = (s.max - s.min) * skill as f32 / 100.0;
    let r = better + s.min + skill_term + worse * s.scavenge_mult;
    let mut needed = None;
    if round_half_up(r * 100.0) <= round_half_up(better * 100.0) {
        let span = s.skill_max - s.skill_base;
        needed = Some(
            ((f64::from(better) + 0.1 - f64::from(s.skill_base)) * 100.0 / f64::from(span)) as i32,
        );
    }
    (r * 10.0, needed)
}

/// An item's condition, percent (`004bcdb0(1)`): a weapon's from the
/// world; everything else whole.
pub fn condition_percent(order: &LoadOrder, state: &GameState, item: FormId) -> f32 {
    match order.get(item).map(|r| r.entry.header.kind) {
        Some(k) if k.as_bytes() == b"WEAP" => {
            crate::combat::weapon_condition(state, PLAYER_REF, item) * 100.0
        }
        _ => 100.0,
    }
}

/// What can mend an item (`004d4bd0`, read with the inventory menu's
/// `00781860`): the item itself (another of it) and its repair list
/// (`REPL`).
pub fn menders(order: &LoadOrder, item: FormId) -> Vec<FormId> {
    let mut out = vec![item];
    let list = order
        .get(item)
        .and_then(|r| r.record().ok().map(|rec| (r.plugin, rec)))
        .and_then(|(plugin, rec)| {
            rec.get(FourCC::new(b"REPL"))
                .filter(|s| s.data.len() >= 4)
                .map(|s| {
                    plugin.to_global(FormId(u32::from_le_bytes([
                        s.data[0], s.data[1], s.data[2], s.data[3],
                    ])))
                })
        });
    if let Some(list) = list {
        for f in crate::perks::form_list(order, list) {
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// The player's Repair skill now.
pub fn repair_skill(order: &LoadOrder, state: &GameState) -> i32 {
    crate::scripting::Facts {
        order,
        state,
        speaker: None,
    }
    .current_actor_value(PLAYER_REF, REPAIR_SKILL)
    .unwrap_or(0.0) as i32
}

/// Takes `count` of an item from the player.
fn take(order: &LoadOrder, state: &mut GameState, item: FormId, count: i32) {
    state.stock(order, PLAYER_REF);
    let n = state.items.entry((PLAYER_REF, item)).or_insert(0);
    *n = (*n - count).max(0);
    if *n == 0 {
        state.items.remove(&(PLAYER_REF, item));
        state.unequip(PLAYER_REF, item);
    }
}

/// Mends the player's `broken` item with one of `with`: the broken item's
/// condition becomes the repaired one (at most full), one `with` goes, and
/// "Items Repaired" goes up. False when it can't be (not carried; apparel,
/// whose condition the world doesn't keep, stays as it is).
// Translated from 007b5d80 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn repair(order: &LoadOrder, state: &mut GameState, broken: FormId, with: FormId) -> bool {
    let have = |s: &GameState, f: FormId| s.item_count(order, PLAYER_REF, f);
    if have(state, broken) <= 0 || have(state, with) <= 0 {
        return false;
    }
    if with == broken && have(state, broken) < 2 {
        return false;
    }
    let settings = RepairSettings::load(order);
    let skill = repair_skill(order, state);
    let a = condition_percent(order, state, broken);
    let b = condition_percent(order, state, with);
    let share = (repaired_condition(&settings, skill, b, a).0 / 100.0).min(1.0);
    take(order, state, with, 1);
    let is_weapon = order
        .get(broken)
        .is_some_and(|r| r.entry.header.kind.as_bytes() == b"WEAP");
    if is_weapon {
        state.weapon_health.insert((PLAYER_REF, broken), share);
    }
    crate::stats::bump(state, STAT_ITEMS_REPAIRED, 1);
    true
}

/// A weapon's three mod slots: (flag, mod item `WMI1`..`WMI3`, the effect
/// kind and its two values from `DNAM` 140.., 152.., 184..), as the
/// weapon's `+0x350`, `+0x180`, `+0x18c` and `+0x1ac` hold them
/// (`004bd570`, `004bd880`, `004bcf60`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModSlot {
    pub flag: u8,
    pub item: Option<FormId>,
    pub effect: u32,
    pub value_a: f32,
    pub value_b: f32,
}

/// The mod effect that raises the weapon's most health
/// ("Increase Max. Condition").
pub const EFFECT_MAX_CONDITION: u32 = 10;

pub fn mod_slots(order: &LoadOrder, weapon: FormId) -> [ModSlot; 3] {
    let mut out = [1u8, 2, 4].map(|flag| ModSlot {
        flag,
        item: None,
        effect: 0,
        value_a: 0.0,
        value_b: 0.0,
    });
    let Some((plugin, rec)) = order
        .get(weapon)
        .filter(|r| r.entry.header.kind.as_bytes() == b"WEAP")
        .and_then(|r| r.record().ok().map(|rec| (r.plugin, rec)))
    else {
        return out;
    };
    for (i, sig) in [b"WMI1", b"WMI2", b"WMI3"].iter().enumerate() {
        out[i].item = rec
            .get(FourCC::new(sig))
            .filter(|s| s.data.len() >= 4)
            .map(|s| {
                plugin.to_global(FormId(u32::from_le_bytes([
                    s.data[0], s.data[1], s.data[2], s.data[3],
                ])))
            })
            .filter(|f| f.0 != 0);
    }
    if let Some(d) = rec.get(FourCC::new(b"DNAM")).map(|s| s.data.clone()) {
        let u32_at = |i: usize| {
            d.get(i..i + 4)
                .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        let f32_at = |i: usize| f32::from_bits(u32_at(i));
        for (i, slot) in out.iter_mut().enumerate() {
            slot.effect = u32_at(140 + 4 * i);
            slot.value_a = f32_at(152 + 4 * i);
            slot.value_b = f32_at(184 + 4 * i);
        }
    }
    out
}

/// The mods fitted to the player's weapon (flags 1, 2, 4).
pub fn fitted(state: &GameState, weapon: FormId) -> u8 {
    state
        .weapon_mods
        .get(&(PLAYER_REF, weapon))
        .copied()
        .unwrap_or(0)
}

/// Fits a mod item to the player's weapon: the first slot without its mod
/// whose mod item it is gets it, the mod item goes, "Weapon Modifications"
/// goes up. False when it doesn't fit. (The game splits one weapon off a
/// stack first; the world keeps one set of mods per carried weapon. A max
/// condition mod's added health, `004bcf60(10, 0)` on the weapon's health,
/// isn't kept: the world holds condition as a share of the record's.)
// Translated from 00783af0 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn fit_mod(order: &LoadOrder, state: &mut GameState, weapon: FormId, item: FormId) -> bool {
    if state.item_count(order, PLAYER_REF, weapon) <= 0
        || state.item_count(order, PLAYER_REF, item) <= 0
    {
        return false;
    }
    let have = fitted(state, weapon);
    let Some(slot) = mod_slots(order, weapon)
        .into_iter()
        .find(|s| have & s.flag == 0 && s.item == Some(item))
    else {
        return false;
    };
    state
        .weapon_mods
        .insert((PLAYER_REF, weapon), have | slot.flag);
    take(order, state, item, 1);
    crate::stats::bump(state, STAT_WEAPON_MODS, 1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `00648090` with the exe's settings: a half-skilled player mending a
    /// 50% gun with another 50% one gets 65%; the better condition counts
    /// whole, the worse at `fRepairScavengeMult`.
    #[test]
    fn the_repair_formula_is_the_exes() {
        let s = RepairSettings::default();
        let (c, needed) = repaired_condition(&s, 50, 50.0, 50.0);
        assert!((c - 65.0).abs() < 1e-4, "{c}");
        assert_eq!(needed, None);
        // Order doesn't matter; the better one counts whole.
        let (c1, _) = repaired_condition(&s, 0, 20.0, 80.0);
        let (c2, _) = repaired_condition(&s, 0, 80.0, 20.0);
        assert_eq!(c1, c2);
        assert!((c1 - (8.0 + 0.5 + 2.0 * 0.05) * 10.0).abs() < 1e-4);
        // Full skill: + 1.5 tenths.
        let (c, _) = repaired_condition(&s, 100, 0.0, 0.0);
        assert!((c - 20.0).abs() < 1e-4);
        // Settings making it worse than the better item: the skill needed.
        let bad = RepairSettings { min: -5.0, ..s };
        let (_, needed) = repaired_condition(&bad, 0, 60.0, 60.0);
        assert_eq!(needed, Some(((6.0f64 + 0.1 - 4.0) * 100.0 / 5.0) as i32));
    }
}

//! Blocking and power attacks, as `FalloutNV.exe` 1.4.0.525 does them.
//!
//! **Blocking.** The Aim control (6, "Block" in the INI) with a melee
//! weapon or fists out starts the block (`0093e860` → `00894cc0(1)`): the
//! process's block flag (+0xf0) is set and, for the player, `00894940`
//! plays the weapon's `BlockIdle` group (0xaa) as anim action 7; letting go
//! clears it. Someone is blocking while their anim action is 7
//! (`00894d60`). A hit on someone blocking (`009b5a30`, the hit's damage
//! through armour) adds to their damage threshold when the attacker (or the
//! projectile's shooter) is within their hit cone ([`in_hit_cone`],
//! `009a6ae0`): `fBlockSkillBase` + skill × `fBlockSkillMult`
//! ([`block_threshold`], `006463a0`), the skill Melee Weapons when both
//! have a weapon and the blocker's is a one- or two-handed melee weapon,
//! else Unarmed when the attacker has none or the blocker's is a
//! hand-to-hand weapon, else 0; nothing when blocking with fists against a
//! gun or a projectile. The hit is flagged blocked (`00407e00(1, 1)`) and
//! the player's counter-attack timer (+0xe28) is set to
//! `fCounterAttackTimer`.
//!
//! **Power attacks** (`00948310`, the Attack control): an attack starts on
//! the press; while the control stays held a timer (`011e07b0`) counts the
//! frame's seconds, and past `fPowerAttackDelay` (0.3) a power attack
//! follows ([`power_attack_group`]): `AttackPower` (0x5c) sneaking or with
//! both legs crippled, else by the movement keys `AttackForwardPower`
//! (0x5d), `AttackBackPower` (0x5e), `AttackLeftPower` (0x5f),
//! `AttackRightPower` (0x60), forward first; unarmed perks replace them by
//! the custom power attacks (entry points 61–66), and an unarmed attack
//! within the counter-attack timer with entry point 64 is the `Counter`
//! (0xa8). The power attack's damage is × `fDamagePowerAttackBonus`
//! unless the attacker sneaks (`009b5170`, `00644520`).

use esm::{FormId, LoadOrder};

use crate::combat::{av, Weapon};
use crate::perks;
use crate::scripting::{game_setting, Facts, GameState};

/// The animation groups the player's melee attacks use (the table at
/// `011977d8`).
pub mod group {
    pub const ATTACK_RIGHT: u8 = 0x20;
    pub const ATTACK_POWER: u8 = 0x5c;
    pub const ATTACK_FORWARD_POWER: u8 = 0x5d;
    pub const ATTACK_BACK_POWER: u8 = 0x5e;
    pub const ATTACK_LEFT_POWER: u8 = 0x5f;
    pub const ATTACK_RIGHT_POWER: u8 = 0x60;
    pub const ATTACK_CUSTOM1_POWER: u8 = 0x61;
    pub const ATTACK_CUSTOM2_POWER: u8 = 0x62;
    pub const ATTACK_CUSTOM3_POWER: u8 = 0x63;
    pub const ATTACK_CUSTOM4_POWER: u8 = 0x64;
    pub const ATTACK_CUSTOM5_POWER: u8 = 0x65;
    pub const COUNTER: u8 = 0xa8;
    pub const BLOCK_IDLE: u8 = 0xaa;
    pub const BLOCK_HIT: u8 = 0xab;
}

/// The unarmed perks' entry points `00948310` asks (`005e58f0(0x3d…0x42)`;
/// the exe's names "Has Unarmed Forward/Back/Crouched/Left/Right Power
/// Attack").
pub mod entry {
    pub const UNARMED_FORWARD_POWER: u8 = 61;
    pub const UNARMED_BACK_POWER: u8 = 62;
    pub const UNARMED_CROUCHED_POWER: u8 = 63;
    pub const UNARMED_COUNTER: u8 = 64;
    pub const UNARMED_LEFT_POWER: u8 = 65;
    pub const UNARMED_RIGHT_POWER: u8 = 66;
}

/// The settings blocking and power attacks read (game settings; the exe's
/// defaults when the data doesn't set them).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// `fBlockSkillBase` (exe 0, `FalloutNV.esm` 5) and `fBlockSkillMult`
    /// (exe 1, data 0.3).
    pub block_skill_base: f32,
    pub block_skill_mult: f32,
    /// `fCombatHitConeAngle` (35 degrees).
    pub hit_cone: f32,
    /// `fCounterAttackTimer` (1 s).
    pub counter_attack_time: f32,
    /// `fPowerAttackDelay` (0.3 s).
    pub power_attack_delay: f32,
    /// `fDamagePowerAttackBonus` (exe 3, data 2).
    pub power_attack_bonus: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            block_skill_base: 0.0,
            block_skill_mult: 1.0,
            hit_cone: 35.0,
            counter_attack_time: 1.0,
            power_attack_delay: 0.3,
            power_attack_bonus: 3.0,
        }
    }
}

impl Settings {
    pub fn read(order: &LoadOrder) -> Settings {
        let d = Settings::default();
        let g = |n: &str, v: f32| game_setting(order, n).unwrap_or(v);
        Settings {
            block_skill_base: g("fBlockSkillBase", d.block_skill_base),
            block_skill_mult: g("fBlockSkillMult", d.block_skill_mult),
            hit_cone: g("fCombatHitConeAngle", d.hit_cone),
            counter_attack_time: g("fCounterAttackTimer", d.counter_attack_time),
            power_attack_delay: g("fPowerAttackDelay", d.power_attack_delay),
            power_attack_bonus: g("fDamagePowerAttackBonus", d.power_attack_bonus),
        }
    }
}

/// Whether `attacker_at` is within the hit cone of someone at `at` facing
/// `heading` (radians clockwise from north): the angle between the heading
/// and the direction to the attacker, in degrees, at most `cone` (× 3
/// within 128 units).
// Translated from 009a6ae0 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn in_hit_cone(at: [f32; 3], heading: f32, attacker_at: [f32; 3], cone: f32) -> bool {
    let d = [attacker_at[0] - at[0], attacker_at[1] - at[1]];
    let to_attacker = d[0].atan2(d[1]);
    // `00408860` (fabs) of the difference, in degrees, folded past 180.
    let mut angle = ((heading - to_attacker) * 57.29578).abs();
    if angle > 180.0 {
        angle = (angle - 360.0).abs();
    }
    // The distance (`00457990`, the whole vector's length) under 128.
    let dz = attacker_at[2] - at[2];
    let distance = (d[0] * d[0] + d[1] * d[1] + dz * dz).sqrt();
    let cone = if distance < 128.0 { cone * 3.0 } else { cone };
    angle <= cone
}

/// What a block adds to the blocker's damage threshold against one hit
/// (`009b5a30`, `006463a0`): `defender_weapon` the blocker's weapon
/// (`None`: fists), `attacker_weapon` the attacker's (`None`: unarmed);
/// `projectile` when the hit came from a projectile or an explosion.
// Translated from 009b5a30 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn block_threshold(
    order: &LoadOrder,
    state: &GameState,
    s: &Settings,
    defender: FormId,
    defender_weapon: Option<&Weapon>,
    attacker_weapon: Option<&Weapon>,
    projectile: bool,
) -> f32 {
    // Fists against a gun or a projectile block nothing.
    let fists = defender_weapon.is_none();
    if fists && (attacker_weapon.is_some_and(|w| !w.is_melee()) || projectile) {
        return 0.0;
    }
    let skill = |a: u16| {
        Facts {
            order,
            state,
            speaker: None,
        }
        .current_actor_value(defender, a)
        .unwrap_or(0.0) as f32
    };
    let skill = match (attacker_weapon, defender_weapon) {
        (Some(_), Some(d)) if d.animation == 1 || d.animation == 2 => skill(av::MELEE_WEAPONS),
        (None, _) => skill(av::UNARMED),
        (Some(_), Some(d)) if d.animation == 0 => skill(av::UNARMED),
        _ => 0.0,
    };
    s.block_skill_base + skill * s.block_skill_mult
}

/// What `target`'s block adds to their threshold against `attacker`'s hit
/// with `weapon` (`None`: unarmed), if they're blocking
/// (`GameState::blocking`) and the attacker is in their hit cone: see
/// [`block_threshold`]. A gun's shot counts as a projectile. Hits with no
/// attacker (no position to test the cone against) aren't blocked here.
pub fn block_bonus(
    order: &LoadOrder,
    state: &GameState,
    attacker: FormId,
    weapon: Option<FormId>,
    target: FormId,
) -> Option<f32> {
    let &heading = state.blocking.get(&target)?;
    let (_, _, at, _) = state.place(order, target)?;
    let (_, _, from, _) = state.place(order, attacker)?;
    let s = Settings::read(order);
    if !in_hit_cone(at, heading, from, s.hit_cone) {
        return None;
    }
    let attacker_weapon = weapon.and_then(|w| Weapon::load(order, w));
    let defender_weapon = crate::combat::weapon_in_hand(order, state, target);
    let projectile = attacker_weapon.as_ref().is_some_and(|w| !w.is_melee());
    Some(block_threshold(
        order,
        state,
        &s,
        target,
        defender_weapon.as_ref(),
        attacker_weapon.as_ref(),
        projectile,
    ))
}

/// Which movement keys are held (the movement flags' low bits).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Moving {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}

/// The power attack's group (`00948310`, after the hold passes
/// `fPowerAttackDelay`): see the module notes. `unarmed`: no weapon or an
/// Unarmed-skill weapon (the perks' moves are asked only then);
/// `legs_crippled`: both legs' condition (actor values 29, 30) at most 0.
// Translated from 00948310 (decompiled, FalloutNV.exe 1.4.0.525)
pub fn power_attack_group(
    order: &LoadOrder,
    state: &GameState,
    moving: Moving,
    sneaking: bool,
    legs_crippled: bool,
    unarmed: bool,
) -> u8 {
    let has = |e: u8| unarmed && perks::apply(order, state, e, 0.0) > 0.0;
    if sneaking || legs_crippled {
        return if has(entry::UNARMED_CROUCHED_POWER) {
            group::ATTACK_CUSTOM3_POWER
        } else {
            group::ATTACK_POWER
        };
    }
    let (g, custom, e) = if moving.forward {
        (
            group::ATTACK_FORWARD_POWER,
            group::ATTACK_CUSTOM1_POWER,
            entry::UNARMED_FORWARD_POWER,
        )
    } else if moving.back {
        (
            group::ATTACK_BACK_POWER,
            group::ATTACK_CUSTOM2_POWER,
            entry::UNARMED_BACK_POWER,
        )
    } else if moving.left {
        (
            group::ATTACK_LEFT_POWER,
            group::ATTACK_CUSTOM4_POWER,
            entry::UNARMED_LEFT_POWER,
        )
    } else if moving.right {
        (
            group::ATTACK_RIGHT_POWER,
            group::ATTACK_CUSTOM5_POWER,
            entry::UNARMED_RIGHT_POWER,
        )
    } else {
        return group::ATTACK_POWER;
    };
    if has(e) {
        custom
    } else {
        g
    }
}

/// An unarmed attack within the counter-attack timer becomes the `Counter`
/// (0xa8) with entry point 64 (`00948310`).
pub fn counter_attack(
    order: &LoadOrder,
    state: &GameState,
    unarmed: bool,
    counter_timer: f32,
) -> bool {
    unarmed && counter_timer > 0.0 && perks::apply(order, state, entry::UNARMED_COUNTER, 0.0) > 0.0
}

/// A power attack's damage multiplier (`009b5170`: `00644520` returns
/// `fDamagePowerAttackBonus`): only when not sneaking.
pub fn power_attack_mult(s: &Settings, power: bool, sneaking: bool) -> f32 {
    if power && !sneaking {
        s.power_attack_bonus
    } else {
        1.0
    }
}

/// The animation group's file stem in `_1stperson` / `_male` (`AttackPower`
/// → `attackpower`), for the weapon kinds' files (`1hmattackpower.kf`).
pub fn group_file_stem(g: u8) -> Option<String> {
    crate::animation::GROUPS
        .get(usize::from(g))
        .map(|(name, _, _)| name.to_ascii_lowercase())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn the_hit_cone_faces_the_heading_and_widens_up_close() {
        // Facing north; an attacker 500 north is dead ahead, 500 east is
        // 90 degrees off.
        assert!(in_hit_cone([0.0; 3], 0.0, [0.0, 500.0, 0.0], 35.0));
        assert!(!in_hit_cone([0.0; 3], 0.0, [500.0, 0.0, 0.0], 35.0));
        // 40 degrees off at 500: outside; at 100 units: inside (35 × 3).
        let off = 40f32.to_radians();
        assert!(!in_hit_cone(
            [0.0; 3],
            0.0,
            [500.0 * off.sin(), 500.0 * off.cos(), 0.0],
            35.0
        ));
        assert!(in_hit_cone(
            [0.0; 3],
            0.0,
            [100.0 * off.sin(), 100.0 * off.cos(), 0.0],
            35.0
        ));
        // Headings either side of north fold through 360.
        assert!(in_hit_cone([0.0; 3], 6.2, [0.0, 500.0, 0.0], 35.0));
    }

    pub(crate) fn weapon(animation: u32) -> Weapon {
        Weapon {
            form_id: FormId(0x10),
            name: String::new(),
            damage: 10.0,
            clip: 1,
            health: 100,
            animation,
            ammo: Vec::new(),
            ammo_use: 1,
            min_spread: 0.0,
            spread: 0.0,
            projectile: None,
            projectiles: 1,
            min_range: 0.0,
            max_range: 0.0,
            shots_per_second: 1.0,
            reload_time: 0.0,
            skill: 38,
            crit_damage: 0.0,
            crit_mult: 1.0,
            sound: None,
            attack_animation: 255,
            reload_animation: 255,
            kill_impulse: 0.0,
            impulse_distance: 0.0,
            reach: 1.0,
            limb_damage_mult: 1.0,
            flags1: 0,
            flags2: 0,
            fire_rate: 0.0,
            attack_mult: 1.0,
            aim_arc: 0.0,
            semi_auto_delay: (0.0, 0.0),
            speed: 1.0,
            cone_mult: 1.0,
            crit_effect: None,
            crit_on_death: false,
            resist: None,
        }
    }

    #[test]
    fn a_block_adds_the_blocking_skill_to_the_threshold() {
        let order = LoadOrder::from_plugins(Vec::new()).unwrap();
        let state = GameState::default();
        let s = Settings {
            block_skill_base: 5.0,
            block_skill_mult: 0.3,
            ..Settings::default()
        };
        let who = FormId(0x14);
        let skill = |a: u16| {
            Facts {
                order: &order,
                state: &state,
                speaker: None,
            }
            .current_actor_value(who, a)
            .unwrap_or(0.0) as f32
        };
        let melee = skill(av::MELEE_WEAPONS);
        let unarmed = skill(av::UNARMED);
        let machete = weapon(1);
        let pistol = weapon(3);
        let knuckles = weapon(0);
        // A machete blocking a machete or a pistol: Melee Weapons.
        let t = block_threshold(
            &order,
            &state,
            &s,
            who,
            Some(&machete),
            Some(&machete),
            false,
        );
        assert_eq!(t, 5.0 + melee * 0.3);
        let t = block_threshold(&order, &state, &s, who, Some(&machete), Some(&pistol), true);
        assert_eq!(t, 5.0 + melee * 0.3);
        // Anything blocking a punch: Unarmed.
        let t = block_threshold(&order, &state, &s, who, Some(&machete), None, false);
        assert_eq!(t, 5.0 + unarmed * 0.3);
        // Brass knuckles blocking a machete: Unarmed.
        let t = block_threshold(
            &order,
            &state,
            &s,
            who,
            Some(&knuckles),
            Some(&machete),
            false,
        );
        assert_eq!(t, 5.0 + unarmed * 0.3);
        // Fists blocking a machete: no skill, the base.
        assert_eq!(
            block_threshold(&order, &state, &s, who, None, Some(&machete), false),
            5.0
        );
        // Fists against a gun or a projectile: nothing.
        assert_eq!(
            block_threshold(&order, &state, &s, who, None, Some(&pistol), true),
            0.0
        );
        assert_eq!(
            block_threshold(&order, &state, &s, who, None, None, true),
            0.0
        );
    }

    #[test]
    fn power_attacks_go_the_way_the_player_moves() {
        let order = LoadOrder::from_plugins(Vec::new()).unwrap();
        let state = GameState::default();
        let still = Moving::default();
        let g = |m: Moving, sneak: bool, legs: bool| {
            power_attack_group(&order, &state, m, sneak, legs, true)
        };
        assert_eq!(g(still, false, false), group::ATTACK_POWER);
        let forward = Moving {
            forward: true,
            left: true,
            ..still
        };
        assert_eq!(g(forward, false, false), group::ATTACK_FORWARD_POWER);
        let back = Moving {
            back: true,
            ..still
        };
        assert_eq!(g(back, false, false), group::ATTACK_BACK_POWER);
        let left = Moving {
            left: true,
            ..still
        };
        assert_eq!(g(left, false, false), group::ATTACK_LEFT_POWER);
        let right = Moving {
            right: true,
            ..still
        };
        assert_eq!(g(right, false, false), group::ATTACK_RIGHT_POWER);
        // Sneaking or both legs crippled: the plain one.
        assert_eq!(g(forward, true, false), group::ATTACK_POWER);
        assert_eq!(g(forward, false, true), group::ATTACK_POWER);
        assert_eq!(
            group_file_stem(group::ATTACK_LEFT_POWER).as_deref(),
            Some("attackleftpower")
        );
        assert_eq!(
            group_file_stem(group::BLOCK_IDLE).as_deref(),
            Some("blockidle")
        );
        // The damage bonus, not while sneaking.
        let s = Settings {
            power_attack_bonus: 2.0,
            ..Settings::default()
        };
        assert_eq!(power_attack_mult(&s, true, false), 2.0);
        assert_eq!(power_attack_mult(&s, true, true), 1.0);
        assert_eq!(power_attack_mult(&s, false, false), 1.0);
        assert!(!counter_attack(&order, &state, true, 0.5));
    }
}

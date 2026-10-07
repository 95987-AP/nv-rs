//! Combat groups (`CombatGroup`, `CombatManager` (Xbox PDB)): those who
//! fight together, as `GetGroupMemberCount` and `GetGroupTargetCount`
//! count them.
//!
//! Read from the game's code (FalloutNV.exe 1.4.0.525, the Xbox 360
//! prototype's names):
//!
//! - Someone who starts a fight on their own makes a group of their own
//!   with their target as its first target (`Actor::StartCombat` →
//!   `CombatManager::AddCombatant`: a new group, the actor its member).
//! - Someone who takes a friend's side joins the friend's group: the
//!   detection run (`008ff350`) sees the friend fighting a target it also
//!   detects (1 or more) and wants to help it (`00992530`, the
//!   [`crate::combat_ai::helps`] rule), and queues a start of combat with
//!   the friend's group (`CombatManager::AddGroupMember`); someone already
//!   fighting in another group merges it into the friend's
//!   (`CombatGroup::MergeGroup`: every member moves). This runs whether
//!   or not the helper is fighting already, while the two groups differ.
//! - The player's group (`PlayerCharacter` +0xd64, `0093a690`) is made
//!   when someone starts fighting the player, with the player as its
//!   member and the attacker as a target, and dropped when it has no
//!   targets left (`009444d0`); those who help the player join it as
//!   above (teammates move with the player when the group is split,
//!   `CombatManager::SplitPlayerCombatGroup`, `00991f80`).
//! - A member leaves its group as it stops fighting (the combat
//!   controller goes, `CombatGroup::RemoveMember`).
//!
//! The counts (`005a4240`, `005a42b0`): the members and the targets of
//! the actor's group (actor vtable +0x3f8, through its combat controller,
//! the player's own pointer); 0 for someone not fighting. A group's
//! targets here are its members' targets (`GameState::combat` and the
//! extra targets in `GameState::hit_targets`), and for the player's group
//! everyone fighting the player.

use std::collections::HashMap;

use esm::{FormId, LoadOrder};

use crate::dialogue::PLAYER_REF;
use crate::scripting::GameState;

/// Who joined whose group (member → the group, named by the one who made
/// it). Those who made their own aren't listed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombatGroups {
    pub joined: HashMap<FormId, FormId>,
}

/// Whether someone is fighting, so in a group: alive with a combat
/// target; the player while someone alive fights them.
pub fn fighting(state: &GameState, who: FormId) -> bool {
    if state.dead.contains(&who) {
        return false;
    }
    if who == PLAYER_REF {
        return state
            .combat
            .iter()
            .any(|(w, t)| *t == PLAYER_REF && *w != PLAYER_REF && !state.dead.contains(w));
    }
    state.combat.contains_key(&who)
}

/// The group someone fights in (named by the one who made it), if
/// they're fighting.
pub fn group_of(state: &GameState, who: FormId) -> Option<FormId> {
    if !fighting(state, who) {
        return None;
    }
    Some(state.combat_groups.joined.get(&who).copied().unwrap_or(who))
}

/// Everyone fighting in someone's group, sorted (none when they aren't
/// fighting).
pub fn members(state: &GameState, who: FormId) -> Vec<FormId> {
    let Some(group) = group_of(state, who) else {
        return Vec::new();
    };
    let mut all: Vec<FormId> = state
        .combat
        .keys()
        .copied()
        .chain(std::iter::once(PLAYER_REF))
        .filter(|&m| group_of(state, m) == Some(group))
        .collect();
    all.sort();
    all.dedup();
    all
}

/// The targets of someone's group, sorted: its members' targets, alive;
/// for a group the player is in, everyone fighting the player too.
pub fn targets(state: &GameState, who: FormId) -> Vec<FormId> {
    let members = members(state, who);
    let mut all = Vec::new();
    for &m in &members {
        if let Some(&t) = state.combat.get(&m) {
            all.push(t);
        }
        if let Some(list) = state.hit_targets.get(&m) {
            all.extend(list.iter().copied());
        }
        if m == PLAYER_REF {
            all.extend(
                state
                    .combat
                    .iter()
                    .filter(|(_, t)| **t == PLAYER_REF)
                    .map(|(w, _)| *w),
            );
        }
    }
    all.retain(|t| !state.dead.contains(t) && !members.contains(t));
    all.sort();
    all.dedup();
    all
}

/// `GetGroupMemberCount` (`005a4240`).
pub fn member_count(state: &GameState, who: FormId) -> usize {
    members(state, who).len()
}

/// `GetGroupTargetCount` (`005a42b0`).
pub fn target_count(state: &GameState, who: FormId) -> usize {
    targets(state, who).len()
}

/// Whether `helper` takes the side of `friend`'s group (`008ff350`):
/// `helper` sees `friend` (`friend_value` above 0), who is fighting a
/// target `helper` detects at 1 or more (`enemy_value`); their groups
/// differ, the friend's has targets, and `helper` would help `friend`
/// ([`crate::combat_ai::helps`]).
pub fn helps_group(
    order: &LoadOrder,
    state: &GameState,
    helper: FormId,
    friend: FormId,
    friend_value: i32,
    enemy_value: impl FnOnce(FormId) -> Option<i32>,
) -> bool {
    if friend_value <= 0 || helper == friend || state.dead.contains(&helper) {
        return false;
    }
    let Some(theirs) = group_of(state, friend) else {
        return false;
    };
    if group_of(state, helper) == Some(theirs) || target_count(state, friend) == 0 {
        return false;
    }
    let Some(&enemy) = state.combat.get(&friend) else {
        return false;
    };
    if enemy == helper || !crate::combat_ai::helps(order, state, helper, friend) {
        return false;
    }
    enemy_value(enemy).is_some_and(|v| v >= 1)
}

/// `helper` joins `friend`'s group (`CombatManager::AddGroupMember`); if
/// already fighting in a group of its own, every member of that group
/// moves (`CombatGroup::MergeGroup`). Nothing when `friend` isn't
/// fighting.
pub fn join(state: &mut GameState, helper: FormId, friend: FormId) {
    let Some(theirs) = group_of(state, friend) else {
        return;
    };
    let moving = match group_of(state, helper) {
        Some(mine) if mine == theirs => return,
        Some(_) => members(state, helper),
        None => vec![helper],
    };
    for m in moving {
        if m == theirs {
            state.combat_groups.joined.remove(&m);
        } else {
            state.combat_groups.joined.insert(m, theirs);
        }
    }
}

/// Those who stopped fighting leave the group they joined.
pub fn tidy(state: &mut GameState) {
    let gone: Vec<FormId> = state
        .combat_groups
        .joined
        .keys()
        .copied()
        .filter(|&m| !fighting(state, m))
        .collect();
    for m in gone {
        state.combat_groups.joined.remove(&m);
    }
}

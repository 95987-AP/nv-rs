# NPC combat: weapon choice, ammunition, reloads, fight events

Batch `claude/m2-npc-combat`, 2026-10-06, for Ghost Town Gunfight
(`VMS16`/`VMS16b`). Code: `crates/world/src/npc_combat.rs` (rules),
`viewer/src/fighting.rs` and `viewer/src/ai.rs` (calls), `GetShouldAttack`
and `SetUnconscious` in `crates/world/src/scripting.rs`. Status:
**implemented and tested on generated data; not compared with the
original game.** Executable: `FalloutNV.exe` 1.4.0.525; names marked
(Xbox PDB) come from the Xbox 360 prototype's symbols.

## What the game does (traced)

### Weapon rating and the arsenal

- `00646060` `CombatFormulas::CalcCombatWeaponDPS` (Xbox PDB): 0 for
  weapons flagged "not used in normal combat" (`DNAM` flags2 0x40,
  `00646230`); otherwise `00645380` (below); grenades ÷ 10, mines 0
  (`00522c80` kinds 3/4); the actor's combat style weapon restriction
  (`TESCombatStyle` +0x40 on PC = `cWeaponRestrictions` (Xbox PDB), CSSD
  u32 40): 1 → guns (animation 3–13, `004c0c30`) ÷ 10000, 2 → melee and
  fists ÷ 10000. Constants: `01020758` 10.0, `0102f078` 10000.0.
- `00645380` `CombatFormulas::CalcWeaponDamagePerSecond` (Xbox PDB), as the
  controller calls it (no mods, no perks, own skill, own critical
  chance), read from its disassembly:
  - damage = `DATA` damage × `fDamageWeaponMult`, ammunition damage
    effects (`0059a030`: multiplying effects first), + projectiles ×
    explosion damage when the projectile explodes (`00525b20`,
    explosion +0x78); grenades and mines (animation 10/11) use the
    explosion damage × `fDamageWeaponMult` instead and no jam index;
  - × (`fDamageSkillBase` + `fDamageSkillMult` × skill ÷ 100);
  - rounds = the clip (`BGSClipRounds`) for animations 3–9 and 12, else 1;
  - time = rounds ÷ (`DNAM` 88 attack shots/s + rounds × jam chance ×
    `DNAM` 96 jam time), jam chance `fWeaponConditionJam1…10` at index
    round-half-up(condition × 10) (`004bd510`, table `0119b26c`,
    `006477b0`); + `DNAM` 92 reload time when rounds ≤
    `iMinClipSizeToAddReloadDelay` (2) and not melee/grenade/mine/thrown;
  - not the player, a gun: semi-automatic (flags1 0x02 clear) adds rounds
    × (average semi-auto delay − 1 ÷ shots/s) when positive; automatic or
    short burst (flags2 0x200) × (`fAutomaticWeaponBurstFireTime` +
    `…CooldownTime`) ÷ cooldown;
  - critical share = clamp(Critical Chance AV 14 × `CRDT` mult, 0, 100) ×
    0.01 (÷ shots/s for automatics) × `CRDT` damage;
  - condition multiplier `00646d00` (1 above 0.75, else 1 − (0.75 − c) ×
    0.67);
  - DPS = (critical share + damage) × rounds ÷ time; 0 at condition 0;
  - fists: `fUnarmedDamageMult` × AV 56 + `fDamageSkillMult` × AV 45 ÷ 100
    over `fUnarmedNPCDPSMult` (1.57), a creature's own damage over
    `fUnarmedCreatureDPSMult`.
- `00522c80` `TESObjectWEAP::GetCombatWeaponType` (Xbox PDB): animation
  0–2 melee (2); 3–9 and 13 ranged (1), or ranged explosive (0) when the
  projectile's explosion damage > `fDangerousProjectileExplosionDamage` (5)
  and radius > `fDangerousProjectileExplosionRadius` (30); 10 grenade (3);
  11–12 mine (4).
- `009993c0` (`CombatState.cpp` per its assert path; the Xbox
  `CombatState::UpdateWeapons`): candidates are inventory weapons that the
  actor can use (`008bc9d0`: not "player only", flags2 0x1) with ammunition
  carried (`00525980`, `009962f0`, `004c7300`) or none needed; DPS ≤ 0
  drops one. Guns (kinds 0/1) with a target: absolute maximum range
  (`009a9180`), × `fCombatCurrentWeaponAbsoluteMaxRangeMult` (1.5) for the
  weapon in hand; out of reach → set aside (score ÷ 10000, and the
  controller keeps the range at which it would reach, +0x68) unless no gun
  reaches, when the best of them stays. Per kind the best score
  (`+0x3c + kind×4`) and weapon (`+0xc + kind×4`); best DPS +0x58, best gun
  DPS +0x5c, best score +0x60; fists in slot 6 (+0x54). The embedded-weapon
  pick (+0x34, +1 000 000 bonus `0106b798`) is for creatures' embedded
  weapons (`fEmbeddedWeaponSwitchTime/Chance` in `00998a50`).
- Rebuilt at combat start (`00997c30`), when the controller's
  `fCombatInventoryUpdateTimer` (5 s, set at the end of `009993c0`) runs
  out or a flag (+0x77) asks (`00997cf0`, `00999160`), after a disarm
  (`0099a720`).
- Planner costs: action table `011a4280` (name, base cost): ranged
  explosive 1.5, ranged 1.5, grenade 1.5, melee 2.0, hand to hand 2.0,
  switch weapon 2.0, …; an attack action's cost (`0097ae00`, `0097b240`,
  `0097bb90`, `0097b4a0`, `0097b5a0`) = base + `0097ac30` = best score −
  the kind's score (FLT_MAX for a kind rated 0). Followers: ranged costs
  FLT_MAX with the global `CombatStyleMelee` = 1; hand to hand FLT_MAX
  when `008a1710` (not done here).

### Ammunition and reloads

- `008a8dd0` `Actor::ShouldUseAmmo` (Xbox PDB): the player; any actor with
  a weapon flagged flags2 0x2 ("NPCs use ammo", `008a8e30`; set on every
  weapon when `bForceNPCsUseAmmo:Combat`, `0051f456` → `0051f590`); a
  teammate (`00566950`) with a playable weapon (flags1 0x80 clear,
  `0047bcf0`). Vanilla: dynamite has 0x2 (flags2 0x10A); the varmint
  rifle (0x3008) and .357 (0x0C) don't.
- `008a89a0` `Actor::UseAmmo`: the clip loses ammo use × shots; the
  inventory too when `ShouldUseAmmo`; at 0 the reload/next-ammo handling
  and a controller notice (`0097f6d0`).
- `008a8420` `Actor::ReloadWeapon`: thrown weapons are their own
  ammunition; loads a full clip when not using ammo (or a clip's worth is
  carried), else what is carried; none carried → no reload.
- `009993c0` drops weapons without ammunition, so a dry weapon is
  replaced at the next arsenal.

### Fight events and script functions

- `OnStartCombat` (block `0118e6d8`, opcode 0x19, event mask 0x8000 in
  `005ca860`): raised by `00980830` (the controller's target set from none
  to one, the target's group detection level > 0) and by `009887b0` (the
  group first detecting a target), for each non-player group member, with
  its combat target (`005ac750`). `OnCombatEnd` is mask 0x40 (`005ca930`;
  not done).
- `GetShouldAttack` (command `01191360`, handler `005c74d0`, condition
  `0059ed30`): both must be actors (form types 0x3B/0x3C; the caller
  vtable +0x100); early 0 when the caller is in combat and `00992640`
  (combat group test) passes; else 100 when `008b06d0`
  (`factions::attacks_on_sight`) passes.
- `SetUnconscious` (command `01192eb8`, handler `005d0760`): 1 → actor
  vtable +0x434(0) (stop its fights; `StopCombatAlarmOnActor` uses the same
  slot per attacker), life state 3 (`008ace10` → `008a1800`), `0087faa0`,
  process +0x338; 0 → life state 0 only from 3.

## Implemented (with tests)

- `world::npc_combat`: `combat_weapon_type`, `should_use_ammo`,
  `condition_mult`, `jam_index`, `dps`/`damage_per_second`, `combat_dps`,
  `assemble`/`arsenal`, `cheapest_kind`, `choose_weapon` (5 s timer, equip
  or fists), `fired` (clip, inventory, reload over `DNAM` 92 ÷
  `combat::reload_rate`, dry → re-choose), `reloading`,
  `start_combat_event`, `combat_over`. `combat::weapon_in_hand` honours a
  fists choice.
- Viewer: the fight frame chooses the weapon, holds fire while
  reloading, accounts each shot/throw, runs `OnStartCombat` after the
  target's first detection (fight memory seen > 0), resets per fight;
  unconscious actors are skipped.
- Scripts: `GetShouldAttack` evaluated; `SetUnconscious 1` ends the
  actor's fight.
- Tests: 9 unit tests (`npc_combat::tests`, formula cases from the varmint
  rifle's and dynamite's record values) and 7 integration tests
  (`crates/world/tests/npc_combat.rs`, generated `testdata::fighting` now
  with a round, a rifle, dynamite and a raider script).

## Inferred or not done (labelled in code)

- **The planner**: the cheapest single attack action decides the weapon
  kind; ties go to the first in action order. The A* search over the
  planner's world state (`0097ade0`/`00984390` preconditions and effects)
  isn't traced. With these costs gunmen carrying dynamite rarely throw it
  (its rating is ÷ 10); whether the game's planner throws more often
  (e.g. `COMBAT_ACTION_ATTACK_GRENADE_FLUSH_TARGET`) is **not compared**.
- Switching is immediate; `CombatProcedureSwitchWeapon` (`009da7c0`)
  waits for the equip animation. The weapon model in the hand isn't
  swapped in the viewer.
- Reload duration = `DNAM` 92 ÷ reload rate (as the player's); the reload
  animation and its length aren't played/read. The first clip is taken as
  full (or what's carried when ammunition is used up).
- Thrown weapons: no clip/reload model; each throw uses one when
  `ShouldUseAmmo`, and running out re-chooses.
- An ammunition's own projectile isn't used for the explosive test.
  Creatures don't choose (their own weapons and embedded-weapon switching
  untouched). The +0x68 recheck range is computed but doesn't trigger an
  early re-choose.
- `GetShouldAttack`'s combat-group early 0 and the vtable +0x344 call;
  followers' `FollowerSwitchAggressive` (as before).
- `OnStartCombat`'s "detected" is the viewer's detection of the target
  above 0 since the fight began; group detection sharing isn't modelled.
- Unconscious actors: frozen, no knock-down pose; the process flags
  `005d0760` changes aren't traced.
- Out of combat, which weapon an NPC holds is still `actor::best_weapon`'s
  guess.

## Next

Record a Goodsprings powder-ganger fight in the original game (weapon
switches, dynamite throws, reload pauses, VMS16b stage 110 on the
gangers' `OnStartCombat`) and compare.

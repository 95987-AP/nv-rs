# Weapon mods

A weapon has up to three mod slots; a mod (`IMOD`) fitted to one changes the
weapon. Until 2026-10-06 nv-rs kept no mods at all.

## Read from `FalloutNV.exe` 1.4.0.525 (names from the Xbox 360 PDB)

| What | Xbox 360 (PDB) | PC |
| --- | --- | --- |
| The slots' mod items | `TESObjectWEAP::GetModItem` | `004bd570` |
| A slot's effect | `TESObjectWEAP::GetModEffect` | `004bd880` |
| An effect's value | `TESObjectWEAP::GetModEffectValue` | `004bcf60` |
| The fitted slots | `ExtraDataList::GetWeaponModFlags` (extra 0x8D) | |
| A fitted mod has the effect | | `004bda70` |
| The fitted mod's value | | `004bd8d0` |
| Damage | | `00644ce0`, `00645380` |
| Clip | | `004fe160` |
| Spread, minimum spread | | `00524be0`, `00524b80` |
| Weight | | `004be380` |
| Attack speed | | `00646020` |
| Projectiles | | `00525b20` |
| V.A.T.S. to-hit | | `00647730` |
| Most condition | | `004bcf00` |
| Worth | | `004bd400` |
| Fitting one | | `00783af0` (from the mod menu, `007838a0`) |

- The record: `WMI1`–`WMI3` the slots' mod items; in `DNAM` (`OBJ_WEAP`)
  each slot's effect (`eModActionOne`–`Three`, 140, 144, 148), value (152,
  156, 160) and second value (184, 188, 192). `MWD1`–`MWD7` and `WNM1`–
  `WNM7` are the world and first-person models with the mods on.
- The effects (`WEAPON_MOD_EFFECT`): 1 damage, 2 clip, 3 spread, 4 weight,
  5 and 6 ammo regeneration (per shot, per second), 7 equip speed, 8 fire
  speed, 9 projectile speed, 10 most condition, 11 silence, 12 split beam,
  13 V.A.T.S. to-hit, 14 sights, 15 V.A.T.S. special attack.
- The fitted slots: a byte (1, 2, 4) on the weapon's extra data. A fitted
  mod has an effect when a fitted slot, looked at in the order 1, 2, 3, has
  it (`004bda70`); its value is that slot's (`004bd8d0`).
  `GetModEffectValue` takes the first slot with the effect, fitted or not.
- What they change, each only with the mod fitted: damage + the value,
  added with the melee or unarmed bonus before condition (`00644ce0`); clip
  + the value rounded to nearest (`004fe160`); spread and minimum spread −
  the value, at least 0; weight − the value; attack speed (`DNAM` 60) + the
  value; projectiles + the value rounded (split beam; `00523150` also scales
  each projectile's damage by its second value); the V.A.T.S. to-hit chance
  (`DNAM` 40) + the value rounded; most condition: (health + the value)
  truncated; worth: + each fitted mod's own value, before rounding to
  tenths (`004bd400`).
- Fitting one (`00783af0`): the first slot (1, 2, 3) not yet fitted whose
  item is the mod; a condition mod also adds its value to the weapon's
  health; misc stat 0x21 "Weapon Modifications" + 1. The mod menu then
  takes one of the mod from the player (`007838a0`) and plays
  `UIItemGunsSmallUp`.

## Here

`world::weapon_mods`: `slots`, `flags` (`GameState::weapon_mods`, saved
as `weaponmods` lines), `has_effect`, `bonus`, `value_of`, `modded` (what
`combat::weapon_in_hand` returns: clip, spread, attack speed, projectiles,
most condition), `weight_off` (in `GameState::inventory_weight`),
`worth_added` (in `barter::item_value`), `attach`, `fitting`; damage in
`combat::weapon_damage_at`; the V.A.T.S. to-hit in `vats::part_chance`.
Checks: `crates/world/tests/weapon_mods.rs`, `world::weapon_mods::tests`.

## Not done

- The mod menu (`ItemModMenu`, the Pip-Boy's Mod button) to fit them.
- Split beam's damage scale, projectile speed, silence, sights, the ammo
  regeneration and equip speed effects; the modded models.
- Mods are kept per holder and weapon (as its condition is), where the
  game splits a modded weapon off its stack.
- Repair menus' most condition doesn't count a condition mod yet.

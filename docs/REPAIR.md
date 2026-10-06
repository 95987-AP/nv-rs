# Repairing: merchants' repairs and the Pip-Boy's

The game repairs weapons and armour two ways: a merchant's repair
services (`ShowRepairMenu` in ten dialogue lines of the official master:
Raul, Samuel at the 188, Mick in Freeside, Old Lady Gibson, Calamity,
Major Knight, Dale Barton, Sato, and one on the player), and the Pip-Boy's
Repair button on ITEMS, which mends an item with another. Until 2026-10-06
nv-rs had neither.

## Read from `FalloutNV.exe` 1.4.0.525

| What | PC |
| --- | --- |
| `ShowRepairMenu` (on a person or creature, vtable +0x100) | `005d5200` → `00704690` → `007b7570(vendor)` |
| `RepairServicesMenu` (class 1058, vtable `01075db4`) | create `007b7570`, click `007b7d80`, pointer on `007b82f0`, off `007b8ae0` |
| Its list: filling, a line, the filter, the order | `007b8b30`, `007b7920`, `007b7b40`, `007b7c40` |
| A merchant's repair, the payment | `007b7f70`, `008924e0` → `004cb4b0` |
| A merchant's cost | `00648230` |
| `RepairMenu` (the Pip-Boy's, class 1035, `repair_menu.xml`) | create `007b5520`, fill `007b7020`, list `007b6aa0`, a line `007b57f0`, click `007b5b40`, sort `007b5950` |
| The Pip-Boy's repair | formula `00648090`, action `007b5d80` |
| What can mend what (Jury Rigging) | `0047bb50` (repair list `0047bac0`, `0047ba80`) |
| Whether ITEMS offers Repair | `00781860` |
| An NPC's skill (`cSkill` + `cOffset`, Xbox PDB `NPC_DATA`) | `00607850`, `005f0d00` |

Settings (exe default → the master's): `fRepairMax` 2, `fRepairMin` 0.5,
`fRepairScavengeMult` 0.05, `fRepairSkillBase` 4, `fRepairSkillMax` 9 → 10,
`fItemRepairCostMult` 1 → 2.

### Merchants

- Their Repair, s, is read as the menus read skills (`0066ef20`): the
  current value within 0..100, truncated. A person's skill is their record's
  value plus its offset (`DNAM`: 14 skills, then 14 offsets) unless the
  game works their stats out (`ACBS` 0x10): Mick's is 15 + 60 = 75.
- They mend to t = (`fRepairSkillBase` + (`fRepairSkillMax` −
  `fRepairSkillBase`) × s / 100) / 10, at most 1, of an item's full health
  (`007b7f70`): Mick to 85%.
- The cost (`00648230`): `fItemRepairCostMult` × value × (t × 10 −
  condition% / 10) / 10, rounded (halves up) and at least 1 when above
  0.5; otherwise nothing, and the line can't be repaired. Mick mending a 9mm
  pistol (value 100) at 30%: 2 × 100 × (8.5 − 3) / 10 = 110 caps. No
  Barter adjustment.
- The list (`007b7b40`): the player's weapons and armour with health; not
  caps, quest items or armour marked not playable; below `fRepairSkillMax`
  × 10 percent (100); armour only with a damage threshold or resistance.
  Each line (`007b7920`): its cost; bright (alpha 255, `_CanRepair`) when it
  costs something the player can pay, else 128; the meter condition × 0.6
  wide; the worn mark. Order (`007b7c40`): what can be paid for first; then
  the most health first, worn first on a tie; among those that can't be,
  the reverse.
- The labels: "`sRepairSkill`   s", "`sInventoryCaps`   n" ("n+" past
  `iCapsLimit`), Repair All "`sRepairAllItems`" with every line's cost,
  usable when that's above 0 and the player can pay it.
- The pointer on a line (`007b82f0`): the item's picture, its condition and
  figure (a weapon's damage, rounded; armour's DT, else DR, at its
  condition, truncated, `00646d40`: × (0.5 + c) at or below half); when t
  is above the condition (both rounded to thousandths) the repaired figures,
  "+n%" and "+n" (the differences, at least 0, rounded), "`sRepairCost`"
  bright when it can be paid; else "`sCantRepairPastMax`" with t in percent.
- A click (`007b7d80`): a line that can be paid for is mended and paid
  (`UIRepairWeapon`), the list filled again; one that can't,
  `UIVATSInsufficientAP`. Repair All mends every line that can be paid for
  and pays the total. The caps go to the vendor.

### The Pip-Boy

- The new condition (`00648090`, `007b57f0`, `007b5d80`): with a and b the
  two items' conditions in percent / 10, hi and lo the larger and smaller,
  min(1, (hi + `fRepairMin` + (`fRepairMax` − `fRepairMin`) × Repair / 100 +
  lo × `fRepairScavengeMult`) / 10). Repair doesn't cap it: the function
  works out the skill cap and never uses it, and its "skill needed" (only
  when the result is no better than hi, which `fRepairMin` 0.5 rules out)
  never shows.
- What mends an item (`0047bb50`): another of it, or something on its
  repair list (`REPL`); with Jury Rigging (perk entry point 48 "Has Jury
  Rigging"), for an item that has a repair list, anything of the same kind
  that isn't a quest item: weapons playable with the same skill and
  animation type, armour playable with the same weight class and equip
  type.
- ITEMS offers Repair (`00781860`) for a weapon or armour below 100%
  (rounded up) when the player has another of it, or something else that
  mends it that isn't worn.
- The list (`007b6aa0`): every one of each thing that mends it, each its own
  line, the chosen one first and marked; of the chosen kind a worn one
  isn't offered.
- Mending (`007b5d80`, `007b5b40`): the chosen item's new condition, one of
  the other gone, "Items Repaired" counted, `UIRepairWeapon`; the menu
  closes when the condition reaches 99% or nothing's left to use.

## Here

`world::repair` has the rules: `skill`, `mended` / `mended_condition`,
`service_target`, `service_cost`, `service_lists`, `service_lines` (the
list in the menu's order), `repair_by` and `pay`, `mends`, `can_repair`,
`parts`, `repair_with`, `shown_stat`. `ShowRepairMenu` raises
`Event::RepairServices(vendor)`; the viewer opens
`ui::menus::repair_services` (`game_menus::repair`), which asks the world
to repair and refills.

The Pip-Boy: ITEMS' Repair button (R) is usable on an item
`world::repair::can_repair` allows (`ItemLine::repairable`); pressing it
asks the game (`Action::OpenRepair`) for the screen's lines
(`ui::pipboy::gather::repair_input`) and opens `ui::pipboy::repair` over
ITEMS, drawn on the Pip-Boy's screen; Enter on a line asks for the repair
(`Action::Repair`: `world::repair::repair_with`), then the screen is filled
again or closes; E goes back to ITEMS.

Checks: `crates/world/tests/repair.rs` (the formulas, a merchant's list and
order, a repair paid to the vendor, the Pip-Boy's rules with a repair list
and Jury Rigging), `world::repair::tests` (an NPC's skill offset),
`ui::menus::repair_services::tests` (filling, the pointer's stats, clicks),
`ui::pipboy::repair::tests` (the lines' order, what each does, closing).
Seen in the viewer at Mick & Ralph's (`--open-menu
repair:FreesideMickREF`): Repair 75, mending to 85%; a 9mm pistol at 30%
for 110 caps; clicking it paid 110 caps and left it at 85%, "CANNOT
REPAIR PAST 85%". The Pip-Boy in Doc Mitchell's house with three 9mm
pistols at 30% and Repair 15 (`--pipboy items:0 --pipboy-keys r,down,enter`):
the screen lists the pistol in brackets and its two spares, "CHOOSE ITEM TO
REPAIR WITH"; on a spare "+9%" (3.875 → 38.75%) and DAM 6 → 7; Enter mends
it, one spare left.

## Not done

- The Pip-Boy's scroll knob turning with the repair list, the brackets'
  move onto the item, the mouse on the Pip-Boy's screen.
- An item's condition is kept per holder and kind, weapons only
  (`GameState::weapon_health`): a repair mends every one of a kind the
  player has, where the game mends one, and armour is always whole, so it's
  never listed.
- Weapon mods (a condition mod raising full health, `004bda70(10)`), the
  ammunition's and perk entry 0's part in the damage figure (`006450f0`).
- The dialogue topic handed back when the merchant's menu closes
  (`007b78e0`: `0061a2d0(5, 5)`).

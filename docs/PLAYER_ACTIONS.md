# Player actions: activation, pickups, sneaking, iron sights

M2 work, 2026-10-06, branch `claude/m2-player-actions`. What the player
does in the opening, Back in the Saddle and Ghost Town Gunfight beyond
walking and shooting: the crosshair prompt and E, taking things, sneaking
with its meter, and looking down the sights. Read from `FalloutNV.exe`
1.4.0.525 with Ghidra; (Xbox PDB) marks names from the Xbox 360
prototype's symbols.

Status words: **implemented** (code exists), **tested** (regression
test), **live** (seen working in the running viewer with real input, how
below), **compared** (checked against the running original game).
**Nothing here has been compared with the original game.**

## Activation and the Info panel

| Address | What | Status |
| --- | --- | --- |
| `00579280` | Action class by base form type and state: ACTI with a name 4, NPC 7 (sneaking 1, dead 2), CREA only with `ACBS` 0x200000 (`GetAllowPCDialogue`, `0047c790`), CONT 2, DOOR with a name 8, FURN `MNAM` 0x40000000 3 / 0x80000000 5, BOOK 6, LIGH carryable 1 else 4, TERM 4, PWAT 14, items 1; destroyed non-actors nothing | implemented, tested (`world::activation::action_class`) |
| `00f80050` | The words table at `011d5160` (`sTargetTypeTake` "Take" … `sTargetTypeRepair`) | implemented |
| `00775a00` | Info panel: crime → Pickpocket / Steal / teammate Talk; dead people Search; doors Pick / Close / Open; books "Take"; name `%s (%d)` with the count; load doors `%s %s %s` ("Door to Goodsprings"); lock line (`sBroken`, `[sLocked - sHUDUseKey]`, `[sLocked - <level>]`, `[Requires Key]`); `sEmpty` for empty containers and bodies; weight `%.1f` / value `%.0f` (`%.1f` under 1, `--` at 0) for ARMO, carried LIGH, WEAP, AMMO, MISC, ALCH, IMOD (weapons ≥ 10 × perk entry 73); crime colour on all lines | implemented, tested |
| `0076bfe0` | Info lines' layout: lock / Empty at y 65 centred; weight right-justified at w/2 − 25, value at w − 30, labels at 10 and w/2 + 5, y 60; `template_info_seperator` | implemented, tested (`ui::hud`) |
| `00586500` | `TESWorldSpace` +0x138: the map region (`RDAT` 4 / `RDMP`) holding the point, highest priority; the cell's `XCLR` regions first | implemented, tested (`world::region::location_name`) |
| `0070bc20` | Pick: sphere of `fActivatePickSphereRadius` 16 from the camera node along the view, `iActivatePickLength` 150 | radius applied to the viewer's bounds-based pick |

Live (2026-10-06, Doc's house, posted window messages + F12 reports):
"E) Sit / Ruined Couch", E → temporary third person, seated first
person, E on nothing → stood up; "E) Take / Stimpak" with `--`/`75`
and the separator, E → "Stimpak added" with the neutral Vault Boy;
"E) Open / Gun Case"; Prospector Saloon: "E) Talk / Sunny Smiles",
sneaking "E) Pickpocket" in the crime colour. Real-data listing of Doc's
house: every chair/couch Sit, beds Sleep, the front door "Door to
Goodsprings".

Follows the code where it may surprise: the `WG`/`VAL` labels are
left-justified template text with alpha 0 and the code never sets their
alpha (`0076dd03`…), so only the numbers show; compare with the game.

## Pickups (`004ce380`, `008adcf0`)

Taking an item from the world: "<name> added" / "<n> <name>(s) added"
(`sAddItemtoInventory`, `sPlural`) for ARMO, BOOK, LIGH, MISC, WEAP, AMMO,
KEYM, ALCH, COBJ, IMOD, CHIP, CCRD, CMNY, with the item's pickup sound
(own `YNAM`, else a weapon's by equip type, else `UIItemGenericUp`). Before,
items without `YNAM` played nothing (the playtest report) and the message
was the viewer's own. Live: "Stimpak added" (the sound is logged as
"Pick-up sound: …"). Critical hits by the player: "Critical Strike on
<name>" / "Sneak Attack Critical on <name>" with the very happy Vault Boy
(`0089a760`), implemented, not seen live.

## Sneaking

| Address | What | Status |
| --- | --- | --- |
| `0093e860` @ `00940d5b` | Sneak control (8, Left Ctrl) toggles the mover's 0x400; not dead, not in furniture, not in anim actions 0, 1, 8, 10–16 (table `0094423c`); `NPCHumanCrouchDown`/`Up`; out of the sights | implemented, tested, live |
| `00952ff0`, `0094e1d0` | First-person eye is the 1st-person skeleton's `Camera1st`: 118 in `mtidle.kf`, 78 in `sneakmtidle.kf` (read from the files); the 40 drop blends over `fAnimationDefaultBlend` (the linear eye follow is this viewer's reading) | implemented, tested, live (lower view) |
| `007732d0`, `00770430` | Sneak meter: in combat [DANGER] (red, three flashes, `00a07c60` mode 2 / `00a080d0`) or [CAUTION] when all fighters search (`fCombatDetectionLostTime`); hostile detection [CAUTION]; else highest detection (`00973710`) < 1 [HIDDEN] else [DETECTED]; fades in/out 0.5 s | implemented, tested, live ([HIDDEN]) |
| `005f38d0` | Animation move-type prefix "Sneak": third-person body plays `sneakmt*.kf` (falling back to plain groups is assumed) | implemented, not confirmed live |
| speed | `fMoveSneakMult` through `world::locomotion` (existing) | implemented |

Hostile detection's own test (`008b06d0`) isn't followed: "attacks on
sight" stands for it.

## Iron sights (`world::iron_sights`)

| Address | What | Status |
| --- | --- | --- |
| `0093e860` @ `00941f4f` | Aim control (`Block=00380110`: right mouse, Left Alt) held → sights up for a drawn gun; released → down; not while switching view, in V.A.T.S.; melee/fists would block (not here) | implemented, tested, live |
| `0095de30` | FOV eased at 30 × dt ÷ `fIronSightsFOVTimeChange` to the weapon's sight FOV (`DNAM` +28) and its share of the 1st-person FOV; `fIronSightsZoomDefault` with none; `bIronSightsZoomEnable` | implemented, tested, live (zoom) |
| `008bb650` | Weapon groups +3: `<kind>aimis.kf`, `<kind>attack…is.kf`, blended in over 0.2 s (model-space mix: viewer simplification) | implemented, live |
| `008bbbf0`, `00771700`, `00874c10` | `bTrueIronSights`: `##SightingNode` kept; crosshair hidden in first person; the first-person camera at the node | implemented, live (varmint rifle: front post centred in the peep ring, no crosshair) |
| `0093e860` | No running while aiming (`locomotion::may_run`) | implemented |

The player's shot cone stays the weapon's min spread (`findings\hits.md`:
the aim term only feeds the sway, which isn't here).

## Controls and other actions

`viewer::controls` reads `[Controls]` (this install's values as
defaults). Walking, the mouse always looks (pointer held in the window,
free in menus); Caps Lock toggles Always Run (+0x651, from
`bAlwaysRunByDefault`), Q toggles Auto Move (+0x652, ended by a movement
key) (`0093e860` @ `00940c84`). Existing: R reload / hold holster, Space
jump, F view, V V.A.T.S., T wait.

## Not done (not substituted)

- Scoped weapons: scope overlay `0077f3c0` (HUD mode 0x17), forced first
  person, scope zoom branch, `ScopeWobble.nif`; weapon mods' zoom (0xe).
- Gun sway (`00962de0`, `WeaponWobbles\*.nif`, amount from the wobble
  `00646910`), iron-sights depth of field (`009650a0`), blocking.
- Third-person iron sights framing; the Info lines' height adjustment for
  wrapped names; activators' own activation text; class 10; terminals'
  lock line; doors' flag 0x100 (`0057b460`).
- Hotkeys 1–8 (need Pip-Boy assignment), ammo swap, grab.
- The pick is the viewer's bounds/collision ray plus the sphere radius,
  not Havok's sphere cast.

## Checks

Root: `cargo test --workspace` (all pass but `repo_hygiene`, which flags
the worktree's `.git` pointer file), clippy, fmt; viewer: build, test
(115), clippy, fmt. Tests: `world/tests/activation.rs` (4),
`world::iron_sights` (2), `ui` hud and anim (3 new), viewer `walk` (2),
`controls` (1).

Next action: compare in the original game: the Info panel for an item,
locked locker, load door; the sneak meter's states; varmint rifle
sights FOV and alignment.

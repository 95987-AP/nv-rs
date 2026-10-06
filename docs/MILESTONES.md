# nv-rs milestones

Updated 2026-10-06. Priority tracker and session handoff.

## Baseline

README and previous research notes report readers, rendering, streaming,
walking, actors, AI, combat, dialogue, quests, inventory, menus, audio,
V.A.T.S. and custom saves. Many corresponding source modules exist.
The first foundations batch built and tested both workspaces and ran the
opening to Doc's instruction to use the vigor tester. Missing cutscene and
menu behavior prevents calling that a faithful opening. Evidence remains in
ENGINE_REFERENCE.md, CLAUDE_REFERENCE.md and %USERPROFILE%\nv-re\findings.
The clean public source history was prepared on 2026-10-03. Earlier private
history is retained separately; do not publish its build outputs.

## Ordered gates

| Milestone | Completion gate | Status |
| --- | --- | --- |
| M1: Opening and persistent world | Retail opening movie and scripted wakeup, animations/movement, Doc Mitchell's creation sequence, exit into Goodsprings, talk/interact, save, restart and reload. Player, NPC, quest, inventory and reference state survives. Compare with the original game. | Active; initial live run and package-action reader complete; choreography and full route pending. |
| M2: Core gameplay loop | Sunny's tutorial and a representative Goodsprings quest branch through their own scripts: movement, weapons/reloads, damage/death, AI, dialogue, loot, trade and progression. Save/reload at intermediate stages. Verify melee and V.A.T.S.; track other weapon classes explicitly. | Partial implementation reported; acceptance pending. |
| M3: Base-game systems and campaign | Coverage matrix for quests, actor/creature types, weapon classes, effects, factions/crime, companions, travel and menus. Representative routes and ultimately campaign completion, with evidence and regression tests for blockers. | Inventory and acceptance routes needed. |
| M4: Stability and performance | Recorded routes and extended play without crashes or lost state. Measure frame times, memory, loading and streaming stalls on target hardware. Publish traces/settings and agreed budgets; remove measured stalls without changing behavior. | Not measured here; instrument earlier when it helps M1/M2. |
| M5: DLC and mods | Official DLC progression and reproducible plugin/archive/loose-file, script and content-extension cases; document interfaces and exclusions. | Load-order infrastructure exists; broad compatibility unverified. |
| M6: VR | Shared simulation with action inputs, independent aim and multiple views. Headset-tested tracking, controllers, menus, combat, comfort and frame budget. | Architecture documented; headset validation pending. |

Preserve save correctness, mod semantics and VR boundaries throughout; order
does not postpone foundational fixes. Set performance budgets from measured
hardware/display requirements. Original save compatibility and native binary
mod compatibility need separate researched scope; custom saves and plugin
reading do not establish them.

## Active work: M1

Look-IK batch, 2026-10-06 (local session, branch `claude/m1-look-ik`).
Corrections: ADR-0004 (restructure) was rejected on 2026-10-06 and the
layout stays; `codex/m1-reload-update-order` was dropped unmerged; Codex is
no longer used. The Xbox 360 prototype's PDB confirmed the
`bhkRagdollController` (Xbox PDB) field names and the look-IK function pairs
in the PC disassembly. Native head and eye tracking is now ported in
`world::look_ik` and runs on placed people in the viewer. Its helpers and
the direction solve are checked against FalloutNV.exe with `nv-call` (the
first runs of the oracle tools against the real executable). Root clippy
and the look-IK tests pass. Doc's tracking has not been compared with the
original game. Not yet done: the target choice of 008a3100 (the player is
the only supported target), eye meshes (FaceGen eye update), and
`nv-probe` recordings in a private game copy. Evidence:
[OPENING_LOOK_IK.md](OPENING_LOOK_IK.md). Parallel batches (each on its own
`claude/m1-*` branch) cover dialogue exit and fidelity, couch sitting,
Pip-Boy mouse input and player movement. **Next action:** record Doc's
look-IK in a private game copy with `nv-probe` and compare.

The published baseline includes the opening package look lock, Doc's queued
chair exit, native Info HUD and tester bounds correction, SPECIAL interface,
and same-cell trigger reset on F9. Live evidence reaches the tester through
stage55/60 and opens SPECIAL; it is not full-route acceptance.
Historical checks and play hashes remain in [OPENING.md](OPENING.md).

Persistence batch: failed custom save writes preserve the previous save, and
failed F9 destinations preserve the running world. Generated regressions
pass; 913 core and 84 viewer tests, both clippy/format/release checks and an
isolated installed-data forced-save smoke passed on
`codex/m1-opening-overnight`. Evidence and process handoff:
[PERSISTENCE.md](PERSISTENCE.md). PR #5 merged as `637b28d` after both GitHub
checks; live failed/valid F9 also passed. Verified build installed in play/app.

Next bounded batch implements native race/sex hair/eye reconciliation as
core face-editor groundwork, with six regressions and eight official-data
cases. 919 core / 84 viewer tests, both workspaces' checks/releases and the
installed-data opening smoke pass on `codex/m1-native-appearance`. The face
menu still auto-accepts.
Merged as `843926d` through PR #6 after both GitHub checks passed; verified
appearance build installed in play/app. No new face UI is exposed.

The bounded gamedb evaluation silently missed the central look-lock function
in a four-export probe. Full indexing is deferred; source snapshots, hashes,
DB and inventory remain private. Shared query workflow and limitations:
[RESEARCH_WORKFLOW.md](RESEARCH_WORKFLOW.md).

Movement batch (`claude/m1-movement`, unmerged): the player's speed now
follows `00647d10` in `world::locomotion` (weapon away × 1.1, armour and
drawn-weapon penalties, run perk on running only); over-encumbered blocks
running and jumping; a jump drops the run-up and air steering closes 0.3
of the gap a frame. Tested, not compared. Evidence and gaps (slopes,
per-direction animation speeds, camera): [MOVEMENT.md](MOVEMENT.md).

Player furniture batch, 2026-10-06 (`claude/m1-sitting`): E on furniture
now runs the game's own activation and sit procedure for the player
(`TESFurniture::Activate` `005095b0`, approach `00904f50`, temporary third
person `00950340`/`009503d0`, activate-key gate, seated pitch limit,
first-person seated loop with its Camera1st track); E on nothing while
seated gets up. Unit and generated-plugin regressions pass; no live couch
run or original comparison yet; no third-person camera exists for the
entry/exit. Evidence and gaps: [FURNITURE.md](FURNITURE.md). **Next:** live
`--stage VCG01 27` couch run through Doc's questionnaire.

Third-person batch, 2026-10-06 (`claude/m2-third-person`, unmerged): the
player camera's first/third person, view key (F; walk/fly moved to `),
wheel zoom, vanity mode, temporary views (furniture, Pip-Boy, dialogue)
and the chase camera with wall collision are translated from
`0094ae40`/`0094a0c0`/`00950110`… into `world::player_camera`; the
player's third-person body is built from the record and game state
(`world::actor::player_look`) and animated through the NPC path
(idle, directional walk/run, weapon, sitting). Tested, not compared.
Evidence and gaps (sneak/jump groups, body fade, pivot node term):
[CAMERA.md](CAMERA.md). **Next:** compare F/wheel/wall behaviour and the
couch entry framing in the original.

Outstanding M1 gates:
- Exact opening camera transition replay and Doc/player assistance timing.
- Doc head/eye tracking: ported and unit-tested; the original-game
  comparison, target choice and eye meshes remain
  ([OPENING_LOOK_IK.md](OPENING_LOOK_IK.md)).
- Original face editor instead of auto-accept, and opening movie playback
  ([FACE_CREATION.md](FACE_CREATION.md)).
- In-progress animation save restoration, full character-creation route,
  exit to Goodsprings and save/restart/reload acceptance.
- SPECIAL's remaining visual/input fidelity and progression comparison
  ([VIGOR.md](VIGOR.md)); general idle callbacks remain partial.

Camera persistence now passes 926 core/85 viewer tests and both workspaces'
checks/releases, 20 installed-clip continuation cases, live F5/F9 and
fresh-process camera restoration. NPC animations and dialogue continuation
are still missing. No camera build published yet.

Camera PR #7 (`5b3ec14`) exposed a pre-existing parallel fixture collision;
unique fixture directories and a deterministic regression fix it. Its amended
CI is running (amended core passed). Active branch
`codex/m1-reload-dialogue-cleanup` isolates old dialogue/voice state from
successful reloads; failed loads preserve playback. Root926/viewer89 tests,
both workspaces' checks/releases and live success/rejection cases pass.
It does not restore the saved conversation. Native RaceSexMenu
navigation is traced; field callbacks and original comparison remain open.
See [FACE_CREATION.md](FACE_CREATION.md). Camera PR #7 and dialogue PR #8
have since merged; check their build publication against the Codex branch. See
[PERSISTENCE.md](PERSISTENCE.md) for owners and unfinished checks. Gaze cap
constructors are now identified; runtime configuration and original comparison
remain unverified. Do not repeat the established
package-look lock or tester activation fixes.

Pip-Boy input batch (`claude/m1-pipboy`): the mouse now drives the
Pip-Boy (screen picking, rows, tabs, model buttons, world-map markers,
wheel, drags) through the interface, F1-F3 and Tab release work, and the
arm's field-of-view scale is fixed. Traced and unit-tested, not compared
with the original game; gaps in [PIPBOY.md](PIPBOY.md). **Next action:**
live comparison of the mouse paths in the original game.

Dialogue batch, 2026-10-06 (`claude/m1-dialogue`): Doc's farewell at the
front door could not be left because line follow-ups (`TCFU`) were not
read. Follow-ups, Goodbye states, random runs and Intelligence classes are
now traced from `00762ff0`/`0061af30`/`0061a7d0` and implemented, with
generated regressions; a real-data walk of the chain reaches the Goodbye
line, which un-destroys the house door. Not yet compared with the original
game. Evidence and gaps: [DIALOGUE.md](DIALOGUE.md). **Next action:** play
the route through the door with installed data.

M2 blocker batch, 2026-10-06 (`claude/m2-cell-scripts`): object scripts
now run for every attached cell (the interior, or the 5×5 grid with the
game's re-centring margin, including persistent objects), disabled ones
too; `OnLoad` fires once per cell attach or enable; triggers follow the
traced one-event-per-step occupancy; the pass stops after `Activate`/
`MoveTo`-type commands. Traced addresses, tests and gaps:
[SCRIPTS_RUNTIME.md](SCRIPTS_RUNTIME.md). Generated-world tests only; not
compared with the original. **Next action:** play Goodsprings to VCG02
stage 20 and compare trigger/`OnLoad` timing with the original.

M2 blocker batch `claude/m2-explosives` (Ghost Town Gunfight dynamite):
thrown weapons, grenade flight and explosion damage traced and implemented
with generated regressions; not compared with the original game. Evidence,
gaps and next action: [EXPLOSIVES.md](EXPLOSIVES.md).

M2 blocker batch (branch `claude/m2-quest-targets`): quest targets (`QSTA`),
the active quest, the door the compass follows and the blinking compass
quest icons, traced and translated; generated-data tests and an
installed-data check pass; not compared with the game; Pip-Boy map quest
markers and the navmesh-level path search remain. See
[QUEST_TARGETS.md](QUEST_TARGETS.md).

Dialogue NPC batch, 2026-10-06 (`claude/m1-dialogue-npc`, on
`claude/m1-dialogue`): the menu's zoom in/out (`00762950`), the player's
view focused on the speaker's head (`00953060`), the speaker's in-menu
turn (`008a5580`, replacing the inferred rule) and the main topic list
as the player's topics (`0083ec30`/`0083ed50`/`00619030`, dropping the
opening-follow-ups guess) are traced, implemented and unit-tested; not
compared with the original. **Next action:** record a Doc Mitchell and a
Sunny Smiles conversation in the original and compare zoom, turn and list.

## M2 blocker batch: gunfight packages

`claude/m2-packages` (2026-10-06): flee, guard, procedure lists, package
type data (`PKW3`, `PKPT`, `PLD2`...) and begin/end/change action dispatch
traced and implemented for Ghost Town Gunfight (`VMS16`). Generated
regressions pass; not compared with the original. Finding: the gunfight's
flee packages have no target or place, so the settlers just stop and
stand; CF/AM guard packages send settlers back to their editor location
after stage 70's `MoveTo`. Evidence and gaps: [PACKAGES.md](PACKAGES.md).
**Next action:** compare both `bTrudyHelp` branches in the original game.

## M2 blocker batch: NPC combat

`claude/m2-npc-combat` (2026-10-06): people now choose their weapon the
way the combat controller rates it (`009993c0`, DPS `00645380`/`00646060`,
weapon kinds `00522c80`, planner costs `011a4280`), switch when out of
reach or ammunition, empty and reload clips, and only use rounds up for
"NPCs use ammo" weapons (dynamite) or teammates (`008a8dd0`);
`GetShouldAttack` (`0059ed30`), `OnStartCombat` (`00980830`/`009887b0`)
and `SetUnconscious` ending the fight (`005d0760`) are in. Generated
regressions pass; not compared with the original. The planner's search
beyond its action costs is inferred. Evidence and gaps:
[NPC_COMBAT.md](NPC_COMBAT.md). **Next action:** record a powder-ganger
fight in the original game and compare weapon switches and dynamite
throws.

M2 route batch (`claude/m2-vcg02-route`): Back in the Saddle (`VCG02`)
driven in the viewer with installed data and completed in one run, with
the quest start, bottle hits, two package end actions, the walk to the
first well, the kills and the reward request simulated by console lines
(new `--run-at`/`--say` testing aids). Fixed: people walking into the
place joining trigger/talk lists, drawing references enabled after load
(bottles, geckos), `KillActor`'s `OnDeath`, shots at disabled objects.
Blockers: Sunny's first greeting (traced order says "Everything all
right?", which can't start the quest), no long-distance paths, package end
actions. Not compared with the original. Evidence:
[GOODSPRINGS_ROUTE.md](GOODSPRINGS_ROUTE.md). **Next action:** record
Sunny's first greeting in the original game.

## Deferred

Cosmetic material/lighting discrepancies, isolated facial polish, sun glare,
distant water and other rendering leftovers stay in the research archives.
Promote only for a milestone blocker or explicit user priority. Small gameplay
fixes needed by the active route count as blockers; never invent a substitute.

## Handoff

After each batch replace the active-work section with the latest outcome,
evidence links, blockers and one next action. Keep this file short; detailed
logs belong in topic references. Completion requires a reproducible acceptance
run, not a count of parsed records or implemented functions.

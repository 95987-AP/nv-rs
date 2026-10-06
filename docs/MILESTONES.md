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

Overnight batches, 2026-10-06 (local session; integration branch
`claude/overnight-integration`, not merged into `main`; each batch also has
its own pushed `claude/m*-*` branch for review as a PR). Play copy builds 1-6
were published from the integration branch. Everything below is implemented
and unit-tested; **none of it has been compared side by side with the
original game**.
- M1: look-IK (`claude/m1-look-ik`), Pip-Boy mouse (`claude/m1-pipboy`),
  player movement (`claude/m1-movement`), Doc's door farewell via TCFU
  follow-ups (`claude/m1-dialogue`), NPC dialogue facing/zoom and topic
  list (`claude/m1-dialogue-npc`), player furniture sitting
  (`claude/m1-sitting`).
- M2: grid-wide exterior reference scripts (`claude/m2-cell-scripts`),
  package types and actions (`claude/m2-packages`), quest targets on the
  compass (`claude/m2-quest-targets`), dynamite/explosions
  (`claude/m2-explosives`), third-person camera and player body
  (`claude/m2-third-person`), NPC weapon choice/reloads/GetShouldAttack/
  OnStartCombat (`claude/m2-npc-combat`), Back in the Saddle and Ghost Town
  Gunfight route fixes (`claude/m2-vcg02-route`, `claude/m2-vms16-route`).
- Acceptance evidence ([GOODSPRINGS_ROUTE.md](GOODSPRINGS_ROUTE.md)): with
  dialogue choices replayed by `--run` lines, Ghost Town Gunfight reaches
  stage 100 (XP +50) on the integration build; Back in the Saddle completes
  with several steps forced by console lines.
- Blockers: starting VCG02 by talking to Sunny (greeting order traced and
  matching; needs an original-game check), long outdoor NPC paths (agent
  on `claude/m2-long-paths`), NPC aim far too accurate (agent on
  `claude/m2-npc-aim`), eyes not moving (FaceGen eye update untraced),
  explosion visuals, Pip-Boy local-map quest markers (world map done on
  `claude/m2-pipboy-complete`).
**Next action:** review and merge the `claude/m*-*` PRs into `main`, then
play the Goodsprings route in the play copy and file F12 reports.

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

Look-IK reconcile, 2026-10-06 (`claude/m1-lookik-reconcile`, unmerged):
the contributor's head-track target choice, actor look anchors, FaceGen
eye darting and eye limits (`playcon/claude/*`, 9defa47, 5dfb5d9,
ae5b593, ddc0d59) are ported onto the native controller after re-checking
them in Ghidra; where the two ports disagreed the executable settled it
(the eye chain runs for people, so eyes aim at the target; the tracking
distance is between positions). Doc, Sunny and Goodsprings NPCs track
the player and each other in the viewer; 0064c410 and the eye-range
helpers are nv-call regression vectors. Not compared with the original.
**Next action:** record Doc's look in a private game copy with `nv-probe`.

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
- Doc head/eye tracking: ported, unit-tested and seen in the viewer,
  with whom actors look at and the FaceGen eyes (reconciled with a
  contributor's branches, 2026-10-06, `claude/m1-lookik-reconcile`); the
  original-game comparison remains
  ([OPENING_LOOK_IK.md](OPENING_LOOK_IK.md),
  [HEAD_TRACK_TARGET.md](HEAD_TRACK_TARGET.md)).
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

Pip-Boy live batch, 2026-10-06 (`claude/m2-pipboy-live`): driving the
release viewer with injected Windows input showed no Pip-Boy click ever
completed (the buttons were cleared every frame, so no release was seen);
fixed by reading the button events. Added, traced: the fast-travel
question, the player's own map marker (right button, set/move/remove
questions, saved), world-map zoom (wheel, Page Up/Down), the world map's
picture border for markers, ITEMS Drop (right button, quest-item refusal,
"how many?"), the keyring, ITEMS' button states, STATS healing mode (a
Stimpak aimed at a limb, `0082b970`), boxes over the Pip-Boy taking the
input. Each verified live in the viewer; none compared with the original
game. Gaps (Repair/Mod menus, hot keys, local map, radio, the light):
[PIPBOY.md](PIPBOY.md). **Next action:** compare these paths in the
original game, then the Repair menu.

Pip-Boy completion batch, 2026-10-06 (`claude/m2-pipboy-complete`): hot
keys (wheel, assignment, use with the Pip-Boy away), notes by kind (text,
image, audio with its countdown), world-map quest target markers, the
aimed limb's blink, knobs and rad needle turning, the Pip-Boy light
lighting the place, dropped items resting on the floor and taken back
with E. Verified live in the viewer (except the blink and needle, tested
only); none compared with the original game. Repair/Mod belong to another
contributor's branch. Still missing: the start menu on Escape, the radio,
the local map. **Next action:** the start menu (`007cb7d0`, `007cc6e0`).

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

## M2 blocker batch: NPC aim and next target

`claude/m2-npc-aim` (2026-10-06): people's shots now fly as the game aims
them (`00523150` NPC branch: aim point `009a8460`, cone = weapon min
spread + gun wobble × `fNPCMaxGunWobbleAngle`, iron sights past 512 ×
sight usage `008f74c0`) as rays to the first body, object or wall, with
the line-of-fire hold (`009a6e90`); after a kill or give-up fighters take
the best of their remaining targets (`CombatGroup::GetBestTarget`,
`00986c60`). Gangers' leveled guns are now resolved (they had chosen
fists). Gunfight re-run: both `bTrudyHelp` branches reach stage 100; the
varmint-rifle ganger hits the player 1–3 times in 9 shots at 3000–2000
units (was 10/10). Generated regressions pass; not compared with the
original. Evidence and gaps: [NPC_COMBAT.md](NPC_COMBAT.md). **Next
action:** record ganger hit rates at range in the original game.

M2 blocker batch (`claude/m2-long-paths`, 2026-10-06): long outdoor
walks. The game's high-level route over the navmesh info map (`NAVI`,
`NavMeshInfoSearch` (Xbox PDB) `006b8c50`/`006b8490`), the detailed path
only over attached cells (`006c9fc0`) and the virtual handler's node walk
out of sight (`009ea8a0`) are traced and implemented; the viewer's
navmesh is now the attached cells' and people beyond them walk out of
sight. Sunny's VCG02 walks (both wells, sneak, wells 2 and 3) complete by
themselves with the player following or ahead; one run completed VCG02.
Generated regressions pass; not compared with the original. Evidence:
[PATHING.md](PATHING.md), [GOODSPRINGS_ROUTE.md](GOODSPRINGS_ROUTE.md).
**Next action:** record where Sunny stops in the original when the player
stays behind the saloon.

NPC animation batch (`claude/m2-npc-anims`, 2026-10-06): Doc's chair
"jump" was the entry easing out over 0.5 s after the heading had turned
half a turn; the game swaps without a blend there (`cSkipNextBlend`,
`004974a0`, set by `009213e0`/`00921e80`), now carried out. People in the
dialogue menu were frozen because the dialogue menu's own screen counted
as a menu for animation; the speaker now animates and plays the game's
talking idles (`008a5580`/`008a20d0`/`008dab40`, `SNAM` speaker idles,
`MenuMode`/`IsTalking`/`GetDialogueEmotion`). Verified live with screen
captures (chair before/after, Sunny and seated Doc talking); not compared
with the original. Evidence and gaps (lines outside the menu, listener
idles, holstering): [ANIMATION.md](ANIMATION.md). **Next action:** record
Doc sitting down and a Sunny conversation in the original and compare.

M2 batch `claude/m2-player-actions` (2026-10-06): the crosshair's Info
panel from `00579280`/`00775a00` (Sit, Sleep, Take, Open, Talk, Search,
Pickpocket/Steal, "Door to Goodsprings", lock and Empty lines, weight and
value), pickups' "added" message and sound, the Sneak toggle with the
lower eye and the sneak meter, iron sights (Aim control, FOV, `IS`
animations, true iron sights), mouse look with the right button free,
Always Run and Auto Move. Seen live in the viewer (couch sit/stand,
stimpak pickup, Sunny talk/pickpocket, [HIDDEN], varmint rifle sights);
not compared with the original. Evidence and gaps (scopes, sway,
hotkeys): [PLAYER_ACTIONS.md](PLAYER_ACTIONS.md). **Next action:**
compare those screens in the original game.

M2 blocker batch (`claude/m2-npc-nav`, 2026-10-06): NPC routes and
collision. People were set along their path points with no collision, on
a stand-in search and funnel. Now the game's navmesh search costs
(`006a6fa0`), its `PathSmootherPOVSearch` (`006ad770`: corner circles of
1.2 × the request radius, side clearance, tangents, retries keeping off
narrow edges), the straight-line test with side lines (`006cd1f0`), the
stuck test (`009e4cf0`) and the player's character controller for every
person (`009ddc00` → `00930c70`) are implemented. Live: Doc walks to his
door and talks, Sunny's VCG02 runs to stage 45 by herself, settler 04
goes through the saloon door. Generated regressions pass; not compared
with the original. Gaps: light clutter blocks people (Easy Pete stuck at
his eating marker), door triangle registration, off-navmesh ends.
Evidence: [PATHING.md](PATHING.md). **Next action:** record Doc's walk
and Easy Pete's approach in the original.

M2 blocker batch (`claude/m2-npc-nav-2`, 2026-10-06): NPC navigation
gaps. Closed doors marked on the navmesh as the obstacle manager does
(`006997e0`), locations resolved by the game's height window (`00696a50`)
and ends off the navmesh joined by its ray-cast way (`006caa40`,
`006cac90`, `006e6e40`), travel/follow/long-way searches on a path-manager
worker (`006eb9d0`), combat/sitting walks with the actor's radius and no
straight-line fallback, people pushing clutter through the physics
batch's movers. Live: the gunfight replay completes ("Defeat the Powder
Gangers", XP +50, ~100 s; 7 dead), Doc walks to the door and talks, Easy
Pete reaches his place in the saloon, Sunny walks the long way. Not
compared with the original. Gaps: door-detection box, `PATHPICK` layers,
the move into the high process (a viewer bridge puts people back from
offstage onto the navmesh), push mass limit on the physics side. Evidence:
[PATHING.md](PATHING.md). **Next action:** record where an actor coming
into the high process stands (Sunny after `MoveTo SunnySpawnMarker`).

M2 batch (`claude/m2-physics`, 2026-10-06): clutter physics. Rigid body
values read from the models; free bodies simulated at the game's Havok
step clock (`00c66760`) against the cell's collision, each other and the
player; shot pushes (`009c2e80`), explosion pushes (`009b0920`) and the
gameplay impulse scaling (`0062b520`) translated; moved objects kept in
the state and saves. The VCG02 bottles are plain Havok clutter (no `DEST`):
verified live, a varmint rifle shot tips one off the fence onto the ground
behind it and F5/F9 keeps it there. Solver internals are this solver's,
labelled; not compared with the original. Evidence and gaps:
[PHYSICS.md](PHYSICS.md). **Next action:** record a bottle shot off the
fence in the original game and compare.

M2 batch `claude/m2-player-actions-2` (2026-10-06): scopes (the weapon's
`MOD3` overlay framed as `0076bfe0`/`0077ee50`/`00709d50` set it up, HUD
mode 0x17, hidden arms, `ScopeWobble.nif` sway on the view), the gun's
first-person sway (`00962de0`, `WeaponWobbles`), blocking (`BlockIdle`,
the threshold bonus of `009b5a30` in the hit cone of `009a6ae0`, the
block hit and counter timer), power attacks (`00948310`: hold delay,
directions, sneak attacks, perk customs, × `fDamagePowerAttackBonus`),
the Ammo Swap control with the HUD's `AmmoTypeLabel`, and death
(`fPlayerDeathReloadTime`, then the most recent save). Seen live in the
viewer (scope, block, forward power attack, swap to hollow points,
death → reload, [DANGER], third-person sneak pose); not compared with the
original. `WG`/`VAL` re-checked: invisible by data and code. Evidence
and gaps: [PLAYER_ACTIONS.md](PLAYER_ACTIONS.md). **Next action:**
compare the scope, block/power attack and death timing in the original.

NPC animation batch 2 (`claude/m2-npc-anims-2`, 2026-10-06):
`Actor::PickAnimations` (`00895110`) is now translated whole for
standing, walking, running, sneaking, turning and the weapon: every
`.kf` of the actor's folders indexed by group id (weapon kind, movement
kind) with the game's lookup and fallbacks (`00495740`); drawing and
putting away (`Equip`/`Unequip`, the weapon in hand at `Attach`); the aim
over a drawn weapon; NPC sneak and weapon-out rules (`00888b50`,
`008eeec0`); the strafe/back choice facing a target (`009e2aa0`); idles
in their record's section (upper-body greets, movement-section hit
reactions); the say's speaker/listener requests outside the menu
(`008a20d0`, GREET, conversations); hit reactions (`0089a760`); the
player's third-person body on the same path with `cSkipNextBlend`.
Verified live in the gunfight and with injected keys in third person;
not compared with the original. Evidence and gaps:
[ANIMATION.md](ANIMATION.md). **Next action:** record a ganger drawing
and a settler greeting in the original and compare.

M2 batch (`claude/m2-physics-2`, 2026-10-06): physics completion. Traced
and translated: the collision filter's layer table (`00c828f0`, shots
cast on the projectile layer), contact friction/restitution (`00cfd800`),
the land body's friction, the Z-key grab and Havok's mouse spring
(`0095f6c0`…`00961280`, `00cbb1e0`), impact sounds by material
(`00837550`, `00839e00`), physics damage (`006238b0`, `0062be90`),
saved velocities (`00563220`/`00563380`), the walkers' move limit
(`fMoveLimitMass`). Fixed: moved bodies hit and picked where they are; no
prompt on destroyed references (the swinging-door pick took clutter for
doors). Verified live: shot/second shot, no prompt, Z grab and carry,
dynamite moving the bottles, F5/F9 of flying bodies, walking into a
tumbleweed, contact sounds logged. Not compared with the original;
Havok's solver, deactivation and constraints aren't reproduced. Evidence
and gaps: [PHYSICS.md](PHYSICS.md). **Next action:** record a bottle shot
and a Z-grab carry in the original game and compare.

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

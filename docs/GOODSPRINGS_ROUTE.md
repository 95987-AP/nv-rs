# Goodsprings route acceptance

Live viewer runs of Goodsprings quests against the user's installed data,
with the fixes they needed. Each section is self-contained (one owner per
section). Status words: **reached** (seen in a viewer run), **fixed**
(code change with a regression test), **not compared** (not checked
against the original game).

## Ghost Town Gunfight (VMS16)

Branch `claude/m2-vms16-route`, 2026-10-06. Quest `VMS16` (00104EAE),
script `VMS16QuestScript` (00105D4E). Run outputs (logs, screenshots)
stay private in `%USERPROFILE%\nv-re\work\vms16-2026-10-06`.

### How it was driven

`nv-viewer <Data> WastelandNV --at X,Y,Z,180 --walk --weapon
WeapNV9mmPistol --wait 90..150 --screenshot FILE` with `--run` lines that
replay the quest's own result scripts in order (the dialogue choices were
not clicked through):

1. `set VMS16.bTrudyHelp to 1` (Trudy's recruitment INFO; left out for
   the no-help branch).
2. INFO 00105CC2 (Ringo, "All right, I'm ready"): `SetStage VMS16 65`,
   `SunnyRef.ResetHealth`, `SunnyRef.MoveTo SunnySpawnMarker`,
   `SunnyRef.AddScriptPackage SunnyTriggerGunfightDialoguePackage`.
3. INFO 00105CC5 (Sunny, "Time to look alive"):
   `SunnyRef.RemoveScriptPackage`, `GoodspringsPowderGangMarker.Enable`.
4. INFO 00105D09 (Sunny, "I'll be set up near the store"):
   `SunnyRef.AddScriptPackage SunnyTravelPackage`,
   `RingoRef.AddScriptPackage RingoTravelPackage`,
   `RingoRef.AddToFaction GoodspringsFaction 1`, `SetStage VMS16 70`,
   `set VMS16.bGunFightStart to 1`.
5. `player.ModAV Health 5000` so the standing, non-shooting automation
   player survives long enough to watch the fight (not part of the route).

Note: INFO 00105D09 sets `bGunFightStart` itself, so the quest script's
own stage-70 block (`GSJoeCobbRef.AddScriptPackage GSPGTravelPackage`,
the settlers' flee packages) does not run on this path; the gangers and
Joe Cobb take `GSPGTravelPackage` from their own package lists, and the
settlers' own lists carry `GoodspringsFleePackage`/the guard packages
(`docs/PACKAGES.md`). Runs that set only stage 70 exercise the quest
script's block instead; both reach stage 100.

### How far it got

**Reached stage 100** with Trudy's help (runs 5, 7, 8) and the player
standing at -67845,3000 (north of `PowderGangDestination`): stage 70's
moves (Joe Cobb, settlers 01–04, Trudy), all six gangers (Joe Cobb,
GSPG01/02/03/05/06; GSPG04 stays initially disabled with no parent) travel
in on `GSPGTravelPackage`; settlers, Trudy, Sunny and Easy Pete fight them
(Easy Pete helps the player against the gangers); each `OnDeath`
(`GoodspringsPowderGangerScript`, `GoodspringsJoeCobbScript`) counts, and
the sixth sets stage 100: "Completed: Defeat the Powder Gangers", XP +50,
Goodsprings fame and Powder Ganger infamy (Liked / Shunned). After stage
100 Ringo's `GSRingoAfterVMS16DialoguePackage` brings him to the player
and he starts talking (run 8, after fix 2).

Without Trudy's help (run 6) the gangers fight the player and Easy Pete;
the automation player does not shoot back and dies; settlers 02–04 do not
take part, as their packages say.

### Fixes

1. **Gangers following an enable parent never appeared** (commit
   `2e178c7`). GSPG01/02/03 have `XESP` → `GoodspringsPowderGangMarker`
   (00105D4C, persistent, initially disabled). A square loaded before the
   marker's `Enable` left them out ("disabled when the cell loads") and
   nothing brought them in later; only gangers in squares that happened to
   finish loading after the `Enable` came. Squares now record the people
   they left out as disabled (`world::ai::disabled_people_in_square`), and
   `bring_in_people` spawns those a script has since enabled, themselves or
   through their parent (`world::ai::enabled_since_load`; `Enable`
   005c43d0 loads the reference's 3D, `docs/SCRIPTS_RUNTIME.md`). People
   already on screen are not drawn a second time when a square reloads.
   Test: `crates/world/tests/ref_scripts.rs`
   `people_left_out_as_disabled_come_in_once_enabled` (generated square
   0,1 with an enable-parent follower and a persistent disabled person).
2. **People brought in after their place loaded were not talkers**
   (commit `a927d6a`). Anyone spawned by `bring_in_people` (script
   `MoveTo`, doors, fix 1) was missing from `Talkers`, so the player could
   not talk to, hit or V.A.T.S.-target them, and a script's talk request
   failed ("A script has 00104C7D talk to the player, but they aren't
   loaded here" for Ringo after stage 100; Joe Cobb, moved out of the
   saloon interior at stage 70, was not a V.A.T.S. target). They are added
   now and survive the squares' talker rebuild while on screen. Test:
   `viewer/src/exterior.rs` `people_brought_in_stay_talkers_when_squares_change`.
   Run 9 (`--vats 4` next to Joe Cobb) now lists him as a V.A.T.S. target.

### Investigated, not changed

- Target after a kill: when a fighter's target dies the viewer drops it
  out of combat (`world::combat::hurt`), and its `noticed` set means
  enemies noticed meanwhile never "rise" again, so it walks back to its
  package. `008ff350` processes a rise in combat too (it marks the
  detection entry's +0x1f flag and queues the start-combat event), and the
  game keeps fighting the combat group's other targets
  (`CombatGroup::GetBestTarget`, Xbox PDB; ranking not traced). Not
  changed: no run showed it blocking the gunfight (the gangers re-notice
  enemies as they come into sight), and the target choice is untraced.
- Settler 04's `GSSettlerAmbushPackage` takes him into the Prospector
  Saloon: his editor location is in `GSProspectorSaloonInterior`, so this
  follows the data.

### Not compared with the original game / remaining gaps

- Nothing here is compared with the original game: arrival timing, where
  the gang stops, who sees whom (the viewer's line of sight is from 60
  units above the feet), hit rates (gangers hit the player with nearly
  every shot from 3000 units in run 1), fame/infamy titles.
- The dialogue choices (recruitment INFOs, Ringo's and Sunny's lines) were
  replayed as console lines, not chosen in the dialogue menu.
- The player's own fighting was not driven (the automation cannot aim);
  the gangers were killed by the townspeople.
- Owned by the NPC-combat batch (`claude/m2-npc-combat`), not done here:
  `GetShouldAttack` (`GSJoeCobbTriggerScript`, VMS16b), `OnStartCombat`
  (`GoodspringsPowderGangerScript`, VMS16b), `SetUnconscious`
  (`GSVictorRef` at stages 70/100), NPC weapon switching (dynamite), NPC
  reloads and finite ammo.
- The guard packages' intruder scan (`docs/PACKAGES.md`).
- Save/reload during the fight was not tried.

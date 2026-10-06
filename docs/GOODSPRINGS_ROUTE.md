# Goodsprings route: Back in the Saddle (VCG02)

M2 acceptance drive, 2026-10-06, branch `claude/m2-vcg02-route` (on
`claude/overnight-integration`). Quest `VCG02` 0010A214, quest script
`VCG02SCRIPT` 0010A1F0. Installed data, viewer release build. Logs and
screenshots are private (`%USERPROFILE%\nv-re\work\vcg02-2026-10-06`).

**Nothing here has been compared with the original game.** "Reached" means
the viewer's own scripts, packages and dialogue did it; steps marked
*simulated* were done with console lines that stand in for something the
viewer cannot do yet (listed under blockers).

## The quest, from the data

| Stage | Set by | Result |
| --- | --- | --- |
| 5 | `VGenericTimerSCRIPT` event 3 (Doc's farewell) | objective 3 "Talk to Sunny Smiles in the Prospector Saloon" |
| 10 | Sunny's line 0010A1E4 ("Doc Mitchell said you could teach me…") | Sunny/Cheyenne go outside (`VCG02SunnyTravelOutside`), allies, CGTutorial 54 |
| 20 | `VCG02SunnyPatrolTrigger` `OnTrigger SunnyREF` | bark `VCG02SunnyBark` 0010A1E6: bottles enabled (`VCG02BottleMarkerREF`), varmint rifle, objective 10 |
| — | `VCG02TargetSCRIPT` `OnHitWith` ×3 | objective 10 done, `SunnyREF.evp` → `VCG02SunnySmilesDialogueStart` → 0010A1EC |
| 25 | 0010A1E5 ("Okay, I'm in.") | objective 20, geckos enabled, `VCG02SunnyTravelToWell1` |
| 30 | end action of `VCG02SunnyTravelToWell1` | `VCG02SunnySmilesDialogueSneakStart` → 0010A1ED |
| 35 | 0010A1ED | `VCG02SunnySneakCloserToWell` (end action: stage 40), VATS tutorial |
| 40/45 | stage 40 `SayTo VCG02SunnyBark` → 0010A1E7 | objective 30 "Kill the Geckos at the well" |
| 50 | `VCG02GeckoDeathSCRIPT` (2 deaths) | objective 40, `VCG02SunnySmilesDialogueSneakEnd` → 0010A1EE |
| — | 0010A1EB ("Sure, I'll come with you.") | objective 50, `VCG02SunnyTravelToWell2/3`, settler enabled |
| — | well 2/3 gecko `OnDeath` scripts | objective 50 done, 60 "Talk to Sunny about your reward" |
| done | 0010B3DA → 0015D97F → 0015F507 → "Couldn't hurt." 0015D97E | 50 caps, `CompleteQuest VCG02`, `RewardXP 50`, VCG03 10 |

## How far the route got

Command (abridged; the full one is in the private log folder):

```
nv-viewer.exe <Data> WastelandNV --at -68250,5800,8480,180
  --run "SetStage VCG02 5" --run "set VFreeformGoodsprings.bMetSunny to 1"
  --run "StartQuest VCG02" --run "SetObjectiveCompleted VCG02 3 1"
  --run "SetObjectiveDisplayed VCG02 5 1" --run "SetStage VCG02 10"
  --run-at 40 "set VCG02.nTargetCount to 3" --run-at 40 "SetObjectiveCompleted VCG02 10 1"
  --run-at 40 "SunnyREF.evp"
  --run-at 70 "SunnyREF.MoveTo VCG02SunnyWellMarker1" (and Cheyenne, and the player)
  --run-at 80 "SetStage VCG02 30" --run-at 80 "SunnyREF.evp"
  --run-at 120 "SetStage VCG02 40" --run-at 140 "player.MoveTo VCG02SunnySneakMarkerREF"
  --run-at 150/151 "VCG02Gecko1REF.Kill", "VCG02Gecko2REF.Kill"
  --run-at 200..205 "<well 2 and 3 geckos>.Kill"
  --run-at 215 "SunnyREF.StartConversation player"
  --say "I'm in" --say "Sure, I'll come" --say "Couldn't hurt"
  --screenshot ... --wait 330
```

Result: **VCG02 completed** (`CompleteQuest VCG02`, XP +50, 50 caps,
VCG03 started) in one run. Reached by the viewer itself: Sunny leaving the
saloon and walking to her marker, the patrol trigger (stage 20) and her
bark, the bottles appearing, her dialogue packages at the saloon, the
first well and the sneak marker (0010A1EC, 0010A1ED, 0010A1EE), the
stage 40 bark, the gecko deaths (stage 50), her 5,059-unit walk toward
well 2, the reward line with its follow-ups, objectives and journal
lines. Simulated: the dialogue that starts the quest, the three bottle
hits, the walk to the first well (Sunny's and the player's), the two
package end actions (stages 30 and 40), the kills, and the player asking
for the reward (`StartConversation` instead of the player's E).

The bottle-hit path itself was checked separately on installed data (a
throwaway test, not committed): three `Runner::hit` calls with the varmint
rifle drawn count `nTargetCount` to 3, complete objective 10 and queue
Sunny's package evaluation (`GetWeaponAnimType` 5). Aiming at the bottles
in the viewer was not driven.

## Fixes (one commit each)

| Commit | What blocked | Fix and provenance |
| --- | --- | --- |
| Count people who walk into a place among its people | Sunny, walking out of the saloon, was drawn but not among the place's people, so `VCG02SunnyPatrolTrigger` never saw her: no stage 20 | `bring_in_people` adds the people it brings in (and puts drawn ones back after a square rebuild); trigger occupancy follows `TriggerEntry::Update` `0062cc90` as already traced. Viewer test. |
| Draw references a script enables after their place loaded | Disabled-at-load references were never drawn; `VCG02BottleMarkerREF.Enable` showed no bottles ("isn't loaded here"), geckos enabled at stage 25 never appeared | `world::newly_enabled` (children follow their enable parent; `Enable` `005c43d0` ignores a child and queues the reference's enabling `005aa5d0`); viewer `bring_in_enabled` draws them. Generated-data test. |
| Look again for left-out references when the loaded squares change | A reference enabled while a square was still loading stayed left out | `bring_in_enabled` also looks when the place or loaded squares change. |
| Run OnDeath when KillActor kills | `KillActor` (alias `Kill`) skipped `OnDeath`, so `VCG02GeckoDeathSCRIPT` never set stage 50 | `KillActor` `005be2a0` kills through the death routine `0089d900`, the one a fatal hit takes; `OnDeath` now runs as for other deaths. Generated-data test. |
| Keep people brought in among the place's people on square loads | A dialogue package starting right after a square load found Sunny missing and talked to no one | The square rebuild keeps people `world::ai::moved_into` has in the worldspace. |
| Shots and blows meet only shown scripted objects | Disabled bottles could be shot (and counted) before they appeared | `combat::meetable` adds the enabled test (inferred from `0054c740`'s separate "has 3D" and "disabled" cases). Viewer test. |
| Add --run-at and --say | Routes could not be driven without a keyboard | Testing aids: timed console lines and topic choice by text. No behaviour change. |

## Blockers left (not fixed here)

1. **Sunny's first greeting.** With VCG02 running at stage 5 the viewer
   picks 0010B755 "Everything all right?" (VCG02, priority 75) over the
   say-once 00104E7E "Cheyenne, stay…" (VFreeformGoodsprings, priority
   55), and that line's topics lead only to a Goodbye, so the quest cannot
   be started in the viewer. Traced: `SetStage` (`005c7140` →
   `0060d510`) starts a quest (`0060c9c0`) unless it is completed without
   repeatable stages; `GetQuestRunning` `0059e320` tests flag bit 0; the
   topic's quest list (`TESTopic` +0x2c) is built in `DIAL` order by
   `006195d0`/`00905820` (append), `TESTopic::SortQuests` (Xbox PDB,
   PC `0061ba90`) sorts it by priority descending, and `0061a7d0` walks
   it in that order. Every one of those puts VCG02 first, agreeing with
   the viewer, yet the quest's own design needs "Doc Mitchell said you
   could teach me…" (in 00104E7E's topics) to be reachable. Unresolved;
   needs the original game. Dialogue owners' area.
2. **Long travel has no path.** `VCG02SunnyTravelToWell1` (about 15,000
   units, five grid squares) gets no path: the viewer's path search covers
   only the 3 × 3 squares around the player (`ai::CellNav`, untraced), so
   Sunny stands still. The game's long-distance pathing is not traced. AI
   owners' area.
3. **Package end actions are not run.** `VCG02SunnyTravelToWell1` ends with
   `SetStage VCG02 30`, `VCG02SunnySneakCloserToWell` with `SetStage VCG02
   40`; the viewer only reports begin/change actions. Another batch owns
   package actions.
4. Not driven: aiming at the bottles, the player following Sunny, the
   player starting the reward conversation, gecko fights.
5. Seen in passing, not investigated: a sandboxing settler (00104F03)
   "eats" a Super Stimpak and its effect script applies
   `Addiction01ISFX` to the screen; the disabled `VCG02GSSettlerREF`'s
   `GameMode` asks it to `Say` every 10 s ("aren't loaded here"); the VATS
   tutorial hint repeats while CGTutorial stays at stage 70 (the data's
   own timer).

## Not compared with the original game

Everything above: trigger timing, enable timing and drawing, Sunny's
walking, dialogue choice, the reward amounts on screen.

**Next action:** settle blocker 1 in the original game (record which line
Sunny greets with after Doc's farewell), then remove the simulated steps
as packages' end actions and long paths land.

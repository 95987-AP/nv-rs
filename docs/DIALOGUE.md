# Dialogue menu flow

Branch `claude/m1-dialogue`, 2026-10-05. M1 blocker: after Doc Mitchell
walks the player to his front door the conversation could not be left, so
the player could not leave the house.

## The blocker

Data (`FalloutNV.esm`, read with `nvinspect`):

- `VCG01` stage 110 (`DocMitchellREF.evp`, `DisablePlayerControls`) sends
  Doc to the door; `VCG01DocMitchellFarewellDialogueStart` (PACK 00105BD2,
  type 15, `GetStage VCG01 >= 115`, target the player, no topic) then
  opens the dialogue menu on `GREETING`.
- At that point Doc's `GREETING` line is INFO 001057E8 ("Here. These are
  yours.", say once). It has no topics (`TCLT`) but a follow-up (`TCFU`)
  to INFO 001057E4 (the Pip-Boy), which follows up to 001057E5/E6 (by the
  player's sex), which offer topics 083/084/085; their answers follow up
  to INFO 001057E7, flagged Goodbye. Its first result script is
  `GSDocMitchellHouseIntDoorREF.SetDestroyed 0` ("so player can leave"),
  its second `EnablePlayerControls ...` and the `VGenericTimer` event.
- nv-rs did not read `TCFU`. After 001057E8 the menu offered the main
  topic list instead, which has no Goodbye line, and the game's dialogue
  menu has no other way out (its key handler `007628c0` only takes
  code 9, A/Enter). The Pip-Boy, the door and the controls were never
  restored.

## Traced (FalloutNV.exe 1.4.0.525, decompiled; private exports in
`%USERPROFILE%\nv-re\work\dialogue-2026-10-06`)

| Address | What | Used for |
| --- | --- | --- |
| `0061dbd0` | INFO load: `TCLF`, `TCLT`, `TCFU` into the line's conversation data | `Info::follow_ups` |
| `00762ff0` | Menu: next response, or after the last: end script, follow-up, topics or close | `dialogue::after_line` |
| `0061af30` | Matching follow-up info (`TESTopic::GetMatchingFollowUpInfo`, Xbox PDB) | `dialogue::follow_up` |
| `0061a7d0` | Matching info of a topic: per quest, random runs | `dialogue::pick`, `choose` |
| `0061e600`, `0061e720` | Line availability: say once, Intelligence class vs `iDialogueDummySpeakThisIntOrBelow` (`011d0dc8`) | `dialogue::line_available` |
| `00762860` | Goodbye state `+0x2c`: 2 Goodbye flag, 1 `GOODBYE` topic (`0061a2d0(1,2)`) | `dialogue::ending` |
| `00762160` | Close: end script for states 2/3 unless flag 0x08 | `menu_runs_end_script` |
| `0083ebb0` | Begin script unless flag 0x40 | `menu_runs_begin_script` |
| `0083ec30`, `0083ed50`, `0083f110` | Topic list: info's `TCLT` or the player's list; topics 0xFD/0x118 excluded; priority sort | carried out in the next batch (below) |
| `00762950` | Menu update: state 1 cuts a clicked-away voice after 500 ms; state 3 waits the line timer | viewer skip timing |
| `008a20d0`, `008bc590` | Say: missing voice file sets the menu timer to `fDialogSpeechDelaySeconds` (`011d32c4`, exe 2.0) | silent-line duration |
| `0061b320` | NPC say: same matching (`0061a790`); "run immediately" runs script 0 at once | `social::pick_for` |

INFO flag bits as the code tests them (`INFO+0x25`, `+0x26`): 0x01 Goodbye,
0x02 random, 0x04 say once, 0x08 run immediately, 0x20 random end, 0x40 no
menu begin script, 0x80 speech challenge; byte 3: 0x01 say once a day,
0x02 always darken, 0x10 low Intelligence, 0x20 high Intelligence. GECK
names are unverified.

## Implemented

- `world::dialogue`: `TCFU` follow-ups; `follow_up`, `after_line`,
  `ending`, random runs (`choose`), Intelligence classes, flag 0x08/0x40
  script rules. `social::pick_for` uses the same line choice.
- Viewer `dialogue.rs`: after a line, follow-ups are said straight on;
  Goodbye lines and `GOODBYE` answers close the menu; silent lines last
  `fDialogSpeechDelaySeconds` instead of an invented reading pace; a
  clicked-away voice is cut 0.5 s later.

## Tested

`crates/world/tests/dialogue_flow.rs` on `testdata::dialogue` (generated,
shaped like Doc's farewell): follow-up chain to the Goodbye and its
scripts, Goodbye/`GOODBYE`/run-immediately endings, random runs, the
Intelligence filter.

A throwaway program (not committed) walked the real chain through
`world::dialogue` on the installed `FalloutNV.esm` with `VCG01` at stage
115, objective 40 complete and the psych-test greeting said: 001057E8 →
001057E4 → 001057E6 (male player) → topics 083/084 (085 has no line in
the data, so it isn't offered) → 0015E139 → 001057E7 → close; the door
`GSDocMitchellHouseIntDoorREF` is no longer destroyed afterwards.

## Not compared with the original game

Nothing here has been checked side by side with the original yet; the
installed-data route through the door is the next check.

## The speaker, the world and the view during the menu

Branch `claude/m1-dialogue-npc`, 2026-10-06. Private exports in
`%USERPROFILE%\nv-re\work\dialogue-npc-2026-10-06`. Names marked (Xbox
PDB) come from the prototype's symbols: `DialogMenu` (states
`eCameraZoomIn` 0, `eLoadSpeech` 1, `eSpeechIdle` 2, `ePlaySpeech` 3,
`eCameraZoomOut` 4, `eServiceFadeOut` 5; `+0x128 fPercentZoomed`,
`fPackagePercentZoom`), `MenuTopicManager` (`+0x10
bSpeechChallengeLoss`), `MenuTopic` (`+0x8 bTopicIsChoice`). Actor
vtable slots on PC are the Xbox ones + 4 from `+0x258` on (checked
against `Character`'s vtable `01086a6c`: `+0x264` `UpdateInDialogue`
`008a5580`, `+0x280` `InitiateDialogue` `008b2170`, `+0x288`
`EndDialogue` `008b1070`).

| Address | What | Used for |
| --- | --- | --- |
| `0086e650` | Main loop: in menu mode (`00702360`) the process lists aren't updated | everyone but the speaker holds still (unchanged) |
| `00761a20` | `DialogMenu::Create`: speaker's dialogue package zoom (`00672850`, `PKDD` float), first line said at once | `MenuZoom`, `focus_percent` |
| `00762950` | `DoIdle`: zoom in over `fDialogZoomInSeconds` (1.5), out over `fDialogZoomOutSeconds` (0.5), menu destroyed at 0; every frame `FocusOnActor` and, while in dialogue with the PC (`00933840`), the speaker's `UpdateInDialogue` | `world::dialogue_view::MenuZoom`, viewer close after zoom-out |
| `00953060` | `PlayerCharacter::FocusOnActor` (own error text): face node bound (or `Bip01 Head`/`Bip01 Speaker`, radius 32), look point raised by `fDlgLookAdj`, zoom atan(`fDlgFocus` × r / d) × 100 ≤ `fDefaultFOV`, eased over the zoom in; pitch/heading start/stop thresholds `fDlgLookDegStart/Stop`, `fDlgHeadingDegStart/Stop`, rate `fDlgLookMult` | `Focus::frame`, `dialogue::focus_camera` |
| `00fa8d40`…`00fa8e60`, `00f6e610`, `00f6e640`, `00fbc020` | Setting initialisers: 3.2, 13, 0.2, 13, 0.2, −5, 2; 1.5, 0.5; `fDefaultFOV` 75 | `ViewSettings::DEFAULT` |
| `0095de30` | Outside dialogue and V.A.T.S. the field of view returns at 30 ÷ `fIronSightsFOVTimeChange` °/s | `fov_back` after the menu |
| `008a5580` | `Actor::UpdateInDialogue`: not seated, not in combat (`+0x104`), mover not rotating (state 4), > 1° off (`01023128`) → `RequestRotateActor` toward the player (`009dce80`); mover updated with its dialogue flag (`009c9900`) | `speaker_turns`, viewer `ai::move_actors` |
| `0083ec30`, `0083ed50`, `0083f0d0` | `LoadNextTopicList`/`FillTopicList`: a line's `TCLT` in stored order, else the player's topics (`+0x6a8`) sorted by priority (`0083f110`); 0xFD `SpeechChallengeFailure` and 0x118 `InfoRefusal` left out; topic `DATA` flags 0x10/0x20 Intelligence classes | `dialogue::menu_topics`, `answered` |
| `00619030`, `00619410`, `00952830` | `TESTopic::InitItem` adds every topic flagged 0x02 (any kind) to the player's list; `AddTopic` | `Topic::is_top_level` (the `GOODBYE` topic, kind 1, is one) |
| `0083e850` | `DoSpeechChallengeCheck` for lines flagged 0x80: chance from Speech, disposition and difficulty, `rand() % 100`; a loss says the `SpeechChallengeFailure` line and pops back to the previous list | not carried out (see gaps) |

### Implemented

- `world::dialogue_view` (new): `ViewSettings` (game settings and INI),
  `MenuZoom`, `focus_percent`, `Focus::frame` (translated from
  `00953060`), `fov_back`, `speaker_turns` (translated from `008a5580`).
- Viewer: the menu zooms in as it opens and, once to close, zooms out for
  `fDialogZoomOutSeconds` before it goes (world still frozen, nothing more
  said); `dialogue::focus_camera` turns the player's view onto the
  speaker's head and narrows the field of view, then lets it return.
- Viewer `ai::move_actors`: the speaker's in-menu turn follows
  `008a5580` (replaces the inferred "face every frame" rule; combat
  excluded, turn played out once started).
- `world::dialogue`: the main list is the player's topics (top-level of
  any kind plus learned), without the opening line's follow-ups (that
  guess is gone: Sunny Smiles' main list reaches "Goodbye." through the
  top-level `GOODBYE` topic, kind 1, priority 5, her "Until next time.");
  the refusal topics and wrong-Intelligence topics are left out.

### Tested

`crates/world/src/dialogue_view.rs` unit tests (zoom timing, package
zoom, zoom on the head by the end of the zoom in, the FOV clamp, the
start/stop thresholds, no turn while zooming out, the FOV return, the
speaker's turn rule); `crates/world/tests/dialogue_flow.rs`
`the_main_list_is_the_players_topics` on new generated topics.

### Not compared with the original game

None of this has been checked side by side yet; the zoom and turn rates
and the head bound especially need a recording of a conversation.

## Remaining gaps

- The head's bound: the game merges the face node's skinned pieces'
  bounds; here a box-middle/farthest-vertex sphere over the head parts as
  skinned now (an approximation, labelled in `dialogue::head_bound`).
- Not carried out from `00953060`/`00761a20`: the depth of field
  (`fDialogFocalDepthRange` 300, `…Strength` 0.65 × percentage), the
  forced first-person view (`00951a10`, `00950110`) and its restoration,
  a seated player's look offset (`+0x6e4`), the menu tiles showing only
  from 10 % zoom (`00762950` state 0).
- Which update calls `0095de30` (the FOV return) is untraced; the
  first-person view model's own FOV isn't zoomed (it is hidden during
  conversations).
- The speaker's talking idles in the menu are now traced and played
  ([ANIMATION.md](ANIMATION.md)); the face emotion handling in
  `UpdateInDialogue` (`+0xb4` cases 1/5/8) and idles for lines said
  outside the menu are not.
- `0083e850`'s random speech challenge (0x80 lines), the loss pop-back,
  XP and disposition change; rumors (topic flag 0x01, `0042df90`);
  `InfoRefusal` lines; say once a day (`00935a40`); the `+0x24`
  next-speaker rules; the pause after a voiced line (`008bc590`).

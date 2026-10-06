# NPC body animation: furniture transitions and talking

Branch `claude/m2-npc-anims`, 2026-10-06. Maintainer's playtest: people
talked to "are completely still other than their face and mouth and
eyes", and Doc Mitchell "jumps" in his chair after sitting down. Private
exports (decompiles, captures, logs) in
`%USERPROFILE%\nv-re\work\npcanims-2026-10-06`. Names marked (Xbox PDB)
come from `Fallout_Release_MemDebug.pdb`; PC addresses are FalloutNV.exe
1.4.0.525. Older animation and furniture evidence: `findings\animation.md`,
`findings\furniture.md`, [FURNITURE.md](FURNITURE.md).

## The chair jump

Cause: the furniture procedures turn the actor by the marker's heading
delta (or half a turn) at the moment the entry or exit animation hands
over, and the game swaps the animations in that same frame without a
blend. nv-rs turned the heading but let the entry ease out over its
`Blend:15` (0.5 s). `Chair_ForwardEnter.kf` ends with `Bip01 NonAccum`
at 87.2°, `DynamicIdle_ChairSit.kf` holds −92.8°: the half turn the
heading takes. During the ease out the body was the entry's pose on the
new heading, so it swung half a turn and back: the jump.

| Address | What | Used for |
| --- | --- | --- |
| `004974a0` | sets `Animation` +0x120 = `cSkipNextBlend` (Xbox PDB layout matches PC: +0xe0 current sequences, +0x124 `spAnimIdle`) | `animation::Player::skip_next_blend` |
| `004949a0` | playing a group: blend 0 while the flag is set | `Player::play_from_pose` |
| `004994f0`, `00496080` | stopping a section: blend 0, deactivated at once | `Player::stop_section`, ends in `update` |
| `00491180` | the animation update clears the flag at its end | `Player::update` |
| `009213e0` case 3/8 | entry done, marker < 21: heading += delta (`00931d30`), flag, entry freed (`00498910(1,0)`) | `Sitter::update` → `skip_next_blend` |
| `00921e80` case 5/10 | exit loaded: heading −= delta, flag, exit played (`00498230`); exit done, marker < 21: flag, heading += π, `00895110`; no exit: flag, section 0 cleared | `Sitter::update` |

The viewer's `sitting::furniture_frame` hands the Sitter's flag to the
rig's `Player`. Tree idles asked by the idle clock wait for the next frame
when the flag is set (the process plays requests at a later update,
`008dab40` → `008dae00`), so a seated idle doesn't cut in unblended.

## Talking: the speaker's body in the dialogue menu

Two causes of "completely still": the viewer froze every animation while
any menu was open, and the game's own dialogue menu screen counts as one
(`Menus::game_open`); and nothing asked for talking idles.

| Address | What | Used for |
| --- | --- | --- |
| `0061e780` | response loader: `SNAM` speaker idle, `LNAM` listener idle after each `TRDT` (IDLE forms) | `dialogue::Response::speaker_idle`, `listener_idle` |
| `008a5580` | `Actor::UpdateInDialogue` (Xbox PDB): +0x7d = 1 (talking), +0x86 = `bUseEmotion` (`TRDT` byte 20, `008a5cf0`), say (+0x284 `008a20d0`) with the response's emotion and idles and a last argument 1; each later frame with a dialogue package (type 0x1c), special idle done (`004985f0`) and no request waiting (process +0x350, vfunc +0x718 = `006214b0`): idle request mode 2 | `talk_idles::Talking::say`, `between_says` |
| `008a20d0` | say: speaking emotion (+0x2d4/+0x2dc); with an idle or the last argument: request (process +0x44 = `008dab40`) for the idle (mode 3) or the tree (mode 2), forced; the listener likewise (not shown: the player) | `Talking::say` |
| `008dab40` | request: sit state 0/4/9; special idle done, idle named or forced; tree (`00600950`) only when no special idle is starting (`00498f80`); queued at +0x350 | `talk_idles::takes_request`, `Talking::queued` |
| `008dae00`, `00497f20`, `00498290` | the queued idle played: old special idle freed, new one blended from the pose, loops rolled | `sitting::play_requested` (`Player::request_special_idle`) |
| `008b1070` → `008ba600` → `008daf20` | leaving the menu: process flag 0x800; once no special idle is starting, sit state 0/4/9 frees the special idle | `Talking::menu_closed`, `sitting::free_talk_idle` |
| `005a1150` → `008a67f0` | `IsTalking`: actor +0x7d | true from the say's first frame |
| `005a4480` | `GetDialogueEmotion`: speaking emotion if +0x86, else −1 | `IdleQuestion::emotion` |
| `0059c380` | `MenuMode`: 0 any menu, else that menu | `IdleQuestion::menu` (dialogue 1009) |

The idle tree's own `DialogueIdles` (`GetCurrentAIProcedure` 4 OR
`MenuMode 1009`) / `TalkToPlayer` / `DialogTalking` branches and the
seated `ChairDialogueIdles` → `ChairTalk` then pick the gestures by
`IsTalking`, `GetDialogueEmotion`, `IsLastIdlePlayed` and
`GetRandomPercent`. `actors::animate_actors` now keeps animating while the
only menu up is the game's dialogue menu (class 1009); `ai` still holds
everyone but the speaker.

## Implemented and tested

- `world::animation`: `cSkipNextBlend`; unit test
  `skip_next_blend_switches_at_once_for_one_update`.
- `world::furniture::Sitter::skip_next_blend` at the four transitions;
  `world/tests/furniture_blend.rs` runs Sitter + Player on generated
  sequences shaped like Doc's chair: the body's facing changes by at most
  5° a frame (60 fps) with the flag, over 90° without it.
- `world::talk_idles` (new): say, between-says, request gate, menu close;
  unit tests. `world::idles`: `MenuMode`, `GetDialogueEmotion`,
  `IdleClock::played`; test on a generated dialogue branch.
- `world::dialogue::Response`: `use_emotion`, `speaker_idle`,
  `listener_idle`.
- Viewer: `sitting::dialogue_frame`, `dialogue_over`, `free_talk_idle`;
  `ai::move_actors` calls them for the menu speaker; `dialogue::Talk::said`;
  `menus::Menus::only_game_menus`; `actors::animate_actors` menu rule
  (`menu_stops_animation`, viewer test
  `only_the_dialogue_menu_lets_the_speaker_animate`).

Checks: root `cargo test --workspace` (only `repo_hygiene` fails, on the
worktree's `.git` pointer file), clippy and fmt clean; viewer build,
tests, clippy and fmt clean.

## Verified live (installed data, release viewer)

- Chair: `GSDocMitchellHouse --at 2200,2180,7360,60,10` (Doc walks to his
  chair and sits, entry 2.9 s → seated 4.6 s). Screen captured at ~63 fps
  (`burst.ps1`, `montage.ps1`). With the fix the seated pose is steady from
  the moment he settles; with the flag disabled (a temporary local
  switch, not committed) he visibly rises and turns as the entry ends and
  drops back over about half a second: the reported jump.
- Talking, standing: `GSProspectorSaloonInterior --talk` (Sunny Smiles,
  dialogue menu): `DialogNoBodyShiftTalkSubtle` on her line, then
  `DialogNoBodyShiftListenA` at 14.1 s when the topics came up. Picture
  motion (64-px thumbnails, mean absolute difference between samples,
  8–20 s): 0.00 before (frozen), 0.96 after.
- Talking, seated: `GSDocMitchellHouse --stage VCG01 27 --at
  2200,2180,7360,60,10 --run-at 8 "DocMitchellREF.StartConversation
  player"`: seated Doc gets `SitChairTalkPlayerC` from `ChairTalk`; after
  the Goodbye the idle is freed and the relax idles resume.

## Not compared with the original game

No side-by-side recording yet: entry/exit timing, the talking idle
choices and their timing need a recording of Doc's chair and of a Sunny
conversation in the original.

## Gaps (not implemented, not substituted)

- Lines said outside the menu (`SayTo`, `005c9100` → `0061b320`; NPC
  conversations): their say arguments and the listener's idle request
  aren't traced, so no talking idles there.
- The listener idle (`LNAM`, the tree for the listener) is read but not
  played: in the menu the listener is the player (first person); NPC
  listeners belong to the untraced path above.
- A tree answer for the base loop (section 0) during dialogue isn't
  played (how `00497f20` places one isn't traced).
- `008dab40`'s other refusals (vfuncs +0x230/+0x234, `00437bf0`,
  `008dade0`) and the say's +0x104 animation flag (`00493bb0`, treated as
  clear) aren't modelled.
- The player's third-person body (`player_body.rs`, another owner) doesn't
  yet take the Sitter's `skip_next_blend`.
- Holster/unholster (`Equip`/`Unequip` groups), upper-body idles as their
  own section, and replay delays counting during menu mode are unchanged.

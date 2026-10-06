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
| `0083ec30`, `0083ed50`, `0083f110` | Topic list: info's `TCLT` or the global list; topics 0xFD/0x118 excluded; priority sort | not changed (see gaps) |
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

## Remaining gaps

- The main topic list after a line without topics still keeps the opening
  line's follow-ups (a guess); `0083ec30`'s global list, the 0xFD/0x118
  exclusion and `0083f0d0`'s topic flags are not carried out.
- Say once a day (`00935a40` list), speech-challenge outcome (`0083e850`),
  the `+0x24` next-speaker rules inside the menu, the pause after a voiced
  line (`008bc590` sets the timer when called with its second argument 0;
  which callers pass what is untraced).
- Speaker/listener behaviour during the menu (camera focus `00953060`,
  the NPC turning and holding position) is not changed here.

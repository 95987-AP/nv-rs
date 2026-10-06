# Terminals: the terminal's own screen

What the player sees once into a terminal (after hacking it, with its
password note, or unlocked; [HACKING.md](HACKING.md) has getting in):
the game's `ComputersMenu`. Until 2026-10-06 the viewer showed terminals
in a text panel of its own (still the fallback without the game's menu
files).

## Sources

`FalloutNV.exe` 1.4.0.525, read in Ghidra 12.0.4 from the full private
export plus whole-function disassembly; `menus\computers_menu.xml` from
`Fallout - Misc.bsa`.

| What | PC |
| --- | --- |
| `ComputersMenu` vtable (RTTI) | `01072004` |
| Constructor (sounds, note handlers) | `007577f0` |
| Opening | `00757b70` (from `00706130`, menu request 3; hacking is request 2, `00709470`) |
| The logon | `00758ad0` |
| The screen | `00758d30` |
| Filling the list | `007586e0` |
| The typing queue: add, step, finish all | `00759da0`, `007598d0`, `00759f50` |
| Every frame | `00758470` |
| Clicks | `00757f70` |
| Leaving (code 10) | `007583f0` |
| Back a screen | `00758a80` |
| A text note's page | `007590c0` |
| A picture note | `00759470` |
| A sound note | `00759030` |
| An item typed again after a note | `00759560` |
| Running an item's result script | `00501830` |
| Giving a note | `00966a70` |

## The record

`TERM`: `DESC` the welcome text, `DNAM` difficulty, flags, server type
(byte 2, the item `+0xA6`: `sTerminalServerText1`–`10`), then items
(`ITXT` text, `RNAM` result, `ANAM` flags, `INAM` note, `TNAM` sub-menu,
the result script, `CTDA` conditions). An item's flags (its `+0x74`): 0x01
gives its note (`00758350`), 0x02 fills the list again (`00758390`); the
code adds 0x04 when it's picked (only a default for the controller's
cursor, `xdefault` 2). In the official master: 895 items, 535 with a text
note, 4 a voice note, 3 a picture; 137 flagged 0x02, 18 0x01, 1 both; 86
sub-menus, 208 result scripts.

## The menu (`ui::menus::computers`, `viewer/src/game_menus/computers.rs`)

Ids 0 the server line, 1 the list (`computers_file_template`), 2 the
display zone, 3 and 4 the header, 5 the result, 6 the cursor, 7 the
welcome, 8 the prompt, 9–12 the logon's lines, 13 the background, 14 the
zone's text, 15 the separator.

- Every line appears through one queue, typed in order, each at the rate
  set when it was queued: menus `1000 / iComputersDisplayRateMenus`
  (150 a second → 6 ms, whole milliseconds), notes `…Notes` (150), the
  player's input `1000 / iHackingInputRate` (50 ms); exe defaults, none
  in the data. Lines typing play `UIHackingCharScroll`.
- Opening (`00757b70`): `UIHackingFanHumLP`. A terminal that isn't unlocked
  (`00501ae0`: hacked, or opened with its password note) logs on first
  (`00758ad0`): "WELCOME TO ROBCO INDUSTRIES (TM) TERMLINK", "> LOGON
  ADMIN", "ENTER PASSWORD NOW", "> " and one `*` a letter of the hacking
  game's words for it (`00501270`); the "> " lines at the input rate with
  `UIHackingCharSingle` a character and `UIHackingCharEnter` at the end;
  0.5 s after each line; 2 s after the last the screen comes (a click on
  the background brings it at once).
- The screen (`00758d30`): the logon's lines hidden; first in the queue
  `sComputersHeader1`, `sComputersHeader2` and the server line, each
  centred (x = (width − 17 × length) / 2); then the welcome (the
  separator shows when it's typed), each item whose conditions pass on
  the placed terminal as "> text" (`_enabled` 0 until the queue is empty),
  "> `sComputersBack`" below a sub-menu, and the prompt "> ".
- An item (`00757f70`): `UIHackingCharEnter`; its result script runs on
  the placed terminal; its result typed beside the prompt and hidden after
  `iComputersResultDisplayTimeout` (5) s; a sub-menu is gone into; a note
  given with flag 0x01 when the player hasn't it ("Note Added: name",
  `sComputersAddedNote`) and shown: a text note in the zone a page at a
  time (`007590c0`: the zone text's `wraplines` less the welcome's lines,
  + 1, broken as `00a12fb0` breaks lines), a picture (`XNAM`) in the zone,
  a sound note's sound played, a voice note nothing (no handler); with no
  note, flag 0x02 fills the list again.
- The zone clicked: the next page, else back to the list, each item typed
  again with its conditions asked again (`00759560`), or filled again with
  flag 0x02.
- Code 10 (`007583f0`): out of a note, else back a screen ("Back" too,
  `00758a80`); from the first screen the menu closes. The viewer sends it
  for Tab and Escape.

After the password, the hacking menu hands over to this one
(`TransitionToComputersMenu`), so a hacked terminal logs on.

## Checks

Four `ui` tests on a hand-made menu file laid out like the game's (the
screen typing out and the items enabled; the logon with seven asterisks,
the 2 s and the skip; an item asking the world, a long text note in
pages, leaving the note and the result's timeout; a sub-menu and Back).
Live on the official data: `GSSchoolTerminal01Ref` after hacking it (the
logon, then "ROBCO INDUSTRIES UNIFIED OPERATING SYSTEM", "-Server 6-",
the SoftLock welcome, "> Disengage Lock"), and opened directly
(`--open-menu terminal:REF`) with a click on its item: "Clearance granted,
Unlocking..." beside the prompt and the list typed again (flag 0x02).

Not compared with the running original game.

## Not done

- Rendered terminals: by default (`bUseRenderedTerminals=1` in the
  shipped INI, `[RenderedTerminal]`) the game draws these menus and the
  hacking menu onto the terminal's own screen in the world, with the
  camera on it, scanlines and a screen light (`FORenderedTerminal`,
  `00705fe0`). Here they're drawn on the screen as with the setting off:
  the terminal menu's 4:3 area at the top left.
- The controller's cursor and keys (codes 1–4, 9).
- The tutorial message (`00718630(0x18, …)`).
- The cursor's exact place after list items (`_text_width`, `_text_y`).

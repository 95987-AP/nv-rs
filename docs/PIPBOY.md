# Pip-Boy: mouse and keyboard

Branch `claude/m1-pipboy`, 2026-10-05. Owner of the Pip-Boy menu code
(`crates/ui/src/pipboy/`, `viewer/src/pipboy.rs`). Executable: FalloutNV.exe
1.4.0.525. Decompiler exports stay private.

## What was wrong

- The mouse did nothing: no picking on the Pip-Boy screen, no mouse-over,
  clicks, wheel or drags, and the game's cursor was not drawn while the
  Pip-Boy was up.
- The arm was scaled for the world camera's field of view although its
  pieces are drawn by the first-person camera (`FIRST_PERSON_LAYER`, 55°),
  so the Pip-Boy was drawn larger than with `fPipboy1stPersonFOV` 47.
- DATA rebuilt the world map (re-centred, markers remade) and every list,
  and STATS every list, whenever any part of the game state changed (for
  example equipping an item): choices, scrolling and drags were lost.
- DATA's highlight-box title was skipped when the data rectangle was
  missing (one early return covered both).
- Tab closed on press; the game closes on release. F1/F2/F3 were unused.

## Traced behaviour (implemented)

| Behaviour | Addresses |
| --- | --- |
| Cursor on the screen: pick `pipboyscreen`, x = u·H·4/3 (`01078128` 1.333333), y = v·H; model buttons `PipBoyButton01..03` (+0xe8..), press stores the index (`011a0ba0`), release on the same one gives 1..3 | `007f8720`, caller `008761e0`, stored by `00709b80` (+0x4ac/+0x4b0/+0x4b4/+0x4b8) |
| Model button: `UIMenuMode`, then STATS/ITEMS/DATA | `0070c4a0` (0x0070d84f), `00704c10`, `007048f0`, `00704170` |
| No tile under the cursor with a Pip-Boy menu on top and the cursor off the screen | `007126c0`, `00717990`, `00717920` |
| Mouse over/off, click, drag, wheel via the interface | `0070c4a0`, `00717e70`, `00717ef0`, `00718080` |
| ITEMS: row 0x1d mouse-over chooses + card + `UIPipBoyScroll` when listindex changes; off clears card; click equips/uses; tabs 0x18..0x1c with the knob | `00780ff0`, `00781620`, `00780140`, `00782470` → `007fa0f0`, `007f8610` |
| STATS: rows chosen on mouse-over with per-page knob click; status buttons 0x1c/0x1d/0x1e (+0x35/0x37/0x39 hardcore) | `007dc1b0`, `007dfd20`, `007db380`, `007e0160`, `007e0060` |
| DATA: quest rows 0x17 (click = active quest unless finished), notes 0x18, radio 0x19; tabs 0x20..0x24; map click (id 4) presses the highlighted marker | `00798cb0`, `00796fd0` |
| DATA map tabs: highlight box centred on the pointer, cursor alpha 0 inside 0..850 × 0..500 (`01074f68`, `010301a8`), nearest marker within half the box height, `UIPipBoyHighlight`, "Companion" keeps the title | `0079a130`, `00799dc0` |
| F1/F2/F3 (DIK 0x3B..0x3D, raw keys): open on STATS/ITEMS/DATA, switch, or close on the shown one; Tab release closes | `0070c4a0`, `00a24180`, `0070f4e0`, `0070f690` |
| Menu class numbers: Stats 0x3eb, Inventory 0x3ea, Map 0x3ff | `00717920`, rtti vtables `0106ffd4`, `010739b4`, `01074d44` |

Labelled guesses: the menus' camera covering 1280 × 960 units over the
4:3 rectangle (so menu point = u × 1280, v × 960); the map position at
press (the MapMenu's fields +0x120/+0x124, compared in `00796fd0` case
0x1a; where they are stored is not traced) as the drag check; the
marker distance measured between screen centres (the game uses `_x`/`_y`
in the map frame).

## Tests

- `ui::pipboy::tests::the_mouse_chooses_equips_and_turns_tabs_on_items`
- `ui::pipboy::tests::the_models_buttons_and_the_world_maps_markers`
- `ui::pipboy::tests::the_status_pages_mode_buttons`
- `viewer pipboy::tests::the_pointer_lands_on_the_screens_picture`

Generated cut-down menus; no game files.

## NOT compared with the original game

No live comparison has been made. Not implemented: the fast-travel
question box (`00703e80` "%s %s?": travel happens at once), right-click
custom marker (click 0x0c), map zoom (`0079c530`/`0079c5a0`), Drop/Repair/
Mod/keyring, STATS limb picking (`007dc1b0` page 0), STATS/DATA mouse-off
(`007dc490` hides an unidentified tile; list choice kept), held mouse
button slots 0x20/0x24, swapped mouse buttons (`+0x1b4c`), the cursor's own
movement speed (the system pointer is used), and the Escape control.

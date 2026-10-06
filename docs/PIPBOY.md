# Pip-Boy: mouse and keyboard

Branches `claude/m1-pipboy` (2026-10-05) and `claude/m2-pipboy-live`
(2026-10-06). Owner of the Pip-Boy menu code (`crates/ui/src/pipboy/`,
`viewer/src/pipboy.rs`, `viewer/src/game_menus/asks.rs`). Executable:
FalloutNV.exe 1.4.0.525. Decompiler exports stay private.

## Why the mouse didn't work (found live, 2026-10-06)

The first batch was unit-tested only. Driving the running release viewer
with real Windows input (PowerShell, `user32` `SetCursorPos` /
`mouse_event` / `keybd_event`, screenshots by `CopyFromScreen`) showed:
hovering highlighted rows and tabs, but **no click ever happened**. The
Pip-Boy keeps the mouse from the rest of the viewer by clearing Bevy's
`ButtonInput<MouseButton>` every frame (`reset_all`); a cleared button is
no longer "pressed", so Bevy never reports it coming up, and the menus'
click (press and release over the same tile) never completed. The
Pip-Boy now reads the buttons' own events (`MouseButtonInput`), every
frame, open or not (`viewer::pipboy::mouse_buttons`, regression test
`a_click_finishes_although_the_buttons_are_cleared`). The cursor itself
was never grabbed or hidden (the viewer looks around only with the right
button held), so that was not the cause.

Also fixed while there: the Pip-Boy now runs after the game's own menus
and leaves the input to one open over it (a question, "how many?"); a box
closing over the Pip-Boy no longer makes the player ready to walk.

## Verified live (release build, injected input, screenshots)

| What | How |
| --- | --- |
| Tab opens; mouse-over chooses rows (SPECIAL list), tabs highlight | `GSDocMitchellHouse`, Tab, cursor moved over rows/tabs |
| Click a tab (S.P.E.C.I.A.L.) turns the page | left click on the tab |
| Model buttons ITEMS / DATA switch menus (lamp lights) | left click on the arm's buttons |
| DATA tabs by click (Quests) | left click |
| World map: highlight box follows, nearest marker named | `Goodsprings`, F3 |
| Zoom: wheel (in/out) and Page Down / Page Up | wheel notches, PgDn/PgUp (extended keys) |
| Right click on the map: "Do you want to set your marker?" Yes / No; Yes places the marker icon there | right click, then click Yes |
| Click a marker: "Do you want to travel to Goodsprings?"; No stays; Yes lowers the Pip-Boy, then travels (to the Prospector Saloon front) | left clicks |
| ITEMS right click on Stimpak (7): "How many?" over the Pip-Boy; arrows to 3, Ok; list shows Stimpak (4); the dropped reference comes into view | `--run "player.additem Stimpak 7"` |
| STATS healing mode: hover the damaged left leg (meter at 40%), click: Stimpak (3) → (2), the leg's meter to about 80%, health a tenth | `--run "player.damageav LeftMobilityCondition 60"`, F1 |
| ITEMS buttons: Mod lit with the pistol chosen, Repair dim; both dim with nothing chosen | F2, hover |
| Keyring: Misc ends with "Keyring" (its picture); click lists Becky Hostetler's Key and Van Graff Key with "Cancel E)"; E closes it | `--run "player.additem VFSVanGraffKey 1"` … |

Pictures are kept privately in `%USERPROFILE%\nv-re\work\pipboylive-2026-10-06`.

## Traced behaviour (implemented)

| Behaviour | Addresses |
| --- | --- |
| Cursor on the screen: pick `pipboyscreen`, x = u·H·4/3 (`01078128` 1.333333), y = v·H; model buttons `PipBoyButton01..03` (+0xe8..), press stores the index (`011a0ba0`), release on the same one gives 1..3 | `007f8720`, caller `008761e0`, stored by `00709b80` (+0x4ac/+0x4b0/+0x4b4/+0x4b8) |
| Model button: `UIMenuMode`, then STATS/ITEMS/DATA | `0070c4a0` (0x0070d84f), `00704c10`, `007048f0`, `00704170` |
| No tile under the cursor with a Pip-Boy menu on top and the cursor off the screen | `007126c0`, `00717990`, `00717920` |
| Mouse over/off, click, drag, wheel via the interface | `0070c4a0`, `00717e70`, `00717ef0`, `00718080` |
| ITEMS: row 0x1d mouse-over chooses + card + `UIPipBoyScroll` when listindex changes; off clears card; click equips/uses; tabs 0x18..0x1c with the knob | `00780ff0`, `00781620`, `00780140`, `00782470` → `007fa0f0`, `007f8610` |
| ITEMS Drop: right button going down with an item chosen and Tab up clicks 7 (`00781ba0`); quest item → `sDropQuestItemWarning`; count above `iInventoryAskQuantityAt` (5) → "How many?" from all of them (`007aba00`, callback `00780c50`), else one; dropped `fPlayerDropDistance` (100) + the item's size in front, one reference keeping the count | `00780140` case 7, `00780c50`, `009614b0` |
| STATS: rows chosen on mouse-over with per-page knob click; status buttons 0x1c/0x1d/0x1e (+0x35/0x37/0x39 hardcore) | `007dc1b0`, `007dfd20`, `007db380`, `007e0160`, `007e0060` |
| STATS healing mode: the pointer onto a damaged limb (the file makes only those targets) with Stimpaks (Doctor's Bags in hardcore) aims the body part controller at it, `UIPipBoyHighlight`, turns healing mode on; leaving the limb turns it off; a click on it presses the Stimpak's (hardcore: Doctor's Bag's) button. An effect of archetype 34 or 35 added while it's on is aimed at that limb's condition (`00823210` → `00589f50`, part → actor value `007df9c0`); a "value and parts" effect then gives that part all of it and health × `fMagicVACPartTargetedMult` (`0082b970`) | `007dc1b0` case 0, `007dc490`, `007db380`, `007e0230` |
| ITEMS keyring: keys (`KEYM`) never in the tabs (`00782620`); with keys carried the Misc tab ends with a row `sKeyring`, id 0x1e (`00782a90`, sorted last by `007824e0`); its row shows `item_keyring.dds`; clicked, `_KeyringOpen` and the keys listed (`00782810`); Cancel (10) or another tab closes it; Drop there plays `UIVATSInsufficientAP` | `00780140` cases 0x1e, 10, 7; `00780ff0` |
| ITEMS buttons: no item chosen, Equip/Drop/Repair/Hot key/Mod not targets; with one, Equip as equippable, Drop and Mod yes, Hot key unless ammunition, Repair when repairable (weapon or apparel below full condition, two or more carried or a mender from its `REPL` list carried unequipped, `00781860`) | `00781680` |
| DATA: quest rows 0x17 (click = active quest unless finished), notes 0x18, radio 0x19; tabs 0x20..0x24; map click (id 4) presses the highlighted marker when the map didn't move since the press (`00798480`, `MapMenu::DoDownClick` (Xbox PDB), stores the map's place) | `00798cb0`, `00796fd0` |
| Fast travel: `0093d660`'s refusal first; then "%s %s?" (`sTravelQuestion`, the marker's name), Yes / No (`00703e80`, callback `00798710`); Yes puts the Pip-Boy away and travels once it's down (`0070f690` → `00798a00`) | `00796fd0` case 0x1a |
| Player's own marker: right button on the map area (`0079a130`, `00a23a50(1,1)`, control 0xe up) clicks 0x0c; point = (pointer − map) ÷ map width, `MapToWorldCoord` (`0079c450`); `UIPopUpMapMarkerAdded ` (with the exe's trailing space); no marker: `sSetMarkerQuestion` Yes/No numbered from 3; else `sMoveMarkerQuestion` Move It / Remove It / Leave It; callback `00798840` sets (`00952e60`) or removes (`00952f90`); shown as `WorldMapQuestMarkerTemplate` with `glow_hud_compass_pc_marker.dds`, id 0x1c (`0079f360`); saved (`custommarker` line) | `00796fd0` case 0x0c |
| World map border: a place's share × 0.796875 + 0.1015625 on the picture (`01075010`, `01075008`), back × 1.2549 − 0.12745 | `0079c380`, `0079c450` |
| Zoom: wheel ÷120 > 0 in by 1.1 (`01056528`), Page Down in / Page Up out by 1.2 (`01018204`); magnification clamped to `fWorldMapMinZoom` 0.75 .. `MaxZoom` 5; marker, quest marker, arrow sizes lerped; the point under the window's middle kept; `UIPipBoyScroll` via the knob when it changed | `0079c530`, `00799790` 0x0f/0x10, `0079c5a0` |
| DATA map tabs: highlight box centred on the pointer, cursor alpha 0 inside 0..850 × 0..500 (`01074f68`, `010301a8`), nearest marker within half the box height, `UIPipBoyHighlight`, "Companion" keeps the title | `0079a130`, `00799dc0` |
| F1/F2/F3 (DIK 0x3B..0x3D, raw keys): open on STATS/ITEMS/DATA, switch, or close on the shown one; Tab release closes | `0070c4a0`, `00a24180`, `0070f4e0`, `0070f690` |
| Boxes over the Pip-Boy are ordinary menus on the main screen (only files whose `id` is `&pipboymenu;` draw on its screen); the HUD keeps its messages over the Pip-Boy's pages | `menus\*.xml`, `ui::hud::parts_for_menu` mode 3 |
| Menu class numbers: Stats 0x3eb, Inventory 0x3ea, Map 0x3ff | `00717920`, rtti vtables `0106ffd4`, `010739b4`, `01074d44` |

Labelled guesses: the menus' camera covering 1280 × 960 units over the
4:3 rectangle (so menu point = u × 1280, v × 960); the marker distance
measured between screen centres (the game uses `_x`/`_y` in the map
frame); the dropped item's size as half its `OBND` diagonal (the game's
`0050ebf0` isn't traced) and its height the player's; the drop's sound as
the item's put-down sound.

## Tests

- `ui::pipboy::tests::the_mouse_chooses_equips_and_turns_tabs_on_items`
- `ui::pipboy::tests::the_models_buttons_and_the_world_maps_markers`
- `ui::pipboy::tests::the_status_pages_mode_buttons`
- `ui::pipboy::data::tests::zooming_the_world_map_keeps_its_middle`
- `ui::pipboy::tests::the_right_button_drops_and_marks_the_map`
- `ui::pipboy::tests::the_keyring_opens_and_closes`
- `ui::pipboy::tests::items_buttons_follow_the_chosen_item`
- `ui::pipboy::stats::tests::healing_mode_aims_a_stimpak_at_a_limb`
- `world` `limbs::crippled_legs_slow_and_the_player_takes_half_limb_damage` (aimed Stimpak), `limbs::an_aimed_effect_keeps_its_part_in_a_save`
- `ui::pipboy::gather::tests::places_land_on_the_world_map_between_its_corner_cells`
- `world::map::tests::places_and_points_on_the_world_maps_picture`
- `world` `more_functions::the_player_drops_items_in_front`
- `viewer pipboy::tests::the_pointer_lands_on_the_screens_picture`
- `viewer pipboy::tests::a_click_finishes_although_the_buttons_are_cleared`

Generated cut-down menus and plugins; no game files.

## NOT compared with the original game

Nothing here has been compared side by side with the original game; the
live checks above are of the viewer only. Not implemented:

- STATS healing mode's aimed limb blinking (`007dbfe0` animating its
  alpha), the pad's X toggle and the buttons' texts with a pad
  (`007df620`), the Stimpak / Doctor's Bag buttons' `target` by
  `007e05f0` / `007e06f0`; "limb condition" effects (archetype 35, the
  Doctor's Bag) do nothing in the world yet, aimed or not.
- ITEMS: Repair (the Repair menu), Mod (the item mod menu), hot keys (1-8
  with an item chosen, `00781ba0`), the other Drop refusals (during an
  action, in the air, worn items that can't come off, no room), the
  drop's shape cast, ten headings and physics; dropped items can't be
  picked up again (made references aren't among the things E takes); the
  sort's ties by condition and equipped.
- DATA: the local map (rendered from above), quest markers and waypoints
  on the map, the radio playing, notes' audio, challenges, the custom
  marker on the compass and on the local map.
- The Escape control opens the game's start menu over the Pip-Boy
  (`0070c4a0` at `0070e651`, `007ce7a0`); the viewer quits on Escape.
- The Pip-Boy light lighting the place (only its cone on the arm shows),
  knobs, needle and buttons moving, the PC button-label textures, held
  keys repeating, swapped mouse buttons (`+0x1b4c`), the game's own cursor
  speed (the system pointer is used).

# Contributor pull request from Chazm (slaterain/nv-rs#11)

Merged 2026-10-06 on `claude/contrib-chazm` (from
`claude/overnight-integration` at `e1feadc`) with a real `git merge` of
`pr/11` (tip `f9bfd0d`, based on the old main `33dee45`; 46 commits by
`chazmm`), so the history and authorship stay. Chazm owns terminals, item
scripts, Repair and weapon mods, companions, Caravan, casinos and
performance ([CONTRIBUTING.md](CONTRIBUTING.md)). The pull request brought
no game files, executable bytes, recordings or oracle dumps (the hygiene
test's only finding in the worktree is the worktree's own `.git` file).

## What came in

| Area | Topic page | Notes |
| --- | --- | --- |
| Bink intro movie: `crates/bink` (video bit-exact with the `nv-bink` oracle, audio, colour), `nvinspect <movie.bik>`, `viewer/src/movie.rs` (`PlayBink` on `--new-game`; `--movies` / `--no-movies`, skipped with `--screenshot`) | [MOVIES.md](MOVIES.md) | `research/nv-oracle/nv-bink`, `research/pdb/msf_repack.py` (tools, no game content) |
| Crafting: `world::crafting`, `ui::menus::recipe`, `viewer::game_menus::recipe`, `nvinspect recipes` | [CRAFTING.md](CRAFTING.md) | replaces the Dead Money version, below |
| Terminal hacking (`world::hacking`, `ui::menus::hacking`), the terminal menu (`ui::menus::computers`), `ForceTerminalBack` | [HACKING.md](HACKING.md), [TERMINALS.md](TERMINALS.md) | `--open-menu terminal:REF`, `hacking:REF` |
| Item scripts: one script per item, `OnAdd`, `OnEquip`/`OnUnequip`, `OnDrop`, `GameMode`, `RemoveMe`, `GetContainer`, `Drop`, `DropMe` | [ITEM_SCRIPTS.md](ITEM_SCRIPTS.md) | `Runner::run_item_scripts` each update |
| Repair: the Pip-Boy's repair screen, merchants' repair services, armour wear and breaking | [REPAIR.md](REPAIR.md) | `--open-menu repair:REF` |
| Weapon mods: slots, effects, models, the Pip-Boy's mod screen, attack noise and silencers (`world::noise`) | [WEAPON_MODS.md](WEAPON_MODS.md) | |
| Companions: `OpenTeammateContainer` (container mode 3), the companion wheel | [COMPANIONS.md](COMPANIONS.md) | `--open-menu teammate:REF`, `wheel:REF` |
| Caravan: cards, rules, the opponent's AI, bet, the menu and its 3D table | [CARAVAN.md](CARAVAN.md) | |
| Casinos: shared rules, slot machine rules, tiles and 3D scene | [CASINO.md](CASINO.md) | the script event is still only reported in the viewer |
| `GetShouldAttack`, `GetIsAlignment`, `GetContainer`, `SetItemValue` | | |
| Pip-Boy Drop with the pad's X, `--pad`, `--pipboy-keys`, `--menu-keys`, `--menu-click` | | |
| Performance (`eb8ff21`) | | see below |

## Overlaps and which side was kept

| What | Ours (already merged) | Chazm's | Kept |
| --- | --- | --- | --- |
| Crafting (`world::crafting`, the recipe menu, its tests and fixtures) | Dead Money contributor's: rules from the data, order/skill/click labelled guesses, a script's `ShowRecipeMenu` gated behind `NV_GUESSES` | traced (`005a8110`, `00727680`, `00727920`, `00728c10`, `007278c0`, `007284f0`, `RecipeMenu` Xbox PDB) | **Chazm's**, whole: the menu now opens for scripts with guesses off. Kept from ours: `--open-menu recipes:CATEGORY` (now the player's crafting menu) and `ShowRecipeMenu`'s vendor rule (`005deb10`: on a talking activator, its speaker; nobody: "Recipe menu called with NULL vendor!") and its `MenuMode 1077`. `nvinspect craft` dropped for Chazm's `nvinspect recipes` (needs a category; listing every category had no traced counterpart). Both sides' tests kept: the Dead Money world is `testdata::crafting::notes` (with its `GetHasNote` recipe), its four world tests are `crafting.rs`'s `dead_money` module and its six menu tests are in `ui::menus::recipe`, adapted to the traced rules: the list and the filter are by name (`00728c10`, `007278c0`; theirs kept the records' order), "can it be made" is `CanMakeRecipe`'s count rather than a refusal, and making closes the menu (`007284f0`) rather than refreshing the list. |
| `ShowSlotMachineMenuParams` / `ShowBlackJackMenuParams` / `ShowRouletteMenuParams` | `Shown::CasinoMenu` + `MenuMode` number | `Event::Casino` (`world::casino`) | **Chazm's** event, with ours' check that the form is a casino (`CSNO`, else "Invalid EditorFormID ...") and `MenuMode` 1080-1082 (`casino::Game::menu`). |
| `OpenTeammateContainer` | `Shown::TeammateContainer`, opened as a plain container | `Event::TeammateContainer` → container mode 3 with carry weight | **Chazm's**, using ours' actor test (made actors count). |
| `ForceTerminalBack` | only while the terminal menu is open (`005dc4e0`) | always an event; the terminal menu takes it | **Chazm's** event with ours' open-menu test; the terminal menu marks itself open while an item's script runs. |
| `AddCardToPlayer`, `GetContainer`, `RemoveMe` (item scripts) | `Runner::on_add` for picked-up references, a card set | full item-script system, cards in `world::caravan` (`AddHead`) | **Chazm's**; ours' traced `RemoveMe CONTAINER` (`005b53d0` passes the container to the holder's remove-item) added to his, the script following the item. `on_add`, `Runner::container`, `more_functions::carried` and `more_functions::menus` removed. The Dead Money card test now runs through `run_item_scripts`; its card fixture gained the `INTV` suit and value real cards have. |
| `GetShouldAttack` | `scripting.rs` (ours, `0059ed30`) | the same translation in `more_functions` | **ours** (same code; his copy removed). |
| `GetKnockedState` | 2 for an essential who is down | 0 always | **ours**. |
| ITEMS "Repairable" (`00781860`) | `ui::pipboy::gather::repairable` | `world::repair::can_repair` | **Chazm's** (shared with his repair screen, rules in `world`). |
| Pip-Boy Drop | mouse right button, "How many?" through `asks`, the player's drop (`00780c50` → `placed::drop_item`) set on the ground | pad X (`xbuttonx`), refusals `world::items::drop_refusal`, his own "How many?" (`game_menus::pipboy_drop`) and the script drop path | **one path**: ours' click/ask/drop with his pad X (`ItemsMenu::pad_button`) and his refusals (quest item, in the air; equipped during an action needs the action, not tracked). `pipboy_drop` and `Menu::PipboyDrop` removed. The player's drop now carries the items' scripts (`OnDrop`, `scripts_follow`) and `OnUnequip`. |
| Repair and Mod buttons | lit by `00781680`, no screen | screens | Chazm's screens, opened from ours' ITEMS click handler (ids 8, 0x13). The screens take keys only: pointer and wheel do nothing while they're up. |
| Dropped things to pick up | `placed::dropped_items` (per place, wired in the viewer) | `scripting::made_items_here` (per cell) | **ours**; his test moved to it. |
| `--run` lines | all at once | one a frame | **ours**, with the scripted items run after each line (what he needed: a line sees the `OnAdd` of the one before). The acceptance routes depend on the timing. |

## `eb8ff21` (lighting hitch, NPC pathing, mouse look)

- Shared outdoor light buffer (`viewer/src/shared_light.rs`, shaders):
  **taken**; a performance change (one buffer instead of rewriting every
  material each game minute). The lockpicking and Caravan 3D pieces no
  longer take the sun's light (`MenuLit`). Not yet checked across an
  outdoor-to-indoor door in this merge.
- Navmesh grid index for `triangle_at`: **taken** (same answers, tested).
- Islands: **taken**, placed before our traced search (`006cd670`) rather
  than before the ends are resolved: the search crosses only neighbours,
  so different islands can't join, and our planner has no partial paths.
- Sandbox candidates read once per cell: **taken** (record-only data;
  dead/enabled still checked each time).
- `--fps` longest frame: **taken**.
- 3D menus off the shared light (`MenuLit`): a behaviour change, taken as a
  fix: the pieces have their own lights (`lockpick::menu_lighting` from the
  menu's models) and the hour's light overwrote them outdoors. The Dead
  Money Vigor tester's pieces (`game_menus::vigor`, built the same way)
  were marked too, so all three menus agree.
- Mouse look with the cursor captured: **not taken** (ours already turns
  the view without a button and hides the cursor, `grab_cursor`); only his
  Windows detail is: the pointer confined and put back in the middle each
  frame, since Windows can't lock it.

## Not traced / left as reported

- The casino script event is printed, not shown (Chazm's own state).
- Repair and mod screens: no mouse.
- `RemoveMe` doesn't tell an equipped instance apart (`004bfda0`).
- Dead Money: `ShowRecipeMenu`'s guess gate in
  [CONTRIB_PLAYCON.md](CONTRIB_PLAYCON.md) no longer applies.

## Checks (2026-10-07)

- Root: `cargo test --workspace` (1392 passed), clippy and fmt clean.
  Viewer: `cargo test` (142 passed), clippy and fmt clean, release build.
- Live, release viewer on the official data: GSDocMitchellHouse loads and
  plays; `--open-menu hacking:GSSchoolTerminal01Ref` (20 words of 7
  letters, the intro typing); `--open-menu terminal:GSSchoolTerminal01Ref`
  (the Server 6 SoftLock screen, "> Disengage Lock"); `--new-game
  --movies` plays `FNVIntro.bik` (1280x720, 8692 frames, letterboxed).
- `scripts/acceptance.ps1`: doc, vcg02 and vms16 pass (first run).
- Only by tests: crafting, repair, weapon mods, companions, Caravan and
  casinos on screen; the 3D menus' lighting outdoors; an outdoor-to-indoor
  door with the shared light.

## Follow-ups for Chazm

- The casino menus (`Event::Casino`): shown, not just printed.
- Mouse for the Repair and Mod screens.
- `RemoveMe` on an equipped instance (`004bfda0`).
- Crafting: the item card (`00728da0`) and the tutorial help.
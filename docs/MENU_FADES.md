# Menus fading in and out

Every one of the game's own menus fades in when it opens and out when it
closes, run by the interface manager, not by the menus. nv-rs: `ui::fade`
(the rules), `viewer/src/game_menus/mod.rs` (`Screen::show_new`, `close`,
`end_fades`; the drawing in `draw_menus`).

## What the game does (FalloutNV.exe 1.4.0.525)

Names from the Xbox 360 prototype (Xbox PDB); its decompile of the named
functions is in the private `~/nv-re/decomp/x360_menufade.c`.

- **State.** `Menu::xFadeState` (`+0x24`, the same on PC),
  `Interface::FADE_STATE`: shown 1, fading out 2, hidden 4 (a new menu's,
  `Menu::Menu`), fading in 8.
- **Time.** The menu tile's `menufade` (the menu tile's constructor
  `00a1ef30` sets 0.25; of the vanilla files only `loading_menu.xml` gives
  another, 0.75), or `explorefade` when `menufade` is 0; 0.25 for a menu
  without a tile (`0101622c`).
- **Opening** (`Menu::PrepForVisibility`, `00a1dc20`): the menu tile
  hidden, then `StartFadeIn` (`00a1db20`) unless the caller asks for it
  at once. Of the 34 callers only the HUD, the Pip-Boy's inventory, DATA,
  stats, item-mod and repair pages open at once; every other menu fades
  in.
- **Closing** (`Menu::StartFadeOut`, `00a1d910`, called by 37 menus'
  close code): nothing unless the menu tile is shown; else the fade out
  starts, and a menu that marked itself to leave the menu stack (trait
  6002) leaves it now (`00706fd0`), so the keys go to the menu below.
  The message box, "how many?" and two others close with
  `InstantFadeOut` (`00a1d9e0`) instead while a rendered menu (the
  Pip-Boy, a rendered terminal: `00707af0`) is up: hidden at once, gone at
  the frame's end. The terminal and the hacking menu set `menufade` to
  0.75 before closing when the player leaves (`00757ea0`, `00766aa0`); the
  hacking menu handing over to the terminal's (`0076a540`) fades over the
  usual 0.25. The rendered terminal's power button deletes the menu
  outright (`007ffd50`).
- **The fade list** (interface manager `+0x164`): menu, seconds gone,
  seconds it takes; adding one replaces the menu's earlier one
  (`007164c0`). Every frame before the menus run (`0070b8f0` →
  `00716320`), and again whenever a fade starts (`00706f70`), every fade
  moves on by the frame's seconds (`011f6394`+0xc; divided by the time
  multiplier `011ac3a0` in V.A.T.S. playback); finished ones leave. How
  far a fade is (`00716660`): 1 with none, -1 when it takes no time, else
  gone / total held to 0..1 (`0040ebd0`, `00404010`).
- **After the menus run** (`00711ea0`, Xbox
  `InterfaceManager::PostIdleStuff`), for each menu: fading in, part-way:
  the tile shown, faded to the amount; whole: shown at 1, state shown.
  Fading out, part-way: faded to 1 − the amount; whole: state hidden, and
  the menu deleted if it marked itself to leave the stack (the loop stops
  there for the frame), else its tile hidden. While any fade is part-way
  (`+0x11`) the game stays in menu mode when the last menu has closed.
- **The fade itself** (`00712450`, `InterfaceManager::RecursiveFade`):
  down the menu's scene graph, each piece's material alpha becomes its
  tile's `alpha` / 255 × the fade, held to 0..alpha (0 when the fade is
  under 0.0001); a tile with `disablefade` is instead shown only at a
  whole fade and nothing under it is touched; the children of a tile with
  id 9000 are left alone. (No vanilla menu file uses either.)
- **Input** (`0070c4a0`): the pointer's moving onto and off tiles,
  presses, clicks and drags reach a menu only while it's shown (the tile
  under the pointer is let go of, without a "mouse off", while its menu
  fades); the keys go to the deepest menu not fading out or hidden
  (`00720e60`), so a menu fading in takes keys.

## In nv-rs

All the game menus the viewer shows now fade in over 0.25 s and out over
0.25 s (the terminal and hacking menus 0.75 s when the player leaves,
drawn on the rendered terminal's screen while they fade). Closed menus are
drawn, not run, until their fade ends; the player stays held (menu mode)
until then. The message box and "how many?" close at once over the
Pip-Boy or a rendered terminal; the power button still removes the
terminal at once. Changed behaviour: every game menu (message box,
dialogue, container, barter, recipe, repair services, companion wheel,
Caravan, how many, level-up, traits, character generation, text entry,
sleep/wait, Vigor tester, start/pause, hacking, terminal) where before
they appeared and vanished at once. The start menu's own page fades
(`START_MENU.md`) run inside this one. Log lines `Menu NAME fading in/out
(S s).` and `Menu NAME faded out.` show them.

Not done: the image space effect the game fades along with the first menu
opened and the last one closed (the image space manager's effect 15,
`007123f0`/`007123a0`); a menu closed before it was ever shown (whose
`StartFadeOut` does nothing in the game, leaving it to fade in again) is
removed at once here; the menus' own pictures aren't faded by the
material alpha but by the drawn alpha (the same for the flat pictures the
menus use).

## Checks

The dialogue menu fading out under a service menu and back in
(`00763ff0` / `007640a0`, [DIALOGUE.md](DIALOGUE.md)) is the same
`StartFadeOut` / `StartFadeIn` on the same list: a menu kept open, not
marked to leave the stack, so hidden when faded out
(`DialogMenu::service_opened` / `service_closed` take the screen's
`Fades`).

`ui::fade` tests (the times, fading in then shown, fading out and gone,
kept menus hidden, a hidden menu not fading out, instant fade out, every
fade moving on when one starts, a fade taking no time, the loop stopping
after a menu goes, the alpha rule with `disablefade` and id 9000),
`ui::menu`'s `letting_go_of_a_menu`, the hacking and terminal tests'
`menufade` 0.75 on leaving, and the viewer's `game_menus::tests` (fade in,
pointer only once shown, fade out drawn and holding the game, instant and
immediate closes).

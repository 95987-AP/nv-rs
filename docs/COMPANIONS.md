# Companions: trading things with them

A companion's "let's trade" lines call `OpenTeammateContainer` (19 calls in
19 scripts of the official master), which opens the container menu on
their things in its mode 3. Until 2026-10-06 nv-rs didn't carry it out, so
nothing opened.

## Read from `FalloutNV.exe` 1.4.0.525

| What | PC |
| --- | --- |
| `OpenTeammateContainer` (opcode 0x11C8) | `005d9430` → `00709470(…, 3)` |
| The container menu's mode (`+0x80`) | set by `0075b310` |
| Opening, by mode | `0075bc80` |
| Closing, by mode | `0075b750` |
| Moving a thing (the companion's room) | `0075dc80` |
| Their carry limit, what they carry | `008a0c20`, `00577250` |
| A companion's line from a topic | `0075ec60` |

- `OpenTeammateContainer [n]`: on a person or creature (vtable +0x100)
  that's the player's teammate (`00566950`), or on anyone when `n` isn't 0.
- Mode 3 opening (`0075bc80`): the title the companion's name (fitted as a
  container's, `0075e8c0`), `DRSTraderOpen`, the companion says their
  `FollowersTrade` topic's line (`0075ec60`), Take All (id 10) hidden. The
  key that presses Take All does nothing in modes 3 and 4 (`0075d430`).
  Their list is their own things through the usual filter (`0075c280`:
  `00719ef0` with `0075cfc0`).
- Giving them something (`0075dc80`, from the player's list): what they
  carry (`00577250`: their things' weight, at most what they can carry)
  plus its weight × how many, above what they can carry (`008a0c20`: Carry
  Weight, actor value 13, through perk entry 22 "Get Max Carry Weight",
  which only the player's perks touch) is refused: "<name> " +
  `sTeammateOverencumbered` ("can't carry any more.") on screen, and their
  `FollowersOverburdened` line.
- Closing (`0075b750` case 3): the companion sorts out what they wear
  (`00606540` for people, `005f9e00` for creatures), `DRSTraderClose`, the
  dialogue taken up again (`007640a0`).

## Here

`OpenTeammateContainer` raises `Event::TeammateContainer(who)`; the viewer
opens `ui::menus::container` with `mode` 3 (`game_menus::container`,
`menus::Menu::Teammate`): the name, the sounds, the topic lines
(`Event::Talk` without a conversation), the room check
(`world::items::carry`, `has_room`). `--open-menu teammate:REF` opens one
for testing.

Checks: `crates/world/tests/companions.rs` (who it opens on, a companion's
room), `ui::menus::container::tests::a_companions_things_have_no_take_all`.
Seen in the viewer in the Mojave Outpost barracks (`--open-menu
teammate:RoseofSharonCassidyREF`): "Cass" over her things (13 caps, an
empty whiskey bottle), no Take All; clicking the 9mm pistol gave it to her
(the player's weight 17 → 15).

## Not done

- The companion choosing what to wear after trading (`00606540`: their
  outfit and best gear worked out again); a thing given stays unworn.
- The line on the menu's own subtitle (`CM_Subtitle`); the line is said as
  a `SayTo` would say it.
- (Done since: `DropMe` — the companions' faction-outfit scripts drop what
  they're given, `OnAdd CraigBooneREF` … `DropMe`; see
  `docs/ITEM_SCRIPTS.md`.)

# Companions: their wheel, and trading things with them

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

## The companion wheel

Using a teammate brings up their wheel of orders, `CompanionWheelMenu`
(class 1075, `menus\companion_wheel_menu.xml`). Until 2026-10-06 nv-rs
talked to them instead.

| What | PC |
| --- | --- |
| Using a teammate | `005fa330`, `00607990` → `00709470(8, …)` |
| Whether they can take orders | `00754d90` |
| Opening, the switches | `00754de0` |
| The pointer on a slice | `00755760` → `007557d0` |
| The context line | `00755dc0` |
| A click | `007552e0` (buttons `007561b0`, `00756490`, `007563a0`, `00756960`, `00756930`, `007560c0`, `007568d0`, `007562b0`) |
| Keys | `007556b0` |
| Each frame | `00755480` |
| A topic said, its scripts run | `007573d0` (subtitle `00757690`), `007575b0` |
| Closing, talking after | `00755270`, `00754c60` → `00756980` |

- When: the player using a teammate after the pickpocketing (sneaking
  never picks a teammate's pockets) gets the wheel if they're alive and up
  (`00884480`) and wouldn't attack the player (`008b06d0`,
  `GetShouldAttack`), else nothing at all; they're never simply talked to.
- The switches are the companion's script variables (`008c1740`, -1 when
  their script lacks one): aggressive `FollowerSwitchAggressive` == 1,
  keep distance `IsFollowingLong` == 1, ranged `CombatStyleRanged` == 1,
  following `Waiting` == 0. Buttons 0, 2, 5 and 7 show their other picture
  (`user11`) when on (2 when waiting).
- The slices are `radial` tiles: the pointer is on one when its angle from
  the screen's centre (`00a216b0`: 0 up, clockwise) is within `user2` to
  `user3` and its distance within `user4` to `user5` (76 to 332). The
  file's slices run 30–150° on the right (0 Aggressive/Passive, 1 Use
  Stimpak, 2 Stay/Follow, 3 Talk To) and 210–330° on the left (4 Back Up,
  5 Near/Far, 6 Open Inventory, 7 Ranged/Melee).
- The pointer on a slice: `UIPipBoyScroll`, the highlight (`user10`) moved,
  a switch's button showing what a click would make it, the preview text
  (the exe's `sCWheel…` strings) and the context line: `HP  75/150` and
  `Stimpak  3` on two lines on the Stimpak, `Wg  40/150` on the
  inventory, the weapon's name on Ranged/Melee.
- A click (then `UIMenuOK`): a switch flips and the companion says the
  topic for its new state, its begin and end result scripts run on them at
  once: `FollowersTacticsCombatAggressive`/`…Passive` (then the menu sets
  `FollowerSwitchAggressive` itself), `FollowersWait`/`FollowersLetsGo`,
  `FollowersTacticsDistanceLong`/`…Default`,
  `FollowersTacticsCombatRanged`/`…Melee`. The game's lines do the work
  (Boone's `FollowersWait`: `set CraigBooneREF.Waiting to 1`, `evp`, …;
  `…CombatRanged`: `SetCombatStyle FollowersCombatStyleRanged`).
- Use Stimpak (`00756490`): needs a Stimpak (default object 0) in the
  player's things and the companion hurt (health below full, `00893590`)
  or a limb crippled (actor values 25 to 30 at 0). Hurt, the Stimpak's
  effects go on them; one Stimpak leaves the player; conditions 25 to 31
  are restored by 1000; they say their `Regenerating` line (no scripts).
- Talk To closes the wheel and starts the conversation; Back Up
  (`008a7760`: the default package 0x27, away from the player) closes it;
  Open Inventory closes it and says `FollowersTrade`, whose result script
  is `OpenTeammateContainer` (above); Exit (11) closes it. Enter clicks
  the chosen slice (Exit with none); the pad's B (code 10) is Exit.
- The line said shows on the wheel's subtitle (id 12) until its voice ends,
  or for its length × `fNoticeTextTimePerCharacter` (0.06 s) without a
  voice; the frame empties it after.

Here: `world::companions` (`wheel_allowed`, `switches`, `variable` /
`set_variable`, `say_topic`, `heal_with_stimpak`),
`world::living::pickpocket::Use::Wheel` / `Nothing`,
`ui::menus::companion_wheel` (the menu), the UI engine's `radial` tiles
(`ui::menu::in_slice`), the viewer's `game_menus::companion_wheel`
(`--open-menu wheel:REF`; Escape or Tab leave it). Checks:
`crates/world/tests/companions.rs::the_companion_wheel`,
`ui::menus::companion_wheel::tests`.

## Not done

- The companion choosing what to wear after trading (`00606540`: their
  outfit and best gear worked out again); a thing given stays unworn.
  What they are made to wear (`EquipItem`) or lose (trading it away) is
  drawn: [NPC_GEAR.md](NPC_GEAR.md).
- The line on the container menu's own subtitle (`CM_Subtitle`); the line
  is said as a `SayTo` would say it.
- The wheel's Back Up (`008a7760`: default package 0x27 put on them); it
  only closes the wheel.
- Choosing a wheel slice with the pad's stick (`00755480`, `00755c50`).
- The wheel picks the topic's line once for both saying it and running its
  scripts; the game picks it twice (`0061a720` in `007573d0` and again in
  `007575b0`), which differs only for a line said once.
- (Done since: `DropMe` — the companions' faction-outfit scripts drop what
  they're given, `OnAdd CraigBooneREF` … `DropMe`; see
  `docs/ITEM_SCRIPTS.md`.)

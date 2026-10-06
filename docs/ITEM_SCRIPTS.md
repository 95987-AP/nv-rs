# Scripts on items: their own runs, `OnAdd`, `RemoveMe`

An item record can carry a script (`SCRI` on the item). The official
master has 201 `OnAdd` blocks: ammunition bundles that give their rounds
and remove themselves (`Case10mmAddScript`: `Player.AddItem Case10mm 25`,
`RemoveMe`), schematics and recipes that give a note, key cards, and the
main quest's note from Novac (`VMQ01NoteObjectSCRIPT`: objectives,
`VStoryEventKnowsKhansInvolved`, `RemoveMe`). Until 2026-10-06 nv-rs ran
none of them, and `RemoveMe` (92 calls) stopped any script that reached it.

## Read from `FalloutNV.exe` 1.4.0.525

| What | PC |
| --- | --- |
| Script block table (name, block type, its test) | `0118e2f0`, 0x28 bytes an entry |
| `OnAdd`'s test: the event's bit 1, the block's argument the container | `005ca200` → `005ca1c0` → `005a8ef0` |
| Setting an event on an item's script | `005ac750(container, extra list, mask)` → `005a8e20` |
| A container's `AddItem` (sets `OnAdd`, mask 1) | `00574fa0`; pick-up `00574b30` |
| Adding items with a notice (scripted items one at a time, each a script of its own) | `004821a0` |
| A reference's script run: its own, then a container's or actor's items' | `00565870` → `004d2480` |
| Running a script (events cleared afterwards) | `005ac1e0` → `005e2590` → `005e0d20` (`005a8ea0`) |
| `RemoveMe`, `DropMe` | `005b53d0`, `005b5860` (command table `011922d8`, `01192300`) |

Event bits set by the code (`005ac750`'s callers): 1 `OnAdd`, 2 `OnEquip`,
4 `OnDrop`, 8 `OnUnequip`.

- Adding a scripted item flags `OnAdd` on that item's script with the
  container (`00574fa0`); the block runs on the container's next script
  run (`00565870`, for actors and containers `004d2480` runs every
  scripted stack's script with the container), its argument empty or the
  container (`OnAdd Player`), then the events are cleared.
- `004821a0` adds scripted items one at a time, each with a fresh script
  (`0055a2f0`) run once before the add.
- `RemoveMe` (`005b53d0`): the container the script runs in removes one of
  the item (`RemoveItem`, vtable `+0x17C`, the item's own stack), and the
  inventory run stops (`00952c30`). `DropMe` drops it into the world the
  same way.

## Here (`world::scripting`)

Each scripted item gets a script of its own (`GameState::item_scripts`:
holder, item, its variables, its events waiting), made when it's added at
runtime (`GameState::added`: script `AddItem`, leveled lists through it,
`AddItemHealthPercent`, picking up, a person picking up food) with `OnAdd`
for the holder; moved between holders (`move_item`, `take_all`,
`RemoveAllItems` into a container) it keeps its variables and sees `OnAdd`
again; equipping (`equip`) and unequipping (`unequip_item`: the Pip-Boy,
scripts, selling, giving away) add `OnEquip` / `OnUnequip`. A holder's
own contents get scripts only when they move or are equipped.

`Runner::run_item_scripts`, at each `update`: items no longer held lose
their scripts; every item the player holds runs, and any other with events
waiting; a run takes the script's blocks in order: `GameMode`, and the
event blocks whose events wait and whose argument is empty or names the
event's reference; the holder is the reference it runs on; the events are
then cleared. `RemoveMe` takes one of the item from its holder, and its
script with it.

Check: `scripted_items_see_their_onadd` (`crates/world/tests/scripting.rs`):
a bundle giving 25 cases added twice to the player (50 cases, no bundle);
in a chest nothing (`OnAdd Player`); taken from the chest, it runs; a
bundle whose `OnAdd` only marks it and whose `GameMode` gives 5 cases and
removes it (as `PrimerShotshellAddScript`); a hat whose `OnEquip` /
`OnUnequip` set a global (as the faction outfits' warnings).

## Dropping (`Drop`, `DropMe`, `OnDrop`)

- `RemoveItem` (`005750a0`, the actor's slot 0x17C) flags `OnDrop` (event
  4) on the item's script for the holder whatever becomes of it
  (`005ac750(…, 4)`): put in a container, the script goes with it and runs
  `OnDrop Player` there; destroyed (a plain `RemoveItem`), it never runs (as
  the snow globe's script notes). The Platinum Chip's `OnDrop Player`
  removes its note; given back, its `OnAdd Player` adds it again.
- With its drop flag (`Drop item count`, `005b58d0`; `DropMe`, `005b5860`,
  one of the item its script runs for) the things become a new reference
  (`004c37d0` → `004c6dd0`) at the holder's position plus (0, 50, 30)
  turned by their rotation (50 in front, 30 up), turned as they are, with
  an extra count for more than one; the script goes with it.
- Here: `more_functions::placed::drop_into_world` (a made reference with a
  `count`, saved), `GameState::scripts_follow` / `drop_item`, `moved`
  adding `OnDrop` for the giver; `DropMe` moves the running item's script
  to the new reference with `OnDrop` waiting; scripts on dropped things run
  for their events; `GameState::pick_up` brings a dropped one's script
  back (with `OnAdd`); `scripting::made_items_here` lets the viewer pick
  dropped things up. The player's heading is kept
  (`GameState::player_heading`) so things land in front of them.
- Checks: `crates/world/tests/dropping.rs`.

## Not done

- The Pip-Boy's Drop button (ITEMS id 7, the X button: its refusals
  `sDropQuestItemWarning`, `sDropEquippedItemWarning`, `sNoJumpWarning`,
  `sCantRemoveWornItem`, `sNotEnoughRoomWarning`, and "how many?" inside
  the Pip-Boy); the free spot the game's physics finds for the player's
  drop (`009614b0`) and the fall; the inventory run stopping after
  `RemoveMe` or `DropMe`.
- Items held by others run only for their events; the game runs them with
  their holder's script run (`00565870`) when it's processed.
- A crippled arm dropping its weapon doesn't send `OnUnequip`.
- Item scripts aren't saved.

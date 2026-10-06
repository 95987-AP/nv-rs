# Scripts on items: `OnAdd` and `RemoveMe`

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

`GameState::added` queues one `OnAdd` an item for scripted items added at
runtime (script `AddItem`, leveled lists through it, `AddItemHealthPercent`,
`RemoveAllItems` into a container, a container emptied or moved from,
picking up, a person picking up food); a holder's own contents aren't
added. `Runner::run_item_adds` runs the queue at the next `update`: a fresh
copy of the item's script, its `OnAdd` blocks whose argument is empty or
names the holder, the holder as the reference it runs on. `RemoveMe` takes
one of the item from its holder.

Check: `scripted_items_see_their_onadd` (`crates/world/tests/scripting.rs`,
a bundle giving 25 cases): added twice to the player, 50 cases and no
bundle; in a chest nothing (`OnAdd Player`); taken from the chest, it runs.

## Not done

- `DropMe` (dropping needs items placed in the world, which `Drop` lacks
  too), and the run stopping after `RemoveMe`.
- `OnEquip`, `OnUnequip`, `OnDrop`, and items' `GameMode` blocks (an
  inventory item's script runs every time its holder's does).
- Variables kept per item between runs (only `OnAdd`'s fresh copy here).

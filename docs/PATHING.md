# Long paths: the navmesh info map and the long way

Branch `claude/m2-long-paths` (on `claude/overnight-integration`),
2026-10-06. How the game plans a path whose goal is on a navmesh that
isn't loaded, read from FalloutNV.exe 1.4.0.525 in Ghidra; names marked
(Xbox PDB) come from the Xbox 360 prototype's symbols. Raw exports stay
private (`%USERPROFILE%\nv-re\work\longpaths-2026-10-06`). Status words:
**implemented** (code exists), **tested** (generated regression), **not
compared** (nothing here has been checked against the running game).

## What was wrong

`VCG02SunnyTravelToWell1` sends Sunny from behind the Prospector Saloon
(about -68250, 5800, square -17,1) to `VCG02SunnyWellMarker1` (-67298,
-8968, square -17,-3): about 14,800 units, four squares south. The viewer
searched only the navmesh of the 3 × 3 squares around the player (an
untraced choice), found no triangle under the goal and gave no path, so she
stood still. Out of sight people walked a navmesh path of the 3 × 3 squares
around themselves, which fails the same way.

## How the game does it

| What | Address | Status |
| --- | --- | --- |
| `NAVI` is read into one `NavMeshInfoMap` (Xbox PDB) kept in memory: `NVER`, then per `NVMI` a new `NavMeshInfo` (0x5c bytes, `006b4b50`): flags (+8), navmesh (+0), place (+4, a worldspace or interior cell), grid word (+0xc), rough position (+0x10), island data when flags & 0x20, preferred factor (+0x20, NVER > 9). `NVCI` (`006b51a0`, NVER > 4): navmesh, then three counted lists: navmeshes joined (+0x24), a second list (+0x34, NVER > 10), doors (+0x44) | `006b5d80` (chunks `0x494d564e`/`0x4943564e`) | implemented, tested |
| The grid word holds y then x: navmesh 001639F7 (cell 000DEBB6, `XCLC` 1, -17, vertices near x 4096..8192) reads -17, 1 | data | implemented, tested |
| Every plugin's version of the record adds its entries: the DLCs override `FalloutNV.esm`'s `NAVI` 00014B92 with their own few entries, so the winning version alone has one entry | data (6129 entries with all versions, 1 with the winner) | implemented, tested |
| A path request (`PathingRequest` (Xbox PDB), types by `006e2f80`) is solved in `Pathfind.cpp`: type 0 → `006d0b10` → `006c8e10`: resolve start and goal to navmesh infos; same info → nodes start, goal; else the info search (`006c94c0`) | `006d0900`, `006c8e10`, `006c8f50` | implemented |
| `NavMeshInfoSearch` (Xbox PDB, vtable `0106c22c`), an `AStarSearch<NavMeshInfo const*, TESObjectREFR*>`: start node cost 0; open list ordered by g + h (`006b9710`; buckets `006f4790` put every node with h ≥ 0 in one list, a new node before equal ones, `006b94a0`); goal test info == goal info (`006b8c20`) | `006b8c50`, `006b9180`, `006b8ec0`, `006b8fe0` | implemented, tested |
| Edge costs (`GetNodeConnections`, `006b8490`): from the current info (at the start point for the start info) to each joined info (at the goal point for the goal info): distance × k × 3 for the first list, × k for the second, k = clamp(1 − 100 × the current info's factor, 0.1, 1); estimate 0.1 × distance from the info's rough position to the goal when start and goal are outdoors in one worldspace (`00690800`), else 1 | `006b8490`, constants `01021928` (3.0), `01017a40` (100), `0101e2bc` (0.1), `0101ffa0` (0.1) | implemented, tested |
| A cheaper way to a node takes it up again; a neighbour improved through the node's own parent ends the search as failed; a failed search returns the route to the node with the least estimate (`006b8f40` with "closest" set) | `006b9180`, `006b8f40`, `00996ae0` | implemented, tested |
| The route becomes virtual nodes (`VirtualPathingNode` (Xbox PDB), `006f4e40`: an info's node sits at its rough position): start, each info between, goal | `006c94c0`, `006c90d0`, `006c9920` | implemented, tested |
| Only the run of nodes from the actor's whose cells are attached (cell state 6, `00450ff0`; a teleport-door node, flag 4, ends it) gets a detailed path (`006ca0e0` → `006ca850` → `006cb950`: one navmesh search over the run) | `006c9fc0`, `006ca0e0`, `006ca500` | implemented (detailed path from the walker to the last attached node), tested |
| A path solution moving to a high-process actor builds the detailed part (`006d5420` → `006c93f0`); moving to a lower one drops it (`006d53b0` → `006c93a0`): the virtual handler walks the virtual nodes only | `009dbdc0`, `006d5420`, `006d53b0` | implemented |
| The virtual handler (`009ea8a0`) walks node to node at walk speed (run in combat) × the time budget, facing each leg, the last within the request's radius | `009ea8a0` | implemented, tested |
| The detailed handler extends its path at its end (`006d5340` → `006c9170`) only when that end is a teleport door node (handler +0xdf, set in `009e0470`); otherwise the walk ends there, and the travel procedure asks again whenever it is idle short of its place (`008e5e90`) | `009e8000`, `009e0470`, `008e5e90` | implemented: the walker stands at the last attached node until the attached squares change |

So a person on screen walks toward a far place as far as the attached cells
go (to the rough position of the last attached navmesh on the route,
resolved onto the navmesh) and waits there; when the player's grid moves
the attached cells change and the walk goes on; once their own cell is
detached they are in a lower process and walk the virtual nodes out of
sight in game-time steps (`world::movement::offstage_seconds`).

## In the code

- `world::ai::navinfo`: `NavInfoMap` (load, search), `NavInfos` (which
  navmesh a point is on, `virtual_path`), `attached_run`, `walk_nodes`.
- `world::ai::NavMesh`: `owners` (each triangle's navmesh), `navmesh_at`,
  `load_navmeshes`, `closest_point` (the nearest navmesh point to a node;
  the game's `PathingLocation::ResolveToClosestNavmeshAndTriangle` (Xbox
  PDB) is not traced in detail).
- `world::ai::move_offstage` walks the planned virtual nodes (kept from
  update to update, planned again when the goal or place changes).
- Viewer `ai.rs`: `CellNav` is the attached cells' navmesh (the
  `uGridsToLoad` grid around the game's grid centre, `00452580`, its loaded
  squares) instead of the 3 × 3 squares; `long_walk` plans the long way for
  someone idle short of a place off that navmesh (again only when the
  attached squares change, or a moving target moved more than
  `fAIMoveDistanceToRecalcFollowPath`); a start or goal off the navmesh
  (out of sight people walk straight between rough positions, which can
  lie in a navmesh's hole) is taken to the nearest navmesh point; people
  outdoors beyond the attached squares are hidden and left to
  `move_offstage`, and shown again when it brings them into an attached
  square.
- Viewer `scripts.rs`: `player.MoveTo <someone>` goes where the state has
  them, not to their editor cell (needed to drive the player after Sunny).

## Not done or unresolved (labelled in code)

- The search's door edges (`006b8490`'s third list, for an actor's
  requests; 409600 more for a locked door): places are still changed
  through `world::ai::door_toward`.
- What a failed info search falls back to (`006c8f50` → `006c9b20`): no
  path here.
- Which navmesh a point is on when its cell has several and the point is on
  none of them (nearest rough position here); the "island" data.
- The detailed handler's arrival radius for a path ending short (`009e0470`
  uses 40² or 80² in cases `006b05d0`/`006e2330` not identified); the
  request's radius is used.
- Data without `NAVI` (generated test worlds): out of sight people walk the
  navmesh path around them as before.

## Checks and runs

Regressions: `world` `ai::navinfo::tests` (costs ×3/×1, preferred
factor, failed search, `NVMI`/`NVCI` reading, node walking),
`crates/world/tests/long_paths.rs` (generated five-square world with a
`NAVI` record overridden by an empty `DeadMoney.esm` version: map reading,
virtual path, attached run, detailed path to the last attached node,
walking out of sight), viewer `ai::tests::
a_far_place_is_walked_toward_as_far_as_the_attached_cells_go`.

Installed data (throwaway check, not committed): the map has 6129 entries
with every `NAVI` version (1 with the winning version alone); Sunny's
route from behind the saloon to `VCG02SunnyWellMarker1` is 5 nodes, 14,826
units, one navmesh per square south (-17,1 → -17,-3).

Viewer runs: the VCG02 section of
[GOODSPRINGS_ROUTE.md](GOODSPRINGS_ROUTE.md). Nothing here is compared
with the original game. **Next action:** record in the original game, with
the player standing behind the saloon, where Sunny stops on her way to the
first well (the rule traced here says: the last attached navmesh on her
route), and whether she walks on out of sight once the player turns away.

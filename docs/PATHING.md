# NPC paths and movement

Two batches: how a path is found and walked on the attached navmesh, and
how people move through the cell's collision (`claude/m2-npc-nav`,
below), then the long way over the navmesh info map
(`claude/m2-long-paths`, further down).

## Navmesh search, path smoother and character controller

Branch `claude/m2-npc-nav` (on `claude/overnight-integration` at
`915d220`), 2026-10-06. Read from FalloutNV.exe 1.4.0.525 in Ghidra; raw
exports private in `%USERPROFILE%\nv-re\work\npcnav-2026-10-06`. Same
status words as below.

### What was wrong

Maintainer's playtest: people don't always take the right routes and walk
through things. Causes found:

1. **No collision.** The viewer set a walker's position straight along
   its path points, at the path's height (`ai.rs` `step`). Nothing stopped
   them: furniture, rocks, walls at corners, each other (only the
   avoidance wait/way-round, which is a path rule, not collision).
2. **Not the game's search or smoother.** A* between triangle middles
   with the full distance as estimate and no costs, then a "funnel"
   (shortest line through the shared edges, touching the corridor's
   corners): routes the game doesn't take and lines grazing every corner.
3. **No stuck handling**, so a blocked walker stood forever.
4. Load doors were walked to at the door reference's own position, inside
   its frame.

### Traced and implemented

| What | Address | Status |
| --- | --- | --- |
| A path request (`PathingRequest` (Xbox PDB), constructor `006e2420`): radius 35 by default; an actor's is 0.6 × its width (`006e29f0` → `008be280` → `00885140`: bounds' x extent × scale); may swim (+0x9d), 3 tries (+0xa8) | `006e2420`, `006e29f0` | implemented (`world::ai::request_radius`: the base's `OBND` × `XSCL`; the game's bound vfuncs `00933630`/`00933700` take a process bound first, not traced) |
| Straight line first only without avoid nodes; with both ends ≥ 25 above the land (always indoors) the lines ± the radius to either side must stay on the navmesh too | `006cc5e0`, `006cd1f0`, land height `0045cbc0` | implemented, tested |
| The navmesh search (`NavMeshSearch` (Xbox PDB), vtable `0106b63c`): the A* template of the info search; nodes at triangle middles; cost = middles' distance × 0.01 preferred (0x40) × (avoid nodes' summed cost + 1) × marked obstacles' cost × (0.5 + 0.5·clamp(1 − edge length/512)) × 4 water / 5 into or out of water × 100 locked door × 100 edge to keep off; skips links that don't lead back, links of kind 1, flag 0x10 for wide actors; estimate 0.01 × distance to the goal triangle's middle | `006a6b70`, `006a6fa0`, `006a6ef0`, `006a79c0`, `006a8010`, `006a7e00`, `006dc990`/`006dc780` | implemented, tested (`world::ai::navsearch`) |
| Triangle flags: `NVTR` flags + cover flags read as one u32 (`00692950`); door flag 0x1000 is set at run time on triangles under a loaded door (`006997e0`), never in the data (checked on the installed data: 1363 portal triangles carry 0x400, none 0x1000) | `00692950`, `006997e0` | read; the door registration isn't done, so doors cost nothing extra |
| Smoother choice: `bUseAlternateSmoothingForPrime` (1) and radius > 150 → `0069f010`; else `bUseOldPathSmoothing` (0) → `PathSmootherPOVSearch` | `006cd8a0` | read; the POV smoother implemented |
| `PathSmootherPOVSearch` (Xbox PDB): points = start, goal, corridor corners (shared edges' ends by side, circles of 1.2 × the radius where the side bends in by more than ∓0.05, and each side's first and last), avoid nodes and marked obstacles (their radius + 1.2 × the walker's); A* over (point, way round) with common tangents, corners passed on their side, arcs ≤ 3.4657 rad, steps no steeper than 1.2, clear with side lines at 0.9 × the radius (the ends half across and half along), the reached point's navmesh height within −64..180; cost (1 − 0.9·preferred share)·(arc + length + obstacle chords × (share × 7 + 1)); estimate 0.1 × distance; runs of 4 failures; output the tangent points, a corner's second one dropped after an arc under 50 | `006ad770`, `006adcb0`, `006ae370`, `006aeba0`, `006aefc0`, `006af2b0`, `006b0ad0`, `006b0f40`, `006bb570`, `006b25f0`, `006b2060`, `00698320`, `006b2c00`, `006b1df0`, `006afcc0`; constants `010290f0` 1.2, `0106b9e8` 0.9, `0106ba00`, `0101e2c0` 50, `0106b9f0` −64 | implemented, tested (`world::ai::smoother`) |
| A failed smoothing keeps off the corridor's edges narrower than twice the radius and the last corner's edge (× 100), and the search goes again, up to 3 tries; no smoothed way: no path | `006b2fc0`, `006ad770`, `006cc5e0` | implemented, tested |
| The mover: the path handler's move vector (speed × dt, cut on sharp turns `009e4800`, turned toward the steering point `009e3560`, shortened to the way left `009e0a00`) goes to the mover (`009ddc00`, +0x10) and so to the character controller every actor gets (`00930c70`), as the player's | `009e0a00`, `009e3520`, `009e3560`, `009ddc00`, `00930c70` | implemented (viewer `move_body`: `physics::Character` with the player's shape, other people and the player as cylinders); one controller update per frame assumed as for the player |
| Stuck test: moved less than √0.125 of the last frame's move; blocked time (× 0.2 against an actor) past 1.5 s → stuck for the rest of the path: an obstacle record on the triangle (radius, cost 1, flag 0x80000000), the walk fails and the way is asked for again | `009e4cf0`, `00691510`, `006915d0`; constants `010924a8`, `01016ff0`, `01018a90` | implemented, tested (`world::movement::Stuck`, viewer `unstick`) |

### Not done or unresolved (labelled in code)

- The door triangles' run-time registration (`006997e0`), so locked doors
  aren't avoided by cost; walking to a load door uses the navmesh point
  nearest the door within the door reference's travel radius (`00678670`
  rule; that the door is taken as a travel location is an inference).
- An end off the navmesh: the game builds a ray-cast way onto it first
  (`PathBuilder::BuildPathToNavMeshSearchStart` (Xbox PDB)); here such an
  end is moved to its triangle and needs only the line itself, no side
  lines.
- 006af500's start/goal edge obstacles, the turn-angle request (+0xa2),
  the search radius (+0x78), avoid nodes of kind 2, a start inside a
  circle (chord), `006a0660` at short arcs, `0057b460` doors, the last
  try's acceptance of a path ending near the goal.
- The stuck test's "someone in the way is asked to make room" branch.
- Obstacles marked on triangles never expire (the navmesh is rebuilt
  when the attached cells change).
- Light clutter: in the game the controller pushes bodies lighter than
  `fMoveLimitMass` (95); here every model's collision is solid
  (havok rigid bodies are another batch's), so e.g. Easy Pete never gets
  to his eating marker at the saloon (stuck 25 short, asking again every
  1.5 s) and some gangers stand stuck in the gunfight.
- Collision still loading under someone: no controller until there is
  ground within 256 below (a viewer bridge, not game behaviour); fall
  damage for people isn't applied.
- Callers in `sitting.rs`/`fighting.rs` (other batches') still ask with
  the default radius; `fighting.rs` falls back to a straight line when no
  path is found.
- Search cost on the main thread: Sunny's 7 km route takes ~0.1 s in a
  release build.

### Verified live (viewer, installed data; nothing compared with the original)

- **Doc Mitchell to the door** (`GSDocMitchellHouse --stage VCG01 110`):
  the first search's corridor goes through a 46-wide edge his circles
  can't pass; the retry keeps off it and Doc walks the west rooms to the
  door in 36 s on his controller and starts talking there. Before the
  retries his partial path made him turn on the spot forever.
- **Sunny, Back in the Saddle** (VCG02 opening lines, bottle hits by
  console, the player `MoveTo`'d after her): she walks the long way to
  the first well, the end action sets stage 30, her sneak line, the walk
  closer and stage 45 follow by themselves.
- **Gunfight** (`VMS16`, `bTrudyHelp` 1, the documented line): settler 04
  walks to the saloon door and goes through it; the fight runs; stuck
  walkers are listed above.

Regressions: `world` `ai::navsearch::tests` (costs and flags, links
leading back, avoid nodes and obstacles), `ai::smoother::tests`
(tangents and arcs, going round an avoid node at its radius + 1.2 r,
a corridor too narrow for the side lines), `ai::tests::
a_path_turns_the_corner_round_the_inside_corner`, `movement::tests::
walkers_held_back_for_a_second_and_a_half_are_stuck`; viewer
`ai::tests::walkers_are_moved_by_their_controller_and_a_wall_stops_them_till_they_are_stuck`,
`walkers_go_round_each_other_not_through`.

**Next action:** record Doc's walk to the door and Easy Pete's eating
approach in the original game, and trace `006997e0` (door triangles) and
the start/goal ray-cast paths.

## Long paths: the navmesh info map and the long way

Branch `claude/m2-long-paths` (on `claude/overnight-integration`),
2026-10-06. How the game plans a path whose goal is on a navmesh that
isn't loaded, read from FalloutNV.exe 1.4.0.525 in Ghidra; names marked
(Xbox PDB) come from the Xbox 360 prototype's symbols. Raw exports stay
private (`%USERPROFILE%\nv-re\work\longpaths-2026-10-06`). Status words:
**implemented** (code exists), **tested** (generated regression), **not
compared** (nothing here has been checked against the running game).

### What was wrong

`VCG02SunnyTravelToWell1` sends Sunny from behind the Prospector Saloon
(about -68250, 5800, square -17,1) to `VCG02SunnyWellMarker1` (-67298,
-8968, square -17,-3): about 14,800 units, four squares south. The viewer
searched only the navmesh of the 3 × 3 squares around the player (an
untraced choice), found no triangle under the goal and gave no path, so she
stood still. Out of sight people walked a navmesh path of the 3 × 3 squares
around themselves, which fails the same way.

### How the game does it

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

### In the code

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

### Not done or unresolved (labelled in code)

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

### Checks and runs

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

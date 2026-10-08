# Performance: measurements and review notes

The viewer's performance work (PR #12, `claude/perf-latest`) and its
follow-ups (`claude/perf-streaming`). The rule throughout: nothing drawn or
played may change; every change is measured before and after, and
screenshots must match within the noise of two runs of the same build.

Measured on the contributor's machine: RTX 5090, Ryzen 7 9800X3D,
2560×1440 at 360 Hz, Vulkan. **The 2026-10-07 numbers below were taken
while four other agents were compiling on the same machine**: they are
medians of alternating runs (old, new, old, new) and show the direction
and rough size of each change; re-measure on a quiet machine before
quoting them as frame rates.

## How to measure

- Frame rate: the viewer's `--fps` prints frames per second, the mean
  frame time and the longest frame of each 2 s window. Skip the first two
  windows (loading).
- Routes: the acceptance routes of `scripts/acceptance.ps1` (Doc's house,
  `VCG02` Back in the Saddle, `VMS16` Ghost Town Gunfight) with the same
  arguments plus `--fps`.
- Streaming: fly out of Goodsprings, `WastelandNV --at
  -68250,5800,8480,180 --key-at 5 wheel=3 --key-at 6 w:40 --fps` (the
  wheel speeds the fly camera up, so many squares load). Walking south from
  there runs into a wall after a few seconds: not a streaming test.
- Traces: `cargo build --release --features bevy/trace_chrome
  --target-dir target-trace` in `viewer/`; the trace JSON (5–6 GB for
  40 s) lands in the working directory. With tracing on, `println!` can
  stall behind the trace's disk writes: a long frame right next to a print
  is a trace artifact. Use `--fps` runs without tracing for frame numbers.
- Screenshots: `--screenshot`, with `--freeze-ai --no-hud` for faces and
  `--cloud-time 58.022` outdoors. Two runs of the same build differ: sky
  and grass drift, tumbleweeds, idle head poses, and Goodsprings' road
  decals flip between two looks from run to run (overlapping decals sort
  in an unstable order).

## PR #12 review points (2026-10-07)

### Shared textures and tinted faces

Places share the textures already on the GPU (`UploadedTextures`,
`Spawner::texture`), keyed by (path, colour space, layers, compression,
anisotropy). Tinted textures are not mixed up: a face's tinted skin is
built under the name `"<base> + FaceGen <tint file>"`
(`cellview` `TextureData::plus_face_tint`), a hair's layered texture
`"<base> + layer <layer file>"` (`with_layer`), glow products `"a x b"`
(`times`), and the tint files are per NPC
(`facemods\falloutnv.esm\<form id>_0.dds`), so the key carries what the
texture was made from. Hair colour is a material value (`hair_tint`), not
in the texture. Nothing writes into a cached image after upload (the HUD's
`get_mut` images are its own; the vigor tester, lock pick and swaps
upload directly).

The review found a real bug in the cache's lifetime instead (fixed in
`ba8e864`): it shared through `Assets::get_strong_handle`, which in Bevy
0.16.1 fails for render-world-only images once extraction has removed the
main world's copy (so later places uploaded their own copy), and whose
extra handle count extraction forgets (`remove_untracked` clears
`duplicate_handles`), so a texture shared within one frame was freed when
its first holder went while others still drew with it. Instrumented, a
75 s flight out of Goodsprings: 3,417 shares, 7,665 re-uploads, 527
textures freed while held. The cache now keeps weak references to the
holders' own handle (`Weak<StrongHandle>`): alive exactly while some
place holds it, across frames. Tested with Bevy's asset events.

### Screenshots before and after

Before = the integration build without PR #12; after = PR #12 + `ba8e864`.
Eight shots: Doc's room and Doc's face (`GSDocMitchellHouse`), Trudy and
Sunny Smiles (Prospector Saloon), Chet (General Store), Easy Pete,
Goodsprings ahead and looking down. Mean absolute difference / share of
pixels differing by more than 8 of 255:

| shot | before vs before | after vs after | before vs after |
| --- | --- | --- | --- |
| Doc's room | 0.00 / 0.00% | 0.00 / 0.00% | 0.00 / 0.00% |
| Doc's face | 0.34 / 0.95% | 0.53 / 1.99% | 0.21–0.49 / 0.55–1.75% |
| Trudy | 0.02 / 0.04% | 0.00 / 0.00% | 0.04–0.05 / 0.11–0.16% |
| Sunny | 0.08 / 0.15% | 0.68 / 3.04% | 0.21–0.69 / 0.68–3.18% |
| Chet | 0.12 / 0.24% | 0.14 / 0.31% | 0.09–0.11 / 0.16–0.23% |
| Pete | 0.13 / 0.30% | 2.67 / 10.52% | 1.37–2.95 / 6.24–11.02% |
| Goodsprings | 0.74 / 3.00% | 0.07 / 0.34% | 0.03–0.75 / 0.12–3.06% |
| looking down | 1.36 / 6.67% | 2.12 / 8.46% | 2.48–3.45 / 10.38–13.48% |

Every difference image shows idle head poses (Pete's face shading moves
with his head), grass sway and tumbleweeds; no texture, tint or lighting
differences. Mailbox presentation, bindless materials and "no light
clusters" change nothing drawn.

### Frame times on the acceptance routes

Same arguments as `scripts/acceptance.ps1` plus `--fps`; all routes pass
in both builds. Median frames per second over the run's 2 s windows (10th
percentile in brackets). **Busy machine** (other agents compiling; a
Ghidra headless run may have overlapped the first runs); one or two runs
each:

| route | before | after (PR #12 + `ba8e864`) |
| --- | --- | --- |
| Doc's house (`doc`) | 264, 252 (234, 228) | 474, 504 (426, 470) |
| Back in the Saddle (`vcg02`) | 132 (44) | 273 (71) |
| Ghost Town Gunfight (`vms16`) | 113 (28) | 240 (195) |

## Streaming hitches (`claude/perf-streaming`)

- **Player body and view relit, not rebuilt** (`d0bbcc0`). Every square and
  every distant-object block that came on screen rebuilt the third-person
  body and the first-person view from scratch (models, textures, face
  tint, animations): `update_player_body` 65–90 ms and
  `update_view_model` 20–30 ms per square. Now only their materials are
  made again with the new light. Each 2 s window's longest frame while
  flying out: 130–230 ms before, 45–115 ms after.
- **Outdoor collider gathered without re-measuring** (`1fe77bf`). The
  loaded squares' colliders are gathered into one on every square change;
  each triangle was checked and bucketed again. Now the squares' buckets
  are taken over, renumbered. ~190 gathers of up to ~240 k triangles in
  40 s: ~5.0 s in all (longest 43–46 ms) before, ~1.9 s (longest
  14–16 ms) after.
- **Materials kept when their lights don't change** (`5e7f8eb`). Every
  square change gave every loaded square's materials their lights again
  through `get_mut`, so the render world prepared them all again. Now
  only materials whose lights differ are touched. With the previous
  change: each 2 s window's longest frame while flying out, median 75 ms
  before, 36–39 ms after; windows with a frame over 50 ms 20 → 4–7 (of 26).

## Found, not changed

- **Minimized window, 60 fps.** With mailbox presentation the viewer
  drops to 59 fps as soon as it's minimized, and stays there after it's
  restored and brought to the front (fifo: no drop when minimized, but the
  same 59 after restore and focus in one run of two; immediate: ~161).
  Traced: the main thread idles ~13 ms a frame inside winit's event wait,
  update 4–5 ms, render 3.4 ms, present 0.03 ms. Bevy 0.16 on Windows runs
  each update on `RedrawRequested` (`bevy_winit` `state.rs`), and once the
  window has been minimized those arrive at 60 Hz. Not fixable from the
  viewer without patching `bevy_winit` (`UpdateMode::Reactive { wait: 0 }`
  still waits for a redraw after one update).
- **What lights the player's body is timing-dependent.** Outdoors the
  place lighting the body and first-person view take is whichever square
  or distant-object block loaded last (`lod_objects.rs` spawns blocks with
  `spawn_parts(None)`, which sets the place lighting too, though the
  comment there says "the last square loaded"). Instrumented: rebuilding
  ended on a block's light in 5 of 5 runs, relighting on a square's in 4
  of 5. Visible as brighter skin in third person. Fixing it changes what's
  drawn, so it's left for a traced decision about which light the game
  uses.
- **A fleeing stand-in.** `00171B3E` (base `NVProspectorMaleADEADJohnny
  StandIn`, a placed body with ragdoll data) runs from a gecko near
  Goodsprings; while it finds no way to flee, it searches three flee paths
  every frame (6–9 ms a frame for a few seconds). Whether it should be
  standing at all is a behaviour question, not a performance one.

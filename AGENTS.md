# nv-rs: session instructions

Goal: faithfully reimplement Fallout: New Vegas in Rust/Bevy using the user's
own game installation, with a stable, efficient 64-bit runtime, mod support
and an architecture that supports VR.

Read `docs/MILESTONES.md` first. Read only relevant source and reference
sections. For research work, `docs/METHODOLOGY.md` describes the method and
`research/*/README.md` the tools. `README.md` documents features and commands. Previous instructions
are preserved verbatim in `docs/ENGINE_REFERENCE.md` and
`docs/CLAUDE_REFERENCE.md`; their relevant rules still apply, but their
historical next-step lists do not determine current priorities.

## Working rules

- Derive behavior and values from game files, compiled shaders, Ghidra or
  recordings. No stand-ins or invented behavior; leave unsupported features
  out. Label unresolved guesses in code and topic findings.
- Since 2026-10-05 (decisions in `docs/adr/`): names, types and layouts from
  the Xbox 360 prototype symbols may be used, marked `(Xbox PDB)` (ADR-0002),
  and decompiled logic may be translated into Rust, marked with its address
  (ADR-0003). This replaces the older reference-doc rule "never copy
  decompiled code into the project". Raw decompiler exports, databases and
  executable bytes are still never committed.
- For behavior investigations, start with FNV's Ghidra decompilation, then
  inspect the associated game data and verify in the original game. The user
  explicitly prefers this order over trial-and-error viewer changes.
- Keep provenance: executable address/build, asset/record or recording,
  branch conditions and a regression test for each behavioral fix.
- Rules belong in world/physics/preview/cellview; viewer handles presentation
  and input. Menus use game XML through ui. Core crates have no external
  libraries. Viewer is its own workspace, pinned to Bevy 0.16; consult 0.16.1.
- Preserve the player/camera, aim/head, action-input and per-view boundaries
  in `docs/VR.md`. VR feature adaptations there are proposals.
- Never redistribute game files or modify the user's installation.
- Focus on the active milestone. Small fixes come first only for milestone
  blockers, crashes/data loss or explicit user requests. Keep other work in
  the deferred list.
- Distinguish implemented, tested and compared against the original game.
  Module existence and coverage counts do not prove playable completeness.
- Update the milestone tracker after each batch with evidence, blockers and
  one next action. Detailed research belongs in topic files, not this file.
- Claude and Codex share this file and the milestone tracker. Record active
  processes, changed files and unfinished checks in the linked topic handoff;
  do not maintain competing backlogs. Use the checked-out branch and GitHub
  milestone issues as the shared queue. Use small, bounded agents only when useful;
  use a cheaper capable model and keep one owner for each edited file.
- Explain outcomes and uncertainties in plain English.

## User customizations and repository updates

- Document every change made by the user or at the user's request in the
  customization register below during the same batch. Record the requested
  behavior, affected files, implementation/verification status and any commit,
  PR or topic-document reference that is available. Keep detailed evidence in
  the linked topic document. These explicit preferences take precedence over
  the default retail-fidelity goal for the affected behavior.
- Before pulling, merging, rebasing, switching to an updated checkout or
  replacing a local play build, read this register and inspect Git status and
  the incoming changes. Preserve both committed and uncommitted user work;
  make a recoverable Git commit or patch backup before an operation that could
  overwrite it. Never discard, reset or overwrite that work to simplify an
  update.
- Every updated checkout and local play build must retain the registered user
  customizations unless the user explicitly chooses otherwise. Carry them
  forward onto the updated code and verify the affected behavior; a clean Git
  merge alone does not establish preservation. Report what was preserved,
  what was checked and any remaining uncertainty.
- If an incoming change contradicts, replaces or removes a customization,
  including a behavior conflict without a textual merge conflict, notify the
  user before applying the conflicting part. Explain the existing behavior,
  the incoming behavior and the choices (keep the customization, accept the
  incoming change, or combine them where feasible). Wait for the user's
  choice; do not resolve it automatically. Independent, compatible work may
  continue while that choice is pending.
- Update this register whenever the user changes or retires a preference.
  Keep it through upstream updates; do not treat missing upstream entries as
  permission to remove local preferences. Add any newly discovered user work
  before proceeding with an update.

### Customization register

| Date | User change to preserve | Files and evidence | Current status |
| --- | --- | --- | --- |
| 2026-10-09 | FPS counter: visible at the top right by default, F3 toggles it, refreshes every 0.25 seconds, follows HUD visibility and is hidden in screenshots; retain existing `--fps` console reporting. | `viewer/src/main.rs` (`FpsText`, `FrameCounter`, `report_fps`, setup, screenshot exclusion and help text). | Implemented in a staged local change; included in the checks reported in `docs/NOTIFICATION_CARDS.md`. Separate live FPS verification is not recorded. Preserve this uncommitted work during updates. |
| 2026-10-09 | Subtle notifications: acknowledgement-only notices become nonblocking right-side cards, expire after five seconds of visible time, and queue overflow; questions retain their choice menu. | `viewer/src/game_menus/{message,mod,notifications}.rs`; evidence and handoff: [NOTIFICATION_CARDS.md](docs/NOTIFICATION_CARDS.md), branch `codex/subtle-notifications`. | Implemented; root and viewer checks and live cards/automatic clearing verified. Explicit user-requested deviation from retail. |
| 2026-10-09 | Local playtest launcher uses the current viewer release build after rebuilding, while keeping its existing selection menu and save folder. | Local ignored `Launch-Playtest.cmd` and `playtest/nv-rs-experimental-2026-10-03.2-windows-x64/Play.cmd`; [NOTIFICATION_CARDS.md](docs/NOTIFICATION_CARDS.md) launcher follow-up. | Implemented; launched through the menu and verified the running executable is `viewer/target/release/nv-viewer.exe`. Preserve this routing when replacing the local playtest. |

## Machine and checking

Project: current checkout, Windows/PowerShell is the tested platform.
Game Data: the contributor's own installed Data folder; never commit it.
Reference cell: GSDocMitchellHouse. GPU: RTX 5070 Ti Laptop, Vulkan.
Research/tools: `%USERPROFILE%\nv-re`, especially findings and decomp.
Read the archive's Ghidra/recording sections when needed.

Inspect Git status before edits. Use a branch and pull request;
do not claim commits, merges or remote synchronization without verification.

For code batches run appropriate checks, including:

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
cargo build --release
```

Check/build the viewer separately from viewer/; root checks exclude it.
Run it with the Data path and reference cell; read startup diagnostics.
Tests generate inputs from scratch, using testdata fixtures.

Play copy: `%USERPROFILE%\OneDrive\Desktop\nv-rs-play`. Reproduce F12
reports in reports/NNN from report.txt in the original game and compare.
After merged code batches pass checks, the maintainer packages official builds
using `scripts/package-playtest.ps1`; see docs/MAINTAINING.md. A local play
copy may have its own publication helper outside the repository.
Documentation-only work does not require play publication.

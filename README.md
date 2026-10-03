# nv-rs

A from-scratch Rust and Bevy project exploring how to run Fallout: New Vegas using game data from an installation you own. The engine code is original; this repository does not include game files and never modifies your installation.

The project is experimental and under active development. The current focus is the opening sequence and persistent world (milestone M1). Some parsers, rendering, world loading, simulation systems and menus exist, but that does not establish a complete playable game. The base-game campaign is not complete, and VR support and performance targets have not been validated. See [the milestone tracker](docs/MILESTONES.md) for current status and evidence.

The project is maintained by **slaterain**. It is a personal project; maintainers make final decisions about scope and changes. Community help is welcome through forks and pull requests; no engine switch or mod port is expected of anyone.

## Try a playtest

Download a Windows x64 ZIP from [Releases](https://github.com/slaterain/nv-rs/releases), extract it, and run `Play.cmd`. Provide your own Fallout: New Vegas `Data` folder. Official plugins are used by default; saves and F12 reports stay in the package's `userdata` folder. These are experimental builds, not complete playthroughs. Read the [playtesting guide](docs/PLAYTESTING.md) first.

## What exists so far

The project includes game-file readers and inspection tools; indoor/outdoor rendering and streaming; collision, actors and animation; scripting, dialogue and quest infrastructure; combat and progression systems; audio; original-data menus, Pip-Boy and V.A.T.S.; and custom saves. Implementation and verification vary by system.

The opening segment from the tester trigger through Doc's instruction to the SPECIAL menu has been exercised live. Full opening/campaign acceptance, Doc's gaze and choreography, and VR remain unfinished. Loading plugins is not a promise of general mod compatibility. Native NVSE DLLs are not supported, and original-game saves are not interchangeable with nv-rs saves.

## Build

Install stable Rust, then build the core workspace from the repository root:

```sh
cargo build --release
```

The real-time viewer is a separate workspace pinned to Bevy 0.16. Build it from `viewer/`:

```sh
cd viewer
cargo build --release
```

The core tools and test fixtures do not require game data. Running the viewer and inspecting real assets require a legally obtained Fallout: New Vegas `Data` directory. See [playtesting](docs/PLAYTESTING.md) for the current launch workflow.

## Project notes

- [Technical reference](docs/TECHNICAL_REFERENCE.md) preserves the detailed historical README and feature descriptions.
- [Milestones](docs/MILESTONES.md) records active work, blockers and evidence.
- [Contributing](CONTRIBUTING.md) explains how to propose code and research changes.
- [Playtesting](docs/PLAYTESTING.md) describes local verification.
- [VR architecture notes](docs/VR.md) record boundaries and proposals; they are not a claim of headset support.
- [Governance](docs/GOVERNANCE.md) describes project decisions.

The project is licensed under MIT or Apache-2.0, at your option. Fallout and Fallout: New Vegas are trademarks of their respective owners. This project is not affiliated with or endorsed by Bethesda Softworks, ZeniMax Media or Obsidian Entertainment.

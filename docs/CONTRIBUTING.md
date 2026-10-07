# Contributing to nv-rs

Several people work on nv-rs at once, each with their own AI agents. This
page says how to do that without breaking each other's work. The project
rules themselves are in [AGENTS.md](../AGENTS.md): read it first and give
it to your agents.

## One repository, small pull requests

- The one source of truth is `main` on `slaterain/nv-rs`. Fork it, branch
  from the latest `main`, and open pull requests against it. Nobody pushes
  to `main` directly; the maintainer merges.
- One system or one fix per pull request. A pull request that touches many
  unrelated areas, or adds thousands of lines across systems, will be sent
  back to be split. Stacked branches are fine if each one is reviewable on
  its own and says what it is based on.
- Rebase on `main` at least once a day while a branch is open, and again
  right before asking for review. Resolve conflicts in favour of traced
  behaviour; if two traced versions disagree, say so in the pull request.
- Before starting something sizeable, claim it in the area table below
  (a pull request editing this table is enough), so two people's agents
  don't build the same system twice.

## Claiming a task

Major systems open to contributors are listed in [TASKS.md](TASKS.md)
and as GitHub issues titled `[task] …`. (The maintainer's list in
TASKS.md isn't open for claiming.)

1. Pick an issue nobody has claimed (no assignee, no claim comment).
2. Comment `Claiming this` with the branch name you'll use. The maintainer
   assigns you. First claim wins; don't start on a claimed task.
3. One task per person at a time (two if they're small). Big tasks say
   how to split them; claim one part at a time.
4. Post a short progress comment at least once a day. A claim with no
   comment or pull request for 48 hours can be released by the
   maintainer.
5. Link the pull request to the issue (`Closes #N`). If you give up, say
   so on the issue so someone else can take it.

Found a new bug? Open an issue for it; don't fix it inside an unrelated
pull request.

## What every pull request must show

1. **Checks.** From the root: `cargo test --workspace`, `cargo clippy
   --workspace --all-targets`, `cargo fmt --all -- --check`. In `viewer/`:
   `cargo test`, `cargo clippy --all-targets`, `cargo fmt --all -- --check`,
   `cargo build --release`.
2. **Acceptance routes.** `powershell -File scripts\acceptance.ps1 -Data
   "<your Fallout New Vegas\Data>"` replays Doc Mitchell's walk, Back in
   the Saddle and Ghost Town Gunfight in the release viewer. All three must
   pass, or the pull request must say which one failed and show that it
   fails the same way on `main` (the gunfight has some randomness: if it
   fails once, run it again and report both). Paste the result table.
3. **Provenance.** Every behaviour comes from the original game: translated
   from FalloutNV.exe 1.4.0.525 with the address in a comment, named from
   the Xbox prototype symbols where they help, or recorded from the original
   game. No stand-ins, no "close enough" constants. Something you could not
   trace is listed in the pull request and must not change the base-game
   routes.
4. **Evidence of play.** Say what you checked by playing (with the keys,
   mouse or gamepad) and what you only checked by script (`--run`,
   `--say`, `--use`) or unit test. Keep logs and screenshots private.
5. **Docs.** The system's topic page in `docs/` says what is done, what is
   traced, what is missing. Add a short line to `docs/MILESTONES.md`.

Never commit game files, extracted assets, executable bytes, decompiler
databases or exports, recordings, or logs.

## Areas

Who is working on what. Ask the owner before changing their area in a
large way; small fixes anywhere are welcome with a test.

| Area | Owner | Notes |
| --- | --- | --- |
| Opening (VCG01), Doc Mitchell's house, character creation, look-IK | maintainer | Face editor and opening movie still open. |
| Goodsprings routes: Back in the Saddle (VCG02), Ghost Town Gunfight (VMS16) | maintainer | Acceptance routes; see GOODSPRINGS_ROUTE.md. |
| NPC AI: packages, pathing, combat, animation, dialogue behaviour | maintainer | PACKAGES, PATHING, NPC_COMBAT, ANIMATION, DIALOGUE. |
| Player: movement, actions, sneaking, aiming, furniture, physics | maintainer | MOVEMENT, PLAYER_ACTIONS, FURNITURE, PHYSICS. |
| Pip-Boy (except Repair and weapon mods), start menu, radio, local map | maintainer | PIPBOY.md. |
| Terminals and hacking | Chazm | |
| Item scripts, Repair, weapon mods | Chazm | The Pip-Boy's Repair and Mod pages are Chazm's. |
| Companions | Chazm | Dead Money's companion script functions overlap: coordinate. |
| Caravan | Chazm | Dead Money's caravan card functions overlap: coordinate. |
| Casinos (base game) | Chazm | Dead Money's casino menus overlap: coordinate. |
| Performance (M4) | Chazm | Measure before and after; no behaviour changes. |
| Dead Money (DLC01), crafting and the recipe menu | Dead Money contributor | HANDOFF_DEAD_MONEY.md, DEAD_MONEY_COVERAGE.md. |

Overlapping rows: whoever starts first writes the shared engine part in its
own pull request, and the other builds on it once it is merged.

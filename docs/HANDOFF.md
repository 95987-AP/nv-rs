# Handoff: overnight session, 2026-10-06 → 07

For the next lead session (Claude). Read [AGENTS.md](../AGENTS.md),
[MILESTONES.md](MILESTONES.md), [CONTRIBUTING.md](CONTRIBUTING.md) and
[TASKS.md](TASKS.md) first. Your job tonight: fix the playtest bugs and do the
optimizations on the maintainer's list in TASKS.md, using small bounded
agents; push and publish as each fix lands, and keep the integration
branch green.

## State at handoff

- Integration branch: `claude/overnight-integration` (pushed). `main` is
  untouched; merging integration into `main` is the user's decision.
- Published: play build 11 in `Desktop\nv-rs-play` (see "Publishing").
  Doc's walk, Back in the Saddle and Ghost Town Gunfight pass
  (`scripts/acceptance.ps1`, the gunfight 2 of 2).
- Merged today: NPC animations, NPC navigation, look-IK (reconciled with
  the Dead Money contributor's head tracking), physics batches 2 and 3,
  third-person weapon, start menu/radio/local map, NPC hit fix, the Dead
  Money contributor's branches (docs/CONTRIB_PLAYCON.md).
- Stopped mid-task by a computer restart (their agents are gone; the work
  is on disk in their worktrees under `%USERPROFILE%\nv-re\work\`):
  - `wt-death`, branch `claude/m2-death-ragdoll` (death animation into
    ragdoll; Cheyenne left standing after `Kill`): 2 commits plus
    uncommitted changes in 4 files.
  - `wt-radio-unify`, branch `claude/m2-radio-unify` (one radio state for
    the Pip-Boy and the script functions; the 2 key, Ammo Swap vs hot
    key 2): 1 commit plus uncommitted changes in 8 files.
  - `wt-contrib-chazm`, branch `claude/contrib-chazm`: merging Chazm's
    PR #11 (`pr/11`; terminals, hacking, item scripts, repair, weapon
    mods, companions, Caravan, casinos, crafting, Bink intro, "faster NPC
    pathing / mouse look"). The merge is in progress and uncommitted
    (about 160 files staged or conflicted). Crafting, Caravan and
    companion functions overlap the Dead Money merge: keep one traced
    implementation (Chazm owns these areas, so his where equally traced)
    without losing tests. Write docs/CONTRIB_CHAZM.md.
  Resume each as a new agent in the same worktree: tell it to inspect
  `git status`/`git log`/`git diff`, keep the work already there, and
  finish the original task (each needs live verification, full checks
  and `scripts/acceptance.ps1`). Then merge into integration, run
  acceptance, publish, push.

## First steps

1. GitHub CLI: `gh` was installed for this session (`C:\Program
   Files\GitHub CLI\gh.exe`; if it isn't there, `winget install
   GitHub.cli`). It needs the user to sign in once (`gh auth login`, in
   their own terminal); never handle their credentials. If `gh auth
   status` fails, ask the user to run it, and use GitHub Desktop's git
   for pushing meanwhile.
2. GitHub tasks: the user may have created the 12 major-system issues
   (TASKS.md M1–M12, titled `[task] M<n>. <name>`) from prefilled links.
   Check with `gh issue list`; create any missing ones with `gh issue
   create` (title and body from TASKS.md, plus the claiming rules from
   CONTRIBUTING.md). Never post the maintainer's list (B-items) as issues.
3. Resume the three stopped branches above (at most 3–4 agents at once).
4. Then the maintainer's list.
## Choosing work

1. Work the **maintainer's list** in TASKS.md (B1–B21: the build-11
   playtest bugs and polish). These are ours; they aren't GitHub tasks.
2. Don't start anything in an area claimed on GitHub (`[task] …` issues
   with an assignee or a `Claiming this` comment) or owned by a
   contributor in CONTRIBUTING.md. If a bug fix needs a change there, keep
   it minimal and say so on that issue.
3. When you find a **major system** that's missing (one that would take a
   whole agent-night and isn't in our areas), add it to TASKS.md's
   "Major systems" list and open a `[task]` issue for it, so contributors
   can claim it. Close or update issues as things land.
4. Suggested order: B12 Doc dialogue trap, B13 barter over dialogue, B3
   crosshair pick, B4 NPCs through the ground, B10 greetings, B11 voice
   after skipping, B5/B6 weapon effects and impacts, B14 opening, B1/B2
   physics (large; split), B7, B8, B9, B16, B17, B18–B21.
5. At most 3–4 agents at once (memory: builds with `-j 2`, the lead's
   with `-j 3`; more agents run out of memory and disk).
## Agent workflow (what worked today)

- One worktree and branch per task, created by the lead from the
  integration tip:
  `git worktree add -b claude/<name> %USERPROFILE%\nv-re\work\wt-<name> <integration tip>`.
  Agents must use absolute paths and must never edit the main checkout
  (`Desktop\nv-rs`) or another worktree; they have leaked there before,
  so check `git status` of the main checkout after each agent.
- Each agent gets its own Ghidra project copy
  (`%USERPROFILE%\nv-re\ghidra_N`, set `NVRE_GHIDRA`; scripts in
  `%USERPROFILE%\nv-re\scripts`) and its own `CARGO_TARGET_DIR` for root
  tests (`%USERPROFILE%\nv-re\work\target-<name>`).
- Prompts must require: tracing in Ghidra with provenance, **live
  verification in the release viewer** (screenshots read back; the
  `--key-at SECONDS KEY[:HOLD]` option injects input inside the viewer),
  reporting verified-live separately from unit-tested, the full checks,
  and `scripts/acceptance.ps1`.
- The `repo_hygiene` root test fails in worktrees only (the `.git`
  pointer file). Any other hygiene finding is real.

## Merging, checking, publishing

- Merge each finished branch into integration in its worktree
  (`%USERPROFILE%\nv-re\work\wt-integration`). docs/MILESTONES.md
  conflicts on most merges; `%USERPROFILE%\nv-re\work\union-milestones.ps1`
  (run from the worktree root) keeps both sides. `resolve.ps1 <file>
  o,t,b,...` in the same folder takes ours/theirs/both per conflict.
- After each merge: root `cargo test --workspace`, clippy, fmt; viewer
  tests, clippy, fmt, release build; then
  `powershell -File scripts\acceptance.ps1`. The gunfight is partly
  random: one failure → rerun; two → investigate before publishing
  (compare hit/miss and death lines with a passing log).
- Publish: `powershell -File %USERPROFILE%\nv-re\work\publish-play.ps1
  -Notes "...", "..."` (copies the integration release build into the
  play copy, writes WHATS-NEW.txt). Push integration afterwards.
- Git isn't on PATH: use GitHub Desktop's
  (`%LOCALAPPDATA%\GitHubDesktop\app-3.6.6\resources\app\git\cmd\git.exe`).
  `gh` (GitHub CLI) is installed once the user has signed it in; use it for
  issues and pull requests (open PRs against `main` for contributors'
  review when the user asks).

## Rules to keep

- No stand-ins: nothing ships that isn't traced or recorded; untraced
  behaviour goes behind `NV_GUESSES=1` (`world::guesses`) or stays out.
- Never commit game files, exe bytes, decompiler output, recordings or
  logs; keep them under `%USERPROFILE%\nv-re`.
- Don't touch AGENTS.md. Don't merge into `main`.
- Whole systems, not patches: when fixing a bug, finish the system it
  belongs to as the game has it.

## Morning report

Update docs/MILESTONES.md and this file's "State at handoff", update any
GitHub issues touched (what landed, what's left), and leave the
user a short summary: builds published, tasks done, what's verified
live, what isn't, and anything that needs their decision.

# Diagnostic recorder and saved-session investigation

User-requested opt-in tooling, 2026-10-09, branch `codex/diagnostics`.
Source requirement: the user's `Downloads/PLAN.md`. This records the runtime
that exists; it does not change game behavior or assert retail equivalence.

## Recording

Pass `--diagnostics` to the current viewer release executable. Output defaults
to `reports/diagnostics` under the working directory. `--diagnostics-dir PATH`
also enables recording. The local launcher's scene selection is followed by
an off/on choice; its current release executable and `userdata` remain in use.
Existing captures are retained in uniquely created session directories.

Each session contains a version-1 `manifest.json`, rotated `events-NNNN.jsonl`
files and `status.json`. Build revision/dirty status and launch arguments are
captured before game files open. Runtime configuration adds plugin order,
archive winning priority, adapter/backend, resolution and presentation modes
through events and background manifest updates. CPU frame work and monotonic
frame intervals use separate main/render IDs; these are not GPU timings.

The standard-library-only `diagnostics` crate has an optional shared observer,
thread-local parent context and RAII CPU spans. Disabled capture creates no
writer or files and bypasses event formatting. Producers use a nonblocking
queue lock; contention as well as full queues can drop evidence. Counters are
shown in the console and saved in status. Limits: 8,192 queued events, 16 MiB
encoded queued data, 16 MiB per trace file, 512 MiB trace data per session.
The background writer flushes at least once per second and on orderly exit.
Disk errors or the session limit stop capture while play continues. A forced
exit can lose buffered events; readers retain complete JSONL lines and label
missing status, torn tails and shortened bookmark windows.

Exterior cells record request, worker execution, result availability, insertion,
failure and unload stages. Parent IDs survive worker threads and insertion.
Asset reads record candidates, winning sources, normalized paths, byte counts
and duration. Candidate misses are distinct from final scene/texture failures.
Texture events record cache results, parsing, transformations, dimensions,
formats, mip counts and estimated logical bytes. Optional tint/layer failures
remain separate. Known requesting mesh/reference information is recorded;
otherwise ownership is explicitly unknown. Bevy image creation means **queued
for rendering**: GPU upload completion, residency and first visible use are
unavailable. Logical texture bytes do not measure VRAM.

F12 still writes picture, saved state, location and reproduction commands.
Its report adds a session reference and bookmark for 30 seconds before and
five after. Capture events identify report writing and screenshot activity.
Slow frame intervals above 50 ms and final failures produce automatic
evidence markers without screenshots or saves.

## Investigation

See [the standalone server/CLI instructions](../research/diagnostics/README.md).
The official MCP Python SDK is pinned in that directory's independent lockfile.
The server reads saved files through stdio, beneath configured capture/report
roots. It cannot launch or control the viewer, modify assets or apply fixes.
Capture strings, plugin names and report notes are evidence, not instructions.
The CLI works without importing the SDK. Results include evidence IDs,
timestamps, completeness and loss counts, with bounded pagination.

## Delivery and checks

Three stacked draft reviews are open in the contributor's fork:

1. [Recorder and frame timing](https://github.com/95987-AP/nv-rs/pull/1),
   `codex/diagnostics-recorder` over `codex/subtle-notifications`.
2. [Loading, textures and F12](https://github.com/95987-AP/nv-rs/pull/2),
   `codex/diagnostics-loading` over the recorder branch.
3. [MCP/CLI, launchers and live corrections](https://github.com/95987-AP/nv-rs/pull/3),
   `codex/diagnostics` over the loading branch.

Core workspace tests, clippy, formatting and release build pass. Viewer checks
run separately: 172 tests, clippy with warnings denied, formatting and release
build pass. The final Rust release is clean source `9ae3334`. The reader has
14 passing synthetic/SDK tests, covering numeric IDs, path confinement,
pagination, interrupted/malformed files, multiple bookmarks, absolute time
bounds, unknown loss counts, staged loading durations and mismatched comparison.
The independent SDK lockfile validates; Python compilation passes.

Recorder tests cover disabled operation, JSON escaping, cross-thread parents,
queue saturation, rotation, session limits, orderly shutdown and disk failure.
Generated asset/cellview fixtures cover archive/loose-file precedence, fallback
candidates versus final failures, corrupt DDS, cache reuse, ownership and
optional-input classification. Game files are not fixtures and are not committed.

The initial full check ran out of space on D: writing compiler outputs. Only
package-scoped generated outputs were cleaned; no sources or captures were
removed. Core checks use a C: target directory with incremental caching disabled.
Private check logs remain in Downloads as
`nvrust-diagnostics-{core,viewer}-{tests,clippy,build}.log` and final-check variants.
The initial acceptance run passed Doc's success condition, then was stopped to
rebuild after audit repairs. All three routes will rerun on the final release
through `scripts/acceptance.ps1`; pass conditions and route input are unchanged.

## Live evidence

Private evidence root: `%USERPROFILE%/nv-re/work/diagnostics-validation`.
Doc's house is a startup/UI smoke test. At the user's request, final performance
measurements use Goodsprings, including walking across exterior cell boundaries.
The actual adapter is **GTX 1080, Vulkan**, rather than the laptop in historical
machine notes. Captured settings include plugin/archive order, 1280x720,
Mailbox presentation and independent main/render CPU IDs; GPU timing is unavailable.

The final exterior walk (`final-live/walk.log`, `mcp-final.json`, captures and
`reports/001`) crosses from cell `000DAEBB` to `000DAEB9`, x -72151 to -69010.
The SDK initializes over stdio and queries session summary, slow frames, repeated
startup/gameplay phase samples, a seven-stage cell trace through unload, linked
asset evidence and the F12 report. The report window correctly flags the shortened
startup side and retains its five-second after window. Its picture excludes FPS
and retains the requested right-side notification cards. Capture loss is explicit;
a complete shutdown status does not mean every event was retained.

Live validation found and corrected three concrete issues: the reader interpreted
absolute F12 bounds as durations; a dropped one-time phase transition classified
the rest of the walk as startup; a dropped initial configuration snapshot omitted
plugin/archive metadata. Bounds now have an exact regression fixture, phases are
repeated once per second and configuration retries when emission loses evidence.

## Goodsprings overhead measurement

Three interleaved off/on pairs use the identical final release executable, with
no other viewer/build running. All measured frames are ready and focused.
The route starts at `-72151,639,8281,90`, walks east for 25 seconds from second 5,
strafes for four seconds from second 31 and walks back for six from second 36.
It includes cell streaming. Runs use `--official --walk --fps --screen-size 1280,720`,
a screenshot and `--wait 45`. The measurement interval is seconds 10..40 after
App startup, before the end screenshot. Both sides enable the same explicit
`NV_DIAGNOSTICS_BENCHMARK_PATH` in-memory CSV harness; only the on side starts
the recorder. CSV output is written on exit. Captures include screenshot overhead
outside the measured interval.

| Pair | Median off/on (ms) | Median change | p95 off/on (ms) | p95 change (ms) | On dropped events |
| --- | --- | --- | --- | --- | --- |
| 1 | 6.1764 / 6.2661 | +1.45% | 7.6173 / 7.6721 | +0.0548 | 515 |
| 2 | 6.1876 / 6.2381 | +0.82% | 7.7829 / 7.6688 | -0.1141 | 560 |
| 3 | 6.1914 / 6.2259 | +0.56% | 7.5637 / 7.6480 | +0.0843 | 543 |

Every pair meets the requested <=3% median and <=1 ms additional p95 targets on
this measured route. Median paired change is +0.82%; median p95
change is +0.0548 ms. This does not establish overhead for all
worldspaces or GPU workloads. Dropped events prevent treating these JSONL captures
as lossless comparisons: the frame benchmark instead uses the identical buffered
harness in both off/on runs. Other machine load and run-to-run variation remain
measurement limits. Earlier house measurements are supporting smoke evidence;
one final house run lost focus and is excluded from the final acceptance claim.

Evidence: `goodsprings-benchmark/benchmark-results.json`, six CSVs/logs/screenshots
and three captures; private reproduction script `run-goodsprings-pairs.py`.
Executable SHA-256: `ce842ce8e8be7a63e8ea028d95a1c0f8abc98b4d21bc86c8ffb01dbda2555cd3`.

## Customizations and handoff

The original staged FPS work was preserved in `7a01072`, with a recoverable
external patch at `%USERPROFILE%/Downloads/nvrust-before-diagnostics.patch`.
Live checks confirm default FPS visibility, F3 hide/restore, `--no-hud` hides FPS,
F12 screenshot exclusion and continued `--fps` console reporting. The installed
controls also bind F3 to Pip-Boy DATA; that pre-existing overlap is unchanged.
The 0.25-second refresh and notification expiry/overflow/choice rules are preserved.
The final F12 image also confirms the notification cards remain present.

The ignored local launcher and checked-in CMD distribution launcher save the
choice result immediately: resetting the flag first had cleared CMD's errorlevel
and incorrectly launched without recording. Live on/off runs now exit zero and
show the expected command lines. On captures are under existing `userdata`; off
creates no new trace. Both route through the current viewer release via
`NV_RS_VIEWER`. PowerShell `-ValidateOnly` verified its diagnostics flags, paths
with spaces, executable override and save folder. Backup of the original ignored
launcher: `%USERPROFILE%/Downloads/Play-before-diagnostics.cmd`. No official play
package was published and no game installation files were changed.

Final audits repaired omitted evidence references incorrectly invalidating complete
comparisons and added missing/corrupt texture markers reported by preview loading.
Both have regression fixtures (14 reader tests pass). The texture fixture checks
final classification and the expected cell parent on each marker.

Unfinished: final Rust checks/build, three exterior measurement pairs after the
marker addition, all acceptance routes and pushing updated review evidence.
Next action: validate the audit repairs and build the final release.

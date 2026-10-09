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

## Delivery batches and handoff

1. Recorder and frame timing: implemented. Recorder fixture tests cover JSON
   escaping, disabled capture, cross-thread correlation, saturation, rotation,
   session limits, shutdown and disk failure. Initial six tests passed. Full
   core/viewer validation and live overhead checks are in progress.
2. Loading, textures and F12: implemented. Generated fixtures cover source
   priority, fallback misses versus final failures, corrupt DDS, cache reuse
   and optional-input classification. Targeted assets/cellview tests passed
   before final assertion additions; those additions await the full rerun.
3. MCP/CLI and launcher: implemented. Seven synthetic reader tests initially
   passed, including real SDK stdio initialization/query. Review found numeric
   ID and bounded-reader cases that are being repaired before live verification.

Existing FPS code was preserved and committed with the customization register
in `7a01072`; the prior uncommitted diff has a recoverable external patch at
`%USERPROFILE%/Downloads/nvrust-before-diagnostics.patch`. Ignored launcher
backup: `%USERPROFILE%/Downloads/Play-before-diagnostics.cmd`.

Initial full validation failed because D: ran out of space writing compiler
caches. No source or captures were removed. Package-scoped `cargo clean` freed
generated viewer/nvinspect outputs; the viewer release is being rebuilt before
any local launcher run. Core checks now use a target directory on C:; checks
disable incremental caching. Logs are private in Downloads as
`nvrust-diagnostics-{core,viewer}-{tests,clippy,build}.log`.

Active processes: core full checks and viewer full checks. Unfinished: final
tests/builds, live Doc's house and exterior walk, F12 through MCP, three
interleaved foreground off/on benchmark pairs, customization verification and
review PRs. Next action: finish validation, then run the same release build
through those live checks. Do not infer overhead compliance from test counts.

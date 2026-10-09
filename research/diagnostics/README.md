# nv-rs diagnostic reader

This directory contains a read-only stdio MCP server and a standard-library CLI
for captures produced by the nv-rs diagnostics recorder. It does not import the
game, open saves, or write under the configured capture root.

## Run

Install/sync the pinned MCP environment with `uv sync --locked`. Start the MCP
server with `uv run server.py --capture-root PATH` (or set
`NV_DIAGNOSTICS_DIR`). Optional `--reports-root PATH` (or `NV_REPORTS_DIR`)
defaults to the capture root's parent. `get_report` follows a saved
`marker`/`report_bookmark` `fields.report_path` within that root; pass its
`bookmark_id` to select a particular F12 report (the latest is the default).
It returns the recorded window and nearby screenshot/report activity, then
reads bounded `report.txt`, `note.txt`, and `state.txt` text plus `picture.png`
metadata. The server exposes `list_sessions`,
`get_session_summary`, `get_events`, `get_slow_frames`, `inspect_cell_load`,
`inspect_asset`, `get_report`, and `compare_sessions`.

The CLI does not import `mcp` and can run in a standard Python environment:

```powershell
py research/diagnostics/cli.py --capture-root PATH list
py research/diagnostics/cli.py --capture-root PATH summary SESSION_ID
py research/diagnostics/cli.py --capture-root PATH compare SESSION_A SESSION_B
```

## Capture layout

The reader expects `ROOT/SESSION_ID/manifest.json`, one or more
`events-NNNN.jsonl` files, and optional `status.json`. F12 reports live under
the reports root in a bookmark directory. JSONL records follow
the version-1 shape from the diagnostics plan. A final event line without a
newline is treated as an interrupted partial write and ignored. Malformed
complete lines are skipped and reported.

Every event query scans at most 512 MiB plus one maximum-sized event, returns
at most 1,000 events and 2 MiB of event data, and reports truncation/read
issues. Individual JSONL lines over 256 KiB are skipped and reported. The report reader only follows explicit
bookmarked report folders within the reports root, reads named text artifacts,
and returns picture metadata, never image bytes. Configured roots are resolved
once; session IDs cannot contain path separators or escape the capture root.
No tool accepts an arbitrary file path.

`compare_sessions` divides `main_frame_interval` samples at `phase` events whose
`fields.name` is `startup` or `gameplay`. Until a gameplay phase event is
written, frame samples are classified as startup. Configuration differences
are surfaced, and the result says whether configurations match. Runtime
`configuration` event fields are merged over manifest settings before the
comparison, so late-resolved plugin and archive order is included.
Cell loading compares request-to-worker queue time, worker CPU work, worker-end
to-result queue time, insertion time, and total request-to-insert time. Each
phase includes counts for requested, completed, failed, and still-incomplete
loads, plus bounded provenance. `benchmark_comparable` is false when configs
differ or either capture is incomplete, dropped, malformed, or truncated; the
result lists the reason for each side.
Cell and asset inspection normalizes numeric and string IDs and follows direct
`id` correlations, `parent_id` links, and ownership events. Texture and fallback
events can be matched through requested, candidate, fallback, resolved, asset,
texture, and base path fields. Candidate `asset_read` misses are evidence, not final failure
counts; final texture/asset failures, optional texture failures, and cell
failures are reported separately.
F12 `before_us` and `after_us` fields are absolute session-time bounds; the
reader clamps them to the recorded interval and reports requested and actual
bounds. Missing `dropped_events` status is marked as unknown, not known zero.

## Verify

Run `uv run --locked python -m unittest discover -s tests -v` for synthetic
reader and MCP stdio tests. The fixtures contain no game data.

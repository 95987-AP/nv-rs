"""Bounded, read-only access to nv-rs diagnostic capture sessions."""

from __future__ import annotations

import json
import math
import re
from collections import deque
from pathlib import Path
from typing import Any, Iterator

MAX_PAGE = 1000
DEFAULT_PAGE = 100
MAX_SCAN_BYTES = 512 * 1024 * 1024 + 256 * 1024
MAX_EVENT_BYTES = 256 * 1024
MAX_RESPONSE_BYTES = 2 * 1024 * 1024
MAX_REPORT_BYTES = 1024 * 1024
MAX_METADATA_BYTES = 64 * 1024
MAX_CELL_LOADS = 50_000
MAX_CELL_ALIASES = 500_000
MAX_CELL_PROVENANCE = 1_000
MAX_FRAME_SAMPLES_PER_PHASE = 200_000
EVENT_FILE = re.compile(r"events-(\d{4,})\.jsonl\Z")
SESSION_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")


class CaptureError(ValueError):
    """Invalid request or inaccessible capture evidence."""


def _json_file(path: Path, *, limit: int = MAX_REPORT_BYTES) -> dict[str, Any] | None:
    try:
        if path.stat().st_size > limit:
            return None
        value = json.loads(path.read_text(encoding="utf-8"))
        return value if isinstance(value, dict) else None
    except (OSError, UnicodeError, ValueError, RecursionError):
        return None


def _percentile(values: list[float], percentile: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    return ordered[max(0, math.ceil(percentile * len(ordered)) - 1)]


def _event_time(event: dict[str, Any]) -> int | None:
    value = event.get("time_us")
    return value if isinstance(value, int) and value >= 0 else None


def _id_text(value: Any) -> str | None:
    if isinstance(value, bool) or value is None:
        return None
    if isinstance(value, (str, int)):
        value = str(value)
        return value if len(value) <= 256 else None
    return None


def _bounded_metadata(value: Any, max_bytes: int = MAX_METADATA_BYTES) -> Any:
    budget = [max_bytes]

    def clip(item: Any, depth: int = 0) -> Any:
        if budget[0] <= 0:
            return "[metadata truncated]"
        budget[0] -= 16
        if isinstance(item, str):
            max_chars = min(4096, max(0, budget[0]))
            value = item[:max_chars]
            budget[0] -= len(value.encode("utf-8", errors="replace"))
            return value + ("…" if len(value) < len(item) else "")
        if isinstance(item, (int, float, bool)) or item is None:
            return item
        if depth >= 8:
            return "[nested metadata truncated]"
        if isinstance(item, list):
            out = []
            for child in item[:200]:
                if budget[0] <= 0:
                    break
                out.append(clip(child, depth + 1))
            return out
        if isinstance(item, dict):
            out = {}
            for key, child in list(item.items())[:200]:
                if budget[0] <= 0:
                    break
                k = str(key)[:256]
                budget[0] -= len(k)
                out[k] = clip(child, depth + 1)
            return out
        return str(item)[:256]

    return clip(value)


class _CellLoadingCollector:
    """Join cell lifecycle events by IDs and summarize bounded stage timings."""

    EVENT_STAGE = {
        "cell_request": "request",
        "cell_worker_start": "worker_start",
        "cell_worker_end": "worker_end",
        "cell_result": "result",
        "cell_insert_start": "insert_start",
        "cell_insert_end": "insert_end",
        "cell_failure": "failure",
        "cell_unload": "unload",
    }
    DURATION_PAIRS = {
        "request_to_worker_start_queue": ("request", "worker_start"),
        "worker_start_to_end_work": ("worker_start", "worker_end"),
        "worker_end_to_result_queue": ("worker_end", "result"),
        "insert_start_to_end": ("insert_start", "insert_end"),
        "request_to_insert_end_total": ("request", "insert_end"),
    }

    def __init__(self) -> None:
        self.loads: dict[str, dict[str, Any]] = {}
        self.aliases: dict[str, str] = {}
        self.truncation_reasons: set[str] = set()

    def add(self, event: dict[str, Any], phase: str) -> None:
        kind = event.get("type")
        stage = self.EVENT_STAGE.get(kind)
        if stage is None:
            return
        fields = event.get("fields") if isinstance(event.get("fields"), dict) else {}
        raw_tokens = [event.get("id"), event.get("parent_id"),
                      fields.get("load_id"), fields.get("correlation_id")]
        tokens = list(dict.fromkeys(token for value in raw_tokens if (token := _id_text(value)) is not None))
        known = [self.aliases[token] for token in tokens if token in self.aliases]
        if kind == "cell_request":
            key = known[0] if known else (tokens[0] if tokens else f"sequence:{event.get('sequence', '')}")
        else:
            preferred = tokens[1] if len(tokens) > 1 else (tokens[0] if tokens else None)
            key = known[0] if known else (preferred or f"sequence:{event.get('sequence', '')}")

        # If this event bridges aliases already assigned to separate provisional loads,
        # merge their bounded state into one canonical load.
        for other in known[1:]:
            if other == key or other not in self.loads:
                continue
            source = self.loads.pop(other)
            target = self.loads.get(key)
            if target is None:
                self.loads[key] = source
            else:
                target["stages"].update({k: v for k, v in source["stages"].items() if k not in target["stages"]})
                target["refs"].update({k: v for k, v in source["refs"].items() if k not in target["refs"]})
                if target.get("phase") is None:
                    target["phase"] = source.get("phase")
            for token, alias in list(self.aliases.items()):
                if alias == other:
                    self.aliases[token] = key

        if key not in self.loads:
            if len(self.loads) >= MAX_CELL_LOADS:
                self.truncation_reasons.add("cell_state_limit")
                return
            self.loads[key] = {"phase": phase, "stages": {}, "refs": {}}
        load = self.loads[key]
        if kind == "cell_request":
            load["phase"] = phase
        event_time = _event_time(event)
        load["stages"].setdefault(stage, event_time)
        if stage not in load["refs"]:
            load["refs"][stage] = {
                "id": str(event.get("id"))[:128] if event.get("id") is not None else None,
                "parent_id": str(event.get("parent_id"))[:128] if event.get("parent_id") is not None else None,
                "sequence": event.get("sequence"), "time_us": event_time, "type": kind,
            }
        for token in tokens:
            if token in self.aliases or len(self.aliases) < MAX_CELL_ALIASES:
                self.aliases[token] = key
            else:
                self.truncation_reasons.add("cell_alias_limit")

    def summarize(self) -> dict[str, Any]:
        stats: dict[str, dict[str, Any]] = {
            name: {"request_count": 0, "completed_count": 0, "failed_count": 0,
                   "incomplete_count": 0, "duration_values": {metric: [] for metric in self.DURATION_PAIRS},
                   "provenance": []}
            for name in ("startup", "gameplay")
        }
        provenance_count = 0
        for load_id, load in self.loads.items():
            stages = load["stages"]
            if "request" not in stages:
                continue
            phase = load.get("phase") if load.get("phase") in stats else "startup"
            row = stats[phase]
            row["request_count"] += 1
            if "failure" in stages:
                row["failed_count"] += 1
            elif "insert_end" in stages:
                row["completed_count"] += 1
            else:
                row["incomplete_count"] += 1
            for metric, (start_name, end_name) in self.DURATION_PAIRS.items():
                start, end = stages.get(start_name), stages.get(end_name)
                if isinstance(start, int) and isinstance(end, int) and end >= start:
                    row["duration_values"][metric].append(end - start)
            if provenance_count < MAX_CELL_PROVENANCE:
                row["provenance"].append({"load_id": load_id[:128], "phase": phase,
                                          "events": list(load["refs"].values())[:8]})
                provenance_count += 1
        result = {}
        for phase, row in stats.items():
            durations = {
                name: {"count": len(values), "median_us": _percentile(values, .5),
                       "p95_us": _percentile(values, .95), "max_us": max(values) if values else None}
                for name, values in row.pop("duration_values").items()
            }
            result[phase] = {**row, "durations_us": durations}
        return {"phases": result, "tracked_loads": len(self.loads),
                "state_limit": MAX_CELL_LOADS, "provenance_limit": MAX_CELL_PROVENANCE,
                "alias_limit": MAX_CELL_ALIASES, "truncated": bool(self.truncation_reasons),
                "truncation_reasons": sorted(self.truncation_reasons)}


class CaptureStore:
    """Read captures below one configured root; never follows caller paths."""

    def __init__(self, root: str | Path, reports_root: str | Path | None = None):
        self.root = Path(root).expanduser().resolve(strict=False)
        self.reports_root = Path(reports_root).expanduser().resolve(strict=False) if reports_root else self.root.parent

    @staticmethod
    def _contained(path: Path, base: Path) -> bool:
        try:
            return path.resolve(strict=False).is_relative_to(base.resolve(strict=False))
        except (OSError, RuntimeError):
            return False

    def _session_dir(self, session_id: str) -> Path:
        if not isinstance(session_id, str) or not SESSION_ID.fullmatch(session_id):
            raise CaptureError("invalid session_id")
        path = (self.root / session_id).resolve(strict=False)
        if path.parent != self.root:
            raise CaptureError("session path escapes configured capture root")
        if not path.is_dir():
            raise CaptureError("session not found")
        return path

    def _session(self, session_id: str) -> tuple[Path, dict[str, Any], dict[str, Any] | None]:
        path = self._session_dir(session_id)
        manifest_path = path / "manifest.json"
        manifest = _json_file(manifest_path) if self._contained(manifest_path, path) else None
        if not manifest or manifest.get("version") != 1 or manifest.get("session_id") != session_id:
            raise CaptureError("missing, invalid, or mismatched manifest")
        status_path = path / "status.json"
        status = _json_file(status_path) if self._contained(status_path, path) else None
        return path, manifest, status

    def _event_paths(self, path: Path) -> list[Path]:
        found = [(int(m.group(1)), p) for p in path.iterdir()
                 if (m := EVENT_FILE.fullmatch(p.name)) and p.is_file() and self._contained(p, path)]
        return [p for _, p in sorted(found)]

    def _iter_events(self, path: Path) -> Iterator[tuple[dict[str, Any] | None, int, str | None]]:
        scanned = 0
        for file in self._event_paths(path):
            try:
                with file.open("rb") as stream:
                    while True:
                        raw = stream.readline(MAX_EVENT_BYTES + 1)
                        if not raw:
                            break
                        scanned += len(raw)
                        if scanned > MAX_SCAN_BYTES:
                            yield None, scanned, "scan_limit"
                            return
                        if len(raw) > MAX_EVENT_BYTES:
                            while raw and not raw.endswith(b"\n"):
                                raw = stream.readline(MAX_EVENT_BYTES + 1)
                                scanned += len(raw)
                                if scanned > MAX_SCAN_BYTES:
                                    yield None, scanned, "scan_limit"
                                    return
                            yield None, scanned, "oversized_event"
                            continue
                        if not raw.endswith(b"\n"):
                            # A torn final write in this file is expected after interruption.
                            yield None, scanned, "trailing_partial_line"
                            break
                        try:
                            item = json.loads(raw)
                            if (isinstance(item, dict) and isinstance(item.get("type"), str)
                                    and isinstance(item.get("fields"), dict)):
                                yield item, scanned, None
                            else:
                                yield None, scanned, "invalid_event_shape"
                        except (UnicodeError, ValueError, RecursionError):
                            yield None, scanned, "invalid_event_json"
            except OSError:
                yield None, scanned, "unreadable_event_file"
                continue

    @staticmethod
    def _base(session_id: str, status: dict[str, Any] | None) -> dict[str, Any]:
        dropped_value = status.get("dropped_events") if status else None
        loss_count_known = isinstance(dropped_value, int) and not isinstance(dropped_value, bool) and dropped_value >= 0
        dropped = dropped_value if loss_count_known else 0
        if not loss_count_known:
            dropped = 0
        return {
            "session_id": session_id,
            "complete": bool(status and status.get("complete") is True),
            "dropped_events": dropped,
            "loss_count_known": loss_count_known,
        }

    def list_sessions(self, limit: int = DEFAULT_PAGE, offset: int = 0) -> dict[str, Any]:
        limit = _limit(limit)
        offset = _offset(offset)
        if not self.root.is_dir():
            return {"sessions": [], "total": 0, "offset": offset, "limit": limit}
        entries = []
        for path in self.root.iterdir():
            if not path.is_dir() or not SESSION_ID.fullmatch(path.name):
                continue
            if not self._contained(path, self.root):
                continue
            manifest_path = path / "manifest.json"
            manifest = _json_file(manifest_path) if self._contained(manifest_path, path) else None
            if not manifest or manifest.get("version") != 1 or manifest.get("session_id") != path.name:
                continue
            status_path = path / "status.json"
            status = _json_file(status_path) if self._contained(status_path, path) else None
            entries.append({**self._base(path.name, status),
                            "started_at": str(manifest.get("started_at", ""))[:128],
                            "configuration": _bounded_metadata(manifest.get("configuration", {}), 1024)})
        entries.sort(key=lambda x: (str(x.get("started_at") or ""), x["session_id"]), reverse=True)
        return {"sessions": entries[offset:offset + limit], "total": len(entries), "offset": offset, "limit": limit}

    def get_session_summary(self, session_id: str) -> dict[str, Any]:
        path, manifest, status = self._session(session_id)
        counts: dict[str, int] = {}
        first = last = None
        corrupt = 0
        for event, _, error in self._iter_events(path):
            if error:
                corrupt += 1
            if event is None:
                continue
            kind = event.get("type")
            if isinstance(kind, str):
                key = kind[:128]
                if key in counts or len(counts) < 100:
                    counts[key] = counts.get(key, 0) + 1
                else:
                    counts["_other_types"] = counts.get("_other_types", 0) + 1
            t = _event_time(event)
            if t is not None:
                first = t if first is None else min(first, t)
                last = t if last is None else max(last, t)
        return {**self._base(session_id, status), "manifest": _bounded_metadata(manifest),
                "status": _bounded_metadata(status or {"complete": False, "reason": "status_missing"}),
                "event_counts": counts, "first_time_us": first, "last_time_us": last,
                "read_issues": corrupt, "scan_limited": corrupt > 0}

    def get_events(self, session_id: str, *, start_us: int | None = None, end_us: int | None = None,
                   event_type: str | None = None, offset: int = 0,
                   limit: int = DEFAULT_PAGE) -> dict[str, Any]:
        path, _, status = self._session(session_id)
        limit, offset = _limit(limit), _offset(offset)
        if start_us is not None and start_us < 0 or end_us is not None and end_us < 0:
            raise CaptureError("time bounds must be non-negative")
        if start_us is not None and end_us is not None and start_us > end_us:
            raise CaptureError("start_us must not exceed end_us")
        events, match_count, issues, scanned, stopped, response_bytes = [], 0, [], 0, False, 0
        for event, scanned, issue in self._iter_events(path):
            if issue:
                issues.append(issue)
                if issue == "scan_limit":
                    stopped = True
                    break
            if event is None:
                continue
            t = _event_time(event)
            if start_us is not None and (t is None or t < start_us):
                continue
            if end_us is not None and (t is None or t > end_us):
                continue
            if event_type and event.get("type") != event_type:
                continue
            if match_count >= offset and len(events) < limit:
                event_size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                if response_bytes + event_size > MAX_RESPONSE_BYTES:
                    stopped = True
                    issues.append("response_limit")
                    break
                events.append(event)
                response_bytes += event_size
            match_count += 1
        return {**self._base(session_id, status), "events": events,
                "offset": offset, "limit": limit, "matching_events_seen": match_count,
                "has_more": offset + len(events) < match_count or stopped,
                "bytes_scanned": min(scanned, MAX_SCAN_BYTES), "read_issues": issues[:20],
                "truncated": stopped or any(x in issues for x in {"oversized_event", "trailing_partial_line"})}

    def get_slow_frames(self, session_id: str, *, threshold_ms: float = 50.0,
                        before_us: int = 100_000, after_us: int = 100_000,
                        limit: int = DEFAULT_PAGE) -> dict[str, Any]:
        path, _, status = self._session(session_id)
        limit = _limit(limit)
        if not math.isfinite(threshold_ms) or threshold_ms < 0 or before_us < 0 or after_us < 0:
            raise CaptureError("threshold and surrounding windows must be non-negative")
        frames, surrounding, issues, recent, after = [], [], [], deque(), []
        surrounding_keys, response_bytes = set(), 0
        recent_bytes = 0
        for event, _, issue in self._iter_events(path):
            if issue:
                issues.append(issue)
            if event is None:
                continue
            t = _event_time(event)
            if t is None:
                continue
            while recent and t - recent[0][0] > before_us:
                _, _, old_size = recent.popleft()
                recent_bytes -= old_size
            near_frame = any(t <= end_time for end_time in after)
            if near_frame and len(surrounding) < limit * 20:
                key = (event.get("sequence"), event.get("type"), event.get("id"))
                size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                if key not in surrounding_keys and response_bytes + size <= MAX_RESPONSE_BYTES:
                    surrounding.append(event)
                    surrounding_keys.add(key)
                    response_bytes += size
                elif response_bytes + size > MAX_RESPONSE_BYTES:
                    issues.append("response_limit")
            after = [end_time for end_time in after if t < end_time]
            fields = event.get("fields") if isinstance(event.get("fields"), dict) else {}
            if event.get("type") in {"main_frame_interval", "main_frame"} and isinstance(fields.get("interval_ms"), (int, float)) and fields["interval_ms"] >= threshold_ms:
                if len(frames) < limit:
                    size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                    if response_bytes + size <= MAX_RESPONSE_BYTES:
                        frames.append(event)
                        response_bytes += size
                    else:
                        issues.append("response_limit")
                    for _, item, _ in recent:
                        if len(surrounding) >= limit * 20:
                            break
                        key = (item.get("sequence"), item.get("type"), item.get("id"))
                        item_size = len(json.dumps(item, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                        if key not in surrounding_keys and response_bytes + item_size <= MAX_RESPONSE_BYTES:
                            surrounding.append(item)
                            surrounding_keys.add(key)
                            response_bytes += item_size
                        elif response_bytes + item_size > MAX_RESPONSE_BYTES:
                            issues.append("response_limit")
                    after.append(t + after_us)
            recent_size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
            recent.append((t, event, recent_size))
            recent_bytes += recent_size
            while recent and (len(recent) > 2000 or recent_bytes > MAX_RESPONSE_BYTES):
                _, _, old_size = recent.popleft()
                recent_bytes -= old_size
        return {**self._base(session_id, status), "threshold_ms": threshold_ms,
                "frames": frames, "surrounding_events": surrounding,
                "read_issues": issues[:20], "truncated": any(x in issues for x in {"scan_limit", "oversized_event", "trailing_partial_line", "response_limit"})}

    def inspect_cell_load(self, session_id: str, load_id: str) -> dict[str, Any]:
        if not isinstance(load_id, str) or not load_id or len(load_id) > 256:
            raise CaptureError("invalid load_id")
        path, _, status = self._session(session_id)
        kinds = {"cell_request", "cell_worker_start", "cell_worker_end", "cell_result",
                 "cell_insert_start", "cell_insert_end", "cell_unload", "cell_failure"}
        matched, issues, scanned, response_bytes = [], [], 0, 0
        for event, scanned, issue in self._iter_events(path):
            if issue:
                issues.append(issue)
            if event and (load_id in {_id_text(event.get("id")), _id_text(event.get("parent_id"))}) and event.get("type") in kinds:
                event_size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                if len(matched) < MAX_PAGE and response_bytes + event_size <= MAX_RESPONSE_BYTES:
                    matched.append(event)
                    response_bytes += event_size
                else:
                    issues.append("correlation_result_or_byte_limit")
                    break
        return {**self._base(session_id, status), "load_id": load_id, "events": matched,
                "bytes_scanned": min(scanned, MAX_SCAN_BYTES), "read_issues": issues[:20],
                "correlation_truncated": "scan_limit" in issues or "correlation_result_or_byte_limit" in issues}

    def inspect_asset(self, session_id: str, *, asset_id: str | None = None,
                      requested: str | None = None, resolved: str | None = None) -> dict[str, Any]:
        if not any((asset_id, requested, resolved)):
            raise CaptureError("provide asset_id, requested, or resolved")
        if any(value is not None and (not isinstance(value, str) or len(value) > 4096)
               for value in (asset_id, requested, resolved)):
            raise CaptureError("asset filters must be strings no longer than 4096 characters")
        path, _, status = self._session(session_id)
        requested_keys = ("requested_path", "requested", "original_path", "texture_path", "asset_path", "path", "base_path")
        resolved_keys = ("resolved_path", "resolved", "candidate_path", "candidate", "fallback_path",
                         "fallback", "winning_path", "source_path", "texture_path", "asset_path", "path", "base_path")

        def matches(event: dict[str, Any]) -> bool:
            fields = event.get("fields") if isinstance(event.get("fields"), dict) else {}
            ids = {_id_text(event.get("id")), _id_text(event.get("parent_id")),
                   _id_text(fields.get("asset_id")), _id_text(fields.get("request_id")),
                   _id_text(fields.get("owner_id")), _id_text(fields.get("texture_id"))}
            return ((asset_id is None or asset_id in ids) and
                    (requested is None or requested in {str(fields[key]) for key in requested_keys if isinstance(fields.get(key), str)}) and
                    (resolved is None or resolved in {str(fields[key]) for key in resolved_keys if isinstance(fields.get(key), str)}))
        matched, issues, scanned, response_bytes = [], [], 0, 0
        for event, scanned, issue in self._iter_events(path):
            if issue:
                issues.append(issue)
            kind = event.get("type") if event else ""
            is_asset_evidence = kind == "asset_read" or kind.startswith("asset_") or kind.startswith("texture_") or "fallback" in kind
            if event and is_asset_evidence and matches(event):
                event_size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                if len(matched) < MAX_PAGE and response_bytes + event_size <= MAX_RESPONSE_BYTES:
                    matched.append(event)
                    response_bytes += event_size
                else:
                    issues.append("correlation_result_or_byte_limit")
                    break
        return {**self._base(session_id, status), "events": matched,
                "bytes_scanned": min(scanned, MAX_SCAN_BYTES), "read_issues": issues[:20],
                "correlation_truncated": "scan_limit" in issues or "correlation_result_or_byte_limit" in issues}

    def get_report(self, session_id: str, bookmark_id: str | None = None) -> dict[str, Any]:
        if bookmark_id is not None and (not isinstance(bookmark_id, str) or len(bookmark_id) > 256):
            raise CaptureError("bookmark_id must be a string no longer than 256 characters")
        path, _, status = self._session(session_id)
        bookmark = None
        selected_event = None
        bookmark_events, issues, scanned, capture_start_us, capture_end_us = [], [], 0, None, None
        for event, scanned, issue in self._iter_events(path):
            if issue:
                if len(issues) < 20:
                    issues.append(issue)
            event_time = _event_time(event) if event else None
            if event_time is not None:
                capture_start_us = event_time if capture_start_us is None else min(capture_start_us, event_time)
                capture_end_us = event_time if capture_end_us is None else max(capture_end_us, event_time)
            if event and event.get("type") in {"marker", "report_bookmark"}:
                fields = event.get("fields") if isinstance(event.get("fields"), dict) else {}
                report_path = fields.get("report_path")
                if isinstance(report_path, str) and report_path:
                    report_path = report_path[:4096]
                    candidate = Path(report_path)
                    candidate = candidate if candidate.is_absolute() else self.reports_root / candidate
                    candidate = candidate.resolve(strict=False)
                    id_matches = bookmark_id is None or _id_text(event.get("id")) == bookmark_id
                    if id_matches and self._contained(candidate, self.reports_root):
                        bookmark, selected_event = candidate, event
                        if len(bookmark_events) < 100:
                            bookmark_events.append(str(event.get("id") or "")[:128])
        if bookmark_id is not None and selected_event is None:
            raise CaptureError("report bookmark not found")
        if bookmark and bookmark.suffix:
            bookmark = bookmark.parent

        selected_fields = selected_event.get("fields", {}) if selected_event else {}
        bookmark_time = _event_time(selected_event) if selected_event else None
        duration = status.get("duration_us") if status else None
        if not isinstance(duration, int) or isinstance(duration, bool) or duration < 0:
            duration = None
        requested_start_us = selected_fields.get("before_us")
        requested_end_us = selected_fields.get("after_us")
        if not isinstance(requested_start_us, int) or isinstance(requested_start_us, bool) or requested_start_us < 0:
            requested_start_us = max(0, (bookmark_time or 0) - 30_000_000)
        if not isinstance(requested_end_us, int) or isinstance(requested_end_us, bool) or requested_end_us < 0:
            requested_end_us = (bookmark_time or 0) + 5_000_000
        available_start_us = capture_start_us
        available_end_us = max(x for x in (capture_end_us, duration) if x is not None) if any(
            x is not None for x in (capture_end_us, duration)) else None
        actual_start_us = max(requested_start_us, available_start_us) if available_start_us is not None else requested_start_us
        actual_end_us = min(requested_end_us, available_end_us) if available_end_us is not None else requested_end_us
        if actual_end_us < actual_start_us:
            actual_end_us = actual_start_us

        def read_text(name: str, limit: int) -> tuple[str | None, bool]:
            if not bookmark:
                return None, False
            file_path = bookmark / name
            try:
                if not self._contained(file_path, self.reports_root) or not file_path.is_file():
                    return None, False
                with file_path.open("rb") as stream:
                    raw = stream.read(limit + 1)
                truncated = len(raw) > limit
                return raw[:limit].decode("utf-8", errors="replace"), truncated
            except OSError:
                return None, False

        report, report_truncated = read_text("report.txt", MAX_REPORT_BYTES)
        note, note_truncated = read_text("note.txt", MAX_REPORT_BYTES)
        state_text, state_truncated = read_text("state.txt", 64 * 1024)
        pictures, screenshot_filenames = [], []
        if bookmark and bookmark.is_dir():
            for filename in ("picture.png",):
                file_path = bookmark / filename
                try:
                    if self._contained(file_path, self.reports_root) and file_path.is_file():
                        pictures.append({"filename": filename, "bytes": file_path.stat().st_size})
                except OSError:
                    pass
            try:
                screenshot_filenames = sorted(
                    p.name for p in bookmark.iterdir()
                    if p.is_file() and p.suffix.lower() in {".png", ".jpg", ".jpeg"}
                    and (p.name.lower().startswith("screenshot") or p.name.lower() == "picture.png")
                )[:100]
            except OSError:
                pass
        report_activity, activity_bytes, activity_truncated = [], 0, False
        if selected_event and bookmark_time is not None:
            start, end = actual_start_us, actual_end_us
            for event, _, issue in self._iter_events(path):
                if issue == "scan_limit":
                    activity_truncated = True
                    break
                if not event:
                    continue
                event_time = _event_time(event)
                kind = event.get("type", "")
                if (event_time is None or event_time < start or event_time > end or
                        not ("report" in kind or "screenshot" in kind)):
                    continue
                size = len(json.dumps(event, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
                if len(report_activity) >= 100 or activity_bytes + size > MAX_RESPONSE_BYTES // 4:
                    break
                report_activity.append(event)
                activity_bytes += size
        return {**self._base(session_id, status), "report_path": str(bookmark) if bookmark else None,
                "bookmark_evidence_ids": bookmark_events[:100],
                "evidence_id": str(selected_event.get("id"))[:128] if selected_event else None,
                "bookmark_time_us": bookmark_time,
                "window": {"requested_start_us": requested_start_us,
                           "requested_end_us": requested_end_us,
                           "start_us": actual_start_us, "end_us": actual_end_us,
                           "before_us": actual_start_us, "after_us": actual_end_us,
                           "available_start_us": available_start_us,
                           "available_end_us": available_end_us,
                           "shortened_before": actual_start_us > requested_start_us,
                           "shortened_after": actual_end_us < requested_end_us},
                "report_activity": report_activity,
                "report_text": report, "note_text": note, "state_metadata": state_text,
                "picture_metadata": pictures, "screenshot_filenames": screenshot_filenames,
                "bytes_scanned": min(scanned, MAX_SCAN_BYTES),
                "truncated": activity_truncated or any(x in issues for x in {"scan_limit", "oversized_event", "trailing_partial_line"}),
                "report_truncated": report_truncated,
                "note_truncated": note_truncated, "state_truncated": state_truncated,
                "read_issues": issues[:20]}

    def compare_sessions(self, left_id: str, right_id: str) -> dict[str, Any]:
        left = self._session_summary_events(left_id)
        right = self._session_summary_events(right_id)
        differences = _bounded_metadata(_configuration_diff(left["configuration"], right["configuration"]))
        comparison_reasons = []
        if differences:
            comparison_reasons.append("configuration_mismatch")
        for side, report in (("left", left["data_quality"]), ("right", right["data_quality"])):
            if not report["complete"]:
                comparison_reasons.append(f"{side}_session_incomplete")
            if not report["loss_count_known"]:
                comparison_reasons.append(f"{side}_loss_count_unknown")
            if report["dropped_events"]:
                comparison_reasons.append(f"{side}_dropped_events")
            if report["truncated"]:
                comparison_reasons.append(f"{side}_truncated_or_malformed")
        return {"sessions": [left["meta"], right["meta"]], "configuration_differences": differences,
                "comparable_configuration": not differences,
                "benchmark_comparable": not differences and left["data_quality"]["reliable_for_comparison"] and right["data_quality"]["reliable_for_comparison"],
                "data_quality": {"left": left["data_quality"], "right": right["data_quality"]},
                "startup_gameplay": {"left": left["phases"], "right": right["phases"]},
                "cell_loading": {"left": left["cell_loading"], "right": right["cell_loading"]},
                "asset_failures": {"left": left["asset_failures"], "right": right["asset_failures"]},
                "optional_texture_failures": {"left": left["optional_texture_failures"],
                                               "right": right["optional_texture_failures"]},
                "cell_failures": {"left": left["cell_failures"], "right": right["cell_failures"]},
                "evidence_ids": left["evidence_ids"] + right["evidence_ids"],
                "evidence": left["evidence"] + right["evidence"],
                "benchmark_comparability_reasons": comparison_reasons,
                "truncated": left["truncated"] or right["truncated"]}

    def _session_summary_events(self, session_id: str) -> dict[str, Any]:
        path, manifest, status = self._session(session_id)
        frame_values: dict[str, list[float]] = {"startup": [], "gameplay": []}
        phase, failures, optional_failures, cell_failures = "startup", [], [], []
        ids, evidence, truncated = [], [], False
        read_issues: dict[str, int] = {}
        truncation_reasons: set[str] = set()
        cell_collector = _CellLoadingCollector()
        configuration = dict(manifest.get("configuration", {})) if isinstance(manifest.get("configuration"), dict) else {}
        for event, _, issue in self._iter_events(path):
            if issue:
                if len(read_issues) < 20 or issue in read_issues:
                    read_issues[issue] = read_issues.get(issue, 0) + 1
                truncated = True
                truncation_reasons.add(issue)
            if issue == "scan_limit":
                break
            if not event:
                continue
            if len(ids) < 1000:
                ids.append(str(event.get("id") or event.get("sequence") or "")[:128])
                evidence.append({"id": str(event.get("id") or "")[:128], "sequence": event.get("sequence"),
                                 "time_us": event.get("time_us"), "type": str(event.get("type", ""))[:128]})
            else:
                truncated = True
                truncation_reasons.add("evidence_record_limit")
            if event.get("type") == "phase":
                name = (event.get("fields") or {}).get("name")
                if isinstance(name, str) and name in frame_values:
                    phase = name
            fields = event.get("fields") if isinstance(event.get("fields"), dict) else {}
            cell_collector.add(event, phase)
            if event.get("type") == "configuration":
                configuration.update(fields)
                configuration = _bounded_metadata(configuration)
            if event.get("type") in {"main_frame_interval", "main_frame"} and isinstance(fields.get("interval_ms"), (int, float)):
                if math.isfinite(fields["interval_ms"]):
                    if len(frame_values[phase]) < MAX_FRAME_SAMPLES_PER_PHASE:
                        frame_values[phase].append(float(fields["interval_ms"]))
                    else:
                        truncated = True
                        truncation_reasons.add("frame_sample_limit")
            failure_type = event.get("type")
            failure_list = (failures if failure_type in {"texture_failure", "asset_failure"} else
                            optional_failures if failure_type in {"optional_texture_failure", "texture_optional_input_failure"} else
                            cell_failures if failure_type == "cell_failure" else None)
            if failure_list is not None and len(failure_list) < 1000:
                path_value = next((fields[key] for key in ("requested_path", "path", "resolved_path", "base_path")
                                   if isinstance(fields.get(key), str)), "")
                resolved_value = next((fields[key] for key in ("resolved_path", "base_path", "path")
                                       if isinstance(fields.get(key), str)), "")
                failure_list.append({"id": _id_text(event.get("id")), "time_us": _event_time(event),
                                     "type": failure_type, "requested": path_value[:1024],
                                     "resolved": resolved_value[:1024],
                                     "source": str(fields.get("source", ""))[:512],
                                     "outcome": str(fields.get("result", fields.get("outcome", fields.get("failure", ""))))[:512]})
        phases = {name: {"frame_count": len(vals), "median_interval_ms": _percentile(vals, .5),
                         "p95_interval_ms": _percentile(vals, .95), "max_interval_ms": max(vals) if vals else None}
                  for name, vals in frame_values.items()}
        cell_loading = cell_collector.summarize()
        truncated = truncated or cell_loading["truncated"]
        truncation_reasons.update(cell_loading["truncation_reasons"])
        if truncated and not truncation_reasons:
            truncation_reasons.add("capture_truncated")
        data_quality = {
            "complete": bool(status and status.get("complete") is True),
            "dropped_events": self._base(session_id, status)["dropped_events"],
            "loss_count_known": self._base(session_id, status)["loss_count_known"],
            "truncated": truncated,
            "read_issues": read_issues,
            "truncation_reasons": sorted(truncation_reasons),
            "reliable_for_comparison": bool(status and status.get("complete") is True)
                and self._base(session_id, status)["loss_count_known"]
                and self._base(session_id, status)["dropped_events"] == 0
                and not truncated and not read_issues,
        }
        return {"manifest": manifest, "configuration": _bounded_metadata(configuration),
                "phases": phases, "asset_failures": failures,
                "optional_texture_failures": optional_failures, "cell_failures": cell_failures,
                "cell_loading": cell_loading, "data_quality": data_quality,
                "evidence_ids": ids[:1000], "evidence": evidence,
                "truncated": truncated,
                "meta": {**self._base(session_id, status), "duration_us": (status or {}).get("duration_us")}}


def _limit(value: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 1:
        raise CaptureError("limit must be between 1 and 1000")
    return min(value, MAX_PAGE)


def _offset(value: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise CaptureError("offset must be non-negative")
    return value


def _configuration_diff(left: Any, right: Any) -> list[dict[str, Any]]:
    def flatten(value: Any, prefix: str = "") -> dict[str, Any]:
        out = {}
        if isinstance(value, dict):
            for key, child in value.items():
                out.update(flatten(child, f"{prefix}.{key}" if prefix else str(key)))
        else:
            out[prefix] = value
        return out
    a, b = flatten(left), flatten(right)
    return [{"key": key, "left": a.get(key), "right": b.get(key)}
            for key in sorted(a.keys() | b.keys()) if a.get(key) != b.get(key)]

from __future__ import annotations

import json
import sys
import tempfile
import unittest
import io
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE))

from reader import MAX_SCAN_BYTES, CaptureError, CaptureStore  # noqa: E402


def write_session(root: Path, name: str, events: list[dict], *, config=None,
                  trailing: bytes = b"", status=None) -> Path:
    folder = root / name
    folder.mkdir(parents=True)
    (folder / "manifest.json").write_text(json.dumps({
        "version": 1, "session_id": name, "started_at": "2026-10-09T12:00:00Z",
        "configuration": config or {"resolution": [1920, 1080], "plugins": ["base"]},
    }), encoding="utf-8")
    payload = b"".join((json.dumps(event, separators=(",", ":")) + "\n").encode() for event in events)
    (folder / "events-0000.jsonl").write_bytes(payload + trailing)
    (folder / "status.json").write_text(json.dumps(status or {
        "complete": True, "dropped_events": 2, "recorded_bytes": len(payload),
        "duration_us": 900000,
    }), encoding="utf-8")
    return folder


def event(seq: int, kind: str, fields: dict, *, time: int | None = None,
          ident: str | None = None, thread="main") -> dict:
    return {"version": 1, "sequence": seq, "time_us": seq * 1000 if time is None else time,
            "thread": thread, "type": kind, "id": ident or f"e{seq}",
            "parent_id": None, "fields": fields}


class CaptureReaderTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.events = [
            event(1, "phase", {"name": "startup"}),
            event(2, "main_frame_interval", {"interval_ms": 16.0}),
            event(3, "cell_request", {"cell": "A"}, ident="load-A"),
            {**event(4, "cell_worker_start", {}, ident="worker-start", thread="worker-1"), "parent_id": "load-A"},
            event(5, "cell_worker_end", {}, ident="load-A", thread="worker-1"),
            event(6, "cell_insert_start", {}, ident="load-A"),
            event(7, "cell_insert_end", {}, ident="load-A"),
            event(8, "asset_read", {"requested_path": "x.dds", "resolved_path": "Data\\x.dds",
                                     "source": "loose", "bytes": 10, "duration_us": 25,
                                     "result": "ok"}, ident="asset-1"),
            event(9, "phase", {"name": "gameplay"}),
            event(10, "main_frame_interval", {"interval_ms": 90.0}),
            event(11, "asset_read", {"requested_path": "bad.dds", "result": "missing"}, ident="asset-2"),
        ]
        write_session(self.root, "session-a", self.events)

    def tearDown(self):
        self.tmp.cleanup()

    def test_pagination_filters_and_bounds(self):
        store = CaptureStore(self.root)
        first = store.get_events("session-a", event_type="main_frame_interval", limit=1)
        second = store.get_events("session-a", event_type="main_frame_interval", limit=1, offset=1)
        self.assertEqual(first["events"][0]["sequence"], 2)
        self.assertEqual(second["events"][0]["sequence"], 10)
        self.assertFalse(second["has_more"])
        self.assertEqual(first["dropped_events"], 2)
        self.assertEqual(len(store.get_events("session-a", limit=5000)["events"]), len(self.events))

    def test_partial_final_line_and_mismatched_manifest(self):
        write_session(self.root, "interrupted", self.events[:2], trailing=b'{"version":1,"sequence":3')
        store = CaptureStore(self.root, reports_root=self.root / "reports")
        got = store.get_events("interrupted")
        self.assertEqual(len(got["events"]), 2)
        self.assertIn("trailing_partial_line", got["read_issues"])
        wrong = self.root / "wrong"
        wrong.mkdir()
        (wrong / "manifest.json").write_text(json.dumps({"version": 1, "session_id": "somewhere-else"}))
        self.assertNotIn("wrong", [x["session_id"] for x in store.list_sessions()["sessions"]])

    def test_cell_asset_report_and_path_confinement(self):
        store = CaptureStore(self.root, reports_root=self.root / "reports")
        cell = store.inspect_cell_load("session-a", "load-A")
        self.assertEqual([e["type"] for e in cell["events"]], [
            "cell_request", "cell_worker_start", "cell_worker_end", "cell_insert_start", "cell_insert_end"])
        asset = store.inspect_asset("session-a", requested="x.dds")
        self.assertEqual(asset["events"][0]["id"], "asset-1")
        folder = self.root / "session-a"
        report_dir = self.root / "reports" / "007"
        report_dir.mkdir(parents=True)
        (report_dir / "report.txt").write_text("report", encoding="utf-8")
        (report_dir / "state.txt").write_text("location=test", encoding="utf-8")
        (report_dir / "picture.png").write_bytes(b"image")
        (report_dir / "unrelated.bin").write_text("secret", encoding="utf-8")
        with (folder / "events-0000.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(event(12, "report_bookmark", {"report_path": str(report_dir)}, ident="bookmark-1")) + "\n")
        report = store.get_report("session-a")
        self.assertEqual(report["report_text"], "report")
        self.assertEqual(report["state_metadata"], "location=test")
        self.assertEqual(report["picture_metadata"], [{"filename": "picture.png", "bytes": 5}])
        with self.assertRaises(CaptureError):
            store.get_session_summary("../outside")

    def test_slow_frame_context_and_configuration_events(self):
        store = CaptureStore(self.root)
        slow = store.get_slow_frames("session-a", threshold_ms=50, before_us=2000, after_us=2000)
        self.assertEqual([e["sequence"] for e in slow["frames"]], [10])
        self.assertIn(9, [e["sequence"] for e in slow["surrounding_events"]])
        self.assertIn(11, [e["sequence"] for e in slow["surrounding_events"]])

        other = write_session(self.root, "session-config", self.events)
        with (other / "events-0000.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(event(12, "configuration", {"archive_order": ["mod.bsa", "base.bsa"]})) + "\n")
            stream.write(json.dumps(event(13, "texture_failure", {"requested_path": "broken.dds", "failure": "parse"})) + "\n")
        comparison = store.compare_sessions("session-a", "session-config")
        self.assertEqual(comparison["configuration_differences"][0]["key"], "archive_order")
        self.assertIn("texture_failure", [item["type"] for item in comparison["asset_failures"]["right"]])

    def test_numeric_ids_texture_ownership_and_fallback_paths(self):
        self.assertGreaterEqual(MAX_SCAN_BYTES, 512 * 1024 * 1024)
        events = [
            event(1, "cell_request", {"world": "Wasteland", "square": [1, 2]}, ident="123"),
            {**event(2, "cell_worker_start", {}, ident=124), "parent_id": 123},
            event(3, "texture_cache_miss", {"requested_path": "meshes/a.nif", "asset_id": 789}, ident=789),
            event(4, "asset_ownership", {"owner_id": 789, "path": "meshes/a.nif", "owner": "ref"}, ident="owner-event"),
            event(5, "fallback_resolution", {"requested_path": "textures/a.dds", "fallback_path": "textures/b.dds"}, ident=789),
        ]
        write_session(self.root, "numeric-ids", events)
        store = CaptureStore(self.root)
        self.assertEqual(len(store.inspect_cell_load("numeric-ids", "123")["events"]), 2)
        inspected = store.inspect_asset("numeric-ids", asset_id="789")
        self.assertEqual([e["type"] for e in inspected["events"]], [
            "texture_cache_miss", "asset_ownership", "fallback_resolution"])
        self.assertEqual(len(store.inspect_asset("numeric-ids", resolved="textures/b.dds")["events"]), 1)

    def test_report_bookmark_selection_windows_and_bounded_reads(self):
        session = self.root / "bookmarks"
        session.mkdir()
        (session / "manifest.json").write_text(json.dumps({"version": 1, "session_id": "bookmarks"}))
        (session / "status.json").write_text(json.dumps({
            "complete": False, "dropped_events": 4, "duration_us": 40_000_000,
        }))
        reports = self.root / "reports"
        first, second = reports / "010", reports / "011"
        first.mkdir(parents=True)
        second.mkdir()
        (first / "report.txt").write_text("first report", encoding="utf-8")
        (second / "report.txt").write_text("second report", encoding="utf-8")
        events = [
            event(1, "report_bookmark", {"report_path": str(first), "before_us": 30_000_000,
                                          "after_us": 5_000_000}, time=1_000_000, ident="bookmark-one"),
            event(2, "screenshot_write_start", {}, time=1_100_000),
            event(3, "report_bookmark", {"report_path": str(second), "before_us": 30_000_000,
                                          "after_us": 5_000_000}, time=39_000_000, ident="bookmark-two"),
            event(4, "report_write_end", {}, time=39_500_000),
        ]
        (session / "events-0000.jsonl").write_text("".join(json.dumps(item) + "\n" for item in events))
        store = CaptureStore(self.root, reports_root=reports)
        first_result = store.get_report("bookmarks", "bookmark-one")
        self.assertEqual(first_result["report_text"], "first report")
        self.assertTrue(first_result["window"]["shortened_before"])
        self.assertIn("screenshot_write_start", [e["type"] for e in first_result["report_activity"]])
        second_result = store.get_report("bookmarks", "bookmark-two")
        self.assertEqual(second_result["report_text"], "second report")
        self.assertTrue(second_result["window"]["shortened_after"])
        with patch("reader.MAX_REPORT_BYTES", 8):
            bounded = store.get_report("bookmarks", "bookmark-two")
        self.assertEqual(bounded["report_text"], "second r")
        self.assertTrue(bounded["report_truncated"])

    def test_candidate_reads_are_not_final_failure_counts(self):
        events = [
            event(1, "asset_read", {"requested_path": "candidate.dds", "result": "missing"}),
            event(2, "texture_failure", {"requested_path": "bad.dds", "failure": "parse"}),
            event(3, "asset_failure", {"path": "gone.nif", "failure": "not_found"}),
            event(4, "optional_texture_failure", {"path": "optional.dds", "failure": "missing"}),
            event(5, "cell_failure", {"square": [1, 2], "failure": "worker"}),
        ]
        write_session(self.root, "failure-types", events)
        compared = CaptureStore(self.root).compare_sessions("failure-types", "failure-types")
        self.assertEqual([item["type"] for item in compared["asset_failures"]["left"]],
                         ["texture_failure", "asset_failure"])
        self.assertEqual(len(compared["optional_texture_failures"]["left"]), 1)
        self.assertEqual(len(compared["cell_failures"]["left"]), 1)

    def test_cell_loading_stage_distributions_phase_counts_and_provenance(self):
        events = [
            event(1, "phase", {"name": "startup"}, time=0),
            event(2, "cell_request", {}, time=100, ident=100),
            {**event(3, "cell_worker_start", {}, time=150, ident=201), "parent_id": 100},
            {**event(4, "cell_worker_end", {}, time=300, ident=201), "parent_id": 100},
            {**event(5, "cell_result", {}, time=350, ident=202), "parent_id": 100},
            {**event(6, "cell_insert_start", {}, time=400, ident=202), "parent_id": 100},
            {**event(7, "cell_insert_end", {}, time=700, ident=202), "parent_id": 100},
            event(8, "phase", {"name": "gameplay"}, time=800),
            event(9, "cell_request", {}, time=900, ident=300),
            {**event(10, "cell_worker_start", {}, time=1000, ident=301), "parent_id": 300},
            {**event(11, "cell_failure", {"reason": "fixture"}, time=1200, ident=301), "parent_id": 300},
            event(12, "cell_request", {}, time=1300, ident=400),
        ]
        write_session(self.root, "cell-metrics", events, status={
            "complete": True, "dropped_events": 0, "duration_us": 2000,
        })
        result = CaptureStore(self.root).compare_sessions("cell-metrics", "cell-metrics")
        self.assertTrue(result["benchmark_comparable"])
        startup = result["cell_loading"]["left"]["phases"]["startup"]
        gameplay = result["cell_loading"]["left"]["phases"]["gameplay"]
        self.assertEqual(startup["completed_count"], 1)
        self.assertEqual(startup["durations_us"]["request_to_worker_start_queue"]["median_us"], 50)
        self.assertEqual(startup["durations_us"]["worker_start_to_end_work"]["p95_us"], 150)
        self.assertEqual(startup["durations_us"]["worker_end_to_result_queue"]["max_us"], 50)
        self.assertEqual(startup["durations_us"]["insert_start_to_end"]["count"], 1)
        self.assertEqual(startup["durations_us"]["request_to_insert_end_total"]["max_us"], 600)
        self.assertEqual(gameplay["failed_count"], 1)
        self.assertEqual(gameplay["incomplete_count"], 1)
        self.assertTrue(startup["provenance"])
        self.assertEqual(startup["provenance"][0]["load_id"], "100")

    def test_malformed_or_truncated_sessions_are_marked_unreliable(self):
        write_session(self.root, "damaged", self.events[:1], trailing=b"{\"trailing\"")
        comparison = CaptureStore(self.root).compare_sessions("damaged", "damaged")
        self.assertTrue(comparison["truncated"])
        self.assertFalse(comparison["benchmark_comparable"])
        quality = comparison["data_quality"]["left"]
        self.assertTrue(quality["truncated"])
        self.assertIn("trailing_partial_line", quality["read_issues"])
        self.assertIn("left_truncated_or_malformed", comparison["benchmark_comparability_reasons"])

    def test_compare_separates_phases_and_reports_configuration(self):
        write_session(self.root, "session-b", self.events, config={"resolution": [1280, 720]})
        compared = CaptureStore(self.root).compare_sessions("session-a", "session-b")
        self.assertFalse(compared["comparable_configuration"])
        self.assertEqual(compared["startup_gameplay"]["left"]["startup"]["frame_count"], 1)
        self.assertEqual(compared["startup_gameplay"]["left"]["gameplay"]["p95_interval_ms"], 90.0)
        self.assertEqual(len(compared["asset_failures"]["left"]), 0)

    def test_cli_module_does_not_import_mcp(self):
        sys.modules.pop("mcp", None)
        import cli  # noqa: PLC0415
        self.assertNotIn("mcp", sys.modules)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(cli.main(["--capture-root", str(self.root), "summary", "session-a"]), 0)


if __name__ == "__main__":
    unittest.main()

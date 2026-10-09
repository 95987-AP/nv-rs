from __future__ import annotations

import asyncio
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE))

from mcp import Client, StdioServerParameters  # noqa: E402


class McpServerTests(unittest.TestCase):
    def test_sdk_connection_tool_discovery_and_query(self):
        async def exercise(root: Path):
            folder = root / "sdk-session"
            folder.mkdir()
            (folder / "manifest.json").write_text(json.dumps({
                "version": 1, "session_id": "sdk-session", "configuration": {"backend": "vulkan"}
            }), encoding="utf-8")
            event = {"version": 1, "sequence": 1, "time_us": 1234, "thread": "main",
                     "type": "main_frame", "id": "frame-1", "parent_id": None,
                     "fields": {"interval_ms": 22.0, "cpu_ms": 4.0}}
            (folder / "events-0000.jsonl").write_text(json.dumps(event) + "\n", encoding="utf-8")
            (folder / "status.json").write_text(json.dumps({
                "complete": False, "dropped_events": 3, "duration_us": 1234,
            }), encoding="utf-8")

            params = StdioServerParameters(
                command=sys.executable,
                args=[str(HERE / "server.py"), "--capture-root", str(root)],
            )
            async with Client(params) as client:
                tool_names = {tool.name for tool in (await client.list_tools()).tools}
                self.assertEqual(tool_names, {
                    "list_sessions", "get_session_summary", "get_events", "get_slow_frames",
                    "inspect_cell_load", "inspect_asset", "get_report", "compare_sessions",
                })
                result = await client.call_tool("get_events", {"session_id": "sdk-session", "limit": 10})
                self.assertFalse(result.is_error)
                self.assertEqual(result.structured_content["events"][0]["id"], "frame-1")
                self.assertFalse(result.structured_content["complete"])
                self.assertEqual(result.structured_content["dropped_events"], 3)

        with tempfile.TemporaryDirectory() as directory:
            asyncio.run(exercise(Path(directory)))


if __name__ == "__main__":
    unittest.main()

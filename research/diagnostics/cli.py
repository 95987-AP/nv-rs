"""Small stdlib-only CLI for diagnostic capture summaries and comparisons."""

from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Any

from reader import CaptureError, CaptureStore


def _store(root: str | None) -> CaptureStore:
    configured = root or os.environ.get("NV_DIAGNOSTICS_DIR")
    if not configured:
        raise CaptureError("set --capture-root or NV_DIAGNOSTICS_DIR")
    return CaptureStore(configured)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Read nv-rs diagnostic captures")
    parser.add_argument("--capture-root", help="configured capture root")
    sub = parser.add_subparsers(dest="command", required=True)
    listing = sub.add_parser("list", help="list saved sessions")
    listing.add_argument("--limit", type=int, default=100)
    summary = sub.add_parser("summary", help="summarize one session")
    summary.add_argument("session_id")
    compare = sub.add_parser("compare", help="compare startup/gameplay frame and asset evidence")
    compare.add_argument("left_session_id")
    compare.add_argument("right_session_id")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        store = _store(args.capture_root)
        if args.command == "list":
            result: Any = store.list_sessions(limit=args.limit)
        elif args.command == "summary":
            result = store.get_session_summary(args.session_id)
        else:
            result = store.compare_sessions(args.left_session_id, args.right_session_id)
        json.dump(result, sys.stdout, indent=2, ensure_ascii=False)
        sys.stdout.write("\n")
        return 0
    except (CaptureError, OSError) as exc:
        print(f"diagnostics: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

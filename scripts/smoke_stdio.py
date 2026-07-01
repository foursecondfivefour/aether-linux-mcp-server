#!/usr/bin/env python3
"""Minimal stdio smoke test for AETHER_02.

The rmcp stdio transport uses newline-delimited JSON-RPC messages. This script
starts the server, verifies initialize/tools/list, and calls a safe read-only
system_info action.
"""

from __future__ import annotations

import json
import os
import selectors
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BIN = ROOT / "target" / "debug" / "aether-mcp-server"
LEGACY_BIN = ROOT / "target" / "debug" / "aether-linux-mcp-server"
BIN = Path(os.environ.get("AETHER_BIN", DEFAULT_BIN if DEFAULT_BIN.exists() else LEGACY_BIN))


def send(proc: subprocess.Popen[str], obj: dict) -> None:
    assert proc.stdin is not None
    proc.stdin.write(json.dumps(obj, separators=(",", ":")) + "\n")
    proc.stdin.flush()


def read_until_id(
    selector: selectors.DefaultSelector,
    proc: subprocess.Popen[str],
    request_id: int,
    seconds: float = 5.0,
) -> tuple[dict | None, list[str], list[str]]:
    outs: list[str] = []
    errs: list[str] = []
    end = time.time() + seconds

    while time.time() < end:
        events = selector.select(max(0.0, end - time.time()))
        if not events:
            continue

        for key, _ in events:
            line = key.fileobj.readline()
            if not line:
                continue
            line = line.rstrip("\n")

            if key.fileobj is proc.stdout:
                outs.append(line)
                try:
                    message = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if message.get("id") == request_id:
                    return message, outs, errs
            else:
                errs.append(line)

    return None, outs, errs


def main() -> int:
    if not BIN.exists():
        print(f"Binary not found: {BIN}", file=sys.stderr)
        print("Run `cargo build` first or set AETHER_BIN=/path/to/binary.", file=sys.stderr)
        return 2

    proc = subprocess.Popen(
        [str(BIN)],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    selector = selectors.DefaultSelector()
    assert proc.stdout is not None and proc.stderr is not None
    selector.register(proc.stdout, selectors.EVENT_READ)
    selector.register(proc.stderr, selectors.EVENT_READ)

    try:
        send(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "aether-smoke", "version": "0.1.0"},
                },
            },
        )
        init, outs, errs = read_until_id(selector, proc, 1)
        if not init or "result" not in init:
            print("initialize failed", file=sys.stderr)
            print("stdout:", outs, file=sys.stderr)
            print("stderr:", errs, file=sys.stderr)
            return 1

        send(proc, {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        send(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
        tools, outs, errs = read_until_id(selector, proc, 2)
        if not tools or "process_control" not in json.dumps(tools):
            print("tools/list failed", file=sys.stderr)
            print("stdout:", outs, file=sys.stderr)
            print("stderr:", errs, file=sys.stderr)
            return 1

        send(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "system_info",
                    "arguments": {"action": "uptime", "params": {}},
                },
            },
        )
        call, outs, errs = read_until_id(selector, proc, 3)
        if not call or "result" not in call or call["result"].get("isError"):
            print("tools/call failed", file=sys.stderr)
            print("stdout:", outs, file=sys.stderr)
            print("stderr:", errs, file=sys.stderr)
            return 1

        print("SMOKE_OK")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())

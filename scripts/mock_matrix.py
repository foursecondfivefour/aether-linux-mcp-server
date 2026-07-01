#!/usr/bin/env python3
"""MCP-level mocked action matrix for AETHER_02.

This starts the server with AETHER_MOCK_COMMANDS=1 and calls representative
implemented actions across all 12 tool groups. Dangerous calls use dry_run or
verify that force/feature gates block them.
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
BIN = Path(os.environ.get("AETHER_BIN", ROOT / "target" / "debug" / "aether-mcp-server"))

CASES: list[tuple[str, str, dict, str]] = [
    ("system_info", "uptime", {}, "ok"),
    ("system_info", "disk_list", {"dry_run": True}, "DRY-RUN"),
    ("process_control", "list", {"dry_run": True}, "DRY-RUN"),
    ("process_control", "kill", {"pid": "1"}, "Force required"),
    ("process_control", "kill", {"pid": "1", "dry_run": True}, "DRY-RUN"),
    ("file_system", "exists", {"path": "/etc/os-release"}, "ok"),
    ("file_system", "disk_list", {"dry_run": True}, "DRY-RUN"),
    ("file_system", "delete", {"path": "/tmp/aether-nope"}, "Force required"),
    ("file_system", "delete", {"path": "/tmp/aether-nope", "dry_run": True}, "DRY-RUN"),
    ("package_manager", "search", {"package": "bash"}, "MOCK"),
    ("package_manager", "install", {"package": "curl"}, "Force required"),
    ("package_manager", "install", {"package": "curl", "dry_run": True}, "DRY-RUN"),
    ("system_config", "os_release", {}, "ok"),
    ("system_config", "modprobe_load", {"module": "dummy", "force": True}, "Feature disabled"),
    ("system_config", "hostname_set", {"hostname": "aether-test"}, "Force required"),
    ("service_manager", "status", {"service": "ssh"}, "MOCK"),
    ("service_manager", "stop", {"service": "ssh"}, "Force required"),
    ("service_manager", "stop", {"service": "ssh", "dry_run": True}, "DRY-RUN"),
    ("gui_automation", "display_list", {}, "MOCK"),
    ("gui_automation", "audio_volume_set", {"volume": "10%", "dry_run": True}, "DRY-RUN"),
    ("network_manager", "route_list", {}, "MOCK"),
    ("network_manager", "adapter_down", {"interface": "eth0"}, "Force required"),
    ("network_manager", "adapter_down", {"interface": "eth0", "dry_run": True}, "DRY-RUN"),
    ("user_management", "current_user", {}, "MOCK"),
    ("user_management", "user_create", {"username": "aether-test", "force": True}, "Feature disabled"),
    ("security_audit", "aslr_status", {}, "ok"),
    ("security_audit", "firewall_status", {"dry_run": True}, "DRY-RUN"),
    ("hardware_control", "usb_devices", {}, "MOCK"),
    ("hardware_control", "cpufreq_governor_set", {"governor": "powersave"}, "Force required"),
    ("hardware_control", "cpufreq_governor_set", {"governor": "powersave", "dry_run": True}, "DRY-RUN"),
    ("system_automation", "journal_query", {}, "MOCK"),
    ("system_automation", "sysctl_d_apply", {}, "Force required"),
    ("system_automation", "sysctl_d_apply", {"dry_run": True}, "DRY-RUN"),
]


def send(proc: subprocess.Popen[str], obj: dict) -> None:
    assert proc.stdin is not None
    proc.stdin.write(json.dumps(obj, separators=(",", ":")) + "\n")
    proc.stdin.flush()


def read_until_id(selector: selectors.DefaultSelector, proc: subprocess.Popen[str], request_id: int, seconds: float = 5.0) -> dict:
    end = time.time() + seconds
    stdout: list[str] = []
    stderr: list[str] = []
    while time.time() < end:
        for key, _ in selector.select(max(0.0, end - time.time())):
            line = key.fileobj.readline()
            if not line:
                continue
            line = line.rstrip("\n")
            if key.fileobj is proc.stdout:
                stdout.append(line)
                try:
                    msg = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if msg.get("id") == request_id:
                    return msg
            else:
                stderr.append(line)
    raise RuntimeError(f"timeout waiting for id={request_id}; stdout={stdout}; stderr={stderr[-10:]}")


def text_from_result(message: dict) -> str:
    if "error" in message:
        return json.dumps(message["error"], ensure_ascii=False)
    result = message.get("result", {})
    content = result.get("content", [])
    if content and isinstance(content[0], dict):
        return str(content[0].get("text", ""))
    return json.dumps(result, ensure_ascii=False)


def main() -> int:
    if not BIN.exists():
        print(f"Binary not found: {BIN}. Run cargo build first.", file=sys.stderr)
        return 2

    env = os.environ.copy()
    env["AETHER_MOCK_COMMANDS"] = "1"
    proc = subprocess.Popen(
        [str(BIN)],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=env,
    )
    selector = selectors.DefaultSelector()
    assert proc.stdout is not None and proc.stderr is not None
    selector.register(proc.stdout, selectors.EVENT_READ)
    selector.register(proc.stderr, selectors.EVENT_READ)

    try:
        send(proc, {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "aether-mock-matrix", "version": "0.1.0"},
            },
        })
        init = read_until_id(selector, proc, 1)
        if "result" not in init:
            raise RuntimeError(f"initialize failed: {init}")
        send(proc, {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})

        failures: list[str] = []
        for i, (tool, action, params, expected) in enumerate(CASES, start=2):
            send(proc, {
                "jsonrpc": "2.0",
                "id": i,
                "method": "tools/call",
                "params": {"name": tool, "arguments": {"action": action, "params": params}},
            })
            msg = read_until_id(selector, proc, i)
            text = text_from_result(msg)
            if expected == "ok":
                bad = "panic" in text.lower() or "not implemented" in text.lower()
            else:
                bad = expected not in text
            if bad:
                failures.append(f"{tool}.{action}: expected {expected!r}, got {text[:300]!r}")

        if failures:
            print("MOCK_MATRIX_FAILED", file=sys.stderr)
            print("\n".join(failures), file=sys.stderr)
            return 1

        print(f"MOCK_MATRIX_OK {len(CASES)} cases")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())

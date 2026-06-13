#!/usr/bin/env python3
"""Small deterministic JSON-RPC client for Sifr LSP protocol tests."""

from __future__ import annotations

import json
import os
import select
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]


class LspProtocolError(RuntimeError):
    pass


class LspClient:
    def __init__(self, timeout: float = 90.0, extra_args: list[str] | None = None) -> None:
        command = os.environ.get("SIFR_LSP_COMMAND")
        binary = REPO_ROOT / "target" / "debug" / "sifr"
        args = (
            command.split()
            if command
            else [str(binary), "lsp", "--stdio"]
            if binary.exists()
            else ["cargo", "run", "-q", "-p", "sifr", "--", "lsp", "--stdio"]
        )
        if extra_args:
            args.extend(extra_args)
        self.process = subprocess.Popen(
            args,
            cwd=REPO_ROOT,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.args = args
        self.timeout = timeout
        self.next_id = 1
        self.notifications: list[dict[str, Any]] = []
        self.events: list[dict[str, Any]] = []
        self._record_event("spawn", {"pid": self.process.pid, "args": args})

    def request(self, method: str, params: dict[str, Any] | None = None) -> Any:
        request_id = self.next_id
        self.next_id += 1
        self._send({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params or {}})
        response = self._wait_for_response(request_id)
        if "error" in response:
            raise LspProtocolError(f"{method} returned error: {response['error']}")
        return response.get("result")

    def request_error(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        request_id = self.next_id
        self.next_id += 1
        self._send({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params or {}})
        response = self._wait_for_response(request_id)
        error = response.get("error")
        if not isinstance(error, dict):
            raise LspProtocolError(f"{method} succeeded; expected protocol error")
        return error

    def notify(self, method: str, params: dict[str, Any] | None = None) -> None:
        self._send({"jsonrpc": "2.0", "method": method, "params": params or {}})

    def wait_for_notification(self, method: str) -> dict[str, Any]:
        deadline = time.monotonic() + self.timeout
        while time.monotonic() < deadline:
            for index, notification in enumerate(self.notifications):
                if notification.get("method") == method:
                    return self.notifications.pop(index)
            message = self._read_message(deadline)
            if "method" in message and "id" not in message:
                self.notifications.append(message)
        raise LspProtocolError(f"timed out waiting for notification {method}")

    def close(self) -> None:
        try:
            self.notify("exit")
        except Exception:
            pass
        if self.process.stdin is not None:
            self.process.stdin.close()
        try:
            self.process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=10)
            stderr = self.process.stderr.read().decode("utf-8", errors="replace") if self.process.stderr else ""
            raise LspProtocolError(self._diagnostic_context("LSP did not exit after stdin close", stderr))
        if self.process.returncode not in {0, None}:
            stderr = self.process.stderr.read().decode("utf-8", errors="replace") if self.process.stderr else ""
            raise LspProtocolError(self._diagnostic_context(f"LSP exited {self.process.returncode}", stderr))

    def _wait_for_response(self, request_id: int) -> dict[str, Any]:
        deadline = time.monotonic() + self.timeout
        while time.monotonic() < deadline:
            message = self._read_message(deadline)
            if message.get("id") == request_id:
                return message
            if "method" in message and "id" not in message:
                self.notifications.append(message)
        raise LspProtocolError(f"timed out waiting for response {request_id}")

    def _send(self, payload: dict[str, Any]) -> None:
        if self.process.stdin is None:
            raise LspProtocolError("LSP stdin is closed")
        self._record_event("send", payload)
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
        self.process.stdin.write(header + body)
        self.process.stdin.flush()

    def _read_message(self, deadline: float) -> dict[str, Any]:
        if self.process.stdout is None:
            raise LspProtocolError("LSP stdout is closed")
        while True:
            if self.process.poll() is not None:
                stderr = self.process.stderr.read().decode("utf-8", errors="replace") if self.process.stderr else ""
                raise LspProtocolError(
                    self._diagnostic_context(f"LSP exited before response: {self.process.returncode}", stderr)
                )
            remaining = max(deadline - time.monotonic(), 0.0)
            if remaining == 0:
                raise LspProtocolError("timed out waiting for LSP output")
            ready, _, _ = select.select([self.process.stdout], [], [], remaining)
            if ready:
                break
        header = b""
        while b"\r\n\r\n" not in header:
            chunk = self.process.stdout.read(1)
            if not chunk:
                raise LspProtocolError("LSP closed stdout while reading header")
            header += chunk
        length = None
        for line in header.decode("ascii", errors="replace").split("\r\n"):
            if line.lower().startswith("content-length:"):
                length = int(line.split(":", 1)[1].strip())
                break
        if length is None:
            raise LspProtocolError(f"missing Content-Length header: {header!r}")
        body = self.process.stdout.read(length)
        if len(body) != length:
            raise LspProtocolError("LSP closed stdout while reading body")
        message = json.loads(body.decode("utf-8"))
        self._record_event("recv", message)
        return message

    def _record_event(self, kind: str, payload: dict[str, Any]) -> None:
        event = {
            "kind": kind,
            "at_unix": round(time.time(), 3),
            "summary": protocol_summary(payload),
        }
        self.events.append(event)
        self.events = self.events[-20:]

    def _diagnostic_context(self, reason: str, stderr: str) -> str:
        lifecycle = {
            "pid": self.process.pid,
            "returncode": self.process.returncode,
            "args": self.args,
        }
        return (
            f"{reason}\n"
            f"lifecycle={json.dumps(lifecycle, sort_keys=True)}\n"
            f"last_protocol_events={json.dumps(self.events[-10:], sort_keys=True)}\n"
            f"stderr_tail={stderr[-4000:]}"
        )


def file_uri(path: Path) -> str:
    return path.resolve().as_uri()


def protocol_summary(payload: dict[str, Any]) -> dict[str, Any]:
    summary: dict[str, Any] = {}
    if "id" in payload:
        summary["id"] = payload.get("id")
    if "method" in payload:
        summary["method"] = payload.get("method")
    if "error" in payload:
        error = payload.get("error")
        if isinstance(error, dict):
            summary["error"] = {
                "code": error.get("code"),
                "message": error.get("message"),
            }
    if "result" in payload:
        summary["result_type"] = type(payload.get("result")).__name__
    params = payload.get("params")
    if isinstance(params, dict):
        if isinstance(params.get("textDocument"), dict):
            summary["uri"] = params["textDocument"].get("uri")
            if "version" in params["textDocument"]:
                summary["version"] = params["textDocument"].get("version")
        if "uri" in params:
            summary["uri"] = params.get("uri")
    return summary


def assert_has_keys(value: dict[str, Any], keys: set[str], label: str) -> None:
    missing = sorted(keys - set(value))
    if missing:
        raise LspProtocolError(f"{label} is missing keys: {missing}")


def main() -> int:
    print("lsp protocol helper module; run lsp_protocol_smoke.py or lsp_protocol_stress.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())

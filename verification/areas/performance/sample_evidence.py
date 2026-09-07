"""Retain command measurements before warmup exclusion or sample rejection."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

from controlled_sampling import write_json


def record_command_sample(
    run_root: Path,
    case_id: str,
    sample_index: int,
    warmup: bool,
    command: list[str],
    result: dict[str, Any],
) -> None:
    # Command stdout can contain arbitrarily large generated sources. Bind it
    # without duplicating it; stderr_tail already retains the timing counters.
    stdout = result.get("stdout", "")
    stdout_bytes = stdout if isinstance(stdout, bytes) else stdout.encode("utf-8")
    evidence = {key: value for key, value in result.items() if key != "stdout"}
    stderr = evidence.get("stderr_tail")
    if isinstance(stderr, bytes):
        evidence["stderr_tail"] = stderr.decode("utf-8", errors="replace")
    write_json(
        run_root / "samples" / case_id / f"{sample_index}.json",
        {
            "schema_version": 1,
            "case_id": case_id,
            "sample_index": sample_index,
            "warmup": warmup,
            "command": command,
            "stdout_sha256": hashlib.sha256(stdout_bytes).hexdigest(),
            "stdout_bytes": len(stdout_bytes),
            "result": evidence,
        },
    )


def run_self_test() -> None:
    with TemporaryDirectory(prefix="sifr-command-evidence-") as raw:
        root = Path(raw)
        result = {
            "retired_instructions": 123,
            "exit_code": 0,
            "timed_out": False,
            "stdout": "formatted\n",
            "stderr_tail": "123 instructions retired\n",
        }
        record_command_sample(root, "case", 0, True, ["sifr", "fmt"], result)
        saved = json.loads((root / "samples/case/0.json").read_text())
        assert saved["warmup"] is True
        assert saved["result"]["retired_instructions"] == 123
        assert saved["stdout_sha256"] == hashlib.sha256(b"formatted\n").hexdigest()
        assert "stdout" not in saved["result"]
        assert result["stdout"] == "formatted\n"
        failed = {**result, "stdout": b"partial", "stderr_tail": b"timeout", "timed_out": True}
        record_command_sample(root, "case", 1, False, ["sifr", "fmt"], failed)
        saved = json.loads((root / "samples/case/1.json").read_text())
        assert saved["warmup"] is False
        assert saved["result"]["timed_out"] is True
        assert saved["result"]["stderr_tail"] == "timeout"
        assert (root / "samples/case/0.json").exists()

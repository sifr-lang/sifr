"""Retain command measurements before warmup exclusion or sample rejection."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any
from collections.abc import Callable
from unittest.mock import Mock

from controlled_sampling import write_json


def record_command_sample(
    run_root: Path,
    case_id: str,
    sample_index: int,
    warmup: bool,
    command: list[str],
    result: dict[str, Any],
    *,
    query_role: str | None = None,
) -> None:
    # Generated stdout may be large. Bind its available representation without
    # duplicating it. B23 returns decoded text; do not claim original pipe bytes.
    stdout = result["stdout"]
    stdout_bytes = stdout if isinstance(stdout, bytes) else stdout.encode("utf-8")
    stderr = result["stderr"]
    stderr_bytes = stderr if isinstance(stderr, bytes) else stderr.encode("utf-8")
    sample_dir = run_root / "samples" / case_id
    sample_dir.mkdir(parents=True, exist_ok=True)
    stderr_path = sample_dir / f"{sample_index}.stderr"
    stderr_path.write_bytes(stderr_bytes)
    query_evidence = {}
    if query_role is not None:
        stdout_path = sample_dir / f"{sample_index}.stdout"
        stdout_path.write_bytes(stdout_bytes)
        query_evidence = {"query_role": query_role, "stdout_path": str(stdout_path)}
    evidence = {
        key: value for key, value in result.items() if key not in {"stdout", "stderr"}
    }
    if isinstance(evidence.get("stderr_tail"), bytes):
        evidence["stderr_tail"] = evidence["stderr_tail"].decode("utf-8", errors="replace")
    write_json(
        sample_dir / f"{sample_index}.json",
        {
            "schema_version": 1,
            "case_id": case_id,
            "sample_index": sample_index,
            "warmup": warmup,
            "command": command,
            "stdout_sha256": hashlib.sha256(stdout_bytes).hexdigest(),
            "stdout_bytes": len(stdout_bytes),
            "stdout_representation": "bytes" if isinstance(stdout, bytes) else "decoded-text-utf8",
            "stderr": {
                "path": str(stderr_path),
                "sha256": hashlib.sha256(stderr_bytes).hexdigest(),
                "bytes": len(stderr_bytes),
                "representation": "bytes" if isinstance(stderr, bytes) else "decoded-text-utf8",
            },
            "result": evidence,
            **query_evidence,
        },
    )


def recording_query_runner(
    run_root: Path,
    case_id: str,
    run_process: Callable[[list[str], int], dict[str, Any]],
) -> Callable[[list[str], int], dict[str, Any]]:
    index = 0

    def run(command: list[str], timeout_ms: int) -> dict[str, Any]:
        nonlocal index
        result = run_process(command, timeout_ms)
        # Each query process includes its own protocol-defined warmup iterations.
        # Index 0 is aggregate latency; later processes supply work samples.
        record_command_sample(
            run_root, case_id, index, False, command, result,
            query_role="aggregate-with-internal-warmups" if index == 0 else "work-sample-with-internal-warmups",
        )
        index += 1
        return result

    return run


def run_self_test() -> None:
    with TemporaryDirectory(prefix="sifr-command-evidence-") as raw:
        root = Path(raw)
        for index, stderr in enumerate(["x" * 3000 + "123 instructions retired\n", b"partial\xff"]):
            stdout = "formatted café\n" if index == 0 else b"partial\xff"
            result = {
                "retired_instructions": 123 if index == 0 else None,
                "exit_code": 0 if index == 0 else None,
                "timed_out": index == 1,
                "stdout": stdout,
                "stderr": stderr,
                "stderr_tail": stderr[-2000:],
            }
            before = result.copy()
            record_command_sample(root, "case", index, index == 0, ["sifr", "fmt"], result)
            saved = json.loads((root / f"samples/case/{index}.json").read_text())
            raw_stderr = stderr if isinstance(stderr, bytes) else stderr.encode("utf-8")
            raw_stdout = stdout if isinstance(stdout, bytes) else stdout.encode("utf-8")
            assert saved["warmup"] is (index == 0)
            assert saved["result"]["timed_out"] is (index == 1)
            assert saved["stdout_sha256"] == hashlib.sha256(raw_stdout).hexdigest()
            assert saved["stdout_bytes"] == len(raw_stdout)
            assert saved["stderr"]["sha256"] == hashlib.sha256(raw_stderr).hexdigest()
            assert saved["stderr"]["bytes"] == len(raw_stderr)
            assert Path(saved["stderr"]["path"]).read_bytes() == raw_stderr
            assert "stdout" not in saved["result"] and "stderr" not in saved["result"]
            assert result == before
        assert (root / "samples/case/0.json").exists()
        process = Mock(return_value=result)
        record_query = recording_query_runner(root, "query", process)
        for command, timeout in [(["query", "3"], 100), (["query", "2"], 200)]:
            assert record_query(command, timeout) is result
        assert process.call_count == 2
        assert process.call_args_list[0].args == (["query", "3"], 100)
        assert process.call_args_list[1].args == (["query", "2"], 200)
        for index in range(2):
            saved = json.loads((root / f"samples/query/{index}.json").read_text())
            assert Path(saved["stdout_path"]).read_bytes() == result["stdout"]
        print("benchmark evidence: text/bytes, full stderr, hash binding, unchanged stdout passed")

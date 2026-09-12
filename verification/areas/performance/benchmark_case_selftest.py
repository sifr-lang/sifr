"""Deterministic command-case warmup validation and evidence contracts."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from types import ModuleType
from unittest.mock import patch

from benchmark_manifest import BenchmarkCase, BenchmarkError
from controlled_sampling import run_controlled_case


class FakeMonitor:
    def __init__(self, **_kwargs: object) -> None:
        self.snapshots = [{"self_test": True}]

    def __enter__(self) -> FakeMonitor:
        return self

    def __exit__(self, *_args: object) -> None:
        pass

    def rejection_reasons(self) -> list[str]:
        return []


def run_self_test(runner: ModuleType) -> None:
    base = {
        "id": "warmup-contract", "kind": "command", "mode": "check",
        "group": "check-single-file", "source_path": "unused.sifr",
        "warmups": 2, "measured": 3, "timeout_ms": 1234,
        "expected_exit_codes": [0], "budget_id": "self-test",
        "evidence_category": "self-test",
    }
    valid = {
        "duration_ms": 10.0, "peak_rss_bytes": 100,
        "retired_instructions": 1000, "cycles_elapsed": 2000,
        "exit_code": 0, "timed_out": False,
        "stdout": "unchanged\n", "stderr": "raw timing\n", "stderr_tail": "raw timing\n",
    }
    bad_timeout = {**valid, "timed_out": True, "stderr": b"partial\xff"}
    failures = [
        ("warmup_timeout_stops_before_measurement", [bad_timeout], "timed out after 1234ms"),
        ("warmup_nonzero_exit_stops_before_measurement", [{**valid, "exit_code": 7}], "exited 7, expected [0]"),
        ("later_warmup_failure_stops_before_measurement", [valid, {**valid, "exit_code": 7}], "exited 7"),
        ("measured_timeout_rejected", [valid, valid, bad_timeout], "timed out"),
        ("measured_unexpected_exit_rejected", [valid, valid, {**valid, "exit_code": 7}], "exited 7"),
    ]
    with TemporaryDirectory(prefix="sifr-warmup-contract-") as raw:
        root = Path(raw)
        for name, results, diagnostic in failures:
            case_root = root / name
            with (
                patch.object(runner, "ensure_sifr_binary") as ensure,
                patch.object(runner, "run_subprocess", side_effect=results) as launch,
            ):
                try:
                    run_controlled_case(
                        BenchmarkCase(base), case_root, "manifest",
                        require_controlled_host=True, control_mode="work",
                        run_case_fn=runner.run_case, retry_admission_fn=None,
                        monitor_factory=FakeMonitor,
                    )
                except BenchmarkError as error:
                    assert diagnostic in str(error), str(error)
                else:
                    raise AssertionError(f"{name} admitted an invalid result")
                assert launch.call_count == len(results), name
                ensure.assert_called_once()
            failure = json.loads((case_root / "control-failures/warmup-contract.json").read_text())
            assert len(failure["attempts"]) == 1
            attempt = failure["attempts"][0]
            assert attempt["attempt"] == 1 and attempt["status"] == "error"
            assert attempt["host_snapshots"] == [{"self_test": True}]
            assert len(attempt["sample_receipts"]) == len(results)
            for index, ref in enumerate(attempt["sample_receipts"]):
                sample = json.loads(Path(ref).read_text())
                assert sample["sample_index"] == index
                assert sample["warmup"] == (index < 2)
                stderr = results[index]["stderr"]
                stderr_bytes = stderr if isinstance(stderr, bytes) else stderr.encode("utf-8")
                assert Path(sample["stderr"]["path"]).read_bytes() == stderr_bytes
                assert sample["stderr"]["sha256"] == hashlib.sha256(stderr_bytes).hexdigest()
            assert not (case_root / "attempts/2").exists()
            print(f"benchmark case: {name}, durable receipts and no following launch passed")

        for scale, warmups, measured in [("manifest", 2, 3), ("smoke", 1, 1)]:
            for expected in ([1], [0, 1]):
                case = BenchmarkCase({**base, "expected_exit_codes": expected})
                warm = {**valid, "exit_code": 1, "duration_ms": 9999.0,
                        "peak_rss_bytes": 9999, "retired_instructions": 99999,
                        "cycles_elapsed": 99999}
                observation = {**valid, "exit_code": 1}
                results = [warm] * warmups + [observation] * measured
                case_root = root / f"valid-{scale}-{len(expected)}"
                with (
                    patch.object(runner, "ensure_sifr_binary"),
                    patch.object(runner, "run_subprocess", side_effect=results) as launch,
                ):
                    report = runner.run_case(case, case_root, scale)
                assert launch.call_count == warmups + measured
                assert report["sample_count"] == measured
                assert report["samples_ms"] == [10.0] * measured
                assert report["metrics"]["peak_rss_bytes"] == 100
                assert report["samples_instructions"] == [1000] * measured
                assert report["metrics"]["median_instructions"] == 1000
                assert report["metrics"]["median_cycles_per_instruction"] == 2.0
                assert report["metrics"]["median_ms"] == 10.0
                assert report["metrics"]["p95_ms"] == 10.0
                assert len(list((case_root / "samples/warmup-contract").glob("*.json"))) == warmups + measured
                json.dumps(report)
                print(f"benchmark case: allowed expected exits {expected}, {scale} exact counts and warmup exclusion passed")

        for kind in ("frontend-query", "lsp-query"):
            case = BenchmarkCase({**base, "kind": kind, "scenario": "self-test",
                                  "project_root": "unused", "measured": 2, "warmups": 1})
            payloads = [[999, 10, 10], [999, 10], [999, 10]]
            results = [{**valid, "stdout": json.dumps({"samples_ms": samples})}
                       for samples in payloads]
            case_root = root / kind
            with (
                patch.object(runner, "ensure_frontend_query_bench"),
                patch.object(runner, "run_subprocess", side_effect=results) as launch,
            ):
                report = runner.run_case(case, case_root, "manifest")
            assert launch.call_count == 3
            assert report["samples_ms"] == [10, 10]
            assert report["samples_instructions"] == [1000, 1000]
            for index in range(3):
                saved = json.loads((case_root / f"samples/warmup-contract/{index}.json").read_text())
                assert saved["query_role"] == ("aggregate-with-internal-warmups" if index == 0
                                                 else "work-sample-with-internal-warmups")
                assert Path(saved["stdout_path"]).read_text() == results[index]["stdout"]
                assert saved["warmup"] is False
            with (
                patch.object(runner, "ensure_frontend_query_bench"),
                patch.object(runner, "run_subprocess", return_value=bad_timeout) as launch,
            ):
                failed_root = root / f"{kind}-failure"
                try:
                    runner.run_case(case, failed_root, "manifest")
                except BenchmarkError as error:
                    assert "timed out" in str(error)
                else:
                    raise AssertionError("query timeout did not fail")
                launch.assert_called_once()
            assert (failed_root / "samples/warmup-contract/0.json").exists()
            malformed = {**valid, "stdout": "{malformed"}
            with (
                patch.object(runner, "ensure_frontend_query_bench"),
                patch.object(runner, "run_subprocess", return_value=malformed) as launch,
            ):
                invalid_root = root / f"{kind}-invalid-json"
                try:
                    runner.run_case(case, invalid_root, "manifest")
                except BenchmarkError as error:
                    assert "invalid JSON" in str(error)
                else:
                    raise AssertionError("query malformed JSON did not fail")
                launch.assert_called_once()
            saved = json.loads((invalid_root / "samples/warmup-contract/0.json").read_text())
            assert Path(saved["stdout_path"]).read_text() == "{malformed"
            print(f"benchmark case: {kind} aggregate/work receipts, internal warmups, failure-before-exception passed")

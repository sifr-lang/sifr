from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from env import RunnerPaths
from live_case_config import LIVE_CASES, LIVE_IMAGES
from live_packages import BuiltLiveBinary, build_live_binaries, validate_live_source_presence
from live_services import run_live_cases


@dataclass(frozen=True)
class DockerAvailability:
    available: bool | None
    reason: str


SourceBuilder = Callable[
    [RunnerPaths],
    tuple[list[dict[str, Any]], dict[str, BuiltLiveBinary]],
]
LiveRunner = Callable[[dict[str, BuiltLiveBinary]], list[dict[str, Any]]]


def build_live_examples_report(
    paths: RunnerPaths,
    docker_probe: Callable[[], DockerAvailability] | None = None,
    live_runner: LiveRunner | None = None,
    compile_sources: bool = True,
    source_builder: SourceBuilder | None = None,
) -> dict[str, Any]:
    if compile_sources:
        builder = source_builder or build_live_binaries
        source_checks, binaries = builder(paths)
    else:
        source_checks = validate_live_source_presence(paths)
        binaries = {}
    failed_sources = [check for check in source_checks if check["status"] != "pass"]
    if failed_sources:
        return _report(
            status="live-failed",
            source_checks=source_checks,
            cases=[],
            skipped=0,
            failures=len(failed_sources),
            docker=DockerAvailability(None, "Docker probe skipped because binary builds failed"),
        )

    probe = docker_probe or probe_docker
    docker = probe()
    if docker.available is None:
        raise SystemExit("python interop live examples Docker probe returned an unprobed result")
    if docker.available is False:
        cases = [
            {
                "id": case.case_id,
                "status": "structured-skip",
                "reason": docker.reason,
                "sifr_source": case.relative_source,
                "execution_model": "compiled-sifr-binary",
                "binary_built": case.case_id in binaries,
                "binary_executed": False,
            }
            for case in LIVE_CASES.values()
        ]
        return _report(
            status="structured-skip",
            source_checks=source_checks,
            cases=cases,
            skipped=len(cases),
            failures=0,
            docker=docker,
        )

    runner = live_runner or run_live_cases
    cases = runner(binaries)
    failures = sum(1 for case in cases if case.get("status") != "live-passed")
    observed_ids = [case.get("id") for case in cases]
    invalid_execution = [
        case.get("id")
        for case in cases
        if case.get("status") == "live-passed"
        and (
            case.get("execution_model") != "compiled-sifr-binary"
            or case.get("binary_built") is not True
            or case.get("binary_executed") is not True
        )
    ]
    if (
        len(observed_ids) != len(set(observed_ids))
        or set(observed_ids) != set(LIVE_CASES)
        or invalid_execution
    ):
        failures = max(1, failures)
    return _report(
        status="live-failed" if failures else "live-passed",
        source_checks=source_checks,
        cases=cases,
        skipped=0,
        failures=failures,
        docker=docker,
    )


def run_live_examples_self_tests(paths: RunnerPaths) -> None:
    payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(False, "self-test docker unavailable"),
        compile_sources=False,
    )
    if payload["status"] != "structured-skip":
        raise SystemExit("live examples self-test expected structured-skip without Docker")
    _assert_case_ids(payload)

    success_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker available"),
        live_runner=lambda _binaries: [_synthetic_success(case_id) for case_id in LIVE_CASES],
        compile_sources=False,
    )
    if success_payload["status"] != "live-passed":
        raise SystemExit("live examples self-test expected live-passed with compiled evidence")

    missing_binary_evidence = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker available"),
        live_runner=lambda _binaries: [
            {
                **_synthetic_success(case_id),
                "binary_executed": False if case_id == "kafka" else True,
            }
            for case_id in LIVE_CASES
        ],
        compile_sources=False,
    )
    if missing_binary_evidence["status"] != "live-failed":
        raise SystemExit("live examples self-test accepted unexecuted binary evidence")

    live_failure_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker available"),
        live_runner=lambda _binaries: [
            {
                **_synthetic_success(case_id),
                "status": "live-failed" if case_id == "kafka" else "live-passed",
            }
            for case_id in LIVE_CASES
        ],
        compile_sources=False,
    )
    if live_failure_payload["status"] != "live-failed":
        raise SystemExit("live examples self-test expected live-failed from fake live runner")
    if live_failure_payload["summary"]["total_failures"] != 1:
        raise SystemExit("live examples self-test expected one live failure")

    source_failure_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test Docker should not be probed"),
        source_builder=lambda _paths: (
            [
                {
                    "id": "redis",
                    "status": "fail",
                    "sifr_source": LIVE_CASES["redis"].relative_source,
                    "reason": "synthetic binary build failure",
                }
            ],
            {},
        ),
    )
    if source_failure_payload["container_runtime"]["docker_available"] is not None:
        raise SystemExit("live examples self-test expected unprobed Docker on build failure")
    if source_failure_payload["cases"]:
        raise SystemExit("live examples self-test expected no service cases on build failure")

    try:
        build_live_examples_report(
            paths,
            docker_probe=lambda: DockerAvailability(None, "synthetic unprobed callback"),
            compile_sources=False,
        )
    except SystemExit as error:
        if "unprobed result" not in str(error):
            raise
    else:
        raise SystemExit("live examples self-test expected unprobed Docker callback to fail")


def _synthetic_success(case_id: str) -> dict[str, Any]:
    return {
        "id": case_id,
        "status": "live-passed",
        "sifr_source": LIVE_CASES[case_id].relative_source,
        "execution_model": "compiled-sifr-binary",
        "binary_built": True,
        "binary_executed": True,
        "stdout_marker": LIVE_CASES[case_id].stdout_marker,
        "stdout_marker_observed": True,
        "elapsed_ms": 0,
    }


def _assert_case_ids(payload: dict[str, Any]) -> None:
    case_ids = {case["id"] for case in payload["cases"]}
    if case_ids != set(LIVE_CASES):
        raise SystemExit(f"live examples self-test case drift: {sorted(case_ids)}")
    source_ids = {check["id"] for check in payload["source_checks"]}
    if source_ids != set(LIVE_CASES):
        raise SystemExit(f"live examples self-test source drift: {sorted(source_ids)}")


def probe_docker() -> DockerAvailability:
    try:
        import docker
        from docker.errors import DockerException
    except ImportError as error:
        return DockerAvailability(False, f"docker Python package unavailable: {error}")
    try:
        client = docker.from_env(timeout=5)
        try:
            client.ping()
        finally:
            client.close()
    except DockerException as error:
        return DockerAvailability(False, f"Docker daemon unavailable: {error}")
    except OSError as error:
        return DockerAvailability(False, f"Docker daemon unavailable: {error}")
    return DockerAvailability(True, "Docker daemon reachable")


def _report(
    *,
    status: str,
    source_checks: list[dict[str, Any]],
    cases: list[dict[str, Any]],
    skipped: int,
    failures: int,
    docker: DockerAvailability,
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "area": "python_interop",
        "suite": "live-examples",
        "status": status,
        "execution_model": "compiled-sifr-binary",
        "service_client_owner": "compiled-sifr-bridge",
        "result_statuses": ["live-passed", "structured-skip", "live-failed"],
        "container_runtime": {
            "provider": "testcontainers",
            "responsibility": "container lifecycle and endpoint discovery only",
            "docker_available": docker.available,
            "reason": docker.reason,
        },
        "images": LIVE_IMAGES,
        "source_checks": source_checks,
        "cases": cases,
        "summary": {
            "total_variants": len(source_checks) + len(cases),
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
            "skipped": skipped,
            "compiled_binaries": sum(
                1 for check in source_checks if check.get("binary_built") is True
            ),
            "executed_binaries": sum(
                1 for case in cases if case.get("binary_executed") is True
            ),
        },
    }

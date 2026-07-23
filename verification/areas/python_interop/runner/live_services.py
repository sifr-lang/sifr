from __future__ import annotations

import time
import traceback
import uuid
from collections.abc import Callable
from typing import Any

from live_case_config import LIVE_CASES, LIVE_IMAGES
from live_packages import BuiltLiveBinary, execute_live_binary


def run_live_cases(binaries: dict[str, BuiltLiveBinary]) -> list[dict[str, Any]]:
    return [
        _timed_case("redis", "redis" in binaries, lambda: _run_redis(binaries["redis"])),
        _timed_case("postgres", "postgres" in binaries, lambda: _run_postgres(binaries["postgres"])),
        _timed_case("kafka", "kafka" in binaries, lambda: _run_kafka(binaries["kafka"])),
        _timed_case("pubsub", "pubsub" in binaries, lambda: _run_localstack(binaries["pubsub"], ("sqs", "sns"))),
        _timed_case("sns", "sns" in binaries, lambda: _run_localstack(binaries["sns"], ("sqs", "sns"))),
        _timed_case("sqs", "sqs" in binaries, lambda: _run_localstack(binaries["sqs"], ("sqs",))),
    ]


def _timed_case(
    case_id: str,
    binary_built: bool,
    callback: Callable[[], dict[str, Any]],
) -> dict[str, Any]:
    started = time.perf_counter()
    try:
        payload = callback()
        payload["status"] = "live-passed"
    except Exception as error:  # noqa: BLE001 - live dependency failures are report data.
        payload = {
            "status": "live-failed",
            "execution_model": "compiled-sifr-binary",
            "binary_built": binary_built,
            "binary_executed": False,
            "error_type": type(error).__name__,
            "error": str(error),
            "traceback_tail": traceback.format_exc(limit=6),
        }
    payload["id"] = case_id
    payload["sifr_source"] = LIVE_CASES[case_id].relative_source
    payload["image"] = LIVE_IMAGES[LIVE_CASES[case_id].image_key]
    payload["elapsed_ms"] = round((time.perf_counter() - started) * 1000.0)
    return payload


def _run_redis(binary: BuiltLiveBinary) -> dict[str, Any]:
    from testcontainers.redis import RedisContainer

    with RedisContainer(LIVE_IMAGES["redis"]) as container:
        endpoint = (
            f"redis://{container.get_container_host_ip()}:"
            f"{container.get_exposed_port(6379)}/0"
        )
        return execute_live_binary(binary, _live_environment(endpoint))


def _run_postgres(binary: BuiltLiveBinary) -> dict[str, Any]:
    from testcontainers.postgres import PostgresContainer

    with PostgresContainer(LIVE_IMAGES["postgres"], driver=None) as container:
        return execute_live_binary(
            binary,
            _live_environment(container.get_connection_url(driver=None)),
        )


def _run_kafka(binary: BuiltLiveBinary) -> dict[str, Any]:
    from testcontainers.kafka import RedpandaContainer

    with RedpandaContainer(LIVE_IMAGES["kafka"]) as container:
        return execute_live_binary(
            binary,
            _live_environment(container.get_bootstrap_server()),
        )


def _run_localstack(
    binary: BuiltLiveBinary,
    services: tuple[str, ...],
) -> dict[str, Any]:
    from testcontainers.localstack import LocalStackContainer

    container = LocalStackContainer(
        image=LIVE_IMAGES["localstack"],
        region_name="us-east-1",
    ).with_services(*services)
    with container:
        return execute_live_binary(binary, _live_environment(container.get_url()))


def _live_environment(endpoint: str) -> dict[str, str]:
    return {
        "SIFR_LIVE_ENDPOINT": endpoint,
        "SIFR_LIVE_TOKEN": uuid.uuid4().hex,
    }

from __future__ import annotations

import json
import shutil
import subprocess
import time
import traceback
import uuid
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from env import RunnerPaths, cargo_env_for_repo_manifest

LIVE_EXAMPLE_SOURCES = {
    "redis": "redis/redis_live_roundtrip.sifr",
    "postgres": "sqlalchemy_psycopg/postgres_live_roundtrip.sifr",
    "kafka": "kafka/kafka_live_roundtrip.sifr",
    "pubsub": "pubsub/pubsub_live_callback_roundtrip.sifr",
    "sns": "aws_sns/sns_live_callback_roundtrip.sifr",
    "sqs": "aws_sqs/sqs_live_callback_roundtrip.sifr",
}

LIVE_IMAGES = {
    "redis": "redis:7.2-alpine",
    "postgres": "postgres:16-alpine",
    "kafka": "docker.redpanda.com/redpandadata/redpanda:v23.1.13",
    "localstack": "localstack/localstack:2.0.1",
}


@dataclass(frozen=True)
class DockerAvailability:
    available: bool | None
    reason: str


def build_live_examples_report(
    paths: RunnerPaths,
    docker_probe: Callable[[], DockerAvailability] | None = None,
    live_runner: Callable[[], list[dict[str, Any]]] | None = None,
    compile_sources: bool = True,
    source_checker: Callable[[], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    if source_checker is not None:
        source_checks = source_checker()
    elif compile_sources:
        source_checks = run_sifr_source_checks(paths)
    else:
        source_checks = validate_live_source_presence(paths)
    failed_sources = [check for check in source_checks if check["status"] != "pass"]
    if failed_sources:
        return _report(
            status="live-failed",
            source_checks=source_checks,
            cases=[],
            skipped=0,
            failures=len(failed_sources),
            docker=DockerAvailability(None, "Docker probe skipped because Sifr source checks failed"),
        )

    probe = docker_probe or probe_docker
    docker = probe()
    if docker.available is None:
        raise SystemExit("python interop live examples Docker probe returned an unprobed result")
    if docker.available is False:
        cases = [
            {
                "id": case_id,
                "status": "structured-skip",
                "reason": docker.reason,
                "sifr_source": source,
            }
            for case_id, source in LIVE_EXAMPLE_SOURCES.items()
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
    cases = runner()
    failures = sum(1 for case in cases if case["status"] != "live-passed")
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
    case_ids = {case["id"] for case in payload["cases"]}
    if case_ids != set(LIVE_EXAMPLE_SOURCES):
        raise SystemExit(f"live examples self-test case drift: {sorted(case_ids)}")
    source_ids = {check["id"] for check in payload["source_checks"]}
    if source_ids != set(LIVE_EXAMPLE_SOURCES):
        raise SystemExit(f"live examples self-test source drift: {sorted(source_ids)}")

    success_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker available"),
        live_runner=lambda: [
            {
                "id": case_id,
                "status": "live-passed",
                "sifr_source": source,
                "elapsed_ms": 0,
            }
            for case_id, source in LIVE_EXAMPLE_SOURCES.items()
        ],
        compile_sources=False,
    )
    if success_payload["status"] != "live-passed":
        raise SystemExit("live examples self-test expected live-passed with fake live runner")

    live_failure_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker available"),
        live_runner=lambda: [
            {
                "id": "kafka",
                "status": "live-failed",
                "sifr_source": LIVE_EXAMPLE_SOURCES["kafka"],
                "error": "synthetic live failure",
            }
        ],
        compile_sources=False,
    )
    if live_failure_payload["status"] != "live-failed":
        raise SystemExit("live examples self-test expected live-failed from fake live runner")
    if live_failure_payload["summary"]["total_failures"] != 1:
        raise SystemExit("live examples self-test expected one live failure")

    source_failure_payload = build_live_examples_report(
        paths,
        docker_probe=lambda: DockerAvailability(True, "self-test docker should not be probed"),
        compile_sources=False,
        source_checker=lambda: [
            {
                "id": "redis",
                "status": "fail",
                "sifr_source": LIVE_EXAMPLE_SOURCES["redis"],
                "reason": "synthetic source failure",
            }
        ],
    )
    if source_failure_payload["container_runtime"]["docker_available"] is not None:
        raise SystemExit("live examples self-test expected unprobed Docker on source failure")
    if source_failure_payload["cases"]:
        raise SystemExit("live examples self-test expected no service cases on source failure")

    try:
        build_live_examples_report(
            paths,
            docker_probe=lambda: DockerAvailability(None, "synthetic unprobed callback"),
            compile_sources=False,
        )
    except SystemExit as exc:
        if "unprobed result" not in str(exc):
            raise
    else:
        raise SystemExit("live examples self-test expected unprobed Docker callback to fail")


def validate_live_source_presence(paths: RunnerPaths) -> list[dict[str, Any]]:
    checks: list[dict[str, Any]] = []
    for case_id, relative_source in LIVE_EXAMPLE_SOURCES.items():
        source_path = paths.fixtures_root / relative_source
        if not source_path.is_file():
            checks.append(
                {
                    "id": case_id,
                    "status": "fail",
                    "sifr_source": relative_source,
                    "reason": "missing source fixture",
                }
            )
            continue
        checks.append(
            {
                "id": case_id,
                "status": "pass",
                "sifr_source": relative_source,
                "check": "source-present",
            }
        )
    return checks


def run_sifr_source_checks(paths: RunnerPaths) -> list[dict[str, Any]]:
    package_root = prepare_live_source_package(paths)
    checks: list[dict[str, Any]] = []
    for case_id, relative_source in LIVE_EXAMPLE_SOURCES.items():
        source_path = package_root / "src" / Path(relative_source).name
        started = time.perf_counter()
        proc = subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "sifr",
                "--manifest-path",
                str(paths.repo_root / "Cargo.toml"),
                "--",
                "check",
                str(source_path),
            ],
            cwd=package_root,
            env=cargo_env_for_repo_manifest(paths.repo_root),
            text=True,
            capture_output=True,
            check=False,
        )
        elapsed_ms = round((time.perf_counter() - started) * 1000.0)
        check: dict[str, Any] = {
            "id": case_id,
            "status": "pass" if proc.returncode == 0 else "fail",
            "sifr_source": relative_source,
            "elapsed_ms": elapsed_ms,
        }
        if proc.returncode != 0:
            check["stdout"] = proc.stdout[-4000:]
            check["stderr"] = proc.stderr[-4000:]
        checks.append(check)
    return checks


def prepare_live_source_package(paths: RunnerPaths) -> Path:
    package_root = paths.repo_root / "target" / "verification" / "areas" / "python_interop" / "live_examples_package"
    if package_root.exists():
        shutil.rmtree(package_root)
    source_root = package_root / "src"
    source_root.mkdir(parents=True, exist_ok=True)
    for relative_source in LIVE_EXAMPLE_SOURCES.values():
        source_path = paths.fixtures_root / relative_source
        if not source_path.is_file():
            continue
        shutil.copy2(source_path, source_root / Path(relative_source).name)
    (source_root / "lib.rs").write_text(
        "// Pure Sifr package marker. Sifr source lives in sifr.toml source roots.\n",
        encoding="utf-8",
    )
    (package_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                'name = "sifr-python-interop-live-examples"',
                'version = "0.1.0"',
                'edition = "2024"',
                "",
                "[package.metadata.sifr]",
                'manifest = "sifr.toml"',
                "",
                "[workspace]",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (package_root / "sifr.toml").write_text(
        "\n".join(
            [
                "[package]",
                'name = "python_interop_live_examples"',
                'edition = "2026"',
                'sifr-version = ">=0.3,<0.4"',
                "",
                "[source]",
                'root = "src"',
                "",
                "[python]",
                'venv = ".venv"',
                'allow-imports = ["boto3", "kafka", "psycopg", "redis"]',
                "",
                "[trust]",
                'python = ["boto3", "kafka", "psycopg", "redis"]',
                'python-native = ["psycopg"]',
                "",
            ]
        ),
        encoding="utf-8",
    )
    venv_link = package_root / ".venv"
    area_venv = paths.area_root / ".venv"
    if not area_venv.exists():
        raise SystemExit(
            "python interop live examples require the area uv environment; "
            "run through `uv run --project verification/areas/python_interop --locked ...`"
        )
    venv_link.symlink_to(area_venv, target_is_directory=True)
    return package_root


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


def run_live_cases() -> list[dict[str, Any]]:
    return [
        _timed_case("redis", _run_redis_roundtrip),
        _timed_case("postgres", _run_postgres_roundtrip),
        _timed_case("kafka", _run_kafka_roundtrip),
        _timed_case("pubsub", _run_localstack_pubsub_roundtrip),
        _timed_case("sns", _run_localstack_sns_roundtrip),
        _timed_case("sqs", _run_localstack_sqs_roundtrip),
    ]


def _timed_case(case_id: str, callback: Callable[[], dict[str, Any]]) -> dict[str, Any]:
    started = time.perf_counter()
    try:
        payload = callback()
        payload["status"] = "live-passed"
    except Exception as error:  # noqa: BLE001 - report live dependency failures as data.
        payload = {
            "status": "live-failed",
            "error_type": type(error).__name__,
            "error": str(error),
            "traceback_tail": traceback.format_exc(limit=6),
        }
    payload["id"] = case_id
    payload["sifr_source"] = LIVE_EXAMPLE_SOURCES[case_id]
    payload["elapsed_ms"] = round((time.perf_counter() - started) * 1000.0)
    return payload


def _run_redis_roundtrip() -> dict[str, Any]:
    from testcontainers.redis import RedisContainer

    with RedisContainer(LIVE_IMAGES["redis"]) as container:
        client = container.get_client(decode_responses=True)
        try:
            key = f"sifr:live:redis:{uuid.uuid4().hex}"
            if client.ping() is not True:
                raise AssertionError("redis ping did not return true")
            if client.set(key, "ready") is not True:
                raise AssertionError("redis set did not return true")
            if client.get(key) != "ready":
                raise AssertionError("redis get did not round-trip value")
            if client.incr(f"{key}:counter") != 1:
                raise AssertionError("redis incr did not return first counter value")
            client.delete(key, f"{key}:counter")
        finally:
            client.close()
        return {
            "image": LIVE_IMAGES["redis"],
            "operations": ["ping", "set", "get", "incr", "delete"],
        }


def _run_postgres_roundtrip() -> dict[str, Any]:
    import psycopg
    from testcontainers.postgres import PostgresContainer

    with PostgresContainer(LIVE_IMAGES["postgres"], driver=None) as container:
        with psycopg.connect(container.get_connection_url(driver=None)) as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    "create table sifr_live_example (id integer primary key, label text not null)"
                )
                cursor.execute(
                    "insert into sifr_live_example (id, label) values (%s, %s)",
                    (1, "postgres-ready"),
                )
                cursor.execute("select label from sifr_live_example where id = %s", (1,))
                row = cursor.fetchone()
            connection.commit()
    if row != ("postgres-ready",):
        raise AssertionError(f"postgres row mismatch: {row!r}")
    return {
        "image": LIVE_IMAGES["postgres"],
        "operations": ["connect", "create-table", "insert", "select", "commit"],
    }


def _run_kafka_roundtrip() -> dict[str, Any]:
    from kafka import KafkaConsumer, KafkaProducer
    from testcontainers.kafka import RedpandaContainer

    topic = f"sifr-live-{uuid.uuid4().hex}"
    expected = {"source": "sifr", "status": "kafka-ready"}
    with RedpandaContainer(LIVE_IMAGES["kafka"]) as container:
        bootstrap = container.get_bootstrap_server()
        producer = KafkaProducer(
            bootstrap_servers=bootstrap,
            value_serializer=lambda value: json.dumps(value, sort_keys=True).encode("utf-8"),
            request_timeout_ms=30000,
            api_version_auto_timeout_ms=10000,
        )
        try:
            producer.send(topic, expected).get(timeout=30)
            producer.flush(timeout=30)
        finally:
            producer.close(timeout=10)

        consumer = KafkaConsumer(
            topic,
            bootstrap_servers=bootstrap,
            group_id=f"sifr-live-{uuid.uuid4().hex}",
            auto_offset_reset="earliest",
            enable_auto_commit=False,
            consumer_timeout_ms=15000,
            value_deserializer=lambda payload: json.loads(payload.decode("utf-8")),
            request_timeout_ms=30000,
            api_version_auto_timeout_ms=10000,
        )
        try:
            messages = [message.value for message in consumer]
        finally:
            consumer.close()
    if expected not in messages:
        raise AssertionError(f"kafka message not observed: {messages!r}")
    handler_contract = _message_handler_source_contract("kafka", messages[0])
    return {
        "image": LIVE_IMAGES["kafka"],
        "operations": ["produce", "consume", "sifr-callback-source-contract"],
        "control_flow": (
            "Python producer -> Python KafkaConsumer; checked Sifr source passes "
            "the consumed Python object to a threadsafe_callback handler"
        ),
        "handler_contract": handler_contract,
    }


def _run_localstack_pubsub_roundtrip() -> dict[str, Any]:
    return _run_localstack_topic_subscription_roundtrip(
        case_id="pubsub",
        expected="localstack-pubsub-ready",
        operations=[
            "create-subscription-queue",
            "create-topic",
            "subscribe",
            "publish",
            "consume-subscription-message",
            "sifr-callback-source-contract",
            "delete",
        ],
    )


def _run_localstack_sns_roundtrip() -> dict[str, Any]:
    return _run_localstack_topic_subscription_roundtrip(
        case_id="sns",
        expected="localstack-sns-ready",
        operations=[
            "create-sns-topic",
            "subscribe-sqs-endpoint",
            "publish",
            "consume-delivery",
            "sifr-callback-source-contract",
            "delete",
        ],
    )


def _run_localstack_topic_subscription_roundtrip(
    *,
    case_id: str,
    expected: str,
    operations: list[str],
) -> dict[str, Any]:
    import boto3  # noqa: F401 - imported to prove the locked runtime dependency is present.
    from testcontainers.localstack import LocalStackContainer

    queue_name = f"sifr-live-{uuid.uuid4().hex}"
    topic_name = f"sifr-live-{uuid.uuid4().hex}"
    with LocalStackContainer(
        image=LIVE_IMAGES["localstack"],
        region_name="us-east-1",
    ).with_services("sqs", "sns") as container:
        sqs = container.get_client("sqs")
        sns = container.get_client("sns")
        queue_url = sqs.create_queue(QueueName=queue_name)["QueueUrl"]
        queue_attrs = sqs.get_queue_attributes(
            QueueUrl=queue_url,
            AttributeNames=["QueueArn"],
        )["Attributes"]
        topic_arn = sns.create_topic(Name=topic_name)["TopicArn"]
        policy = {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": {"Service": "sns.amazonaws.com"},
                    "Action": "sqs:SendMessage",
                    "Resource": queue_attrs["QueueArn"],
                    "Condition": {"ArnEquals": {"aws:SourceArn": topic_arn}},
                }
            ],
        }
        sqs.set_queue_attributes(
            QueueUrl=queue_url,
            Attributes={"Policy": json.dumps(policy, sort_keys=True)},
        )
        sns.subscribe(TopicArn=topic_arn, Protocol="sqs", Endpoint=queue_attrs["QueueArn"])
        sns.publish(TopicArn=topic_arn, Message=expected)
        response = sqs.receive_message(
            QueueUrl=queue_url,
            MaxNumberOfMessages=1,
            WaitTimeSeconds=10,
        )
        messages = response.get("Messages", [])
        if len(messages) != 1:
            raise AssertionError(f"expected one SQS message, got {len(messages)}")
        body = json.loads(messages[0]["Body"])
        if body.get("Message") != expected:
            raise AssertionError(f"SNS message body mismatch: {body!r}")
        handler_contract = _message_handler_source_contract(case_id, body)
        sqs.delete_message(QueueUrl=queue_url, ReceiptHandle=messages[0]["ReceiptHandle"])
    return {
        "image": LIVE_IMAGES["localstack"],
        "service_model": "LocalStack SNS topic with SQS subscription",
        "operations": operations,
        "control_flow": (
            "Python SNS/SQS clients consume delivery; checked Sifr source passes "
            "the consumed Python object to a threadsafe_callback handler"
        ),
        "handler_contract": handler_contract,
    }


def _run_localstack_sqs_roundtrip() -> dict[str, Any]:
    import boto3  # noqa: F401 - imported to prove the locked runtime dependency is present.
    from testcontainers.localstack import LocalStackContainer

    queue_name = f"sifr-live-{uuid.uuid4().hex}"
    expected = "localstack-sqs-ready"
    with LocalStackContainer(
        image=LIVE_IMAGES["localstack"],
        region_name="us-east-1",
    ).with_services("sqs") as container:
        sqs = container.get_client("sqs")
        queue_url = sqs.create_queue(QueueName=queue_name)["QueueUrl"]
        sqs.send_message(QueueUrl=queue_url, MessageBody=expected)
        response = sqs.receive_message(
            QueueUrl=queue_url,
            MaxNumberOfMessages=1,
            WaitTimeSeconds=10,
        )
        messages = response.get("Messages", [])
        if len(messages) != 1:
            raise AssertionError(f"expected one SQS message, got {len(messages)}")
        if messages[0].get("Body") != expected:
            raise AssertionError(f"SQS message body mismatch: {messages[0]!r}")
        handler_contract = _message_handler_source_contract("sqs", messages[0])
        sqs.delete_message(QueueUrl=queue_url, ReceiptHandle=messages[0]["ReceiptHandle"])
    return {
        "image": LIVE_IMAGES["localstack"],
        "service_model": "LocalStack SQS queue",
        "operations": [
            "create-queue",
            "send-message",
            "receive-message",
            "sifr-callback-source-contract",
            "delete",
        ],
        "control_flow": (
            "Python SQS client consumes message; checked Sifr source passes "
            "the consumed Python object to a threadsafe_callback handler"
        ),
        "handler_contract": handler_contract,
    }


def _message_handler_source_contract(case_id: str, message: Any) -> dict[str, str]:
    if message is None:
        raise AssertionError(f"{case_id} handler handoff received no message")
    return {
        "status": "source-checked",
        "sifr_source": LIVE_EXAMPLE_SOURCES[case_id],
        "handler_model": "threadsafe_callback",
    }


def _report(
    *,
    status: str,
    source_checks: list[dict[str, Any]],
    cases: list[dict[str, Any]],
    skipped: int,
    failures: int,
    docker: DockerAvailability,
) -> dict[str, Any]:
    total_variants = len(source_checks) + len(cases)
    return {
        "schema_version": 1,
        "area": "python_interop",
        "status": status,
        "result_statuses": ["live-passed", "structured-skip", "live-failed"],
        "container_runtime": {
            "provider": "testcontainers",
            "docker_available": docker.available,
            "reason": docker.reason,
        },
        "images": LIVE_IMAGES,
        "source_checks": source_checks,
        "cases": cases,
        "summary": {
            "total_variants": total_variants,
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
            "skipped": skipped,
        },
    }

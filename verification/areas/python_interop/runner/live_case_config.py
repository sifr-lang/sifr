from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class LiveCase:
    case_id: str
    relative_source: str
    bridge_file: str
    import_roots: tuple[str, ...]
    native_roots: tuple[str, ...]
    stdout_marker: str
    image_key: str


LIVE_IMAGES = {
    "redis": "redis:7.2-alpine",
    "postgres": "postgres:16-alpine",
    "kafka": "docker.redpanda.com/redpandadata/redpanda:v23.1.13",
    "localstack": "localstack/localstack:2.0.1",
}

LIVE_CASES = {
    "redis": LiveCase(
        case_id="redis",
        relative_source="redis/redis_live_roundtrip.sifr",
        bridge_file="live_services/python_bridges/redis_live.py",
        import_roots=("redis",),
        native_roots=(),
        stdout_marker="sifr-python-interop:live:redis:roundtrip=ok:resources=zero",
        image_key="redis",
    ),
    "postgres": LiveCase(
        case_id="postgres",
        relative_source="sqlalchemy_psycopg/postgres_live_roundtrip.sifr",
        bridge_file="live_services/python_bridges/postgres_live.py",
        import_roots=("psycopg",),
        native_roots=("psycopg",),
        stdout_marker="sifr-python-interop:live:postgres:roundtrip=ok:resources=zero",
        image_key="postgres",
    ),
    "kafka": LiveCase(
        case_id="kafka",
        relative_source="kafka/kafka_live_roundtrip.sifr",
        bridge_file="live_services/python_bridges/kafka_live.py",
        import_roots=("kafka", "threading"),
        native_roots=(),
        stdout_marker="sifr-python-interop:live:kafka:callback=ack:resources=zero",
        image_key="kafka",
    ),
    "pubsub": LiveCase(
        case_id="pubsub",
        relative_source="pubsub/pubsub_live_callback_roundtrip.sifr",
        bridge_file="live_services/python_bridges/aws_live.py",
        import_roots=("boto3", "json", "threading"),
        native_roots=(),
        stdout_marker="sifr-python-interop:live:pubsub:callback=ack:resources=zero",
        image_key="localstack",
    ),
    "sns": LiveCase(
        case_id="sns",
        relative_source="aws_sns/sns_live_callback_roundtrip.sifr",
        bridge_file="live_services/python_bridges/aws_live.py",
        import_roots=("boto3", "json", "threading"),
        native_roots=(),
        stdout_marker="sifr-python-interop:live:sns:callback=ack:resources=zero",
        image_key="localstack",
    ),
    "sqs": LiveCase(
        case_id="sqs",
        relative_source="aws_sqs/sqs_live_callback_roundtrip.sifr",
        bridge_file="live_services/python_bridges/aws_live.py",
        import_roots=("boto3", "json", "threading"),
        native_roots=(),
        stdout_marker="sifr-python-interop:live:sqs:callback=ack:resources=zero",
        image_key="localstack",
    ),
}

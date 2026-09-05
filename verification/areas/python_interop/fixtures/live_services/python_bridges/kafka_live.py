import threading

import kafka
from kafka import KafkaConsumer, KafkaProducer

EXPECTED_KAFKA_VERSION = "3.0.11"


def _invoke_foreign(handler, value: str) -> str:
    results = []
    errors = []

    def invoke() -> None:
        try:
            results.append(handler(value))
        except BaseException as error:  # noqa: BLE001 - rethrow on the declaration thread.
            errors.append(error)

    worker = threading.Thread(target=invoke, name="sifr-live-kafka-callback")
    worker.start()
    worker.join(timeout=30)
    if worker.is_alive():
        raise RuntimeError("Kafka Sifr callback did not finish")
    if errors:
        raise errors[0]
    if len(results) != 1:
        raise RuntimeError("Kafka Sifr callback produced no result")
    return results[0]


def run(handler, endpoint: str, token: str) -> str:
    if kafka.__version__ != EXPECTED_KAFKA_VERSION:
        raise RuntimeError(
            f"Expected kafka-python {EXPECTED_KAFKA_VERSION}, got {kafka.__version__}"
        )
    topic = f"sifr-live-{token}"
    payload = token.encode("utf-8")
    producer = KafkaProducer(
        bootstrap_servers=endpoint,
        request_timeout_ms=30_000,
    )
    try:
        producer.send(topic, payload).get(timeout=30)
        producer.flush(timeout=30)
    finally:
        producer.close(timeout=10)

    consumer = KafkaConsumer(
        topic,
        bootstrap_servers=endpoint,
        group_id=f"sifr-live-{token}",
        auto_offset_reset="earliest",
        enable_auto_commit=False,
        consumer_timeout_ms=30_000,
        request_timeout_ms=30_000,
    )
    observed = None
    try:
        for message in consumer:
            observed = message.value.decode("utf-8")
            break
    finally:
        consumer.close()
    if observed != token:
        raise RuntimeError(f"Kafka message mismatch: {observed!r}")
    if _invoke_foreign(handler, observed) != f"ack:{token}":
        raise RuntimeError("Kafka Sifr callback acknowledgement mismatch")
    return (
        f"sifr-python-interop:live:kafka:version={kafka.__version__}:"
        "callback=ack:resources=zero"
    )

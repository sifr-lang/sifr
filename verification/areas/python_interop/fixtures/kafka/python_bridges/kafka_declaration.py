import threading

import kafka
from kafka import TopicPartition
from kafka.protocol.admin.cluster import DescribeClusterRequest

EXPECTED_KAFKA_VERSION = "3.0.11"


def _generated_schema_marker() -> str:
    if kafka.__version__ != EXPECTED_KAFKA_VERSION:
        raise RuntimeError(
            f"Expected kafka-python {EXPECTED_KAFKA_VERSION}, got {kafka.__version__}"
        )
    request = DescribeClusterRequest(
        version=2,
        include_cluster_authorized_operations=True,
        endpoint_type=1,
        include_fenced_brokers=False,
    )
    encoded = request.encode()
    decoded = DescribeClusterRequest.decode(encoded, version=2)
    fields = decoded.to_dict()
    expected_fields = {
        "include_cluster_authorized_operations": True,
        "endpoint_type": 1,
        "include_fenced_brokers": False,
    }
    if fields != expected_fields:
        raise RuntimeError(f"Kafka generated schema round trip changed: {fields!r}")
    return (
        f"version={kafka.__version__}:schema={DescribeClusterRequest.name}"
        f"-v{decoded.version}:fields={len(fields)}"
    )


def poll(handler, partition):
    topic_partition = TopicPartition("sifr-events", partition)
    results = []
    errors = []

    def invoke() -> None:
        try:
            results.append(handler(topic_partition.partition))
        except BaseException as error:  # noqa: BLE001 - rethrow on the declaring thread.
            errors.append(error)

    worker = threading.Thread(
        target=invoke,
        name="sifr-kafka-callback",
    )
    worker.start()
    worker.join(timeout=30)
    if worker.is_alive():
        raise RuntimeError("Kafka Sifr callback did not finish")
    if errors:
        raise errors[0]
    if len(results) != 1:
        raise RuntimeError("Kafka Sifr callback produced no result")
    return f"{_generated_schema_marker()}:ack={results[0]}"

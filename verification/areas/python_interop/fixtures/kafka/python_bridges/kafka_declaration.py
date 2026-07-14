import threading

from kafka import TopicPartition


def poll(handler, partition):
    topic_partition = TopicPartition("sifr-events", partition)
    results = []
    worker = threading.Thread(
        target=lambda: results.append(handler(topic_partition.partition)),
        name="sifr-kafka-callback",
    )
    worker.start()
    worker.join()
    return results[0]

import boto3
from botocore.stub import Stubber


QUEUE_URL = "https://sqs.us-east-1.amazonaws.com/123456789012/sifr-queue"


def run() -> str:
    client = boto3.client(
        "sqs",
        region_name="us-east-1",
        aws_access_key_id="test",
        aws_secret_access_key="test",
    )
    with Stubber(client) as stubber:
        stubber.add_response(
            "create_queue",
            {"QueueUrl": QUEUE_URL},
            {"QueueName": "sifr-queue"},
        )
        queue_url = client.create_queue(QueueName="sifr-queue")["QueueUrl"]
    if queue_url != QUEUE_URL:
        raise RuntimeError("boto3/botocore SQS stub returned an unexpected URL")
    return f"sifr-python-interop:boto3-botocore:queue={queue_url}"

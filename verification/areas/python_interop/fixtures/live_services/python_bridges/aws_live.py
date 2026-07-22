import json
import threading

import boto3


def _client(service: str, endpoint: str):
    return boto3.client(
        service,
        endpoint_url=endpoint,
        region_name="us-east-1",
        aws_access_key_id="test",
        aws_secret_access_key="test",
    )


def _invoke_foreign(handler, value: str) -> str:
    results = []
    errors = []

    def invoke() -> None:
        try:
            results.append(handler(value))
        except BaseException as error:  # noqa: BLE001 - rethrow on the declaration thread.
            errors.append(error)

    worker = threading.Thread(target=invoke, name="sifr-live-aws-callback")
    worker.start()
    worker.join(timeout=30)
    if worker.is_alive():
        raise RuntimeError("AWS Sifr callback did not finish")
    if errors:
        raise errors[0]
    if len(results) != 1:
        raise RuntimeError("AWS Sifr callback produced no result")
    return results[0]


def _receive_one(sqs, queue_url: str) -> dict:
    response = sqs.receive_message(
        QueueUrl=queue_url,
        MaxNumberOfMessages=1,
        WaitTimeSeconds=10,
    )
    messages = response.get("Messages", [])
    if len(messages) != 1:
        raise RuntimeError(f"Expected one SQS message, got {len(messages)}")
    return messages[0]


def _run_cleanup(actions) -> list[BaseException]:
    errors = []
    for action in actions:
        try:
            action()
        except BaseException as error:  # noqa: BLE001 - finish every cleanup action.
            errors.append(error)
    return errors


def run_sqs(handler, endpoint: str, token: str) -> str:
    sqs = _client("sqs", endpoint)
    queue_url = None
    failure = None
    try:
        queue_url = sqs.create_queue(QueueName=f"sifr-live-{token}")["QueueUrl"]
        sqs.send_message(QueueUrl=queue_url, MessageBody=token)
        message = _receive_one(sqs, queue_url)
        if message.get("Body") != token:
            raise RuntimeError(f"SQS message mismatch: {message!r}")
        if _invoke_foreign(handler, message["Body"]) != f"ack:{token}":
            raise RuntimeError("SQS Sifr callback acknowledgement mismatch")
        sqs.delete_message(QueueUrl=queue_url, ReceiptHandle=message["ReceiptHandle"])
    except BaseException as error:
        failure = error
        raise
    finally:
        cleanup_errors = _run_cleanup(
            [
                lambda: sqs.delete_queue(QueueUrl=queue_url) if queue_url is not None else None,
                sqs.close,
            ]
        )
        if cleanup_errors and failure is None:
            raise cleanup_errors[0]
    return "sifr-python-interop:live:sqs:callback=ack:resources=zero"


def run_sns_to_sqs(handler, endpoint: str, token: str, case_id: str) -> str:
    sqs = _client("sqs", endpoint)
    sns = _client("sns", endpoint)
    queue_url = None
    topic_arn = None
    subscription_arn = None
    failure = None
    try:
        queue_url = sqs.create_queue(QueueName=f"sifr-live-{case_id}-{token}")["QueueUrl"]
        attributes = sqs.get_queue_attributes(
            QueueUrl=queue_url,
            AttributeNames=["QueueArn"],
        )["Attributes"]
        queue_arn = attributes["QueueArn"]
        topic_arn = sns.create_topic(Name=f"sifr-live-{case_id}-{token}")["TopicArn"]
        policy = {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": {"Service": "sns.amazonaws.com"},
                    "Action": "sqs:SendMessage",
                    "Resource": queue_arn,
                    "Condition": {"ArnEquals": {"aws:SourceArn": topic_arn}},
                }
            ],
        }
        sqs.set_queue_attributes(
            QueueUrl=queue_url,
            Attributes={"Policy": json.dumps(policy, sort_keys=True)},
        )
        subscription_arn = sns.subscribe(
            TopicArn=topic_arn,
            Protocol="sqs",
            Endpoint=queue_arn,
        )["SubscriptionArn"]
        sns.publish(TopicArn=topic_arn, Message=token)
        message = _receive_one(sqs, queue_url)
        body = json.loads(message["Body"])
        if body.get("Message") != token:
            raise RuntimeError(f"SNS-to-SQS message mismatch: {body!r}")
        if _invoke_foreign(handler, body["Message"]) != f"ack:{token}":
            raise RuntimeError("SNS-to-SQS Sifr callback acknowledgement mismatch")
        sqs.delete_message(QueueUrl=queue_url, ReceiptHandle=message["ReceiptHandle"])
    except BaseException as error:
        failure = error
        raise
    finally:
        cleanup_errors = _run_cleanup(
            [
                lambda: (
                    sns.unsubscribe(SubscriptionArn=subscription_arn)
                    if subscription_arn is not None
                    else None
                ),
                lambda: sns.delete_topic(TopicArn=topic_arn) if topic_arn is not None else None,
                lambda: sqs.delete_queue(QueueUrl=queue_url) if queue_url is not None else None,
                sns.close,
                sqs.close,
            ]
        )
        if cleanup_errors and failure is None:
            raise cleanup_errors[0]
    return f"sifr-python-interop:live:{case_id}:callback=ack:resources=zero"

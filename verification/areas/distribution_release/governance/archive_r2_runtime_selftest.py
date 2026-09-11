"""Real SDK wire contracts, with all sockets denied and local HTTP doubles."""

from __future__ import annotations

import io
import json
import os
from pathlib import Path
import socket
import ssl
import sys
import tomllib
import unittest
from dataclasses import replace
from unittest.mock import patch

import boto3
import botocore
from botocore.awsrequest import AWSResponse
from botocore.credentials import CredentialResolver
from botocore.exceptions import EndpointConnectionError
from botocore.httpsession import URLLib3Session
from botocore.stub import Stubber

from .archive_manifest import construct_manifest
from .archive_offline_selftest import ATTEMPT, RUN, SOURCE, synthetic_bundle
from .archive_r2_config import R2Target
from .archive_r2_control import ControlCredentials, MAX_CONTROL_BYTES, observe_control
from .archive_r2_runtime import R2Credentials, create_r2_client, observe_lifecycle
from .archive_r2_store import R2ArchiveReader, R2ArchiveStore
from .common import GovernanceError, canonical_json_bytes, sha256_bytes

SDK_VERSION = "1.43.89"
SDK_HASHES = {
    "boto3": {"sha256:fe4190afe63eb562b6ba6a3911cf4427473b35fa047adde093bf696d3ae09fc0",
              "sha256:c28abbe472e9b7cad08807356311aeec51bde5218c18489da827045d2267bfd9"},
    "botocore": {"sha256:d7211220c815427fe71225acc6909e4ab5dfab3b03770e72fd16cf9eb86b3d1a",
                 "sha256:f0574942970742657b0e0716cf08c2dfe6bef8e6de5fbb7081c3424e262b4cca"},
}


class Raw(io.BytesIO):
    def stream(self, amt=1024, decode_content=False):
        while chunk := self.read(amt):
            yield chunk


class ControlResponse(Raw):
    def __init__(self, raw, status=200, headers=None):
        super().__init__(raw)
        self.status = status
        self.headers = headers if headers is not None else [
            ("Content-Length", str(len(raw))), ("CF-Ray", "synthetic-request-ARN")]

    def getheaders(self):
        return self.headers


class ControlConnection:
    def __init__(self, response):
        self.response = response
        self.calls = []
        self.closed = False

    def request(self, method, path, *, headers):
        self.calls.append((method, path, headers))

    def getresponse(self):
        return self.response

    def close(self):
        self.closed = True


class R2RuntimeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        inventory, _ = synthetic_bundle()
        raw = canonical_json_bytes(construct_manifest(inventory))
        cls.target = R2Target("a" * 32, "default", "synthetic-primary", "synthetic-producer",
                              raw, sha256_bytes(raw), SOURCE, RUN, ATTEMPT)
        cls.key = cls.target.root + "objects/sha256/" + "b" * 64

    def setUp(self):
        self.addCleanup(patch.stopall)
        self.socket = patch.object(socket.socket, "connect", side_effect=AssertionError("real socket forbidden")).start()
        patch.object(socket, "create_connection", side_effect=AssertionError("real connection forbidden")).start()
        self.send = patch.object(URLLib3Session, "send", side_effect=AssertionError("unregistered SDK send")).start()
        self.credentials = R2Credentials(self.target.credential_ref, "TESTACCESSKEY", "secret-value", "session-secret")

    def client(self, target=None, purpose="producer"):
        target = target or self.target
        client = create_r2_client(target, replace(self.credentials, reference=target.credential_ref), purpose=purpose)
        self.addCleanup(client.close)
        return client

    def wire(self, *, status=200, content=b"", headers=None):
        raw = Raw(content)
        headers = headers if headers is not None else {"etag": '"opaque"'}
        self.send.side_effect = lambda request: AWSResponse(request.url, status, headers, raw)
        return raw

    def control(self, raw=None, *, target=None, operation="metadata", status=200, headers=None):
        target = target or self.target
        if raw is None:
            result = ({"rules": []} if operation == "locks" else
                      {"name": target.bucket, "jurisdiction": target.jurisdiction,
                       "storageClass": "Standard", "creationDate": "2026-09-08T00:00:00Z"})
            raw = canonical_json_bytes({"success": True, "errors": [], "messages": [], "result": result})
        response = ControlResponse(raw, status, headers)
        connection = ControlConnection(response)
        return response, connection, raw

    def observe(self, connection, *, target=None, operation="metadata"):
        with patch("http.client.HTTPSConnection", return_value=connection) as factory:
            result = observe_control(target or self.target, ControlCredentials("config-reader", "bearer-secret"),
                                     operation=operation)
        return result, factory

    def test_exact_runtime_versions_and_lock_hashes(self):
        self.assertEqual(sys.version_info[:3], (3, 14, 7))
        self.assertEqual(boto3.__version__, SDK_VERSION)
        self.assertEqual(botocore.__version__, SDK_VERSION)
        runtime = Path(__file__).resolve().parents[1] / "runtime"
        lock = tomllib.loads((runtime / "uv.lock").read_text())
        self.assertEqual(lock["requires-python"], "==3.14.7")
        for name, hashes in SDK_HASHES.items():
            record = next(p for p in lock["package"] if p["name"] == name)
            self.assertEqual(record["version"], SDK_VERSION)
            self.assertEqual(record["source"], {"registry": "https://pypi.org/simple"})
            self.assertEqual({record["sdist"]["hash"], *(w["hash"] for w in record["wheels"])}, hashes)
            if metadata_root := os.environ.get("SIFR_R2_SDK_METADATA_DIR"):
                metadata = json.loads((Path(metadata_root) / f"{name}.json").read_text())
                self.assertEqual(metadata["info"]["version"], SDK_VERSION)
                self.assertEqual({"sha256:" + f["digests"]["sha256"] for f in metadata["urls"]}, hashes)

    def test_conditional_put_serialization_and_target_binding(self):
        for jurisdiction in ("default", "eu", "us", "fedramp"):
            target = replace(self.target, jurisdiction=jurisdiction)
            client = self.client(target)
            self.wire()
            self.assertTrue(R2ArchiveStore(target, client).create(self.key, b"new"))
            request = self.send.call_args.args[0]
            self.assertEqual(request.url, f"{target.endpoint}/{target.bucket}/{self.key}")
            self.assertEqual(request.method, "PUT")
            self.assertEqual(request.headers["If-None-Match"], b"*")
            self.assertEqual(request.headers["Content-Length"], b"3")
            self.assertEqual(request.headers["x-amz-storage-class"], b"STANDARD")
            self.assertIn(b"/auto/s3/aws4_request", request.headers["Authorization"])
            self.assertIn(b"AWS4-HMAC-SHA256", request.headers["Authorization"])
            self.assertIs(client._endpoint.http_session._verify, True)
            self.assertEqual(client.meta.config.proxies, {})
            self.assertFalse(client.meta.config.endpoint_discovery_enabled)
        self.send.reset_mock()
        client = self.client()
        for params in ({"Bucket": "wrong-bucket", "Key": self.key},
                       {"Bucket": self.target.bucket, "Key": self.key, "Range": "bytes=0-1"}):
            with self.assertRaises(GovernanceError):
                client.get_object(**params)
        with self.assertRaises(GovernanceError):
            client.delete_object(Bucket=self.target.bucket, Key=self.key)
        self.send.assert_not_called()
        client.meta.events.register("before-sign.s3.PutObject", lambda request, **_: setattr(
            request, "url", "https://untrusted.invalid/diverted"))
        with self.assertRaises(GovernanceError):
            R2ArchiveStore(self.target, client).create(self.key, b"new")
        self.send.assert_not_called()

    def test_one_total_attempt_and_redirect_refusal(self):
        client = self.client()
        store = R2ArchiveStore(self.target, client)
        for status, code in ((301, "PermanentRedirect"), (307, "TemporaryRedirect"),
                             (400, "AuthorizationHeaderMalformed"), (403, "AccessDenied"),
                             (409, "ConditionalRequestConflict"), (429, "SlowDown"),
                             (500, "InternalError"), (503, "SlowDown"), (412, "Other")):
            self.send.reset_mock()
            self.wire(status=status, content=f"<Error><Code>{code}</Code><Message>secret-value</Message></Error>".encode(),
                      headers={"location": "https://untrusted.invalid/", "x-amz-bucket-region": "other"})
            with self.assertRaises(GovernanceError):
                store.create(self.key, b"new")
            self.assertEqual(self.send.call_count, 1)
        self.send.reset_mock()
        self.wire(status=412, content=b"<Error><Code>PreconditionFailed</Code></Error>")
        self.assertFalse(store.create(self.key, b"new"))
        self.assertEqual(self.send.call_count, 1)
        self.send.reset_mock()
        self.send.side_effect = EndpointConnectionError(endpoint_url="https://secret-value.invalid")
        with self.assertRaises(GovernanceError):
            store.create(self.key, b"new")
        self.assertEqual(self.send.call_count, 1)
        self.socket.assert_not_called()

    def test_no_ambient_credentials_or_configuration(self):
        hostile = {"AWS_PROFILE": "not-present", "AWS_DEFAULT_PROFILE": "not-present",
                   "AWS_CONFIG_FILE": "/must-not-read", "AWS_SHARED_CREDENTIALS_FILE": "/must-not-read",
                   "AWS_ACCESS_KEY_ID": "AMBIENT", "AWS_SECRET_ACCESS_KEY": "ambient-secret",
                   "AWS_SESSION_TOKEN": "ambient-token", "AWS_WEB_IDENTITY_TOKEN_FILE": "/must-not-read",
                   "AWS_ROLE_ARN": "arn:aws:iam::123456789012:role/ambient",
                   "AWS_CONTAINER_CREDENTIALS_FULL_URI": "http://169.254.170.2/",
                   "AWS_EC2_METADATA_DISABLED": "false", "AWS_ENDPOINT_URL": "http://wrong.invalid",
                   "AWS_ENDPOINT_URL_S3": "http://wrong.invalid", "AWS_CA_BUNDLE": "/must-not-read",
                   "AWS_DATA_PATH": "/must-not-read", "AWS_DEFAULT_REGION": "wrong",
                   "AWS_MAX_ATTEMPTS": "99", "AWS_RETRY_MODE": "adaptive",
                   "AWS_S3_USE_ARN_REGION": "true", "AWS_S3_US_EAST_1_REGIONAL_ENDPOINT": "regional",
                   "AWS_CSM_ENABLED": "true", "HTTPS_PROXY": "http://wrong.invalid",
                   "SSL_CERT_FILE": "/must-not-read", "SSL_CERT_DIR": "/must-not-read"}
        with patch.dict(os.environ, hostile), \
             patch.object(CredentialResolver, "load_credentials", side_effect=AssertionError("ambient credentials")), \
             patch("botocore.configloader.load_config", side_effect=AssertionError("ambient config")), \
             patch("botocore.configloader.raw_config_parse", side_effect=AssertionError("ambient profile")):
            client = self.client()
            self.wire()
            self.assertTrue(R2ArchiveStore(self.target, client).create(self.key, b"new"))
            authorization = self.send.call_args.args[0].headers["Authorization"]
            self.assertIn(b"TESTACCESSKEY", authorization)
            self.assertNotIn(b"AMBIENT", authorization)
            self.assertEqual(self.send.call_args.args[0].headers["X-Amz-Security-Token"], b"session-secret")
            self.assertEqual(client.meta.config.retries["total_max_attempts"], 1)
            response, connection, _ = self.control()
            self.observe(connection)
            self.assertTrue(response.closed)
        self.socket.assert_not_called()
        with self.assertRaises(GovernanceError):
            R2Credentials(self.target.credential_ref, "", "")

    def test_required_only_checksums(self):
        client = self.client()
        self.wire()
        R2ArchiveStore(self.target, client).create(self.key, b"body")
        headers = {key.lower(): value for key, value in self.send.call_args.args[0].headers.items()}
        self.assertFalse(any(key.startswith("x-amz-checksum-") for key in headers))
        self.assertFalse({"x-amz-sdk-checksum-algorithm", "x-amz-trailer", "transfer-encoding", "content-encoding"} & headers.keys())
        self.assertIn("x-amz-content-sha256", headers)
        self.assertEqual(client.meta.config.request_checksum_calculation, "when_required")
        self.assertEqual(client.meta.config.response_checksum_validation, "when_required")

    def test_fresh_complete_sdk_reads_and_cleanup(self):
        client = self.client(purpose="reader")
        reader = R2ArchiveReader(self.target, client)
        for content in (b"first", b"second"):
            raw = self.wire(content=content, headers={"content-length": str(len(content)), "x-amz-storage-class": "STANDARD"})
            self.assertEqual(reader.read(self.key), content)
            self.assertTrue(raw.closed)
            request = self.send.call_args.args[0]
            self.assertEqual(request.method, "GET")
            self.assertNotIn("Range", request.headers)
            self.assertNotIn("If-None-Match", request.headers)
        self.assertEqual(self.send.call_count, 2)
        for status, headers in ((200, {"content-length": "20", "x-amz-storage-class": "STANDARD"}),
                                (206, {"content-length": "3", "x-amz-storage-class": "STANDARD"}),
                                (200, {"content-length": "3"}),
                                (200, {"content-length": "3", "x-amz-storage-class": "STANDARD", "content-encoding": "gzip"})):
            raw = self.wire(status=status, content=b"bad", headers=headers)
            with self.assertRaises(GovernanceError):
                reader.read(self.key)
            self.assertTrue(raw.closed)
        raw = self.wire(content=b"bad", headers={"content-length": "3", "x-amz-storage-class": "STANDARD"})
        with patch.object(raw, "read", side_effect=OSError("secret-value interrupted body")):
            with self.assertRaises(GovernanceError):
                reader.read(self.key)
        self.assertTrue(raw.closed)
        self.send.reset_mock()
        with self.assertRaises(GovernanceError):
            R2ArchiveStore(self.target, client).create(self.key, b"forbidden")
        self.send.assert_not_called()

    def test_secret_safe_factory_and_transport_errors(self):
        self.assertNotIn("secret-value", repr(self.credentials))
        self.assertNotIn("bearer-secret", repr(ControlCredentials("config", "bearer-secret")))
        with patch.object(boto3.Session, "client", side_effect=RuntimeError("secret-value")):
            with self.assertRaises(GovernanceError) as caught:
                self.client()
        self.assertEqual(str(caught.exception), "R2 SDK construction failed")
        client = self.client()
        self.send.side_effect = RuntimeError("secret-value Authorization bearer-secret")
        with self.assertRaises(GovernanceError) as caught:
            R2ArchiveStore(self.target, client).create(self.key, b"body")
        self.assertNotIn("secret", str(caught.exception))
        _, connection, _ = self.control()
        with patch.object(connection, "request", side_effect=RuntimeError("bearer-secret")):
            with self.assertRaises(GovernanceError) as caught:
                self.observe(connection)
        self.assertEqual(str(caught.exception), "R2 control observation failed")
        self.assertTrue(connection.closed)
        _, connection, _ = self.control()
        with patch.object(connection, "close", side_effect=RuntimeError("bearer-secret")):
            with self.assertRaises(GovernanceError) as caught:
                self.observe(connection)
        self.assertEqual(str(caught.exception), "R2 control cleanup failed")

    def test_controlled_get_target_binding_and_observations(self):
        seen = []
        for jurisdiction in ("default", "eu", "us", "fedramp"):
            target = replace(self.target, jurisdiction=jurisdiction)
            for operation in ("metadata", "locks"):
                response, connection, raw = self.control(target=target, operation=operation)
                observation, factory = self.observe(connection, target=target, operation=operation)
                parsed = json.loads(observation.envelope)
                self.assertEqual(parsed["response_sha256"], sha256_bytes(raw))
                self.assertEqual(parsed["raw_response"].encode(), raw)
                self.assertNotIn(b"bearer-secret", observation.envelope)
                self.assertNotIn(b"Authorization", observation.envelope)
                self.assertEqual(factory.call_args.args, ("api.cloudflare.com",))
                context = factory.call_args.kwargs["context"]
                self.assertEqual(context.verify_mode, ssl.CERT_REQUIRED)
                self.assertTrue(context.check_hostname)
                method, path, headers = connection.calls[0]
                self.assertEqual(method, "GET")
                self.assertEqual(path, "/client/v4" + parsed["path"])
                self.assertEqual(headers["Authorization"], "Bearer bearer-secret")
                self.assertEqual(headers.get("cf-r2-jurisdiction"), None if jurisdiction == "default" else jurisdiction)
                self.assertTrue(response.closed and connection.closed)
                seen.append(response)
                with self.assertRaises(GovernanceError):
                    observation.parse(replace(target, bucket="wrong-bucket"), operation)
        self.assertEqual(len({id(response) for response in seen}), 8)

    def test_controlled_get_failures_and_cleanup(self):
        good = canonical_json_bytes({"success": True, "errors": [], "messages": [], "result": {}})
        cases = [(good, 302, [("Location", "https://wrong.invalid")]),
                 (good, 403, []), (good, 500, []),
                 (good, 200, [("Content-Length", str(len(good) + 1))]),
                 (good, 200, [("Content-Length", "-1")]),
                 (good, 200, [("Content-Length", "5"), ("content-length", "5")]),
                 (good, 200, [("Content-Encoding", "gzip")]),
                 (b"x" * (MAX_CONTROL_BYTES + 1), 200, []),
                 (b'{"success":true,"success":true}', 200, []),
                 (b"not-json bearer-secret", 200, []), (b"\xff", 200, []),
                 (good, 200, [("CF-Ray", "https://secret.invalid")])]
        for raw, status, headers in cases:
            with self.subTest(status=status, size=len(raw)):
                response, connection, _ = self.control(raw, status=status, headers=headers)
                with self.assertRaises(GovernanceError) as caught:
                    self.observe(connection)
                self.assertEqual(str(caught.exception), "R2 control observation failed")
                self.assertTrue(response.closed and connection.closed)
                self.assertEqual(len(connection.calls), 1)
        with patch("http.client.HTTPSConnection") as factory:
            for operation in ("delete", "lifecycle", "metadata?redirect"):
                with self.assertRaises(GovernanceError):
                    observe_control(self.target, ControlCredentials("config", "bearer-secret"), operation=operation)
            factory.assert_not_called()

    def test_lifecycle_sdk_observation_binding(self):
        target = replace(self.target, credential_ref="config-reader")
        client = self.client(target, purpose="configuration")
        self.wire(content=b'<LifecycleConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">'
                  b'<Rule><ID>abort</ID><Status>Enabled</Status><Filter><Prefix></Prefix></Filter>'
                  b'<AbortIncompleteMultipartUpload><DaysAfterInitiation>7</DaysAfterInitiation>'
                  b'</AbortIncompleteMultipartUpload></Rule></LifecycleConfiguration>',
                  headers={"x-amz-request-id": "synthetic-lifecycle"})
        observation = observe_lifecycle(self.target, client, authority_ref="config-reader")
        self.assertEqual(observation.parse(self.target, "lifecycle")["Rules"][0]["ID"], "abort")
        request = self.send.call_args.args[0]
        self.assertEqual(request.url, f"{self.target.endpoint}/{self.target.bucket}?lifecycle")
        self.assertEqual(request.method, "GET")
        with Stubber(client) as stub:
            stub.add_client_error("get_bucket_lifecycle_configuration", "NoSuchLifecycleConfiguration",
                                  http_status_code=404, expected_params={"Bucket": target.bucket})
            observation = observe_lifecycle(self.target, client, authority_ref="config-reader")
            self.assertEqual(observation.parse(self.target, "lifecycle"), {"Rules": []})
            stub.add_client_error("get_bucket_lifecycle_configuration", "AccessDenied",
                                  service_message="secret-value", http_status_code=403,
                                  expected_params={"Bucket": target.bucket})
            with self.assertRaises(GovernanceError):
                observe_lifecycle(self.target, client, authority_ref="config-reader")
            stub.assert_no_pending_responses()
        self.send.reset_mock()
        with self.assertRaises(GovernanceError):
            observe_lifecycle(replace(self.target, bucket="wrong-bucket"), client, authority_ref="config-reader")
        self.send.assert_not_called()


if __name__ == "__main__":
    unittest.main()

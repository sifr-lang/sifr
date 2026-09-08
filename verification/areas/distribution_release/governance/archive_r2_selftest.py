"""Nine offline R2 contracts. Doubles prove logic, never live custody/access."""

from __future__ import annotations

import copy
import io
import json
import socket
import subprocess
import unittest
from dataclasses import replace
from types import SimpleNamespace
from unittest.mock import patch

from .archive_manifest import construct_manifest
from .archive_offline_selftest import ATTEMPT, RUN, SOURCE, synthetic_bundle
from .archive_r2_config import PolicyObservation, R2Target, admit_configuration
from .archive_r2_store import (
    MAX_SINGLE_PART_BYTES, R2ArchiveReader, R2ArchiveStore, R2ReadbackStore,
)
from .archive_store import attest_readback, manifest_key, readback, seal
from .common import GovernanceError, canonical_json_bytes, sha256_bytes


class SDKError(Exception):
    def __init__(self, status, code):
        super().__init__("secret-value https://private.example/?token=never-print")
        self.response = {"ResponseMetadata": {"HTTPStatusCode": status}, "Error": {"Code": code}}


class Body(io.BytesIO):
    def __init__(self, content, *, fail=False, close_fail=False):
        super().__init__(content)
        self.fail = fail
        self.close_fail = close_fail
        self.eof = False
        self.read_calls = 0

    def read(self, amount=-1):
        self.read_calls += 1
        if self.fail:
            raise OSError("secret-value read failed")
        result = super().read(amount)
        if not result:
            self.eof = True
        return result

    def close(self):
        super().close()
        if self.close_fail:
            raise OSError("secret-value close failed")


class S3Double:
    """Only GET/PUT exist; every attempted operation is recorded."""

    def __init__(self, target, objects=None, *, read_only=False):
        config = SimpleNamespace(region_name="auto", s3={"addressing_style": "path"},
                                 retries={"total_max_attempts": 1, "mode": "standard"},
                                 request_checksum_calculation="when_required",
                                 response_checksum_validation="when_required")
        self.meta = SimpleNamespace(endpoint_url=target.endpoint, config=config)
        self.objects = {} if objects is None else objects
        self.calls = []
        self.bodies = []
        self.read_only = read_only
        self.put_error = None
        self.commit_before_error = False
        self.put_response = {"ResponseMetadata": {"HTTPStatusCode": 200}, "ETag": '"opaque"'}
        self.get_error = None
        self.get_override = {}
        self.body_fail = False
        self.close_fail = False
        self.fail_receipt = False

    def put_object(self, **kwargs):
        self.calls.append(("put", kwargs))
        if self.read_only:
            raise AssertionError("read-only client received write")
        if self.fail_receipt and "/receipts/" in kwargs["Key"]:
            raise SDKError(503, "ServiceUnavailable")
        if self.put_error and not self.commit_before_error:
            raise self.put_error
        if kwargs["Key"] in self.objects:
            raise SDKError(412, "PreconditionFailed")
        self.objects[kwargs["Key"]] = kwargs["Body"]
        if self.put_error:
            raise self.put_error
        return self.put_response

    def get_object(self, **kwargs):
        self.calls.append(("get", kwargs))
        if self.get_error:
            raise self.get_error
        if kwargs["Key"] not in self.objects:
            raise SDKError(404, "NoSuchKey")
        raw = self.objects[kwargs["Key"]]
        body = Body(raw, fail=self.body_fail, close_fail=self.close_fail)
        self.bodies.append(body)
        return {"ResponseMetadata": {"HTTPStatusCode": 200}, "ContentLength": len(raw),
                "Body": body, "StorageClass": "STANDARD", "ETag": '"not-a-sha256"',
                **self.get_override}


def observation(target, operation, result, *, status=200):
    path = f"/accounts/{target.account}/r2/buckets/{target.bucket}"
    if operation == "lifecycle":
        raw = canonical_json_bytes({"ResponseMetadata": {"HTTPStatusCode": status, "RequestId": "request-1"}, **result})
    else:
        raw = canonical_json_bytes({"success": True, "errors": [], "messages": [], "result": result})
    envelope = {
        "account": target.account, "jurisdiction": target.jurisdiction, "bucket": target.bucket,
        "archive_root": target.root, "source_commit": target.expected_source,
        "run_id": target.expected_run, "run_attempt": target.expected_attempt,
        "endpoint": target.endpoint if operation == "lifecycle" else "https://api.cloudflare.com/client/v4",
        "method": "GET", "path": f"/{target.bucket}?lifecycle" if operation == "lifecycle" else path + ("/lock" if operation == "locks" else ""),
        "operation": operation,
        "headers": {"cf-r2-jurisdiction": target.jurisdiction} if operation != "lifecycle" and target.jurisdiction != "default" else {},
        "observed_at": "2026-09-08T10:00:00Z", "status": status, "request_id": "request-1",
        "authority_ref": "synthetic-config-reader", "raw_response": raw.decode(), "response_sha256": sha256_bytes(raw),
    }
    return PolicyObservation(canonical_json_bytes(envelope))


class R2ArchiveTests(unittest.TestCase):
    def setUp(self):
        inventory, self.payloads = synthetic_bundle()
        self.manifest = construct_manifest(inventory)
        self.raw = canonical_json_bytes(self.manifest)
        self.pin = sha256_bytes(self.raw)
        self.target = R2Target("a" * 32, "default", "synthetic-primary", "synthetic-producer",
                               self.raw, self.pin, SOURCE, RUN, ATTEMPT)
        self.key = manifest_key(self.manifest)
        self.object_key = self.manifest["artifacts"][0]["key"]
        self.expected = dict(manifest_sha256=self.pin, expected_source=SOURCE,
                             expected_run=RUN, expected_attempt=ATTEMPT)

    def store(self, target=None, objects=None):
        target = target or self.target
        client = S3Double(target, objects)
        return R2ArchiveStore(target, client), client

    def seal(self, store):
        return seal(store, self.manifest, lambda record: self.payloads[record["path"]])

    def policy(self, *, target=None, rules=None, lifecycle=None, metadata=None, lifecycle_status=200):
        target = target or self.target
        if rules is None:
            rules = [{"id": "forever", "enabled": True, "condition": {"type": "Indefinite"}}]
        if metadata is None:
            metadata = {"name": target.bucket, "jurisdiction": target.jurisdiction,
                        "storageClass": "Standard", "creationDate": "2026-09-08T00:00:00Z"}
        return dict(metadata=observation(target, "metadata", metadata),
                    locks=observation(target, "locks", {"rules": rules}),
                    lifecycle=observation(target, "lifecycle", {"Rules": []} if lifecycle is None else lifecycle,
                                          status=lifecycle_status))

    def assert_policy_fails(self, **kwargs):
        with self.assertRaises(GovernanceError):
            admit_configuration(self.target, **self.policy(**kwargs))

    def test_target_endpoint_key_and_location_binding(self):
        for jurisdiction in ("default", "eu", "us", "fedramp"):
            target = replace(self.target, jurisdiction=jurisdiction)
            self.assertEqual(target.endpoint, "https://" + "a" * 32 +
                             ("." if jurisdiction == "default" else f".{jurisdiction}.") + "r2.cloudflarestorage.com")
            store, client = self.store(target)
            self.assertTrue(store.create(self.object_key, b"fresh"))
            self.assertEqual(store.location, target.location)
        for field, values in {
            "account": ["A" * 32, "a" * 31, "user@host"],
            "jurisdiction": ["EU", "default/path"],
            "bucket": ["UPPER", "a", "a.b", "-bad", "bad-", "b" * 64, "a/b"],
            "credential_ref": ["", "https://secret?token=value"],
            "storage_class": ["Standard", "STANDARD_IA"],
            "expected_source": ["b" * 40], "expected_run": [RUN + 1, True],
            "expected_attempt": [ATTEMPT + 1], "manifest_sha256": ["b" * 64],
        }.items():
            for value in values:
                with self.subTest(field=field, value=value), self.assertRaises(GovernanceError):
                    replace(self.target, **{field: value})
        wrong = copy.deepcopy(self.manifest)
        wrong["archive_root"] = "sifr/wrong/"
        raw = canonical_json_bytes(wrong)
        with self.assertRaises(GovernanceError):
            replace(self.target, manifest_bytes=raw, manifest_sha256=sha256_bytes(raw))
        store, client = self.store()
        for key in ["/" + self.key, self.key.replace(SOURCE, "b" * 40), self.target.root + "../manifest.json",
                    self.target.root + "objects/sha256/" + "A" * 64, self.target.root + "%6danifest.json",
                    self.target.root + "x\\manifest.json", self.target.root + "unknown", self.key + "?x=1",
                    self.target.root + "x" * 1025]:
            with self.assertRaises(GovernanceError):
                store.create(key, b"x")
            with self.assertRaises(GovernanceError):
                store.read(key)
        self.assertEqual(client.calls, [])
        for field, value in [("endpoint_url", self.target.endpoint + "/"), ("endpoint_url", "https://custom.example"),
                             ("endpoint_url", self.target.endpoint + ":443"), ("endpoint_url", self.target.endpoint + "?x=1")]:
            bad = S3Double(self.target)
            setattr(bad.meta, field, value)
            with self.assertRaises(GovernanceError):
                R2ArchiveStore(self.target, bad)
        for field, value in [("region_name", "us-east-1"), ("s3", {"addressing_style": "virtual"}),
                             ("retries", {"total_max_attempts": 2}), ("retries", {"max_attempts": 1}),
                             ("request_checksum_calculation", "when_supported")]:
            bad = S3Double(self.target)
            setattr(bad.meta.config, field, value)
            with self.assertRaises(GovernanceError):
                R2ArchiveStore(self.target, bad)
        client.meta.endpoint_url = "https://other.example"
        with self.assertRaises(GovernanceError):
            store.create(self.key, b"x")
        self.assertEqual(client.calls, [])
        self.assertEqual(replace(self.target, credential_ref="other-handle").location, self.target.location)
        inventory, _ = synthetic_bundle()
        inventory["identity"]["run_id"] += 1
        for record in inventory["artifacts"]:
            record["producer"]["run_id"] += 1
        another_raw = canonical_json_bytes(construct_manifest(inventory))
        another = replace(self.target, manifest_bytes=another_raw, manifest_sha256=sha256_bytes(another_raw),
                          expected_run=RUN + 1)
        self.assertNotEqual(another.root, self.target.root)
        self.assertEqual(another.location, self.target.location)

    def test_atomic_conditional_create_and_existing_content(self):
        store, client = self.store()
        competitor, other = self.store(objects=client.objects)
        self.assertTrue(store.create(self.object_key, b"first"))
        self.assertFalse(competitor.create(self.object_key, b"second"))
        self.assertEqual(client.objects[self.object_key], b"first")
        self.assertEqual(client.calls, [("put", dict(Bucket=self.target.bucket, Key=self.object_key, Body=b"first",
                                                    ContentLength=5, IfNoneMatch="*", StorageClass="STANDARD"))])
        self.assertEqual(len(other.calls), 1)
        with self.assertRaises(GovernanceError):
            self.seal(store)
        self.assertNotIn(self.key, client.objects)
        store, client = self.store()
        record = self.manifest["artifacts"][0]
        store.create(record["key"], self.payloads[record["path"]])
        self.assertEqual(self.seal(store), self.pin)
        original = client.objects[self.key]
        with self.assertRaises(GovernanceError):
            self.seal(store)
        self.assertEqual(client.objects[self.key], original)

    def test_create_errors_and_ambiguous_completion_fail_closed(self):
        errors = [SDKError(code, "Denied") for code in (401, 403, 404, 409, 429, 500, 503)]
        errors += [SDKError(412, "WrongCode"), SDKError(403, "PreconditionFailed"),
                   SDKError(500, "https://secret-value/token"), TimeoutError("secret-value")]
        for error in errors:
            store, client = self.store()
            client.put_error = error
            with self.assertRaises(GovernanceError) as caught:
                store.create(self.object_key, b"x")
            self.assertNotIn("secret-value", str(caught.exception))
            self.assertEqual(len(client.calls), 1)
            self.assertEqual(client.objects, {})
        for response in (None, {}, {"ResponseMetadata": {"HTTPStatusCode": 201}, "ETag": "x"},
                         {"ResponseMetadata": {"HTTPStatusCode": 200}},
                         {"ResponseMetadata": {"HTTPStatusCode": 200}, "ETag": "x", "Error": {}}):
            store, client = self.store()
            client.put_response = response
            with self.assertRaises(GovernanceError):
                store.create(self.object_key, b"x")
            self.assertEqual(len(client.calls), 1)
        for committed in (False, True):
            store, client = self.store()
            client.put_error = TimeoutError("secret-value")
            client.commit_before_error = committed
            with self.assertRaises(GovernanceError):
                self.seal(store)
            self.assertEqual(len(client.calls), 1)
            self.assertNotIn(self.key, client.objects)
            self.assertEqual(bool(client.objects), committed)
            self.assertFalse(any("/receipts/" in key for key in client.objects))
            client.put_error = None
            self.assertEqual(self.seal(store), self.pin)

    def test_fresh_complete_reads_and_body_cleanup(self):
        store, client = self.store()
        client.objects[self.object_key] = b"abc"
        self.assertEqual(store.read(self.object_key), b"abc")
        client.objects[self.object_key] = b"changed"
        self.assertEqual(store.read(self.object_key), b"changed")
        self.assertEqual(client.calls, [("get", {"Bucket": self.target.bucket, "Key": self.object_key})] * 2)
        self.assertTrue(all(body.closed and body.eof for body in client.bodies))
        for override in ({"ResponseMetadata": {"HTTPStatusCode": 206}}, {"ContentRange": "bytes 0-2/7"},
                         {"ResponseMetadata": {"HTTPStatusCode": 200, "HTTPHeaders": {"content-range": "bytes 0-2/7"}}},
                         {"ContentLength": 100}, {"ContentLength": 1}, {"ContentLength": None},
                         {"ContentLength": MAX_SINGLE_PART_BYTES + 1}, {"ContentLength": True},
                         {"StorageClass": None}, {"ContentEncoding": "gzip"}, {"Expiration": "expiry"}):
            client.get_override = override
            with self.assertRaises(GovernanceError):
                store.read(self.object_key)
            self.assertTrue(client.bodies[-1].closed)
        client.get_override = {}
        for setting in ("body_fail", "close_fail"):
            setattr(client, setting, True)
            with self.assertRaises(GovernanceError) as caught:
                store.read(self.object_key)
            self.assertNotIn("secret-value", str(caught.exception))
            self.assertTrue(client.bodies[-1].closed)
            setattr(client, setting, False)
        for error in (SDKError(403, "AccessDenied"), SDKError(404, "NoSuchKey"), OSError("secret-value")):
            client.get_error = error
            with self.assertRaises(GovernanceError):
                store.read(self.object_key)
        client.get_error = None
        with patch.object(client, "get_object", return_value=None):
            with self.assertRaises(GovernanceError):
                store.read(self.object_key)
        for body in (SimpleNamespace(read=lambda _: "not bytes", close=lambda: None),
                     SimpleNamespace(read=lambda _: b"", close=lambda: None)):
            with patch.object(client, "get_object", return_value={"ResponseMetadata": {"HTTPStatusCode": 200},
                              "Body": body, "ContentLength": 3, "StorageClass": "STANDARD"}):
                with self.assertRaises(GovernanceError):
                    store.read(self.object_key)
        client.get_error = GovernanceError("secret-value from transport")
        with self.assertRaises(GovernanceError) as caught:
            store.read(self.object_key)
        self.assertNotIn("secret-value", str(caught.exception))
        client.get_error = None
        client.objects.clear()
        self.seal(store)
        client.objects[self.object_key] += b"mutation"
        with self.assertRaises(GovernanceError):
            readback(store, key=self.key, **self.expected)

    def test_indefinite_complete_root_lock_admission(self):
        for prefix in (None, "", "sifr/", self.target.root):
            rule = {"id": "forever", "enabled": True, "condition": {"type": "Indefinite"}}
            if prefix is not None:
                rule["prefix"] = prefix
            admitted = admit_configuration(self.target, **self.policy(rules=[rule]))
            self.assertEqual(admitted.assurance, "supplied-policy-observations-only")
        base = {"id": "forever", "enabled": True, "condition": {"type": "Indefinite"}}
        for rules in ([], [{**base, "enabled": False}], [{**base, "prefix": self.target.root + "objects/"}],
                      [{**base, "condition": {"type": "Age", "maxAgeSeconds": 50}}],
                      [{**base, "condition": {"type": "Date", "date": "2099-01-01T00:00:00Z"}}],
                      [base, {**base, "id": "other", "condition": {"type": "Unknown"}}],
                      [base, base], [{**base, "enabled": 1}], [{**base, "unexpected": True}]):
            self.assert_policy_fails(rules=rules)
        policies = self.policy()
        for key, value in {"account": "b" * 32, "bucket": "wrong-bucket", "endpoint": "https://wrong.example",
                           "archive_root": "sifr/wrong/", "source_commit": "b" * 40, "run_id": RUN + 1,
                           "run_attempt": ATTEMPT + 1, "method": "PUT", "path": "/wrong", "headers": {"Authorization": "x"},
                           "authority_ref": self.target.credential_ref, "observed_at": "2026-09-08", "status": 403,
                           "response_sha256": "b" * 64}.items():
            envelope = json.loads(policies["locks"].envelope)
            envelope[key] = value
            with self.assertRaises(GovernanceError):
                admit_configuration(self.target, **{**policies, "locks": PolicyObservation(canonical_json_bytes(envelope))})
        for operation in ("metadata", "locks", "lifecycle"):
            envelope = json.loads(policies[operation].envelope)
            raw = envelope["raw_response"]
            first_key = next(iter(json.loads(raw)))
            raw = '{"' + first_key + '":null,' + raw[1:]
            envelope.update(raw_response=raw, response_sha256=sha256_bytes(raw.encode()))
            with self.assertRaises(GovernanceError):
                admit_configuration(self.target, **{**policies, operation: PolicyObservation(canonical_json_bytes(envelope))})
        duplicate = b'{"account":"duplicate",' + policies["locks"].envelope[1:]
        with self.assertRaises(GovernanceError):
            admit_configuration(self.target, **{**policies, "locks": PolicyObservation(duplicate)})
        eu = replace(self.target, jurisdiction="eu")
        admit_configuration(eu, **self.policy(target=eu))

    def test_no_expiry_standard_lifecycle_admission(self):
        for prefix in ("", "sifr/", self.target.root, self.target.root + "receipts/"):
            for action in ({"Expiration": {"Days": 30}}, {"Transitions": [{"Days": 0, "StorageClass": "STANDARD_IA"}]},
                           {"NoncurrentVersionExpiration": {"NoncurrentDays": 1}}):
                self.assert_policy_fails(lifecycle={"Rules": [{"Status": "Enabled", "Filter": {"Prefix": prefix}, **action}]})
        for rule in ({"Status": "Disabled", "Prefix": "", "Expiration": {"Days": 1}},
                     {"Status": "Enabled", "Filter": {"Prefix": "unrelated/"}, "Expiration": {"Days": 1}},
                     {"Status": "Enabled", "Filter": {}, "AbortIncompleteMultipartUpload": {"DaysAfterInitiation": 7}}):
            admit_configuration(self.target, **self.policy(lifecycle={"Rules": [rule]}))
        for lifecycle in ({}, {"Rules": None}, {"Rules": [{"Status": "Enabled", "Filter": {"Tag": {"Key": "x", "Value": "y"}}, "Expiration": {"Days": 1}}]},
                          {"Rules": [{"Status": "Enabled", "Prefix": "", "MysteryAction": {}}]},
                          {"Rules": [{"Status": "Enabled", "Prefix": "", "Filter": {}, "Expiration": {"Days": 1}}]},
                          {"Rules": [{"Status": "Enabled", "Prefix": ""}]},
                          {"Rules": [{"Status": "Disabled", "Prefix": "", "Expiration": {"Mystery": 1}}]},
                          {"Error": {"Code": "AccessDenied"}}):
            self.assert_policy_fails(lifecycle=lifecycle)
        admit_configuration(self.target, **self.policy(lifecycle={"Error": {"Code": "NoSuchLifecycleConfiguration"}}, lifecycle_status=404))
        self.assert_policy_fails(lifecycle={"Error": {"Code": "AccessDenied"}}, lifecycle_status=404)
        self.assert_policy_fails(lifecycle_status=403)
        good = {"name": self.target.bucket, "jurisdiction": "default", "storageClass": "Standard", "creationDate": "2026-09-08T00:00:00Z"}
        for field, value in (("name", "wrong-bucket"), ("jurisdiction", "eu"), ("storageClass", "InfrequentAccess"), ("storageClass", None)):
            self.assert_policy_fails(metadata={**good, field: value})
        self.assert_policy_fails(metadata={key: value for key, value in good.items() if key != "storageClass"})

    def readback_handle(self, target, objects):
        reader_target = replace(target, credential_ref="synthetic-reader")
        writer_target = replace(target, credential_ref="synthetic-receipt-writer")
        reader = S3Double(reader_target, objects, read_only=True)
        writer = S3Double(writer_target, objects)
        handle = R2ReadbackStore(R2ArchiveReader(reader_target, reader), R2ArchiveStore(writer_target, writer))
        return handle, reader, writer

    def test_reader_receipt_writer_and_custody_separation(self):
        store, producer = self.store()
        self.seal(store)
        handle, reader, writer = self.readback_handle(self.target, producer.objects)
        producer_calls = len(producer.calls)
        receipt = attest_readback([handle], key=self.key, receipt_id="read-only-1", kind="readback", **self.expected)
        self.assertEqual(receipt["assurance"], "complete-byte-readback-only")
        self.assertEqual(len(producer.calls), producer_calls)
        self.assertTrue(all(op == "get" for op, _ in reader.calls))
        self.assertEqual(len(writer.calls), 1)
        self.assertTrue(all(op == "put" and "/receipts/" in args["Key"] for op, args in writer.calls))
        with self.assertRaises(GovernanceError):
            handle.create(self.object_key, b"x")
        reader.get_error = SDKError(403, "AccessDenied")
        with self.assertRaises(GovernanceError):
            handle.read(self.key)
        self.assertEqual(len(producer.calls), producer_calls)
        reader.get_error = None
        reader_only = R2ArchiveReader(replace(self.target, credential_ref="reader"), reader)
        with self.assertRaises(GovernanceError):
            R2ReadbackStore(reader_only, None)
        wrong, _ = self.store(replace(self.target, bucket="synthetic-other"))
        with self.assertRaises(GovernanceError):
            R2ReadbackStore(reader_only, wrong)
        with self.assertRaises(GovernanceError):
            R2ReadbackStore(reader_only, R2ArchiveStore(reader_only.target, reader))
        alias, _, _ = self.readback_handle(replace(self.target, credential_ref="alias"), producer.objects)
        with self.assertRaises(GovernanceError):
            attest_readback([handle, alias], key=self.key, receipt_id="aliases", kind="copy", **self.expected)

    def test_all_class_seal_readback_and_interrupted_copy(self):
        primary, primary_client = self.store()
        copy_target = replace(self.target, bucket="synthetic-copy")
        secondary, copy_client = self.store(copy_target)
        self.assertEqual(len(self.manifest["matrix"]), 6)
        self.assertEqual(self.seal(primary), self.pin)
        self.assertEqual(self.seal(secondary), self.pin)
        primary_reader, _, _ = self.readback_handle(self.target, primary_client.objects)
        copy_reader, _, writer = self.readback_handle(copy_target, copy_client.objects)
        stores = [primary_reader, copy_reader]
        original = primary_client.objects[self.key]
        for bad in (None, b"mutated"):
            saved = copy_client.objects.pop(self.object_key)
            if bad is not None:
                copy_client.objects[self.object_key] = bad
            with self.assertRaises(GovernanceError):
                attest_readback(stores, key=self.key, receipt_id="copy-missing", kind="copy", **self.expected)
            self.assertFalse(any("/receipts/" in key for key in primary_client.objects))
            copy_client.objects[self.object_key] = saved
        for field, value in (("manifest_sha256", "b" * 64), ("expected_source", "b" * 40),
                             ("expected_run", RUN + 1), ("expected_attempt", ATTEMPT + 1)):
            with self.assertRaises(GovernanceError):
                attest_readback(stores, key=self.key, receipt_id="wrong-pin", kind="copy", **{**self.expected, field: value})
        writer.fail_receipt = True
        with self.assertRaises(GovernanceError):
            attest_readback(stores, key=self.key, receipt_id="interrupted-copy", kind="copy", **self.expected)
        partial = [raw for key, raw in primary_client.objects.items() if "/receipts/" in key]
        self.assertEqual(len(partial), 1)
        self.assertEqual(json.loads(partial[0])["assurance"], "complete-byte-readback-only")
        self.assertFalse(any("/receipts/" in key for key in copy_client.objects))
        self.assertEqual(primary_client.objects[self.key], original)
        self.assertEqual(copy_client.objects[self.key], original)
        writer.fail_receipt = False
        receipt = attest_readback(stores, key=self.key, receipt_id="complete-copy", kind="copy", **self.expected)
        self.assertEqual(receipt["locations"], sorted([self.target.location, copy_target.location]))
        self.assertEqual(receipt["assurance"], "complete-byte-readback-only")
        self.assertNotIn("independence", receipt)

    def test_single_part_size_and_no_live_side_effects(self):
        class OverLimit(bytes):
            def __len__(self):
                return MAX_SINGLE_PART_BYTES + 1

        with patch.object(socket, "socket", side_effect=AssertionError("network forbidden")), \
             patch.object(subprocess, "Popen", side_effect=AssertionError("subprocess forbidden")), \
             patch.dict("os.environ", {}, clear=True):
            store, client = self.store()
            with self.assertRaises(GovernanceError):
                store.create(self.object_key, OverLimit(b"tiny"))
            self.assertEqual(client.calls, [])
            with patch("verification.areas.distribution_release.governance.archive_r2_store.MAX_SINGLE_PART_BYTES", 2):
                with self.assertRaises(GovernanceError):
                    store.create(self.object_key, b"abc")
            self.assertEqual(client.calls, [])
            client.objects[self.object_key] = b"tiny"
            client.get_override = {"ContentLength": MAX_SINGLE_PART_BYTES + 1}
            with self.assertRaises(GovernanceError):
                store.read(self.object_key)
            self.assertTrue(client.bodies[-1].closed)
            self.assertEqual(client.bodies[-1].read_calls, 0)
            client.get_override = {}
            client.objects.clear()
            self.assertEqual(self.seal(store), self.pin)
            admit_configuration(self.target, **self.policy())


if __name__ == "__main__":
    unittest.main()

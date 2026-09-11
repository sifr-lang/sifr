"""Synthetic archive contracts only: no network, native build, or real custody."""

from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest.mock import patch

from .archive_bindings import REQUIRED_ROLES, UPLOAD_JOBS
from .archive_manifest import (
    SCHEMA, SKIP_TARGETS, WORKFLOW, construct_manifest, inventory_of, parse_manifest,
    plan, read_local, verify_local, verify_payloads,
)
from .archive_store import attest_readback, manifest_key, readback, seal
from .artifact_index import validate_qualification_artifact_index
from .common import (
    BUILDERS, TARGETS, GovernanceError, canonical_json_bytes, sha256_bytes,
)

SOURCE = "123456789abcdef0" * 2 + "12345678"
RUN = 987654321
ATTEMPT = 3
SUBMODULES = {"editor_integrations": "23456789abcdef01" * 2 + "23456789"}


class MemoryStore:
    """Intentionally a test double, never imported by the offline CLI."""

    def __init__(self, location="test-memory:primary"):
        self.location = location
        self.objects = {}
        self.reads = []
        self.fail_after = None
        self.creates = 0

    def create(self, key, content):
        if self.fail_after is not None and self.creates >= self.fail_after:
            raise GovernanceError("synthetic interrupted create")
        self.creates += 1
        if key in self.objects:
            return False
        self.objects[key] = content
        return True

    def read(self, key):
        self.reads.append(key)
        if key not in self.objects:
            raise GovernanceError("synthetic missing object")
        return self.objects[key]


def synthetic_bundle():
    """All-class fresh identities; older generic fixtures supply shapes only."""
    from .schema_contracts import qualification_index, release_report
    index = qualification_index()
    index.update(source_commit=SOURCE, submodules=SUBMODULES)
    index["workflow"].update(run_id=RUN, run_attempt=ATTEMPT, expires_at="2000-01-31T00:00:00Z")
    prefix = f"sifr-stable-candidate-0.1.0-{SOURCE}-"
    report = release_report()
    report["source"].update(commit=SOURCE, submodules=SUBMODULES)
    # An extra synthetic area proves expanded coverage is not the old four-area list.
    report["profile"]["expanded_selected_areas"].append({"area": "new_area", "suites": ["extra"]})
    report["steps"].append({"name": "new_area", "status": "pass", "elapsed_ms": 1,
                            "suite_results": [{"area": "new_area", "suite": "extra", "status": "pass",
                                               "case_ids": ["fresh-case"], "result_artifact_sha256": "a" * 64}]})
    profile = {"schema_version": 2, "name": "release",
               "selected_areas": report["profile"]["expanded_selected_areas"]}
    identity = {"repository": "sifr-lang/sifr", "source_commit": SOURCE, "workflow_path": WORKFLOW,
                "run_id": RUN, "run_attempt": ATTEMPT, "index_sha256": "a" * 64,
                "submodules": SUBMODULES, "lock_sha256": sha256_bytes(b"synthetic lock\n"),
                "toolchain": report["toolchain"], "profile_sha256": sha256_bytes(canonical_json_bytes(profile))}
    producer = {key: identity[key] for key in ("repository", "source_commit", "workflow_path", "run_id", "run_attempt")}
    rows = {}
    payloads = {}

    def add(role, data, *, path=None, job="local-release-profile", execution="local", artifact_id=None):
        raw = data if isinstance(data, bytes) else canonical_json_bytes(data)
        path = path or "payload/" + role.replace(":", "_").replace("/", "_") + ".data"
        owner = {**producer, "job": job, "execution": execution}
        if artifact_id is not None:
            owner["artifact_id"] = artifact_id
        rows[role] = {"role": role, "path": path, "size_bytes": len(raw), "sha256": sha256_bytes(raw), "producer": owner}
        payloads[path] = raw

    # Create the native bytes before their JSON reports and the index that binds them.
    for artifact in index["artifacts"]:
        job = artifact["workflow_artifact_name"].rsplit("-", 1)[-1]
        job = next((target for target in TARGETS if artifact["id"].endswith(target)), job)
        artifact["workflow_artifact_name"] = prefix + job
        artifact["expires_at"] = "2000-01-31T00:00:00Z"
        add(artifact["id"], f"synthetic:{artifact['id']}\n".encode(), path=f"native/{job}/{artifact['name']}",
            job=job, execution="workflow", artifact_id=artifact["workflow_artifact_id"])
    for target in TARGETS:
        role = f"qualification-report-{target}"
        add(role, {"source_commit": SOURCE, "target": target, "builder": BUILDERS[target], "smoke_status": "pass",
                   "archive_sha256": rows[f"binary-archive-{target}"]["sha256"],
                   "checksum_sha256": rows[f"checksum-{target}"]["sha256"],
                   "sysroot_bundle_sha256": rows[f"sysroot-{target}"]["sha256"]},
            path=rows[role]["path"], job=target, execution="workflow", artifact_id=rows[role]["producer"]["artifact_id"])
    add("editor-qualification-report", {"source_commit": SOURCE, "vsix_sha256": rows["vsix"]["sha256"]},
        path=rows["editor-qualification-report"]["path"], job="editor", execution="workflow", artifact_id=11)
    for artifact in index["artifacts"]:
        artifact.update({key: rows[artifact["id"]][key] for key in ("size_bytes", "sha256")})
    add("qualification-index", index, path="payload/qualification-artifact-index.json", job="index", execution="workflow", artifact_id=12)
    identity["index_sha256"] = rows["qualification-index"]["sha256"]
    uploads = []
    for job in UPLOAD_JOBS:
        artifacts = [artifact for artifact in index["artifacts"] if artifact["workflow_artifact_name"] == prefix + job]
        upload_id = 12 if job == "index" else artifacts[0]["workflow_artifact_id"]
        uploads.append({"id": upload_id, "name": prefix + job, "workflow_run": {"id": RUN},
                        "digest": "sha256:" + "a" * 64})
        add(f"workflow-log:{job}", f"synthetic workflow log {job}\n".encode(), job=job, execution="workflow")
    add("upload-metadata", {"artifacts": uploads})
    add("run-metadata", {"id": RUN, "run_attempt": ATTEMPT, "head_sha": SOURCE, "path": WORKFLOW,
                         "event": "workflow_dispatch", "repository": {"full_name": "sifr-lang/sifr"}})
    add("profile-manifest", profile)
    report["profile"]["manifest_sha256"] = identity["profile_sha256"]
    report["result_artifacts"] = []
    for step in report["steps"]:
        area = step["suite_results"][0]["area"]
        result_path = f"target/verification/{area}-results.json"
        role = "result:" + result_path
        add(role, {"area": area, "status": "pass", "synthetic": True})
        report["result_artifacts"].append({"path": result_path, "sha256": rows[role]["sha256"]})
        add("step-log:" + step["name"], b"synthetic shared log\n")
        for suite in step["suite_results"]:
            suite["result_artifact_sha256"] = rows[role]["sha256"]
            for case in suite["case_ids"]:
                add(f"case-log:{suite['area']}:{suite['suite']}:{case}", b"synthetic shared log\n")
    add("release-profile-report", report)
    matrix = []
    policy_rows = []
    for target in (*TARGETS, *SKIP_TARGETS):
        reasons = [] if target in TARGETS else ["synthetic declared host restriction"]
        matrix.append({"target": target, "mode": "native" if target in TARGETS else "structured-skip", "reasons": reasons})
        policy_rows.append({"triple": target, "status": "supported" if target in TARGETS else "host-limited", "allowed_skips": reasons})
        if target in SKIP_TARGETS:
            add(f"structured-skip:{target}", {"source_commit": SOURCE, "target": target, "status": "structured-skip", "reasons": reasons})
    add("platform-policy", {"host_triples": policy_rows})
    add("documentation-result", {"synthetic": True, "status": "pass"})
    add("documentation-summary", {"source_commit": SOURCE, "result_sha256": rows["documentation-result"]["sha256"]})
    add("support-claims", {"claims": ["synthetic"]})
    add("release-notes", b"Synthetic release notes\n")
    add("review-log", b"Synthetic offline review; no real approval\n")
    add("review-evidence", {"source_commit": SOURCE, "log_sha256": rows["review-log"]["sha256"], "verdict": "synthetic"})
    add("candidate-plan", {
        "source_commit": SOURCE, "submodules": SUBMODULES, "cargo_lock_sha256": identity["lock_sha256"],
        "toolchain": {"rustc": identity["toolchain"]["rustc"], "cargo": identity["toolchain"]["cargo"],
                      "profile_manifest_sha256": identity["profile_sha256"]},
        "release_profile_report": {"sha256": rows["release-profile-report"]["sha256"]},
        "qualification_artifact_index": {"sha256": rows["qualification-index"]["sha256"]},
        "documentation_report": {"sha256": rows["documentation-summary"]["sha256"]},
        "rust_interop": {"stable_support_claims_sha256": rows["support-claims"]["sha256"]},
        "release_notes_sha256": rows["release-notes"]["sha256"],
    })
    for target in TARGETS:
        removed = rows.pop(f"sysroot-{target}")
        del payloads[removed["path"]]
    return {"schema_version": 1, "identity": identity, "matrix": matrix, "artifacts": list(rows.values())}, payloads


def archive_schema_example():
    inventory, _ = synthetic_bundle()
    return construct_manifest(inventory)


class ArchiveOfflineTests(unittest.TestCase):
    def setUp(self):
        self.inventory, self.payloads = synthetic_bundle()
        self.manifest = construct_manifest(self.inventory)
        self.raw = canonical_json_bytes(self.manifest)
        self.pin = sha256_bytes(self.raw)
        self.expected = dict(manifest_sha256=self.pin, expected_source=SOURCE, expected_run=RUN, expected_attempt=ATTEMPT)

    def read(self, record):
        if record["path"] not in self.payloads:
            raise GovernanceError("synthetic missing payload")
        return self.payloads[record["path"]]

    def verify(self):
        verify_payloads(self.manifest, self.read)

    def write_payloads(self, root):
        for path, raw in self.payloads.items():
            destination = root / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)

    def rewrite(self, role, update):
        """Rehash a changed assertion so negative tests reach cross-link validation."""
        row = next(row for row in self.manifest["artifacts"] if row["role"] == role)
        document = json.loads(self.payloads[row["path"]])
        update(document)
        raw = canonical_json_bytes(document)
        self.payloads[row["path"]] = raw
        inventory = inventory_of(self.manifest)
        target = next(row for row in inventory["artifacts"] if row["role"] == role)
        target.update(sha256=sha256_bytes(raw), size_bytes=len(raw))
        self.manifest = construct_manifest(inventory)

    def test_complete_all_class_roundtrip(self):
        from verification.json_schema_202012 import validate_instance
        from verification.runner.sifr_verify.selftest import GOVERNANCE_SCHEMA_COUNT
        from . import schema_epoch
        from .schema_contracts import schema_fixtures
        validate_instance(self.manifest, SCHEMA)
        registered = schema_fixtures()
        self.assertEqual(set(registered), {path.name for path in SCHEMA.parent.glob("*.schema.json")})
        self.assertEqual(registered[SCHEMA.name], self.manifest)
        self.assertEqual(len(registered), GOVERNANCE_SCHEMA_COUNT)
        declaration = json.loads(SCHEMA.read_bytes())
        schema_epoch.check_schema_declaration(SCHEMA, declaration)
        with self.assertRaises(ValueError):
            schema_epoch.check_schema_declaration(SCHEMA.with_name("release_index.schema.json"), declaration)
        changed_declaration = copy.deepcopy(declaration)
        changed_declaration["properties"]["schema_version"] = {"const": 2}
        with self.assertRaises(ValueError):
            schema_epoch.check_schema_declaration(SCHEMA, changed_declaration)
        schema_epoch.check_schema_declaration(SCHEMA.with_name("release_index.schema.json"), changed_declaration)
        expected_exclusions = {Path(__file__), Path(__file__).with_name("archive_store.py")}
        self.assertEqual(schema_epoch.ARCHIVE_SOURCE_EXCLUSIONS, expected_exclusions)
        self.assertTrue(expected_exclusions.isdisjoint(schema_epoch.governed_sources()))
        with self.assertRaises(ValueError):
            schema_epoch.check_source_text(Path("existing-release-owner.py"), '{"schema_version": 1}')
        roles = {row["role"] for row in self.manifest["artifacts"]}
        self.assertTrue(REQUIRED_ROLES <= roles)
        self.assertEqual(len(self.manifest["matrix"]), 6)
        shuffled = copy.deepcopy(self.inventory)
        shuffled["artifacts"].reverse()
        shuffled["matrix"].reverse()
        self.assertEqual(canonical_json_bytes(construct_manifest(shuffled)), self.raw)
        self.verify()
        store = MemoryStore()
        self.assertEqual(seal(store, self.manifest, self.read), self.pin)
        self.assertEqual(readback(store, key=manifest_key(self.manifest), **self.expected), self.manifest)
        # Distinct logical roles deliberately share identical content bytes.
        unique = {row["key"] for row in self.manifest["artifacts"]}
        self.assertLess(len(unique), len(roles))
        self.assertEqual(len(store.objects), len(unique) + 1)
        with tempfile.TemporaryDirectory(prefix="sifr-archive-test-") as temporary:
            root = Path(temporary).resolve()
            self.write_payloads(root / "input")
            self.write_payloads(root / "copy")
            self.assertEqual(plan(self.inventory, root / "input"), self.manifest)
            self.assertEqual(verify_local(self.raw, payload_root=root / "copy", **self.expected), self.manifest)
            entry = Path(__file__).resolve().parents[4] / "scripts/distribution/archive_release_evidence.py"
            spec = importlib.util.spec_from_file_location("archive_cli_test", entry)
            cli = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(cli)
            (root / "inventory.json").write_bytes(canonical_json_bytes(self.inventory))
            planning = ["plan", "--inventory", str(root / "inventory.json"), "--payload-root", str(root / "input"), "--out", str(root / "manifest.json")]
            verifying = ["verify-local", "--manifest", str(root / "manifest.json"), "--manifest-sha256", self.pin,
                         "--expected-source", SOURCE, "--expected-run", str(RUN), "--expected-attempt", str(ATTEMPT),
                         "--payload-root", str(root / "copy")]
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(cli.main(planning), 0)
                self.assertEqual(cli.main(verifying), 0)
                self.assertEqual(cli.main(planning), 2)
                self.assertEqual((root / "manifest.json").read_bytes(), self.raw)
                with patch.object(cli.os, "link", side_effect=OSError("synthetic interrupted install")):
                    self.assertEqual(cli.main(planning[:-1] + [str(root / "interrupted.json")]), 2)
                self.assertFalse((root / "interrupted.json").exists())
                self.assertFalse(list(root.glob(".archive-plan-*")))

    def test_missing_mutated_truncated_bytes(self):
        for row in self.manifest["artifacts"]:
            original = self.payloads[row["path"]]
            for mutation in (None, b"X" * len(original), original[:-1]):
                with self.subTest(role=row["role"], mutation="missing" if mutation is None else len(mutation)):
                    if mutation is None:
                        del self.payloads[row["path"]]
                    else:
                        self.payloads[row["path"]] = mutation
                    with self.assertRaises(GovernanceError):
                        self.verify()
                    self.payloads[row["path"]] = original
        for role in REQUIRED_ROLES:
            inventory = copy.deepcopy(self.inventory)
            inventory["artifacts"] = [row for row in inventory["artifacts"] if row["role"] != role]
            with self.subTest(missing_role=role), self.assertRaises(GovernanceError):
                construct_manifest(inventory)
        result = next(row for row in self.inventory["artifacts"] if row["role"].startswith("result:"))
        inventory = copy.deepcopy(self.inventory)
        inventory["artifacts"] = [row for row in inventory["artifacts"] if row["role"] != result["role"]]
        with self.assertRaisesRegex(GovernanceError, "missing bound result"):
            verify_payloads(construct_manifest(inventory), self.read)
        for role in ("step-log:area_rust_interop", "case-log:new_area:extra:fresh-case"):
            inventory = copy.deepcopy(self.inventory)
            inventory["artifacts"] = [row for row in inventory["artifacts"] if row["role"] != role]
            verify_payloads(construct_manifest(inventory), self.read)
        for role, update in (
            ("release-profile-report", lambda doc: doc["result_artifacts"][0].update(sha256="b" * 64)),
            ("release-profile-report", lambda doc: doc["profile"]["expanded_selected_areas"].pop()),
            ("documentation-summary", lambda doc: doc.update(result_sha256="b" * 64)),
            ("candidate-plan", lambda doc: doc.update(cargo_lock_sha256="b" * 64)),
            ("review-evidence", lambda doc: doc.update(log_sha256="b" * 64)),
            ("upload-metadata", lambda doc: doc["artifacts"][0].update(id=999999)),
            ("upload-metadata", lambda doc: doc["artifacts"][0].update(id=doc["artifacts"][1]["id"])),
            ("upload-metadata", lambda doc: doc["artifacts"][0].update(digest="not-a-digest")),
            (f"qualification-report-{TARGETS[0]}", lambda doc: doc.update(sysroot_bundle_sha256="b" * 64)),
        ):
            with self.subTest(rehashed_cross_link=role):
                self.setUp()
                self.rewrite(role, update)
                with self.assertRaises(GovernanceError):
                    self.verify()

    def test_minimal_retention_and_optional_log_bindings(self):
        inventory = copy.deepcopy(self.inventory)
        inventory["artifacts"] = [row for row in inventory["artifacts"]
                                  if not row["role"].startswith(("step-log:", "case-log:", "workflow-log:"))]
        manifest = construct_manifest(inventory)
        verify_payloads(manifest, self.read)
        roles = {row["role"] for row in manifest["artifacts"]}
        self.assertFalse(any(role.startswith(("transport-zip:", "sysroot-")) for role in roles))
        self.assertTrue({"vsix", "installer", *(f"binary-archive-{target}" for target in TARGETS)} <= roles)
        for bad_role in ("step-log:never-executed", "case-log:new_area:extra:unknown"):
            changed = copy.deepcopy(self.inventory)
            row = next(row for row in changed["artifacts"] if row["role"].startswith("step-log:"))
            row["role"] = bad_role
            with self.subTest(role=bad_role), self.assertRaisesRegex(GovernanceError, "no recorded execution"):
                verify_payloads(construct_manifest(changed), self.read)
        changed = copy.deepcopy(self.inventory)
        row = next(row for row in changed["artifacts"] if row["role"].startswith("workflow-log:"))
        row["producer"]["job"] = "wrong-job"
        with self.assertRaisesRegex(GovernanceError, "workflow log job"):
            verify_payloads(construct_manifest(changed), self.read)

    def test_duplicate_logical_records_and_wrong_source_run_attempt(self):
        duplicate = copy.deepcopy(self.inventory)
        duplicate["artifacts"].append(copy.deepcopy(duplicate["artifacts"][0]))
        with self.assertRaisesRegex(GovernanceError, "duplicate"):
            construct_manifest(duplicate)
        for key, value in (("source_commit", "c" * 40), ("run_id", RUN + 1), ("run_attempt", ATTEMPT + 1),
                           ("repository", "other/repo"), ("workflow_path", "other.yml")):
            inventory = copy.deepcopy(self.inventory)
            inventory["artifacts"][0]["producer"][key] = value
            with self.subTest(producer=key), self.assertRaises(GovernanceError):
                construct_manifest(inventory)
        for key, value in (("expected_source", "c" * 40), ("expected_run", RUN + 1), ("expected_attempt", ATTEMPT + 1)):
            with self.subTest(expected=key), self.assertRaises(GovernanceError):
                parse_manifest(self.raw, **{**self.expected, key: value})
        for field, value in (("head_sha", "c" * 40), ("id", RUN + 1), ("run_attempt", ATTEMPT + 1)):
            self.setUp()
            self.rewrite("run-metadata", lambda doc: doc.update({field: value}))
            with self.subTest(metadata=field), self.assertRaises(GovernanceError):
                self.verify()
        for mutation in (lambda inv: inv["matrix"].pop(), lambda inv: inv["matrix"].append(inv["matrix"][0]),
                         lambda inv: inv["matrix"][0].update(mode="structured-skip", reasons=["fake skip"])):
            inventory = copy.deepcopy(self.inventory)
            mutation(inventory)
            with self.assertRaises(GovernanceError):
                construct_manifest(inventory)
        duplicate_json = self.raw.replace(b'"schema_version":1', b'"schema_version":1,"schema_version":1', 1)
        with self.assertRaisesRegex(GovernanceError, "duplicate object key"):
            parse_manifest(duplicate_json, **{**self.expected, "manifest_sha256": sha256_bytes(duplicate_json)})
        noncanonical = b" " + self.raw
        with self.assertRaisesRegex(GovernanceError, "canonical"):
            parse_manifest(noncanonical, **{**self.expected, "manifest_sha256": sha256_bytes(noncanonical)})

    def test_create_only_conflict_and_interrupted_manifest(self):
        first = self.manifest["artifacts"][0]
        store = MemoryStore()
        store.objects[first["key"]] = b"conflicting existing object"
        with self.assertRaises(GovernanceError):
            seal(store, self.manifest, self.read)
        self.assertEqual(store.objects[first["key"]], b"conflicting existing object")
        self.assertNotIn(manifest_key(self.manifest), store.objects)
        for boundary in (0, 3, len(self.manifest["artifacts"])):
            store = MemoryStore()
            store.fail_after = boundary
            with self.subTest(interruption=boundary), self.assertRaises(GovernanceError):
                seal(store, self.manifest, self.read)
            self.assertNotIn(manifest_key(self.manifest), store.objects)
            store.fail_after = None
            self.assertEqual(seal(store, self.manifest, self.read), self.pin)
            before = dict(store.objects)
            with self.assertRaisesRegex(GovernanceError, "already sealed"):
                seal(store, self.manifest, self.read)
            self.assertEqual(store.objects, before)
        incomplete = copy.deepcopy(self.manifest)
        incomplete["artifacts"].pop()
        store = MemoryStore()
        with self.assertRaises(GovernanceError):
            seal(store, incomplete, self.read)
        self.assertFalse(store.objects)

    def test_independent_copy_requires_complete_verified_readback(self):
        primary, secondary = MemoryStore(), MemoryStore("test-memory:separate-copy")
        seal(primary, self.manifest, self.read)
        seal(secondary, self.manifest, self.read)
        key = manifest_key(self.manifest)
        before = primary.objects[key]
        row = self.manifest["artifacts"][0]
        secondary.objects.pop(row["key"])
        for locations in ([primary], [primary, primary], [primary, secondary]):
            with self.assertRaises(GovernanceError):
                attest_readback(locations, key=key, receipt_id="copy-001", kind="copy", **self.expected)
            self.assertFalse(any("/receipts/" in path for path in primary.objects))
        secondary.objects[row["key"]] = self.read(row)
        primary.reads.clear()
        secondary.reads.clear()
        receipt = attest_readback([primary, secondary], key=key, receipt_id="copy-001", kind="copy", **self.expected)
        self.assertEqual(receipt["assurance"], "complete-byte-readback-only")
        self.assertEqual(receipt["manifest_sha256"], self.pin)
        self.assertEqual(primary.objects[key], before)
        for store in (primary, secondary):
            self.assertTrue({record["key"] for record in self.manifest["artifacts"]} <= set(store.reads))
        with self.assertRaisesRegex(GovernanceError, "already exists"):
            attest_readback([primary, secondary], key=key, receipt_id="copy-001", kind="copy", **self.expected)
        attest_readback([secondary], key=key, receipt_id="read-002", kind="readback", **self.expected)
        self.assertEqual(primary.objects[key], before)
        secondary.objects[key] = b"changed manifest"
        with self.assertRaises(GovernanceError):
            attest_readback([primary, secondary], key=key, receipt_id="copy-003", kind="copy", **self.expected)

    def test_archive_read_does_not_relax_live_freshness(self):
        row = next(row for row in self.inventory["artifacts"] if row["role"] == "qualification-index")
        raw = self.payloads[row["path"]]
        index = json.loads(raw)
        now = datetime(2026, 9, 8, tzinfo=timezone.utc)
        with self.assertRaisesRegex(GovernanceError, "expired"):
            validate_qualification_artifact_index(index, require_unexpired=True, now=now)
        self.verify()
        seal(MemoryStore(), self.manifest, self.read)
        self.assertEqual(self.payloads[row["path"]], raw)
        with self.assertRaisesRegex(GovernanceError, "expired"):
            validate_qualification_artifact_index(index, require_unexpired=True, now=now)
        wrong_retention = copy.deepcopy(index)
        wrong_retention["workflow"]["retention_days"] = 365
        with self.assertRaises(GovernanceError):
            validate_qualification_artifact_index(wrong_retention)

    def test_unsafe_paths_symlinks_and_untrusted_manifest_digest(self):
        for path in ("/absolute", "../escape", "a/../b", "a//b", "./name", "C:/file", "a\\b", "a/./b", "a/", "a\x00b", "a/space "):
            inventory = copy.deepcopy(self.inventory)
            inventory["artifacts"][0]["path"] = path
            with self.subTest(path=path), self.assertRaises(GovernanceError):
                construct_manifest(inventory)
        for second in ("DATA", "data/file"):
            inventory = copy.deepcopy(self.inventory)
            inventory["artifacts"][0]["path"] = "data"
            inventory["artifacts"][1]["path"] = second
            with self.assertRaisesRegex(GovernanceError, "conflict"):
                construct_manifest(inventory)
        with self.assertRaisesRegex(GovernanceError, "untrusted"):
            parse_manifest(self.raw, **{**self.expected, "manifest_sha256": "b" * 64})
        with tempfile.TemporaryDirectory(prefix="sifr-archive-path-test-") as temporary:
            root = Path(temporary).resolve()
            self.write_payloads(root / "input")
            row = self.manifest["artifacts"][0]
            path = root / "input" / row["path"]
            path.unlink()
            (root / "outside").write_bytes(self.read(row))
            path.symlink_to(root / "outside")
            with self.assertRaises(GovernanceError):
                verify_local(self.raw, payload_root=root / "input", **self.expected)
            (root / "linked-root").symlink_to(root / "input", target_is_directory=True)
            with self.assertRaises(GovernanceError):
                read_local(root / "linked-root", row["path"])
            (root / "linked-parent").symlink_to(root, target_is_directory=True)
            with self.assertRaises(GovernanceError):
                read_local(root / "linked-parent" / "input", row["path"])


if __name__ == "__main__":
    unittest.main()

"""Archive inventory completeness and immutable cross-artifact identity links."""

from __future__ import annotations

from pathlib import PurePosixPath
from typing import Any, Callable

from .archive_manifest import SKIP_TARGETS, safe_path
from .artifact_index import EXPECTED_ARTIFACT_IDS, validate_qualification_artifact_index
from .common import (
    BUILDERS, TARGETS, GovernanceError, canonical_json_bytes,
    load_json_bytes_strict, require_object, require_sha256, sha256_bytes,
)
from .release_report import validate_release_profile_report

UPLOAD_JOBS = (*TARGETS, "editor", "assemble", "index")
SINGLETON_ROLES = {
    "qualification-index", "run-metadata", "upload-metadata", "release-profile-report",
    "documentation-result", "documentation-summary",
    "profile-manifest", "platform-policy", "support-claims",
    "candidate-plan", "release-notes", "review-evidence", "review-log",
}
REQUIRED_ROLES = (
    SINGLETON_ROLES | (EXPECTED_ARTIFACT_IDS - {f"sysroot-{target}" for target in TARGETS})
    | {f"structured-skip:{target}" for target in SKIP_TARGETS}
)


def equal(observed: Any, expected: Any, label: str) -> None:
    if observed != expected:
        raise GovernanceError(f"{label}: archive cross-link mismatch")


def validate_required_roles(manifest: dict[str, Any]) -> None:
    roles = {record["role"] for record in manifest["artifacts"]}
    missing = REQUIRED_ROLES - roles
    if missing:
        raise GovernanceError(f"missing required producer classes: {sorted(missing)}")
    # Later protected approval/publication needs a separately scoped additive manifest.
    for role in roles - REQUIRED_ROLES:
        if not role.startswith(("result:", "step-log:", "case-log:")) and role not in {
            f"workflow-log:{job}" for job in UPLOAD_JOBS
        }:
            raise GovernanceError(f"unregistered archive role: {role}")
    matrix = manifest["matrix"]
    equal([row["target"] for row in matrix], sorted((*TARGETS, *SKIP_TARGETS)), "complete platform matrix")
    for row in matrix:
        native = row["target"] in TARGETS
        equal(row["mode"], "native" if native else "structured-skip", "platform mode")
        if (native and row["reasons"]) or (not native and not row["reasons"]):
            raise GovernanceError("native rows cannot skip; host-limited rows require reasons")
        equal(row["reasons"], sorted(row["reasons"]), "ordered skip reasons")


def verify_bindings(manifest: dict[str, Any], read: Callable[[dict[str, Any]], bytes]) -> None:
    records = {record["role"]: record for record in manifest["artifacts"]}
    identity = manifest["identity"]

    def document(role: str) -> dict[str, Any]:
        return require_object(load_json_bytes_strict(read(records[role]), source=role), role)

    index_raw = read(records["qualification-index"])
    index = validate_qualification_artifact_index(load_json_bytes_strict(
        index_raw, source="qualification-index", require_canonical=True))
    equal(records["qualification-index"]["sha256"], identity["index_sha256"], "index digest")
    equal(index["source_commit"], identity["source_commit"], "index source")
    equal(index["submodules"], identity["submodules"], "index submodules")
    for key in ("repository", "run_id", "run_attempt"):
        equal(index["workflow"][key], identity[key], f"index {key}")
    run = document("run-metadata")
    for key, field in (("id", "run_id"), ("run_attempt", "run_attempt"),
                       ("head_sha", "source_commit"), ("path", "workflow_path")):
        equal(run.get(key), identity[field], f"run {key}")
    equal(run.get("repository", {}).get("full_name"), identity["repository"], "run repository")
    equal(run.get("event"), "workflow_dispatch", "run event")
    for artifact in index["artifacts"]:
        # Qualification staging sysroots keep their original indexed provenance;
        # shipped compiler archives already contain the installation sysroot.
        if artifact["id"] not in records:
            continue
        record = records[artifact["id"]]
        equal(record["sha256"], artifact["sha256"], f"{artifact['id']} digest")
        equal(record["size_bytes"], artifact["size_bytes"], f"{artifact['id']} size")
        equal(PurePosixPath(record["path"]).name, artifact["name"], "transported filename")
        equal(record["producer"].get("artifact_id"), artifact["workflow_artifact_id"], "upload identity")
    verify_uploads(index, records, document("upload-metadata"), identity)
    verify_report(records, document, identity)
    verify_platforms(manifest, records, document, identity)
    docs = document("documentation-summary")
    equal(docs.get("source_commit"), identity["source_commit"], "documentation source")
    equal(docs.get("result_sha256"), records["documentation-result"]["sha256"], "standalone documentation result")
    candidate = document("candidate-plan")
    equal(candidate.get("source_commit"), identity["source_commit"], "candidate source")
    equal(candidate.get("submodules"), identity["submodules"], "candidate submodules")
    equal(candidate.get("cargo_lock_sha256"), identity["lock_sha256"], "candidate lock")
    for field, role in (("release_profile_report", "release-profile-report"),
                        ("qualification_artifact_index", "qualification-index"),
                        ("documentation_report", "documentation-summary")):
        equal(candidate.get(field, {}).get("sha256"), records[role]["sha256"], f"candidate {field}")
    equal(candidate.get("rust_interop", {}).get("stable_support_claims_sha256"),
          records["support-claims"]["sha256"], "candidate support claims")
    equal(candidate.get("release_notes_sha256"), records["release-notes"]["sha256"], "candidate release notes")
    equal(candidate.get("toolchain"), {
        "rustc": identity["toolchain"]["rustc"], "cargo": identity["toolchain"]["cargo"],
        "profile_manifest_sha256": identity["profile_sha256"],
    }, "candidate toolchain")
    review = document("review-evidence")
    equal(review.get("source_commit"), identity["source_commit"], "review source")
    equal(review.get("log_sha256"), records["review-log"]["sha256"], "review log")
    # Archive preserves the actual review outcome, with no fabricated approval requirement.


def verify_uploads(index: dict[str, Any], records: dict[str, Any], metadata: dict[str, Any],
                   identity: dict[str, Any]) -> None:
    uploads = metadata.get("artifacts")
    if not isinstance(uploads, list) or len(uploads) != len(UPLOAD_JOBS):
        raise GovernanceError("must retain compact metadata for all seven producer uploads")
    ids: set[int] = set()
    by_job: dict[str, Any] = {}
    prefix = f"sifr-stable-candidate-{index['candidate_version']}-{identity['source_commit']}-"
    for upload in uploads:
        upload = require_object(upload, "upload")
        upload_id = upload.get("id")
        if type(upload_id) is not int or upload_id < 1 or upload_id in ids:
            raise GovernanceError("duplicate or invalid upload identity")
        ids.add(upload_id)
        name = upload.get("name")
        if not isinstance(name, str) or not name.startswith(prefix):
            raise GovernanceError("wrong upload source/version name")
        job = name[len(prefix):]
        if job not in UPLOAD_JOBS or job in by_job:
            raise GovernanceError("duplicate or unknown upload job")
        by_job[job] = upload
        equal(upload.get("workflow_run", {}).get("id"), identity["run_id"], "upload run")
        digest = upload.get("digest")
        if not isinstance(digest, str) or not digest.startswith("sha256:"):
            raise GovernanceError("invalid recorded upload digest")
        require_sha256(digest.removeprefix("sha256:"), "recorded upload digest")
        expected = {item["name"]: item for item in index["artifacts"]
                    if item["workflow_artifact_id"] == upload_id}
        if job == "index":
            expected = {"qualification-artifact-index.json": records["qualification-index"]}
            owner = records["qualification-index"]["producer"]
            equal(owner.get("artifact_id"), upload_id, "index upload identity")
            equal(owner["job"], job, "index producer job")
            equal(owner["execution"], "workflow", "index producer execution")
        if not expected:
            raise GovernanceError("upload does not bind any indexed payload")
        for artifact in index["artifacts"]:
            if artifact["workflow_artifact_id"] == upload_id:
                equal(artifact["workflow_artifact_name"], name, "index upload name")
                if artifact["id"] not in records:
                    continue
                equal(records[artifact["id"]]["producer"]["job"], job, "payload producer job")
                equal(records[artifact["id"]]["producer"]["execution"], "workflow", "payload execution")
        if (log := records.get(f"workflow-log:{job}")) is not None:
            equal(log["producer"]["job"], job, "workflow log job")
            equal(log["producer"]["execution"], "workflow", "workflow log execution")


def verify_report(records: dict[str, Any], document: Callable[[str], dict[str, Any]],
                  identity: dict[str, Any]) -> None:
    report = validate_release_profile_report(document("release-profile-report"),
                                             expected_profile_sha256=identity["profile_sha256"])
    equal(report["source"]["commit"], identity["source_commit"], "release report source")
    equal(report["source"]["submodules"], identity["submodules"], "release report submodules")
    equal(report["toolchain"], identity["toolchain"], "release report toolchain")
    profile = document("profile-manifest")
    equal(sha256_bytes(canonical_json_bytes(profile)), identity["profile_sha256"], "profile manifest digest")
    equal(profile.get("name"), "release", "profile name")
    expected: dict[str, set[str]] = {}
    selections = profile.get("selected_areas")
    if not isinstance(selections, list):
        raise GovernanceError("profile selected_areas must be an array")
    for selection in selections:
        if not isinstance(selection, dict) or not isinstance(selection.get("suites"), list):
            raise GovernanceError("invalid selected profile suites")
        expected.setdefault(selection["area"], set()).update(selection["suites"])
    if "full" in expected.get("developer_tooling", set()):
        expected["developer_tooling"].add("editor-release")
    equal({row["area"]: set(row["suites"]) for row in report["profile"]["expanded_selected_areas"]},
          expected, "complete expanded release profile")
    result_roles = set()
    for artifact in report["result_artifacts"]:
        role = "result:" + safe_path(artifact["path"])
        result_roles.add(role)
        if role not in records:
            raise GovernanceError(f"missing bound result: {role}")
        equal(records[role]["sha256"], artifact["sha256"], role)
    log_roles = set()
    observed: dict[str, set[str]] = {}
    for step in report["steps"]:
        log_roles.add("step-log:" + step["name"])
        for suite in step["suite_results"]:
            observed.setdefault(suite["area"], set()).add(suite["suite"])
            for case in suite["case_ids"]:
                log_roles.add(f"case-log:{suite['area']}:{suite['suite']}:{case}")
    equal(observed, expected, "every selected suite executed")
    equal({role for role in records if role.startswith("result:")}, result_roles, "exact bound results")
    retained_logs = {role for role in records if role.startswith(("step-log:", "case-log:"))}
    if not retained_logs <= log_roles:
        raise GovernanceError("retained step/case log has no recorded execution")


def verify_platforms(manifest: dict[str, Any], records: dict[str, Any],
                     document: Callable[[str], dict[str, Any]], identity: dict[str, Any]) -> None:
    policy = document("platform-policy")
    rows = policy.get("host_triples")
    if not isinstance(rows, list) or len(rows) != 6:
        raise GovernanceError("complete six-host platform policy required")
    equal(sorted(row.get("triple", "") for row in rows), sorted((*TARGETS, *SKIP_TARGETS)), "platform policy")
    for row in manifest["matrix"]:
        target = row["target"]
        declared = next(entry for entry in rows if entry["triple"] == target)
        equal(declared.get("status"), "supported" if target in TARGETS else "host-limited", "declared platform status")
        if target in TARGETS:
            report = document(f"qualification-report-{target}")
            for key, value in (("source_commit", identity["source_commit"]), ("target", target),
                               ("builder", BUILDERS[target]), ("smoke_status", "pass")):
                equal(report.get(key), value, f"target report {key}")
            for field, role in (("archive_sha256", "binary-archive"), ("checksum_sha256", "checksum")):
                equal(report.get(field), records[f"{role}-{target}"]["sha256"], f"target {field}")
            index = document("qualification-index")
            sysroot = next(item for item in index["artifacts"] if item["id"] == f"sysroot-{target}")
            equal(report.get("sysroot_bundle_sha256"), sysroot["sha256"], "staging sysroot provenance")
        else:
            equal(row["reasons"], sorted(declared.get("allowed_skips", [])), "declared skip reasons")
            skip = document(f"structured-skip:{target}")
            equal(skip, {"source_commit": identity["source_commit"], "target": target,
                         "status": "structured-skip", "reasons": row["reasons"]}, "structured skip evidence")
    editor = document("editor-qualification-report")
    equal(editor.get("source_commit"), identity["source_commit"], "editor source")
    equal(editor.get("vsix_sha256"), records["vsix"]["sha256"], "editor VSIX")

"""Read-only protected preparation for GA and normal stable publication."""

from __future__ import annotations

import re
import subprocess
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from .artifact_index import (
    EXPECTED_ARTIFACT_IDS,
    validate_qualification_artifact_index,
)
from .common import (
    GovernanceError,
    fail,
    load_json_strict,
    require_commit,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    sha256_file,
    version_channel,
)
from .planner import (
    bind_aggregate_artifacts,
    bind_target_reports,
    stable_claim_ids,
    validate_documentation_report,
    validate_rust_candidate_result,
    verify_transported_artifacts,
)
from .release_plan import validate_release_plan, validate_release_signoff
from .release_report import (
    canonical_profile_digest,
    validate_release_profile_report,
)
from .stable_planner import (
    materialize_stable_mutation,
    validate_stable_mutation_evidence,
)

CANDIDATE_PATH_RE = re.compile(r"^plans/releases/candidates/([0-9]+\.[0-9]+\.[0-9]+)$")
REQUIRED_CANDIDATE_FILES = {
    "stable-release-plan.json",
    "release-profile-report.json",
    "qualification-artifact-index.json",
    "stable-support-claims.json",
    "rust-validation-report.json",
    "documentation-report.json",
    "release-notes.md",
}
OPTIONAL_CANDIDATE_FILES = {"stable-release-signoff.json"}
MINIMUM_PUBLICATION_WINDOW = timedelta(days=7)


def materialize_stable_prepare(
    *,
    operation: str,
    mode: str,
    evidence_root: Path,
    evidence_commit: str,
    candidate_path: str,
    expected_plan_sha256: str,
    source_root: Path,
    live_index_path: Path,
    artifact_root: Path,
    proposed_generation: int,
    now: datetime | None = None,
) -> dict[str, Any]:
    """Revalidate exact reviewer-visible inputs and return a read-only summary."""
    operation = require_enum(
        operation,
        {"ga-activation", "normal"},
        "operation",
    )
    mode = require_enum(mode, {"initial", "resume"}, "mode")
    evidence_commit = require_commit(evidence_commit, "evidence_commit")
    require_sha256(expected_plan_sha256, "expected_plan_sha256")
    require_positive_int(proposed_generation, "proposed_generation")
    match = CANDIDATE_PATH_RE.fullmatch(candidate_path)
    if match is None:
        fail("candidate_path", "must be a normalized candidate evidence directory")

    evidence_root = evidence_root.resolve()
    source_root = source_root.resolve()
    _require_checkout(evidence_root, evidence_commit, "evidence")
    candidate_root = evidence_root / candidate_path
    if (
        candidate_root.is_symlink()
        or not candidate_root.is_dir()
        or not candidate_root.resolve().is_relative_to(evidence_root)
    ):
        fail("candidate_path", "must remain inside the evidence checkout")
    _require_candidate_files(candidate_root)

    plan_path = candidate_root / "stable-release-plan.json"
    report_path = candidate_root / "release-profile-report.json"
    qualification_path = candidate_root / "qualification-artifact-index.json"
    claims_path = candidate_root / "stable-support-claims.json"
    rust_report_path = candidate_root / "rust-validation-report.json"
    documentation_path = candidate_root / "documentation-report.json"
    release_notes_path = candidate_root / "release-notes.md"
    signoff_path = candidate_root / "stable-release-signoff.json"
    if sha256_file(plan_path) != expected_plan_sha256:
        fail("expected_plan_sha256", "does not match candidate evidence")

    plan = validate_release_plan(
        load_json_strict(plan_path, require_canonical=True),
        active_index=load_json_strict(live_index_path, require_canonical=True),
    )
    if plan["transition"] != operation or plan["version"] != match.group(1):
        fail("candidate_path", "does not match the requested plan operation/version")
    if signoff_path.is_file():
        signoff = validate_release_signoff(
            load_json_strict(signoff_path, require_canonical=True)
        )
        if (
            signoff["version"] != plan["version"]
            or signoff["plan_sha256"] != expected_plan_sha256
        ):
            fail("stable-release-signoff.json", "does not bind the candidate plan")
    _require_checkout(source_root, plan["source_commit"], "source")
    _validate_source_contracts(plan, source_root)

    report_bytes = report_path.read_bytes()
    profile_sha256 = canonical_profile_digest(
        source_root / "verification/profiles/release.json"
    )
    report = validate_release_profile_report(
        load_json_strict(report_path, require_canonical=True),
        canonical_bytes=report_bytes,
        source_root=source_root,
        expected_profile_sha256=profile_sha256,
    )
    _require_reference(
        plan["release_profile_report"],
        identifier=report["report_id"],
        path=report_path,
        location="$.release_profile_report",
    )
    if (
        report["source"]["commit"] != plan["source_commit"]
        or report["source"]["submodules"] != plan["submodules"]
    ):
        fail("$.release_profile_report", "does not match the candidate source")

    current_time = now or datetime.now(timezone.utc)
    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_path, require_canonical=True),
        require_unexpired=True,
        now=current_time,
    )
    qualification_id = (
        f"qualification-{qualification['workflow']['run_id']}-"
        f"{qualification['workflow']['run_attempt']}"
    )
    _require_reference(
        plan["qualification_artifact_index"],
        identifier=qualification_id,
        path=qualification_path,
        location="$.qualification_artifact_index",
    )
    if (
        qualification["candidate_version"] != plan["version"]
        or qualification["source_commit"] != plan["source_commit"]
        or qualification["submodules"] != plan["submodules"]
    ):
        fail("$.qualification_artifact_index", "candidate provenance drifted")
    expires_at = _require_publication_window(qualification, now=current_time)

    artifact_paths = verify_transported_artifacts(qualification, artifact_root)
    bind_target_reports(plan, qualification, artifact_paths)
    bind_aggregate_artifacts(
        plan,
        qualification,
        artifact_paths,
        source_root=source_root,
    )
    marketplace = _validate_supporting_evidence(
        plan=plan,
        report=report,
        claims_path=claims_path,
        rust_report_path=rust_report_path,
        documentation_path=documentation_path,
        release_notes_path=release_notes_path,
        editor_report_path=artifact_paths["editor-qualification-report"],
    )

    live_sha256 = sha256_file(live_index_path)
    live_index = load_json_strict(live_index_path, require_canonical=True)
    mutation = materialize_stable_mutation(
        plan_path=plan_path,
        live_index_path=live_index_path,
        expected_generation=live_index["generation"],
        expected_sha256=live_sha256,
        proposed_generation=proposed_generation,
    )
    mutation_evidence = mutation.evidence()
    validate_stable_mutation_evidence(mutation_evidence)
    summary = {
        "schema_version": 2,
        "operation": operation,
        "mode": mode,
        "version": plan["version"],
        "evidence": {
            "commit": evidence_commit,
            "candidate_path": candidate_path,
            "plan_sha256": expected_plan_sha256,
        },
        "source": {
            "commit": plan["source_commit"],
            "submodules": plan["submodules"],
        },
        "release_report": {
            "id": report["report_id"],
            "sha256": sha256_file(report_path),
        },
        "qualification": {
            "id": qualification_id,
            "sha256": sha256_file(qualification_path),
            "run_id": qualification["workflow"]["run_id"],
            "run_attempt": qualification["workflow"]["run_attempt"],
            "expires_at": expires_at,
        },
        "live_index": {
            "generation": live_index["generation"],
            "sha256": live_sha256,
        },
        "mutation": mutation_evidence,
        "artifacts": {
            artifact["id"]: {
                "name": artifact["name"],
                "sha256": artifact["sha256"],
                "size_bytes": artifact["size_bytes"],
                "workflow_artifact_id": artifact["workflow_artifact_id"],
                "workflow_artifact_name": artifact["workflow_artifact_name"],
            }
            for artifact in qualification["artifacts"]
        },
        "marketplace": marketplace,
        "site": {
            "repository": plan["site"]["repository"],
            "base_commit": plan["site"]["base_commit"],
        },
    }
    validate_stable_prepare_summary(summary)
    return summary


def validate_stable_prepare_summary(payload: object) -> dict[str, Any]:
    """Validate the protected reviewer-visible summary contract."""
    summary = require_object(payload, "$")
    require_exact_keys(
        summary,
        required={
            "schema_version",
            "operation",
            "mode",
            "version",
            "evidence",
            "source",
            "release_report",
            "qualification",
            "live_index",
            "mutation",
            "artifacts",
            "marketplace",
            "site",
        },
        location="$",
    )
    require_schema_v2(summary)
    operation = require_enum(
        summary["operation"],
        {"ga-activation", "normal"},
        "$.operation",
    )
    require_enum(summary["mode"], {"initial", "resume"}, "$.mode")
    version = summary["version"]
    if version_channel(version, "$.version") != "stable":
        fail("$.version", "must be an exact stable version")
    evidence = require_object(summary["evidence"], "$.evidence")
    require_exact_keys(
        evidence,
        required={"commit", "candidate_path", "plan_sha256"},
        location="$.evidence",
    )
    require_commit(evidence["commit"], "$.evidence.commit")
    require_sha256(evidence["plan_sha256"], "$.evidence.plan_sha256")
    match = CANDIDATE_PATH_RE.fullmatch(
        require_nonempty_string(evidence["candidate_path"], "$.evidence.candidate_path")
    )
    if match is None or match.group(1) != version:
        fail("$.evidence.candidate_path", "does not bind the summary version")
    source = require_object(summary["source"], "$.source")
    require_exact_keys(
        source,
        required={"commit", "submodules"},
        location="$.source",
    )
    source_commit = require_commit(source["commit"], "$.source.commit")
    submodules = require_object(source["submodules"], "$.source.submodules")
    if not submodules:
        fail("$.source.submodules", "must contain recursive submodule identities")
    for path, commit in submodules.items():
        require_nonempty_string(path, "$.source.submodules key")
        require_commit(commit, f"$.source.submodules.{path}")

    _validate_reference(summary["release_report"], "$.release_report")
    qualification = require_object(summary["qualification"], "$.qualification")
    require_exact_keys(
        qualification,
        required={"id", "sha256", "run_id", "run_attempt", "expires_at"},
        location="$.qualification",
    )
    qualification_id = require_nonempty_string(
        qualification["id"],
        "$.qualification.id",
    )
    require_sha256(qualification["sha256"], "$.qualification.sha256")
    run_id = require_positive_int(
        qualification["run_id"],
        "$.qualification.run_id",
    )
    run_attempt = require_positive_int(
        qualification["run_attempt"],
        "$.qualification.run_attempt",
    )
    if qualification_id != f"qualification-{run_id}-{run_attempt}":
        fail("$.qualification.id", "does not bind the workflow run identity")
    expires_at = require_nonempty_string(
        qualification["expires_at"],
        "$.qualification.expires_at",
    )
    try:
        expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
    except ValueError:
        fail("$.qualification.expires_at", "must be an ISO-8601 timestamp")
    if expiry.tzinfo is None:
        fail("$.qualification.expires_at", "must include a timezone")

    mutation = validate_stable_mutation_evidence(summary["mutation"])
    if (
        mutation["transition"] != operation
        or mutation["version"] != version
        or mutation["plan_sha256"] != evidence["plan_sha256"]
    ):
        fail("$.mutation", "does not bind the requested operation and plan")
    release = mutation["proposed_index"]["releases"].get(version)
    if not isinstance(release, dict) or release.get("source_commit") != source_commit:
        fail("$.source.commit", "does not equal the proposed release source")

    live = require_object(summary["live_index"], "$.live_index")
    require_exact_keys(
        live,
        required={"generation", "sha256"},
        location="$.live_index",
    )
    require_positive_int(live["generation"], "$.live_index.generation")
    require_sha256(live["sha256"], "$.live_index.sha256")
    if (
        mutation["previous_index"]["generation"] != live["generation"]
        or mutation["previous_index"]["sha256"] != live["sha256"]
    ):
        fail("$.live_index", "does not equal the mutation predecessor")

    artifacts = require_object(summary["artifacts"], "$.artifacts")
    if set(artifacts) != EXPECTED_ARTIFACT_IDS:
        fail("$.artifacts", "must contain exact governed artifact identifiers")
    workflow_id_to_name: dict[int, str] = {}
    workflow_name_to_id: dict[str, int] = {}
    for artifact_id, value in artifacts.items():
        artifact = require_object(value, f"$.artifacts.{artifact_id}")
        require_exact_keys(
            artifact,
            required={
                "name",
                "sha256",
                "size_bytes",
                "workflow_artifact_id",
                "workflow_artifact_name",
            },
            location=f"$.artifacts.{artifact_id}",
        )
        require_nonempty_string(
            artifact["name"],
            f"$.artifacts.{artifact_id}.name",
        )
        require_sha256(
            artifact["sha256"],
            f"$.artifacts.{artifact_id}.sha256",
        )
        workflow_artifact_id = require_positive_int(
            artifact["workflow_artifact_id"],
            f"$.artifacts.{artifact_id}.workflow_artifact_id",
        )
        workflow_name = require_nonempty_string(
            artifact["workflow_artifact_name"],
            f"$.artifacts.{artifact_id}.workflow_artifact_name",
        )
        require_positive_int(
            artifact["size_bytes"],
            f"$.artifacts.{artifact_id}.size_bytes",
        )
        if (
            workflow_id_to_name.setdefault(workflow_artifact_id, workflow_name)
            != workflow_name
            or workflow_name_to_id.setdefault(workflow_name, workflow_artifact_id)
            != workflow_artifact_id
        ):
            fail(
                f"$.artifacts.{artifact_id}",
                "workflow artifact id/name mapping must be one-to-one",
            )
    if len(workflow_id_to_name) != 6:
        fail("$.artifacts", "must bind the six governed transported uploads")

    marketplace = require_object(summary["marketplace"], "$.marketplace")
    require_exact_keys(
        marketplace,
        required={"publisher", "extension", "version", "vsix_sha256"},
        location="$.marketplace",
    )
    for key in ("publisher", "extension", "version"):
        require_nonempty_string(marketplace[key], f"$.marketplace.{key}")
    vsix_sha256 = require_sha256(
        marketplace["vsix_sha256"],
        "$.marketplace.vsix_sha256",
    )
    if vsix_sha256 != artifacts["vsix"]["sha256"]:
        fail("$.marketplace.vsix_sha256", "does not match the transported VSIX")

    site = require_object(summary["site"], "$.site")
    require_exact_keys(
        site,
        required={"repository", "base_commit"},
        location="$.site",
    )
    if site["repository"] != "sifr-lang/sifr-website":
        fail("$.site.repository", "must be sifr-lang/sifr-website")
    require_commit(site["base_commit"], "$.site.base_commit")
    return summary


def _require_checkout(root: Path, expected_commit: str, label: str) -> None:
    if _git(root, "rev-parse", "HEAD") != expected_commit:
        fail(label, "checkout does not equal the expected commit")
    if _git(root, "status", "--porcelain", "--untracked-files=all"):
        fail(label, "checkout must be clean")
    if _git(root, "diff", "--name-only", "--diff-filter=U"):
        fail(label, "checkout has unresolved files")


def _require_candidate_files(candidate_root: Path) -> None:
    entries = list(candidate_root.iterdir())
    unsafe = sorted(
        path.name for path in entries if path.is_symlink() or not path.is_file()
    )
    if unsafe:
        fail(
            "candidate_path",
            f"contains unsupported entry/entries: {', '.join(unsafe)}",
        )
    files = {path.name for path in entries}
    missing = sorted(REQUIRED_CANDIDATE_FILES.difference(files))
    unknown = sorted(
        files.difference(REQUIRED_CANDIDATE_FILES | OPTIONAL_CANDIDATE_FILES)
    )
    if missing:
        fail("candidate_path", f"missing evidence file(s): {', '.join(missing)}")
    if unknown:
        fail(
            "candidate_path",
            f"contains unsupported evidence file(s): {', '.join(unknown)}",
        )


def _validate_source_contracts(plan: dict[str, Any], source_root: Path) -> None:
    if sha256_file(source_root / "Cargo.lock") != plan["cargo_lock_sha256"]:
        fail("$.cargo_lock_sha256", "does not match the source checkout")
    profile_sha256 = canonical_profile_digest(
        source_root / "verification/profiles/release.json"
    )
    if profile_sha256 != plan["toolchain"]["profile_manifest_sha256"]:
        fail("$.toolchain.profile_manifest_sha256", "does not match the source")
    for key, relative in (
        (
            "compatibility_matrix_sha256",
            "verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json",
        ),
        (
            "facts_schema_sha256",
            "verification/areas/distribution_release/schemas/stable_site_release_facts.schema.json",
        ),
        (
            "facts_generator_sha256",
            "verification/areas/distribution_release/governance/release_plan.py",
        ),
    ):
        expected = (
            plan["rust_interop"][key]
            if key == "compatibility_matrix_sha256"
            else plan["site"][key]
        )
        if sha256_file(source_root / relative) != expected:
            fail(f"$.{key}", "does not match the source checkout")
    for tool in ("rustc", "cargo"):
        if _command(source_root, tool, "--version") != plan["toolchain"][tool]:
            fail(f"$.toolchain.{tool}", "does not match the source checkout")


def _validate_supporting_evidence(
    *,
    plan: dict[str, Any],
    report: dict[str, Any],
    claims_path: Path,
    rust_report_path: Path,
    documentation_path: Path,
    release_notes_path: Path,
    editor_report_path: Path,
) -> dict[str, str]:
    if sha256_file(claims_path) != plan["rust_interop"]["stable_support_claims_sha256"]:
        fail("$.rust_interop.stable_support_claims_sha256", "evidence digest drifted")
    claims = load_json_strict(claims_path, require_canonical=True)
    if stable_claim_ids(claims) != plan["rust_interop"]["advertised_claim_ids"]:
        fail("$.rust_interop.advertised_claim_ids", "does not match candidate evidence")
    if (
        sha256_file(rust_report_path)
        != plan["rust_interop"]["validation_report_sha256"]
    ):
        fail("$.rust_interop.validation_report_sha256", "evidence digest drifted")
    validate_rust_candidate_result(
        load_json_strict(rust_report_path, require_canonical=True),
        expected_digest=plan["rust_interop"]["validation_report_sha256"],
        release_report=report,
    )
    if sha256_file(documentation_path) != plan["documentation_report"]["sha256"]:
        fail("$.documentation_report.sha256", "evidence digest drifted")
    documentation = validate_documentation_report(
        load_json_strict(documentation_path, require_canonical=True),
        source_commit=plan["source_commit"],
    )
    if documentation["report_id"] != plan["documentation_report"]["id"]:
        fail("$.documentation_report.id", "does not match candidate evidence")
    if sha256_file(release_notes_path) != plan["release_notes_sha256"]:
        fail("$.release_notes_sha256", "evidence digest drifted")
    editor = load_json_strict(editor_report_path, require_canonical=True)
    marketplace = require_object(
        editor["marketplace_publish_plan"],
        "$.marketplace_publish_plan",
    )
    return {
        "publisher": marketplace["publisher"],
        "extension": marketplace["extension"],
        "version": marketplace["version"],
        "vsix_sha256": marketplace["vsix_sha256"],
    }


def _require_publication_window(
    qualification: dict[str, Any],
    *,
    now: datetime,
) -> str:
    expires_at = qualification["workflow"]["expires_at"]
    expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
    if now.tzinfo is None:
        fail("now", "must include a timezone")
    if expiry - now < MINIMUM_PUBLICATION_WINDOW:
        fail("$.qualification.expires_at", "must leave at least seven full days")
    return expires_at


def _require_reference(
    payload: object,
    *,
    identifier: str,
    path: Path,
    location: str,
) -> None:
    reference = require_object(payload, location)
    if reference["id"] != identifier or reference["sha256"] != sha256_file(path):
        fail(location, "does not match the exact candidate evidence")


def _validate_reference(payload: object, location: str) -> dict[str, Any]:
    reference = require_object(payload, location)
    require_exact_keys(
        reference,
        required={"id", "sha256"},
        location=location,
    )
    require_nonempty_string(reference["id"], f"{location}.id")
    require_sha256(reference["sha256"], f"{location}.sha256")
    return reference


def _git(root: Path, *args: str) -> str:
    return _command(root, "git", *args)


def _command(root: Path, *args: str) -> str:
    result = subprocess.run(
        list(args),
        cwd=root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(
            f"{' '.join(args)} failed in {root}: {result.stderr.strip()}"
        )
    return result.stdout.rstrip("\r\n")

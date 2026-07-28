"""Read-only protected preparation for stable incident publication."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path
from typing import Any

from .common import (
    GovernanceError,
    canonical_json_bytes,
    fail,
    load_json_strict,
    require_commit,
    require_enum,
    require_exact_keys,
    require_incident_id,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    sha256_bytes,
    sha256_file,
    version_channel,
)
from .generation import SNAPSHOT_RE, allocate_next_generation
from .incident import validate_incident_request
from .incident_evidence import validate_incident_evidence_commit
from .incident_planner import IncidentMutation, materialize_incident_mutation
from .release_index import validate_release_index
from .release_plan import validate_release_plan
from .stable_prepare import (
    materialize_stable_prepare,
    validate_stable_prepare_summary,
)

INCIDENT_PATH_RE = re.compile(
    r"^plans/releases/incidents/([a-z0-9][a-z0-9-]{2,63})/"
    r"stable-incident-request[.]json$"
)


def materialize_incident_prepare(
    *,
    operation: str,
    mode: str,
    governance_root: Path,
    incident_root: Path,
    incident_commit: str,
    incident_path: str,
    expected_request_sha256: str,
    live_index_path: Path,
    snapshot_root: Path,
    proposed_generation: int,
    candidate_root: Path | None = None,
    candidate_commit: str = "",
    candidate_path: str = "",
    expected_plan_sha256: str = "",
    source_root: Path | None = None,
    artifact_root: Path | None = None,
) -> dict[str, Any]:
    """Revalidate an approved incident and return reviewer-visible intent."""
    operation = require_enum(
        operation,
        {"rollback", "incident-roll-forward"},
        "operation",
    )
    mode = require_enum(mode, {"initial", "resume"}, "mode")
    incident_commit = require_commit(incident_commit, "incident_commit")
    require_sha256(expected_request_sha256, "expected_request_sha256")
    require_positive_int(proposed_generation, "proposed_generation")
    match = INCIDENT_PATH_RE.fullmatch(incident_path)
    if match is None:
        fail("incident_path", "must be a normalized incident request path")

    governance_root = governance_root.resolve()
    incident_root = incident_root.resolve()
    _require_clean_checkout(governance_root, "governance")
    _require_checkout(incident_root, incident_commit, "incident")
    request_path = incident_root / incident_path
    evidence_path = request_path.with_name("withdrawal-evidence.txt")
    if (
        request_path.is_symlink()
        or evidence_path.is_symlink()
        or not request_path.is_file()
        or not evidence_path.is_file()
        or not request_path.resolve().is_relative_to(incident_root)
        or not evidence_path.resolve().is_relative_to(incident_root)
    ):
        fail("incident_path", "must bind regular files inside the incident checkout")
    _require_head_file(incident_root, request_path, "incident request")
    _require_head_file(incident_root, evidence_path, "withdrawal evidence")
    if sha256_file(request_path) != expected_request_sha256:
        fail("expected_request_sha256", "does not match incident evidence")

    parent = _git(incident_root, "rev-parse", f"{incident_commit}^")
    validate_incident_evidence_commit(
        repository=incident_root,
        base=parent,
        head=incident_commit,
        request_path=incident_path,
        evidence_path=str(Path(incident_path).with_name("withdrawal-evidence.txt")),
    )
    request = validate_incident_request(
        load_json_strict(request_path, require_canonical=True)
    )
    if request["operation"] != operation or request["incident_id"] != match.group(1):
        fail("incident_path", "does not match the requested incident operation/id")

    affected_version = request["affected_release"]["version"]
    affected_plan_path = _canonical_plan_path(governance_root, affected_version)
    if sha256_file(affected_plan_path) != request["affected_release"]["plan_sha256"]:
        fail("$.affected_release.plan_sha256", "does not match protected main evidence")
    affected_plan = validate_release_plan(
        load_json_strict(affected_plan_path, require_canonical=True)
    )
    live_sha256 = sha256_file(live_index_path)
    live_index = load_json_strict(live_index_path, require_canonical=True)
    next_generation = allocate_next_generation(
        live_index_path=live_index_path,
        snapshot_root=snapshot_root,
    )
    if proposed_generation != next_generation:
        fail("proposed_generation", "does not equal the next retained generation")

    release_prepare: dict[str, Any] | str
    if operation == "rollback":
        if any(
            (
                candidate_root is not None,
                bool(candidate_commit),
                bool(candidate_path),
                bool(expected_plan_sha256),
                source_root is not None,
                artifact_root is not None,
            )
        ):
            fail("rollback", "must not receive successor qualification inputs")
        target = request["rollback_target"]
        successor_version = target["version"]
        successor_plan_path = _canonical_plan_path(
            governance_root,
            successor_version,
        )
        if sha256_file(successor_plan_path) != target["plan_sha256"]:
            fail("$.rollback_target.plan_sha256", "does not match protected main evidence")
        successor_plan = validate_release_plan(
            load_json_strict(successor_plan_path, require_canonical=True)
        )
        publication_state, mutation = _materialize_or_recover(
            mode=mode,
            request_path=request_path,
            live_index_path=live_index_path,
            snapshot_root=snapshot_root,
            affected_plan_path=affected_plan_path,
            successor_plan_path=successor_plan_path,
            proposed_generation=proposed_generation,
        )
        mutation_evidence = _mutation_evidence(mutation)
        # Rollback uses the affected plan's newer, approved site commit and
        # dispatcher contract; the generated facts derive the active target
        # and withdrawal from the proposed index, never from site prose.
        site = {
            "repository": affected_plan["site"]["repository"],
            "base_commit": affected_plan["site"]["base_commit"],
        }
        release_prepare = "none"
    else:
        if (
            candidate_root is None
            or source_root is None
            or artifact_root is None
            or not candidate_commit
            or not candidate_path
            or not expected_plan_sha256
        ):
            fail(
                "incident-roll-forward",
                "requires exact successor candidate, source, and artifact inputs",
            )
        release_prepare = materialize_stable_prepare(
            operation=operation,
            mode=mode,
            evidence_root=candidate_root,
            evidence_commit=candidate_commit,
            candidate_path=candidate_path,
            expected_plan_sha256=expected_plan_sha256,
            source_root=source_root,
            live_index_path=live_index_path,
            snapshot_root=snapshot_root,
            artifact_root=artifact_root,
            proposed_generation=proposed_generation,
            incident_request_path=request_path,
            affected_plan_path=affected_plan_path,
        )
        publication_state = release_prepare["publication_state"]
        successor_version = release_prepare["version"]
        mutation_evidence = _mutation_evidence_from_stable(
            release_prepare["mutation"],
            request_sha256=expected_request_sha256,
            affected_version=affected_version,
            affected_plan_sha256=sha256_file(affected_plan_path),
        )
        site = release_prepare["site"]

    summary = {
        "schema_version": 2,
        "operation": operation,
        "mode": mode,
        "publication_state": publication_state,
        "next_generation": next_generation,
        "incident": {
            "commit": incident_commit,
            "path": incident_path,
            "incident_id": request["incident_id"],
            "request_sha256": expected_request_sha256,
            "withdrawal_evidence_sha256": sha256_file(evidence_path),
        },
        "affected": {
            "version": affected_version,
            "plan_sha256": sha256_file(affected_plan_path),
        },
        "successor": {
            "version": successor_version,
            "plan_sha256": mutation_evidence["plan_sha256"],
        },
        "live_index": {
            "generation": live_index["generation"],
            "sha256": live_sha256,
        },
        "mutation": mutation_evidence,
        "site": site,
        "release_prepare": release_prepare,
    }
    return validate_incident_prepare_summary(summary)


def validate_incident_prepare_summary(payload: object) -> dict[str, Any]:
    """Validate protected incident prepare bytes."""
    summary = require_object(payload, "$")
    require_exact_keys(
        summary,
        required={
            "schema_version",
            "operation",
            "mode",
            "publication_state",
            "next_generation",
            "incident",
            "affected",
            "successor",
            "live_index",
            "mutation",
            "site",
            "release_prepare",
        },
        location="$",
    )
    require_schema_v2(summary)
    operation = require_enum(
        summary["operation"],
        {"rollback", "incident-roll-forward"},
        "$.operation",
    )
    mode = require_enum(summary["mode"], {"initial", "resume"}, "$.mode")
    publication_state = require_enum(
        summary["publication_state"],
        {"pending", "activated"},
        "$.publication_state",
    )
    next_generation = require_positive_int(
        summary["next_generation"],
        "$.next_generation",
    )
    if publication_state == "activated" and mode != "resume":
        fail("$.publication_state", "activated state requires resume mode")

    incident = require_object(summary["incident"], "$.incident")
    require_exact_keys(
        incident,
        required={
            "commit",
            "path",
            "incident_id",
            "request_sha256",
            "withdrawal_evidence_sha256",
        },
        location="$.incident",
    )
    require_commit(incident["commit"], "$.incident.commit")
    require_incident_id(incident["incident_id"], "$.incident.incident_id")
    match = INCIDENT_PATH_RE.fullmatch(
        require_nonempty_string(incident["path"], "$.incident.path")
    )
    if match is None or match.group(1) != incident["incident_id"]:
        fail("$.incident.path", "does not bind incident_id")
    require_sha256(incident["request_sha256"], "$.incident.request_sha256")
    require_sha256(
        incident["withdrawal_evidence_sha256"],
        "$.incident.withdrawal_evidence_sha256",
    )
    affected = _validate_plan_ref(summary["affected"], "$.affected")
    successor = _validate_plan_ref(summary["successor"], "$.successor")
    if affected["version"] == successor["version"]:
        fail("$.successor.version", "must differ from the affected release")

    live = require_object(summary["live_index"], "$.live_index")
    require_exact_keys(
        live,
        required={"generation", "sha256"},
        location="$.live_index",
    )
    require_positive_int(live["generation"], "$.live_index.generation")
    require_sha256(live["sha256"], "$.live_index.sha256")
    mutation = validate_incident_mutation_evidence(summary["mutation"])
    if (
        mutation["operation"] != operation
        or mutation["request_sha256"] != incident["request_sha256"]
        or mutation["affected_plan_sha256"] != affected["plan_sha256"]
        or mutation["successor_plan_sha256"] != successor["plan_sha256"]
        or mutation["affected_version"] != affected["version"]
        or mutation["successor_version"] != successor["version"]
    ):
        fail("$.mutation", "does not bind the approved incident and plans")
    proposed_generation = mutation["proposed_index"]["generation"]
    if publication_state == "pending":
        if (
            mutation["previous_index"]["generation"] != live["generation"]
            or mutation["previous_index"]["sha256"] != live["sha256"]
            or proposed_generation != next_generation
        ):
            fail("$.mutation", "pending mutation does not bind the live lease")
    elif (
        proposed_generation != live["generation"]
        or mutation["proposed_index_sha256"] != live["sha256"]
        or next_generation <= proposed_generation
    ):
        fail("$.mutation", "activated mutation does not bind the realized index")

    site = require_object(summary["site"], "$.site")
    require_exact_keys(
        site,
        required={"repository", "base_commit"},
        location="$.site",
    )
    if site["repository"] != "sifr-lang/sifr-website":
        fail("$.site.repository", "must be sifr-lang/sifr-website")
    require_commit(site["base_commit"], "$.site.base_commit")
    if operation == "rollback":
        if summary["release_prepare"] != "none":
            fail("$.release_prepare", "rollback must not publish a successor release")
    else:
        release = validate_stable_prepare_summary(summary["release_prepare"])
        if (
            release["operation"] != operation
            or release["publication_state"] != publication_state
            or release["mutation"]["proposed_index"]
            != mutation["proposed_index"]
            or release["mutation"]["proposed_index_sha256"]
            != mutation["proposed_index_sha256"]
            or release["version"] != successor["version"]
            or release["site"] != site
        ):
            fail("$.release_prepare", "does not equal the successor publication intent")
    return summary


def _materialize_or_recover(
    *,
    mode: str,
    request_path: Path,
    live_index_path: Path,
    snapshot_root: Path,
    affected_plan_path: Path,
    successor_plan_path: Path,
    proposed_generation: int,
) -> tuple[str, IncidentMutation]:
    live = load_json_strict(live_index_path, require_canonical=True)
    try:
        mutation = materialize_incident_mutation(
            request_path=request_path,
            live_index_path=live_index_path,
            affected_plan_path=affected_plan_path,
            successor_plan_path=successor_plan_path,
            expected_generation=live["generation"],
            expected_sha256=sha256_file(live_index_path),
            proposed_generation=proposed_generation,
        )
    except GovernanceError:
        if mode != "resume":
            raise
        mutation = _recover_realized_mutation(
            request_path=request_path,
            affected_plan_path=affected_plan_path,
            successor_plan_path=successor_plan_path,
            live_index_path=live_index_path,
            snapshot_root=snapshot_root,
        )
        return "activated", mutation
    return "pending", mutation


def _recover_realized_mutation(
    *,
    request_path: Path,
    affected_plan_path: Path,
    successor_plan_path: Path,
    live_index_path: Path,
    snapshot_root: Path,
) -> IncidentMutation:
    live = load_json_strict(live_index_path, require_canonical=True)
    candidates: list[tuple[int, Path]] = []
    for path in snapshot_root.iterdir():
        match = SNAPSHOT_RE.fullmatch(path.name)
        if match is not None:
            generation = int(match.group(1))
            if generation < live["generation"]:
                candidates.append((generation, path))
    for generation, path in sorted(candidates, reverse=True):
        try:
            mutation = materialize_incident_mutation(
                request_path=request_path,
                live_index_path=path,
                affected_plan_path=affected_plan_path,
                successor_plan_path=successor_plan_path,
                expected_generation=generation,
                expected_sha256=sha256_file(path),
                proposed_generation=live["generation"],
            )
        except GovernanceError:
            continue
        if canonical_json_bytes(mutation.proposed_index) == live_index_path.read_bytes():
            return mutation
    fail("snapshot_root", "does not contain the realized incident predecessor")


def _mutation_evidence(mutation: IncidentMutation) -> dict[str, Any]:
    return {
        "schema_version": 2,
        "operation": mutation.operation,
        "request_sha256": mutation.request_sha256,
        "affected_plan_sha256": mutation.affected_plan_sha256,
        "successor_plan_sha256": mutation.successor_plan_sha256,
        "affected_version": mutation.affected_version,
        "successor_version": mutation.successor_version,
        "previous_index": {
            "generation": mutation.previous_index["generation"],
            "sha256": sha256_bytes(canonical_json_bytes(mutation.previous_index)),
        },
        "proposed_index": mutation.proposed_index,
        "proposed_index_sha256": sha256_bytes(
            canonical_json_bytes(mutation.proposed_index)
        ),
        "plan_sha256": mutation.successor_plan_sha256,
    }


def _mutation_evidence_from_stable(
    mutation: dict[str, Any],
    *,
    request_sha256: str,
    affected_version: str,
    affected_plan_sha256: str,
) -> dict[str, Any]:
    return {
        "schema_version": 2,
        "operation": "incident-roll-forward",
        "request_sha256": request_sha256,
        "affected_plan_sha256": affected_plan_sha256,
        "successor_plan_sha256": mutation["plan_sha256"],
        "affected_version": affected_version,
        "successor_version": mutation["version"],
        "previous_index": mutation["previous_index"],
        "proposed_index": mutation["proposed_index"],
        "proposed_index_sha256": mutation["proposed_index_sha256"],
        "plan_sha256": mutation["plan_sha256"],
    }


def validate_incident_mutation_evidence(payload: object) -> dict[str, Any]:
    """Validate incident mutation evidence independently of its prepare wrapper."""
    evidence = require_object(payload, "$.mutation")
    require_exact_keys(
        evidence,
        required={
            "schema_version",
            "operation",
            "request_sha256",
            "affected_plan_sha256",
            "successor_plan_sha256",
            "affected_version",
            "successor_version",
            "previous_index",
            "proposed_index",
            "proposed_index_sha256",
            "plan_sha256",
        },
        location="$.mutation",
    )
    require_schema_v2(evidence, "$.mutation")
    require_enum(
        evidence["operation"],
        {"rollback", "incident-roll-forward"},
        "$.mutation.operation",
    )
    for name in (
        "request_sha256",
        "affected_plan_sha256",
        "successor_plan_sha256",
        "proposed_index_sha256",
        "plan_sha256",
    ):
        require_sha256(evidence[name], f"$.mutation.{name}")
    if evidence["plan_sha256"] != evidence["successor_plan_sha256"]:
        fail("$.mutation.plan_sha256", "must equal successor_plan_sha256")
    for name in ("affected_version", "successor_version"):
        if version_channel(evidence[name], f"$.mutation.{name}") != "stable":
            fail(f"$.mutation.{name}", "must be an exact stable version")
    previous = require_object(evidence["previous_index"], "$.mutation.previous_index")
    require_exact_keys(
        previous,
        required={"generation", "sha256"},
        location="$.mutation.previous_index",
    )
    previous_generation = require_positive_int(
        previous["generation"],
        "$.mutation.previous_index.generation",
    )
    require_sha256(previous["sha256"], "$.mutation.previous_index.sha256")
    proposed = validate_release_index(evidence["proposed_index"])
    proposed_generation = require_positive_int(
        proposed.get("generation"),
        "$.mutation.proposed_index.generation",
    )
    if proposed_generation <= previous_generation:
        fail("$.mutation.proposed_index.generation", "must follow predecessor")
    if sha256_bytes(canonical_json_bytes(proposed)) != evidence["proposed_index_sha256"]:
        fail("$.mutation.proposed_index_sha256", "does not match proposed bytes")
    return evidence


def _validate_plan_ref(payload: object, location: str) -> dict[str, Any]:
    value = require_object(payload, location)
    require_exact_keys(
        value,
        required={"version", "plan_sha256"},
        location=location,
    )
    if version_channel(value["version"], f"{location}.version") != "stable":
        fail(f"{location}.version", "must be an exact stable version")
    require_sha256(value["plan_sha256"], f"{location}.plan_sha256")
    return value


def _canonical_plan_path(root: Path, version: str) -> Path:
    path = root / "plans/releases/candidates" / version / "stable-release-plan.json"
    if (
        path.is_symlink()
        or not path.is_file()
        or not path.resolve().is_relative_to(root)
    ):
        fail("approved plan", f"is missing for {version} on protected main")
    _require_head_file(root, path, f"approved plan for {version}")
    return path


def _require_checkout(root: Path, expected_commit: str, label: str) -> None:
    if _git(root, "rev-parse", "HEAD") != expected_commit:
        fail(label, "checkout does not equal the expected commit")
    _require_clean_checkout(root, label)


def _require_clean_checkout(root: Path, label: str) -> None:
    # Publish checks the workspace-root checkout, where Actions places other
    # exact checkouts as untracked siblings. Tracked bytes must stay clean;
    # every consumed request/evidence/plan is separately matched to HEAD.
    if _git(root, "status", "--porcelain", "--untracked-files=no"):
        fail(label, "checkout must be clean")
    if _git(root, "diff", "--name-only", "--diff-filter=U"):
        fail(label, "checkout has unresolved files")


def _require_head_file(root: Path, path: Path, label: str) -> None:
    relative = path.resolve().relative_to(root.resolve()).as_posix()
    result = subprocess.run(
        ["git", "-C", str(root), "show", f"HEAD:{relative}"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0 or result.stdout != path.read_bytes():
        fail(label, "must equal the exact tracked bytes at HEAD")


def _git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(
            f"git {' '.join(args)} failed in {root}: {result.stderr.strip()}"
        )
    return result.stdout.rstrip("\r\n")

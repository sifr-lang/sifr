"""Credential-free filesystem harness for stable incident recovery."""

from __future__ import annotations

import os
import re
import tempfile
from copy import deepcopy
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from .common import (
    GovernanceError,
    PRODUCTION_CREDENTIAL_NAMES,
    canonical_json_bytes,
    fail,
    load_json_strict,
    require_array,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_schema_v2,
    sha256_bytes,
    sha256_file,
    version_channel,
)
from .incident import validate_attempts, validate_incident_request, validate_incident_signoff
from .incident_planner import IncidentMutation, materialize_incident_mutation
from .release_index import validate_release_index
from .release_plan import generate_site_release_facts, validate_site_release_facts

SNAPSHOT_RE = re.compile(r"^channels-generation-([1-9][0-9]*)\.json$")
ATTEMPT_RE = re.compile(r"^attempt-([1-9][0-9]*)\.json$")
FORBIDDEN_CREDENTIALS = set(PRODUCTION_CREDENTIAL_NAMES)
SITE_WAIT_MINUTES = 20


def run_incident_fixture(
    *,
    fixture_root: Path,
    live_index_path: Path,
    governance_root: Path,
    release_assets_root: Path,
    marketplace_path: Path,
    extension_metadata_path: Path,
    site_root: Path,
    request_path: Path,
    affected_plan_path: Path,
    successor_plan_path: Path,
    mode: str,
    approver: str,
    fail_at: str = "none",
) -> dict[str, Any]:
    """Run or resume one local-only incident operation."""
    root = validate_fixture_root(fixture_root)
    _reject_fixture_symlinks(root)
    mode = require_enum(mode, {"initial", "resume"}, "mode")
    fail_at = require_enum(
        fail_at,
        {
            "none",
            "after-reservation",
            "race-before-index",
            "after-index",
            "site-timeout",
        },
        "fail_at",
    )
    require_nonempty_string(approver, "approver")
    _reject_production_credentials()
    request_path = require_fixture_path(root, request_path, "request")
    affected_plan_path = require_fixture_path(root, affected_plan_path, "affected plan")
    successor_plan_path = require_fixture_path(root, successor_plan_path, "successor plan")
    live_index_path = require_fixture_path(root, live_index_path, "live index")
    governance_root = require_fixture_path(
        root,
        governance_root,
        "governance release",
    )
    release_assets_root = require_fixture_path(
        root,
        release_assets_root,
        "release assets",
    )
    marketplace_path = require_fixture_path(root, marketplace_path, "Marketplace stub")
    extension_metadata_path = require_fixture_path(
        root,
        extension_metadata_path,
        "extension metadata",
    )
    site_root = require_fixture_path(root, site_root, "site repository")
    if not (site_root / ".non-deploying-fixture").is_file():
        fail(str(site_root), "site repository must carry the non-deploying fixture marker")
    governance_root.mkdir(parents=True, exist_ok=True)
    request = validate_incident_request(
        load_json_strict(request_path, require_canonical=True)
    )
    request_sha256 = sha256_file(request_path)
    incident_id = request["incident_id"]
    request_asset = governance_root / (
        f"stable-incident-request-{incident_id}-{request_sha256[:16]}.json"
    )
    completed_signoff = governance_root / (
        f"stable-incident-signoff-{incident_id}-{request_sha256[:16]}.json"
    )
    if completed_signoff.exists():
        fail(str(completed_signoff), "incident is already signed off")

    with metadata_lease(root, request["operation"]):
        attempts_root = root / "state" / request_sha256
        attempts_root.mkdir(parents=True, exist_ok=True)
        snapshots = load_snapshots(governance_root)
        live_bytes = live_index_path.read_bytes()
        live = validate_release_index(
            load_json_strict(live_index_path, require_canonical=True)
        )
        _require_live_snapshot(snapshots, live, live_bytes)

        previous, mutation, already_applied = _resolve_mutation(
            request_path=request_path,
            affected_plan_path=affected_plan_path,
            successor_plan_path=successor_plan_path,
            live_index_path=live_index_path,
            request=request,
            live=live,
            live_bytes=live_bytes,
            snapshots=snapshots,
            mode=mode,
        )
        _verify_immutable_installer(
            release_assets_root,
            mutation.proposed_index,
            mutation.successor_version,
        )
        _validate_extension_and_marketplace(
            extension_metadata_path,
            marketplace_path,
            mutation,
        )
        _preserve_write_once(request_asset, request_path.read_bytes(), mode=mode)
        run_id = _next_attempt_id(attempts_root)
        attempt = _new_attempt(run_id, mode, approver, request_asset, request_sha256)
        snapshot_path = governance_root / (
            f"channels-generation-{mutation.proposed_index['generation']}.json"
        )
        proposed_bytes = canonical_json_bytes(mutation.proposed_index)
        if already_applied:
            if snapshot_path.read_bytes() != proposed_bytes:
                fail(str(snapshot_path), "realized snapshot bytes drifted before resume")
        else:
            if not _previous_identity_matches(live_index_path, mutation.previous_index):
                _add_mutation(
                    attempt,
                    "stale-index",
                    "channels.json",
                    sha256_file(live_index_path),
                )
                return _record_failure(attempts_root, attempt, "stale-generation")
            _write_once_bytes(snapshot_path, proposed_bytes)
            _add_mutation(
                attempt,
                "generation-reservation",
                snapshot_path.name,
                sha256_bytes(proposed_bytes),
            )
            if fail_at == "after-reservation":
                return _record_failure(attempts_root, attempt, "after-reservation")
            if fail_at == "race-before-index":
                _simulate_newer_fixture_generation(
                    live_index_path=live_index_path,
                    governance_root=governance_root,
                    previous=mutation.previous_index,
                    generation=mutation.proposed_index["generation"] + 1,
                )
            if not _previous_identity_matches(live_index_path, mutation.previous_index):
                _add_mutation(
                    attempt,
                    "stale-index",
                    "channels.json",
                    sha256_file(live_index_path),
                )
                return _record_failure(attempts_root, attempt, "stale-generation")
            _replace_canonical(live_index_path, mutation.proposed_index)
            _add_mutation(attempt, "release-index", "channels.json", sha256_bytes(proposed_bytes))
            if fail_at == "after-index":
                return _record_failure(attempts_root, attempt, "after-index")

        realized = validate_release_index(
            load_json_strict(live_index_path, require_canonical=True)
        )
        realized_sha256 = sha256_file(live_index_path)
        if realized != mutation.proposed_index:
            fail(str(live_index_path), "realized index does not equal the approved mutation")
        if fail_at == "site-timeout":
            site_attempt = _record_site_attempt(
                site_root=site_root,
                live_index_path=live_index_path,
                request_sha256=request_sha256,
                run_id=run_id,
                generation=realized["generation"],
                index_sha256=realized_sha256,
                status="terminal-timeout",
            )
            _add_mutation(attempt, "site-attempt", site_attempt.name, sha256_file(site_attempt))
            return _record_failure(attempts_root, attempt, "site-timeout")

        _require_site_identity(live_index_path, realized["generation"], realized_sha256)
        site_facts_path = _reconcile_site(
            site_root=site_root,
            governance_root=governance_root,
            mutation=mutation,
            realized_index_path=live_index_path,
            run_id=run_id,
        )
        site_attempt = _record_site_attempt(
            site_root=site_root,
            live_index_path=live_index_path,
            request_sha256=request_sha256,
            run_id=run_id,
            generation=realized["generation"],
            index_sha256=realized_sha256,
            status="succeeded",
        )
        _add_mutation(attempt, "site-attempt", site_attempt.name, sha256_file(site_attempt))
        site_facts_sha256 = sha256_file(site_facts_path)
        _add_mutation(attempt, "site-reconciliation", site_facts_path.name, site_facts_sha256)
        attempt["status"] = "completed"
        _write_attempt(attempts_root, attempt)
        signoff_path = _write_signoff(
            governance_root=governance_root,
            request=request,
            request_sha256=request_sha256,
            previous=previous,
            realized=realized,
            realized_sha256=realized_sha256,
            mutation=mutation,
            attempts_root=attempts_root,
            site_facts_sha256=site_facts_sha256,
        )
        return {
            "status": "completed",
            "operation": mutation.operation,
            "generation": realized["generation"],
            "index_sha256": realized_sha256,
            "signoff": str(signoff_path),
        }


def validate_fixture_root(path: Path) -> Path:
    root = path.resolve()
    temporary_root = Path(tempfile.gettempdir()).resolve()
    if root == temporary_root or temporary_root not in root.parents:
        fail(str(path), "fixture root must be a dedicated directory under the system temp root")
    if not root.is_dir():
        fail(str(path), "fixture root must already exist")
    return root


def require_fixture_path(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve()
    if resolved == root or root not in resolved.parents:
        fail(label, "must resolve inside the explicit fixture root")
    return resolved


def load_snapshots(governance_root: Path) -> dict[int, tuple[Path, dict[str, Any], bytes]]:
    snapshots: dict[int, tuple[Path, dict[str, Any], bytes]] = {}
    for path in sorted(governance_root.glob("channels-generation-*.json")):
        match = SNAPSHOT_RE.fullmatch(path.name)
        if match is None:
            fail(str(path), "invalid generation snapshot name")
        generation = int(match.group(1))
        raw = path.read_bytes()
        value = validate_release_index(load_json_strict(path, require_canonical=True))
        if value["generation"] != generation:
            fail(str(path), "snapshot name and payload generation disagree")
        snapshots[generation] = (path, value, raw)
    if not snapshots:
        fail(str(governance_root), "must retain the live generation snapshot")
    return snapshots


def check_release_submission_allowed(fixture_root: Path, submission: str) -> None:
    """Fail a preview/stable submission while an incident owns metadata."""
    root = validate_fixture_root(fixture_root)
    _reject_fixture_symlinks(root)
    submission = require_enum(submission, {"preview", "stable"}, "submission")
    lock_path = root / "state" / "metadata-concurrency.lock"
    if lock_path.exists():
        owner = lock_path.read_text(encoding="utf-8").strip()
        if owner in {"rollback", "incident-roll-forward"}:
            fail(submission, f"blocked while {owner} is pending")


def plan_fixture_recovery(
    *,
    fixture_root: Path,
    current_version: str | None,
    entrypoint: str,
    force: bool,
) -> dict[str, Any]:
    """Resolve fresh install or explicit incident downgrade to immutable bytes."""
    root = validate_fixture_root(fixture_root)
    _reject_fixture_symlinks(root)
    entrypoint = require_enum(
        entrypoint,
        {"fresh-install", "self-update", "out-of-band"},
        "entrypoint",
    )
    index = validate_release_index(
        load_json_strict(root / "live" / "channels.json", require_canonical=True)
    )
    target_version = index["channels"]["stable"]
    target = index["releases"][target_version]
    if target["status"] != "active":
        fail("stable", "governed stable target is not active")
    if current_version is not None:
        if version_channel(current_version, "current_version") != "stable":
            fail("current_version", "incident recovery supports exact stable versions")
        if _stable_tuple(target_version) < _stable_tuple(current_version) and not force:
            command = (
                "sifr self update --channel stable --force"
                if entrypoint == "self-update"
                else "curl -fsSL https://sifr.sh/install/stable | sh -s -- --force"
            )
            fail("force", f"downgrade requires explicit consent; recovery command: {command}")
    installer = root / "release-assets" / target_version / f"sifr-installer-{target_version}"
    _verify_immutable_installer(root / "release-assets", index, target_version)
    return {
        "status": "ready",
        "entrypoint": entrypoint,
        "current_version": current_version or "none",
        "target_version": target_version,
        "installer": str(installer),
        "installer_sha256": sha256_file(installer),
        "force": force,
        "action": "delegate-to-immutable-installer",
    }


@contextmanager
def metadata_lease(root: Path, operation: str) -> Iterator[None]:
    lock_path = root / "state" / "metadata-concurrency.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        descriptor = os.open(lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError as exc:
        raise GovernanceError("metadata concurrency lease is already held") from exc
    try:
        os.write(descriptor, f"{operation}\n".encode())
        os.close(descriptor)
        yield
    finally:
        try:
            os.close(descriptor)
        except OSError:
            pass
        lock_path.unlink(missing_ok=True)


def _resolve_mutation(
    *,
    request_path: Path,
    affected_plan_path: Path,
    successor_plan_path: Path,
    live_index_path: Path,
    request: dict[str, Any],
    live: dict[str, Any],
    live_bytes: bytes,
    snapshots: dict[int, tuple[Path, dict[str, Any], bytes]],
    mode: str,
) -> tuple[tuple[dict[str, Any], str], IncidentMutation, bool]:
    affected_version = request["affected_release"]["version"]
    live_affected = live["releases"].get(affected_version)
    already_applied = (
        isinstance(live_affected, dict)
        and live_affected.get("status") == "withdrawn"
        and live_affected.get("incident_id") == request["incident_id"]
    )
    if already_applied:
        if mode != "resume":
            fail("mode", "an already-realized incident requires resume")
        previous_candidates = [
            (value, raw)
            for generation, (_, value, raw) in snapshots.items()
            if generation < live["generation"]
            and value["channels"].get("stable") == affected_version
            and value["releases"].get(affected_version, {}).get("status") == "active"
        ]
        if not previous_candidates:
            fail("resume", "cannot recover the exact pre-incident generation")
        previous, previous_bytes = max(previous_candidates, key=lambda item: item[0]["generation"])
        with tempfile.NamedTemporaryFile(
            dir=live_index_path.parent,
            prefix=".incident-previous-",
            suffix=".json",
            delete=False,
        ) as stream:
            stream.write(previous_bytes)
            temporary = Path(stream.name)
        try:
            mutation = materialize_incident_mutation(
                request_path=request_path,
                live_index_path=temporary,
                affected_plan_path=affected_plan_path,
                successor_plan_path=successor_plan_path,
                expected_generation=previous["generation"],
                expected_sha256=sha256_bytes(previous_bytes),
                proposed_generation=live["generation"],
            )
        finally:
            temporary.unlink(missing_ok=True)
        return (previous, sha256_bytes(previous_bytes)), mutation, True

    proposed_generation = max(snapshots) + 1
    mutation = materialize_incident_mutation(
        request_path=request_path,
        live_index_path=live_index_path,
        affected_plan_path=affected_plan_path,
        successor_plan_path=successor_plan_path,
        expected_generation=live["generation"],
        expected_sha256=sha256_bytes(live_bytes),
        proposed_generation=proposed_generation,
    )
    return (live, sha256_bytes(live_bytes)), mutation, False


def _require_live_snapshot(
    snapshots: dict[int, tuple[Path, dict[str, Any], bytes]],
    live: dict[str, Any],
    live_bytes: bytes,
) -> None:
    snapshot = snapshots.get(live["generation"])
    if snapshot is None or snapshot[2] != live_bytes:
        fail("live index", "must equal its retained immutable generation snapshot")


def _previous_identity_matches(
    live_index_path: Path,
    expected: dict[str, Any],
) -> bool:
    try:
        live = validate_release_index(
            load_json_strict(live_index_path, require_canonical=True)
        )
    except GovernanceError:
        return False
    return live == expected


def _simulate_newer_fixture_generation(
    *,
    live_index_path: Path,
    governance_root: Path,
    previous: dict[str, Any],
    generation: int,
) -> None:
    raced = deepcopy(previous)
    raced["generation"] = generation
    validate_release_index(raced)
    _replace_canonical(live_index_path, raced)
    _write_once_bytes(
        governance_root / f"channels-generation-{generation}.json",
        canonical_json_bytes(raced),
    )


def _validate_extension_and_marketplace(
    extension_metadata_path: Path,
    marketplace_path: Path,
    mutation: IncidentMutation,
) -> None:
    extension = _load_range_fixture(extension_metadata_path, "extension metadata")
    marketplace = _load_range_fixture(marketplace_path, "Marketplace stub")
    if mutation.operation == "rollback":
        for label, value in (("extension metadata", extension), ("Marketplace stub", marketplace)):
            if not _version_in_range(mutation.successor_version, value["compiler_compatibility"]):
                fail(label, "compiler compatibility range excludes the rollback target")
    published = require_array(marketplace["published_versions"], "$.published_versions")
    if mutation.successor_version not in published:
        fail("Marketplace stub", "does not contain the successor version")


def _load_range_fixture(path: Path, label: str) -> dict[str, Any]:
    value = require_object(load_json_strict(path, require_canonical=True), label)
    require_exact_keys(
        value,
        required={"schema_version", "compiler_compatibility", "published_versions"},
        location=label,
    )
    require_schema_v2(value, label)
    require_nonempty_string(value["compiler_compatibility"], f"{label}.compiler_compatibility")
    versions = require_array(value["published_versions"], f"{label}.published_versions")
    if not versions or any(not isinstance(item, str) for item in versions):
        fail(f"{label}.published_versions", "must contain exact stable versions")
    return value


def _version_in_range(version: str, expression: str) -> bool:
    match = re.fullmatch(
        r">=([0-9]+\.[0-9]+\.[0-9]+),<([0-9]+\.[0-9]+\.[0-9]+)",
        expression,
    )
    if match is None:
        fail("compiler_compatibility", "must use >=X.Y.Z,<X.Y.Z")
    parsed = tuple(int(part) for part in version.split("."))
    lower = tuple(int(part) for part in match.group(1).split("."))
    upper = tuple(int(part) for part in match.group(2).split("."))
    return lower <= parsed < upper


def _stable_tuple(version: str) -> tuple[int, int, int]:
    version_channel(version, "version")
    major, minor, patch = (int(part) for part in version.split("."))
    return major, minor, patch


def _verify_immutable_installer(
    release_assets_root: Path,
    index: dict[str, Any],
    version: str,
) -> None:
    installer = release_assets_root / version / f"sifr-installer-{version}"
    if not installer.is_file():
        fail(str(installer), "immutable installer fixture is missing")
    if sha256_file(installer) != index["releases"][version]["installer_sha256"]:
        fail(str(installer), "immutable installer digest does not match the governed index")


def _reconcile_site(
    *,
    site_root: Path,
    governance_root: Path,
    mutation: IncidentMutation,
    realized_index_path: Path,
    run_id: int,
) -> Path:
    install_root = site_root / "install"
    dispatchers = {
        name: sha256_file(install_root / name)
        for name in ("index", "stable", "alpha", "beta")
    }
    realized = load_json_strict(realized_index_path, require_canonical=True)
    facts = generate_site_release_facts(
        realized,
        source_plan_sha256=mutation.successor_plan_sha256,
        release_index_sha256=sha256_file(realized_index_path),
        dispatchers=dispatchers,
    )
    deployed_path = site_root / "release-facts.json"
    _replace_canonical(deployed_path, facts)
    validate_site_release_facts(
        load_json_strict(deployed_path, require_canonical=True),
        governed_index=realized,
    )
    retained = governance_root / (
        f"site-release-facts-generation-{realized['generation']}-attempt-{run_id}.json"
    )
    _write_once_bytes(retained, deployed_path.read_bytes())
    return retained


def _record_site_attempt(
    *,
    site_root: Path,
    live_index_path: Path,
    request_sha256: str,
    run_id: int,
    generation: int,
    index_sha256: str,
    status: str,
) -> Path:
    status = require_enum(status, {"succeeded", "terminal-timeout"}, "site status")
    _require_site_identity(live_index_path, generation, index_sha256)
    cancellation = "not-required" if status == "succeeded" else "requested"
    value = (
        f"request_sha256={request_sha256}\n"
        f"run_id={run_id}\n"
        f"generation={generation}\n"
        f"index_sha256={index_sha256}\n"
        f"deadline_minutes={SITE_WAIT_MINUTES}\n"
        f"status={status}\n"
        f"cancellation={cancellation}\n"
    )
    path = site_root / "attempts" / (
        f"{request_sha256[:16]}-attempt-{run_id}.txt"
    )
    _write_once_bytes(path, value.encode())
    return path


def _require_site_identity(
    live_index_path: Path,
    generation: int,
    index_sha256: str,
) -> None:
    live = validate_release_index(
        load_json_strict(live_index_path, require_canonical=True)
    )
    if live["generation"] != generation or sha256_file(live_index_path) != index_sha256:
        fail("site attempt", "live generation/digest changed before dispatch")


def _write_signoff(
    *,
    governance_root: Path,
    request: dict[str, Any],
    request_sha256: str,
    previous: tuple[dict[str, Any], str],
    realized: dict[str, Any],
    realized_sha256: str,
    mutation: IncidentMutation,
    attempts_root: Path,
    site_facts_sha256: str,
) -> Path:
    evidence_digests = {
        name: _write_text_evidence(
            governance_root / f"incident-{name}-{request['incident_id']}.txt",
            _evidence_text(name, request, realized, mutation),
        )
        for name in ("validation", "communications", "closure")
    }
    attempts = [
        load_json_strict(path, require_canonical=True)
        for path in sorted(
            attempts_root.glob("attempt-*.json"),
            key=_attempt_id,
        )
    ]
    validate_attempts(attempts, "$.attempts")
    signoff = {
        "schema_version": 2,
        "incident_id": request["incident_id"],
        "operation": mutation.operation,
        "request_sha256": request_sha256,
        "attempts": attempts,
        "index_mutation": {
            "previous_generation": previous[0]["generation"],
            "previous_sha256": previous[1],
            "realized_generation": realized["generation"],
            "realized_sha256": realized_sha256,
            "affected_version": mutation.affected_version,
            "successor_version": mutation.successor_version,
        },
        "site_reconciliation": {
            "status": "pass",
            "evidence_sha256": site_facts_sha256,
        },
        "validation": {"status": "pass", "evidence_sha256": evidence_digests["validation"]},
        "communications": {
            "status": "pass",
            "evidence_sha256": evidence_digests["communications"],
        },
        "closure": {"status": "pass", "evidence_sha256": evidence_digests["closure"]},
    }
    validate_incident_signoff(signoff, incident_request=request)
    path = governance_root / (
        f"stable-incident-signoff-{request['incident_id']}-{request_sha256[:16]}.json"
    )
    _write_once_bytes(path, canonical_json_bytes(signoff))
    return path


def _evidence_text(
    name: str,
    request: dict[str, Any],
    realized: dict[str, Any],
    mutation: IncidentMutation,
) -> str:
    return (
        f"kind={name}\n"
        f"incident={request['incident_id']}\n"
        f"operation={mutation.operation}\n"
        f"generation={realized['generation']}\n"
        f"stable={mutation.successor_version}\n"
    )


def _new_attempt(
    run_id: int,
    mode: str,
    approver: str,
    request_asset: Path,
    request_sha256: str,
) -> dict[str, Any]:
    attempt = {
        "run_id": run_id,
        "mode": mode,
        "approver": approver,
        "status": "started",
        "mutations": [],
    }
    _add_mutation(attempt, "incident-request", request_asset.name, request_sha256)
    return attempt


def _add_mutation(attempt: dict[str, Any], kind: str, identity: str, digest: str) -> None:
    attempt["mutations"].append({"kind": kind, "identity": identity, "sha256": digest})


def _record_failure(
    attempts_root: Path,
    attempt: dict[str, Any],
    failure: str,
) -> dict[str, Any]:
    attempt["status"] = "failed"
    _add_mutation(
        attempt,
        "failure",
        failure,
        sha256_bytes(f"{failure}\n".encode()),
    )
    _write_attempt(attempts_root, attempt)
    return {"status": "failed", "failure": failure, "run_id": attempt["run_id"]}


def _write_attempt(root: Path, attempt: dict[str, Any]) -> None:
    validate_attempts([attempt], "$.attempts")
    _write_once_bytes(
        root / f"attempt-{attempt['run_id']}.json",
        canonical_json_bytes(attempt),
    )


def _next_attempt_id(root: Path) -> int:
    ids = []
    for path in root.glob("attempt-*.json"):
        match = ATTEMPT_RE.fullmatch(path.name)
        if match is None:
            fail(str(path), "invalid attempt evidence name")
        ids.append(int(match.group(1)))
    return max(ids, default=0) + 1


def _preserve_write_once(path: Path, expected: bytes, *, mode: str) -> None:
    if path.exists():
        if mode != "resume" or path.read_bytes() != expected:
            fail(str(path), "existing request evidence requires exact resume")
        return
    if mode != "initial":
        fail(str(path), "resume requires retained request evidence")
    _write_once_bytes(path, expected)


def _write_once_bytes(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError:
        fail(str(path), "refusing to overwrite retained evidence")
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(value)
        stream.flush()
        os.fsync(stream.fileno())


def _replace_canonical(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = canonical_json_bytes(value)
    with tempfile.NamedTemporaryFile(
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".next",
        delete=False,
    ) as stream:
        stream.write(encoded)
        stream.flush()
        os.fsync(stream.fileno())
        temporary = Path(stream.name)
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _write_text_evidence(path: Path, value: str) -> str:
    encoded = value.encode()
    if path.exists():
        if path.read_bytes() != encoded:
            fail(str(path), "retained evidence bytes drifted")
    else:
        _write_once_bytes(path, encoded)
    return sha256_bytes(encoded)


def _reject_production_credentials() -> None:
    present = sorted(name for name in FORBIDDEN_CREDENTIALS if os.environ.get(name))
    if present:
        fail("environment", f"fixture harness refuses production credential(s): {', '.join(present)}")


def _reject_fixture_symlinks(root: Path) -> None:
    symlinks = sorted(path for path in root.rglob("*") if path.is_symlink())
    if symlinks:
        fail(str(symlinks[0]), "fixture tree must not contain symbolic links")


def _attempt_id(path: Path) -> int:
    match = ATTEMPT_RE.fullmatch(path.name)
    if match is None:
        fail(str(path), "invalid attempt evidence name")
    return int(match.group(1))

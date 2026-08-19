"""Repository custody checks for immutable stable release and incident evidence."""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

from .common import GovernanceError, load_json_strict, sha256_bytes
from .artifact_index import validate_qualification_artifact_index
from .incident_evidence import validate_incident_evidence_commit
from .incident import validate_incident_request, validate_incident_signoff
from .planner import (
    stable_claim_ids,
    validate_documentation_report,
    validate_rust_candidate_result,
)
from .release_plan import validate_release_plan, validate_release_signoff
from .release_report import validate_release_profile_report

REPO_ROOT = Path(__file__).resolve().parents[4]
RELEASES_ROOT = REPO_ROOT / "plans" / "releases"
CANDIDATE_PATH_RE = re.compile(
    r"^plans/releases/candidates/([0-9]+\.[0-9]+\.[0-9]+)/"
    r"((stable-release-plan|release-profile-report|qualification-artifact-index|"
    r"stable-support-claims|rust-validation-report|documentation-report|"
    r"stable-release-signoff)\.json|release-notes\.md)$"
)
INCIDENT_PATH_RE = re.compile(
    r"^plans/releases/incidents/([a-z0-9][a-z0-9-]{2,63})/"
    r"((stable-incident-request|stable-incident-signoff)\.json|withdrawal-evidence\.txt)$"
)


def run_evidence_custody_checks() -> int:
    try:
        validate_changed_evidence_scope()
        validate_committed_incident_addition()
        validate_existing_evidence()
    except GovernanceError as exc:
        print(f"evidence-custody: {exc}", file=sys.stderr)
        return 2
    print("evidence custody ok")
    return 0


def validate_changed_evidence_scope() -> None:
    base = comparison_base()
    validate_changed_path_sets(committed_changed_path_sets(base))
    validate_changed_path_set(working_changed_paths())


def validate_changed_path_sets(changed_sets: list[set[str]]) -> None:
    for changed in changed_sets:
        validate_changed_path_set(changed)


def validate_changed_path_set(changed: set[str]) -> None:
    evidence_paths = [
        path
        for path in changed
        if path.startswith("plans/releases/candidates/")
        or path.startswith("plans/releases/incidents/")
    ]
    if not evidence_paths:
        return
    invalid = [
        path
        for path in evidence_paths
        if CANDIDATE_PATH_RE.fullmatch(path) is None
        and INCIDENT_PATH_RE.fullmatch(path) is None
    ]
    if invalid:
        raise GovernanceError(f"invalid evidence path(s): {', '.join(sorted(invalid))}")
    readme_allowed = all(
        CANDIDATE_PATH_RE.fullmatch(path) is not None for path in evidence_paths
    )
    non_evidence = [
        path
        for path in changed
        if path not in evidence_paths
        and not (readme_allowed and path == "plans/releases/README.md")
    ]
    if non_evidence:
        raise GovernanceError(
            "release evidence changes cannot mix with source changes in one commit: "
            + ", ".join(sorted(non_evidence))
        )
    identities = {
        ("candidate", match.group(1))
        if (match := CANDIDATE_PATH_RE.fullmatch(path)) is not None
        else ("incident", INCIDENT_PATH_RE.fullmatch(path).group(1))  # type: ignore[union-attr]
        for path in evidence_paths
    }
    if len(identities) != 1:
        raise GovernanceError("an evidence change must contain exactly one candidate or incident")


def committed_changed_path_sets(base: str) -> list[set[str]]:
    commits = git_output("rev-list", "--first-parent", "--reverse", f"{base}..HEAD")
    changed_sets: list[set[str]] = []
    for commit in commits.splitlines():
        parent = git_output("rev-parse", f"{commit}^1", allow_failure=True)
        if not parent:
            raise GovernanceError(f"cannot establish first parent for commit {commit}")
        changed_sets.append(
            {
                line
                for line in git_output("diff", "--name-only", parent, commit).splitlines()
                if line
            }
        )
    return changed_sets


def working_changed_paths() -> set[str]:
    tracked: set[str] = set()
    tracked.update(
        line
        for line in git_output("diff", "--name-only").splitlines()
        if line
    )
    tracked.update(
        line
        for line in git_output("diff", "--cached", "--name-only").splitlines()
        if line
    )
    tracked.update(
        line
        for line in git_output("ls-files", "--others", "--exclude-standard").splitlines()
        if line
    )
    return tracked


def comparison_base() -> str:
    base_ref = os.environ.get("SIFR_EVIDENCE_BASE_REF", "origin/main")
    merge_base = git_output("merge-base", base_ref, "HEAD", allow_failure=True)
    if not merge_base:
        merge_base = git_output("rev-parse", "HEAD^", allow_failure=True)
    return require_comparison_base(merge_base, base_ref=base_ref)


def validate_committed_incident_addition() -> None:
    merge_base = comparison_base()
    committed = {
        line
        for line in git_output("diff", "--name-only", f"{merge_base}...HEAD").splitlines()
        if line
    }
    request_paths = sorted(
        path
        for path in committed
        if path.endswith("/stable-incident-request.json")
        and path.startswith("plans/releases/incidents/")
    )
    for request_path in request_paths:
        directory = request_path.rsplit("/", 1)[0]
        evidence_path = f"{directory}/withdrawal-evidence.txt"
        validate_incident_evidence_commit(
            repository=REPO_ROOT,
            base=merge_base,
            head="HEAD",
            request_path=request_path,
            evidence_path=evidence_path,
        )


def require_comparison_base(value: str, *, base_ref: str) -> str:
    if not value:
        raise GovernanceError(
            f"cannot establish an evidence custody comparison base from {base_ref} or HEAD^"
        )
    return value


def validate_existing_evidence() -> None:
    candidates_root = RELEASES_ROOT / "candidates"
    if candidates_root.is_dir():
        for directory in sorted(path for path in candidates_root.iterdir() if path.is_dir()):
            validate_candidate_directory(directory)
    incidents_root = RELEASES_ROOT / "incidents"
    if incidents_root.is_dir():
        for directory in sorted(path for path in incidents_root.iterdir() if path.is_dir()):
            validate_incident_directory(directory)


def validate_candidate_directory(directory: Path) -> None:
    expected_version = directory.name
    if directory.is_symlink() or not directory.is_dir():
        raise GovernanceError(f"{directory}: candidate evidence must be a directory")
    entries = list(directory.iterdir())
    unsafe = sorted(
        path.name for path in entries if path.is_symlink() or not path.is_file()
    )
    if unsafe:
        raise GovernanceError(
            f"{directory}: unsupported candidate evidence entries: {', '.join(unsafe)}"
        )
    files = {path.name for path in entries}
    allowed = {
        "stable-release-plan.json",
        "release-profile-report.json",
        "qualification-artifact-index.json",
        "stable-support-claims.json",
        "rust-validation-report.json",
        "documentation-report.json",
        "release-notes.md",
        "stable-release-signoff.json",
    }
    unknown = sorted(files.difference(allowed))
    if unknown:
        raise GovernanceError(
            f"{directory}: unsupported evidence files: {', '.join(unknown)}"
        )
    required = {
        "stable-release-plan.json",
        "release-profile-report.json",
        "qualification-artifact-index.json",
        "stable-support-claims.json",
        "rust-validation-report.json",
        "documentation-report.json",
        "release-notes.md",
    }
    missing = sorted(required.difference(files))
    if missing:
        raise GovernanceError(
            f"{directory}: missing candidate evidence: {', '.join(missing)}"
        )

    plan_path = directory / "stable-release-plan.json"
    report_path = directory / "release-profile-report.json"
    qualification_path = directory / "qualification-artifact-index.json"
    plan = validate_release_plan(load_json_strict(plan_path, require_canonical=True))
    report = validate_release_profile_report(
        load_json_strict(report_path, require_canonical=True),
        canonical_bytes=report_path.read_bytes(),
    )
    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_path, require_canonical=True)
    )
    if (
        plan["version"] != expected_version
        or report["source"]["commit"] != plan["source_commit"]
    ):
        raise GovernanceError(f"{directory}: candidate identity/source mismatch")
    if qualification["candidate_version"] != expected_version:
        raise GovernanceError(f"{directory}: qualification candidate version mismatch")
    if qualification["source_commit"] != plan["source_commit"]:
        raise GovernanceError(f"{directory}: qualification source mismatch")
    if qualification["submodules"] != plan["submodules"]:
        raise GovernanceError(f"{directory}: qualification submodule mismatch")
    if plan["release_profile_report"]["sha256"] != sha256_bytes(
        report_path.read_bytes()
    ):
        raise GovernanceError(f"{directory}: release report digest mismatch")
    if plan["qualification_artifact_index"]["sha256"] != sha256_bytes(
        qualification_path.read_bytes()
    ):
        raise GovernanceError(
            f"{directory}: qualification artifact index digest mismatch"
        )
    claims_path = directory / "stable-support-claims.json"
    if plan["rust_interop"]["stable_support_claims_sha256"] != sha256_bytes(
        claims_path.read_bytes()
    ):
        raise GovernanceError(f"{directory}: stable support claims digest mismatch")
    claims = load_json_strict(claims_path, require_canonical=True)
    if plan["rust_interop"]["advertised_claim_ids"] != stable_claim_ids(claims):
        raise GovernanceError(f"{directory}: stable support claim ids drifted")
    rust_report_path = directory / "rust-validation-report.json"
    if plan["rust_interop"]["validation_report_sha256"] != sha256_bytes(
        rust_report_path.read_bytes()
    ):
        raise GovernanceError(f"{directory}: Rust validation report digest mismatch")
    validate_rust_candidate_result(
        load_json_strict(rust_report_path, require_canonical=True),
        expected_digest=plan["rust_interop"]["validation_report_sha256"],
        release_report=report,
    )
    documentation_path = directory / "documentation-report.json"
    if plan["documentation_report"]["sha256"] != sha256_bytes(
        documentation_path.read_bytes()
    ):
        raise GovernanceError(f"{directory}: documentation report digest mismatch")
    documentation = validate_documentation_report(
        load_json_strict(documentation_path, require_canonical=True),
        source_commit=plan["source_commit"],
    )
    if documentation["report_id"] != plan["documentation_report"]["id"]:
        raise GovernanceError(f"{directory}: documentation report id mismatch")
    if plan["release_notes_sha256"] != sha256_bytes(
        (directory / "release-notes.md").read_bytes()
    ):
        raise GovernanceError(f"{directory}: release notes digest mismatch")
    signoff_path = directory / "stable-release-signoff.json"
    if signoff_path.is_file():
        signoff = validate_release_signoff(
            load_json_strict(signoff_path, require_canonical=True)
        )
        if signoff["version"] != expected_version or signoff[
            "plan_sha256"
        ] != sha256_bytes(plan_path.read_bytes()):
            raise GovernanceError(f"{directory}: stable sign-off provenance mismatch")


def validate_incident_directory(directory: Path) -> None:
    files = {path.name for path in directory.iterdir() if path.is_file()}
    allowed = {
        "stable-incident-request.json",
        "withdrawal-evidence.txt",
        "stable-incident-signoff.json",
    }
    unknown = sorted(files.difference(allowed))
    if unknown:
        raise GovernanceError(f"{directory}: unsupported incident evidence: {', '.join(unknown)}")
    request_path = directory / "stable-incident-request.json"
    if not request_path.is_file():
        raise GovernanceError(f"{directory}: stable-incident-request.json is required")
    evidence_path = directory / "withdrawal-evidence.txt"
    if not evidence_path.is_file():
        raise GovernanceError(f"{directory}: withdrawal-evidence.txt is required")
    request = validate_incident_request(load_json_strict(request_path, require_canonical=True))
    if request["incident_id"] != directory.name:
        raise GovernanceError(f"{directory}: incident id does not match directory")
    if request["withdrawal"]["evidence_sha256"] != sha256_bytes(evidence_path.read_bytes()):
        raise GovernanceError(f"{directory}: withdrawal evidence digest mismatch")
    signoff_path = directory / "stable-incident-signoff.json"
    if signoff_path.is_file():
        signoff = validate_incident_signoff(
            load_json_strict(signoff_path, require_canonical=True),
            incident_request=request,
        )
        if signoff["incident_id"] != directory.name:
            raise GovernanceError(f"{directory}: incident sign-off identity mismatch")
        if signoff["request_sha256"] != sha256_bytes(request_path.read_bytes()):
            raise GovernanceError(f"{directory}: incident request digest mismatch")


def git_output(*args: str, allow_failure: bool = False) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        if allow_failure:
            return ""
        raise GovernanceError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


if __name__ == "__main__":
    raise SystemExit(run_evidence_custody_checks())

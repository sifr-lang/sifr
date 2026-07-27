"""Fail-closed materialization of an immutable stable release plan."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path
from typing import Any

from .artifact_index import validate_qualification_artifact_index
from .common import (
    GovernanceError,
    TARGETS,
    fail,
    load_json_strict,
    require_array,
    require_commit,
    require_nonempty_string,
    require_object,
    require_sha256,
    sha256_file,
)
from .release_plan import validate_release_plan
from .release_report import (
    canonical_profile_digest,
    collect_submodules,
    validate_release_profile_report,
)

RELEASE_PROFILE = Path("verification/profiles/release.json")
COMPATIBILITY_MATRIX = Path(
    "verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json"
)
SITE_FACTS_SCHEMA = Path(
    "verification/areas/distribution_release/schemas/stable_site_release_facts.schema.json"
)
SITE_FACTS_GENERATOR = Path(
    "verification/areas/distribution_release/governance/release_plan.py"
)
RUST_CLAIMS_SCHEMA_VERSION = 1


def materialize_stable_plan(
    *,
    plan_spec: Path,
    source_root: Path,
    source_ref: str,
    active_index_path: Path,
    release_report_path: Path,
    qualification_index_path: Path,
    artifact_root: Path,
    stable_support_claims_path: Path,
    rust_validation_report_path: Path,
    documentation_report_path: Path,
    release_notes_path: Path,
) -> dict[str, Any]:
    source_root = source_root.resolve()
    plan = validate_release_plan(
        load_json_strict(plan_spec, require_canonical=True),
        active_index=load_json_strict(active_index_path, require_canonical=True),
    )
    resolved_commit = resolve_source_once(source_root, source_ref)
    validate_source_identity(
        plan, source_root=source_root, resolved_commit=resolved_commit
    )

    profile_path = source_root / RELEASE_PROFILE
    profile_digest = canonical_profile_digest(profile_path)
    if plan["toolchain"]["profile_manifest_sha256"] != profile_digest:
        fail("$.toolchain.profile_manifest_sha256", "does not match release profile")
    if plan["cargo_lock_sha256"] != sha256_file(source_root / "Cargo.lock"):
        fail("$.cargo_lock_sha256", "does not match source Cargo.lock")
    if plan["toolchain"]["rustc"] != command_output(source_root, "rustc", "--version"):
        fail("$.toolchain.rustc", "does not match the release checkout toolchain")
    if plan["toolchain"]["cargo"] != command_output(source_root, "cargo", "--version"):
        fail("$.toolchain.cargo", "does not match the release checkout toolchain")

    report_bytes = release_report_path.read_bytes()
    report = validate_release_profile_report(
        load_json_strict(release_report_path, require_canonical=True),
        canonical_bytes=report_bytes,
        source_root=source_root,
        expected_profile_sha256=profile_digest,
        verify_artifacts=True,
    )
    require_reference(
        plan["release_profile_report"],
        identifier=report["report_id"],
        path=release_report_path,
        location="$.release_profile_report",
    )
    if report["source"]["commit"] != resolved_commit:
        fail("$.release_profile_report", "source commit does not match the plan")
    if report["source"]["submodules"] != plan["submodules"]:
        fail("$.release_profile_report", "submodules do not match the plan")

    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_index_path, require_canonical=True),
        require_unexpired=True,
    )
    qualification_id = (
        f"qualification-{qualification['workflow']['run_id']}-"
        f"{qualification['workflow']['run_attempt']}"
    )
    require_reference(
        plan["qualification_artifact_index"],
        identifier=qualification_id,
        path=qualification_index_path,
        location="$.qualification_artifact_index",
    )
    if (
        qualification["candidate_version"] != plan["version"]
        or qualification["source_commit"] != resolved_commit
        or qualification["submodules"] != plan["submodules"]
    ):
        fail(
            "$.qualification_artifact_index",
            "candidate provenance does not match the plan",
        )
    artifact_paths = verify_transported_artifacts(qualification, artifact_root)
    bind_target_reports(plan, qualification, artifact_paths)
    bind_aggregate_artifacts(plan, qualification, artifact_paths)

    compatibility_path = source_root / COMPATIBILITY_MATRIX
    require_digest(
        plan["rust_interop"]["compatibility_matrix_sha256"],
        compatibility_path,
        "$.rust_interop.compatibility_matrix_sha256",
    )
    require_digest(
        plan["rust_interop"]["stable_support_claims_sha256"],
        stable_support_claims_path,
        "$.rust_interop.stable_support_claims_sha256",
    )
    require_digest(
        plan["rust_interop"]["validation_report_sha256"],
        rust_validation_report_path,
        "$.rust_interop.validation_report_sha256",
    )
    validate_rust_candidate_result(
        load_json_strict(rust_validation_report_path),
        expected_digest=plan["rust_interop"]["validation_report_sha256"],
        release_report=report,
    )
    claim_ids = stable_claim_ids(load_json_strict(stable_support_claims_path))
    if plan["rust_interop"]["advertised_claim_ids"] != claim_ids:
        fail(
            "$.rust_interop.advertised_claim_ids",
            "must exactly match the ordered stable support claims",
        )

    documentation_report = validate_documentation_report(
        load_json_strict(documentation_report_path, require_canonical=True),
        source_commit=plan["source_commit"],
    )
    require_reference(
        plan["documentation_report"],
        identifier=documentation_report["report_id"],
        path=documentation_report_path,
        location="$.documentation_report",
    )
    require_digest(
        plan["release_notes_sha256"],
        release_notes_path,
        "$.release_notes_sha256",
    )
    require_digest(
        plan["site"]["facts_schema_sha256"],
        source_root / SITE_FACTS_SCHEMA,
        "$.site.facts_schema_sha256",
    )
    require_digest(
        plan["site"]["facts_generator_sha256"],
        source_root / SITE_FACTS_GENERATOR,
        "$.site.facts_generator_sha256",
    )
    return plan


def resolve_source_once(source_root: Path, source_ref: str) -> str:
    require_commit(source_ref, "--source-ref")
    result = subprocess.run(
        ["git", "rev-parse", "--verify", f"{source_ref}^{{commit}}"],
        cwd=source_root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(
            f"could not resolve source ref exactly once: {source_ref}"
        )
    return require_commit(result.stdout.strip(), "--source-ref")


def validate_source_identity(
    plan: dict[str, Any],
    *,
    source_root: Path,
    resolved_commit: str,
) -> None:
    head = command_output(source_root, "git", "rev-parse", "HEAD")
    if resolved_commit != head or plan["source_commit"] != resolved_commit:
        fail("$.source_commit", "must equal the resolved checkout HEAD")
    status = command_output(
        source_root,
        "git",
        "status",
        "--porcelain",
        "--untracked-files=all",
    )
    if status:
        fail("$.source_commit", "release checkout must be clean")
    unresolved = command_output(
        source_root,
        "git",
        "diff",
        "--name-only",
        "--diff-filter=U",
    )
    if unresolved:
        fail("$.source_commit", "release checkout has unresolved paths")
    if collect_submodules(source_root) != plan["submodules"]:
        fail("$.submodules", "does not match the recursive checkout")


def verify_transported_artifacts(
    qualification: dict[str, Any],
    artifact_root: Path,
) -> dict[str, Path]:
    result: dict[str, Path] = {}
    resolved_root = artifact_root.resolve()
    for artifact in qualification["artifacts"]:
        container = artifact_root / artifact["workflow_artifact_name"]
        path = container / artifact["name"]
        resolved_path = path.resolve()
        if container.is_symlink() or not resolved_path.is_relative_to(resolved_root):
            fail(
                f"$.artifacts.{artifact['id']}",
                "transported path escapes the artifact custody root",
            )
        if not path.is_file() or path.is_symlink():
            fail(
                f"$.artifacts.{artifact['id']}", "transported file is missing or unsafe"
            )
        if path.stat().st_size != artifact["size_bytes"]:
            fail(
                f"$.artifacts.{artifact['id']}.size_bytes",
                "does not match transported file",
            )
        if sha256_file(path) != artifact["sha256"]:
            fail(
                f"$.artifacts.{artifact['id']}.sha256",
                "does not match transported file",
            )
        result[artifact["id"]] = path
    return result


def bind_target_reports(
    plan: dict[str, Any],
    qualification: dict[str, Any],
    artifact_paths: dict[str, Path],
) -> None:
    plan_targets = {target["triple"]: target for target in plan["targets"]}
    artifacts = {artifact["id"]: artifact for artifact in qualification["artifacts"]}
    for target in TARGETS:
        report_path = artifact_paths[f"qualification-report-{target}"]
        report = validate_target_report(
            load_json_strict(report_path, require_canonical=True),
            version=plan["version"],
            source_commit=plan["source_commit"],
            target=target,
        )
        plan_target = plan_targets[target]
        for field in (
            "builder",
            "binary_sha256",
            "sysroot_sha256",
            "archive_sha256",
            "checksum_sha256",
            "sifr_version",
            "installer_version",
            "receipt_channel",
            "sysroot_version",
            "sysroot_target",
        ):
            if plan_target[field] != report[field]:
                fail(
                    f"$.targets.{target}.{field}", "does not match qualification report"
                )
        for artifact_id, report_field in (
            (f"binary-archive-{target}", "archive_sha256"),
            (f"checksum-{target}", "checksum_sha256"),
            (f"sysroot-{target}", "sysroot_bundle_sha256"),
        ):
            if artifacts[artifact_id]["sha256"] != report[report_field]:
                fail(f"$.artifacts.{artifact_id}", "does not match target report")


def bind_aggregate_artifacts(
    plan: dict[str, Any],
    qualification: dict[str, Any],
    artifact_paths: dict[str, Path],
) -> None:
    artifacts = {artifact["id"]: artifact for artifact in qualification["artifacts"]}
    if artifacts["installer"]["sha256"] != plan["installer_sha256"]:
        fail("$.installer_sha256", "does not match the transported aggregate installer")
    validate_installer_identity(
        artifact_paths["installer"],
        version=plan["version"],
    )
    validate_aggregate_checksums(
        artifact_paths["checksums"],
        qualification["artifacts"],
    )
    if artifacts["vsix"]["sha256"] != plan["vscode"]["vsix_sha256"]:
        fail("$.vscode.vsix_sha256", "does not match the transported VSIX")
    editor_report_path = artifact_paths["editor-qualification-report"]
    editor_report = validate_editor_report(
        load_json_strict(editor_report_path, require_canonical=True),
        source_commit=plan["source_commit"],
        submodule_commit=plan["submodules"].get("editor_integrations"),
    )
    if sha256_file(editor_report_path) != plan["vscode"]["validation_report_sha256"]:
        fail("$.vscode.validation_report_sha256", "does not match editor qualification")
    if (
        editor_report.get("package_path") != plan["vscode"]["package_path"]
        or editor_report.get("package_version") != plan["vscode"]["version"]
        or editor_report.get("compiler_compatibility")
        != plan["vscode"]["compiler_compatibility"]
        or editor_report.get("vsix_sha256") != plan["vscode"]["vsix_sha256"]
    ):
        fail("$.vscode", "does not match editor qualification evidence")


def validate_installer_identity(installer_path: Path, *, version: str) -> None:
    assignments: dict[str, list[str]] = {
        "APP_VERSION": [],
        "APP_CHANNEL": [],
    }
    assignment = re.compile(r"^\s*(?:export\s+)?(APP_VERSION|APP_CHANNEL)\s*=")
    for line in read_evidence_text(
        installer_path,
        location="$.installer_sha256",
    ).splitlines():
        match = assignment.match(line)
        if match is not None:
            assignments[match.group(1)].append(line)
    expected = {
        "APP_VERSION": [f'APP_VERSION="{version}"'],
        "APP_CHANNEL": ['APP_CHANNEL="stable"'],
    }
    if assignments != expected:
        fail(
            "$.installer_sha256",
            "installer must contain exactly one canonical candidate version and channel assignment",
        )


def validate_aggregate_checksums(
    checksums_path: Path,
    artifacts: list[dict[str, Any]],
) -> None:
    expected = {
        artifact["name"]: artifact["sha256"]
        for artifact in artifacts
        if artifact["kind"] in {"binary-archive", "checksum", "sysroot"}
    }
    observed: dict[str, str] = {}
    for line in read_evidence_text(
        checksums_path,
        location="$.artifacts.checksums",
    ).splitlines():
        parts = line.split()
        if len(parts) != 2 or parts[1] in observed:
            fail(
                "$.artifacts.checksums", "contains malformed or duplicate checksum rows"
            )
        observed[parts[1]] = parts[0]
    if observed != expected:
        fail("$.artifacts.checksums", "does not bind the complete target artifact set")


def validate_editor_report(
    payload: Any,
    *,
    source_commit: str,
    submodule_commit: str | None,
) -> dict[str, Any]:
    report = require_object(payload, "editor qualification report")
    required = {
        "schema_version",
        "kind",
        "source_commit",
        "submodule_commit",
        "package_path",
        "package_version",
        "compiler_compatibility",
        "vsix_sha256",
        "status",
    }
    if set(report) != required:
        fail("$.vscode.validation_report_sha256", "editor report fields are not exact")
    if (
        report["schema_version"] != 2
        or report["kind"] != "stable-editor-qualification"
        or report["status"] != "pass"
        or report["source_commit"] != source_commit
        or report["submodule_commit"] != submodule_commit
    ):
        fail("$.vscode.validation_report_sha256", "editor report identity did not pass")
    for field in ("package_path", "package_version", "compiler_compatibility"):
        require_nonempty_string(report[field], f"editor qualification report.{field}")
    require_sha256(report["vsix_sha256"], "editor qualification report.vsix_sha256")
    return report


def validate_documentation_report(
    payload: Any,
    *,
    source_commit: str,
) -> dict[str, Any]:
    report = require_object(payload, "documentation qualification report")
    required = {"schema_version", "kind", "report_id", "source_commit", "status"}
    if set(report) != required:
        fail("$.documentation_report", "documentation report fields are not exact")
    if (
        report["schema_version"] != 2
        or report["kind"] != "stable-documentation-qualification"
        or report["source_commit"] != source_commit
        or report["status"] != "pass"
    ):
        fail("$.documentation_report", "documentation qualification did not pass")
    require_nonempty_string(report["report_id"], "$.documentation_report.id")
    return report


def read_evidence_text(path: Path, *, location: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise GovernanceError(
            f"{location}: evidence is not readable UTF-8: {exc}"
        ) from exc


def validate_target_report(
    payload: Any,
    *,
    version: str,
    source_commit: str,
    target: str,
) -> dict[str, Any]:
    report = require_object(payload, f"qualification-report-{target}")
    required = {
        "schema_version",
        "kind",
        "candidate_version",
        "source_commit",
        "target",
        "builder",
        "binary_sha256",
        "sysroot_sha256",
        "archive_sha256",
        "checksum_sha256",
        "sysroot_bundle_sha256",
        "sifr_version",
        "installer_version",
        "receipt_channel",
        "sysroot_version",
        "sysroot_target",
        "smoke_status",
        "self_version_sha256",
    }
    if set(report) != required:
        fail(
            f"qualification-report-{target}",
            "fields do not match the target report contract",
        )
    if (
        report["schema_version"] != 2
        or report["kind"] != "stable-target-qualification"
        or report["candidate_version"] != version
        or report["source_commit"] != source_commit
        or report["target"] != target
        or report["smoke_status"] != "pass"
    ):
        fail(f"qualification-report-{target}", "candidate identity or status mismatch")
    for field in (
        "binary_sha256",
        "sysroot_sha256",
        "archive_sha256",
        "checksum_sha256",
        "sysroot_bundle_sha256",
        "self_version_sha256",
    ):
        require_digest_value(report[field], f"qualification-report-{target}.{field}")
    return report


def stable_claim_ids(payload: Any) -> list[str]:
    claims = require_object(payload, "stable_support_claims.json")
    expected_fields = {
        "schema_version",
        "role",
        "source_compatibility_matrix",
        "public_document",
        "runtime_deferrals",
        "claims",
    }
    if (
        set(claims) != expected_fields
        or claims.get("schema_version") != RUST_CLAIMS_SCHEMA_VERSION
        or claims.get("role") != "compatibility-derived-release-plan-input"
        or claims.get("source_compatibility_matrix")
        != "verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json"
        or claims.get("public_document") != "docs/rust-interop.mdx"
    ):
        fail(
            "stable_support_claims.json", "does not match the certified claims contract"
        )
    deferrals = require_array(
        claims["runtime_deferrals"],
        "stable_support_claims.json runtime_deferrals",
    )
    if not all(isinstance(value, str) and value for value in deferrals):
        fail("stable_support_claims.json runtime_deferrals", "must contain row ids")
    raw = claims["claims"]
    values = require_array(raw, "stable_support_claims.json claims")
    identifiers: list[str] = []
    for position, value in enumerate(values):
        claim = require_object(value, f"stable_support_claims.json claims[{position}]")
        if set(claim) != {"id", "category", "execution_kind", "capability"}:
            fail(
                f"stable_support_claims.json claims[{position}]",
                "fields do not match the certified claim contract",
            )
        identifiers.append(
            require_nonempty_string(
                claim["id"],
                f"stable_support_claims.json claims[{position}].id",
            )
        )
    if not identifiers or len(set(identifiers)) != len(identifiers):
        fail("stable_support_claims.json claims", "must be non-empty and unique")
    return identifiers


def validate_rust_candidate_result(
    payload: Any,
    *,
    expected_digest: str,
    release_report: dict[str, Any],
) -> None:
    result = require_object(payload, "rust stable-candidate result")
    if (
        result.get("area") != "rust_interop"
        or result.get("manifest") != "verification/areas/rust_interop/manifest.json"
        or result.get("bless") is not False
    ):
        fail(
            "$.rust_interop.validation_report_sha256",
            "does not identify the authoritative unblessed Rust-interop result",
        )
    suites = require_array(result.get("suites"), "rust stable-candidate result suites")
    expected_suites = {
        "matrix",
        "tiers",
        "compatibility-matrix",
        "stale-drafts",
        "stable-candidate",
    }
    by_name: dict[str, dict[str, Any]] = {}
    for position, value in enumerate(suites):
        suite = require_object(
            value, f"rust stable-candidate result suites[{position}]"
        )
        name = require_nonempty_string(
            suite.get("name"),
            f"rust stable-candidate result suites[{position}].name",
        )
        if name in by_name:
            fail("$.rust_interop.validation_report_sha256", "contains duplicate suites")
        by_name[name] = suite
        if (
            suite.get("blocking") is not True
            or suite.get("failed_cases") != 0
            or suite.get("total_failures") != 0
            or not isinstance(suite.get("total_variants"), int)
            or suite["total_variants"] < 1
        ):
            fail(
                "$.rust_interop.validation_report_sha256",
                f"Rust-interop suite {name} did not pass as a blocking suite",
            )
        validate_passing_cases(suite.get("cases"), suite_name=name)
    if set(by_name) != expected_suites:
        fail(
            "$.rust_interop.validation_report_sha256",
            "does not contain the exact release Rust-interop suite set",
        )
    stable_case_ids = {
        case.get("id")
        for case in require_array(
            by_name["stable-candidate"].get("cases"),
            "rust stable-candidate cases",
        )
        if isinstance(case, dict)
    }
    if stable_case_ids != {
        "rust-interop-stable-candidate",
        "rust-interop-stable-candidate-self-test",
    }:
        fail(
            "$.rust_interop.validation_report_sha256",
            "stable-candidate result omitted its validator or adversarial self-test",
        )
    summary = require_object(
        result.get("summary"), "rust stable-candidate result summary"
    )
    if (
        summary.get("blocking_failures") != 0
        or summary.get("non_blocking_failures") != 0
        or summary.get("total_failures") != 0
    ):
        fail("$.rust_interop.validation_report_sha256", "Rust-interop result failed")
    artifact_matches = [
        artifact
        for artifact in release_report["result_artifacts"]
        if artifact["sha256"] == expected_digest
        and Path(artifact["path"]).name == "rust-interop-release-results.json"
    ]
    suite_matches = [
        suite
        for step in release_report["steps"]
        for suite in step["suite_results"]
        if suite["area"] == "rust_interop"
        and suite["suite"] == "stable-candidate"
        and suite["result_artifact_sha256"] == expected_digest
    ]
    if len(artifact_matches) != 1 or len(suite_matches) != 1:
        fail(
            "$.rust_interop.validation_report_sha256",
            "is not the stable-candidate result bound by the release report",
        )


def validate_passing_cases(payload: Any, *, suite_name: str) -> None:
    cases = require_array(payload, f"Rust-interop suite {suite_name} cases")
    if not cases:
        fail(
            "$.rust_interop.validation_report_sha256",
            f"Rust-interop suite {suite_name} has no cases",
        )
    for case_position, value in enumerate(cases):
        case = require_object(
            value, f"Rust-interop suite {suite_name} case[{case_position}]"
        )
        variants = require_array(
            case.get("variants"),
            f"Rust-interop suite {suite_name} case[{case_position}].variants",
        )
        if not variants:
            fail(
                "$.rust_interop.validation_report_sha256",
                f"Rust-interop suite {suite_name} has an unexecuted case",
            )
        for variant in variants:
            row = require_object(variant, f"Rust-interop suite {suite_name} variant")
            if (
                row.get("status") != "pass"
                or row.get("actual_exit_code") != row.get("expected_exit_code")
                or row.get("mismatches") != []
            ):
                fail(
                    "$.rust_interop.validation_report_sha256",
                    f"Rust-interop suite {suite_name} contains a failing variant",
                )


def require_reference(
    reference: dict[str, Any],
    *,
    identifier: str,
    path: Path,
    location: str,
) -> None:
    if reference["id"] != identifier:
        fail(f"{location}.id", f"does not match {identifier}")
    require_digest(reference["sha256"], path, f"{location}.sha256")


def require_digest(expected: str, path: Path, location: str) -> None:
    if not path.is_file() or sha256_file(path) != expected:
        fail(location, f"does not match {path}")


def require_digest_value(value: Any, location: str) -> None:
    require_sha256(value, location)


def command_output(source_root: Path, *command: str) -> str:
    result = subprocess.run(
        list(command),
        cwd=source_root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(
            f"{' '.join(command)} failed with exit {result.returncode}: {result.stderr.strip()}"
        )
    return result.stdout.strip()

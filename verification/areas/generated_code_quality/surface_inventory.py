"""Fail-closed discovery of every maintained generated-Rust source surface."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

REQUIRED_ENTRYPOINT_CLASSES = {
    "algorithmic-corpus",
    "cli-build-project",
    "cli-build-single",
    "cli-emit-single",
    "cli-run-single",
    "cli-test-project",
    "rust-interop-project",
    "static-program",
    "sysroot-project",
}
REQUIRED_ENTRYPOINT_DISCOVERIES = {
    "algorithmic-corpus": {"algorithmic-sources"},
    "cli-build-project": {"project-workspace-sources"},
    "cli-build-single": {"demo-emitted-companions"},
    "cli-emit-single": {"demo-emitted-companions"},
    "cli-run-single": {"e2e-pass-sources"},
    "cli-test-project": {"project-workspace-sources"},
    "rust-interop-project": {
        "rust-interop-positive-sources",
        "rust-interop-runtime-projects",
    },
    "static-program": {"rust-interop-positive-sources"},
    "sysroot-project": {"sysroot-release-sources"},
}
REQUIRED_ENTRYPOINT_QUALIFICATIONS = {
    "algorithmic-corpus": {"algorithmic_compatibility:leetcode-full"},
    "cli-build-project": {"project_workspace:audit-fixtures"},
    "cli-build-single": {"generated_code_quality:full"},
    "cli-emit-single": {"generated_code_quality:full"},
    "cli-run-single": {"generated_code_quality:full"},
    "cli-test-project": {"project_workspace:audit-fixtures"},
    "rust-interop-project": {"rust_interop:matrix"},
    "static-program": {"rust_interop:matrix"},
    "sysroot-project": {"sysroot_release:host-installed-stdlib-heavy"},
}
REQUIRED_DISCOVERY_IDS = {
    "algorithmic-sources",
    "demo-emitted-companions",
    "e2e-pass-sources",
    "negative-gate-seeds",
    "project-workspace-sources",
    "rust-interop-positive-sources",
    "rust-interop-runtime-projects",
    "sysroot-release-sources",
}


def path_set_digest(paths: list[str]) -> str:
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.encode("utf-8"))
        digest.update(b"\0")
    return digest.hexdigest()


def discover_paths(repo_root: Path, root: str, pattern: str) -> list[str]:
    resolved_repo = repo_root.resolve()
    root_path = (repo_root / root).resolve()
    if not root_path.is_relative_to(resolved_repo) or not root_path.is_dir():
        raise ValueError(f"discovery root must be a repository directory: {root}")
    pattern_path = Path(pattern)
    if (
        pattern_path.is_absolute()
        or ".." in pattern_path.parts
        or any(character in pattern for character in "?[]{}")
    ):
        raise ValueError(f"discovery pattern must be a constrained repository glob: {pattern}")
    paths = []
    for path in root_path.glob(pattern):
        resolved = path.resolve()
        if not resolved.is_relative_to(resolved_repo):
            raise ValueError(f"discovery result escapes the repository: {path}")
        if path.is_file():
            paths.append(resolved.relative_to(resolved_repo).as_posix())
    paths.sort()
    return paths


def validate_surface_inventory(payload: dict[str, Any], repo_root: Path) -> list[str]:
    errors: list[str] = []
    qualification_profile = json.loads(
        (repo_root / "verification/profiles/release.json").read_text(encoding="utf-8")
    )
    selected_qualifications = {
        f"{selection['area']}:{suite}"
        for selection in qualification_profile.get("selected_areas", [])
        for suite in selection.get("suites", [])
    }
    if payload.get("schema_version") != 1:
        errors.append("schema_version must equal 1")
    entrypoints = payload.get("entrypoint_classes")
    if not isinstance(entrypoints, list):
        errors.append("entrypoint_classes must be a list")
    else:
        class_ids = {
            entry.get("id")
            for entry in entrypoints
            if isinstance(entry, dict) and isinstance(entry.get("id"), str)
        }
        if class_ids != REQUIRED_ENTRYPOINT_CLASSES:
            errors.append("entrypoint_classes must contain the exact required class set")
        if len(entrypoints) != len(class_ids):
            errors.append("entrypoint class ids must be unique")
        for entry in entrypoints:
            if not isinstance(entry, dict):
                errors.append("entrypoint class must be an object")
                continue
            for field in ("id", "command", "representative"):
                if not isinstance(entry.get(field), str) or not entry[field].strip():
                    errors.append(f"entrypoint class {entry!r} requires non-empty {field}")
            entry_id = entry.get("id")
            discoveries = entry.get("discoveries")
            valid_discoveries = (
                isinstance(discoveries, list)
                and all(isinstance(value, str) for value in discoveries)
            )
            discovery_set = set(discoveries) if valid_discoveries else set()
            if isinstance(entry_id, str) and (
                not valid_discoveries
                or discovery_set != REQUIRED_ENTRYPOINT_DISCOVERIES.get(entry_id)
                or len(discoveries) != len(discovery_set)
            ):
                errors.append(f"entrypoint class {entry_id} must name its exact discovery set")
            qualifications = entry.get("qualifications")
            valid_qualifications = (
                isinstance(qualifications, list)
                and all(isinstance(value, str) for value in qualifications)
            )
            qualification_set = set(qualifications) if valid_qualifications else set()
            if isinstance(entry_id, str) and (
                not valid_qualifications
                or qualification_set != REQUIRED_ENTRYPOINT_QUALIFICATIONS.get(entry_id)
                or len(qualifications) != len(qualification_set)
            ):
                errors.append(
                    f"entrypoint class {entry_id} must name its exact qualification set"
                )
            missing_qualifications = qualification_set.difference(selected_qualifications)
            if missing_qualifications:
                errors.append(
                    f"entrypoint class {entry_id} qualifications are absent from the release profile: "
                    f"{sorted(missing_qualifications)}"
                )
            representative = entry.get("representative")
            if isinstance(representative, str):
                resolved = (repo_root / representative).resolve()
                if (
                    Path(representative).is_absolute()
                    or ".." in Path(representative).parts
                    or not resolved.is_relative_to(repo_root.resolve())
                    or not resolved.is_file()
                ):
                    errors.append(f"entrypoint representative does not exist: {representative}")

    discoveries = payload.get("discoveries")
    if not isinstance(discoveries, list):
        errors.append("discoveries must be a list")
        return errors
    discovery_ids = {
        entry.get("id")
        for entry in discoveries
        if isinstance(entry, dict) and isinstance(entry.get("id"), str)
    }
    if discovery_ids != REQUIRED_DISCOVERY_IDS:
        errors.append("discoveries must contain the exact required discovery set")
    if len(discoveries) != len(discovery_ids):
        errors.append("discovery ids must be unique")
    for discovery in discoveries:
        if not isinstance(discovery, dict):
            errors.append("discovery entry must be an object")
            continue
        discovery_id = discovery.get("id", "<missing>")
        root = discovery.get("root")
        pattern = discovery.get("pattern")
        if (
            not isinstance(root, str)
            or not root.strip()
            or not isinstance(pattern, str)
            or not pattern.strip()
        ):
            errors.append(f"{discovery_id}: root and pattern must be non-empty strings")
            continue
        if Path(root).is_absolute() or ".." in Path(root).parts:
            errors.append(f"{discovery_id}: root must stay inside the repository")
            continue
        try:
            paths = discover_paths(repo_root, root, pattern)
        except ValueError as error:
            errors.append(f"{discovery_id}: {error}")
            continue
        if not paths:
            errors.append(f"{discovery_id}: discovery must not be empty")
        if discovery.get("expected_count") != len(paths):
            errors.append(
                f"{discovery_id}: expected_count={discovery.get('expected_count')} actual={len(paths)}"
            )
        actual_digest = path_set_digest(paths)
        if discovery.get("path_set_sha256") != actual_digest:
            errors.append(
                f"{discovery_id}: path_set_sha256 mismatch actual={actual_digest}"
            )
    return errors


def load_and_validate(path: Path, repo_root: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("surface inventory root must be an object")
    errors = validate_surface_inventory(payload, repo_root)
    if errors:
        raise ValueError("\n".join(errors))
    return payload


def expect_invalid(payload: dict[str, Any], repo_root: Path, expected: str) -> None:
    errors = validate_surface_inventory(payload, repo_root)
    if not any(expected in error for error in errors):
        raise AssertionError(f"expected {expected!r}, got {errors or ['<no errors>']}")


def run_self_test(payload: dict[str, Any], repo_root: Path) -> None:
    if errors := validate_surface_inventory(payload, repo_root):
        raise AssertionError("valid surface inventory rejected:\n" + "\n".join(errors))

    missing_class = copy.deepcopy(payload)
    missing_class["entrypoint_classes"].pop()
    expect_invalid(missing_class, repo_root, "exact required class set")

    missing_class_discovery = copy.deepcopy(payload)
    missing_class_discovery["entrypoint_classes"][0]["discoveries"] = []
    expect_invalid(missing_class_discovery, repo_root, "exact discovery set")

    missing_class_qualification = copy.deepcopy(payload)
    missing_class_qualification["entrypoint_classes"][0]["qualifications"] = []
    expect_invalid(missing_class_qualification, repo_root, "exact qualification set")

    missing_discovery = copy.deepcopy(payload)
    missing_discovery["discoveries"].pop()
    expect_invalid(missing_discovery, repo_root, "exact required discovery set")

    reduced_count = copy.deepcopy(payload)
    reduced_count["discoveries"][0]["expected_count"] -= 1
    expect_invalid(reduced_count, repo_root, "expected_count=")

    stale_digest = copy.deepcopy(payload)
    stale_digest["discoveries"][0]["path_set_sha256"] = "0" * 64
    expect_invalid(stale_digest, repo_root, "path_set_sha256 mismatch")

    escaped_root = copy.deepcopy(payload)
    escaped_root["discoveries"][0]["root"] = "../outside"
    expect_invalid(escaped_root, repo_root, "root must stay inside the repository")

    escaped_pattern = copy.deepcopy(payload)
    escaped_pattern["discoveries"][0]["pattern"] = "../**/*.sifr"
    expect_invalid(escaped_pattern, repo_root, "constrained repository glob")

    ambiguous_pattern = copy.deepcopy(payload)
    ambiguous_pattern["discoveries"][0]["pattern"] = "**/[ab]*.sifr"
    expect_invalid(ambiguous_pattern, repo_root, "constrained repository glob")

    missing_representative = copy.deepcopy(payload)
    missing_representative["entrypoint_classes"][0]["representative"] = "missing.sifr"
    expect_invalid(missing_representative, repo_root, "representative does not exist")


def evidence(payload: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    discoveries = []
    for discovery in payload["discoveries"]:
        paths = discover_paths(repo_root, discovery["root"], discovery["pattern"])
        discoveries.append(
            {
                "id": discovery["id"],
                "count": len(paths),
                "path_set_sha256": path_set_digest(paths),
            }
        )
    return {
        "entrypoint_classes": [
            {
                "id": entry["id"],
                "qualifications": entry["qualifications"],
            }
            for entry in sorted(payload["entrypoint_classes"], key=lambda entry: entry["id"])
        ],
        "discoveries": discoveries,
    }

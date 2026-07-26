#!/usr/bin/env python3
"""Collect one read-only stable qualification run into a canonical artifact index."""

from __future__ import annotations

import argparse
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance import GovernanceError, validate_qualification_artifact_index  # noqa: E402
from governance.common import (  # noqa: E402
    TARGETS,
    load_json_strict,
    require_commit,
    require_nonempty_string,
    require_object,
    require_positive_int,
    sha256_file,
    version_channel,
    write_canonical_json,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--submodules", required=True)
    parser.add_argument("--run-id", required=True, type=int)
    parser.add_argument("--run-attempt", required=True, type=int)
    parser.add_argument("--run-metadata", required=True)
    parser.add_argument("--run-artifacts", required=True)
    parser.add_argument("--artifact-root", required=True)
    parser.add_argument("--out", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        payload = collect_index(
            version=args.version,
            source_commit=args.source_commit,
            submodules_path=Path(args.submodules),
            run_id=args.run_id,
            run_attempt=args.run_attempt,
            run_metadata_path=Path(args.run_metadata),
            metadata_path=Path(args.run_artifacts),
            artifact_root=Path(args.artifact_root),
        )
        write_canonical_json(Path(args.out), payload, refuse_existing=True)
    except GovernanceError as exc:
        print(f"qualification-artifacts: {exc}", file=sys.stderr)
        return 2
    print(
        "qualification artifact collection ok: "
        f"version={args.version} source={args.source_commit} artifacts={len(payload['artifacts'])}"
    )
    return 0


def collect_index(
    *,
    version: str,
    source_commit: str,
    submodules_path: Path,
    run_id: int,
    run_attempt: int,
    run_metadata_path: Path,
    metadata_path: Path,
    artifact_root: Path,
) -> dict[str, Any]:
    if version_channel(version, "--version") != "stable":
        raise GovernanceError("--version must be exact stable SemVer")
    require_commit(source_commit, "--source-commit")
    require_positive_int(run_id, "--run-id")
    require_positive_int(run_attempt, "--run-attempt")
    submodules = require_object(load_json_strict(submodules_path), str(submodules_path))
    run_metadata = require_object(
        load_json_strict(run_metadata_path),
        str(run_metadata_path),
    )
    repository = require_object(
        run_metadata.get("repository"),
        f"{run_metadata_path}:repository",
    )
    if (
        run_metadata.get("id") != run_id
        or run_metadata.get("run_attempt") != run_attempt
        or run_metadata.get("event") != "workflow_dispatch"
        or run_metadata.get("name") != "release-qualification"
        or repository.get("full_name") != "sifr-lang/sifr"
    ):
        raise GovernanceError(
            f"{run_metadata_path}: run identity is not the canonical qualification workflow"
        )
    metadata = require_object(load_json_strict(metadata_path), str(metadata_path))
    raw_artifacts = metadata.get("artifacts")
    if not isinstance(raw_artifacts, list):
        raise GovernanceError(f"{metadata_path}: artifacts must be an array")

    prefix = f"sifr-stable-candidate-{version}-{source_commit}-"
    expected_names = {f"{prefix}{target}" for target in TARGETS}
    expected_names.update({f"{prefix}assemble", f"{prefix}editor"})
    by_name: dict[str, dict[str, Any]] = {}
    for position, value in enumerate(raw_artifacts):
        item = require_object(value, f"{metadata_path}:artifacts[{position}]")
        name = require_nonempty_string(item.get("name"), f"artifacts[{position}].name")
        if name in by_name:
            raise GovernanceError(
                f"{metadata_path}: duplicate workflow artifact name: {name}"
            )
        by_name[name] = item
    if set(by_name) != expected_names:
        missing = sorted(expected_names.difference(by_name))
        unexpected = sorted(set(by_name).difference(expected_names))
        raise GovernanceError(
            f"{metadata_path}: workflow artifact set mismatch "
            f"(missing={missing}, unexpected={unexpected})"
        )

    rows: list[dict[str, Any]] = []
    expiries: list[tuple[datetime, str]] = []
    for workflow_name in sorted(by_name):
        metadata_item = by_name[workflow_name]
        workflow_artifact_id = require_positive_int(
            metadata_item.get("id"),
            f"{workflow_name}.id",
        )
        if metadata_item.get("expired") is not False:
            raise GovernanceError(f"{workflow_name}: artifact is expired")
        expires_at = require_nonempty_string(
            metadata_item.get("expires_at"),
            f"{workflow_name}.expires_at",
        )
        created_at = require_nonempty_string(
            metadata_item.get("created_at"),
            f"{workflow_name}.created_at",
        )
        try:
            parsed_expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
            parsed_creation = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
        except ValueError as exc:
            raise GovernanceError(
                f"{workflow_name}: invalid artifact custody timestamps"
            ) from exc
        if parsed_expiry.tzinfo is None or parsed_creation.tzinfo is None:
            raise GovernanceError(
                f"{workflow_name}: artifact timestamps need timezones"
            )
        if parsed_expiry - parsed_creation != timedelta(days=30):
            raise GovernanceError(
                f"{workflow_name}: artifact retention is not exactly 30 days"
            )
        expiries.append((parsed_expiry, expires_at))
        workflow_run = require_object(
            metadata_item.get("workflow_run"),
            f"{workflow_name}.workflow_run",
        )
        if workflow_run.get("id") != run_id:
            raise GovernanceError(f"{workflow_name}: workflow run id mismatch")
        directory = artifact_root / workflow_name
        rows.extend(
            collect_container_rows(
                directory=directory,
                workflow_name=workflow_name,
                workflow_artifact_id=workflow_artifact_id,
                expires_at=expires_at,
                version=version,
                source_commit=source_commit,
            )
        )

    payload = {
        "schema_version": 2,
        "candidate_version": version,
        "source_commit": source_commit,
        "submodules": dict(sorted(submodules.items())),
        "workflow": {
            "run_id": run_id,
            "run_attempt": run_attempt,
            "repository": repository["full_name"],
            "retention_days": 30,
            "overwrite": False,
            "expires_at": min(expiries)[1],
        },
        "artifacts": sorted(rows, key=lambda row: row["id"]),
    }
    return validate_qualification_artifact_index(payload, require_unexpired=True)


def collect_container_rows(
    *,
    directory: Path,
    workflow_name: str,
    workflow_artifact_id: int,
    expires_at: str,
    version: str,
    source_commit: str,
) -> list[dict[str, Any]]:
    if not directory.is_dir():
        raise GovernanceError(
            f"{workflow_name}: downloaded artifact directory is missing"
        )
    files = sorted(path for path in directory.iterdir() if path.is_file())
    if any(path.is_symlink() for path in directory.iterdir()):
        raise GovernanceError(f"{workflow_name}: symlinks are not allowed")
    if any(path.is_dir() for path in directory.iterdir()):
        raise GovernanceError(
            f"{workflow_name}: nested artifact directories are not allowed"
        )
    suffix = workflow_name.removeprefix(
        f"sifr-stable-candidate-{version}-{source_commit}-"
    )
    specs = expected_file_specs(suffix=suffix, version=version, files=files)
    observed = {path.name for path in files}
    expected = {name for _, name, _ in specs}
    if observed != expected:
        raise GovernanceError(
            f"{workflow_name}: transported file set mismatch "
            f"(missing={sorted(expected - observed)}, unexpected={sorted(observed - expected)})"
        )
    rows: list[dict[str, Any]] = []
    for kind, name, target in specs:
        path = directory / name
        size = path.stat().st_size
        if size < 1:
            raise GovernanceError(f"{workflow_name}/{name}: artifact is empty")
        artifact_id = kind if target is None else f"{kind}-{target}"
        if kind == "report":
            artifact_id = (
                "editor-qualification-report"
                if target is None
                else f"qualification-report-{target}"
            )
        row: dict[str, Any] = {
            "id": artifact_id,
            "kind": kind,
            "name": name,
            "sha256": sha256_file(path),
            "size_bytes": size,
            "workflow_artifact_id": workflow_artifact_id,
            "workflow_artifact_name": workflow_name,
            "expires_at": expires_at,
        }
        if target is not None and kind != "report":
            row["target"] = target
        rows.append(row)
    return rows


def expected_file_specs(
    *,
    suffix: str,
    version: str,
    files: list[Path],
) -> list[tuple[str, str, str | None]]:
    if suffix in TARGETS:
        archive = f"sifr-{version}-{suffix}.tar.gz"
        return [
            ("binary-archive", archive, suffix),
            ("checksum", f"{archive}.sha256", suffix),
            ("sysroot", f"sifr-{version}-{suffix}-sysroot.tar.gz", suffix),
            ("report", f"qualification-{suffix}.json", suffix),
        ]
    if suffix == "assemble":
        return [
            ("checksums", "checksums.txt", None),
            ("installer", f"sifr-installer-{version}", None),
        ]
    if suffix == "editor":
        vsix_names = [path.name for path in files if path.suffix == ".vsix"]
        if len(vsix_names) != 1:
            raise GovernanceError(
                "editor qualification must transport exactly one VSIX"
            )
        return [
            ("report", "qualification-editor.json", None),
            ("vsix", vsix_names[0], None),
        ]
    raise GovernanceError(f"unexpected qualification artifact suffix: {suffix}")


if __name__ == "__main__":
    raise SystemExit(main())

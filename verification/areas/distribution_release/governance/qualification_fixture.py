"""Fixture-backed evidence construction for stable qualification tests and demos."""

from __future__ import annotations

import gzip
import hashlib
import importlib.util
import io
import json
import shutil
import subprocess
import tarfile
from pathlib import Path
from typing import Any

from .common import (
    BUILDERS,
    TARGETS,
    canonical_json_bytes,
    sha256_file,
    write_canonical_json,
)
from .planner import RUST_CLAIMS_SCHEMA_VERSION, stable_claim_ids
from .qualification_rust_fixture import rust_candidate_result
from .qualification_fixture_support import (
    command_output,
    configure_git,
    digest_text,
    git,
    git_output,
)
from .release_report import canonical_profile_digest, collect_submodules
from .schema_contracts import preview_index, release_plan, release_report

REPO_ROOT = Path(__file__).resolve().parents[4]
VERSION = "0.1.0"
PROFILE_SCHEMA_VERSION = 1


def create_fixture_source(root: Path, *, variant: str = "baseline") -> Path:
    source_root = root / "source"
    submodule_root = root / "editor-source"
    submodule_root.mkdir(parents=True)
    git(submodule_root, "init")
    configure_git(submodule_root)
    (submodule_root / "package.json").write_text(
        json.dumps(
            {
                "name": "sifr-vscode",
                "variant": "changed" if variant == "submodule" else "baseline",
            }
        )
        + "\n",
        encoding="utf-8",
    )
    git(submodule_root, "add", "package.json")
    git(submodule_root, "commit", "-m", "fixture editor")
    if variant == "submodule":
        (submodule_root / "variant.txt").write_text("changed\n", encoding="utf-8")
        git(submodule_root, "add", "variant.txt")
        git(submodule_root, "commit", "-m", "change fixture editor")

    source_root.mkdir()
    git(source_root, "init")
    configure_git(source_root)
    (source_root / ".gitignore").write_text("target/\n", encoding="utf-8")
    (source_root / "Cargo.lock").write_text(
        f"# fixture lock {variant if variant == 'lock' else 'baseline'}\n",
        encoding="utf-8",
    )
    write_source_contracts(source_root)
    git(
        source_root,
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        str(submodule_root),
        "editor_integrations",
    )
    gitmodules = source_root / ".gitmodules"
    gitmodules.write_text(
        gitmodules.read_text(encoding="utf-8").replace(
            str(submodule_root),
            "https://example.invalid/sifr-editor-fixture.git",
        ),
        encoding="utf-8",
    )
    if variant == "source":
        (source_root / "source-variant.txt").write_text("changed\n", encoding="utf-8")
    git(source_root, "add", ".")
    git(source_root, "commit", "-m", "fixture source")
    return source_root


def write_source_contracts(source_root: Path) -> None:
    profile = {
        "schema_version": PROFILE_SCHEMA_VERSION,
        "name": "release",
        "selected_areas": [
            {
                "area": "rust_interop",
                "suites": [
                    "matrix",
                    "tiers",
                    "compatibility-matrix",
                    "stale-drafts",
                    "stable-candidate",
                ],
            },
            {"area": "developer_tooling", "suites": ["full"]},
            {
                "area": "documentation",
                "suites": ["structure", "ga-release"],
            },
            {
                "area": "distribution_release",
                "suites": [
                    "full",
                    "qualification",
                    "evidence-custody",
                    "incident-governance",
                    "epoch-bootstrap",
                    "protected-drill",
                    "stable-prepare",
                    "stable-publish-primitives",
                ],
            },
        ],
    }
    files: dict[str, bytes] = {
        "verification/profiles/release.json": canonical_json_bytes(profile),
        (
            "verification/areas/rust_interop/data/"
            "rust_interop_compatibility_matrix.json"
        ): canonical_json_bytes(
            {"schema_version": RUST_CLAIMS_SCHEMA_VERSION, "rows": []}
        ),
        (
            "verification/areas/distribution_release/schemas/"
            "stable_site_release_facts.schema.json"
        ): canonical_json_bytes({"schema_version": 2, "fixture": True}),
        (
            "verification/areas/distribution_release/governance/release_plan.py"
        ): b"# fixture site-facts generator\n",
    }
    for relative, content in files.items():
        path = source_root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    for relative in (
        "scripts/distribution/generate_version_installer.sh",
        "scripts/distribution/verify_release_archive.py",
    ):
        source = REPO_ROOT / relative
        destination = source_root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())
    (source_root / "scripts/distribution/generate_version_installer.sh").chmod(0o755)


def build_evidence_bundle(
    *,
    source_root: Path,
    evidence_root: Path,
    result_root: Path,
    variant: str = "baseline",
    transition: str = "ga-activation",
    host_archive: Path | None = None,
    host_qualification_dir: Path | None = None,
) -> dict[str, Any]:
    source_root = source_root.resolve()
    evidence_root.mkdir(parents=True)
    result_root.mkdir(parents=True)
    source_commit = git_output(source_root, "rev-parse", "HEAD")
    submodules = collect_submodules(source_root)
    editor_commit = submodules["editor_integrations"]
    prefix = f"sifr-stable-candidate-{VERSION}-{source_commit}-"
    artifact_root = evidence_root / "artifacts"
    artifact_root.mkdir()
    reports: dict[str, dict[str, Any]] = {}
    host_target = detect_host_target(host_qualification_dir)

    for workflow_artifact_id, target in enumerate(TARGETS, start=1):
        container = artifact_root / f"{prefix}{target}"
        container.mkdir()
        if target == host_target:
            if host_archive is None or host_qualification_dir is None:
                raise ValueError("host qualification evidence is incomplete")
            copy_host_evidence(
                target=target,
                archive=host_archive,
                qualification_dir=host_qualification_dir,
                destination=container,
            )
            report = json.loads(
                (container / f"qualification-{target}.json").read_text(encoding="utf-8")
            )
        else:
            report = write_synthetic_target(
                container=container,
                target=target,
                source_commit=source_commit,
                variant=variant,
            )
        reports[target] = report

    assemble = artifact_root / f"{prefix}assemble"
    assemble.mkdir()
    installer = assemble / f"sifr-installer-{VERSION}"
    installer_inputs = evidence_root / "installer-inputs"
    installer_inputs.mkdir()
    for target in TARGETS:
        target_container = artifact_root / f"{prefix}{target}"
        for name in (
            f"sifr-{VERSION}-{target}.tar.gz",
            f"sifr-{VERSION}-{target}.tar.gz.sha256",
        ):
            shutil.copyfile(target_container / name, installer_inputs / name)
    subprocess.run(
        [
            str(
                source_root
                / "scripts"
                / "distribution"
                / "generate_version_installer.sh"
            ),
            "--version",
            VERSION,
            "--artifact-dir",
            str(installer_inputs),
            "--out",
            str(installer),
        ],
        cwd=source_root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    shutil.rmtree(installer_inputs)
    checksums = assemble / "checksums.txt"
    checksum_rows = []
    for target in TARGETS:
        container = artifact_root / f"{prefix}{target}"
        for name in (
            f"sifr-{VERSION}-{target}.tar.gz",
            f"sifr-{VERSION}-{target}.tar.gz.sha256",
            f"sifr-{VERSION}-{target}-sysroot.tar.gz",
        ):
            checksum_rows.append(f"{sha256_file(container / name)}  {name}")
    checksums.write_text("\n".join(sorted(checksum_rows)) + "\n", encoding="utf-8")

    editor = artifact_root / f"{prefix}editor"
    editor.mkdir()
    vsix = editor / "sifr-vscode-0.2.0.vsix"
    vsix.write_bytes(
        f"fixture-vsix:{'changed' if variant == 'vsix' else 'baseline'}\n".encode()
    )
    candidate_target = host_target or TARGETS[0]
    candidate_report_path = (
        artifact_root
        / f"{prefix}{candidate_target}"
        / f"qualification-{candidate_target}.json"
    )
    editor_report = {
        "schema_version": 2,
        "kind": "stable-editor-qualification",
        "source_commit": source_commit,
        "submodule_commit": editor_commit,
        "package_path": "editor_integrations/vscode",
        "package_version": "0.2.0",
        "compiler_compatibility": (
            ">=0.1.0,<0.2.0"
            if transition == "ga-activation"
            else ">=0.0.9,<0.2.0"
        ),
        "candidate_version": VERSION,
        "rollback_version": (
            "none"
            if transition == "ga-activation"
            else "0.0.9"
        ),
        "candidate_target": candidate_target,
        "candidate_binary_sha256": reports[candidate_target]["binary_sha256"],
        "target_report_sha256": sha256_file(candidate_report_path),
        "vsix_sha256": sha256_file(vsix),
        "vsix_package_smoke": "pass",
        "lsp_smoke": "pass",
        "marketplace_publish_plan": {
            "publisher": "sifr",
            "extension": "sifr-vscode",
            "version": "0.2.0",
            "package_path": "sifr-vscode-0.2.0.vsix",
            "vsix_sha256": sha256_file(vsix),
            "command": [
                "npx",
                "--no-install",
                "vsce",
                "publish",
                "--packagePath",
                "sifr-vscode-0.2.0.vsix",
            ],
            "rebuild": False,
            "execution_owner": "stable-publication-workflow",
            "status": "planned",
        },
        "status": "pass",
    }
    write_canonical_json(editor / "qualification-editor.json", editor_report)

    run_metadata_path, run_artifacts_path = write_workflow_metadata(
        evidence_root=evidence_root,
        artifact_root=artifact_root,
        prefix=prefix,
        source_commit=source_commit,
    )
    submodules_path = evidence_root / "submodules.json"
    write_canonical_json(submodules_path, submodules)
    qualification_index_path = evidence_root / "qualification-artifact-index.json"
    collector = load_collector()
    qualification = collector.collect_index(
        version=VERSION,
        source_commit=source_commit,
        submodules_path=submodules_path,
        run_id=42,
        run_attempt=1,
        run_metadata_path=run_metadata_path,
        metadata_path=run_artifacts_path,
        artifact_root=artifact_root,
    )
    write_canonical_json(qualification_index_path, qualification)

    result_paths = write_release_results(
        source_root=source_root,
        result_root=result_root,
    )
    release_report_path = evidence_root / "release-profile-report.json"
    report = build_release_report(
        source_root=source_root,
        source_commit=source_commit,
        submodules=submodules,
        result_paths=result_paths,
    )
    write_canonical_json(release_report_path, report)

    claims_path = evidence_root / "stable_support_claims.json"
    claims = stable_claims(variant=variant)
    write_canonical_json(claims_path, claims)
    documentation_report_path = evidence_root / "documentation-report.json"
    write_canonical_json(
        documentation_report_path,
        {
            "schema_version": 2,
            "kind": "stable-documentation-qualification",
            "report_id": "docs-fixture",
            "source_commit": source_commit,
            "suites": [
                {"name": "structure", "status": "pass", "total_variants": 1},
                {"name": "ga-release", "status": "pass", "total_variants": 1},
            ],
            "result_sha256": "8" * 64,
            "status": "pass",
        },
    )
    release_notes_path = evidence_root / "release-notes.md"
    release_notes_path.write_text("# Stable fixture notes\n", encoding="utf-8")
    active_index_path = evidence_root / "active-index.json"
    write_canonical_json(active_index_path, live_index(transition=transition))

    plan = build_plan(
        source_root=source_root,
        source_commit=source_commit,
        submodules=submodules,
        reports=reports,
        installer=installer,
        qualification_index_path=qualification_index_path,
        release_report_path=release_report_path,
        claims_path=claims_path,
        rust_result_path=result_paths["rust_interop"],
        documentation_report_path=documentation_report_path,
        release_notes_path=release_notes_path,
        editor_report_path=editor / "qualification-editor.json",
        vsix=vsix,
        transition=transition,
    )
    plan_spec_path = evidence_root / "plan-spec.json"
    write_canonical_json(plan_spec_path, plan)
    return {
        "source_root": source_root,
        "source_ref": source_commit,
        "active_index": active_index_path,
        "release_report": release_report_path,
        "qualification_index": qualification_index_path,
        "artifact_root": artifact_root,
        "stable_support_claims": claims_path,
        "rust_validation_report": result_paths["rust_interop"],
        "documentation_report": documentation_report_path,
        "release_notes": release_notes_path,
        "plan_spec": plan_spec_path,
    }


def write_synthetic_target(
    *,
    container: Path,
    target: str,
    source_commit: str,
    variant: str,
) -> dict[str, Any]:
    archive = container / f"sifr-{VERSION}-{target}.tar.gz"
    archive_marker = (
        "changed"
        if (variant == "target-artifact" and target == TARGETS[0])
        else "baseline"
    )
    sysroot_marker = (
        "changed" if (variant == "sysroot" and target == TARGETS[0]) else "baseline"
    )
    binary_bytes = f"fixture-binary:{target}:{archive_marker}\n".encode()
    sysroot_files = {
        "Cargo.toml": b"[workspace]\nmembers = []\n",
        "Cargo.lock": b"# fixture lock\n",
        ".cargo/config.toml": b"[net]\noffline = true\n",
        "crates/sifr_runtime/Cargo.toml": b'[package]\nname = "sifr_runtime"\n',
        "crates/sifr_stdlib/Cargo.toml": b'[package]\nname = "sifr_stdlib"\n',
        "lib/sifr/stdlib/sifr/fixture.sifr": b"def fixture() -> int:\n    return 1\n",
        "lib/sifr/stdlib/_sifr/fixture.sifr": b"def fixture() -> int:\n    return 1\n",
        "vendor/fixture.txt": f"vendor:{sysroot_marker}\n".encode(),
    }
    sysroot_content_sha = sysroot_digest(sysroot_files)
    manifest = (
        f'"schema-version" = 1\n'
        f'"sifr-version" = "{VERSION}"\n'
        f'"target-triple" = "{target}"\n'
        f'"built-by-compiler-commit" = "fixture"\n'
        f'"sysroot-content-sha256" = "{sysroot_content_sha}"\n'
        f'"cargo-lock-sha256" = "{hashlib.sha256(sysroot_files["Cargo.lock"]).hexdigest()}"\n'
    ).encode()
    write_deterministic_archive(
        archive,
        {
            "bin/sifr": binary_bytes,
            "sysroot.toml": manifest,
            **sysroot_files,
        },
    )
    checksum = Path(f"{archive}.sha256")
    checksum.write_text(sha256_file(archive) + "\n", encoding="utf-8")
    sysroot = container / f"sifr-{VERSION}-{target}-sysroot.tar.gz"
    sysroot.write_bytes(f"sysroot:{target}:{sysroot_marker}\n".encode())
    report = {
        "schema_version": 2,
        "kind": "stable-target-qualification",
        "candidate_version": VERSION,
        "source_commit": source_commit,
        "target": target,
        "builder": BUILDERS[target],
        "binary_sha256": hashlib.sha256(binary_bytes).hexdigest(),
        "sysroot_sha256": sysroot_content_sha,
        "archive_sha256": sha256_file(archive),
        "checksum_sha256": sha256_file(checksum),
        "sysroot_bundle_sha256": sha256_file(sysroot),
        "sifr_version": VERSION,
        "installer_version": VERSION,
        "receipt_channel": "stable",
        "sysroot_version": VERSION,
        "sysroot_target": target,
        "smoke_status": "pass",
        "self_version_sha256": digest_text(f"self-version:{target}"),
    }
    write_canonical_json(container / f"qualification-{target}.json", report)
    return report


def sysroot_digest(files: dict[str, bytes]) -> str:
    digest = hashlib.sha256()
    for name in sorted(files):
        digest.update(name.encode())
        digest.update(b"\n")
        digest.update(hashlib.sha256(files[name]).hexdigest().encode())
        digest.update(b"\n")
    return digest.hexdigest()


def write_deterministic_archive(path: Path, files: dict[str, bytes]) -> None:
    with path.open("wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as archive:
                for name, content in sorted(files.items()):
                    info = tarfile.TarInfo(name)
                    info.size = len(content)
                    info.mtime = 0
                    info.mode = 0o755 if name == "bin/sifr" else 0o644
                    archive.addfile(info, io.BytesIO(content))


def copy_host_evidence(
    *,
    target: str,
    archive: Path,
    qualification_dir: Path,
    destination: Path,
) -> None:
    expected_archive = f"sifr-{VERSION}-{target}.tar.gz"
    if archive.name != expected_archive:
        raise ValueError(f"unexpected host archive: {archive}")
    for path in (
        archive,
        Path(f"{archive}.sha256"),
        qualification_dir / f"sifr-{VERSION}-{target}-sysroot.tar.gz",
        qualification_dir / f"qualification-{target}.json",
    ):
        if not path.is_file():
            raise ValueError(f"missing host qualification evidence: {path}")
        shutil.copy2(path, destination / path.name)


def detect_host_target(qualification_dir: Path | None) -> str | None:
    if qualification_dir is None:
        return None
    matches = [
        target
        for target in TARGETS
        if (qualification_dir / f"qualification-{target}.json").is_file()
    ]
    if len(matches) != 1:
        raise ValueError(
            "host qualification directory must contain exactly one target report"
        )
    return matches[0]


def write_workflow_metadata(
    *,
    evidence_root: Path,
    artifact_root: Path,
    prefix: str,
    source_commit: str,
) -> tuple[Path, Path]:
    run_metadata_path = evidence_root / "run-metadata.json"
    write_canonical_json(
        run_metadata_path,
        {
            "id": 42,
            "run_attempt": 1,
            "head_sha": source_commit,
            "event": "workflow_dispatch",
            "name": f"Qualify stable candidate 0.1.0 at {source_commit}",
            "path": ".github/workflows/release-qualification.yml",
            "repository": {"full_name": "sifr-lang/sifr"},
        },
    )
    artifacts = []
    for artifact_id, suffix in enumerate([*TARGETS, "assemble", "editor"], start=1):
        artifacts.append(
            {
                "id": artifact_id,
                "name": f"{prefix}{suffix}",
                "expired": False,
                "created_at": "2098-12-02T00:00:05Z",
                "expires_at": "2099-01-01T00:00:00Z",
                "workflow_run": {"id": 42},
            }
        )
    run_artifacts_path = evidence_root / "run-artifacts.json"
    write_canonical_json(run_artifacts_path, {"artifacts": artifacts})
    return run_metadata_path, run_artifacts_path


def write_release_results(
    *,
    source_root: Path,
    result_root: Path,
) -> dict[str, Path]:
    rust_result = rust_candidate_result()
    paths = {
        "rust_interop": result_root / "rust-interop-release-results.json",
        "developer_tooling": result_root / "developer-tooling-release-results.json",
        "documentation": result_root / "documentation-release-results.json",
        "distribution_release": result_root
        / "distribution-release-release-results.json",
    }
    write_canonical_json(paths["rust_interop"], rust_result)
    for area in ("developer_tooling", "documentation", "distribution_release"):
        write_canonical_json(paths[area], {"area": area, "status": "pass"})
    for path in paths.values():
        if not path.resolve().is_relative_to(source_root.resolve()):
            raise ValueError(
                "release result fixtures must remain inside the source checkout"
            )
    return paths


def build_release_report(
    *,
    source_root: Path,
    source_commit: str,
    submodules: dict[str, str],
    result_paths: dict[str, Path],
) -> dict[str, Any]:
    report = release_report()
    profile_digest = canonical_profile_digest(
        source_root / "verification" / "profiles" / "release.json"
    )
    report["report_id"] = f"release-{source_commit[:12]}-{profile_digest[:12]}"
    report["source"] = {
        "commit": source_commit,
        "clean": True,
        "unresolved": False,
        "submodules": submodules,
    }
    report["profile"]["manifest_sha256"] = profile_digest
    source_profile = json.loads(
        (source_root / "verification" / "profiles" / "release.json").read_text(
            encoding="utf-8"
        )
    )
    expanded: dict[str, set[str]] = {}
    for selection in source_profile["selected_areas"]:
        expanded.setdefault(selection["area"], set()).update(selection["suites"])
    if "full" in expanded.get("developer_tooling", set()):
        expanded["developer_tooling"].add("editor-release")
    report["profile"]["expanded_selected_areas"] = [
        {"area": area, "suites": sorted(suites)}
        for area, suites in sorted(expanded.items())
    ]
    report["toolchain"]["rustc"] = command_output(source_root, "rustc", "--version")
    report["toolchain"]["cargo"] = command_output(source_root, "cargo", "--version")
    digests = {area: sha256_file(path) for area, path in result_paths.items()}
    step_areas = {
        "rust_interop_checks": "rust_interop",
        "developer_tooling_checks": "developer_tooling",
        "documentation_checks": "documentation",
        "distribution_validation": "distribution_release",
    }
    for step in report["steps"]:
        digest = digests[step_areas[step["name"]]]
        for suite in step["suite_results"]:
            suite["result_artifact_sha256"] = digest
    report["result_artifacts"] = [
        {
            "path": path.resolve().relative_to(source_root.resolve()).as_posix(),
            "sha256": digests[area],
        }
        for area, path in sorted(result_paths.items())
    ]
    return report


def build_plan(
    *,
    source_root: Path,
    source_commit: str,
    submodules: dict[str, str],
    reports: dict[str, dict[str, Any]],
    installer: Path,
    qualification_index_path: Path,
    release_report_path: Path,
    claims_path: Path,
    rust_result_path: Path,
    documentation_report_path: Path,
    release_notes_path: Path,
    editor_report_path: Path,
    vsix: Path,
    transition: str,
) -> dict[str, Any]:
    plan = release_plan()
    plan["plan_id"] = f"stable-{VERSION}-{source_commit[:12]}"
    plan["transition"] = transition
    plan["source_commit"] = source_commit
    plan["submodules"] = submodules
    plan["cargo_lock_sha256"] = sha256_file(source_root / "Cargo.lock")
    plan["toolchain"] = {
        "rustc": command_output(source_root, "rustc", "--version"),
        "cargo": command_output(source_root, "cargo", "--version"),
        "profile_manifest_sha256": canonical_profile_digest(
            source_root / "verification" / "profiles" / "release.json"
        ),
    }
    plan["installer_sha256"] = sha256_file(installer)
    if transition == "normal":
        predecessor = {
            "version": "0.0.9",
            "status": "active",
            "plan_sha256": "9" * 64,
        }
        plan["expected_stable_predecessor"] = predecessor
        plan["rollback_target"] = {
            "version": predecessor["version"],
            "plan_sha256": predecessor["plan_sha256"],
        }
    plan["desired_release"]["source_commit"] = source_commit
    plan["desired_release"]["installer_sha256"] = plan["installer_sha256"]
    target_rows = []
    for target in TARGETS:
        report = reports[target]
        target_rows.append(
            {
                field: report[field]
                for field in (
                    "target",
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
                )
            }
        )
        target_rows[-1]["triple"] = target_rows[-1].pop("target")
        plan["desired_release"]["targets"][target] = {
            "artifact_sha256": report["archive_sha256"],
            "sysroot_content_sha256": report["sysroot_sha256"],
        }
    plan["targets"] = target_rows
    release_report_payload = json.loads(release_report_path.read_text(encoding="utf-8"))
    plan["release_profile_report"] = {
        "id": release_report_payload["report_id"],
        "sha256": sha256_file(release_report_path),
    }
    plan["qualification_artifact_index"] = {
        "id": "qualification-42-1",
        "sha256": sha256_file(qualification_index_path),
    }
    plan["rust_interop"] = {
        "compatibility_matrix_sha256": sha256_file(
            source_root / "verification/areas/rust_interop/data/"
            "rust_interop_compatibility_matrix.json"
        ),
        "stable_support_claims_sha256": sha256_file(claims_path),
        "advertised_claim_ids": stable_claim_ids(
            json.loads(claims_path.read_text(encoding="utf-8"))
        ),
        "validation_report_sha256": sha256_file(rust_result_path),
    }
    plan["documentation_report"] = {
        "id": "docs-fixture",
        "sha256": sha256_file(documentation_report_path),
    }
    plan["release_notes_sha256"] = sha256_file(release_notes_path)
    plan["site"]["facts_schema_sha256"] = sha256_file(
        source_root / "verification/areas/distribution_release/schemas/"
        "stable_site_release_facts.schema.json"
    )
    plan["site"]["facts_generator_sha256"] = sha256_file(
        source_root
        / "verification/areas/distribution_release/governance/release_plan.py"
    )
    editor_report = json.loads(editor_report_path.read_text(encoding="utf-8"))
    plan["vscode"].update(
        {
            "version": editor_report["package_version"],
            "vsix_sha256": sha256_file(vsix),
            "compiler_compatibility": editor_report["compiler_compatibility"],
            "validation_report_sha256": sha256_file(editor_report_path),
        }
    )
    return plan


def live_index(*, transition: str) -> dict[str, Any]:
    index = preview_index()
    if transition == "ga-activation":
        return index
    if transition != "normal":
        raise ValueError(f"unsupported qualification fixture transition: {transition}")
    predecessor = copy_release_record(index["releases"]["0.1.0-beta.2"])
    predecessor["channel"] = "stable"
    index["generation"] = 8
    index["ga_status"] = "active"
    index["channels"]["stable"] = "0.0.9"
    index["releases"]["0.0.9"] = predecessor
    return index


def copy_release_record(record: dict[str, Any]) -> dict[str, Any]:
    return json.loads(json.dumps(record))


def stable_claims(*, variant: str) -> dict[str, Any]:
    claims = [
        {
            "id": "direct_crate_fixture",
            "category": "supported",
            "execution_kind": "cargo-probe",
            "capability": "fixture direct crate",
        },
        {
            "id": "bridge_fixture",
            "category": "supported-through-bridge",
            "execution_kind": "contract-only",
            "capability": "fixture bridge",
        },
    ]
    if variant == "rust-claims":
        claims.append(
            {
                "id": "diagnostic_fixture",
                "category": "unsupported-by-design",
                "execution_kind": "compiler-diagnostic",
                "capability": "fixture diagnostic",
            }
        )
    return {
        "schema_version": RUST_CLAIMS_SCHEMA_VERSION,
        "role": "compatibility-derived-release-plan-input",
        "source_compatibility_matrix": (
            "verification/areas/rust_interop/data/"
            "rust_interop_compatibility_matrix.json"
        ),
        "public_document": "docs/rust-interop.mdx",
        "runtime_deferrals": ["fixture_runtime_deferral"],
        "claims": claims,
    }




def load_collector() -> Any:
    path = REPO_ROOT / "scripts" / "distribution" / "collect_qualification_artifacts.py"
    spec = importlib.util.spec_from_file_location(
        "qualification_collector_fixture", path
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load qualification artifact collector")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

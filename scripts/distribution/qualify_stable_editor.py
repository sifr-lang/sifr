#!/usr/bin/env python3
"""Qualify one packaged VS Code extension against an exact stable Sifr binary."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import stat
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
STABLE_RE = re.compile(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$")
RANGE_RE = re.compile(
    r"^>=([0-9]+)\.([0-9]+)\.([0-9]+),<([0-9]+)\.([0-9]+)\.([0-9]+)$"
)


class EditorQualificationError(ValueError):
    """Stable editor qualification failed."""


def canonical_json_text(value: Any) -> str:
    return (
        json.dumps(
            value,
            ensure_ascii=False,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_output(command: list[str], *, cwd: Path) -> str:
    result = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).strip()
        raise EditorQualificationError(
            f"{' '.join(command)} failed with exit {result.returncode}: {detail}"
        )
    return result.stdout.strip()


def candidate_lsp_command(candidate_binary: Path) -> str:
    return shlex.join([str(candidate_binary), "lsp", "--stdio"])


def parse_version(value: str, label: str) -> tuple[int, int, int]:
    match = STABLE_RE.fullmatch(value)
    if match is None:
        raise EditorQualificationError(f"{label} must be exact stable SemVer")
    return tuple(int(part) for part in match.groups())


def range_contains(expression: Any, version: tuple[int, int, int]) -> bool:
    if not isinstance(expression, str):
        return False
    match = RANGE_RE.fullmatch(expression)
    if match is None:
        return False
    parts = tuple(int(part) for part in match.groups())
    return parts[:3] <= version < parts[3:]


def validate_archive_members(archive: zipfile.ZipFile) -> None:
    names: set[str] = set()
    allowed_root_files = {"extension.vsixmanifest", "[Content_Types].xml"}
    for member in archive.infolist():
        path = PurePosixPath(member.filename)
        if (
            not member.filename
            or path.is_absolute()
            or ".." in path.parts
            or (
                path.parts[0] != "extension"
                and member.filename not in allowed_root_files
            )
        ):
            raise EditorQualificationError(
                f"VSIX contains unsafe or non-extension path: {member.filename!r}"
            )
        if member.filename in names:
            raise EditorQualificationError(
                f"VSIX contains duplicate path: {member.filename}"
            )
        names.add(member.filename)
        unix_mode = member.external_attr >> 16
        if stat.S_ISLNK(unix_mode):
            raise EditorQualificationError(
                f"VSIX contains unsupported symlink: {member.filename}"
            )
    required = {
        "extension/package.json",
        "extension/out/src/extension.js",
        "extension/out/src/config.js",
        "extension/node_modules/vscode-languageclient/package.json",
    }
    missing = sorted(required - names)
    if missing:
        raise EditorQualificationError(
            f"VSIX is missing packaged runtime files: {missing}"
        )


def load_packaged_manifest(
    archive: zipfile.ZipFile,
    *,
    candidate_version: tuple[int, int, int],
    rollback_version: tuple[int, int, int] | None,
) -> dict[str, Any]:
    try:
        payload = json.loads(archive.read("extension/package.json"))
    except (KeyError, UnicodeError, json.JSONDecodeError) as exc:
        raise EditorQualificationError(f"VSIX package.json is invalid: {exc}") from exc
    if not isinstance(payload, dict):
        raise EditorQualificationError("VSIX package.json must be an object")
    if payload.get("name") != "sifr-vscode" or payload.get("publisher") != "sifr":
        raise EditorQualificationError("VSIX Marketplace identity must be sifr.sifr-vscode")
    package_version = payload.get("version")
    if not isinstance(package_version, str) or STABLE_RE.fullmatch(package_version) is None:
        raise EditorQualificationError("VSIX package version must be exact SemVer")
    compatibility = payload.get("sifrCompilerCompatibility")
    if not range_contains(compatibility, candidate_version):
        raise EditorQualificationError(
            "VSIX compiler compatibility range does not contain the candidate"
        )
    if rollback_version is not None and not range_contains(
        compatibility,
        rollback_version,
    ):
        raise EditorQualificationError(
            "VSIX compiler compatibility range does not contain the rollback target"
        )
    return payload


def validate_target_report(
    path: Path,
    *,
    candidate_version: str,
    source_commit: str,
    candidate_binary: Path,
) -> tuple[str, str]:
    try:
        raw = path.read_bytes()
        payload = json.loads(raw)
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise EditorQualificationError(
            f"target qualification report is invalid: {exc}"
        ) from exc
    if not isinstance(payload, dict):
        raise EditorQualificationError(
            "target qualification report must be an object"
        )
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
    if set(payload) != required:
        raise EditorQualificationError(
            "target qualification report fields are not exact"
        )
    if (
        payload["schema_version"] != 2
        or payload["kind"] != "stable-target-qualification"
        or payload["candidate_version"] != candidate_version
        or payload["source_commit"] != source_commit
        or payload["sifr_version"] != candidate_version
        or payload["installer_version"] != candidate_version
        or payload["sysroot_version"] != candidate_version
        or payload["receipt_channel"] != "stable"
        or payload["smoke_status"] != "pass"
        or payload["target"] != payload["sysroot_target"]
    ):
        raise EditorQualificationError(
            "target qualification report identity did not pass"
        )
    if payload["binary_sha256"] != sha256_file(candidate_binary):
        raise EditorQualificationError(
            "candidate binary does not match target qualification evidence"
        )
    canonical = canonical_json_text(payload).encode()
    if raw != canonical:
        raise EditorQualificationError(
            "target qualification report is not canonical JSON"
        )
    return str(payload["target"]), hashlib.sha256(raw).hexdigest()


def qualify(args: argparse.Namespace) -> dict[str, Any]:
    source_root = Path(args.source_root).resolve()
    candidate_binary = Path(args.candidate_binary).resolve()
    target_report = Path(args.target_report).resolve()
    vsix = Path(args.vsix).resolve()
    out = Path(args.out).resolve()
    if not COMMIT_RE.fullmatch(args.source_commit):
        raise EditorQualificationError("source commit must be lowercase 40-hex")
    candidate_version = parse_version(args.candidate_version, "candidate version")
    rollback_version = (
        None
        if args.rollback_version == "none"
        else parse_version(args.rollback_version, "rollback version")
    )
    if out.exists():
        raise EditorQualificationError("output path already exists")
    if not candidate_binary.is_file() or not os.access(candidate_binary, os.X_OK):
        raise EditorQualificationError("candidate binary must exist and be executable")
    if not vsix.is_file():
        raise EditorQualificationError("VSIX does not exist")
    source_head = command_output(["git", "rev-parse", "HEAD"], cwd=source_root)
    if source_head != args.source_commit:
        raise EditorQualificationError("source checkout does not match source commit")
    submodule_commit = command_output(
        ["git", "rev-parse", "HEAD:editor_integrations"],
        cwd=source_root,
    )
    if not COMMIT_RE.fullmatch(submodule_commit):
        raise EditorQualificationError("editor_integrations gitlink is invalid")
    actual_version = command_output([str(candidate_binary), "--version"], cwd=source_root)
    if actual_version != f"sifr {args.candidate_version}":
        raise EditorQualificationError(
            f"candidate binary reported {actual_version!r}, expected sifr {args.candidate_version}"
        )
    candidate_target, target_report_sha256 = validate_target_report(
        target_report,
        candidate_version=args.candidate_version,
        source_commit=args.source_commit,
        candidate_binary=candidate_binary,
    )

    try:
        with zipfile.ZipFile(vsix) as archive:
            validate_archive_members(archive)
            package = load_packaged_manifest(
                archive,
                candidate_version=candidate_version,
                rollback_version=rollback_version,
            )
            expected_name = f"sifr-vscode-{package['version']}.vsix"
            if vsix.name != expected_name:
                raise EditorQualificationError(
                    f"VSIX name must be {expected_name}, found {vsix.name}"
                )
            with tempfile.TemporaryDirectory(prefix="sifr-vsix-install-") as directory:
                install_root = Path(directory)
                archive.extractall(install_root)
                config_path = install_root / "extension" / "out" / "src" / "config.js"
                node_program = (
                    "const c=require(process.argv[1]);"
                    "const v=c.serverCommand(process.argv[2]);"
                    "if(v.command!==process.argv[2]||"
                    "JSON.stringify(v.args)!=='[\"lsp\",\"--stdio\"]')process.exit(2);"
                )
                command_output(
                    ["node", "-e", node_program, str(config_path), str(candidate_binary)],
                    cwd=install_root,
                )
    except (OSError, zipfile.BadZipFile) as exc:
        raise EditorQualificationError(f"VSIX cannot be installed: {exc}") from exc

    environment = dict(os.environ)
    environment["SIFR_LSP_COMMAND"] = candidate_lsp_command(candidate_binary)
    try:
        smoke = subprocess.run(
            [
                sys.executable,
                str(
                    source_root
                    / "verification"
                    / "areas"
                    / "developer_tooling"
                    / "lsp_protocol_smoke.py"
                ),
                "--candidate-smoke",
            ],
            cwd=source_root,
            env=environment,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=120,
        )
    except subprocess.TimeoutExpired as exc:
        raise EditorQualificationError(
            "exact candidate LSP smoke exceeded the governed 120s timeout"
        ) from exc
    if smoke.returncode != 0:
        detail = (smoke.stderr or smoke.stdout).strip()
        raise EditorQualificationError(
            f"exact candidate LSP smoke failed with exit {smoke.returncode}: {detail}"
        )

    payload = {
        "schema_version": 2,
        "kind": "stable-editor-qualification",
        "source_commit": args.source_commit,
        "submodule_commit": submodule_commit,
        "package_path": "editor_integrations/vscode",
        "package_version": package["version"],
        "compiler_compatibility": package["sifrCompilerCompatibility"],
        "candidate_version": args.candidate_version,
        "rollback_version": args.rollback_version,
        "candidate_target": candidate_target,
        "candidate_binary_sha256": sha256_file(candidate_binary),
        "target_report_sha256": target_report_sha256,
        "vsix_sha256": sha256_file(vsix),
        "vsix_package_smoke": "pass",
        "lsp_smoke": "pass",
        "marketplace_publish_plan": {
            "publisher": package["publisher"],
            "extension": package["name"],
            "version": package["version"],
            "package_path": vsix.name,
            "vsix_sha256": sha256_file(vsix),
            "command": [
                "npx",
                "--no-install",
                "vsce",
                "publish",
                "--packagePath",
                vsix.name,
            ],
            "rebuild": False,
            "execution_owner": "stable-publication-workflow",
            "status": "planned",
        },
        "status": "pass",
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        canonical_json_text(payload),
        encoding="utf-8",
    )
    return payload


def run_self_test() -> None:
    if not range_contains(">=0.1.0,<0.2.0", (0, 1, 0)):
        raise EditorQualificationError("valid compatibility range was rejected")
    invalid_ranges = (
        ">=0.1.1,<0.2.0",
        ">=0.1.0 <0.2.0",
        "0.1.0",
        None,
    )
    if any(range_contains(value, (0, 1, 0)) for value in invalid_ranges):
        raise EditorQualificationError("invalid compatibility range was accepted")
    spaced_binary = Path("/tmp/sifr candidate/bin/sifr")
    if shlex.split(candidate_lsp_command(spaced_binary)) != [
        str(spaced_binary),
        "lsp",
        "--stdio",
    ]:
        raise EditorQualificationError("candidate LSP command quoting is unsafe")
    with tempfile.TemporaryDirectory(prefix="sifr-editor-self-test-") as directory:
        root = Path(directory)
        archive_path = root / "unsafe.vsix"
        with zipfile.ZipFile(archive_path, "w") as archive:
            archive.writestr("../escape", "bad")
        with zipfile.ZipFile(archive_path) as archive:
            try:
                validate_archive_members(archive)
            except EditorQualificationError:
                pass
            else:
                raise EditorQualificationError("unsafe VSIX path was accepted")
        binary = root / "sifr"
        binary.write_bytes(b"candidate")
        report_path = root / "qualification-target.json"
        report = {
            "schema_version": 2,
            "kind": "stable-target-qualification",
            "candidate_version": "0.1.0",
            "source_commit": "e" * 40,
            "target": "x86_64-unknown-linux-gnu",
            "builder": "ubuntu-24.04",
            "binary_sha256": sha256_file(binary),
            "sysroot_sha256": "a" * 64,
            "archive_sha256": "b" * 64,
            "checksum_sha256": "c" * 64,
            "sysroot_bundle_sha256": "d" * 64,
            "sifr_version": "0.1.0",
            "installer_version": "0.1.0",
            "receipt_channel": "stable",
            "sysroot_version": "0.1.0",
            "sysroot_target": "x86_64-unknown-linux-gnu",
            "smoke_status": "pass",
            "self_version_sha256": "f" * 64,
        }
        report_path.write_text(
            canonical_json_text(report),
            encoding="utf-8",
        )
        validate_target_report(
            report_path,
            candidate_version="0.1.0",
            source_commit="e" * 40,
            candidate_binary=binary,
        )
        report["binary_sha256"] = "1" * 64
        report_path.write_text(
            canonical_json_text(report),
            encoding="utf-8",
        )
        try:
            validate_target_report(
                report_path,
                candidate_version="0.1.0",
                source_commit="e" * 40,
                candidate_binary=binary,
            )
        except EditorQualificationError:
            pass
        else:
            raise EditorQualificationError(
                "mismatched candidate target report was accepted"
            )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--source-root", default=str(REPO_ROOT))
    parser.add_argument("--source-commit")
    parser.add_argument("--candidate-version")
    parser.add_argument("--rollback-version", default="none")
    parser.add_argument("--candidate-binary")
    parser.add_argument("--target-report")
    parser.add_argument("--vsix")
    parser.add_argument("--out")
    args = parser.parse_args()
    if not args.self_test:
        missing = [
            name
            for name in (
                "source_commit",
                "candidate_version",
                "candidate_binary",
                "target_report",
                "vsix",
                "out",
            )
            if getattr(args, name) is None
        ]
        if missing:
            parser.error(f"missing required arguments: {', '.join(missing)}")
    return args


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            run_self_test()
            print("stable editor qualification self-test: PASS")
        else:
            payload = qualify(args)
            print(
                "stable editor qualification: PASS "
                f"extension={payload['package_version']} "
                f"candidate={payload['candidate_version']}"
            )
    except (EditorQualificationError, OSError, UnicodeError) as exc:
        print(f"stable editor qualification: FAIL: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

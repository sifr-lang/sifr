"""Installed sysroot release certification area adapter."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
from pathlib import Path, PurePosixPath
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
HEAVY_FIXTURE_PATH = AREA_ROOT / "fixtures" / "stdlib_heavy_release_smoke.sifr"
COMPILE_FIXTURE_PATH = AREA_ROOT / "fixtures" / "stdlib_compile_release_smoke.sifr"
BOUNDARY_FIXTURE_PATH = AREA_ROOT / "fixtures" / "stdlib_boundary_recertification.sifr"
BOUNDARY_DEPENDENCY_SNAPSHOT_PATH = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "stdlib_parity"
    / "data"
    / "stdlib_compiler_boundary_dependency_snapshot.json"
)
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "sysroot-release-results.json"
ACTUAL_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "sysroot_release"
RELEASE_VERSION = "0.1.0-beta.1300"
BUILT_ARCHIVES: dict[str, Path] = {}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable sysroot release result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit a legacy verification summary line for direct area invocations.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("sysroot_release area does not support --bless")
    selected = select_suites(set(args.suite))

    print("Running sysroot release verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    ACTUAL_ROOT.mkdir(parents=True, exist_ok=True)
    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "sysroot_release",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": total_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)

    if total_failures:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={total_failures}, non_blocking_failures=0",
            file=sys.stderr,
            flush=True,
        )
        return 1
    prefix = "verification ok" if args.hardening_summary else "sysroot release verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
    )
    return 0


def select_suites(requested: set[str]) -> list[str]:
    available = {
        "boundary-equivalence",
        "host-installed-smoke",
        "host-installed-stdlib-heavy",
        "path-leakage-self-test",
    }
    if requested:
        missing = sorted(requested.difference(available))
        if missing:
            raise SystemExit(f"unknown sysroot_release suite filter(s): {', '.join(missing)}")
        return [suite for suite in sorted(available) if suite in requested]
    return ["host-installed-smoke"]


def run_suite(suite: str) -> dict[str, Any]:
    started = time.perf_counter()
    if suite == "path-leakage-self-test":
        status, mismatches = run_path_leakage_self_test()
    elif suite == "boundary-equivalence":
        status, mismatches = run_boundary_equivalence()
    elif suite == "host-installed-stdlib-heavy":
        status, mismatches = run_host_installed_stdlib_heavy()
    else:
        status, mismatches = run_host_installed_smoke()
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    label = "pass" if status == 0 else "fail"
    print(
        f"[sifr-case-timing] bucket=sysroot_release case={suite} "
        f"elapsed_ms={int(elapsed_ms)} status={label}",
        flush=True,
    )
    return {
        "name": suite,
        "owner": "release/distribution",
        "blocking": True,
        "runner": "sysroot-release",
        "cases": [
            {
                "id": suite,
                "entry": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
                "command": f"sysroot-release:{suite}",
                "variants": [
                    {
                        "label": suite,
                        "argv": ["sysroot_release", suite],
                        "status": label,
                        "mismatches": mismatches,
                        "expected_exit_code": 0,
                        "actual_exit_code": status,
                        "duration_ms": round(elapsed_ms, 3),
                    }
                ],
            }
        ],
        "failed_cases": 0 if status == 0 else 1,
        "total_variants": 1,
        "total_failures": 0 if status == 0 else 1,
    }


def run_path_leakage_self_test() -> tuple[int, list[str]]:
    result = run_command(
        [sys.executable, str(AREA_ROOT / "check_no_path_leakage.py"), "--self-test"],
        cwd=REPO_ROOT,
        env=base_env(),
    )
    return result.returncode, [] if result.returncode == 0 else [result.summary()]


def run_boundary_equivalence() -> tuple[int, list[str]]:
    try:
        host = host_triple()
        archive_path = archive_for_host(host)
        source_sifr = build_source_sifr()
        with tempfile.TemporaryDirectory(prefix="sifr-boundary-cert.") as temp:
            temp_root = Path(temp)
            install_root = temp_root / "install"
            work_root = temp_root / "outside-repo"
            installed_output = temp_root / "installed-output"
            source_output = temp_root / "source-output"
            extract_archive(archive_path, install_root)
            work_root.mkdir()
            fixture = work_root / BOUNDARY_FIXTURE_PATH.name
            fixture.write_text(BOUNDARY_FIXTURE_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            env = installed_env(temp_root)
            installed_sifr = install_root / "bin" / "sifr"

            for label, compiler, output, extra in (
                ("installed", installed_sifr, installed_output, []),
                ("source-tree", source_sifr, source_output, ["--sysroot", str(REPO_ROOT)]),
            ):
                run_checked(
                    [str(compiler), *extra, "build", str(fixture), "-o", str(output), "--quiet"],
                    cwd=work_root,
                    env=env,
                    label=f"{label} boundary build",
                    timeout=1200,
                )
                binary = output / "sifr_output" / "target" / "release" / "sifr_output"
                result = run_checked(
                    [str(binary)], cwd=work_root, env=env, label=f"{label} boundary run"
                )
                if "stdlib boundary recertification: pass" not in result.stdout:
                    return 1, [f"{label} boundary fixture did not execute successfully"]

            installed_shape = cargo_dependency_shape(
                installed_output / "sifr_output" / "Cargo.toml"
            )
            source_shape = cargo_dependency_shape(source_output / "sifr_output" / "Cargo.toml")
            if installed_shape != source_shape:
                return 1, [
                    "source-tree and installed dependency plans differ: "
                    f"source={source_shape!r} installed={installed_shape!r}"
                ]
            snapshot = json.loads(
                BOUNDARY_DEPENDENCY_SNAPSHOT_PATH.read_text(encoding="utf-8")
            )
            expected_shape = snapshot.get("dependency_shape")
            if source_shape != expected_shape:
                return 1, [
                    "boundary dependency plan differs from the reviewed snapshot: "
                    f"observed={source_shape!r} expected={expected_shape!r}"
                ]
    except CertificationError as error:
        return 1, [str(error)]
    return 0, []


def build_source_sifr() -> Path:
    target = REPO_ROOT / "target" / "sysroot_release" / "source-cargo-target"
    env = base_env()
    env["CARGO_TARGET_DIR"] = str(target)
    run_checked(
        ["cargo", "build", "-p", "sifr"],
        cwd=REPO_ROOT,
        env=env,
        label="build source-tree compiler",
        timeout=900,
    )
    binary = target / "debug" / "sifr"
    if not binary.is_file():
        raise CertificationError(f"source-tree compiler was not produced: {binary}")
    return binary


def cargo_dependency_shape(manifest_path: Path) -> dict[str, Any]:
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    dependencies = manifest.get("dependencies", {})
    shape: dict[str, Any] = {}
    for package, spec in sorted(dependencies.items()):
        if not isinstance(spec, dict):
            shape[package] = spec
            continue
        normalized = {key: value for key, value in spec.items() if key != "path"}
        if "path" in spec:
            normalized["sysroot-crate"] = Path(str(spec["path"])).name
        shape[package] = normalized
    return shape


def run_host_installed_smoke() -> tuple[int, list[str]]:
    try:
        host = host_triple()
        archive_path = archive_for_host(host)

        with tempfile.TemporaryDirectory(prefix="sifr-installed-cert.") as temp:
            temp_root = Path(temp)
            install_root = temp_root / "install"
            work_root = temp_root / "outside-repo"
            build_root = temp_root / "build-output"
            lsp_trace = ACTUAL_ROOT / "lsp-trace.jsonl"
            sysroot_json_path = ACTUAL_ROOT / "installed-sysroot.json"
            emit_path = ACTUAL_ROOT / "installed-smoke-emit.rs"
            doctor_text_path = ACTUAL_ROOT / "installed-doctor-healthy.txt"
            doctor_json_path = ACTUAL_ROOT / "installed-doctor-healthy.json"
            broken_doctor_path = ACTUAL_ROOT / "installed-doctor-broken.txt"
            broken_doctor_json_path = ACTUAL_ROOT / "installed-doctor-broken.json"
            self_version_path = ACTUAL_ROOT / "installed-self-version.json"
            self_update_dry_run_path = ACTUAL_ROOT / "installed-self-update-dry-run.json"
            extract_archive(archive_path, install_root)
            work_root.mkdir()
            build_root.mkdir()

            installed_sifr = install_root / "bin" / "sifr"
            compile_fixture = work_root / "stdlib_compile_release_smoke.sifr"
            compile_fixture.write_text(COMPILE_FIXTURE_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            env = installed_env(temp_root)

            sysroot_output = run_checked(
                [str(installed_sifr), "--print", "sysroot", "--json"],
                cwd=work_root,
                env=env,
                label="installed sysroot json",
            )
            sysroot_json_path.write_text(sysroot_output.stdout, encoding="utf-8")
            sysroot_payload = validate_sysroot_json(sysroot_output.stdout, install_root, host)
            write_install_receipt(install_root, host, sysroot_payload)
            run_self_update_snapshots(
                installed_sifr,
                install_root,
                work_root,
                env,
                self_version_path,
                self_update_dry_run_path,
            )
            run_doctor_snapshots(
                installed_sifr,
                install_root,
                temp_root,
                work_root,
                env,
                doctor_text_path,
                doctor_json_path,
                broken_doctor_path,
                broken_doctor_json_path,
            )

            emit = run_checked(
                [str(installed_sifr), "emit", str(compile_fixture)],
                cwd=work_root,
                env=env,
                label="emit migrated stdlib smoke",
                stdout_path=emit_path,
                echo_output=False,
            )
            if "sifr_stdlib" not in emit.stdout:
                return 1, ["installed migrated stdlib emit output did not reference sifr_stdlib"]
            run_lsp_smoke(installed_sifr, work_root, env, lsp_trace)

            leakage = run_command(
                [
                    sys.executable,
                    str(AREA_ROOT / "check_no_path_leakage.py"),
                    "--forbidden-path",
                    str(REPO_ROOT),
                    str(archive_path),
                    str(sysroot_json_path),
                    str(emit_path),
                    str(lsp_trace),
                    str(doctor_text_path),
                    str(doctor_json_path),
                    str(broken_doctor_path),
                    str(broken_doctor_json_path),
                    str(self_version_path),
                    str(self_update_dry_run_path),
                ],
                cwd=REPO_ROOT,
                env=env,
                timeout=120,
            )
            if leakage.returncode != 0:
                return leakage.returncode, [leakage.summary()]
            home_leakage = run_command(
                [
                    sys.executable,
                    str(AREA_ROOT / "check_no_path_leakage.py"),
                    "--forbidden-path",
                    str(Path.home()),
                    str(archive_path),
                    str(emit_path),
                ],
                cwd=REPO_ROOT,
                env=env,
                timeout=120,
            )
            if home_leakage.returncode != 0:
                return home_leakage.returncode, [home_leakage.summary()]
    except CertificationError as error:
        return 1, [str(error)]
    return 0, []


def run_host_installed_stdlib_heavy() -> tuple[int, list[str]]:
    try:
        host = host_triple()
        archive_path = archive_for_host(host)

        with tempfile.TemporaryDirectory(prefix="sifr-installed-heavy.") as temp:
            temp_root = Path(temp)
            install_root = temp_root / "install"
            work_root = temp_root / "outside-repo"
            build_root = temp_root / "build-output"
            sysroot_json_path = ACTUAL_ROOT / "installed-heavy-sysroot.json"
            emit_path = ACTUAL_ROOT / "installed-heavy-emit.rs"
            tree_path = ACTUAL_ROOT / "installed-heavy-cargo-tree-features.txt"
            extract_archive(archive_path, install_root)
            work_root.mkdir()
            build_root.mkdir()

            installed_sifr = install_root / "bin" / "sifr"
            heavy_fixture = work_root / "stdlib_heavy_release_smoke.sifr"
            heavy_fixture.write_text(HEAVY_FIXTURE_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            compile_fixture = work_root / "stdlib_compile_release_smoke.sifr"
            compile_fixture.write_text(COMPILE_FIXTURE_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            env = installed_env(temp_root)

            sysroot_output = run_checked(
                [str(installed_sifr), "--print", "sysroot", "--json"],
                cwd=work_root,
                env=env,
                label="installed sysroot json",
            )
            sysroot_json_path.write_text(sysroot_output.stdout, encoding="utf-8")
            validate_sysroot_json(sysroot_output.stdout, install_root, host)

            run_checked(
                [str(installed_sifr), "check", str(heavy_fixture)],
                cwd=work_root,
                env=env,
                label="heavy check",
                timeout=1200,
            )
            emit = run_checked(
                [str(installed_sifr), "emit", str(heavy_fixture)],
                cwd=work_root,
                env=env,
                label="heavy emit",
                stdout_path=emit_path,
                echo_output=False,
                timeout=1200,
            )
            if "sifr_stdlib" not in emit.stdout:
                return 1, ["installed heavy emit output did not reference sifr_stdlib"]

            run_checked(
                [str(installed_sifr), "build", str(compile_fixture), "-o", str(build_root), "--quiet"],
                cwd=work_root,
                env=env,
                label="build",
                timeout=1200,
            )
            generated = build_root / "sifr_output"
            binary = generated / "target" / "release" / "sifr_output"
            run_output = run_checked([str(binary)], cwd=work_root, env=env, label="run built binary")
            if "sysroot release compile smoke: pass" not in run_output.stdout:
                return 1, ["built stdlib compile fixture did not execute successfully"]
            run_checked(
                [
                    "cargo",
                    "metadata",
                    "--offline",
                    "--locked",
                    "--manifest-path",
                    str(generated / "Cargo.toml"),
                    "--format-version",
                    "1",
                    "--no-deps",
                ],
                cwd=work_root,
                env=env,
                label="cargo metadata offline",
                echo_output=False,
            )
            run_checked(
                [
                    "cargo",
                    "tree",
                    "-e",
                    "features",
                    "--offline",
                    "--locked",
                    "--manifest-path",
                    str(generated / "Cargo.toml"),
                ],
                cwd=work_root,
                env=env,
                label="cargo tree features",
                stdout_path=tree_path,
                echo_output=False,
            )
            run_checked(
                [
                    "cargo",
                    "build",
                    "--offline",
                    "--frozen",
                    "--manifest-path",
                    str(generated / "Cargo.toml"),
                    "--quiet",
                ],
                cwd=work_root,
                env=env,
                label="cargo build offline frozen",
                timeout=1200,
            )

            leakage = run_command(
                [
                    sys.executable,
                    str(AREA_ROOT / "check_no_path_leakage.py"),
                    "--forbidden-path",
                    str(REPO_ROOT),
                    str(archive_path),
                    str(sysroot_json_path),
                    str(emit_path),
                    str(generated / "Cargo.toml"),
                    str(generated / "Cargo.lock"),
                    str(generated / "src"),
                    str(tree_path),
                ],
                cwd=REPO_ROOT,
                env=env,
                timeout=120,
            )
            if leakage.returncode != 0:
                return leakage.returncode, [leakage.summary()]
            home_leakage = run_command(
                [
                    sys.executable,
                    str(AREA_ROOT / "check_no_path_leakage.py"),
                    "--forbidden-path",
                    str(Path.home()),
                    str(archive_path),
                    str(emit_path),
                ],
                cwd=REPO_ROOT,
                env=env,
                timeout=120,
            )
            if home_leakage.returncode != 0:
                return home_leakage.returncode, [home_leakage.summary()]
    except CertificationError as error:
        return 1, [str(error)]
    return 0, []


def archive_for_host(host: str) -> Path:
    artifact_dir = ACTUAL_ROOT / "artifacts"
    archive_path = artifact_dir / f"sifr-{RELEASE_VERSION}-{host}.tar.gz"
    cached = BUILT_ARCHIVES.get(host)
    if cached == archive_path and archive_path.is_file():
        return archive_path
    if artifact_dir.exists():
        shutil.rmtree(artifact_dir)
    artifact_dir.mkdir(parents=True)
    build_artifact(host, artifact_dir)
    if not archive_path.is_file():
        raise CertificationError(f"expected archive was not produced: {archive_path}")
    BUILT_ARCHIVES[host] = archive_path
    return archive_path


def build_artifact(host: str, artifact_dir: Path) -> None:
    env = base_env()
    env["CARGO_TARGET_DIR"] = str((REPO_ROOT / "target" / "sysroot_release" / "cargo-target").resolve())
    env["SIFR_RELEASE_VERSION"] = RELEASE_VERSION
    run_checked(
        [
            "scripts/distribution/build_preview_artifacts.sh",
            "--version",
            RELEASE_VERSION,
            "--output-dir",
            str(artifact_dir),
            "--target",
            host,
            "--cargo-build",
        ],
        cwd=REPO_ROOT,
        env=env,
        label="build preview artifact",
        timeout=900,
    )


def host_triple() -> str:
    result = run_checked(["rustc", "-vV"], cwd=REPO_ROOT, env=base_env(), label="rustc host")
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    raise CertificationError("rustc -vV did not report host triple")


def extract_archive(archive_path: Path, install_root: Path) -> None:
    install_root.mkdir(parents=True)
    with tarfile.open(archive_path, "r:gz") as archive:
        for member in archive.getmembers():
            pure = PurePosixPath(member.name)
            if pure.is_absolute() or ".." in pure.parts or member.issym() or member.islnk():
                raise CertificationError(f"unsafe archive member: {member.name}")
        try:
            archive.extractall(install_root, filter="data")
        except TypeError:
            archive.extractall(install_root)


def validate_sysroot_json(raw: str, install_root: Path, host: str) -> dict[str, Any]:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"sysroot json did not parse: {error}") from error
    if Path(str(payload.get("root"))).resolve() != install_root.resolve():
        raise CertificationError("installed sifr resolved a sysroot outside the extracted archive")
    if payload.get("sifr_version") != RELEASE_VERSION:
        raise CertificationError("installed sysroot version did not match packaged compiler version")
    if payload.get("target_triple") != host:
        raise CertificationError("installed sysroot target triple did not match host")
    if not isinstance(payload.get("sysroot_content_sha256"), str):
        raise CertificationError("installed sysroot json omitted sysroot_content_sha256")
    return payload


def write_install_receipt(install_root: Path, host: str, sysroot_payload: dict[str, Any]) -> None:
    receipt = {
        "schema_version": 2,
        "name": "sifr",
        "version": RELEASE_VERSION,
        "channel": "beta",
        "target": host,
        "install_dir": str(install_root / "bin"),
        "binary_path": str(install_root / "bin" / "sifr"),
        "sysroot_path": str(install_root),
        "sysroot_schema_version": 1,
        "sysroot_sifr_version": RELEASE_VERSION,
        "sysroot_target_triple": host,
        "sysroot_content_sha256": str(sysroot_payload.get("sysroot_content_sha256")),
        "artifact": f"sifr-{RELEASE_VERSION}-{host}.tar.gz",
        "modify_path": False,
    }
    (install_root / "install.json").write_text(json.dumps(receipt, indent=2, sort_keys=True), encoding="utf-8")


def run_self_update_snapshots(
    installed_sifr: Path,
    install_root: Path,
    work_root: Path,
    env: dict[str, str],
    self_version_path: Path,
    self_update_dry_run_path: Path,
) -> None:
    version = run_checked(
        [str(installed_sifr), "self", "version", "--format", "json"],
        cwd=work_root,
        env=env,
        label="self version json",
        stdout_path=self_version_path,
        echo_output=False,
    )
    validate_self_version_json(version.stdout, install_root)
    dry_run = run_checked(
        [
            str(installed_sifr),
            "self",
            "update",
            "--dry-run",
            "--version",
            "0.1.0-beta.1301",
            "--format",
            "json",
        ],
        cwd=work_root,
        env=env,
        label="self update dry-run json",
        stdout_path=self_update_dry_run_path,
        echo_output=False,
    )
    validate_self_update_dry_run_json(dry_run.stdout, install_root)


def validate_self_version_json(raw: str, install_root: Path) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"self version json did not parse: {error}") from error
    if payload.get("current_version") != RELEASE_VERSION or payload.get("receipt_version") != RELEASE_VERSION:
        raise CertificationError("self version json did not preserve the installed receipt version")
    if Path(str(payload.get("sysroot_path"))).resolve() != install_root.resolve():
        raise CertificationError("self version json did not preserve the installed sysroot path")
    if Path(str(payload.get("binary_path"))).resolve() != (install_root / "bin" / "sifr").resolve():
        raise CertificationError("self version json did not preserve the installed binary path")
    if payload.get("matches_receipt") is not True:
        raise CertificationError("self version json did not report a matching receipt")


def validate_self_update_dry_run_json(raw: str, install_root: Path) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"self update dry-run json did not parse: {error}") from error
    if payload.get("current_version") != RELEASE_VERSION or payload.get("target_version") != "0.1.0-beta.1301":
        raise CertificationError("self update dry-run json did not preserve requested version transition")
    if payload.get("action") != "update" or payload.get("would_run_installer") is not True:
        raise CertificationError("self update dry-run json did not plan an update action")
    if Path(str(payload.get("sysroot_path"))).resolve() != install_root.resolve():
        raise CertificationError("self update dry-run json did not preserve the installed sysroot path")
    if Path(str(payload.get("binary_path"))).resolve() != (install_root / "bin" / "sifr").resolve():
        raise CertificationError("self update dry-run json did not preserve the installed binary path")


def run_doctor_snapshots(
    installed_sifr: Path,
    install_root: Path,
    temp_root: Path,
    work_root: Path,
    env: dict[str, str],
    doctor_text_path: Path,
    doctor_json_path: Path,
    broken_doctor_path: Path,
    broken_doctor_json_path: Path,
) -> None:
    text = run_checked(
        [str(installed_sifr), "doctor"],
        cwd=work_root,
        env=env,
        label="doctor healthy",
        stdout_path=doctor_text_path,
        echo_output=False,
    )
    if "Sifr doctor: ok" not in text.stdout or str(install_root) not in text.stdout:
        raise CertificationError("installed doctor text output did not report the extracted sysroot")
    raw_json = run_checked(
        [str(installed_sifr), "doctor", "--json"],
        cwd=work_root,
        env=env,
        label="doctor healthy json",
        stdout_path=doctor_json_path,
        echo_output=False,
    )
    validate_doctor_json(raw_json.stdout, install_root)

    broken_root = temp_root / "broken-install"
    shutil.copytree(install_root, broken_root)
    broken_runtime_manifest = broken_root / "crates" / "sifr_runtime" / "Cargo.toml"
    broken_runtime_manifest.unlink()
    broken_sifr = broken_root / "bin" / "sifr"
    broken = run_command([str(broken_sifr), "doctor"], cwd=work_root, env=env, timeout=60)
    broken_doctor_path.write_text(broken.stdout + broken.stderr, encoding="utf-8")
    if broken.returncode == 0:
        raise CertificationError("doctor unexpectedly accepted a broken installed sysroot")
    if "missing or invalid asset" not in broken.stderr or "sifr_runtime/Cargo.toml" not in broken.stderr:
        raise CertificationError("broken doctor output did not identify the missing runtime manifest")
    broken_json = run_command([str(broken_sifr), "doctor", "--json"], cwd=work_root, env=env, timeout=60)
    broken_doctor_json_path.write_text(broken_json.stdout + broken_json.stderr, encoding="utf-8")
    if broken_json.returncode == 0:
        raise CertificationError("doctor --json unexpectedly accepted a broken installed sysroot")
    validate_broken_doctor_json(broken_json.stdout)


def validate_doctor_json(raw: str, install_root: Path) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"doctor json did not parse: {error}") from error
    if payload.get("status") != "ok":
        raise CertificationError("doctor json did not report ok status")
    if Path(str(payload.get("root"))).resolve() != install_root.resolve():
        raise CertificationError("doctor json root did not match the extracted archive")
    check_names = {
        str(check.get("name"))
        for check in payload.get("checks", [])
        if isinstance(check, dict) and check.get("status") == "ok"
    }
    required = {"manifest", "runtime_crate", "stdlib_crate", "cargo_lock", "vendor"}
    missing = sorted(required.difference(check_names))
    if missing:
        raise CertificationError(f"doctor json omitted required ok check(s): {', '.join(missing)}")


def validate_broken_doctor_json(raw: str) -> None:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise CertificationError(f"broken doctor json did not parse: {error}") from error
    if payload.get("status") != "error":
        raise CertificationError("broken doctor json did not report error status")
    asset_path = str(payload.get("asset_path"))
    if "sifr_runtime/Cargo.toml" not in asset_path:
        raise CertificationError("broken doctor json did not identify the missing runtime manifest")


def run_lsp_smoke(installed_sifr: Path, work_root: Path, env: dict[str, str], trace_path: Path) -> None:
    messages = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"processId": None, "rootUri": work_root.as_uri(), "capabilities": {}},
        },
        {"jsonrpc": "2.0", "method": "initialized", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": None},
        {"jsonrpc": "2.0", "method": "exit", "params": None},
    ]
    stdin = "".join(encode_lsp_message(message) for message in messages)
    result = run_command(
        [str(installed_sifr), "lsp", "--stdio"],
        cwd=work_root,
        env=env,
        input_text=stdin,
        timeout=60,
    )
    trace_path.write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode != 0:
        raise CertificationError(f"installed LSP exited with {result.returncode}: {result.summary()}")
    if '"id":1' not in result.stdout.replace(" ", ""):
        raise CertificationError("installed LSP did not answer initialize request")


def encode_lsp_message(message: dict[str, Any]) -> str:
    body = json.dumps(message, separators=(",", ":"))
    return f"Content-Length: {len(body.encode('utf-8'))}\r\n\r\n{body}"


def installed_env(temp_root: Path) -> dict[str, str]:
    env = base_env()
    env.pop("SIFR_SYSROOT", None)
    env.pop("SIFR_RUNTIME_PATH", None)
    env.pop("SIFR_INSTALL_MANIFEST_DIR", None)
    env["CARGO_NET_OFFLINE"] = "true"
    probe_cache_root = ACTUAL_ROOT / "probe-cache"
    probe_cache_root.mkdir(parents=True, exist_ok=True)
    env["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)
    return env


def base_env() -> dict[str, str]:
    env = os.environ.copy()
    env.setdefault("CARGO_NET_OFFLINE", "true")
    return env


def run_checked(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    label: str,
    timeout: int = 120,
    stdout_path: Path | None = None,
    echo_output: bool = True,
) -> CommandResult:
    result = run_command(command, cwd=cwd, env=env, timeout=timeout, echo_output=echo_output)
    if stdout_path is not None:
        stdout_path.write_text(result.stdout, encoding="utf-8")
    if result.returncode != 0:
        raise CertificationError(f"{label} failed with exit={result.returncode}: {result.summary()}")
    return result


def run_command(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: int = 120,
    input_text: str | None = None,
    echo_output: bool = True,
) -> CommandResult:
    print(f"  $ {' '.join(command)}", flush=True)
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            input=input_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        return CommandResult(command, 124, error.stdout or "", error.stderr or f"timeout after {timeout}s")
    if echo_output and completed.stdout:
        sys.stdout.write(completed.stdout)
    if echo_output and completed.stderr:
        sys.stderr.write(completed.stderr)
    return CommandResult(command, completed.returncode, completed.stdout, completed.stderr)


class CertificationError(Exception):
    """A sysroot release certification assertion failed."""


class CommandResult:
    def __init__(self, command: list[str], returncode: int, stdout: str, stderr: str) -> None:
        self.command = command
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr

    def summary(self) -> str:
        combined = "\n".join(part for part in (self.stdout, self.stderr) if part)
        if len(combined) > 2000:
            combined = combined[-2000:]
        return combined.strip() or "no output"


if __name__ == "__main__":
    raise SystemExit(main())

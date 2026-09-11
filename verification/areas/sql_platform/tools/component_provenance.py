"""Source and producer identities shared by all owned compiler components."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import tomllib
from pathlib import Path
from typing import Any

from wasi_sdk_inputs import SDK_ASSETS, SDK_VERSION, content_sha256, sha256
from wasi_virt_inputs import WASI_VIRT_COMMIT, WASI_VIRT_SOURCE_SHA256, WASI_VIRT_VERSION

ROOT = Path(__file__).resolve().parents[4]
WORDS = "crates/sifr_compiler_component/fixtures/words_component"
FAMILIES = {
    "postgresql": "crates/sifr_sql_postgresql",
    "mysql": "crates/sifr_sql_mysql",
    "sqlite": "crates/sifr_sql_sqlite",
    "words": WORDS,
}
SERIES = {"postgresql": [13, 14, 15, 16, 17, 18], "mysql": ["8.4", "9.7", "26.7"], "sqlite": ["3.53.2"]}
TOOLS = Path("verification/areas/sql_platform/tools")


def toml(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def source_inputs(family: str, root: Path = ROOT) -> dict[str, str]:
    """Follow actual local dependencies, including shared and Ruff sources."""
    workspace = toml(root / "Cargo.toml")["workspace"]["dependencies"]
    pending = [root / FAMILIES[family]]
    visited: set[Path] = set()
    paths = {
        root / "rust-toolchain.toml",
        root / "crates/sifr_compiler_component/wit/compiler-component.wit",
        root / TOOLS / "component_provenance.py",
        root / TOOLS / "wasi_sdk_inputs.py",
    }
    if family == "words":
        paths.update((root / WORDS / name for name in ("Cargo.lock", "Cargo.toml")))
        paths.add(root / TOOLS / "build_words_component.py")
    else:
        paths.update(root / name for name in ("Cargo.toml", "Cargo.lock", ".cargo/config.toml", "third_party/ruff/Cargo.toml"))
        paths.update(root / TOOLS / name for name in ("build_sql_components.py", "wasi_sdk_inputs.py", "wasi_virt_inputs.py"))
        builder = f"build_{family}_component{'s' if family != 'sqlite' else ''}.py"
        paths.add(root / TOOLS / builder)
    while pending:
        directory = pending.pop().resolve()
        if directory in visited:
            continue
        visited.add(directory)
        manifest = directory / "Cargo.toml"
        data = toml(manifest)
        paths.add(manifest)
        for name in ("build.rs", "component-sources.json"):
            if (directory / name).is_file():
                paths.add(directory / name)
        for name in ("src", "wit", "wasi_compat"):
            paths.update(path for path in (directory / name).rglob("*") if path.is_file())
        tables = [data, *data.get("target", {}).values()]
        for table in tables:
            for section in ("dependencies", "build-dependencies"):
                for name, spec in table.get(section, {}).items():
                    if not isinstance(spec, dict):
                        continue
                    if spec.get("workspace"):
                        selected = workspace.get(name, {})
                        base = root
                    else:
                        selected, base = spec, directory
                    if isinstance(selected, dict) and "path" in selected:
                        pending.append(base / selected["path"])
    return {path.relative_to(root).as_posix(): sha256(path) for path in sorted(paths)}


def inputs_digest(inputs: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(inputs, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def rust_identity(target: str) -> dict[str, str]:
    expected = toml(ROOT / "rust-toolchain.toml")["toolchain"]["channel"]
    result = {}
    for tool in ("rustc", "cargo"):
        executable = shutil.which(tool)
        if executable is None:
            raise ValueError(f"missing {tool}")
        version = subprocess.check_output([executable, "--version"], text=True).strip()
        if version.split()[1] != expected:
            raise ValueError(f"component builds require {tool} {expected}: {version}")
        result[f"{tool}_version"] = version
        result[f"{tool}_sha256"] = sha256(Path(executable).resolve())
    result["rustc_verbose"] = subprocess.check_output(["rustc", "-vV"], text=True).strip()
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
    libraries = sysroot / "lib/rustlib" / target / "lib"
    if not libraries.is_dir():
        raise ValueError(f"the pinned Rust toolchain has no {target} standard library")
    result["target"] = target
    result["target_std_sha256"] = content_sha256(libraries)
    return result


def target_directory(manifest: Path) -> Path:
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--offline", "--no-deps",
        "--format-version", "1", "--manifest-path", str(manifest),
    ], cwd=ROOT, text=True))
    return Path(metadata["target_directory"])


def guest_environment(environment: dict[str, str]) -> dict[str, str]:
    result = environment.copy()
    for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC", "CARGO_BUILD_TARGET"):
        if result.get(key):
            raise ValueError(f"ambient component compiler override is unsupported: {key}")
    if any(key.startswith("CARGO_TARGET_") and key != "CARGO_TARGET_DIR" for key in result):
        raise ValueError("ambient target linker or compiler flags are unsupported")
    result["RUSTFLAGS"] = f"--remap-path-prefix={ROOT}=/sifr"
    return result


def provenance(family: str, rust: dict[str, str], **tools: object) -> dict[str, object]:
    inputs = source_inputs(family)
    return {"schema_version": 1, "family": family, "source_inputs": inputs,
            "source_sha256": inputs_digest(inputs), "rust": rust, "tools": tools,
            "rustflags": "--remap-path-prefix=<repository>=/sifr",
            "build_environment": {key: value for key, value in sorted(os.environ.items())
                                  if key.startswith("CARGO_PROFILE_") or key in {"CARGO_INCREMENTAL", "CARGO_BUILD_JOBS"}}}


def validate_provenance(value: object, family: str, root: Path = ROOT) -> None:
    if not isinstance(value, dict) or value.get("schema_version") != 1 or value.get("family") != family:
        raise ValueError("component producer identity is missing or invalid")
    inputs = source_inputs(family, root)
    if value.get("source_inputs") != inputs or value.get("source_sha256") != inputs_digest(inputs):
        raise ValueError(f"{family} component source provenance has drifted; rebuild its artifacts")
    rust = value.get("rust", {})
    if not isinstance(rust, dict):
        raise ValueError("component Rust producer record is invalid")
    target = "wasm32-unknown-unknown" if family == "words" else "wasm32-wasip2"
    if rust.get("target") != target:
        raise ValueError("component Rust target differs")
    require_digest(rust.get("target_std_sha256"))
    expected = toml(root / "rust-toolchain.toml")["toolchain"]["channel"]
    for tool in ("cargo", "rustc"):
        if not str(rust.get(f"{tool}_version", "")).startswith(f"{tool} {expected} "):
            raise ValueError("component Rust producer version is not exact")
        require_digest(rust.get(f"{tool}_sha256"))
    tools = value.get("tools", {})
    if not isinstance(rust, dict) or not isinstance(tools, dict):
        raise ValueError("component producer tool records are invalid")
    if family == "words":
        if tools.get("wit_bindgen") != "0.61.1" or tools.get("wit_component") != "0.258.0":
            raise ValueError("words WIT producer versions differ")
        require_digest(tools.get("componentizer_sha256"))
    else:
        if tools.get("wit_bindgen") != "0.61.1":
            raise ValueError("SQL WIT producer version differs")
        virt = tools.get("virtualizer", {})
        if (virt.get("version"), virt.get("commit"), virt.get("source_sha256")) != (
            WASI_VIRT_VERSION, WASI_VIRT_COMMIT, WASI_VIRT_SOURCE_SHA256
        ):
            raise ValueError("component virtualizer producer differs")
        require_digest(virt.get("binary_sha256"))
        require_digest(virt.get("lock_sha256"))
        # MySQL is pure Rust and does not claim SDK use.
        if family != "mysql":
            sdk = tools.get("sdk", {})
            key = sdk.get("platform")
            if key not in SDK_ASSETS or sdk.get("version") != SDK_VERSION or sdk.get("archive_sha256") != SDK_ASSETS[key][1]:
                raise ValueError("component SDK archive identity differs")
            for field in ("compiler_sha256", "archiver_sha256", "sysroot_sha256"):
                require_digest(sdk.get(field))


def require_digest(value: object) -> None:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("component producer digest is invalid")

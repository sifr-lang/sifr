"""Installed/source attached-API execution and cache certification."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any, Callable


RUNTIME_DEPENDENCY = re.compile(
    r'^sifr_runtime = \{ path = "[^"]+", features = \["structural"\] \}$',
    re.MULTILINE,
)


def bind_runtime_dependency(*, fixture: Path, runtime_crate: Path) -> str | None:
    manifest_path = fixture / "Cargo.toml"
    try:
        manifest = manifest_path.read_text(encoding="utf-8")
    except OSError as error:
        return f"failed to read attached API Cargo manifest: {error}"
    replacement = (
        "sifr_runtime = { path = "
        f"{json.dumps(str(runtime_crate))}, features = [\"structural\"] }}"
    )
    updated, replacements = RUNTIME_DEPENDENCY.subn(replacement, manifest)
    if replacements != 1:
        return (
            "attached API Cargo manifest must contain exactly one structural "
            "sifr_runtime path dependency"
        )
    try:
        manifest_path.write_text(updated, encoding="utf-8")
    except OSError as error:
        return f"failed to bind attached API runtime dependency: {error}"
    return None


def run_attached_api_certification(
    *,
    compiler: Path,
    extra: list[str],
    fixture: Path,
    output: Path,
    env: dict[str, str],
    label: str,
    runtime_crate: Path,
    run_checked: Callable[..., Any],
) -> str | None:
    dependency_error = bind_runtime_dependency(
        fixture=fixture, runtime_crate=runtime_crate
    )
    if dependency_error is not None:
        return dependency_error
    build_command = [
        str(compiler),
        *extra,
        "build",
        str(fixture / "src" / "app.sifr"),
        "-o",
        str(output),
        "--quiet",
    ]
    run_checked(
        build_command,
        cwd=fixture,
        env=env,
        label=f"{label} attached API build",
        timeout=1200,
    )
    binary = output / "sifr_output" / "target" / "release" / "sifr_output"
    result = run_checked(
        [str(binary)], cwd=fixture, env=env, label=f"{label} attached API run"
    )
    if result.stdout.strip() != "attached-contract":
        return f"{label} attached API fixture returned an unexpected value"

    api_path = fixture / "src" / "api.sifr"
    original_api = api_path.read_text(encoding="utf-8")
    edited_api = original_api.replace(
        'return "attached-contract"', 'return "attached-contract-edited"'
    )
    if edited_api == original_api:
        return "attached API cache probe could not edit its package function"
    api_path.write_text(edited_api, encoding="utf-8")
    run_checked(
        build_command,
        cwd=fixture,
        env=env,
        label=f"{label} attached API cache rebuild",
        timeout=1200,
    )
    edited_result = run_checked(
        [str(binary)], cwd=fixture, env=env, label=f"{label} attached API cache run"
    )
    api_path.write_text(original_api, encoding="utf-8")
    if edited_result.stdout.strip() != "attached-contract-edited":
        return f"{label} attached API edit did not invalidate code generation"
    return None

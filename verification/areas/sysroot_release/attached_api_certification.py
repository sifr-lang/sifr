"""Installed/source attached-API execution and cache certification."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


def run_attached_api_certification(
    *,
    compiler: Path,
    extra: list[str],
    fixture: Path,
    output: Path,
    env: dict[str, str],
    label: str,
    run_checked: Callable[..., Any],
) -> str | None:
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

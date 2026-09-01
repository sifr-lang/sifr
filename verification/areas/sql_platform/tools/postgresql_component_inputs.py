"""Compute the deterministic PostgreSQL component guest-source identity."""

from __future__ import annotations

import hashlib
import json
import tomllib
from collections import deque
from pathlib import Path
from typing import Any


COMPONENT_PACKAGES = {
    "sifr_compiler_component",
    "sifr_sql_contract",
    "sifr_sql_postgresql",
}


def guest_source_sha256(repo_root: Path) -> str:
    crate = repo_root / "crates/sifr_sql_postgresql"
    roots = (
        repo_root / "crates/sifr_compiler_component",
        repo_root / "crates/sifr_sql_contract",
        crate,
    )
    paths: list[Path] = []
    for root in roots:
        paths.extend(root.glob("Cargo.toml"))
        paths.extend(root.glob("build.rs"))
        paths.extend(root.glob("component-sources.json"))
        paths.extend(root.glob("src/**/*.rs"))
        paths.extend(path for path in root.glob("wit/**/*") if path.is_file())
        paths.extend(path for path in root.glob("wasi_compat/**/*") if path.is_file())
    digest = hashlib.sha256()
    for path in sorted(set(paths)):
        relative = path.relative_to(repo_root).as_posix().encode()
        digest.update(relative)
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    update_virtual_input(digest, "Cargo.toml#component-workspace", workspace_manifest_input(repo_root))
    update_virtual_input(digest, "Cargo.lock#component-closure", lockfile_component_closure(repo_root))
    return digest.hexdigest()


def workspace_manifest_input(repo_root: Path) -> dict[str, Any]:
    root = load_toml(repo_root / "Cargo.toml")
    workspace = root.get("workspace", {})
    dependencies = workspace.get("dependencies", {})
    selected = component_workspace_dependencies(repo_root)
    return {
        "workspace_package": workspace.get("package", {}),
        "workspace_dependencies": {
            name: dependencies[name]
            for name in sorted(selected)
            if name in dependencies
        },
        "profile": root.get("profile", {}),
    }


def component_workspace_dependencies(repo_root: Path) -> set[str]:
    selected: set[str] = set()
    for package in COMPONENT_PACKAGES:
        manifest = load_toml(repo_root / "crates" / package / "Cargo.toml")
        for section in ("dependencies", "build-dependencies", "dev-dependencies"):
            for name, specification in manifest.get(section, {}).items():
                if isinstance(specification, dict) and specification.get("workspace") is True:
                    selected.add(str(name))
    return selected


def lockfile_component_closure(repo_root: Path) -> list[dict[str, Any]]:
    packages = load_toml(repo_root / "Cargo.lock").get("package", [])
    by_name: dict[str, list[dict[str, Any]]] = {}
    for package in packages:
        if isinstance(package, dict) and isinstance(package.get("name"), str):
            by_name.setdefault(str(package["name"]), []).append(package)
    pending = deque(sorted(COMPONENT_PACKAGES))
    selected: set[tuple[str, str, str]] = set()
    rows: list[dict[str, Any]] = []
    while pending:
        name = pending.popleft()
        for package in by_name.get(name, []):
            identity = (
                str(package.get("name", "")),
                str(package.get("version", "")),
                str(package.get("source", "")),
            )
            if identity in selected:
                continue
            selected.add(identity)
            rows.append(package)
            for dependency in package.get("dependencies", []):
                if isinstance(dependency, str) and dependency:
                    pending.append(dependency.split(" ", maxsplit=1)[0])
    return sorted(
        rows,
        key=lambda row: (
            str(row.get("name", "")),
            str(row.get("version", "")),
            str(row.get("source", "")),
        ),
    )


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        payload = tomllib.load(handle)
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must contain a TOML table")
    return payload


def update_virtual_input(digest: Any, label: str, payload: object) -> None:
    rendered = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    digest.update(label.encode())
    digest.update(b"\0")
    digest.update(hashlib.sha256(rendered).digest())

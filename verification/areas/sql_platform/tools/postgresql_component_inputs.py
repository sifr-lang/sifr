"""Compute the deterministic PostgreSQL component guest-source identity."""

from __future__ import annotations

import hashlib
from pathlib import Path


def guest_source_sha256(repo_root: Path) -> str:
    crate = repo_root / "crates/sifr_sql_postgresql"
    roots = (
        repo_root / "crates/sifr_compiler_component",
        repo_root / "crates/sifr_sql_contract",
        crate,
    )
    paths = [repo_root / "Cargo.lock", repo_root / "Cargo.toml"]
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
    return digest.hexdigest()

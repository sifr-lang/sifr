"""Version-matched producer inputs shared by package and source-reference tests."""
from __future__ import annotations
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts/distribution"))
from metadata_artifact import stage_source_manifest  # noqa: E402


def prepare_source_snapshot(root: Path, destination: Path, version: str) -> Path:
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    shutil.copytree(root / "stdlib", destination / "stdlib")
    for relative in ("Cargo.toml", "Cargo.lock", ".cargo/config.toml",
                     "crates/sifr_runtime/Cargo.toml", "crates/sifr_stdlib/Cargo.toml",
                     "crates/sifr_structural_identity/Cargo.toml"):
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(root / relative, target)
    (destination / "vendor").mkdir()
    stage_source_manifest(root / "sysroot.toml", destination / "sysroot.toml", version)
    return destination

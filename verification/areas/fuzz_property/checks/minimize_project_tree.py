"""Copy a failing project tree into a deterministic minimization workspace."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
OUTPUT_ROOT = REPO_ROOT / "target" / "verification" / "actual" / "fuzz_property" / "minimized"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, help="Hardening target id from fuzz_smoke_manifest.json.")
    parser.add_argument("failing_project_dir", help="Path to the failing project directory.")
    parser.add_argument("--output", help="Optional output directory for the minimized candidate.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    project_dir = Path(args.failing_project_dir)
    if not project_dir.is_dir():
        raise SystemExit(f"project directory not found: {project_dir}")
    digest = project_digest(project_dir)
    output = Path(args.output) if args.output else OUTPUT_ROOT / args.target / digest
    if output.exists():
        output_resolved = output.resolve()
        output_root_resolved = OUTPUT_ROOT.resolve()
        if not output_resolved.is_relative_to(output_root_resolved):
            raise SystemExit(f"refusing to replace output outside {OUTPUT_ROOT}: {output}")
        shutil.rmtree(output)
    ignore = shutil.ignore_patterns("target", ".git", "__pycache__")
    shutil.copytree(project_dir, output, ignore=ignore)
    print(output)
    return 0


def project_digest(project_dir: Path) -> str:
    hasher = hashlib.sha256()
    for path in sorted(item for item in project_dir.rglob("*") if item.is_file()):
        relative = path.relative_to(project_dir).as_posix()
        if relative.startswith(("target/", ".git/")) or "__pycache__" in path.parts:
            continue
        hasher.update(relative.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(path.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()[:16]


if __name__ == "__main__":
    raise SystemExit(main())

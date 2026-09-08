#!/usr/bin/env python3
"""Offline archive planning/readback only; no upload, provider, or qualification."""

from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.archive_manifest import (  # noqa: E402
    plan, read_local, verify_local,
)
from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError, canonical_json_bytes, load_json_bytes_strict, sha256_bytes,
)


def create_output(path: Path, raw: bytes) -> None:
    """Install complete bytes atomically without clobbering an existing output."""
    descriptor, temporary = tempfile.mkstemp(prefix=".archive-plan-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, path)
    finally:
        os.unlink(temporary)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    planning = commands.add_parser("plan")
    planning.add_argument("--inventory", type=Path, required=True)
    planning.add_argument("--payload-root", type=Path, required=True)
    planning.add_argument("--out", type=Path, required=True)
    verifying = commands.add_parser("verify-local")
    verifying.add_argument("--manifest", type=Path, required=True)
    verifying.add_argument("--manifest-sha256", required=True)
    verifying.add_argument("--expected-source", required=True)
    verifying.add_argument("--expected-run", type=int, required=True)
    verifying.add_argument("--expected-attempt", type=int, required=True)
    verifying.add_argument("--payload-root", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "plan":
            inventory = load_json_bytes_strict(read_local(args.inventory.parent, args.inventory.name), source="inventory")
            manifest = plan(inventory, args.payload_root)
            raw = canonical_json_bytes(manifest)
            create_output(args.out, raw)
        else:
            raw = read_local(args.manifest.parent, args.manifest.name)
            manifest = verify_local(
                raw, manifest_sha256=args.manifest_sha256, expected_source=args.expected_source,
                expected_run=args.expected_run, expected_attempt=args.expected_attempt,
                payload_root=args.payload_root,
            )
    except (GovernanceError, OSError) as exc:
        print(f"archive-evidence: {exc}", file=sys.stderr)
        return 2
    print(f"archive {args.command} ok: sha256={sha256_bytes(raw)} objects={len(manifest['artifacts'])}; custody only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Generate the canonical facts binding one release-index site deployment."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(
    0,
    str(REPO_ROOT / "verification" / "areas" / "distribution_release"),
)

from governance import GovernanceError, validate_site_publication_facts  # noqa: E402

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
DISPATCHERS = ("index", "stable", "alpha", "beta")


def fail(message: str) -> None:
    raise SystemExit(f"site-publication-facts: {message}")


def require_sha256(value: str, field: str) -> str:
    if not SHA256_RE.fullmatch(value):
        fail(f"{field} must be a lowercase SHA-256 digest")
    if set(value) == {"0"}:
        fail(f"{field} must not be the zero SHA-256 digest")
    return value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--site-base-commit", required=True)
    parser.add_argument("--release-plan-sha256", required=True)
    parser.add_argument("--publication-attempt", required=True)
    parser.add_argument("--release-index-generation", type=int, required=True)
    parser.add_argument("--release-index-sha256", required=True)
    parser.add_argument(
        "--dispatcher-default-channel",
        required=True,
        choices=("beta", "stable"),
    )
    for dispatcher in DISPATCHERS:
        parser.add_argument(f"--dispatcher-{dispatcher}-sha256", required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.out.exists():
        fail(f"refusing to overwrite {args.out}")
    if not COMMIT_RE.fullmatch(args.source_commit):
        fail("source_commit must be an exact lowercase 40-character commit")
    if not COMMIT_RE.fullmatch(args.site_base_commit):
        fail("site_base_commit must be an exact lowercase 40-character commit")
    if not args.publication_attempt or any(
        char not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-"
        for char in args.publication_attempt
    ):
        fail("publication_attempt must use only letters, digits, dot, underscore, or dash")
    if args.release_index_generation < 1:
        fail("release_index_generation must be positive")

    dispatchers = {
        name: require_sha256(
            getattr(args, f"dispatcher_{name}_sha256"),
            f"dispatchers.{name}",
        )
        for name in DISPATCHERS
    }
    payload = {
        "schema_version": 2,
        "contract": "sifr-site-publication-binding-v2",
        "publication_attempt": args.publication_attempt,
        "source_commit": args.source_commit,
        "site_base_commit": args.site_base_commit,
        "release_plan_sha256": require_sha256(
            args.release_plan_sha256,
            "release_plan_sha256",
        ),
        "release_index": {
            "generation": args.release_index_generation,
            "sha256": require_sha256(
                args.release_index_sha256,
                "release_index.sha256",
            ),
        },
        "dispatcher_default_channel": args.dispatcher_default_channel,
        "dispatchers": dispatchers,
    }
    try:
        validate_site_publication_facts(payload)
    except GovernanceError as exc:
        fail(str(exc))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(
        json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()

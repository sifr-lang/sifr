#!/usr/bin/env python3
"""Bootstrap self-update channel metadata from public GitHub prereleases."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

PREVIEW_RE = re.compile(r"(\d+)\.(\d+)\.(\d+)-(alpha|beta)\.(\d+)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--releases-json", required=True, help="gh release list JSON output")
    parser.add_argument("--channel", required=True, choices=("alpha", "beta"))
    parser.add_argument("--version", required=True, help="Current release version")
    parser.add_argument("--out", required=True, help="Output channels.json path")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    version_channel = preview_channel(args.version)
    if version_channel != args.channel:
        raise SystemExit(f"version {args.version} belongs to {version_channel}, not {args.channel}")

    releases = json.loads(Path(args.releases_json).read_text(encoding="utf-8"))
    if not isinstance(releases, list):
        raise SystemExit("release list JSON must be an array")

    latest = latest_public_prereleases(releases)
    existing = latest.get(args.channel)
    if existing is not None and version_key(args.version) < version_key(existing):
        raise SystemExit(
            f"refusing to downgrade {args.channel} channel from {existing} to {args.version}"
        )
    latest[args.channel] = args.version

    other_channel = "beta" if args.channel == "alpha" else "alpha"
    if other_channel not in latest:
        raise SystemExit(
            f"could not bootstrap channels metadata: missing public {other_channel} prerelease"
        )

    output = {
        "schema_version": 1,
        "channels": {
            "alpha": latest["alpha"],
            "beta": latest["beta"],
        },
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    return 0


def latest_public_prereleases(releases: list[Any]) -> dict[str, str]:
    latest: dict[str, str] = {}
    for release in releases:
        if not isinstance(release, dict):
            continue
        if release.get("isDraft") or not release.get("isPrerelease"):
            continue
        tag = release.get("tagName")
        if not isinstance(tag, str):
            continue
        match = PREVIEW_RE.fullmatch(tag)
        if match is None:
            continue
        channel = match.group(4)
        current = latest.get(channel)
        if current is None or version_key(tag) > version_key(current):
            latest[channel] = tag
    return latest


def preview_channel(version: str) -> str:
    match = PREVIEW_RE.fullmatch(version)
    if match is None:
        raise SystemExit(f"version must be an alpha or beta semver prerelease: {version}")
    return match.group(4)


def version_key(version: str) -> tuple[int, int, int, int, int]:
    match = PREVIEW_RE.fullmatch(version)
    if match is None:
        raise ValueError(version)
    major, minor, patch, channel, prerelease = match.groups()
    channel_rank = {"alpha": 1, "beta": 2}[channel]
    return (int(major), int(minor), int(patch), channel_rank, int(prerelease))


if __name__ == "__main__":
    raise SystemExit(main())

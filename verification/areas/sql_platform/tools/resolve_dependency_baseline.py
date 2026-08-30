#!/usr/bin/env python3
"""Resolve and validate the schema-first SQL dependency baseline."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
import tomllib
import urllib.parse
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
BASELINE_PATH = REPO_ROOT / "verification/areas/sql_platform/dependency_baseline.toml"
USER_AGENT = "sifr-sql-baseline-resolver/1.0"
HEX64 = re.compile(r"[0-9a-f]{64}")
HEX40 = re.compile(r"[0-9a-f]{40}")
NAME = re.compile(r"[a-z][a-z0-9_-]*")
VERSION = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?")
CRATE_FIELDS = (
    "name",
    "display_name",
    "version",
    "policy",
    "release_group",
    "compatible_min",
    "compatible_max",
    "compatibility_owner",
    "release_authority",
    "api_authority",
    "source_authority",
    "checksum",
    "license",
    "published_at",
    "yanked",
)
SOURCE_FIELDS = (
    "name",
    "provider",
    "server_major",
    "tag",
    "commit",
    "policy",
    "release_authority",
    "api_authority",
    "license",
)


class BaselineError(ValueError):
    """The dependency baseline is invalid."""


@dataclass(frozen=True, order=True)
class SemVer:
    major: int
    minor: int
    patch: int

    @classmethod
    def parse_stable(cls, value: str) -> SemVer:
        match = VERSION.fullmatch(value)
        if match is None or match.group(4) is not None:
            raise BaselineError(f"version is not stable semantic versioning: {value}")
        return cls(int(match.group(1)), int(match.group(2)), int(match.group(3)))


def load_baseline(path: Path = BASELINE_PATH) -> dict[str, Any]:
    with path.open("rb") as handle:
        payload = tomllib.load(handle)
    if not isinstance(payload, dict):
        raise BaselineError("dependency baseline must be a TOML table")
    return payload


def validate_baseline(payload: dict[str, Any]) -> None:
    if payload.get("schema_version") != 1:
        raise BaselineError("dependency baseline schema_version must be 1")
    for field in ("verified_at", "generated_by", "root_lock_package", "rust_version", "sqlite_amalgamation"):
        if not isinstance(payload.get(field), str) or not payload[field]:
            raise BaselineError(f"dependency baseline has invalid {field}")

    crates = payload.get("crate")
    if not isinstance(crates, list) or not crates:
        raise BaselineError("dependency baseline has no crate rows")
    seen_crates: set[str] = set()
    release_groups: dict[str, str] = {}
    for row in crates:
        validate_crate(row, seen_crates, release_groups)

    sources = payload.get("source")
    if not isinstance(sources, list) or not sources:
        raise BaselineError("dependency baseline has no source rows")
    seen_sources: set[tuple[str, int]] = set()
    majors: set[int] = set()
    for row in sources:
        validate_source(row, seen_sources, majors)
    if majors != set(range(13, 19)):
        raise BaselineError(f"libpg_query majors must be 13 through 18, found {sorted(majors)}")


def validate_crate(row: object, seen: set[str], groups: dict[str, str]) -> None:
    if not isinstance(row, dict):
        raise BaselineError("crate baseline row must be a table")
    name = row.get("name")
    if not isinstance(name, str) or NAME.fullmatch(name) is None:
        raise BaselineError(f"invalid crate identity: {name}")
    if name in seen:
        raise BaselineError(f"duplicate crate identity: {name}")
    seen.add(name)

    version = row.get("version")
    if not isinstance(version, str):
        raise BaselineError(f"crate {name} has no exact version")
    selected = SemVer.parse_stable(version)
    if row.get("yanked") is not False:
        raise BaselineError(f"crate {name} is yanked or has no yanked evidence")
    if not isinstance(row.get("checksum"), str) or HEX64.fullmatch(row["checksum"]) is None:
        raise BaselineError(f"crate {name} has no crates.io checksum")
    for field in ("display_name", "license", "published_at"):
        if not isinstance(row.get(field), str) or not row[field]:
            raise BaselineError(f"crate {name} has invalid {field}")
    release = row.get("release_authority")
    api = row.get("api_authority")
    if release != f"https://crates.io/crates/{name}/{version}":
        raise BaselineError(f"crate {name} has an invalid release authority")
    if api != f"https://crates.io/api/v1/crates/{name}":
        raise BaselineError(f"crate {name} has an invalid API authority")
    source_authority = row.get("source_authority")
    if name == "syntaqlite":
        expected_source = "https://github.com/LalitMaganti/syntaqlite"
        if source_authority != expected_source:
            raise BaselineError("syntaqlite has no canonical source authority")
    elif source_authority is not None:
        raise BaselineError(f"crate {name} has an unexpected source authority override")

    policy = row.get("policy")
    if policy == "latest-compatible":
        minimum = row.get("compatible_min")
        maximum = row.get("compatible_max")
        owner = row.get("compatibility_owner")
        if not all(isinstance(item, str) and item for item in (minimum, maximum, owner)):
            raise BaselineError(f"crate {name} has an incomplete compatibility constraint")
        if not (SemVer.parse_stable(minimum) <= selected < SemVer.parse_stable(maximum)):
            raise BaselineError(f"crate {name} is outside its compatible release family")
    elif policy != "latest-stable":
        raise BaselineError(f"crate {name} has invalid selection policy: {policy}")

    group = row.get("release_group")
    if group is not None:
        if not isinstance(group, str) or not group:
            raise BaselineError(f"crate {name} has invalid release group")
        previous = groups.setdefault(group, version)
        if previous != version:
            raise BaselineError(f"release group {group} has version drift")


def validate_source(
    row: object,
    seen: set[tuple[str, int]],
    majors: set[int],
) -> None:
    if not isinstance(row, dict):
        raise BaselineError("source baseline row must be a table")
    name = row.get("name")
    major = row.get("server_major")
    identity = (str(name), int(major) if isinstance(major, int) else -1)
    if identity in seen:
        raise BaselineError(f"duplicate source identity: {identity}")
    seen.add(identity)
    if name != "libpg_query" or row.get("provider") != "postgresql":
        raise BaselineError(f"unsupported source baseline: {identity}")
    if not isinstance(major, int):
        raise BaselineError("libpg_query source has no server major")
    majors.add(major)
    tag = row.get("tag")
    commit = row.get("commit")
    if not isinstance(tag, str) or not tag or tag in {"main", "master", "HEAD"}:
        raise BaselineError(f"libpg_query {major} has an unlocked tag")
    if not isinstance(commit, str) or HEX40.fullmatch(commit) is None:
        raise BaselineError(f"libpg_query {major} has no exact source commit")
    if row.get("policy") != "latest-stable-for-major":
        raise BaselineError(f"libpg_query {major} has invalid policy")
    expected_release = f"https://github.com/pganalyze/libpg_query/releases/tag/{tag}"
    if row.get("release_authority") != expected_release:
        raise BaselineError(f"libpg_query {major} has invalid release authority")
    expected_api = "https://api.github.com/repos/pganalyze/libpg_query/releases"
    if row.get("api_authority") != expected_api:
        raise BaselineError(f"libpg_query {major} has invalid API authority")
    if row.get("license") != "BSD-3-Clause":
        raise BaselineError(f"libpg_query {major} has invalid license record")


def fetch_json(url: str) -> Any:
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/vnd.github+json", "User-Agent": USER_AGENT},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def refresh(payload: dict[str, Any]) -> dict[str, Any]:
    refreshed = copy.deepcopy(payload)
    for row in refreshed["crate"]:
        metadata = fetch_json(row["api_authority"])
        candidates = []
        for version in metadata.get("versions", []):
            number = version.get("num")
            if version.get("yanked") is True or not isinstance(number, str):
                continue
            try:
                parsed = SemVer.parse_stable(number)
            except BaselineError:
                continue
            if row["policy"] == "latest-compatible":
                minimum = SemVer.parse_stable(row["compatible_min"])
                maximum = SemVer.parse_stable(row["compatible_max"])
                if not minimum <= parsed < maximum:
                    continue
            candidates.append((parsed, version))
        if not candidates:
            raise BaselineError(f"release authority has no eligible version for {row['name']}")
        _, selected = max(candidates, key=lambda item: item[0])
        row["version"] = selected["num"]
        row["release_authority"] = f"https://crates.io/crates/{row['name']}/{selected['num']}"
        row["checksum"] = selected["checksum"]
        row["license"] = selected["license"]
        row["published_at"] = selected["created_at"]
        row["yanked"] = False

    releases = fetch_json("https://api.github.com/repos/pganalyze/libpg_query/releases?per_page=100")
    for row in refreshed["source"]:
        major = row["server_major"]
        prefixes = (f"{major}.", f"{major}-")
        release = next(
            (
                item
                for item in releases
                if not item.get("draft")
                and not item.get("prerelease")
                and str(item.get("tag_name", "")).startswith(prefixes)
            ),
            None,
        )
        if release is None:
            raise BaselineError(f"release authority has no libpg_query tag for PostgreSQL {major}")
        tag = str(release["tag_name"])
        row["tag"] = tag
        row["commit"] = resolve_github_tag(tag)
        row["release_authority"] = f"https://github.com/pganalyze/libpg_query/releases/tag/{tag}"
    validate_baseline(refreshed)
    return refreshed


def resolve_github_tag(tag: str) -> str:
    encoded = urllib.parse.quote(tag, safe="")
    payload = fetch_json(f"https://api.github.com/repos/pganalyze/libpg_query/git/ref/tags/{encoded}")
    current = payload["object"]
    while current["type"] == "tag":
        current = fetch_json(current["url"])["object"]
    if current["type"] != "commit" or HEX40.fullmatch(current["sha"]) is None:
        raise BaselineError(f"tag {tag} does not resolve to a commit")
    return str(current["sha"])


def render_toml(payload: dict[str, Any]) -> str:
    lines = []
    for field in ("schema_version", "verified_at", "generated_by", "root_lock_package", "rust_version", "sqlite_amalgamation"):
        lines.append(render_assignment(field, payload[field]))
    for table, fields in (("crate", CRATE_FIELDS), ("source", SOURCE_FIELDS)):
        for row in payload[table]:
            lines.extend(("", f"[[{table}]]"))
            for field in fields:
                if field in row:
                    lines.append(render_assignment(field, row[field]))
    return "\n".join(lines) + "\n"


def render_assignment(name: str, value: object) -> str:
    if isinstance(value, bool):
        rendered = "true" if value else "false"
    elif isinstance(value, int):
        rendered = str(value)
    elif isinstance(value, str):
        rendered = json.dumps(value)
    else:
        raise BaselineError(f"cannot render TOML value for {name}")
    return f"{name} = {rendered}"


def self_test() -> None:
    payload = load_baseline()
    validate_baseline(payload)
    mutations = [
        ("prerelease", lambda data: data["crate"][0].__setitem__("version", "1.53.2-rc.1")),
        ("yanked", lambda data: data["crate"][0].__setitem__("yanked", True)),
        ("broad-range", lambda data: data["crate"][0].__setitem__("version", "1")),
        ("incompatible-family", lambda data: data["crate"][10].__setitem__("version", "0.38.2")),
        ("unlocked-source", lambda data: data["source"][0].__setitem__("tag", "main")),
        ("missing-commit", lambda data: data["source"][0].__setitem__("commit", "pending")),
        ("missing-source-authority", lambda data: data["crate"][11].pop("source_authority")),
        ("duplicate-crate", lambda data: data["crate"].append(copy.deepcopy(data["crate"][0]))),
    ]
    for label, mutate in mutations:
        candidate = copy.deepcopy(payload)
        mutate(candidate)
        try:
            validate_baseline(candidate)
        except BaselineError:
            continue
        raise AssertionError(f"baseline mutation was accepted: {label}")
    print(f"dependency baseline self-test ok: mutations={len(mutations)}")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--refresh", action="store_true")
    mode.add_argument("--verify-authorities", action="store_true")
    mode.add_argument("--self-test", action="store_true")
    parser.add_argument("--output", type=Path, default=BASELINE_PATH)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    payload = load_baseline()
    if args.refresh:
        refreshed = refresh(payload)
        args.output.write_text(render_toml(refreshed), encoding="utf-8")
        print(f"dependency baseline refreshed: {args.output}")
        return 0
    if args.verify_authorities:
        refreshed = refresh(payload)
        if render_toml(refreshed) != render_toml(payload):
            raise BaselineError("dependency baseline differs from current release authorities")
        print("dependency release authorities match the checked baseline")
        return 0
    validate_baseline(payload)
    print(f"dependency baseline ok: crates={len(payload['crate'])}, sources={len(payload['source'])}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (BaselineError, OSError, urllib.error.URLError) as error:
        print(f"dependency baseline error: {error}", file=sys.stderr)
        raise SystemExit(1) from error

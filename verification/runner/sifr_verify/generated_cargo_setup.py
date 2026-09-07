"""Prepare real portable generated Cargo graphs during the online profile prelude."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tomllib
from pathlib import Path

from .paths import REPO_ROOT
from .profile_commands import run_command
from .profiles import load_profile

GIT_SOURCE = "https://github.com/sifr-lang/sifr.git"


def quality_module():
    sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "generated_code_quality"))
    import generated_code_quality

    return generated_code_quality


def preparation_entries(quality, suites: list[str]):
    # Preparation covers the complete positive manifest, independently of the
    # execution adapter's smoke/representative limits or inherited CLI filters.
    entries = [entry for entry in quality.load_manifest(quality.MANIFEST)
               if entry.group in quality.POSITIVE_GROUPS]
    if "full" in suites or "companions" in suites:
        entries.extend(entry for _, entry in quality.authoritative_companion_entries())
    return list({entry.id: entry for entry in entries}.values())


def portable_graph(crate_root: Path, revision: str) -> dict[str, str]:
    """Reject stale/local graphs before Cargo can use them or change a lock."""
    if re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        raise ValueError("generated Cargo preparation requires an exact Git revision")
    manifest_path = crate_root / "Cargo.toml"
    lock_path = crate_root / "Cargo.lock"
    manifest = tomllib.loads(manifest_path.read_text())
    lock = tomllib.loads(lock_path.read_text())
    if manifest.get("patch") or manifest.get("replace"):
        raise ValueError(f"{manifest_path}: portable graph must not override sources")
    sections = [manifest, *manifest.get("target", {}).values()]
    for section in sections:
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            for name, spec in section.get(kind, {}).items():
                if not isinstance(spec, dict):
                    continue
                if "path" in spec:
                    raise ValueError(f"{manifest_path}: local dependency {name}")
                if spec.get("git") == GIT_SOURCE and spec.get("rev") != revision:
                    raise ValueError(f"{manifest_path}: stale Sifr revision for {name}")
    expected_source = f"git+{GIT_SOURCE}?rev={revision}#{revision}"
    for package in lock.get("package", []):
        if package["name"] in {"sifr_runtime", "sifr_stdlib"}:
            if package.get("source") != expected_source:
                raise ValueError(f"{lock_path}: nonportable or stale {package['name']}")
    return {name: hashlib.sha256((crate_root / name).read_bytes()).hexdigest()
            for name in ("Cargo.toml", "Cargo.lock")}


def fetch_generated_graph(crate_root: Path, revision: str, command_runner=run_command):
    before = portable_graph(crate_root, revision)
    command_runner(["cargo", "fetch", "--locked", "--manifest-path", str(crate_root / "Cargo.toml")],
                   env=os.environ.copy())
    if portable_graph(crate_root, revision) != before:
        raise ValueError(f"{crate_root}: preparation changed the generated manifest or lock")
    return before


def prepare_generated_graphs(profile: dict, revision: str) -> dict:
    if os.environ.get("CARGO_NET_OFFLINE", "").lower() in {"true", "1"}:
        raise ValueError("generated Cargo preparation must precede offline execution")
    quality = quality_module()
    suites = [suite for area in profile["selected_areas"] if area["area"] == "generated_code_quality"
              for suite in area["suites"]]
    # This build uses the outer workspace target; generated commands retain
    # their existing separate target through SIFR_GCQ_SHARED_ROOT.
    run_command(["cargo", "build", "--locked", "-p", "sifr"], env=os.environ.copy())
    shared_root = quality.shared_artifact_root()
    if shared_root is None:
        raise ValueError("generated Cargo preparation requires its profile-owned artifact root")
    records = []
    for entry in preparation_entries(quality, suites):
        crate_root = quality.materialize_entry(entry, shared_root / "preparation")
        hashes = fetch_generated_graph(crate_root, revision)
        records.append({"id": entry.id, "group": entry.group,
                        "crate_root": str(crate_root.relative_to(REPO_ROOT)), **hashes})
        print(f"[sifr-profile-setup] generated={entry.id} revision={revision} status=pass", flush=True)
    report = {"revision": revision, "profile": profile["name"], "entries": records}
    path = REPO_ROOT / "target" / "verification" / "areas" / f"generated-cargo-setup-{profile['name']}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    prepare_generated_graphs(load_profile(args.profile), args.revision)


if __name__ == "__main__":
    main()

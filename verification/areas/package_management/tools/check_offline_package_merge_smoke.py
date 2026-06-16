#!/usr/bin/env python3
"""Validate the repo-local offline package-management merge smoke fixture."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
FIXTURE_ROOT = REPO_ROOT / "verification/areas/package_management/fixtures/offline_registry"
REGISTRY_PATH = FIXTURE_ROOT / "registry.json"
WORKSPACE_PATH = FIXTURE_ROOT / "workspace/package.json"
DEMO_LOCK_DIGESTS_PATH = (
    REPO_ROOT / "verification/areas/package_management/data/offline_demo_lockfile_digests.json"
)
GENERATED_BY = "verification/areas/package_management/tools/check_offline_package_merge_smoke.py"
LOCK_PACKAGE_RE = re.compile(r'^name = "([^"]+)"\nversion = "([^"]+)"$', re.MULTILINE)


@dataclass(frozen=True)
class Dependency:
    name: str
    version: str

    @property
    def package_id(self) -> str:
        return f"{self.name}@{self.version}#offline-registry"


@dataclass(frozen=True)
class RegistryPackage:
    name: str
    version: str
    root: Path
    checksum_sha256: str
    dependencies: tuple[Dependency, ...]

    @property
    def package_id(self) -> str:
        return f"{self.name}@{self.version}#offline-registry"


@dataclass(frozen=True)
class WorkspacePackage:
    name: str
    version: str
    lockfile: Path
    dependencies: tuple[Dependency, ...]

    @property
    def package_id(self) -> str:
        return f"{self.name}@{self.version}#workspace"


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        failures = run_self_test()
    elif args.demo_corpus:
        failures = run_demo_corpus_check()
    else:
        failures = run_fixture_check()
    if failures:
        for failure in failures:
            print(f"offline package smoke failure: {failure}", file=sys.stderr)
        return 1
    print("offline package merge smoke ok")
    return 0


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Exercise negative fixture mutations to prove the checker fails closed.",
    )
    parser.add_argument(
        "--demo-corpus",
        action="store_true",
        help="Validate broader demo-repository lockfile determinism for nightly/release.",
    )
    return parser.parse_args(argv)


def run_fixture_check() -> list[str]:
    failures: list[str] = []
    registry = load_registry(REGISTRY_PATH, failures)
    workspace = load_workspace(WORKSPACE_PATH, failures)
    if registry is None or workspace is None:
        return failures

    failures.extend(validate_registry_sources(registry))
    lock = build_lock(workspace, registry)
    failures.extend(validate_lockfile(workspace.lockfile, lock))
    failures.extend(validate_graph_behavior(lock))

    reversed_registry = dict(reversed(list(registry.items())))
    if lock != build_lock(workspace, reversed_registry):
        failures.append("lockfile generation must be deterministic across registry input order")
    reversed_workspace = WorkspacePackage(
        name=workspace.name,
        version=workspace.version,
        lockfile=workspace.lockfile,
        dependencies=tuple(reversed(workspace.dependencies)),
    )
    if lock != build_lock(reversed_workspace, registry):
        failures.append("lockfile generation must be deterministic across workspace dependency order")
    return failures


def run_self_test() -> list[str]:
    registry = load_registry(REGISTRY_PATH, [])
    workspace = load_workspace(WORKSPACE_PATH, [])
    if registry is None or workspace is None:
        return ["self-test baseline fixture could not load"]

    lock = build_lock(workspace, registry)
    failures: list[str] = []
    if not validate_lockfile(workspace.lockfile, {**lock, "graph_digest_sha256": "bad"}):
        failures.append("self-test expected graph digest mutation to fail")

    broken_registry = dict(registry)
    first_key = next(iter(broken_registry))
    package = broken_registry[first_key]
    broken_registry[first_key] = RegistryPackage(
        name=package.name,
        version=package.version,
        root=package.root,
        checksum_sha256="0" * 64,
        dependencies=package.dependencies,
    )
    if not validate_registry_sources(broken_registry):
        failures.append("self-test expected checksum mutation to fail")

    missing_dependency = WorkspacePackage(
        name=workspace.name,
        version=workspace.version,
        lockfile=workspace.lockfile,
        dependencies=(Dependency("missing-package", "1.0.0"),),
    )
    try:
        build_lock(missing_dependency, registry)
    except FixtureError:
        pass
    else:
        failures.append("self-test expected missing dependency to fail")
    return failures


def run_demo_corpus_check() -> list[str]:
    failures: list[str] = []
    raw = load_json(DEMO_LOCK_DIGESTS_PATH, failures)
    if not isinstance(raw, dict):
        failures.append(f"{relative(DEMO_LOCK_DIGESTS_PATH)} must contain a JSON object")
        return failures
    if raw.get("schema_version") != 1:
        failures.append(f"{relative(DEMO_LOCK_DIGESTS_PATH)} must use schema_version 1")

    seen_paths: set[str] = set()
    for entry in expect_list(raw.get("lockfiles"), "demo lockfiles", failures):
        if not isinstance(entry, dict):
            failures.append("demo lockfile entries must be objects")
            continue
        path = expect_string(entry.get("path"), "demo lockfile path", failures)
        sha256 = expect_string(entry.get("sha256"), "demo lockfile sha256", failures)
        if not path or not sha256:
            continue
        if "://" in path or Path(path).is_absolute() or ".." in Path(path).parts:
            failures.append(f"demo lockfile path must be repo-relative: {path}")
            continue
        if path in seen_paths:
            failures.append(f"duplicate demo lockfile digest entry: {path}")
        seen_paths.add(path)

        lockfile = REPO_ROOT / path
        if not lockfile.is_file():
            failures.append(f"missing demo lockfile: {path}")
            continue
        actual = hashlib.sha256(lockfile.read_bytes()).hexdigest()
        if actual != sha256:
            failures.append(f"{path} digest mismatch: expected {sha256}, got {actual}")
        locked_packages = parse_lock_packages(lockfile.read_text(encoding="utf-8"))
        expected_packages = entry.get("expected_packages_present")
        for required in expect_list(
            expected_packages,
            f"{path} expected_packages_present",
            failures,
        ):
            if not isinstance(required, str):
                failures.append(f"{path} expected package entries must be strings")
                continue
            if required not in locked_packages:
                failures.append(f"{path} must contain locked package {required}")
    if not seen_paths:
        failures.append("demo corpus lockfile digest list must not be empty")
    return failures


def load_registry(path: Path, failures: list[str]) -> dict[str, RegistryPackage] | None:
    raw = load_json(path, failures)
    if not isinstance(raw, dict):
        failures.append(f"{relative(path)} must contain a JSON object")
        return None
    if raw.get("schema_version") != 1:
        failures.append(f"{relative(path)} must use schema_version 1")
    if raw.get("name") != "sifr-offline-fixture":
        failures.append(f"{relative(path)} must name the repo-local offline fixture")

    registry: dict[str, RegistryPackage] = {}
    for entry in expect_list(raw.get("packages"), f"{relative(path)} packages", failures):
        package = parse_registry_package(path, entry, failures)
        if package is None:
            continue
        if package.package_id in registry:
            failures.append(f"duplicate offline package id: {package.package_id}")
        registry[package.package_id] = package
    if not registry:
        failures.append("offline registry fixture must contain at least one package")
    return registry


def parse_registry_package(
    registry_path: Path,
    entry: object,
    failures: list[str],
) -> RegistryPackage | None:
    if not isinstance(entry, dict):
        failures.append("offline registry package entry must be an object")
        return None
    name = expect_string(entry.get("name"), "registry package name", failures)
    version = expect_string(entry.get("version"), f"{name or '<unknown>'} version", failures)
    raw_path = expect_string(entry.get("path"), f"{name or '<unknown>'} path", failures)
    checksum = expect_string(
        entry.get("checksum_sha256"),
        f"{name or '<unknown>'} checksum_sha256",
        failures,
    )
    if not name or not version or not raw_path or not checksum:
        return None
    if len(checksum) != 64 or any(char not in "0123456789abcdef" for char in checksum):
        failures.append(f"{name}@{version} checksum_sha256 must be a lowercase SHA-256 hex digest")
    if "://" in raw_path or Path(raw_path).is_absolute() or ".." in Path(raw_path).parts:
        failures.append(f"{name}@{version} path must be a relative offline fixture path")
        return None

    root = registry_path.parent / raw_path
    manifest = load_json(root / "manifest.json", failures)
    dependencies = parse_dependencies(
        manifest.get("dependencies") if isinstance(manifest, dict) else None,
        f"{name}@{version} dependencies",
        failures,
    )
    if isinstance(manifest, dict):
        if manifest.get("name") != name:
            failures.append(f"{relative(root / 'manifest.json')} name must match registry entry")
        if manifest.get("version") != version:
            failures.append(f"{relative(root / 'manifest.json')} version must match registry entry")
    return RegistryPackage(name, version, root, checksum, dependencies)


def load_workspace(path: Path, failures: list[str]) -> WorkspacePackage | None:
    raw = load_json(path, failures)
    if not isinstance(raw, dict):
        failures.append(f"{relative(path)} must contain a JSON object")
        return None
    if raw.get("schema_version") != 1:
        failures.append(f"{relative(path)} must use schema_version 1")
    name = expect_string(raw.get("name"), "workspace package name", failures)
    version = expect_string(raw.get("version"), "workspace package version", failures)
    lockfile = expect_string(raw.get("lockfile"), "workspace lockfile", failures)
    dependencies = parse_dependencies(raw.get("dependencies"), "workspace dependencies", failures)
    if not name or not version or not lockfile:
        return None
    if "://" in lockfile or Path(lockfile).is_absolute() or ".." in Path(lockfile).parts:
        failures.append("workspace lockfile must be a relative fixture path")
        return None
    return WorkspacePackage(name, version, path.parent / lockfile, dependencies)


def build_lock(
    workspace: WorkspacePackage,
    registry: dict[str, RegistryPackage],
) -> dict[str, object]:
    resolved: dict[str, RegistryPackage] = {}
    edges: set[tuple[str, str]] = set()

    def visit(parent_id: str, dependency: Dependency, trail: tuple[str, ...]) -> None:
        package = registry.get(dependency.package_id)
        if package is None:
            raise FixtureError(f"missing offline registry package {dependency.package_id}")
        if package.package_id in trail:
            cycle = " -> ".join((*trail, package.package_id))
            raise FixtureError(f"offline package dependency cycle: {cycle}")
        edges.add((parent_id, package.package_id))
        if package.package_id in resolved:
            return
        resolved[package.package_id] = package
        for child in package.dependencies:
            visit(package.package_id, child, (*trail, package.package_id))

    for dependency in workspace.dependencies:
        visit(workspace.package_id, dependency, (workspace.package_id,))

    packages = [
        {
            "id": workspace.package_id,
            "name": workspace.name,
            "version": workspace.version,
            "source": "workspace",
            "checksum_sha256": None,
            "dependencies": [dependency.package_id for dependency in workspace.dependencies],
        }
    ]
    for package in sorted(resolved.values(), key=lambda item: item.package_id):
        packages.append(
            {
                "id": package.package_id,
                "name": package.name,
                "version": package.version,
                "source": "offline-registry",
                "checksum_sha256": package.checksum_sha256,
                "dependencies": [dependency.package_id for dependency in package.dependencies],
            }
        )
    graph_edges = [[left, right] for left, right in sorted(edges)]
    digest_payload = {"packages": packages, "graph_edges": graph_edges}
    return {
        "schema_version": 1,
        "generated_by": GENERATED_BY,
        "graph_digest_sha256": digest_json(digest_payload),
        "packages": packages,
        "graph_edges": graph_edges,
    }


def validate_registry_sources(registry: dict[str, RegistryPackage]) -> list[str]:
    failures: list[str] = []
    for package in registry.values():
        actual = digest_tree(package.root)
        if actual != package.checksum_sha256:
            failures.append(
                f"{package.package_id} checksum mismatch: expected "
                f"{package.checksum_sha256}, got {actual}"
            )
    return failures


def validate_lockfile(lockfile: Path, expected: dict[str, object]) -> list[str]:
    failures: list[str] = []
    actual = load_json(lockfile, failures)
    if actual != expected:
        failures.append(f"{relative(lockfile)} does not match deterministic offline lock output")
    return failures


def validate_graph_behavior(lock: dict[str, object]) -> list[str]:
    failures: list[str] = []
    packages = lock.get("packages")
    graph_edges = lock.get("graph_edges")
    if not isinstance(packages, list) or not isinstance(graph_edges, list):
        return ["lock must contain package and graph_edges lists"]
    package_ids = {entry.get("id") for entry in packages if isinstance(entry, dict)}
    expected_ids = {
        "offline-app@0.1.0#workspace",
        "sifr-json@1.0.0#offline-registry",
        "sifr-core@1.0.0#offline-registry",
    }
    if package_ids != expected_ids:
        failures.append(f"package graph ids mismatch: expected {sorted(expected_ids)}, got {sorted(package_ids)}")
    expected_edges = {
        ("offline-app@0.1.0#workspace", "sifr-json@1.0.0#offline-registry"),
        ("sifr-json@1.0.0#offline-registry", "sifr-core@1.0.0#offline-registry"),
    }
    edge_set = {tuple(edge) for edge in graph_edges if isinstance(edge, list) and len(edge) == 2}
    if edge_set != expected_edges:
        failures.append(f"package graph edges mismatch: expected {sorted(expected_edges)}, got {sorted(edge_set)}")
    for package in packages:
        if not isinstance(package, dict):
            failures.append("package graph package entries must be objects")
            continue
        if package.get("source") == "offline-registry" and not package.get("checksum_sha256"):
            failures.append(f"{package.get('id')} must carry an offline registry checksum")
    return failures


def parse_dependencies(raw: object, label: str, failures: list[str]) -> tuple[Dependency, ...]:
    dependencies: list[Dependency] = []
    for entry in expect_list(raw, label, failures):
        if not isinstance(entry, dict):
            failures.append(f"{label} entries must be objects")
            continue
        name = expect_string(entry.get("name"), f"{label} name", failures)
        version = expect_string(entry.get("version"), f"{label} version", failures)
        if name and version:
            dependencies.append(Dependency(name, version))
    return tuple(sorted(dependencies, key=lambda item: item.package_id))


def load_json(path: Path, failures: list[str]) -> Any:
    if not path.is_file():
        failures.append(f"missing fixture file: {relative(path)}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        failures.append(f"{relative(path)} is invalid JSON: {error}")
        return None


def expect_list(raw: object, label: str, failures: list[str]) -> list[object]:
    if isinstance(raw, list):
        return raw
    failures.append(f"{label} must be a list")
    return []


def expect_string(raw: object, label: str, failures: list[str]) -> str | None:
    if isinstance(raw, str) and raw:
        return raw
    failures.append(f"{label} must be a non-empty string")
    return None


def digest_tree(root: Path) -> str:
    hasher = hashlib.sha256()
    for path in sorted(file for file in root.rglob("*") if file.is_file()):
        relative_path = path.relative_to(root).as_posix()
        hasher.update(relative_path.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(path.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()


def digest_json(payload: object) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def parse_lock_packages(source: str) -> set[str]:
    return {f"{name}@{version}" for name, version in LOCK_PACKAGE_RE.findall(source)}


def relative(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return str(path)


class FixtureError(Exception):
    """Raised when fixture dependency resolution fails closed."""


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except FixtureError as exc:
        print(f"offline package smoke failure: {exc}", file=sys.stderr)
        raise SystemExit(1)

from __future__ import annotations

import argparse
import ast
import copy
import json
import re
import tomllib
from datetime import date
from importlib.metadata import version
from pathlib import Path
from urllib.parse import urlparse

from dependency_requirements import (
    discover_projects, requirements, validate_requirements,
)

REPO_ROOT = Path(__file__).resolve().parents[4]
AUDIT_PATH = (
    REPO_ROOT / "verification/areas/python_interop/data/latest_stable_python.json"
)
LIVE_CASE_CONFIG_PATH = (
    REPO_ROOT / "verification/areas/python_interop/runner/live_case_config.py"
)
NORMALIZED_NAME = re.compile(r"[-_.]+")
RETIRED_DISTRIBUTIONS = frozenset({"httpcore", "httpx"})
EXPECTED_SERVICE_IMAGES = frozenset({"localstack", "redis"})


def normalize_name(name: str) -> str:
    return NORMALIZED_NAME.sub("-", name).lower()


def load_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as source:
        return tomllib.load(source)


def load_literal_assignment(path: Path, name: str) -> object:
    module = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    for statement in module.body:
        if not isinstance(statement, ast.Assign):
            continue
        if any(
            isinstance(target, ast.Name) and target.id == name
            for target in statement.targets
        ):
            return ast.literal_eval(statement.value)
    raise ValueError(f"{path}: missing literal assignment {name}")


def release_map(audit: dict[str, object]) -> dict[str, dict[str, object]]:
    packages = audit.get("packages")
    if not isinstance(packages, list):
        raise ValueError("audit packages must be a list")
    releases: dict[str, dict[str, object]] = {}
    for package in packages:
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            raise ValueError("every audited package must have a string name")
        name = normalize_name(package["name"])
        if name in releases:
            raise ValueError(f"duplicate audited package: {name}")
        artifact = package.get("artifact")
        if not isinstance(artifact, dict):
            raise ValueError(f"{name}: artifact must be an object")
        digest = artifact.get("sha256")
        if not isinstance(digest, str) or len(digest) != 64:
            raise ValueError(f"{name}: artifact SHA-256 must contain 64 characters")
        if any(character not in "0123456789abcdef" for character in digest):
            raise ValueError(f"{name}: artifact SHA-256 must be lowercase hexadecimal")
        releases[name] = package
    expected = frozenset().union(*(frozenset(p["packages"]) for p in audit["projects"]))
    if releases.keys() != expected:
        raise ValueError("audited package set differs from the maintained package set")
    return releases


def runtime_version_marker(*names: str) -> str:
    """Validate installed feature dependencies against the canonical audit."""
    audit = json.loads(AUDIT_PATH.read_text(encoding="utf-8"))
    releases = release_map(audit)
    markers = []
    for name in names:
        expected = releases[name]["selected_version"]
        installed = version(name)
        if installed != expected:
            raise RuntimeError(
                f"{name}: expected audited release {expected}, found {installed}"
            )
        markers.append(f"{name}={installed}")
    return " ".join(markers)


def run_runtime_version_self_tests(audit: dict[str, object]) -> int:
    from unittest.mock import patch

    names = (
        "schwifty", "numpy", "pandas", "redis", "fakeredis", "hiredis", "testcontainers"
    )
    releases = release_map(audit)
    installed = {name: releases[name]["selected_version"] for name in names}
    with patch(f"{__name__}.version", side_effect=installed.__getitem__):
        expected = " ".join(f"{name}={installed[name]}" for name in names)
        if runtime_version_marker(*names) != expected:
            raise AssertionError("runtime version markers differ from audited releases")
        for name in names:
            original = installed[name]
            installed[name] = "0.0.0"
            try:
                runtime_version_marker(*names)
            except RuntimeError as error:
                if str(error) != (
                    f"{name}: expected audited release {original}, found {installed[name]}"
                ):
                    raise AssertionError(
                        "runtime version mismatch lost its identity"
                    ) from error
            else:
                raise AssertionError(f"stale installed release was accepted: {name}")
            finally:
                installed[name] = original

        changed = copy.deepcopy(audit)
        for package in changed["packages"]:
            if package["name"] in names:
                package["selected_version"] = installed[package["name"]] = "99.0.1"
        with patch(f"{__name__}.AUDIT_PATH") as audit_path:
            audit_path.read_text.return_value = json.dumps(changed)
            expected_changed = " ".join(f"{name}=99.0.1" for name in names)
            if runtime_version_marker(*names) != expected_changed:
                raise AssertionError("runtime version markers did not follow the audit")
    return len(names) + 1


def project_map(
    audit: dict[str, object],
) -> dict[str, tuple[str, str, frozenset[str]]]:
    projects = audit.get("projects")
    if not isinstance(projects, list):
        raise ValueError("audit projects must be a list")
    mapped: dict[str, tuple[str, str, frozenset[str]]] = {}
    for project in projects:
        if not isinstance(project, dict):
            raise ValueError("every audited project must be an object")
        name = project.get("name")
        packages = project.get("packages")
        if not isinstance(name, str):
            raise ValueError("every audited project must have a string name")
        if not isinstance(packages, list) or not all(
            isinstance(package, str) for package in packages
        ):
            raise ValueError(f"{name}: packages must be strings")
        if name in mapped:
            raise ValueError(f"duplicate audited project: {name}")
        pyproject, lock = project["pyproject"], project["lock"]
        mapped[name] = (
            pyproject,
            lock,
            frozenset(normalize_name(package) for package in packages),
        )
    paths = [entry[0] for entry in mapped.values()]
    if len(paths) != len(set(paths)):
        raise ValueError("multiple audited owners claim the same project")
    return mapped


def direct_dependency_names(project: dict[str, object]) -> frozenset[str]:
    return frozenset(normalize_name(req.name) for _, req in requirements(project))


def validate_project(
    label: str,
    expected: frozenset[str],
    releases: dict[str, dict[str, object]],
    project: dict[str, object],
    lock: dict[str, object],
) -> list[str]:
    errors: list[str] = []
    direct = direct_dependency_names(project)
    for name in sorted(RETIRED_DISTRIBUTIONS.intersection(direct)):
        errors.append(f"{label}: retired direct dependency: {name}")
    for name in sorted(expected.difference(direct)):
        errors.append(f"{label}: missing direct dependency {name}")
    for name in sorted(direct.difference(expected).difference(RETIRED_DISTRIBUTIONS)):
        errors.append(f"{label}: unaudited direct dependency {name}")

    locked_packages = lock.get("package")
    if not isinstance(locked_packages, list):
        return [*errors, f"{label}: uv.lock packages must be a list"]
    locked_names = {
        normalize_name(str(package.get("name", "")))
        for package in locked_packages
        if isinstance(package, dict)
    }
    for name in sorted(RETIRED_DISTRIBUTIONS.intersection(locked_names)):
        errors.append(f"{label}: retired package remains locked: {name}")
    for name in sorted(expected):
        matching = [
            package
            for package in locked_packages
            if isinstance(package, dict)
            and normalize_name(str(package.get("name", ""))) == name
        ]
        if len(matching) != 1:
            errors.append(
                f"{label}: expected one locked {name} package, found {len(matching)}"
            )
            continue
        package = matching[0]
        release = releases[name]
        if package.get("version") != release["selected_version"]:
            errors.append(
                f"{label}: {name} lock version {package.get('version')!r} is not "
                f"audited selection {release['selected_version']!r}"
            )
        artifact = release["artifact"]
        assert isinstance(artifact, dict)
        filename = artifact["filename"]
        digest = f"sha256:{artifact['sha256']}"
        candidates = []
        sdist = package.get("sdist")
        if isinstance(sdist, dict):
            candidates.append(sdist)
        wheels = package.get("wheels")
        if isinstance(wheels, list):
            candidates.extend(wheel for wheel in wheels if isinstance(wheel, dict))
        if not any(
            Path(urlparse(str(candidate.get("url", ""))).path).name == filename
            and candidate.get("hash") == digest
            for candidate in candidates
        ):
            errors.append(f"{label}: {name} lock lacks audited artifact {filename}")
        official = {entry["url"]: entry for entry in release["artifacts"]}
        for candidate in candidates:
            entry = official.get(candidate.get("url"))
            if entry is None or candidate.get("hash") != "sha256:" + entry["sha256"] or entry["yanked"]:
                errors.append(f"{label}: {name} locked artifact is not an authenticated nonyanked PyPI file")
    return errors


def validate_repository(audit: dict[str, object]) -> list[str]:
    if audit.get("schema_version") != 4:
        raise ValueError("unsupported stable-release audit schema")
    audited_at = audit.get("audited_at")
    if not isinstance(audited_at, str):
        raise ValueError("stable-release audit date must be a string")
    try:
        date.fromisoformat(audited_at)
    except ValueError as error:
        raise ValueError(
            "stable-release audit date must use ISO 8601 format"
        ) from error
    if audit.get("python") != "3.14.7":
        raise ValueError("stable-release audit must target Python 3.14.7")
    releases = release_map(audit)
    projects = project_map(audit)
    discovered = discover_projects(REPO_ROOT)
    if {pyproject: lock for pyproject, lock, _ in projects.values()} != discovered:
        raise ValueError("audited projects/locks differ from actual maintained discovery")
    errors: list[str] = []
    if audit.get("deferred_packages") != {"kafka-python": "61"}:
        raise ValueError("Kafka convergence must retain explicit Item61 ownership")
    for name, release in releases.items():
        if release["selected_version"] != release["latest_stable"] and name not in audit["deferred_packages"]:
            errors.append(f"{name}: selected {release['selected_version']} is behind official latest stable {release['latest_stable']}")
    for name, (pyproject_path, lock_path, expected) in projects.items():
        project = load_toml(REPO_ROOT / pyproject_path)
        lock = load_toml(REPO_ROOT / lock_path)
        errors.extend(
            validate_project(
                name,
                expected,
                releases,
                project,
                lock,
            )
        )
        owner = next(owner for owner in audit["projects"] if owner["name"] == name)
        errors.extend(validate_requirements(name, project, lock, releases, owner["requirements"]))
    errors.extend(validate_service_images(audit))
    return errors


def validate_service_images(audit: dict[str, object]) -> list[str]:
    images = audit.get("service_images")
    if not isinstance(images, list):
        return ["audit service_images must be a list"]
    mapped = {
        image.get("name"): image
        for image in images
        if isinstance(image, dict) and isinstance(image.get("name"), str)
    }
    if len(mapped) != len(images) or mapped.keys() != EXPECTED_SERVICE_IMAGES:
        return ["audit service image set differs from the maintained image set"]
    live_images = load_literal_assignment(LIVE_CASE_CONFIG_PATH, "LIVE_IMAGES")
    if not isinstance(live_images, dict):
        return ["live service image mapping must be a dictionary"]
    errors = []
    for name, image in mapped.items():
        repository = image.get("image")
        version = image.get("latest_stable")
        digest = image.get("manifest_digest")
        if not all(isinstance(value, str) for value in (repository, version, digest)):
            errors.append(f"{name}: image audit fields must be strings")
            continue
        if not digest.startswith("sha256:") or len(digest) != 71:
            errors.append(f"{name}: manifest digest must be a SHA-256 value")
            continue
        expected = f"{repository}:{version}@{digest}"
        if live_images.get(name) != expected:
            errors.append(
                f"{name}: live image pin does not match audited stable image {expected}"
            )
    return errors


def run_self_tests(audit: dict[str, object]) -> int:
    releases = release_map(audit)
    projects = project_map(audit)
    project_name = "python-interop"
    pyproject_path, lock_path, expected = projects[project_name]
    project = load_toml(REPO_ROOT / pyproject_path)
    lock = load_toml(REPO_ROOT / lock_path)

    stale_lock = copy.deepcopy(lock)
    next(
        package
        for package in stale_lock["package"]
        if isinstance(package, dict) and package.get("name") == "polars"
    )["version"] = "1.44.0"
    if not validate_project(project_name, expected, releases, project, stale_lock):
        raise AssertionError("stale lock mutation was not rejected")

    secondary_name = "dlpack-demo"
    secondary_pyproject_path, secondary_lock_path, secondary_expected = projects[
        secondary_name
    ]
    secondary_project = load_toml(REPO_ROOT / secondary_pyproject_path)
    secondary_lock = load_toml(REPO_ROOT / secondary_lock_path)
    stale_secondary_lock = copy.deepcopy(secondary_lock)
    next(
        package
        for package in stale_secondary_lock["package"]
        if isinstance(package, dict) and package.get("name") == "numpy"
    )["version"] = "2.5.1"
    if not validate_project(
        secondary_name,
        secondary_expected,
        releases,
        secondary_project,
        stale_secondary_lock,
    ):
        raise AssertionError("stale secondary lock mutation was not rejected")

    stale_pyarrow = copy.deepcopy(lock)
    next(
        package
        for package in stale_pyarrow["package"]
        if isinstance(package, dict) and package.get("name") == "pyarrow"
    )["version"] = "25.0.0"
    if not validate_project(project_name, expected, releases, project, stale_pyarrow):
        raise AssertionError("stale PyArrow mutation was not rejected")

    missing_artifact = copy.deepcopy(lock)
    next(
        package
        for package in missing_artifact["package"]
        if isinstance(package, dict) and package.get("name") == "torch"
    )["wheels"] = []
    if not validate_project(
        project_name, expected, releases, project, missing_artifact
    ):
        raise AssertionError("artifact mutation was not rejected")

    missing_direct = copy.deepcopy(project)
    missing_direct["project"]["dependencies"] = [
        requirement
        for requirement in missing_direct["project"]["dependencies"]
        if not str(requirement).startswith("schwifty")
    ]
    if not validate_project(project_name, expected, releases, missing_direct, lock):
        raise AssertionError("direct-dependency mutation was not rejected")

    retired_mutations = 0
    for label in (*projects, "renamed-owner"):
        for retired in sorted(RETIRED_DISTRIBUTIONS):
            retired_dependency = copy.deepcopy(lock)
            retired_dependency["package"].append(
                {"name": retired.upper(), "version": "0.0.0"}
            )
            errors = validate_project(
                label, expected, releases, project, retired_dependency
            )
            if errors != [f"{label}: retired package remains locked: {retired}"]:
                raise AssertionError(f"retired lock mutation failed: {label}/{retired}")
            retired_direct = copy.deepcopy(project)
            retired_direct["project"]["dependencies"].append(
                f"{retired.upper()}[extra]>=0; python_version >= '3.14'"
            )
            errors = validate_project(label, expected, releases, retired_direct, lock)
            if errors != [f"{label}: retired direct dependency: {retired}"]:
                raise AssertionError(f"retired direct mutation failed: {label}/{retired}")
            retired_mutations += 2

    stale_image = copy.deepcopy(audit)
    stale_image["service_images"][0]["latest_stable"] = "4.13.1"
    if not validate_service_images(stale_image):
        raise AssertionError("stale service-image mutation was not rejected")
    return 6 + retired_mutations + run_runtime_version_self_tests(audit)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    audit = json.loads(AUDIT_PATH.read_text(encoding="utf-8"))
    errors = validate_repository(audit)
    mutation_count = run_self_tests(audit) if args.self_test else 0
    if args.self_test:
        import unittest
        from test_dependency_versions import DependencyProjectPathsTests, RequirementAuthorityTests
        suite = unittest.TestSuite(
            unittest.defaultTestLoader.loadTestsFromTestCase(case)
            for case in (DependencyProjectPathsTests, RequirementAuthorityTests)
        )
        result = unittest.TextTestRunner().run(suite)
        if not result.wasSuccessful():
            errors.append("dependency requirement authority regression tests failed")
    if errors:
        print(f"python dependency audit mutations={mutation_count}; convergence remains unsatisfied")
        for error in errors:
            print(f"python dependency audit error: {error}")
        return 1
    releases = release_map(audit)
    projects = project_map(audit)
    lock_count = len({lock for _, lock, _ in projects.values()})
    images = audit.get("service_images")
    image_count = len(images) if isinstance(images, list) else 0
    print(
        f"python dependency audit ok: projects={len(projects)} "
        f"packages={len(releases)} locks={lock_count} images={image_count} "
        f"mutations={mutation_count}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

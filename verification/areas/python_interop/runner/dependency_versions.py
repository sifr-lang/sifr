from __future__ import annotations

import argparse
import ast
import copy
import json
import re
import tomllib
from datetime import date
from pathlib import Path
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[4]
AUDIT_PATH = (
    REPO_ROOT / "verification/areas/python_interop/data/latest_stable_python.json"
)
LIVE_CASE_CONFIG_PATH = (
    REPO_ROOT / "verification/areas/python_interop/runner/live_case_config.py"
)
EXPECTED_PROJECTS = {
    "python-interop": frozenset(
        {
            "alembic",
            "boto3",
            "certifi",
            "cffi",
            "cryptography",
            "fakeredis",
            "fastapi",
            "hiredis",
            "httpx2",
            "numpy",
            "pandas",
            "polars",
            "pyarrow",
            "redis",
            "schwifty",
            "sqlalchemy",
            "starlette",
            "testcontainers",
            "torch",
        }
    ),
    "dlpack-demo": frozenset({"numpy", "torch"}),
}
DLPACK_PROJECT_ROOT = Path("demos/python_dlpack")
PROJECT_PATHS = {
    "python-interop": (
        "verification/areas/python_interop/pyproject.toml",
        "verification/areas/python_interop/uv.lock",
    ),
    "dlpack-demo": (
        str(DLPACK_PROJECT_ROOT / "pyproject.toml"),
        str(DLPACK_PROJECT_ROOT / "uv.lock"),
    ),
}
NORMALIZED_NAME = re.compile(r"[-_.]+")
REQUIREMENT_NAME = re.compile(r"^[A-Za-z0-9._-]+")
RETIRED_PYTHON_INTEROP_PACKAGES = frozenset({"httpcore", "httpx"})
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
    expected = frozenset().union(*EXPECTED_PROJECTS.values())
    if releases.keys() != expected:
        raise ValueError("audited package set differs from the maintained package set")
    return releases


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
        if name not in PROJECT_PATHS:
            raise ValueError(f"unknown audited project: {name}")
        pyproject, lock = PROJECT_PATHS[name]
        mapped[name] = (
            pyproject,
            lock,
            frozenset(normalize_name(package) for package in packages),
        )
    if mapped.keys() != EXPECTED_PROJECTS.keys():
        raise ValueError("audited project set differs from the maintained project set")
    for name, expected in EXPECTED_PROJECTS.items():
        if mapped[name][2] != expected:
            raise ValueError(f"{name}: audited package ownership drifted")
    return mapped


def direct_dependency_names(project: dict[str, object]) -> frozenset[str]:
    metadata = project.get("project")
    if not isinstance(metadata, dict):
        return frozenset()
    dependencies = metadata.get("dependencies")
    if not isinstance(dependencies, list):
        return frozenset()
    names = set()
    for requirement in dependencies:
        if not isinstance(requirement, str):
            continue
        match = REQUIREMENT_NAME.match(requirement)
        if match is not None:
            names.add(normalize_name(match.group()))
    return frozenset(names)


def validate_project(
    label: str,
    expected: frozenset[str],
    releases: dict[str, dict[str, object]],
    project: dict[str, object],
    lock: dict[str, object],
) -> list[str]:
    errors: list[str] = []
    direct = direct_dependency_names(project)
    for name in sorted(expected.difference(direct)):
        errors.append(f"{label}: missing direct dependency {name}")

    locked_packages = lock.get("package")
    if not isinstance(locked_packages, list):
        return [*errors, f"{label}: uv.lock packages must be a list"]
    locked_names = {
        normalize_name(str(package.get("name", "")))
        for package in locked_packages
        if isinstance(package, dict)
    }
    if label == "python-interop":
        for name in sorted(RETIRED_PYTHON_INTEROP_PACKAGES.intersection(locked_names)):
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
        if package.get("version") != release.get("latest_stable"):
            errors.append(
                f"{label}: {name} lock version {package.get('version')!r} is not "
                f"latest stable {release.get('latest_stable')!r}"
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
    return errors


def validate_repository(audit: dict[str, object]) -> list[str]:
    if audit.get("schema_version") != 3:
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
    errors: list[str] = []
    for name, (pyproject_path, lock_path, expected) in projects.items():
        errors.extend(
            validate_project(
                name,
                expected,
                releases,
                load_toml(REPO_ROOT / pyproject_path),
                load_toml(REPO_ROOT / lock_path),
            )
        )
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

    retired_dependency = copy.deepcopy(lock)
    retired_dependency["package"].append({"name": "httpx", "version": "0.28.1"})
    if not validate_project(
        project_name, expected, releases, project, retired_dependency
    ):
        raise AssertionError("retired HTTP client mutation was not rejected")

    stale_image = copy.deepcopy(audit)
    stale_image["service_images"][0]["latest_stable"] = "4.13.1"
    if not validate_service_images(stale_image):
        raise AssertionError("stale service-image mutation was not rejected")
    return 7


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    audit = json.loads(AUDIT_PATH.read_text(encoding="utf-8"))
    errors = validate_repository(audit)
    if errors:
        for error in errors:
            print(f"python dependency audit error: {error}")
        return 1
    mutation_count = run_self_tests(audit) if args.self_test else 0
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

"""Discover maintained Python owners and validate their actual requirement graph."""
from __future__ import annotations

import subprocess
from pathlib import Path, PurePosixPath

from packaging.markers import default_environment
from packaging.requirements import Requirement
from packaging.specifiers import SpecifierSet
from packaging.utils import canonicalize_name
from packaging.version import Version

PYTHON = "3.14.7"
EXCLUDED = {"vendor", "third_party", "fixtures", "corpora", "snapshots", "target", ".venv", "plans"}


def discover_projects(root: Path) -> dict[str, str]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root, capture_output=True, text=True, check=True, timeout=30,
    )
    paths = {name for name in result.stdout.split("\0") if name
             and not EXCLUDED.intersection(PurePosixPath(name).parts)}
    manifests = {name for name in paths if PurePosixPath(name).name == "pyproject.toml"}
    locks = {name for name in paths if PurePosixPath(name).name == "uv.lock"}
    projects = {name: str(PurePosixPath(name).with_name("uv.lock")) for name in manifests}
    if set(projects.values()) != locks:
        raise ValueError(f"maintained project/lock discovery mismatch: {sorted(set(projects.values()) ^ locks)}")
    for path in sorted(manifests | locks):
        if (root / path).is_symlink() or not (root / path).is_file():
            raise ValueError(f"missing or symlinked maintained Python input: {path}")
    return dict(sorted(projects.items()))


def requirements(project: dict) -> list[tuple[str, Requirement]]:
    metadata = project.get("project")
    if not isinstance(metadata, dict):
        raise ValueError("maintained Python project lacks [project] metadata")
    rows = []

    def add(section, raw):
        if not isinstance(raw, list) or not all(isinstance(r, str) for r in raw):
            raise ValueError(f"{section}: requirements must be a string list")
        rows.extend((section, Requirement(r)) for r in raw)

    add("project.dependencies", metadata.get("dependencies", []))
    for extra, raw in metadata.get("optional-dependencies", {}).items():
        add(f"project.optional-dependencies.{extra}", raw)
    groups = project.get("dependency-groups", {})

    def group(name, stack):
        if name in stack or name not in groups:
            raise ValueError(f"invalid dependency group inclusion: {[*stack, name]}")
        raw = []
        for entry in groups[name]:
            if isinstance(entry, dict) and set(entry) == {"include-group"}:
                raw.extend(group(entry["include-group"], [*stack, name]))
            elif isinstance(entry, str):
                raw.append(entry)
            else:
                raise ValueError(f"invalid requirement in dependency group {name}")
        return raw

    for name in groups:
        add(f"dependency-groups.{name}", group(name, []))
    add("build-system.requires", project.get("build-system", {}).get("requires", []))
    return rows


def requirement_record(section: str, requirement: Requirement) -> dict:
    return {"section": section, "name": canonicalize_name(requirement.name),
            "extras": sorted(canonicalize_name(e) for e in requirement.extras),
            "specifier": str(requirement.specifier),
            "marker": str(requirement.marker) if requirement.marker else None,
            "url": requirement.url}


def requirement_records(project: dict) -> list[dict]:
    rows = [requirement_record(section, requirement) for section, requirement in requirements(project)]
    return sorted(rows, key=lambda r: (r["section"], r["name"], str(r)))


def lane_environments() -> list[dict[str, str]]:
    env = default_environment()
    env.update(python_version="3.14", python_full_version=PYTHON,
               implementation_name="cpython", implementation_version=PYTHON,
               platform_python_implementation="CPython", extra="")
    return [{**env, "sys_platform": platform, "platform_system": system,
             "os_name": os_name, "platform_machine": machine}
            for platform, system, os_name, machine in (
                ("darwin", "Darwin", "posix", "arm64"),
                ("linux", "Linux", "posix", "x86_64"),
                ("win32", "Windows", "nt", "AMD64"),
            )]


def locked_requirement(raw: dict, section: str) -> dict:
    name = canonicalize_name(raw["name"])
    extras = "[" + ",".join(raw.get("extras", [])) + "]" if raw.get("extras") else ""
    specifier = raw.get("specifier", "")
    if "url" in raw:
        specifier = " @ " + raw["url"]
    marker = "; " + raw["marker"] if raw.get("marker") else ""
    return requirement_record(section, Requirement(name + extras + specifier + marker))


def validate_requirements(label: str, project: dict, lock: dict, releases: dict,
                          audited_requirements: list[dict]) -> list[str]:
    errors = []
    actual = requirement_records(project)
    if actual != audited_requirements:
        errors.append(f"{label}: direct requirements/extras/markers differ from audit")
    for description, value in (("project", project["project"].get("requires-python")),
                               ("lock", lock.get("requires-python"))):
        if value != "==" + PYTHON:
            errors.append(f"{label}: {description} must select only Python {PYTHON}")
    roots = [p for p in lock.get("package", [])
             if canonicalize_name(p.get("name", "")) == canonicalize_name(project["project"]["name"])]
    if len(roots) != 1:
        return [*errors, f"{label}: expected one project package in uv.lock"]
    metadata = roots[0].get("metadata", {})
    for section, declared in (("project.dependencies", roots[0].get("dependencies", [])),
                              *((f"dependency-groups.{group}", entries)
                                for group, entries in roots[0].get("dev-dependencies", {}).items())):
        names = {canonicalize_name(entry["name"]) for entry in declared}
        expected_names = {canonicalize_name(req.name) for owner, req in requirements(project)
                          if owner == section}
        if names != expected_names:
            errors.append(f"{label}: uv.lock resolved direct edges differ for {section}")
    declared_groups = set(project.get("dependency-groups", {}))
    if set(roots[0].get("dev-dependencies", {})) != declared_groups:
        errors.append(f"{label}: uv.lock dependency group ownership differs from manifest")
    build_names = {canonicalize_name(req.name) for section, req in requirements(project)
                   if section == "build-system.requires"}
    if build_names:
        closure = set()
        pending = list(build_names)
        packages = {p["name"]: p for p in lock["package"]}
        while pending:
            name = pending.pop()
            if name in closure:
                continue
            closure.add(name)
            if name not in packages:
                errors.append(f"{label}: build dependency {name} is not locked")
                continue
            pending.extend(d["name"] for d in packages[name].get("dependencies", []))
        constraints = project.get("tool", {}).get("uv", {}).get("build-constraint-dependencies", [])
        actual_constraints = {str(Requirement(raw)) for raw in constraints}
        expected_constraints = {name + "==" + packages[name]["version"]
                                for name in closure if name in packages}
        if actual_constraints != expected_constraints:
            errors.append(f"{label}: build constraints must pin the complete locked backend closure")
    locked = [locked_requirement(r, "project.dependencies")
              for r in metadata.get("requires-dist", [])]
    for group, rows in metadata.get("requires-dev", {}).items():
        locked.extend(locked_requirement(r, f"dependency-groups.{group}") for r in rows)
    # uv stores optional dependencies with their PEP508 extra marker.
    expected = []
    for section, requirement in requirements(project):
        if section == "build-system.requires":
            continue
        if section.startswith("project.optional-dependencies."):
            extra = section.removeprefix("project.optional-dependencies.")
            base = str(requirement).split(";", 1)[0]
            marker = f"({requirement.marker}) and extra == '{extra}'" if requirement.marker else f"extra == '{extra}'"
            requirement = Requirement(base + "; " + marker)
            section = "project.dependencies"
        expected.append(requirement_record(section, requirement))
    key = lambda r: (r["section"], r["name"], str(r))
    if sorted(locked, key=key) != sorted(expected, key=key):
        errors.append(f"{label}: uv.lock direct requirement metadata differs from manifest")
    for section, requirement in requirements(project):
        name = canonicalize_name(requirement.name)
        release = releases.get(name)
        if release is None:
            errors.append(f"{label}: unaudited direct requirement {name}")
            continue
        selected = release["selected_version"]
        if requirement.url or not requirement.specifier.contains(selected, prereleases=False):
            errors.append(f"{label}: {name} requirement does not admit locked {selected}")
        if section == "build-system.requires" and str(requirement.specifier) != "==" + selected:
            errors.append(f"{label}: build requirement {name} must be exactly pinned")
        if section == "build-system.requires" and not any(
            owner.startswith("dependency-groups.") and canonicalize_name(req.name) == name
            for owner, req in requirements(project)
        ):
            errors.append(f"{label}: build requirement {name} lacks a locked dependency group")
        environments = lane_environments()
        extra = section.removeprefix("project.optional-dependencies.") if section.startswith("project.optional-dependencies.") else ""
        if requirement.marker and not any(requirement.marker.evaluate({**env, "extra": extra}) for env in environments):
            errors.append(f"{label}: {name} marker excludes the maintained Python lane")
        available_extras = set(release.get("provides_extra", []))
        missing = {canonicalize_name(e) for e in requirement.extras} - available_extras
        if missing:
            errors.append(f"{label}: {name} requests unavailable extras {sorted(missing)}")
        # Requires-Python is optional core metadata; absence means no restriction.
        python_requirement = release["requires_python"]
        if python_requirement is None:
            python_requirement = ""
        if not SpecifierSet(python_requirement).contains(PYTHON, prereleases=False):
            errors.append(f"{label}: locked {name} Requires-Python excludes {PYTHON}")
        if Version(selected).is_prerelease or Version(selected).is_devrelease:
            errors.append(f"{label}: {name} selected a nonstable release")
    return errors

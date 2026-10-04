"""The single declared CPU wheel variant; all other owners keep PyPI authority."""
from __future__ import annotations

import re
from datetime import date

from packaging.markers import Marker, default_environment
from packaging.requirements import Requirement
from packaging.specifiers import SpecifierSet

PROJECT = "sifr-python-interop-verification"
PYPROJECT = "verification/areas/python_interop/pyproject.toml"
CPU_MARKER = "sys_platform == 'linux' and platform_machine == 'x86_64'"
PYPI_MARKER = "sys_platform != 'linux' or platform_machine != 'x86_64'"
PYPI_SOURCE = {"registry": "https://pypi.org/simple"}


def validate_distribution_audit(releases: dict) -> None:
    """A declared variant is mandatory and confined to this one owned lane."""
    for name, release in releases.items():
        if name != "torch" and "verification_cpu" in release:
            raise ValueError(f"{name}: CPU distribution belongs only to torch")
    release = releases["torch"]
    variant = release.get("verification_cpu")
    fields = {"project", "pyproject", "marker", "distribution_version", "upstream_version",
              "url", "sha256", "size_bytes", "index_url", "index_sha256", "observed_at",
              "metadata_url", "metadata_sha256", "requires_python", "requires_dist"}
    if not isinstance(variant, dict) or set(variant) != fields:
        raise ValueError("torch: exact CPU distribution audit is required")
    version = release["selected_version"]
    url = ("https://download.pytorch.org/whl/cpu/torch-" + version
           + "%2Bcpu-cp314-cp314-manylinux_2_28_x86_64.whl")
    identities = {"project": PROJECT, "pyproject": PYPROJECT, "marker": CPU_MARKER,
                  "distribution_version": version + "+cpu", "upstream_version": version,
                  "url": url, "metadata_url": url + ".metadata",
                  "index_url": "https://download.pytorch.org/whl/cpu/torch/"}
    if any(variant[key] != value for key, value in identities.items()):
        raise ValueError("torch: CPU distribution owner/platform/source/version drift")
    for name in ("sha256", "index_sha256", "metadata_sha256"):
        if not isinstance(variant[name], str) or not re.fullmatch("[a-f0-9]{64}", variant[name]):
            raise ValueError("torch: invalid CPU distribution digest")
    if type(variant["size_bytes"]) is not int or variant["size_bytes"] <= 0:
        raise ValueError("torch: invalid CPU distribution size")
    date.fromisoformat(variant["observed_at"])
    if not SpecifierSet(variant["requires_python"]).contains("3.14.7"):
        raise ValueError("torch: CPU distribution excludes canonical Python")
    if not isinstance(variant["requires_dist"], list) or not all(
        isinstance(raw, str) for raw in variant["requires_dist"]
    ):
        raise ValueError("torch: CPU distribution requires authenticated dependency metadata")
    runtime = cpu_requirements(variant)
    if set(runtime) != {"filelock", "typing-extensions", "setuptools", "sympy",
                        "networkx", "jinja2", "fsspec"}:
        raise ValueError("torch: CPU distribution runtime dependency inventory drift")


def cpu_requirements(variant: dict) -> dict[str, Requirement]:
    environment = {**default_environment(), "sys_platform": "linux",
                   "platform_machine": "x86_64", "python_version": "3.14",
                   "python_full_version": "3.14.7", "extra": ""}
    result = {}
    for raw in variant["requires_dist"]:
        requirement = Requirement(raw)
        if requirement.marker is None or requirement.marker.evaluate(environment):
            if requirement.name in result or requirement.url or requirement.extras:
                raise ValueError("torch: ambiguous CPU distribution requirement")
            result[requirement.name] = requirement
    return result


def is_cpu_project(project: dict) -> bool:
    return project.get("project", {}).get("name") == PROJECT


def runtime_distribution_version(name: str, releases: dict, environment: dict | None = None) -> str:
    release = releases[name]
    if name == "torch" and Marker(CPU_MARKER).evaluate(environment or default_environment()):
        return release["verification_cpu"]["distribution_version"]
    return release["selected_version"]


def validate_distribution_project(label: str, project: dict, lock: dict, releases: dict) -> list[str]:
    variant = releases["torch"]["verification_cpu"]
    configured = project.get("tool", {}).get("uv", {}).get("sources", {}).get("torch")
    expected_source = [{"url": variant["url"], "marker": CPU_MARKER}]
    if not is_cpu_project(project):
        return [f"{label}: CPU source is restricted to its verification owner"] if configured else []
    errors = []
    if configured != expected_source:
        errors.append(f"{label}: exact CPU source marker/URL differs from audit")
    torch = [p for p in lock["package"] if p.get("name") == "torch"]
    cpu = [p for p in torch if p.get("version") == variant["distribution_version"]]
    base = [p for p in torch if p.get("version") == releases["torch"]["selected_version"]]
    if len(torch) != 2 or len(cpu) != 1 or len(base) != 1:
        return [*errors, f"{label}: expected exactly the audited CPU and PyPI torch variants"]
    package = cpu[0]
    if package.get("source") != {"url": variant["url"]}:
        errors.append(f"{label}: CPU locked source differs from audit")
    if package.get("wheels") != [{"url": variant["url"], "hash": "sha256:" + variant["sha256"]}]:
        errors.append(f"{label}: CPU locked wheel/hash differs from audit")
    if set(package) != {"name", "version", "source", "dependencies", "wheels"}:
        errors.append(f"{label}: CPU lock has unexpected metadata")
    requirements = cpu_requirements(variant)
    if package.get("dependencies") != [{"name": name} for name in sorted(requirements)]:
        errors.append(f"{label}: CPU dependency closure differs from authenticated metadata")
    for name, requirement in requirements.items():
        matches = [p for p in lock["package"] if p.get("name") == name]
        if len(matches) != 1 or not requirement.specifier.contains(matches[0].get("version", "0")):
            errors.append(f"{label}: CPU requirement {name} is not satisfied by its lock")
    roots = [p for p in lock["package"] if p.get("name") == PROJECT]
    edges = [edge for p in roots for edge in p.get("dependencies", []) if edge.get("name") == "torch"]
    expected = [
        {"name": "torch", "version": releases["torch"]["selected_version"],
         "source": PYPI_SOURCE, "marker": PYPI_MARKER},
        {"name": "torch", "version": variant["distribution_version"],
         "source": {"url": variant["url"]}, "marker": CPU_MARKER},
    ]
    if len(roots) != 1 or edges != expected:
        errors.append(f"{label}: CPU/PyPI direct edge partition differs from audit")
    return errors


def distribution_requirement_records(project: dict, records: list[dict], releases: dict) -> list[dict]:
    """uv records source overrides as an exact URL edge plus the PyPI complement."""
    if not is_cpu_project(project):
        return records
    result = []
    for row in records:
        if row["section"] == "project.dependencies" and row["name"] == "torch":
            variant = releases["torch"]["verification_cpu"]
            result.extend([
                {**row, "marker": str(Marker(PYPI_MARKER))},
                {**row, "specifier": "", "marker": str(Marker(CPU_MARKER)), "url": variant["url"]},
            ])
        else:
            result.append(row)
    return result

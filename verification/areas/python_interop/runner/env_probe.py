from __future__ import annotations

import importlib.machinery
import json
import platform
import site
import struct
import sys
import sysconfig
from pathlib import Path
from typing import Any

REQUIRED_NEGATIVE_RULES = {
    "unsupported_interpreter",
    "free_threaded_cpython",
    "venv_prefix_mismatch",
    "missing_site_packages",
    "missing_declared_import",
    "native_import_failure",
    "multiple_venv_selection",
    "missing_venv_selection",
    "stale_project_or_lock",
    "probe_execution_failure",
}


def run_env_probe(area_root: Path) -> dict[str, Any]:
    fixture_root = area_root / "fixtures" / "env_probe"
    positive = load_json(fixture_root / "positive_probe.json")
    negative = load_json(fixture_root / "negative_probe_cases.json")
    validate_fixture_contract(positive, negative)
    live_probe = current_interpreter_probe()
    return {
        "status": "passed",
        "implementation_name": live_probe["implementation_name"],
        "implementation_version": live_probe["implementation_version"],
        "pointer_width": live_probe["pointer_width"],
        "soabi": live_probe["soabi"],
        "extension_suffixes": live_probe["extension_suffixes"],
        "fixture_contract": {
            "positive": positive["name"],
            "negative_rules": sorted({case["expected_rule"] for case in negative["cases"]}),
        },
        "uv_sync_invoked": False,
    }


def validate_fixture_contract(positive: dict[str, Any], negative: dict[str, Any]) -> None:
    validate_positive_probe(positive)
    validate_negative_cases(negative)


def validate_positive_probe(positive: dict[str, Any]) -> None:
    rule = validate_probe(positive["request"], positive["probe"], interpreter_exists=True)
    if rule is not None:
        raise SystemExit(f"env positive fixture failed validation rule: {rule}")

def validate_negative_cases(negative: dict[str, Any]) -> None:
    cases = negative.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit("env negative fixture must contain cases")
    rules = {case.get("expected_rule") for case in cases}
    missing = sorted(REQUIRED_NEGATIVE_RULES.difference(rules))
    if missing:
        raise SystemExit(f"env negative fixture is missing rule(s): {', '.join(missing)}")
    for case in cases:
        expected = case.get("expected_rule")
        if expected not in REQUIRED_NEGATIVE_RULES:
            raise SystemExit(f"env negative fixture has unknown rule: {expected}")
        actual = validate_negative_case(case)
        if actual != expected:
            raise SystemExit(
                f"env negative case {case.get('name', '<unnamed>')} expected {expected}, got {actual}"
            )


def validate_negative_case(case: dict[str, Any]) -> str | None:
    if "graph" in case:
        return validate_graph_selection(case["graph"])
    return validate_probe(
        case["request"],
        case.get("probe"),
        interpreter_exists=case.get("interpreter_exists", True),
    )


def validate_graph_selection(graph: dict[str, Any]) -> str | None:
    selections = graph.get("venv_selections", [])
    if not selections and graph.get("requires_python"):
        return "missing_venv_selection"
    distinct = {selection["venv"] for selection in selections}
    if len(distinct) > 1:
        return "multiple_venv_selection"
    return None


def validate_probe(
    request: dict[str, Any],
    probe: dict[str, Any] | None,
    *,
    interpreter_exists: bool,
) -> str | None:
    if not interpreter_exists:
        return "probe_execution_failure"
    if probe is None:
        return "probe_execution_failure"
    if probe.get("implementation_name") != "CPython":
        return "unsupported_interpreter"
    if probe.get("free_threaded") is True:
        return "free_threaded_cpython"
    venv = request["venv_root"]
    if not path_within(probe["sys_prefix"], venv) or same_path(
        probe["sys_prefix"], probe["sys_base_prefix"]
    ):
        return "venv_prefix_mismatch"
    if not any(path_within(path, venv) for path in probe.get("site_packages", [])):
        return "missing_site_packages"
    if any(not item.get("ok") for item in probe.get("imports", [])):
        return "missing_declared_import"
    if any(not item.get("ok") for item in probe.get("native_imports", [])):
        return "native_import_failure"
    if request.get("pyproject") and probe.get("pyproject_digest") is None:
        return "stale_project_or_lock"
    if request.get("lock") and probe.get("uv_lock_digest") is None:
        return "stale_project_or_lock"
    return None


def same_path(left: str, right: str) -> bool:
    return Path(left).resolve() == Path(right).resolve()


def path_within(path: str, parent: str) -> bool:
    resolved = Path(path).resolve()
    resolved_parent = Path(parent).resolve()
    return resolved == resolved_parent or resolved_parent in resolved.parents


def current_interpreter_probe() -> dict[str, Any]:
    return {
        "implementation_name": platform.python_implementation(),
        "implementation_version": platform.python_version(),
        "executable": str(Path(sys.executable).resolve()),
        "sys_prefix": str(Path(sys.prefix).resolve()),
        "sys_base_prefix": str(Path(sys.base_prefix).resolve()),
        "site_packages": [str(Path(path).resolve()) for path in site.getsitepackages()],
        "sys_path": [str(Path(path).resolve()) for path in sys.path],
        "soabi": sysconfig.get_config_var("SOABI"),
        "extension_suffixes": list(importlib.machinery.EXTENSION_SUFFIXES),
        "pointer_width": struct.calcsize("P") * 8,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "free_threaded": bool(sysconfig.get_config_var("Py_GIL_DISABLED")),
    }


def load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise SystemExit(f"missing env fixture: {path}")
    return json.loads(path.read_text(encoding="utf-8"))

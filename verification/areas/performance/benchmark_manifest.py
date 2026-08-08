"""Schema validation and deterministic selection for performance benchmarks."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
RUNNER_VERSION = 1
COMMAND_KINDS = {"command", "frontend-query", "lsp-query"}
COMMAND_MODES = {"check", "build", "fmt-check"}


class BenchmarkError(Exception):
    pass


@dataclass(frozen=True)
class BenchmarkCase:
    raw: dict[str, Any]

    @property
    def id(self) -> str:
        return str(self.raw["id"])

    @property
    def group(self) -> str:
        return str(self.raw["group"])

    @property
    def kind(self) -> str:
        return str(self.raw["kind"])

    @property
    def measured(self) -> int:
        return int(self.raw["measured"])

    @property
    def warmups(self) -> int:
        return int(self.raw["warmups"])

    @property
    def timeout_ms(self) -> int:
        return int(self.raw["timeout_ms"])

    @property
    def stability_limit(self) -> float:
        return float(self.raw.get("stability_limit", 0.10))

    @property
    def work_stability_limit(self) -> float:
        return float(self.raw.get("work_stability_limit", 0.02))


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise BenchmarkError(f"failed to read JSON {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise BenchmarkError(f"malformed JSON {path}: {error}") from error
    if not isinstance(data, dict):
        raise BenchmarkError(f"{path} root must be an object")
    return data


def validate_manifest(manifest: dict[str, Any]) -> list[BenchmarkCase]:
    if manifest.get("version") != 1:
        raise BenchmarkError("benchmark manifest version must be 1")
    if manifest.get("runner_version") != RUNNER_VERSION:
        raise BenchmarkError(f"benchmark manifest runner_version must be {RUNNER_VERSION}")
    cases_raw = manifest.get("cases")
    if not isinstance(cases_raw, list):
        raise BenchmarkError("benchmark manifest cases must be a list")

    cases: list[BenchmarkCase] = []
    ids: list[str] = []
    budget_ids: list[str] = []
    for raw in cases_raw:
        if not isinstance(raw, dict):
            raise BenchmarkError("benchmark manifest case entries must be objects")
        validate_case(raw)
        case = BenchmarkCase(raw)
        cases.append(case)
        ids.append(case.id)
        budget_ids.append(str(raw["budget_id"]))

    if ids != sorted(ids):
        raise BenchmarkError("benchmark cases must be sorted lexicographically by id")
    if len(ids) != len(set(ids)):
        raise BenchmarkError("benchmark case ids must be unique")
    if len(budget_ids) != len(set(budget_ids)):
        raise BenchmarkError("benchmark budget ids must be unique")

    required = manifest.get("required_groups", {})
    if not isinstance(required, dict):
        raise BenchmarkError("benchmark manifest required_groups must be an object")
    by_group: dict[str, int] = {}
    for case in cases:
        by_group[case.group] = by_group.get(case.group, 0) + 1
    for group, minimum in required.items():
        if not isinstance(minimum, int) or minimum < 0:
            raise BenchmarkError(f"required group {group!r} must have a non-negative integer threshold")
        actual = by_group.get(group, 0)
        if actual < minimum:
            raise BenchmarkError(f"manifest group {group!r} has {actual} cases, need >= {minimum}")

    return cases


def validate_case(raw: dict[str, Any]) -> None:
    required = {
        "id",
        "group",
        "kind",
        "source_path",
        "warmups",
        "measured",
        "timeout_ms",
        "budget_id",
        "evidence_category",
    }
    missing = sorted(required - raw.keys())
    if missing:
        raise BenchmarkError(f"benchmark case is missing required fields: {missing}")
    for field in [
        "id",
        "group",
        "kind",
        "source_path",
        "budget_id",
        "evidence_category",
    ]:
        if not isinstance(raw[field], str) or not raw[field]:
            raise BenchmarkError(f"benchmark case field {field} must be a non-empty string")
    if raw["kind"] not in COMMAND_KINDS:
        raise BenchmarkError(f"benchmark case {raw['id']} has unsupported kind {raw['kind']!r}")
    for field in ["warmups", "measured", "timeout_ms"]:
        if not isinstance(raw[field], int) or raw[field] <= 0:
            raise BenchmarkError(f"benchmark case {raw['id']} field {field} must be a positive integer")
    if raw["kind"] == "command":
        if raw.get("mode") not in COMMAND_MODES:
            raise BenchmarkError(f"command benchmark {raw['id']} must use mode check, build, or fmt-check")
        exit_codes = raw.get("expected_exit_codes")
        if not isinstance(exit_codes, list) or not exit_codes or not all(isinstance(code, int) for code in exit_codes):
            raise BenchmarkError(f"command benchmark {raw['id']} must define integer expected_exit_codes")
        global_args = raw.get("global_args", [])
        if not isinstance(global_args, list) or not all(isinstance(value, str) for value in global_args):
            raise BenchmarkError(f"command benchmark {raw['id']} global_args must be a list of strings")
    if raw["kind"] == "frontend-query":
        if not isinstance(raw.get("scenario"), str) or not raw["scenario"]:
            raise BenchmarkError(f"frontend query benchmark {raw['id']} must define scenario")
    if raw["kind"] == "lsp-query":
        if not isinstance(raw.get("scenario"), str) or not raw["scenario"]:
            raise BenchmarkError(f"LSP query benchmark {raw['id']} must define scenario")
        if raw.get("workspace_mode") not in {"isolated", "package"}:
            raise BenchmarkError(f"LSP query benchmark {raw['id']} must define workspace_mode as isolated or package")
    path = REPO_ROOT / raw["source_path"]
    if not path.exists():
        raise BenchmarkError(f"benchmark case {raw['id']} input path does not exist: {raw['source_path']}")
    if raw["kind"] == "lsp-query":
        has_manifest = path.parent.joinpath("sifr.toml").is_file()
        if raw["workspace_mode"] == "package" and not has_manifest:
            raise BenchmarkError(f"package LSP benchmark {raw['id']} requires a sibling sifr.toml")
        if raw["workspace_mode"] == "isolated" and has_manifest:
            raise BenchmarkError(f"isolated LSP benchmark {raw['id']} cannot use a package source")


def select_cases(
    cases: list[BenchmarkCase],
    groups: set[str],
    case_ids: set[str],
    case_limit: int,
) -> list[BenchmarkCase]:
    selected = [case for case in cases if (not groups or case.group in groups) and (not case_ids or case.id in case_ids)]
    if case_ids:
        known = {case.id for case in cases}
        missing = sorted(case_ids - known)
        if missing:
            raise BenchmarkError(f"unknown benchmark case ids requested: {missing}")
    if case_limit:
        if case_limit < 0:
            raise BenchmarkError("--case-limit must be non-negative")
        selected = selected[:case_limit]
    return selected

#!/usr/bin/env python3
"""Validate diagnostic catalog, rendered baseline ownership, and recovery coverage."""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import sys
from typing import Any

import code_coverage


ROOT = pathlib.Path(__file__).resolve().parents[4]
AREA_ROOT = ROOT / "verification" / "areas" / "diagnostics"
DATA_ROOT = AREA_ROOT / "data"
MANIFEST_PATH = AREA_ROOT / "manifest.json"
CATALOG_PATH = DATA_ROOT / "code_catalog.json"
COVERAGE_PATH = DATA_ROOT / "code_baseline_coverage.json"
METADATA_PATH = DATA_ROOT / "baseline_metadata.json"
RECOVERY_PATH = DATA_ROOT / "recovery_surface_coverage.json"
ALLOWED_RENDERERS = {"human", "json", "compact"}
ALLOWED_STABILITY = {"stable", "unstable", "internal"}
NORMALIZERS = {"workspace-path", "tmp-path", "crlf", "artifact-cache-lines", "json-sort"}
CODE_RE = re.compile(r"^SIFR-[A-Z]+-\d{4}$")
BASELINE_RE = re.compile(r"^(?P<label>.+)\.(?P<stream>stdout|stderr|exit-code)\.txt$")


def load_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def repo_relative(path: pathlib.Path) -> str:
    return str(path.relative_to(ROOT))


def sha256_file(path: pathlib.Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def active_registry() -> dict[str, dict[str, str]]:
    _, active_code_to_constant, active_fixtures = code_coverage.parse_registry()
    source = code_coverage.read_rust_with_local_sources(code_coverage.CODES_RS)
    severities = {
        code: severity
        for _constant, code, severity in re.findall(
            r"pub const ([A-Z0-9_]+): Self\s*=\s*(?:\n\s*)?Self::new\(\""
            + f"({code_coverage.CODE_RE})"
            + r"\",\s*Severity::([A-Za-z]+)\)",
            source,
        )
    }
    return {
        code: {
            "constant": constant,
            "fixture": active_fixtures.get(code, ""),
            "severity": severities.get(code, ""),
        }
        for code, constant in active_code_to_constant.items()
    }


def manifest_baseline_cases() -> dict[str, dict[str, Any]]:
    manifest = load_json(MANIFEST_PATH)
    cases: dict[str, dict[str, Any]] = {}
    for suite in manifest.get("suites", []):
        suite_name = str(suite.get("name", ""))
        for case in suite.get("cases", []):
            if not isinstance(case, dict) or case.get("command") == "area-check":
                continue
            case_id = str(case.get("id"))
            if case_id in cases:
                raise ValueError(f"duplicate diagnostics baseline case id: {case_id}")
            entry = ROOT / str(case.get("entry", ""))
            formats = case.get("diagnostic_formats") or [None]
            cases[case_id] = {
                "suite": suite_name,
                "entry": entry,
                "command": str(case.get("command")),
                "formats": {str(item) for item in formats if item is not None},
            }
    return cases


def expected_baseline_files(cases: dict[str, dict[str, Any]]) -> set[pathlib.Path]:
    files: set[pathlib.Path] = set()
    for case in cases.values():
        entry = case["entry"]
        command = case["command"]
        for renderer in case["formats"]:
            label = f"{command}-{renderer}"
            baseline_dir = entry.parent / "baselines"
            files.add(baseline_dir / f"{label}.stdout.txt")
            files.add(baseline_dir / f"{label}.stderr.txt")
            files.add(baseline_dir / f"{label}.exit-code.txt")
    return files


def actual_baseline_files() -> set[pathlib.Path]:
    fixture_root = AREA_ROOT / "fixtures" / "diagnostics"
    return {
        path
        for path in fixture_root.glob("**/baselines/*.txt")
        if BASELINE_RE.fullmatch(path.name)
    }


def baseline_file_keys(files: set[pathlib.Path]) -> dict[tuple[str, str], set[str]]:
    keys: dict[tuple[str, str], set[str]] = {}
    fixture_root = AREA_ROOT / "fixtures" / "diagnostics"
    for path in files:
        match = BASELINE_RE.fullmatch(path.name)
        if match is None:
            continue
        label = match.group("label")
        renderer = label.rsplit("-", maxsplit=1)[-1]
        fixture_id = path.relative_to(fixture_root).parts[0]
        keys.setdefault((fixture_id, renderer), set()).add(match.group("stream"))
    return keys


def validate_catalog(errors: list[str], active: dict[str, dict[str, str]]) -> dict[str, dict[str, Any]]:
    payload = load_json(CATALOG_PATH)
    entries = payload.get("codes")
    if payload.get("schema_version") != 1 or not isinstance(entries, list):
        errors.append("code_catalog.json must have schema_version=1 and a codes array")
        return {}
    by_code: dict[str, dict[str, Any]] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            errors.append("code_catalog.json entries must be objects")
            continue
        code = str(entry.get("code", ""))
        if code in by_code:
            errors.append(f"{code}: duplicate catalog entry")
        by_code[code] = entry
        if not CODE_RE.fullmatch(code):
            errors.append(f"{code}: invalid diagnostic code")
        if entry.get("severity") not in {"Error", "Warning", "Note"}:
            errors.append(f"{code}: invalid severity {entry.get('severity')!r}")
        if entry.get("stability") not in ALLOWED_STABILITY:
            errors.append(f"{code}: invalid stability {entry.get('stability')!r}")
        if not entry.get("owner"):
            errors.append(f"{code}: missing owner")
        if entry.get("docs_link") != f"docs/errors/{code}.md":
            errors.append(f"{code}: docs_link must be docs/errors/{code}.md")
        renderers = set(entry.get("renderer_support", []))
        if not renderers or not renderers.issubset(ALLOWED_RENDERERS):
            errors.append(f"{code}: invalid renderer_support {sorted(renderers)}")
        if not isinstance(entry.get("machine_applicable"), bool):
            errors.append(f"{code}: machine_applicable must be boolean")
        if "suggestion_applicability" not in entry:
            errors.append(f"{code}: missing suggestion_applicability")
    active_codes = set(active)
    catalog_codes = set(by_code)
    for code in sorted(active_codes - catalog_codes):
        errors.append(f"{code}: active diagnostic missing from code_catalog.json")
    for code in sorted(catalog_codes - active_codes):
        errors.append(f"{code}: catalog entry is not active")
    for code in sorted(active_codes & catalog_codes):
        entry = by_code[code]
        registry = active[code]
        if entry.get("constant") != registry["constant"]:
            errors.append(f"{code}: catalog constant does not match registry")
        if entry.get("severity") != registry["severity"]:
            errors.append(f"{code}: catalog severity does not match registry")
    return by_code


def validate_coverage(
    errors: list[str],
    active: dict[str, dict[str, str]],
    catalog: dict[str, dict[str, Any]],
    cases: dict[str, dict[str, Any]],
) -> None:
    payload = load_json(COVERAGE_PATH)
    entries = payload.get("coverage")
    if payload.get("schema_version") != 1 or not isinstance(entries, list):
        errors.append("code_baseline_coverage.json must have schema_version=1 and a coverage array")
        return
    by_code: dict[str, dict[str, Any]] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            errors.append("code_baseline_coverage.json entries must be objects")
            continue
        code = str(entry.get("code", ""))
        if code in by_code:
            errors.append(f"{code}: duplicate coverage entry")
        by_code[code] = entry
        fixture_id = entry.get("baseline_fixture_id")
        renderers = set(entry.get("renderer_formats", []))
        deferral = entry.get("deferral")
        if fixture_id is None:
            if renderers:
                errors.append(f"{code}: renderer_formats require baseline_fixture_id")
            if not isinstance(deferral, dict):
                errors.append(f"{code}: missing rendered baseline requires documented deferral")
            else:
                for field in ("owner", "reason", "issue", "expires_in_wave"):
                    if not deferral.get(field):
                        errors.append(f"{code}: deferral missing {field}")
        else:
            fixture_key = str(fixture_id)
            case = cases.get(fixture_key)
            if case is None:
                errors.append(f"{code}: unknown baseline fixture {fixture_key}")
            if not renderers:
                errors.append(f"{code}: baseline fixture requires renderer_formats")
            if not renderers.issubset(ALLOWED_RENDERERS):
                errors.append(f"{code}: invalid renderer_formats {sorted(renderers)}")
            if case is not None and not renderers.issubset(case["formats"]):
                errors.append(f"{code}: renderer_formats are not in manifest case formats")
            if code in catalog and not renderers.issubset(set(catalog[code].get("renderer_support", []))):
                errors.append(f"{code}: renderer_formats exceed catalog renderer_support")
            if case is not None:
                validate_coverage_baseline_evidence(errors, code, case, renderers)
    active_codes = set(active)
    coverage_codes = set(by_code)
    for code in sorted(active_codes - coverage_codes):
        errors.append(f"{code}: active diagnostic missing from code_baseline_coverage.json")
    for code in sorted(coverage_codes - active_codes):
        errors.append(f"{code}: coverage entry is not active")


def validate_coverage_baseline_evidence(
    errors: list[str],
    code: str,
    case: dict[str, Any],
    renderers: set[str],
) -> None:
    baseline_dir = case["entry"].parent / "baselines"
    for renderer in sorted(renderers):
        baseline = baseline_dir / f"{case['command']}-{renderer}.stderr.txt"
        if not baseline.is_file():
            errors.append(f"{code}: coverage baseline evidence is missing: {repo_relative(baseline)}")
            continue
        if code not in baseline.read_text(encoding="utf-8"):
            errors.append(
                f"{code}: coverage references {case['entry'].parent.name}/{renderer}, "
                f"but {repo_relative(baseline)} does not render that code"
            )


def validate_baseline_metadata(
    errors: list[str],
    cases: dict[str, dict[str, Any]],
    covered_pairs: set[tuple[str, str]],
) -> None:
    payload = load_json(METADATA_PATH)
    entries = payload.get("baselines")
    if payload.get("schema_version") != 1 or not isinstance(entries, list):
        errors.append("baseline_metadata.json must have schema_version=1 and a baselines array")
        return
    metadata: dict[tuple[str, str], dict[str, Any]] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            errors.append("baseline metadata entries must be objects")
            continue
        key = (str(entry.get("fixture_id", "")), str(entry.get("renderer", "")))
        if key in metadata:
            errors.append(f"{key[0]}/{key[1]}: duplicate baseline metadata")
        metadata[key] = entry

    expected_files = expected_baseline_files(cases)
    synthetic_files: set[pathlib.Path] = set()
    for (fixture_id, renderer), entry in metadata.items():
        if not entry.get("synthetic"):
            continue
        baseline_dir = AREA_ROOT / "fixtures" / "diagnostics" / fixture_id / "baselines"
        label = f"check-{renderer}"
        synthetic_files.add(baseline_dir / f"{label}.stdout.txt")
        synthetic_files.add(baseline_dir / f"{label}.stderr.txt")
        synthetic_files.add(baseline_dir / f"{label}.exit-code.txt")
    allowed_files = expected_files | synthetic_files
    actual_files = actual_baseline_files()
    for path in sorted(allowed_files - actual_files):
        errors.append(f"{repo_relative(path)}: fixture is missing required baseline file")
    for path in sorted(actual_files - allowed_files):
        errors.append(f"{repo_relative(path)}: baseline file has no owning manifest fixture")

    stream_keys = baseline_file_keys(actual_files)
    for key, streams in sorted(stream_keys.items()):
        if streams != {"stdout", "stderr", "exit-code"}:
            errors.append(f"{key[0]}/{key[1]}: baseline trio is incomplete: {sorted(streams)}")

    for key, entry in sorted(metadata.items()):
        case = cases.get(key[0])
        if case is None and not entry.get("synthetic"):
            errors.append(f"{key[0]}/{key[1]}: metadata fixture is not in manifest")
            continue
        if case is not None and entry.get("suite") != case["suite"]:
            errors.append(f"{key[0]}/{key[1]}: metadata suite does not match manifest")
        if case is not None and key[1] not in case["formats"]:
            errors.append(f"{key[0]}/{key[1]}: metadata renderer is not in manifest formats")
        source_path = (
            case["entry"]
            if case is not None
            else AREA_ROOT / "fixtures" / "diagnostics" / key[0] / "main.sifr"
        )
        if not source_path.is_file():
            errors.append(f"{key[0]}/{key[1]}: metadata source fixture does not exist")
            continue
        if entry.get("source_hash") != sha256_file(source_path):
            errors.append(f"{key[0]}/{key[1]}: metadata source_hash is stale")
        if not entry.get("owner"):
            errors.append(f"{key[0]}/{key[1]}: metadata missing owner")
        if not entry.get("bless_reference"):
            errors.append(f"{key[0]}/{key[1]}: metadata missing bless_reference")
        if not entry.get("bless_reason"):
            errors.append(f"{key[0]}/{key[1]}: metadata missing bless_reason")
        normalizers = set(entry.get("normalizers", []))
        if not normalizers or not normalizers.issubset(NORMALIZERS):
            errors.append(f"{key[0]}/{key[1]}: metadata has invalid normalizers")

    actual_pairs = set(stream_keys)
    for key in sorted(actual_pairs - set(metadata)):
        errors.append(f"{key[0]}/{key[1]}: baseline missing metadata")
    for key in sorted(set(metadata) - actual_pairs):
        errors.append(f"{key[0]}/{key[1]}: metadata has no baseline files")
    for key in sorted(covered_pairs - actual_pairs):
        errors.append(f"{key[0]}/{key[1]}: code coverage references missing baseline")


def validate_recovery_surfaces(errors: list[str]) -> None:
    payload = load_json(RECOVERY_PATH)
    surfaces = payload.get("surfaces")
    if payload.get("schema_version") != 1 or not isinstance(surfaces, list):
        errors.append("recovery_surface_coverage.json must have schema_version=1 and a surfaces array")
        return
    seen: set[str] = set()
    for surface in surfaces:
        if not isinstance(surface, dict):
            errors.append("recovery surface entries must be objects")
            continue
        surface_id = str(surface.get("id", ""))
        if surface_id in seen:
            errors.append(f"{surface_id}: duplicate recovery surface")
        seen.add(surface_id)
        fixtures = surface.get("multi_error_fixtures")
        if not isinstance(fixtures, list) or not fixtures:
            errors.append(f"{surface_id}: recovery surface has zero multi-error fixtures")
            continue
        for fixture in fixtures:
            fixture_path = ROOT / str(fixture)
            if not fixture_path.is_file():
                errors.append(f"{surface_id}: recovery fixture does not exist: {fixture}")
                continue
            expected_codes = surface.get("expected_codes", [])
            if isinstance(expected_codes, list) and len(expected_codes) >= 2:
                verify_recovery_codes(errors, surface_id, fixture_path, expected_codes)
                continue
            expect_error_count = fixture_path.read_text(encoding="utf-8").count("expect-error")
            if expect_error_count < 2:
                errors.append(f"{surface_id}: recovery fixture is not multi-error: {fixture}")


def verify_recovery_codes(
    errors: list[str],
    surface_id: str,
    fixture_path: pathlib.Path,
    expected_codes: list[Any],
) -> None:
    codes = [str(code) for code in expected_codes]
    if fixture_path.is_relative_to(AREA_ROOT / "fixtures" / "diagnostics"):
        baseline = fixture_path.parent / "baselines" / "check-compact.stderr.txt"
        if not baseline.is_file():
            errors.append(f"{surface_id}: recovery baseline is missing: {repo_relative(baseline)}")
            return
        evidence = baseline.read_text(encoding="utf-8")
    else:
        evidence = fixture_path.read_text(encoding="utf-8")

    for code in set(codes):
        expected_count = codes.count(code)
        actual_count = evidence.count(code)
        if actual_count < expected_count:
            errors.append(
                f"{surface_id}: expected {expected_count} occurrence(s) of {code}, "
                f"found {actual_count} in recovery evidence for {repo_relative(fixture_path)}"
            )


def covered_pairs() -> set[tuple[str, str]]:
    payload = load_json(COVERAGE_PATH)
    pairs: set[tuple[str, str]] = set()
    for entry in payload.get("coverage", []):
        fixture_id = entry.get("baseline_fixture_id")
        if fixture_id is None:
            continue
        for renderer in entry.get("renderer_formats", []):
            pairs.add((str(fixture_id), str(renderer)))
    return pairs


def main() -> int:
    errors: list[str] = []
    active = active_registry()
    cases = manifest_baseline_cases()
    catalog = validate_catalog(errors, active)
    validate_coverage(errors, active, catalog, cases)
    validate_baseline_metadata(errors, cases, covered_pairs())
    validate_recovery_surfaces(errors)
    if errors:
        for error in errors:
            print(f"diagnostic baseline coverage: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

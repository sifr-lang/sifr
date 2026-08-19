"""Structured Rust-test provenance validation for Rust interop evidence."""

from __future__ import annotations

import copy
import json
import re
import tempfile
import tomllib
from pathlib import Path
from typing import Any

from _rust_test_evidence import (
    RustTestDefinition,
    clear_module_declaration_cache,
    external_module_features,
    rust_test_definitions,
    rust_test_path,
)
from _rust_test_outcomes import run_self_test as run_outcome_self_test
from _rust_test_outcomes import validate_bound_test_outcome

PROFILE_ORDER = ("create-pr", "merge", "nightly", "release")
VALIDATION_FIELDS = {"profile", "step", "suite_id", "test_file", "test_name"}


def load_profiles(repo_root: Path) -> dict[str, dict[str, Any]]:
    """Load the four ordered validation profiles."""
    profiles: dict[str, dict[str, Any]] = {}
    for profile_name in PROFILE_ORDER:
        path = repo_root / "verification" / "profiles" / f"{profile_name}.json"
        profiles[profile_name] = json.loads(path.read_text(encoding="utf-8"))
    return profiles


def load_fixture_manifests(
    fixtures_root: Path,
    failures: list[str],
) -> dict[str, dict[str, Any]]:
    """Load fixture manifests by directory id without consulting README prose."""
    manifests: dict[str, dict[str, Any]] = {}
    for fixture_dir in sorted(path for path in fixtures_root.iterdir() if path.is_dir()):
        path = fixture_dir / "fixture.json"
        if not path.is_file():
            continue
        try:
            manifest = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            failures.append(f"{fixture_dir.name}: fixture.json is not valid JSON: {error}")
            continue
        if not isinstance(manifest, dict):
            failures.append(f"{fixture_dir.name}: fixture.json must be an object")
            continue
        manifests[fixture_dir.name] = manifest
    return manifests


def validate_evidence_provenance(
    failures: list[str],
    *,
    repo_root: Path,
    profiles: dict[str, dict[str, Any]],
    fixture_id: str,
    side: str,
    evidence: Any,
    execution_kind: str | None,
    used_tests: dict[tuple[str, str], str],
) -> None:
    """Validate one evidence record and reserve its test for exactly one side."""
    label = f"{fixture_id}: evidence.{side}"
    if not isinstance(evidence, dict):
        return
    status = evidence.get("status")
    validation = evidence.get("validation")
    if status != "passing":
        if validation is not None:
            failures.append(f"{label}.validation is allowed only for passing evidence")
        return
    if not isinstance(validation, dict):
        failures.append(f"{label}.validation is required for passing evidence")
        return
    if set(validation) != VALIDATION_FIELDS:
        failures.append(
            f"{label}.validation must contain exactly "
            "profile, step, suite_id, test_file, and test_name"
        )
        return

    profile_name = validation.get("profile")
    if profile_name not in PROFILE_ORDER:
        failures.append(f"{label}.validation.profile is invalid")
        return
    if validation.get("step") != "crate_tests":
        failures.append(f"{label}.validation.step must be crate_tests")

    suite_id = validation.get("suite_id")
    if not isinstance(suite_id, str) or not suite_id:
        failures.append(f"{label}.validation.suite_id is required")
        return
    suite = _suite_by_id(profiles.get(str(profile_name)), suite_id)
    if suite is None:
        failures.append(
            f"{label}.validation suite {suite_id!r} is missing from profile {profile_name}"
        )
        return
    _validate_suite_selection(failures, label, profiles[str(profile_name)], suite)

    test_file = validation.get("test_file")
    test_name = validation.get("test_name")
    if not isinstance(test_file, str) or not test_file:
        failures.append(f"{label}.validation.test_file is required")
        return
    if not isinstance(test_name, str) or not test_name:
        failures.append(f"{label}.validation.test_name is required")
        return
    source_path = _resolve_test_path(failures, label, repo_root, test_file, suite)
    if source_path is None:
        return
    test_definition = _validate_test_definition(failures, label, source_path, test_name)
    if test_definition is None:
        return
    validate_bound_test_outcome(
        failures,
        repo_root=repo_root,
        label=label,
        source_path=source_path,
        test_name=test_name,
        evidence=evidence,
    )
    test_ignored = test_definition.ignored
    required_features = test_definition.required_features | external_module_features(
        source_path,
        repo_root / "crates" / str(suite.get("package")),
    )
    command = suite.get("command")
    command_uses_ignored = isinstance(command, list) and "--ignored" in command
    if test_ignored != command_uses_ignored:
        expectation = "include" if test_ignored else "not include"
        failures.append(
            f"{label}.validation suite command must {expectation} --ignored "
            f"for test {test_name}"
        )
    _validate_feature_selection(
        failures,
        label,
        repo_root,
        suite,
        required_features,
    )
    full_test_path = rust_test_path(
        source_path,
        repo_root / "crates" / str(suite.get("package")),
        test_definition,
        test_name,
    )
    _validate_command_filters(failures, label, suite, test_name, full_test_path)
    _validate_execution_kind_source(
        failures,
        label,
        execution_kind,
        side,
        test_file,
        test_name,
        suite_id,
        test_definition.executes_cargo_probe,
        test_definition.executes_runtime_observed,
        evidence.get("expected_result"),
    )

    weakest = _weakest_executing_profile(profiles, suite_id, test_ignored)
    if weakest is None:
        failures.append(f"{label}.validation test has no blocking executing profile")
    elif profile_name != weakest:
        failures.append(
            f"{label}.validation.profile must be weakest mandatory profile "
            f"{weakest}, got {profile_name}"
        )

    normalized_test_file = source_path.resolve().relative_to(repo_root.resolve()).as_posix()
    key = (normalized_test_file, test_name)
    owner = used_tests.get(key)
    current_owner = f"{fixture_id}/{side}"
    if owner is not None and owner != current_owner:
        failures.append(
            f"{label}.validation test {test_file}::{test_name} is already used by {owner}"
        )
    else:
        used_tests[key] = current_owner


def _suite_by_id(profile: Any, suite_id: str) -> dict[str, Any] | None:
    if not isinstance(profile, dict):
        return None
    membership = profile.get("crate_test_membership")
    suites = membership.get("suites") if isinstance(membership, dict) else None
    if not isinstance(suites, list):
        return None
    matches = [
        suite
        for suite in suites
        if isinstance(suite, dict) and suite.get("id") == suite_id
    ]
    return matches[0] if len(matches) == 1 else None


def _selected_mode(profile: dict[str, Any]) -> Any:
    steps = profile.get("toolchain_steps")
    if not isinstance(steps, list):
        return None
    modes = (("cargo-test-sifr-smoke", "smoke"), ("cargo-test-sifr-full", "full"))
    return next((mode for step, mode in modes if step in steps), None)


def _validate_suite_selection(
    failures: list[str],
    label: str,
    profile: dict[str, Any],
    suite: dict[str, Any],
) -> None:
    suite_id = suite.get("id")
    if suite.get("status") != "blocking":
        failures.append(f"{label}.validation suite {suite_id} must be blocking")
    if not _suite_selected(profile, suite):
        failures.append(
            f"{label}.validation suite {suite_id} is not enabled by "
            f"profile mode {_selected_mode(profile)!r}"
        )


def _suite_selected(profile: dict[str, Any], suite: dict[str, Any]) -> bool:
    modes = suite.get("modes")
    if not isinstance(modes, list):
        return False
    return _selected_mode(profile) in modes


def _resolve_test_path(
    failures: list[str],
    label: str,
    repo_root: Path,
    raw_path: str,
    suite: dict[str, Any],
) -> Path | None:
    relative_path = Path(raw_path)
    if relative_path.is_absolute() or ".." in relative_path.parts:
        failures.append(f"{label}.validation.test_file must stay inside the repository")
        return None
    if relative_path.suffix != ".rs":
        failures.append(f"{label}.validation.test_file must point to a .rs file")
        return None
    package = suite.get("package")
    expected_root = Path("crates") / str(package)
    try:
        relative_path.relative_to(expected_root)
    except ValueError:
        failures.append(
            f"{label}.validation.test_file must belong to suite package {package}"
        )
        return None
    package_root = (repo_root / expected_root).resolve()
    source_path = (repo_root / relative_path).resolve()
    try:
        source_path.relative_to(repo_root.resolve())
    except ValueError:
        failures.append(f"{label}.validation.test_file must stay inside the repository")
        return None
    try:
        source_path.relative_to(package_root)
    except ValueError:
        failures.append(
            f"{label}.validation.test_file must stay inside suite package {package}"
        )
        return None
    if not source_path.is_file():
        failures.append(f"{label}.validation.test_file does not exist: {raw_path}")
        return None
    return source_path


def _validate_test_definition(
    failures: list[str],
    label: str,
    source_path: Path,
    test_name: str,
) -> RustTestDefinition | None:
    matches = rust_test_definitions(source_path, test_name)
    if len(matches) != 1:
        failures.append(
            f"{label}.validation.test_name {test_name!r} must occur exactly once "
            f"as a Rust test in {source_path.name}, found {len(matches)}"
        )
        return None
    return matches[0]


def _validate_feature_selection(
    failures: list[str],
    label: str,
    repo_root: Path,
    suite: dict[str, Any],
    required_features: frozenset[str],
) -> None:
    if not required_features:
        return
    package = suite.get("package")
    manifest_path = repo_root / "crates" / str(package) / "Cargo.toml"
    if not manifest_path.is_file():
        failures.append(f"{label}.validation cannot resolve Cargo features for {package}")
        return
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    raw_features = manifest.get("features")
    feature_definitions = raw_features if isinstance(raw_features, dict) else {}
    command = suite.get("command")
    enabled = _enabled_cargo_features(
        command if isinstance(command, list) else [],
        feature_definitions,
    )
    missing = required_features - enabled
    if missing:
        failures.append(
            f"{label}.validation suite does not enable required test features: "
            f"{', '.join(sorted(missing))}"
        )


def _enabled_cargo_features(
    command: list[str],
    definitions: dict[str, Any],
) -> set[str]:
    if "--all-features" in command:
        return set(definitions)
    roots: set[str] = set()
    if "--no-default-features" not in command:
        roots.add("default")
    for index, argument in enumerate(command):
        raw_value: str | None = None
        if argument in {"--features", "-F"} and index + 1 < len(command):
            raw_value = command[index + 1]
        elif argument.startswith("--features="):
            raw_value = argument.removeprefix("--features=")
        if raw_value is not None:
            roots.update(item for item in re.split(r"[\s,]+", raw_value) if item)

    enabled: set[str] = set()
    pending = list(roots)
    while pending:
        feature = pending.pop()
        if feature in enabled:
            continue
        enabled.add(feature)
        members = definitions.get(feature)
        if not isinstance(members, list):
            continue
        for member in members:
            if not isinstance(member, str):
                continue
            dependency_feature = member.split("/", 1)[0].removeprefix("dep:")
            if dependency_feature in definitions and dependency_feature not in enabled:
                pending.append(dependency_feature)
    return enabled


def _validate_command_filters(
    failures: list[str],
    label: str,
    suite: dict[str, Any],
    test_name: str,
    full_test_path: str,
) -> None:
    command = suite.get("command")
    if not isinstance(command, list) or "--" not in command:
        return
    test_arguments = command[command.index("--") + 1 :]
    for index, argument in enumerate(test_arguments):
        skip_filter: str | None = None
        if argument == "--skip" and index + 1 < len(test_arguments):
            skip_filter = test_arguments[index + 1]
        elif argument.startswith("--skip="):
            skip_filter = argument.removeprefix("--skip=")
        if skip_filter and (
            skip_filter in test_name or skip_filter in full_test_path
        ):
            failures.append(
                f"{label}.validation suite --skip filter {skip_filter!r} excludes "
                f"test {test_name}"
            )


def _validate_execution_kind_source(
    failures: list[str],
    label: str,
    execution_kind: str | None,
    side: str,
    test_file: str,
    test_name: str,
    suite_id: str,
    executes_cargo_probe: bool,
    executes_runtime_observed: bool,
    expected_result: Any,
) -> None:
    is_runtime_test = test_file.startswith("crates/sifr_runtime/")
    if execution_kind == "runtime-observed":
        is_negative_diagnostic = side == "negative" and expected_result == "diagnostic"
        if not is_runtime_test and not executes_runtime_observed and not is_negative_diagnostic:
            failures.append(
                f"{label}.validation runtime-observed evidence must execute a "
                "runtime test or carry the generated-runtime execution marker"
            )
        return
    if is_runtime_test:
        failures.append(
            f"{label}.validation {execution_kind} evidence cannot use a runtime test"
        )
        return
    if execution_kind == "cargo-probe" and side == "positive":
        is_generated_probe = suite_id in {
            "sifr_driver_generated_builds",
            "sifr_cli_generated_builds",
        }
        if not is_generated_probe and not executes_cargo_probe:
            failures.append(
                f"{label}.validation positive cargo-probe evidence must use an "
                "explicit probe test"
            )


def _weakest_executing_profile(
    profiles: dict[str, dict[str, Any]],
    suite_id: str,
    test_ignored: bool,
) -> str | None:
    for profile_name in PROFILE_ORDER:
        profile = profiles.get(profile_name)
        suite = _suite_by_id(profile, suite_id)
        if suite is None or suite.get("status") != "blocking":
            continue
        if not _suite_selected(profile, suite):
            continue
        command = suite.get("command")
        if not isinstance(command, list):
            continue
        if ("--ignored" in command) != test_ignored:
            continue
        return profile_name
    return None


def run_self_test() -> tuple[int, str | None]:
    """Mutation-test every provenance failure mode against temporary data."""
    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-provenance-") as raw_root:
        repo_root = Path(raw_root)
        test_file = repo_root / "crates" / "sifr_driver" / "src" / "evidence.rs"
        test_file.parent.mkdir(parents=True)
        test_file.write_text(
            "#[test]\nfn positive_test() {}\n"
            "#[test]\nfn negative_test() {}\n"
            "#[test]\n#[ignore]\nfn ignored_test() {}\n"
            "#[cfg(feature = \"special\")]\n#[test]\nfn feature_test() {}\n"
            "#[cfg(all(\n    feature = \"special\",\n    unix,\n))]\n"
            "#[test]\nfn multiline_feature_test() {}\n"
            "#[doc = \"sifr-evidence: executes-cargo-probe\"]\n"
            "#[test]\nfn cargo_probe_test() {}\n"
            "#[doc = \"sifr-evidence: executes-runtime-observed\"]\n"
            "#[test]\nfn generated_runtime_test() {}\n",
            encoding="utf-8",
        )
        (test_file.parents[1] / "Cargo.toml").write_text(
            "[package]\nname = \"sifr_driver\"\nversion = \"0.0.0\"\n"
            "[features]\ndefault = []\nspecial = []\nouter = []\n",
            encoding="utf-8",
        )
        profiles = _self_test_profiles()
        base = {
            "status": "passing",
            "validation": {
                "profile": "create-pr",
                "step": "crate_tests",
                "suite_id": "driver",
                "test_file": "crates/sifr_driver/src/evidence.rs",
                "test_name": "positive_test",
            },
        }
        control: list[str] = []
        validate_evidence_provenance(
            control,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="control",
            side="positive",
            evidence=base,
            execution_kind="contract-only",
            used_tests={},
        )
        if control:
            return 0, f"valid provenance was rejected: {control}"

        cases: list[tuple[str, dict[str, Any], dict[str, dict[str, Any]], str]] = []
        missing_suite = copy.deepcopy(base)
        missing_suite["validation"]["suite_id"] = "missing"
        cases.append(("missing suite", missing_suite, profiles, "is missing from profile"))
        wrong_mode_profiles = copy.deepcopy(profiles)
        wrong_mode_profiles["create-pr"]["toolchain_steps"] = ["cargo-test-sifr-full"]
        wrong_mode_profiles["create-pr"]["crate_test_membership"]["suites"][0]["modes"] = ["smoke"]
        cases.append(("wrong profile mode", base, wrong_mode_profiles, "is not enabled"))
        nonblocking_profiles = copy.deepcopy(profiles)
        nonblocking_profiles["create-pr"]["crate_test_membership"]["suites"][0]["status"] = "advisory"
        cases.append(("nonblocking suite", base, nonblocking_profiles, "must be blocking"))
        wrong_weakest = copy.deepcopy(base)
        wrong_weakest["validation"]["profile"] = "merge"
        cases.append(
            (
                "wrong weakest profile",
                wrong_weakest,
                profiles,
                "must be weakest mandatory profile create-pr",
            )
        )
        wrong_package = copy.deepcopy(base)
        wrong_package["validation"]["test_file"] = (
            "crates/sifr_lowering/src/evidence.rs"
        )
        cases.append(
            (
                "wrong package ownership",
                wrong_package,
                profiles,
                "must belong to suite package sifr_driver",
            )
        )
        wrong_step = copy.deepcopy(base)
        wrong_step["validation"]["step"] = "not_crate_tests"
        cases.append(("wrong step", wrong_step, profiles, "step must be crate_tests"))
        extra_field = copy.deepcopy(base)
        extra_field["validation"]["command"] = "cargo test"
        cases.append(
            (
                "extra validation field",
                extra_field,
                profiles,
                "must contain exactly",
            )
        )
        missing_test = copy.deepcopy(base)
        missing_test["validation"]["test_name"] = "absent_test"
        cases.append(("missing test", missing_test, profiles, "found 0"))
        duplicate_file = repo_root / "crates" / "sifr_driver" / "src" / "duplicate.rs"
        duplicate_file.write_text(
            "#[test]\nfn duplicate_test() {}\n#[test]\nfn duplicate_test() {}\n",
            encoding="utf-8",
        )
        duplicate_test = copy.deepcopy(base)
        duplicate_test["validation"]["test_file"] = "crates/sifr_driver/src/duplicate.rs"
        duplicate_test["validation"]["test_name"] = "duplicate_test"
        cases.append(("duplicate test", duplicate_test, profiles, "found 2"))
        ignored_mismatch = copy.deepcopy(base)
        ignored_mismatch["validation"]["test_name"] = "ignored_test"
        cases.append(("ignored mismatch", ignored_mismatch, profiles, "must include --ignored"))
        feature_mismatch = copy.deepcopy(base)
        feature_mismatch["validation"]["test_name"] = "feature_test"
        cases.append(
            (
                "feature mismatch",
                feature_mismatch,
                profiles,
                "does not enable required test features: special",
            )
        )
        multiline_feature_mismatch = copy.deepcopy(base)
        multiline_feature_mismatch["validation"]["test_name"] = (
            "multiline_feature_test"
        )
        cases.append(
            (
                "multiline feature mismatch",
                multiline_feature_mismatch,
                profiles,
                "does not enable required test features: special",
            )
        )
        file_gated = test_file.with_name("file_gated.rs")
        file_gated.write_text(
            "#![cfg(feature = \"special\")]\n"
            "#[test]\nfn file_feature_test() {}\n",
            encoding="utf-8",
        )
        file_feature_mismatch = copy.deepcopy(base)
        file_feature_mismatch["validation"]["test_file"] = (
            "crates/sifr_driver/src/file_gated.rs"
        )
        file_feature_mismatch["validation"]["test_name"] = "file_feature_test"
        cases.append(
            (
                "file feature mismatch",
                file_feature_mismatch,
                profiles,
                "does not enable required test features: special",
            )
        )
        gated_file = test_file.with_name("gated.rs")
        gated_file.write_text("#[test]\nfn gated_test() {}\n", encoding="utf-8")
        test_file.with_name("lib.rs").write_text(
            "#[cfg(feature = \"outer\")]\nmod gated;\n",
            encoding="utf-8",
        )
        module_feature_mismatch = copy.deepcopy(base)
        module_feature_mismatch["validation"]["test_file"] = (
            "crates/sifr_driver/src/gated.rs"
        )
        module_feature_mismatch["validation"]["test_name"] = "gated_test"
        cases.append(
            (
                "module feature mismatch",
                module_feature_mismatch,
                profiles,
                "does not enable required test features: outer",
            )
        )
        skipped_profiles = copy.deepcopy(profiles)
        skipped_profiles["create-pr"]["crate_test_membership"]["suites"][0][
            "command"
        ] += ["--", "--skip", "positive_test"]
        cases.append(
            (
                "skip filter",
                base,
                skipped_profiles,
                "--skip filter 'positive_test' excludes test",
            )
        )
        module_skipped_profiles = copy.deepcopy(profiles)
        module_skipped_profiles["create-pr"]["crate_test_membership"]["suites"][0][
            "command"
        ] += ["--", "--skip", "evidence"]
        cases.append(
            (
                "module skip filter",
                base,
                module_skipped_profiles,
                "--skip filter 'evidence' excludes test",
            )
        )
        remapped_file = test_file.with_name("remapped_test.rs")
        remapped_file.write_text(
            "#[test]\nfn remapped_test() {}\n",
            encoding="utf-8",
        )
        test_file.with_name("path_parent.rs").write_text(
            '#[cfg(feature = "outer")]\n'
            '#[path = "remapped_test.rs"]\n'
            "mod evidence_alias;\n",
            encoding="utf-8",
        )
        clear_module_declaration_cache()
        remapped_skip = copy.deepcopy(base)
        remapped_skip["validation"]["test_file"] = (
            "crates/sifr_driver/src/remapped_test.rs"
        )
        remapped_skip["validation"]["test_name"] = "remapped_test"
        remapped_feature_mismatch = copy.deepcopy(remapped_skip)
        cases.append(
            (
                "path-remapped module feature mismatch",
                remapped_feature_mismatch,
                profiles,
                "does not enable required test features: outer",
            )
        )
        remapped_skipped_profiles = copy.deepcopy(profiles)
        remapped_skipped_profiles["create-pr"]["crate_test_membership"]["suites"][0][
            "command"
        ] += ["--", "--skip", "path_parent::evidence_alias"]
        cases.append(
            (
                "path-remapped module skip filter",
                remapped_skip,
                remapped_skipped_profiles,
                "--skip filter 'path_parent::evidence_alias' excludes test",
            )
        )
        escaped = copy.deepcopy(base)
        escaped["validation"]["test_file"] = "../evidence.rs"
        cases.append(("path escape", escaped, profiles, "stay inside the repository"))
        nonpassing = copy.deepcopy(base)
        nonpassing["status"] = "planned"
        cases.append(("status/provenance mismatch", nonpassing, profiles, "allowed only for passing"))
        (repo_root / "README.md").write_text(
            "Passing evidence: `positive_test`.\n",
            encoding="utf-8",
        )
        readme_only = {"status": "passing"}
        cases.append(("README-only passing claim", readme_only, profiles, "validation is required"))

        commented_file = repo_root / "crates" / "sifr_driver" / "src" / "commented.rs"
        commented_file.write_text(
            "// #[test]\n// fn commented_test() {}\n"
            "/* nested /* test */ comment\n#[test]\nfn commented_test() {}\n*/\n"
            "const EXAMPLE: &str = r#\"\n#[test]\nfn commented_test() {}\n\"#;\n",
            encoding="utf-8",
        )
        commented_test = copy.deepcopy(base)
        commented_test["validation"]["test_file"] = "crates/sifr_driver/src/commented.rs"
        commented_test["validation"]["test_name"] = "commented_test"
        cases.append(("commented pseudo-test", commented_test, profiles, "found 0"))

        for name, evidence, case_profiles, expected in cases:
            failures: list[str] = []
            validate_evidence_provenance(
                failures,
                repo_root=repo_root,
                profiles=case_profiles,
                fixture_id=name,
                side="positive",
                evidence=evidence,
                execution_kind="contract-only",
                used_tests={},
            )
            if not any(expected in failure for failure in failures):
                return len(cases), f"{name} did not report {expected!r}: {failures}"

        shared_failures: list[str] = []
        used: dict[tuple[str, str], str] = {}
        for side in ("positive", "negative"):
            validate_evidence_provenance(
                shared_failures,
                repo_root=repo_root,
                profiles=profiles,
                fixture_id=f"shared_{side}",
                side=side,
                evidence=base,
                execution_kind="contract-only",
                used_tests=used,
            )
        if not any("is already used by" in failure for failure in shared_failures):
            return len(cases) + 1, "shared evidence test was accepted"

        strength_failures: list[str] = []
        validate_evidence_provenance(
            strength_failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="runtime_source",
            side="positive",
            evidence=base,
            execution_kind="runtime-observed",
            used_tests={},
        )
        if not any(
            "runtime-observed evidence must execute" in failure
            for failure in strength_failures
        ):
            return len(cases) + 2, "weak runtime-observed source was accepted"

        generated_runtime = copy.deepcopy(base)
        generated_runtime["validation"]["test_name"] = "generated_runtime_test"
        generated_runtime_failures: list[str] = []
        validate_evidence_provenance(
            generated_runtime_failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="generated_runtime_source",
            side="positive",
            evidence=generated_runtime,
            execution_kind="runtime-observed",
            used_tests={},
        )
        if generated_runtime_failures:
            return (
                len(cases) + 3,
                "generated runtime marker was rejected: "
                f"{generated_runtime_failures}",
            )

        nonruntime_kind_failures: list[str] = []
        _validate_execution_kind_source(
            nonruntime_kind_failures,
            "contract-only-source",
            "contract-only",
            "negative",
            "crates/sifr_runtime/src/interop.rs",
            "runtime_test",
            "sifr_runtime",
            False,
            False,
            "runtime-error-state",
        )
        if not any(
            "contract-only evidence cannot use a runtime test" in failure
            for failure in nonruntime_kind_failures
        ):
            return len(cases) + 4, "runtime test was accepted as contract-only evidence"

        cargo_negative_runtime_failures: list[str] = []
        _validate_execution_kind_source(
            cargo_negative_runtime_failures,
            "cargo-negative-source",
            "cargo-probe",
            "negative",
            "crates/sifr_runtime/src/interop.rs",
            "runtime_test",
            "sifr_runtime",
            False,
            False,
            "diagnostic",
        )
        if not any(
            "cargo-probe evidence cannot use a runtime test" in failure
            for failure in cargo_negative_runtime_failures
        ):
            return len(cases) + 5, "runtime test was accepted as negative cargo evidence"

        cargo_strength_failures: list[str] = []
        validate_evidence_provenance(
            cargo_strength_failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="cargo_source",
            side="positive",
            evidence=base,
            execution_kind="cargo-probe",
            used_tests={},
        )
        if not any("explicit probe test" in failure for failure in cargo_strength_failures):
            return len(cases) + 6, "weak positive cargo-probe source was accepted"

        cargo_probe_control = copy.deepcopy(base)
        cargo_probe_control["validation"]["test_name"] = "cargo_probe_test"
        cargo_probe_control_failures: list[str] = []
        validate_evidence_provenance(
            cargo_probe_control_failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="cargo_probe_control",
            side="positive",
            evidence=cargo_probe_control,
            execution_kind="cargo-probe",
            used_tests={},
        )
        if cargo_probe_control_failures:
            return (
                len(cases) + 7,
                f"explicit cargo-probe marker was rejected: {cargo_probe_control_failures}",
            )

        ignored_binding = copy.deepcopy(base)
        ignored_binding["validation"]["profile"] = "merge"
        ignored_binding["validation"]["suite_id"] = "driver_ignored"
        ignored_binding["validation"]["test_name"] = "ignored_test"
        ignored_failures: list[str] = []
        validate_evidence_provenance(
            ignored_failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id="ignored_control",
            side="positive",
            evidence=ignored_binding,
            execution_kind="contract-only",
            used_tests={},
        )
        if ignored_failures:
            return len(cases) + 8, f"valid ignored provenance was rejected: {ignored_failures}"
        outcome_cases, outcome_error = run_outcome_self_test()
        if outcome_error is not None:
            return len(cases) + 9, outcome_error
        return len(cases) + 9 + outcome_cases, None


def _self_test_profiles() -> dict[str, dict[str, Any]]:
    profiles: dict[str, dict[str, Any]] = {}
    for profile_name in PROFILE_ORDER:
        mode = "smoke" if profile_name == "create-pr" else "full"
        profiles[profile_name] = {
            "toolchain_steps": [f"cargo-test-sifr-{mode}"],
            "crate_test_membership": {
                "suites": [
                    {
                        "id": "driver",
                        "package": "sifr_driver",
                        "command": ["test", "-p", "sifr_driver", "--lib"],
                        "modes": ["smoke", "full"],
                        "status": "blocking",
                    },
                    {
                        "id": "driver_ignored",
                        "package": "sifr_driver",
                        "command": [
                            "test",
                            "-p",
                            "sifr_driver",
                            "--lib",
                            "--",
                            "--ignored",
                        ],
                        "modes": ["full"],
                        "status": "blocking",
                    },
                ]
            },
        }
    return profiles

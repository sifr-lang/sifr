"""Self-tests for the verification runner foundation."""

from __future__ import annotations

import contextlib
import io
import json
import tempfile
from contextlib import redirect_stdout
from pathlib import Path
from typing import Callable

from .areas import discover_areas
from .cargo_setup import cargo_setup_command
from .profiles import (
    ProfileError,
    crate_test_suites_for_mode,
    failure_reproduction_command,
    legacy_facade,
    load_all_profiles,
    required_rust_interop_suites,
    selected_resource_classes,
    validate_crate_test_membership,
    validate_selected_area_suites,
)
from .profile_runner import (
    ProfileRunner,
    ProfileRunnerError,
    StepResult,
    legacy_facade_step_names,
    timed_step,
    validate_rust_interop_result,
)
from .profile_area_steps import run_selected_area
from .profile_results import AreaResultError, validate_area_result
from .release_evidence import (
    CRITICAL_RESULTS,
    build_release_profile_payload,
    prepare_release_report_output,
    validate_release_profile_report,
)
from .results import build_result
from .schemas import load_schema, validate_all_committed_schemas, validate_data, validate_schema_requirement
from verification.json_schema_202012 import lint_schema


def run_all() -> list[str]:
    checks = [
        ("schema self-tests", _schema_self_test),
        ("profile schema self-test", _profile_schema_self_test),
        ("crate membership self-test", _crate_membership_self_test),
        ("Rust interop profile execution self-test", _rust_interop_profile_self_test),
        ("documentation profile execution self-test", _documentation_profile_self_test),
        ("release report precondition self-test", _release_report_precondition_self_test),
        ("release report production self-test", _release_report_production_self_test),
        ("e2e profile self-test", _e2e_profile_self_test),
        ("runner discovery self-test", _discovery_self_test),
        ("resource class selection self-test", _resource_class_self_test),
        ("resume/failure-reproduction self-test", _failure_reproduction_self_test),
    ]
    passed: list[str] = []
    for name, check in checks:
        check()
        passed.append(name)
    return passed


def _schema_self_test() -> None:
    committed = validate_all_committed_schemas()
    required = {
        "verification/schemas/profile.schema.json",
        "verification/schemas/area.schema.json",
        "verification/schemas/suite.schema.json",
        "verification/schemas/case.schema.json",
        "verification/schemas/result.schema.json",
        "verification/schemas/owners.schema.json",
    }
    missing = required - set(committed)
    if missing:
        raise AssertionError(f"missing schema self-test coverage: {sorted(missing)}")
    governance_schemas = (
        Path(__file__).resolve().parents[3]
        / "verification"
        / "areas"
        / "distribution_release"
        / "schemas"
    )
    governed = [lint_schema(path) for path in sorted(governance_schemas.glob("*.schema.json"))]
    if len(governed) != 11:
        raise AssertionError("release-governance schema lint registration drifted")
    try:
        validate_schema_requirement({"type": "object", "oneOf": []}, Path("bad.schema.json"))
    except Exception as exc:
        if "unsupported schema keyword 'oneOf'" not in str(exc):
            raise
    else:
        raise AssertionError("unsupported schema keyword was accepted")

    area_schema = load_schema("area.schema.json")
    invalid_area = {
        "schema_version": 1,
        "name": "core_language",
        "description": "Missing owner and wrong parallel_safe type.",
        "parallel_safe": "yes",
        "resource_classes": ["default-local"],
        "timeout_seconds": 60,
        "suites": [],
    }
    try:
        validate_data(invalid_area, area_schema, source="invalid area self-test")
    except Exception as exc:
        if "missing required key 'owner'" not in str(exc):
            raise
    else:
        raise AssertionError("invalid area manifest data was accepted")


def _profile_schema_self_test() -> None:
    profiles = load_all_profiles()
    expected = {"create-pr", "merge", "nightly", "python-interop-live", "release"}
    if set(profiles) != expected:
        raise AssertionError(f"unexpected profiles: {sorted(profiles)}")
    if profiles["python-interop-live"].get("execution_mode") != "selected-areas-only":
        raise AssertionError("python-interop-live must use selected-areas-only execution")
    if cargo_setup_command(profiles["python-interop-live"]) != [
        "cargo",
        "fetch",
        "--locked",
    ]:
        raise AssertionError("python-interop-live has a noncanonical Cargo setup command")
    for profile_name in sorted(expected):
        setup_budget = profiles[profile_name].get("step_budgets", {}).get(
            "cargo_cache_setup"
        )
        if setup_budget != {
            "budget_ms": 300_000,
            "enforcement": "advisory",
        }:
            raise AssertionError(
                f"{profile_name} has a noncanonical Cargo setup budget: "
                f"{setup_budget}"
            )
    create_pr_step_budgets = profiles["create-pr"].get("step_budgets", {})
    required_blocking_steps = {
        "generated_code_quality_checks",
        "rust_interop_checks",
        "crate_tests",
        "runtime_platform_suites",
        "e2e_pass_suite",
    }
    missing_step_budgets = sorted(required_blocking_steps.difference(create_pr_step_budgets))
    if missing_step_budgets:
        raise AssertionError(f"create-pr step budgets missing: {missing_step_budgets}")
    for step in sorted(required_blocking_steps):
        budget = create_pr_step_budgets[step]
        if budget.get("enforcement") != "blocking" or int(budget.get("budget_ms", 0)) <= 0:
            raise AssertionError(f"create-pr step budget is not blocking/positive: {step}={budget}")


def _crate_membership_self_test() -> None:
    profiles = load_all_profiles()
    merge = profiles["merge"]
    full_suites = crate_test_suites_for_mode(merge, "full")
    by_id = {str(suite.get("id")): suite for suite in full_suites}
    expected_executed = {
        "sifr_type_system",
        "sifr_format",
        "sifr_lint",
        "sifr_source",
        "sifr_ir",
    }
    missing = sorted(expected_executed.difference(by_id))
    if missing:
        raise AssertionError(f"merge crate membership omitted expected suites: {missing}")
    for suite_id in sorted(expected_executed):
        suite = by_id[suite_id]
        if suite.get("status") != "blocking" or suite.get("executed_in_merge") is not True:
            raise AssertionError(f"merge crate suite is not blocking/executed: {suite_id}")

    for profile_name in ("create-pr", "merge", "nightly", "release"):
        profile = profiles[profile_name]
        if cargo_setup_command(profile) != ["cargo", "fetch", "--locked"]:
            raise AssertionError(f"{profile_name} has a noncanonical Cargo setup command")
        profile_full_suites = crate_test_suites_for_mode(profile, "full")
        profile_smoke_suites = crate_test_suites_for_mode(profile, "smoke")
        profile_by_id = {str(suite.get("id")): suite for suite in profile_full_suites}
        codegen = profile_by_id.get("sifr_codegen")
        if not isinstance(codegen, dict):
            raise AssertionError(f"sifr_codegen missing from {profile_name} crate membership")
        if codegen.get("status") != "blocking" or codegen.get("executed_in_merge") is not True:
            raise AssertionError(
                f"sifr_codegen is not blocking/executed in {profile_name}: {codegen}",
            )
        generated_build_suites = {
            "sifr_cli_generated_builds",
            "sifr_driver_generated_builds",
        }
        missing_generated = sorted(generated_build_suites.difference(profile_by_id))
        if missing_generated:
            raise AssertionError(
                f"generated-build crate suites missing from {profile_name}: {missing_generated}",
            )
        for suite_id in sorted(generated_build_suites):
            suite = profile_by_id[suite_id]
            if suite.get("status") != "blocking" or suite.get("executed_in_merge") is not True:
                raise AssertionError(
                    f"generated-build crate suite is not blocking/executed in {profile_name}: "
                    f"{suite}",
                )
        smoke_ids = {str(suite.get("id")) for suite in profile_smoke_suites}
        misplaced_generated = sorted(generated_build_suites.intersection(smoke_ids))
        if misplaced_generated:
            raise AssertionError(
                f"generated-build crate suites must not run in smoke for {profile_name}: "
                f"{misplaced_generated}",
            )

    duplicate_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "duplicate",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
                {
                    "id": "duplicate",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(duplicate_profile)
    except ProfileError as exc:
        if "duplicate crate test suite duplicate" not in str(exc):
            raise
    else:
        raise AssertionError("duplicate crate membership suite was accepted")

    unknown_crate_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "unknown",
                    "package": "sifr_does_not_exist",
                    "command": ["test", "-p", "sifr_does_not_exist"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(unknown_crate_profile)
    except ProfileError as exc:
        if "references unknown package sifr_does_not_exist" not in str(exc):
            raise
    else:
        raise AssertionError("unknown crate membership package was accepted")

    non_executed_full_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "not_executed",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": False,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(non_executed_full_profile)
    except ProfileError as exc:
        if "must execute in merge unless it is a red-blocker" not in str(exc):
            raise
    else:
        raise AssertionError("non-executed full-mode blocking suite was accepted")

    unknown_suite_profile = {
        "name": "self-test",
        "selected_areas": [{"area": "core_language", "suites": ["not_a_suite"]}],
    }
    try:
        validate_selected_area_suites(unknown_suite_profile)
    except ProfileError as exc:
        if "selects unknown suite core_language:not_a_suite" not in str(exc):
            raise
    else:
        raise AssertionError("unknown selected area suite was accepted")

    incomplete_certification_profile = {
        "name": "self-test",
        "selected_areas": [
            {
                "area": "python_interop",
                "suites": [
                    "async-declaration-examples",
                    "buffer-examples",
                    "callback-examples",
                    "dlpack-examples",
                ],
            }
        ],
    }
    try:
        validate_selected_area_suites(incomplete_certification_profile)
    except ProfileError as exc:
        if "omits required Python interop certification suites: arrow-examples" not in str(exc):
            raise
    else:
        raise AssertionError("incomplete Python interop certification profile was accepted")


def _rust_interop_profile_self_test() -> None:
    profiles = load_all_profiles()
    required_suites = required_rust_interop_suites()
    for profile_name in ("create-pr", "merge", "nightly", "release"):
        profile = profiles[profile_name]
        selected = {
            str(suite)
            for selection in profile["selected_areas"]
            if selection.get("area") == "rust_interop"
            for suite in selection.get("suites", [])
        }
        if selected != required_suites:
            raise AssertionError(
                f"{profile_name} Rust interop plan mismatch: "
                f"expected={sorted(required_suites)} actual={sorted(selected)}"
            )
        step_names = legacy_facade_step_names(profile)
        if "rust_interop_checks" not in step_names:
            raise AssertionError(f"{profile_name} omits the executable Rust interop step")

    class RecordingProfileRunner(ProfileRunner):
        def __init__(self) -> None:
            self.profile = {
                "execution_mode": "selected-areas-only",
                "cargo_policy": {"offline": True},
            }
            self.events: list[str] = []

        def print_header(self) -> None:
            self.events.append("header")

        def prepare_cargo_cache(self) -> None:
            self.events.append("cargo-cache-setup")

        def run_timed_step(self, name: str, callback: Callable[[], None]) -> StepResult:
            self.events.append(f"timed:{name}")
            callback()
            return StepResult(status=0, elapsed_ms=0)

        def enable_offline_cargo(self) -> None:
            self.events.append("offline")

        def run_selected_areas_only(self) -> int:
            self.events.append("selected-areas")
            return 0

    recording_runner = RecordingProfileRunner()
    if recording_runner.run() != 0:
        raise AssertionError("recording profile runner failed")
    expected_events = [
        "header",
        "timed:cargo_cache_setup",
        "cargo-cache-setup",
        "offline",
        "selected-areas",
    ]
    if recording_runner.events != expected_events:
        raise AssertionError(
            "profile runner did not execute cache setup before offline steps: "
            f"{recording_runner.events}"
        )
    timing_output = io.StringIO()
    with redirect_stdout(timing_output):
        timing_result = timed_step("cargo_cache_setup", lambda: None)
    if timing_result.status != 0 or (
        "[sifr-lane-step] name=cargo_cache_setup" not in timing_output.getvalue()
    ):
        raise AssertionError(
            "cache setup timing seam did not emit its lane-step report: "
            f"{timing_output.getvalue()!r}"
        )

    invalid_setup_profile = {
        "cargo_policy": {
            "locked": True,
            "offline": True,
            "setup_command": "cargo fetch",
        }
    }
    try:
        cargo_setup_command(invalid_setup_profile)
    except ValueError as exc:
        if "must be 'cargo fetch --locked'" not in str(exc):
            raise
    else:
        raise AssertionError("noncanonical Cargo setup command was accepted")

    incomplete_profile = {
        "name": "self-test",
        "selected_areas": [
            {
                "area": "rust_interop",
                "suites": ["matrix", "tiers", "compatibility-matrix"],
            }
        ],
    }
    try:
        validate_selected_area_suites(incomplete_profile)
    except ProfileError as exc:
        selected = {"matrix", "tiers", "compatibility-matrix"}
        expected_missing = ", ".join(sorted(required_suites - selected))
        expected_message = (
            "omits required Rust interop verification suites: "
            f"{expected_missing}"
        )
        if expected_message not in str(exc):
            raise
    else:
        raise AssertionError("incomplete Rust interop profile was accepted")

    missing_area_profile = {
        "name": "self-test",
        "selected_areas": [],
    }
    try:
        validate_selected_area_suites(missing_area_profile)
    except ProfileError as exc:
        if "omits the required Rust interop area" not in str(exc):
            raise
    else:
        raise AssertionError("legacy profile without the Rust interop area was accepted")

    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-result-self-test-") as temp_dir:
        result_path = Path(temp_dir) / "result.json"
        try:
            validate_rust_interop_result(result_path, sorted(required_suites))
        except ProfileRunnerError as exc:
            if "emitted no result JSON" not in str(exc):
                raise
        else:
            raise AssertionError("missing Rust interop result JSON was accepted")

        valid_payload = {
            "schema_version": 1,
            "area": "rust_interop",
            "bless": False,
            "suites": [
                {
                    "name": suite,
                    "blocking": True,
                    "total_variants": 1,
                    "total_failures": 0,
                }
                for suite in sorted(required_suites)
            ],
            "summary": {
                "blocking_failures": 0,
                "total_variants": len(required_suites),
            },
        }
        result_path.write_text(json.dumps(valid_payload), encoding="utf-8")
        validate_rust_interop_result(result_path, sorted(required_suites))

        invalid_payloads = [
            {**valid_payload, "schema_version": 2},
            {**valid_payload, "area": "python_interop"},
            {**valid_payload, "bless": True},
            {**valid_payload, "suites": valid_payload["suites"][:-1]},
            {**valid_payload, "suites": "not-a-list"},
            {
                **valid_payload,
                "summary": {"blocking_failures": 1, "total_variants": len(required_suites)},
            },
            {
                **valid_payload,
                "suites": [
                    {**valid_payload["suites"][0], "blocking": False},
                    *valid_payload["suites"][1:],
                ],
            },
            {
                **valid_payload,
                "suites": [
                    {**valid_payload["suites"][0], "total_failures": 1},
                    *valid_payload["suites"][1:],
                ],
            },
            {
                **valid_payload,
                "suites": [
                    {**valid_payload["suites"][0], "total_variants": 0},
                    *valid_payload["suites"][1:],
                ],
            },
            {
                **valid_payload,
                "suites": [
                    {**valid_payload["suites"][0], "total_variants": True},
                    *valid_payload["suites"][1:],
                ],
            },
            {
                **valid_payload,
                "summary": {"blocking_failures": 0, "total_variants": 0},
            },
            {
                **valid_payload,
                "summary": {"blocking_failures": 0, "total_variants": True},
            },
        ]
        for index, payload in enumerate(invalid_payloads):
            result_path.write_text(json.dumps(payload), encoding="utf-8")
            try:
                validate_rust_interop_result(result_path, sorted(required_suites))
            except ProfileRunnerError:
                pass
            else:
                raise AssertionError(
                    f"invalid Rust interop result JSON mutation {index} was accepted"
                )
        result_path.write_text("{not-json", encoding="utf-8")
        try:
            validate_rust_interop_result(result_path, sorted(required_suites))
        except ProfileRunnerError:
            pass
        else:
            raise AssertionError("malformed Rust interop result JSON was accepted")


def _documentation_profile_self_test() -> None:
    profiles = load_all_profiles()
    release = profiles["release"]
    selected = [
        selection
        for selection in release["selected_areas"]
        if selection.get("area") == "documentation"
    ]
    if len(selected) != 1 or selected[0].get("suites") != ["structure"]:
        raise AssertionError("release profile must select documentation:structure exactly once")
    if "documentation_suites" in legacy_facade(release):
        raise AssertionError("documentation suites must have selected_areas as their sole authority")
    if "documentation_checks" not in legacy_facade_step_names(release):
        raise AssertionError("release profile omitted executable documentation_checks step")

    result_path = (
        Path(__file__).resolve().parents[3]
        / "target"
        / "verification"
        / "areas"
        / "documentation-documentation-self-test-results.json"
    )

    def write_documentation_result(command: list[str]) -> None:
        if command[-2] != "--result-json":
            raise AssertionError(f"documentation result path was not requested: {command}")
        result_path.parent.mkdir(parents=True, exist_ok=True)
        result_path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "area": "documentation",
                    "bless": False,
                    "suites": [
                        {
                            "name": "structure",
                            "blocking": True,
                            "total_variants": 1,
                            "total_failures": 0,
                        }
                    ],
                    "summary": {"blocking_failures": 0, "total_variants": 1},
                }
            ),
            encoding="utf-8",
        )

    output = io.StringIO()
    with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
        result = timed_step(
            "documentation_checks",
            lambda: run_selected_area(
                area="documentation",
                suites=["structure"],
                profile_name="documentation-self-test",
                result_slug="documentation",
                command_builder=lambda *args: list(args),
                command_runner=write_documentation_result,
            ),
        )
    if result.status != 0:
        raise AssertionError(f"documentation step failed:\n{output.getvalue()}")
    if "[sifr-lane-step] name=documentation_checks" not in output.getvalue() or "status=pass" not in output.getvalue():
        raise AssertionError("documentation step did not emit visible passing lane evidence")

    validate_area_result(
        result_path,
        area="documentation",
        expected_suites=["structure"],
    )
    result_path.unlink(missing_ok=True)
    with tempfile.TemporaryDirectory(prefix="sifr-doc-result-self-test-") as temp_dir:
        missing = Path(temp_dir) / "selected-but-unrun.json"
        try:
            validate_area_result(
                missing,
                area="documentation",
                expected_suites=["structure"],
            )
        except AreaResultError:
            pass
        else:
            raise AssertionError("selected-but-unrun documentation suite was accepted")


def _release_report_precondition_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-release-report-self-test-") as directory:
        existing = Path(directory) / "release-profile-report.json"
        existing.write_text("occupied", encoding="utf-8")
        cases = [
            (existing, "release", "already exists"),
            (Path(directory) / "non-release.json", "merge", "accepted only for the release"),
            (
                Path(__file__).resolve().parents[3]
                / "target"
                / "release-profile-report.json",
                "release",
                "outside the repository checkout",
            ),
        ]
        for path, profile, expected in cases:
            try:
                prepare_release_report_output(str(path), profile_name=profile)
            except ValueError as exc:
                if expected not in str(exc):
                    raise AssertionError(f"unexpected release report rejection: {exc}") from exc
            else:
                raise AssertionError(f"release report precondition mutation passed: {path}")


def _release_report_production_self_test() -> None:
    selected = {
        "rust_interop": [
            "matrix",
            "tiers",
            "compatibility-matrix",
            "stale-drafts",
            "stable-candidate",
        ],
        "developer_tooling": ["full"],
        "documentation": ["structure"],
        "distribution_release": ["full", "qualification", "evidence-custody"],
    }
    with tempfile.TemporaryDirectory(prefix="sifr-release-report-production-") as directory:
        root = Path(directory)
        result_root = root / "target" / "verification" / "areas"
        result_root.mkdir(parents=True)
        for area, filename in CRITICAL_RESULTS.items():
            suites = []
            for suite_name in selected[area]:
                labels = [f"{suite_name}:case"]
                if area == "developer_tooling":
                    labels.append("editor-release:package")
                suites.append(
                    {
                        "name": suite_name,
                        "cases": [
                            {
                                "variants": [
                                    {"label": label}
                                    for label in labels
                                ]
                            }
                        ],
                    }
                )
            (result_root / filename).write_text(
                json.dumps({"area": area, "suites": suites}),
                encoding="utf-8",
            )
        log_path = root / "release.log"
        log_path.write_text(
            "".join(
                f"[sifr-lane-step] name={name} elapsed_ms=1 status=pass\n"
                for name in (
                    "rust_interop_checks",
                    "developer_tooling_checks",
                    "documentation_checks",
                    "distribution_validation",
                )
            ),
            encoding="utf-8",
        )
        output_path = root / "release-profile-report.json"
        payload = build_release_profile_payload(
            output_path=output_path,
            log_path=log_path,
            profile={
                "selected_areas": [
                    {"area": area, "suites": suites}
                    for area, suites in selected.items()
                ]
            },
            profile_digest="a" * 64,
            commit="e" * 40,
            submodules={},
            toolchain={
                "rustc": "rustc fixture",
                "cargo": "cargo fixture",
                "uv": "uv fixture",
                "python": "python fixture",
            },
            result_root=result_root,
            artifact_root=root,
        )
        canonical = (
            json.dumps(
                payload,
                ensure_ascii=False,
                allow_nan=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            + "\n"
        ).encode()
        output_path.write_bytes(canonical)
        validate_release_profile_report(
            json.loads(output_path.read_text(encoding="utf-8")),
            canonical_bytes=output_path.read_bytes(),
            expected_profile_sha256="a" * 64,
        )


def _e2e_profile_self_test() -> None:
    profiles = load_all_profiles()
    create_pr_manifest = legacy_facade(profiles["create-pr"])["e2e"].get("fixture_manifest")
    if create_pr_manifest != "verification/areas/core_language/data/create_pr_e2e_manifest.json":
        raise AssertionError(f"create-pr e2e must remain representative, got: {create_pr_manifest}")

    for profile_name in ("merge", "nightly", "release"):
        fixture_manifest = legacy_facade(profiles[profile_name])["e2e"].get("fixture_manifest")
        if fixture_manifest:
            raise AssertionError(
                f"{profile_name} e2e must use the full pass corpus, got fixture manifest: "
                f"{fixture_manifest}",
            )

    merge_full_suites = crate_test_suites_for_mode(profiles["merge"], "full")
    by_id = {str(suite.get("id")): suite for suite in merge_full_suites}
    cli_suite = by_id.get("sifr_cli_full")
    if not isinstance(cli_suite, dict):
        raise AssertionError("merge full crate tests must include sifr_cli_full")
    command = cli_suite.get("command")
    if command != ["test", "-p", "sifr", "--", "--skip", "test_e2e_pass"]:
        raise AssertionError(
            "sifr_cli_full must skip only test_e2e_pass so the full fail corpus remains "
            f"merge-blocking, got: {command}",
        )


def _discovery_self_test() -> None:
    committed_areas = {area.name for area in discover_areas()}
    if "core_language" not in committed_areas:
        raise AssertionError(f"core_language area was not discovered: {sorted(committed_areas)}")
    if "diagnostics" not in committed_areas:
        raise AssertionError(f"diagnostics area was not discovered: {sorted(committed_areas)}")
    if "project_workspace" not in committed_areas:
        raise AssertionError(f"project_workspace area was not discovered: {sorted(committed_areas)}")
    if "regression" not in committed_areas:
        raise AssertionError(f"regression area was not discovered: {sorted(committed_areas)}")
    if "fuzz_property" not in committed_areas:
        raise AssertionError(f"fuzz_property area was not discovered: {sorted(committed_areas)}")

    with tempfile.TemporaryDirectory() as tmp:
        areas_dir = Path(tmp) / "areas"
        demo_dir = areas_dir / "core_language"
        demo_dir.mkdir(parents=True)
        (demo_dir / "manifest.json").write_text(
            """
{
  "schema_version": 1,
  "name": "core_language",
  "owner": "compiler/core",
  "description": "Temporary discovery fixture.",
  "parallel_safe": true,
  "resource_classes": ["default-local"],
  "timeout_seconds": 60,
  "suites": []
}
""".strip()
            + "\n",
            encoding="utf-8",
        )
        areas = discover_areas(areas_dir)
    if [area.name for area in areas] != ["core_language"]:
        raise AssertionError(f"unexpected discovery result: {areas}")


def _resource_class_self_test() -> None:
    profile = {
        "resource_policy": {"classes": ["default-local", "network", "container-runtime"]},
        "selected_areas": [
            {"area": "core_language", "resource_classes": ["default-local"]},
            {"area": "ecosystem_compatibility", "resource_classes": ["external-corpus"]},
        ],
    }
    expected = {"default-local", "network", "external-corpus", "container-runtime"}
    actual = selected_resource_classes(profile)
    if actual != expected:
        raise AssertionError(f"resource class selection mismatch: {actual}")


def _failure_reproduction_self_test() -> None:
    command = failure_reproduction_command("create-pr", "case-001")
    expected = "uv run --project verification python -m sifr_verify --profile create-pr --case case-001"
    if command != expected:
        raise AssertionError(f"unexpected reproduction command: {command}")
    result = build_result(
        profile="create-pr",
        status="fail",
        elapsed_ms=7,
        cases=[{"id": "case-001", "status": "fail", "elapsed_ms": 7}],
    )
    schema = load_schema("result.schema.json")
    validate_data(result, schema, source="self-test result")
    failures = result.get("failures")
    if failures != [{"case_id": "case-001", "reproduce": expected}]:
        raise AssertionError(f"unexpected failure reproduction data: {failures}")

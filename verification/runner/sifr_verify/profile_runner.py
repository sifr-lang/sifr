"""Execute validation profiles through the verification runner."""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from .cargo_setup import (
    enable_offline_cargo as enable_profile_offline_cargo,
    prepare_cargo_cache as prepare_profile_cargo_cache,
)
from .errors import VerificationError
from .paths import REPO_ROOT
from .profile_commands import (
    CommandFailed,
    cargo_command,
    run_command,
    run_python,
    uv_area_command,
)
from .profile_area_steps import AreaResultError, run_selected_area, validate_area_result
from .profile_reporting import run_profile_with_report
from .profiles import (
    crate_test_suites_for_mode,
    legacy_facade,
    load_profile,
    resolve_fixture_manifest,
    selected_suites_for_area,
)
from .step_budgets import (
    StepBudgetContext,
    enforce_step_budget as enforce_prepared_step_budget,
    prepare_step_budget,
    record_step_success,
)

sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "common"))

from sifr_binary import resolve_sifr_binary  # noqa: E402


class ProfileRunnerError(VerificationError):
    """Profile execution failed before a validation command could run."""


@dataclass(frozen=True)
class StepResult:
    status: int
    elapsed_ms: int


LEGACY_FACADE_STEPS_BEFORE_GENERATED = (
    ("coverage_matrix_checks", "run_coverage_matrix_checks"),
    ("core_guardrails", "run_core_guardrails"),
    ("diagnostic_rules", "run_diagnostic_rules"),
    ("cpython_differential", "run_cpython_differential_suites"),
    ("python_interop", "run_python_interop_suites"),
    ("rust_interop_checks", "run_rust_interop_checks"),
    ("frontend_syntax_guardrails", "run_frontend_syntax_guardrails"),
    ("developer_tooling_checks", "run_developer_tooling_checks"),
    ("documentation_checks", "run_documentation_checks"),
    ("performance_budget_checks", "run_performance_budget_checks"),
    ("verification_hardening_self_tests", "run_verification_hardening_self_tests"),
    ("verification_runner_foundation", "run_verification_runner_foundation_checks"),
    ("fuzz_property_checks", "run_fuzz_property_suites"),
    ("algorithmic_compatibility_checks", "run_algorithmic_compatibility_suites"),
    ("distribution_validation", "run_distribution_checks"),
    ("sysroot_release_certification", "run_sysroot_release_checks"),
)
GENERATED_CODE_QUALITY_STEP = (
    "generated_code_quality_checks",
    "run_generated_code_quality_checks",
)
LEGACY_FACADE_STEPS_AFTER_GENERATED = (
    ("crate_tests", "run_crate_tests"),
    ("validation_suite_matrix", "run_validation_suites"),
    ("runtime_platform_suites", "run_runtime_platform_suites"),
    ("e2e_pass_suite", "run_e2e_pass_suite"),
    ("verification_hardening_suites", "run_hardening_suites"),
    ("extra_e2e_checks", "run_extra_e2e_checks"),
)


def legacy_facade_step_methods(profile: dict[str, Any]) -> list[tuple[str, str]]:
    steps = list(LEGACY_FACADE_STEPS_BEFORE_GENERATED)
    if legacy_facade(profile)["generated_code_quality"] != "none":
        steps.append(GENERATED_CODE_QUALITY_STEP)
    steps.extend(LEGACY_FACADE_STEPS_AFTER_GENERATED)
    return steps


def legacy_facade_step_names(profile: dict[str, Any]) -> set[str]:
    return {name for name, _method_name in legacy_facade_step_methods(profile)}


def validate_rust_interop_result(result_path: Path, expected_suites: list[str]) -> None:
    try:
        validate_area_result(
            result_path,
            area="rust_interop",
            expected_suites=expected_suites,
        )
    except AreaResultError as exc:
        raise ProfileRunnerError(str(exc)) from exc


def now_ms() -> int:
    return time.monotonic_ns() // 1_000_000


def timed_step(name: str, callback: Callable[[], None]) -> StepResult:
    start_ms = now_ms()
    status = 0
    try:
        callback()
    except CommandFailed as exc:
        status = exc.returncode
    except ProfileRunnerError as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        status = 2
    end_ms = now_ms()
    elapsed_ms = end_ms - start_ms
    label = "pass" if status == 0 else "fail"
    print(f"[sifr-lane-step] name={name} elapsed_ms={elapsed_ms} status={label}")
    return StepResult(status=status, elapsed_ms=elapsed_ms)


class ProfileRunner:
    """Run the legacy validation sequence from profile data."""

    def __init__(self, profile_name: str, forward_args: list[str]) -> None:
        self.profile = load_profile(profile_name)
        self.profile_name = str(self.profile["name"])
        self.legacy = legacy_facade(self.profile)
        self.forward_args = forward_args
        self.env = os.environ.copy()
        configured_sifr_binary = self.env.get("SIFR_GCQ_BIN") or self.env.get("SIFR_RUNTIME_PLATFORM_BIN")
        sifr_binary = (
            Path(configured_sifr_binary)
            if configured_sifr_binary
            else resolve_sifr_binary(
                REPO_ROOT,
                default_binary=REPO_ROOT / "target" / "debug" / "sifr",
            )
        )
        self.env.setdefault("SIFR_GCQ_BIN", str(sifr_binary))
        self.env.setdefault("SIFR_RUNTIME_PLATFORM_BIN", str(sifr_binary))
        os.environ.setdefault("SIFR_GCQ_BIN", str(sifr_binary))
        os.environ.setdefault("SIFR_RUNTIME_PLATFORM_BIN", str(sifr_binary))
        probe_cache_root = REPO_ROOT / "target" / "sifr_rust_bridge_probe_cache" / self.profile_name
        self.env["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)
        os.environ["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)

    def run(self) -> int:
        self.print_header()
        setup_budget = self.prepare_step_budget("cargo_cache_setup")
        setup_result = self.run_timed_step("cargo_cache_setup", self.prepare_cargo_cache)
        if setup_result.status != 0:
            return setup_result.status
        setup_budget_status = self.enforce_step_budget(
            "cargo_cache_setup",
            setup_result.elapsed_ms,
            setup_budget,
        )
        if setup_budget_status != 0:
            return setup_budget_status
        record_step_success(setup_budget)
        if self.profile.get("cargo_policy", {}).get("offline") is True:
            self.enable_offline_cargo()
        if self.execution_mode == "selected-areas-only":
            return self.run_selected_areas_only()

        steps = self.legacy_facade_steps()

        for name, callback in steps:
            budget = self.prepare_step_budget(name)
            result = self.run_timed_step(name, callback)
            if result.status != 0:
                return result.status
            budget_status = self.enforce_step_budget(name, result.elapsed_ms, budget)
            if budget_status != 0:
                return budget_status
            record_step_success(budget)
        return 0

    def prepare_cargo_cache(self) -> None:
        try:
            prepare_profile_cargo_cache(self.profile, self.env, run_command)
        except ValueError as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def run_timed_step(self, name: str, callback: Callable[[], None]) -> StepResult:
        """Execute and report one profile step."""
        return timed_step(name, callback)

    def enable_offline_cargo(self) -> None:
        enable_profile_offline_cargo(self.env)

    def legacy_facade_steps(self) -> list[tuple[str, Callable[[], None]]]:
        return [(name, getattr(self, method_name)) for name, method_name in legacy_facade_step_methods(self.profile)]

    @property
    def execution_mode(self) -> str:
        return str(self.profile.get("execution_mode", "legacy-facade"))

    @property
    def budgets(self) -> dict[str, Any]:
        return self.profile["budgets"]

    @property
    def matrix_suites(self) -> list[str]:
        return list(self.legacy["matrix_suites"])

    @property
    def tooling_suites(self) -> list[str]:
        return list(self.legacy["tooling_suites"])

    @property
    def documentation_suites(self) -> list[str]:
        return selected_suites_for_area(self.profile, "documentation")

    @property
    def hardening_suites(self) -> list[str]:
        return list(self.legacy["hardening_suites"])

    @property
    def extra_checks(self) -> list[str]:
        return list(self.legacy["extra_checks"])

    @property
    def e2e(self) -> dict[str, Any]:
        return self.legacy["e2e"]

    @property
    def distribution_mode(self) -> str:
        return str(self.legacy["distribution"])

    @property
    def generated_code_quality_mode(self) -> str:
        return str(self.legacy["generated_code_quality"])

    @property
    def performance_budget_mode(self) -> str:
        return str(self.legacy["performance_budget"])

    @property
    def crate_test_mode(self) -> str:
        return str(self.legacy["crate_tests"])

    def print_header(self) -> None:
        print("Running local-first validation")
        print(f"  profile={self.profile_name}")
        print(f"  lane={self.profile_name}")
        print(
            f"  budget=warm<={self.budgets['warm_wall_time_minutes']}m cold<={self.budgets['cold_wall_time_minutes']}m"
        )
        print(f"  policy=thermal:{self.legacy['thermal_policy']} memory:{self.legacy['memory_policy']}")

    def selected_suites_for_area(self, area_name: str) -> list[str]:
        suites: list[str] = []
        for selection in self.profile.get("selected_areas", []):
            if not isinstance(selection, dict) or selection.get("area") != area_name:
                continue
            raw_suites = selection.get("suites", [])
            if isinstance(raw_suites, list):
                suites.extend(str(suite) for suite in raw_suites)
        return suites

    def prepare_step_budget(self, name: str) -> StepBudgetContext | None:
        return prepare_step_budget(
            repo_root=REPO_ROOT,
            profile=self.profile,
            profile_name=str(self.profile.get("name", "unknown")),
            name=name,
            env=getattr(self, "env", os.environ),
        )

    def enforce_step_budget(
        self,
        _name: str,
        elapsed_ms: int,
        context: StepBudgetContext | None = None,
    ) -> int:
        return enforce_prepared_step_budget(context, elapsed_ms)

    def run_selected_areas_only(self) -> int:
        selections = [selection for selection in self.profile.get("selected_areas", []) if isinstance(selection, dict)]
        if not selections:
            print(
                f"sifr_verify: profile {self.profile_name} selects no areas",
                file=sys.stderr,
            )
            return 2
        for selection in selections:
            area = str(selection["area"])
            suites = [str(suite) for suite in selection.get("suites", [])]
            step_name = f"{area}_selected_suites"
            result = timed_step(
                step_name,
                lambda area=area, suites=suites: self.run_area_suites(area, suites),
            )
            if result.status != 0:
                return result.status
        return 0

    def run_area_suites(self, area: str, suites: list[str]) -> None:
        args = ["--area", area]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args), env=self.env)

    def run_coverage_matrix_checks(self) -> None:
        suites = self.selected_suites_for_area("coverage_matrix")
        if not suites:
            print(f"Skipping coverage matrix checks for lane {self.profile_name}")
            return
        args = ["--area", "coverage_matrix"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args))

    def run_core_guardrails(self) -> None:
        print("Running lowering maintainability guardrails")
        run_python("scripts/check_hir_maintainability_guardrails.py")

        print("Running file-size guardrails")
        run_python("scripts/check_file_size_guardrails.py")

        print("Running source crate dependency-direction guardrail")
        run_python("scripts/check_source_crate_dependency_direction.py")
        run_python("scripts/check_source_crate_dependency_direction.py", "--self-test")

        print("Running submodule ownership guardrail")
        run_python("scripts/check_submodule_ownership.py")
        run_python("scripts/check_submodule_ownership.py", "--self-test")

        print("Running sysroot stdlib resource certification gate")
        run_python("scripts/check_sysroot_stdlib_resource_certification_gate.py")
        run_python("scripts/check_sysroot_stdlib_resource_certification_gate.py", "--self-test")

        print("Running stdlib native intrinsic allowlist guard")
        run_python("scripts/check_stdlib_native_intrinsic_allowlist.py")
        run_python("scripts/check_stdlib_native_intrinsic_allowlist.py", "--self-test")

        print("Running stdlib native adapter reachability guard")
        run_python("scripts/check_stdlib_native_adapter_reachability.py")
        run_python("scripts/check_stdlib_native_adapter_reachability.py", "--self-test")

        print("Running stdlib retained manifest schema guard")
        run_python("scripts/check_stdlib_manifest_schema.py")
        run_python("scripts/check_stdlib_manifest_schema.py", "--self-test")

        print("Running stdlib bootstrap ordering guard")
        run_python("scripts/check_stdlib_bootstrap_ordering.py")
        run_python("scripts/check_stdlib_bootstrap_ordering.py", "--self-test")

        print("Running audit fixture smoke suites")
        run_command(uv_area_command("--area", "core_language", "--suite", "audit-fixtures"))
        run_command(uv_area_command("--area", "project_workspace", "--suite", "audit-fixtures"))
        run_command(uv_area_command("--area", "stdlib_parity", "--suite", "audit-fixtures"))

        print("Running TypeScript-Go architecture transfer guardrails")
        run_command(uv_area_command("--area", "developer_tooling", "--suite", "typescript-go-transfer"))

        print("Running sifr_driver maintainability guardrails")
        run_python("scripts/check_sifr_driver_maintainability_guardrails.py")

        print("Running package-manager guardrails")
        run_command(uv_area_command("--area", "package_management", "--suite", "guardrails"))
        print("Running offline package merge smoke")
        run_command(uv_area_command("--area", "package_management", "--suite", "offline-merge-smoke"))
        for suite in self.selected_suites_for_area("package_management"):
            if suite in {"guardrails", "offline-merge-smoke"}:
                continue
            print(f"Running package-management suite {suite}")
            run_command(uv_area_command("--area", "package_management", "--suite", suite))

        print("Running stdlib parity inventory guardrails")
        run_command(uv_area_command("--area", "stdlib_parity", "--suite", "complexity-resource"))
        run_command(uv_area_command("--area", "stdlib_parity", "--suite", "module-inventory"))
        print("Running stdlib module parity merge check")
        run_command(uv_area_command("--area", "stdlib_parity", "--suite", "module-merge-check"))
        for suite in self.selected_suites_for_area("stdlib_parity"):
            if suite in {
                "audit-fixtures",
                "complexity-resource",
                "module-inventory",
                "module-merge-check",
            }:
                continue
            print(f"Running stdlib parity suite {suite}")
            run_command(uv_area_command("--area", "stdlib_parity", "--suite", suite))

    def run_diagnostic_rules(self) -> None:
        print("Running diagnostics area rules checks")
        run_command(uv_area_command("--area", "diagnostics", "--suite", "rules"))

        print("Running diagnostic presentation rules check")
        run_command(uv_area_command("--area", "developer_tooling", "--suite", "diagnostic-rules"))

    def run_cpython_differential_suites(self) -> None:
        suites = self.selected_suites_for_area("cpython_differential")
        if not suites:
            print(f"Skipping CPython differential checks for lane {self.profile_name}")
            return
        print("Running CPython differential checks")
        args = ["--area", "cpython_differential"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args))

    def run_python_interop_suites(self) -> None:
        suites = self.selected_suites_for_area("python_interop")
        if not suites:
            print(f"Skipping Python interop checks for lane {self.profile_name}")
            return
        print("Running Python interop checks")
        args = ["--area", "python_interop"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args), env=self.env)

    def run_rust_interop_checks(self) -> None:
        suites = self.selected_suites_for_area("rust_interop")
        if not suites:
            raise ProfileRunnerError(f"profile {self.profile_name} has no Rust interop suites to execute")
        print("Running Rust interop checks")
        try:
            run_selected_area(
                area="rust_interop",
                suites=suites,
                profile_name=self.profile_name,
                result_slug="rust-interop",
                command_builder=uv_area_command,
                command_runner=run_command,
            )
        except AreaResultError as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def run_frontend_syntax_guardrails(self) -> None:
        print("Running frontend and syntax guardrails")
        run_command(uv_area_command("--area", "performance", "--suite", "frontend-syntax-guardrails"))

    def run_developer_tooling_checks(self) -> None:
        print("Running Developer Tooling Checks")
        print("  suites=" + (",".join(self.tooling_suites) if self.tooling_suites else "none"))
        if self.tooling_suites:
            try:
                run_selected_area(
                    area="developer_tooling",
                    suites=self.tooling_suites,
                    profile_name=self.profile_name,
                    result_slug="developer-tooling",
                    command_builder=uv_area_command,
                    command_runner=run_command,
                )
            except AreaResultError as exc:
                raise ProfileRunnerError(str(exc)) from exc

    def run_documentation_checks(self) -> None:
        if not self.documentation_suites:
            print(f"Skipping documentation checks for lane {self.profile_name}")
            return
        print("Running Documentation Checks")
        print("  suites=" + ",".join(self.documentation_suites))
        try:
            run_selected_area(
                area="documentation",
                suites=self.documentation_suites,
                profile_name=self.profile_name,
                result_slug="documentation",
                command_builder=uv_area_command,
                command_runner=run_command,
            )
        except AreaResultError as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def run_performance_budget_checks(self) -> None:
        print("Running Performance Budget Checks")
        print(f"  mode={self.performance_budget_mode}")
        if self.performance_budget_mode in {"smoke", "representative", "full"}:
            run_command(uv_area_command("--area", "performance", "--suite", self.performance_budget_mode))
        else:
            print(f"Skipping performance benchmark execution for lane {self.profile_name}")

    def run_verification_hardening_self_tests(self) -> None:
        print("Running verification hardening runner self-tests")
        run_command([sys.executable, "-m", "sifr_verify.hardening", "--self-test"])

    def run_verification_runner_foundation_checks(self) -> None:
        print("Running verification runner foundation checks")
        run_command(["uv", "lock", "--project", "verification", "--check"])
        run_command(
            [
                "uv",
                "run",
                "--project",
                "verification",
                "--locked",
                "python",
                "-m",
                "sifr_verify",
                "--self-test",
            ]
        )

    def run_fuzz_property_suites(self) -> None:
        legacy_fuzz_suites = {suite for suite in self.hardening_suites if suite in {"property", "fuzz-smoke"}}
        suites = [suite for suite in self.selected_suites_for_area("fuzz_property") if suite not in legacy_fuzz_suites]
        if not suites:
            print(f"Skipping fuzz/property checks for lane {self.profile_name}")
            return
        print("Running fuzz/property checks")
        args = ["--area", "fuzz_property"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args, "--hardening-summary"))

    def run_algorithmic_compatibility_suites(self) -> None:
        suites = self.selected_suites_for_area("algorithmic_compatibility")
        if not suites:
            print(f"Skipping algorithmic compatibility checks for lane {self.profile_name}")
            return
        print("Running algorithmic compatibility checks")
        args = ["--area", "algorithmic_compatibility"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args, "--hardening-summary"))

    def run_distribution_checks(self) -> None:
        if self.distribution_mode == "none":
            print(f"Skipping distribution validation for lane {self.profile_name}")
            return
        print("Running distribution validation")
        print(f"  mode={self.distribution_mode}")
        if self.distribution_mode not in {"representative", "full"}:
            raise ProfileRunnerError(f"unknown distribution validation mode: {self.distribution_mode}")
        suites = self.selected_suites_for_area("distribution_release")
        if self.distribution_mode not in suites:
            raise ProfileRunnerError(
                f"profile {self.profile_name} does not select distribution mode {self.distribution_mode}"
            )
        try:
            run_selected_area(
                area="distribution_release",
                suites=suites,
                profile_name=self.profile_name,
                result_slug="distribution-release",
                command_builder=uv_area_command,
                command_runner=run_command,
            )
        except AreaResultError as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def run_sysroot_release_checks(self) -> None:
        suites = self.selected_suites_for_area("sysroot_release")
        if not suites:
            print(f"Skipping sysroot release certification for lane {self.profile_name}")
            return
        print("Running sysroot release certification")
        args = ["--area", "sysroot_release"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args))

    def run_generated_code_quality_checks(self) -> None:
        print("Running Generated Code Quality Checks")
        print(f"  mode={self.generated_code_quality_mode}")
        if self.generated_code_quality_mode not in {
            "smoke",
            "representative",
            "full",
            "release-full",
        }:
            raise ProfileRunnerError(f"unsupported generated-code quality mode: {self.generated_code_quality_mode}")
        shared_root = REPO_ROOT / "target" / "sifr_generated_code_quality" / f"{self.profile_name}.shared"
        env = self.env | {"SIFR_GCQ_SHARED_ROOT": str(shared_root.relative_to(REPO_ROOT))}
        run_command(
            uv_area_command(
                "--area",
                "generated_code_quality",
                "--suite",
                self.generated_code_quality_mode,
                "--hardening-summary",
            ),
            env=env,
        )

    def run_crate_tests(self) -> None:
        print("Running crate tests")
        print(f"  mode={self.crate_test_mode}")
        suites = crate_test_suites_for_mode(self.profile, self.crate_test_mode)
        for suite in suites:
            suite_id = str(suite["id"])
            status = str(suite["status"])
            executed = bool(suite["executed_in_merge"])
            if status == "red-blocker" and not executed:
                print(
                    "Planned crate test red-blocker "
                    f"{suite_id}: must_be_executed_by={suite.get('must_be_executed_by', 'unknown')}"
                )
                continue
            command = suite.get("command", [])
            if not isinstance(command, list) or not all(isinstance(arg, str) for arg in command):
                raise ProfileRunnerError(f"crate test suite {suite_id} has invalid command")
            print(f"Running crate test suite {suite_id}")
            start_ms = now_ms()
            status = 0
            try:
                run_command(cargo_command(*command), env=self.env)
            except CommandFailed as exc:
                status = exc.returncode
            elapsed_ms = now_ms() - start_ms
            case_status = "pass" if status == 0 else "fail"
            print(f"[sifr-case-timing] bucket=crate_tests case={suite_id} elapsed_ms={elapsed_ms} status={case_status}")
            if status != 0:
                raise CommandFailed(status)

    def run_validation_suites(self) -> None:
        if not self.matrix_suites:
            return
        print("Running validation suite area checks")
        core_language = {
            "integer_dtype_rules",
            "hir_analysis_behaviors",
            "cfg_flow_behaviors",
            "syntax_parser_lexer_matrix",
        }
        project_workspace = {"frontend_mode_parity", "project_graph_isolation"}
        core_args: list[str] = []
        project_args: list[str] = []
        for suite in self.matrix_suites:
            if suite in core_language:
                core_args.extend(["--suite", suite])
            elif suite in project_workspace:
                project_args.extend(["--suite", suite])
            else:
                raise ProfileRunnerError(f"unknown validation suite: {suite}")
        if core_args:
            run_command(uv_area_command("--area", "core_language", *core_args))
        if project_args:
            run_command(uv_area_command("--area", "project_workspace", *project_args))

    def run_runtime_platform_suites(self) -> None:
        suites = self.selected_suites_for_area("runtime_platform")
        if not suites:
            print(f"Skipping runtime platform suites for lane {self.profile_name}")
            return
        print("Running runtime platform suites")
        args = ["--area", "runtime_platform"]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args))

    def run_e2e_pass_suite(self) -> None:
        print("Running e2e pass suite")
        e2e_args = [
            "--profile",
            self.profile_name,
            "--sifr-jobs",
            str(self.e2e["sifr_jobs"]),
            "--rust-jobs",
            str(self.e2e["rust_jobs"]),
            "--run-jobs",
            str(self.e2e["run_jobs"]),
            "--cargo-build-jobs",
            str(self.e2e["cargo_build_jobs"]),
        ]
        max_group_fixtures = self.e2e.get("max_group_fixtures")
        if max_group_fixtures not in {None, ""}:
            e2e_args.extend(["--max-group-fixtures", str(max_group_fixtures)])
        fixture_manifest = resolve_fixture_manifest(str(self.e2e.get("fixture_manifest", "")))
        if fixture_manifest is not None:
            e2e_args.extend(["--fixture-manifest", str(fixture_manifest)])
        if bool(self.e2e["disable_cache"]):
            e2e_args.append("--no-cache")
        run_command(
            [
                "bash",
                "verification/runner/e2e/run_e2e_pass.sh",
                *e2e_args,
                *self.forward_args,
            ]
        )

    def run_hardening_suites(self) -> None:
        if not self.hardening_suites:
            return
        print("Running verification hardening suites")
        diagnostics = False
        project_workspace = False
        regression_args: list[str] = []
        fuzz_property_args: list[str] = []
        ecosystem_args: list[str] = []
        legacy_args = ["--profile", self.profile_name]
        for suite in self.hardening_suites:
            if suite == "diagnostics":
                diagnostics = True
            elif suite == "project":
                project_workspace = True
            elif suite in {"fixedbugs", "crashes"}:
                regression_args.extend(["--suite", suite])
            elif suite in {"property", "fuzz-smoke"}:
                fuzz_property_args.extend(["--suite", suite])
            elif suite in {"oss-curated", "ecosystem-broader"}:
                ecosystem_args.extend(["--suite", suite])
            else:
                legacy_args.extend(["--suite", suite])
        if diagnostics:
            run_command(
                uv_area_command(
                    "--area",
                    "diagnostics",
                    "--suite",
                    "baselines",
                    "--hardening-summary",
                )
            )
        if project_workspace:
            run_command(
                uv_area_command(
                    "--area",
                    "project_workspace",
                    "--suite",
                    "baselines",
                    "--hardening-summary",
                )
            )
        if regression_args:
            run_command(uv_area_command("--area", "regression", *regression_args, "--hardening-summary"))
        if fuzz_property_args:
            run_command(
                uv_area_command(
                    "--area",
                    "fuzz_property",
                    *fuzz_property_args,
                    "--hardening-summary",
                )
            )
        if ecosystem_args:
            run_command(
                uv_area_command(
                    "--area",
                    "ecosystem_compatibility",
                    *ecosystem_args,
                    "--hardening-summary",
                )
            )
        if len(legacy_args) > 2:
            run_command([sys.executable, "-m", "sifr_verify.hardening", *legacy_args])

    def run_extra_e2e_checks(self) -> None:
        if "e2e_report_determinism" in self.extra_checks:
            print("Running e2e report determinism check")
            run_command(
                [
                    "bash",
                    "verification/runner/e2e/check_report_determinism.sh",
                    "--profile",
                    self.profile_name,
                ]
            )
        if "e2e_sequential_parallel_equivalence" in self.extra_checks:
            print("Running e2e sequential-vs-parallel equivalence check")
            run_command(
                [
                    "bash",
                    "verification/runner/e2e/check_sequential_parallel_equivalence.sh",
                    "--profile",
                    self.profile_name,
                ]
            )


def run_profile(
    profile_name: str,
    forward_args: list[str],
    *,
    release_report_out: str | None = None,
) -> int:
    return run_profile_with_report(
        profile_name,
        lambda: ProfileRunner(profile_name, forward_args).run(),
        handled_error=ProfileRunnerError,
        release_report_out=release_report_out,
    )

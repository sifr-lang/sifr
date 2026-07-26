"""Execute validation profiles through the verification runner."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import resource
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, TextIO

from . import reports
from .errors import VerificationError
from .paths import REPO_ROOT
from .profiles import crate_test_suites_for_mode, legacy_facade, load_profile, resolve_fixture_manifest

sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "common"))

from sifr_binary import resolve_sifr_binary  # noqa: E402


class ProfileRunnerError(VerificationError):
    """Profile execution failed before a validation command could run."""


class CommandFailed(Exception):
    """A subprocess returned a non-zero exit code."""

    def __init__(self, returncode: int) -> None:
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


@dataclass(frozen=True)
class StepResult:
    status: int
    elapsed_ms: int


class Tee:
    """Write text to more than one stream."""

    def __init__(self, *streams: TextIO) -> None:
        self._streams = streams

    def write(self, data: str) -> int:
        for stream in self._streams:
            stream.write(data)
            stream.flush()
        return len(data)

    def flush(self) -> None:
        for stream in self._streams:
            stream.flush()


LEGACY_FACADE_STEPS_BEFORE_GENERATED = (
    ("coverage_matrix_checks", "run_coverage_matrix_checks"),
    ("core_guardrails", "run_core_guardrails"),
    ("diagnostic_rules", "run_diagnostic_rules"),
    ("cpython_differential", "run_cpython_differential_suites"),
    ("python_interop", "run_python_interop_suites"),
    ("rust_interop_checks", "run_rust_interop_checks"),
    ("frontend_syntax_guardrails", "run_frontend_syntax_guardrails"),
    ("developer_tooling_checks", "run_developer_tooling_checks"),
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
    if not result_path.is_file():
        raise ProfileRunnerError(f"Rust interop area emitted no result JSON: {result_path}")
    try:
        payload = json.loads(result_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ProfileRunnerError(f"Rust interop area emitted invalid result JSON: {result_path}") from exc
    if not isinstance(payload, dict) or payload.get("area") != "rust_interop":
        raise ProfileRunnerError(f"Rust interop area emitted an invalid result document: {result_path}")
    raw_suites = payload.get("suites")
    if not isinstance(raw_suites, list):
        raise ProfileRunnerError(f"Rust interop result JSON has no suite results: {result_path}")
    for suite in raw_suites:
        if (
            not isinstance(suite, dict)
            or suite.get("blocking") is not True
            or not isinstance(suite.get("total_variants"), int)
            or int(suite["total_variants"]) <= 0
            or suite.get("total_failures") != 0
        ):
            raise ProfileRunnerError(
                f"Rust interop result JSON contains invalid suite evidence: {result_path}"
            )
    actual_suites = {
        str(suite.get("name"))
        for suite in raw_suites
        if isinstance(suite, dict) and isinstance(suite.get("name"), str)
    }
    if actual_suites != set(expected_suites):
        raise ProfileRunnerError(
            "Rust interop result JSON suite mismatch: "
            f"expected={sorted(expected_suites)} actual={sorted(actual_suites)}"
        )
    summary = payload.get("summary")
    if (
        not isinstance(summary, dict)
        or summary.get("blocking_failures") != 0
        or not isinstance(summary.get("total_variants"), int)
        or int(summary["total_variants"]) <= 0
    ):
        raise ProfileRunnerError(
            f"Rust interop result JSON contains invalid blocking summary: {result_path}"
        )


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> None:
    proc = subprocess.Popen(
        command,
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert proc.stdout is not None
    for line in proc.stdout:
        sys.stdout.write(line)
    returncode = proc.wait()
    if returncode != 0:
        raise CommandFailed(returncode)


def uv_area_command(*args: str) -> list[str]:
    return [
        "uv",
        "run",
        "--project",
        "verification",
        "--locked",
        "python",
        "-m",
        "sifr_verify",
        "areas",
        "run",
        *args,
    ]


def cargo_command(*args: str) -> list[str]:
    command = ["cargo", *args]
    if "--" in command:
        separator = command.index("--")
        return [*command[:separator], "--locked", *command[separator:]]
    return [*command, "--locked"]


def run_python(script: str, *args: str) -> None:
    run_command(["python3", script, *args])


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
        sifr_binary = Path(configured_sifr_binary) if configured_sifr_binary else resolve_sifr_binary(
            REPO_ROOT,
            default_binary=REPO_ROOT / "target" / "debug" / "sifr",
        )
        self.env.setdefault("SIFR_GCQ_BIN", str(sifr_binary))
        self.env.setdefault("SIFR_RUNTIME_PLATFORM_BIN", str(sifr_binary))
        os.environ.setdefault("SIFR_GCQ_BIN", str(sifr_binary))
        os.environ.setdefault("SIFR_RUNTIME_PLATFORM_BIN", str(sifr_binary))
        probe_cache_root = REPO_ROOT / "target" / "sifr_rust_bridge_probe_cache" / self.profile_name
        self.env["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)
        os.environ["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)
        if self.profile.get("cargo_policy", {}).get("offline") is True:
            self.env["CARGO_NET_OFFLINE"] = "true"
            os.environ["CARGO_NET_OFFLINE"] = "true"

    def run(self) -> int:
        self.print_header()
        if self.execution_mode == "selected-areas-only":
            return self.run_selected_areas_only()

        steps = self.legacy_facade_steps()

        for name, callback in steps:
            result = timed_step(name, callback)
            if result.status != 0:
                return result.status
            budget_status = self.enforce_step_budget(name, result.elapsed_ms)
            if budget_status != 0:
                return budget_status
        return 0

    def legacy_facade_steps(self) -> list[tuple[str, Callable[[], None]]]:
        return [
            (name, getattr(self, method_name))
            for name, method_name in legacy_facade_step_methods(self.profile)
        ]

    @property
    def execution_mode(self) -> str:
        return str(self.profile.get("execution_mode", "legacy-facade"))

    @property
    def budgets(self) -> dict[str, Any]:
        return self.profile["budgets"]

    @property
    def step_budgets(self) -> dict[str, Any]:
        budgets = self.profile.get("step_budgets", {})
        return budgets if isinstance(budgets, dict) else {}

    @property
    def matrix_suites(self) -> list[str]:
        return list(self.legacy["matrix_suites"])

    @property
    def tooling_suites(self) -> list[str]:
        return list(self.legacy["tooling_suites"])

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
            "  budget="
            f"warm<={self.budgets['warm_wall_time_minutes']}m "
            f"cold<={self.budgets['cold_wall_time_minutes']}m"
        )
        print(
            "  policy="
            f"thermal:{self.legacy['thermal_policy']} "
            f"memory:{self.legacy['memory_policy']}"
        )

    def selected_suites_for_area(self, area_name: str) -> list[str]:
        suites: list[str] = []
        for selection in self.profile.get("selected_areas", []):
            if not isinstance(selection, dict) or selection.get("area") != area_name:
                continue
            raw_suites = selection.get("suites", [])
            if isinstance(raw_suites, list):
                suites.extend(str(suite) for suite in raw_suites)
        return suites

    def enforce_step_budget(self, name: str, elapsed_ms: int) -> int:
        raw_budget = self.step_budgets.get(name)
        if not isinstance(raw_budget, dict):
            return 0
        budget_ms = int(raw_budget.get("budget_ms", 0) or 0)
        enforcement = str(raw_budget.get("enforcement", "advisory"))
        exceeded = budget_ms > 0 and elapsed_ms > budget_ms
        budget_status = "fail" if exceeded else "pass"
        print(
            "[sifr-lane-step-budget] "
            f"name={name} elapsed_ms={elapsed_ms} budget_ms={budget_ms} "
            f"enforcement={enforcement} status={budget_status}"
        )
        disabled = os.environ.get("SIFR_VERIFY_DISABLE_STEP_BUDGETS", "").lower() in {
            "1",
            "true",
            "yes",
        }
        if exceeded and enforcement == "blocking" and not disabled:
            print(
                f"sifr_verify: step budget exceeded for {name}: "
                f"{elapsed_ms}ms > {budget_ms}ms",
                file=sys.stderr,
            )
            return 124
        return 0

    def run_selected_areas_only(self) -> int:
        selections = [
            selection
            for selection in self.profile.get("selected_areas", [])
            if isinstance(selection, dict)
        ]
        if not selections:
            print(f"sifr_verify: profile {self.profile_name} selects no areas", file=sys.stderr)
            return 2
        for selection in selections:
            area = str(selection["area"])
            suites = [str(suite) for suite in selection.get("suites", [])]
            step_name = f"{area}_selected_suites"
            result = timed_step(step_name, lambda area=area, suites=suites: self.run_area_suites(area, suites))
            if result.status != 0:
                return result.status
        return 0

    def run_area_suites(self, area: str, suites: list[str]) -> None:
        args = ["--area", area]
        for suite in suites:
            args.extend(["--suite", suite])
        run_command(uv_area_command(*args))

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
        run_command(uv_area_command(*args))

    def run_rust_interop_checks(self) -> None:
        suites = self.selected_suites_for_area("rust_interop")
        if not suites:
            print(f"Skipping Rust interop checks for lane {self.profile_name}")
            return
        print("Running Rust interop checks")
        result_path = (
            REPO_ROOT
            / "target"
            / "verification"
            / "areas"
            / f"rust-interop-{self.profile_name}-results.json"
        )
        result_path.unlink(missing_ok=True)
        args = ["--area", "rust_interop"]
        for suite in suites:
            args.extend(["--suite", suite])
        args.extend(["--result-json", str(result_path.relative_to(REPO_ROOT))])
        run_command(uv_area_command(*args))
        validate_rust_interop_result(result_path, suites)

    def run_frontend_syntax_guardrails(self) -> None:
        print("Running frontend and syntax guardrails")
        run_command(uv_area_command("--area", "performance", "--suite", "frontend-syntax-guardrails"))

    def run_developer_tooling_checks(self) -> None:
        print("Running Developer Tooling Checks")
        print("  suites=" + (",".join(self.tooling_suites) if self.tooling_suites else "none"))
        if self.tooling_suites:
            args = ["--area", "developer_tooling"]
            for suite in self.tooling_suites:
                args.extend(["--suite", suite])
            run_command(uv_area_command(*args))

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
        run_command(["uv", "run", "--project", "verification", "--locked", "python", "-m", "sifr_verify", "--self-test"])

    def run_fuzz_property_suites(self) -> None:
        legacy_fuzz_suites = {
            suite for suite in self.hardening_suites if suite in {"property", "fuzz-smoke"}
        }
        suites = [
            suite
            for suite in self.selected_suites_for_area("fuzz_property")
            if suite not in legacy_fuzz_suites
        ]
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
        run_command(uv_area_command("--area", "distribution_release", "--suite", self.distribution_mode))

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
        if self.generated_code_quality_mode not in {"smoke", "representative", "full"}:
            raise ProfileRunnerError(
                f"unsupported generated-code quality mode: {self.generated_code_quality_mode}"
            )
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
            print(
                "[sifr-case-timing] "
                f"bucket=crate_tests case={suite_id} "
                f"elapsed_ms={elapsed_ms} status={case_status}"
            )
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
        run_command(["bash", "verification/runner/e2e/run_e2e_pass.sh", *e2e_args, *self.forward_args])

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
            run_command(uv_area_command("--area", "diagnostics", "--suite", "baselines", "--hardening-summary"))
        if project_workspace:
            run_command(uv_area_command("--area", "project_workspace", "--suite", "baselines", "--hardening-summary"))
        if regression_args:
            run_command(uv_area_command("--area", "regression", *regression_args, "--hardening-summary"))
        if fuzz_property_args:
            run_command(uv_area_command("--area", "fuzz_property", *fuzz_property_args, "--hardening-summary"))
        if ecosystem_args:
            run_command(uv_area_command("--area", "ecosystem_compatibility", *ecosystem_args, "--hardening-summary"))
        if len(legacy_args) > 2:
            run_command([sys.executable, "-m", "sifr_verify.hardening", *legacy_args])

    def run_extra_e2e_checks(self) -> None:
        if "e2e_report_determinism" in self.extra_checks:
            print("Running e2e report determinism check")
            run_command(["bash", "verification/runner/e2e/check_report_determinism.sh", "--profile", self.profile_name])
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


def write_time_file(path: Path, *, start: float, usage_start: resource.struct_rusage) -> None:
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    real_seconds = time.monotonic() - start
    user_seconds = max(0.0, usage.ru_utime - usage_start.ru_utime)
    sys_seconds = max(0.0, usage.ru_stime - usage_start.ru_stime)
    max_rss = int(usage.ru_maxrss)
    swaps = max(0, int(usage.ru_nswap - usage_start.ru_nswap))
    path.write_text(
        f"{real_seconds:.2f} real\n"
        f"{user_seconds:.2f} user\n"
        f"{sys_seconds:.2f} sys\n"
        f"{max_rss} maximum resident set size\n"
        f"{swaps} swaps\n",
        encoding="utf-8",
    )


def temporary_report_path(report_dir: Path, prefix: str) -> Path:
    with tempfile.NamedTemporaryFile(prefix=prefix, dir=report_dir, delete=False) as temp_file:
        return Path(temp_file.name)


def run_profile(profile_name: str, forward_args: list[str]) -> int:
    report_dir = REPO_ROOT / "target" / "validation_lane_reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    temp_log = temporary_report_path(report_dir, f"lane.{profile_name}.log.")
    temp_time = temporary_report_path(report_dir, f"lane.{profile_name}.time.")
    latest_log = report_dir / f"{profile_name}.latest.log"
    latest_time = report_dir / f"{profile_name}.latest.time"
    json_file = report_dir / f"{profile_name}.latest.json"
    start = time.monotonic()
    usage_start = resource.getrusage(resource.RUSAGE_CHILDREN)
    status = 0

    with temp_log.open("w", encoding="utf-8") as log_file:
        tee = Tee(sys.stdout, log_file)
        with contextlib.redirect_stdout(tee), contextlib.redirect_stderr(tee):
            try:
                status = ProfileRunner(profile_name, forward_args).run()
            except ProfileRunnerError as exc:
                print(f"sifr_verify: {exc}", file=sys.stderr)
                status = 2

    write_time_file(temp_time, start=start, usage_start=usage_start)
    shutil.copyfile(temp_log, latest_log)
    shutil.copyfile(temp_time, latest_time)
    try:
        reports.summarize(
            argparse.Namespace(
                profile=profile_name,
                log=str(latest_log),
                time_file=str(latest_time),
                json_out=str(json_file),
            )
        )
    except Exception as exc:  # Preserve validation status while surfacing report regressions.
        print(f"warning: lane report summarization failed: {exc}", file=sys.stderr)
    temp_log.unlink(missing_ok=True)
    temp_time.unlink(missing_ok=True)
    return status

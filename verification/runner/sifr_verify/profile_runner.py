"""Execute validation profiles from their canonical selections."""

from __future__ import annotations

import os
import json
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable

from .build_allocation import E2E_ENV, allocate, extract_overrides
from .native_test_allocation import ALLOCATION_ENV, allocate_native
from .cargo_setup import (
    enable_offline_cargo as enable_profile_offline_cargo,
    cargo_preparation_plan,
)
from .errors import VerificationError
from .execution_scope import evidence_name, scope_name, select_execution_scope
from .preparation import PreparationRun
from .paths import REPO_ROOT
from .profile_area_steps import AreaResultError, run_selected_area, run_segmented_python_interop
from .demo_freshness_policy import (
    STRICT_FLAG, requires_strict_freshness, validate_strict_freshness_evidence,
)
from .process_execution import SAFETY_DEADLINE_ENV, deadline_environment
from .profile_commands import CommandFailed, cargo_command, run_command, area_command
from .profile_reporting import run_profile_with_report
from .compiler_configuration_plan import configuration_plan
from .native_test_execution import NATIVE_SUITES, run_native_configuration
from .corpus_ownership import corpus_owner_selected
from .profiles import crate_test_mode, crate_test_suites_for_mode, load_profile, resolve_fixture_manifest
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
    cause: str | None = None


CRITICAL_RESULT_SLUGS = {
    "rust_interop": "rust-interop",
    "developer_tooling": "developer-tooling",
    "documentation": "documentation",
    "distribution_release": "distribution-release",
}


def now_ms() -> int:
    return time.monotonic_ns() // 1_000_000


def timed_step(name: str, callback: Callable[[], None]) -> StepResult:
    start_ms = now_ms()
    status = 0
    cause = None
    try:
        callback()
    except CommandFailed as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        status = exc.returncode
        cause = exc.cause
    except (VerificationError, AreaResultError) as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        status = 2
    elapsed_ms = now_ms() - start_ms
    label = "pass" if status == 0 else "fail"
    print(f"[sifr-lane-step] name={name} elapsed_ms={elapsed_ms} status={label}")
    return StepResult(status=status, elapsed_ms=elapsed_ms, cause=cause)


def step_name(kind: str, name: str) -> str:
    return f"{kind}_{name.replace('-', '_')}"


class ProfileRunner:
    """Run one profile from selected areas, guardrails, and toolchain steps."""

    def __init__(self, profile_name: str, forward_args: list[str], *, correctness_only: bool = False) -> None:
        self.profile = select_execution_scope(load_profile(profile_name), correctness_only)
        self.correctness_only = correctness_only
        self.evidence_name = evidence_name(profile_name, correctness_only)
        self.profile_name = str(self.profile["name"])
        self.no_fail_fast = "--no-fail-fast" in forward_args
        self.forward_args = [arg for arg in forward_args if arg != "--no-fail-fast"]
        self.preparation = None
        self.completed_steps = set()
        self.stop_requested = False
        self.functional_exit_status = 0
        self.performance_exit_status = 0
        self.env = os.environ.copy()
        overrides, self.forward_args = extract_overrides(self.forward_args)
        self.build_allocation = allocate(self.profile, self.env, overrides)
        self.env[ALLOCATION_ENV] = json.dumps(asdict(allocate_native(self.build_allocation)))
        self.env["SIFR_VERIFY_PROCESS_SLOTS"] = str(self.build_allocation.slot_limit)
        self.env["CARGO_BUILD_JOBS"] = str(self.build_allocation.compiler_jobs)
        self.env["RAYON_NUM_THREADS"] = str(self.build_allocation.sifr_jobs)
        for field, variable in E2E_ENV.items():
            self.env[variable] = str(getattr(self.build_allocation, field))
        target_root = Path(self.env.get("CARGO_TARGET_DIR", REPO_ROOT / "target"))
        if not target_root.is_absolute():
            target_root = REPO_ROOT / target_root
        configured_sifr_binary = self.env.get("SIFR_GCQ_BIN") or self.env.get("SIFR_RUNTIME_PLATFORM_BIN")
        sifr_binary = (
            Path(configured_sifr_binary)
            if configured_sifr_binary
            else target_root / "debug" / "sifr"
        )
        self.env.setdefault("SIFR_GCQ_BIN", str(sifr_binary))
        self.env.setdefault("SIFR_RUNTIME_PLATFORM_BIN", str(sifr_binary))
        probe_cache_root = REPO_ROOT / "target" / "sifr_rust_bridge_probe_cache" / self.evidence_name
        self.env["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)

    def run(self) -> int:
        self.print_header()
        if any(area["area"] == "performance" and set(area["suites"]).intersection(
                {"rules", "smoke", "representative", "full"})
               for area in self.profile["selected_areas"]):
            admission = self.execute_step("performance_reference_admission", self.admit_performance_reference)
            if admission:
                self.block_steps("performance_reference_admission")
                return admission
        early = {"hir-maintainability", "file-size", "source-crate-dependency-direction",
                 "submodule-ownership", "stdlib-manifest-schema", "dx-metadata-inventory"}
        failed = 0
        for guardrail in self.profile["guardrail_steps"]:
            if guardrail in early:
                status = self.execute_step(step_name("guardrail", guardrail),
                                           lambda g=guardrail: self.run_guardrail(g))
                failed = failed or status
                if self.stop_requested or (status and not self.no_fail_fast):
                    break
        if failed:
            self.block_steps("invalid-inventory")
            return failed
        prepared = self.execute_step("cargo_cache_setup", self.prepare_cargo_cache)
        if prepared and (not self.no_fail_fast or self.stop_requested or
                         self.preparation is None or self.preparation.fatal_reason):
            self.block_steps(self.preparation.fatal_reason if self.preparation and
                             self.preparation.fatal_reason else "cargo_cache_setup")
            return prepared
        # Independent consumers still run offline after partial preparation.
        if self.profile.get("cargo_policy", {}).get("offline") is True:
            enable_profile_offline_cargo(self.env)
        failed = prepared
        for name, callback in self.selected_steps():
            if name in self.completed_steps:
                continue
            prerequisite = self.preparation.blocked_by(name) if self.preparation else ""
            if name in {"toolchain_e2e_report_determinism", "toolchain_e2e_sequential_parallel_equivalence"}:
                if getattr(self, "e2e_failed", False):
                    prerequisite = "toolchain_e2e_pass"
            if prerequisite:
                self.block_step(name, prerequisite)
                failed = failed or 2
                self.functional_exit_status = self.functional_exit_status or 2
                if name == "toolchain_e2e_pass":
                    self.e2e_failed = True
                continue
            status = self.execute_step(name, callback)
            if name == "toolchain_e2e_pass":
                self.e2e_failed = bool(status)
            failed = failed or status
            if self.stop_requested or (status and not self.no_fail_fast):
                self.block_steps(name)
                return failed
        return failed

    def selected_steps(self):
        steps = [(step_name("guardrail", g), lambda g=g: self.run_guardrail(g))
                 for g in self.profile["guardrail_steps"]]
        steps += [(step_name("area", a["area"]),
                   lambda a=a: self.run_area(str(a["area"]), [str(s) for s in a["suites"]]))
                  for a in self.profile["selected_areas"]]
        steps += [(step_name("toolchain", t), lambda t=t: self.run_toolchain_step(t))
                  for t in self.profile["toolchain_steps"]]
        return steps

    def block_step(self, name: str, prerequisite: str) -> None:
        self.completed_steps.add(name)
        print(f"[sifr-lane-step] name={name} elapsed_ms=0 status=blocked")
        print(f"blocked {name}: prerequisite {prerequisite}")

    def block_steps(self, prerequisite: str) -> None:
        for name in ["cargo_cache_setup", *(name for name, _ in self.selected_steps())]:
            if name not in self.completed_steps:
                self.block_step(name, prerequisite)

    def execute_step(self, name: str, callback: Callable[[], None]) -> int:
        self.completed_steps.add(name)
        budget = self.prepare_step_budget(name)
        step_seconds = self.env.get("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS")
        if step_seconds is None:
            result = timed_step(name, callback)
        else:
            _, deadline = deadline_environment(self.env, step_seconds)
            previous_deadline = self.env.get(SAFETY_DEADLINE_ENV)
            previous_process_deadline = os.environ.get(SAFETY_DEADLINE_ENV)
            self.env[SAFETY_DEADLINE_ENV] = repr(deadline)
            # Some setup adapters copy os.environ directly. Carry the same
            # bounded step deadline through those commands as well.
            os.environ[SAFETY_DEADLINE_ENV] = repr(deadline)
            try:
                result = timed_step(name, callback)
            finally:
                if previous_deadline is None:
                    self.env.pop(SAFETY_DEADLINE_ENV, None)
                else:
                    self.env[SAFETY_DEADLINE_ENV] = previous_deadline
                if previous_process_deadline is None:
                    os.environ.pop(SAFETY_DEADLINE_ENV, None)
                else:
                    os.environ[SAFETY_DEADLINE_ENV] = previous_process_deadline
        if result.cause not in (None, "exit") or result.status < 0:
            self.stop_requested = True
        if result.status != 0:
            self.functional_exit_status = result.status
            return result.status
        budget_status = enforce_prepared_step_budget(budget, result.elapsed_ms)
        if budget_status != 0:
            self.performance_exit_status = budget_status
        if budget_status == 0:
            record_step_success(budget, result.elapsed_ms)
        return budget_status

    def admit_performance_reference(self) -> None:
        script = REPO_ROOT / "verification/areas/performance/reference_admission.py"
        run_command([sys.executable, str(script)], env=self.env)

    def prepare_compiler_identity(self):
        binary = resolve_sifr_binary(REPO_ROOT, env=self.env)
        for variable in ("SIFR_GCQ_BIN", "SIFR_RUNTIME_PLATFORM_BIN"):
            if os.environ.get(variable):
                resolve_sifr_binary(REPO_ROOT, explicit_env_var=variable, env=self.env)
            self.env[variable] = str(binary)

    def prepare_cargo_cache(self) -> None:
        try:
            nodes, consumers = cargo_preparation_plan(
                self.profile, self.env, run_command, identity=self.prepare_compiler_identity)
            self.preparation = PreparationRun(nodes, consumers, no_fail_fast=self.no_fail_fast)
            self.preparation.run()
        except (ValueError, RuntimeError, OSError) as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def prepare_step_budget(self, name: str) -> StepBudgetContext | None:
        return prepare_step_budget(
            repo_root=REPO_ROOT,
            profile=self.profile,
            profile_name=self.evidence_name,
            name=name,
            env=self.env,
        )

    def print_header(self) -> None:
        print(self.build_allocation.report())
        budgets = self.profile["budgets"]
        policy = self.profile["resource_policy"]
        print("Running local-first validation")
        print(f"  profile={self.profile_name}")
        print(f"  lane={self.evidence_name}")
        print(f"  execution_scope={scope_name(self.correctness_only)}")
        print(
            f"  budget=warm<={budgets['warm_wall_time_minutes']}m "
            f"cold<={budgets['cold_wall_time_minutes']}m"
        )
        print(
            f"  policy=thermal:{policy['thermal_policy']} "
            f"memory:{policy['memory_policy']}"
        )

    def run_guardrail(self, guardrail: str) -> None:
        if guardrail == "hir-maintainability":
            self.run_python("scripts/check_hir_maintainability_guardrails.py")
        elif guardrail == "method-dispatch-authority":
            self.run_script_with_self_test("scripts/check_method_dispatch_authority.py")
        elif guardrail == "unsafe-abi-contracts":
            self.run_script_with_self_test("scripts/check_unsafe_abi_contracts.py")
        elif guardrail == "file-size":
            self.run_python("scripts/check_file_size_guardrails.py")
        elif guardrail == "demo-emitted-freshness":
            self.run_python("scripts/check_demo_emitted_freshness.py")
        elif guardrail == "source-crate-dependency-direction":
            self.run_script_with_self_test("scripts/check_source_crate_dependency_direction.py")
        elif guardrail == "submodule-ownership":
            self.run_script_with_self_test("scripts/check_submodule_ownership.py")
        elif guardrail == "sysroot-resource-certification":
            self.run_script_with_self_test("scripts/check_sysroot_stdlib_resource_certification_gate.py")
        elif guardrail == "stdlib-native-intrinsic-allowlist":
            self.run_script_with_self_test("scripts/check_stdlib_native_intrinsic_allowlist.py")
        elif guardrail == "stdlib-native-adapter-reachability":
            self.run_script_with_self_test("scripts/check_stdlib_native_adapter_reachability.py")
        elif guardrail == "stdlib-manifest-schema":
            self.run_script_with_self_test("scripts/check_stdlib_manifest_schema.py")
        elif guardrail == "dx-metadata-inventory":
            self.run_script_with_self_test("scripts/check_dx_metadata_inventory.py")
        elif guardrail == "stdlib-bootstrap-ordering":
            self.run_script_with_self_test("scripts/check_stdlib_bootstrap_ordering.py")
        elif guardrail == "driver-maintainability":
            self.run_python("scripts/check_sifr_driver_maintainability_guardrails.py")
        elif guardrail == "verification-hardening-self-test":
            run_command([sys.executable, "-m", "sifr_verify.hardening", "--self-test"], env=self.env)
        elif guardrail == "verification-runner-foundation":
            run_command(["uv", "lock", "--project", "verification", "--check"], env=self.env)
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
                ],
                env=self.env,
            )
        elif guardrail == "hardening-determinism-scale":
            run_command(
                [
                    sys.executable,
                    "-m",
                    "sifr_verify.hardening",
                    "--profile",
                    self.profile_name,
                    "--suite",
                    "determinism-scale",
                ],
                env=self.env,
            )
        else:
            raise ProfileRunnerError(f"unsupported guardrail step: {guardrail}")

    def run_python(self, path: str, *args: str) -> None:
        run_command(["python3", path, *args], env=self.env)

    def run_script_with_self_test(self, path: str) -> None:
        self.run_python(path)
        self.run_python(path, "--self-test")

    def run_area(self, area: str, suites: list[str]) -> None:
        if area == "python_interop":
            run_segmented_python_interop(
                suites=suites, profile_name=self.evidence_name,
                command_runner=lambda command: run_command(command, env=self.env),
            )
            return
        result_slug = CRITICAL_RESULT_SLUGS.get(area, area.replace("_", "-"))
        strict_freshness = requires_strict_freshness(self.profile, area)
        result_path = run_selected_area(
            area=area,
            suites=suites,
            profile_name=self.evidence_name,
            result_slug=result_slug,
            command_builder=lambda *args: area_command(
                *args, *(["--no-fail-fast"] if self.no_fail_fast else []),
                *([STRICT_FLAG] if strict_freshness else [])),
            command_runner=lambda command: run_command(command, env=self.env),
        )
        if strict_freshness:
            validate_strict_freshness_evidence(json.loads(result_path.read_text()))

    def run_toolchain_step(self, toolchain_step: str) -> None:
        if toolchain_step == "cargo-build-release":
            run_command(["python3", "-m", "sifr_verify.metadata_setup", "--",
                         *cargo_command("build", "--release")], env=self.env)
        elif toolchain_step == "cargo-fmt-check":
            run_command(["cargo", "fmt", "--check"], env=self.env)
        elif toolchain_step == "cargo-clippy-workspace":
            run_command(cargo_command("clippy", "--workspace", "--", "-D", "warnings"), env=self.env)
        elif toolchain_step in {"cargo-test-sifr-smoke", "cargo-test-sifr-full"}:
            mode = crate_test_mode(self.profile)
            if mode is None:
                raise ProfileRunnerError("crate-test toolchain step has no canonical mode")
            self.run_crate_tests(mode)
        elif toolchain_step == "cargo-test-workspace":
            run_command(cargo_command("test", "--workspace"), env=self.env)
        elif toolchain_step == "e2e-pass":
            self.run_e2e_pass_suite()
        elif toolchain_step == "e2e-report-determinism":
            run_command(
                [
                    "bash",
                    "verification/runner/e2e/check_report_determinism.sh",
                    "--profile",
                    self.profile_name,
                ],
                env=self.env,
            )
        elif toolchain_step == "e2e-sequential-parallel-equivalence":
            run_command(
                [
                    "bash",
                    "verification/runner/e2e/check_sequential_parallel_equivalence.sh",
                    "--profile",
                    self.profile_name,
                ],
                env=self.env,
            )
        else:
            raise ProfileRunnerError(f"unsupported toolchain step: {toolchain_step}")

    def run_crate_tests(self, mode: str) -> None:
        first_failure = None
        for configuration in configuration_plan(self.profile, mode):
            suite_id = ",".join(configuration.ids)
            command = configuration.execution()
            start_ms = now_ms()
            case_status = "pass"
            try:
                if any(suite in NATIVE_SUITES for suite in configuration.ids):
                    run_native_configuration(
                        configuration, env=self.env, no_fail_fast=self.no_fail_fast,
                        corpus_owner_selected=corpus_owner_selected(self.profile),
                        preparation_complete=(self.preparation is not None
                            and not self.preparation.fatal_reason
                            and not self.preparation.missing(("crate_tests",))),
                    )
                else:
                    if self.no_fail_fast:
                        command.insert(2, "--no-fail-fast")
                    run_command(command, env=self.env)
            except (CommandFailed, VerificationError) as error:
                case_status = "fail"
                if not self.no_fail_fast or (
                    isinstance(error, CommandFailed)
                    and (error.cause not in (None, "exit") or error.returncode < 0)
                ):
                    raise
                first_failure = first_failure or error
            finally:
                elapsed_ms = now_ms() - start_ms
                print(
                    f"[sifr-case-timing] bucket=crate_tests case={suite_id} "
                    f"elapsed_ms={elapsed_ms} status={case_status}"
                )

        if first_failure is not None:
            raise first_failure

    def run_e2e_pass_suite(self) -> None:
        e2e = self.profile["e2e"]
        args = [
            "--profile",
            self.profile_name,
            "--sifr-jobs",
            str(self.build_allocation.sifr_jobs),
            "--rust-jobs",
            str(self.build_allocation.rust_jobs),
            "--run-jobs",
            str(self.build_allocation.run_jobs),
            "--cargo-build-jobs",
            str(self.build_allocation.cargo_build_jobs),
            "--max-group-fixtures",
            str(e2e["max_group_fixtures"]),
        ]
        fixture_manifest = resolve_fixture_manifest(str(e2e.get("fixture_manifest", "")))
        if fixture_manifest is not None:
            args.extend(["--fixture-manifest", str(fixture_manifest)])
        if bool(e2e["disable_cache"]):
            args.append("--no-cache")
        run_command(
            ["bash", "verification/runner/e2e/run_e2e_pass.sh", *args, *self.forward_args],
            env=self.env,
        )


def run_profile(
    profile_name: str,
    forward_args: list[str],
    *,
    release_report_out: str | None = None,
    correctness_only: bool = False,
) -> int:
    if correctness_only and release_report_out is not None:
        raise ProfileRunnerError("correctness-only execution cannot produce release evidence")
    runner = ProfileRunner(profile_name, forward_args, correctness_only=correctness_only)
    return run_profile_with_report(
        profile_name,
        runner.run,
        execution_outcomes=lambda: {
            "functional_exit_status": runner.functional_exit_status,
            "performance_exit_status": runner.performance_exit_status,
        },
        handled_error=ProfileRunnerError,
        release_report_out=release_report_out,
        correctness_only=correctness_only,
    )

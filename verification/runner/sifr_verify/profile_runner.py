"""Execute validation profiles from their canonical selections."""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Callable

from .cargo_setup import (
    enable_offline_cargo as enable_profile_offline_cargo,
    prepare_cargo_cache as prepare_profile_cargo_cache,
)
from .assertion_resource_forecast import assertion_allocation
from .early_sql import EarlySqlOutcome
from .errors import VerificationError
from .paths import REPO_ROOT
from .profile_area_steps import AreaResultError, run_selected_area, run_segmented_python_interop
from .process_execution import SAFETY_DEADLINE_ENV, deadline_environment
from .profile_commands import CommandFailed, cargo_command, run_command, uv_area_command
from .profile_reporting import run_profile_with_report
from .compiler_configuration_plan import configuration_plan
from .native_test_execution import NATIVE_SUITES, run_native_configuration
from .performance_partition import MEASUREMENT_SUITES, combine as combine_performance, result_path as performance_result_path
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


CRITICAL_RESULT_SLUGS = {
    "rust_interop": "rust-interop",
    "developer_tooling": "developer-tooling",
    "documentation": "documentation",
    "distribution_release": "distribution-release",
}


def now_ms() -> int:
    return time.monotonic_ns() // 1_000_000


def timed_step(name: str, callback: Callable[[], None], *, prior_elapsed_ms: int = 0) -> StepResult:
    start_ms = now_ms() - prior_elapsed_ms
    status = 0
    try:
        callback()
    except CommandFailed as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        status = exc.returncode
    except (VerificationError, AreaResultError) as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
        status = 2
    except OSError as exc:
        print(f"sifr_verify: infrastructure failure: {exc}", file=sys.stderr)
        status = 2
    elapsed_ms = now_ms() - start_ms
    label = "pass" if status == 0 else "fail"
    print(f"[sifr-lane-step] name={name} elapsed_ms={elapsed_ms} status={label}")
    return StepResult(status=status, elapsed_ms=elapsed_ms)


def step_name(kind: str, name: str) -> str:
    return f"{kind}_{name.replace('-', '_')}"


class ProfileRunner:
    """Run one profile from selected areas, guardrails, and toolchain steps."""

    def __init__(self, profile_name: str, forward_args: list[str]) -> None:
        self.profile = load_profile(profile_name)
        self.profile_name = str(self.profile["name"])
        self.require_performance = "--require-performance" in forward_args
        if self.require_performance and self.profile_name != "cloud":
            raise ProfileRunnerError("--require-performance is specific to the cloud profile")
        forward_args = [arg for arg in forward_args if arg != "--require-performance"]
        self.compact_resources = "--compact-resources" in forward_args
        forward_args = [arg for arg in forward_args if arg != "--compact-resources"]
        if self.compact_resources and self.profile_name not in {"create-pr", "merge", "nightly", "cloud"}:
            raise ProfileRunnerError("compact resources require a source validation profile")
        self.no_fail_fast = "--no-fail-fast" in forward_args
        self.forward_args = [arg for arg in forward_args if arg != "--no-fail-fast"]
        self.functional_exit_status = 0
        self.performance_exit_status = 0
        self.env = os.environ.copy()
        if "SIFR_VERIFY_SYSROOT_GRAPH_SESSION" in self.env:
            raise ProfileRunnerError("sysroot graph sessions are assigned by the owned scheduler")
        inherited_policy = self.env.get("SIFR_VERIFY_RESOURCE_POLICY")
        if "SIFR_VERIFY_RESOURCE_POLICY" in self.env and (
                not self.compact_resources or inherited_policy != "compact"):
            raise ProfileRunnerError("resource policy requires its explicit compact resource option")
        if self.compact_resources:
            for key, value in (("CARGO_PROFILE_DEV_DEBUG", "0"), ("CARGO_INCREMENTAL", "0")):
                if self.env.get(key) not in (None, value):
                    raise ProfileRunnerError("compact resource compiler configuration conflicts with " + key)
                self.env[key] = value
            self.env["SIFR_VERIFY_RESOURCE_POLICY"] = "compact"
        self.env["SIFR_VALIDATION_PROFILE"] = self.profile_name
        if self.profile_name == "cloud":
            # An unrelated physical-host reference must not affect correctness.
            self.env.pop("SIFR_PERFORMANCE_REFERENCE", None)
        self.env["CARGO_BUILD_JOBS"] = str(self.profile["e2e"]["cargo_build_jobs"])
        self.env["RAYON_NUM_THREADS"] = str(self.profile["resource_policy"]["max_parallel"])
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
        probe_cache_root = REPO_ROOT / "target" / "sifr_rust_bridge_probe_cache" / self.profile_name
        self.env["SIFR_RUST_BRIDGE_PROBE_CACHE_DIR"] = str(probe_cache_root)

    def run(self) -> int:
        self.print_header()
        performance_elapsed_ms = 0
        if self.profile_name != "cloud":
            performance_result_path(self.profile_name).unlink(missing_ok=True)
        early = {"hir-maintainability", "file-size", "source-crate-dependency-direction",
                 "submodule-ownership", "stdlib-manifest-schema"}
        failed = 0
        for guardrail in self.profile["guardrail_steps"]:
            if guardrail not in early:
                continue
            status = self.execute_assertion(step_name("guardrail", guardrail),
                                            lambda g=guardrail: self.run_guardrail(g))
            failed = failed or status
            if status and not self.no_fail_fast:
                return failed
        # Invalid inventories are unsafe preparation inputs.
        if failed:
            self.block_steps("invalid-inventory")
            return failed
        if self.profile_name == "cloud":
            from .cloud_schedule import run_staged_cloud
            return run_staged_cloud(self, early)
        if self.compact_resources:
            from .compact_profile import prepare_compact
            prepared = prepare_compact(self)
        else:
            prepared = self.execute_step("cargo_cache_setup", self.prepare_cargo_cache)
        if prepared and not self.no_fail_fast:
            return prepared
        if not prepared and self.profile.get("cargo_policy", {}).get("offline") is True:
            enable_profile_offline_cargo(self.env)
        early_sql = getattr(self, "early_sql_outcome", EarlySqlOutcome())
        failed = prepared or early_sql.status
        for guardrail in self.profile["guardrail_steps"]:
            if guardrail in early:
                continue
            if prepared:
                self.block_step(step_name("guardrail", guardrail), "cargo_cache_setup")
                continue
            status = self.execute_assertion(step_name("guardrail", guardrail),
                                            lambda g=guardrail: self.run_guardrail(g))
            failed = failed or status
            if status and not self.no_fail_fast:
                return failed
        for selection in self.profile["selected_areas"]:
            area = str(selection["area"])
            suites = [str(suite) for suite in selection["suites"]]
            if (area in getattr(self, "compact_completed_areas", set())
                    or (area == "sql_platform" and early_sql.selected)):
                continue
            if area == "performance":
                suites = [suite for suite in suites if suite not in MEASUREMENT_SUITES]
                if suites and not prepared:
                    started = now_ms()
                    status = self.execute_assertion("area_performance_correctness",
                        lambda s=suites: self.run_performance_part(s, "correctness"))
                    performance_elapsed_ms += now_ms() - started
                    failed = failed or status
                    if status and not self.no_fail_fast:
                        return failed
                elif suites:
                    self.block_step("area_performance_correctness", "cargo_cache_setup")
                continue
            if prepared:
                self.block_step(step_name("area", area), "cargo_cache_setup")
                continue
            status = self.execute_assertion(step_name("area", area),
                                       lambda a=area, s=suites: self.run_area(a, s))
            failed = failed or status
            if status and not self.no_fail_fast:
                return failed
        failed_toolchain = set()
        for toolchain_step in self.profile["toolchain_steps"]:
            if toolchain_step in {"e2e-report-determinism", "e2e-sequential-parallel-equivalence"} and "e2e-pass" in failed_toolchain:
                self.block_step(step_name("toolchain", toolchain_step), "e2e-pass")
                continue
            if prepared:
                self.block_step(step_name("toolchain", toolchain_step), "cargo_cache_setup")
                continue
            status = self.execute_assertion(step_name("toolchain", toolchain_step),
                                       lambda t=toolchain_step: self.run_toolchain_step(t))
            failed = failed or status
            if status:
                failed_toolchain.add(toolchain_step)
            if status and not self.no_fail_fast:
                return failed
        # A host-sensitive measurement must not suppress correctness, including
        # toolchain checks that follow the performance area in the inventory.
        for selection in self.profile["selected_areas"]:
            if selection["area"] != "performance":
                continue
            if prepared:
                self.block_step("area_performance", "cargo_cache_setup")
                continue
            suites = list(selection["suites"])
            measured = [suite for suite in suites if suite in MEASUREMENT_SUITES]
            if measured:
                admission = self.execute_step("performance_reference_admission",
                    self.admit_performance_reference, performance=True)
                if admission:
                    self.block_step("area_performance", "performance_reference_admission")
                    continue
            def finish(selected=suites, measurements=measured):
                if measurements:
                    self.run_performance_part(measurements, "measurement")
                combine_performance(self.profile_name, selected)
            self.execute_step("area_performance", finish, performance=bool(measured),
                              prior_elapsed_ms=performance_elapsed_ms)
        return failed or self.performance_exit_status

    def execute_assertion(self, name, callback):
        schedule = getattr(self, 'compact_schedule', None)
        if schedule is None:
            return self.execute_step(name,callback)
        return schedule.step(name, callback, allocation=assertion_allocation(name, self.profile),
                             monitor_disk=True)

    def block_step(self, name: str, prerequisite: str) -> None:
        print(f"[sifr-lane-step] name={name} elapsed_ms=0 status=blocked")
        print(f"blocked {name}: prerequisite {prerequisite}")

    def block_steps(self, prerequisite: str, *, handled_areas=(), include_setup=True) -> None:
        if include_setup:
            self.block_step("cargo_cache_setup", prerequisite)
        for selection in self.profile["selected_areas"]:
            if selection["area"] not in handled_areas:
                self.block_step(step_name("area", str(selection["area"])), prerequisite)
        for name in self.profile["toolchain_steps"]:
            self.block_step(step_name("toolchain", name), prerequisite)

    def execute_step(self, name: str, callback: Callable[[], None], *, performance: bool = False,
                     prior_elapsed_ms: int = 0) -> int:
        budget = self.prepare_step_budget(name)
        step_seconds = self.env.get("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS")
        if step_seconds is None:
            result = (timed_step(name, callback, prior_elapsed_ms=prior_elapsed_ms)
                      if prior_elapsed_ms else timed_step(name, callback))
        else:
            _, deadline = deadline_environment(self.env, step_seconds)
            previous_deadline = self.env.get(SAFETY_DEADLINE_ENV)
            previous_process_deadline = os.environ.get(SAFETY_DEADLINE_ENV)
            self.env[SAFETY_DEADLINE_ENV] = repr(deadline)
            # Some setup adapters copy os.environ directly. Carry the same
            # bounded step deadline through those commands as well.
            os.environ[SAFETY_DEADLINE_ENV] = repr(deadline)
            try:
                result = (timed_step(name, callback, prior_elapsed_ms=prior_elapsed_ms)
                          if prior_elapsed_ms else timed_step(name, callback))
            finally:
                if previous_deadline is None:
                    self.env.pop(SAFETY_DEADLINE_ENV, None)
                else:
                    self.env[SAFETY_DEADLINE_ENV] = previous_deadline
                if previous_process_deadline is None:
                    os.environ.pop(SAFETY_DEADLINE_ENV, None)
                else:
                    os.environ[SAFETY_DEADLINE_ENV] = previous_process_deadline
        if result.status != 0:
            if performance:
                self.performance_exit_status = result.status
            else:
                self.functional_exit_status = result.status
            return result.status
        if self.profile_name == "cloud" and budget is not None:
            # Preserve the recorded budget and safety deadline. Scheduling
            # uncertainty cannot fail or suppress later correctness checks.
            # These are diagnostic step timings, not the paired benchmark
            # contract. Only a checked cloud receipt qualifies performance.
            budget = replace(budget, enforcement="advisory")
        budget_status = enforce_prepared_step_budget(budget, result.elapsed_ms)
        if budget_status != 0:
            self.performance_exit_status = budget_status
        if budget_status == 0:
            record_step_success(budget, result.elapsed_ms)
        # Retain a blocking timing verdict for the final gate, while allowing
        # subsequent selected correctness assertions to execute.
        return 0

    def admit_performance_reference(self) -> None:
        script = REPO_ROOT / "verification/areas/performance/reference_admission.py"
        run_command([sys.executable, str(script)], env=self.env)

    def prepare_cargo_cache(self) -> None:
        try:
            prepare_profile_cargo_cache(self.profile, self.env, run_command)
            # Overrides must identify the same Cargo-prepared candidate.
            binary = resolve_sifr_binary(REPO_ROOT, env=self.env)
            for variable in ("SIFR_GCQ_BIN", "SIFR_RUNTIME_PLATFORM_BIN"):
                if os.environ.get(variable):
                    resolve_sifr_binary(REPO_ROOT, explicit_env_var=variable, env=self.env)
                self.env[variable] = str(binary)
        except (ValueError, RuntimeError, OSError) as exc:
            raise ProfileRunnerError(str(exc)) from exc

    def prepare_step_budget(self, name: str) -> StepBudgetContext | None:
        return prepare_step_budget(
            repo_root=REPO_ROOT,
            profile=self.profile,
            profile_name=self.profile_name,
            name=name,
            env=self.env,
        )

    def print_header(self) -> None:
        budgets = self.profile["budgets"]
        policy = self.profile["resource_policy"]
        print("Running local-first validation")
        print(f"  profile={self.profile_name}")
        print(f"  lane={self.profile_name}")
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
            if self.profile_name == "cloud":
                from .checkpoint_recipes import run_guard
                run_guard(guardrail, env=self.env)
            else:
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

    def run_area(self, area: str, suites: list[str], *, result_slug: str | None = None) -> None:
        if area == "python_interop":
            run_segmented_python_interop(
                suites=suites, profile_name=self.profile_name,
                command_runner=lambda command: run_command(command, env=self.env),
            )
            return
        result_slug = result_slug or CRITICAL_RESULT_SLUGS.get(area, area.replace("_", "-"))
        if area == "performance" and getattr(self, "performance_result_phase", None):
            result_slug += "-" + self.performance_result_phase
        run_selected_area(
            area=area,
            suites=suites,
            profile_name=self.profile_name,
            result_slug=result_slug,
            command_builder=lambda *args: uv_area_command(
                *args, *(["--no-fail-fast"] if self.no_fail_fast else [])),
            command_runner=lambda command: run_command(command, env=self.env),
        )

    def run_performance_part(self, suites: list[str], phase: str) -> None:
        self.performance_result_phase = phase
        try:
            self.run_area("performance", suites)
        finally:
            del self.performance_result_phase

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
        for configuration in configuration_plan(self.profile, mode):
            suite_id = ",".join(configuration.ids)
            command = configuration.execution()
            start_ms = now_ms()
            case_status = "pass"
            try:
                if any(suite in NATIVE_SUITES for suite in configuration.ids):
                    run_native_configuration(configuration, env=self.env, no_fail_fast=self.no_fail_fast)
                else:
                    run_command(command, env=self.env)
            except (CommandFailed, VerificationError):
                case_status = "fail"
                raise
            finally:
                elapsed_ms = now_ms() - start_ms
                print(
                    f"[sifr-case-timing] bucket=crate_tests case={suite_id} "
                    f"elapsed_ms={elapsed_ms} status={case_status}"
                )

    def run_e2e_pass_suite(self) -> None:
        e2e = self.profile["e2e"] | getattr(self, "e2e_worker_limits", {})
        args = [
            "--profile",
            self.profile_name,
            "--sifr-jobs",
            str(e2e["sifr_jobs"]),
            "--rust-jobs",
            str(e2e["rust_jobs"]),
            "--run-jobs",
            str(e2e["run_jobs"]),
            "--cargo-build-jobs",
            str(e2e["cargo_build_jobs"]),
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
) -> int:
    runner = ProfileRunner(profile_name, forward_args)
    run_lane = runner.run
    if profile_name == "cloud":
        from .cloud_profile import run_cloud_profile
        run_lane = lambda: run_cloud_profile(runner)
    return run_profile_with_report(
        profile_name,
        run_lane,
        execution_outcomes=lambda: {
            "functional_exit_status": runner.functional_exit_status,
            "performance_exit_status": runner.performance_exit_status,
            "worker_limits": getattr(runner, "e2e_worker_limits", None),
        },
        handled_error=ProfileRunnerError,
        release_report_out=release_report_out,
    )

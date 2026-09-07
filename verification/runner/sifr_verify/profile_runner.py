"""Execute validation profiles from their canonical selections."""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .cargo_setup import (
    enable_offline_cargo as enable_profile_offline_cargo,
    prepare_cargo_cache as prepare_profile_cargo_cache,
)
from .errors import VerificationError
from .paths import REPO_ROOT
from .profile_area_steps import AreaResultError, run_selected_area
from .profile_commands import CommandFailed, cargo_command, run_command, uv_area_command
from .profile_reporting import run_profile_with_report
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


def timed_step(name: str, callback: Callable[[], None]) -> StepResult:
    start_ms = now_ms()
    status = 0
    try:
        callback()
    except CommandFailed as exc:
        status = exc.returncode
    except (ProfileRunnerError, AreaResultError) as exc:
        print(f"sifr_verify: {exc}", file=sys.stderr)
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
        self.forward_args = forward_args
        self.env = os.environ.copy()
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
        status = self.execute_step("cargo_cache_setup", self.prepare_cargo_cache)
        if status != 0:
            return status
        if self.profile.get("cargo_policy", {}).get("offline") is True:
            enable_profile_offline_cargo(self.env)

        for guardrail in self.profile["guardrail_steps"]:
            status = self.execute_step(
                step_name("guardrail", guardrail),
                lambda guardrail=guardrail: self.run_guardrail(guardrail),
            )
            if status != 0:
                return status

        for selection in self.profile["selected_areas"]:
            area = str(selection["area"])
            suites = [str(suite) for suite in selection["suites"]]
            status = self.execute_step(
                step_name("area", area),
                lambda area=area, suites=suites: self.run_area(area, suites),
            )
            if status != 0:
                return status

        for toolchain_step in self.profile["toolchain_steps"]:
            status = self.execute_step(
                step_name("toolchain", toolchain_step),
                lambda toolchain_step=toolchain_step: self.run_toolchain_step(toolchain_step),
            )
            if status != 0:
                return status
        return 0

    def execute_step(self, name: str, callback: Callable[[], None]) -> int:
        budget = self.prepare_step_budget(name)
        result = timed_step(name, callback)
        if result.status != 0:
            return result.status
        budget_status = enforce_prepared_step_budget(budget, result.elapsed_ms)
        if budget_status == 0:
            record_step_success(budget)
        return budget_status

    def prepare_cargo_cache(self) -> None:
        try:
            prepare_profile_cargo_cache(self.profile, self.env, run_command)
            if not (os.environ.get("SIFR_GCQ_BIN") or os.environ.get("SIFR_RUNTIME_PLATFORM_BIN")):
                # Resolve/build only after the workspace setup has succeeded.
                binary = resolve_sifr_binary(REPO_ROOT)
                self.env["SIFR_GCQ_BIN"] = str(binary)
                self.env["SIFR_RUNTIME_PLATFORM_BIN"] = str(binary)
        except ValueError as exc:
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
            self.run_python("scripts/check_hir_maintainability_guardrails.py")
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

    def run_area(self, area: str, suites: list[str]) -> None:
        result_slug = CRITICAL_RESULT_SLUGS.get(area, area.replace("_", "-"))
        run_selected_area(
            area=area,
            suites=suites,
            profile_name=self.profile_name,
            result_slug=result_slug,
            command_builder=uv_area_command,
            command_runner=lambda command: run_command(command, env=self.env),
        )

    def run_toolchain_step(self, toolchain_step: str) -> None:
        if toolchain_step == "cargo-build-release":
            run_command(cargo_command("build", "--release"), env=self.env)
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
        for suite in crate_test_suites_for_mode(self.profile, mode):
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
            start_ms = now_ms()
            case_status = "pass"
            try:
                run_command(cargo_command(*command), env=self.env)
            except CommandFailed:
                case_status = "fail"
                raise
            finally:
                elapsed_ms = now_ms() - start_ms
                print(
                    f"[sifr-case-timing] bucket=crate_tests case={suite_id} "
                    f"elapsed_ms={elapsed_ms} status={case_status}"
                )

    def run_e2e_pass_suite(self) -> None:
        e2e = self.profile["e2e"]
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
    return run_profile_with_report(
        profile_name,
        lambda: ProfileRunner(profile_name, forward_args).run(),
        handled_error=ProfileRunnerError,
        release_report_out=release_report_out,
    )

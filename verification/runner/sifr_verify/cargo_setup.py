"""Resolve the canonical Cargo cache preparation for validation profiles."""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

from .paths import REPO_ROOT
from .cargo_fixture_setup import prepare_locked_fixture_caches
from .cargo_crate_setup import prepare_crate_test_binaries
from .area_cargo_setup import prepare_area_graphs
from .python_preparation import python_consumers, prepare_python_environment

from verification.areas.common.release_compiler import (
    INSTALLED_PACKAGE_SUITES, release_compiler_owners,
)

CANONICAL_SETUP_COMMAND = "cargo fetch --locked"


def cargo_setup_command(profile: dict[str, Any]) -> list[str]:
    """Return the one supported profile cache-setup command."""
    policy = profile.get("cargo_policy")
    if not isinstance(policy, dict):
        raise ValueError("profile cargo_policy must be an object")
    if policy.get("locked") is not True:
        raise ValueError("profile Cargo execution must be locked")
    if not isinstance(policy.get("offline"), bool):
        raise ValueError("profile Cargo offline policy must be a boolean")
    if policy.get("setup_command") != CANONICAL_SETUP_COMMAND:
        raise ValueError(
            f"profile cargo_policy.setup_command must be {CANONICAL_SETUP_COMMAND!r}"
        )
    return shlex.split(CANONICAL_SETUP_COMMAND)


def cargo_preparation_plan(profile, env, command_runner, *, identity=None):
    """Keep existing adapter boundaries and bind each selected consumer explicitly."""
    from .preparation import Preparation
    from .profiles import crate_test_mode

    command = cargo_setup_command(profile)
    setup_env = env.copy()
    setup_env.pop("CARGO_NET_OFFLINE", None)
    compiler_env = setup_env | {"CARGO_NET_OFFLINE": "true"}
    nodes = []
    consumers = {}
    for guard in profile.get("guardrail_steps", []):
        consumers[f"guardrail_{guard.replace('-', '_')}"] = []
    for area in profile.get("selected_areas", []):
        consumers[f"area_{area['area']}"] = ["workspace"]
    for step in profile.get("toolchain_steps", []):
        consumers[f"toolchain_{step.replace('-', '_')}"] = (
            [] if step == "cargo-fmt-check" else ["workspace"])

    def add(name, requires, callback, owners=(), unsafe=False):
        nodes.append(Preparation(name, tuple(requires), callback, unsafe))
        for owner in owners:
            if owner in consumers:
                consumers[owner].append(name)

    python_owners = python_consumers(profile)

    def area_requires(area):
        return (("workspace", "python_environment") if f"area_{area}" in python_owners
                else ("workspace",))

    add("workspace", (), lambda: command_runner(command, env=setup_env))
    crate_owners = [name for name in consumers if name in {
        "toolchain_cargo_test_sifr_smoke", "toolchain_cargo_test_sifr_full"}]
    add("fixtures", ("workspace",),
        lambda: prepare_locked_fixture_caches(profile, setup_env, command_runner))
    if python_owners:
        add("python_environment", (),
            lambda: prepare_python_environment(setup_env, command_runner), python_owners)

    def generated():
        if not any(area["area"] == "generated_code_quality"
                   for area in profile.get("selected_areas", [])):
            return
        revision = subprocess.check_output(
            ["git", "rev-parse", "--verify", "HEAD^{commit}"], cwd=REPO_ROOT,
            text=True).strip()
        shared_root = REPO_ROOT / "target" / "sifr_generated_code_quality" / f"{profile['name']}.{revision}.shared"
        for target_env in (env, setup_env, compiler_env):
            target_env["SIFR_GCQ_SHARED_ROOT"] = str(shared_root)
        command_runner([sys.executable, "-m", "sifr_verify.generated_cargo_setup",
                        "--profile", str(profile["name"]), "--revision", revision], env=setup_env)

    add("generated", ("workspace",), generated, ("area_generated_code_quality",))
    if crate_test_mode(profile) is not None:
        requires = ("workspace", "fixtures")
        if python_owners.intersection(crate_owners):
            requires += ("python_environment",)
        add("crate_tests", requires,
            lambda: prepare_crate_test_binaries(profile, compiler_env, command_runner), crate_owners)
    release_owners = release_compiler_owners(profile)
    if release_owners:
        add("release_compiler", ("workspace",),
            lambda: command_runner([sys.executable, str(REPO_ROOT /
                "verification/areas/common/release_compiler.py")], env=compiler_env),
            release_owners)
    adapters = (
        ("authoring", prepare_authoring_test_binaries, "python_interop"),
        ("tooling", prepare_tooling_test_binaries, "developer_tooling"),
        ("performance", prepare_performance_binaries, "performance"),
        ("oracle", prepare_generated_oracle_binary, "cpython_differential"),
        ("sysroot_source", prepare_sysroot_source_binary, "sysroot_release"),
        ("sysroot_package", prepare_sysroot_package_binary, "sysroot_release"),
    )
    for name, adapter, area in adapters:
        if f"area_{area}" in consumers:
            requires = area_requires(area)
            if name == "sysroot_package" and "area_sysroot_release" in release_owners:
                requires += ("release_compiler",)
            add(name, requires,
                lambda fn=adapter: fn(profile, setup_env, command_runner), (f"area_{area}",))
    for area in profile.get("selected_areas", []):
        if area["area"] in {"sql_platform", "fuzz_property", "python_interop"} or (
            area["area"] == "runtime_platform"
            and {"sanitizer-smoke", "sanitizer-full"}.intersection(area["suites"])
        ):
            selected = profile | {"selected_areas": [area]}
            add("area_graph_" + area["area"], area_requires(area["area"]),
                lambda p=selected: prepare_area_graphs(p, compiler_env, command_runner),
                (f"area_{area['area']}",))
    if "area_rust_interop" in consumers:
        add("maintained_demos", ("workspace",),
            lambda: prepare_maintained_demo_cache(profile, setup_env, command_runner),
            ("area_rust_interop",))
    if identity is not None:
        owners = [name for name in consumers if name.startswith("area_")]
        owners.append("guardrail_demo_emitted_freshness")
        add("compiler_identity", ("workspace",), identity, owners, unsafe=True)
    return nodes, consumers


def prepare_cargo_cache(profile, env, command_runner):
    """Run the same preparation plan with the documented fail-fast default."""
    from .preparation import PreparationRun
    nodes, consumers = cargo_preparation_plan(profile, env, command_runner)
    PreparationRun(nodes, consumers).run()


def prepare_generated_oracle_binary(profile, env, command_runner) -> None:
    """Charge cold release compilation to setup for selected generated oracles."""
    suites = {suite for area in profile.get("selected_areas", [])
              if area["area"] == "cpython_differential" for suite in area["suites"]}
    if not suites.intersection({"generated_broader", "generated_minimized_seeds"}):
        return
    manifest = json.loads((REPO_ROOT / "verification/areas/cpython_differential/"
                           "data/generated_seed_manifest.json").read_text())
    command = [*manifest["release_binary"]["build_command"], "--locked", "--offline"]
    print(f"[sifr-profile-setup] generated-oracle-build={' '.join(command)}", flush=True)
    command_runner(command, env=env)
    prepare = [sys.executable, str(REPO_ROOT / "verification/areas/cpython_differential/"
                                   "checks/prepare_generated.py")]
    for suite in sorted(suites.intersection({"generated_broader", "generated_minimized_seeds"})):
        prepare.extend(["--suite", suite])
    command_runner(prepare, env=env)


def prepare_sysroot_source_binary(profile, env, command_runner) -> None:
    """Prepare the source compiler used by boundary and structural qualification."""
    if not any(area["area"] == "sysroot_release" and
               {"boundary-equivalence", "metadata-structural"}.intersection(area["suites"])
               for area in profile.get("selected_areas", [])):
        return
    command_runner([sys.executable, str(REPO_ROOT /
        "verification/areas/sysroot_release/source_build.py")], env=env)


def prepare_sysroot_package_binary(profile, env, command_runner) -> None:
    """Prepare the exact release graph once for every installed-package suite."""
    required = INSTALLED_PACKAGE_SUITES
    if any(area["area"] == "sysroot_release" and required.intersection(area["suites"])
           for area in profile.get("selected_areas", [])):
        command_runner([sys.executable, str(REPO_ROOT /
            "verification/areas/sysroot_release/package_build.py"), "--corpus-only"], env=env)


def prepare_authoring_test_binaries(profile, env, command_runner) -> None:
    """Charge cold Rust test compilation to the explicit setup step."""
    selected = any(
        area["area"] == "python_interop"
        and "lsp-declaration-authoring" in area["suites"]
        for area in profile.get("selected_areas", [])
    )
    if not selected:
        return
    # Separate invocations match each execution command's feature resolution.
    for package in ("sifr_lsp", "sifr_driver", "sifr_analysis"):
        command = ["cargo", "test", "--locked", "--offline", "--no-run", "-p", package]
        print(f"[sifr-profile-setup] authoring-test-build={' '.join(command)}", flush=True)
        command_runner(command, env=env)



def prepare_tooling_test_binaries(profile, env, command_runner) -> None:
    """Match selected tooling test graphs, including completion's incremental policy."""
    suites = {suite for area in profile.get("selected_areas", [])
              if area["area"] == "developer_tooling" for suite in area["suites"]}
    builds = []
    if suites.intersection({"static", "full"}):
        builds.append(("sifr_lint", env))
        completion_env = env.copy()
        completion_env.setdefault("CARGO_INCREMENTAL", "0")
        builds.append(("sifr_analysis", completion_env))
    if suites.intersection({"formatter", "full"}):
        builds.append(("sifr_format", env))
    if suites.intersection({"analysis", "full"}):
        builds.append(("sifr_analysis", env))
    for package, build_env in builds:
        command = ["cargo", "test", "--locked", "--offline", "--no-run", "-p", package]
        incremental = build_env.get("CARGO_INCREMENTAL", "default")
        print(f"[sifr-profile-setup] tooling-test-build={' '.join(command)} "
              f"incremental={incremental}", flush=True)
        command_runner(command, env=build_env)


def prepare_performance_binaries(profile, env, command_runner) -> None:
    """Build the exact compiler and query-helper graphs before timed benchmarks."""
    suites = {suite for area in profile.get("selected_areas", [])
              if area["area"] == "performance" for suite in area["suites"]}
    commands = []
    if suites.intersection({"smoke", "representative", "full"}):
        commands.extend([
            ["cargo", "build", "--locked", "--offline", "-p", "sifr"],
            ["cargo", "build", "--locked", "--offline", "-p", "sifr_frontend",
             "--bin", "frontend_query_bench"],
        ])
    if "frontend-syntax-guardrails" in suites:
        for package in ("sifr_syntax", "sifr_frontend"):
            commands.append(["cargo", "test", "--locked", "--offline", "--no-run",
                             "-p", package, "--lib"])
    for command in commands:
        print(f"[sifr-profile-setup] performance-build={' '.join(command)}", flush=True)
        command_runner(command, env=env)


def prepare_maintained_demo_cache(profile, env, command_runner) -> None:
    """Prepare the same complete demo graph before its bounded execution area."""
    if not any(area["area"] == "rust_interop" and "matrix" in area["suites"]
               for area in profile.get("selected_areas", [])):
        return
    target = REPO_ROOT / "target"
    target.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(
        prefix=f"{profile['name']}-rust-demo-setup-", dir=target)) / "compile"
    command = [sys.executable, str(REPO_ROOT / "verification/areas/rust_interop/checks/"
                                   "check_maintained_rust_demos.py"),
               "--output", str(output)]
    print(f"[sifr-profile-setup] maintained-demo-preparation={output}", flush=True)
    command_runner(command, env=env)


def enable_offline_cargo(env: dict[str, str]) -> None:
    """Force profile execution to use the prepared Cargo cache."""
    env["CARGO_NET_OFFLINE"] = "true"
    os.environ["CARGO_NET_OFFLINE"] = "true"

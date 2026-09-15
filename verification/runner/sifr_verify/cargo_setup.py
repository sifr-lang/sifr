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


def prepare_cargo_cache(
    profile: dict[str, Any],
    env: dict[str, str],
    command_runner: Callable[..., None],
) -> None:
    """Populate workspace, selected fixture and generated graphs before offline execution."""
    command = cargo_setup_command(profile)
    setup_env = env.copy()
    setup_env.pop("CARGO_NET_OFFLINE", None)
    print(f"[sifr-profile-setup] command={' '.join(command)}")
    command_runner(command, env=setup_env)
    prepare_locked_fixture_caches(profile, setup_env, command_runner)
    if any(area["area"] == "generated_code_quality" for area in profile.get("selected_areas", [])):
        revision = subprocess.check_output(
            ["git", "rev-parse", "--verify", "HEAD^{commit}"], cwd=REPO_ROOT, text=True
        ).strip()
        # A source-identical later commit still names a different Cargo Git source.
        shared_root = REPO_ROOT / "target" / "sifr_generated_code_quality" / f"{profile['name']}.{revision}.shared"
        env["SIFR_GCQ_SHARED_ROOT"] = str(shared_root)
        setup_env["SIFR_GCQ_SHARED_ROOT"] = str(shared_root)
        command_runner(
            [sys.executable, "-m", "sifr_verify.generated_cargo_setup",
             "--profile", str(profile["name"]), "--revision", revision],
            env=setup_env,
        )

    prepare_crate_test_binaries(profile, setup_env, command_runner)
    prepare_authoring_test_binaries(profile, setup_env, command_runner)
    prepare_tooling_test_binaries(profile, setup_env, command_runner)
    prepare_performance_binaries(profile, setup_env, command_runner)
    prepare_generated_oracle_binary(profile, setup_env, command_runner)
    prepare_sysroot_source_binary(profile, setup_env, command_runner)
    prepare_maintained_demo_cache(profile, setup_env, command_runner)


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
    """Prepare the boundary check's private source graph before timed execution."""
    if not any(area["area"] == "sysroot_release" and "boundary-equivalence" in area["suites"]
               for area in profile.get("selected_areas", [])):
        return
    command_runner([sys.executable, str(REPO_ROOT /
        "verification/areas/sysroot_release/source_build.py")], env=env)


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

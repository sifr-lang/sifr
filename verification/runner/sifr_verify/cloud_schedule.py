"""Run the live merge inventory with bounded, owned graph lifetimes on Linux.

Preparation is scheduling, never qualifying assertion evidence. Successful
build graphs retire into immutable outputs before independent runtime assertions.
"""
from __future__ import annotations

import json
import math
import os
import sys
import uuid
from datetime import UTC, datetime
from pathlib import Path

from .assertion_resource_forecast import SQL_BUILD_ALLOCATION, assertion_allocation
from .cargo_setup import (acquire_cargo_dependencies, enable_offline_cargo,
                          prepare_remaining_graphs)
from .early_sql import EarlySqlOutcome, run_early_sql
from .execution_evidence import write_evidence
from .execution_identity import artifact_identity
from .execution_identity import execution_key
from .fixture_inventory import inventory
from .paths import REPO_ROOT
from .profile_commands import run_command
from .resource_admission import ResourceError, admit, discover, worker_limit
from .validation_contracts import stage_plan
from .schemas import load_schema, validate_data
from .prepared_sysroot import OWNER_VARIABLE, command as preparation_command
from .cloud_failure import classify_failure
from .cargo_resource_forecast import command_cache_hint, generated_preparation, test_cache_hint
from .process_disk_budget import DiskBudget, FLOOR_VARIABLE, PATH_VARIABLE

WORKER_FLAGS = {"--sifr-jobs": "sifr_jobs", "--rust-jobs": "rust_jobs",
                "--run-jobs": "run_jobs", "--cargo-build-jobs": "cargo_build_jobs"}


def clamp_workers(runner, resources):
    requested = dict(getattr(runner, "e2e_worker_requests",
                             {field: runner.profile["e2e"][field] for field in WORKER_FLAGS.values()}))
    arguments, preserved, index = runner.forward_args, [], 0
    while index < len(arguments):
        option, _, inline = arguments[index].partition("=")
        if option not in WORKER_FLAGS:
            preserved.append(arguments[index])
        else:
            if not inline:
                index += 1
                if index == len(arguments):
                    raise ResourceError("E2E worker option requires an integer", "unavailable")
                inline = arguments[index]
            try:
                requested[WORKER_FLAGS[option]] = int(inline)
            except ValueError as error:
                raise ResourceError("E2E worker option requires an integer", "unavailable") from error
        index += 1
    runner.forward_args = preserved
    runner.e2e_worker_requests = requested
    runner.e2e_worker_limits = {field: worker_limit(resources, value) for field, value in requested.items()}
    for variable in ("CARGO_BUILD_JOBS", "RAYON_NUM_THREADS"):
        runner.env[variable] = str(worker_limit(resources, int(runner.env[variable])))
    return runner.e2e_worker_limits


def load_schedule(root: Path = REPO_ROOT, mode: str = "cloud") -> dict:
    if mode not in {"cloud", "compact"}:
        raise ResourceError("unknown resource schedule", "unavailable")
    policy = json.loads((root / ("verification/policy/" + mode + "_resource_schedule.json")).read_text())
    validate_data(policy, load_schema("cloud_resource_schedule.schema.json"), source="cloud resource schedule")
    expected = {"dependency-acquisition", "sysroot-source", "sysroot-package", "sysroot-assertions",
                "graph-retirement", "sysroot-metadata", "remaining-preparation", "remaining-assertions",
                "sysroot-metadata-cached", "preparation-coordination", "preparation-command-cold",
                "preparation-command-cached", "generated-preparation", "sysroot-structural-assertions",
                SQL_BUILD_ALLOCATION}
    if policy.get("schema_version") != 1 or set(policy.get("stages", {})) != expected:
        raise ResourceError("cloud resource schedule is incomplete", "unavailable")
    for field in ("cold_preparation_deadline_seconds", "assertion_command_deadline_seconds"):
        value = policy.get(field)
        if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
            raise ResourceError("cloud deadlines must be prospective positive seconds", "unavailable")
    return policy


class Schedule:
    def __init__(self, runner, *, root: Path = REPO_ROOT):
        self.runner, self.root = runner, root
        self.owner = str(uuid.uuid4())
        self.graph_owner = runner.env.get("SIFR_VERIFY_GRAPH_OWNER", self.owner)
        self.journal = root / "target/verification/execution-journals" / self.owner
        self.index = 0
        try:
            self.policy = load_schedule(root, runner.env.get("SIFR_VERIFY_RESOURCE_POLICY", "cloud"))
            resources = discover(disk_path=root)
        except (ResourceError, ValueError, OSError) as error:
            self.record("admission-unavailable", {"state": "infrastructure-failure",
                         "classification": "unavailable", "detail": str(error)})
            raise
        workers = clamp_workers(runner, resources)
        self.key = execution_key(selection={"contract": stage_plan(getattr(runner, "profile_name", "cloud")), "required_kinds": {}},
            commands=[[sys.executable, "-m", "sifr_verify", *sys.argv[1:]]], artifacts=[],
            producer={"kind": "owned-local-runner", "session": self.owner}, services={},
            env=runner.env, root=root, resource_identity=resources.identity())
        self.record("schedule", {"owner": self.owner, "graph_owner": self.graph_owner, "profile": runner.profile,
                                  "policy": self.policy, "resources": resources.identity(), "key": self.key,
                                  "e2e_workers": workers,
                                  "reuse": "disabled until individual dependency closure is proven"})

    def record(self, name: str, payload: dict):
        self.index += 1
        write_evidence(self.journal / f"{self.index:04d}-{name}.json",
                       {"schema_version": 1, "claim": "execution-observation",
                        "observed_at": datetime.now(UTC).isoformat(), **payload})

    def step(self, name, callback, *, allocation, preparation=False, monitor_disk=False,
             command_env=None, propagate_failure=False):
        monitored_env = self.runner.env if command_env is None else command_env
        saved_disk = {field: monitored_env.get(field) for field in (FLOOR_VARIABLE, PATH_VARIABLE)}
        def admitted():
            try:
                if inventory(self.root) != self.key["inputs"]["source"]:
                    raise ResourceError("validation inputs changed during the owned run", "unavailable")
                resources = discover(disk_path=self.root)
                workers = clamp_workers(self.runner, resources)
                if command_env is not None:
                    for variable in ("CARGO_BUILD_JOBS", "RAYON_NUM_THREADS"):
                        if variable in command_env:
                            command_env[variable] = str(worker_limit(resources, int(command_env[variable])))
                observation = admit(resources, self.policy["stages"][allocation])
            except ResourceError as error:
                self.record(name, {"state": "infrastructure-failure", "classification": error.classification,
                                   "detail": str(error)})
                raise
            self.record(name, {"state": "admitted", "e2e_workers": workers, **observation})
            try:
                if monitor_disk:
                    # A bounded attempt always runs the original native command.
                    # Keep the unchanged reserve plus 1GiB stopping headroom;
                    # this guard is polling containment, not a filesystem quota.
                    requirements = observation["requirements"]
                    floor = max(resources.disk_available_bytes - requirements["disk_growth_bytes"],
                                requirements["disk_reserve_bytes"] + 1024**3)
                    try:
                        inherited = DiskBudget.from_environment(monitored_env)
                    except ValueError as error:
                        raise ResourceError(str(error), "unavailable") from error
                    if inherited is not None:
                        if inherited.path != self.root.resolve():
                            raise ResourceError("inherited disk floor describes a different filesystem", "unavailable")
                        floor = max(floor, inherited.floor)
                    monitored_env[FLOOR_VARIABLE] = str(floor)
                    monitored_env[PATH_VARIABLE] = str(self.root.resolve())
                    self.record(name + "-disk-floor", {"floor_bytes": floor, "allocation": allocation,
                        "claim": "bounded-preparation-attempt", "cache_reuse_claim": False})
                callback()
                if inventory(self.root) != self.key["inputs"]["source"]:
                    raise ResourceError("validation inputs changed during step execution", "unavailable")
            except BaseException as error:
                try:
                    after = discover(disk_path=self.root).diagnostics
                except ResourceError as observation_error:
                    after = {"observation_error": str(observation_error)}
                classification = classify_failure(error, resources.diagnostics, after)
                self.record(name, {"state": "failed", "classification": classification,
                                   "detail": str(error), "resources_after": after})
                raise
            self.record(name, {"state": "completed", "preparation": preparation,
                               "resources_after": discover(disk_path=self.root).__dict__})
        variable = "SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS" if preparation else "SIFR_VERIFY_SAFETY_DEADLINE_SECONDS"
        previous = self.runner.env.get(variable)
        field = "cold_preparation_deadline_seconds" if preparation else "assertion_command_deadline_seconds"
        seconds = float(self.policy[field])
        for inherited in (previous, self.runner.env.get("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS")):
            if inherited is not None:
                value = float(inherited)
                if not math.isfinite(value) or value <= 0:
                    raise ResourceError("inherited safety duration must be positive and finite", "unavailable")
                seconds = min(seconds, value)
        self.runner.env[variable] = str(seconds)
        failures = []
        def observed():
            try:
                return admitted()
            except BaseException as error:
                failures.append(error)
                raise
        try:
            status = self.runner.execute_step(name, observed)
            if status and propagate_failure and failures:
                raise failures[0]
            return status
        finally:
            for field, value in saved_disk.items():
                if value is None:
                    monitored_env.pop(field, None)
                else:
                    monitored_env[field] = value
            if previous is None:
                self.runner.env.pop(variable, None)
            else:
                self.runner.env[variable] = previous

    def prepare_command(self, command, *, env):
        generated = generated_preparation(command)
        cached = not generated and command_cache_hint(self.root, env, command,
                                    include_library=env.get("SIFR_VERIFY_RESOURCE_POLICY") == "compact")
        allocation = ("generated-preparation" if generated else
                      "preparation-command-cached" if cached else "preparation-command-cold")
        self.record("preparation-command", {"argv": command, "allocation": allocation,
                    "cache_presence_hint": cached, "assertion_reuse": False})
        status = self.step(f"preparation_command_{self.index:04d}",
            lambda: run_command(command, env=env), allocation=allocation,
            preparation=True, monitor_disk=True, command_env=env, propagate_failure=True)
        if status:
            raise ResourceError(f"preparation failed without a recorded exception: status={status}", "unavailable")


def run_staged_cloud(runner, early: set[str]) -> int:
    from .profile_runner import step_name
    if not sys.platform.startswith("linux"):
        raise ResourceError("the shared-cloud scheduler requires Linux cgroup v2", "unavailable")
    schedule = Schedule(runner)
    print(f"Cloud execution journal: {schedule.journal}")
    profile, env = runner.profile, runner.env
    selected = [area for area in profile["selected_areas"] if area["area"] == "sysroot_release"]
    if len(selected) != 1:
        raise ResourceError("cloud requires the canonical sysroot consumer selection", "unavailable")
    acquired = schedule.step("preparation_dependencies", lambda: acquire_cargo_dependencies(profile, env, run_command),
                             allocation="dependency-acquisition", preparation=True)
    if acquired:
        runner.block_steps("preparation_dependencies")
        return acquired
    enable_offline_cargo(env)
    env[OWNER_VARIABLE] = schedule.owner
    env["SIFR_VERIFY_GRAPH_OWNER"] = schedule.graph_owner
    if env.get("SIFR_VERIFY_RESOURCE_POLICY") == "compact":
        env["SIFR_VERIFY_SYSROOT_GRAPH_SESSION"] = schedule.owner
    failed = 0
    for kind in ("source", "package"):
        status = schedule.step("preparation_sysroot_" + kind,
            lambda k=kind: run_command(preparation_command("prepare", k), env=env),
            allocation="sysroot-" + kind, preparation=True)
        failed = failed or status
        if status and not runner.no_fail_fast:
            return failed
    # Private graphs now have retained immutable outputs. The large library
    # preparations run after retirement, rather than extending both lifetimes.
    cached_metadata = test_cache_hint(REPO_ROOT, env, "sifr_driver")
    schedule.record("metadata-forecast", {"cache_presence_hint": cached_metadata,
                    "assertion_reuse": False, "native_cache_validation": "Cargo always runs"})
    metadata = schedule.step("preparation_sysroot_metadata", lambda: run_command(
        [sys.executable, str(REPO_ROOT / "verification/areas/sysroot_release/package_build.py"),
         "--metadata-only"], env=env), allocation="sysroot-metadata-cached" if cached_metadata else "sysroot-metadata",
         preparation=True, monitor_disk=True)
    failed = failed or metadata
    if failed:
        runner.block_step("area_sysroot_release", "sysroot-preparation")
    else:
        status = schedule.step("area_sysroot_release",
            lambda: runner.run_area("sysroot_release", selected[0]["suites"]), allocation="sysroot-assertions")
        failed = failed or status
        if not status:
            result = REPO_ROOT / "target/verification/areas/sysroot-release-cloud-results.json"
            schedule.record("sysroot-results", {"result": json.loads(result.read_text()),
                                                "identity": artifact_identity(result)})
    if failed and not runner.no_fail_fast:
        return failed
    early_sql = run_early_sql(runner, schedule) if not failed else EarlySqlOutcome()
    failed = failed or early_sql.status
    if early_sql.status and not runner.no_fail_fast:
        return failed
    # Retain every other preparation. SQL was handled in this invocation;
    # its outcome remains authoritative even when independent work continues.
    prepared = schedule.step("cargo_cache_setup", lambda: prepare_remaining_graphs(
        profile, env, schedule.prepare_command, include_sysroot=False,
        sql_preparation_handled=early_sql.selected), allocation="preparation-coordination", preparation=True)
    failed = failed or prepared
    if prepared:
        runner.block_steps("cargo_cache_setup",
            handled_areas={"sysroot_release"} | ({"sql_platform"} if early_sql.selected else set()),
            include_setup=False)
        return failed
    for guard in profile["guardrail_steps"]:
        if guard in early:
            continue
        status = schedule.step(step_name("guardrail", guard), lambda g=guard: runner.run_guardrail(g),
                               allocation="remaining-assertions")
        failed = failed or status
        if status and not runner.no_fail_fast:
            return failed
    for area in profile["selected_areas"]:
        if area["area"] == "sysroot_release" or (area["area"] == "sql_platform" and early_sql.selected):
            continue
        name = step_name("area", area["area"])
        allocation = assertion_allocation(name, profile)
        status = schedule.step(name, lambda a=area: runner.run_area(a["area"], a["suites"]),
                               allocation=allocation, monitor_disk=allocation == SQL_BUILD_ALLOCATION)
        failed = failed or status
        if status and not runner.no_fail_fast:
            return failed
    failed_toolchain = set()
    for name in profile["toolchain_steps"]:
        if name in {"e2e-report-determinism", "e2e-sequential-parallel-equivalence"} and "e2e-pass" in failed_toolchain:
            runner.block_step(step_name("toolchain", name), "e2e-pass")
            continue
        status = schedule.step(step_name("toolchain", name), lambda n=name: runner.run_toolchain_step(n),
                               allocation="remaining-assertions")
        failed = failed or status
        if status:
            failed_toolchain.add(name)
        if status and not runner.no_fail_fast:
            return failed
    schedule.record("completion", {"state": "failed" if failed else "completed", "status": failed,
                                   "scope": "functional-only", "performance": "unqualified"})
    return failed

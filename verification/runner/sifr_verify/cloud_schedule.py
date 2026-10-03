"""Run the live merge inventory with bounded, owned graph lifetimes on Linux.

Preparation is scheduling, never qualifying assertion evidence. All selected
assertions run once; an early failure keeps its graph and blocks its retirement.
"""
from __future__ import annotations

import json
import sys
import uuid
from datetime import UTC, datetime
from pathlib import Path

from .cargo_setup import (acquire_cargo_dependencies, enable_offline_cargo,
                          prepare_remaining_graphs, prepare_sysroot_package_binary,
                          prepare_sysroot_source_binary)
from .execution_evidence import write_evidence
from .execution_identity import artifact_identity
from .execution_identity import execution_key
from .fixture_inventory import inventory
from .graph_retirement import GRAPH_PATHS, GraphLease
from .paths import REPO_ROOT
from .profile_commands import run_command
from .resource_admission import ResourceError, admit, discover, worker_limit
from .validation_contracts import stage_plan
from .schemas import load_schema, validate_data


def load_schedule(root: Path = REPO_ROOT) -> dict:
    policy = json.loads((root / "verification/policy/cloud_resource_schedule.json").read_text())
    validate_data(policy, load_schema("cloud_resource_schedule.schema.json"), source="cloud resource schedule")
    expected = {"dependency-acquisition", "sysroot-source", "sysroot-package", "sysroot-assertions",
                "graph-retirement", "remaining-preparation", "remaining-assertions"}
    if policy.get("schema_version") != 1 or set(policy.get("stages", {})) != expected:
        raise ResourceError("cloud resource schedule is incomplete", "unavailable")
    for field in ("cold_preparation_deadline_seconds", "assertion_deadline_seconds"):
        value = policy.get(field)
        if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
            raise ResourceError("cloud deadlines must be prospective positive seconds", "unavailable")
    return policy


class Schedule:
    def __init__(self, runner, *, root: Path = REPO_ROOT):
        self.runner, self.root = runner, root
        self.policy = load_schedule(root)
        self.owner = str(uuid.uuid4())
        self.journal = root / "target/verification/execution-journals" / self.owner
        self.index = 0
        resources = discover(disk_path=root)
        for variable in ("CARGO_BUILD_JOBS", "RAYON_NUM_THREADS"):
            runner.env[variable] = str(worker_limit(resources, int(runner.env[variable])))
        self.key = execution_key(selection={"contract": stage_plan("cloud"), "required_kinds": {}},
            commands=[[sys.executable, "-m", "sifr_verify", *sys.argv[1:]]], artifacts=[],
            producer={"kind": "owned-local-runner", "session": self.owner}, services={},
            env=runner.env, root=root, resource_identity=resources.identity())
        self.record("schedule", {"owner": self.owner, "profile": runner.profile,
                                  "policy": self.policy, "resources": resources.identity(), "key": self.key,
                                  "reuse": "disabled until individual dependency closure is proven"})

    def record(self, name: str, payload: dict):
        self.index += 1
        write_evidence(self.journal / f"{self.index:04d}-{name}.json",
                       {"schema_version": 1, "claim": "execution-observation",
                        "observed_at": datetime.now(UTC).isoformat(), **payload})

    def step(self, name, callback, *, allocation, preparation=False):
        def admitted():
            try:
                if inventory(self.root) != self.key["inputs"]["source"]:
                    raise ResourceError("validation inputs changed during the owned run", "unavailable")
                observation = admit(discover(disk_path=self.root), self.policy["stages"][allocation])
            except ResourceError as error:
                self.record(name, {"state": "infrastructure-failure", "classification": error.classification,
                                   "detail": str(error)})
                raise
            self.record(name, {"state": "admitted", **observation})
            try:
                callback()
                if inventory(self.root) != self.key["inputs"]["source"]:
                    raise ResourceError("validation inputs changed during step execution", "unavailable")
            except BaseException as error:
                classification = getattr(error, "cause", None) or getattr(error, "classification", None)
                classification = {"safety_deadline": "timeout", "exit": "assertion"}.get(classification, classification)
                self.record(name, {"state": "failed", "classification": classification or "assertion",
                                   "detail": str(error), "resources_after": discover(disk_path=self.root).diagnostics})
                raise
            self.record(name, {"state": "completed", "preparation": preparation,
                               "resources_after": discover(disk_path=self.root).__dict__})
        previous = self.runner.env.get("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS")
        field = "cold_preparation_deadline_seconds" if preparation else "assertion_deadline_seconds"
        self.runner.env["SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS"] = str(self.policy[field])
        try:
            return self.runner.execute_step(name, admitted)
        finally:
            if previous is None:
                self.runner.env.pop("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS", None)
            else:
                self.runner.env["SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS"] = previous


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
    leases = []
    failed = 0
    try:
        for graph in GRAPH_PATHS:
            leases.append(GraphLease(REPO_ROOT, graph, schedule.owner).acquire({"area_sysroot_release"}))
        for name, prepare, allocation in (
                ("preparation_sysroot_source", prepare_sysroot_source_binary, "sysroot-source"),
                ("preparation_sysroot_package", prepare_sysroot_package_binary, "sysroot-package")):
            status = schedule.step(name, lambda p=prepare: p(profile, env, run_command),
                                   allocation=allocation, preparation=True)
            failed = failed or status
            if status and not runner.no_fail_fast:
                return failed
        if failed:
            runner.block_step("area_sysroot_release", "sysroot-preparation")
        else:
            status = schedule.step("area_sysroot_release",
                                   lambda: runner.run_area("sysroot_release", selected[0]["suites"]),
                                   allocation="sysroot-assertions")
            failed = failed or status
            if not status:
                # Results survive both graph lifetimes in an immutable owned journal.
                result = REPO_ROOT / "target/verification/areas/sysroot-release-cloud-results.json"
                payload = json.loads(result.read_text())
                schedule.record("sysroot-results", {"result": payload, "identity": artifact_identity(result)})
                for lease in leases:
                    lease.passed_consumer("area_sysroot_release")
                    binary = lease.path / ("debug/sifr" if lease.relative == GRAPH_PATHS[0] else "release/sifr")
                    retirement = schedule.step("retirement_" + lease.path.name.replace("-", "_"),
                        lambda l=lease, b=binary: schedule.record("retirement", l.retire(
                            retained=REPO_ROOT / "target/verification/retained-compilers" / schedule.owner,
                            protected=[b], command_runner=run_command, env=env)), allocation="graph-retirement")
                    failed = failed or retirement
            if failed and not runner.no_fail_fast:
                return failed
    except (ResourceError, OSError, ValueError) as error:
        schedule.record("graph-lifetime-failure", {"state": "infrastructure-failure",
                         "classification": getattr(error, "classification", "unavailable"), "detail": str(error)})
        raise
    finally:
        for lease in leases:
            lease.close()
    # Every original remaining preparation is retained; only completed sysroot
    # preparation/consumers move earlier. No selection or timeout assertion changes.
    prepared = schedule.step("cargo_cache_setup", lambda: prepare_remaining_graphs(
        profile, env, run_command, include_sysroot=False), allocation="remaining-preparation", preparation=True)
    failed = failed or prepared
    if prepared:
        runner.block_steps("cargo_cache_setup")
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
        if area["area"] == "sysroot_release":
            continue
        status = schedule.step(step_name("area", area["area"]),
                               lambda a=area: runner.run_area(a["area"], a["suites"]),
                               allocation="remaining-assertions")
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

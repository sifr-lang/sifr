"""Explicit local-correctness groups sharing one profile allocation."""
from __future__ import annotations

from dataclasses import asdict, replace
import json

from .build_allocation import E2E_ENV, MIB, positive
from .native_test_allocation import ALLOCATION_ENV, allocate_native

PAIR = {"python_interop": 1, "generated_code_quality": 3}
LOCAL_AREAS = ("coverage_matrix", "core_language", "diagnostics", "distribution_release")
GROUP_AREAS = (*PAIR, *LOCAL_AREAS)


def execution_groups(profile, correctness_only):
    if not correctness_only or profile["name"] not in {"create-pr", "merge"}:
        return []
    groups = []
    owners = (
        ("python-quality", PAIR, {
            "python_interop": ["target/verification/areas/python_interop", "target/python"],
            "generated_code_quality": ["target/sifr_generated_code_quality"],
            "shared": "prepared immutable compiler; existing native-family leases",
        }),
        ("local-contracts", LOCAL_AREAS, {
            "coverage_matrix": ["source inventories; temporary negative-control roots"],
            "core_language": ["target/verification/actual/core_language"],
            "diagnostics": ["target/verification/actual/diagnostics; temporary examples"],
            "distribution_release": ["target/distribution_release; per-case temporary roots",
                                     "target/qualification-plan-must-not-exist.json"],
            "shared": "prepared immutable compiler; Cargo-owned target locks; distinct area reports",
        }),
    )
    for name, areas, ownership in owners:
        selected = [item["area"] for item in profile["selected_areas"] if item["area"] in areas]
        if len(selected) == len(areas) and set(selected) == set(areas):
            groups.append({"id": name, "areas": selected, "ownership": ownership,
                           "normal_slots": partition_slots(
                               selected, profile["resource_policy"]["max_parallel"])[0]})
    return groups


def partition_slots(areas, total):
    total = positive(total, "profile process slots")
    if (not areas or len(set(areas)) != len(areas)
            or not any(set(areas) <= set(group) for group in (PAIR, LOCAL_AREAS))):
        raise ValueError("unqualified area group selection")
    if len(areas) == 1:
        return {areas[0]: total}, 1
    if set(areas) <= set(LOCAL_AREAS):
        # Admit the complete ready group together, or retain serial execution.
        # This leaves no queued sibling behind a partially failed parallel group.
        if total < len(LOCAL_AREAS):
            return {area: total for area in areas}, 1
        return {area: 1 for area in areas}, len(areas)
    if total == 1:
        return {area: 1 for area in areas}, 1
    # Keep the measured 3+1 split at four slots; the pair never exceeds the owner.
    python_slots = max(1, total // 4)
    return {"python_interop": python_slots, "generated_code_quality": total - python_slots}, 2


def lane_environment(build, env, slots):
    slots = positive(slots, "area process slots")
    if slots > build.slot_limit:
        raise ValueError("area allocation exceeds its profile owner")
    inner = min(build.cargo_build_jobs, slots)
    # Reserve only this lane's memory share for any native sub-owner. These areas
    # currently use direct runtime checks; a future native worker cannot silently
    # borrow the sibling's memory allocation.
    memory = min(build.available_memory_mib, slots * build.policy["memory_per_build_job_mib"]
                 + build.policy["reserved_memory_mib"])
    lane = replace(build, slot_limit=slots, compiler_jobs=min(build.compiler_jobs, slots),
                   sifr_jobs=min(build.sifr_jobs, slots), rust_jobs=min(build.rust_jobs, slots // inner),
                   run_jobs=min(build.run_jobs, slots), cargo_build_jobs=inner,
                   available_memory_mib=memory, policy=build.policy | {"max_parallel": slots})
    result = dict(env, CARGO_BUILD_JOBS=str(lane.compiler_jobs),
                  RAYON_NUM_THREADS=str(lane.sifr_jobs), SIFR_VERIFY_PROCESS_SLOTS=str(slots))
    result["RUST_TEST_THREADS"] = str(min(positive(env.get("RUST_TEST_THREADS", slots),
                                                 "RUST_TEST_THREADS"), slots))
    for field, name in E2E_ENV.items():
        result[name] = str(getattr(lane, field))
    result[ALLOCATION_ENV] = json.dumps(asdict(allocate_native(lane)))
    for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
        result[name] = str(min(positive(env.get(name, slots), name), slots))
    return lane, result


def reserve_bytes(build):
    return build.policy["reserved_memory_mib"] * MIB


def order_area_steps(profile, steps):
    """Keep area report order canonical while preserving actual elapsed values."""
    order = {"area_" + item["area"]: index for index, item in enumerate(profile["selected_areas"])}
    positions = [index for index, step in enumerate(steps) if step["name"] in order]
    areas = sorted((steps[index] for index in positions), key=lambda step: order[step["name"]])
    result = list(steps)
    for index, step in zip(positions, areas, strict=True):
        result[index] = step
    return result

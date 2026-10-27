"""The qualified local-correctness pair and its share of one profile allocation."""
from __future__ import annotations

from dataclasses import asdict, replace
import json

from .build_allocation import E2E_ENV, MIB, positive
from .native_test_allocation import ALLOCATION_ENV, allocate_native

PAIR = {"python_interop": 1, "generated_code_quality": 3}


def execution_groups(profile, correctness_only):
    selected = [item["area"] for item in profile["selected_areas"] if item["area"] in PAIR]
    if not correctness_only or profile["name"] not in {"create-pr", "merge"} or len(selected) != 2 or set(selected) != set(PAIR):
        return []
    return [{
        "id": "python-quality",
        "areas": selected,
        "normal_slots": {area: PAIR[area] for area in selected},
        "ownership": {
            "python_interop": ["target/verification/areas/python_interop", "target/python"],
            "generated_code_quality": ["target/sifr_generated_code_quality"],
            "shared": "prepared immutable compiler; existing native-family leases",
        },
    }]


def partition_slots(areas, total):
    total = positive(total, "profile process slots")
    if not areas or len(set(areas)) != len(areas) or not set(areas) <= PAIR.keys():
        raise ValueError("unqualified area group selection")
    if len(areas) == 1:
        return {areas[0]: total}, 1
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

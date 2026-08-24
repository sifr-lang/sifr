from __future__ import annotations

import json
from pathlib import Path

from dlpack_examples import DLPACK_EXAMPLE_CASES


def validate_dlpack_declaration_evidence(payload: object, fixtures_root: Path) -> None:
    required_keys = {
        "schema_version",
        "capability",
        "surface",
        "positive",
        "negative",
        "cleanup",
        "cancellation",
        "live",
        "profiles",
    }
    if not isinstance(payload, dict) or set(payload) != required_keys:
        raise SystemExit("DLPack evidence top-level schema drift")
    if payload.get("schema_version") != 1:
        raise SystemExit("DLPack evidence schema_version must be 1")
    if payload.get("capability") != "dlpack-one-shot-declaration":
        raise SystemExit("DLPack evidence capability drift")
    expected_ids = {
        "positive": {"compiler-contract", "runtime-protocol"},
        "negative": {"static-and-runtime-rejection"},
        "cleanup": {"one-shot-exact-release"},
    }
    repo_root = fixtures_root.parents[3]
    for matrix_name, ids in expected_ids.items():
        matrix = payload.get(matrix_name)
        observed_ids = (
            {row.get("id") for row in matrix if isinstance(row, dict)}
            if isinstance(matrix, list)
            else set()
        )
        if observed_ids != ids:
            raise SystemExit(f"DLPack evidence {matrix_name} row id drift")
        for row in matrix:
            if set(row) != {"id", "owners", "covers"}:
                raise SystemExit(f"DLPack evidence {matrix_name} row schema drift")
            owners = row.get("owners")
            covers = row.get("covers")
            if (
                not isinstance(owners, list)
                or not owners
                or len(owners) != len(set(owners))
            ):
                raise SystemExit(f"DLPack evidence owner drift: {row['id']}")
            if (
                not isinstance(covers, list)
                or not covers
                or len(covers) != len(set(covers))
            ):
                raise SystemExit(f"DLPack evidence coverage drift: {row['id']}")
            for owner in owners:
                if not isinstance(owner, str) or not (repo_root / owner).is_file():
                    raise SystemExit(f"DLPack evidence owner is missing: {owner}")
    if payload.get("cancellation") != {
        "status": "not-applicable",
        "reason": "DLPack acquisition and transfer are synchronous blocking boundaries",
    }:
        raise SystemExit("DLPack evidence cancellation contract drift")
    live = payload.get("live")
    registered = {
        (case.case_id, case.relative_source, case.stdout_marker)
        for case in DLPACK_EXAMPLE_CASES.values()
    }
    observed = set()
    if not isinstance(live, list):
        raise SystemExit("DLPack live evidence must be a list")
    for row in live:
        if not isinstance(row, dict) or set(row) != {"id", "source", "stdout_marker"}:
            raise SystemExit("DLPack live evidence row schema drift")
        source = fixtures_root / row["source"]
        if not source.is_file() or row["stdout_marker"] not in source.read_text(
            encoding="utf-8"
        ):
            raise SystemExit(
                f"DLPack live evidence source/marker is missing: {row['id']}"
            )
        observed.add((row["id"], row["source"], row["stdout_marker"]))
    if observed != registered:
        raise SystemExit("DLPack live evidence must match its executable case registry")
    required_profiles = ["create-pr", "merge", "nightly", "release"]
    if payload.get("profiles") != required_profiles:
        raise SystemExit(
            "DLPack evidence must remain blocking in every delivery profile"
        )
    manifest = json.loads(
        (repo_root / "verification/areas/python_interop/manifest.json").read_text(
            encoding="utf-8"
        )
    )
    suites = [
        suite for suite in manifest["suites"] if suite["name"] == "dlpack-examples"
    ]
    if (
        len(suites) != 1
        or suites[0].get("kind") != "adapter"
        or suites[0].get("cases", [{}])[0].get("command")
        != "python-interop-dlpack-examples"
    ):
        raise SystemExit("DLPack example manifest ownership drift")
    runtime_suites = [
        suite for suite in manifest["suites"] if suite["name"] == "dlpack-runtime"
    ]
    if (
        len(runtime_suites) != 1
        or runtime_suites[0].get("kind") != "adapter"
        or runtime_suites[0].get("cases", [{}])[0].get("command")
        != "python-interop-dlpack-runtime"
    ):
        raise SystemExit("DLPack runtime test manifest ownership drift")
    for profile in required_profiles:
        profile_payload = json.loads(
            (repo_root / f"verification/profiles/{profile}.json").read_text(
                encoding="utf-8"
            )
        )
        python_areas = [
            area
            for area in profile_payload["selected_areas"]
            if area["area"] == "python_interop"
        ]
        required_suites = {"dlpack-examples", "dlpack-runtime"}
        if len(python_areas) != 1 or not required_suites.issubset(
            python_areas[0]["suites"]
        ):
            raise SystemExit(f"DLPack evidence suites are not blocking in {profile}")

from __future__ import annotations

import json
from pathlib import Path

from arrow_examples import ARROW_EXAMPLE_CASES


def validate_arrow_declaration_evidence(payload: object, fixtures_root: Path) -> None:
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
        raise SystemExit("Arrow evidence top-level schema drift")
    if payload.get("schema_version") != 2:
        raise SystemExit("Arrow evidence schema_version must be 2")
    if payload.get("capability") != "arrow-c-data-interface-declaration":
        raise SystemExit("Arrow evidence capability drift")
    expected_ids = {
        "positive": {
            "compiler-contract",
            "runtime-protocol",
            "executable-certification",
        },
        "negative": {
            "declaration-and-ownership",
            "capsule-and-device-validation",
            "certification-rejection",
        },
        "cleanup": {"capsule-release", "consumer-reconciliation"},
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
            raise SystemExit(f"Arrow evidence {matrix_name} row id drift")
        for row in matrix:
            if set(row) != {"id", "owners", "covers"}:
                raise SystemExit(f"Arrow evidence {matrix_name} row schema drift")
            owners = row.get("owners")
            covers = row.get("covers")
            if (
                not isinstance(owners, list)
                or not owners
                or len(owners) != len(set(owners))
            ):
                raise SystemExit(f"Arrow evidence owner drift: {row['id']}")
            if (
                not isinstance(covers, list)
                or not covers
                or len(covers) != len(set(covers))
            ):
                raise SystemExit(f"Arrow evidence coverage drift: {row['id']}")
            for owner in owners:
                if not isinstance(owner, str) or not (repo_root / owner).is_file():
                    raise SystemExit(f"Arrow evidence owner is missing: {owner}")
    if payload.get("cancellation") != {
        "status": "not-applicable",
        "reason": "Arrow acquisition and transfer are synchronous blocking boundaries",
    }:
        raise SystemExit("Arrow evidence cancellation contract drift")
    live = payload.get("live")
    if not isinstance(live, list) or len(live) != 1:
        raise SystemExit("Arrow live evidence requires exactly one registered row")
    row = live[0]
    if set(row) != {"id", "source", "fixture", "targets", "stdout_marker"}:
        raise SystemExit("Arrow live evidence row schema drift")
    case = ARROW_EXAMPLE_CASES.get(row.get("id"))
    if case is None or case.relative_source.removeprefix("pyarrow_capsule/") != row.get(
        "source"
    ):
        raise SystemExit("Arrow live evidence is not registered")
    if case.stdout_marker != row.get("stdout_marker"):
        raise SystemExit("Arrow live evidence marker drift")
    registered = {
        target
        for target, fixture in case.arrow_certifications
        if fixture == row.get("fixture")
    }
    if registered != set(row.get("targets", [])):
        raise SystemExit("Arrow live evidence certification target drift")
    source = fixtures_root / "pyarrow_capsule" / row["source"]
    fixture = fixtures_root / "pyarrow_capsule" / row["fixture"]
    if not source.is_file() or row["stdout_marker"] not in source.read_text(
        encoding="utf-8"
    ):
        raise SystemExit("Arrow live evidence source/marker is missing")
    fixture_text = fixture.read_text(encoding="utf-8")
    for required in (
        "source_address == observed",
        'observed_format == b"l"',
        "instrument_release",
        '"target": target',
        '"kind": kind',
        '"identity_method": identity_method',
    ):
        if required not in fixture_text:
            raise SystemExit(
                f"Arrow executable evidence is missing measurement: {required}"
            )
    if payload.get("profiles") != ["create-pr", "merge", "nightly", "release"]:
        raise SystemExit(
            "Arrow evidence must remain blocking in every delivery profile"
        )
    manifest = json.loads(
        (repo_root / "verification/areas/python_interop/manifest.json").read_text(
            encoding="utf-8"
        )
    )
    required_suites = {
        "arrow-examples": "python-interop-arrow-examples",
        "arrow-runtime": "python-interop-arrow-runtime",
    }
    for suite_name, command in required_suites.items():
        suites = [suite for suite in manifest["suites"] if suite["name"] == suite_name]
        if (
            len(suites) != 1
            or suites[0].get("kind") != "adapter"
            or suites[0].get("cases", [{}])[0].get("command") != command
        ):
            raise SystemExit(f"{suite_name} manifest ownership drift")
    for profile in payload["profiles"]:
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
        if len(python_areas) != 1 or not set(required_suites).issubset(
            python_areas[0]["suites"]
        ):
            raise SystemExit(f"Arrow evidence suites are not blocking in {profile}")

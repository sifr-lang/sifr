"""Focused self-tests for release-profile evidence production."""

from __future__ import annotations

import json
import tempfile
from collections.abc import Callable
from pathlib import Path

from .release_evidence import (
    CRITICAL_RESULTS,
    GovernanceError,
    build_release_profile_payload,
    canonicalize_custodied_results,
    load_json_strict,
    prepare_release_report_output,
    sha256_file,
    validate_release_profile_report,
)


def release_report_precondition_self_test() -> None:
    with tempfile.TemporaryDirectory(
        prefix="sifr-release-report-self-test-"
    ) as directory:
        existing = Path(directory) / "release-profile-report.json"
        existing.write_text("occupied", encoding="utf-8")
        cases = [
            (existing, "release", "already exists"),
            (
                Path(directory) / "non-release.json",
                "merge",
                "accepted only for the release",
            ),
            (
                Path(__file__).resolve().parents[3]
                / "target"
                / "release-profile-report.json",
                "release",
                "outside the repository checkout",
            ),
        ]
        for path, profile, expected in cases:
            try:
                prepare_release_report_output(str(path), profile_name=profile)
            except ValueError as exc:
                if expected not in str(exc):
                    raise AssertionError(
                        f"unexpected release report rejection: {exc}"
                    ) from exc
            else:
                raise AssertionError(
                    f"release report precondition mutation passed: {path}"
                )


def release_report_production_self_test() -> None:
    selected = {
        "rust_interop": [
            "matrix",
            "tiers",
            "compatibility-matrix",
            "stale-drafts",
            "stable-candidate",
        ],
        "developer_tooling": ["full"],
        "documentation": ["structure", "ga-release"],
        "distribution_release": [
            "full",
            "qualification",
            "evidence-custody",
            "incident-governance",
            "epoch-bootstrap",
            "protected-drill",
            "stable-prepare",
            "stable-publish-primitives",
            "stable-publication",
        ],
    }
    with tempfile.TemporaryDirectory(
        prefix="sifr-release-report-production-"
    ) as directory:
        root = Path(directory)
        result_root = root / "target" / "verification" / "areas"
        result_root.mkdir(parents=True)
        for area, filename in CRITICAL_RESULTS.items():
            suites = []
            for suite_name in selected[area]:
                labels = [f"{suite_name}:case"]
                if area == "developer_tooling":
                    labels.append("editor-release:package")
                suites.append(
                    {
                        "name": suite_name,
                        "cases": [{"variants": [{"label": label} for label in labels]}],
                    }
                )
            (result_root / filename).write_text(
                json.dumps({"area": area, "suites": suites}, indent=2, sort_keys=True),
                encoding="utf-8",
            )
        rust_result = result_root / CRITICAL_RESULTS["rust_interop"]
        before = rust_result.read_bytes()
        canonicalize_custodied_results(result_root)
        after = rust_result.read_bytes()
        if before == after or after != (
            json.dumps(
                json.loads(before),
                ensure_ascii=False,
                allow_nan=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            + "\n"
        ).encode():
            raise AssertionError(
                "release custody did not canonicalize the exact Rust result bytes"
            )
        load_json_strict(rust_result, require_canonical=True)
        log_path = root / "release.log"
        log_path.write_text(
            "".join(
                f"[sifr-lane-step] name={name} elapsed_ms=1 status=pass\n"
                for name in (
                    "rust_interop_checks",
                    "developer_tooling_checks",
                    "documentation_checks",
                    "distribution_validation",
                )
            ),
            encoding="utf-8",
        )
        output_path = root / "release-profile-report.json"
        payload = build_release_profile_payload(
            output_path=output_path,
            log_path=log_path,
            profile={
                "selected_areas": [
                    {"area": area, "suites": suites}
                    for area, suites in selected.items()
                ]
            },
            profile_digest="a" * 64,
            commit="e" * 40,
            submodules={},
            toolchain={
                "rustc": "rustc fixture",
                "cargo": "cargo fixture",
                "uv": "uv fixture",
                "python": "python fixture",
            },
            result_root=result_root,
            artifact_root=root,
        )
        rust_artifact = next(
            artifact
            for artifact in payload["result_artifacts"]
            if Path(artifact["path"]).name == CRITICAL_RESULTS["rust_interop"]
        )
        if rust_artifact["sha256"] != sha256_file(rust_result):
            raise AssertionError(
                "release report did not bind canonicalized Rust result bytes"
            )
        canonical = (
            json.dumps(
                payload,
                ensure_ascii=False,
                allow_nan=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            + "\n"
        ).encode()
        output_path.write_bytes(canonical)
        validate_release_profile_report(
            json.loads(output_path.read_text(encoding="utf-8")),
            canonical_bytes=output_path.read_bytes(),
            expected_profile_sha256="a" * 64,
        )
        _assert_canonicalization_rejections(root)


def _assert_canonicalization_rejections(root: Path) -> None:
    for name, raw, expected in (
        ("missing", None, "no critical area result"),
        ("wrong-area", b'{"area":"documentation"}\n', "identity mismatch"),
        (
            "duplicate-key",
            b'{"area":"rust_interop","area":"rust_interop"}\n',
            "duplicate object key",
        ),
    ):
        result_root = root / name
        result_root.mkdir()
        if raw is not None:
            (result_root / CRITICAL_RESULTS["rust_interop"]).write_bytes(raw)
        _require_governance_rejection(
            lambda result_root=result_root: canonicalize_custodied_results(
                result_root
            ),
            expected,
        )


def _require_governance_rejection(
    action: Callable[[], None],
    expected: str,
) -> None:
    try:
        action()
    except GovernanceError as exc:
        if expected not in str(exc):
            raise AssertionError(f"unexpected governance rejection: {exc}") from exc
        return
    raise AssertionError(f"release custody mutation passed: {expected}")

"""Mutation tests for the one-time schema-v2 preview epoch bootstrap."""

from __future__ import annotations

import copy
import tempfile
from pathlib import Path
from unittest.mock import patch

from . import schema_bootstrap as bootstrap_module
from .common import GovernanceError
from .common import write_canonical_json
from .schema_bootstrap import (
    BOOTSTRAP_GENERATION,
    LEGACY_INDEX_SHA256,
    LEGACY_INDEX_SIZE_BYTES,
    build_preview_epoch,
    expected_asset_names,
    materialize_bootstrap_evidence,
    resolve_distinct_approvers,
    validate_bootstrap_evidence,
)
from .schema_contracts import COMMIT, SHA_A, SHA_B, SHA_C, SHA_D, release_record


def main() -> int:
    alpha = wrapper("0.1.0-alpha.2", "alpha")
    beta = wrapper("0.1.0-beta.15", "beta")
    index = build_preview_epoch(
        legacy_index_sha256=LEGACY_INDEX_SHA256,
        legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
        alpha_wrapper=alpha,
        beta_wrapper=beta,
    )
    assert index["generation"] == BOOTSTRAP_GENERATION
    assert index["ga_status"] == "preview"
    assert set(index["channels"]) == {"alpha", "beta"}
    assert "stable" not in index["channels"]
    evidence = valid_evidence()
    assert validate_bootstrap_evidence(evidence) == evidence
    stage = copy.deepcopy(evidence)
    stage["stage"] = "alpha-assets"
    for key in ("alpha_evidence", "beta", "index", "public_smoke"):
        del stage[key]
    assert validate_bootstrap_evidence(stage) == stage
    stage_with_beta = copy.deepcopy(stage)
    stage_with_beta["beta"] = release_evidence("0.1.0-beta.15")
    expect_failure(
        lambda: validate_bootstrap_evidence(stage_with_beta),
        "alpha stage with beta evidence",
    )
    invalid_stage = copy.deepcopy(stage)
    invalid_stage["stage"] = "unknown-stage"
    expect_failure(
        lambda: validate_bootstrap_evidence(invalid_stage),
        "unknown bootstrap evidence stage",
    )
    approvals = [
        {
            "state": "approved",
            "environments": [{"name": "stable-release"}],
            "user": {"login": "release-reviewer"},
        }
    ]
    assert (
        resolve_distinct_approvers(approvals, initiator="release-initiator")
        == ["release-reviewer"]
    )
    multi_approvals = approvals + [
        {
            "state": "approved",
            "environments": [{"name": "stable-release"}],
            "user": {"login": "second-reviewer"},
        }
    ]
    assert resolve_distinct_approvers(
        multi_approvals,
        initiator="release-initiator",
    ) == ["release-reviewer", "second-reviewer"]
    expect_failure(
        lambda: build_preview_epoch(
            legacy_index_sha256="f" * 64,
            legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
            alpha_wrapper=alpha,
            beta_wrapper=beta,
        ),
        "opaque legacy drift",
    )
    expect_failure(
        lambda: build_preview_epoch(
            legacy_index_sha256=LEGACY_INDEX_SHA256,
            legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES - 1,
            alpha_wrapper=alpha,
            beta_wrapper=beta,
        ),
        "opaque legacy size drift",
    )
    expect_failure(
        lambda: build_preview_epoch(
            legacy_index_sha256=LEGACY_INDEX_SHA256,
            legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
            alpha_wrapper=beta,
            beta_wrapper=beta,
        ),
        "channel swap",
    )
    extra_wrapper_key = copy.deepcopy(alpha)
    extra_wrapper_key["unexpected"] = True
    expect_failure(
        lambda: build_preview_epoch(
            legacy_index_sha256=LEGACY_INDEX_SHA256,
            legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
            alpha_wrapper=extra_wrapper_key,
            beta_wrapper=beta,
        ),
        "release wrapper extra key",
    )
    withdrawn_alpha = copy.deepcopy(alpha)
    withdrawn_alpha["release"]["status"] = "withdrawn"
    withdrawn_alpha["release"]["incident_id"] = "inc-2026-001"
    expect_failure(
        lambda: build_preview_epoch(
            legacy_index_sha256=LEGACY_INDEX_SHA256,
            legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
            alpha_wrapper=withdrawn_alpha,
            beta_wrapper=beta,
        ),
        "withdrawn bootstrap release",
    )
    expect_failure(
        lambda: resolve_distinct_approvers(
            [
                {
                    "state": "approved",
                    "environments": [{"name": "stable-release"}],
                    "user": {"login": "Release-Initiator"},
                }
            ],
            initiator="release-initiator",
        ),
        "case-insensitive self approval",
    )
    expect_failure(
        lambda: resolve_distinct_approvers(
            approvals,
            initiator="",
        ),
        "empty approval initiator",
    )
    empty_login_approval = copy.deepcopy(approvals)
    empty_login_approval[0]["user"]["login"] = ""
    expect_failure(
        lambda: resolve_distinct_approvers(
            empty_login_approval,
            initiator="release-initiator",
        ),
        "empty approval login",
    )
    for rejected, label in (
        ([], "empty approval history"),
        (
            [
                {
                    "state": "rejected",
                    "environments": [{"name": "stable-release"}],
                    "user": {"login": "release-reviewer"},
                }
            ],
            "non-approved review",
        ),
        (
            [
                {
                    "state": "approved",
                    "environments": [{"name": "preview-release"}],
                    "user": {"login": "release-reviewer"},
                }
            ],
            "wrong approval environment",
        ),
    ):
        expect_failure(
            lambda rejected=rejected: resolve_distinct_approvers(
                rejected,
                initiator="release-initiator",
            ),
            label,
        )
    mutations = (
        lambda value: value.update({"unexpected": True}),
        lambda value: value.update({"schema_version": 0}),
        lambda value: value.update({"operation": "preview-publication"}),
        lambda value: value.update({"run_id": 0}),
        lambda value: value.update({"run_attempt": 0}),
        lambda value: value.update({"initiator": ""}),
        lambda value: value.update({"approvers": []}),
        lambda value: value.update({"approvers": "abc"}),
        lambda value: value.update({"approvers": [""]}),
        lambda value: value.update({"approvers": [value["initiator"]]}),
        lambda value: value["legacy_index"].update({"unexpected": True}),
        lambda value: value["legacy_index"].update({"sha256": SHA_A}),
        lambda value: value["legacy_index"].update({"size_bytes": 104}),
        lambda value: value["index"].update({"unexpected": True}),
        lambda value: value["index"].update({"generation": 2}),
        lambda value: value["index"].update({"sha256": "not-a-digest"}),
        lambda value: value.update({"prepare_summary_sha256": "not-a-digest"}),
        lambda value: value["alpha_evidence"].update({"sha256": "not-a-digest"}),
        lambda value: value["alpha_evidence"].update({"unexpected": True}),
        lambda value: value["alpha_evidence"].update({"run_id": 0}),
        lambda value: value["alpha_evidence"].update({"run_attempt": 0}),
        lambda value: value["alpha_evidence"].update({"initiator": ""}),
        lambda value: value["alpha_evidence"].update({"approvers": []}),
        lambda value: value["alpha_evidence"].update(
            {"prepare_summary_sha256": "not-a-digest"}
        ),
        lambda value: value.update({"approvers": ["Release-Reviewer", "release-reviewer"]}),
        lambda value: value["alpha"].update({"unexpected": True}),
        lambda value: value.update(
            {"alpha": release_evidence("0.1.0-beta.15")}
        ),
        lambda value: value["alpha"].update({"source_commit": "not-a-commit"}),
        lambda value: value["alpha"].update(
            {"release_record_sha256": "not-a-digest"}
        ),
        lambda value: value["alpha"]["published_assets"].update(
            {
                next(iter(value["alpha"]["published_assets"])): "not-a-digest",
            }
        ),
        lambda value: value["alpha"]["published_assets"].pop(
            next(iter(value["alpha"]["published_assets"]))
        ),
        lambda value: value["alpha"].update(
            {"published_assets": release_evidence("0.1.0-alpha.99")["published_assets"]}
        ),
        lambda value: value["beta"].update({"unexpected": True}),
        lambda value: value.update(
            {"beta": release_evidence("0.1.0-alpha.7")}
        ),
        lambda value: value["beta"].update({"source_commit": "not-a-commit"}),
        lambda value: value["beta"].update(
            {"release_record_sha256": "not-a-digest"}
        ),
        lambda value: value["beta"]["published_assets"].update(
            {
                next(iter(value["beta"]["published_assets"])): "not-a-digest",
            }
        ),
        lambda value: value["beta"]["published_assets"].pop(
            next(iter(value["beta"]["published_assets"]))
        ),
        lambda value: value["beta"].update(
            {"published_assets": release_evidence("0.1.0-beta.99")["published_assets"]}
        ),
        lambda value: value["public_smoke"].append(
            copy.deepcopy(value["public_smoke"][0])
        ),
        lambda value: value["public_smoke"].pop(),
        lambda value: (
            value["public_smoke"][1].update(
                {
                    "id": value["public_smoke"][0]["id"],
                    "sha256": SHA_A,
                }
            )
        ),
        lambda value: value["public_smoke"][0].update({"id": "unknown-smoke"}),
        lambda value: value["public_smoke"][0].update({"unexpected": True}),
        lambda value: value["public_smoke"][0].update({"status": "fail"}),
        lambda value: value["public_smoke"][0].update({"sha256": "not-a-digest"}),
    )
    for index_value, mutation in enumerate(mutations):
        changed = copy.deepcopy(evidence)
        mutation(changed)
        expect_failure(
            lambda changed=changed: validate_bootstrap_evidence(changed),
            f"evidence mutation {index_value}",
        )
    test_materializer()
    print("schema-v2 preview epoch bootstrap self-test: PASS")
    return 0


def wrapper(version: str, channel: str) -> dict[str, object]:
    record = release_record(channel)
    record["source_commit"] = COMMIT
    return {"version": version, "release": record}


def valid_evidence() -> dict[str, object]:
    return {
        "schema_version": 2,
        "operation": "schema-epoch-bootstrap",
        "stage": "preview-index",
        "run_id": 42,
        "run_attempt": 1,
        "initiator": "release-initiator",
        "approvers": ["release-reviewer"],
        "prepare_summary_sha256": SHA_A,
        "legacy_index": {
            "sha256": LEGACY_INDEX_SHA256,
            "size_bytes": LEGACY_INDEX_SIZE_BYTES,
        },
        "alpha": release_evidence("0.1.0-alpha.2"),
        "alpha_evidence": {
            "sha256": SHA_A,
            "run_id": 41,
            "run_attempt": 1,
            "initiator": "alpha-initiator",
            "approvers": ["alpha-reviewer"],
            "prepare_summary_sha256": SHA_B,
        },
        "beta": release_evidence("0.1.0-beta.15"),
        "index": {"generation": BOOTSTRAP_GENERATION, "sha256": SHA_D},
        "public_smoke": [
            {"id": smoke_id, "status": "pass", "sha256": SHA_C}
            for smoke_id in (
                "dispatcher-default",
                "dispatcher-stable-rejection",
                "governance-index",
                "installed-self-update",
            )
        ],
    }


def release_evidence(version: str) -> dict[str, object]:
    return {
        "version": version,
        "source_commit": COMMIT,
        "release_record_sha256": SHA_A,
        "published_assets": {
            name: SHA_B for name in sorted(expected_asset_names(version))
        },
    }


def test_materializer() -> None:
    alpha_version = "0.1.0-alpha.2"
    beta_version = "0.1.0-beta.15"
    alpha_record = wrapper(alpha_version, "alpha")
    beta_record = wrapper(beta_version, "beta")
    index = build_preview_epoch(
        legacy_index_sha256=LEGACY_INDEX_SHA256,
        legacy_index_size_bytes=LEGACY_INDEX_SIZE_BYTES,
        alpha_wrapper=alpha_record,
        beta_wrapper=beta_record,
    )
    with tempfile.TemporaryDirectory() as raw_root:
        root = Path(raw_root)
        legacy_path = root / "legacy.json"
        prepare_summary = root / "prepare-summary.json"
        alpha_record_path = root / "alpha-record.json"
        beta_record_path = root / "beta-record.json"
        index_path = root / "channels.json"
        alpha_assets = root / "alpha-assets"
        beta_assets = root / "beta-assets"
        smoke_dir = root / "smoke"
        legacy_path.write_bytes(b"x" * LEGACY_INDEX_SIZE_BYTES)
        prepare_summary.write_bytes(b'{"schema_version":2}\n')
        write_canonical_json(alpha_record_path, alpha_record, refuse_existing=True)
        write_canonical_json(beta_record_path, beta_record, refuse_existing=True)
        write_canonical_json(index_path, index, refuse_existing=True)
        for directory, version in (
            (alpha_assets, alpha_version),
            (beta_assets, beta_version),
        ):
            directory.mkdir()
            for name in expected_asset_names(version):
                (directory / name).write_bytes(name.encode())
        smoke_dir.mkdir()
        for smoke_id in (
            "dispatcher-default",
            "dispatcher-stable-rejection",
            "governance-index",
            "installed-self-update",
        ):
            (smoke_dir / f"{smoke_id}.txt").write_text(
                f"{smoke_id}: pass\n",
                encoding="utf-8",
            )
        alpha_evidence_path = root / "alpha-evidence.json"
        final_evidence_path = root / "final-evidence.json"

        def materialize_alpha(
            out: Path,
            *,
            final_alpha_record: Path = alpha_record_path,
            final_alpha_source: str = COMMIT,
        ) -> dict[str, object]:
            return materialize_bootstrap_evidence(
                stage="alpha-assets",
                run_id=41,
                run_attempt=1,
                initiator="alpha-initiator",
                approvers=["alpha-reviewer"],
                prepare_summary_path=prepare_summary,
                legacy_index_path=legacy_path,
                alpha_version=alpha_version,
                alpha_source_commit=final_alpha_source,
                alpha_record_path=final_alpha_record,
                alpha_assets_dir=alpha_assets,
                out=out,
            )

        def materialize_final(
            out: Path,
            *,
            final_beta_record: Path = beta_record_path,
            final_smoke_dir: Path = smoke_dir,
            final_alpha_evidence: Path = alpha_evidence_path,
        ) -> dict[str, object]:
            return materialize_bootstrap_evidence(
                stage="preview-index",
                run_id=42,
                run_attempt=1,
                initiator="beta-initiator",
                approvers=["beta-reviewer", "second-reviewer"],
                prepare_summary_path=prepare_summary,
                legacy_index_path=legacy_path,
                alpha_version=alpha_version,
                alpha_source_commit=COMMIT,
                alpha_record_path=alpha_record_path,
                alpha_assets_dir=alpha_assets,
                out=out,
                beta_version=beta_version,
                beta_source_commit=COMMIT,
                beta_record_path=final_beta_record,
                beta_assets_dir=beta_assets,
                index_path=index_path,
                smoke_dir=final_smoke_dir,
                alpha_evidence_path=final_alpha_evidence,
            )

        with patch.object(
            bootstrap_module,
            "sha256_bytes",
            return_value=LEGACY_INDEX_SHA256,
        ):
            materialize_alpha(alpha_evidence_path)
            final = materialize_final(final_evidence_path)
            assert final["alpha_evidence"]["run_id"] == 41
            withdrawn_record = copy.deepcopy(alpha_record)
            withdrawn_record["release"]["status"] = "withdrawn"
            withdrawn_record["release"]["incident_id"] = "inc-2026-001"
            withdrawn_record_path = root / "withdrawn-alpha-record.json"
            write_canonical_json(
                withdrawn_record_path,
                withdrawn_record,
                refuse_existing=True,
            )
            expect_failure(
                lambda: materialize_alpha(
                    root / "withdrawn-alpha-evidence.json",
                    final_alpha_record=withdrawn_record_path,
                ),
                "producer rejects withdrawn alpha release",
            )
            expect_failure(
                lambda: materialize_alpha(
                    root / "source-mismatch-alpha-evidence.json",
                    final_alpha_source="f" * 40,
                ),
                "producer rejects record and source-commit disagreement",
            )
            expect_failure(
                lambda: materialize_bootstrap_evidence(
                    stage="preview-index",
                    run_id=43,
                    run_attempt=1,
                    initiator="beta-initiator",
                    approvers=["beta-reviewer"],
                    prepare_summary_path=prepare_summary,
                    legacy_index_path=legacy_path,
                    alpha_version=alpha_version,
                    alpha_source_commit=COMMIT,
                    alpha_record_path=alpha_record_path,
                    alpha_assets_dir=alpha_assets,
                    out=final_evidence_path,
                ),
                "preview producer missing required inputs",
            )
            expect_failure(
                lambda: materialize_final(final_evidence_path),
                "producer refuses existing evidence",
            )
            expect_failure(
                lambda: materialize_final(
                    root / "wrong-alpha-stage-evidence.json",
                    final_alpha_evidence=final_evidence_path,
                ),
                "producer rejects preview-index evidence as the alpha stage",
            )
            other_alpha_version = "0.1.0-alpha.3"
            other_alpha_record_path = root / "other-alpha-record.json"
            other_alpha_assets = root / "other-alpha-assets"
            other_alpha_evidence = root / "other-alpha-evidence.json"
            write_canonical_json(
                other_alpha_record_path,
                wrapper(other_alpha_version, "alpha"),
                refuse_existing=True,
            )
            other_alpha_assets.mkdir()
            for name in expected_asset_names(other_alpha_version):
                (other_alpha_assets / name).write_bytes(name.encode())
            expect_failure(
                lambda: materialize_alpha(
                    root / "version-mismatch-alpha-evidence.json",
                    final_alpha_record=other_alpha_record_path,
                ),
                "producer rejects record and evidence version disagreement",
            )
            materialize_bootstrap_evidence(
                stage="alpha-assets",
                run_id=44,
                run_attempt=1,
                initiator="other-alpha-initiator",
                approvers=["other-alpha-reviewer"],
                prepare_summary_path=prepare_summary,
                legacy_index_path=legacy_path,
                alpha_version=other_alpha_version,
                alpha_source_commit=COMMIT,
                alpha_record_path=other_alpha_record_path,
                alpha_assets_dir=other_alpha_assets,
                out=other_alpha_evidence,
            )
            expect_failure(
                lambda: materialize_final(
                    root / "wrong-alpha-release-evidence.json",
                    final_alpha_evidence=other_alpha_evidence,
                ),
                "producer rejects evidence for a different alpha release",
            )
            unexpected_asset = alpha_assets / "unexpected"
            unexpected_asset.write_bytes(b"unexpected")
            expect_failure(
                lambda: materialize_final(root / "unexpected-asset-evidence.json"),
                "producer rejects unexpected alpha asset",
            )
            unexpected_asset.unlink()
            missing_smoke = smoke_dir / "governance-index.txt"
            missing_smoke.unlink()
            expect_failure(
                lambda: materialize_final(root / "missing-smoke-evidence.json"),
                "producer rejects missing smoke record",
            )
            missing_smoke.write_text("governance-index: pass\n", encoding="utf-8")
            changed_beta = copy.deepcopy(beta_record)
            changed_beta["release"]["installer_sha256"] = SHA_D
            changed_beta_path = root / "changed-beta-record.json"
            write_canonical_json(
                changed_beta_path,
                changed_beta,
                refuse_existing=True,
            )
            expect_failure(
                lambda: materialize_final(
                    root / "mismatched-index-evidence.json",
                    final_beta_record=changed_beta_path,
                ),
                "producer rejects index and record disagreement",
            )


def expect_failure(callback: object, label: str) -> None:
    try:
        callback()  # type: ignore[operator]
    except (GovernanceError, OSError):
        return
    raise AssertionError(f"{label} unexpectedly passed")


if __name__ == "__main__":
    raise SystemExit(main())

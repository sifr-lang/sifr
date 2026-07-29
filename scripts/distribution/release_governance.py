#!/usr/bin/env python3
"""Generate and validate canonical stable release-governance artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance import (  # noqa: E402
    GovernanceError,
    generate_site_release_facts,
    materialize_incident_prepare,
    materialize_stable_prepare,
    validate_bootstrap_evidence,
    validate_drill_evidence,
    validate_incident_mutation_evidence,
    validate_incident_prepare_summary,
    validate_incident_request,
    validate_incident_signoff,
    validate_install_receipt,
    validate_qualification_artifact_index,
    validate_release_index,
    validate_release_index_transition,
    validate_release_plan,
    validate_release_profile_report,
    validate_release_signoff,
    validate_self_update_plan,
    validate_self_version,
    validate_site_publication_facts,
    validate_site_release_facts,
    validate_stable_mutation_evidence,
    validate_stable_prepare_summary,
)
from governance.common import (  # noqa: E402
    load_json_strict,
    require_sha256,
    sha256_bytes,
    sha256_file,
    write_canonical_json,
)
from governance.approval_waiver import (  # noqa: E402
    validate_repository_approval_waiver,
    validate_single_maintainer_waiver,
)
from governance.incident_evidence import validate_incident_evidence_commit  # noqa: E402
from governance.incident_planner import materialize_incident_mutation  # noqa: E402
from governance.planner import (  # noqa: E402
    materialize_stable_plan,
    stage_stable_support_claims,
)
from governance.release_index import (  # noqa: E402
    propose_preview_release,
    validate_release_record,
)
from governance.schema_bootstrap import (  # noqa: E402
    build_preview_epoch,
    resolve_approval_decision,
)
from governance.stable_planner import materialize_stable_mutation  # noqa: E402


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    validate = commands.add_parser("validate")
    validate.add_argument(
        "--kind",
        required=True,
        choices=(
            "release-index",
            "protected-drill-evidence",
            "schema-bootstrap-evidence",
            "single-maintainer-approval-waiver",
            "release-plan",
            "release-signoff",
            "site-facts",
            "incident-request",
            "incident-signoff",
            "incident-index-mutation-evidence",
            "incident-publication-prepare",
            "release-profile-report",
            "qualification-artifact-index",
            "install-receipt",
            "self-update-plan",
            "self-version",
            "site-publication-facts",
            "stable-index-mutation-evidence",
            "stable-publication-prepare",
        ),
    )
    validate.add_argument("--input", required=True)
    validate.add_argument("--previous")
    validate.add_argument("--live-index")
    validate.add_argument(
        "--expected-drill-scenario",
        choices=("publication", "rollback", "first-ga"),
    )
    validate.add_argument("--require-canonical", action="store_true")

    generate_index = commands.add_parser("generate-release-index")
    generate_index.add_argument("--out", required=True)
    generate_index.add_argument("--generation", required=True, type=int)
    generate_index.add_argument("--ga-status", required=True, choices=("preview", "active"))
    generate_index.add_argument(
        "--release",
        action="append",
        required=True,
        help="Path to {version, release} JSON; repeat for every retained release.",
    )
    generate_index.add_argument(
        "--channel",
        action="append",
        required=True,
        help="Channel mapping in name=version form.",
    )

    update = commands.add_parser("update-preview-index")
    update.add_argument("--current", required=True)
    update.add_argument("--out", required=True)
    update.add_argument("--channel", required=True, choices=("alpha", "beta"))
    update.add_argument("--release", required=True, help="Path to {version, release} JSON.")
    update.add_argument("--expected-generation", required=True, type=int)
    update.add_argument("--expected-sha256", required=True)
    update.add_argument("--proposed-generation", required=True, type=int)

    plan = commands.add_parser("generate-release-plan")
    plan.add_argument("--spec", required=True)
    plan.add_argument("--out", required=True)
    plan.add_argument("--live-index")

    stable_plan = commands.add_parser("plan-stable-release")
    stable_plan.add_argument("--spec", required=True)
    stable_plan.add_argument("--source-root", default=str(REPO_ROOT))
    stable_plan.add_argument("--source-ref", required=True)
    stable_plan.add_argument("--live-index", required=True)
    stable_plan.add_argument("--release-report", required=True)
    stable_plan.add_argument("--qualification-index", required=True)
    stable_plan.add_argument("--artifact-root", required=True)
    stable_plan.add_argument("--stable-support-claims", required=True)
    stable_plan.add_argument("--rust-validation-report", required=True)
    stable_plan.add_argument("--documentation-report", required=True)
    stable_plan.add_argument("--release-notes", required=True)
    stable_plan.add_argument("--out", required=True)

    stage_claims = commands.add_parser("stage-stable-support-claims")
    stage_claims.add_argument("--source-root", default=str(REPO_ROOT))
    stage_claims.add_argument("--out", required=True)

    record = commands.add_parser("build-release-record")
    record.add_argument("--version", required=True)
    record.add_argument("--channel", required=True, choices=("alpha", "beta", "stable"))
    record.add_argument("--source-commit", required=True)
    record.add_argument("--installer", required=True)
    record.add_argument("--artifact-dir", required=True)
    record.add_argument("--out", required=True)

    site = commands.add_parser("generate-site-facts")
    site.add_argument("--release-index", required=True)
    site.add_argument("--source-plan-sha256", required=True)
    site.add_argument("--release-index-sha256", required=True)
    site.add_argument("--dispatcher", action="append", required=True)
    site.add_argument("--out", required=True)

    for command_name in ("generate-incident-request", "generate-incident-signoff"):
        command = commands.add_parser(command_name)
        command.add_argument("--spec", required=True)
        command.add_argument("--out", required=True)
        if command_name == "generate-incident-request":
            command.add_argument("--live-index", required=True)
            command.add_argument("--withdrawal-evidence", required=True)
            command.add_argument("--affected-plan", required=True)
            command.add_argument("--rollback-plan")
    incident_index = commands.add_parser("plan-incident-index")
    incident_index.add_argument("--request", required=True)
    incident_index.add_argument("--live-index", required=True)
    incident_index.add_argument("--affected-plan", required=True)
    incident_index.add_argument("--successor-plan", required=True)
    incident_index.add_argument("--expected-generation", required=True, type=int)
    incident_index.add_argument("--expected-sha256", required=True)
    incident_index.add_argument("--proposed-generation", required=True, type=int)
    incident_index.add_argument("--out", required=True)
    evidence_commit = commands.add_parser("validate-incident-evidence-commit")
    evidence_commit.add_argument("--repository", default=str(REPO_ROOT))
    evidence_commit.add_argument("--base", required=True)
    evidence_commit.add_argument("--head", required=True)
    evidence_commit.add_argument("--request-path", required=True)
    evidence_commit.add_argument("--evidence-path", required=True)
    bootstrap_index = commands.add_parser("generate-schema-bootstrap-index")
    bootstrap_index.add_argument("--current", required=True)
    bootstrap_index.add_argument("--alpha-release", required=True)
    bootstrap_index.add_argument("--beta-release", required=True)
    bootstrap_index.add_argument("--out", required=True)
    stable_index = commands.add_parser("plan-stable-index")
    stable_index.add_argument("--plan", required=True)
    stable_index.add_argument("--live-index", required=True)
    stable_index.add_argument("--expected-generation", required=True, type=int)
    stable_index.add_argument("--expected-sha256", required=True)
    stable_index.add_argument("--proposed-generation", required=True, type=int)
    stable_index.add_argument("--out", required=True)
    stable_prepare = commands.add_parser("prepare-stable-publication")
    stable_prepare.add_argument(
        "--operation",
        required=True,
        choices=("ga-activation", "normal", "incident-roll-forward"),
    )
    stable_prepare.add_argument("--mode", required=True, choices=("initial", "resume"))
    stable_prepare.add_argument("--evidence-root", required=True)
    stable_prepare.add_argument("--evidence-commit", required=True)
    stable_prepare.add_argument("--candidate-path", required=True)
    stable_prepare.add_argument("--expected-plan-sha256", required=True)
    stable_prepare.add_argument("--source-root", required=True)
    stable_prepare.add_argument("--live-index", required=True)
    stable_prepare.add_argument("--snapshot-root", required=True)
    stable_prepare.add_argument("--artifact-root", required=True)
    stable_prepare.add_argument("--incident-request")
    stable_prepare.add_argument("--affected-plan")
    stable_prepare.add_argument("--proposed-generation", required=True, type=int)
    stable_prepare.add_argument("--out", required=True)
    incident_prepare = commands.add_parser("prepare-incident-publication")
    incident_prepare.add_argument(
        "--operation",
        required=True,
        choices=("rollback", "incident-roll-forward"),
    )
    incident_prepare.add_argument("--mode", required=True, choices=("initial", "resume"))
    incident_prepare.add_argument("--governance-root", required=True)
    incident_prepare.add_argument("--incident-root", required=True)
    incident_prepare.add_argument("--incident-commit", required=True)
    incident_prepare.add_argument("--incident-path", required=True)
    incident_prepare.add_argument("--expected-request-sha256", required=True)
    incident_prepare.add_argument("--live-index", required=True)
    incident_prepare.add_argument("--snapshot-root", required=True)
    incident_prepare.add_argument("--proposed-generation", required=True, type=int)
    incident_prepare.add_argument("--candidate-root")
    incident_prepare.add_argument("--candidate-commit", default="")
    incident_prepare.add_argument("--candidate-path", default="")
    incident_prepare.add_argument("--expected-plan-sha256", default="")
    incident_prepare.add_argument("--source-root")
    incident_prepare.add_argument("--artifact-root")
    incident_prepare.add_argument("--out", required=True)
    approvers = commands.add_parser("resolve-publication-approvers")
    approvers.add_argument("--approvals", required=True)
    approvers.add_argument("--initiator", required=True)
    approvers.add_argument("--environment", default="stable-release")
    approvers.add_argument("--repository", default="sifr-lang/sifr")
    approvers.add_argument("--operation", default="")
    approvers.add_argument("--single-maintainer-waiver")
    approvers.add_argument("--expected-waiver-sha256")
    approvers.add_argument("--include-policy", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "validate":
            validate_command(args)
        elif args.command == "generate-release-index":
            generate_release_index(args)
        elif args.command == "update-preview-index":
            update_preview_index(args)
        elif args.command == "build-release-record":
            build_release_record(args)
        elif args.command == "generate-release-plan":
            generate_release_plan(args)
        elif args.command == "plan-stable-release":
            plan_stable_release(args)
        elif args.command == "stage-stable-support-claims":
            stage_support_claims(args)
        elif args.command == "generate-site-facts":
            generate_site_facts(args)
        elif args.command == "generate-incident-request":
            generate_incident(args)
        elif args.command == "generate-incident-signoff":
            generate_incident_signoff(args)
        elif args.command == "plan-incident-index":
            plan_incident_index(args)
        elif args.command == "validate-incident-evidence-commit":
            validate_incident_evidence(args)
        elif args.command == "generate-schema-bootstrap-index":
            generate_schema_bootstrap_index(args)
        elif args.command == "plan-stable-index":
            plan_stable_index(args)
        elif args.command == "prepare-stable-publication":
            prepare_stable_publication(args)
        elif args.command == "prepare-incident-publication":
            prepare_incident_publication(args)
        elif args.command == "resolve-publication-approvers":
            resolve_publication_approvers(args)
        else:
            raise AssertionError(args.command)
    except GovernanceError as exc:
        print(f"release-governance: {exc}", file=sys.stderr)
        return 2
    return 0


def validate_command(args: argparse.Namespace) -> None:
    path = Path(args.input)
    payload = load_json_strict(path, require_canonical=args.require_canonical)
    validators: dict[str, Callable[[Any], Any]] = {
        "release-index": validate_release_index,
        "protected-drill-evidence": validate_drill_evidence,
        "schema-bootstrap-evidence": validate_bootstrap_evidence,
        "single-maintainer-approval-waiver": (
            lambda payload: validate_repository_approval_waiver(
                payload, require_unexpired=True
            )
        ),
        "release-plan": validate_release_plan,
        "release-signoff": validate_release_signoff,
        "site-facts": validate_site_release_facts,
        "incident-request": validate_incident_request,
        "incident-signoff": validate_incident_signoff,
        "incident-index-mutation-evidence": validate_incident_mutation_evidence,
        "incident-publication-prepare": validate_incident_prepare_summary,
        "release-profile-report": validate_release_profile_report,
        "qualification-artifact-index": validate_qualification_artifact_index,
        "install-receipt": validate_install_receipt,
        "self-update-plan": validate_self_update_plan,
        "self-version": validate_self_version,
        "site-publication-facts": validate_site_publication_facts,
        "stable-index-mutation-evidence": validate_stable_mutation_evidence,
        "stable-publication-prepare": validate_stable_prepare_summary,
    }
    if args.kind == "protected-drill-evidence" and args.expected_drill_scenario:
        validate_drill_evidence(
            payload,
            expected_scenarios=(args.expected_drill_scenario,),
        )
    elif args.kind == "release-index" and args.previous:
        validate_release_index_transition(load_json_strict(Path(args.previous)), payload)
    elif args.kind == "incident-request" and args.live_index:
        validate_incident_request(payload, live_index=load_json_strict(Path(args.live_index)))
    elif args.kind == "release-plan" and args.live_index:
        validate_release_plan(payload, active_index=load_json_strict(Path(args.live_index)))
    elif args.kind == "site-facts" and args.live_index:
        validate_site_release_facts(payload, governed_index=load_json_strict(Path(args.live_index)))
    elif args.kind == "release-profile-report":
        validators[args.kind](payload, canonical_bytes=path.read_bytes())
    else:
        validators[args.kind](payload)
    print(f"release-governance validation ok: kind={args.kind} input={path}")


def generate_release_index(args: argparse.Namespace) -> None:
    channels = parse_assignments(args.channel, label="channel")
    releases: dict[str, Any] = {}
    for path_text in args.release:
        version, release = load_release(Path(path_text))
        if version in releases:
            raise GovernanceError(f"duplicate release version: {version}")
        releases[version] = release
    payload = {
        "schema_version": 2,
        "generation": args.generation,
        "ga_status": args.ga_status,
        "channels": dict(sorted(channels.items())),
        "releases": dict(sorted(releases.items())),
    }
    validate_release_index(payload)
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


def update_preview_index(args: argparse.Namespace) -> None:
    current_path = Path(args.current)
    current = validate_release_index(load_json_strict(current_path, require_canonical=True))
    if current["generation"] != args.expected_generation:
        raise GovernanceError("current index generation does not match expected generation")
    require_sha256(args.expected_sha256, "--expected-sha256")
    if sha256_bytes(current_path.read_bytes()) != args.expected_sha256:
        raise GovernanceError("current index digest does not match expected digest")
    version, release = load_release(Path(args.release))
    proposed = propose_preview_release(
        current,
        channel=args.channel,
        version=version,
        release_value=release,
        proposed_generation=args.proposed_generation,
    )
    write_canonical_json(Path(args.out), proposed, refuse_existing=True)


def generate_site_facts(args: argparse.Namespace) -> None:
    facts = generate_site_release_facts(
        load_json_strict(Path(args.release_index), require_canonical=True),
        source_plan_sha256=args.source_plan_sha256,
        release_index_sha256=args.release_index_sha256,
        dispatchers=parse_assignments(args.dispatcher, label="dispatcher"),
    )
    write_canonical_json(Path(args.out), facts, refuse_existing=True)


def generate_release_plan(args: argparse.Namespace) -> None:
    payload = load_json_strict(Path(args.spec))
    live_index = load_json_strict(Path(args.live_index)) if args.live_index else None
    validate_release_plan(payload, active_index=live_index)
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


def plan_stable_release(args: argparse.Namespace) -> None:
    output = Path(args.out).resolve()
    try:
        output.relative_to(REPO_ROOT.resolve())
    except ValueError:
        pass
    else:
        raise GovernanceError("stable release evidence output must be outside the repository")
    if output.exists():
        raise GovernanceError(f"refusing to overwrite stable release evidence: {output}")
    payload = materialize_stable_plan(
        plan_spec=Path(args.spec),
        source_root=Path(args.source_root),
        source_ref=args.source_ref,
        active_index_path=Path(args.live_index),
        release_report_path=Path(args.release_report),
        qualification_index_path=Path(args.qualification_index),
        artifact_root=Path(args.artifact_root),
        stable_support_claims_path=Path(args.stable_support_claims),
        rust_validation_report_path=Path(args.rust_validation_report),
        documentation_report_path=Path(args.documentation_report),
        release_notes_path=Path(args.release_notes),
    )
    write_canonical_json(output, payload, refuse_existing=True)


def stage_support_claims(args: argparse.Namespace) -> None:
    stage_stable_support_claims(
        source_root=Path(args.source_root),
        output_path=Path(args.out),
    )
    print(f"stable support claims evidence staged: {Path(args.out)}")


def build_release_record(args: argparse.Namespace) -> None:
    version = args.version
    installer = Path(args.installer)
    artifact_dir = Path(args.artifact_dir)
    targets: dict[str, dict[str, str]] = {}
    from governance.common import TARGETS, require_commit, version_channel

    if version_channel(version, "--version") != args.channel:
        raise GovernanceError("release record version does not match --channel")
    require_commit(args.source_commit, "--source-commit")
    if not installer.is_file():
        raise GovernanceError(f"installer does not exist: {installer}")
    for target in TARGETS:
        archive = artifact_dir / f"sifr-{version}-{target}.tar.gz"
        if not archive.is_file():
            raise GovernanceError(f"release archive does not exist: {archive}")
        manifest = subprocess.run(
            ["tar", "-xOf", str(archive), "sysroot.toml"],
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if manifest.returncode != 0:
            raise GovernanceError(f"{archive}: could not read sysroot.toml")
        match = re.search(
            r'^"sysroot-content-sha256" = "([0-9a-f]{64})"$',
            manifest.stdout,
            re.MULTILINE,
        )
        if match is None:
            raise GovernanceError(f"{archive}: missing sysroot-content-sha256")
        targets[target] = {
            "artifact_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
            "sysroot_content_sha256": match.group(1),
        }
    wrapper = {
        "version": version,
        "release": {
            "channel": args.channel,
            "status": "active",
            "source_commit": args.source_commit,
            "installer_sha256": hashlib.sha256(installer.read_bytes()).hexdigest(),
            "targets": targets,
        },
    }
    validate_release_record(wrapper["release"], version=version)
    write_canonical_json(Path(args.out), wrapper, refuse_existing=True)


def generate_incident(args: argparse.Namespace) -> None:
    spec_path = Path(args.spec).resolve()
    evidence_path = Path(args.withdrawal_evidence).resolve()
    output = Path(args.out).resolve()
    _require_clean_external_incident_directory(
        output=output,
        spec_path=spec_path,
        evidence_path=evidence_path,
    )
    payload = validate_incident_request(
        load_json_strict(spec_path, require_canonical=True)
    )
    affected_plan_path = Path(args.affected_plan)
    affected_plan = validate_release_plan(
        load_json_strict(affected_plan_path, require_canonical=True)
    )
    approved = {affected_plan["version"]: sha256_file(affected_plan_path)}
    if payload.get("affected_release") != {
        "version": affected_plan["version"],
        "plan_sha256": approved[affected_plan["version"]],
    }:
        raise GovernanceError("incident request does not bind the exact affected plan")
    if payload.get("withdrawal", {}).get("evidence_sha256") != sha256_file(evidence_path):
        raise GovernanceError("incident request does not bind the withdrawal evidence bytes")
    if payload.get("operation") == "rollback":
        if not args.rollback_plan:
            raise GovernanceError("rollback request generation requires --rollback-plan")
        rollback_plan_path = Path(args.rollback_plan)
        rollback_plan = validate_release_plan(
            load_json_strict(rollback_plan_path, require_canonical=True)
        )
        approved[rollback_plan["version"]] = sha256_file(rollback_plan_path)
    elif args.rollback_plan:
        raise GovernanceError("incident-roll-forward request must not supply --rollback-plan")
    validate_incident_request(
        payload,
        live_index=load_json_strict(Path(args.live_index), require_canonical=True),
        approved_plan_digests=approved,
    )
    write_canonical_json(output, payload, refuse_existing=True)


def generate_incident_signoff(args: argparse.Namespace) -> None:
    payload = load_json_strict(Path(args.spec))
    validate_incident_signoff(payload)
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


def plan_incident_index(args: argparse.Namespace) -> None:
    mutation = materialize_incident_mutation(
        request_path=Path(args.request),
        live_index_path=Path(args.live_index),
        affected_plan_path=Path(args.affected_plan),
        successor_plan_path=Path(args.successor_plan),
        expected_generation=args.expected_generation,
        expected_sha256=args.expected_sha256,
        proposed_generation=args.proposed_generation,
    )
    write_canonical_json(Path(args.out), mutation.proposed_index, refuse_existing=True)


def validate_incident_evidence(args: argparse.Namespace) -> None:
    request = validate_incident_evidence_commit(
        repository=Path(args.repository),
        base=args.base,
        head=args.head,
        request_path=args.request_path,
        evidence_path=args.evidence_path,
    )
    print(
        "release-governance incident evidence ok: "
        f"incident_id={request['incident_id']} operation={request['operation']}"
    )


def generate_schema_bootstrap_index(args: argparse.Namespace) -> None:
    legacy_bytes = Path(args.current).read_bytes()
    payload = build_preview_epoch(
        legacy_index_sha256=sha256_bytes(legacy_bytes),
        legacy_index_size_bytes=len(legacy_bytes),
        alpha_wrapper=load_json_strict(
            Path(args.alpha_release), require_canonical=True
        ),
        beta_wrapper=load_json_strict(Path(args.beta_release), require_canonical=True),
    )
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


def plan_stable_index(args: argparse.Namespace) -> None:
    mutation = materialize_stable_mutation(
        plan_path=Path(args.plan),
        live_index_path=Path(args.live_index),
        expected_generation=args.expected_generation,
        expected_sha256=args.expected_sha256,
        proposed_generation=args.proposed_generation,
    )
    evidence = mutation.evidence()
    validate_stable_mutation_evidence(evidence)
    write_canonical_json(
        Path(args.out),
        evidence,
        refuse_existing=True,
    )


def prepare_stable_publication(args: argparse.Namespace) -> None:
    summary = materialize_stable_prepare(
        operation=args.operation,
        mode=args.mode,
        evidence_root=Path(args.evidence_root),
        evidence_commit=args.evidence_commit,
        candidate_path=args.candidate_path,
        expected_plan_sha256=args.expected_plan_sha256,
        source_root=Path(args.source_root),
        live_index_path=Path(args.live_index),
        snapshot_root=Path(args.snapshot_root),
        artifact_root=Path(args.artifact_root),
        proposed_generation=args.proposed_generation,
        incident_request_path=(
            Path(args.incident_request) if args.incident_request else None
        ),
        affected_plan_path=(
            Path(args.affected_plan) if args.affected_plan else None
        ),
    )
    write_canonical_json(Path(args.out), summary, refuse_existing=True)


def prepare_incident_publication(args: argparse.Namespace) -> None:
    summary = materialize_incident_prepare(
        operation=args.operation,
        mode=args.mode,
        governance_root=Path(args.governance_root),
        incident_root=Path(args.incident_root),
        incident_commit=args.incident_commit,
        incident_path=args.incident_path,
        expected_request_sha256=args.expected_request_sha256,
        live_index_path=Path(args.live_index),
        snapshot_root=Path(args.snapshot_root),
        proposed_generation=args.proposed_generation,
        candidate_root=Path(args.candidate_root) if args.candidate_root else None,
        candidate_commit=args.candidate_commit,
        candidate_path=args.candidate_path,
        expected_plan_sha256=args.expected_plan_sha256,
        source_root=Path(args.source_root) if args.source_root else None,
        artifact_root=Path(args.artifact_root) if args.artifact_root else None,
    )
    write_canonical_json(Path(args.out), summary, refuse_existing=True)


def resolve_publication_approvers(args: argparse.Namespace) -> None:
    waiver_path = None
    waiver_sha256 = "none"
    if args.single_maintainer_waiver:
        waiver_path = Path(args.single_maintainer_waiver)
        if args.expected_waiver_sha256 is None:
            raise GovernanceError(
                "expected waiver SHA-256 is required with a waiver path"
            )
        require_sha256(args.expected_waiver_sha256, "expected waiver SHA-256")
        waiver_sha256 = sha256_file(waiver_path)
        if waiver_sha256 != args.expected_waiver_sha256:
            raise GovernanceError("single-maintainer approval waiver digest drifted")
    decision = resolve_approval_decision(
        load_json_strict(Path(args.approvals)),
        initiator=args.initiator,
        environment=args.environment,
        allowed_self_approver=args.initiator if waiver_path else None,
    )
    if decision["mode"] == "single-maintainer-waiver":
        waiver = validate_single_maintainer_waiver(
            load_json_strict(waiver_path, require_canonical=True),
            repository=args.repository,
            environment=args.environment,
            operation=args.operation,
            initiator=args.initiator,
            require_unexpired=True,
        )
        if waiver["owner_login"].casefold() != decision["approvers"][0].casefold():
            raise GovernanceError("waiver owner does not match the recorded approver")
    policy = {
        "mode": decision["mode"],
        "waiver_sha256": (
            waiver_sha256
            if decision["mode"] == "single-maintainer-waiver"
            else "none"
        ),
    }
    output: Any = (
        {"approvers": decision["approvers"], "approval_policy": policy}
        if args.include_policy
        else decision["approvers"]
    )
    print(json.dumps(output, separators=(",", ":")))


def _require_clean_external_incident_directory(
    *,
    output: Path,
    spec_path: Path,
    evidence_path: Path,
) -> None:
    try:
        output.relative_to(REPO_ROOT.resolve())
    except ValueError:
        pass
    else:
        raise GovernanceError("incident request generation output must be outside the repository")
    if not output.parent.is_dir():
        raise GovernanceError("incident request work directory must already exist")
    if spec_path.parent != output.parent or evidence_path.parent != output.parent:
        raise GovernanceError("incident request spec and withdrawal evidence must be in the work directory")
    allowed = {spec_path, evidence_path}
    unexpected = sorted(path.name for path in output.parent.iterdir() if path.resolve() not in allowed)
    if unexpected:
        raise GovernanceError(
            "incident request work directory is not clean: " + ", ".join(unexpected)
        )


def load_release(path: Path) -> tuple[str, dict[str, Any]]:
    wrapper = load_json_strict(path)
    if not isinstance(wrapper, dict) or set(wrapper) != {"version", "release"}:
        raise GovernanceError(f"{path}: expected exactly version and release")
    version = wrapper["version"]
    if not isinstance(version, str):
        raise GovernanceError(f"{path}: version must be a string")
    release = validate_release_record(wrapper["release"], version=version)
    return version, release


def parse_assignments(values: list[str], *, label: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for value in values:
        if "=" not in value:
            raise GovernanceError(f"{label} must use name=value: {value}")
        name, assigned = value.split("=", 1)
        if not name or not assigned or name in result:
            raise GovernanceError(f"invalid or duplicate {label}: {value}")
        result[name] = assigned
    return result


if __name__ == "__main__":
    raise SystemExit(main())

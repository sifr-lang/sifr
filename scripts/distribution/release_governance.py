#!/usr/bin/env python3
"""Generate and validate canonical stable release-governance artifacts."""

from __future__ import annotations

import argparse
import hashlib
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
    validate_incident_request,
    validate_incident_signoff,
    validate_release_index,
    validate_release_index_transition,
    validate_release_plan,
    validate_release_profile_report,
    validate_release_signoff,
    validate_qualification_artifact_index,
    validate_install_receipt,
    validate_self_update_plan,
    validate_self_version,
    validate_site_release_facts,
)
from governance.common import (  # noqa: E402
    canonical_json_bytes,
    load_json_strict,
    require_sha256,
    sha256_bytes,
    write_canonical_json,
)
from governance.release_index import propose_preview_release, validate_release_record  # noqa: E402
from governance.planner import materialize_stable_plan  # noqa: E402


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    validate = commands.add_parser("validate")
    validate.add_argument(
        "--kind",
        required=True,
        choices=(
            "release-index",
            "release-plan",
            "release-signoff",
            "site-facts",
            "incident-request",
            "incident-signoff",
            "release-profile-report",
            "qualification-artifact-index",
            "install-receipt",
            "self-update-plan",
            "self-version",
        ),
    )
    validate.add_argument("--input", required=True)
    validate.add_argument("--previous")
    validate.add_argument("--live-index")
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
        elif args.command == "generate-site-facts":
            generate_site_facts(args)
        elif args.command == "generate-incident-request":
            generate_incident(args)
        elif args.command == "generate-incident-signoff":
            generate_incident_signoff(args)
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
        "release-plan": validate_release_plan,
        "release-signoff": validate_release_signoff,
        "site-facts": validate_site_release_facts,
        "incident-request": validate_incident_request,
        "incident-signoff": validate_incident_signoff,
        "release-profile-report": validate_release_profile_report,
        "qualification-artifact-index": validate_qualification_artifact_index,
        "install-receipt": validate_install_receipt,
        "self-update-plan": validate_self_update_plan,
        "self-version": validate_self_version,
    }
    if args.kind == "release-index" and args.previous:
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
    payload = load_json_strict(Path(args.spec))
    validate_incident_request(payload, live_index=load_json_strict(Path(args.live_index)))
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


def generate_incident_signoff(args: argparse.Namespace) -> None:
    payload = load_json_strict(Path(args.spec))
    validate_incident_signoff(payload)
    write_canonical_json(Path(args.out), payload, refuse_existing=True)


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

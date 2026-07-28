#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

fixture="${REPO_ROOT}/verification/areas/distribution_release/fixtures/site_release_contract.json"
publication_workflow="${REPO_ROOT}/.github/workflows/release-publication.yml"
identity_helper="${REPO_ROOT}/scripts/distribution/verify_site_workflow_identity.sh"
dispatch_helper="${REPO_ROOT}/scripts/distribution/dispatch_stable_site_publication.sh"
preview_validator="${REPO_ROOT}/scripts/distribution/validate_preview_publication_inputs.sh"
public_docs_verifier="${REPO_ROOT}/scripts/distribution/verify_public_stable_docs.py"
python3 - "${fixture}" "${publication_workflow}" "${identity_helper}" \
  "${dispatch_helper}" "${preview_validator}" "${public_docs_verifier}" <<'PY'
import ast
import copy
import json
import pathlib
import re
import sys

fixture = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
publication = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
identity = pathlib.Path(sys.argv[3]).read_text(encoding="utf-8")
dispatch = pathlib.Path(sys.argv[4]).read_text(encoding="utf-8")
preview_validator = pathlib.Path(sys.argv[5]).read_text(encoding="utf-8")
public_docs_verifier = pathlib.Path(sys.argv[6]).read_text(encoding="utf-8")
sha = re.compile(r"^[0-9a-f]{64}$")
commit = re.compile(r"^[0-9a-f]{40}$")
attempt = re.compile(r"^[A-Za-z0-9._-]{1,80}$")
workflow_ref = re.compile(r"^[A-Za-z0-9._-]{1,80}$")
dispatchers = {"index", "stable", "alpha", "beta"}

assert fixture == {
    "schema_version": 2,
    "repository": "sifr-lang/sifr-website",
    "workflow": ".github/workflows/release-site.yml",
    "workflow_commit": "ff472f2af59255c8031b1a6f9b9b294c4b820496",
    "workflow_ref": "sifr-release-site-stable-facts",
    "workflow_ref_ruleset": {
        "bypass_actors": [],
        "enforcement": "active",
        "id": 19899766,
        "updated_at": "2026-07-28T13:22:41.496Z",
        "rules": ["deletion", "update"],
    },
    "workflow_sha256": "a9360c82395f6e9d9822f201e56cc0f2eabab1bacda01c31e4e9f22d0202b3af",
    "workflow_pr": "https://github.com/sifr-lang/sifr-website/pull/16",
    "dispatcher_generation": {
        "script": "scripts/distribution/generate_dispatchers.sh",
        "default_channel_by_ga_status": {"active": "stable", "preview": "beta"},
        "entrypoints": ["index", "stable", "alpha", "beta"],
    },
    "stable_documentation": {
        "facts_input": "stable-site-release-facts.json",
        "canonical_producer": (
            "scripts/distribution/release_governance.py generate-site-facts"
        ),
        "renderer": (
            "apps/sifr-site/scripts/render-stable-release-page.mjs"
        ),
        "renderer_sha256": (
            "5382ec769ab77ea667912a24b5752c7ec0c09ce106dc22f01b1adba76d6ee1ff"
        ),
        "rendered_labels": [
            "Active stable version",
            "Withdrawn stable versions",
        ],
        "required_fields": [
            "stable_version",
            "withdrawals[].version",
            "withdrawals[].incident_id",
        ],
        "route": "/releases/stable/",
        "preview_behavior": "absent",
    },
    "permissions": {"contents": "read", "release_metadata_write": False},
    "required_inputs": fixture["required_inputs"],
    "terminal_run": {
        "event": "workflow_dispatch",
        "title_prefix": "Sifr site release ",
        "head_sha": "workflow_commit",
        "deadline_minutes": 20,
    },
}
assert fixture["required_inputs"] == [
    "sifr_source_commit",
    "release_plan_sha256",
    "publication_attempt",
    "release_index_generation",
    "release_index_sha256",
    "site_base_commit",
    "dispatcher_index_sha256",
    "dispatcher_stable_sha256",
    "dispatcher_alpha_sha256",
    "dispatcher_beta_sha256",
    "dispatcher_default_channel",
    "publication_facts_sha256",
    "stable_site_facts_sha256",
]
verifier_tree = ast.parse(public_docs_verifier)
rendered_labels = next(
    ast.literal_eval(node.value)
    for node in verifier_tree.body
    if isinstance(node, ast.Assign)
    and any(
        isinstance(target, ast.Name) and target.id == "RENDERED_LABELS"
        for target in node.targets
    )
)
assert list(rendered_labels) == fixture["stable_documentation"]["rendered_labels"]
assert commit.fullmatch(fixture["workflow_commit"])
assert workflow_ref.fullmatch(fixture["workflow_ref"])
assert sha.fullmatch(fixture["workflow_sha256"])
for fragment in (
    "SITE_WORKFLOW_REF: sifr-release-site-stable-facts",
    'SITE_WORKFLOW_RULESET_ID: "19899766"',
    'SITE_WORKFLOW_RULESET_UPDATED_AT: "2026-07-28T13:22:41.496Z"',
    "SITE_WORKFLOW_SHA256: a9360c82395f6e9d9822f201e56cc0f2eabab1bacda01c31e4e9f22d0202b3af",
    '--workflow-sha256 "${SITE_WORKFLOW_SHA256}"',
    "scripts/distribution/validate_preview_publication_inputs.sh",
    "scripts/distribution/dispatch_stable_site_publication.sh",
    "scripts/distribution/generate_dispatchers.sh \\\n"
    "            --install-root dispatchers \\\n"
    '            --default-channel "${site_default_channel}"',
):
    assert fragment in publication, fragment
assert "scripts/distribution/verify_site_workflow_identity.sh" in preview_validator
for fragment in (
    '"${ruleset_id}" =~ ^[1-9][0-9]*$',
    '-H "Time-Zone: UTC"',
    ".updated_at == $updated_at",
    "(.bypass_actors // []) == []",
    '(.current_user_can_bypass // "never") == "never"',
    "immutable tag ruleset is not active and exact",
    "protected workflow tag moved",
    "pinned workflow bytes do not match the reviewed contract",
):
    assert fragment in identity, fragment
identity_call = "scripts/distribution/verify_site_workflow_identity.sh"
assert preview_validator.count(identity_call) == 1
assert publication.index("validate_preview_publication_inputs.sh") < publication.index(
    "Publish write-once version release and verify assets"
)
for fragment in (
    '--arg ref "${workflow_ref}"',
    "repos/${repository}/actions/workflows/${workflow}/dispatches",
    "poll_site_release_run.sh",
    '--default-channel beta|stable',
):
    assert fragment in dispatch, fragment
assert dispatch.index(identity_call) < dispatch.index(
    "repos/${repository}/actions/workflows/${workflow}/dispatches"
) < dispatch.index("poll_site_release_run.sh")

payload = {
    "sifr_source_commit": "a" * 40,
    "release_plan_sha256": "b" * 64,
    "publication_attempt": "12345-1",
    "release_index_generation": "8",
    "release_index_sha256": "c" * 64,
    "site_base_commit": fixture["workflow_commit"],
    "dispatcher_index_sha256": "d" * 64,
    "dispatcher_stable_sha256": "e" * 64,
    "dispatcher_alpha_sha256": "f" * 64,
    "dispatcher_beta_sha256": "1" * 64,
    "dispatcher_default_channel": "beta",
    "publication_facts_sha256": "2" * 64,
    "stable_site_facts_sha256": "none",
}
generated = {
    "index": payload["dispatcher_index_sha256"],
    "stable": payload["dispatcher_stable_sha256"],
    "alpha": payload["dispatcher_alpha_sha256"],
    "beta": payload["dispatcher_beta_sha256"],
}
run = {
    "event": "workflow_dispatch",
    "display_title": "Sifr site release 12345-1",
    "head_sha": fixture["workflow_commit"],
    "created_at": "2026-07-27T03:30:01Z",
}


def validate(
    candidate,
    *,
    live_generation=8,
    live_digest="c" * 64,
    live_ga_status="preview",
    observed=None,
    run_value=None,
):
    if set(candidate) != set(fixture["required_inputs"]):
        raise ValueError("site dispatch inputs drifted")
    if not commit.fullmatch(candidate["sifr_source_commit"]):
        raise ValueError("moving Sifr source ref")
    if candidate["site_base_commit"] != fixture["workflow_commit"]:
        raise ValueError("moving or unpinned site ref")
    for name in (
        "release_plan_sha256",
        "release_index_sha256",
        "publication_facts_sha256",
    ):
        if not sha.fullmatch(candidate[name]):
            raise ValueError(f"invalid digest: {name}")
    if not attempt.fullmatch(candidate["publication_attempt"]):
        raise ValueError("invalid publication attempt")
    if not candidate["release_index_generation"].isdigit() or int(
        candidate["release_index_generation"]
    ) < 1:
        raise ValueError("invalid generation")
    observed = generated if observed is None else observed
    if set(observed) != dispatchers:
        raise ValueError("dispatcher set drifted")
    for name in dispatchers:
        expected = candidate[f"dispatcher_{name}_sha256"]
        if not sha.fullmatch(expected) or observed[name] != expected:
            raise ValueError(f"dispatcher digest mismatch: {name}")
    if int(candidate["release_index_generation"]) != live_generation:
        raise ValueError("stale release-index generation")
    if candidate["release_index_sha256"] != live_digest:
        raise ValueError("stale release-index digest")
    expected_default = fixture["dispatcher_generation"]["default_channel_by_ga_status"].get(
        live_ga_status
    )
    if candidate["dispatcher_default_channel"] != expected_default:
        raise ValueError("dispatcher default disagrees with live GA status")
    expected_stable_facts = (
        "none" if live_ga_status == "preview" else "3" * 64
    )
    if candidate["stable_site_facts_sha256"] != expected_stable_facts:
        raise ValueError("stable facts identity disagrees with live GA status")
    run_value = run if run_value is None else run_value
    if (
        run_value["event"] != "workflow_dispatch"
        or run_value["display_title"]
        != f'Sifr site release {candidate["publication_attempt"]}'
        or run_value["head_sha"] != fixture["workflow_commit"]
        or run_value["created_at"] < "2026-07-27T03:30:00Z"
    ):
        raise ValueError("site run is not attributable to the publication attempt")


validate(payload)
active_payload = copy.deepcopy(payload)
active_payload["dispatcher_default_channel"] = "stable"
active_payload["stable_site_facts_sha256"] = "3" * 64
validate(active_payload, live_ga_status="active")

negatives = []
moving = copy.deepcopy(payload)
moving["site_base_commit"] = "main"
negatives.append(("moving site ref", moving, {}))

wrong_dispatcher = dict(generated)
wrong_dispatcher["stable"] = "9" * 64
negatives.append(("dispatcher mismatch", payload, {"observed": wrong_dispatcher}))

negatives.append(("stale generation", payload, {"live_generation": 9}))

wrong_default = copy.deepcopy(payload)
wrong_default["dispatcher_default_channel"] = "stable"
negatives.append(("GA default mismatch", wrong_default, {}))

wrong_facts = copy.deepcopy(payload)
wrong_facts["stable_site_facts_sha256"] = "3" * 64
negatives.append(("preview stable facts", wrong_facts, {}))

wrong_run = dict(run)
wrong_run["display_title"] = "Sifr site release another-attempt"
negatives.append(("unattributable run", payload, {"run_value": wrong_run}))

for label, candidate, kwargs in negatives:
    try:
        validate(candidate, **kwargs)
    except ValueError:
        continue
    raise AssertionError(f"site contract accepted {label}")

for fragment in (
    '--stable-site-facts-sha256 "${STABLE_SITE_FACTS_SHA256}"',
    "stable_site_facts_sha256: $stable_facts",
):
    assert fragment in publication + dispatch, fragment

print("site release cross-repository contract: PASS")
PY

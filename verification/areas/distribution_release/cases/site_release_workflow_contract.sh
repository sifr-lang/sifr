#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

fixture="${REPO_ROOT}/verification/areas/distribution_release/fixtures/site_release_contract.json"
publication_workflow="${REPO_ROOT}/.github/workflows/release-publication.yml"
python3 - "${fixture}" "${publication_workflow}" <<'PY'
import copy
import json
import pathlib
import re
import sys

fixture = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
publication = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
sha = re.compile(r"^[0-9a-f]{64}$")
commit = re.compile(r"^[0-9a-f]{40}$")
attempt = re.compile(r"^[A-Za-z0-9._-]{1,80}$")
workflow_ref = re.compile(r"^[A-Za-z0-9._-]{1,80}$")
dispatchers = {"index", "stable", "alpha", "beta"}

assert fixture == {
    "schema_version": 2,
    "repository": "sifr-lang/sifr-website",
    "workflow": ".github/workflows/release-site.yml",
    "workflow_commit": "07d88cc3c24707e386c5ad73fb0875c06ffd598f",
    "workflow_ref": "sifr-release-site-m40-2",
    "workflow_ref_ruleset": {
        "bypass_actors": [],
        "enforcement": "active",
        "id": 19790146,
        "updated_at": "2026-07-27T04:17:06.204Z",
        "rules": ["deletion", "update"],
    },
    "workflow_sha256": "7a27abaf9d7e67298ea3033abf19f1c504c68bf50bdcd4e3cc5577330456a958",
    "workflow_pr": "https://github.com/sifr-lang/sifr-website/pull/15",
    "dispatcher_generation": {
        "script": "scripts/distribution/generate_dispatchers.sh",
        "default_channel_by_ga_status": {"active": "stable", "preview": "beta"},
        "entrypoints": ["index", "stable", "alpha", "beta"],
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
]
assert commit.fullmatch(fixture["workflow_commit"])
assert workflow_ref.fullmatch(fixture["workflow_ref"])
assert sha.fullmatch(fixture["workflow_sha256"])
for fragment in (
    "SITE_WORKFLOW_REF: sifr-release-site-m40-2",
    'SITE_WORKFLOW_RULESET_ID: "19790146"',
    'SITE_WORKFLOW_RULESET_UPDATED_AT: "2026-07-27T04:17:06.204Z"',
    "SITE_WORKFLOW_SHA256: 7a27abaf9d7e67298ea3033abf19f1c504c68bf50bdcd4e3cc5577330456a958",
    ".updated_at == $updated_at",
    "(.bypass_actors // []) == []",
    '(.current_user_can_bypass // "never") == "never"',
    "site workflow tag immutability ruleset is not active and exact",
    "site workflow tag lost immutable protection before dispatch",
    "protected site workflow tag does not resolve to site_base_commit",
    "protected site workflow tag moved before dispatch",
    "pinned site workflow bytes do not match the reviewed contract",
    '--arg ref "${SITE_WORKFLOW_REF}"',
    "scripts/distribution/generate_dispatchers.sh \\\n"
    "            --install-root dispatchers \\\n"
    '            --default-channel "${site_default_channel}"',
):
    assert fragment in publication, fragment
assert publication.index("pinned site workflow bytes do not match") < publication.index(
    "Publish write-once version release and verify assets"
)

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

wrong_run = dict(run)
wrong_run["display_title"] = "Sifr site release another-attempt"
negatives.append(("unattributable run", payload, {"run_value": wrong_run}))

for label, candidate, kwargs in negatives:
    try:
        validate(candidate, **kwargs)
    except ValueError:
        continue
    raise AssertionError(f"site contract accepted {label}")

print("site release cross-repository contract: PASS")
PY

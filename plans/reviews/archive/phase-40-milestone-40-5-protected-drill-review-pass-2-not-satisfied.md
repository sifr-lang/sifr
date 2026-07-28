All listed validations re-run locally. Here is the second-pass audit.

---

## Re-audit of the six pass-1 findings — all remediated

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | Drill occupies `sifr-release-index` lock | **Fixed** — `.github/workflows/release-publication.yml:56` now keys the group `${{ startsWith(inputs.governance_mode, 'drill-') && 'sifr-release-drill' \|\| 'sifr-release-index' }}`. `inputs` is a valid context for workflow-level `concurrency` on both `workflow_call` and `workflow_dispatch`, and `workflow_call` supplies `default: preview` (`:38`), so the non-drill branch is total. Pinned by `preview_release_workflow_yaml_parses.sh:55` and `protected_release_drill_workflow_contract.sh:42`. |
| 2 | Unknown `drill-*` modes fail open to `publication` | **Fixed** — the ternary is gone; `release-publication.yml:815` passes `mode: ${{ inputs.governance_mode }}` verbatim, and `release-publication-drill.yml:34-42` maps exactly the three modes with `*) exit 2`. The dispatch surface is additionally narrowed to `type: choice` with three options (`:8-12`). A `workflow_call` with `drill-typo` now skips prepare/publish and fails the run at the boundary step. Fail-closed confirmed. |
| 3 | `SIFR_WEBSITE_ACTIONS_TOKEN` downgraded | **Fixed** — `release-publication.yml:45-48` is back to `required: true` (no diff hunk against base), and `protected_release_drill_workflow_contract.sh:47-50` slices the secrets block and asserts it, so a future downgrade fails the suite. |
| 4 | Credential deny-list forked three ways, Python guard missing the site token | **Fixed** — `governance/common.py:18-25` defines the 6-name `PRODUCTION_CREDENTIAL_NAMES`; `incident_fixture.py:36`, `runner.py:14-16,285-288`, and `protected_drill_selftest.py:106-112` all consume it, and `protected_release_drill_workflow_contract.sh:75-79` contract-checks both the workflow's boundary loop and its `sudo env -u` scrub list against that tuple name-for-name. Verified live: the local run aborted with `refuses production credential(s): VSCE_PAT` before executing anything. |
| 5 | Transition negative tests never reached the guards they named | **Fixed** — `stable_planner_selftest.py:139-206` (`test_direct_transition_defenses`) drives `propose_stable_release` directly and reaches all five guards (`release_index.py:196-197`, `:199-200`, `:194`, `:205-206`, `:207-208`), and `:126-136` adds the `incident-roll-forward`-plan case that finally reaches `stable_planner.py:95-96`. Confirmed load-bearing by inverting each guard. |
| 6 | `plan-stable-index` discarded the plan digest | **Fixed** — `StableMutation.evidence()` (`stable_planner.py:40-55`) emits transition, version, exact `plan_sha256`, previous generation+digest, the full proposed index, and `proposed_index_sha256`; `release_governance.py:484-494` writes it canonically with `refuse_existing=True`. Also fixes the pass-1 TOCTOU observation: `stable_planner.py:69-70` reads each file once and both parses and hashes those same bytes. |

Registration is complete: `sifr_verify/selftest.py:89` 13→15, both new schemas lint-clean, `protected-drill` present in the manifest, all three profiles, `profile_assignment_matrix.json`, `release_report.REQUIRED_SUITES`, `selftest.valid_report`, and `qualification_fixture`.

---

## Actionable findings

### 1. The mutation-evidence validator rejects generation gaps the planner deliberately allows — `governance/stable_planner.py:157-158`

```python
if proposed["generation"] != previous_generation + 1:
    fail("$.proposed_index.generation", "must immediately follow the previous index")
```

The producer side imposes no such rule. `propose_stable_release` gates only through `_require_incident_generation` (`release_index.py:378-384`), which requires `proposed_generation > current["generation"]`, and `validate_release_index_transition` (`release_index.py:344-345`) only requires monotonic increase. Generation gaps are not hypothetical here — they are the documented resume behaviour of this exact governance model (`internal_docs/distribution_pipeline.md:568`, "burns a generation after reservation failure"; `test_rollback_burns_generation_and_resumes`).

The wave's own fixtures already straddle the disagreement: `stable_planner_selftest.py:68-81` certifies a `normal` mutation from generation **8 → 12** as valid planning, but that test never runs the validator, and `test_cli_producer` only exercises 7 → 8. Reproduced:

```
planner accepted gap; proposed generation: 12 previous: 8
validator REJECTED: $.proposed_index.generation: must immediately follow the previous index
```

Failure scenario: a GA activation whose generation reservation fails once, burning generation 8. The retry plans 7 → 9, `plan-stable-index` writes the artifact (`release_governance.py:484-494` does **not** validate before writing), and `release_governance.py validate --kind stable-index-mutation-evidence` then rejects the very artifact the wave just produced — with no way to represent the resume-after-burn case the milestone is required to exercise. The direction is fail-closed, so this is not a safety hole, but the two halves of the wave's identity contract disagree and the gap-tolerant path is unrepresentable.

Either relax to `proposed["generation"] > previous_generation` (matching the producer and the incident model), or tighten `propose_stable_release` to reject gaps — but not both as they stand. If the strict form is intentional, `test_normal_successor` should not use a 4-generation gap, and the CLI should validate before writing.

### 2. The durable protected-publication doc still describes `release-publication.yml` as bootstrap-only — `internal_docs/distribution_pipeline.md:582-583`

> "The first protected-publication slice wires **only** the one-time schema-epoch bootstrap into `.github/workflows/release-publication.yml`."

That file now also carries a `workflow_dispatch` entry point (`:4-12`) and a nested `release-publication-drill.yml` job (`:809-815`). The wave documents this only in `plans/issues/active/phase-40-stable-channel-ga-execution.md:481-495`; `internal_docs/` gained just two suite-command lines (`architecture.md:1434`, `distribution_pipeline.md:618-619`) and `architecture.md` never mentions `release-publication.yml` at all. Per AGENTS.md, `internal_docs/` is the durable reference. The paragraph should record the drill dispatch surface, the isolated concurrency group, the read-only/no-secret/no-network guarantees, the exact-mode fail-closed mapping, and the write-once 30-day evidence.

The one-scenario-per-dispatch precision fix itself is correct and consistent across `stable_gate_inventory.json:140-147` and the plan issue.

---

## Optional observations (no action required)

- **The external evidence check doesn't bind the dispatched scenario.** `release-publication-drill.yml:70-73` validates with `--kind protected-drill-evidence` but no expected-scenario argument, so only the in-process `validate_drill_evidence(report, expected_scenarios=selected)` (`protected_drill_selftest.py:137`) ties the report to `DRILL_SCENARIO`. Producer and checker are the same process. A `jq`-free grep of `"name":"${DRILL_SCENARIO}"` in the workflow would close the loop cheaply.
- **`stable_index_mutation_evidence.schema.json` has no negative fixture** in `schema_contracts.py`, unlike the drill schema (`:98-115`), and its `proposed_index` is `{"type": "object"}` (`:49-51`) — all structure lives in the Python validator. Consistent with how `release_index` is handled elsewhere, but the asymmetry with the sibling schema added in the same wave is worth a note.
- **`sudo env -u …` is belt-and-braces.** `sudo` runs with `env_reset` by default, so the six `-u` flags in `release-publication-drill.yml:58-64` are already redundant; they are useful as a contract-checkable artifact, which is exactly how the case script uses them.
- **`test_concurrency_and_credential_boundaries` still runs twice under `--scenario all`** (`protected_drill_evidence.py:26,29`) — carried over from pass 1, unchanged and harmless.
- **Ruff on new files matches the repo baseline, not clean.** `protected_drill_selftest.py:3` (I001, `.protected_drill_evidence` imported after `.stable_planner_selftest`) and `stable_planner_selftest.py:9` (UP035, `typing.Callable`) — both mirror existing files (`selftest.py:3,10`). There is no ruff config or CI ruff step in this repo, and `verification/` has 551 baseline hits, so this is not a regression.
- **`_require_incident_generation` style change** (`release_index.py:378-384`) uses `isinstance` + bool exclusion while `common.py:81,98` kept `type(x) is not int` with a new `# noqa: E721`. Two idioms for the same intent, both correct; incidental to this wave.

---

## Validation re-run

| Gate | Result |
|---|---|
| `stable_planner_selftest` | **5/5 pass** |
| `protected_drill_selftest` (all scenarios) | **11 tests pass**, plus credential gate verified live |
| `validate_schema_contracts` | pass |
| `sifr_verify --self-test` (incl. 15-schema epoch) | pass |
| `protected_release_drill_workflow_contract.sh` | pass |
| `preview_release_workflow_yaml_parses.sh` (4 workflows YAML-parse) | pass |
| `coverage_matrix` readiness | **4/4**, 5 variants, 0 failures |
| distribution_release full+qualification+evidence-custody+incident-governance+epoch-bootstrap+protected-drill | **variants=60, failures=0** |
| File-size guardrail | largest touched file 882 lines; all new files ≤ 318 |
| Scope | no `crates/`, no `demos/`, no Rust interop, no phase-numbered demos, no production mutation |

**Not SATISFIED** — finding 1 is a genuine producer/validator contradiction in the evidence contract this wave exists to establish, and finding 2 is a durable-docs gap. Everything from pass 1 is correctly closed, and no new workflow-safety, credential-boundary, fail-closed, or planner-correctness defect surfaced.

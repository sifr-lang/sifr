# Re-review: `milestone_40_0` — Architecture and Gate Lock (pass 4)

Scope inspected: the complete working tree (39 modified, 23 untracked), the Phase 40 plan,
the execution issue, and review passes 1–3. Re-run locally during this pass:
`governance.selftest` (14/14), `governance.schema_epoch`,
`distribution_release` `full` + `evidence-custody` (42 variants, 0 failures, including
`governance-contracts`, `schema-epoch`, `evidence-custody`), `documentation:structure`
(incl. the GA mutation harness), `python -m sifr_verify --self-test` (11/11),
`cargo fmt --check` (clean), and `scripts/check_file_size_guardrails.py`
(PASS, 2841 files, limit 900).

Two independent adversarial sweeps were run in addition to the declared suites:

- **Exhaustive value-corruption sweep** over every field of all fourteen governed
  fixtures (14 corrupt values + deletion per path, ~9k cases): **zero** non-`GovernanceError`
  escapes across all eleven validators. The pass-3 defect class is closed, not patched.
- **Differential schema-vs-validator sweep** over all eleven checked-in schemas with the
  same corruption grid: exactly **one** divergence in the unsafe direction (schema rejects,
  validator accepts) — finding 1 below.

## Pass-3 findings: re-audit

| # | Pass-3 finding | Status |
|---|---|---|
| 1 | Nine enum sites raise `TypeError` on unhashable values | **Fixed at the root.** `common.py:79-82` `require_enum` checks `isinstance(value, str)` before membership and routes through `fail()`. All nine reported sites now call it: `release_index.py:32,76,84,135`, `release_plan.py:66`, `incident.py:61,169,175`, `artifact_index.py:78`, `surface_contracts.py:141,155`. The two remaining raw membership tests are safe by construction — `artifact_index.py:90` and `surface_contracts.py:81` test against the `TARGETS` **tuple** (`in` uses `==`, never hashes), and `release_plan.py:259` `len(set(claims))` runs only after every element passed `require_nonempty_string` at `:257-258`. Confirmed empirically: 0 escapes in the corruption sweep |
| 2 | `expires_at` raises bare `ValueError`; schema `format` unenforced | **Fixed.** `artifact_index.py:57` now uses `fail()`; `:58-59` rejects timezone-less values. `json_schema_202012.py:207-213` enforces `format: date-time` including the tz requirement, and `:90-91` rejects any other `format` value, so the keyword cannot be added decoratively. `schema_contracts.py:28-38` asserts the schema itself rejects an invalid expiry. Mutation added at `selftest.py:656-659`. Verified independently: `"2026-08-20T00:00:00"` → `GovernanceError: $.workflow.expires_at: must include a timezone` |
| 3 | `expect_rejected` accepted bare `ValueError` | **Fixed.** `selftest.py:315` is `except GovernanceError`. Unhashable-value mutations now prove the reported paths: `selftest.py:399` (`ga_status`), `:407` (`release[].status`), `:482` (`transition`), `:486` (`advertised_claim_ids`), `:534` (`operation`), `:582`/`:586` (`attempt.mode`/`.status`), `:662` (`artifact.kind`), `:732` (`plan.action`). The harness now structurally detects any recurrence |
| 4 | `schema_epoch` newline character class | **Fixed.** `schema_epoch.py:29` is `[^\n]{0,40}`. The `if schema_version == 1:` fixture at `:83` now exercises the pattern for real |
| 5 | Inventory mutation names not bound to executed mutations | **Fixed for `ga-release`; residual for `structure`.** `check_ga_release_docs.py:26-32,47-58` binds the declared tuple to the executed mutation dict keys and fails on drift; `check_structure.py:13,29-32,80-81` requires the inventory to equal that tuple exactly. The `structure` half (`check_structure.py:23-28` vs the anonymous lambdas at `:115-120`) is still name-only — see finding 3 below |
| 6 | Duplicate documentation-suite authority | **Fixed.** `profile_runner.py:262-263` `documentation_suites` reads `selected_suites_for_area(self.profile, "documentation")`; no execution path reads `legacy_facade.documentation_suites`. `profiles.py:491` still emits that key in the resolved profile, but it is *derived* from `selected_suites_for_area`, so divergence is impossible by construction |

Spot-verified beyond the pass-3 list: the publish-release compare-and-swap re-fetches,
re-validates, and re-compares generation + digest at `preview-release.yml:305-320` before
the only index mutation, inside the workflow-level `sifr-release-index` concurrency group
(`:27-28`); `build-release-record` runs before any `gh release` call, so an ineligible
version fails closed prior to publication; `validate.source_sha` (`:84`) pins the build and
publish jobs to one resolved commit rather than a mutable ref (`:105`, `:150`);
`release_evidence.py:243-251` splits `developer_tooling:full` into non-editor and
`editor-release` evidence, reinforced by `release_report.py:212-214,219-221`;
`prepare_release_report_output` (`release_evidence.py:43-62`) refuses in-repo output,
pre-existing output, and a dirty tree; the Rust surface keeps stable unreachable at every
input (`self_update_metadata.rs:204-210` metadata key, `:281-292` `parse_channel`,
`:47-51` stable-version pin) with `-rc.N` still rejected and tested at `:711-720`.

## Local findings

**1. MEDIUM — `plan_id` is enforced by the JSON Schema but not by the validator; the
executable gate is the weaker of the two.**

`stable_release_plan.schema.json:12` constrains the candidate identity to
`^stable-[0-9]+\.[0-9]+\.[0-9]+-[0-9a-f]{12}$`, but `release_plan.py:65` applies only
`require_nonempty_string`, and nothing binds `plan_id` to `plan["version"]`. The
differential sweep flagged exactly this class and nothing else:

```
SCHEMA-STRICTER (validator accepts what schema rejects): stable_release_plan  plan_id  'xx'
SCHEMA-STRICTER (validator accepts what schema rejects): stable_release_plan  plan_id  '0.1.0-rc.1'
SCHEMA-STRICTER (validator accepts what schema rejects): stable_release_plan  plan_id  '000…0'
```

The schemas are applied only to in-process fixtures (`schema_contracts.py:26-27`); every
path that touches a *real* artifact — `release_governance.py validate --kind release-plan`
(`:172-179`), `generate-release-plan` (`:230-234`), and
`evidence_custody.validate_candidate_directory` (`:154`) — goes through the Python
validator alone. A checked-in `stable-release-plan.json` whose `plan_id` violates the
published contract, or names a different version than the plan does, therefore passes every
executable gate in the repository and is written to immutable candidate evidence.
`plan_id` is the canonical candidate identity that `milestone_40_1` binds provenance to, so
the divergence should be closed before that milestone consumes it. The fix is mechanical: a
`PLAN_ID_RE` in `common.py`, a `plan_id`-vs-`version` agreement check in
`validate_release_plan`, and one mutation in `test_release_plan_mutations`.

**2. LOW — the timezone-less `expires_at` branch is correct but never mutation-covered.**

`artifact_index.py:58-59` enforces the phase's expiry-binding rule and works (verified by
hand). But `selftest.py:656-659` and `schema_contracts.py:29` both mutate only to
`"not-a-timestamp"`, which is caught one line earlier at `:57`. Neither that validator
branch nor the `json_schema_202012.py:212-213` tz branch has an executable case, so deleting
either would not fail any suite. Add a `"2026-08-01T00:00:00"` mutation alongside the
existing one.

**3. LOW — `structure` mutation-case names remain nominal (pass-3 finding 5 residual).**

`check_ga_release_docs.py:57` now binds declared names to executed mutations by dict key.
`check_structure.py` did not receive the same treatment: `STRUCTURE_MUTATION_CASES`
(`:23-28`) is compared against the inventory at `:80-81`, but `run_self_tests()`
(`:115-120`) executes four anonymous lambdas with no name binding and no length assertion,
so a fifth declared name could be added to both the tuple and `docs_inventory.json` without
a corresponding executed mutation. Converting that list to a `{case_id: callback}` dict and
asserting `tuple(mutations) == STRUCTURE_MUTATION_CASES`, exactly as the GA harness does,
closes it.

**4. LOW — `preview-release.yml:69` still admits `-rc.N` in the input regex.**

The canonical epoch decision is that `rc` does not exist (`common.py:121`
"rc is not supported"; `selftest.py:336` asserts no schema mentions it; the Rust surface
rejects it). The workflow's version pattern is still
`^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta|rc)\.[0-9]+$`. This is unreachable in practice — the
`channel` input is a two-option choice and the `version_channel != channel` check at
`:73-77` rejects any rc pin — so it is a dead branch, not a hole. It is stale surface in a
file listed in `schema_epoch.GOVERNED_FILES`, and dropping `|rc` costs nothing.

## External integration blocker (unchanged; do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`,
`compatibility-matrix`, `stale-drafts`; `verification/areas/rust_interop/data/` contains no
`stable_support_claims.json`; and `verification/profiles/release.json` `selected_areas`
therefore cannot list `stable-candidate` while `release_report.py:38` requires it. The
milestone is correctly fail-closed on this: a real `--release-report-out` run rejects at
`validate_profile` and `validate_steps` rather than emitting silently-incomplete evidence.
Integration once the prerequisite merges remains a one-line change — append
`"stable-candidate"` to the `rust_interop` suites array in `release.json`; it flows through
`run_rust_interop_checks` → `run_selected_area` → `validate_area_result` and satisfies both
required-suite checks. No other local change is needed.

The strict schema-v2 dispatcher parser (`generate_dispatchers.sh:125-134` still text-matches)
and stable resolution behavior remain correctly assigned to `milestone_40_2` by the
execution issue (`:156-159`) and the phase plan; requesting them here would be scope creep.

## Verdict

**CHANGES REQUESTED — 4 local findings (1 MEDIUM, 3 LOW).**

All six pass-3 findings are genuinely resolved, and findings 1–4 of that pass were fixed at
their root rather than at their reported sites — verified by re-running the same class of
adversarial sweep that produced them, with zero escapes across ~9k corruption cases and all
eleven validators. Fail-closed diagnostics, evidence custody, workflow publication safety,
release-report production purity, documentation authority, scope discipline, planning status,
formatting, and the file-size guardrail are all satisfied.

What remains is finding 1: the one surviving schema-vs-validator divergence in the unsafe
direction, on the field that `milestone_40_1` will use as the candidate's canonical identity.
It is the same shape as the class this milestone exists to lock — a governed field whose
published contract is stricter than the code that actually gates artifacts — and it is a few
lines to close, together with the three LOW items. A fifth pass should be short.

`rust_interop:stable-candidate` and `stable_support_claims.json` remain correctly scoped as
an external integration blocker owned by another worktree; they are not the reason for this
verdict.

## Review — Phase 40 / Milestone 40.5 protected credential-free drill + stable index planner

**Scope check (clean).** No `crates/`, no `demos/`, no Rust interop implementation. The wave is confined to release governance, workflow plumbing, verification suites, and planning docs. Schema/suite/profile registration is complete and self-enforcing (`schema_contracts.py:29-31` set-equality on fixture names; `sifr_verify/selftest.py:89` bumped 13→14; merge/nightly/release profiles + `profile_assignment_matrix.json` all updated consistently). The new case script is auto-discovered by `runner.py:248` glob and is `0755`. The zero-byte `plans/reviews/active/…pass-1.md` matches the repo's existing handoff-slot convention (pass 14 did the same) — not a finding.

Findings below, most severe first.

---

### 1. The drill occupies the production release-index concurrency lock — `.github/workflows/release-publication.yml:55-57`

```yaml
concurrency:
  group: sifr-release-index
  cancel-in-progress: false
```

This is workflow-level, so it applies to the `drill:` job path too. A read-only, credential-free drill (up to 20 min) now holds the same mutual-exclusion group as real preview/bootstrap index mutation. Worse: with `cancel-in-progress: false`, GitHub keeps only **one** pending run per group and cancels earlier pending ones — so dispatching two drills while a real publication is queued evicts the queued production run.

The drill is precisely the kind of job that must not contend with the production lock. Suggest keying the group on mode, e.g.
`group: ${{ startsWith(inputs.governance_mode, 'drill-') && 'sifr-release-drill' || 'sifr-release-index' }}`.

### 2. Scenario mapping fails open on unrecognized `drill-*` modes — `.github/workflows/release-publication.yml:810,815`

```yaml
if: ${{ startsWith(inputs.governance_mode, 'drill-') }}
...
scenario: ${{ ... == 'drill-first-ga' && 'first-ga' || ... == 'drill-rollback' && 'rollback' || 'publication' }}
```

The `workflow_call` `governance_mode` input is an unvalidated `type: string` (`:36-40`). Any value starting with `drill-` — including a typo or an unintended value — now (a) skips `prepare` and `publish` via the two `!startsWith(...)` guards at `:61` and `:75`, and (b) is silently normalized to `publication` by the trailing `|| 'publication'` fallback. The run then reports **success** having published nothing.

Before this change, an unsupported `governance_mode` was rejected by the publish job's `*) echo "::error::unsupported governance_mode"; exit 2` (`:202`). The drill's own boundary validator (`release-publication-drill.yml:34-40`) cannot compensate, because the ternary has already erased the bogus value. Map exactly the three modes and fail on anything else (or pass `inputs.governance_mode` through verbatim and let the boundary step reject it).

### 3. `SIFR_WEBSITE_ACTIONS_TOKEN` downgraded to `required: false` unnecessarily — `.github/workflows/release-publication.yml:46-49`

`secrets:` under `workflow_call` is only evaluated for `workflow_call` invocations; `workflow_dispatch` (the only way to reach the drill) never consults it, and the sole caller `preview-release.yml:201-202` always passes the token. So the drill did not require this relaxation, and the change removes a call-site fail-closed guarantee on the production publication path: a future caller that omits the secret now passes call validation and runs `publish` with an empty `SITE_TOKEN`.

Blast radius is bounded — `verify_site_workflow_identity.sh:29` requires `-n "${GH_TOKEN:-}"` and exits 2 at the first validation step, before any mutation — but the guarantee has moved from the interface contract to a downstream script with no test pinning it. Restore `required: true`, or add an explicit non-empty assertion in the publish job's validate step.

### 4. Credential deny-list is forked three ways and the Python guard misses the site token — `governance/incident_fixture.py:35-41`

`FORBIDDEN_CREDENTIALS` contains 5 names and **omits `SIFR_WEBSITE_ACTIONS_TOKEN`**, yet it is the only in-process refusal used by the drill (`protected_drill_selftest.py:104`). The other two lists — `runner.py:719-727` (scrub) and `release-publication-drill.yml:41-48` (boundary) — both carry 6 names including the site token.

Consequence: `run_drill` will happily execute with the production site token live in the environment (local invocation, or any future job that maps it), which is exactly the boundary the drill exists to assert. Hoist one shared constant and have all three sites consume it.

### 5. The transition negative tests don't reach the code they name — `governance/stable_planner_selftest.py:111-125`

Both cases assert on `"active live stable predecessor"`, which is emitted by `release_plan.py:126` (plan↔index consistency), not by the stable-planner transition logic. As a result:

- `stable_planner.py:56-57` (`"stable publication accepts ga-activation or normal"`) is **never exercised**. The case that would reach it is an `incident-roll-forward` plan carrying a well-formed `expected_stable_predecessor` against an active index — `valid_plan(transition="incident-roll-forward")` leaves the predecessor as `"none"`, so it short-circuits earlier.
- `release_index.py:205-206`, `:207-208`, and `:202-203` are unreachable via `materialize_stable_mutation` (all three are already enforced by `release_plan.py:94-96` and `:121-131`) and untested via the direct `propose_stable_release` entry point.

The guards are correct defense-in-depth; the issue is that a test named `test_fail_closed_identity_and_transition` currently certifies the transition guard without touching it. The `expected_generation` / `expected_sha256` / `proposed_generation` assertions in the same test (`:80-103`) *are* load-bearing.

### 6. `plan-stable-index` discards the plan digest it computes — `scripts/distribution/release_governance.py:481-493`, `governance/stable_planner.py:73-76`

`materialize_stable_mutation` computes `plan_sha256` (a second full-file hash) and `transition`, but `plan_stable_index` writes only `mutation.proposed_index`. Nothing in the emitted artifact binds the proposed index to the exact plan bytes that produced it — which is the identity property the planner exists to provide for the future protected adapter. Today the hash is dead work; emitting the mutation record (or at minimum `plan_sha256` + `transition` alongside the index) would make it load-bearing.

---

### Optional observations (no action required)

- **TOCTOU on the live index** — `stable_planner.py:43-49` reads the file once via `load_json_strict` and again via `sha256_file`. A concurrent rewrite between the two reads means the validated bytes and the digested bytes differ. Single read + hash-in-memory would close it. (Irrelevant in the drill; matters if the protected adapter reuses this path against a shared working tree.)
- **The drill's boundary env loop is vacuous today** — `release-publication-drill.yml:41-48` checks names that are never in the job environment (`GITHUB_TOKEN` is not exported by default; no `env:` maps secrets). It is useful defense-in-depth against a future edit, but it does not verify the `stable-release-drill` GitHub environment is secret-free. The real protection is the absence of a `secrets:` block on the `uses:` call plus `sudo env -u` at `:55-61`, which is correct.
- **Doc overstatement** — `plans/releases/stable_gate_inventory.json:140` and `plans/issues/active/phase-40-stable-channel-ga-execution.md:481-485` describe the drill as running GA, normal, rollback, site-timeout resume, and first-GA together. Only the local suite does (`--scenario` defaults to `all`); a single `workflow_dispatch` runs exactly one scenario and no dispatch option selects `all`. `internal_docs/architecture.md:1434` is accurate as written since it documents the local suite.
- **Duplicate scenario execution under `all`** — `test_concurrency_and_credential_boundaries` is listed in both `rollback` and `first-ga` (`protected_drill_evidence.py:26,29`) so it runs twice. It mutates `os.environ["GH_TOKEN"]` (`incident_recovery_selftest.py:296`); `run_drill`'s credential gate runs once up front, so a failed restore would not be re-caught.
- **Overlapping workflow contracts** — `cases/preview_release_workflow_yaml_parses.sh:110-111` and the new `cases/protected_release_drill_workflow_contract.sh` both YAML-parse and textually assert the drill file.
- **`release_index.py:378-384`** — replacing `type(x) is not int` with `isinstance` + explicit bool exclusion is behaviour-preserving for `bool` but now admits `int` subclasses (`IntEnum`); the previous form was strictly tighter, and the change is incidental to this wave.

---

**Not SATISFIED** — findings 1–3 (production workflow safety) and 4 are actionable; 5–6 are test/evidence integrity gaps worth closing in this wave. Everything else I checked — schema-v2-only behaviour, canonical/`refuse_existing` evidence writes, retained-release and preview-channel preservation in `propose_stable_release` (`release_index.py:212-229`), the import-time `SCENARIOS`↔`GOVERNED_SCENARIO_TESTS` cross-check (`protected_drill_selftest.py:94-98`), read-only drill permissions, write-once bounded-retention artifact, and file sizes (all new files ≤ 226 lines) — is correct.

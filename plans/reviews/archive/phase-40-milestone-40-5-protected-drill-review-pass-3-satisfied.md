All listed validations re-run locally and reproduce. Third-pass audit below.

---

## Re-audit of the pass-2 findings — both remediated

**Pass-2 #1 (producer/validator generation contradiction)** — **Fixed.** `stable_planner.py:157-158` is now `if proposed["generation"] <= previous_generation: fail(..., "must follow the previous index")`, which matches `_require_incident_generation` (`release_index.py:379-384`) and `validate_release_index_transition` exactly. The burned-generation resume case is now representable: `test_normal_successor` plans 8→12 and calls `validate_stable_mutation_evidence(mutation.evidence())` (`stable_planner_selftest.py:87,95`) — the validator is now on that path, which it was not in pass 2. The strict direction is still pinned by a negative: `test_ga_activation` builds a same-generation `stale` artifact, re-digests it so only the generation rule can fire, and asserts `"must follow the previous index"` (`:70-78`). The CLI validates before writing (`release_governance.py:498-499` → `write_canonical_json(..., refuse_existing=True)`), so the artifact the wave produces can no longer be one the validator rejects. Verified 5/5 green.

**Pass-2 #2 (durable doc described `release-publication.yml` as bootstrap-only)** — **Fixed.** `internal_docs/distribution_pipeline.md:582-598` now says the slice wires "the one-time schema-epoch bootstrap **and credential-free protected drills**", followed by a paragraph covering every boundary I checked, and each claim is true against the tree: exact one-scenario dispatch (`release-publication.yml:4-12`, `release-publication-drill.yml:34-42`), verbatim mode pass-through (`release-publication.yml:815`), fail-closed on unknown modes (`*) exit 2`, and non-`drill-` typos still hit the publish job's existing `*) exit 2`), isolated `sifr-release-drill` group (`:56`), `contents: read` (`:812`, drill `:11-12,21-22`), no inherited secret (no `secrets:` on the `uses:`, contract-asserted), production-credential scrub (`drill:44-51,58-64`), blocked network (`unshare --net`), scenario-bound write-once schema-v2 evidence with 30-day retention (`:70-83`), and never taking the production lock.

**Also verified as claimed:** the external CLI check now binds the scenario — `--expected-drill-scenario "${DRILL_SCENARIO}"` (`release-publication-drill.yml:73`) reaches `validate_drill_evidence(payload, expected_scenarios=(...))` (`release_governance.py:255-259`), and the case script pins that exact fragment (`protected_release_drill_workflow_contract.sh:59`). I confirmed the guard is load-bearing by feeding a `rollback` report with `expected_scenarios=("first-ga",)`: rejected at `protected_drill_evidence.py:80-81`. The new mutation schema has two negative fixtures (`schema_contracts.py:117-133`, bad `transition`, `previous_index.generation: 0`).

**All six pass-1 findings remain closed** — I re-checked each against the current tree, not the pass-2 write-up: concurrency ternary, verbatim mode + `type: choice`, `SIFR_WEBSITE_ACTIONS_TOKEN: required: true` (sliced and asserted at `protected_release_drill_workflow_contract.sh:47-50`), single `PRODUCTION_CREDENTIAL_NAMES` consumed by all four sites, direct transition tests reaching all five guards, and plan-bound mutation evidence.

---

## Actionable findings

None.

**SATISFIED.**

---

## Optional observations (no action required)

- **`expected_scenarios` mismatch has no negative test** — `protected_drill_selftest.py:53` exercises the binding positively and the seven mutations at `:54-64` never vary `expected_scenarios` (they call `validate_drill_evidence(changed)` with no binding). The workflow's new scenario gate therefore rests on a branch (`protected_drill_evidence.py:80-81`) that no test asserts rejects. I verified it manually; a one-line addition to the mutation loop would pin it.
- **`--expected-drill-scenario` is silently ignored for other kinds** — `release_governance.py:255` gates on `args.kind == "protected-drill-evidence" and args.expected_drill_scenario`; passing it with `--kind release-index` is a no-op rather than an error. Identical laxity to the pre-existing `--previous` / `--live-index` flags, so this is consistent, not a regression.
- **No `--live-index` re-binding for `stable-index-mutation-evidence`** — the evidence records `previous_index.sha256` (`stable_planner.py:47-50`) but the CLI has no path to re-verify it against a live index, unlike `release-plan`, `incident-request`, and `site-facts` (`release_governance.py:260-266`). The artifact is self-consistent (`proposed_index_sha256` is recomputed at `:165-166`) but externally unanchored until the protected adapter lands.
- **`test_no_production_adapter_surface` assertion narrowed** — `incident_recovery_selftest.py:405` went from `"rollback" not in dispatch` to `"\n          - rollback\n" not in dispatch`. Necessary now that `- drill-rollback` is a legitimate option, but it no longer catches an arbitrary future `rollback`-bearing dispatch option at other indentation. `"incident-roll-forward" not in dispatch` is unweakened.
- **`proposed_index` remains `{"type": "object"}`** in `stable_index_mutation_evidence.schema.json:49-51`; all structure lives in `validate_release_index`. The pass-2 asymmetry note is now half-closed (negative fixtures exist); the loose subschema is unchanged and consistent with how `release_index` is handled elsewhere.
- **The drill core runs as root** — `sudo env … unshare` (`release-publication-drill.yml:58-65`) executes `python3` as root, leaving `drill-evidence/protected-drill.json` root-owned in the workspace. Harmless here: umask 022 makes it readable by the non-root validate and upload steps, and none of the 11 governed tests run `git` inside the checkout (`git()` at `incident_recovery_selftest.py:843` is used only by `test_evidence_only_commit_validator`, which is not in any drill scenario), so there is no dubious-ownership exposure.
- **`sudo env -u` is belt-and-braces** — `env_reset` already applies; the flags earn their place as the contract-checkable artifact consumed by `protected_release_drill_workflow_contract.sh:77-80`.
- **`test_concurrency_and_credential_boundaries` still runs twice under `--scenario all`** (`protected_drill_evidence.py:26,30`) — carried from pass 1, unchanged, harmless; it restores `GH_TOKEN` in a `finally` (`incident_recovery_selftest.py:301-306`).
- **Ruff I001/UP035 on the new files** matches the existing `verification/` baseline; no ruff config or CI ruff step exists in this repo.

---

## Validation re-run

| Gate | Result |
|---|---|
| `stable_planner_selftest` | **5/5 pass** |
| `protected-drill` suite via `sifr_verify` | **11 tests, variants=1, failures=0** |
| Credential gate, live | aborts with `refuses production credential(s): VSCE_PAT` |
| `protected_release_drill_workflow_contract.sh` | pass |
| `preview_release_workflow_yaml_parses.sh` (4 workflows) | pass |
| `sifr_verify --self-test` (incl. 15-schema epoch) | pass |
| distribution_release full+qualification+evidence-custody+incident-governance+epoch-bootstrap+protected-drill | **variants=60, failures=0** |
| Stock `python3` import of the drill module (no `uv`) | ok — pure stdlib chain, so the CI `python3 -m` invocation resolves |
| File-size guardrail | largest touched 882 (`incident_recovery_selftest.py`); all new files ≤ 333 |
| Scope | no `crates/`, no `demos/`, no Rust interop, no phase-numbered demos, no live production mutation |

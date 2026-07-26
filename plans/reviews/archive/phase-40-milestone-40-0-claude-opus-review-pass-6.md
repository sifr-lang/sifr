# Re-review: `milestone_40_0` — Architecture and Gate Lock (pass 6)

Scope inspected: the complete working tree (39 modified, 25 untracked), the Phase 40 plan
(`milestone_40_0` scope and DoD), the execution issue
(`plans/issues/active/phase-40-stable-channel-ga-execution.md`), and review passes 1–5.

Re-run locally during this pass: `governance.selftest` (14/14 pass), `governance.schema_epoch`
(pass), `documentation/check_structure.py` including the GA mutation harness (pass),
`preview_release_workflow_yaml_parses.sh` (exit 0), `self_update_json_surface_parity.sh`
(exit 0), `artifact_self_update_receipt_rules.sh` (exit 0), `python -m sifr_verify --self-test`
(11/11 pass), `demos/milestone_40_0_demo.sh` (exit 0), `cargo fmt --check` (clean), and
`scripts/check_file_size_guardrails.py` (PASS, 2841 files, limit 900; largest touched file is
`governance/selftest.py` at 859).

Two adversarial sweeps were re-executed, not re-read, over all eleven governed fixtures — every
JSON path × 26 corrupt values + deletion, **12258 cases**:

- **Raw-exception corruption sweep**: **zero** non-`GovernanceError` escapes from any of the
  eleven validators, and zero non-`JsonSchemaError` escapes from the schema engine. The pass-3
  class remains closed.
- **Differential schema-vs-validator sweep**: **two** distinct fields diverge in the unsafe
  direction (schema rejects, validator accepts) — findings 1 and 2 below. 875 divergences in
  the safe direction (validator stricter) are expected and correct.
- Supplementary static sweep: pairwise comparison of every same-named top-level property across
  the eleven schemas found no remaining semantic disagreement. `self_version.channel` omits a
  redundant `"type": "string"` next to its `enum`, and `sysroot_sifr_version` is expressed as a
  `$ref` in one schema and inline in the other with an identical pattern — both cosmetic.

## Pass-5 findings: re-audit

| # | Pass-5 finding | Status |
|---|---|---|
| 1 | `sysroot_schema_version` pinned to `1` by the receipt schema and the Rust reader but only `require_positive_int` in the Python validator; no mutation coverage | **Fixed.** `surface_contracts.py:85-89` and `:119-123` now reject anything that is not exactly integer `1`, using `type(...) is not int` so `True` cannot pass as `1`; `self_version.schema.json:31` was tightened from `{"type": "integer", "minimum": 1}` to `{"const": 1}`, so it now agrees with `self_update_install_receipt.schema.json:37` and with `self_update_receipt.rs:189-193`. Negative mutation coverage exists on **both** surfaces — `selftest.py:702-704` (receipt) and `:726-729` (self version), each setting `7`. The earlier CLI repro (`release_governance.py validate --kind install-receipt`) now exits 2, and the differential sweep reports zero `sysroot_schema_version` divergences |
| 2 | `preview-release.yml:71` still named `-rc.N` as an accepted pin | **Fixed.** The diagnostic now reads `version must be a semver prerelease using -alpha.N or -beta.N` (`preview-release.yml:71`), matching the regex at `:70`. `grep -n 'rc\.' .github/workflows/preview-release.yml` returns nothing, so the workflow — a `schema_epoch.GOVERNED_FILES` member — has no remaining `rc` surface. The three independent non-alpha/beta rejections (`:60-62`, `:64-67`, `:76-79`) and their assertions in `preview_release_workflow_yaml_parses.sh` are unchanged. `BASH_REMATCH[1]` at `:75` is still correctly populated by the negated match at `:70` |

Spot-verified beyond the pass-5 list, all still correct: the publish-release compare-and-swap
re-fetches, re-validates, and re-compares generation + digest (`preview-release.yml:303-320`)
in the same job that exported them (`:194-231`) and under the workflow-level
`sifr-release-index` concurrency group (`:27-28`); `source_sha` pins validate/build/publish to
one resolved commit (`:84`, `:105`, `:150`); the v1 bootstrap fallback is gone and an
unavailable v2 index now hard-fails (`:200-205`); `prepare_release_report_output`
(`release_evidence.py:43-62`) refuses in-repo output, pre-existing output, a non-fresh parent,
and a dirty or unresolved tree, and `:44-45` restricts `--release-report-out` to the `release`
profile; `profiles._optional_arg` (`profiles.py:599-608`) rejects a repeated flag; evidence
custody still fails closed with no comparison base (`evidence_custody.py:111-116`) and binds
plan version, report/qualification digests, and sign-off provenance to the candidate directory
(`:164-182`); documentation suite authority is single-sourced through `selected_suites_for_area`
(`profiles.py:140-147`); `run_all_tests.sh` now rejects valueless `--profile` and
`--release-report-out`. Roadmap row 40 and the execution issue checklist accurately reflect
state, including the two unchecked items.

## Local findings

**1. MEDIUM — `validate_release_signoff` accepts a non-stable version; the published sign-off
schema pins stable-only, and there is no mutation covering the field.**

`stable_release_signoff.schema.json:20` declares
`"version": {"type": "string", "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$"}`. The executable
validator is weaker: `release_plan.py:334` calls

```python
version_channel(signoff["version"], "$.version")
```

and discards the result. `version_channel` (`common.py:127-136`) returns `"alpha"`, `"beta"`,
or `"stable"`, so any preview version validates. This is the **only** stable-version field in
the governance module that omits the `!= "stable"` guard — every sibling has it:
`release_plan.py:64-65` (`$.version`), `:163-164` (`expected_stable_predecessor.version`),
`:176-177` (`rollback_target.version`), `:427-428` (`site_facts.stable_version`),
`artifact_index.py:38-39` (`candidate_version`), `incident.py:70-71` and `:91-92`.

Reproduces through the real entrypoint, not only in-process:

```
$ python3 scripts/distribution/release_governance.py validate \
    --kind release-signoff --input <signoff with "version": "0.1.0-alpha.0"> --require-canonical
release-governance validation ok: kind=release-signoff input=/tmp/signoff_alpha.json
exit=0
```

The same payload is rejected by the checked-in schema. The artifact this weakens is the record
that certifies a GA publication occurred, including `published_assets`, `marketplace`, and
`channel_generation`. Evidence custody happens to mask it — `evidence_custody.py:179` requires
`signoff["version"] == expected_version` and the plan in the same directory must be stable — but
the standalone `--kind release-signoff` gate is the surface `milestone_40_3` will call directly
during protected publication, and it is exactly the "published contract stricter than the
executable gate" class this milestone exists to lock (the identical shape as pass-4 finding 1 and
pass-5 finding 1).

There is also no mutation for `$.version` in `test_signoff_mutations` (`selftest.py:559-591`):
the fixture sets `"version": "0.1.0"` at `:562` and the only three mutations target
`attempts[0]`, so the constraint could be dropped without failing a suite.

Fix: `if version_channel(signoff["version"], "$.version") != "stable": fail(...)` at
`release_plan.py:334`, plus one mutation in `test_signoff_mutations`.

**2. LOW — `qualification_artifact_index` artifact `id` is pattern-constrained by the schema but
only checked for non-emptiness by the validator.**

`qualification_artifact_index.schema.json:31` declares
`"id": {"type": "string", "pattern": "^[a-z0-9][a-z0-9_.-]+$"}`. `artifact_index.py:74` applies
only `require_nonempty_string`, which (`common.py:76-79`) accepts any non-empty string —
whitespace-only, uppercase, leading punctuation, and single characters all pass:

```
$ python3 scripts/distribution/release_governance.py validate \
    --kind qualification-artifact-index --input <artifacts[0].id = " "> --require-canonical
release-governance validation ok: kind=qualification-artifact-index input=/tmp/qual_space.json
exit=0
```

The id is a governed identity — it keys the uniqueness set at `artifact_index.py:75-77` and is
mirrored into `stable_release_plan.qualification_artifact_index.id` — so a consumer validating
against the published schema rejects an index the gate accepted. Lower severity than finding 1
because the artifact's real binding is its sha256 and the shape is cosmetic, but it is the same
schema-versus-gate disagreement class, on a governed surface, with no mutation coverage
(`test_artifact_index_mutations`, `selftest.py:630-670`, never mutates `id`). Fix: add a
`require_artifact_id` helper in `common.py` mirroring the schema pattern, call it at
`artifact_index.py:74`, and add one mutation.

## Informational (no action required in this milestone)

- `docs/self_update.md:51` still contains the literal `-rc.N` (now phrased as "`rc` is not a
  public channel and `-rc.N` pins are rejected"), which `check_ga_release_docs.FORBIDDEN_CLAIMS`
  (`:31`) would reject. The `ga-release` check is `reserved` in `docs_inventory.json`, so nothing
  fails today; `milestone_40_4` owns it when the GA check is pointed at this file.
- `FORBIDDEN_CLAIMS` entries `"Windows installer"` and `"-rc."` still have no corresponding
  entry in `check_ga_release_docs.MUTATION_CASES` (`:33-39`). Same reserved suite, same
  `milestone_40_4` owner.
- The stale `-rc.N` operator strings in `generate_version_installer.sh:71`,
  `build_preview_artifacts.sh:83`, `trigger_preview_release.sh:159`, `create_new_version.sh:187`,
  and `generate_dispatchers.sh:88` remain pre-existing and are each gated downstream by an
  alpha/beta channel check. Not requested here; the strict schema-v2 dispatcher parser is
  `milestone_40_2` scope.
- `profiles.py:387` emits a `DOCUMENTATION_SUITES` shell export that nothing consumes, matching
  the pre-existing unread `TOOLING_SUITES`/`VALIDATION_SUITES` exports — consistency, not new
  dead code.
- `governance/selftest.py` is at 859 of 900 lines. Under the cap, but the two mutations the
  findings above require will push it closer; consider splitting by artifact family when it
  next grows.

## External integration blocker (unchanged; do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`,
`compatibility-matrix`, `stale-drafts`, and `verification/areas/rust_interop/data/` still has no
`stable_support_claims.json`. `verification/profiles/release.json`
`selected_areas[rust_interop].suites` therefore cannot list `stable-candidate` while
`release_report.py:32-43` requires it. The milestone is correctly fail-closed: a real
`--release-report-out` run rejects at `validate_profile` (`release_report.py:148-154`) and
`validate_steps` (`:225-228`) rather than emitting silently-incomplete evidence.

**Integration condition, once the prerequisite merges:** append `"stable-candidate"` to
`verification/profiles/release.json` `selected_areas[rust_interop].suites`. It flows through
`run_rust_interop_checks` → `run_selected_area` → `validate_area_result` and satisfies both
required-suite checks. No other local change is needed.

Dispatcher stable behavior and the strict schema-v2 dispatcher parser
(`generate_dispatchers.sh:125-134` still text-matches) remain correctly assigned to
`milestone_40_2` by the execution issue and the phase plan; requesting them here would be scope
creep.

## Verdict

**CHANGES REQUESTED — 2 local findings (1 MEDIUM, 1 LOW).**

Both pass-5 findings are fully resolved: `sysroot_schema_version` is now `const: 1` in both
published schemas, exactly-`1` in both Python validators, mutation-covered on both surfaces, and
in agreement with `self_update_receipt.rs:189`; the `preview-release.yml` diagnostic now names
only `-alpha.N` and `-beta.N` and the workflow has no `rc` surface left. The fresh 12258-case
sweep found zero raw-exception escapes.

What blocks approval is that the same differential sweep that produced the last two findings
still reports two unsafe divergences, and the MEDIUM one is on the GA publication sign-off
itself: `validate_release_signoff` is the single stable-version field in the module missing its
`!= "stable"` guard, reproducible through `release_governance.py validate --kind release-signoff`,
with no mutation coverage. Both remediations are a few lines plus one mutation each. Workflow
publication safety, evidence custody and report integrity, profile selection and execution
agreement, documentation contracts and mutation binding, scope discipline, planning status,
formatting, and the file-size guardrail are all satisfied.

`rust_interop:stable-candidate` and `stable_support_claims.json` remain correctly scoped as an
external integration blocker owned by another worktree; they are not the reason for this verdict.

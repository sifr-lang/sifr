## Review pass 7 — Phase 40, `milestone_40_0`

Scope inspected: full working tree (39 modified, 25 untracked), the Phase 40 plan `milestone_40_0` scope/DoD (`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:356-470`), the execution issue, and review passes 1–6. `plans/reviews/active/phase-40-milestone-40-0-agent-review-pass-7.md` exists but is empty (0 bytes); I did not write to it, per instruction.

### Pass-6 fixes: verified

**Finding 1 (MEDIUM) — `validate_release_signoff` accepted a non-stable version. Fixed.**
`release_plan.py:334-335` now reads `if version_channel(signoff["version"], "$.version") != "stable": fail("$.version", "must be an exact stable version")`, matching `stable_release_signoff.schema.json:20` (`^[0-9]+\.[0-9]+\.[0-9]+$`) and every sibling stable-version guard. Mutation coverage added at `selftest.py:580-586` (`"version": "0.1.0-alpha.1"`). Reproduced through the real entrypoint, not in-process — the pass-6 repro now inverts:

```
release_governance.py validate --kind release-signoff --input <version=0.1.0-alpha.0> --require-canonical
  exit=2  release-governance: $.version: must be an exact stable version
  (same for 0.1.0-beta.3; stable 0.1.0 still exit=0)
```

**Finding 2 (LOW) — qualification artifact `id` only checked non-empty. Fixed.**
`common.py:33` adds `ARTIFACT_ID_RE = re.compile(r"^[a-z0-9][a-z0-9_.-]+$")` — byte-identical to `qualification_artifact_index.schema.json:31` — with `require_artifact_id` at `common.py:117-120`, called at `artifact_index.py:75` before the uniqueness set. Mutation at `selftest.py:684-687`. CLI confirms `" "`, `"A"`, `"-x"`, `"_x"`, `"Archive-1"`, and single-char `"a"` all exit 2; the checked-in `compiler-aarch64-macos` still exits 0. The single-char case matters: the schema's `+` quantifier requires ≥2 characters and the validator now agrees.

### Sweeps: re-run, broadened, clean

Both sweeps were executed fresh, not re-read, over all eleven governed fixtures from `schema_contracts.schema_fixtures()`.

**Single-argument sweep — 22,583 cases** (every JSON path × 54 corrupt values including near-miss strings `0.1.0-rc.1`, `0.1.0-alpha.01`, `0.1.0-alpha.1+meta`, uppercase/63-char/65-char SHAs, trailing-whitespace and trailing-newline versions, `\x00`, `10**18`, `-abc`, `_abc`, `ABC`, `\u00e9`, plus deletion and unknown-key injection):

- **0** non-`GovernanceError` escapes from any of the eleven validators.
- **0** non-`JsonSchemaError` escapes from the schema engine.
- **0** unsafe divergences (schema rejects / validator accepts). 2,258 safe divergences (validator stricter) — expected.

**Cross-argument sweep — 5,568 cases**, the paths the single-argument sweep cannot reach: `validate_release_index_transition(previous, proposed)` in both directions, `validate_release_plan(plan, active_index=…)`, `validate_site_release_facts(facts, governed_index=…)`, and `validate_incident_request(req, live_index=…, approved_plan_digests=…)`, corrupting each argument in turn. **0** raw-exception escapes.

28,151 cases total, no unsafe divergence remaining. The pass-3 and pass-4/5/6 finding classes are both closed.

### Fresh audit

- **Workflow safety** — `preview-release.yml:65-79` rejects `X.Y.Z` and anything not `-alpha.N`/`-beta.N`; `grep 'rc\.'` returns nothing. Defence in depth beyond the regex: `propose_preview_release` (`release_index.py:133-134`) refuses to mutate an index whose `ga_status` is `active`, so the DoD "no publication workflow can accept stable yet" holds in code, not only in a shell pattern. `:142-151` enforce monotonic preview ordering and refuse to redefine an existing release record. Inputs move through `env:` rather than direct `${{ }}` splicing (`:50-58`); `source_sha` pins validate→build→publish to one commit (`:84`, `:105`, `:150`); the v1 bootstrap fallback is gone and an unavailable v2 index hard-fails (`:200-205`); the compare-and-swap re-fetches, re-validates and re-compares generation + digest (`:303-320`) in the job that exported them (`:228-231`), under the workflow-level `sifr-release-index` concurrency group (`:27-29`).
- **Evidence custody and reporting** — custody fails closed with no comparison base (`evidence_custody.py:111-116`), refuses source/evidence mixing and multi-identity commits (`:61-78`), and binds plan version, report and qualification digests, and sign-off provenance to the candidate directory (`:164-182`). `prepare_release_report_output` (`release_evidence.py:43-62`) rejects a non-`release` profile, in-repo output, pre-existing output, a non-fresh parent, and a dirty or unresolved tree; the report is validated pre-write and re-validated from disk bytes (`:109-122`).
- **Profile / documentation contracts** — `documentation_checks` is a real executable step (`profile_runner.py:508-534`) sourced solely from `selected_areas` via `selected_suites_for_area`; `_documentation_profile_self_test` (`sifr_verify/selftest.py:452-540`) proves `[sifr-lane-step] name=documentation_checks … status=pass` is emitted and that a selected-but-unrun suite fails — both DoD clauses. `tooling_suites=["full"]` expansion to exactly one `editor-release` is asserted against the real runner constant (`governance/selftest.py:371-386`), with no duplicate selection in `release.json`.
- **Scope** — no `milestone_40_1`–`40_5` work has leaked in. The residual `-rc.N` operator strings in `generate_version_installer.sh:71`, `build_preview_artifacts.sh:83`, `trigger_preview_release.sh:159`, `create_new_version.sh:187`, `generate_dispatchers.sh:88`, and `docs/self_update.md:51` are pre-existing, each gated downstream, and belong to the `milestone_40_2` strict dispatcher parser and the `milestone_40_4` GA docs check (`docs_inventory.json` keeps `ga-release` at `reserved`, so nothing regresses today). All 16 stable-gate inventory entries are owned and their locations exist.
- **Guardrails / validation re-run this pass** — governance self-tests 14/14; `governance.schema_epoch` pass; `sifr_verify --self-test` 11/11; `distribution_release:full` 41 variants, 0 failures; `evidence-custody` pass; `documentation/check_structure.py` including the GA mutation harness pass; `cargo test -p sifr --lib self_update` pass; `demos/milestone_40_0_demo.sh` exit 0; `git diff --check` clean; `cargo fmt --check` clean; file-size guardrail PASS (2841 files, limit 900). `governance/selftest.py` is 870/900 — the two new mutations cost 11 lines; a split by artifact family is worth doing before the next growth, but it is under the cap and not a blocker.

**No new findings.** I could not construct a divergence, a raw-exception escape, or a scope violation in this pass.

### Remaining blocker (external; do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, and `verification/areas/rust_interop/data/` contains only the fixture matrix, compatibility matrix, and tiers TOML — no `stable_support_claims.json`. `verification/profiles/release.json` therefore cannot list `stable-candidate`, while `release_report.py:32-43` requires it. The milestone is correctly fail-closed, confirmed by direct execution:

```
$.profile.expanded_selected_areas: missing required rust_interop suite(s): stable-candidate
```

so a real `--release-report-out` run rejects at `validate_profile` and again at `validate_steps` (`:225-228`) rather than emitting silently-incomplete evidence. This blocks two Phase 40 plan lines (`:394-395` "register the upstream stable-candidate validator", `:456-457` "the release profile visibly executes … plus stable-candidate") and the two unchecked execution-issue items at lines 54 and 57.

**Integration step once the prerequisite merges:** append `"stable-candidate"` to `verification/profiles/release.json` `selected_areas[rust_interop].suites`. It flows through `run_rust_interop_checks` → `run_selected_area` → `validate_area_result` and satisfies both required-suite checks. No other local change is needed.

## Verdict

**BLOCKED ON EXTERNAL PREREQUISITE.**

Both pass-6 findings are fully resolved — in the validator, in the mutation suite, and through the real `release_governance.py validate` entrypoint. The re-run adversarial sweeps, broadened to 28,151 cases across the single-argument and cross-argument validator surfaces, report zero raw-exception escapes and zero unsafe schema-vs-validator divergences; no divergence class from any earlier pass survives. Workflow publication safety, evidence custody and report integrity, profile selection and execution agreement, documentation contracts and mutation binding, the stable-gate inventory, scope discipline, planning status, formatting, and the file-size guardrail are all satisfied.

The local milestone is satisfactory. The only outstanding work is the `rust_interop:stable-candidate` suite and `stable_support_claims.json`, owned by another worktree; the dispatcher parser and stable behavior remain correctly assigned to `milestone_40_2`.

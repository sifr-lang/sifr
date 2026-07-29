# Round 4 review — Phase 40 Milestone 40.4 evidence/closure slice

No files modified. I read archived passes 1–3, the full working diff against `origin/main` (3 docs files, +94/−0), all six untracked review artifacts, and recomputed every digest against a real file.

## Pass-3 finding — closure status

**CLOSED & VERIFIED.** Both missing digests are now archived, and both attributions are correct.

| Claim | Archive line | Recomputed |
|---|---|---|
| First canonical report attempt log | `plans/reviews/archive/phase-40-milestone-40-4-exact-source-evidence.md:73-78` | `/tmp/sifr-phase40-release-profile-7242.XXXXXX.log` (mtime 22:45) → `babdace21ecbecf5d07f3b997ee01cfe028f6a6e58b473136d76ad6cc2678aca` — **exact match** |
| Standalone-pass benchmark evidence | `…exact-source-evidence.md:94-99` | `/private/tmp/sifr-phase40-release-source/target/performance/evidence/bench-1785182272-87135.json` (mtime 23:00) → `2bf8a8eb589cffcdd70741afd43811543af79dbc64ed8235331efe3622e8279f` — **exact match** |

Attribution spot-checks against those two files:

- **Lane list** (`plans/issues/active/adhoc_performance_budget_host_variance.md:61-67`) is exact in the `babdace2` log: `coverage_matrix_checks`, `core_guardrails`, `diagnostic_rules`, `cpython_differential`, `python_interop variants=25`, `rust_interop_checks variants=10` (consumed gate), `frontend_syntax_guardrails`, `developer_tooling_checks variants=48`, `documentation_checks variants=2` — all `status=pass`; then `performance_budget_checks status=fail` (`:159,186,221,336,368,451,554,566,606`).
- **"Three check/diagnostic medians"** is exact: `babdace2:600-602` emits exactly three regressions — `check-project-004` 3053.557>1357.524, `check-single-file-001` 1415.769>1334.139, `diagnostic-non-regression-002` 1420.408>1335.954, every one `waiver_status=no_waiver`. `lsp-query-003` did **not** regress in that run, so three (not four) is right.
- **Digest belongs to that attempt**: the log's own line `596` names `bench-1785181349-81935.json`, distinct from the standalone file — no cross-attribution.
- **1.27–1.31 s cluster** is genuinely in `2bf8a8eb`: medians 1276.8 / 1279.2 / 1281.1 ms; and the standalone log `04e8182b` (`/tmp/sifr-phase40-performance-7242-rerun-2.log`, 23:00) names `bench-1785182272-87135.json` at line 28 and ends `variants=8, failures=0` — so the evidence file is correctly bound to the standalone pass, which is where the numbers had to come from.

## Earlier findings — all remain closed

Pass-1 1–6 and pass-2 1–3 re-verified this round, not taken on trust: 40.5 checklist items (`plans/issues/active/phase-40-stable-channel-ga-execution.md:290-296`), 40.5 scope (`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:947-958`) and DoD (`:1038-1043`); "required `generation`, `ga_status`, and `releases` fields are absent"; "in a local canonical report"; "four-benchmark, five-metric set"; all archived artifacts non-empty. Every archived digest recomputed and matched: `26fd6f8c…` (index), `aea9f6f9…` (docs report), `038b0eab…` (docs result, and it is the `docs-7242e4737b1e-038b0eabc1c1` id prefix), `e31e6113…` (canonical log), `5aefc8f0…` (`target/validation_lane_reports/release.latest.json`), `3a024e88…` (sysroot JSON), `04e8182b…`, `08107ac3…`, `af2331d5…`, `4b4b752c…`, `6c9b3f10…`, `71b32439…`. Twelve for twelve. `--source-root`/`--source-commit`/`--out` and `--release-report-out` all exist (`scripts/run_all_tests.sh:24,52,114`).

## Independently re-verified this round

- **Canonical pass run** (`e31e6113`): exactly two `status=fail` lines in 1,100+ lines, both the same failure (`sysroot_release:host-installed-smoke` at `:1049` and its lane at `:1101`); `performance variants=8 failures=0` (`:557`), `distribution release variants=56 failures=0` (`:766`), `host-installed-stdlib-heavy` pass (`:1049` precedes it at `1049`/heavy pass logged just above). Matches `adhoc:78-84` and issue doc `:281-289`.
- **Stale-diagnostic defect is exactly as described**: the `c17` control's `budget-subset` (`control log:22-26`) reports 2381.003 / 1780.17 / 2281.952 / 13.393 / 15.574 — byte-for-byte the *22:51 retry's* numbers (`bench-1785181545-97661.json`), read after `benchmark-subset` failed at `:20`. `verification/areas/performance/runner.py:195-203` checks the fixed `target/performance/<suite>.budget.latest.json` unconditionally. Lane still fails → no false pass, as claimed. This also independently re-confirms the retry's five-metric/four-benchmark count.
- **Scope guards intact.** Diff touches only `plans/**` (3 files); no `crates/**`, no baseline, threshold, waiver, or profile-manifest file. Every `schema-v1` occurrence in the diff is a prohibition or an observation of external state — no v1 reader, migration, fallback, or synthesized digest; no stable mapping; no GA activation; no Rust-interop implementation. All five `milestone_40_4` boxes unchecked (`issue:129-135`), status still "In progress" (`:5`), archive states "It does not close the milestone" (`:4-5`). Milestone ownership is explicit: epoch bootstrap + endpoint override → 40.5; stale-diagnostic defect → this ad-hoc follow-up's Scope (`adhoc:94-95`).

## Remaining actionable findings

None.

Non-blocking notes (unchanged from pass 3, all previously disclosed): `plans/reviews/active/phase-40-milestone-40-4-evidence-closure-review-pass-4.md` is 0 bytes — this pass's own placeholder, must be populated before commit. The archive records reviewed head `95d5e2bbb` and source `7242e473…` without noting the merge rebased onto `6ffb34432`; the patches are byte-identical, so coverage is intact. The standalone run's sample span is 1.264–1.328 s, slightly wider than the stated "1.27–1.31", but all three medians (1.2768/1.2792/1.2811) and 13 of 15 samples fall inside it and the text hedges with "clustered around".

VERDICT: SATISFIED

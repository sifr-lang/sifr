# Review — `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`

## Findings (severity-ordered)

### P0 — Plan-blocking gaps

**1. Three of the four "must merge-gate" crates have zero tests; adding them is a no-op.**
`cargo test -p sifr_type_system`, `-p sifr_format`, and `-p sifr_lint` each report `0 passed; 0 failed; 0 filtered out` locally today. The plan's hedge "if they still exist as workspace crates" misses the actual state: the crates exist but are empty. Adding them to the merge crate list satisfies the letter of the acceptance criterion while delivering zero coverage. The plan must require either (a) seeding minimal first-party tests as part of the same wave or (b) tagging each empty crate `tests:none` with an issue link and expiry, surfaced by the coverage matrix.

**2. `sifr_ir` and `sifr_source` are first-party compiler crates the plan never mentions.**
Both have zero tests today. Wave 1's "guardrail that detects workspace first-party crates with tests but no profile membership" will not flag them (they have no tests), so the plan satisfies its own gate while leaving these crates invisibly uncovered. The "no first-party compiler crate test suite silently omitted" criterion is therefore unenforceable as written. Either widen the guardrail to "first-party crates with no test suite require an explicit decision row in the coverage matrix" or list these crates by name in Wave 1.

**3. Wave 0's enforcing coverage matrix lands before Waves 1–9 fill the surfaces.**
Wave 0 says: "Add a check that fails when a required surface is missing a merge suite, except for explicitly approved temporary exceptions with issue links and expiry dates." On day one, most rows are `missing` or `broad-only`. That forces a long allowlist of exceptions — exactly the "silent quarantine / fallback" the plan explicitly forbids in §Decisions and §Non-Acceptable Closeout States. The plan needs a defined phased-enforcement model: Wave 0 lands the matrix in *advisory* mode with a tracked list of `expected-missing` rows tied 1:1 to later waves, and the matrix check is *promoted* to blocking in Wave 10. State this transition explicitly.

**4. CPython differential subset is named but its divergence catalogue is not pre-committed.**
Sifr intentionally differs from CPython on multiple semantic axes: Result/Option error model vs exceptions, ownership/borrow, integer overflow handling (especially around the Decimal phase), default-arg evaluation, narrowing, string semantics, dict ordering, division/floor semantics, async runtime model. Wave 6 lists subset *inclusions* but no exclusion catalogue. Without that catalogue checked in *before* the oracle runs, the oracle will produce divergences that either become silent quarantine or churn the merge gate. Promote "Authoritative divergence catalogue" (`verification/policy/cpython_differential.md` enumerated, not narrative) to a *pre-Wave-6 PR* with its own acceptance criteria.

### P1 — Plan-shaping gaps

**5. Wave 5 and Wave 9 each bundle six-to-ten surfaces into a single "wave" while requiring "Each wave … opened as a PR."**
Wave 5 covers parsed shape, HIR, name resolution, type/ownership facts, CFG/flow, codegen IR, and emitted Rust — seven snapshot suites. Wave 9 covers LSP marker corpus, ecosystem expansion, package manager, stdlib parity, and runtime platform — five independent surfaces with different owners. Either drop the "one PR per wave" framing or split these into explicit numbered sub-PRs (e.g., 5.1 HIR snapshots → 5.2 CFG snapshots → … each with its own exit criteria and validation block). Project rule is small reviewable PRs.

**6. Wave 2 conflates "triage 52 failures" with "fix them and turn on the gate" in one wave.**
The 52 failures span four causes the plan itself lists (stale expectation, obsolete test, real compiler bug, unresolved production bug). Real-compiler-bug fixes should ship as their own PRs with regression fixtures, not bundled into a single "make codegen green" PR. Insert an explicit **2.0 inventory PR** that lands `plans/issues/active/codegen-test-triage.md` with one row per failure and its classification; **2.1+** then ship per-classification PRs; **2.N** flips the merge gate after the inventory closes.

**7. Merge gate budget pressure is not addressed.**
`merge.json` declares `warm_wall_time_minutes: 15` / `cold: 25`. Wave 1 adds four crate suites; Wave 2 adds ~707 codegen tests; Wave 3 wants full e2e pass (≈651 fixtures vs the current curated manifest); Wave 7 adds sanitizer smoke; Wave 4 expands diagnostic baselines. The plan addresses budget only in Wave 3 ("if it fits"). Each gate-expanding wave needs an *Evidence* sub-step: "measure current warm/cold profile-merge wall time before and after this wave; if the wave pushes over 15/25 min, ship sharding or push to nightly in the same PR." Add this as a standing requirement in §Required Tracking Updates Per Wave.

**8. "Diagnostic family" is undefined relative to the existing schema.**
`verification/policy/suite_taxonomy.md` and `verification/areas/diagnostics/manifest.json` work in terms of fixtures and `SIFR-*` codes, not families. Wave 4 introduces "family" without mapping. Replace "every active diagnostic family" with "every active `SIFR-*` code that has a stable user-facing message" and add the mapping file (e.g., `verification/areas/diagnostics/data/code_baseline_coverage.json`).

**9. Wave 3 overlaps with `nightly.json` without addressing profile boundaries.**
`nightly` already runs the full corpus. Acceptance criterion "Merge runs the full semantic e2e pass corpus or a documented deterministic full-corpus shard plan" needs to say *where* the shard plan lives (`merge.json` `legacy_facade.e2e.fixture_manifest` swap or new `shard_policy.shards: N`), how merge/nightly diverge after the change, and whether the existing `verification/areas/core_language/data/merge_e2e_manifest.json` is deleted, frozen, or split into shards.

**10. Wave 7 reinvents fuzz scaffolding the repo already has.**
`verification/areas/fuzz_property/` already declares `property`, `fuzz-smoke`, `cargo-smoke` suites and `verification/policy/fuzz_property.md` already exists. The plan never references either. Wave 7 should state explicitly: "extend existing manifests with the five new entrypoints; do not add a parallel runner; update `verification/policy/fuzz_property.md` with the sustained-lane runtime budget, seed rotation policy, and crash-promotion-to-`crashes` mechanics."

**11. "At least 50 LSP marker tests" is arbitrary.**
TypeScript fourslash has thousands; 50 is below smoke. Replace the absolute number with a coverage rule: "every documented LSP capability in `crates/sifr_lsp` has ≥1 marker test per category (diagnostics, hover, definition, references, completion, project reload, long-session), and the LSP marker corpus area-check fails when a documented capability has no marker."

**12. Profile assignment is ambiguous for non-merge work.**
The plan declares merge gates for CPython smoke and fuzz smoke but does not pin LSP corpus, ecosystem broader, stdlib parity, platform evidence, or perf trend artifacts to a specific profile. Add a table in §Decisions: surface → profile (`create-pr` / `merge` / `nightly` / `release`) → suite name. Reviewers cannot validate "this is a blocking gate" without it.

**13. Doc-update references point to a path that may not exist.**
Waves 1, 9, and 10 say "Update `verification/README.md`" but the policy lives under `verification/policy/{profile_policy,suite_taxonomy,…}.md`. The plan should target the actual docs by path: `verification/policy/profile_policy.md` (profile membership and merge gate), `verification/policy/suite_taxonomy.md` (new suite kinds), and `internal_docs/architecture.md` (verification architecture). Confirm `verification/README.md` exists before referencing it.

### P2 — Tightening

**14. Wave 5 depends on Wave 2 but the dependency is not stated.**
Wave 5's focused validation re-runs `cargo test -p sifr_codegen`. State the dependency in §Implementation Sequence: Wave 5 cannot start until Wave 2 closes.

**15. `algorithmic_compatibility` is never mentioned.**
The area already exists with a LeetCode corpus. It is either subsumed by the CPython differential (Wave 6) or a distinct surface. The coverage matrix (Wave 0) must include it as a row, and the plan must say which.

**16. Perf-trend artifact storage is unspecified.**
Wave 8 produces "median, variance, previous baseline comparison" trends but does not say where they are stored or how stale baselines are detected. Persistent baselines that drift silently are themselves a quarantine vector. Specify: artifacts checked in under `verification/areas/performance/data/trend/`, with a stale-baseline check that fails when no run in N days touches a benchmark id.

**17. Wave 6 runtime budget is unaddressed.**
"Run each generated program with `python3` and `cargo run -q -p sifr -- run`" rebuilds Sifr per invocation. Specify reuse of a release binary (`cargo build --release -p sifr` once, then `target/release/sifr run …`) and a per-program timeout/budget; otherwise the merge smoke is unbounded.

**18. "Parser fuzzing" vs "parser acceptance corpus" are conflated.**
Wave 7 says "Keep parser-fork fuzzing separate from Sifr-original compiler fuzzing." Fine. But §World-Class Verification Standard lists "parser and syntax acceptance/rejection" — that is a positive/negative source corpus, not fuzzing. Add a sentence clarifying the two are distinct surfaces with distinct rows in the coverage matrix.

**19. No durable home for review answers.**
"Initial Review Questions" exists; "Open Decisions" / "Decisions Log" does not. Reviewer responses will live in scattered PR descriptions. Add a §Decisions Log section that must be updated before each wave starts.

**20. Closeout doesn't prove the broader profiles work.**
Wave 10 runs `scripts/run_all_tests.sh` (merge) and `--profile create-pr`. It does not require a clean nightly and a clean release run before phase closure. Either add `scripts/run_all_tests.sh --profile nightly` and `--profile release` to the Wave 10 validation block or state explicitly why nightly/release closure is deferred.

---

## Suggested Plan Edits

Replace or insert the following text into the plan.

**§Decisions — append:**

```
- First-party compiler crates without any test suite (today: sifr_codegen has tests but is red; sifr_type_system, sifr_format, sifr_lint, sifr_ir, sifr_source have zero tests) must each have either:
  - a minimal unit-test seed PR landed in Wave 1, or
  - an explicit `tests:none` row in the coverage matrix with issue link and expiry no later than phase close.
- Wave 0's coverage-matrix check lands in `advisory` mode with a closed allowlist of `expected-missing` rows mapped 1:1 to Waves 1–9. The check is promoted to `blocking` in Wave 10. No new `expected-missing` rows may be added after Wave 0 lands.
- Each gate-expanding wave (1, 2, 3, 4, 7) must include measured warm/cold merge-gate wall-time before/after the change. If the change pushes warm > 15 min or cold > 25 min, the wave must ship sharding or move the broader subset to nightly in the same PR.
- Profile assignment table (authoritative after this phase):
  | Surface | Merge | Nightly | Release |
  | --- | --- | --- | --- |
  | first-party crate tests | all green crates | all green crates | all green crates |
  | e2e pass | full corpus or sharded | full corpus | full corpus |
  | diagnostic baselines | every active SIFR-* code | same | same |
  | CPython differential | deterministic seed smoke | broader generated corpus | broader generated corpus + extended subset |
  | fuzz | deterministic seed smoke | sustained per-target budget | sustained per-target budget |
  | sanitizer | smoke where host-supported | full lanes | full lanes |
  | LSP marker corpus | smoke subset | full corpus | full corpus |
  | ecosystem-broader | n/a | full pinned set | full pinned set |
  | algorithmic_compatibility (LeetCode) | representative subset | full corpus | full corpus |
- "Diagnostic family" is replaced by "every active SIFR-* code with a stable user-facing message." Mapping lives in `verification/areas/diagnostics/data/code_baseline_coverage.json`.
```

**§Existing Facts To Verify — replace the current list with measured facts already verified at plan-authoring time and the facts that still need confirmation at implementation start:**

```
Verified at plan authoring (re-verify before implementing):
- `cargo test -p sifr_codegen`: 655 passed, 52 failed, 707 total (red, excluded from merge).
- `cargo test -p sifr_type_system`: 0 tests (crate exists, no tests).
- `cargo test -p sifr_format`: 0 tests (crate exists, no tests).
- `cargo test -p sifr_lint`: 0 tests (crate exists, no tests).
- `cargo test -p sifr_ir`: 0 tests (crate exists, no tests; not in profile_runner crate list).
- `cargo test -p sifr_source`: 0 tests (crate exists, no tests; not in profile_runner crate list).
- `verification/runner/sifr_verify/profile_runner.py:307–334` hard-codes the merge crate list and omits sifr_codegen, sifr_type_system, sifr_format, sifr_lint, sifr_ir, sifr_source.
- `verification/profiles/merge.json` exists and uses `legacy_facade.e2e.fixture_manifest: verification/areas/core_language/data/merge_e2e_manifest.json`.
- `verification/areas/diagnostics/manifest.json` lists exactly two rendered baseline fixtures (`decimal_invalid_literal`, `multiline_span_rendering`).
- `verification/areas/algorithmic_compatibility` and `verification/areas/fuzz_property` already exist; this phase extends, not replaces, them.
- `verification/areas/coverage_matrix` does not exist yet (Wave 0 creates it).

Re-measure at implementation start:
- e2e pass and fail fixture counts under `verification/areas/core_language/`.
- current merge warm/cold wall time on the implementer's host.
```

**§Wave 1 — replace the "Add the omitted first-party crate suites to merge" block with:**

```
- Add the following first-party crate suites to the profile-owned merge crate list:
  - `cargo test -p sifr_codegen` (added by Wave 2 after green; Wave 1 only reserves the slot).
  - `cargo test -p sifr_type_system` — requires seeding at least one unit test in this wave; otherwise add `tests:none` row in the coverage matrix.
  - `cargo test -p sifr_format` — same rule.
  - `cargo test -p sifr_lint` — same rule.
  - `cargo test -p sifr_ir` — same rule.
  - `cargo test -p sifr_source` — same rule.
- Empty crates without seeded tests in this wave must land an issue-linked `tests:none` row in the coverage matrix with expiry no later than Wave 10. No empty crate may remain `tests:none` at phase close.
```

**§Wave 2 — split into:**

```
Wave 2.0 — Codegen failure inventory PR
  - Run `cargo test -p sifr_codegen -- --nocapture`.
  - Land `plans/issues/active/codegen-test-triage.md` with one row per failure: id, classification (stale | obsolete | compiler-bug | production-bug), proposed PR slice, owner.
  - No code changes; this PR is documentation-only and closes by reviewer sign-off on the classification.

Wave 2.1..2.N — Per-classification PRs
  - Stale expectations: one PR (snapshot regenerate explained in commit body).
  - Obsolete tests: one PR (deletion with named replacement coverage).
  - Compiler bugs: one PR per root-cause fix with a paired regression fixture.
  - Production bugs not fixable in this phase: one PR adding sentinels under `verification/areas/regression/crashes` with issue links.

Wave 2.final — Promote to merge
  - Flip `cargo test -p sifr_codegen` into the profile-owned merge crate list.
  - Re-measure merge wall time and record the delta in the closeout note.
```

**§Wave 6 — insert a Wave 6.0 prerequisite:**

```
Wave 6.0 — Divergence catalogue PR
  - Land `verification/policy/cpython_differential.md` with two enumerated tables:
    - Supported: each Python construct + the exact Sifr-equivalent behavior.
    - Excluded: each known Sifr/CPython semantic divergence (Result-vs-exception, integer-overflow policy, ownership, default-arg evaluation, division/floor, dict-ordering guarantees, string encoding, etc.), with a one-line generator-exclusion rule per row.
  - The grammar generator in Wave 6.1 must lint against this catalogue and refuse to emit programs whose semantics depend on any excluded behavior. No filtering of generated programs after the fact (that is silent quarantine).
```

**§Acceptance Criteria — append:**

```
- All empty first-party compiler crates either ship seeded tests or carry an explicit `tests:none` matrix row with issue link and expiry.
- Wave 0 coverage-matrix check is in `blocking` mode at phase close with zero `expected-missing` rows.
- The Wave 6.0 CPython divergence catalogue exists and the generator lints against it.
- Each gate-expanding wave records measured warm/cold merge wall time before and after the change.
- Profile assignment table in §Decisions is reflected by the contents of `verification/profiles/{create-pr,merge,nightly,release}.json` at phase close.
- `scripts/run_all_tests.sh --profile nightly` and `--profile release` pass at least once before the Wave 10 closeout PR.
```

**§Wave 5 — add at the top:**

```
Wave 5 ships as numbered sub-PRs, not a single PR:
  5.1 HIR-lowering snapshots
  5.2 Name-resolution snapshots
  5.3 Type/ownership-fact snapshots
  5.4 CFG/flow-fact snapshots
  5.5 Codegen IR / structured input snapshots
  5.6 Emitted-Rust snapshots for stable constructs
Dependency: blocked on Wave 2.final.
```

**§Wave 9 — add at the top:**

```
Wave 9 ships as numbered sub-PRs, not a single PR:
  9.1 LSP marker corpus
  9.2 Ecosystem-broader expansion
  9.3 Package manager integration suites
  9.4 Stdlib parity per-module suites
  9.5 Runtime/platform executable evidence
```

**§Wave 7 — replace the fuzz scaffolding bullets with:**

```
- Extend `verification/areas/fuzz_property/manifest.json` with new cases under the existing `fuzz-smoke` and `property` suites; do not introduce a parallel runner.
- Update `verification/policy/fuzz_property.md` with the sustained-lane runtime budget, seed rotation policy, corpus directory layout, and the crash-promotion mechanism into `verification/areas/regression/crashes`.
```

**Add a §Decisions Log section near the bottom (before §Review Log):**

```
## Decisions Log

| date | decision | rationale | owner |
| --- | --- | --- | --- |
| | | | |
```

---

## Verdict

**Needs another review.** Plan structure, scope, and command surface are largely correct, and validation commands resolve against the real runner. But four P0 issues — empty first-party crates being silently mergeable, missing coverage rows for `sifr_ir`/`sifr_source`, Wave 0's enforcement landing before the surfaces it gates, and the missing CPython divergence catalogue — let the plan close while violating its own "no fallback paths / no silent quarantine" rules. After the suggested edits land (especially Wave 1 empty-crate rule, Wave 0 advisory→blocking promotion, Wave 2 split, Wave 6.0 divergence catalogue, and the profile-assignment table), one more review pass is warranted before this is treated as implementation-ready.

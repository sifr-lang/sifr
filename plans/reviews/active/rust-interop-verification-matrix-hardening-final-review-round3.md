## Post-archive audit — Rust interop verification matrix hardening closeout

Read-only. No files modified (`git status --porcelain` identical before and after; all commands run were checkers/self-tests plus `cargo fmt --check`).

### 1. Archive move and archived content — correct
`plans/issues/active/rust-interop-verification-matrix-hardening.md` is deleted (` D` in status) and the successor lives at `plans/issues/archive/rust-interop-verification-matrix-hardening.md`. Diffing the archived file against `HEAD`'s active version shows the complete 284-line issue preserved verbatim except for exactly the intended closeout edits:
- Status → "Completed and archived on 2026-07-26 after final agent review and all local closure gates passed" (date matches today).
- Sibling link retargeted `rust-interop-runtime-ecosystem-certification.md` → `../active/…` (resolves).
- Phase 40 dependency sentence corrected from `milestone_40_1` to `milestone_40_0`, which now matches Phase 40's actual structure (`40:53-58` requires the artifacts *before* `milestone_40_0`; registration is at `40:401-402` and `40:419-424`, both inside `milestone_40_0`, which spans `:363-487`). This closes the cross-doc drift carried since the phase40 round-3 iteration review.
- `hardening_4` → merged with #3023 evidence; `hardening_5` → complete with the review/gate evidence.
- New `## Closeout Inventory`.

Anchor check: `#hardening_4-replace-lexical-rejection-context` resolves against `### hardening_4: Replace Lexical Rejection Context` (archive file line 208).

Inventory recomputed independently from checked-in data, not carried over: 34 fixture rows / 34 compatibility rows / 34 manifests all `schema_version: 2`; evidence 47 `passing` + 21 `planned` and nothing else; 47 validation records, 47 distinct `(test_file, test_name)`; categories 17 `supported` / 5 `supported-through-bridge` / 1 `unsupported-by-design` / 11 `future-owned-by-separate-phase`; kinds 13 `cargo-probe` / 4 `compiler-diagnostic` / 10 `contract-only` / 7 `runtime-observed`. Every number in the archived inventory and in the certification issue's restated baseline matches.

### 2. Durable links — all six repointed, none stale
Repo-wide grep for `rust-interop-verification-matrix-hardening` outside `plans/reviews/**` returns exactly six hits, all on the archive path and all resolving: `plans/roadmap.md:82`; `plans/phases/39_rust_interop.md:5` and `:275`; `plans/phases/40_…:53`; `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:8` (with fragment) and `:52`. Relative resolution verified from each file's directory. Remaining `plans/issues/active/…` occurrences are confined to historical review prose (`plans/reviews/active/**`, `plans/reviews/iterations/**`), which is permitted. No checker or script references the issue path (`check_sysroot_stdlib_resource_certification_gate.py:22` targets only the certification issue, which stays active). All 11 `future_owner` values point at the still-active certification issue.

### 3. Successor honesty and Phase 40 consistency — holds
The certification issue's new entry note (`:7-15`) claims only that `hardening_1`–`hardening_4` merged and that `certification_0` may start, and states explicitly that "the row implementation sequence remains blocked until `certification_0` completes the remaining pre-row entry criteria below." Entry Criteria (`:49-70`) keeps all four pre-row bullets. The all-profile contract is stated consistently in three places — cert issue `:167-170`, Phase 40 `:60-67` and `:419-424` — and Phase 40's exit criterion (`:464-466`) now reads "Create-PR, merge, nightly, and release visibly execute…all four structural suites plus `stable-candidate`", with the governed release report retained as the concrete-candidate authority. Phase 40's Validation Contract (`:1050`) already lists all five suites. No surviving release-only registration language.

### 4. README correction — timeless, evidence exact
`verification/areas/rust_interop/README.md:123-128` records the blocking 5,000 ms budget and the measurements **3,244 ms (create-PR)** and **3,479 ms (merge)** for all eight cases, unchanged. The post-review rewrite removed the named future-suite/phase reference in favor of "Changes to the selected suites require a complete-area measurement and a same-change budget adjustment" — no `Phase 40`, `currently`, `now`, or `today` remains in the file, so the taxonomy correction is timeless and did not disturb the measured numbers.

### 5. Reported gates supported by the stated results
`verification/profiles/create-pr.json` confirms `rust_interop_checks: {budget_ms: 5000, enforcement: "blocking"}`, so 3,317 ms is under budget. The "all 22 steps" claim is structurally exact: `profile_runner.py:64-92` defines 15 + 1 + 6 = 22 legacy-facade steps. Merge figures (Rust interop 8/8, Python interop 25/25, E2E 674/674, hardening 261/0) match the area's own variant count and the established merge-lane shape. The archived text attributes the superseding complete run, not the earlier collision-tainted attempt, and adds the honest caveat that warm-time/cache-hit/group-skew advisories concealed no failed or skipped blocking gate.

Re-run here on this tree: full area `variants=8, failures=0, blocking_failures=0`; `check_fixture_matrix` 68 cases, `check_compatibility_matrix` 4, `check_tiers` 6, `check_stale_drafts` 20 — all pass; file-size guardrail PASS (2828 files); HIR maintainability PASS; `cargo fmt --check` clean; `git diff --check` clean.

### 6. Scope, attribution, regressions — clean
Working-tree delta is Markdown-only (5 modified docs + 1 archived issue + 3 review artifacts), so no compiler/runner behavior can have regressed and round-1's no-new-panic/fallback/skip/network conclusion carries. `#3021` appears in none of the five touched documents; all four durable locations enumerate exactly #3018, #3019, #3020, #3022, #3023. The stale-draft scan passes with the issue already under `plans/issues/archive/`, which `SKIP_PARTS = {"archive", "reviews"}` intentionally excludes — the archive-scope change is by design and non-breaking, and no other checker enumerates active issue files.

### Blockers
None.

### Non-blocking observations (no change required)
- **Two different create-PR measurements coexist.** The README calls 3,244 ms the "post-`hardening_4` authoritative" figure while the archived closeout reports 3,317 ms for the closeout lane. Both are honest measurements of different runs on the same 5,000 ms blocking budget, and neither is inflated (I measured 3.16–3.48 s warm across runs), but a future reader may wonder which is canonical.
- **`check_stale_drafts.py --self-test` still absent** from the certification issue's minimum common gate (`:325-337`). Carried unchanged from rounds 1–2 and still not a coverage gap: the area runner executes it as `stale-drafts/rust-interop-stale-drafts-self-test`, and `areas run --area rust_interop` is the gate's first line.
- **This review's own artifact** `plans/reviews/active/rust-interop-verification-matrix-hardening-final-review-round3.md` is currently 0 bytes; its content needs to land there before the commit. Left untouched per the read-only constraint and not counted as a finding, consistent with how round 2 handled its own record.

Actionable findings: 0. SATISFIED.

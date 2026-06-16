I have all the evidence needed. Writing the review findings now.

## Wave 8 Closeout Review — Findings

**Scope reviewed:** PR #2637 (trend policy), #2638 (frontend edit equivalence), #2639 (trend report artifacts), #2640 (output-size metrics). All merged to `main`; current branch `codex/wave-8-closeout` matches `main` HEAD `eed947167`.

**Verdict: No blockers preventing Wave 8 closeout. A tracker-only closeout PR is appropriate.**

---

### Findings by severity

#### LOW — Tracker text is stale for Wave 8.4 (tracker-only fix)

- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:3` still says "Wave 8.1 through Wave 8.3 ... merged through PR #2639." Needs "Wave 8.1 through Wave 8.4 ... merged through PR #2640."
- Line 1459 still says Wave 8.4 is "implemented locally ... PR pending." `git log` confirms `eed947167 Track build output sizes in performance trends (#2640)` is merged. Update to "merged in PR #2640."
- The closeout PR is the natural place to record the closeout validation evidence inline (representative `variants=8 failures=0`, determinism/equivalence signature `ee5e5d44306f270c`).

#### LOW — Empty closeout review file in working tree

- `plans/reviews/active/ad-hoc-world-class-verification-wave-8-closeout-review-pass-1.md` exists as a 0-byte file (the only untracked diff per `git status`). Either populate it with this review or delete before the closeout PR — committing the empty file would be misleading.

#### LOW (operational, not a Wave 8 blocker) — Baseline freshness window expires soon

- `verification/areas/performance/data/trend/current.json` was captured at unix `1778968823` = **2026-05-16**. Policy `baseline_window_days=45` means the stale-baseline gate begins failing every benchmark around **2026-06-30** — 14 days from today. The existing deferral `wave8-legacy-baseline-environment-metadata` is metadata-only and does NOT extend the per-benchmark freshness check (`check_trend_policy.py:307-308` only consults `benchmark_ids`).
- This is documented in tracker line 1439 as a known fact ("approved reference-run refresh required before ~2026-07-01"). It does NOT block Wave 8 exit per the exit criteria, but the closeout PR should at minimum name an owner and date for the refresh, or schedule it as a Wave 9.0/maintenance follow-up. Otherwise the refresh deadline will silently break ordinary `create-pr` runs in two weeks.

#### LOW (operational guidance) — Serial-only invocation of merge-profile reruns

- `check_report_determinism.sh --profile merge` and `check_sequential_parallel_equivalence.sh --profile merge` both drive the full e2e corpus and share filesystem state under `target/`. The concurrent attempt's I/O/logging fixture failures were a predictable consequence, not a real-bug signal. The serial reruns producing the same signature `ee5e5d44306f270c` are sufficient evidence.
- Tracker lines 1429–1430 list these as two separate commands without noting they cannot be parallelized. Add a one-line operational note in the Wave 8 closeout entry stating these must be run serially against `--profile merge`. No implementation action needed; this is documentation only.

---

### Answers to your five questions

1. **Blockers preventing Wave 8 closeout:** None. Exit criteria are met by merged work.

2. **Wave 8 metric coverage gaps:** None that block Wave 8 closeout.
   - All required goal metrics are tracked: compile wall time (`median_ms`/`p95_ms`), peak RSS (`peak_rss_bytes`), emitted Rust lines/bytes and binary size (Wave 8.4 — `tracked_optional_metrics` in policy, populated for `mode: build` cases), diagnostic rendering time (`phase27-non-regression-002/003/004`), LSP cold start (`lsp-query-002-cold-start`), LSP steady-state edit latency (`lsp-query-003..018`).
   - `emitted_rust_lines`/`emitted_rust_bytes`/`generated_binary_bytes` are currently `null` for every entry in `current.json` because the checked-in baseline pre-dates Wave 8.4. The schema validator permits this (`check_trend_policy.py:278` — null is acceptable for tracked-optional). Populating them requires the next approved reference-hardware capture, which is the Wave 8.1 baseline-refresh ticket already named in the tracker. This is a baseline-refresh operational task, not a Wave-8 code gap.
   - Package resolution/install time is correctly deferred to Wave 9.3 ("where applicable" — package management is not yet shipped at the merge-blocking level).

3. **Honest representation of non-incremental/incremental boundary:** Yes. `query_diagnostics_equivalence_tests.rs` proves the cache-assisted-edit-vs-clean-context equivalence the tracker explicitly asks for in lines 1383–1389. The tests do not claim true incremental compilation — they prove that repeated/edit workflows on a single `FrontendContext` produce identical `RenderedDiagnostic` and module-graph edges to a freshly-loaded `FrontendContext`. All four named edit scenarios are covered (success-preserving, diagnostic-introducing, diagnostic-recovery, project-graph dependency change at `:67-122`).

4. **Sufficiency of serial determinism/equivalence reruns:** Yes, sufficient. Both scripts produced signature `ee5e5d44306f270c` against the merge profile in their serial reruns. The concurrent attempt's failures were filesystem-collision artifacts (shared `target/` state), not real determinism violations — once serialized, both came back green and matching. No tracker note is strictly required, but a one-line "run serially" annotation is worth adding to prevent future repetition (see LOW finding above).

5. **Tracker-only closeout PR appropriate:** Yes. Recommended contents:
   - Update tracker status line 3 to include Wave 8.4 / PR #2640.
   - Update Wave 8.4 section status from "PR pending" to "merged in PR #2640."
   - Add Wave 8 closeout validation block with: `areas run --area performance --suite representative` (variants=8, 0 failures), `check_report_determinism.sh --profile merge` (signature `ee5e5d44306f270c`), `check_sequential_parallel_equivalence.sh --profile merge` (same signature), and create-pr signature `5edef8cd4b961ef8`.
   - Optionally add a one-line operational note that these two merge-profile scripts must run serially.
   - Either populate or delete the empty `plans/reviews/active/ad-hoc-world-class-verification-wave-8-closeout-review-pass-1.md` before the PR.
   - Optionally name the owner/date for the baseline-refresh due before 2026-06-30.

**No further review round is required for the Wave 8 implementation itself.** A follow-up review of the closeout PR text is optional and would only need to confirm the tracker edits above.

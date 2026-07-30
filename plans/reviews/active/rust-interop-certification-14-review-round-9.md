I verified every fix and claim against durable artifacts at exact head `5a14ae4dfe94216a5d8ad678c75eac19ab2d0e42`.

## Round-8 findings — status

**1. Lock-wait wording — FIXED.** `target/validation_lane_reports/merge.latest.log` contains exactly one `Blocking waiting for file lock on package cache` (line 354); the ledger now reads "a Cargo package-cache lock wait" (`rust-interop-runtime-ecosystem-certification.md:1866-1867`). Singular matches the log.

**2. Incident in the governing performance ledger — RECORDED, but the control is misstated.** See finding below.

**3. Durable continuation summaries — FIXED.** `plans/reviews/active/rust-interop-certification-14-merge-continuation-evidence.md` is checked in and every compact rerun reproduces exactly:
- Project matrices: JSON at the unique path exists, `total_variants=2, total_failures=0`, SHA-256 `a5d8c3a8…9d81a` matches the doc byte-for-byte. Seven rows confirmed from `verification/areas/project_workspace/data/validation_suites/manifest.json` (2 + 5) with names matching the doc exactly.
- CLI generated `6/6` in 164.08s, driver generated `65/65` in 1794.34s — exact match to `/tmp/rust-interop-cert14-cli-generated.log` and `-driver-generated.log`, including the three matching frozen/locked/offline builds and five rejection diagnostics.
- E2E: `/tmp/rust-interop-cert14-e2e.log` matches every quoted line — `cache_hits=178/178`, `groups=178`, `report_signature=5e45a6a7b96f2688`, `678 passed, 0 failed`, `finished in 30.99s`.

Also re-confirmed: bench evidence `bench-1785416213-22823.json` carries 1358.717 / 1366.015 / 1354.814 / 5.962 / 11.664; the cross-branch control log timestamps 16:04; diff vs `origin/main` is 24 files, the only Rust change still a `cfg(test)` assertion-message edit — the performance failure remains non-attributable and correctly held as the governed `PERF-HOST` exception. Tracking state is right: final checklist box open, `certification_14` row `in progress`, round-8 summary bullet accurate.

## Actionable finding

**LOW — accuracy — `plans/issues/active/adhoc_performance_budget_host_variance.md:119-121`**

> "an unrelated class-field branch on the same host failed the **same four cases** much more severely at 3313.437 ms, 4612.439 ms, 17.918 ms, and 22.939 ms"

The control log (`/tmp/sifr-class-field-item2-performance-pass14.log:32-36`) reports five regressions across four cases: project-graph 3313.437, arithmetic 4612.439, **json-diagnostic 4132.029** (threshold 1335.954), LSP median 17.918 and LSP p95 22.939. The paragraph drops the json-diagnostic measurement and puts the LSP p95 in its place, so the four listed numbers do not map to the four cases it claims — and it is inconsistent with its own preceding sentence, which correctly enumerates the closeout as four cases plus a separate p95 (`…1354.814/1335.954 ms, and 5.962/5.91 ms median plus 11.664/10.933 ms p95`). The omitted datum is the strongest one in the control: a 3.1× overshoot on the very case the closeout missed by 1.4%. Correct to `3313.437 ms, 4612.439 ms, 4132.029 ms, and 17.918 ms median plus 22.939 ms p95`.

This is the same class and severity as round-8 finding 1, which was accepted as actionable, and it lands in the ledger entry created specifically to close round-8 finding 2.

## Minor, non-blocking

`merge-continuation-evidence.md:105-106` states the earlier cold continuation "took 801.93 seconds with zero cache hits." That figure appears in no durable artifact (grep finds it only in this doc). The load-bearing claims — 678/678 and the signature — are now durably reproduced, so this is a corroborating detail only; worth either citing a log or dropping the number while the paragraph is being touched.

VERDICT: NOT SATISFIED

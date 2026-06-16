Reviewed the Wave 8.1 performance trend policy implementation. Findings below.

## 1. Blocking findings

**None.** The implementation correctly enforces every-id-or-deferral coverage, stale-baseline windows, rename mapping, required metrics/cache/metadata, and gates the contracts/smoke/representative/full suites. The legacy deferral is appropriately narrow (4 named metadata fields, owner + rationale + 2026-07-31 expiry) and trend deltas are not computed locally, so noisy local timings cannot fail developer machines.

## 2. Non-blocking suggestions

- **`required_metrics` accepts `None`** — `verification/areas/performance/check_trend_policy.py:261` only calls `require_number` when `metrics[field] is not None`. For `required_metrics` (median_ms / p95_ms / mad_ms / coefficient_variation / peak_rss_bytes) a null value should fail; today the same `is not None` gate is shared with `tracked_optional_metrics` at line 266 where null is intentional. Tightening required-vs-optional would prevent future baselines from silently regressing to null statistics.
- **`manifest_sha256` is unvalidated** — `verification/areas/performance/data/trend/current.json:17` records a manifest hash but `check_trend_policy.py` never compares it to `sha256(benchmark_manifest.json)`. Today the manifest-vs-results id set is checked, but a future stealth manifest mutation that preserves ids would not be detected. Consider validating the hash matches the current manifest.
- **Wildcard `"*"` deferrals are accepted but undocumented** — `check_trend_policy.py:156,296,300`. A single deferral with `benchmark_ids: ["*"]` would silently disable trend coverage for the entire corpus. Document the wildcard in `trend_policy.json` or restrict it to metadata only.
- **Rename does not require `old_id ∉ cases`** — `check_trend_policy.py:165` validates rename entries but doesn't enforce that `old_id` is actually a retired id. A rename entry whose `old_id` is still an active manifest case is ambiguous and could mask a missed cleanup.
- **Operational runway is tight** — baseline captured_at `1778968823` (~2026-05-17) with `baseline_window_days: 45` means the stale-baseline gate begins firing around 2026-07-01. The legacy deferral (expires 2026-07-31) covers only `metadata_fields`, not freshness, so the approved-reference-run cadence needs to be in place before then or every create-pr/merge run starts failing. The tracker entry should note this lifecycle obligation.
- **Trend tracker entry wording** — `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1438` says "create-pr, smoke, representative, and full" run the trend policy. The actual wiring runs trend policy via the `contracts` manifest suite and via `run_contract_variants` for `smoke`/`representative`/`full`. Consider clarifying that "contracts" (not "create-pr") is the suite name; create-pr is the profile that selects contracts.

## 3. Re-review

Not required for this slice. The non-blocking items can be addressed in the next Wave 8 sub-PR (or a follow-up tightening pass) without gating this merge. If the required-metric nullability or manifest-hash validation is tightened in this PR, a quick targeted re-read of just those diffs is sufficient — no full review pass needed.

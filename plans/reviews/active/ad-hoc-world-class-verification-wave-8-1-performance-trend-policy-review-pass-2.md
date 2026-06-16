## Wave 8.1 review pass 3 — targeted reread

### Verdict
**No blocking issues. No re-review required.**

### Confirmation of each review-driven change

- **Required metrics reject null while optional permit null** — confirmed at `check_trend_policy.py:271-279`. Required metrics now unconditionally call `require_number` (no `is not None` skip); `tracked_optional_metrics` still keeps the `metrics[field] is not None` guard. The `null_required_metric` self-test at `check_trend_policy.py:391-396` sets `median_ms = None` and asserts "must be numeric".

- **manifest_sha256 compared to benchmark_manifest.json** — confirmed at `check_trend_policy.py:196-199`. `validate_manifest_hash` recomputes the sha256 of the manifest file and rejects mismatches. The `wrong_hash` self-test at `check_trend_policy.py:384-389` exercises this.

- **Wildcard `"*"` benchmark deferrals rejected** — confirmed at `check_trend_policy.py:159-160`. Metadata `"*"` remains valid (consumed by `has_metadata_deferral`). The wildcard self-test at `check_trend_policy.py:368-382` exercises this.

- **Rename `old_id` must not be an active benchmark id** — confirmed at `check_trend_policy.py:181-182`, before the duplicate-old-id and unknown-new-id checks. The `active_old_id` self-test at `check_trend_policy.py:351-366` exercises this with `manifest_ids[0]`.

- **Tracker wording + lifecycle note** — confirmed at `ad-hoc-world-class-verification-standard-and-gate-closure.md:1438-1439`. Line 1438 now says "the performance `contracts` suite and the smoke, representative, and full performance profiles" (no longer conflating create-pr profile with the contracts suite). Line 1439 adds an explicit baseline refresh obligation: an approved reference run is required before the 45-day stale-baseline gate begins firing around 2026-07-01 (legacy deferral covers metadata only, not freshness).

### Validation results

| Check | Result |
| --- | --- |
| `python3 -m py_compile …` | OK |
| `jq empty …` on manifest.json, trend_policy.json, current.json | OK |
| `check_trend_policy.py --self-test` | `performance trend policy self-test passed` |
| `check_trend_policy.py` | `performance trend policy check passed` |
| `sifr_verify areas run --area performance --suite contracts` | variants=6 failures=0 blocking_failures=0 |
| `sifr_verify areas run --area performance --suite smoke` | variants=7 failures=0 blocking_failures=0 |

### Non-blocking observations
None worth flagging — every previous suggestion landed cleanly and the negative self-tests are tight enough to be load-bearing rather than ornamental.

### Re-review needed?
No. The slice is ready for PR.

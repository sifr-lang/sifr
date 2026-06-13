

## M0 Code Review — LeetCode Failed Benchmark Inventory

**Reviewer is satisfied for M0. No blocking issues.**

### What was changed

**`benchmarks/failed_inventory.py`** (new file, 365 lines): The core data and logic module. Contains:
- `FAILED_DETAILS`: 53-entry dict mapping problem_id → `{primary_track, failure_mode, first_fix}` — one row per entry in the phase doc table, all cross-validated
- `build_inventory()`: assembles the JSON, sorted by problem_id, filtering to `is_partial | has_no_pair` analyses
- `validate_details()`: self-contained seed alignment check (FAILED_DETAILS vs FAILED_SEED, valid track/mode values)
- `failure_excerpt()`: moved from `analyze_slowness.py` to share it

**`benchmarks/analyze_slowness.py`** (delta: +24/-12 lines): Minimal extension that wires the new module in:
- Imports `build_inventory`, `failure_excerpt`, `validate_details` from `failed_inventory`
- Adds `--failed-json` and `--generated-at` CLI args
- Calls `build_inventory()` when `--failed-json` is given; output is `sort_keys=True` (alphabetical top-level key order)
- Integrates `validate_details()` into `validate_metadata()` so `--check-metadata` catches alignment issues

### Validation results

| Check | Result |
|---|---|
| Row count | 53 rows, exact match with phase doc table |
| `FAILED_DETAILS` ↔ `FAILED_SEED` alignment | 53↔53, zero missing, zero extra |
| `validate_details()` | No diagnostics |
| All `primary_track` values | Valid set: `benchmark_harness` (28), `leetcode_sifr_code` (25) |
| All `failure_mode` values | Valid vocabulary (11 modes across 53 rows) |
| All `benchmark_status` values | `failed_build` (49), `failed_correctness` (2), `failed_timeout` (1), `partial` (1) |
| `related_slowness_phase` | `True` for 0234 only, `False` for all other 52 — matches SLOWNESS_SEED |
| `failure_excerpt` | Non-empty for all 53 rows |
| Sorting | Deterministic: problem_id ascending |
| `benchmark_status` for 0234 | `partial` ✓ |
| Determinism | Two back-to-back runs with fixed `--generated-at` produce byte-identical JSON |
| `--check-metadata` | Exits 0, no diagnostics |
| Line counts | `analyze_slowness.py` 440, `failed_inventory.py` 365 — both well under 900-line guardrail |
| Schema version | `leetcode_failed_benchmark_inventory_v1` ✓ |
| Track/mode per phase doc table | Cross-checked all 53 — 100% match |

### One advisory note (not a blocker)

The phase doc "Analyzer Schema" section lists top-level keys in a specific order (`schema_version`, `source_raw_dir`, `generated_at`, `problem_count`, `problems`). The implementation uses `json.dumps(indent=2, sort_keys=True)` which produces alphabetical key order (`generated_at`, `problem_count`, `problems`, `schema_version`, `source_raw_dir`). This is standard JSON canonicalization and is semantically equivalent — all consumers read by key name, not position. No fix needed.

### Conclusion

M0 is a clean, minimal, well-validated extension of the slowness phase analyzer. All 53 rows are locked, classification is consistent with both the phase doc table and the slowness phase metadata, the JSON is deterministic, and `validate_details()` ensures future changes to `FAILED_SEED` will catch mismatches. Ready to merge.

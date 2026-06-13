# Code Review: Validation Lane Rebalancing

## Findings (ordered by severity)

### 1. CRITICAL — Performance budget retry hides legitimate failures
`scripts/run_all_tests.sh:280-294` retries the performance budget subset up to **5 times with unchanged thresholds**:

```bash
for attempt in 2 3 4 5; do
  echo "performance budget subset failed; retrying attempt ${attempt}/5 with unchanged thresholds"
  if run_performance_budget_subset "${RETRY_PERF_RESULTS}"; then
    PERFORMANCE_PASSED=1
    break
  fi
done
```

This is a direct regression against the issue's stated goal of "deterministic, actionable failure evidence" (root cause #6) and against the broader "compiler-relevant" gate posture. A budget that fails once and passes on retry-3 is a flake, not a pass — silently masking it short‑circuits the entire purpose of the budget. Either move retries behind an explicit `--allow-perf-retries` flag, halve the count to a justified value with rationale, or remove entirely.

### 2. CRITICAL — Policy doc and manifest disagree on create-pr contract matrix
`internal_docs/validation_lane_policy.md:14-25` lists "frontend-mode validation contract matrix" as part of the create-PR lane. But `verification/validation_lanes/manifest.json:17` sets `"matrix_suites": []` for `create-pr`, and `scripts/run_all_tests.sh:393-405` `run_validation_contract_suites` no-ops when `CONTRACT_SUITES` is empty.

Result: the documented "fast compiler-relevant" gate ships **zero** contract-matrix coverage, contradicting the policy doc that just landed in the same PR. Either add `frontend_mode_parity` to the manifest's create-pr `matrix_suites`, or strike the bullet from the policy doc.

### 3. CRITICAL — Issue marked `status: implemented` with acceptance items unchecked
`issues/ad-hoc-pr-gate-speed-and-validation-lane-rebalancing.md:3` states `status: implemented`, but two acceptance criteria are still `[ ]`:
- L127 (milestone 4): "generated-code smoke is under 30s warm; full corpus reports per-fixture timings…"
- L134 (milestone 5): "create-PR lane stays under 120s warm and under 300s cold, while merge lane preserves documented coverage."

The measured warm 94.89s and smoke 17.57s appear to satisfy these, but the issue is internally inconsistent and the "Required Validation" closing section (L142-153) is missing the warm/cold before/after table the issue itself mandates. Don't ship `implemented` without checking the boxes and providing the table.

### 4. HIGH — Lane policy duplication contradicts the policy doc's own claim
`internal_docs/validation_lane_policy.md:12` says: *"`scripts/validation_lane.py` is the only shell-facing resolver for lane metadata; scripts should not duplicate profile policy."* Yet:

- `scripts/run_verification_hardening/core.py:91-98` reimplements `canonicalize_profile` (now hardcoding `quick→create-pr`, `pr|full→merge`).
- `scripts/run_verification_hardening/core.py:145-157` hardcodes the merge-lane suite list (`diagnostics, project, fixedbugs, crashes, oss-curated`) — which is also in `manifest.json:69-75`. Two sources of truth.
- `scripts/run_e2e_pass.sh:50-83` `set_profile_defaults` reimplements worker counts that already live in `manifest.json` `e2e.*_jobs`.

Either route all of these through `validation_lane.py` (the stated single resolver) or weaken the doc claim. Right now the doc is enforceable in spirit but provably false in code.

### 5. HIGH — `exit 1` inside `timed_step` bypasses lane-step status emission
`scripts/run_all_tests.sh:292` uses `exit 1` from `run_performance_budget_checks`. `timed_step` (L100-120) wraps the call with `set +e; "$@"; status=$?` — but `exit` terminates the shell **before** the closing `echo "[sifr-lane-step] … status=fail"` runs. The whole purpose of milestone 1 was machine-readable per-bucket timing including failures (issue L108). On a real perf failure, the JSON report loses the `performance_budget_checks` step entirely. Change to `return 1`.

### 6. HIGH — `runner.rs` silently leaks the temp dir on failure
`crates/sifr/tests/validation_contract_support/runner.rs:60-87`: the `let _ = std::fs::remove_dir_all(&tmp_dir);` was moved **inside** the inner closure. Now any failure in `temp_root`, `run_row_commands`, or `apply_assertion` returns `Err` before cleanup, leaking the directory under `target/`. The old code cleaned up unconditionally.

If the intent is "preserve evidence on failure," say so with a comment **and** print the preserved path (like `generated_code_quality.py:543-544` does). Otherwise restore the unconditional cleanup. As written, it's an accidental behavior change.

### 7. MEDIUM — `nightly` and `release` lanes are nearly identical
`verification/validation_lanes/manifest.json:79-164`: every field except `extra_checks` is byte-identical between `nightly` and `release` (same matrix_suites, tooling_suites, distribution, generated_code_quality, performance_budget, e2e, hardening_suites). The only release-only signal is `e2e_report_determinism` and `e2e_sequential_parallel_equivalence`. That's a thin justification for a 4th lane and contradicts the policy doc's "highest-confidence release qualification" framing. Either widen the release lane (additional cold/thermal coverage, longer-running fuzz, larger sample-scale) or collapse `release` into `nightly` with a `--extra-checks` opt-in.

### 8. MEDIUM — Dead exports in `validation_lane.py`
`scripts/validation_lane.py:126-129` emits `RUN_FRONTEND_MODE_PARITY`, `RUN_PHASE23_GRAPH_ISOLATION`, `RUN_PHASE24_HIR_ANALYSIS`, `RUN_PHASE25_CFG_FLOW`. `grep` confirms these are read **nowhere** (`grep` returns the source file only). Either consume them in `run_all_tests.sh` for fine-grained per-suite gating, or delete to avoid drift.

### 9. MEDIUM — Hardening JSON `profile` field semantics changed
`scripts/run_verification_hardening/core.py:91-98` now canonicalizes `pr→merge`. `main_flow.py:218` writes `args.profile` into `target/verification/hardening-results.json`. Any consumer that previously matched on `"profile": "pr"` now needs `"profile": "merge"`. There's no schema version bump or migration note. If you intend to keep this, bump `schema_version` and call it out in the policy doc.

### 10. MEDIUM — `e2e_metrics` dict carries both `groups` and `group_count`
`scripts/validation_lane_report.py:243-261`: `E2E_TIMING_RE` writes `group_count` (the denominator after `cache_hits=`), then `E2E_GROUP_RE` overlays `groups`, `largest_group_fixtures`, `median_group_fixtures`. Two keys for the same concept. The display at L408 uses `group_count`; the advisory at L164 uses `group_count` and `largest_group_fixtures`. If `E2E_GROUP_RE` ever lands without `E2E_TIMING_RE` (regex change, log truncation), `cache_hit_rate` silently goes to None. Normalize to one key and assert the regex still matches the e2e bucket's actual output.

### 11. MEDIUM — Generated-code cache key only hashes the entry file
`verification/generated_code_quality/generated_code_quality.py:327-334` builds the cache key from `entry.id || source_path || absolute_source.read_bytes()`. Multi-module fixtures (`multi-module-projects` group, REQUIRED_GROUP_COUNTS:35) have **secondary modules** that aren't in the hash. The shared root is wiped at the top of `run_generated_code_quality_checks` (`run_all_tests.sh:323`) so within a single lane invocation this is benign, but if anyone ever points `SIFR_GCQ_SHARED_ROOT` at a persistent dir (e.g. for fast iteration), a stale secondary module produces a false cache hit. At minimum, document the cache-key contract; better, hash the whole entry directory.

### 12. LOW — `gate_demos` mutates the parsed `args`
`verification/generated_code_quality/generated_code_quality.py:696-698`:
```python
def gate_demos(entries, args):
    args.group = ["demos-required"]
    gate_corpus(entries, args)
```
Today each mode is its own process so this is harmless, but it's a footgun if anyone ever calls multiple gates in-process. Pass a fresh `argparse.Namespace` or a small kwarg instead.

### 13. LOW — `/usr/bin/time -l` is macOS-only
`scripts/run_all_tests.sh:64`: `-l` is BSD/macOS. On Linux, GNU time uses `-v` and the `BSD_TIME_*` regexes in `validation_lane_report.py` won't match. AGENTS.md says "CI mirrors these exact scripts — no CI-only behavior." If CI runs on Linux, max_rss/swaps parsing silently produces no advisories. Detect platform or use a Python `resource` helper.

### 14. LOW — `set_profile_defaults` in `run_e2e_pass.sh` duplicates manifest jobs
`scripts/run_e2e_pass.sh:50-83`: per-profile job counts duplicate `manifest.json` `e2e.{sifr,rust,run}_jobs`. Drift risk if someone tunes one and not the other. Make `run_e2e_pass.sh` read the manifest via `validation_lane.py shell` when invoked standalone.

### 15. LOW — `scripts/run_distribution_validation.sh:31` exits on first failure but skips the rest of the loop
`exit "${status}"` halts the loop after the first failing distribution script. The per-case timing records that would be useful for diagnosing fan-out failure are lost. Consider accumulating failures and exiting at the end of the loop (matches the spirit of milestone 1's per-case timing visibility).

## Lane composition coherence

The four-lane split is **structurally sound**: create-pr is genuinely a smoke profile, merge is representative, nightly is full corpus, release adds determinism/equivalence. Aliases (`quick→create-pr`, `pr→merge`, `full→merge`, `stress→release`) preserve backward compatibility cleanly.

But two coherence concerns survive:
- **Boundary blurring** (finding #7): `nightly` and `release` are too close to justify two lanes.
- **Doc/manifest skew** (finding #2): the create-PR coverage claim in the policy doc is not honored by the manifest.

## Measured-evidence sanity check

- 94.89s warm < 120s warm target → milestone 5 acceptance is satisfiable; tick the box.
- 17.57s generated-code smoke < 30s → milestone 4 acceptance is satisfiable; tick the box.
- 3.18s warm for the diagnostic source canonicalization wrapper, against the ≤10s milestone-2 target → comfortably met.
- 18/18 cache_hits warm and 0/18 cold with median group 2, largest 8 → group skew advisory should not trigger (ratio 4.0, abs delta 6 < 8). Good.

## Verdict

**Request another revision round.** The structural design is solid and the measured numbers hit their warm targets, but findings #1–#3 are blockers — a performance gate that retries 5× silently, a policy doc that contradicts its own manifest, and an `implemented` status with unchecked acceptance items together undermine the very "deterministic, actionable" posture this phase was meant to establish. Findings #4–#6 are also non-optional before merge: the policy doc states a "single resolver" invariant that the code violates, `exit 1` defeats the new lane-step instrumentation, and the runner.rs cleanup regression is silent.

Once those six are addressed, the remaining items are reasonable follow-ups.

All validation reproduces cleanly. Now compiling the review report.

## Findings

### Pass-1 actions — verified resolved

1. **`pydantic-core-native` removed.** `packages/native.toml` now contains exactly `grpcio`, `lxml`, `tiktoken`, `google-crc32c`. The phantom entry is gone; `pydantic-core` keeps its single canonical home in `tier1.toml:7` with `groups = ["imports", "native"]`.
2. **Tier 1a/1b discrimination landed.** `gate = "tier1a"|"tier1b"` annotates every Tier 1 entry; `runner/smoke_matrix.py:23` exposes `KNOWN_GATES`; `runner/run.py:56` plumbs `--gate`; `runner/run.py:73,134-135,160` validates and filters. `--gate tier1a` selects exactly 21 packages and they match the phase plan's Tier 1a list byte-for-byte (set diff empty, both 21). `--gate tier1b` selects 25. `--gate tier99` exits with `unknown python interop gate filter(s): tier99`. This closes pass-1 finding #2 cleanly.
3. **`--self-test` short-circuits** at `run.py:66-69` — returns 0 after self-tests, no report written. Verified `reports/` stays clean after `--self-test`.
4. **Unused `from typing import Any` removed** from `run.py` (only `argparse`, `sys`, `Path`, plus internal `env`/`import_matrix`/`report`/`smoke_matrix` imports remain).
5. **`summary.total_variants`** at `run.py:97` is now `max(1, len(selected))`. Verified: 21 for `--gate tier1a`, 25 for `--gate tier1b`, 1 for `--group native --package numpy`. Honest evidence even at scaffold.
6. **Probe stubs documented.** Each of `native_probe.py`, `callback_probe.py`, `zero_copy_probe.py`, `resource_probe.py` carries a single-line "Filled in by milestone_py_N…" comment naming the responsible milestone.

Pass-1 findings #5 (hardcoded `MATRIX_FILES`) and #7 (default `reports/latest.json` clobber) remain unaddressed — both were cleanup-only and pass-1 didn't ask for them pre-PR. Acceptable to defer.

### New observations from this pass

7. **Tier 1b matrix membership drifts from the phase plan's bullet lists.** Six matrix entries carry `gate = "tier1b"` but do not appear in the corresponding section of `plans/issues/active/ad-hoc-embedded-python-interop.md:576-585`: `grpcio` (`native.toml`), `pika` and `moto` (`brokers.toml`), `pillow` / `tensorflow` / `scikit-learn` (`data.toml`). Either the plan's section bullets are non-exhaustive (in which case the matrix is the source of truth and the plan should say so) or these are matrix-only additions that need to be reflected in the phase plan. Not a blocker for py_0 scaffold, but milestone_py_11 will run the gate against whichever side is authoritative.
8. **No tier ↔ gate consistency check.** `validate_matrix_entries` (`run.py:126-139`) accepts `tier = "tier3"` + `gate = "tier1a"` silently because `gate` membership in `KNOWN_GATES` is checked independently of `tier`. A typo in a future tier2/3/4 entry would survive validation. One-line fix when convenient: `if entry.gate and not entry.tier == "tier1": raise SystemExit(...)`.
9. **`gate` is unconditionally optional.** A Tier 1 entry added without `gate` would silently match neither `--gate tier1a` nor `--gate tier1b` (no enforcement that tier1 implies a gate value). Easy to enforce: when `entry.tier == "tier1"`, require `entry.gate is not None`. Mild; same shape as #8.

None of #7–#9 are blockers for py_0 acceptance; they are concrete tightening targets when the gate becomes load-bearing in milestone_py_11.

### Spot-checked, all clean
- `crates/sifr_diagnostics/src/codes/registry.rs`, `registry_entries/reserved.rs`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md` still carry the 8 reserved families consistently (no change since pass 1).
- `crates/sifr_lowering/src/lower/ipc_payload_calls.rs` remains the same rustfmt drift pass 1 noted.
- All 9 matrix files / 21 fixtures / 9 runner modules still present.
- `runner/import_matrix.py:13,32` correctly adds optional `gate` to `PackageEntry`; `optional_string` rejects empty strings — good.
- Self-test exits before any report write; no `reports/latest.json` clobber from `--self-test`.

## Open Questions

1. Is the phase plan's Tier 1b bullet list at `plans/issues/active/ad-hoc-embedded-python-interop.md:576-585` exhaustive, or representative? Six matrix entries (#7 above) are gated `tier1b` without appearing there. Whichever side is authoritative, the other should match before milestone_py_11.
2. Should `gate` be required for `tier == "tier1"` entries, and rejected for non-tier1 entries (findings #8 and #9)? Cheap to enforce now while the matrix is small.
3. Carrying forward from pass 1: host-dependent packages still have no structured `skip_when`/`evidence` field. Confirmed deferred to milestone_py_11 — flagging so it doesn't get forgotten.

## Verdict

**No blocking issues. milestone_py_0 is ready to ship.** The single pass-1 actionable (`pydantic-core-native`) is resolved, and the gate-metadata work goes further than pass 1 asked for: it closes pass-1 finding #2 with a clean filter API plus validator, the Tier 1a set matches the phase plan exactly (21/21, no diff), and the other recommended cleanups (#3, #4, #6, plus probe-stub comments) all landed. Local validation reproduces: self-test, `--gate tier1a`, `--gate tier1b`, `--gate tier99` (correctly rejected), `--group native --package numpy`, and `uv lock --check` all behave as described. Findings #7–#9 are post-merge tightening for milestone_py_11; do not gate this PR on them.

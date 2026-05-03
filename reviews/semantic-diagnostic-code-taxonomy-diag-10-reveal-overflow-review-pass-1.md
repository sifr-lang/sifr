

## Review Findings

### Finding 1 — Dedupe inconsistency with `omitted_kind` (HIGH severity)

**Files:** `crates/sifr_diagnostics/src/codes.rs:1355`, `crates/sifr_driver/src/diagnostics.rs:172-181`

The registry declares `dedupe_args = ["omitted_kind", "cap_kind"]` for `SIFR-INTERNAL-0002`. Two recovery-omission summaries with different `omitted_kind` values (e.g., `"reveal_type results"` vs `"diagnostics (including 5 reveal_type results)"`) but the same `cap_kind` will NOT be deduplicated — they have different `RecoveryDedupeKey` values because `omitted_kind` is part of the dedupe key.

This is a regression: the previous implementation had no `omitted_kind` field, so all omission summaries with the same `cap_kind` deduplicated together. Now, if a compilation produces multiple top-level omission events with different mixes of reveal_type results, they will all appear as separate summary diagnostics instead of being collapsed.

**Example scenario not covered by tests:** A file that triggers both a similar-group cap (with some reveal_types omitted) and a top-level cap (with different reveal_type count) would produce two separate `SIFR-INTERNAL-0002` summaries when logically they should be one.

---

### Finding 2 — No test for `omitted_kind` in similar-diagnostic-group cap path (MEDIUM severity)

**File:** `crates/sifr_driver/src/diagnostics.rs:82-89`

The `omitted_kind()` function is exercised by the top-level cap tests (lines 243 and 276), but there is **no test** for the similar-diagnostic-group cap path:

```rust
if group.len() > MAX_SIMILAR_DIAGNOSTICS_PER_GROUP {
    let omitted_diagnostics = &group[MAX_SIMILAR_DIAGNOSTICS_PER_GROUP..];
    bounded.push(recovery_omission_summary(
        omitted_diagnostics.len(),
        SIMILAR_DIAGNOSTIC_CAP_KIND,
        &omitted_kind(omitted_diagnostics),  // <-- untested
    ));
}
```

If reveal_type diagnostics are omitted via the **similar diagnostic group cap** (not the top-level cap), the `omitted_kind` text will be wrong and there will be no test catching it.

---

### Finding 3 — Hardcoded string in `test_apply_diagnostic_recovery_limits_uses_registry_dedupe_args_only` (LOW severity)

**File:** `crates/sifr_driver/src/tests/diagnostics.rs:72-73`

The test constructs `first` with `message = "5 additional diagnostics omitted by recovery cap..."` (hardcoded string, not constructed via the template). This is a pre-existing issue but becomes more visible now that the message template has changed. The test still passes because it checks `.args.get("omitted_count")`, not the rendered message.

---

### Phase Contract Check

The requirement "reveal_type overflow summaries must say how many reveal_type results were omitted" is **satisfied** by the `omitted_kind` field. The three cases work correctly:
- All reveal_type → `"reveal_type results"`
- Some reveal_type → `"diagnostics (including N reveal_type results)"`
- No reveal_type → `"diagnostics"`

---

### Registry/Docs Sync

All three are consistent:
- `codes.rs:1352-1355`: template and args correct
- `docs/errors/SIFR-INTERNAL-0002.md`: updated correctly  
- `internal_docs/diagnostic_codes.md`: updated correctly

---

## Verdict

**No, not satisfied.** Blocking findings:

1. **HIGH:** Dedupe logic inconsistency — different `omitted_kind` values prevent deduplication of logically equivalent omission summaries. Fix: either exclude `omitted_kind` from `dedupe_args` (if summaries with same `cap_kind` should always collapse regardless of omitted kind) or add a test that explicitly covers cross-kind deduplication.

2. **MEDIUM:** Missing test coverage for the similar-diagnostic-group cap path with reveal_type diagnostics in the omitted set.

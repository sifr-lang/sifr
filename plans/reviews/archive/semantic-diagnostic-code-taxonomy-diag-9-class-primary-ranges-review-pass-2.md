# Review: milestone_diag_9 slice 9 — class diagnostic primary ranges (pass 2)

## Verdict: SATISFIED

No fallback/compatibility shortcuts. Span accuracy is correct. Regression coverage is confirmed via 4/4 unit tests + e2e fail suite. Maintainability is clean.

---

## Diff correctness

### SIFR-CLASS-0002 — required field after default (classes.rs:586-599)

The defensive fallback from pass 1 is removed. The fix is structural:

- `own_fields: Vec<(String, TextRange)>` tracks only **own** field declarations with their ranges.
- `own_field_default_indices: HashSet<usize>` tracks own-field indices that have defaults.
- The validation loop iterates `own_fields` and uses `*range` (the field **name** range, not declaration range) as the diagnostic span.

Because `own_field_idx` (0-based into `own_fields`) is used consistently — and `own_fields` is populated before the loop — a required field that appears after a defaulted field is caught by construction, with no fallback needed.

**Span accuracy**: `name.range()` on the `HirPattern::Name` binding gives the exact identifier span. For `name: str` in `auto_init_required_after_default.sifr`, this is `col=5` (4 spaces of indentation + first char of `name`). ✓

### SIFR-CLASS-0001 — missing initializer (classes.rs:614-621)

Emits at `class_def.name.range()` — the class name identifier. Correct (class name anchor). ✓

### SIFR-CLASS-0003 — enum duplicate value (classes.rs:261-271)

Emits at `variant.name_range` — the exact identifier span of the duplicate variant. `EnumVariantInfo` is now a struct carrying `name_range` from `name.range()` at collect time.

**Span accuracy**: `enum_duplicate_value.sifr` expects `col=5`. Line 7 `    SUCCESS = 200` has 4 leading spaces; col=5 lands on `S`. The duplicate is `SUCCESS` whose `name_range` starts at `S`. ✓

### SIFR-CLASS-0004 — missing field (expressions.rs)

Unchanged in this diff; primary range was added in a prior pass. ✓

---

## Regression coverage

```
cargo test -p sifr_hir class_code -- --nocapture --test-threads=1
→ 4 passed (test_auto_init_inheritance_missing_super_has_class_code,
            test_auto_init_required_after_default_has_class_code,
            test_enum_duplicate_value_has_class_code,
            test_missing_field_has_class_code)

cargo test -p sifr --test e2e test_e2e_fail --
  auto_init_inheritance_missing_super auto_init_required_after_default
  enum_duplicate_value missing_field --nocapture
→ 248 fail tests completed, 1 passed
```

All four fixtures are exercising the updated code paths. ✓

---

## Formatting / lint

```
cargo fmt --check  → (no output, clean)
cargo clippy -p sifr_hir --no-deps -- -D warnings  → Finished ... clean
```

---

## Maintainability

- `EnumVariantInfo` replaces the raw tuple `(String, Option<i64>)` — self-documenting, range-extensible.
- `collect_enum_variants` return type updated; call site at `classes.rs:782` updated to map `.name` and `.value` at the call site (no intermediate tuple).
- `own_fields`/`own_field_default_indices` are focused accumulator variables with clear semantics.
- No defensive fallbacks, no TODO/FIXME comments for missing spans.

---

## No-findings list (confirmed absent)

- No `unwrap()` or `expect()` on user-controllable data paths.
- No fallback-to-class-name or fallback-to-stmt-range for `SIFR-CLASS-0002` — removed from pass 1.
- No duplication of range-collection logic; each diagnostic site uses a direct range from a well-typed anchor.

# Review: `milestone_diag_9` slice 9 — class diagnostic primary ranges

**Reviewer**: agent (compiler-code reviewer)
**Round**: pass-1
**Files**: `crates/sifr_hir/src/lower/classes.rs`, `diagnostics.rs`, `expressions.rs`, `expressions_tests.rs`, e2e fixtures

---

## Findings

### Correctness

**`classes.rs` — `EnumVariantInfo` rename of tuple** (lines 179–183, 186–244)
`collect_enum_variants` now returns `Vec<EnumVariantInfo>` with `{name, value, name_range}` instead of `Vec<(String, Option<i64>)>`. This is correct — name ranges are needed at the call site in `collect_class_type` for `SIFR-CLASS-0003` (duplicate variant). No tuple discard.

**`classes.rs:266` — `SIFR-CLASS-0003` primary range is `variant.name_range`** ✓
`ctx.error_with_code_at(..., variant.name_range)` points at the second variant name (`SUCCESS` in `OK = 200 / SUCCESS = 200`). This matches `col=5` in the fixture (the `SUCCESS` line starts at column 5).

**`classes.rs:594–603` — `SIFR-CLASS-0002` primary range from `field_name_ranges`** ✓
`field_name_ranges` is populated at line 432 when processing field annotations. The range is looked up via `.get(fname)`. Fallback to `class_def.name.range()` if the field name is absent (defensively sound, unreachable in the actual triggering path).

**`classes.rs:623` — `SIFR-CLASS-0001` primary range is `class_def.name.range()`** ✓
Points at `Dog` (column 7, zero-based from `class ` anchor). Matches `col=7` in fixture.

**`expressions.rs:2335` — `SIFR-CLASS-0004` primary range is `attr.attr.range()`** ✓
`attr.attr` is the identifier node (`z` in `p.z`). Column 13 (1-based) = position of `z` in `print(p.z)`. Matches `col=13` in fixture.

### Span accuracy

All four primary ranges use the correct source-span anchors:
- `SIFR-CLASS-0001`: class name identifier
- `SIFR-CLASS-0002`: field name identifier (from pre-collected map)
- `SIFR-CLASS-0003`: duplicate enum variant name identifier
- `SIFR-CLASS-0004`: attribute name identifier

No off-by-one errors detected. All use `error_with_code_at` (span-aware) rather than `error_with_code` (no span).

### Regression coverage

| Diagnostic | HIR unit test | E2E fixture |
|---|---|---|
| SIFR-CLASS-0001 | `test_auto_init_inheritance_missing_super_has_class_code` | `auto_init_inheritance_missing_super.sifr` |
| SIFR-CLASS-0002 | `test_auto_init_required_after_default_has_class_code` | `auto_init_required_after_default.sifr` |
| SIFR-CLASS-0003 | `test_enum_duplicate_value_has_class_code` | `enum_duplicate_value.sifr` |
| SIFR-CLASS-0004 | `test_missing_field_has_class_code` | `missing_field.sifr` |

Each unit test now checks `e.primary_range == Some(range_for_after(...))`. Each e2e fixture carries `col=N` anchors. Comprehensive column coverage for the class diagnostic family.

### Maintainability

- `EnumVariantInfo` is a focused, minimal struct (3 fields) replacing a raw tuple. Intent is clearer at all call sites.
- `field_name_ranges: HashMap<String, TextRange>` is a standard pattern for deferred span lookup — clean and idiomatic.
- The `unwrap_or_else(|| class_def.name.range())` fallback is defensively sound while unreachable in the actual triggering path.
- All imports correct: `ruff_text_size::Ranged` in `classes.rs`, `TextRange` in `diagnostics.rs`.

### No fallback/compatibility shortcuts

No `.unwrap()` in user-facing paths. No TODO/FIXME. No suppressed warnings. No behavior changes beyond span attachment.

---

## Verdict

**Satisfied.** The slice correctly attaches primary ranges to all four class diagnostics, carries enum variant name ranges through `collect_enum_variants` without losing information, and provides full HIR + e2e column coverage. No actionable findings.

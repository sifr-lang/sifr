## Review: diag-9 unknown type primary ranges (pass 1)

**Files reviewed:**

- `crates/sifr_hir/src/lower/typing_and_functions.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr/tests/e2e/fail/unknown_type_annotation.sifr`
- `crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr`

### Changes summary

**`typing_and_functions.rs`** - `unknown_type` signature updated from `fn(ctx, name)` to
`fn(ctx, name, range)`. Body now calls `ctx.error_with_code_at` forwarding the range
as primary range. Two call sites updated:

- Simple name: `name.range()` attaches the bare identifier range.
- Generic subscript: `sub.value.range()` attaches just the base-name range of
  `UnknownType[int]`, correctly isolating the base from the full subscript.

**`expressions_tests.rs`** - Two new tests:

- `test_unknown_type_annotation_has_primary_range` asserts `primary_range` points to
  `MissingType` in `value: MissingType`.
- `test_unknown_generic_type_annotation_has_primary_range` asserts `primary_range`
  points to `UnknownType` in `x: UnknownType[int]`.

**E2E fixtures** - Column-specific assertions added:

- `unknown_type_annotation.sifr`: `col=20` for `MissingType`
- `generic_class_missing_type_arg.sifr`: `col=8` for `UnknownType`

### Findings

1. No fallback/raw diagnostics. Ranges passed explicitly at call sites.
2. Correct range choice for subscript base. `sub.value.range()` isolates the unresolved base type.
3. Signature change is callers-contained. All callers updated in the same diff.
4. Test coverage is adequate. Unit tests plus column-specific e2e assertions validate behavior end-to-end.
5. Formatting/linting pass per the validation record provided.

## Verdict

**Satisfied.** Implementation is minimal, correct, and focused. No issues identified.
Another pass is not required.

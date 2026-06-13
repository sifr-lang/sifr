# Review: semantic-diagnostic-code-taxonomy-diag-9-name-import-primary-ranges (slice 4, pass 1 retry)

## Summary

The slice migrates `SIFR-NAME-0001`, `SIFR-NAME-0002`, `SIFR-NAME-0004`, `SIFR-IMPORT-0001`, and `SIFR-IMPORT-0002` from `error_with_code` (no range) to `error_with_code_at` (primary range from AST). The helper functions now take a `TextRange` instead of deriving fallbacks internally.

**Verdict: SATISFIED — no blockers.**

---

## 1. Helper API changes are clean

**`name_diagnostics.rs`** — three functions, all migrated correctly:
- `undefined_variable(ctx, name, range)` → `error_with_code_at` with the name node's range
- `undefined_function(ctx, name, range)` → `error_with_code_at` with `call.func.range()`
- `missing_member(ctx, container, member, range)` → `error_with_code_at` with `imported_name_range(member)`

**`import_diagnostics.rs`** — two functions, all migrated correctly:
- `forbidden_intrinsic(ctx, module, range)` → `error_with_code_at` with `import_range` (the full `from X import Y` statement)
- `unknown_import_target(ctx, module, range)` → `error_with_code_at` with `import_range`

All five helpers now require a source range — no optional fallback parameter was added. API is clean and non-regressive.

---

## 2. Call sites pass correct AST-origin ranges

### Undefined variable path (assignment/augassign/tuple-unpack)

**`aug_assign_lowering.rs:264`** — extracts `name_range` from `aug.target` via `Ranged::range()` on the `Expr::Name` node:
```rust
let (name, name_range): (String, TextRange) = if let Expr::Name(n) = aug.target.as_ref() {
    (n.id.to_string(), n.range())
```
This points at the identifier being assigned-to, which is the correct semantic anchor.

**`statements.rs:1488`** — same pattern for simple assignment:
```rust
let (name, name_range) = if let Expr::Name(n) = &assign.targets[0] {
    (n.id.to_string(), n.range())
```
Correct.

**`expressions.rs:248`** — `lower_name` for bare name reference:
```rust
name_diagnostics::undefined_variable(ctx, &var_name, name.range());
```
Correct — points at the unresolved name identifier.

**`tuple_unpack.rs:17`** — `TupleAssignTarget::Name` now stores a `range: TextRange` alongside the name, propagated from `n.range()` at lowering time. The undefined-variable path at line 106 passes it correctly:
```rust
name_diagnostics::undefined_variable(ctx, &name, range);
```

### Undefined function path

**`expressions.rs:1735`** — `lower_call` when function not found:
```rust
name_diagnostics::undefined_function(ctx, &func_name, call.func.range());
```
Correct — points at the callee expression (e.g., `foo` in `foo()`).

### Missing member path

**`mod.rs:796, 961, 1104`** — all use `imported_name_range(name)`:
```rust
name_diagnostics::missing_member(
    &mut ctx,
    &module_name,
    name,
    imported_name_range(name),
);
```

The closure `imported_name_range` (defined at line 758):
```rust
let import_range = import_from.range();
let imported_name_range = |original: &str| -> TextRange {
    import_from
        .names
        .iter()
        .find(|alias| alias.name.as_str() == original)
        .map_or(import_range, Ranged::range)
};
```
finds the specific `Alias` node in `import_from.names` that matches the imported name, and returns its range. If not found (shouldn't happen for a successfully resolved import), falls back to the full statement range.

### Forbidden/unknown import paths

**`mod.rs:779`** — `forbidden_intrinsic` at `import_range`:
```rust
import_diagnostics::forbidden_intrinsic(&mut ctx, &module_name, import_range);
```

**`mod.rs:804, 970, 979`** — `unknown_import_target` at `import_range`:
```rust
import_diagnostics::unknown_import_target(&mut ctx, &module_name, import_range);
```

---

## 3. Range choice is semantically appropriate

| Diagnostic | Code | Range choice | Correct? |
|---|---|---|---|
| Undefined variable | SIFR-NAME-0001 | Points at the name identifier | Yes |
| Undefined function | SIFR-NAME-0002 | Points at the callee expression | Yes |
| Missing member | SIFR-NAME-0004 | Points at the imported member name | Yes |
| Forbidden intrinsic | SIFR-IMPORT-0001 | Points at the full import statement | Yes |
| Unknown import target | SIFR-IMPORT-0002 | Points at the full import statement | Yes |

The e2e fixture column positions confirm this: `undefined_function` col=5 (`foo` starts at column 5), `undefined_var` col=11 (`x` at column 11 in `print(x)`), `stdlib_missing_function` col=23 (`nonexistent_func` starts at column 23).

---

## 4. Borrow/lifetime / `imported_name_range` closure safety

**No issues detected.**

The closure captures `import_from` by reference (`&`) and `import_range` by value (`Copy`). The closure is used only within `lower_module_impl` and is dropped before the function returns. There is no risk of the closure outliving the reference.

`TextRange` is `Copy`, so `import_range` is copied into the closure's environment with no lifetime concerns.

`Ranged::range` is called via UFCS which resolves correctly for `Alias` (which implements `Ranged` in ruff).

---

## 5. Test coverage

### Unit tests (`name_import_diagnostics_tests.rs`)

Five tests updated with `range_for` helper that computes expected ranges from source strings:
- `undefined_variable_has_name_code` — checks `primary_range == Some(range_for(source, "x"))`
- `undefined_function_has_name_code` — checks `primary_range == Some(range_for(source, "foo"))`
- `missing_stdlib_member_has_name_code` — checks `primary_range == Some(range_for(source, "nonexistent_func"))`
- `forbidden_intrinsic_import_has_import_code` — checks `primary_range == Some(range_for(source, "from _sifr.io import read_text"))`
- `unknown_module_import_has_import_code` — checks `primary_range == Some(range_for(source, "from missing_module import value"))`

All five tests assert `primary_range` against the exact needle substring in source. Local validation confirmed `--nocapture` passing.

### E2E fixtures

Seven fail fixtures updated with `col=` anchor on the `expect-error` directive:

| File | col | Points at |
|---|---|---|
| `import_intrinsic.sifr` | col=1 | `from` — correct for IMPORT-0001 full statement |
| `import_nonexistent_local.sifr` | col=1 | `from` — correct for IMPORT-0002 |
| `stdlib_intrinsic_direct_import.sifr` | col=1 | `from` — correct for IMPORT-0001 |
| `stdlib_invalid_module.sifr` | col=1 | `from` — correct for IMPORT-0002 |
| `stdlib_missing_function.sifr` | col=23 | `nonexistent_func` — correct for NAME-0004 |
| `undefined_function.sifr` | col=5 | `foo` — correct for NAME-0002 |
| `undefined_var.sifr` | col=11 | `x` — correct for NAME-0001 |

No test overfits to unrelated diagnostics. Column positions are minimal and precise.

---

## 6. `error_with_code` still present in diagnostics files (pre-existing)

No `error_with_code` calls remain in `name_diagnostics.rs` or `import_diagnostics.rs` after this change. The migration is complete for the targeted codes.

---

## Blocker assessment

**None.**

The implementation is correct end-to-end:
1. Helper APIs now require (not optional) source ranges via `error_with_code_at`.
2. All call sites pass real AST-derived ranges, not synthetic fallbacks.
3. Range anchoring is semantically correct for each diagnostic category.
4. No borrow/lifetime issues in the `imported_name_range` closure.
5. Unit tests and e2e fixtures both validate the new ranges precisely.
6. All local validation passed (`cargo clippy`, unit tests, e2e pass harness).

The slice is ready for PR.

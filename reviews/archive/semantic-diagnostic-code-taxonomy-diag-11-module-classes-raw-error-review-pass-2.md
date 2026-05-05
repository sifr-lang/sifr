# Review: `codex/diag-11-raw-hir-module-classes` — Pass 2 (Post-Adjustment)

**Review scope**: SIFR-IMPORT-0003/0004, SIFR-CLASS-0005/0006 addition; `mod.rs` and `classes.rs` raw-error migration; test coverage; generated docs; transport guardrail updates; CLI assertion updates.

## Verdict: APPROVED / SATISFIED

---

## 1. Diagnostic Code Taxonomy

### IMPORT family codes

| Code | Name | Discriminator | Status |
|------|------|---------------|--------|
| `SIFR-IMPORT-0001` | `IMPORT_FORBIDDEN_INTRINSIC` | `_sifr.*` module import | Pre-existing |
| `SIFR-IMPORT-0002` | `IMPORT_UNKNOWN_SOURCE_MODULE` | Unknown module | Pre-existing |
| `SIFR-IMPORT-0003` | `IMPORT_UNSUPPORTED_FORM` | Unsupported import syntax | NEW |
| `SIFR-IMPORT-0004` | `IMPORT_PRIVATE_MEMBER` | Private `_`-prefixed name import from local module | NEW |

**Correctness**: Scoping confirmed correct. `IMPORT_UNSUPPORTED_FORM` covers bare `import X`, bare relative (`from . import X`), and multi-level relative (`from ..X import Y`). `IMPORT_PRIVATE_MEMBER` is scoped to the local module private-import path only (starts with `_` prefix check before module lookup). Existing stdlib `.sifr` files (e.g., `sifr.heapq`) that import `_sifr.*` private helpers are not affected.

### CLASS family codes

| Code | Name | Discriminator | Status |
|------|------|---------------|--------|
| `SIFR-CLASS-0003` | `CLASS_DUPLICATE_OR_INVALID_VALUE` | Duplicate enum/class value | Pre-existing |
| `SIFR-CLASS-0004` | `CLASS_MISSING_MEMBER` | Field access on non-member | Pre-existing |
| `SIFR-CLASS-0005` | `CLASS_INVALID_BASE` | Unknown or non-class parent | NEW |
| `SIFR-CLASS-0006` | `CLASS_UNSUPPORTED_DECLARATION` | Unsupported class-body statement or unsupported field default | NEW |

**Correctness**: Clean separation — `CLASS_INVALID_BASE` for inheritance resolution failures, `CLASS_UNSUPPORTED_DECLARATION` for structural class-body problems. No overlap with existing `CLASS_REQUIRED_FIELD_AFTER_DEFAULT` (order constraint) or other class codes.

---

## 2. Raw `ctx.error(String)` Elimination

Confirmed by exhaustive grep:

- `crates/sifr_hir/src/lower/classes.rs`: **0 remaining** `ctx.error(String)` sites (was handling: non-class parent, unknown parent, unsupported field default, unsupported class-body statement)
- `crates/sifr_hir/src/lower/mod.rs`: **0 remaining** `ctx.error(String)` sites (was handling: bare import, bare relative import, multi-level relative import, private name import)

Both files now route all error paths through `ctx.error_with_code_at(...)` via named helpers.

---

## 3. Source Ranges

### `parent_class_range` helper (`classes.rs:92–101`)

```rust
fn parent_class_range(class_def: &StmtClassDef, parent_name: &str) -> TextRange {
    class_def.bases().iter().find_map(|base| match base {
        Expr::Name(name) if name.id.as_str() == parent_name => Some(name.range()),
        _ => None,
    }).unwrap_or_else(|| class_def.name.range())
}
```

Correct: When base name is found in the `bases` list, uses that name's range. Falls back to class name range when base not found (e.g., unknown parent — the identifier `MissingParent` is the primary span).

### `imported_name_range` closure (`mod.rs:726–732`)

Correct: Finds the matching `alias.name` in `import_from.names` and returns its range. Falls back to full `import_range` if not found (should never occur for successfully parsed imports).

### E2E fixture column markers

| File | Marker | Note |
|------|--------|------|
| `unsupported_import_statement.sifr` | `col=8` | Points at `local_math` identifier in `import local_math` |
| `class_unknown_parent.sifr` | `col=13` | Points at `MissingParent` in `class Child(MissingParent):` |
| `class_unsupported_field_default.sifr` | `col=18` | Points at `1 + 2` default expression in `value: int = 1 + 2` |

---

## 4. Message Format / Schema

### IMPORT-0003 (`unsupported import form: {form}`)

| Scenario | Message |
|----------|---------|
| `import local_math` | `unsupported import form: import local_math; use 'from local_math import <name>'` |
| Bare relative `from . import X` | `unsupported import form: bare relative import; use 'from <module> import ...'` |
| Multi-level relative `from ..X import Y` | `unsupported import form: relative import level 2 for module '<none>'` |

### IMPORT-0004 (`cannot import private name '{name}' from module '{module}'`)

Example: `from local_math import _secret` → `cannot import private name '_secret' from module 'local_math'`

### CLASS-0005 (`invalid base class for '{class_name}': {reason}`)

Two reason variants exercised by tests:
- `parent type 'X' is not a class`
- `parent class 'X' not defined`

### CLASS-0006 (`unsupported class declaration in '{class_name}': {detail}`)

Two detail variants exercised by tests:
- `unsupported default expression for field 'value'`
- `unsupported statement in class body`

---

## 5. CLI Assertion Updates (`crates/sifr/src/main.rs`)

Updated three test assertions to match the new structured message text:

| Location | Old fragment | New fragment |
|----------|-------------|--------------|
| Line ~1094 | `unsupported import statement 'import helper'` | `unsupported import form: import helper` |
| Line ~1121 | `unsupported bare relative import` | `unsupported import form: bare relative import` |
| Line ~1148 | `unsupported relative import level 2` | `unsupported import form: relative import level 2` |

All three use `String::contains(...)` so they are stable against additional context in the message.

---

## 6. Test Coverage

### HIR unit tests

| Test | Code | File |
|------|------|------|
| `unsupported_import_statement_has_import_code` | IMPORT-0003 | `name_import_diagnostics_tests.rs:94` |
| `private_import_member_has_import_code` | IMPORT-0004 | `name_import_diagnostics_tests.rs:107` |
| `test_unknown_parent_class_has_class_code` | CLASS-0005 | `expressions_tests.rs:2939` |
| `test_unsupported_class_field_default_has_class_code` | CLASS-0006 | `expressions_tests.rs:2953` |

All four verify: `message`, `code`, `primary_range`.

### E2E fail fixtures

| Fixture | Code | Note |
|---------|------|------|
| `unsupported_import_statement.sifr` | IMPORT-0003 | Bare `import` statement |
| `class_unknown_parent.sifr` | CLASS-0005 | Unknown parent `MissingParent` |
| `class_unsupported_field_default.sifr` | CLASS-0006 | Unsupported default `1 + 2` |

**IMPORT-0004**: No e2e fail fixture (confirmed intentional per adjustment scope — existing pass fixtures rely on private stdlib helpers). Covered by HIR unit test in `name_import_diagnostics_tests.rs` using `lower_module_with_externals`.

---

## 7. Transport Guardrail / Docs Sync

- `check_diagnostic_transport_cleanup.py`: Both `classes.rs` and `mod.rs` added to `RAW_HIR_ERROR_FREE_FILES`.
- `check_diagnostic_docs_sync.py`: Clean (no output).
- `check_diagnostic_schema_sync.py`: Clean (no output).
- `check_diagnostic_code_coverage.py`: Clean (no output).
- Generated docs for all four new codes (`SIFR-IMPORT-0003.md`, `SIFR-IMPORT-0004.md`, `SIFR-CLASS-0005.md`, `SIFR-CLASS-0006.md`) verified present and correctly formatted.

---

## 8. Compatibility with Existing Pass Fixtures

The private import exemption for compiled stdlib modules is correctly implemented as a check that only applies to the local module private import path (`name.starts_with('_')` before any module lookup), not to stdlib `.sifr` files that legitimately use `_sifr.*` intrinsics. Existing pass fixtures in `crates/sifr/tests/e2e/pass/` that import `sifr.heapq` private helpers (e.g., `_siftype`, `_heap_push`) remain unaffected.

---

## 9. Validation Results Summary

After the post-adjustment validation pass, all authoritatively required checks passed:
- `cargo fmt` and `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_diagnostic_docs_sync.py`
- `python3 scripts/check_diagnostic_schema_sync.py`
- `python3 scripts/check_diagnostic_code_coverage.py`
- `python3 scripts/check_diagnostic_transport_cleanup.py`
- `cargo test -p sifr_hir import_code -- --nocapture`
- `cargo test -p sifr_hir class_code -- --nocapture`
- `cargo test -p sifr --test e2e test_e2e_fail -- unsupported_import_statement class_unknown_parent class_unsupported_field_default --nocapture`
- `cargo test -p sifr compile_entrypoint_error_consistency -- --nocapture`
- `cargo clippy -p sifr_hir -p sifr_diagnostics -- -D warnings`

---

**No required fixes identified. This diff is ready for commit.**

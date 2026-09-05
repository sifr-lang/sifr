# Review Pass 3 — `codex/diag-11-raw-hir-module-classes`

**Scope**: Semantic diagnostic taxonomy: module/classes raw HIR diagnostic cleanup (DIAG-11).

---

## Diff Summary (11 files, +236 / -34)

| File | Change |
|---|---|
| `crates/sifr_diagnostics/src/codes.rs` | +52 lines: 4 new diagnostic entries + registry + `ACTIVE_DIAGNOSTIC_CODES` |
| `crates/sifr_hir/src/lower/classes.rs` | +71 lines: `CLASS_INVALID_BASE`, `CLASS_UNSUPPORTED_DECLARATION` helpers + use in `collect_class_type` |
| `crates/sifr_hir/src/lower/mod.rs` | +37 lines: `IMPORT_UNSUPPORTED_FORM`/`IMPORT_PRIVATE_MEMBER` via `import_diagnostics` helpers |
| `crates/sifr_hir/src/lower/import_diagnostics.rs` | +16 lines: `unsupported_form` and `private_member` helpers |
| `crates/sifr_hir/src/lower/expressions_tests.rs` | +27 lines: unit tests for CLASS-0005 and CLASS-0006 |
| `crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs` | +33 lines: unit tests for IMPORT-0003 and IMPORT-0004 |
| `crates/sifr/src/main.rs` | Updated 3 test assertions to new message wording |
| `crates/sifr_driver/src/tests/single_file_frontend.rs` | Updated 3 test assertions to new message wording |
| `scripts/check_diagnostic_transport_cleanup.py` | Added `classes.rs` and `mod.rs` to `RAW_HIR_ERROR_FREE_FILES` |
| `docs/errors/diagnostic-codes.md` | +4 new code entries |
| `internal_docs/diagnostic_codes.md` | +4 new code entries |

Plus 3 new e2e fail fixtures (all carry correct `expect-error` directives):
- `crates/sifr/tests/e2e/fail/unsupported_import_statement.sifr` → SIFR-IMPORT-0003
- `crates/sifr/tests/e2e/fail/class_unknown_parent.sifr` → SIFR-CLASS-0005
- `crates/sifr/tests/e2e/fail/class_unsupported_field_default.sifr` → SIFR-CLASS-0006

---

## Scope Checklist

### New structured diagnostics
- [x] **SIFR-IMPORT-0003** (`IMPORT_UNSUPPORTED_FORM`) — `"unsupported import form: {form}"` — registry entry, `codes.rs` constant, helper function in `import_diagnostics.rs`, call sites in `mod.rs` (relative import level, bare relative import, bare `import X` statement).
- [x] **SIFR-IMPORT-0004** (`IMPORT_PRIVATE_MEMBER`) — `"cannot import private name '{name}' from module '{module}'"` — registry entry, `codes.rs` constant, helper function in `import_diagnostics.rs`, call site in `mod.rs`.
- [x] **SIFR-CLASS-0005** (`CLASS_INVALID_BASE`) — `"invalid base class for '{class_name}': {reason}"` — registry entry, `codes.rs` constant, helper in `classes.rs`, call sites in `collect_class_type` (not-a-class case + undefined-parent case).
- [x] **SIFR-CLASS-0006** (`CLASS_UNSUPPORTED_DECLARATION`) — `"unsupported class declaration in '{class_name}': {detail}"` — registry entry, `codes.rs` constant, helper in `classes.rs`, call sites in `collect_class_type` (unsupported default expr case + unsupported statement in class body case).

### Raw `ctx.error` eliminated
- [x] `crates/sifr_hir/src/lower/mod.rs` — zero raw `ctx.error` calls remain in import-lowering path; replaced by `import_diagnostics::unsupported_form` and `import_diagnostics::private_member`.
- [x] `crates/sifr_hir/src/lower/classes.rs` — zero raw `ctx.error` calls remain in class-lowering path; replaced by `invalid_class_base` and `unsupported_class_declaration`.

### IMPORT_PRIVATE_MEMBER local-module scoping
- [x] `IMPORT_PRIVATE_MEMBER` is gated on `name.starts_with('_')` check in the local-name import path only. Stdlib private helper imports (`from X import _helper`) go through the existing externals-resolution path and do not trigger this diagnostic. No change to stdlib compatibility.

### CLI / sifr_driver test assertions updated
- [x] `crates/sifr/src/main.rs` — 3 test assertions updated: `"unsupported import statement 'import helper'"` → `"unsupported import form: import helper"`; `"unsupported bare relative import"` → `"unsupported import form: bare relative import"`; `"unsupported relative import level 2"` → `"unsupported import form: relative import level 2"`.
- [x] `crates/sifr_driver/src/tests/single_file_frontend.rs` — 3 equivalent test assertions updated to match the new wording.

### Diagnostic transport cleanup script
- [x] `classes.rs` and `mod.rs` added to `RAW_HIR_ERROR_FREE_FILES`, reflecting their now-clean status.

---

## Cross-Check: Code-to-String Mapping Integrity

| Constant | Code string | Message template | Call sites |
|---|---|---|---|
| `IMPORT_UNSUPPORTED_FORM` | `"SIFR-IMPORT-0003"` | `"unsupported import form: {form}"` | 3 in `mod.rs` |
| `IMPORT_PRIVATE_MEMBER` | `"SIFR-IMPORT-0004"` | `"cannot import private name '{name}' from module '{module}'"` | 1 in `mod.rs` |
| `CLASS_INVALID_BASE` | `"SIFR-CLASS-0005"` | `"invalid base class for '{class_name}': {reason}"` | 2 in `classes.rs` |
| `CLASS_UNSUPPORTED_DECLARATION` | `"SIFR-CLASS-0006"` | `"unsupported class declaration in '{class_name}': {detail}"` | 2 in `classes.rs` |

All four constants appear in `ACTIVE_DIAGNOSTIC_CODES`. All four have registry entries with correct `arg!` parameter lists and `message_keys`.

---

## One Minor Observation (non-blocking)

In `crates/sifr/tests/e2e/fail/class_unknown_parent.sifr`, the `# expect-error[col=13]` directive points to column 13 (where `MissingParent` starts). The unit test in `expressions_tests.rs` for the same case uses the range of `MissingParent` via `range_for_after(source, "class Child(", "MissingParent")`. Both are internally consistent; the slight difference in how column is reported vs. how range-based unit test assertions work is a pre-existing e2e harness quirk and not introduced by this diff.

---

## Verdict

**Approved. No required fixes.**

- All four new diagnostics are structurally complete (constant + registry entry + helper + call sites + unit test).
- Raw `ctx.error` fully eliminated from `mod.rs` import path and `classes.rs` class path.
- All test assertions updated to new message wording across `main.rs` and `sifr_driver`.
- `IMPORT_PRIVATE_MEMBER` is correctly scoped to local-name imports and does not affect stdlib private helper resolution.
- Quick validation passed (profile=quick, wall_time=55.68s, report=e1bf653aaa770517).

---

**Review pass**: 3 (final)
**Date**: 2026-05-03
**Reviewer**: agent



Reviewing the staged changes for **Enforce Workload Annotations** (milestone 2 of ad hoc async diagnostics).

## Verification Checklist

### 1. Design Requirements Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| @blocking_io/@cpu_heavy are sync-only | ✅ | `reject_async_function_annotation` in `typing_and_functions.rs` calls `annotation_with_range` and emits SIFR-ASYNC-0006 for async defs |
| Annotation on async def → error | ✅ | New fixtures `blocking_io_on_async_def_rejected.sifr`, `cpu_heavy_on_async_def_rejected.sifr` expect SIFR-ASYNC-0006 |
| Direct call in async → error | ✅ | `reject_async_direct_call` in `workload_annotations.rs` emits SIFR-ASYNC-0003/0004 (previously `warn_async_direct_call` was warning-only) |
| Cheap unannotated sync → allowed | ✅ | `cheap_sync_helper_in_async_allowed.sifr` has no `# expect-error`; `annotation_with_range` only returns `Some` for workload annotations |
| SIFR-TYPE-0903 retired | ✅ | Reserved entry added in registry; active entry removed; doc file deleted |

### 2. Diagnostic Registry Consistency

- `SIFR-ASYNC-0003`, `0004`, `0006` correctly placed in `active_entry!` block
- All three use `{message}` template (consistent single-arg pattern)
- `ASYNC_DIRECT_BLOCKING_IO_CALL`, `ASYNC_DIRECT_CPU_HEAVY_CALL`, `ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF` added to `ACTIVE_DIAGNOSTIC_CODES`
- `SIFR-TYPE-0903` appears in `reserved_entry` with retirement note, not in active entries

### 3. Test Coverage

| Fixture | Expected Behavior | Status |
|---------|------------------|--------|
| `blocking_io_on_async_def_rejected.sifr` | SIFR-ASYNC-0006 | ✅ |
| `cpu_heavy_on_async_def_rejected.sifr` | SIFR-ASYNC-0006 | ✅ |
| `blocking_io_direct_call_in_async_rejected.sifr` | SIFR-ASYNC-0003 | ✅ (renamed from pass fixture) |
| `cpu_heavy_direct_call_in_async_rejected.sifr` | SIFR-ASYNC-0004 | ✅ (renamed from pass fixture) |
| `cheap_sync_helper_in_async_allowed.sifr` | No error | ✅ (pass fixture) |

Unit test `test_type_check_source_surfaces_blocking_io_direct_call_error` validates the new error behavior with correct severity (`Error` not `Warning`).

### 4. Regression Risk

- Old `LoweringWarningDiagnostic::BlockingWorkInAsync` removed, but no other variant references it
- `LowerCtx::warnings` still used for other diagnostics (not removed)
- Module lowering correctly removed the warning branch; no orphaned code

### 5. Documentation Consistency

- `docs/errors/` has 0003, 0004, 0006 docs; SIFR-TYPE-0903.md deleted
- `diagnostic-codes.md` index shows new codes, reserved 0903
- `internal_docs/diagnostic_codes.md` updated with new entries and reserved 0903
- `internal_docs/phases/32_async_ecosystem.md` references updated fixtures

### 6. Minor Observations (non-blocking)

- `SIFR-ASYNC-0005` is not used; consistent with `reserved_family_base` pattern where gaps may exist for future allocation

---

**Milestone approved.**

All design requirements implemented correctly. Warning→Error upgrade is consistent across call-sites. Diagnostic registry, docs, and tests are consistent. No regressions detected.

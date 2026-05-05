

## Code Review: `codex/diag-11-raw-hir-bytes-methods`

Reviewed diff for `bytes_methods.rs`, `expressions.rs`, `expressions_tests.rs`, and `check_diagnostic_transport_cleanup.py`.

### Findings

**No blocking issues. Reviewer is satisfied.**

---

### Diagnostic codes — correct and active

All three codes used are established, active entries already in use across `builtin_calls.rs`, `expressions.rs`, `method_call_args.rs`, and `statements.rs`:

| Code | Used for | Consistent? |
|---|---|---|
| `CALL_WRONG_POSITIONAL_COUNT` | Arity violations (`len()` no args, `count()` exact 1, etc.) | Yes — matches `builtin_calls.rs:54` and `method_call_args.rs:94` |
| `TYPE_MISMATCH` | Encoding arg type check (`str` vs `int`) | Yes — matches established type-error path |
| `STDLIB_UNSUPPORTED_SURFACE` | Unknown method names, non-UTF-8 codecs | Yes — matches `builtin_calls.rs` pattern |

No retired symbols, no fallback paths, no raw `ctx.error(String)` in the migrated file.

---

### Range threading — preserved correctly

The helper functions `arity_range` and `arg_range` at `bytes_methods.rs:20-28` correctly handle the transition:

- When `arg_ranges` is non-empty, they return the last or indexed arg range (precise primary ranges for the new HIR test)
- When `arg_ranges` is empty, they fall back to `method_range`, preserving pre-migration behavior where no per-arg range was threaded

The callers thread through correctly:
- `resolve_str_encode_method_type` signature extended with `arg_ranges` and `method_range`
- `resolve_bytes_method_type` same
- `expressions.rs:3186` and `expressions.rs:3189` pass both through from `resolve_method_type`'s existing `arg_ranges` and `method_range`

No behavioral regression in range reporting.

---

### Test coverage

**HIR unit test** (`expressions_tests.rs:728-752`, `test_bytes_codec_type_errors_have_structured_codes`): Directly validates both `str.encode(1)` and `bytes.decode(1)` produce `TYPE_MISMATCH` with the correct message **and** the correct `primary_range` pointing at the argument token. This is the appropriate unit-level regression shield for the migration.

**e2e fixtures** (`bytes_encode_non_string_codec.sifr`, `bytes_decode_non_string_codec.sifr`): These files carry `# Reference:` comments but no `# expect-error:` markers. The user's validation report confirms they were included in `test_e2e_fail -- bytes_encode_non_string_codec bytes_decode_non_string_codec bytes_append_unsupported` and passed, which suggests they are accepted as regression-validation fixtures (type-erroring without explicit code annotation) under the existing `fail/` contract.

**Script extension** (`check_diagnostic_transport_cleanup.py:21`): `bytes_methods.rs` correctly added to `RAW_HIR_ERROR_FREE_FILES`. No `ctx.error(` invocations remain in the file.

---

### Summary

The migration is clean: structured codes, correct ranges, no fallback paths, no retired symbols, and test coverage that validates the specific new behavior (primary range pointing at the offending argument). All validation already passed locally.

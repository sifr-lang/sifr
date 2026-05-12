

## Review Summary

**Changes are correct and well-scoped.** Three focused diffs implement async generator return value rejection as specified:

1. **`mod.rs`**: Adds `current_function_is_async_generator: bool` to `LowerCtx` with proper init.
2. **`typing_and_functions.rs`**: Computes `is_async_generator` once before body lowering; sets/restores the flag correctly around body lowering in L1155-1160.
3. **`statements.rs`**: New `lower_return` path for async generators with two cases:
   - `return <expr>` (including `return None`): emits targeted diagnostic
   - bare `return`: emits diagnostic and produces `HirStmt::Return { value: None }`

**Scope verification**: Only touches async generator return handling. No changes to non-async paths, no unrelated refactoring.

**Diagnostics**: `DiagnosticCode::TYPE_MISMATCH` → `SIFR-TYPE-0002` is correct. The three error messages are precise and distinguish:
- bare `return` (not yet lowered)
- `return None` (needs state machine)
- `return <non-None>` (rejected per v1 design)

**Fail-closed behavior**: Both `return None` and bare `return` produce diagnostics and continue lowering with a return statement. This prevents silent invalid Rust generation in the current eager helper implementation. Acceptable given design constraints.

**Regression risk**: Minimal — only nested functions could theoretically observe `previous_async_generator` from the enclosing scope, but that's expected behavior for the fail-closed approach.

**Fixture**: `async_generator_return_value_rejected.sifr` tests the `return 2` path with `# expect-error[col=12]: SIFR-TYPE-0002`, correctly targeting the expression position.

REVIEW_STATUS: SATISFIED



## Review: `cancellation_error_type_surface` slice

### Findings

**1. HIR type registration (`typing_and_functions.rs:78–113`)**

- `ScopeFailure`, `TaskCancelled`, `SecondaryError` are registered in `error_types` — they are ordinary `Error` subclasses.
- `CancellationError` is registered as a `Class` type with a `message: str` field but is **not** inserted into `error_types`.
- The `is_valid_error_type` check at line 662 (`ctx.error_types.contains(name)`) will therefore reject `Result[..., CancellationError]`.
- This is the correct implementation of the model contract.

**2. Codegen (`lib.rs`, `stdlib_filter.rs`, `stmt_support_emitter.rs`)**

- `ScopeFailure`, `TaskCancelled`, `SecondaryError` are added to `BUILTIN_ERROR_CLASSES` (emit `impl Error`), `GLOBAL_INFRA_TYPES` (emit struct definition), and `can_construct_error_from_message_for_ir` (emit `StructName::new(message)`).
- `CancellationError` is absent from all three lists — correct for this slice. It will surface only through `TaskResult.Cancelled(...)` or as `SecondaryError` evidence, never through ordinary `Result[E]` path.

**3. `emit` verification**

The generated Rust correctly produces:
- `ScopeFailure`, `TaskCancelled`, `SecondaryError` as proper `struct`s with `impl Error for X {}`
- No `CancellationError` struct generated

**4. Fail fixture (`cancellation_error_not_result_error.sifr`)**

```
# expect-error: SIFR-RESULT-0002
def bad() -> Result[None, CancellationError]:
```

Running `cargo run -q -p sifr -- check` produces:
```
`CancellationError` is not a valid error type in Result — use a class extending Error, e.g. `Result[None, ValueError]`
```
Exit code 1. The diagnostic code `SIFR-RESULT-0002` is present and matches the existing diagnostic that rejects non-`Error` types as `Result` error parameters (same family as `error_str_not_allowed.sifr`).

**5. PR manifest and phase doc**

- `cancellation_error_type_surface` added to `pr_e2e_manifest.json`.
- Phase doc's progress note reads: *"In progress cancellation/error type surface slice: registered `ScopeFailure`, `TaskCancelled`, and `SecondaryError` as ordinary built-in error classes, and registered `CancellationError` as a non-`Error` control-evidence class so it cannot be used as a `Result` error."* — accurate, neither overclaiming nor underclaiming.

### No blockers found.

---

**VERDICT: SATISFIED**

The slice correctly registers the three ordinary built-in error classes with HIR and codegen, and correctly registers `CancellationError` as a control-evidence class outside the `error_types` set, so it cannot be used in `Result` while remaining accessible as `TaskResult.Cancelled` evidence and as a `SecondaryError` variant payload. Local validation passed. Phase doc is accurate. No changes needed.

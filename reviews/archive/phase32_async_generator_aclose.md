

## Review Summary

### Correctness: HIR Type Surface

`agen.aclose()` correctly resolves to `Awaitable[Result[None, GeneratorCloseError]]` in `async_generator_methods.rs:22-25`. This matches the design spec. The codegen maps `Type::None` → Rust `()` (unit), so `Result[None, E]` becomes `Result<(), E>` — which is the identical semantic. The test at line 14 confirms: `assert str(closed) == "Ok(())"` — the `str()` output correctly shows `()` for the unit payload.

### Codegen Path

The standard `HirExpr::MethodCall` → `RustExpr::MethodCall` path in `stmt_support_emitter.rs` handles `agen.aclose()` without special-casing. The existing preamble `AsyncGenerator::aclose` in `preamble.rs:434-456` (return type `Result<(), GeneratorCloseError>`, body sets `closed = true`) is correctly wired. `"aclose"` is already listed in the `QUERY_SPECIAL_METHODS` set in `queries.rs:28`, so no missing stubs.

### Helper-Module Split

`method_diagnostics.rs` cleanly exports the four arity helpers (`method_count_range`, `reject_method_arg_count`, `reject_exact_method_arg_count`, `reject_max_method_arg_count`, `reject_no_method_args`) that were extracted from `expressions.rs`. The move is mechanical and correct. `async_generator_methods.rs` is a focused single-responsibility module for the `AsyncGenerator` method surface.

### Guardrail

`scripts/check_hir_maintainability_guardrails.py` passes.

### Missing Coverage (Not a Bug)

No negative test for `agen.aclose(x)` — the arity validation is implemented (`async_generator_methods.rs:18-21`) but not exercised by a fixture. This is acceptable for scope.

---

REVIEW_STATUS: SATISFIED

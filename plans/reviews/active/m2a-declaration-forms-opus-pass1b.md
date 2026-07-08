CHANGES_REQUESTED

Summary: The M2a math-constant native-boundary work is well-structured — the retained Rust-expression fallback is fully removed, `sifr.math` constants flow through compiled private declarations, and the tests in `stateless_math_codegen_tests.rs` and `name_import_diagnostics_tests.rs` verify the happy path. However, two correctness concerns and two test-coverage gaps warrant follow-up before merge:

- `_sifr.*` externals-first branch omits `externals.classes` while the bootstrap now emits class exports for private declarations (latent, unexercised).
- `is_supported_annotated_module_constant_expr` accepts `HirExpr::Name` shapes that the codegen materializes with lowercase Rust identifiers, contradicting the uppercase `const` naming in `try_lower_simple_module_const_item`.
- Missing symmetric tests for private-declaration class re-export and for the forbidden-user-import path against the newly-exposed scalar constants.

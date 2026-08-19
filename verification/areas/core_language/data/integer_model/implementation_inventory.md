# Integer Model Implementation Inventory

This inventory records the implemented exact-integer and fixed-width integer surfaces.

## Compiler Frontend and Type System

- `Type::Int` is the only source-level arbitrary-precision integer type.
- Integer literals and const evaluation preserve exact values through compiler-owned arbitrary-precision storage.
- Explicit fixed-width types own representation-sensitive arithmetic, fitting, and narrowing rules.
- Type annotations, generic bounds, and runtime type objects use the shared name-resolution path.

Primary owners:

- `crates/sifr_type_system/src/types/`
- `crates/sifr_type_system/src/check.rs`
- `crates/sifr_type_system/src/infer.rs`
- `crates/sifr_lowering/src/lower/typing_and_functions/`
- `crates/sifr_lowering/src/lower/type_bounds.rs`
- `crates/sifr_lowering/src/lower/fixed_width_fitting.rs`
- `crates/sifr_frontend/src/const_evaluator.rs`

## Runtime and Codegen

- `crates/sifr_runtime` owns `SifrInt`, normalized hashing, parsing, formatting, arithmetic helpers, and serialization support.
- Generated code may optimize proven-small integer values, but source semantics remain exact.
- Generated projects request runtime and numeric dependencies from structured emitted-code needs.
- Decimal implementation details may use `num_bigint::BigInt`; that Rust dependency is not a Sifr source type.

Primary owners:

- `crates/sifr_runtime/src/int/`
- `crates/sifr_codegen/src/entrypoints.rs`
- `crates/sifr_codegen/src/ir_imports.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/`
- `crates/sifr_codegen/src/intrinsic_method_emitters/`
- `crates/sifr_driver/src/build/`

Generated Rust must not expose user-triggerable integer panics.

## Diagnostics and Verification

- Fixed-width fitting, exact arithmetic, conversion, bool comparison, and serialization diagnostics use the active `SIFR-INT-*` registry entries.
- Removed compatibility-only diagnostics and fixtures are absent from the registry, docs, and diagnostic baselines.
- `crates/sifr/tests/e2e/pass/exact_int_arbitrary_precision.sifr` covers arithmetic, comparison, recursion, and hashing beyond `i64` range.
- `crates/sifr/tests/e2e/pass/stdlib_heapq_consolidated.sifr` covers generic heap behavior with large exact integers.
- Fixed-width positive and negative fixtures remain under `crates/sifr/tests/e2e/`.

## Serialization and Data Boundaries

- JSON profile rules live in `sifr_runtime::json` and the compiler-owned const-specialization descriptors.
- SQL, Arrow, dataframe, tensor, and binary boundaries use explicit fixed-width types or explicit fallible conversion from `int`.
- `verification/areas/core_language/data/integer_model/serialization_boundary_rules.md` and `integer_dtype_rules.md` define those contracts.

## Validation

```bash
python3 verification/areas/developer_tooling/check_no_pre_v1_compatibility.py
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/exact_int_arbitrary_precision.sifr
```

The compatibility guard must report no removed public integer support.

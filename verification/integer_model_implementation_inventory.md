# Integer Model Implementation Inventory

This inventory supports `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` INT-0. It records the expected implementation blast radius for replacing the bootstrap machine-integer plus public `bigint` model with exact source-level `int` and explicit fixed-width integer families.

## Compiler Frontend and Type System

Primary files and areas:

- `crates/sifr_type_system/src/types.rs`: `Type::Int`, `Type::BigInt`, ownership kind, Rust type mapping, display names, assignability, union ordering.
- `crates/sifr_type_system/src/literal.rs`: integer literal representation and normalization.
- `crates/sifr_type_system/src/infer.rs`: builtin type-name resolution, including `bigint` transition and fixed-width names.
- `crates/sifr_type_system/src/check.rs`: arithmetic, comparisons, bool/integer separation, decimal/float mixing, fixed-width promotion.
- `crates/sifr_type_system/src/union.rs`: deterministic ordering and equality for added integer variants.
- `crates/sifr_lowering/src/hir_nodes.rs`: HIR literal payloads and type references.
- `crates/sifr_lowering/src/lower/type_bounds.rs`: `Addable`, `Hashable`, and future output-typed numeric protocol handling.
- `crates/sifr_lowering/src/lower/expressions.rs`: constructors, casts, arithmetic lowering, range/index-related conversions.
- `crates/sifr_lowering/src/lower/statements.rs`: assignment fitting, implicit narrowing rejection, pattern literal checks.
- `crates/sifr_lowering/src/lower/decimal_methods.rs`: decimal/bigdecimal conversion contracts currently mentioning `bigint`.

Parser boundary work should avoid broad Ruff submodule churn unless necessary. The intended first step is a parser-driver or AST-to-HIR shim that preserves integer literal lexemes when parser-side numeric storage would be lossy.

## Runtime and Codegen

Primary files and areas:

- New `crates/sifr_runtime`: target home for `SifrInt`, normalized integer hashing, parsing/formatting, arithmetic budget helpers, and JSON integer profile helpers.
- Workspace `Cargo.toml`: add the runtime crate as a member when INT-1 lands.
- `crates/sifr_codegen/src/lib.rs`: runtime dependency tracking, generated Cargo dependency emission, existing `RuntimeNeed::BigInt`.
- `crates/sifr_codegen/src/entrypoints.rs`: generated dependency/preamble behavior.
- `crates/sifr_codegen/src/ir_imports.rs`: existing `needs_bigint` detection to replace with runtime integer needs.
- `crates/sifr_codegen/src/helpers.rs`: `module_uses_bigint`, ownership helpers, clone behavior.
- `crates/sifr_codegen/src/type_emitters.rs`: borrow/value lowering for non-`Copy` exact `int`.
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs`: `int`, `bigint`, `bool`, decimal conversion, fixed-width constructors.
- `crates/sifr_codegen/src/stmt_support_emitter.rs`: arithmetic lowering, decimal lowering, exponentiation/shift helpers.
- `crates/sifr_codegen/src/error_refs.rs`: error type references for generated `Result` paths.
- `crates/sifr_codegen/src/lib_codegen_tests.rs`: codegen snapshots and unit tests covering `BigInt` today.
- `crates/sifr_driver/src/build/*`: generated project materialization and Cargo manifest dependencies.
- `crates/sifr/tests/e2e.rs`: e2e dependency inference currently scans for `num_bigint::BigInt`.

Generated Rust must not expose user-triggerable integer panics. Existing generated emitted demo files with `as usize` and `as i64` casts are audit targets when the implementation reaches indexing/range milestones.

## Diagnostics

Primary files and areas:

- `crates/sifr_diagnostics/src/codes.rs`: add `SIFR-INT-0001..0011`; retire or migrate `TYPE_INT_BIGINT_MIXED`.
- `crates/sifr_diagnostics/src/lib.rs`: expose new code families and metadata.
- `internal_docs/diagnostic_emission_inventory.md`: currently lists `SIFR-TYPE-0006` for int/bigint mixing and bytes diagnostics that may need `uint8`/bytearray updates.
- `internal_docs/diagnostic_codes.md`: synchronize once the registry is updated.

New error classes registered by the design:

- `ArithmeticLimitError`
- `FloatOverflowError`
- `FloatPrecisionLossError`
- `JsonIntegerRangeError`
- `JsonLimitError`

## Stdlib, Builtins, and Collections

Primary surfaces:

- Builtins: `int`, fixed-width constructors, `bool`, `abs`, `sum`, `min`, `max`, `hash`, `range`, `len`, `enumerate`.
- Collections: dict/set key hashing, cross-family integer key equality, fixed-width key normalization.
- Bytes: `bytes` indexing/iteration to `uint8`; `bytearray` read/write rules; `bytes.from_ints` validation.
- Random/math: `random.randrange`, `secrets.randbelow`, `gcd`, `lcm`, `isqrt`, factorial-like APIs with exact `int` and budgets.
- Pattern matching and enums: fitting literal patterns for fixed-width subjects, explicit enum representation rules.

Relevant current tests include bytes fixtures, bigint fixtures, generic accumulator/counter/heapq bigint fixtures, and parse/conversion safety fixtures under `crates/sifr/tests/e2e`.

## Serialization, Web, and Data Contracts

Primary surfaces:

- JSON profile helpers in `sifr_runtime::json`.
- Future web/API schema generation and TypeScript/OpenAPI mapping.
- Generated `serde::Serialize`/`Deserialize` behavior for structs/classes with `int` fields.
- SQL/model mapping: fixed-width or explicit decimal/string representation for storage.
- Arrow/Parquet/dataframe/tensor contracts: fixed-width dtypes, dtype-preserving arithmetic, explicit overflow policy, explicit widen kernels. INT-6A locks this surface in `verification/areas/core_language/data/integer_dtype_contract.md` and `verification/areas/core_language/checks/integer_dtype_contract.py`.

The dtype contract must land as a verification artifact or blocked fixtures even if array/tensor/dataframe runtime kernels are deferred.

## Known Legacy References to Retire or Quarantine

These references are intentional audit findings, not blockers for the docs split:

- `internal_docs/phases/01_language_foundations.md`: describes `int` overflow panicking in debug and wrapping in release.
- `internal_docs/phases/13_type_system_completion.md`: milestone for public `bigint` as arbitrary-precision alternative.
- `internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md`: decimal conversions mention `bigint`; these should become exact `int` conversions or transition-alias notes.
- `internal_docs/diagnostic_emission_inventory.md`: `SIFR-TYPE-0006` int/bigint mixing entries.
- `verification/validation_lanes/*_e2e_manifest.json`: historical `bigint_arithmetic` lane entries are retired from create-pr/merge manifests by INT-7.
- `crates/sifr/tests/e2e/pass/*bigint*.sifr`, `crates/sifr/tests/e2e/fail/bigint_*.sifr`, and adjacent consolidated fixtures that still exercise the temporary alias are quarantined in `verification/integer_model_bigint_transition_quarantine.md` until the public alias-removal PR deletes or rewrites them.
- `demos/cargo_manifest`, `demos/project_build`, and `demos/generic_stdlib`: demo references to `bigint` or generated `num_bigint::BigInt`.
- `issues/archive/*`: historical references should remain archived unless a current doc links to them as active guidance.

## Validation Commands for INT-0

```bash
rg -n 'Primitives \(Copy\)|`int`.*are `Copy`|BigInt,?\s*$|bigint.*int\(b\)|int\s*=\s*i64|Type::Int([^A-Za-z0-9_]|$)|wraps in release' internal_docs docs issues demos crates verification
git diff --check
```

The `rg` command is expected to find current implementation and tracked migration references until their owning milestones retire them. INT-0 uses it to classify remaining matches, not to require a zero-result tree.

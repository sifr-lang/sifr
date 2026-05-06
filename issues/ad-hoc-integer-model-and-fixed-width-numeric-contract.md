# Ad-Hoc Phase: Integer Model and Fixed-Width Numeric Contract

## Objective

Implement Sifr's final pre-production integer model: Python-simple exact `int` for ordinary code, explicit fixed-width integer families for representation-sensitive work, and strict boundary contracts for serialization, data science, web APIs, and Rust interop.

Canonical design: `internal_docs/integer_model.md`.

This issue is the implementation phase tracker. It intentionally does not restate every semantic rule from the design doc. When this issue and the design doc conflict, update both in the same PR and treat `internal_docs/integer_model.md` as the semantic source of truth.

## Status

- Phase state: ad-hoc, ready for implementation breakdown.
- Compatibility stance: pre-production clean break; no long-term compatibility layer for the historical machine-integer model or user-facing `bigint`.
- Required local validation before each implementation PR: `scripts/run_all_tests.sh --profile quick`.
- Required full closure validation: `scripts/run_all_tests.sh`.

## Design Summary

- `int` is an exact signed arbitrary-precision value-semantic scalar backed by canonical `SifrInt`.
- `x: int = 42` lowers semantically to `SifrInt`, typically `SifrInt::Small(42)` before optimization.
- `bigint` is not a long-term user-facing type; arbitrary precision is folded into `int`.
- Fixed-width types are explicit: `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`.
- `isize` and `usize` are FFI/low-level interop types only.
- There is no bare `uint`.
- Unsuffixed integer literals infer as `int`.
- Fixed-width assignment accepts only compiler-proven fitting constants or explicit fallible constructors.
- Ordinary fixed-width scalar arithmetic promotes to exact `int`.
- Representation-preserving fixed-width behavior is explicit through checked/wrapping/saturating/overflowing APIs.
- `bytes` indexing and iteration yield `uint8`.
- Array/tensor/dataframe arithmetic is dtype-preserving and exposes overflow policy; it does not inherit scalar widening by accident.
- JSON, SQL, binary formats, web models, and TypeScript/OpenAPI generation must never silently lose integer precision.

## Dependencies and Sequencing Constraints

- Result/Option enforcement and no-panic lowering must already be available for fallible division, narrowing, parsing, and serialization errors.
- Ownership-aware lowering must handle `int` as source-level value-semantic even though `SifrInt` is not Rust `Copy`.
- Diagnostics should use existing diagnostic plumbing and stable code registry patterns.
- Runtime integer support lives in a new `crates/sifr_runtime` workspace crate. INT-1 owns creating it and teaching codegen to emit the generated Cargo dependency.
- JSON integer profiles live in `sifr_runtime::json` initially. Future stdlib/web layers can wrap that API, but they must not introduce a second profile implementation.
- Dataframe/tensor/array milestones may not all exist yet. Their integer dtype contracts can land as docs, type stubs, or blocked tests until the relevant runtime surfaces exist.
- Web/API/serialization work may need separate framework/library phases. This phase must at least lock compiler and schema contracts so later web work cannot infer unsafe defaults.

## Non-Goals

- Do not add integer literal suffix syntax.
- Do not keep a public `bigint` type beyond a temporary transition alias if absolutely needed during migration.
- Do not add implicit narrowing in any source construct.
- Do not make fixed-width scalar operators wrap, panic, or return fixed-width values for ordinary arithmetic.
- Do not expose `SifrInt` as C ABI compatible.
- Do not infer storage width from ordinary `int` annotations in web, ORM, dataframe, or tensor APIs.

## Milestone INT-0: Contract Lock and Legacy Audit

Goal: make the integer contract discoverable and remove contradictory design references before code changes begin.

Scope:

- Add or maintain `internal_docs/integer_model.md` as the semantic source of truth.
- Reference it from `internal_docs/architecture.md`.
- Patch architecture anchors that still describe `int` as Rust `Copy`, keep public `BigInt`, or describe `OverflowError` as `bigint` to `int` conversion.
- Add the integer error classes introduced by the design to the architecture error hierarchy.
- Audit docs, issues, demos, and tests for claims that `int` is `i64`, `bigint` is the preferred arbitrary-precision type, or fixed-width arithmetic wraps by default.
- Produce `verification/integer_model_implementation_inventory.md` with affected compiler files and test suites.

Acceptance criteria:

- Architecture points at the internal design doc and this implementation issue.
- No canonical docs still describe `int` as Rust `i64`.
- Architecture no longer treats source-level `int` as Rust `Copy` or public `BigInt` as the arbitrary-precision target.
- `ArithmeticLimitError`, `FloatOverflowError`, `FloatPrecisionLossError`, `JsonIntegerRangeError`, and `JsonLimitError` have documented parents and fields.
- Inventory names the parser/AST, HIR, type checker, codegen, driver/runtime, stdlib, verification, and docs surfaces affected.
- Review history names the most recent Claude review artifact and a human/codex acknowledgement that blocking findings were addressed.

Validation:

```bash
rg -n 'Primitives \(Copy\)|`int`.*are `Copy`|BigInt,?\s*$|bigint.*int\(b\)|int\s*=\s*i64|Type::Int([^A-Za-z0-9_]|$)|wraps in release' internal_docs docs issues demos crates verification
```

- `git diff --check`

Checklist:

- [x] Canonical design doc created: `internal_docs/integer_model.md`.
- [x] Implementation phase issue created: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`.
- [x] Architecture source-of-truth reference updated.
- [x] Architecture legacy `int`/`BigInt`/`Copy` anchors amended.
- [x] Legacy audit completed with tracked findings: `verification/integer_model_implementation_inventory.md`.
- [x] Contract-lock review pass accepted by phase-plan review pass 2.

## Milestone INT-1: Runtime `SifrInt` and Ownership Semantics

Goal: introduce the exact integer runtime representation without changing every integer operator at once.

Scope:

- Create `crates/sifr_runtime` as the shared runtime crate for generated projects.
- Teach codegen/build materialization to emit the generated Cargo dependency on `sifr_runtime` and `num-bigint` through that crate rather than vendoring integer helpers into every generated file.
- Add canonical `SifrInt` runtime type with `Small(i64)` and `Big(Box<num_bigint::BigInt>)` or an equivalent representation approved by the design doc.
- Implement construction from fitting Rust primitives and decimal strings with digit limits.
- Implement clone, equality, ordering, hashing, formatting, and basic conversions needed by generated code.
- Implement normalized integer hashing helpers for exact and fixed-width dict/set keys.
- Ensure source-level `int` remains value-semantic even though runtime `SifrInt` is not `Copy`.
- Expect integer-bearing pass fixtures and codegen snapshots to churn in this milestone; reset intentional snapshot baselines through the repository's normal `cargo insta review` workflow.
- Add generated-code panic-shape tests for runtime integer paths.

Acceptance criteria:

- Generated code can construct, clone/reuse, compare, hash, and format `int` values through `SifrInt`.
- Generated Cargo manifests link the shared runtime crate; generated files do not carry duplicate hand-written `SifrInt` modules.
- Reusing an `int` after calls or expressions is legal at source level.
- Small integer construction and simple reuse do not allocate on the big-integer path.
- Runtime integer helpers return typed errors instead of panicking on user-triggerable failures.

Validation:

- Unit tests for `SifrInt` small/big representation, equality, ordering, hashing, and formatting.
- E2E fixture showing repeated use of an `int` binding after calls.
- Panic-shape sweep for generated integer runtime paths.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-2A: Parser Boundary and Literal Capture

Goal: make parsed source preserve arbitrary-size integer literal text without requiring a broad Ruff submodule representation fork.

Scope:

- Keep the Ruff parser token representation stable unless an upstream-maintainable change is proven necessary.
- Add a parser-driver or AST-to-HIR shim that captures the original integer literal lexeme when the parser-side numeric value would be lossy.
- Normalize decimal, hex, octal, and binary literals into a lossless integer-literal representation for HIR.
- Return a typed parser/frontend diagnostic for malformed or over-budget integer literal text.
- Add `SIFR-INT-0003` for reserved `int128`/`uint128` names before support lands.

Acceptance criteria:

- A parsed `.sifr` file containing `x: int = 10 ** 100` reaches HIR without truncation or parser overflow.
- The constructed-AST path and parsed-source path produce equivalent HIR literal representations.
- Reserved `int128`/`uint128` names produce a specific reserved-width diagnostic.
- The Ruff submodule is not modified unless the PR includes an explicit upstream-maintenance rationale.

Validation:

- Parser/frontend tests for decimal, hex, octal, and binary literals beyond `i64`.
- Negative parser/frontend tests for malformed integer token text and reserved `int128`/`uint128`.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-2B: HIR, Type System, and Const Fitting

Goal: represent exact integer literals, fixed-width families, and const fitting in compiler-owned IR/type layers.

Scope:

- Replace compiler-owned `LiteralInt` internals away from `i64` in HIR and type-system code.
- Add fixed-width type variants for signed and unsigned integer families, plus `isize`/`usize` interop types.
- Remove public-facing `bigint` type paths or convert them to a temporary transition alias with diagnostics.
- Implement const-evaluable fitting for literals, unary signs, basic integer arithmetic, shifts, non-negative exponentiation within the 4096-decimal-digit compile-time budget, parentheses, and immutable module constants.
- Propagate const-evaluable status across imports only through the canonical frontend/query API (`sifr_frontend` target facade, or `sifr_driver::frontend` until that crate split exists) when the module dependency graph is acyclic and within budget.
- Update HIR maintainability guardrails for the added integer type variants.

Acceptance criteria:

- Unsuffixed literals infer as `int`.
- `x: int = 10 ** 100` type-checks.
- `x: uint8 = 255` type-checks; `x: uint8 = 256` and `x: uint8 = -1` are compile errors with range diagnostics.
- `x: uint8 = 10 ** 5000` or an equivalent over-budget const expression fails with `SIFR-INT-0004`.
- No implicit narrowing occurs in assignments, calls, returns, list literals, dict literals, or generic specialization.
- `bigint` is gone from public docs/tests or emits intentional `SIFR-INT-0011` transition diagnostics only.

Validation:

- Type-check tests for large literals and fixed-width fitting failures.
- Cross-module const fitting tests for imported immutable constants.
- Negative tests for compile-time evaluator budget exhaustion.
- Negative tests for implicit narrowing in every source construct listed above.
- Parser/resolver diagnostics for unsupported `int128`/`uint128` if referenced before support lands.
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-3: Scalar Arithmetic and Numeric Mixing

Goal: make scalar arithmetic follow the final exact/widening/fallible rules.

Scope:

- Lower `Type::Int` arithmetic to exact `SifrInt` operations.
- Make fixed-width scalar `+`, `-`, `*`, `//`, `%`, and `**` promote to `int` for ordinary operators.
- Add fallible semantics for division, modulo, exact-to-float conversion, negative exponentiation, and explosive output budgets.
- Register `ArithmeticLimitError`, `FloatOverflowError`, and `FloatPrecisionLossError` in the canonical built-in error registry and architecture docs with parent class, fields, and display rules.
- Add fixed-width checked/wrapping/saturating/overflowing APIs.
- Enforce exact comparisons, exact int/float comparison without lossy casts, bool/integer separation, and decimal/float mixing rules.
- Update `Addable` or introduce an output-typed integer/numeric protocol so `T + T -> T` is not satisfied by fixed-width ordinary arithmetic unless the output is assignable to `T`.
- Emit `SIFR-INT-0005` for unhandled exact integer failure cases, `SIFR-INT-0006` for exact-to-float overflow/precision loss, and `SIFR-INT-0007` for bool/integer comparison mistakes.

Acceptance criteria:

- `int32(2_000_000_000) + int32(2_000_000_000)` returns `int`, not `int32`.
- `int32.checked_add`, `int32.wrapping_add`, `int32.saturating_add`, and `int32.overflowing_add` expose representation-preserving behavior.
- `int / int` is handled as fallible unless divisor and float representability are proven.
- `2 ** -1` does not silently become `0.5`.
- `int(2 ** 53 + 1) == float(2 ** 53 + 1)` is evaluated exactly and does not pass through a lossy integer-to-float cast.
- Generic `T + T -> T` examples fail for fixed-width scalar types unless they use an explicit accumulator/output type.
- `True == 1` is rejected; `int(True) == 1` is explicit and valid if bool conversion exists in the current stdlib surface.

Validation:

- Positive and negative e2e fixtures for exact arithmetic, fixed-width promotion, checked/wrapping/saturating APIs, and bool comparison rejection.
- Unit tests for output type inference across mixed numeric expressions.
- Unit tests for exact int/float equality and ordering, including values above JavaScript and IEEE-754 exact integer ranges.
- Protocol/generic tests covering `Addable` or the replacement output-typed numeric protocol.
- Regression tests for no `.unwrap()`/`.expect()` in user-triggerable arithmetic paths.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-4: Builtins, Indexing, Bytes, Ranges, and Pattern Matching

Goal: update language and stdlib surfaces that expose integer behavior outside arithmetic.

Scope:

- Make `len`, `enumerate`, `range`, indexes, and slice math source-level `int`.
- Keep compiler-owned checked `usize` conversions at Rust indexing/materialization boundaries.
- Make `bytes` indexing and iteration yield `uint8`.
- Make `bytearray` indexing and iteration yield `uint8`; writes require fitting literals or explicit `uint8(...)` narrowing.
- Emit `SIFR-INT-0010` for byte and bytearray construction/mutation values that do not fit `uint8`.
- Implement integer dict/set key hashing through normalized integer hashing so equal mathematical values hash consistently across `int` and fixed-width families where equality is allowed.
- Update `sum`, `abs`, `min`, `max`, `random.randrange`, `secrets.randbelow`, and math integer helpers.
- Add literal pattern fitting for fixed-width subjects.
- Constrain valued enum discriminants to explicit representation rules.
- Ensure `range(10 ** 100)` stays lazy and materialization failures are typed.

Acceptance criteria:

- User code does not need `usize` for ordinary indexing or lengths.
- `bytes_value[0]` and byte iteration expose `uint8`.
- `bytearray_value[0] = 255` is valid, `bytearray_value[0] = 256` is rejected, and runtime-dependent writes require handled `uint8(value)` narrowing.
- `dict[int, V]` lookups using equal fixed-width integer keys behave coherently according to the integer equality/hash contract.
- `sum(list[int32])` returns `int`; dtype-preserving reductions are explicit APIs.
- `abs(int8.MIN)` returns `int` rather than overflowing a fixed-width type.
- `case 256` against `uint8` is a compile-time error.

Validation:

- E2E fixtures for indexing, negative indexing, bytes, bytearray, ranges, stdlib builtins, pattern matching, integer dict/set keys, and enum value constraints.
- Target-width tests for 32-bit/wasm-owned conversion behavior where test infrastructure supports it.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-5: Serialization, Web, and Schema Boundaries

Goal: prevent exact integers from silently losing precision when crossing public boundaries.

Scope:

- Implement or lock JSON integer profiles: `json.exact`, `json.web`, and `json.string_ints`.
- Implement profile machinery in `sifr_runtime::json` and expose wrappers from future stdlib/web layers instead of duplicating profile logic.
- Register `JsonIntegerRangeError` and `JsonLimitError` in the canonical built-in error registry and architecture docs with parent class, fields, and display rules.
- Emit `SIFR-INT-0009` for JSON/web-safe integer serialization policy failures.
- Add parser digit limits and typed errors for untrusted JSON/CSV/env/URL integer tokens.
- Map OpenAPI/JSON Schema integer fields according to static range and selected profile.
- Define TypeScript client mappings for safe numbers, branded decimal integer strings, and future exact-client bigint profiles.
- Define generated `serde::Serialize`/`Deserialize` derive behavior so Sifr structs/classes with `int` fields use an explicit integer profile.
- Enforce SQL/storage range checks and explicit dtype/schema choices in model layers.
- Emit diagnostics with field paths and policy suggestions for serialization failures.

Acceptance criteria:

- `json.web` never emits JS-unsafe integer numbers by default.
- `int64`, `uint64`, and exact `int` response fields default to decimal string encoding or typed serialization errors unless explicitly range-constrained.
- Profile rules apply recursively to nested collections.
- Generated serde support does not bypass `json.web`/`json.exact`/`json.string_ints` profile selection for `SifrInt`.
- SQL and binary-schema mappings require fixed-width, decimal, or string representation choices.

Validation:

- JSON exact/web/string profile tests, including nested collections.
- Serde derive/profile tests for a struct with exact `int`, fixed-width, and nested integer fields.
- OpenAPI/JSON Schema snapshot tests for fixed-width and exact integer fields.
- Boundary negative tests for JS-unsafe values, digit-limit overflow, SQL range overflow, and missing policy diagnostics.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-6A: Dtype Contract Lock

Goal: lock integer dtype semantics so future data-science runtime work cannot choose unsafe defaults.

Scope:

- Add a dtype contract artifact under `verification/validation_contracts/` or an equivalent test-owned location.
- Define fixed-width integer dtype names and scalar-to-dtype conversion rules.
- Define default dtype arithmetic as dtype-preserving and fallible, with explicit wrapping/saturating/overflowing/widen policy APIs.
- Emit `SIFR-INT-0008` for fixed-width array/tensor/dataframe arithmetic that lacks an explicit overflow policy once those surfaces exist.
- Require explicit dtype when constructing compact column/tensor storage from `list[int]`.
- Add type-checker stubs, pending fixtures, or blocked tests proving an implementation PR cannot later introduce silent fixed-width dtype wrapping.
- Define Arrow/Parquet integer schema mapping expectations even if loaders are not implemented yet.

Acceptance criteria:

- The contract artifact states that `array[int32] + array[int32]` cannot silently wrap and cannot accidentally widen to `array[int]`.
- A future PR implementing `array[int32] + array[int32] -> array[int32]` without a fallible or explicit overflow policy fails an existing contract test or pending-fixture gate.
- Creating compact columns/tensors from `list[int]` requires a dtype in the contract.
- Loading external integer columns is specified as matching fixed-width dtypes unless explicitly widened.

Validation:

- Contract tests or pending fixtures tied to the owning data-science phase.
- Documentation/contract lint if available.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-6B: Deferred Dtype Runtime Integration

Goal: implement actual array/tensor/dataframe kernels and external loaders when the owning data-science runtime surfaces exist.

Scope:

- Preserve dtype for array/tensor/dataframe arithmetic and expose overflow policy.
- Add explicit widen APIs for exact arbitrary-precision results when supported.
- Load Arrow/Parquet integer columns as matching fixed-width dtypes unless explicitly widened.
- Add row/column context to narrowing diagnostics where available.

Acceptance criteria:

- Runtime kernels satisfy the INT-6A contract.
- Wrapping/saturating/overflowing/widen kernels are explicit.
- Loading external integer columns does not silently widen to arbitrary `int` or narrow without checks.

Validation:

- Dtype arithmetic fixtures where array/tensor/dataframe support exists.
- Arrow/Parquet schema mapping tests when those integrations exist.
- Owning data-science phase validation plus `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-7: Diagnostics, Documentation, and Migration Cleanup

Goal: make the new model teachable and remove bootstrap leftovers.

Scope:

- Add stable diagnostic codes for integer range, narrowing, unsafe division, float precision, bool comparison, JSON policy, and dtype overflow-policy errors.
- Reserve and document the `SIFR-INT-0001..0011` diagnostic families listed in `internal_docs/integer_model.md`.
- Update public docs, internal docs, demos, and issue references to use exact `int` and explicit fixed-width types.
- Remove or quarantine transition fixtures that mention public `bigint`.
- Add examples for web APIs, dataframes/tensors, bytes, FFI, and common domain values.
- Ensure architecture, roadmap, and relevant phase docs point at the canonical design.

Acceptance criteria:

- Users can discover when to use `int`, fixed-width types, decimal, domain newtypes, and serialization profiles.
- Diagnostics include actionable suggestions, stable `SIFR-INT-*` codes, and do not imply `int` is machine-sized.
- Public docs contain no stale `bigint` recommendation.

Validation:

- `rg "bigint|int = i64|i64-backed|wrap" docs internal_docs issues demos crates verification` reviewed for intentional remaining matches.
- Documentation lint/check commands where available.
- `scripts/run_all_tests.sh --profile quick`.

## Milestone INT-8: Closure Hardening and Performance Gates

Goal: close the phase with confidence that exact `int` is safe, performant for common code, and not leaking representation surprises.

Scope:

- Add performance checks for small integer loops, counters, range iteration, hashing, and formatting.
- Add `verification/perf/sifr_int_loop.sifr` or the phase-35 equivalent benchmark fixture for a small-int accumulation loop.
- Add fuzz/property tests for parsing, narrowing, arithmetic budgets, serialization profiles, and fixed-width helper APIs.
- Verify generated Rust panic-shape gates across pass fixtures.
- Run full validation and record closure artifacts.
- Produce a final phase closure review with Claude and human review notes if applicable.

Acceptance criteria:

- Common small-`int` loops do not allocate in the loop body for proven-small values, as measured by the repository's phase-35 performance tooling or an explicitly documented allocator probe.
- Small-int accumulation throughput is within the phase-35 budget gate. If phase-35 tooling is not active yet, the INT-8 closure artifact must record a ratified threshold before closure; the default target is within 2x of an equivalent optimized Rust `i64` loop for proven-small values.
- Fuzz/property tests cover high-risk external-input integer paths.
- No user-triggerable integer runtime path panics under generated-code sweep.
- Full validation passes locally.

Validation:

- `scripts/run_all_tests.sh`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- Any targeted benchmark command added during this phase.

## Review History

- [x] Claude review pass 1 completed for original contract: `reviews/integer-model-fixed-width-contract-review-pass-1.md`.
- [x] Claude review pass 2 completed after addressing pass 1 findings: `reviews/integer-model-fixed-width-contract-review-pass-2.md`.
- [x] Claude review pass 3 completed after lock-ready polish: `reviews/integer-model-fixed-width-contract-review-pass-3.md`.
- [x] Principal-engineer broader-surface review pass 4 completed: `reviews/integer-model-fixed-width-contract-review-pass-4-broader-surfaces.md`.
- [x] Principal-engineer broader-surface review pass 5 completed after pass 4 polish: `reviews/integer-model-fixed-width-contract-review-pass-5-broader-surfaces-final.md`.
- [x] Phase-plan review pass 1 completed after splitting design and milestones: `reviews/integer-model-phase-plan-review-pass-1.md`.
- [x] Phase-plan review pass 2 completed after addressing pass 1 blockers: `reviews/integer-model-phase-plan-review-pass-2.md`.
- [x] INT-1 runtime substrate wave 1 review pass 1 completed with blockers: `reviews/integer-model-int-1-runtime-wave-1-review-pass-1.md`.
- [x] INT-1 runtime substrate wave 1 review pass 2 satisfied after addressing blockers: `reviews/integer-model-int-1-runtime-wave-1-review-pass-2.md`.
- [x] INT-1 fixed-width conversion substrate wave review satisfied: `reviews/integer-model-int-1-runtime-fixed-width-conversions-review-pass-1b.md`.
- [x] INT-1 oversized module `int` constant codegen review satisfied: `reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md`.
- [x] INT-1 oversized module `int` constant direct use-site review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md`.
- [x] INT-1 `SifrInt` local propagation and direct comparison use-site review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md`.
- [x] INT-1 `SifrInt` local value-semantics review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md`.
- [x] INT-1 `SifrInt` assignment target review pass 1 completed with value-position alias blocker: `reviews/integer-model-int-1-sifrint-assignment-targets-review-pass-1.md`.
- [x] INT-1 `SifrInt` assignment target review pass 2 satisfied after addressing the alias blocker: `reviews/integer-model-int-1-sifrint-assignment-targets-review-pass-2.md`.
- [x] INT-1 `SifrInt` augmented assignment target review pass 1 satisfied with optional test-hardening notes: `reviews/integer-model-int-1-sifrint-augassign-targets-review-pass-1.md`.
- [x] INT-1 `SifrInt` augmented assignment target review pass 2 satisfied after adding focused registered-source and supported-op unit coverage: `reviews/integer-model-int-1-sifrint-augassign-targets-review-pass-2.md`.
- [x] INT-1 `SifrInt` function return boundary review pass 1 satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-function-return-boundaries-review-pass-1.md`.
- [x] INT-1 `SifrInt` function call arguments and closure return-state review satisfied: `reviews/integer-model-int-1-sifrint-function-call-args-and-closure-return-state-review-pass-1.md`.
- [x] INT-1 nested helper `SifrInt` return propagation review satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-nested-helper-return-propagation-review-pass-1.md`.
- [x] INT-1 module-source recursive nested helper capture parameter review satisfied with non-blocking local-source capture follow-ups: `reviews/integer-model-int-1-sifrint-recursive-capture-params-review-pass-1.md`.
- [x] INT-2A reserved-width diagnostic review pass 1 completed with doc-code normalization blocker: `reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-1.md`.
- [x] INT-2A reserved-width diagnostic review pass 2 satisfied after addressing blocker: `reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-2b.md`.
- [x] INT-2A large integer literal HIR review pass 1 completed with canonical-representation blocker: `reviews/integer-model-int-2a-large-literal-hir-review-pass-1b.md`.
- [x] INT-2A large integer literal HIR review pass 2 approved after canonical decimal normalization: `reviews/integer-model-int-2a-large-literal-hir-review-pass-2.md`.
- [x] INT-2A large integer literal default/unary parity review satisfied: `reviews/integer-model-int-2a-large-literal-defaults-parity-review-pass-1b.md`.
- [x] INT-2A boundary diagnostics and parsed/constructed parity review satisfied: `reviews/integer-model-int-2a-boundary-diagnostics-review-pass-1.md`.
- [x] INT-2B fixed-width type variants and annotation resolution review satisfied: `reviews/integer-model-int-2b-fixed-width-type-variants-review-pass-1.md`.
- [x] INT-2B fixed-width const literal fitting review satisfied after retry: `reviews/integer-model-int-2b-fixed-width-const-fitting-review-pass-1b.md`.
- [x] INT-2B `bigint` transition annotation diagnostic review satisfied: `reviews/integer-model-int-2b-bigint-transition-diagnostic-review-pass-1.md`.
- [x] INT-2B fixed-width const expression fitting review pass 1c completed with shadowing/diagnostic/test blockers: `reviews/integer-model-int-2b-const-expression-fitting-review-pass-1c.md`.
- [x] INT-2B fixed-width const expression fitting review pass 2 satisfied after addressing blockers: `reviews/integer-model-int-2b-const-expression-fitting-review-pass-2.md`.
- [x] INT-2B cross-module const fitting review satisfied: `reviews/integer-model-int-2b-cross-module-const-fitting-review-pass-1b.md`.
- [x] INT-2B `bigint` warning coverage review pass 1 completed with duplicate-warning/test-coverage blockers: `reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-1.md`.
- [x] INT-2B `bigint` warning coverage review pass 2 satisfied after addressing blockers: `reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-2.md`.
- [x] INT-2B `bigint(...)` constructor warning review satisfied: `reviews/integer-model-int-2b-bigint-constructor-warning-review-pass-1.md`.
- [x] INT-2B stdlib constant integer value export review satisfied: `reviews/integer-model-int-2b-stdlib-const-values-review-pass-1.md`.
- [x] INT-2B stdlib constant folding integration coverage review satisfied: `reviews/integer-model-int-2b-stdlib-folding-coverage-review-pass-1.md`.
- [x] INT-2B `SIFR-INT-0003` registry placement and e2e fail fixture review satisfied: `reviews/integer-model-int-2b-int0003-registry-e2e-review-pass-1.md`.
- [x] INT-2B transitive const re-export semantics documentation review satisfied: `reviews/integer-model-int-2b-transitive-reexport-doc-review-pass-1.md`.
- [x] INT-2B reserved-width shadowing policy documentation review satisfied: `reviews/integer-model-int-2b-reserved-width-shadowing-policy-review-pass-1.md`.
- [x] INT-2B fixed-width fail fixture marker cleanup review satisfied: `reviews/integer-model-int-2b-fixed-width-fail-fixture-markers-review-pass-1.md`.
- [x] INT-2B module const/fixed-width fallback cleanup review pass 4 satisfied after addressing pass 2 and pass 3 blockers: `reviews/integer-model-int-2b-module-const-fallback-cleanup-review-pass-4.md`.
- [x] INT-2B milestone closure review pass 1 found the milestone ready to close with non-blocking follow-ups: `reviews/integer-model-int-2b-milestone-closure-review-pass-1.md`.

## Implementation Checklist

- [x] INT-0 contract lock and legacy audit
- [ ] INT-1 runtime `SifrInt` and ownership semantics
  - [x] Wave 1 runtime substrate and generated Cargo dependency plumbing reviewed and quick-validated: PR #1789.
  - [x] Wave 1B typed fixed-width conversion substrate reviewed and quick-validated: PR #1790.
  - [x] Module-level `int` constants whose in-budget values exceed `i64` now lower through `SifrInt` helper codegen, removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review; review is satisfied and quick validation is passing: PR #1817.
  - [x] Direct `int`-typed use sites and `+`/`-`/`*` arithmetic that reference oversized `SifrInt` module-constant helpers now coerce participating operands through `SifrInt` and retype receiving local bindings, so expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust; review is satisfied and quick validation is passing: PR #1819.
  - [x] Chained `SifrInt` locals and direct oversized-helper comparisons now lower through the same operand coercion path, so single-use expressions like `oversized_local + 2`, `BIG_LIMIT > 100`, and `oversized_local > BIG_LIMIT` no longer emit invalid legacy `i64` Rust; review is satisfied and quick validation is passing: PR #1821.
  - [x] Repeated direct use of non-`Copy` `SifrInt` locals in helper/local arithmetic, comparisons, and unary negation now borrows exact-int operands where Rust ownership would otherwise move the local, preserving source-level value semantics for expressions like `big + 1`, `big + 2`, `-big`, and `big < other_big`; review is satisfied and quick validation is passing: PR #1823.
  - [x] Plain local assignment targets that later receive exact-int helper/local values now pre-promote their Rust storage to `SifrInt`, coerce small initializers through `SifrInt::from_i64`, and clone registered exact-int locals in value position so aliases like `b: int = a` and `total = a` preserve source value semantics; review is satisfied and quick validation is passing: PR #1825.
  - [x] Local augmented assignment targets for supported exact-int arithmetic now pre-promote receiving `int` locals to `SifrInt` and rewrite `+=`, `-=`, and `*=` as plain assignments with borrowed exact-int operands, preserving value semantics for shapes like `total += big`; review is satisfied and quick validation is passing: PR #1827.
  - [x] Module-level `-> int` functions whose returns transitively depend on exact-int helpers, locals, or promoted zero-argument helper calls now return generated Rust `SifrInt`; return statements are value-coerced and downstream zero-argument call sites retype exact-int locals/arithmetic, preserving shapes like `value: int = returned_big_limit()` and `returned_big_limit() + 1`; review is satisfied and quick validation is passing: PR #1829.
  - [x] Calls to promoted `SifrInt`-returning functions now retype receiving `int` locals/arithmetic even when the call has ordinary arguments, and nested closure bodies no longer inherit promoted outer-function return coercion state, preserving shapes like `result: int = make_big_with_arg(3)` and promoted outers that call small nested helpers; review is satisfied and quick validation is passing: PR #1831.
  - [x] Nested helpers whose annotated `-> int` returns transitively produce `SifrInt` through module exact-int sources or sibling/deeper nested helpers now participate in enclosing function return promotion, so outer functions returning those helper results lower to Rust `SifrInt`; review is satisfied and quick validation is passing: PR #1833.
  - [x] Recursive nested helper capture parameters for module exact-int sources now lower as Rust `SifrInt` and their hidden capture arguments use the exact-int value path, preserving recursive helpers that capture `BIG_LIMIT`; review is satisfied and quick validation is passing: PR #1835.
  - [ ] Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`, and captured-local-only nested helpers plus local-source recursive capture body coercion still need propagation through the broader function-boundary migration.
- [x] INT-2A parser boundary and literal capture
  - [x] Reserved `int128`/`uint128` names emit `SIFR-INT-0003`, with registry docs generated, review satisfied, and quick validation passing: PR #1791.
  - [x] Parsed integer literals beyond the historical `i64` slot lower to canonical decimal `LargeIntLiteral` HIR across decimal, hex, octal, and binary spellings, with review satisfied and quick validation passing: PR #1792.
  - [x] Default-argument large-literal parity and negative-large-literal unary coverage implemented, review satisfied, and quick validation passing: PR #1793.
  - [x] Malformed integer token text has typed parser diagnostic coverage, over-budget integer literals emit `SIFR-INT-0004`, parsed/constructed HIR parity is covered, review is satisfied, and quick validation is passing: PR #1794.
- [x] INT-2B HIR, type system, and const fitting
  - [x] Fixed-width signed, unsigned, and pointer-sized integer annotations resolve in compiler-owned type layers, nested reserved-width annotation coverage was broadened, review is satisfied, and quick validation is passing: PR #1795.
  - [x] Direct fixed-width const literal fitting accepts fitting annotated assignments/module constants, rejects out-of-range initializers with `SIFR-INT-0001`, preserves non-const/call narrowing rejections, emits suffixed Rust literals, review is satisfied, and quick validation is passing: PR #1796.
  - [x] `bigint` annotations emit warning-only `SIFR-INT-0011` transition diagnostics while preserving the temporary `Type::BigInt` path, review is satisfied, and quick validation is passing: PR #1797.
  - [x] Same-module fixed-width const expression fitting covers integer arithmetic, shifts, non-negative exponentiation, parentheses, immutable module constants, shadowing-safe name lookup, over-budget diagnostics, and no implicit narrowing outside assignment/module-constant surfaces; review is satisfied and quick validation is passing: PR #1798.
  - [x] Imported immutable module constants carry const-evaluable integer values through the project frontend/export API, including alias-aware fitting and shadowing-safe rejection; review is satisfied and quick validation is passing: PR #1799.
  - [x] `SIFR-INT-0011` transition warning coverage includes `isinstance(..., bigint)`, TypeVar bounds/constraints, PEP 695 function/class bounds, and class-bound single-emission behavior; review is satisfied and quick validation is passing: PR #1800.
  - [x] `bigint(...)` constructor calls now emit `SIFR-INT-0011` transition warnings, with constructor-only coverage and annotation-only warning coverage preserved; review is satisfied and quick validation is passing: PR #1801.
  - [x] Stdlib bootstrap now exports public recorded `constant_integer_values` for `.sifr` stdlib constants, preserving project-module export parity without adding a direct `num-bigint` dependency to `sifr_driver`; review is satisfied and quick validation is passing: PR #1802.
  - [x] Stdlib fixed-width fitting now has integration coverage proving a real `.sifr` stdlib integer constant (`sifr.logging.DEBUG`) folds through import into a `uint8` initializer; review is satisfied and quick validation is passing: PR #1804.
  - [x] `SIFR-INT-0003` now has a representative e2e fail fixture and the active INT diagnostic rows are ordered after DECIMAL and before CALL to match family ordering; review is satisfied and quick validation is passing: PR #1806.
  - [x] Imported const-evaluable status is documented as local to the importing module, with no transitive const-value re-export unless the intermediate module defines its own public const-evaluable constant; review is satisfied and quick validation is passing: PR #1808.
  - [x] Reserved `int128`/`uint128` diagnostics are documented as applying after ordinary annotation name resolution, preserving user-defined type variable, alias, and class shadowing until a future language-wide reserved-identifier policy; review is satisfied and quick validation is passing: PR #1810.
  - [x] Fixed-width const-expression fail fixture markers are canonical top-level `expect-error` entries, so the e2e fail harness now enforces `SIFR-INT-0001` and `SIFR-INT-0004` columns; review is satisfied and quick validation is passing: PR #1812.
  - [x] Module constant integer fallback paths now preserve budget diagnostics for over-budget module `int`/fixed-width constants, support same-module `int` const reuse through names/unary/binops, reject mixed fixed-width-to-`int` const reuse before codegen, and smoke-test the new codegen shapes; review is satisfied and quick validation is passing: PR #1814.
- [ ] INT-3 scalar arithmetic and numeric mixing
  - [ ] Add hardening tests that keep implicit `int`-to-fixed-width narrowing rejected in returns, list literals, dict literals, and generic specialization as scalar arithmetic and numeric mixing evolve.
- [ ] INT-4 builtins, indexing, bytes, ranges, and pattern matching
- [ ] INT-5 serialization, web, and schema boundaries
- [ ] INT-6A dtype contract lock
- [ ] INT-6B deferred dtype runtime integration
- [ ] INT-7 diagnostics, documentation, and migration cleanup
- [ ] INT-8 closure hardening and performance gates

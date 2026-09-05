# Ad-Hoc Phase: Integer Model and Fixed-Width Numeric Contract

## Objective

Implement Sifr's final pre-production integer model: Python-simple exact `int` for ordinary code, explicit fixed-width integer families for representation-sensitive work, and strict boundary contracts for serialization, data science, web APIs, and Rust interop.

Canonical design: `internal_docs/integer_model.md`.

This issue is the implementation phase tracker. It intentionally does not restate every semantic rule from the design doc. When this issue and the design doc conflict, update both in the same PR and treat `internal_docs/integer_model.md` as the semantic source of truth.

## Status

- Phase state: ad-hoc, completed.
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
- Review history names the most recent agent review artifact and a human/codex acknowledgement that blocking findings were addressed.

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
- Produce a final phase closure review with agent and human review notes if applicable.

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

- [x] agent review pass 1 completed for original contract: `reviews/integer-model-fixed-width-contract-review-pass-1.md`.
- [x] agent review pass 2 completed after addressing pass 1 findings: `reviews/integer-model-fixed-width-contract-review-pass-2.md`.
- [x] agent review pass 3 completed after lock-ready polish: `reviews/integer-model-fixed-width-contract-review-pass-3.md`.
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
- [x] INT-1 local-source recursive nested helper capture body review satisfied with non-blocking non-recursive capture follow-ups: `reviews/integer-model-int-1-sifrint-local-recursive-capture-body-review-pass-1.md`.
- [x] INT-1 local-source non-recursive nested helper capture body review satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-local-nonrecursive-capture-body-review-pass-1.md`.
- [x] INT-1 function parameter boundary review pass 1 completed with registered-local double-coercion blocker: `reviews/integer-model-int-1-sifrint-function-parameter-boundaries-review-pass-1.md`.
- [x] INT-1 function parameter boundary review pass 2 satisfied after addressing registered-local argument coercion: `reviews/integer-model-int-1-sifrint-function-parameter-boundaries-review-pass-2.md`.
- [x] INT-1 immediate lexical shadowing review satisfied with non-blocking nested-scope shadowing follow-up: `reviews/integer-model-int-1-sifrint-lexical-shadowing-review-pass-1b.md`.
- [x] INT-1 single-level nested lexical shadowing review satisfied with non-blocking multi-level nesting follow-up: `reviews/integer-model-int-1-sifrint-nested-shadowing-review-pass-1.md`.
- [x] INT-1 multi-level nested lexical shadowing review satisfied with non-blocking forced-local capture follow-up: `reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md`.
- [x] INT-1 multi-level forced-local capture review satisfied with non-blocking chained-forcing follow-up: `reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md`.
- [x] INT-1 seeded chained-forcing coverage review satisfied with optional forced-set seeding note: `reviews/integer-model-int-1-sifrint-seeded-chained-forcing-review-pass-1.md`.
- [x] INT-1 checked `SifrInt` floor division/modulo runtime review satisfied with non-blocking overflow-boundary coverage note: `reviews/integer-model-int-1-sifrint-checked-floor-mod-runtime-review-pass-1.md`.
- [x] INT-1 proven exact-int floor/modulo literal-divisor codegen review satisfied with non-blocking naming and coverage hardening notes: `reviews/integer-model-int-1-proven-floor-mod-codegen-review-pass-1.md`.
- [x] INT-1 proven exact-int floor/modulo augmented-assignment review satisfied with no blockers: `reviews/integer-model-int-1-proven-floor-mod-augassign-review-pass-1.md`.
- [x] INT-1 exact-int division/modulo diagnostic scaffold review satisfied with non-blocking direct `//=` literal coverage follow-up: `reviews/integer-model-int-1-exact-int-division-diagnostic-scaffold-review-pass-1.md`.
- [x] INT-1 exact-int augmented-assignment literal suppression coverage review satisfied with no blockers: `reviews/integer-model-int-1-exact-int-augassign-literal-coverage-review-pass-1.md`.
- [x] INT-1 exact-int non-zero guard proof review satisfied with non-blocking `elif`/nested-logic coverage follow-ups: `reviews/integer-model-int-1-exact-int-nonzero-guards-review-pass-1.md`.
- [x] INT-1 exact-int non-zero guard follow-up review satisfied after adding `elif` early-exit and nested boolean guard coverage: `reviews/integer-model-int-1-exact-int-nonzero-guards-review-pass-2.md`.
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
- [x] INT-3 fitting fixed-width scalar `+`/`-`/`*` promotion review satisfied with no blockers and non-blocking broader-width coverage follow-up: `reviews/integer-model-int-3-fixed-width-scalar-promotion-review-pass-1.md`.
- [x] INT-3 fitting fixed-width scalar promotion coverage review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-promotion-coverage-review-pass-1.md`.
- [x] INT-3 fixed-width scalar promotion policy dedupe review satisfied after sharing the temporary scalar promotion helper between type checking and codegen: `reviews/integer-model-int-3-fixed-width-promotion-policy-dedupe-review-pass-1.md`.
- [x] INT-3 fixed-width promotion narrowing-boundary hardening review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-narrowing-hardening-review-pass-1.md`.
- [x] INT-3 fixed-width floor/modulo diagnostic scaffold review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-floor-mod-diagnostic-review-pass-1.md`.
- [x] INT-3 integer exponentiation diagnostic scaffold review satisfied with no blockers: `reviews/integer-model-int-3-integer-power-diagnostic-review-pass-1.md`.
- [x] INT-3 bool/integer comparison diagnostic review satisfied with no blockers: `reviews/integer-model-int-3-bool-integer-comparison-diagnostic-review-pass-1.md`.
- [x] INT-3 exact integer true-division diagnostic review satisfied with no blockers: `reviews/integer-model-int-3-exact-int-true-division-diagnostic-review-pass-1.md`.
- [x] INT-3 generic `Addable` output-boundary review pass 1 completed with a mixed-TypeVar blocker and fixture-diagnostic question: `reviews/integer-model-int-3-generic-addable-output-boundary-review-pass-1.md`.
- [x] INT-3 generic `Addable` output-boundary review pass 2 satisfied after narrowing the guard to same-TypeVar operands and verifying the call-site protocol diagnostic: `reviews/integer-model-int-3-generic-addable-output-boundary-review-pass-2.md`.
- [x] INT-3 fixed-width add API review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-add-apis-review-pass-1.md`.
- [x] INT-3 fixed-width subtraction API review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-sub-apis-review-pass-1.md`.
- [x] INT-3 fixed-width multiplication API review satisfied with no blockers: `reviews/integer-model-int-3-fixed-width-mul-apis-review-pass-1.md`.
- [x] INT-3 milestone closure review pass 1 satisfied: INT-3 scalar arithmetic and numeric mixing is ready to close, with remaining serialization/web, dtype, docs, and hardening surfaces assigned to later milestones: `reviews/integer-model-int-3-milestone-closure-review-pass-1.md`; closure PR #1888.
- [x] INT-4 bytes `uint8` surface review pass 1 satisfied with a non-blocking display-option fallback cleanup note: `reviews/integer-model-int-4-bytes-uint8-surface-review-pass-1.md`.
- [x] INT-4 bytes `uint8` surface review pass 2 satisfied after addressing the pass 1 cleanup note: `reviews/integer-model-int-4-bytes-uint8-surface-review-pass-2.md`.
- [x] INT-4 fixed-width literal pattern fitting review pass 1 satisfied with a non-blocking positive-coverage note: `reviews/integer-model-int-4-fixed-width-match-literal-review-pass-1.md`.
- [x] INT-4 fixed-width literal pattern fitting review pass 2 satisfied after adding positive in-range coverage: `reviews/integer-model-int-4-fixed-width-match-literal-review-pass-2.md`.
- [x] INT-4 fixed-width `sum`/`abs` builtin review pass 1 completed with guardrail and duplicate-policy blockers: `reviews/integer-model-int-4-fixed-width-sum-abs-builtins-review-pass-1.md`.
- [x] INT-4 fixed-width `sum`/`abs` builtin review pass 2 satisfied after extracting `abs` lowering and sharing the widening policy: `reviews/integer-model-int-4-fixed-width-sum-abs-builtins-review-pass-2.md`.
- [x] INT-4 milestone closure review pass 1 satisfied: INT-4 builtins, indexing, bytes, ranges, and pattern matching is ready to close; deferred `bytearray`/`SIFR-INT-0010` work remains a future-slice follow-up, not an INT-4 blocker: `reviews/integer-model-int-4-milestone-closure-review-pass-1.md`; closure PR #1889.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` review satisfied after adding HIR typing, local/try `SifrInt` codegen, and focused e2e coverage: `reviews/integer-model-int-1-exact-int-floor-mod-result-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` return-boundary review pass 1 completed with recursive nested return-type blocker: `reviews/integer-model-int-1-exact-int-floor-mod-result-returns-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` return-boundary review pass 2 satisfied after making recursive nested return typing explicit: `reviews/integer-model-int-1-exact-int-floor-mod-result-returns-review-pass-2.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` local-result return review satisfied after adding local binding promotion and chained helper coverage: `reviews/integer-model-int-1-exact-int-result-local-return-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` nested result-helper return review satisfied after adding nested fixed-point propagation and e2e coverage: `reviews/integer-model-int-1-exact-int-nested-result-return-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` parameter-boundary review satisfied after promoting result params and passthrough returns: `reviews/integer-model-int-1-exact-int-result-param-boundary-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` local-alias review satisfied after registering `Result<SifrInt, E>` locals and alias coverage: `reviews/integer-model-int-1-exact-int-result-local-alias-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` class-method return review satisfied after adding `Class::method` result-return propagation and method call-site coverage: `reviews/integer-model-int-1-exact-int-method-result-return-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` field-receiver method call review satisfied after resolving promoted calls through `self.field.clone().method(...)`: `reviews/integer-model-int-1-exact-int-method-field-result-call-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` nested-field receiver review satisfied after recursively lowering structured field access receivers: `reviews/integer-model-int-1-exact-int-nested-field-result-call-review-pass-1.md`.
- [x] INT-1 exact-int floor/mod `Result[int, DivisionError]` class-method parameter review satisfied after promoting method parameters and direct promoted call arguments: `reviews/integer-model-int-1-exact-int-method-result-params-review-pass-1.md`.
- [x] INT-1 milestone closure review pass 1 satisfied: INT-1 is ready to close; all 38 checklist items (including the broad `Type::Int` migration follow-up) are addressed by the landed PR sequence; remaining `bigint` transition fixtures are owned by INT-7; validation passes: `reviews/integer-model-int-1-milestone-closure-review-pass-1.md`.
- [x] INT-5 runtime JSON integer profile machinery review satisfied after adding `sifr_runtime::json` exact/web/string profile helpers, JS-safe integer enforcement, digit-limit validation, canonical builtin error registration, and focused runtime/e2e coverage: `reviews/integer-model-int-5-json-profile-runtime-review-pass-1.md`; PR #1890.
- [x] INT-5 stdlib JSON profile wrapper review satisfied after exposing `dumps_exact`, `dumps_web`, and `dumps_string_ints` through the current `sifr.json` `JsonValue` boundary with recursive runtime-profile enforcement and focused e2e coverage: `reviews/integer-model-int-5-json-profile-stdlib-wrappers-review-pass-1.md`; PR #1891.
- [x] INT-5 schema/client/generated-serde/storage boundary contract review satisfied after locking OpenAPI/JSON Schema mappings, TypeScript precision-safe client types, generated serde profile routing, SQL/storage explicit representation rules, and the `SIFR-INT-0009` diagnostic contract: `reviews/integer-model-int-5-schema-boundary-contracts-review-pass-1.md`; PR #1892.
- [x] INT-5 JSON load digit-limit review satisfied after adding runtime JSON integer-token scanning, pre-`serde_json` `json_loads` budget enforcement, a typed `sifr.json.validate_integer_digit_limits` `JsonLimitError` boundary, and quick-lane fixture coverage: `reviews/integer-model-int-5-json-load-digit-limits-review-pass-1.md`; PR #1893.
- [x] INT-5 milestone closure review pass 1 satisfied: runtime JSON exact/web/string profile machinery, public `sifr.json` profile wrappers, JSON load digit limits, builtin error surfaces, and the schema/client/generated-serde/storage/`SIFR-INT-0009` boundary contract are complete for current surfaces; web/schema/model emitters remain explicitly deferred to later surface-owning phases: `reviews/integer-model-int-5-milestone-closure-review-pass-1.md`; PR #1894.
- [x] INT-6A dtype contract lock review satisfied after adding the test-owned integer dtype contract artifact, quick/pr validation sentinels, fixed-width dtype construction/arithmetic/Arrow/Parquet rules, and the future `SIFR-INT-0008` emission contract: `reviews/integer-model-int-6a-dtype-contract-lock-review-pass-1.md`; PR #1895.
- [x] INT-6B deferred dtype runtime integration closure review satisfied: array, tensor, dataframe, and Arrow/Parquet runtime surfaces are Phase 42 scope; the INT-6A dtype contract is wired into quick/pr/nightly/release validation lanes and fails closed against silent wrapping or implicit widening until those owning surfaces exist: `reviews/integer-model-int-6b-deferred-runtime-closure-review-pass-1.md`; PR #1896.
- [x] INT-7 diagnostic family reservation wave 1 review satisfied after adding reserved non-emittable registry entries for `SIFR-INT-0002`, `SIFR-INT-0008`, `SIFR-INT-0009`, and `SIFR-INT-0010`, completing public/internal documentation coverage for `SIFR-INT-0001..0011`: `reviews/integer-model-int-7-diagnostic-family-reservation-review-pass-1.md`; PR #1897.
- [x] INT-7 closure gap review pass 1 completed with blockers for demo hygiene, phase docs, manifest cleanup, transition fixture quarantine, and diagnostic inventory sync: `reviews/integer-model-int-7-closure-gap-review-pass-1.md`.
- [x] INT-7 demo hygiene and phase doc cleanup review satisfied after updating targeted demos from public `bigint` examples to exact `int`, refreshing generated/idiomatic artifacts, pointing phase docs 13/14/28 at the canonical integer model, and marking `SIFR-TYPE-0006` transition-only in the diagnostic emission inventory: `reviews/integer-model-int-7-demo-phase-doc-cleanup-review-pass-1.md`; PR #1898.
- [x] INT-7 transition fixture quarantine review satisfied after removing `bigint_arithmetic` from quick/pr manifests, adding transition-quarantine comments and the quarantine tracking artifact for remaining alias fixtures, updating decimal demos/fixtures away from public `bigint(...)` forms, and syncing the implementation inventory: `reviews/integer-model-int-7-transition-fixture-quarantine-review-pass-1.md`; PR #1899.
- [x] INT-7 milestone closure review pass 1 satisfied: all acceptance criteria are met, the five prior closure blockers are resolved by PRs #1897/#1898/#1899, quick validation passes, transition quarantine artifacts are in place, and `SIFR-TYPE-0006` is documented as transition-only until alias removal: `reviews/integer-model-int-7-milestone-closure-review-pass-1.md`; PR #1900.
- [x] INT-8 closure hardening gates review pass 1 satisfied: the Sifr small-int loop fixture, integer closure performance runner, ratified 10x pre-Phase-35 throughput threshold, zero-allocation probes, integer JSON/fixed-width property seeds, and closure hardening artifact satisfy the implementation wave with no blockers: `reviews/integer-model-int-8-closure-hardening-gates-review-pass-1.md`; PR #1901.
- [x] INT-8 clippy closure cleanup review pass 1 satisfied: mechanical codegen iterator/helper refactors preserve behavior while restoring `cargo clippy --workspace -- -D warnings` for the closure gate: `reviews/integer-model-int-8-clippy-closure-cleanup-review-pass-1.md`; PR #1902.
- [x] INT-8 milestone closure review pass 1 satisfied: the integer closure performance runner reports zero allocations for small `SifrInt` accumulation/counter/hash loops and a latest local observed slowdown of 3.03x under the ratified 10x pre-Phase-35 threshold; integer external-boundary and fixed-width helper property/fuzz-smoke seeds are registered and green; `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, HIR guardrails, `scripts/run_integer_model_closure_perf.py`, and full `scripts/run_all_tests.sh` pass locally: `reviews/integer-model-int-8-milestone-closure-review-pass-1.md`; PR #1903.
- [x] Final whole-phase closure review pass 1 satisfied: all 11 milestones (INT-0 through INT-8) have individually satisfied closure reviews, complete implementation checklists, and documented review-history entries; every deferral is explicit, owned, and bounded (dtype runtime -> Phase 42, performance tooling -> Phase 35 with ratified 10x threshold, web/schema emitters -> Phase 40/41, public `bigint` fixtures -> quarantined); no code or docs blockers remain; only tracker phase-state wording update required: `reviews/integer-model-phase-closure-review-pass-1.md`; PR #1904.
- [x] Post-closure implementation gap review pass 1 satisfied: adversarial review found no blockers in contract coverage, validation manifests, deferrals, stale public docs, fixed-width semantics, JSON/profile boundaries, or user-triggerable panic paths: `reviews/integer-model-phase-implementation-gap-review-pass-1.md`; PR #1905.
- [x] Post-closure adversarial implementation review pass 1 satisfied: code-path review found no blockers in exact-int promotion contexts, fixed-width helper APIs, floor/mod/hash/display semantics, JSON recursion and digit limits, generated runtime dependency materialization, or panic/no-unwrap guarantees: `reviews/integer-model-phase-implementation-adversarial-review-pass-1.md`; PR #1905.

## Implementation Checklist

- [x] INT-0 contract lock and legacy audit
- [x] INT-1 runtime `SifrInt` and ownership semantics
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
  - [x] Recursive nested helpers that capture outer locals already forced to `SifrInt` now propagate that exact-int state through nested return pre-scan, helper body lowering, recursive hidden capture arguments, and enclosing function return promotion, preserving shapes like `big: int = BIG_LIMIT + 1` followed by `return helper(2)`; review is satisfied and quick validation is passing: PR #1837.
  - [x] Non-recursive nested helper closures that capture outer locals already forced to `SifrInt` now propagate that exact-int state through nested return analysis and closure body lowering, preserving shapes like `big: int = BIG_LIMIT + 1` followed by a captured-local `helper()` call; review is satisfied and quick validation is passing: PR #1839.
  - [x] Module-level `int` parameter positions whose call sites receive `SifrInt`-shaped arguments now promote to Rust `SifrInt`, coerce exact and small call arguments consistently, and register promoted parameters as exact-int locals inside function bodies, preserving module helpers such as `echo_int_parameter(BIG_LIMIT)`, `echo_int_parameter(reusable_oversized_local)`, and mixed-position exact-int arguments; review is satisfied after two passes and quick validation is passing: PR #1841.
  - [x] Function-local and parameter bindings that shadow oversized exact-int module constants now suppress helper rewrites and SifrInt pre-scan promotion in their immediate function scope, preserving `BIG_LIMIT: int = 5` and `def f(BIG_LIMIT: int)` shadow cases while unshadowed module constants still lower through `SifrInt`; review is satisfied and quick validation is passing: PR #1843.
  - [x] Single-level nested helpers now preserve outer locals and parameters that shadow oversized exact-int module constants across nested return analysis, closure body rewriting, and recursive hidden capture parameters, preserving non-recursive, recursive, and parameter-shadow helper shapes while unshadowed module constants still lower through `SifrInt`; review is satisfied and quick validation is passing: PR #1845.
  - [x] Multi-level nested helpers now preserve outer locals and parameters that shadow oversized exact-int module constants across nested return analysis, closure body rewriting, and recursive hidden capture parameters, preserving helper-inside-helper local, recursive, and parameter-shadow shapes while unshadowed module constants still lower through `SifrInt`; review is satisfied and quick validation is passing: PR #1847.
  - [x] Multi-level nested helpers now propagate outer locals already forced to `SifrInt` transitively through helper-inside-helper return analysis, closure body lowering, and recursive hidden capture parameters, preserving both non-recursive and recursive local-source forced capture shapes; review is satisfied and quick validation is passing: PR #1849.
  - [x] Multi-level nested helpers with locals derived from captured forced `SifrInt` parents now have non-recursive and recursive regression coverage proving current codegen lowers chained derived locals through `SifrInt`; review is satisfied and quick validation is passing: PR #1851.
  - [x] `SifrInt` now exposes checked floor-division and floor-modulo runtime primitives that return `None` for zero divisors and preserve exact/Python floor semantics across positive, negative, divisible, and large values; review is satisfied and quick validation is passing: PR #1853.
  - [x] `SifrInt`-shaped `//` and `%` expressions with syntactically non-zero integer literal divisors now lower through compiler-proven non-zero floor division/modulo runtime helpers, preserving oversized exact-int module constants, unary receivers, and derived local values without `i64` fallback; review is satisfied and quick validation is passing: PR #1855.
  - [x] `SifrInt`-shaped `//=` and `%=` augmented assignments with syntactically non-zero integer literal right-hand sides now rewrite to plain assignments through compiler-proven non-zero floor division/modulo runtime helpers, preserving promoted exact-int locals without Rust `/=` or `%=` fallback; review is satisfied and quick validation is passing: PR #1856.
  - [x] User-code exact-int `//`, `%`, `//=`, and `%=` with unproven exact-int divisors now fail closed with active `SIFR-INT-0005` diagnostics unless the divisor is a syntactically non-zero integer literal; trusted stdlib lowering remains exempt until broader guard/proof tracking covers its internal non-zero loops; review is satisfied and quick validation is passing: PR #1857.
  - [x] Direct HIR coverage now proves exact-int `//=` and `%=` with syntactically non-zero integer literal divisors still lower successfully, closing the scaffold review hardening note; review is satisfied and quick validation is passing: PR #1858.
  - [x] Conservative non-literal exact-int non-zero facts now suppress `SIFR-INT-0005` for guarded divisors in `x != 0` true branches, `if x == 0: return/raise` early-exit false paths, and `while x != 0` bodies; reassignment and augmented assignment clear those facts, and the guarded `DivisionError` pass fixture now uses the checked parameter divisor again; review is satisfied and quick validation is passing: PR #1859.
  - [x] Exact-int non-zero proof now covers early-exit `elif` zero guards and nested boolean guard composition such as `not (left == 0 or right == 0)`, with focused HIR/e2e coverage, review satisfied, and quick validation passing: PR #1875.
  - [x] Unproven exact-int `//` and `%` now lower as `Result[int, DivisionError]` instead of `SIFR-INT-0005`, with local `Result` bindings and try-block unwrapping emitted through `Result<SifrInt, DivisionError>` and focused review/e2e coverage; direct `int` assignment remains a type mismatch outside handling, fixed-width and augassign remain fail-closed: PR #1876.
  - [x] `Result[int, DivisionError]` function return boundaries that directly return unproven exact-int `//`/`%` or call another promoted result-returning helper now lower as `Result<SifrInt, DivisionError>`; promoted callers, try-unwrapped locals, chained helpers, and mixed plain-`Ok(int)` branches are covered, review is satisfied, and quick validation is passing: PR #1877.
  - [x] `Result[int, DivisionError]` function return boundaries now also promote local `Result` bindings initialized from exact floor/mod results or promoted helper calls, preserving chained local-result helper returns through `Result<SifrInt, DivisionError>`; review is satisfied and quick validation is passing: PR #1878.
  - [x] Nested `Result[int, DivisionError]` helpers discovered inside a returning function now participate in result-return promotion, so outers returning nested helper calls lower through `Result<SifrInt, DivisionError>`; review is satisfied and quick validation is passing: PR #1879.
  - [x] `Result[int, DivisionError]` parameter boundaries whose call sites receive promoted exact result expressions now lower those params as `Result<SifrInt, DivisionError>`, preserving owned passthrough helpers and direct promoted call arguments; review is satisfied and quick validation is passing: PR #1880.
  - [x] `Result[int, DivisionError]` local aliases that receive promoted result params or locals now register as `Result<SifrInt, DivisionError>`, preserving alias-return and downstream call shapes; review is satisfied and quick validation is passing: PR #1881.
  - [x] `Result[int, DivisionError]` class method returns now participate in exact result promotion, including method-to-method calls such as `self.divide(...)` and try-unwrapping of promoted method results; review is satisfied and quick validation is passing: PR #1882.
  - [x] Promoted `Result[int, DivisionError]` method calls through class fields now recover the receiver class from field metadata, so `self.calc.divide(...)` local aliases lower as `Result<SifrInt, DivisionError>` and unwrap through `SifrInt`; review is satisfied and quick validation is passing: PR #1883.
  - [x] Nested field receivers such as `self.holder.calc.divide(...)` now recursively lower through structured field access instead of falling into the production `compile_error!` stub, preserving promoted `Result<SifrInt, DivisionError>` local aliases; review is satisfied and quick validation is passing: PR #1884.
  - [x] `Result[int, DivisionError]` class method parameters whose call sites receive promoted exact result expressions now lower as `Result<SifrInt, DivisionError>`, preserving method passthrough helpers and direct promoted call arguments; review is satisfied and quick validation is passing: PR #1885.
  - [x] Continue the broader `Type::Int` migration beyond direct helper/local expression rewrites: direct function-return promotion and remaining `Result[int, DivisionError]` integration surfaces now fully covered (PRs #1876–#1885); closure review is satisfied and quick validation is passing: PR #1886.
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
- [x] INT-3 scalar arithmetic and numeric mixing
  - [x] Ordinary fixed-width scalar `+`, `-`, and `*` now promote fitting fixed-width operands to source-level `int` and cast operands before generated Rust arithmetic, preserving `int32(2_000_000_000) + int32(2_000_000_000) -> int`; review is satisfied and quick validation is passing: PR #1860.
  - [x] Fixed-width scalar promotion coverage now spans all fitting fixed-width families (`int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, and `isize`) while keeping `uint64` and `usize` blocked until the broader `SifrInt` promotion path; review is satisfied and quick validation is passing: PR #1861.
  - [x] Deduplicate the temporary `fixed_width_promotes_to_current_int` policy between type checking and codegen once the broader `SifrInt` promotion path lands; review is satisfied and quick validation is passing: PR #1887.
  - [x] Promoted fixed-width arithmetic results are now hardened against implicit narrowing in fixed-width returns, list literals, dict literals, and generic class specialization; review is satisfied and quick validation is passing: PR #1862.
  - [x] Fixed-width scalar `//`, `%`, `//=`, and `%=` now fail closed with `SIFR-INT-0005` instead of lowering through ordinary Rust integer operators while the typed `Result[int, DivisionError]` path is pending; exact `int` non-zero proof behavior is preserved, review is satisfied, and quick validation is passing: PR #1863.
  - [x] Exact integer `**` now fails closed for negative or runtime-dependent exponents while preserving non-negative literal exponents, and fixed-width `**`/`**=` now emit `SIFR-INT-0005` instead of silently becoming float or lowering through unchecked casts; review is satisfied and quick validation is passing: PR #1864.
  - [x] Direct bool/integer equality and ordering comparisons now emit active `SIFR-INT-0007` across exact, bigint-transition, literal, and fixed-width integer shapes while preserving bool/bool and unrelated comparisons; review is satisfied and quick validation is passing: PR #1865.
  - [x] Exact and fixed-width integer true division now fails closed with active `SIFR-INT-0006` instead of silently lowering through `float` casts while the fallible `Result[float, ...]` path is pending; review is satisfied and quick validation is passing: PR #1866.
  - [x] Generic addition now rejects unbounded `T + T -> T`, preserves `Addable` exact-int generic addition, and proves fixed-width `int32` cannot satisfy `Addable` for `T + T -> T` because ordinary fixed-width `+` promotes to `int`; review is satisfied and quick validation is passing: PR #1867.
  - [x] Fixed-width instance `checked_add`, `wrapping_add`, `saturating_add`, and `overflowing_add` now expose explicit representation-preserving addition for same-width operands, reject mixed-width operands, lower to Rust primitive no-panic APIs, and cover overflow/non-overflow behavior in e2e; review is satisfied and quick validation is passing: PR #1868.
  - [x] Fixed-width instance `checked_sub`, `wrapping_sub`, `saturating_sub`, and `overflowing_sub` now expose explicit representation-preserving subtraction for same-width operands, reject mixed-width operands, lower to Rust primitive no-panic APIs, and cover underflow/non-underflow behavior in e2e; review is satisfied and quick validation is passing: PR #1869.
  - [x] Fixed-width instance `checked_mul`, `wrapping_mul`, `saturating_mul`, and `overflowing_mul` now expose explicit representation-preserving multiplication for same-width operands, reject mixed-width operands, lower to Rust primitive no-panic APIs, and cover overflow/non-overflow behavior in e2e; review is satisfied and quick validation is passing: PR #1870.
- [x] INT-4 builtins, indexing, bytes, ranges, and pattern matching
  - [x] `bytes` indexing, guarded indexing, and iteration now expose `uint8`; ordinary indexes and lengths remain `int`, stdlib bytes helpers widen explicitly with `int(b)`, display fallback typing is aligned, and focused bytes fixtures cover the surface; review is satisfied and quick validation is passing: PR #1872.
  - [x] Fixed-width match literal patterns now reuse the fixed-width fitting diagnostic path, so in-range `uint8` cases such as `case 255` lower and out-of-range cases such as `case 256` fail with `SIFR-INT-0001`; review is satisfied and quick validation is passing: PR #1873.
  - [x] `sum(list[int32])` and `abs(int8.MIN)` now widen to `int` for the currently safe fixed-width builtin families, with shared policy coverage, focused e2e coverage, review satisfied, and quick validation passing: PR #1874.
- [x] INT-5 serialization, web, and schema boundaries
  - [x] Shared runtime JSON integer profile primitives now live in `sifr_runtime::json`: `json.exact` emits exact integer numbers, `json.web` rejects JavaScript-unsafe JSON numbers with `JsonIntegerRangeError`, and `json.string_ints` emits decimal strings. `JsonIntegerRangeError` and `JsonLimitError` are registered as builtin errors and covered by runtime/e2e tests; review is satisfied and quick validation is passing: PR #1890.
  - [x] Current `sifr.json` `JsonValue` serialization exposes explicit `dumps_exact`, `dumps_web`, and `dumps_string_ints` wrappers. The wrappers call the shared runtime profile policy through codegen intrinsics, apply the selected integer profile recursively to nested arrays/objects, and report `JsonIntegerRangeError` paths such as `$.items[1]`; review is satisfied and quick validation is passing: PR #1891.
  - [x] OpenAPI/JSON Schema, TypeScript client, generated serde, SQL/storage, and `SIFR-INT-0009` boundary rules are locked in `verification/integer_model_serialization_boundary_rules.md`; review is satisfied and quick validation is passing: PR #1892.
  - [x] JSON input integer tokens are scanned against `DEFAULT_JSON_INTEGER_DIGIT_LIMIT` before `serde_json` parsing, and `sifr.json.validate_integer_digit_limits` exposes typed `JsonLimitError` validation for callers that need the explicit limit boundary; review is satisfied and quick validation is passing: PR #1893.
- [x] INT-6A dtype contract lock
  - [x] Integer dtype semantics are locked in `verification/validation_contracts/integer_dtype_contract.md`: fixed-width dtype names, explicit `list[int]` compact-storage dtype selection, dtype-preserving fallible arithmetic, explicit checked/wrapping/saturating/overflowing/widen APIs, future `SIFR-INT-0008` emission, and Arrow/Parquet fixed-width mappings. The sentinel script is wired into quick/pr/nightly/release validation lanes; review is satisfied and quick validation is passing: PR #1895.
- [x] INT-6B deferred dtype runtime integration
  - [x] Runtime array/tensor/dataframe kernels and Arrow/Parquet loaders are deferred to Phase 42, where the owning data-science surfaces are planned. This phase now closes the milestone by linking the future owner and relying on the INT-6A validation contract to preserve dtype-preserving fallible arithmetic, explicit overflow policy APIs, explicit widening, fixed-width external schema mapping, and future `SIFR-INT-0008` emission until implementation surfaces exist: PR #1896.
- [x] INT-7 diagnostics, documentation, and migration cleanup
  - [x] Reserved and documented the remaining non-emittable integer diagnostic slots `SIFR-INT-0002`, `SIFR-INT-0008`, `SIFR-INT-0009`, and `SIFR-INT-0010`, so the generated public and internal diagnostic tables now account for every `SIFR-INT-0001..0011` family code: PR #1897.
  - [x] Updated targeted demos and generated artifacts to use exact `int` instead of public `bigint`, refreshed phase docs 13/14/28 to defer integer semantics to `internal_docs/integer_model.md`, and documented `SIFR-TYPE-0006` as transition-only until public alias removal: PR #1898.
  - [x] Removed transition-only `bigint_arithmetic` from quick/pr pass manifests, quarantined remaining public `bigint` alias fixtures in `verification/integer_model_bigint_transition_quarantine.md`, and updated decimal demos/fixtures to use exact `int` source forms where the public alias was not needed: PR #1899.
  - [x] INT-7 milestone closure review is satisfied with no blockers: `reviews/integer-model-int-7-milestone-closure-review-pass-1.md`; PR #1900.
- [x] INT-8 closure hardening and performance gates
  - [x] Added `verification/perf/sifr_int_loop.sifr`, `scripts/run_integer_model_closure_perf.py`, integer external-boundary and fixed-width helper fuzz/property seeds, and `verification/integer_model_closure_hardening.md`; the runner reports zero heap allocations for small `SifrInt` accumulation/counter/hash loops and a local observed slowdown near 3.1x under the ratified 10x pre-Phase-35 threshold: PR #1901.
  - [x] Cleaned up codegen clippy findings exposed by the INT-8 closure gate without semantic changes, restoring `cargo clippy --workspace -- -D warnings`: PR #1902.
  - [x] INT-8 milestone closure review is satisfied with no blockers after full local validation, clippy, fmt, guardrails, property/fuzz-smoke hardening, and the explicit integer performance gate: `reviews/integer-model-int-8-milestone-closure-review-pass-1.md`; PR #1903.

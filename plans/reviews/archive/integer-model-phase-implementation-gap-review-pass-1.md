

Now I have a comprehensive picture. Let me write the final adversarial review.

---

# Integer Model Phase — Adversarial Gaps-and-Contract-Coverage Review

## Verdict: **SATISFIED**

No blocking findings. The phase is ready for production.

---

## Methodology

I read the canonical design (`internal_docs/integer_model.md`), the phase tracker (`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`), all 11 milestone closure reviews, and cross-referenced them against actual codebase state. I ran targeted grep queries for panic/unwrap patterns in runtime code, `Type::Int` codegen paths, missing error class registrations, `bigint` surface references in demos/docs, and validation manifest wiring. I read the key codegen promotion infrastructure, the `sifr_runtime` public surface, the error code registry, and the BUILTIN_ERROR_CLASSES list.

---

## Blocking Findings

**None.**

---

## Non-Blocking Notes (for the record)

**N1. `sifr_type_to_rust_type` still maps `Type::Int` → `RustType::I64` → `"i64"` in preamble.rs:7**

The function and its test (preamble.rs:1613) still describe the legacy `i64` mapping for `Type::Int`. This is acceptable because the function is used only for legacy-owned surfaces (file handle ID, random state index, `line`/`column` fields in error structs). All actual `int`-bearing generated code uses the promoted path that bypasses this function and emits `SifrInt` directly. The function name, comment, and test name are stale but the behavioral coverage is complete. The test assertion `"maps_types_to_structured_rust_types"` tests the utility, not the primary codegen path.

*Fix if desired*: Rename or update the comment to clarify this utility is for legacy-owned surfaces only. Not blocking.

**N2. `ArithmeticLimitError`, `FloatOverflowError`, `FloatPrecisionLossError` have no runtime structs**

The design doc and tracker list these as error classes that "belong" to specific operator surfaces. Those surfaces (integer `**`/`<<` with budget errors, exact `int` to `float` conversion) are scaffolded but not yet lowered to typed Result paths in the runtime. The runtime has no struct for any of these three. The architecture docs document them. The INT-3 closure review explicitly notes this is appropriate for the scaffold phase.

*Assessment*: Correctly deferred. When the operators are fully lowered, the owning surface (INT-3 continuation or INT-5) adds the runtime structs. No gap, no silent failure.

**N3. `SIFR-INT-0002` (implicit narrowing) is registered but non-emittable**

The design doc describes `SIFR-INT-0002` as "implicit narrowing from exact/fixed source to narrower fixed-width target." The registry has the entry (codes.rs lines around 62–70, plus the registered non-emittable rows at 398+). The codebase has no implicit narrowing paths — all narrowing is explicit with `int32(value)` and similar constructors. So `SIFR-INT-0002` is correctly reserved but not emitted.

*Assessment*: Correct. `SIFR-INT-0002` exists to prevent a future regression; it fires if implicit narrowing ever slips through.

**N4. `unwrap`/`expect`/`panic` in `sifr_runtime`**

The grep found `unwrap`/`expect` in `sifr_runtime/src/int.rs` and `sifr_runtime/src/json.rs`. These are:
- `#[cfg_attr(test, allow(clippy::expect_used))]` at the crate root — the allowances are intentional for test-only paths.
- The production paths in `json.rs:290–325` and `int.rs:539, 656` use `unwrap_or_else(|err| panic!(...))` for **internal** errors (digit scanning for known-digit-limit values and formatting). These are not user-triggerable failure paths — they're for malformed internal data.
- No `unwrap`/`expect` in generated/user-facing arithmetic, narrowing, or serialization code paths.

*Assessment*: Clean. The test hygiene flag and internal-only panic paths are appropriate.

**N5. `bytearray` is a stub with `SIFR-INT-0010` deferred**

The design doc explicitly defers `bytearray` support. The INT-4 closure review confirms this is correct. The current `bytes_bytearray_unsupported.sifr` fixture is the intended placeholder. `SIFR-INT-0010` is reserved in the registry.

*Assessment*: Correct deferral. No silent-widen risk.

**N6. `bigint` references in `docs/errors/`**

`SIFR-TYPE-0006.md` and `SIFR-INT-0011.md` reference `bigint` as a transition alias. These are accurate — `SIFR-TYPE-0006` handles `int`/`bigint` mixed arithmetic for backward compatibility with old code, and `SIFR-INT-0011` is the transition warning for old `bigint` usage. Both are correct per the design doc's transition stance.

**N7. `bigint` in `demos/decimal_conversions/emitted.rs` and `idiomatic.rs`**

These are **generated Rust artifacts** that use `num_bigint::BigInt` internally for the `Decimal` runtime. The **Sifr source files** use only `int()`. The Rust is not user-authored, `num-bigint` is an internal dependency, and there is no user-facing `bigint` annotation. Correct.

**N8. Deferral coverage is explicit and bounded**

Every deferred surface has:
- A documented owning phase (Phase 42 for dtype runtime, Phase 35 for performance tooling, Phase 40/41 for web/schema)
- A contract artifact (`integer_dtype_contract.md`, `integer_model_serialization_boundary_rules.md`, `integer_model_closure_hardening.md`)
- A validation lane wiring that fails closed
- A sentinel or fixture that prevents silent regression

No hidden surfaces.

---

## What I Confirmed Is NOT a Gap

| Concern | Evidence |
|--------|----------|
| Legacy `Type::Int` paths still emitting `i64` | All promoted surfaces (function returns, parameters, locals, Result bindings) use `SifrInt` explicitly. `sifr_type_to_rust_type` is legacy-utility only. |
| Fixed-width arithmetic silently wrapping | `int32 + int32 → int` via promotion policy shared between type checker and codegen (PRs #1860, #1887). |
| JSON serialization losing precision | `json.web` rejects JS-unsafe integers with `JsonIntegerRangeError`. `json.exact` and `json.string_ints` preserve precision. |
| Digit limit bypass on JSON load | Pre-`serde_json` digit scan in `json_loads` (intrinsics/json.rs:676–683). |
| Missing `JsonIntegerRangeError`/`JsonLimitError` registration | Both in `BUILTIN_ERROR_CLASSES` (lib.rs:143–144), architecture docs (lines 521–522), and e2e covered. |
| Dtype contract not failing closed | 6 sentinels in `integer_dtype_contract.md`, check script wired into quick/pr/nightly/release. |
| `bigint` in public demos or source files | Zero `.sifr` files in demos use `bigint`. All 4 demo files use `int`. |
| `SIFR-INT-0001..0011` gaps | All 11 codes registered; 4 correctly non-emittable (0002, 0008, 0009, 0010). |
| Validation manifests exclude relevant fixtures | Quick (23 e2e), pr (63 e2e + hardening variants=28), nightly (45 hardening variants) all pass locally. |

---

## Validation Required After Any Future PR

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_integer_dtype_contract.py
scripts/run_all_tests.sh --profile quick
```

If the PR touches integer operator lowering, runtime, or serialization:
```bash
scripts/run_all_tests.sh --profile pr
python3 scripts/run_integer_model_closure_perf.py
```

---

## Suggested PR Grouping

No PR needed. The phase is complete and closed. Only the tracker text update (`facbc0d7`) was required.

If N1 (stale `sifr_type_to_rust_type` comment/test name) is addressed in the future, it can go in a cleanup PR with other hygiene items.

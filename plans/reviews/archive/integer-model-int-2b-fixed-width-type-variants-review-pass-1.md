# INT-2B — Fixed-width Type Variants — Review Pass 1

Reviewer: agent (agent), 2026-05-06.
Branch: `int-2b-fixed-width-type-variants`.
Validation: `scripts/run_all_tests.sh --profile quick` passed, `report_signature=e1bf653aaa770517`.
Working tree: 9 modified files, no new files, no submodule changes, no fixture/snapshot churn.

## Scope under review

This slice introduces the `Type::FixedInt(FixedIntType)` variant and its annotation resolution. The user has explicitly carved the scope:

- In scope: fixed-width type representation (`int8`/`int16`/`int32`/`int64`/`uint8`/`uint16`/`uint32`/`uint64`/`isize`/`usize`) and HIR-level annotation resolution.
- Out of scope: const fitting (literal-into-fixed-width assignment), range diagnostics, fixed-width arithmetic promotion, bigint public-removal.

I read every diff hunk in full and walked the surrounding match arms in [crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs), [crates/sifr_type_system/src/check.rs](crates/sifr_type_system/src/check.rs), [crates/sifr_type_system/src/union.rs](crates/sifr_type_system/src/union.rs), [crates/sifr_codegen/src/preamble.rs](crates/sifr_codegen/src/preamble.rs), [crates/sifr_codegen/src/lower_expr.rs](crates/sifr_codegen/src/lower_expr.rs), [crates/sifr_codegen/src/lower_item.rs](crates/sifr_codegen/src/lower_item.rs), [crates/sifr_codegen/src/expr_ref_emitter.rs](crates/sifr_codegen/src/expr_ref_emitter.rs), [crates/sifr_codegen/src/generic_bounds_helpers.rs](crates/sifr_codegen/src/generic_bounds_helpers.rs), [crates/sifr_hir/src/lower/diagnostics.rs](crates/sifr_hir/src/lower/diagnostics.rs), [crates/sifr_hir/src/lower/typing_and_functions.rs](crates/sifr_hir/src/lower/typing_and_functions.rs), and [crates/sifr_hir/src/hir_nodes.rs](crates/sifr_hir/src/hir_nodes.rs) to look for places that match `Type::Int` exhaustively but silently ignore `Type::FixedInt`.

---

## Correctness analysis

### `Type::FixedInt` variant ([crates/sifr_type_system/src/types.rs:7-10](crates/sifr_type_system/src/types.rs:7))

The new variant carries a `FixedIntType` enum with all 10 widths. `FixedIntType` is `Copy + Eq + Hash`, derives match the rest of the file's primitive-helper enums, and the `source_name` / `rust_name` / `variant_prefix` accessors are exhaustive over the 10 variants. No mismatch between source name (`int8`/`uint8`) and Rust name (`i8`/`u8`).

Spot-checked the three key surfaces that consume `Type::Int`:

- `ownership()` ([types.rs:515](crates/sifr_type_system/src/types.rs:515)) — `FixedInt(_)` joins the primitive Copy bucket. Correct: all 10 widths are `Copy` in Rust.
- `display_name()` ([types.rs:571](crates/sifr_type_system/src/types.rs:571)) — emits `source_name()`. Correct.
- `rust_type()` ([types.rs:647](crates/sifr_type_system/src/types.rs:647)) — emits `rust_name()`. Correct.
- `type_to_enum_variant_prefix()` ([types.rs:799](crates/sifr_type_system/src/types.rs:799)) — emits `variant_prefix()` (PascalCase). The new test `test_fixed_width_type_names_and_union_variants` ([types.rs:1442-1447](crates/sifr_type_system/src/types.rs:1442)) exercises both `display_name` and `union_variant_name` for `FixedInt(U32)`.

### Annotation resolution ([crates/sifr_type_system/src/infer.rs:28-39](crates/sifr_type_system/src/infer.rs:28))

All 10 source names are added to `resolve_type_annotation`. The new unit assertions ([infer.rs:73-112](crates/sifr_type_system/src/infer.rs:73)) cover each name. The reserved-name short-circuit at [crates/sifr_hir/src/lower/typing_and_functions.rs:435](crates/sifr_hir/src/lower/typing_and_functions.rs:435) still fires for `int128`/`uint128` *before* `resolve_type_annotation` is consulted, so those still get `INT_RESERVED_WIDTH_NAME` (verified by both the existing `test_reserved_integer_width_annotations_have_int_code` and the new nested test).

### Union sort key ([crates/sifr_type_system/src/union.rs:182-233](crates/sifr_type_system/src/union.rs:182))

The sort key was renumbered: `Int=2`, `FixedInt(_)=3` (with `source_name()` as secondary), and every subsequent variant shifted by +1 through `BigDecimal=32`. Two observations:

- The pre-existing duplicate primary key for `Tuple` and `Range` (both `(13, "")` after the shift, same shape as `(12, "")` before) is carried forward unchanged. Not a new bug.
- The lexicographic secondary key on `source_name()` orders `int16 < int32 < int64 < int8 < isize < uint16 < uint32 < uint64 < uint8 < usize` — that is, `int8` sorts after `int64` because `"int16"` < `"int8"` lexicographically. It is deterministic and has no functional impact (no snapshot in the suite enumerates a union of fixed-width ints today), but worth noting if a future slice cares about presentation order.

`make_union` deduplication relies on `PartialEq` on `Type`, which derives correctly through `FixedInt(FixedIntType)` so `int8 | int8` collapses to `int8`. `types_overlap` and `member_contains` ([union.rs:237-263](crates/sifr_type_system/src/union.rs:237)) only special-case literal-vs-base; `FixedInt(I8)` and `Int` are correctly treated as non-overlapping (no auto-promotion in narrowing).

### Codegen helpers ([crates/sifr_codegen](crates/sifr_codegen))

The four `uses_debug_display_format`-style predicates were updated in lock-step:

- [error_refs.rs:108](crates/sifr_codegen/src/error_refs.rs:108) — leaf primitive bucket (no error-class refs to collect).
- [expr_ref_emitter.rs:7](crates/sifr_codegen/src/expr_ref_emitter.rs:7) — Display (`{}`) bucket, not Debug (`{:?}`).
- [intrinsic_method_emitters.rs:7](crates/sifr_codegen/src/intrinsic_method_emitters.rs:7) — same.
- [stmt_support_emitter.rs:328](crates/sifr_codegen/src/stmt_support_emitter.rs:328) — same.

Since `i8`/`i16`/.../`isize`/`usize` all `impl Display`, classifying `FixedInt(_)` with `Int`/`Float` is correct.

`sifr_type_to_rust_type` in [preamble.rs:5](crates/sifr_codegen/src/preamble.rs:5) reaches `FixedInt(_)` through the `_ => RustType::Named(ty.rust_type())` fallback, which yields `RustType::Named("i8")` etc. — handled correctly without an explicit arm.

### HIR test coverage ([crates/sifr_hir/src/lower/type_alias_tests.rs](crates/sifr_hir/src/lower/type_alias_tests.rs))

Two new tests, both well-targeted:

- `test_nested_reserved_integer_width_annotations_have_int_code` ([type_alias_tests.rs:138-158](crates/sifr_hir/src/lower/type_alias_tests.rs:138)) — `dict[str, uint128]` and `list[int128]` both surface `INT_RESERVED_WIDTH_NAME` *and* the test asserts no `NAME_UNKNOWN_TYPE` leak. Closes a real concern about reserved-name detection inside container annotations.
- `test_fixed_width_integer_annotations_resolve_in_hir_signatures` ([type_alias_tests.rs:161-192](crates/sifr_hir/src/lower/type_alias_tests.rs:161)) — declares a function taking all 10 fixed-width annotations and returning `usize`, then verifies the resolved param/return `Type` values exactly. This is the load-bearing assertion that ties annotation resolution → HIR `FunctionType` correctly.

Validation passing locally without snapshot churn means no existing union ordering or codegen fixture regressed.

---

## Out-of-scope behaviors (intentional, verified)

These are *not* defects — they are consistent with the slice's stated scope, and I confirmed each one fails closed rather than producing wrong code:

- `Type::Int` is *not* assignable to `Type::FixedInt(_)` and vice versa ([types.rs:1238-1244](crates/sifr_type_system/src/types.rs:1238)). `LiteralInt(_) → FixedInt(_)` is also rejected. So `x: int8 = 5` will emit a type error today (no const-fitting). Confirmed by inspecting `is_assignable_to`.
- `is_numeric()` ([types.rs:836](crates/sifr_type_system/src/types.rs:836)) does not include `FixedInt`. Consequence: `int8 + int8`, unary `-int8`, and `<`/`>` on `FixedInt` all hit `TYPE_UNSUPPORTED_OPERATOR` in [check.rs](crates/sifr_type_system/src/check.rs). That matches the explicit deferral of "fixed-width arithmetic promotion."
- `is_integral_numeric_type` in [check.rs:21-23](crates/sifr_type_system/src/check.rs:21) is unchanged, so `Decimal × int8` is rejected too. Consistent.
- `HirExpr::IntLiteral(_).ty()` ([hir_nodes.rs:553](crates/sifr_hir/src/lower/../hir_nodes.rs:553)) still returns `&Type::Int`. Slice does not introduce a `FixedIntLiteral` or fitting; integer literals stay `Int`.

Each of these is the correct closure point for this slice — they cause type errors at the boundary the slice did not extend, not silent miscompilation.

---

## Latent rough edges (not blocking, callouts for the next slice)

The following are paths that the new variant *can* technically reach today but for which the slice did not adjust the existing `Type::Int`-only logic. None is exercised by the slice's tests or by any current demo, but each is a future-slice tax:

1. `format_type_name` in [crates/sifr_hir/src/lower/diagnostics.rs:46-58](crates/sifr_hir/src/lower/diagnostics.rs:46) falls through to `Debug` for `FixedInt`, so a future user-facing diagnostic like `Result[int8, NotAClass]` would render as `"FixedInt(I8) is not a valid error type"` rather than `"int8 is not a valid error type"`. Routing through `Type::display_name()` (or adding a `FixedInt(_)` arm) would fix it. Not exercised today because the slice did not add any code path that drives `format_type_name(&FixedInt(_))`.

2. `option_inner_from_rust_type` in [crates/sifr_codegen/src/expr_ref_emitter.rs:55-73](crates/sifr_codegen/src/expr_ref_emitter.rs:55) only special-cases `i64`/`f64`/`bool`/`String` substrings of `Option<T>`'s rust_type; `Option<i8>` etc. fall to `Some(Type::Unknown)`. The primary path (`option_inner_type` reading the Sifr-side `Type::Union`) catches Sifr-typed Option-of-int8 first, so this fallback only matters for intrinsic-returned options whose Sifr type lost the union shape. Latent at best.

3. `normalize_simple_compare_scalar_type` and `normalize_simple_numeric_scalar_type` ([lower_expr.rs:1676,1687](crates/sifr_codegen/src/lower_expr.rs:1676)) do not classify `FixedInt(_)`. Today this is unreachable because `is_numeric` rejects fixed-width comparisons / arithmetic in the type checker before these helpers get called; if a future slice extends `is_numeric` it should also extend these.

4. `is_simple_module_primitive_const_type` ([lower_item.rs:17-22](crates/sifr_codegen/src/lower_item.rs:17)) excludes `FixedInt`. Module-level `const x: int8 = 5` would not pass type-checking anyway (no const-fit), so this is not yet reachable.

5. `Type::Range` and `Type::Bytes` iteration always yield `Type::Int` ([types.rs:888,893](crates/sifr_type_system/src/types.rs:888)) — out of scope for this slice but a known follow-up if range/bytes become parametrizable.

I did not find any place where the new variant could currently produce wrong code, panic, or bypass a diagnostic; every uncovered match arm is gated upstream by the existing `is_numeric` / `is_assignable_to` discipline, which the slice deliberately did not relax.

---

## Tests / validation

- 11 new `resolve_type_annotation` assertions in [infer.rs](crates/sifr_type_system/src/infer.rs:71-112) — one per width plus the existing `int`.
- 10 new `rust_type` assertions in [types.rs:1425-1435](crates/sifr_type_system/src/types.rs:1425).
- New `test_ownership_primitives_are_copy` row for `FixedInt(U8)` ([types.rs:1409-1412](crates/sifr_type_system/src/types.rs:1409)).
- New `test_fixed_width_type_names_and_union_variants` ([types.rs:1442-1447](crates/sifr_type_system/src/types.rs:1442)).
- New `test_fixed_width_integer_annotations_resolve_in_hir_signatures` end-to-end through HIR lowering.
- New nested-container reserved-width test.
- `report_signature=e1bf653aaa770517` matches a prior INT-2A run, consistent with this slice making no fixture/snapshot/codegen-output changes.

Test surface is appropriate for the scope: representation + annotation resolution. No e2e fixture is needed because no end-user-visible behavior compiles a new program shape — fixed-width values cannot yet be initialized, arithmetic'd, or compared.

---

## Style / scope hygiene

- All four codegen helper edits are pure pass-through additions to existing match alternation lists — no behavior change for non-`FixedInt` types.
- The union sort renumbering was done atomically (every subsequent key shifted +1); no bucket was left at the old index.
- Public re-export of `FixedIntType` from [crates/sifr_type_system/src/lib.rs:18](crates/sifr_type_system/src/lib.rs:18) follows the existing alphabetical-ish neighbor (`FunctionType`).
- No comments were added that explain *what* the obvious code does; the doc comments on the new variant and on `FixedIntType` accessors describe *why* they exist (representation-sensitive values, source vs. Rust naming).
- No fallback paths or compatibility shims introduced.

Slice fits comfortably under "PR-sized." Diff is +240 / -36 across 9 files.

---

## Verdict rationale

The slice does exactly what it says: introduces `Type::FixedInt(FixedIntType)`, plumbs the four representation methods (ownership, display, rust_type, variant prefix) and the union sort key, wires annotation resolution for the 10 source names, and verifies the wiring with both unit assertions and an HIR-level signature test. Every unchanged surface I checked is gated upstream by `is_numeric` / `is_assignable_to`, which the slice intentionally leaves narrow — so the deferred work (const fitting, arithmetic promotion, range diagnostics) cannot leak through. Latent rough edges 1–5 above are real but each requires *future-scope* code paths to reach, none corrupts behavior today, and noting them is preferable to expanding this slice.

VERDICT: SATISFIED

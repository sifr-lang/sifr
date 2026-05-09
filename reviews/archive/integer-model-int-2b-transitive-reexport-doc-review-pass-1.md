# Review — INT-2B Transitive Const Re-export Documentation (pass 1)

Branch: `int-2b-transitive-const-reexport-doc`
Reference: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), [internal_docs/integer_model.md](../internal_docs/integer_model.md), [reviews/integer-model-int-2b-cross-module-const-fitting-review-pass-1b.md](integer-model-int-2b-cross-module-const-fitting-review-pass-1b.md)
Reviewer scope: docs-only diff — correctness vs. implementation, consistency with surrounding doc, scope discipline, whether it closes the prior follow-up.

## Diff under review

A single new paragraph inserted in `internal_docs/integer_model.md` immediately after the existing cross-module const-evaluation rule (line 101 → line 103 in the new revision):

> Const-evaluable import status is local to the importing module. A module may use an imported immutable constant in its own fixed-width fitting checks, but it does not transitively re-export that imported constant's const value with `from other import LIMIT`. Downstream modules must import from the module that defines the constant, or the intermediate module must define its own public immutable constant with a const-evaluable initializer.

No other files modified. `git status` confirms only `internal_docs/integer_model.md` is dirty.

## Correctness vs. implementation

The paragraph claims three things:

1. **Imported const-evaluable status does not transitively propagate.** Verified by reading [crates/sifr_driver/src/project/exports.rs:93](../crates/sifr_driver/src/project/exports.rs:93): `collect_module_exports` iterates `lowering_result.module.constants`, and only writes `constant_integer_values` for names already in that vector. `module.constants` is populated solely by [crates/sifr_hir/src/lower/module_constants_lowering.rs:10](../crates/sifr_hir/src/lower/module_constants_lowering.rs:10), which walks the module's own `Stmt::AnnAssign` and `Stmt::Assign` statements. Imports — handled separately in `imports.rs` and the local/stdlib full-import branches — write into `ctx.const_integer_values` (the per-module fitting map) but do not append to `module.constants`. Therefore an imported name never participates in `collect_module_exports`, and the documented rule is faithful to the implementation.

2. **A module may use an imported immutable constant in its own fixed-width fitting checks.** Verified — this is exactly what the prior INT-2B cross-module slice (PR #1799) implemented, and the consumer-side `validate_fixed_width_initializer` reads `externals.constant_integer_values` plus `ctx.const_integer_values` to fold imported names into local fitting decisions.

3. **Downstream modules must import directly from the defining module.** Verified — this is the corollary of (1). The transitive case in the prior reviewer note 5 (`consumer.sifr` doing `from constants_mod import ANSWER` and then importing `consumer` into `main`) is not honored, because `consumer.sifr`'s `module.constants` does not contain `ANSWER`.

The fallback path — "the intermediate module must define its own public immutable constant with a const-evaluable initializer" — is correct in the literal-binop sense the doc has used elsewhere (line 89-99 enumerate what a const-evaluable initializer covers). It is technically achievable: an intermediate module can write `MY_LIMIT: int = 254` and that will be exported.

## Consistency with surrounding doc

- Placement is correct. The new paragraph attaches to line 101 ("Imported immutable module constants may carry const-evaluable status across module boundaries…"), which is where the cross-module rule first appears. Reading 101 → 103 in sequence is coherent: line 101 says when imports carry const status; line 103 bounds it.
- Vocabulary aligns. "Imported immutable constant," "const-evaluable status," and "const-evaluable initializer" all reuse phrases already in the doc (lines 97, 101). The paragraph does not invent new terminology.
- The example block at lines 105-110 (unchanged) is still anchored to the in-module case (`LIMIT: int = 200; z: uint8 = LIMIT`) and is not invalidated by the new wording.
- Status-table and validation-matrix sections (lines 511-529) are unaffected; transitive re-export is a documented bound, not a new feature, so no validation-matrix row is owed.

## Scope discipline

This is a docs-only slice with one paragraph added to one file. No code, no tests, no schema changes. Scope matches the issue tracker entry at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:437](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:437) — "document or implement transitive re-export semantics for imported constants" — by taking the *document* fork of that disjunction. The "or implement" half remains open as a future option, as expected.

## Closing the referenced follow-up

The prior review's note 5 explicitly flagged "Worth noting for future docs but not a defect" with respect to transitive re-export semantics. The new paragraph documents exactly that boundary, with both the prohibition and the two workarounds (direct import, intermediate redeclaration). The "document" half of the carry-over follow-up is satisfied.

## Notes / suggestions (non-blocking)

1. **Wording of the `with` clause.** "...it does not transitively re-export that imported constant's const value with `from other import LIMIT`" reads slightly awkwardly because the `with` phrase reads as the *mechanism* of re-export rather than the *trigger* it actually describes. A clearer rewrite would be roughly: "...even when an intermediate module writes `from other import LIMIT`, that statement does not re-export `LIMIT` as a const-evaluable value to its own importers." Stylistic; the meaning is recoverable from context.

2. **Asymmetry the paragraph does not call out.** The producer-side gate `lower_integer_const_expr_simple` ([crates/sifr_hir/src/lower/simple_expr.rs:13](../crates/sifr_hir/src/lower/simple_expr.rs:13)) does **not** accept `Expr::Name`, only literals/unary/binop trees. So the suggested workaround "the intermediate module must define its own public immutable constant with a const-evaluable initializer" works for `MY_LIMIT: int = 254` but **not** for `MY_LIMIT: int = LIMIT` even though line 97 of the doc lists "immutable module constants whose initializer is const-evaluable" as a const-evaluable form. The mismatch — consumer-side fitting accepts Name references, producer-side simple-expr lowerer does not — is pre-existing (not introduced by this slice) and is consistent with the deferred "or implement" half of the issue follow-up. Worth tracking but not a blocker for this docs slice.

3. **Stdlib parity is not specifically called out.** PR #1802 brought stdlib constant exports into parity with user modules, but the doc remains silent on stdlib-vs-user-module symmetry. The new paragraph applies uniformly because the implementation rule it documents is producer-agnostic, so this is fine. If a future docs slice spells out stdlib parity explicitly, the transitive-re-export bound stated here will continue to hold without amendment.

## Validation

Docs-only diff. No test or build validation is owed for this slice. The quoted line and column references in the paragraph (and the cross-references it implies) match the code paths verified by the prior INT-2B cross-module review (pass 1b).

## Verdict

VERDICT: SATISFIED

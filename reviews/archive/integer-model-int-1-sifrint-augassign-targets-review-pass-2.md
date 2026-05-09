# Review: INT-1 SifrInt AugAssign Targets Pass 2

## Verdict

Satisfied.

## Findings

None.

The pass-2 delta is +60 test lines + +84 review-file lines. **Production code is unchanged** since pass-1 (verified by `git diff 0beefe00..HEAD -- crates/sifr_codegen/src/function_emitter.rs crates/sifr/tests/` returning empty). The two new unit tests directly address pass-1 N3 and N4.

### Test 1 — pass-1 N3 (bare-Name registered source)

[rewrites_sifr_int_augassign_registered_source_to_borrowed_operand](crates/sifr_codegen/src/expr_render_helpers.rs:1777) stages `total` in `sifr_int_forced_local_bindings` and `source` in `sifr_int_local_bindings`, then asserts that `total += source` rewrites to a plain `Assign` whose RHS BinOp has `right == Ref { Ident("source") }`. This faithfully reproduces the production scenario where the pre-scan + Let rewrite have already inserted the source local into the *registered* set (not the forced set), and exercises the `coerce_expr_to_sifr_int` path on the value side. The test's destructuring checks both that the AugAssign-to-Assign conversion fired (`let RustStmt::Assign { value, .. } = rewritten`) and that the registered local on the RHS is borrowed (`Ref { mutable: false, expr: Ident("source") }`). Not brittle: the test uses partial destructuring with `..` for unrelated fields and only pins the property pass-1 N3 was concerned with. ✓

### Test 2 — pass-1 N4 (parameterized op coverage)

[rewrites_sifr_int_augassign_for_supported_ops](crates/sifr_codegen/src/expr_render_helpers.rs:1808) iterates over `["+", "-", "*"]`, creates a fresh emitter per iteration (test isolation), and asserts each supported op produces an `Assign` whose RHS BinOp's op equals the input op. This directly answers the pass-1 N4 concern that only `+` was unit-tested while `-` and `*` were e2e-only. The fresh-per-iteration emitter avoids cross-test state leakage in the `RefCell` registries — important because the previous test mutates `sifr_int_local_bindings`/`sifr_int_forced_local_bindings`. ✓

### Coverage matrix after pass-2

| Property                                                    | Coverage                                                     |
|-------------------------------------------------------------|--------------------------------------------------------------|
| AugAssign-to-Assign conversion fires                        | All three new/updated unit tests + e2e fixture               |
| Target borrowed via `&target`                               | `rewrites_forced_sifr_int_augassign_to_assignment` (pass-1)  |
| Literal RHS coerced via `from_i64`                          | `rewrites_forced_sifr_int_augassign_to_assignment` (pass-1)  |
| Registered-local RHS borrowed via `&source`                 | `rewrites_sifr_int_augassign_registered_source_to_borrowed_operand` (pass-2) |
| Supported ops `+`/`-`/`*`                                   | `rewrites_sifr_int_augassign_for_supported_ops` (pass-2)     |
| End-to-end runtime correctness for `+=` and `+= small`      | e2e fixture                                                  |

### Saved review file

Adding [reviews/integer-model-int-1-sifrint-augassign-targets-review-pass-1.md](reviews/integer-model-int-1-sifrint-augassign-targets-review-pass-1.md) (84 lines) to the PR is consistent with prior dual-pass patterns — e.g., PR #1825 had its pass-1 and pass-2 review files committed alongside the implementation work before PR #1826's tracker PR ran. The review's verdict ("Satisfied") and the file path it lives at are both accurate. No staleness risk because the pass-1 review describes the implementation that already shipped (commit `0beefe00`), not the test-only delta. ✓

### Scope adherence

The PR remains strictly within the supported AugAssign scope (`+=`, `-=`, `*=`). Production code unchanged since pass-1; the test additions exercise the same predicates (`is_sifr_int_augassign_op` for HIR pre-scan, `is_sifr_int_arithmetic_op` for Rust IR rewrite) without widening either set. Unsupported HIR ops (`//=`, `%=`, `<<=`, `>>=`, `**=`, `&=`, `|=`, `^=`) and Rust IR ops (`/`, `%`, etc.) continue to fall through both gates. ✓

### Validation

- `cargo test -p sifr_codegen expr_render_helpers::tests:: -- --nocapture` reports 10 passed (up from 8 at pass-1 — the two new tests plus the eight pre-existing).
- `scripts/run_all_tests.sh --profile quick` reports `report_signature=e1bf653aaa770517`, identical to the signature recorded across #1817–#1825 and the pass-1 implementation. Test-only delta correctly preserves the signature.

## Notes

(Non-blocking observations only.)

- **N-pass2-1 — Defensive "should not rewrite" coverage is still absent.** The pass-2 tests pin the *positive* shape (supported ops do rewrite). They don't pin the *negative* shape (unsupported ops do not rewrite). A sibling test asserting that, say, `RustStmt::AugAssign { op: "/", … }` against a forced target stays as `RustStmt::AugAssign` (i.e., no Assign-conversion) would document the deliberate gap and protect against a future contributor accidentally widening `is_sifr_int_arithmetic_op` to include `/` or `%`. Not blocking — the e2e and existing unit suite would surface a regression at the next slice's testing — but the cheapest hardening would be a one-line parameterized loop over `["/", "%", "<<", ">>"]` asserting each stays in `RustStmt::AugAssign`. Optional.

- **N-pass2-2 — Test 2 doesn't pin the LHS or RHS shape.** It only verifies the rewritten BinOp's op matches the input op. The LHS borrow (`Ref { Ident("total") }`) and the RHS coercion (`from_i64(2)`) are both already pinned by `rewrites_forced_sifr_int_augassign_to_assignment` (pass-1), so the new test doesn't need to repeat that property. Targeted scope is appropriate; not brittle.

- **N-pass2-3 — Carry-forward open items unchanged.** Pass-1 N1 (mixed unsupported AugAssign ops on forced SifrInt locals fail rustc), N2 (subscript AugAssign out of scope), and N5 (broader migration items at [issues/…/checklist:437](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)) all remain open at the same scope; pass-2 doesn't address or affect them. Each is correctly out-of-scope per the slice's stated boundaries.

- **N-pass2-4 — Review file inclusion timing.** Some prior milestone slices kept the implementation review uncommitted until the tracker PR ran (e.g., #1820, #1822, #1824), while others committed the reviews alongside the implementation (e.g., the pass-1 + pass-2 review pair for #1825 before #1826). Both patterns appear in the milestone history. Including the pass-1 review in this PR is consistent with the latter pattern and won't conflict with the upcoming tracker PR.

# INT-1 SifrInt Local + Comparison Tracker — Review Pass 1

**Verdict:** Satisfied. No blockers.

## Scope reviewed

PR #1822, branch `int-1-sifrint-local-comparison-tracker` (head `1d8a326f`), `main..HEAD` diff:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference docs:
- [reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md](reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md) (the implementation review this tracker change closes; 206 lines, present on disk)
- Merged implementation: PR #1821 (`gh pr view 1821` → `MERGED 2026-05-06T16:46:49Z`, title "Track SifrInt locals in oversized int rewrites" — title is misleadingly tracker-style but the diff is the implementation slice; verified by inspecting commit `31b8284d` on `main`'s history)

## Brief vs. diff

The brief asked for three changes. All three are present, accurate, and appropriately scoped.

1. **Add the satisfied INT-1 SifrInt-local + comparison-use-site review to Review History.** ✓ — line 398 inserts:

   ```
   - [x] INT-1 `SifrInt` local propagation and direct comparison use-site review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md`.
   ```

   Path matches the on-disk file (verified 206 lines, present). Placement after the PR #1819 review entry and before the INT-2A rows preserves the chronological ordering. The tail qualifier "with non-blocking broader migration follow-ups" mirrors the pass-1 verdict ("Satisfied with non-blocking suggestions") and signals to a future reader that closure was conditional on the new open follow-up below — matching the same wording introduced in PR #1820's tracker. Consistent variation, not a deviation.

2. **Mark the SifrInt-local + direct-comparison sub-item complete with PR #1821.** ✓ — line 431 flips the placeholder follow-up from PR #1820's tracker to:

   ```
   - [x] Chained `SifrInt` locals and direct oversized-helper comparisons now lower through the same operand coercion path, so single-use expressions like `oversized_local + 2`, `BIG_LIMIT > 100`, and `oversized_local > BIG_LIMIT` no longer emit invalid legacy `i64` Rust; review is satisfied and quick validation is passing: PR #1821.
   ```

   Truthfulness check against the implementation diff and the pass-1 implementation review:

   - "**Chained `SifrInt` locals**" — accurate. The new `sifr_int_local_bindings: RefCell<HashSet<String>>` field plus the `Ident` arm in `is_sifr_int_expr` make `oversized_local + 2` resolve as SifrInt-shaped.
   - "**direct oversized-helper comparisons**" — accurate. `is_sifr_int_operand_coercion_op` unions `==/!=/</≤/>/≥` with the existing `+/-/*` set for the BinOp coercion gate, while `is_sifr_int_arithmetic_op` (the propagation gate) deliberately stays narrow because comparison results are `bool`, not SifrInt. The pass-1 review confirmed this asymmetry is intentional and correct.
   - "**the same operand coercion path**" — accurate. Both arithmetic and comparison flow through the same `coerce_expr_to_sifr_int` helper in the unified BinOp arm; the slice did not introduce a parallel codepath.
   - "**single-use** expressions" — load-bearing qualifier, accurate, and importantly *non-overclaiming*. The pass-1 review's N1 finding was that repeated use of a registered SifrInt local fails rustc with E0382 (use-after-move) because `SifrInt` is not `Copy` and `Add`/`Sub`/`Mul` consume `Self`. By scoping the closing claim to "single-use", the bullet refuses to imply that `let a = big + 1; let b = big + 2;` works (it does not). This is the precision the closure needed.
   - Examples `oversized_local + 2`, `BIG_LIMIT > 100`, `oversized_local > BIG_LIMIT` are all the new e2e fixture's load-bearing asserts (verified at [crates/sifr/tests/e2e/pass/module_constants.sifr:22-25](crates/sifr/tests/e2e/pass/module_constants.sifr)).
   - "**review is satisfied and quick validation is passing: PR #1821**" — `gh pr view 1821` confirms `MERGED` on 2026-05-06; the merge commit `31b8284d` on this branch's history matches the implementation diff reviewed in pass-1.

   No overclaiming detected. The "single-use" qualifier is the same precision-narrowing move PR #1820's tracker used ("Direct …", "`+`/`-`/`*` arithmetic") to keep the bullet honest about scope.

3. **Preserve the broader-migration gaps as an explicit open INT-1 follow-up bullet.** ✓ — line 432 adds:

   ```
   - [ ] Continue the broader `Type::Int` codegen migration beyond direct helper/local tracking: repeated use of non-`Copy` `SifrInt` locals must preserve source-level value semantics, lexical shadowing and legacy-emission paths need scope-safe exact-int handling, fallible `//` and `%` still need exact-int runtime/codegen support, and function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`.
   ```

   Cross-referenced against the pass-1 N1–N5 findings:

   | Pass-1 finding | Tracker-bullet phrase | Captured? |
   |---|---|---|
   | **N1** Use-after-move on registered SifrInt local violates the [design's value-semantic rule](internal_docs/integer_model.md:474) | "**repeated use of non-`Copy` `SifrInt` locals must preserve source-level value semantics**" | ✓ The phrasing "source-level value semantics" is verbatim from the design doc and accurately frames the contract. The "non-`Copy`" hint surfaces the Rust-level cause. |
   | **N2** Inner-block shadowing corrupts the outer registry (registry is function-scoped, not block-scoped) | "**lexical shadowing … need scope-safe exact-int handling**" | ✓ Captured. |
   | **N3** Comparison rewrite bypasses non-structured (legacy) emission paths | "**legacy-emission paths need scope-safe exact-int handling**" | ✓ Captured, though slightly conflated with N2 (see N-tracker-1 below). |
   | **N4** Test-matrix gaps (sibling operators, helper-on-right, unary `-`, "should-not-rewrite" guards, registry isolation) | (not captured) | Intentionally omitted — not a user-facing failure shape; consistent with prior tracker pattern (#1818, #1820). |
   | **N5** Code-shape nits (Let arm split, asymmetric op gates, Ident arm placement) | (not captured) | Intentionally omitted — same reasoning. |

   Additionally, the bullet preserves the two gaps explicitly deferred by the slice description that PR #1820's tracker had also listed:

   - "**fallible `//` and `%` still need exact-int runtime/codegen support**" — carried forward, with the new "**runtime/codegen**" qualifier (more precise than #1820's plain "fallible `//` and `%`" because `SifrInt` currently has no `Div`/`Rem` impls — the gap is split between runtime API and codegen wiring).
   - "**function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`**" — carried forward verbatim from #1820's bullet.

   All four user-facing failure shapes from pass-1 N1–N3 plus the two explicitly deferred concerns are tracked.

## Cross-checks

- **Top-level INT-1 line stays open.** Line 426 is still `- [ ] INT-1 runtime SifrInt and ownership semantics`, correct because the new follow-up sub-item at line 432 is unchecked. No spurious milestone-level closure.
- **Sub-item ordering inside INT-1.** Wave 1 (427) → Wave 1B (428) → oversized-module-int closure (429) → direct use-site closure (430) → SifrInt-local + comparison closure (431) → broader-migration follow-up (432). Implementation order is preserved: each completed sub-item sits before the open work that depends on it, and the new open bullet is adjacent to its parent slices.
- **PR linkage.** `gh pr view 1821` returns `MERGED 2026-05-06T16:46:49Z`. The branch's `git log` shows the implementation merge commit `31b8284d` immediately preceded by tracker-only commit `1d8a326f Track SifrInt local comparison closure`. PR number is consistent.
- **Validation reproduction.** `report_signature=e1bf653aaa770517` is identical to the signatures recorded for #1817, #1818, #1819, #1820, and the #1821 implementation review. A tracker-only diff should not move the signature, and it doesn't (`wall_time=83.69s` is normal run-to-run variance).
- **No collateral churn.** Diff is +3/-1 lines on a single file. No edits to `internal_docs/architecture.md`, `internal_docs/roadmap.md`, design doc, code, tests, or fixtures. Per AGENTS.md "doc updates as applicable" — no architecture/roadmap claim depends on this closure beyond what the issue tracker already records, so the no-collateral diff is correct.

## Non-blocking observations

None gate merge.

### N-tracker-1 — Pass-1 N2 and N3 are conflated under "scope-safe exact-int handling"

The phrase "lexical shadowing and legacy-emission paths need scope-safe exact-int handling" treats two distinct concerns as a single follow-up. The fixes look different:

- N2 (lexical shadowing) wants the rewriter to save/restore the registry around if-then/else bodies, loop bodies, match arm bodies, and inner `RustStmt::Block` statements — symmetric to how the function emitters already do save/clear/restore.
- N3 (legacy-emission bypass) wants integer-bearing if/while/match conditions to either route through the structured lowering path or have the rewriter applied at the legacy emission site. Different code surface entirely.

"Scope-safe" reads naturally for N2 but does not really describe N3, which is more about codegen-path coverage than lexical scope. That said, both gaps are likely subsumed when the broader `Type::Int` ⇒ `SifrInt` migration applies SifrInt uniformly across all integer-bearing surfaces, so this conflation is forgivable at the tracker level of granularity. The pass-1 review file (linked from the new history row) captures the distinction precisely.

If a future tracker wants to split this into two separate sub-bullets (one for scope-safe rewriter state, one for legacy-emission rewriter coverage), that's a clean follow-up. Not worth blocking this PR for.

### N-tracker-2 — The "non-`Copy`" Rust-level qualifier is descriptive but ties the tracker bullet to the current runtime impl

"repeated use of non-`Copy` `SifrInt` locals" surfaces the Rust-level cause of the failure. If a future runtime change made `SifrInt` `Copy` (unlikely given the `Box<BigInt>` variant) or replaced it with a `Cow`-style accessor, this wording would need an update. A purely contract-framed phrasing like "repeated use of `int` locals derived from oversized helpers" would be more durable, but the current wording is more diagnostically useful for the next contributor. Defensible either way.

## Verdict

**Satisfied. No blockers.** The diff faithfully reflects the merged implementation: the Review History entry points at the on-disk pass-1 review with a verdict-accurate qualifier, the closed sub-item is checked with the correct PR number and a tightly-scoped prose summary that adds the **single-use** qualifier (preventing overclaim against the use-after-move shape that pass-1 N1 surfaced), the parent INT-1 milestone correctly stays open behind the new sub-item, and the new open bullet captures all four user-facing remaining gaps from pass-1 (N1 repeated use / value semantics, N2 lexical shadowing, N3 legacy-emission reach, plus the carried-forward fallible `//`/`%` and function-boundary concerns) with concrete framing tied to the design doc's value-semantic rule. Pass-1 N4 (test-matrix hardening) and N5 (code-shape nits) are correctly left at the review-file level rather than promoted to tracker bullets — consistent with prior tracker PRs (#1818, #1820). Quick validation reproduces `report_signature=e1bf653aaa770517` — same signature recorded across the milestone — confirming no test or build deltas.

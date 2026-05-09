# INT-1 SifrInt Local Value Semantics Tracker — Review Pass 1

**Verdict:** Satisfied. No blockers.

## Scope reviewed

PR #1824, branch `int-1-sifrint-local-value-semantics-tracker` (head `672dc4d0`), `main..HEAD` diff:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference docs:
- [reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md](reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md) (the implementation review this tracker change closes; 179 lines, present on disk)
- Merged implementation: PR #1823 (`gh pr view 1823` → `MERGED 2026-05-06T17:07:12Z`, title "Preserve SifrInt local value semantics in rewrites"; commit `72b5f6a2` on this branch's history)

## Brief vs. diff

The brief asked for three changes. All three are present, accurate, and well-scoped.

1. **Add the satisfied INT-1 SifrInt-local-value-semantics review to Review History.** ✓ — line 399 inserts:

   ```
   - [x] INT-1 `SifrInt` local value-semantics review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md`.
   ```

   Path matches the on-disk file. Placement after the PR #1821 review entry and before the INT-2A rows preserves the chronological ordering. The tail qualifier "with non-blocking broader migration follow-ups" mirrors the pass-1 verdict ("Satisfied with non-blocking suggestions") and matches the same wording introduced by PR #1820's and #1822's trackers. Consistent variation, not a deviation.

2. **Mark the SifrInt-local-value-semantics sub-item complete with PR #1823.** ✓ — line 433 adds:

   ```
   - [x] Repeated direct use of non-`Copy` `SifrInt` locals in helper/local arithmetic, comparisons, and unary negation now borrows exact-int operands where Rust ownership would otherwise move the local, preserving source-level value semantics for expressions like `big + 1`, `big + 2`, `-big`, and `big < other_big`; review is satisfied and quick validation is passing: PR #1823.
   ```

   Truthfulness check against the implementation diff (verified by inspecting [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs) on this branch's predecessor commit) and the pass-1 implementation review:

   - "**Repeated direct use**" — accurate. The slice fixes the use-after-move shape from pass-1-of-#1821 N1: a registered `SifrInt` local can now be referenced in multiple subsequent BinOp/UnaryOp expressions without being consumed.
   - "**non-`Copy` `SifrInt` locals**" — accurate, surfacing the Rust-level cause. The pass-1 review confirmed `SifrInt` is intentionally not `Copy` ([crates/sifr_runtime/src/int.rs:219-346](crates/sifr_runtime/src/int.rs:219)) because of the `Box<BigInt>` variant.
   - "**helper/local arithmetic, comparisons, and unary negation**" — accurate. The slice covers exactly three rewrite surfaces:
     - Arithmetic: `coerce_expr_to_sifr_int` reorder so `Ident(name) if is_registered_sifr_int_local(&name)` arm wraps in `Ref { mutable: false, expr: Ident(name) }`.
     - Comparisons: new `coerce_expr_to_sifr_int_comparison_operand` normalizes both sides to `&SifrInt`.
     - Unary `-`: new guard at the `RustExpr::UnaryOp` arm coerces the operand if it's `is_sifr_int_expr`-true.
   - "**now borrows exact-int operands where Rust ownership would otherwise move the local**" — accurate. The pass-1 review verified the implementation cooperates with the runtime's `Add<&SifrInt>`/`Sub`/`Mul` impl matrix and `Neg for &SifrInt`.
   - "**preserving source-level value semantics**" — verbatim from [internal_docs/integer_model.md:474](internal_docs/integer_model.md:474), the design rule the slice closes.
   - **Examples** — `big + 1` and `big + 2` correspond to the e2e fixture's `reuse_a` and `reuse_b` (using `reusable_oversized_local` twice in arithmetic without consumption); `-big` corresponds to `negated_reuse: int = -reusable_oversized_local`; `big < other_big` corresponds to `reusable_oversized_local < reuse_b`. All four are pinned by [crates/sifr/tests/e2e/pass/module_constants.sifr:26-31](crates/sifr/tests/e2e/pass/module_constants.sifr) and round-trip at runtime.
   - "**review is satisfied and quick validation is passing: PR #1823**" — `gh pr view 1823` confirms `MERGED 2026-05-06T17:07:12Z` with title "Preserve SifrInt local value semantics in rewrites", matching commit `72b5f6a2` on this branch.

   No overclaiming detected. The bullet:
   - Uses **"Direct"** + enumerated operator surfaces ("helper/local arithmetic, comparisons, and unary negation") to refuse implication that *all* SifrInt-local use sites are now value-semantic-safe.
   - Does **not** claim assignment targets work (the pass-1 N2 case `total = total + big` still emits invalid Rust; correctly carried forward into the open follow-up).
   - Does **not** claim function argument/return boundaries work.
   - Does **not** claim legacy-emission paths get exact-int coverage.
   - Does **not** claim fallible `//`/`%` work.

   This is the precision pattern established by PRs #1820 and #1822 ("Direct …", "single-use") applied consistently here.

3. **Replace the prior open INT-1 follow-up bullet with one that drops the closed gap and surfaces the newly-discovered one.** ✓ — line 434 replaces #1822's bullet:

   **Old (from PR #1822's tracker):**
   > "...repeated use of non-`Copy` `SifrInt` locals must preserve source-level value semantics, lexical shadowing and legacy-emission paths need scope-safe exact-int handling, fallible `//` and `%` still need exact-int runtime/codegen support, and function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`."

   **New:**
   > "Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: assignment targets such as `total = total + big` still need exact-int target handling, lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, fallible `//` and `%` still need exact-int runtime/codegen support, and function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`."

   The diff effects:

   - **Removed**: "repeated use of non-`Copy` `SifrInt` locals must preserve source-level value semantics" — correctly removed because PR #1823 closes this for the three operator surfaces named in the closing bullet above. ✓
   - **Added**: "**assignment targets such as `total = total + big` still need exact-int target handling**" — captures the new pass-1 N2 finding. The `total: int = 0; total = total + big` shape (with `big` registered SifrInt) emits `total = SifrInt::from_i64(total) + &big`, which fails rustc with `expected i64, found SifrInt` because the Assign rewrite doesn't retype the target. Pre-PR-#1823 this same code already failed (RHS was `SifrInt::from_i64(total) + big` with use-after-move plus the type mismatch), so this is not a regression — but it's a real reachable failure shape that wasn't surfaced as a tracker item before. The example `total = total + big` is taken straight from the pass-1 review's reproducer. ✓
   - **Carried forward**: "lexical shadowing and legacy-emission paths need scope-safe exact-int **coverage**" — minor wording shift from "handling" to "coverage", same intent. Captures pass-1 N4 (lexical shadowing) and pass-1-of-#1821 N3 (legacy emission). ✓
   - **Carried forward**: "fallible `//` and `%` still need exact-int runtime/codegen support" — verbatim from #1822's bullet. ✓
   - **Carried forward**: "function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`" — verbatim from #1822's bullet, captures pass-1 N3. ✓

   Cross-referenced against pass-1 N1–N5:

   | Pass-1 finding | Tracker phrase | Captured? |
   |---|---|---|
   | **N1** Always-borrow comparison style nit (functionally correct, stylistic) | (not captured) | Reasonable omission — not a user-facing failure shape; consistent with prior tracker pattern. |
   | **N2** `Assign` target retype gap (`total = total + big`) | "**assignment targets such as `total = total + big` still need exact-int target handling**" | ✓ With concrete worked example. |
   | **N3** Function-argument boundary | "**function argument/return boundaries still need uniform `SifrInt` lowering**" | ✓ Carried forward. |
   | **N4** Lexical shadowing | "**lexical shadowing … need scope-safe exact-int coverage**" | ✓ Carried forward. |
   | **N5** Doc/test polish (load-bearing match-arm comment, Ref-arm comment, unary `-` unit test) | (not captured) | Reasonable omission — code-shape and test-hardening, not user-facing. |

   The swap is semantically correct: the old "value semantics" placeholder for the use-after-move concern has been closed by #1823, and the *next* concrete user-facing failure shape ("`total = total + big` fails rustc") that emerged from the implementation review takes its place. This is exactly how a milestone-tracker should evolve as nested follow-ups land — close the closed gap, lift the next concrete one out of the review file.

## Cross-checks

- **Top-level INT-1 line stays open.** Line 427 is still `- [ ] INT-1 runtime SifrInt and ownership semantics`. The new sub-item at line 434 is unchecked. No spurious milestone-level closure.
- **Sub-item ordering inside INT-1.** Wave 1 (428) → Wave 1B (429) → oversized-module-int closure (430) → use-sites direct closure (431) → SifrInt local + comparison closure (432) → SifrInt local value-semantics closure (433) → broader-migration follow-up (434). Implementation order is preserved: each completed slice sits before the open work that depends on it, and the new open bullet is adjacent to its parent slices.
- **PR linkage.** `gh pr view 1823` returns `MERGED 2026-05-06T17:07:12Z`, title "Preserve SifrInt local value semantics in rewrites". The branch's `git log` shows the implementation merge commit `72b5f6a2` immediately preceded by tracker-only commit `672dc4d0 Track SifrInt local value semantics closure`. PR number is consistent.
- **Validation reproduction.** `report_signature=e1bf653aaa770517` is identical to the signatures recorded for #1817–#1823. Tracker-only diff preserves the signature, as expected. `wall_time=72.00s` is normal run-to-run variance.
- **No collateral churn.** Diff is +3/-1 lines on a single file. No edits to `internal_docs/architecture.md`, `internal_docs/roadmap.md`, design doc, code, tests, or fixtures. AGENTS.md "doc updates as applicable" — no architecture/roadmap claim depends on this closure beyond what the issue tracker already records, so the no-collateral diff is correct.

## Non-blocking observations

None gate merge.

### N-tracker-1 — N4 (lexical shadowing) and PR-#1821-N3 (legacy emission) remain conflated under "scope-safe exact-int coverage"

Same observation as #1822's tracker review: the phrase "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage" treats two distinct concerns as one follow-up. The fixes look different — N4 wants block-scoped registry save/restore, N3-of-#1821 wants legacy-emission stmts to be routed through (or imitate) the structured rewriter. Both concerns are likely subsumed when the broader `Type::Int` ⇒ `SifrInt` migration applies SifrInt uniformly, so the conflation is forgivable at tracker granularity. The pass-1 review files (linked from the new history rows) capture the distinction precisely. If a future tracker wants to split this into two separate sub-bullets, that's a clean follow-up.

### N-tracker-2 — The new "assignment target" example is a concrete, reachable shape but the bullet does not call out that pre-PR it failed differently

The bullet says "assignment targets such as `total = total + big` still need exact-int target handling". A reader may infer this is a *new* failure introduced by #1823. The pass-1 review clarifies that pre-#1823 the same code already failed (use-after-move + type mismatch), so #1823 is not a regression. A reader who only reads the tracker bullet without consulting the pass-1 review might mistakenly suspect a regression. Adding ", which is a pre-existing shape that #1823 made more visible by closing the upstream value-semantics gap" would help, but it's verbose for a tracker bullet and the linked review file already explains. Optional polish.

### N-tracker-3 — Stylistic: "borrows exact-int operands" leans on Rust borrow vocabulary

"borrows exact-int operands" surfaces the Rust mechanic that produces the value-semantic property, which is more diagnostically useful for the next contributor than a purely contract-framed phrasing ("does not consume the local"). Defensible. Same observation as #1822's N-tracker-2 about "non-`Copy`" wording — durability vs. clarity tradeoff resolved in favor of clarity, consistent with prior tracker entries.

## Verdict

**Satisfied. No blockers.** The diff faithfully reflects PR #1823's merged scope without overclaiming: the Review History entry points at the on-disk pass-1 review with a verdict-accurate qualifier, the closed sub-item is checked with the correct PR number and a tightly-scoped prose summary that uses the **"Direct"** qualifier and enumerates exactly the three operator surfaces the slice covers (helper/local arithmetic, comparisons, unary negation), the parent INT-1 milestone correctly stays open behind the new sub-item, and the open follow-up bullet correctly *swaps* the now-closed value-semantics gap for the newly-identified pass-1 N2 (assignment-target retype), with a concrete worked example pulled from the pass-1 reproducer. Pass-1 N1 (always-borrow comparison style) and N5 (code-shape polish) are correctly left at the review-file level rather than promoted to tracker bullets — consistent with prior tracker PRs (#1818, #1820, #1822). All four user-facing pass-1 findings (N2 assign target, N3 function boundary, N4 lexical shadowing, plus the carried-forward fallible `//`/`%` and legacy-emission concerns) are tracked. Quick validation reproduces `report_signature=e1bf653aaa770517` — same signature recorded across the milestone — confirming no test or build deltas.

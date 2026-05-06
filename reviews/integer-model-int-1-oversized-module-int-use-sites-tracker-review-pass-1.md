# INT-1 Oversized Module `int` Use Sites Tracker — Review Pass 1

**Verdict:** Satisfied. No blockers.

## Scope reviewed

PR #1820, branch `int-1-use-sites-tracker` (head `c2a4492b`), `main..HEAD` diff:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference docs:
- [reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md](reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md) (the implementation review this tracker change closes; 151 lines, present)
- Merged implementation: PR #1819 (`gh pr view 1819` → `MERGED 2026-05-06T16:24:18Z`, title "Handle oversized int module constant use sites", commit `bce0bdea`)

## Brief vs. diff

The brief asked for three changes. All three are present, accurate, and appropriately scoped.

1. **Add the satisfied INT-1 oversized module int direct-use-site review to Review History.** ✓ — line 397 inserts:

   ```
   - [x] INT-1 oversized module `int` constant direct use-site review satisfied with non-blocking broader migration follow-ups: `reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md`.
   ```

   Path matches the on-disk file. Placement after the prior PR #1817 entry and before the INT-2A rows preserves the chronological/milestone ordering used by the rest of the section. The tail qualifier "with non-blocking broader migration follow-ups" mirrors the pass-1 verdict ("Satisfied with non-blocking suggestions") and signals to a future reader that the closure was conditional on the open follow-up below — an informative variation on the bare "review satisfied" wording used elsewhere, not a deviation from style.

2. **Mark the direct oversized-helper use-site sub-item complete with PR #1819.** ✓ — line 429 flips:

   ```
   - [ ] Wire `int`-typed use sites and arithmetic that reference oversized `SifrInt` module-constant helpers, so expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust during the broader `Type::Int` codegen migration.
   ```

   to

   ```
   - [x] Direct `int`-typed use sites and `+`/`-`/`*` arithmetic that reference oversized `SifrInt` module-constant helpers now coerce participating operands through `SifrInt` and retype receiving local bindings, so expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust; review is satisfied and quick validation is passing: PR #1819.
   ```

   Truthfulness check against the pass-1 implementation review and the on-disk diff at [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs):

   - "**Direct** `int`-typed use sites" — accurate. The rewrite recognizes the helper only when the AST shape exposes the `__const_*()` `FnCall` directly (or after one of the explicitly-handled `BinOp`/`UnaryOp`/`Paren` wrappers); this is exactly what `is_sifr_int_expr` covers. The added "Direct" qualifier correctly narrows the original "Wire `int`-typed use sites" claim to what was actually delivered, leaving chained-`Ident`-resolved-to-SifrInt cases for the open follow-up.
   - "**`+`/`-`/`*` arithmetic**" — accurate. [is_sifr_int_arithmetic_op](crates/sifr_codegen/src/expr_render_helpers.rs:1320) is exactly `matches!(op, "+" | "-" | "*")`. The new qualifier prevents the bullet from over-claiming coverage for `//`, `%`, comparisons, or shifts.
   - "**coerce participating operands through `SifrInt`**" — accurate. [coerce_expr_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1225) strips an outer `Cast { ty: I64 }` and wraps the inner expression in `SifrInt::from_i64`, which is the exact mechanic that turns `BIG_LIMIT + (1 as i64)` into `__const_BIG_LIMIT() + SifrInt::from_i64(1)`.
   - "**retype receiving local bindings**" — accurate. The `RustStmt::Let` arm at [expr_render_helpers.rs:450](crates/sifr_codegen/src/expr_render_helpers.rs:450) rewrites `Some(RustType::I64)` to `Some(RustType::Named("SifrInt"))` whenever the rewritten value is `is_sifr_int_expr`-true.
   - "**expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust**" — accurate, both for the e2e fixture's two new asserts (`BIG_LIMIT + 1` and `oversized_local: int = BIG_LIMIT + LIMIT`) and for the variants probed in the pass-1 review (`-BIG_LIMIT`, `1 + BIG_LIMIT`, `BIG_LIMIT * 2 - 5`, untyped `big = BIG_LIMIT + 1`).
   - "**review is satisfied and quick validation is passing: PR #1819**" — `gh pr view 1819` confirms `MERGED` with title `Handle oversized int module constant use sites`, and `git log` shows commit `bce0bdea` `Handle oversized int module constant use sites` followed by `c2a4492b Track oversized int use-site closure` on this branch. PR number, title, and merge state all match.

   No overclaiming detected. The bullet narrows two important scopes ("Direct …", "`+`/`-`/`*`") that the original placeholder bullet did not, which is the right move.

3. **Preserve the broader-migration gaps as an explicit open INT-1 follow-up bullet.** ✓ — line 430 adds:

   ```
   - [ ] Continue the broader `Type::Int` codegen migration beyond direct helper-touching expressions: chained `SifrInt` locals such as `oversized_local + 2`, comparisons such as `BIG_LIMIT > 100`, fallible `//` and `%`, and function argument/return boundaries still need uniform exact-int lowering instead of legacy `i64`.
   ```

   This bullet enumerates four concrete user-visible failure shapes, each with a worked example. Cross-referenced against the pass-1 review:

   - "chained `SifrInt` locals such as `oversized_local + 2`" — covers pass-1 N1 (the highest-impact remaining gap, where the let-type retype shifts the rustc error one hop downstream). Worded with the canonical example I used in the implementation review.
   - "comparisons such as `BIG_LIMIT > 100`" — covers pass-1 N2 (`is_sifr_int_arithmetic_op` excludes comparison operators).
   - "fallible `//` and `%`" — covers pass-1 N3 (also tied to the absence of `Div`/`Rem` impls on `SifrInt` and the INT-3 fallible-arithmetic milestone, which the bullet implicitly defers to without naming it — fine for tracker-level granularity).
   - "function argument/return boundaries" — covers pass-1 N4 (`fn double(x: i64)` etc., where the broader `Type::Int` ⇒ `SifrInt` migration has not landed at function-signature granularity).

   The bullet correctly does not surface pass-1 N5 (test-suite hardening for `-`/`*`/helper-on-right/unary-`-`/should-not-rewrite assertions) or N6 (O(N) helper-name scan, `coerce_expr_to_sifr_int` invariant assertion, early-return style nit). Those are not user-facing failure shapes — they're code-shape and test-coverage hygiene that the pass-1 review surfaces and that don't need their own milestone-level tracker line. Defensible omission consistent with how prior tracker PRs (e.g. #1818) handled smaller pass-1 nits.

## Cross-checks

- **Top-level INT-1 line stays open.** Line 425 is still `- [ ] INT-1 runtime SifrInt and ownership semantics`, correct because the new follow-up sub-item at line 430 is unchecked. No spurious milestone-level closure.
- **Sub-item ordering inside INT-1.** Wave 1 (426) → Wave 1B (427) → oversized-module-int closure (428) → direct use-site closure (429) → broader-migration follow-up (430). This preserves implementation order: each completed sub-item sits before the open work that depends on it, and the new open bullet is adjacent to its parent slice. Future readers can see at a glance what shipped and what's next.
- **PR linkage.** `gh pr view 1819` returns merged on 2026-05-06; the implementation tip on this branch is `bce0bdea Handle oversized int module constant use sites`, immediately preceded by tracker-only commit `c2a4492b Track oversized int use-site closure`. PR number is consistent.
- **Validation reproduction.** `report_signature=e1bf653aaa770517` is identical to the signatures recorded for PRs #1817, #1818, and #1819. A tracker-only diff should not move the signature, and it doesn't.
- **No collateral churn.** Diff is +3/-1 lines on a single file; no edits to `internal_docs/architecture.md`, `internal_docs/roadmap.md`, design doc, code, tests, or fixtures. AGENTS.md asks for doc updates "as applicable" — no architecture/roadmap claim depends on this closure beyond what the issue tracker already records, so the no-collateral diff is correct.

## Non-blocking observations

None that gate merge.

- The bullet's enumeration of broader-migration gaps groups four distinct user-facing failure shapes into one follow-up bullet. That keeps the tracker compact, but if future waves close them at different times (e.g. function-signature migration likely lands as its own slice separately from comparison rewriting), an editor will need to split this bullet rather than just tick it. Acceptable trade-off — splitting now would make the tracker noisier than it needs to be.
- Pass-1 N5 and N6 (test-coverage hardening and code-shape nits inside the use-site rewrite) remain only in the implementation review. Discoverable via the linked review file, which is what tracker-only PRs typically rely on.

## Verdict

**Satisfied. No blockers.** The diff faithfully reflects the merged implementation: the Review History entry points at the on-disk pass-1 review with a verdict-accurate qualifier, the closed sub-item is checked with the correct PR number and a tightly-scoped prose summary that adds the "Direct" and "`+`/`-`/`*`" qualifiers (avoiding any over-claim against the actual coverage), the parent INT-1 milestone correctly stays open behind the new sub-item, and the new open bullet preserves all four user-facing remaining gaps from pass-1 (N1 chained locals, N2 comparisons, N3 fallible `//`/`%`, N4 function boundaries) with worked examples. Quick validation reproduces the same `report_signature` as the implementation review, confirming the tracker change introduces no test or build deltas.

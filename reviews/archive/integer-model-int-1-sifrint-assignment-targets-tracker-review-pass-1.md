# Review: INT-1 SifrInt Assignment Targets Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +4/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects the merged PR #1825 (`gh pr view 1825` → `MERGED 2026-05-06T17:41:19Z`):

- **Lines 400–401** — both saved review files are recorded with verdict-accurate qualifiers:
  - Pass 1 entry says "completed with value-position alias blocker" — matches the pass-1 verdict ("Changes requested", B1 the bare-Name `b: int = a` regression).
  - Pass 2 entry says "satisfied after addressing the alias blocker" — matches the pass-2 verdict ("Satisfied. No blockers. The pass-1 B1 regression is fully fixed.").
  - Both file paths resolve on disk (296 lines and 212 lines respectively).
  - The dual-entry pattern follows the established convention for INT-1 sub-slices that landed across two review passes (cf. "INT-1 runtime substrate wave 1 review pass 1 completed with blockers" + "pass 2 satisfied after addressing blockers" at lines 393–394).

- **Line 436** — sub-item closure for PR #1825:

  ```
  - [x] Plain local assignment targets that later receive exact-int helper/local values now pre-promote their Rust storage to `SifrInt`, coerce small initializers through `SifrInt::from_i64`, and clone registered exact-int locals in value position so aliases like `b: int = a` and `total = a` preserve source value semantics; review is satisfied and quick validation is passing: PR #1825.
  ```

  Truthfulness checks against the implementation diff (which I reviewed in pass-1 and pass-2):

  - "**Plain** local assignment targets" — load-bearing qualifier. The slice covers `HirStmt::Let` and `HirStmt::Assign` (matched explicitly in the pre-scan visitor at [function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs)) but does *not* cover `HirStmt::AugAssign`. Saying "plain" correctly scopes the closure.
  - "pre-promote their Rust storage to `SifrInt`" — accurate. The new `sifr_int_forced_local_bindings: RefCell<HashSet<String>>` field plus the Let arm retype implement this.
  - "coerce small initializers through `SifrInt::from_i64`" — accurate. Verified in the pass-2 e2e fixture as `let mut assigned_total: SifrInt = SifrInt::from_i64(0);`.
  - "**clone** registered exact-int locals **in value position**" — load-bearing precision. The pass-2 fix introduced `coerce_expr_to_sifr_int_value` whose Ident-registered arm produces `Clone(Ident)` (i.e., `local.clone()`), distinct from the operand-position `Ref(Ident)` (i.e., `&local`). The bullet correctly anchors the value-position semantics.
  - "aliases like `b: int = a` and `total = a` preserve source value semantics" — both shapes verified in pass-2 against the e2e fixture's `alias_reuse: int = reusable_oversized_local` (Let) and `alias_assign = reusable_oversized_local` (Assign), with the trailing `assert str(reusable_oversized_local) == ...` pinning that the source local stays usable.
  - "review is satisfied and quick validation is passing: PR #1825" — `gh pr view 1825` confirms merged 2026-05-06T17:41:19Z.

  No overclaim: the bullet does *not* claim AugAssign coverage, function boundaries, lexical shadowing, legacy-emission paths, or fallible `//`/`%` — all correctly carried forward to the open follow-up.

- **Line 437** — open follow-up bullet correctly *swaps* the closed gap for the new one. The pre-PR text said `assignment targets such as total = total + big still need exact-int target handling`; the post-PR text says `augmented assignment targets such as total += big still need exact-int target handling`. This precisely:
  - Removes the plain-assignment placeholder now closed by #1825.
  - Promotes pass-2's N-pass2-1 (AugAssign unhandled) into the tracker with a worked example (`total += big`).
  - Carries forward "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage", "fallible `//` and `%` still need exact-int runtime/codegen support", and "function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`" verbatim from the prior bullet.

  All four remaining user-facing failure shapes (augmented assignment, scope-safe coverage, fallible `//`/`%`, function boundaries) stay explicit. No required follow-up is missing or ambiguous.

- **Top-level INT-1 line stays `[ ]`** at line 429 — correct, because the new sub-item at line 437 is unchecked. No spurious milestone-level closure.

- **Sub-item ordering** — the new sub-item is appended after the value-semantics closure and before the new open bullet, preserving implementation order: Wave 1 → Wave 1B → oversized-module-int → use-sites-direct → SifrInt-local-comparisons → SifrInt-local-value-semantics → **plain-assignment-targets (this PR)** → broader-migration follow-up.

- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1823 and the pass-1/pass-2 implementation reviews of #1825. Tracker-only diff preserves the signature, as expected.

- **No collateral churn** — diff is +4/-1 lines on a single file. No edits to `internal_docs/architecture.md`, `internal_docs/roadmap.md`, design doc, code, tests, or fixtures. Consistent with prior tracker-only PRs in this milestone (#1818, #1820, #1822, #1824).

## Notes

(Non-blocking observations only.)

- **N1 — `LET` arm carry-forward concision.** The new closing bullet enumerates three mechanisms ("pre-promote", "coerce small initializers through `from_i64`", "clone registered exact-int locals in value position"). This is more mechanism-detail than prior closing bullets in the milestone (cf. #1817's bullet which said "lower through `SifrInt` helper codegen" without spelling out the match cases). The added detail is helpful for diagnostically tracing past a future regression to the right slice, but it does pin the bullet to the current implementation shape — if a future refactor consolidates the value-position helper or replaces `from_i64` with another constructor, this bullet would need updating. Defensible trade-off given the pass-1 B1 incident motivated the precision.

- **N2 — "augmented assignment targets" wording is durable.** The new open follow-up bullet correctly uses the language-neutral "augmented assignment targets" plus the concrete `total += big` example. This wording survives a future renaming/refactoring of the codegen path because it's framed in source-language terms, not Rust-IR terms. Good shape for tracker prose.

- **N3 — Pass-1 N3 (coerce arm-ordering doc comment) and pass-2 N-pass2-2 (same)** remain at the review-file level rather than promoted to the tracker. Consistent with the established pattern of leaving code-shape and documentation polish at the review level (cf. #1818, #1820, #1822, #1824 trackers).

- **N4 — Pass-2 N-pass2-3 (`Clone` chain perf for repeated bare-Name aliases)** is not surfaced in the tracker. This is fair — perf is INT-8 territory, not user-facing correctness. The pass-2 review file documents it for future INT-8 work.

- **N5 — The Review History pattern of "pass 1 with blockers + pass 2 satisfied"** for PR #1825 mirrors the wave-1 entries (lines 393–394). This is the second time in the milestone a slice has needed two passes, and the tracker treats it consistently.

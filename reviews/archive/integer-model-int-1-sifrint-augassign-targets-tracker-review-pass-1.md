# Review: INT-1 SifrInt AugAssign Targets Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +4/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1827 (`gh pr view 1827` → `MERGED 2026-05-06T18:12:23Z`).

### Review History (lines 402–403)

Both saved review files are recorded with verdict-accurate qualifiers:

- **Pass 1** entry says "satisfied with optional test-hardening notes" — matches the pass-1 verdict ("Satisfied", no blockers, with N3/N4 flagged as optional test-coverage gaps).
- **Pass 2** entry says "satisfied after adding focused registered-source and supported-op unit coverage" — precisely describes the two pass-2 additions:
  - `rewrites_sifr_int_augassign_registered_source_to_borrowed_operand` (closes pass-1 N3)
  - `rewrites_sifr_int_augassign_for_supported_ops` (closes pass-1 N4)
- Both file paths resolve on disk (84 + 55 lines).

The dual-entry pattern follows the established convention for INT-1 sub-slices that landed across two passes (cf. INT-1 wave 1 at lines 393–394, INT-1 SifrInt assignment-targets at lines 400–401).

### Sub-item closure (line 439)

```
- [x] Local augmented assignment targets for supported exact-int arithmetic now pre-promote receiving `int` locals to `SifrInt` and rewrite `+=`, `-=`, and `*=` as plain assignments with borrowed exact-int operands, preserving value semantics for shapes like `total += big`; review is satisfied and quick validation is passing: PR #1827.
```

Truthfulness checks against the merged implementation (which I reviewed in pass-1 and pass-2):

- "**Local** augmented assignment targets" — load-bearing qualifier. The slice's rewrite arm at [expr_render_helpers.rs:534-557](crates/sifr_codegen/src/expr_render_helpers.rs:534) only matches `RustExpr::Ident(name)` targets; subscript/field targets fall through unchanged. The pre-scan visitor at [function_emitter.rs:137-148](crates/sifr_codegen/src/function_emitter.rs:137) also only matches `HirStmt::AugAssign { name, … }` for local int bindings.
- "for **supported exact-int arithmetic**" — load-bearing qualifier. Doesn't imply full AugAssign coverage.
- "pre-promote receiving `int` locals to `SifrInt`" — accurate. The new pre-scan arm + the Let arm's `force_sifr_int` retype path implement this.
- "rewrite **`+=`, `-=`, and `*=`**" — explicit op list. `is_sifr_int_augassign_op` matches exactly these three at the HIR level; `is_sifr_int_arithmetic_op` matches `+`/`-`/`*` at the Rust IR level. Doesn't claim `//=`, `%=`, `<<=`, `>>=`, `**=`, `&=`, `|=`, `^=`.
- "as plain assignments with borrowed exact-int operands" — accurate. `total += value` becomes `total = &total + coerce(value)`, where `coerce_expr_to_sifr_int` is the operand-position helper that borrows registered locals (`&source`) or wraps small literals (`SifrInt::from_i64(2)`).
- "preserving value semantics for shapes like `total += big`" — accurate. The `&total` and `&source` borrows preserve Rust ownership across iterations, and the e2e fixture's trailing assert pins that `reusable_oversized_local` stays usable after the AugAssign loop.
- "review is satisfied and quick validation is passing: PR #1827" — `gh pr view 1827` confirms merged 2026-05-06T18:12:23Z.

No overclaim: the bullet refuses to imply coverage of unsupported ops, subscript/field targets, function boundaries, or fallible `//`/`%` semantics.

### Open follow-up (line 440)

Old (from PR #1826's tracker):
> "...augmented assignment targets such as `total += big` still need exact-int target handling, lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, fallible `//` and `%` still need exact-int runtime/codegen support, and function argument/return boundaries still need uniform `SifrInt` lowering..."

New:
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, **unsupported augmented assignment/fallible `//` and `%`** still need exact-int runtime/codegen support, and function argument/return boundaries still need uniform `SifrInt` lowering..."

Diff effects:

- **Removed**: "augmented assignment targets such as `total += big` still need exact-int target handling" — correctly removed because PR #1827 closes it for supported ops. ✓
- **Modified**: "fallible `//` and `%`" → "**unsupported augmented assignment/fallible `//` and `%`**" — captures pass-1 N1 (mixed unsupported AugAssign on a forced SifrInt local emits invalid Rust because the rewrite arm's op-set excludes `/`, `%`, `<<`, `>>`, `**`, etc.). Bundling unsupported-op AugAssigns with fallible `//`/`%` is reasonable because both gaps will likely be unblocked by the same fallible-arithmetic milestone — once `SifrInt` has `FloorDiv`/`Rem` returning `Result`, the codegen can extend the AugAssign rewrite to those ops in the same wave.
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage" and "function argument/return boundaries still need uniform `SifrInt` lowering instead of legacy `i64`".

All four user-facing remaining gaps stay explicit. No required follow-up is missing or ambiguous.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 431 — correct, because the new sub-item at line 440 is unchecked. No spurious milestone-level closure.
- **Sub-item ordering** — Wave 1 → Wave 1B → oversized-module-int → use-sites-direct → SifrInt-local-comparisons → SifrInt-local-value-semantics → plain-assignment-targets → **augmented-assignment-targets (this PR)** → broader-migration follow-up. Implementation order preserved.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1825 and the pass-1/pass-2 implementation reviews of #1827. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +4/-1 lines on a single file. No edits to architecture/roadmap docs, code, tests, or fixtures. Consistent with prior tracker-only PRs in this milestone (#1818, #1820, #1822, #1824, #1826).

## Notes

(Non-blocking observations only.)

- **N1 — Subscript AugAssign isn't surfaced as a separate gap.** Pass-1 N2 noted that `arr[0] += big` (subscript-target AugAssign) is out of scope and stays broken — same shape was broken pre-PR. The new open follow-up bullet doesn't explicitly mention subscript/field AugAssign targets; it implicitly subsumes them under "broader `Type::Int` codegen migration". This is consistent with how prior trackers handled out-of-scope shapes (subscript paths were never explicitly enumerated in the tracker), so I don't think a tracker bullet edit is warranted. But a future closure milestone may want a brief audit listing subscript/field AugAssign as a sibling concern alongside function boundaries.

- **N2 — Pass-2 N-pass2-1 (defensive "should not rewrite" test for unsupported ops)** stays at the review-file level. Code-shape hardening, not user-facing — consistent with prior trackers (#1818, #1820, #1822, #1824, #1826) leaving such polish at the review level.

- **N3 — The "unsupported augmented assignment" wording** is durable. It's framed in source-language terms (matches the integer-model design's vocabulary), so it survives a future refactor of the `is_sifr_int_arithmetic_op`/`is_sifr_int_augassign_op` predicates.

- **N4 — Dual-pass review pattern is now well-established.** This is the third INT-1 sub-slice with two saved review entries (after wave-1 substrate at #1789 and assignment-targets at #1825). The history section keeps both entries chronologically adjacent and consistently described, which makes the slice's progression easy to follow.

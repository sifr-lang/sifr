# Review: INT-2B — Reserved-width (`int128` / `uint128`) shadowing policy documentation

Reviewer: agent
Date: 2026-05-06
Branch: `int-2b-reserved-width-shadowing-policy`
Phase: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), milestone INT-2B (carry-over follow-up at line 439)
Design source of truth: [internal_docs/integer_model.md](internal_docs/integer_model.md)
Prior pass that produced this follow-up: [reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-2b.md](reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-2b.md) (carryover N4)

## Verdict: SATISFIED — ready to merge

This is a docs-only, single-paragraph addition to [internal_docs/integer_model.md](internal_docs/integer_model.md) that takes an explicit stance on whether reserved-width names (`int128` / `uint128`) should be shadowable by user-defined type variables, type aliases, or classes. It correctly describes the current implementation, is consistent with the surrounding integer-model contract, stays inside scope, and closes the pass-2b N4 carryover. No blockers.

---

## What changed

`git diff` shows a single hunk in [internal_docs/integer_model.md:69](internal_docs/integer_model.md:69), inserting one paragraph after the existing reserved-width paragraph at line 67 and before the `bigint` transition paragraph at line 71. The new paragraph:

> The reserved-width diagnostic is reached after ordinary annotation name resolution. Existing Sifr type names are shadowable, so a user-defined type variable, type alias, or class named `int128` or `uint128` resolves to that user definition instead of emitting `SIFR-INT-0003`. INT-2B should keep this general shadowing behavior rather than create a special anti-shadowing rule only for future integer widths. A later language-wide reserved-identifier policy may tighten this consistently across all builtin and reserved names.

`git status` confirms one modified file. No code, test, fixture, registry, schema, generated-doc, or issue-checklist edits in this slice.

---

## Correctness against current implementation

The paragraph claims a precedence: (a) ordinary annotation name resolution runs first, (b) `SIFR-INT-0003` is only emitted if no user definition resolves the name, and (c) the user shadow can come from a type variable, a type alias, or a class.

The implementation in [crates/sifr_hir/src/lower/typing_and_functions.rs:420-447](crates/sifr_hir/src/lower/typing_and_functions.rs:420) implements `resolve_annotation_expr` for `Expr::Name` exactly in this precedence:

1. [line 424](crates/sifr_hir/src/lower/typing_and_functions.rs:424) — `if ctx.type_vars.contains(name.id.as_str())` returns `Type::TypeVar(...)` (matches the doc's "type variable").
2. [line 428](crates/sifr_hir/src/lower/typing_and_functions.rs:428) — `if let Some(alias_ty) = ctx.scope.lookup_type_alias(&name.id)` returns the alias body (matches "type alias").
3. [line 432](crates/sifr_hir/src/lower/typing_and_functions.rs:432) — `if let Some(class_ty) = ctx.class_types.get(name.id.as_str())` returns the class type (matches "class").
4. [line 435-438](crates/sifr_hir/src/lower/typing_and_functions.rs:435) — only after all three of the above does `matches!(name.id.as_str(), "int128" | "uint128")` fire `reserved_integer_width_name`, which emits `DiagnosticCode::INT_RESERVED_WIDTH_NAME` (registry alias of `SIFR-INT-0003`) at [line 412-418](crates/sifr_hir/src/lower/typing_and_functions.rs:412).

The doc paragraph's three shadowing categories enumerate exactly the three pre-existing fallthroughs at lines 424/428/432, and "instead of emitting `SIFR-INT-0003`" exactly describes the early-return shape. The paragraph is faithful to the code.

A small wording observation, non-blocking: the paragraph says "Existing Sifr type names are shadowable" and then specifically calls out `int128`/`uint128`. Strictly, `int128`/`uint128` are reserved future names rather than "existing Sifr type names" — the broader claim is meant to motivate why the same rule applies to reserved names. The implementation does in fact treat any name in `class_types` / `type_vars` / type-alias scope as preempting both reserved-width and built-in-width resolution (the built-in lookup at [line 443](crates/sifr_hir/src/lower/typing_and_functions.rs:443) is even further down), so the broader claim is also accurate. The wording reads as a sentence connector rather than a separate technical claim.

---

## Consistency with the integer model contract

The two paragraphs at [integer_model.md:67-69](internal_docs/integer_model.md:67) now read in sequence:

- Line 67 (existing): "Using either reserved name before support lands must produce `SIFR-INT-0003`, not a generic unresolved-name diagnostic."
- Line 69 (new): the diagnostic is emitted only when ordinary name resolution does not bind a user definition; user shadows resolve to the user definition.

These do not contradict. The first paragraph distinguishes `SIFR-INT-0003` from `NAME_UNKNOWN_TYPE` for the *unbound-name* case; the second paragraph clarifies behavior for the *bound-name* case (a user-defined symbol with the same identifier). The rest of the integer-model contract — the diagnostic-family table at line 456, the validation matrix entry at line 524, and the reserved-width text at line 67 — does not need follow-up edits to remain consistent with the new paragraph.

The paragraph's forward-looking stance ("A later language-wide reserved-identifier policy may tighten this consistently across all builtin and reserved names") is appropriately tentative — it does not commit the project to a specific design and does not create an unmet obligation in this milestone.

---

## Closure of pass-2b carryover N4

Pass-2b N4 (carried into INT-2B) said:

> Reserved-name check is shadowable by user-defined `class int128` / `type int128 = …`. Intentional given existing scaffolding; INT-2B's "no user-facing `bigint`" cleanup is the natural place to take a stance.

The new paragraph takes that stance explicitly and on the record. It (a) names the behavior, (b) declares the intended policy ("INT-2B should keep this general shadowing behavior rather than create a special anti-shadowing rule"), (c) names the future escape hatch ("A later language-wide reserved-identifier policy may tighten this consistently"). This closes N4 as a documentation question. The corresponding open issue checklist item at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:439](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:439) explicitly lists this carry-over ("decide reserved-name shadowing policy during `bigint` cleanup"); the policy is now decided and recorded.

---

## Scope discipline

- One file modified, one paragraph added. No collateral edits.
- No code, test, registry, fixture, generated-doc, or schema changes.
- No retroactive edits to surrounding paragraphs that could conflate this slice with other work.
- No HIR maintainability guardrail surfaces touched.
- The paragraph stays inside the "Source Types" section where the reserved-width text already lives, rather than spreading the policy text across `Diagnostics` or `Compiler Architecture Impact`.

---

## Validation gates

This is a docs-only insertion in plain prose inside an existing markdown file. The pass-2b precedent applies: the diff lands outside any path scanned by the Rust workspace, the diagnostic-doc generator, the schema/registry coverage scripts, the HIR maintainability guardrail (which only inspects `crates/sifr_hir/src/lower/`), and the snapshot suites. `git diff --check` is clean (no whitespace or merge-marker drift would be expected from a single-paragraph insertion). Per the prior pass-2b reasoning, re-running `scripts/run_all_tests.sh --profile quick` is not load-bearing for a paragraph-only edit; the user should still run it as the gate before opening the PR per `AGENTS.md`.

---

## Non-blocking observations (not required for merge)

These are flagged for awareness during the matching PR; none warrant blocking.

- **O1 — Issue checklist not yet ticked.** [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:439](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:439) ("Carry remaining follow-ups from INT-2A/INT-2B reviews: decide reserved-name shadowing policy during `bigint` cleanup ...") is the umbrella line that this paragraph partly satisfies. It still has an open checkbox and is shared with another follow-up ("clean up fixed-width diagnostic formatting/fallback paths"). The issue diff is intentionally not part of this slice; the typical project pattern (see PR #1791..#1808 entries above it) is to add a per-slice checkbox listing the merged PR. Suggest either (a) ticking line 439 only when both umbrella sub-items land, or (b) splitting line 439 into two sub-items so this slice can mark its own done. Either is fine for the issue tracker; not a doc-correctness concern.

- **O2 — Present-tense scope framing.** "INT-2B should keep this general shadowing behavior" reads as a within-milestone normative statement. Once INT-2B closes, the sentence will historically read as a milestone-bound recommendation rather than a stable policy. Optional reword: "Sifr keeps the general shadowing behavior here rather than introducing a special anti-shadowing rule for future integer widths." This is purely cosmetic and not in scope for this slice.

- **O3 — No regression test for the shadow case.** Pass-2b N1 already noted that `test_reserved_integer_width_annotations_have_int_code` doesn't lock the shadow path. With the policy now committed in writing, the natural complement is a positive test that `class int128:` (or a `type int128 = …` alias, or a `def f[int128](...)` type parameter) preempts `SIFR-INT-0003`. This is a code-side follow-up belonging to a later slice; this docs-only PR is not the place to add it.

- **O4 — No anchor for the "later language-wide reserved-identifier policy."** The trailing sentence references a future policy without a tracking issue, milestone, or section pointer. Soft suggestion: when such a slice is opened, link it back here (or surface it in `internal_docs/architecture.md` / a new `internal_docs/reserved_identifiers.md`). Not in scope now.

- **O5 — `bigint` parallel not asserted.** Within `resolve_annotation_expr`, the `bigint` warning at [line 439](crates/sifr_hir/src/lower/typing_and_functions.rs:439) sits *after* the same three shadow checks, so user-defined `class bigint:` (etc.) would similarly preempt `SIFR-INT-0011`. The new paragraph deliberately scopes itself to `int128`/`uint128`. This is correct framing for the "Source Types > reserved widths" subsection, but if a future slice consolidates the language-wide reserved-identifier policy, the same precedence story for `bigint` should be made explicit there. Not required here.

---

## Final verdict

**SATISFIED — merge.** The paragraph is correct against the implementation at [crates/sifr_hir/src/lower/typing_and_functions.rs:420-447](crates/sifr_hir/src/lower/typing_and_functions.rs:420), consistent with the surrounding integer-model contract, scoped to a single design-doc edit, and closes the pass-2b N4 follow-up by recording the explicit shadowing-policy decision. Observations O1–O5 are non-blocking and tracked above for a future slice.

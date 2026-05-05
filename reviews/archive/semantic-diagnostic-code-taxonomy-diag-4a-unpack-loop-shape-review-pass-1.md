# `milestone_diag_4a` slice 2b.5 — for-loop tuple destructuring & star-unpack list-shape migration — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-unpack-loop-shape`
- Target: migrate the three remaining HIR shape-mismatch emission sites called out as residual risks #1 and #2 in the slice 2b.4 review pass 2 onto the active `SIFR-TYPE-0009` (`TYPE_UNPACK_SHAPE_MISMATCH`) code, while leaving message text and surrounding control flow untouched, and pin each migrated site with an e2e fail fixture.
- Sites migrated:
  - For-loop tuple target arity mismatch — [crates/sifr_hir/src/lower/statements.rs:2105](../crates/sifr_hir/src/lower/statements.rs:2105)
  - For-loop tuple target non-tuple element — [crates/sifr_hir/src/lower/statements.rs:2121](../crates/sifr_hir/src/lower/statements.rs:2121)
  - Star-unpack RHS not a list — [crates/sifr_hir/src/lower/tuple_unpack.rs:169](../crates/sifr_hir/src/lower/tuple_unpack.rs:169)
- New fixtures:
  - [crates/sifr/tests/e2e/fail/for_loop_tuple_target_arity_mismatch.sifr](../crates/sifr/tests/e2e/fail/for_loop_tuple_target_arity_mismatch.sifr)
  - [crates/sifr/tests/e2e/fail/for_loop_tuple_target_non_tuple_element.sifr](../crates/sifr/tests/e2e/fail/for_loop_tuple_target_non_tuple_element.sifr)
  - [crates/sifr/tests/e2e/fail/star_unpack_requires_list_type.sifr](../crates/sifr/tests/e2e/fail/star_unpack_requires_list_type.sifr)
- Issue checklist update: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- Validation already executed by the implementer: `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` (report_signature `e1bf653aaa770517`, wall_time 70.76s).

## Findings

### F1 — Taxonomy fit for `SIFR-TYPE-0009` is correct for all three migrated sites

The active registry entry for `SIFR-TYPE-0009` (`TYPE_UNPACK_SHAPE_MISMATCH`) at [crates/sifr_diagnostics/src/codes.rs:36](../crates/sifr_diagnostics/src/codes.rs:36) and [crates/sifr_diagnostics/src/codes.rs:647](../crates/sifr_diagnostics/src/codes.rs:647) describes "Tuple or list unpacking shape mismatch." The three migrated emissions all fall cleanly inside that semantic envelope:

- For-loop tuple target arity mismatch is a tuple-unpack-shape error in disguise: the iterator yields N-tuples and the loop target binds M names, with N ≠ M. This is structurally the same shape as the slice 2b.4 site at [tuple_unpack.rs:64](../crates/sifr_hir/src/lower/tuple_unpack.rs:64) ("tuple unpacking: expected … values, got …").
- For-loop tuple target non-tuple element is the for-loop analogue of the slice 2b.4 site at [tuple_unpack.rs:76](../crates/sifr_hir/src/lower/tuple_unpack.rs:76) ("cannot unpack non-tuple type"): the LHS is a tuple pattern but the iterator's element type is a scalar, so the tuple-shape unpack cannot apply.
- Star-unpack non-list is the RHS-shape error for the `a, *b = expr` family: the spread destructuring pattern requires a list value, and any other value type is a shape mismatch.

No site was promoted to a code outside its semantic family, and `SIFR-TYPE-0009` is now consistently the carrier for unpack/shape diagnostics across both the assignment-statement path and the for-loop iterator path.

### F2 — Five active `SIFR-TYPE-0009` emission sites accounted for

`grep` for `DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH` and `SIFR-TYPE-0009` enumerates exactly the expected surface area after this slice:

- [tuple_unpack.rs:65](../crates/sifr_hir/src/lower/tuple_unpack.rs:65) — tuple-unpack arity mismatch (slice 2b.4)
- [tuple_unpack.rs:77](../crates/sifr_hir/src/lower/tuple_unpack.rs:77) — non-tuple unpack (slice 2b.4)
- [tuple_unpack.rs:170](../crates/sifr_hir/src/lower/tuple_unpack.rs:170) — star-unpack non-list (this slice)
- [statements.rs:2106](../crates/sifr_hir/src/lower/statements.rs:2106) — for-loop tuple target arity (this slice)
- [statements.rs:2122](../crates/sifr_hir/src/lower/statements.rs:2122) — for-loop tuple target non-tuple element (this slice)

Each is pinned by a representative fail fixture: `tuple_unpack_shape_mismatch.sifr`, `tuple_unpack_non_tuple_shape_mismatch.sifr`, plus the three new fixtures from this slice. No `SIFR-TYPE-0009` site is now unpinned.

### F3 — Out-of-scope shape-adjacent sites correctly left on the legacy bridge

The diagnostics that were intentionally not migrated remain plain `ctx.error(…)` calls (no code attached, so they continue to flow through `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` at [crates/sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)):

- [statements.rs:2083](../crates/sifr_hir/src/lower/statements.rs:2083) — "for loop tuple target must contain only simple names" (target-form syntax error, not shape).
- [statements.rs:2089](../crates/sifr_hir/src/lower/statements.rs:2089) — "for loop target must be a simple name or tuple" (target-form syntax error).
- [tuple_unpack.rs:184](../crates/sifr_hir/src/lower/tuple_unpack.rs:184) — "multiple starred expressions in assignment" (non-shape star-unpack syntax).
- [tuple_unpack.rs:193](../crates/sifr_hir/src/lower/tuple_unpack.rs:193) — "starred target must be a simple name" (non-shape star-unpack syntax).
- [tuple_unpack.rs:207](../crates/sifr_hir/src/lower/tuple_unpack.rs:207) — "star unpacking target must be a simple name" (non-shape star-unpack syntax).
- [tuple_unpack.rs:214](../crates/sifr_hir/src/lower/tuple_unpack.rs:214) — "star unpacking requires a starred expression" (non-shape star-unpack syntax).

These all match the user-stated out-of-scope categories ("non-shape star-unpack syntax errors" and target-form syntax errors). The slice's intent of leaving "form" diagnostics for follow-up while migrating "shape" diagnostics holds.

### F4 — Message text preserved verbatim; existing message-substring tests unaffected

All three migrated emissions changed the call from `ctx.error(<msg>)` to `ctx.error_with_code(DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH, <msg>)` with the same string interpolations. The HIR unit test [`test_for_tuple_target_requires_tuple_elements`](../crates/sifr_hir/src/lower/expressions_tests.rs:1744) at [crates/sifr_hir/src/lower/expressions_tests.rs:1744](../crates/sifr_hir/src/lower/expressions_tests.rs:1744) asserts the substring `"for loop tuple target expects iterable elements of tuple type"` and continues to match. The reported `cargo test -p sifr_hir diagnostic_transport_tests` and `cargo test -p sifr -- --skip test_e2e_pass` results corroborate that message contracts are intact.

### F5 — Each migrated site is pinned by an e2e fixture under the harness's joint code+substring contract

The e2e fail harness at [crates/sifr/tests/e2e.rs:2561](../crates/sifr/tests/e2e.rs:2561) requires both `failure.code == expected.code` and (when a message is provided) `failure.message.contains(expected.message_contains)`. The three new fixtures supply both halves:

- [for_loop_tuple_target_arity_mismatch.sifr](../crates/sifr/tests/e2e/fail/for_loop_tuple_target_arity_mismatch.sifr) sets up `list[tuple[int, int, int]]` and binds `for left, right in triples`, which forces the path at [statements.rs:2104](../crates/sifr_hir/src/lower/statements.rs:2104) (`elem_types.len() != names.len()`) and emits `"for loop tuple target expects 2 element(s), iterable yields 3"`. The fixture's expectation `# expect-error: SIFR-TYPE-0009: for loop tuple target expects 2 element(s), iterable yields 3` matches both code and message.
- [for_loop_tuple_target_non_tuple_element.sifr](../crates/sifr/tests/e2e/fail/for_loop_tuple_target_non_tuple_element.sifr) sets up `list[int]` and binds `for left, right in values`, which lands in the else arm at [statements.rs:2120](../crates/sifr_hir/src/lower/statements.rs:2120) and emits `"for loop tuple target expects iterable elements of tuple type, got 'int'"`. Expectation matches.
- [star_unpack_requires_list_type.sifr](../crates/sifr/tests/e2e/fail/star_unpack_requires_list_type.sifr) uses `first, *rest = (1, 2, 3)`, which dispatches via [statements.rs:1345](../crates/sifr_hir/src/lower/statements.rs:1345) into `lower_star_unpack_assign`; with a tuple RHS the `Type::List` match at [tuple_unpack.rs:166](../crates/sifr_hir/src/lower/tuple_unpack.rs:166) fails and the migrated `error_with_code` fires. Expectation matches.

The plumbing also lines up at the bridge: `lowering_error_to_compile_error` at [crates/sifr_driver/src/frontend/module_lowering.rs:47](../crates/sifr_driver/src/frontend/module_lowering.rs:47) prefers the attached code over the legacy `TypeCheck` default, so the harness sees `SIFR-TYPE-0009`, not `SIFR-TYPE-0001`.

### F6 — Diff is tightly scoped; nothing unrelated touched

The branch's `git diff` is limited to the three call-site swaps in HIR lowering plus an issue-checklist update. No registry edits, no bridge edits, no renderer edits, no neighboring control-flow rewrites. The slice respects the "no unrelated cleanup" guidance.

### F7 — Issue checklist correctly reflects slice transition

The issue file marks slice 2b.4 as merged with PR #1676 and adds a "Started slice 2b.5" line. This matches the branch's commit history and slice 2b.4's pass-2 review conclusion.

## Residual risks

### R1 — `SIFR-TYPE-0009` `message_template` still drifts from the emitted text (carried over from slice 2b.4 residual #4)

The registry entry at [crates/sifr_diagnostics/src/codes.rs:653](../crates/sifr_diagnostics/src/codes.rs:653) declares `"cannot unpack {actual_count} value(s) into {expected_count} target(s)"` with placeholders `actual_count` and `expected_count`. None of the five emission sites for `SIFR-TYPE-0009` actually render that template; they each interpolate their own ad-hoc strings (`"tuple unpacking: expected …"`, `"cannot unpack non-tuple type …"`, `"for loop tuple target expects …"`, `"star unpacking requires a list type"`). Slice 2b.5 does not introduce this drift but does broaden it from two emissions to five. Reconciling the abstract template against rendered text remains for a registry-hygiene slice and is acknowledged as out of scope here.

### R2 — Out-of-scope shape-adjacent star-unpack and for-loop "form" diagnostics still on the legacy bridge

The six form/syntax-shape sites listed in F3 still emit code-less errors, which the bridge surfaces as `SIFR-TYPE-0001`. They are correctly excluded from this slice, but they will need their own framing decision (own family, or fold into `SIFR-TYPE-0009`'s shape umbrella) before the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge can be deleted in a later slice.

### R3 — Slice 2b.5 does not address the `tuple_dynamic_list_shape.sifr` registry-fixture pointer

The active entry for `SIFR-TYPE-0009` advertises `crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr` as its representative fixture, but that fixture pins `SIFR-TYPE-0001` (the `tuple()` constructor diagnostic). Out of scope for this slice per the user's instruction set, but worth carrying forward: the registry's own canonical-fixture pointer is currently mis-aimed for `SIFR-TYPE-0009` even though five fail fixtures now genuinely exercise the code.

### R4 — No new HIR-level unit test added for the migrated sites

The slice relies on (a) the preserved-message contract verified by the existing `test_for_tuple_target_requires_tuple_elements` HIR test and (b) the new e2e fail fixtures for joint code+substring coverage. There is no HIR unit test that asserts `LoweringError::code == Some(TYPE_UNPACK_SHAPE_MISMATCH)` for these three sites, so a future change that accidentally swaps `error_with_code` back to `error` on one of them would only be caught by the e2e harness, not the faster HIR test pass. Slice 2b.4 used the same approach, so this is consistent rather than a regression, but it remains a minor coverage gap that a follow-up slice (or the eventual bridge-deletion slice) could close cheaply.

## Verdict

Satisfied / no blocking findings. The slice closes residual risks #1 and #2 from slice 2b.4 review pass 2: the three remaining unpack/shape sites — `statements.rs:2105`, `statements.rs:2118`, and `tuple_unpack.rs:169` — now flow `SIFR-TYPE-0009` end-to-end via `LowerCtx::error_with_code` while preserving their message text verbatim, and each is pinned by a new e2e fail fixture using the harness's joint code+substring contract. Taxonomy fit is correct, scope discipline is clean (out-of-scope syntax/form sites and the legacy bridge are untouched), and the diff is minimal. Carry-over residuals R1 (registry `message_template` drift), R2 (form-style star-unpack and for-loop diagnostics still on the bridge), R3 (`SIFR-TYPE-0009` registry fixture pointer), and R4 (no HIR-level code assertion) are non-blocking and appropriate for follow-up slices.

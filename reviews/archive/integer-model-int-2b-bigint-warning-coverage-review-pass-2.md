# INT-2B Bigint Warning Coverage — Review Pass 2

Branch: `int-2b-bigint-warning-coverage`
Scope reviewed: incremental changes since pass 1, in response to the requested changes in [reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-1.md](reviews/integer-model-int-2b-bigint-warning-coverage-review-pass-1.md).

Files in delta:

- `crates/sifr_hir/src/lower/classes.rs` — class type-param block now gated by `if !validate_iteration_protocols`.
- `crates/sifr_hir/src/lower/typevar_annotations.rs` — unchanged from pass 1 (helper + 6 emit sites already in place).
- `crates/sifr_hir/src/lower/builtin_calls.rs` — unchanged from pass 1 (`isinstance` emit site already in place).
- `crates/sifr_driver/src/tests/single_file_frontend.rs` — five new tests covering the four previously untested TypeVar branches plus the class PEP 695 single-emission contract.

## Verification of pass-1 requested changes

### Requested change 1 — `class C[T: bigint]` single-emission contract

**Status: Resolved.**

[classes.rs:258-287](crates/sifr_hir/src/lower/classes.rs:258) now wraps the entire PEP 695 class type-param block (type-var registration, bound parsing, `class_declared_type_params` insertion) in `if !validate_iteration_protocols { … }`. Because `collect_class_type` is invoked from [mod.rs:587](crates/sifr_hir/src/lower/mod.rs:587) with `false` and from [mod.rs:597](crates/sifr_hir/src/lower/mod.rs:597) with `true`, the bound is now parsed exactly once per class definition.

I traced the dependent state to confirm the gating is safe across passes:

- `ctx.type_vars` is a `HashSet` populated only by inserts; the first-pass insert is observable in pass 2 even though pass 2 no longer re-inserts. Annotations like `value: T` inside the class body still resolve correctly in pass 2.
- `ctx.class_declared_type_params` is read at [classes.rs:727](crates/sifr_hir/src/lower/classes.rs:727) (after the class-body loop) in *both* passes; the first-pass entry persists to the second pass.
- `ctx.type_param_bounds` is the storage for parsed bound specs and is also populated only in pass 1 now. Bounds resolve from the literal AST shape (`Expr::Name`, `Expr::Tuple` of names) and do not depend on alias resolution that occurs between the two passes — so first-pass-only parsing is semantically equivalent to the previous double-parse.

The accompanying test [single_file_frontend.rs:363-370](crates/sifr_driver/src/tests/single_file_frontend.rs:363) (`test_type_check_source_warns_once_for_bigint_class_pep695_bound`) routes `class Box[T: bigint]:\n    value: T\n` through `type_check_source` and asserts `diagnostics.len() == 1`, locking in the single-emission invariant at the HIR layer (i.e., before CLI dedup). This is exactly the missing test pass 1 called out, and it will fail without the guard.

### Requested change 2 — Test the four untested emit branches

**Status: Resolved.**

| Emit site | File:line | Test |
| --- | --- | --- |
| PEP 695 simple-name bound | typevar_annotations.rs:36 | `test_type_check_source_warns_for_bigint_pep695_bound` ✓ (unchanged) |
| PEP 695 tuple constraint | typevar_annotations.rs:43 | `test_type_check_source_warns_for_bigint_pep695_tuple_constraint` ✓ (new) |
| `TypeVar("T", bigint)` positional | typevar_annotations.rs:82 | `test_type_check_source_warns_for_bigint_typevar_constraint` ✓ (unchanged) |
| `TypeVar("T", bound=bigint)` keyword | typevar_annotations.rs:110 | `test_type_check_source_warns_for_bigint_typevar_bound_keyword` ✓ (new) |
| `TypeVar("T", constraints=(bigint, …))` tuple | typevar_annotations.rs:134 | `test_type_check_source_warns_for_bigint_typevar_constraints_tuple_keyword` ✓ (new) |
| `TypeVar("T", constraints=bigint)` single name | typevar_annotations.rs:146 | `test_type_check_source_warns_for_bigint_typevar_constraints_name_keyword` ✓ (new) |
| `lower_isinstance_call` second arg | builtin_calls.rs:931 | `test_type_check_source_warns_for_bigint_isinstance_target` ✓ (unchanged) |
| Class PEP 695 simple-name bound | typevar_annotations.rs:36 (via classes.rs:270) | `test_type_check_source_warns_once_for_bigint_class_pep695_bound` ✓ (new) |

All seven structurally-distinct emit sites that touch a literal `bigint` token are now exercised, plus the class PEP 695 single-emission contract.

To avoid duplication of the assertion boilerplate, the tests share a helper `assert_single_bigint_transition_warning` at [single_file_frontend.rs:284-307](crates/sifr_driver/src/tests/single_file_frontend.rs:284) that asserts:

- exactly one diagnostic,
- code `INT_BIGINT_TRANSITION_ALIAS`,
- severity `Warning`,
- a primary span on file `main`, the expected line, and `byte_end > byte_start`,
- a context string for failure attribution.

Each new test is a one-line call into the helper. The asserted line numbers (`Some(1)` for inline PEP 695 syntax, `Some(3)` for the legacy declaration form) point at the line containing the `bigint` token, which is the user-helpful span. The shared helper does not duplicate the more rigorous assertions in `test_type_check_source_surfaces_bigint_transition_warning` (message-template equality, `args.is_empty()`), but those invariants are implicitly covered by the parent-commit test, so the slim helper is acceptable for the new branches.

### Requested change 3 — Phase tracker per-slice line

**Status: Not addressed.** [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:420-426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:420) still does not have a per-slice bullet for this slice analogous to [line 423](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:423) (which references the prior `bigint`-annotation slice's review and PR). The bundled follow-up at [line 426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426) still lists "add `SIFR-INT-0011` coverage for silent `bigint` mentions in `isinstance` and TypeVar bounds if they remain before removal" without any acknowledgement that this slice closes that bullet.

Pass 1 explicitly classified this as "Not a code blocker; a documentation/hygiene gap." I retain that classification — the PR-opening step is the natural place to add the per-slice line and the eventual PR number, and gating the verdict on it would block on a paperwork item that the slice author can resolve without further code review. **Non-blocking, but please add the line before opening the PR.**

### Requested change 4 — Run `scripts/run_all_tests.sh --profile quick`

**Status: Resolved.** Reported by the user in this review's invocation: `report_signature=e1bf653aaa770517 wall_time=67.26s`. Plus the targeted runs (`cargo fmt`/`check`, `cargo test -p sifr_driver bigint`, `cargo test -p sifr_hir typevar`, `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings`) all clean per the same report.

## Recheck of pass-1 non-blocking concerns

- **Concern 3** (`detect_narrowing_condition` re-resolves the literal `bigint` silently at [statements.rs:1832](crates/sifr_hir/src/lower/statements.rs:1832)): unchanged, still non-blocking. The user-visible span is already counted by the `lower_isinstance_call` emit, so the narrowing path is silent only at the *type-resolution* layer, not at the user-visible diagnostic count. Carries forward to the eventual centralized-resolver consolidation.
- **Concern 4** (`bigint(1)` constructor at [expressions.rs:856](crates/sifr_hir/src/lower/expressions.rs:856) remains silent): out of scope for this slice by design.
- **Concern 5** (phase tracker hygiene): see Requested change 3 above.
- **Concern 6** (registry's representative-fixture pointer at [codes.rs:786](crates/sifr_diagnostics/src/codes.rs:786) and [internal_docs/diagnostic_codes.md:95](internal_docs/diagnostic_codes.md:95)): no action required by the registry contract.
- **Concern 7** (no `.sifr` demo / e2e fixture): not warranted for a diagnostics-only slice.

## What is correct in the new delta

1. **Single-pass guard is at the right granularity.** Wrapping the entire type-params block (not just the bound-parse helper call) means `class_declared_type_params` is also inserted only once. Inserting twice was already idempotent (`HashMap::insert` overwrites with the same value), so this is a cleanup rather than a fix — but it is correct.
2. **The guard predicate is the right condition.** `validate_iteration_protocols` is `false` exactly on the first call site and `true` exactly on the second; so `if !validate_iteration_protocols` reads as "only run during pass 1," which matches the comment at [classes.rs:259-260](crates/sifr_hir/src/lower/classes.rs:259) and the actual scheduler at [mod.rs:585-599](crates/sifr_hir/src/lower/mod.rs:585).
3. **Test helper trades verbosity for failure attribution.** The `context` argument threaded into every `assert_eq!` makes a failing test's output identify the specific TypeVar branch, not just the line number of the assertion. This is a small but real maintainability win.
4. **No new lints, no new warnings, no new format diffs, no panic risks.** The delta is entirely diagnostic plumbing and test code.
5. **The parent-commit test (`test_type_check_source_surfaces_bigint_transition_warning`) still asserts `diagnostics.len() == 1`** for the annotation path, so its stronger message-template/args assertions still anchor the canonical fixture. The new helper-driven tests intentionally are not the canonical fixture.
6. **No regressions to non-`bigint` paths.** All new emit sites are guarded by `name == "bigint"`. `int`, `float`, user classes, and user-defined type aliases all bypass the helper.

## Concerns and gaps

None blocking. The two pass-1 verdict-blocking concerns (class double-emit; four untested branches) are fully resolved, and every reachable emit site that maps a literal `bigint` token to a TypeVar bound/constraint or `isinstance` target now has at least one regression test. The class single-emission contract is enforced at the HIR layer, not just at CLI dedup.

The remaining pass-1 items are non-blocking and either tracked elsewhere (concerns 3, 4, 6, 7) or outside the code-review surface (concern 5 / requested change 3, the phase-tracker line).

## Verdict-blocking summary

- Concern 1 from pass 1 (class PEP 695 double-emit): **Resolved** via the single-pass guard at [classes.rs:261](crates/sifr_hir/src/lower/classes.rs:261), with an explicit `diagnostics.len() == 1` test at [single_file_frontend.rs:363](crates/sifr_driver/src/tests/single_file_frontend.rs:363).
- Concern 2 from pass 1 (four untested emit branches): **Resolved** via four new tests at [single_file_frontend.rs:318-343](crates/sifr_driver/src/tests/single_file_frontend.rs:318) and [single_file_frontend.rs:354-361](crates/sifr_driver/src/tests/single_file_frontend.rs:354).

Validation reported in the invocation:

- `cargo fmt`/`cargo check` clean.
- `cargo test -p sifr_driver bigint` passing.
- `cargo test -p sifr_hir typevar` passing.
- `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` clean.
- `scripts/run_all_tests.sh --profile quick` passing (`report_signature=e1bf653aaa770517`, `wall_time=67.26s`).

## Follow-ups (non-blocking, for the PR-opening step)

1. Add a per-slice bullet under [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:420](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:420) referencing this review and the eventual PR number, consistent with [line 423](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:423). The bundled follow-up at [line 426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426) can then drop the `isinstance`/TypeVar-bound clause (or leave it scoped to the eventual `bigint` removal).
2. The `bigint(...)` constructor at [expressions.rs:856](crates/sifr_hir/src/lower/expressions.rs:856) remains the strongest "silent bigint mention" in the language. If the milestone tracker doesn't already enumerate it as a pre-removal cleanup target, it's worth a single-line entry so it isn't forgotten.

VERDICT: SATISFIED

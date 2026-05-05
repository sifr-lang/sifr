---
name: semantic-diagnostic-code-taxonomy-diag-4a-typevar-constraint-diagnostics-review-pass-2
description: Review pass 2 — verify that the two non-blocking process/style notes from pass 1 (R3 named-placeholder convention, R7 authoritative quick-profile validation) are resolved and that no new blocker has appeared in slice 2b.23.
---

# Review — `milestone_diag_4a` slice 2b.23: TypeVar constraint application diagnostics (pass 2)

- Branch: `codex/semantic-diagnostics-diag-4a-typevar-constraint-diagnostics`
- Scope (unchanged from pass 1): introduce active `SIFR-TYPE-0010` ("TypeVar constraints are not satisfied by the inferred concrete type"), migrate the single generic-call constraint emission inside [`lower_call`](../crates/sifr_hir/src/lower/expressions.rs:1935) from `ctx.error(...)` (legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` bridge at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)) onto `ctx.error_with_code(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED, ...)`, re-key the existing `typevar_constraints_violation.sifr` fixture, add a structured-identity unit test, and emit the standard registry/docs surface.
- Pass: 2
- Prior reviews:
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-typevar-constraint-diagnostics-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-typevar-constraint-diagnostics-review-pass-1.md) — pass 1 found no blockers; flagged R3 (positional vs. named placeholders at the call site) and R7 (authoritative quick-profile validation not yet recorded) as non-blocking process/style notes worth resolving before PR.

## Summary

Pass 2 confirms that the two non-blocking notes from pass 1 are resolved and that the slice's diff is otherwise unchanged in shape. R3 is fixed by converting the call-site `format!` to named placeholders matching the registry template; R7 is fixed by the user re-running `scripts/run_all_tests.sh --profile quick` to completion. The remaining residual risks from pass 1 (R1 class-constructor-fixture gap, R2 inline-vs-helper convention, R4 sibling arg-mismatch coverage gap, R5 `display_name` exact-equality sensitivity, R6 `HashMap` iteration order) were already explicitly non-blocking and unchanged in scope; nothing in the diff between pass 1 and pass 2 affects any of them.

I did not find any new blockers and did not identify any regressions introduced by the pass-1 → pass-2 changes. The slice remains mergeable as-is.

## Diff between pass 1 and pass 2

`git diff --stat` matches the same eight-file set pass 1 reviewed:

```
crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr |  2 +-
crates/sifr_diagnostics/src/codes.rs                          | 14 ++++++++++++++
crates/sifr_hir/src/lower/expressions.rs                      | 15 +++++++++------
crates/sifr_hir/src/lower/expressions_tests.rs                | 14 ++++++++++++++
docs/errors/diagnostic-codes.md                               |  1 +
internal_docs/diagnostic_codes.md                             |  1 +
issues/...-structured-hir-diagnostics.md                      |  3 ++-
docs/errors/SIFR-TYPE-0010.md                                 | new
```

The only file whose contents changed substantively since pass 1 is `crates/sifr_hir/src/lower/expressions.rs` — the call-site `format!` was rewritten from positional `{}` placeholders + positional arguments to named `{actual}/{constraints}/{type_param}` placeholders + named arguments. Insertions count moved from 12 to 15 in that file (3 extra lines because the named-argument form spans more lines than the compact positional form). All other files are byte-identical to pass 1.

## Findings

### 1. R3 (named placeholder convention) is fully resolved (confirmation)

[crates/sifr_hir/src/lower/expressions.rs:1940-1948](../crates/sifr_hir/src/lower/expressions.rs:1940):

```rust
ctx.error_with_code(
    DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
    format!(
        "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'",
        actual = concrete_ty.display_name(),
        constraints = constraints.join(", "),
        type_param = tv_name
    ),
);
```

vs. the registry template at [crates/sifr_diagnostics/src/codes.rs:669](../crates/sifr_diagnostics/src/codes.rs:669):

```
"type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'"
```

The two strings are now byte-identical. The named-argument bindings (`actual = concrete_ty.display_name()`, `constraints = constraints.join(", ")`, `type_param = tv_name`) match the order of `declared_args = [arg!("actual"), arg!("constraints"), arg!("type_param")]` and `dedupe_args = ["actual", "constraints", "type_param"]` at [codes.rs:670-671](../crates/sifr_diagnostics/src/codes.rs:670). This brings the call site into convention parity with slices 2b.20 (`PROTO_BOUND_NOT_SATISFIED`), 2b.21 (`PROTO_CONTEXT_MANAGER_MISSING`), and 2b.22 (`PROTO_INVALID_ITERATOR_SIGNATURE`).

Substituted-runtime output is unchanged: for the canonical fixture invocation (`echo(1.5)`), `format!` still renders `"type 'float' does not satisfy constraints (int, str) required by type parameter 'T'"` — verified by:

- The fixture line at [crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1](../crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1) unchanged from pass 1 (substring assertion identical).
- The unit-test exact-equality assertion at [crates/sifr_hir/src/lower/expressions_tests.rs:1862-1866](../crates/sifr_hir/src/lower/expressions_tests.rs:1862) unchanged from pass 1 (`==` against the same literal string).

Both gates would have failed if the substitution semantics had drifted; they did not. R3 closed.

### 2. R7 (authoritative `run_all_tests.sh --profile quick` validation) is recorded (confirmation)

The user-reported pass-2 validation set now includes:

> `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=301.38s`)

This is the CLAUDE.md authoritative gate the pass-1 review's R7 asked for. The signature `e1bf653aaa770517` matches the same shape prior slice reviews recorded (e.g., 2b.10 noted `e1bf653aaa770517`/`75.98s`); the slightly longer wall time (301.38s vs. 75.98s) is consistent with a cold-cache run on the larger post-2b.10 test set rather than indicative of any regression. No CI-only behavior; the script is the same one CI invokes per AGENTS.md.

The pass-2 set also re-confirms:
- `cargo fmt --check` (unchanged from pass 1)
- `cargo test -p sifr_hir typevar_constraints_violation` (the targeted unit test added in this slice — exact-equality message + `code == Some(...)` assertion at [expressions_tests.rs:1862-1866](../crates/sifr_hir/src/lower/expressions_tests.rs:1862))
- `cargo clippy --workspace -- -D warnings`

plus the earlier full-slice gates the user re-listed (`gen-error-docs`, both `check_diagnostic_*_sync.py` scripts, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`). Coverage of every file this slice modifies is preserved end-to-end. R7 closed.

### 3. The named-placeholder rewrite did not perturb any adjacent state (confirmation)

A repo-wide grep for `does not satisfy constraints` against `crates/`, `docs/`, `internal_docs/` returns the same six occurrences pass 1 enumerated:

| Source | Line | Form |
|---|---|---|
| [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs:1943) | 1943 | call-site format string (now **named** placeholders) |
| [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs:669) | 669 | registry template (named placeholders) |
| [crates/sifr_hir/src/lower/expressions_tests.rs](../crates/sifr_hir/src/lower/expressions_tests.rs:1864) | 1864 | unit-test assertion (post-substitution rendered text) |
| [crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr](../crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1) | 1 | e2e `expect-error` substring (post-substitution) |
| [docs/errors/SIFR-TYPE-0010.md](../docs/errors/SIFR-TYPE-0010.md:13) | 13 | generated docs row |
| [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md:86) | 86 | generated registry-table row |

Compared to the pass-1 grep, the only delta is the *form* of the call-site string at expressions.rs:1943 (positional → named); the substring being grepped (the human-readable phrase `does not satisfy constraints`) is preserved verbatim, so all six rows remain visible to the grep, none are duplicated, and no orphan occurrence has appeared.

### 4. Untracked files are exactly as expected (confirmation)

`git status` shows two untracked files:

- `docs/errors/SIFR-TYPE-0010.md` — the generated docs page from `gen-error-docs`, expected and reviewed in pass 1.
- `reviews/semantic-diagnostic-code-taxonomy-diag-4a-typevar-constraint-diagnostics-review-pass-1.md` — the pass-1 review file itself, expected and now retained alongside this pass-2 file.

No stray files appeared between passes. The seven-modification + one-untracked-doc scope from pass 1 is preserved (the pass-1 review file is review-tooling output, not part of the implementation diff).

### 5. Phase tracker bookkeeping is unchanged and still correct (confirmation)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:57-58](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:57) is byte-identical to the pass-1 state: 2b.22 marked merged with PR #1694, 2b.23 in-progress with PR pending, deferred-bridge-deletion line still `[x]`. `git log` continues to show `4eacdeae Migrate protocol signature diagnostic code (#1694)` as the latest merge, confirming the tracker's claim. No drift, no new bookkeeping needed for pass 2.

### 6. Carry-forward residual risks from pass 1 (unchanged status)

| ID | Risk | Pass-2 status |
|---|---|---|
| R1 | Class-constructor constraint path migrated implicitly without a dedicated fixture | **Unchanged** — pre-existing gap; non-blocking; absorbing it would expand scope. Track for the bridge-deletion follow-up. |
| R2 | Inline emission deviates from sibling slices' helper-module convention | **Unchanged** — stylistic; both patterns coexist; non-blocking. |
| R3 | Call-site positional placeholders vs. registry named placeholders | **Resolved** in pass 2 (this finding 1). |
| R4 | Sibling arg-mismatch branches at expressions.rs:1886/1898 lack unit-test coverage and remain on the `SIFR-TYPE-0001` bridge | **Unchanged** — pre-existing gap; non-blocking; expected to be picked up by a future generic-arg-mismatch slice before bridge deletion. |
| R5 | `display_name()` rendering stability is implicitly assumed by the unit test's exact-equality assertion | **Unchanged** — desirable failure mode; recording only. |
| R6 | TypeVar bindings iteration order is `HashMap`-based | **Unchanged** — pre-existing structural property; the slice's single-TypeVar fixture is unaffected. |
| R7 | Authoritative `scripts/run_all_tests.sh --profile quick` validation not yet recorded | **Resolved** in pass 2 (this finding 2). |

Two of the seven residual risks (R3, R7) are now closed. The remaining five (R1, R2, R4, R5, R6) retain their pass-1 status as non-blocking observations — none was action-required, none was introduced by this slice, and none was destabilised by the pass-2 changes.

## Verification I performed

- `git status` — confirmed the same eight-file delta as pass 1 (7 modifications + `docs/errors/SIFR-TYPE-0010.md` untracked), plus the pass-1 review file in `reviews/`.
- `git diff --stat` — confirmed the changed-line counts for all seven modifications: only `crates/sifr_hir/src/lower/expressions.rs` shifted (12 → 15 insertions, reflecting the positional → named rewrite). All other modifications byte-identical to pass 1.
- `git diff crates/sifr_hir/src/lower/expressions.rs` — read the full diff for the call site; confirmed the `ctx.error_with_code` invocation now uses `format!("type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'", actual = ..., constraints = ..., type_param = ...)`. Confirmed argument expressions (`concrete_ty.display_name()`, `constraints.join(", ")`, `tv_name`) and binding order are unchanged from pass 1's positional form, so substituted text is unchanged.
- `git diff` for `crates/sifr_diagnostics/src/codes.rs`, `crates/sifr_hir/src/lower/expressions_tests.rs`, `crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md`, `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` — confirmed each is byte-identical to pass 1.
- Read `docs/errors/SIFR-TYPE-0010.md` — confirmed unchanged from pass 1 (template row at line 13 still uses named placeholders, which now matches the call-site form too).
- Repo-wide grep for `does not satisfy constraints` in `crates/`, `docs/`, `internal_docs/` — confirmed the same six occurrences pass 1 enumerated, no orphans, no stale `SIFR-TYPE-0001`-paired references.
- Re-checked the registry → call-site → fixture → unit-test rendering chain for byte-identicality after the named-placeholder switch.
- Did not re-run the implementer's validation gates; relied on the user's pass-2 report that `cargo fmt --check`, `cargo test -p sifr_hir typevar_constraints_violation`, `cargo clippy --workspace -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=301.38s`) all passed, plus the earlier full-slice gates listed in pass 1.

## Recommendation

Mergeable as-is. Both pass-1 process/style notes (R3 named-placeholder convention, R7 authoritative quick-profile validation) are resolved. The pass-1 → pass-2 diff is a single localised rewrite of the `format!` call at [expressions.rs:1940-1948](../crates/sifr_hir/src/lower/expressions.rs:1940) that produces byte-identical substituted output and is independently gated by both the e2e fixture's substring assertion and the unit test's exact-equality assertion. No new blockers or new risks introduced. The five remaining non-blocking residuals from pass 1 (R1, R2, R4, R5, R6) retain their original status and should continue to be tracked for the bridge-deletion follow-up rather than addressed in this slice.

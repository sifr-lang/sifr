# Review pass 2: `milestone_diag_7` slice 4 — helper-specific TYPE_MISMATCH equality-comparison fixture

## Scope

This pass reviews the slice 4 changes after the rewrite from typed-return form to assignment form, and after the broader local validation requested in pass 1.

Files inspected:

- `crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr` (new, untracked)
- `internal_docs/diagnostic_emission_inventory.md` (modified)
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` (modified)

Code traced for correctness (re-verified against current tree):

- `crates/sifr_type_system/src/check.rs` `type_check_comparison` equality arm and final mismatch `Err`
- `crates/sifr_hir/src/lower/expressions.rs:495-522` (sole call site of `type_check_comparison`, dispatch through `error_with_code`)
- `crates/sifr_hir/src/lower/typing_and_functions.rs:830-848` (exhaustive-return `catch_unwind` recovery)
- `crates/sifr_hir/src/cfg.rs:535-543` (`build_control_flow_graph` validation panic)

Out-of-scope untracked files (`internal_docs/diagnostic_emission_inventory.md` aside, which is in scope; `issues/ad-hoc-signature-invalid-fixture-*`, `issues/ownership-mutability-boundary-*`, `reviews/ownership-mutability-boundary-*`, `reviews/semantic-diagnostic-code-taxonomy-diag-7-type-mismatch-comparison-fixture-review-pass-1.md`, `package.json`, `package-lock.json`, `verification/leetcode/`) were not touched and not assessed.

## Pass-1 follow-up status

| Pass-1 observation | Status |
| --- | --- |
| 1. CFG-validation panic noise observable in e2e mode | **Partially addressed; finding revised — see below.** Fixture rewritten to `_ = 1 == "1"` per pass-1 suggestion, but the panic still fires. |
| 2. No HIR-level companion test | Acknowledged, treated as optional; not added in this slice. Acceptable. |
| 3. Local validation broader than focused e2e | **Addressed.** Full fail suite, full workspace clippy, and `scripts/run_all_tests.sh --profile quick` were all run and reported in the prompt. |

## What the fixture exercises (confirmed for the new form)

Current fixture:

```sifr
# expect-error: SIFR-TYPE-0002

def main():
    _ = 1 == "1"
```

Lowering trace:

1. `1 == "1"` lowers as a `Compare` at `expressions.rs:485-528`. With `Type::Int` left and `Type::Str` right, `type_check_comparison` is called at line 504.
2. The decimal/bigdecimal/float and int/bigint mixing guards in `type_check_comparison` do not match. `equality_comparable(Int, Str)` returns `false`. The Union folding arms do not apply.
3. Control reaches the final equality-arm `Err((DiagnosticCode::TYPE_MISMATCH, "cannot compare 'int' and 'str' with =="))` — exactly the helper-specific operator-helper path the inventory cell describes.
4. `expressions.rs:506-516` does not detect an overload (left is `Type::Int`, not `Type::Class`), so `expressions.rs:517-520` records the diagnostic via `ctx.error_with_code(code, message)` and returns `None`. The compare expression never enters HIR.
5. `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr` confirms exactly one diagnostic: `type error: [main] cannot compare 'int' and 'str' with ==`.

The slice's claim that no prior fixture exercised this branch still holds. The other `SIFR-TYPE-0002` fixtures exercise different code paths:

- `crates/sifr/tests/e2e/fail/type_mismatch.sifr` — annotated-binding mismatch (HIR-site path, not the operator helper).
- `crates/sifr/tests/e2e/fail/union_type_mismatch.sifr` — union-binding mismatch (HIR-site path).
- `crates/sifr/tests/e2e/fail/filter_requires_explicit_materialization.sifr` — uses `==` only inside a lambda body that does not reach `type_check_comparison` for the failing assignment.

## Inventory diff (correct)

`internal_docs/diagnostic_emission_inventory.md:70` now reads:

> `crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr, crates/sifr/tests/e2e/fail/type_mismatch.sifr, crates/sifr/tests/e2e/fail/union_type_mismatch.sifr`

The "helper-specific comparison fixture pending" phrase has been removed and the new fixture is added at the head of the Representative cell. The category description ("equality comparison mismatch from operator helpers; broader expected/actual mismatch from HIR sites") still aligns with the three listed fixtures: the new one covers the first half, the other two cover the second. No other inventory rows needed updating.

## Issue tracker diff (correct)

`issues/...md:83` adds:

```
- [ ] `milestone_diag_7` slice 4 in progress: add helper-specific e2e fixture coverage for the operator-helper `SIFR-TYPE-0002` equality-comparison mismatch path and retire the corresponding inventory pending note.
```

It is placed in the expected sequential position after the slice-3 implementation and review entries, and matches the formatting of peer in-progress entries.

## Findings

### Revised observation: pass-1's CFG-panic prediction was wrong

Pass-1 observation 1 hypothesized that the panic was tied to feeding poisoned types into the exhaustive-return CFG validation in `lower/typing_and_functions.rs:830-848`, and recommended `_ = 1 == "1"` (or an `if`/`pass` form) to avoid it. That hypothesis is incorrect: the new assignment form **still triggers the same panic**. Re-running `cargo test -p sifr --test e2e test_e2e_fail -- type_comparison_mismatch --nocapture` against the current fixture prints:

```
thread 'test_e2e_fail' panicked at crates/sifr_hir/src/cfg.rs:540:9:
internal compiler error: invalid control-flow graph: branch terminator in block 2 is incomplete (1 target(s))
```

twice, then `1 passed`. The full fail suite (`cargo test -p sifr --test e2e test_e2e_fail -- --nocapture`) shows the same two panic lines, then `241 fail tests completed; ok`.

This means:

- The CFG-validation panic recovery surface is broader than the inventory's current note at `internal_docs/diagnostic_emission_inventory.md:270` claims. That row attributes the `catch_unwind` recovery to `lower/typing_and_functions.rs` exhaustive-return validation; here the panic is reached for a function with no return annotation and no return statement, so a different `flow_facts` / `build_control_flow_graph` call must be the entry point (candidates: `lower/statements.rs:127` always-exits check, `lower/flow_helpers.rs:5`, `lower/function_flow.rs:6`). Investigating and either narrowing the inventory note or fixing the underlying CFG-shape bug is out of slice 4's scope.
- For the slice itself, the situation is unchanged from pass 1: the panic is **pre-existing**, **recovered**, and the test passes. The fixture is still doing its intended job.
- For documentation hygiene, no action is needed inside slice 4. If the milestone owner wants the inventory note at line 270 to reflect that the recovery surface extends beyond exhaustive-return, that is a separate slice or a follow-up item. Worth recording, not a blocker here.

I do not recommend further fixture rewrites in this slice to chase silence — it is unclear which form, if any, would route through `type_check_comparison`'s `==` mismatch arm without also tripping the CFG-validation panic, and trying to find one risks weakening the fixture's targeting.

### Other observations (carried, non-blocking)

- **No HIR-level companion test.** Same standing as pass 1: optional. The e2e fixture is the sole guard for the helper-to-HIR `(code, message)` chain on equality-comparison errors. A focused HIR unit test would localize regressions if a future change reordered the operator-overload pre-check at `expressions.rs:506-516` so it short-circuited before `error_with_code`. Acceptable to defer.

- **Slice entry will need an "implementation complete" line on PR merge.** The current entry is the in-progress placeholder. The eventual completion entry should mirror slice 3's record, including the validation list and `report_signature`/`wall_time` from `scripts/run_all_tests.sh --profile quick`. The prompt's recorded values (`report_signature=e1bf653aaa770517`, `wall_time=387.44s`, advisories: warm wall-time exceeded, warm-cache hit rate below target, group skew high) are ready to be transcribed. No action required at this review stage.

### Local re-verification (run during this pass)

- `cargo fmt --check` — clean.
- `cargo clippy --workspace -- -D warnings` — clean.
- `cargo test -p sifr --test e2e test_e2e_fail -- type_comparison_mismatch --nocapture` — `1 passed`, with the pre-existing CFG panic noise discussed above.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` — `241 fail tests completed; ok`, same panic noise on the new fixture, no other regressions.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr` — exactly one diagnostic, `type error: [main] cannot compare 'int' and 'str' with ==`.
- I did not re-run `scripts/run_all_tests.sh --profile quick`; the prompt records it as already passing for this slice.

### Blockers

None.

## Verdict

Satisfied for PR. The fixture exercises the operator-helper `(DiagnosticCode::TYPE_MISMATCH, message)` equality-comparison branch, the inventory pending note is retired correctly, and the issue tracker carries a well-formed in-progress entry. Local validation has been broadened per pass-1 observation 3.

The only material correction relative to pass 1 is that the assignment-form rewrite did **not** silence the pre-existing CFG-validation panic noise, contrary to pass-1's prediction. That noise is recovered, does not affect the test result, and is out of scope for this slice; it is worth a follow-up to either narrow the inventory note at `internal_docs/diagnostic_emission_inventory.md:270` or address the underlying CFG-shape bug, but not in slice 4.

# Review pass 1: `milestone_diag_7` slice 4 — helper-specific TYPE_MISMATCH equality-comparison fixture

## Scope

This pass reviews the uncommitted slice 4 changes against the slice goal: add focused e2e fail-fixture coverage for the operator-helper `SIFR-TYPE-0002` equality-comparison mismatch path and retire the corresponding inventory pending note left over from slice 3.

Files inspected:

- `crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr` (new)
- `internal_docs/diagnostic_emission_inventory.md`
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`

Code traced for correctness:

- `crates/sifr_type_system/src/check.rs:323-389` (`type_check_comparison`, equality arm)
- `crates/sifr_type_system/src/check.rs:440-470` (`equality_comparable`)
- `crates/sifr_hir/src/lower/expressions.rs:495-522` (only call site of `type_check_comparison`)

Other repo state observed:

- Untracked checklists, ownership-mutability docs, `package.json`/`package-lock.json`, and `verification/leetcode/` are unrelated to this slice; not touched and not within review scope.

## What the fixture exercises

The new fixture is:

```sifr
# expect-error: SIFR-TYPE-0002

def main() -> bool:
    return 1 == "1"
```

Lowering trace for the body:

1. `1 == "1"` is a `Compare` expression. `crates/sifr_hir/src/lower/expressions.rs:504` calls `type_check_comparison(left.ty(), op_str, right.ty())` with `Type::Int` and `Type::Str`.
2. In `type_check_comparison` (`check.rs:323`), the decimal/bigdecimal mixing guard, float/decimal mixing guard, and int/bigint mixing guard for `==` all fall through.
3. `equality_comparable(Int, Str)` returns `false` — types are not equal, neither is `Any`/`Unknown`, and they don't match the `List/Set/Dict/Tuple` recursive arms.
4. The Union folding arms don't apply (neither side is a union).
5. Control falls through to the final equality-arm `Err`:
   ```rust
   Err((
       DiagnosticCode::TYPE_MISMATCH,
       format!("cannot compare '{}' and '{}' with {op}", ...),
   ))
   ```
   This is exactly the helper-specific path the inventory cell describes.
6. HIR records it via `ctx.error_with_code(code, message)` at `expressions.rs:518`, producing a `SIFR-TYPE-0002` emission.

Manual `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr` confirms exactly one diagnostic — `type error: [main] cannot compare 'int' and 'str' with ==` — matching the format string at `check.rs:383-387`. No second diagnostic is emitted, so the single `expect-error: SIFR-TYPE-0002` line is satisfied without coincidental help from another emission.

I also confirmed this is the first SIFR-TYPE-0002 fixture targeting this branch:

- `crates/sifr/tests/e2e/fail/type_mismatch.sifr` is an annotated-binding mismatch (HIR site path).
- `crates/sifr/tests/e2e/fail/union_type_mismatch.sifr` is a union-binding mismatch (HIR site path).
- `crates/sifr/tests/e2e/fail/filter_requires_explicit_materialization.sifr` uses `==` only inside a lambda body that never reaches `type_check_comparison` for the failing assignment; the failure is the assignment-site mismatch, not the helper path.

So slice 4's claim — that a helper-specific comparison fixture was previously missing — is correct, and this fixture closes that gap.

## Inventory diff

`internal_docs/diagnostic_emission_inventory.md:70` is updated correctly:

- Pending phrase "helper-specific comparison fixture pending" is removed.
- New fixture is added to the Representative cell, leaving the prior `type_mismatch.sifr` and `union_type_mismatch.sifr` references intact.
- Category description ("equality comparison mismatch from operator helpers; broader expected/actual mismatch from HIR sites") is unchanged and remains accurate; the new fixture matches the first half of that description while the other two cover the second half.

No other rows in the inventory needed updating. Nothing in the row references slices/PRs that would now be stale.

## Issue diff

`issues/...md:83` adds the slice 4 in-progress entry:

```
- [ ] `milestone_diag_7` slice 4 in progress: add helper-specific e2e fixture coverage for the operator-helper `SIFR-TYPE-0002` equality-comparison mismatch path and retire the corresponding inventory pending note.
```

This matches the formatting of peer "in progress" entries earlier in the milestone log, sits in the expected sequential position after the slice 3 review entry, and accurately describes the change. No PR link is expected yet for an in-progress slice.

## Findings

### Non-blocking observations

1. **CFG validation panic is observable in the e2e test mode for this fixture.** Running `cargo test -p sifr --test e2e test_e2e_fail -- type_comparison_mismatch --nocapture` prints:
   ```
   thread 'test_e2e_fail' panicked at crates/sifr_hir/src/cfg.rs:540:9:
   internal compiler error: invalid control-flow graph: branch terminator in block 2 is incomplete (1 target(s))
   ```
   twice, and the test still reports `1 passed`. This is the wrong-layer `catch_unwind` recovery already documented in the inventory at `internal_docs/diagnostic_emission_inventory.md:270` (exhaustive-return validation panic recovery in `lower/typing_and_functions.rs`). It is **pre-existing** behavior, not introduced by slice 4. However, the new fixture is a fresh way to surface it because the comparison failure leaves the `return` expression poisoned and the CFG terminator incomplete. The driver `check` mode does not panic on the same input, only the e2e test pipeline.

   Suggestion (optional, not a blocker): if the slice author wants to keep stderr clean for this fixture, the body can be rewritten to avoid feeding the failing comparison into a terminating return — for example:

   ```sifr
   def main():
       _ = 1 == "1"
   ```

   or

   ```sifr
   def main():
       if 1 == "1":
           pass
   ```

   Either form still routes through `type_check_comparison`'s `==` branch with `Int`/`Str`, still produces exactly one `SIFR-TYPE-0002` emission, and avoids feeding `Unknown`-typed comparison output into the exhaustive-return CFG validation that triggers the `catch_unwind`. This is purely a cleanliness improvement; if the slice author prefers to keep the explicit `-> bool` form to mirror real user code, that is also defensible.

2. **No HIR-level companion test.** Slice 3 added focused HIR regression tests (`augassign_type_error_keeps_code`, etc.) that lock the helper-to-HIR `(code, message)` pairing for aug-assign and subscript operator paths. There is no analogous unit-level test for the equality-comparison helper branch — the new e2e fixture is the only coverage of the `type_check_comparison` `==`/`!=` mismatch -> `error_with_code(TYPE_MISMATCH, ...)` chain. If a future change in HIR pre-checks (e.g., reordering operator-overload detection at `expressions.rs:506-516` so it short-circuits before `error_with_code`) silently swallowed the diagnostic, the e2e fixture would catch it but a unit test would localize the regression. Worth considering as a follow-up if the milestone owner wants slice-by-slice unit-test parity; not required to land this slice.

3. **Local validation is narrower than slice 3's record.** The slice 4 entry will eventually need a record line analogous to slice 3's. The validation enumerated in the prompt was `cargo fmt --check` and a single focused e2e run. Before opening the slice 4 PR, please also run, at minimum, the same set slice 3 documented:
   - `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` (full fail suite, not just the new fixture)
   - `cargo clippy --workspace -- -D warnings` (or at least `cargo clippy -p sifr_type_system -p sifr_hir --no-deps -- -D warnings` to match slice 3)
   - `scripts/run_all_tests.sh --profile quick`
   The full fail-suite run is the authoritative gate for a fixture-only slice; the focused `-- type_comparison_mismatch` filter alone won't catch a regression where the new fixture interacts with another fail expectation.

### Blockers

None.

## Verdict

Satisfied with the substance of the slice. The fixture genuinely exercises the operator helper's `(DiagnosticCode::TYPE_MISMATCH, message)` equality-comparison branch in `crates/sifr_type_system/src/check.rs:381-388`, which previously had no representative e2e fixture. Inventory and phase-issue updates are accurate and minimal. Recommend addressing observation 3 (broader local validation) before merge and treating observations 1 and 2 as optional polish.

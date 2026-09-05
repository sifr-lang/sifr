Independent post-rebase review of Wave 1 at `3b78a4efe` against `b3495318d`. I inspected the complete diff, built the compiler from this tree, and re-verified every claim empirically. I modified no tracked file, ref, or submodule (working tree still shows only the two `.DS_Store`-only submodule entries and the empty pass-3 destination file).

## Change set

Four functional/doc files, one production line:

- `crates/sifr_lowering/src/lower/type_bounds.rs:220` — `Type::List(element) => supports_total_order(element)`
- `crates/sifr_lowering/src/lower/expressions_tests/algorithmic_corpus_regressions.rs` (new, 52 lines) + `mod` at `expressions_tests.rs:13`
- `crates/sifr/tests/e2e/pass/recursive_list_total_order.sifr` (new, 14 lines)
- issue ledger: 5 lines

## Rebase-specific verification (the point of this pass)

Upstream `b3495318d` (PRs #3065/#3066) rewrote class-field mutating-receiver place semantics and touched 168 files including `method_type_collections.rs`'s neighbourhood and `mutating_methods.rs`. The predicate call sites survived intact and still number exactly two — `expression_sum_sorted.rs:301` (`sorted`) and `expressions/method_type_collections.rs:43` (`list.sort`). I probed the one surface where the two changes actually intersect — a nested-list sort through a mutating class-field receiver:

```python
class Grid:
    rows: list[list[int]]
    def order(self):
        self.rows.sort()
```

builds and runs with the ordering assertion live (exit 0). Nested `bool`, triple-nested `int`, and `list[tuple[list[int], int]]` also build and run correctly.

## Correctness

- The predicate is exactly the generated representation's condition: `Vec<Vec<T>>: Ord` iff `T: Ord`. The accept arm (`208-219`) and `TypeVar => false` (`222`) are untouched, so no new leaf type is admitted. Recursion terminates on the finite `Box` type tree; `Tuple` already recursed identically.
- Codegen confirms the emitted forms: `builtin_core_methods.rs:502-511` emits bare `.sort()`, and the keyed branch (`:474-497`) emits `key(a).cmp(&key(b))`, both valid for `Vec<T>` keys. `list.sort` accepts only `reverse` (`method_type_collections.rs:131-151`), so gating on `elem_ty` is right.
- **CPython differential** on the semantics this widens — nested prefix ordering and nested `str` ordering including non-ASCII — is byte-identical to `python3`: `[[], [1], [1,2], [1,2,3], [1,2,3,0], [2]]` and `[["Z"],["a"],["a","b"],["z"],["é"]]`. UTF-8 byte order equals code-point order, so no divergence hides here.
- Rejections preserved with clean Sifr diagnostics (SIFR-TYPE-0002, caret on `sort`): nested `float`, `set`, `dict`, class, and `int | None` elements. Set/dict *containers* of lists still fail on the untouched hash gate (`set element type 'list[int]' is not hashable`; `dict subscript assignment requires … Eq + Hash`), exactly as the issue requires.
- `min`/`max` share no call site with the predicate (`min_max_validation.rs` never references it), so the issue's deliberate Wave 1 exclusion holds by construction.

## Coverage and diagnostics

Positive nested `list.sort`, triple-nested `sorted(reverse=True)`, and a named list-returning `sorted` key are covered at both unit level and, with discriminating runtime assertions, in the e2e fixture (`assert keyed == [11, 21, 12]` fails under natural int order, so it pins lexicographic key comparison). Negatives pin exact message strings and ranges for both distinct gates: `range_for(source, "sort")` for `list.sort` and `range_for_after_anchor(source, "sorted(", "values")` for the `sorted` iterable branch — I confirmed both ranges against live compiler output.

## Regression evidence at this head

- Complete 411-fixture `sifr check` sweep: **397 pass / 14 fail**. The 14 are precisely the documented 20 minus the six Wave 1 targets — no fixture anywhere in the corpus regressed and the six intended fixtures flipped.
- All six targets (`0056`, `0252`, `0435`, `0452`, `1383`, `2402`) `check` + `build` + native-binary `run` at exit 0, each with 1–3 live `assert`s, so exit 0 is semantic evidence. No fixture source edited; no submodule pointer moved.
- `cargo test -p sifr_lowering`: 882 passed / 1 ignored (up from 865 pre-rebase, consistent with upstream's added tests + this wave's 3). New module's 3 tests pass. `cargo fmt --check` clean; file-size guardrail PASS (2975 files, limit 900 — touched files 361/52/14); HIR maintainability PASS.

## Scope, maintainability, ledger

One production line, one focused test module (per the plan's explicit Wave 1/2 module requirement, `mod` in alphabetical position), one capability-named e2e fixture with no phase number, five doc lines. No baselines, exclusions, fallbacks, profile registration, crate pins, or Rust-interop matrix changes. No public docs describe sort element capability, so nothing is left stale. The ledger row accurately reads "approved; PR pending" with both prior passes linked, and the reworded *Separately Tracked Findings* claims match what I reproduced (`sorted(key=lambda …)` → `E0425` on flat `list[int]`; `list[None]` → `E0308` with no sort at all — both pre-existing and independent of this diff).

Non-blocking, no action for this wave: the pass-3 artifact is still 0 bytes and untracked (it is this report's destination), and `crates/sifr/tests/e2e/pass/Untitled` is a pre-existing tracked stray unrelated to this change.

Zero actionable findings.

APPROVED

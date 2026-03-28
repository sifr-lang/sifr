# Phase 31 Follow-up: `m31_c_stdlib_module_parity` Slice 2

Status: complete
Started: 2026-03-11
Current slice: `m31_c_slice_2_constructor_compatibility`

## Goal

Remove the next stdlib-compatibility root causes inside `stdlib.python_module_surface` by aligning constructor-entry surfaces that the seed corpus expects:

- native `set()` / `set(iterable)` built-in construction
- CPython-style bare `deque()` / `deque(iterable)` compatibility

This slice is intentionally scoped to constructor-entry behavior. It should not paper over downstream typing or lowering gaps that appear after these symbols begin resolving correctly.

## Targeted Cases

- `0003` `longest_substring_without_repeating_characters`
- `0127` `word_ladder`
- `0217` `contains_duplicate`

Secondary watch cases:

- constructor-heavy deque users in the broader audit corpus
- existing `sifr.collections.deque` regression fixtures

## Root-cause Hypothesis

- `set(...)` is a real language/container concept in Sifr (`set[T]`, set methods, set literals, set comprehensions), but the call-entry surface is missing, so corpus code fails before it can use the native set machinery.
- `deque` already exists in `sifr.collections`, but the constructor shape still reflects the original Sifr-specific `maxlen`-first API instead of the CPython-compatible iterable-first shape used by the audit corpus.
- Bare `deque(...)` calls in LeetCode fixtures are not imported and therefore need compatibility resolution to the existing `sifr.collections.deque` surface.

## Planned Work

1. Add builtin lowering for `set()` and `set(iterable)` that returns native `set[T]` values instead of routing through the older list-backed helper surface.
2. Update `sifr.collections.deque` to support iterable-first construction with optional `maxlen`, while preserving explicit bounded-queue behavior via keyword arguments.
3. Resolve bare `deque(...)` calls through the existing compatibility import path so LeetCode-style usage does not require manual imports.
4. Add regression coverage for:
   - builtin `set()` and `set(list[T])`
   - bare `deque()` / `deque(list[T])`
   - existing bounded `deque(maxlen=...)` behavior
5. Re-run the targeted corpus slice and record the resulting status delta.

## Implemented Root-cause Fixes

- Added native builtin lowering for `set()` and `set(iterable)` so LeetCode-style set construction routes into Sifr's real `set[T]` surface instead of failing at symbol resolution.
- Added empty-set local codegen handling that lets Rust infer the concrete `HashSet<T>` element type instead of forcing `Any` into an unusable runtime representation.
- Added empty-set binding refinement on first `in` / `add` / `remove` / `discard` usage so previously untyped `set()` locals can pick up a concrete element type from actual corpus operations.
- Extended containment typing so `'x in my_set'` uses `set[T]` element information the same way list/dict/string already did.
- Added bare-call compatibility resolution for `deque(...)` so LeetCode code no longer needs an explicit `from sifr.collections import deque`.
- Exported and propagated callable default-argument metadata through `LoweringResult` and `ExternalDefs`, fixing imported class/function default handling for stdlib surfaces such as `deque(maxlen=...)`.
- Updated `sifr.collections.deque` to the CPython-compatible constructor shape `deque(items: list[T] = [], maxlen: int = 0)` and kept bounded-queue behavior through the explicit `maxlen` parameter.
- Updated deque constructor codegen so the `_data` backing field is materialized as `VecDeque` even when constructor setup builds an intermediate list.
- Fixed the Phase 31 seed corpus manifest after the repo rename from `audit/` to `audits/`, so targeted reruns produce real compatibility evidence again.

## Regression Coverage

- `crates/sifr/tests/e2e/pass/phase31_constructor_compat.sifr`
- `crates/sifr_hir/src/lower/expressions.rs` tests:
  - `test_builtin_set_constructor_accepts_list_iterable`
  - `test_bare_deque_call_resolves_without_import`
- Updated deque compatibility coverage in:
  - `crates/sifr/tests/e2e/pass/stdlib_collections_consolidated.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_collections_subset.sifr`
  - `crates/sifr/tests/e2e/pass/generic_deque_float.sifr`
  - `crates/sifr/tests/e2e/pass/generic_deque_str.sifr`

## Validation Evidence

- `cargo build -q -p sifr`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_constructor_compat.sifr`
- `target/debug/sifr check audits/leetcode/0003_longest_substring_without_repeating_characters.sifr`
- `target/debug/sifr check audits/leetcode/0127_word_ladder.sifr`
- `target/debug/sifr check audits/leetcode/0217_contains_duplicate.sifr`
- `target/debug/sifr run audits/leetcode/0217_contains_duplicate.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31c_wave2_results.json --case 0003 --case 0127 --case 0217`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Current Measured Outcome

Artifact:

- `verification/leetcode/phase31_m31c_wave2_results.json`

Status counts for the targeted three-case constructor slice after this work:

- `PASS=1`
- `RUN_ERROR=1`
- `CHECK_ERROR=1`

Observed movement:

- `0217` is now a full `PASS`.
- `0127` no longer fails on missing bare `deque(...)`; it now fails on remaining `defaultdict`/`len(deque)` compatibility gaps.
- `0003` no longer fails on missing `set(...)`; it now reaches a deeper codegen panic around the sliding-window loop shape after set-type refinement.

Remaining slice-local blockers:

- `0127`: unresolved `collections.defaultdict(...)` compatibility plus `len(deque)` support
- `0003`: downstream codegen gap after empty-set refinement and native set constructor enablement

## Next Slice Candidates

1. Add `len(deque)` support by treating `deque` as a `Sized` collection in builtin length lowering.
2. Decide and implement the next approved `defaultdict` compatibility surface needed by the corpus, or explicitly reclassify it if it belongs outside `m31_c`.
3. Decompose the `0003` run-stage panic and either fix the structured codegen gap directly or reclassify it into the owning downstream bucket if it is not stdlib-surface work.

## Definition Of Done

- `0003` and `0217` move past undefined-symbol failures caused by missing `set`.
- `0127` moves past undefined-symbol failures caused by missing bare `deque`.
- Existing `deque` regressions continue to pass with the CPython-compatible constructor shape.
- Any remaining `0127` failures are narrowed to downstream non-constructor gaps and recorded explicitly.

## Validation Plan

- focused regression tests for builtin `set` lowering and `deque` constructor compatibility
- targeted corpus rerun for `0003`, `0127`, `0217`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

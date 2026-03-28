# Phase 31 Follow-up: `m31_c_stdlib_module_parity` Slice 3

Status: complete
Started: 2026-03-11
Current slice: `m31_c_slice_3_defaultdict_and_len_deque`

## Goal

Remove the next stdlib-surface blockers exposed after constructor compatibility:

- `collections.defaultdict(...)` compatibility for the corpus factories used today (`list`, `set`, `int`)
- builtin `len(deque)` parity for the existing `sifr.collections.deque` class

This slice is scoped to the module/API surface and the binding/codegen behavior needed to make those APIs usable in real audit programs. It does not attempt to solve the deeper optional-slicing or arithmetic typing failures that appear once the stdlib gap is removed.

## Targeted Cases

Primary seeded case:

- `0127` `word_ladder`

Broader parity probes:

- `0036` `valid_sudoku` (`defaultdict(set)`)
- `0149` `max_points_on_a_line` (`defaultdict(int)`)

## Root-cause Hypothesis

- `collections.defaultdict(...)` was not reaching any stdlib or builtin lowering path, so the compiler failed before it could reason about the container at all.
- `len(q)` for `q: deque[T]` was still hard-coded to builtin container types and ignored sized user/stdlib classes that already expose a `len()` method.
- Once `defaultdict` exists, codegen must let local bindings infer concrete key/value types from first real usage and must emit mutable bindings, because `entry().or_insert(...)` mutates on both writes and missing-key reads.

## Implemented Root-cause Fixes

- Added compat builtin lowering for `collections.defaultdict(...)` and bare `defaultdict(...)` calls.
- Added alias-backed `defaultdict` typing for `int`, `list`, and `set` factories.
- Added HIR refinement for `defaultdict(list)` and `defaultdict(set)` bindings so first indexed writes refine both key and value types instead of staying at `Any`.
- Added alias-aware index typing so `defaultdict[key]` returns the defaulted value type instead of `value | None`.
- Added codegen lowering for:
  - `defaultdict(...)` constructor calls
  - defaulting index reads via `entry(...).or_insert(...)`
  - `defaultdict(list)[key].append(...)`
  - `defaultdict(set)[key].add(...)`
  - `defaultdict(int)[key] += value`
- Omitted explicit local type annotations for unresolved `defaultdict` constructors so Rust infers concrete `HashMap<K, V>` types from first usage.
- Forced `defaultdict` locals to emit as mutable bindings because defaulting reads/writes use `entry(...).or_insert(...)`.
- Extended builtin `len()` lowering to accept sized class instances that already expose a `len` method, which covers `deque`.

## Regression Coverage

- `crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr`
- `crates/sifr_hir/src/lower/expressions.rs` test:
  - `test_defaultdict_list_call_resolves_without_import`
- Demo:
  - `demos/phase31_defaultdict_compat_demo.sifr`

## Validation Evidence

- `cargo build -q -p sifr`
- `cargo test -q -p sifr_hir test_defaultdict_list_call_resolves_without_import`
- `target/debug/sifr run crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr`
- `target/debug/sifr run demos/phase31_defaultdict_compat_demo.sifr`
- `target/debug/sifr check audits/leetcode/0127_word_ladder.sifr`
- `target/debug/sifr check audits/leetcode/0036_valid_sudoku.sifr`
- `target/debug/sifr run audits/leetcode/0036_valid_sudoku.sifr`
- `target/debug/sifr check audits/leetcode/0149_max_points_on_a_line.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31c_wave3_results.json --case 0127`

## Current Measured Outcome

Primary Phase 31 artifact:

- `verification/leetcode/phase31_m31c_wave3_results.json`

Observed movement:

- `0127` remains `CHECK_ERROR`, but the old `collections.defaultdict(...)`, bare `deque(...)`, and `len(deque)` blockers are gone. It now fails on deeper optional-slicing/string-flow issues:
  - `cannot slice type 'None | str'`
  - `undefined variable: 'pattern'`
  - `unsupported operand type(s) for +: 'Any' and 'str'`
- `0036` now fully checks and runs, validating the `defaultdict(set)` surface.
- `0149` now moves past `defaultdict(int)` construction into deeper arithmetic / optional-flow errors:
  - `undefined variable: 'slope'`
  - `unsupported operand type(s) for -: 'int | None' and 'int | None'`

## Remaining Slice-local Follow-on

- `0127` is now blocked by non-stdlib frontend/type-flow issues in string slicing and optional narrowing.
- `0149` is now blocked by arithmetic/optional typing rather than stdlib module parity.
- The `0003` codegen failure from slice 2 remains a separate downstream blocker.

## Definition Of Done

- `collections.defaultdict(...)` is a real compiler surface for the currently used corpus factories.
- `len(deque)` works without rewriting user code to call `q.len()`.
- At least one `defaultdict(set)` parity probe fully passes.
- Seeded case `0127` moves past the prior stdlib surface blockers and is reclassified to deeper non-stdlib work.

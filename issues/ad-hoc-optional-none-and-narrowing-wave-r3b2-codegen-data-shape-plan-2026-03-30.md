# Ad Hoc Optional/None Closure: Wave-R3b2 Codegen Data-Shape Plan (2026-03-30)

Status: reviewer-pass1 corrections applied (pending readiness re-check)
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Owning execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Probe baseline: `verification/leetcode/run_error_quartet_plus_baseline24_probe_20260330_wave_r3a_semantic_gate.json` (`RUN_ERROR=10`, `CHECK_ERROR=6`, `PASS=8`)

## Objective

Close the next compiler-owned run-stage bucket without relaxing Sifr principles:

- keep check-time semantics explicit,
- avoid fixture rewrites,
- eliminate codegen parity defects that currently surface as Rust compile failures.

## Scope (Wave-R3b2)

Fixtures and root causes in this slice:

- `0187_repeated_dna_sequences`
  - `list(set_like)` lowering emits `Vec<&T>` via borrow iteration (`iter().collect::<Vec<_>>()`) instead of owned `Vec<T>`.
- `1461_check_if_a_string_contains_all_binary_codes_of_size_k`
  - `set(generator)` builtin call falls through to unresolved plain `set(...)` emission when strict registry lowering cannot lower generator args.
- `1582_special_positions_in_a_binary_matrix`
  - list repetition `[x] * n` lowers to invalid Rust `Vec<T> * i64`.
- `0441_arranging_coins`
  - comparison lowering does not coerce mixed `float`/`int` operands inside `Compare`, producing Rust `E0308`.
- `1905_count_sub_islands`
  - boolop lowering keeps `Option<i64>` operand in boolean context (`if option && ...`) instead of explicit truthiness conversion.
- `0459_repeated_substring_pattern` (explicit classification)
  - currently `RUN_ERROR` (runtime assertion mismatch, no compile error code) and does not belong to the codegen data-shape defect bucket in this wave.
  - ownership is explicitly deferred to the next semantics/slice-lowering wave.

## Proposed Compiler Changes

### A) Registry/builtin lowering ownership + fallback parity

- File: `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- Changes:
  - For `list(arg)` lowering, provide typed ownership hint from expected result element type when available so conversion produces owned `Vec<T>` in set/list iterator cases.
  - For `set(arg)` lowering, add fallback path that can lower generator arguments through stmt-support lowering when strict registry lowering cannot represent the generator directly.

### B) List repetition lowering

- File: `crates/sifr_codegen/src/stmt_support_emitter.rs`
- Changes:
  - Add dedicated `BinOp('*')` lowering for `list/bytes * int`:
    - non-positive count -> empty vector,
    - positive count -> deterministic repeated-extend construction (no invalid `Vec * i64` emission).

### C) Numeric compare coercion parity

- File: `crates/sifr_codegen/src/stmt_support_emitter.rs`
- Changes:
  - In `Compare` lowering, coerce `int` -> `float` when the paired operand is `float` (symmetrically), matching arithmetic coercion behavior and removing Rust `E0308` mismatch.

### D) Boolop condition coercion for bool-typed expressions

- File: `crates/sifr_codegen/src/stmt_support_emitter.rs`
- Changes:
  - For `HirExpr::BoolOp` with result type `bool`, lower operands through condition-coercion contract (`lower_condition_expr_for_ir`) rather than raw expression lowering.
  - Preserve existing lowering for non-bool boolop expression contexts.

## Tests and Validation

### Unit regressions (required)

- `crates/sifr_codegen/src/lib_codegen_tests.rs`
  - `list(set[str])` lowers to owned `Vec<String>` collection path.
  - `set(generator_expr)` lowers without unresolved plain `set(...)`.
  - `list_repeat` emits repeated-extend block, not `Vec * i64`.
  - float/int compare in `Compare` emits numeric-coerced operands.
  - bool-typed boolop with optional operand emits condition-coerced boolean.

### Targeted fixture reruns

- `target/release/sifr run audits/leetcode/0187_repeated_dna_sequences.sifr`
- `target/release/sifr run audits/leetcode/1461_check_if_a_string_contains_all_binary_codes_of_size_k.sifr`
- `target/release/sifr run audits/leetcode/1582_special_positions_in_a_binary_matrix.sifr`
- `target/release/sifr run audits/leetcode/0441_arranging_coins.sifr`
- `target/release/sifr run audits/leetcode/1905_count_sub_islands.sifr`

### Gate

- `scripts/run_all_tests.sh --profile quick`

## Probe Reconciliation

Baseline probe (`RUN_ERROR=10`) fixtures:

- in-scope for this wave: `0187`, `1461`, `1582`, `0441`, `1905`
- deferred to R3c (HIR/type inference): `0054`, `0071`, `0349`, `0763`
- deferred to slice-semantics wave: `0459`

## Out of Scope for this wave

- Empty-collection `Any` specialization bucket (`0054`, `0071`, `0349`, `0763`) — HIR/type inference lane (`R3c`).
- Runtime slice semantics mismatch (`0459`) — separate semantics/slice-lowering lane (non data-shape codegen parity).

## Reviewer Questions

1. Is it acceptable under Sifr principles to use expected result type as an ownership hint for `list(arg)` lowering when iterator element ownership metadata is absent?
2. For `set(generator)`, is fallback to stmt-support generator lowering acceptable in this wave, or should generator handling be added to strict registry lowering first?
3. Should boolop operand condition coercion be restricted strictly to bool-typed boolops to avoid changing non-bool short-circuit expression semantics?

# Optional/None Closure Follow-up: Wave-8b Run-Stage Ownership Stabilization Plan

Date: 2026-03-29
Owning phase: `issues/ad-hoc-optional-none-and-narrowing-closure.md`
Execution ledger: `issues/ad-hoc-optional-none-and-narrowing-closure-execution.md`
Status: reviewer-gated plan (ready)

## Trigger

Wave-7 removed check-side Optional gates for:

- `0205_isomorphic_strings`
- `0290_word_pattern`

Both fixtures moved from `CHECK_ERROR` to `RUN_ERROR` due to generated Rust move-use failures (`E0382`) in subscript-assignment emission.

## Root Cause

For list/dict subscript assignment codegen paths, lowering reused non-copy `Name` operands without cloning in index/value slots.
Generated Rust consumed ownership in one site and reused the moved binding in another, producing run-stage compile failure in Rust.

Owning loci:

- `crates/sifr_codegen/src/lower_stmt.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`

## Principle Constraints

- no hidden Optional coercion or unwrap insertion
- no fixture-specific recognizers
- fix must stay in codegen ownership handling, not checker semantics

## Proposed Fix

- add non-copy `Name` clone handling for subscript-assignment index/value emission in:
  - direct/simple statement lowering path
  - IR support emitter lowering path
- add unit regression coverage ensuring non-copy dict subscript assignment value path is cloned

## Validation Matrix

- `cargo test -q -p sifr_codegen lowers_simple_dict_subscript_assign_stmt`
- `cargo test -q -p sifr_codegen lowers_simple_dict_subscript_assign_clones_non_copy_name_value`
- `cargo run -q -p sifr -- check audits/leetcode/0205_isomorphic_strings.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0205_isomorphic_strings.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0290_word_pattern.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0290_word_pattern.sifr`
- `scripts/run_all_tests.sh --profile quick`
- full-corpus rerun with `target/release/sifr` over `audits/leetcode` and status delta capture vs wave-8 artifact

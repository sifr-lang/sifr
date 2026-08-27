# Method Lowering And Unsafe Contracts

This document defines the ownership rules for method dispatch, unsafe Python
ABI code, and retained compiler panic invariants.

## Method Dispatch Ownership

`crates/sifr_codegen/src/methods/authority.rs` owns source-language method
semantics in code generation. It maps a resolved Sifr type and method name to
one Rust IR expression. A second codegen path must not define the same
source-language behavior.

Not every method-name branch defines language semantics. The repository uses
four categories:

1. `language-semantics` maps a Sifr type and method name to Rust behavior. This
   category has one owner: `methods/authority.rs`.
2. `typed-hir` checks method types, ownership, narrowing, and receiver rules.
   These sites belong to lowering because they construct or validate HIR.
3. `contextual-codegen` handles a context that the general method authority
   does not own. Examples include indexed storage and compiler-proven receiver
   places.
4. `rust-ir-consumer` reads method names from compiler-owned Rust IR. Examples
   include validation, optimization, and runtime-need analysis.

`verification/policy/method_dispatch_authority.json` records each production
site and its count. `scripts/check_method_dispatch_authority.py` rejects a new
site, a changed count, a stale entry, or a second language-semantics owner. If
a change adds a site, classify its responsibility before you update the
inventory.

## Unsafe Python ABI Boundary

The parent `python.rs` module must not use a file-wide unsafe-code allowance.
A function can allow unsafe code when the function is the complete ABI
boundary. A child file can allow unsafe code when the full file owns one narrow
ABI responsibility.

Each retained unsafe operation must have a nearby `SAFETY:` contract or a
`# Safety` section. The contract must state the condition that makes the
operation valid. It must not only restate the operation.

`scripts/check_unsafe_abi_contracts.py` enforces these rules. It also owns the
small allowlist of child ABI files that can use a file-wide allowance.

## Compiler Panic Invariants

Code generation can retain a panic macro only for a programmer invariant. Put
an `INVARIANT:` comment next to the macro. State which earlier validation,
classification, or compiler-owned state transition proves the condition.

Do not use an invariant panic to reject source input. Return a structured
compiler error for a source shape that can reach the codegen boundary. Sifr
`assert` statements are language behavior and are not compiler panic
invariants.

`scripts/check_codegen_invariant_contracts.py` rejects a production codegen
panic, unreachable branch, or assertion that has no local contract. The scan
excludes test modules and string literals. It continues after inline test
modules, so later production code remains covered.

## Validation Coverage

The create-PR, merge, nightly, and release profiles run all three guardrails.
Each guardrail also runs its negative self-test. The self-tests prove that the
guardrails reject unclassified dispatch, broad unsafe scope, missing safety
contracts, and missing invariant contracts.

# Sifr Stdlib Borrow/Ownership Audit (Post-Parity)

Generated: 2026-02-17

## Scope

- Architecture baseline: `internal_docs/architecture.md` (borrow/lifetime + safety contracts).
- Implemented stdlib: `lib/sifr/*.sifr` (37 modules).
- Borrow conventions + intrinsic typing: `crates/sifr_hir/src/stdlib.rs`, `crates/sifr_type_system/src/types.rs`.
- Runtime behavior in generated Rust: `crates/sifr_codegen/src/lib.rs`.
- Validation status: `cargo test --test e2e` passes.

## Headline Findings

- **Borrow-by-default is structurally in place**, and stdlib mostly compiles by using ownership-light API patterns.
- **Stdlib ownership coverage is shallow**: `lib/sifr` currently has **zero explicit `own`/`mut` parameters** in exported signatures.
- **Intrinsic registry is uniformly borrowed**: `_sifr.*` signatures are overwhelmingly `FunctionType::all_borrow(...)` (124 uses in `stdlib.rs`).
- **Major contract contradiction remains**: many fallible stdlib operations still panic (`unwrap`) in generated Rust instead of returning `Result`/`Option`.
- **Second major contradiction remains**: architecture says references that escape should error, but codegen still inserts implicit `.clone()` in ownership-sensitive returns.

## What Is Working

- Function argument conventions (`Borrow` / `MutBorrow` / `Own`) exist and are wired into type system + codegen.
- Call-site exclusivity checks for mutable/immutable borrow conflicts are implemented in lowering.
- Method receiver mutability (`&self` vs `&mut self`) is inferred for class methods based on field assignment in method bodies.
- `with` protocol checks and cleanup machinery are implemented and tested (ContextManager checks + Drop guards).

## Core Contradictions (Severity)

1. **High** - Safety contract mismatch for fallible stdlib operations  
   - Architecture requires `Result`/`Option` for fallible stdlib APIs.
   - Runtime emission for many intrinsics uses `.unwrap()` and panics.

2. **High** - Escape-analysis/clone policy mismatch  
   - Architecture says no silent clone on escaping borrows.
   - Codegen still auto-inserts `.clone()` for some return paths.

3. **Medium-High** - Method receiver contract gap  
   - Architecture includes consumptive method receiver (`self` by move).
   - Current receiver inference emits only `&self` / `&mut self`.

4. **Medium** - For-loop implementation cost model differs from contract text  
   - Architecture describes borrowed iteration.
   - Codegen borrows collection but clones list/dict elements (`.iter().cloned()`, `.keys().cloned()`).

5. **Medium** - Stdlib API design avoids ownership-heavy paths  
   - Many APIs that would naturally mutate in-place are expressed as copy-return/functional style.
   - This keeps code compiling but limits borrow-checker stress coverage.

## Module-Level Summary

- See `MODULE_OWNERSHIP_MATRIX.md` for all 37 modules with:
  - ownership style,
  - mutability behavior,
  - explicit ownership transfer support,
  - contradiction severity.
- See `CONTRADICTIONS_AND_REMEDIATION.md` for evidence and concrete follow-up plan.

## Overall Verdict

- **Does architecture cover current stdlib?** Partially.
- **Is stdlib fully aligned with borrow/ownership + safety contracts?** No.
- **Primary blockers to full alignment:** fallible intrinsic panic behavior, silent clone policy, and missing consumptive receiver path.

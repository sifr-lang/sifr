# M10 milestone review — pass 10

- Reviewer: Codex CLI
- Model: `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Reviewed HEAD: `7afbd5cd9026c50b995c8ab467d854a1c5f742cd`
- Scope: full M10 implementation, complete milestone history, and review passes 1–9
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — inconsistent source-name escaping for newtypes, enums, and protocols.**
   Definitions use `source_class_rust_name`, while type references, enum access,
   and protocol implementations can retain the source spelling. Legal
   `__Sifr*` declarations therefore pass `sifr check` and fail native Rust
   compilation.
2. **High — local `class std` captures relative compiler-owned standard-library paths.**
   Generated imports, types, expressions, traits, and string preambles still
   contain relative `std::...` paths, producing check/build parity failures.
3. **High — generated union type and variant names are source-collidable.**
   Short display-name concatenation permits both duplicate variants (`int | Int`)
   and generated/user type collisions (`int | str` beside `class IntOrStr`).

## Required remediation

- Centralize nominal Rust-name rendering and use it at every definition and
  reference site for regular classes, newtypes, enums, and protocols.
- Render compiler-owned `std` and external-crate paths absolutely across Rust IR
  expressions, imports, type strings, and generated preambles.
- Give generated union types and variants injective compiler-owned names derived
  from full canonical type identity.
- Add native check/emit/build regressions for every reproducer and the prior
  canonical Python Object/file-handle collision cases.

## Reviewer validation

- Type-system Python identity tests: 3/3 passed.
- Buffer lowering contracts: 39/39 passed.
- Buffer code generation: 10/10 passed.
- Python interop runner self-test passed.
- Prior native file-handle and Object collision regressions passed.
- Buffer runtime with matching CPython 3.13: 30/31 passed; the remaining
  `_ctypes` load failed because the local Homebrew library lacked
  `__PyErr_SetLocaleString`, classified as an environment/linker failure.
- File-size guard: 2693 files under 900 lines.
- HIR maintainability and `git diff --check`: passed.

Final reviewer verdict: `CHANGES REQUESTED — actionable findings remain`.

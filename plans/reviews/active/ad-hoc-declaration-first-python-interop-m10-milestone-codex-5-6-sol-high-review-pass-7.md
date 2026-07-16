# M10 Milestone Codex Review Pass 7

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations, range `e4fdc942ed..0e92e951c`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — coroutine conversion still recognized `Object` by basename.** A
   local record named `Object` passed checking but emitted
   `async_from_object`, `PythonAsyncType::Object`, and `async_to_object`, whose
   runtime contracts require the sealed handle.
2. **High — the sealed stdlib `Object` alias collided with a local record.**
   Importing `sifr.python` caused generated Rust to contain both the canonical
   `type Object = Handle<...>` and a user `struct Object`, so native compilation
   failed even though checking and Rust emission succeeded.
3. **High — callback error unions admitted duplicate generated variants.** A
   canonical `PythonError` imported under an alias plus a local same-named error
   passed checking but emitted an enum with two `PythonError` variants and two
   colliding payload type definitions.

## Reviewer validation

- All earlier writable-`Self`, duplicate-field, writable-owner transfer,
  canonical identity, and clean-vendor remediations were confirmed present.
- Focused compiler suites, the pinned CPython 3.11 buffer lane, vendor checksum
  inventory, HIR maintainability, and file-size guardrails passed.

## Required remediation

- Use the canonical sealed-object predicate throughout asynchronous conversion
  and semantic async-close lowering.
- Give the compiler-owned raw Python object a collision-proof generated Rust
  name while preserving source-level `sifr.python.Object` semantics, and lock
  the result with an actual native package build containing a local `Object`.
- Reject callback error channels whose distinct members map to one generated
  union variant, with lowering and driver check/compile parity regressions.

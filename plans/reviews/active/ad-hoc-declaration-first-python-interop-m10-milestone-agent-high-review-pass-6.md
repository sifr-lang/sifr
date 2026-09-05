# M10 Milestone agent Review Pass 6

- Reviewer: agent
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations, range `e4fdc942ed..bcce3be45`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — raw Python `Object` was recognized by basename.** A local record
   named `Object` could enter the sealed-handle check and code-generation path,
   producing a check/build mismatch when generated Rust passed the record to
   `temporary_argument_handle`.
2. **High — exact `PythonError` validation was confined to buffers.** Ordinary,
   callback, and context declarations still accepted a same-named error with an
   incompatible shape even though generated runtime-error mapping requires the
   exact five-field contract.

## Reviewer validation

- The prior writable-owner transfer, duplicate-safe `PythonError`, writable
  `Self`, tracking, and clean-checkout vendor remediations were confirmed fixed.
- Focused buffer lowering, codegen, type-system, driver parity, evidence,
  formatting, maintainability, and file-size checks passed independently.

## Required remediation

- Preserve the canonical stdlib declaration identity for the sealed raw Python
  handle through re-exports and use it consistently in lowering, recursive
  ownership analysis, callback typing, and code generation.
- Apply the shared exact `PythonError` predicate to ordinary, callback, and
  context declarations and add permanent check/build parity regressions outside
  the buffer-only path.

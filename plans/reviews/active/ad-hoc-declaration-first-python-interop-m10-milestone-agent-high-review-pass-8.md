# M10 Milestone agent Review Pass 8

- Reviewer: agent
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations, range `e4fdc942ed..e260661b9`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — the compiler-owned raw-object alias remained source-spellable.**
   A user class named `__SifrPythonObject` passed checking beside the canonical
   `sifr.python.Object`, but native Rust emitted both the compiler alias and the
   user struct under that name and failed with `E0428` and follow-on type
   errors.
2. **High — compiler-special `open()` still selected handles by basename.**
   An aliased canonical `TextFileHandle` import was rejected as unequal to the
   anonymous same-named fallback, while a local `TextFileHandle` shadow was
   incorrectly selected and passed checking before duplicate generated Rust
   definitions failed natively. The same basename lookup existed for binary
   `FileHandle`.

## Reviewer validation

- Re-grounded all prior milestone findings and the pass-7 remediations over the
  full implementation range.
- Passed the lowering buffer suite `39/39`, codegen buffer suite `10/10`,
  focused callback collision and local-Object native package regressions,
  type-system Python tests, and evidence-runner self-test.
- Passed the pinned CPython 3.11 lane with all five runtime exact-release tests
  and all five compiled buffer examples.
- Passed the exact vendor inventory audit (`442` manifests, `19,730` payloads),
  file-size guardrail (`2692` checked, limit `900`), HIR/driver
  maintainability, formatting, and diff checks.

## Required remediation

- Represent canonical `sifr.python.Object` directly with the fully qualified
  runtime handle type, emitting no compiler alias into the flat user Rust
  namespace. Lock this with an exact-name native regression for
  `class __SifrPythonObject`.
- Resolve text and binary `open()` result classes only by canonical declaration
  identity, including aliased imports, and never reuse a local same-basename
  class. Lock both alias success and local-shadow rejection.

# M10 Milestone agent Review Pass 9

- Reviewer: agent
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus all prior milestone-review
  remediations at committed HEAD `c0c9ea917`
- Review tree: clean detached worktree
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — generated runtime paths remained source-collidable.** A legal
   source class named `sifr_runtime` passed checking beside canonical
   `sifr.python.Object`, but relative generated paths such as
   `sifr_runtime::python::...` became ambiguous and failed native compilation.
2. **High — canonical `open()` handles still lost their identity during Rust
   naming.** Local `FileHandle` and `TextFileHandle` classes could coexist with
   inferred canonical `open()` results during checking, but flattened stdlib
   definitions and generated binding annotations reused the local basenames and
   failed native compilation.

## Reviewer validation

- Re-grounded the complete M10 diff and every prior review remediation.
- Proved aliased canonical binary and text handles build successfully and
  explicit assignments to local shadows are rejected.
- Proved unannotated canonical binary and text handles pass checking beside
  local same-basename classes but fail native compilation.
- Proved a package importing canonical `Object` and declaring local
  `sifr_runtime` passes checking but fails native compilation.
- Re-ran focused lowering, codegen, runtime, evidence, formatting, inventory,
  and maintainability checks before issuing the findings.

## Required remediation

- Render every generated external runtime/stdlib crate path absolutely,
  including assembled stdlib and generated imports, and lock the exact
  `class sifr_runtime` package build.
- Give canonical file-handle declarations and references compiler-owned Rust
  names without discarding nominal identities in alternate type renderers; lock
  inferred binary and text coexistence in lowering, codegen, and native builds.

# Native Pydantic-Sifr PS3 Claude Opus Review Pass 2

## Candidate

- Base: `f8bda6c8170c3078c7d0447ad5f7196b2443c2fa`
- Candidate: `efc56b55c2e640f12a06d6bfb421b03acd51aceb`
- Pull request: [#3138](https://github.com/sifr-lang/sifr/pull/3138)
- Verdict: `NOT SATISFIED`

## Closed Findings

Opus confirmed that the candidate closed both pass 1 blockers. Typed program
emission now uses structural implementation eligibility. Lowering rejects a
second `@const_specialize` decorator.

Opus also confirmed deterministic cache ordering, independent format and bridge
contract checks, direct second-move evidence, and the 899-line driver test root.

## Blocking Finding

The candidate admitted `bytes` in all structural positions, but codegen used the
byte-buffer scalar only for a direct record field. A nested `bytes` value fell
through the blanket `Vec<T>` implementation. It therefore projected as a
sequence of unsigned integers even though its compiler shape was `bytes`.

## Remediation

The next candidate keeps one byte-buffer encoding. It supports `bytes` as a
direct record field, where codegen has complete construction and projection
support. It rejects nested `bytes` through the `StaticProgram` protocol bound.
Codegen eligibility uses the same rule. Focused tests cover direct acceptance and
nested rejection in lowering and codegen.

Focused remediation validation passed:

- codegen direct-byte acceptance and nested-byte rejection: 1 test;
- lowering direct-byte acceptance and nested-byte protocol rejection: 1 test;
- exact positive generated package: 1 test in 22.58 seconds;
- affected lowering, codegen, and driver clippy with warnings denied;
- formatting, fixture matrices and self-tests, compatibility, tiers, stable
  claims, taxonomy, file-size, and lowering maintainability checks.

Review pass 3 must inspect and validate the exact remediation commit.

# Native Pydantic-Sifr PS3 agent Review Pass 1

## Candidate

- Base: `f8bda6c8170c3078c7d0447ad5f7196b2443c2fa`
- Candidate: `c5cb7239435e9d53ef8a69cb5cfcfd5b808e39b7`
- Pull request: [#3138](https://github.com/sifr-lang/sifr/pull/3138)
- Verdict: `NOT SATISFIED`

## Blocking Findings

agent found two generated-Rust failure paths.

First, the compiler emitted `StaticProgramType` for every retained specialization
when a project had structural demand. An unsupported specialization owner could
therefore receive an implementation whose `StructuralType` supertrait did not
exist. The generated crate then failed with a Rust compiler error instead of a
Sifr diagnostic.

Second, two `@const_specialize` decorators on one class produced duplicate
static items and conflicting trait implementations.

agent also found that equal readable specialization keys retained `HashMap`
iteration order in the project cache fragment. This could cause a spurious cache
identity change.

## Remediation

The next candidate makes typed program emission use the same owner eligibility
predicate as structural implementation emission. It does not emit an impossible
typed implementation for an unsupported owner. Retained static bytes remain
valid for specialization consumers that do not request the structural program
contract. Predicate differences that existed before this item remain separate
follow-up work.

Lowering now rejects a second `@const_specialize` decorator on the same class.
The cache fragment now sorts tied readable keys by the complete program identity,
which already includes the declaring module.

The remediation also exports the format and bridge contract constants for an
independent package-side envelope check. It adds a direct second-move rejection
test for arena scalars and keeps the touched driver test root below 900 lines.

Focused remediation validation passed:

- static-program codegen and eligibility: 5 tests;
- duplicate-decorator and unsupported-owner diagnostics: 2 tests;
- arena and envelope runtime contracts: 5 tests;
- exact positive generated package: 1 test in 356.04 seconds;
- corrupt-envelope generated package: 1 test in 10.73 seconds;
- targeted compiler/runtime clippy with warnings denied;
- formatting, fixture matrices and self-tests, compatibility, tiers, stable
  claims, taxonomy, file-size, and lowering maintainability checks.

Review pass 2 must inspect and validate the exact remediation commit.

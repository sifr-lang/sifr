# Ad Hoc Phase: Create-PR Interop Budget Achievability

Status: in progress.

GitHub issue: [#3130](https://github.com/sifr-lang/sifr/issues/3130).

## Problem

The create-PR profile can reject fully passing Python and Rust interop work on
the supported local Mac. A successful-input Python run took 657.733 seconds
against its 600-second warm bound. A Rust interop run took 14.067 seconds
against its fixed 10-second bound, while an immediate hot run took 7.245
seconds. The current policy therefore depends on incidental cache warmth and
can block unrelated changes after every functional variant passes.

This is a post-closure regression found after PERF-HOST completed. It must not
reopen or weaken the governed representative performance budgets.

## Constraints

- Do not add or extend a waiver or deferral.
- Do not hide, skip, or reclassify required variants.
- Do not add an ad hoc caller-side budget or worker override.
- Do not require repeated whole-profile runs to obtain a passing cache state.
- Keep functional per-case timeout guards independent from aggregate
  performance policy.
- Use the governed profile and local-Mac resource policy as the authority.

## Execution Plan

| Milestone | Scope | Status |
| --- | --- | --- |
| M1: current-main diagnosis | Measure cold and successful-input states, inspect cache receipts and shared-cache completeness, and identify avoidable serialized or repeated work | complete |
| M2: controlled policy/work repair | Make the required work achievable through governed resource use, cache-state policy, or removal of redundant work; add deterministic regression tests | in progress |
| M3: qualification and closure | Reproduce cold and warm passing verdicts, run create-PR and merge gates, obtain Claude Opus satisfaction, merge, and archive this record | pending |

## Initial Evidence

On base `dfa63d3cb8cb362464f768f4a80c25770bbe06bf` plus dependent candidate
`0ff5e6ea9f800c1c292720a5c974d9c1a24354d3`:

- Python cold: 19/19 passed in 1,234.032 seconds against 1,200 seconds, with
  inherited package-cache lock waits.
- Python functional retry: 19/19 passed in 709.318 seconds.
- Python successful-input warm: 19/19 passed in 657.733 seconds against 600
  seconds.
- Rust interop: 10/10 passed in 14.067 seconds against 10 seconds.
- Immediate Rust hot control: 10/10 passed in 7.245 seconds.

Current `main` includes later interop probe-cache repairs. M1 must measure that
exact source before deciding whether Python still needs a work repair. The
independently reproduced Rust cold-cache failure remains in scope.

An independent current-main qualification then established the actionable
states. The first exact-input Python run passed 19/19 in 1,021.915 seconds
under the 1,200-second cold bound. Its next successful-input run passed 19/19
in 618.293 seconds, exceeding the 600-second warm bound. The cold Rust area
passed 10/10 in 6.543 seconds, while the dependent 14.067-second cold result
still proves that a fixed 10-second bound conflates cold and warm states.

The Python compiled-example runner was re-entering Cargo for every Sifr
command even though the profile had already resolved and exported the exact
Sifr binary. The repair launches that profile-owned binary directly. The Rust
step adopts the same successful-input receipt classifier as Python: the
10-second warm bound remains unchanged and the explicit cold bound is 20
seconds.

## Scope

- Create-PR Python and Rust interop step execution and cache classification.
- Profile-owned resource allocation used by those steps.
- Deterministic policy, receipt, and runner self-tests.
- Documentation and phase records for the final mechanism.

Product feature behavior, representative performance baselines, unrelated
validation lanes, and dependent PR implementation are out of scope.

## Definition of Done

- Every selected Python and Rust interop variant remains blocking and passes.
- One cold current-main run and one successful-input warm run pass their
  governed policy without caller overrides.
- A seeded over-budget or invalid-cache result still fails deterministically.
- The result is robust to ordinary unrelated host load under the local CPU
  partition; passing does not depend on a perfectly idle desktop.
- The canonical create-PR and merge gates pass on the final reviewed SHA, or
  an independently owned failure is recorded and isolated under the closure
  workflow.
- Claude Opus returns `SATISFIED` with no blocking findings on the complete
  final implementation.

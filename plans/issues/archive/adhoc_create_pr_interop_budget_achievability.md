# Ad Hoc Phase: Create-PR Interop Budget Achievability

Status: complete.

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
| M2: controlled policy/work repair | Make the required work achievable through governed resource use, cache-state policy, or removal of redundant work; add deterministic regression tests | complete |
| M3: qualification and closure | Reproduce cold and warm passing verdicts, run create-PR and merge gates, obtain agent satisfaction, merge, and archive this record | complete |

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
area does not produce or consume a durable step-owned warm cache: it validates
repository evidence and performs one locked offline Cargo fetch. Its fixed
10-second aggregate wall bound therefore conflated a hot control with the
required first-run work. The repair uses one explicit 20-second blocking bound
for all Rust interop runs, covering the observed 14.067-second first run
without inventing a cache state the step does not own.

The dependent packaged-candidate merge gate then exposed a policy cycle after
all Python, Rust, and developer-tooling lanes passed. Work-controlled
representative sampling admitted external CPU observations from 380% to 734%
on the 10-thread Mac because its external CPU limit was disabled. Three
formatter attempts exceeded the `0.02` instruction-variation limit and stopped
the gate. M2 therefore also owns the narrow work-mode admission correction:
reserve 60% of logical CPU capacity for the measured process tree at admission,
retain continuous external-CPU telemetry, and keep the lagging one-minute load
limit disabled in work mode. A qualification run proved that rejecting any
single in-attempt pressure spike was too strict: two retries had instruction
CVs of `0.000433` and `0.000389`, yet transient external activity above 40%
rejected both. Stable work samples now retain pressure as advisory evidence;
pressure remains blocking when the instruction samples are unstable.

## Qualification Evidence

The work-mode admission repair passed its review and qualification in PR
[#3132](https://github.com/sifr-lang/sifr/pull/3132). Its reviewed head was
`34022b32820f212919aba409621fd78424df84af`. Main contains merge commit
`8d1c71150e8e2a5718f0e85bb3d4166de3dc0521`.

The dependent packaged-candidate gate then passed on reviewed head
`eb0599f4f8ed06c1fae5e0055116849e3f4616e3`. The gate passed all ten
representative benchmarks in 166.865 seconds. It also passed 694 E2E fixtures
and 268 hardening variants. PR
[#3102](https://github.com/sifr-lang/sifr/pull/3102) merged as
`3426c7c53025c867c565cb6981cad3d1695b045b`.

The interop qualification used exact candidate
`912947f895837da272381d2758c275f8a08ca9d1`. The local partition assigned one
CPU to qlty and six workers to Sifr. The Mac used AC power.

- The cold receipt used reason `input-changed` and fingerprint
  `c128e13e246c5729`.
- Cold Python interop passed 19/19 in 254.255 seconds under 1,200 seconds.
- Cold Rust interop passed 10/10 in 6.656 seconds under 20 seconds.
- The unchanged warm receipt used reason `successful-input-receipt` with the
  same fingerprint.
- Warm Python interop passed 19/19 in 226.054 seconds under 600 seconds.
- Warm Rust interop passed 10/10 in 6.881 seconds under 20 seconds.
- The warm create-PR profile passed all blocking lanes, including E2E 140/140.
- The warm lane report digest is
  `4efeecdc2227e59d2d09ed16eddc38848c13cd2f93bd864e55376d3fb6392a17`.

The cold profile later stopped in an unrelated generated-code cache miss. Its
lane took 137.870 seconds against 120 seconds. The unchanged warm lane passed
in 9.457 seconds. Issue
[#3134](https://github.com/sifr-lang/sifr/issues/3134) owns this external
create-PR cold-artifact problem.

The final reviewed candidate was
`7b026aca8857d4a60844eecdaca2c768cb03ac38`. Its records-only changes did not
change any interop qualification input.

## Final Review and Merge Evidence

agent reviewed base
`c3d347d7f732fef320a3e971ab91f7c18bc908ae` and final candidate
`7b026aca8857d4a60844eecdaca2c768cb03ac38`. The reviewer returned
`SATISFIED` with no blocking findings. The review response digest is
`32ed1edfbeb774b6a1ba14859601c650bc689f4f3571a7997302ad0b4969a4b2`.

The one authoritative merge gate ran on the same final candidate. It passed
with exit code zero. Key results were:

- Python interoperability: 25/25.
- Rust interoperability: 10/10.
- Representative performance: 10/10.
- Full E2E: 694/694.
- Hardening: 268 variants with zero failures.

The merge report digest is
`932da65248e56bbb7a6534ed866ce2bf2f741a116ce68cc8cad04a48ac9e6eeb`.
The representative-performance evidence digest is
`dae0c13a139dce7484ce5b6a559042ac9afb3f0451a15bd46e431ba48aaec970`.

Implementation PR [#3131](https://github.com/sifr-lang/sifr/pull/3131)
merged the reviewed candidate. Main contains merge commit
`c026b14ba60b5dd09f1ab46e427572b8435ec571`.

Issue [#3134](https://github.com/sifr-lang/sifr/issues/3134) owns the separate
generated-code cold-artifact budget. It is not an interop phase blocker.

## Scope

- Create-PR Python and Rust interop step execution and cache classification.
- Profile-owned resource allocation used by those steps.
- Work-controlled representative-performance CPU-capacity admission and
  per-attempt contamination monitoring.
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
- agent returns `SATISFIED` with no blocking findings on the complete
  final implementation.

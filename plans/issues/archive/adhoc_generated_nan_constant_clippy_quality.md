# Ad Hoc Phase: Generated NaN Constant Clippy Quality

Status: complete (2026-08-11). Implementation
[#3103](https://github.com/sifr-lang/sifr/pull/3103) merged as
`b0d5fe67c4190bc85bba43d5ddbebb61444eb9ef`.

The final reviewed and gated candidate was
`921d19bad6b996e3f7180face318756b7c1e7be6`. Its corrected base was
`c8114ab92b3b96f73207d158e32c6c0c25cc3ea4`.

## Resolution

The compiler now folds annotated non-finite float division constants at the
module-constant boundary. Code generation emits `f64::NAN`, `f64::INFINITY`,
and `f64::NEG_INFINITY`. Finite float division stays unchanged.

The release profile now selects `generated_code_quality:full`, which matches
nightly. PR #3103 removed the `release-full` suite, the GENC-NAN divergence,
and all associated policy and matrix bindings. No Clippy allow was added.

## Closure Evidence

- Focused lowering, codegen, and stdlib integration tests passed on the final
  candidate. The complete generated-code Clippy corpus also passed with zero
  failures.
- The complete Clippy evidence has SHA-256
  `17b8369abfe24e50d476914cb52efcc53ff709d6513b10f5419ece2f6d647388`.
  Its area result has SHA-256
  `0970fcd48bfd9e487977682a4d836dfbda61dc294fce619ba37a3278df40bf85`.
- The required cold create-PR run passed every functional check. The known
  two-miss cold artifact condition exceeded only the warm-cache step budget.
  Its receipt has SHA-256
  `cf7a2b53e429e82694117c2e4bf5409bc00cc8cc3c1e8beb463ec53bca402715`.
- The unchanged warm create-PR run exited 0. Generated-code quality passed
  5/5, Python interop passed 19/19, Rust interop passed 10/10, and E2E passed
  140/140. Its receipt has SHA-256
  `b66324f0e626aabbadaac8cb390375b2387fb10d3e2da665c49e71a38c1adc7e`.
- Claude Opus returned `SATISFIED` on the final candidate with no blocking
  findings. The response has SHA-256
  `f221135d4c7c45b27cf6d1fbe450974cf54f5d3cee4d2e6429b6718419b4e8e9`.
- The authoritative merge gate exited 0 on the same candidate. Generated-code
  quality passed 7/7, generated builds passed 70/70, and E2E passed 694/694.
  Hardening passed 268 variants with zero failures. The merge receipt has
  SHA-256
  `e34742825e24350d8b1f528134e5dd876366147134819b7fda0caa5d4117c9d0`.
  The merge log has SHA-256
  `c6d9383bf33d36e2501adf23c7a66a432ad1047b85c61eabbf305a37afa39f8e`.
- Both milestone reviews and the final implementation review were satisfied.
  No implementation changed after the final review.

## Remaining Work

No in-scope remediation, Clippy allow, release divergence, or deferred blocker
remains. Follow-up [#3141](https://github.com/sifr-lang/sifr/issues/3141)
tracks non-finite local expressions. Follow-up
[#3142](https://github.com/sifr-lang/sifr/issues/3142) tracks non-finite float
match patterns. Both problems are pre-existing and outside this phase.

## Historical Record

## Problem

The generated-code quality release gate rejects the emitted Rust representation
of the Sifr `math.nan` constant in the CPython math semantic-corrections
fixture. The generated constant is:

```rust
const NAN: f64 = (0.0_f64) / (0.0_f64);
```

Rust 1.94 Clippy reports `clippy::zero_divided_by_zero` under the governed
generated-code `-D warnings` policy and recommends the canonical `f64::NAN`
constant.

The fixture compiles, formats, and emits deterministically. This is a generated
stdlib constant quality failure, not a Phase 40 stable-channel governance or
release-artifact failure.

## Evidence

On exact Phase 40 source commit
`8a23f90869a68438a7b4ae3b8f9623531d1ce68f`, the unchanged canonical release
profile:

- passed the full performance area (8 variants);
- passed the complete distribution-release area (69 variants);
- passed all 25 selected Python-interop variants;
- consumed and passed all 10 selected Rust-interop variants;
- passed all 48 developer-tooling variants;
- passed both GA documentation variants;
- passed generated-code corpus, panic scan, intrinsic panic lint, rustfmt,
  determinism, and demos;
- reached its first generated-code Clippy failure at
  `e2e-018-cpython-math-semantic-corrections`, where the Clippy variant stopped.
  Entries later in the ordered corpus were not Clippy-checked by this run and
  must be exercised after the first failure is fixed.

The original release profile therefore exited with the blocking
`generated_code_quality_checks` step failed (`blocking_failures=1`) and no
release-profile report was emitted. It also reported the separate advisory
that the warm wall-time target was exceeded.

Phase 40 consumes the expiry-bound `generated_code_quality:release-full`
suite. That suite runs every full generated-code gate and every Clippy entry,
but treats the three exact entries that materialize this same generated
constant as required expected failures:

- `e2e-018-cpython-math-semantic-corrections`
- `e2e-027-error-mixed-builtin-stdlib`
- `stdlib-007-math`

Each entry must still fail with exactly `clippy::zero_divided_by_zero`; an
unexpected pass, a different lint, an expired or missing record, or any failure
in another entry fails release qualification. Nightly continues to run
`generated_code_quality:full` with no divergence and remains red on these entries.
The release divergence expires on 2026-10-31 and is mechanically cross-bound to
the indexed `GENC-NAN` record.

The generated crate containing the offending code is:

`/private/tmp/sifr-phase40-release-source-8a23f908-20260728T165853Z/target/sifr_generated_code_quality/release.shared/entries/e2e-018-cpython-math-semantic-corrections-0108e9606ab793b2/sifr_output`

The authoritative run log is:

`/tmp/sifr-phase40-ga-release-profile-retry-7.log`

No Clippy allow, performance waiver, source baseline, or generated Rust was
changed. The release profile uses a distinct suite only to enforce this exact,
reproduced lint through the visible, expiry-bound divergence above; its gate and
corpus breadth remain identical to full mode.

## Scope

- Identify the canonical Sifr-to-Rust representation for non-finite float
  constants, including the `stdlib/_sifr/math.sifr` definitions that currently
  express NaN and infinity through division.
- Emit `f64::NAN` for the Sifr `math.nan` constant without changing its Sifr
  runtime semantics.
- Audit positive and negative infinity constants for the same canonical
  representation boundary.
- Add focused codegen coverage for NaN and infinity constant emission.
- Run the focused generated-code Clippy group and the complete generated-code
  quality area.
- Confirm the governed generated-code Clippy allowlist does not gain
  `clippy::zero_divided_by_zero`.

## Out of Scope

- Renaming the CPython math semantic-corrections fixture or any demo.
- Removing the fixture from nightly generated-code quality coverage.
- Adding a broad Clippy allow for constant division by zero.
- Changing Phase 40 release-governance schemas, workflows, artifacts, or
  publication policy.

## Definition of Done

- Non-finite Sifr float constants emit canonical Rust constants.
- The focused math semantic-corrections fixture passes generated-code Clippy
  with `-D warnings`.
- The complete generated-code quality area passes without a new allow.
- Codegen tests cover NaN and both infinity signs.
- `generated_code_quality:release-full` is removed and the release profile
  returns to `generated_code_quality:full`.

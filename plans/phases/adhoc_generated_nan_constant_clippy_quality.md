# Ad Hoc Phase: Generated NaN Constant Clippy Quality

Status: deferred follow-up; not a prerequisite for Phase 40.

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

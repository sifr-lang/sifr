# Rust Interop Hardening 1 Corrective Review — Round 1

## Context

This review covers the narrow corrective delta against `origin/main` after
PR #3018 merged. The delta closes fresh-clone and fail-closed evidence gaps
that were present in the merged implementation.

## Reviewer

agent, invoked through the repository's
`agent review` workflow at medium effort.

## Verdict

**APPROVED**

No blocking or actionable correctness findings remain.

## Verified Findings

- The fixture validator requires
  `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`,
  but the file is absent from `origin/main` because the repository-wide
  `**/Cargo.lock` ignore rule swallowed it. A fresh clone therefore fails the
  authoritative Rust-interop matrix even though an existing worktree can pass
  with an untracked lockfile.
- The `.gitignore` exception and tracked lockfile fix the root cause.
  `cargo metadata --locked --offline` succeeds for the fixture and leaves the
  lockfile byte-identical.
- Changing an empty Rust-interop suite selection from a skip to a
  `ProfileRunnerError` is safe. The selected-areas-only Python interop profile
  exits before legacy-facade steps, while every normal profile is required to
  select the full Rust-interop suite set.
- Exact `schema_version: 1` and `bless: false` checks match the area runner's
  emitted result contract.
- The integer validators correctly reject Python booleans, JSON floats, and
  strings where positive or zero integer evidence is required.
- The corrective scope is tight: one ignore exception, one fixture lockfile,
  result validation hardening, and mutation-test coverage.

## Non-Blocking Observations

- A future follow-up could directly exercise the otherwise unreachable
  no-suite runner branch and add explicit JSON-float mutation cases.
- Summary-only counters and malformed extra suite entries could be validated
  more exhaustively if the result schema later makes those fields independent
  authorities.
- The fixture lockfile ignore exception may be narrowed if in-tree Cargo probes
  later create unrelated lockfiles.

These observations do not affect the current verdict because the implemented
contracts reject the identified malformed values, profile validation makes the
empty-suite path unreachable in accepted normal profiles, and required suite
entries remain the authoritative failure evidence.

## Validation Assessment

The exact reviewed implementation tree passed:

- `scripts/run_all_tests.sh` with every merge-profile lane step passing;
- `rust_interop_checks`: 4 variants, 0 failures, 385 ms;
- representative performance budgets;
- full E2E: 674 passed, 0 failed;
- verification hardening: 261 variants, 0 failures; and
- the verification-runner mutation self-tests.

The reviewer additionally verified the tracked-file state, ignore behavior,
profile-mode control flow, and offline Cargo metadata behavior independently.

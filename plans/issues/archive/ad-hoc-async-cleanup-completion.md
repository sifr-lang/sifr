# Ad hoc issue: Complete abnormal async cleanup

Status: complete and merged on 2026-08-30

Owner: core language and async runtime

## Objective

Complete the accepted async resource contracts that SQL transactions and streams
require before runtime implementation starts.

## Owning architecture

- `internal_docs/async_concurrency_model.md`
- `internal_docs/architecture.md`

## Scope

- abnormal body exit through `async with`
- cancellation-specific `AsyncExitCause` delivery
- secondary cleanup evidence for errors and timeouts
- `AsyncClosable.aclose()` on early `async for` exit

## Acceptance criteria

- [x] `async with` calls `__aexit__` for return, ordinary error, timeout,
  cancellation, and runtime-fault paths.
- [x] `__aexit__` receives the exact accepted `AsyncExitCause` value.
- [x] Cleanup errors preserve the primary result and add typed secondary evidence.
- [x] Cleanup timeout records `SecondaryError.CleanupTimedOut` and invalidates the
  resource.
- [x] Early `async for` exit calls `AsyncClosable.aclose()` exactly once.
- [x] Nested cleanup uses last-in, first-out order for every exit path.
- [x] Compiler, lowering, code generation, runtime, and end-to-end evidence pass.
- [x] An exact-SHA review approves the implementation.

## SQL dependency

Milestone `sql_9_postgresql_runtime` can start from merge commit
`0f01971c4d00cdf7e888360fc79c2703cbafb327`. The SQL verification inventory
records the same evidence.

## Implementation map

- HIR records the active error type and body error effect for native `async
  with` and closable `async for`.
- Native `async with` code generation owns entry cancellation, concrete exit
  classification, nested control-flow propagation, bounded cleanup, and
  runtime-fault cleanup.
- Closable `async for` code generation owns normal exhaustion, `continue`,
  early `break`, return, propagated error, timeout, cancellation, and runtime
  fault as distinct outcomes.
- `sifr_runtime::async_cleanup` owns panic-safe future polling and the closed
  cleanup-evidence variants.
- `CancellationCarrier` owns ordered evidence until the generated task
  observation boundary drains it into `Failure.secondary`.
- A cleanup timeout drops the cleanup future and records a discard signal. A
  resource provider must invalidate that resource and must not return it to a
  pool.

## Closure evidence

- Starting commit: `755e9bb1c3b7840c35dd408267202d9654cea0c6`.
- Initial reviewed candidate: `fc596512f5d166e09050f6538a5cf459992441c2`.
- Final candidate after mainline reconciliation:
  `46bbd40c8bca7538c8331f2ff3f891a98b2e9c88`.
- Pull request: [#3607](https://github.com/sifr-lang/sifr/pull/3607).
- Merge commit: `0f01971c4d00cdf7e888360fc79c2703cbafb327`.
- `cargo test -p sifr_runtime --lib`: passed, 86 tests.
- `cargo test -p sifr_codegen --lib`: passed, 1,177 tests.
- `cargo test -p sifr_lowering --lib`: passed, 1,073 tests and one ignored test.
- All 19 direct native `async_with`, closable `async_for`, and cleanup-evidence
  fixtures passed on the final candidate.
- `cargo fmt --check`, `git diff --check`, the HIR maintainability guardrail,
  and the changed-source file-size guardrail passed.
- Targeted Clippy reached the changed crates on the initial candidate. It found
  only unrelated existing findings in `annotation_resolution.rs` and
  `structural_record_codegen.rs`.
- The one create-PR gate ran on the initial candidate. It stopped before tests
  because `create-pr` omits required `postgresql-live-differential`.
- The one merge gate ran on the same candidate. It stopped before tests because
  `merge` omits the same required suite.
- Main advanced before merge. The branch accepted upstream changes and resolved
  one module-list conflict. The one-gate rule prohibited another gate.
- The [initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3607#issuecomment-5470681745)
  returned `SATISFIED` with no blocking findings.
- The [reconciliation review](https://github.com/sifr-lang/sifr/pull/3607#issuecomment-5470776353)
  returned `SATISFIED`. It confirmed that no approved item hunk changed or
  disappeared.
- The [gate record](https://github.com/sifr-lang/sifr/pull/3607#issuecomment-5470684493)
  records the external verification-profile blocker. SQL Milestone 18 owns it.

## Deferred follow-up

The non-blocking review suggestions are tracked in
`plans/issues/active/ad-hoc-async-cleanup-review-follow-ups.md`. They do not
invalidate this issue's accepted and merged contract.


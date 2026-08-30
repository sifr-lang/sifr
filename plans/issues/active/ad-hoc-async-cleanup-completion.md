# Ad hoc issue: Complete abnormal async cleanup

Status: implementation in progress

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

- [ ] `async with` calls `__aexit__` for return, ordinary error, timeout,
  cancellation, and runtime-fault paths.
- [ ] `__aexit__` receives the exact accepted `AsyncExitCause` value.
- [ ] Cleanup errors preserve the primary result and add typed secondary evidence.
- [ ] Cleanup timeout records `SecondaryError.CleanupTimedOut` and invalidates the
  resource.
- [ ] Early `async for` exit calls `AsyncClosable.aclose()` exactly once.
- [ ] Nested cleanup uses last-in, first-out order for every exit path.
- [ ] Compiler, lowering, code generation, runtime, and end-to-end evidence pass.
- [ ] An exact-SHA review approves the implementation.

## SQL dependency

Milestone `sql_9_postgresql_runtime` cannot start until this issue has merged
evidence. The SQL phase records that merge in its verification inventory.

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

## Verification record

Candidate SHA: pending

- `cargo test -p sifr_runtime --lib`: passed, 86 tests
- `cargo test -p sifr_codegen --lib`: passed, 1,172 tests
- `cargo test -p sifr_lowering --lib`: passed, 1,072 tests and one ignored test
- Direct native fixtures passed for every accepted exit class, nested LIFO
  cleanup, typed secondary evidence, cleanup panic, cleanup timeout, and task
  timeout evidence.
- `cargo fmt --check`, `git diff --check`, and the HIR maintainability
  guardrail passed.
- The file-size guardrail passed for every changed hand-maintained source file.
- Targeted Clippy reached the changed crates. It remains blocked by unrelated
  existing findings in `annotation_resolution.rs` and
  `structural_record_codegen.rs`.
- The manifest E2E runner remains blocked by the existing SQL verification
  profile contradiction. The runner requires `postgresql-live-differential`,
  but the `create-pr` profile omits that suite.
- exact-SHA Opus review: pending
- create-PR gate: pending
- merge gate: pending
- PR and merge evidence: pending

# Ad hoc issue: Async cleanup review follow-ups

Status: pending

Owner: core language and async runtime

## Objective

Resolve non-blocking maintainability, evidence-lifecycle, and coverage findings
from the abnormal async cleanup review.

## Scope

- remove the unused legacy async-cleanup emitter
- consume or remove unused `body_may_raise` HIR metadata
- use a stable Rust representation for `Never` in closable `async for`
- define cleanup-evidence behavior when a task otherwise succeeds
- align union-typed async-for evidence labels with async-with labels
- add direct cause-delivery and nested exit-path coverage

## Acceptance criteria

- [ ] The unused `stmt_support_emitter/async_cleanup.rs` module and its module
  declaration are removed.
- [ ] Native async cleanup either consumes `body_may_raise` or removes it from
  HIR and snapshots.
- [ ] Closable `async for` maps `Never` to `std::convert::Infallible`. A native
  fixture covers an infallible enclosing async function.
- [ ] Successful task completion cannot silently discard cleanup evidence or
  attach stale evidence to a later failure.
- [ ] Union-typed async-for failures produce canonical evidence labels.
- [ ] A native fixture inspects each delivered `AsyncExitCause` value.
- [ ] Nested cancellation, timeout, and runtime-fault fixtures prove last-in,
  first-out cleanup.
- [ ] Focused compiler, runtime, and native fixture suites pass.

## Evidence source

- [Initial review](https://github.com/sifr-lang/sifr/pull/3607#issuecomment-5470681745)
- [Reconciliation review](https://github.com/sifr-lang/sifr/pull/3607#issuecomment-5470776353)

## SQL integration note

Milestone `sql_9_postgresql_runtime` must not rely on unspecified successful-task
cleanup evidence. It can close that row here or use an explicitly proved runtime
boundary with the same semantics.


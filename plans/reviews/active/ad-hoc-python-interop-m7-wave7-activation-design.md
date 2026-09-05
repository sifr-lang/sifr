# M7 Wave 7 Design: Atomic Async Declaration Activation

## Objective

Activate `@python.coroutine(path)` and `cleanup=async_close` together on the
single production lowering/runtime path completed by M7 waves 1-6, close the
remaining cancellation semantic gap, and attach executable evidence before
changing the capability ledger from `reserved` to `active`.

## Activation Boundary

1. Remove only the two M7 reservations:
   - do not emit `SIFR-PYRES-0002` for a valid `@python.coroutine(path)`
     function or method;
   - do not emit `SIFR-PYRES-0002` for
     `@python.opaque(..., cleanup=async_close)`.
2. Retain all shape, conversion, effect, ownership, must-use, and sync/async
   substitution diagnostics. Later decorators remain reserved.
3. Replace the gated lowering tests with active positive and negative tests.
   In particular, valid bodyless async declarations must lower through the
   existing Python interop `Bodyless` path and consuming `aclose` must discharge
   the affine obligation exactly once.

## Active Cancellation Classification

The cancellation carrier already gives one generated Sifr task exclusive
authority over one registered asyncio task, and terminal completion already
waits for Python `finally`. Wave 7 adds no second cancellation mechanism.

1. Record whether the submission bridge received the carrier's cancellation
   request. The typed done callback still obtains the exact Python terminal
   outcome on the owned loop thread.
2. If, and only if, that submission was actively cancelled and the terminal
   exception is `asyncio.CancelledError`, complete the terminal with a private
   `ActiveCancellation` outcome instead of constructing `PythonError`.
   An independently raised `CancelledError` without a Sifr cancellation request
   remains an ordinary declared Python failure.
3. Change submission-start plumbing to carry a private terminal/start error
   enum. `CancellationClaimError::CancelledBeforeClaim` becomes the same
   private `ActiveCancellation` outcome for typed declarations rather than the
   catchable `AsyncSubmissionCancelled` `PythonError`. The raw blocking API
   deliberately maps this private start outcome back to its existing raw
   `PythonError` contract.
4. The terminal releases its exact-task cancellation lease before waking the
   Sifr waiter, as today. On `ActiveCancellation`, the waiter asks the same
   carrier to invoke its already-bound Tokio abort fallback now that Python is
   terminal. It then yields once through an executor-neutral self-waking
   future: the production Tokio task is aborted at that suspension point
   without adding Tokio to the Python-only runtime feature; if a malformed or
   no-op fallback lets the yield return,
   the waiter reports an explicit internal runtime error rather than hanging.
   This makes the generated task end through its native cancellation path
   without exposing a catchable declaration error. There is no panic, sentinel
   user value, or alternate task representation.
5. Add one explicit, idempotent carrier operation with a closed result enum:
   `Invoked`, `AlreadyResumed`, `NotRequested`, `ExactClaimActive`,
   `FallbackUnavailable`, or `StateUnavailable`. It may invoke the fallback at
   most once and only when cancellation was requested, the exact claim has been
   released, and a fallback is bound. Every non-`Invoked` result is handled
   explicitly. `AlreadyResumed` also reaches the one-yield termination check
   for the cancel-before-claim race; malformed production usage becomes a
   runtime error.
6. Terminal completion wins its race atomically: `PythonTerminal::complete`
   stores the already-classified terminal outcome and removes the exact claim
   under one terminal-state lock before dropping the claim and waking the
   waiter. A cancellation request that reaches the exact hook before claim drop
   is observed by the bridge, but cannot rewrite an already stored successful
   or non-cancellation outcome. A request after claim drop uses the native
   fallback. Thus there is no window that loses cancellation authority and no
   stored value is retroactively reclassified.
7. If Python suppresses cancellation and returns a value, output conversion and
   the value win normally. If it suppresses cancellation and raises a different
   exception, that exception becomes the declared `PythonError`. Async semantic
   close uses the same terminal classification; observed active cancellation
   poisons the object, while suppression followed by clean `None` closes it.

The raw coroutine API keeps its documented synchronous `blocking_io` result
surface. Active Sifr task cancellation classification is applied to genuine
typed async declarations, where the generated task carrier and native task
cancellation cause exist.

## Executable Evidence

Add a checked-in `async_declaration` verification fixture and an offline
compiled-example runner using the area-local locked Python environment.

- A package-local bridge implements an httpx-style async client with async
  factory, borrowed request method, consuming `aclose`, deterministic loop and
  thread identity, failure, conversion-failure, cancellable/finally, and
  cancellation-suppression targets. It performs no external network access.
- Compiled Sifr programs cover successful values, Python failure, output
  conversion failure, exact-once async close, concurrent calls sharing one
  application loop/thread, and deterministic stdout markers.
- Runtime and generated-code tests cover cancellation before registration,
  in-flight `CancelledError`, terminal `finally` ordering, suppression return,
  suppression exception, shutdown drain, close success/failure/poison,
  abandonment, duplicate close, use after close, an independently raised
  `CancelledError` remaining a `PythonError`, and fallback-unavailable/no-op
  propagation returning a runtime error instead of hanging.
- Migrate cancellation-classification tests from the file's manual
  `std::thread::park` executor to a Tokio current-thread test runtime. Bind the
  carrier fallback to the spawned test task's real abort handle, and assert the
  receiver observes native cancellation only after the Python `finally`
  marker. Non-cancellation tests may keep the small manual executor.
- Add the compiled suite unconditionally to the authoritative create-PR,
  merge, nightly, and release profiles. If it exceeds the existing Python
  interop lane budget, raise that checked-in budget with measured evidence;
  there is no manual-only or conditional evidence path.
- `demos/m7_demo` invokes the same compiled fixture path and documents its real
  binary output rather than maintaining an independent semantic example.

## Evidence And Documentation Cutover

Only after all executable checks pass:

- set the existing `coroutine-declaration` capability row to `active` and mark
  every required evidence kind `passing` with concrete owners. Consuming
  async-close is recorded under that row's cleanup evidence owner; the locked
  ledger has no separate async-close capability row and Wave 7 does not invent
  one;
- update the Python interop public guide, both durable architecture documents,
  runtime architecture summary, verification README and exit evidence;
- update the roadmap and phase M7/Wave 7 status in the tracker. The final merged
  implementation PR link is recorded by the normal follow-up tracker PR after
  this implementation PR merges.

`SIFR-PYRES-0002` remains active for M8-M12 syntax and its general diagnostic
documentation remains valid.

## Likely Implementation Surfaces

- `crates/sifr_lowering/src/lower/python_interop.rs`
- `crates/sifr_lowering/src/lower/python_coroutine_contract_tests.rs`
- `crates/sifr_runtime/src/cancellation.rs`
- a new focused cancellation-propagation module under
  `crates/sifr_runtime/src/python/` rather than growing `async_runtime.rs`
- `crates/sifr_runtime/src/python/async_terminal.rs`
- `crates/sifr_runtime/src/python/async_declaration.rs`
- split lowering/async runtime/declaration and codegen test modules as needed;
  do not add material code to the 858-line `python_interop.rs`, 812-line
  `async_runtime.rs`, or 853-line async conversion module
- `verification/areas/python_interop/` manifest, runner, fixture, capability,
  README, and exit evidence
- `demos/m7_demo/`
- `docs/python-interop.mdx`, `internal_docs/architecture.md`, and the two Python
  interop architecture documents
- `plans/roadmap.md`

## Validation

Focused tests first:

```text
cargo test -p sifr_lowering python_coroutine_contract_tests
cargo test -p sifr_runtime python::async_runtime_tests
cargo test -p sifr_runtime python::async_declaration_tests
cargo test -p sifr_codegen python_interop_async_tests
verification/areas/python_interop/run.sh --self-test
<compiled async-declaration example suite>
bash demos/m7_demo/run.sh
python3 scripts/check_hir_maintainability_guardrails.py
<file-size guardrail>
```

The compiled async-declaration suite is part of `create-pr` itself. Then review
the frozen diff
until agent is satisfied, open the implementation PR, perform an actual PR review
until satisfied, and merge. The tracker-only PR records the merged link and M7
completion.

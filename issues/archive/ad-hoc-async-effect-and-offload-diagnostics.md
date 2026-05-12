# Ad Hoc Async Effect And Offload Diagnostics

## Status

completed

## Problem

Phase 32 implemented the async ecosystem, but a few easy-to-write programs still express the wrong intent:

- `async def` functions that never suspend,
- `await` used as ceremony around work that has no async effect,
- direct synchronous blocking or CPU-heavy calls from async code,
- `spawn_blocking` used as a generic escape hatch for cheap synchronous work.

These patterns are common sources of accidental latency, unnecessary task machinery, and misleading APIs in other languages. Sifr should reject them before users build habits around them.

## Goal

Add a compiler-enforced async effect discipline:

- every `async def` has a real suspension reason,
- every `await` targets an awaitable with a real async effect,
- direct annotated blocking or CPU-heavy sync work in async code is an error,
- `spawn_blocking` is reserved for sync work classified as `@blocking_io`, `@cpu_heavy`, or known blocking external work.

## Non-Goals

- Do not add a public effect type system to user syntax.
- Do not make `@blocking_io` awaitable. It remains a sync workload annotation.
- Do not silently rewrite direct calls into async tasks or blocking offload.
- Do not reject cheap sync helper calls inside async functions unless they are awaited, offloaded, or annotated as blocking/CPU-heavy.

## Model Decisions

### Suspension Effects

The compiler computes an internal async suspension summary for each async function:

- `NoSuspend`: the body has no operation that can suspend.
- `AsyncIo`: the body awaits async I/O or an async API with a transitive I/O wait.
- `TimerWait`: the body awaits sleep, timeout, or timer-backed scheduling.
- `ChannelWait`: the body awaits channel send/receive or async iteration over a channel-backed stream.
- `TaskWait`: the body awaits a task, blocking task handle, task group/scope cleanup, gather/select/race, or another async function with a non-empty suspension summary.
- `AsyncResourceWait`: the body awaits async context-manager enter/exit, async iterator `anext()`, or async cleanup.
- `GeneratorSuspend`: an async generator body suspends at `yield` or awaits a non-empty suspension effect.

The exact enum names are implementation details. The semantic rule is public: async code must suspend for a real reason.

Suspension summaries are transitive across same-task async calls. If an async function only awaits another async function, the compiler follows that downstream coroutine chain until it reaches a real suspension source such as async I/O, a timer, a channel operation, task wait, async resource wait, or async generator suspension. Awaiting a wrapper coroutine is valid when any downstream same-task callee has a non-`NoSuspend` summary. It is rejected only when the whole downstream same-task chain computes to `NoSuspend`.

### `async def` Must Suspend

An `async def` that computes to `NoSuspend` is rejected:

```sifr
async def value() -> int:
    return 1
```

The diagnostic should suggest `def` unless the function is required to satisfy an async protocol. Protocol-conformance exceptions require an explicit reviewed escape hatch with a reason-bearing annotation; there is no silent exemption.

### `await` Requires Real Awaitability

`await x` remains valid only when `x` is awaitable. Awaiting a sync function result is a hard type error.

In addition, awaiting a same-task `Coroutine[T, E]` whose transitive suspension summary is `NoSuspend` is rejected. The compiler should point to the awaited function and say that it is async in shape only.

Awaiting task handles, blocking task handles, async context-manager operations, async iterator advancement, channel operations, sleep, timeout, gather, select, and race is valid because these operations carry an async suspension effect.

### Workload Annotations Stay Sync

`@blocking_io` and `@cpu_heavy` classify synchronous functions:

- `@blocking_io` means synchronous I/O or blocking OS waits.
- `@cpu_heavy` means CPU-heavy synchronous compute.

They do not make a function awaitable and do not schedule anything. They are valid only on sync `def`. Applying either annotation to `async def` is an error because async APIs use suspension summaries such as `AsyncIo`, not sync workload annotations.

Calling a known `@blocking_io` or `@cpu_heavy` function directly from an `async def` body is an error.

Allowed fixes:

- use a native async API when available,
- offload sync blocking or CPU-heavy work with `task.spawn_blocking`,
- use `sifr.concurrent.ThreadPoolExecutor` when the executor abstraction is the intended surface.

### `spawn_blocking` Requires Classified Work

`task.spawn_blocking(fn)` and `ThreadPoolExecutor.submit(fn)` are accepted only when `fn` is one of:

- annotated `@blocking_io`,
- annotated `@cpu_heavy`,
- known by the stdlib annotation database as blocking or CPU-heavy,
- known by an FFI/external contract as blocking or CPU-heavy.

Unannotated cheap sync helpers are rejected as offload targets. The diagnostic should say to call the helper directly, or add `@blocking_io` / `@cpu_heavy` if the declaration is genuinely blocking or expensive.

`spawn_blocking` on `@blocking_io` work is correct and should not warn by default. A later optional info diagnostic may suggest a native async API only when the compiler knows a specific replacement.

### Direct Cheap Sync Helpers Stay Allowed

Async functions may call ordinary cheap sync helper functions directly:

```sifr
def normalize(x: str) -> str:
    return x.strip()

async def handle() -> Result[None, IOError]:
    value = normalize(try await read_line())
    return None
```

The problem is not sync helper use. The problem is blocking/CPU-heavy sync work in async code, fake async functions, and pointless offload.

## Milestones

### adhoc_async_effect_0: Effect Summary Infrastructure

Scope:

- Add internal async suspension summaries to HIR or the existing analysis layer.
- Compute direct effects for known async primitives:
  - `task.sleep`,
  - `task.timeout`,
  - task handle await/join,
  - `BlockingTask` await/join,
  - `gather`, `select`, `race`,
  - channel send/receive,
  - semaphore acquire and notify wait,
  - async context-manager enter/exit,
  - async iterator `anext`,
  - async generator `yield`.
- Compute transitive summaries through same-task coroutine calls to a deterministic call-graph fixpoint, including recursive/SCC handling.
- Keep summaries internal; do not expose public effect syntax.

Validation:

- `async_effect_summary_sleep.sifr`
- `async_effect_summary_channel_receive.sifr`
- `async_effect_summary_transitive_await.sifr`

### adhoc_async_effect_1: Reject Fake Async And Fake Await

Scope:

- Reject `async def` bodies whose summary is `NoSuspend`.
- Reject `await` of same-task coroutines whose transitive summary is `NoSuspend`.
- Preserve the existing hard error for awaiting non-awaitable values.
- Add a reason-bearing escape hatch only if async protocol conformance requires it; the escape hatch must be explicit and diagnosed if used outside protocol-shaped code.

Validation:

- `async_no_suspend_rejected.sifr`
- `async_transitive_no_suspend_await_rejected.sifr`
- `await_sync_function_rejected.sifr`
- `async_protocol_no_suspend_requires_escape_hatch.sifr`

### adhoc_async_effect_2: Enforce Workload Annotations

Scope:

- Reject `@blocking_io` or `@cpu_heavy` on `async def`.
- Reject direct `@blocking_io` and `@cpu_heavy` calls from async contexts.
- Preserve structured diagnostic code coverage and docs.
- Keep direct cheap sync helper calls allowed.
- Ensure diagnostics distinguish:
  - use an async API,
  - use `task.spawn_blocking`,
  - use `ThreadPoolExecutor`.

Validation:

- `blocking_io_on_async_def_rejected.sifr`
- `cpu_heavy_on_async_def_rejected.sifr`
- `blocking_io_direct_call_in_async_rejected.sifr`
- `cpu_heavy_direct_call_in_async_rejected.sifr`
- `cheap_sync_helper_in_async_allowed.sifr`

### adhoc_async_effect_3: Restrict Blocking Offload Targets

Scope:

- Reject `task.spawn_blocking` on unannotated local sync functions.
- Reject `ThreadPoolExecutor.submit` on unannotated local sync functions.
- Allow `@blocking_io`, `@cpu_heavy`, stdlib-classified, and external-contract-classified targets.
- Keep existing sendability, zero-argument v1, and result/error constraints.
- Do not warn for valid `@blocking_io` offload by default.

Validation:

- `spawn_blocking_blocking_io_allowed.sifr`
- `spawn_blocking_cpu_heavy_allowed.sifr`
- `spawn_blocking_unannotated_rejected.sifr`
- `thread_pool_submit_unannotated_rejected.sifr`
- `spawn_blocking_known_stdlib_blocking_allowed.sifr`

## Definition Of Done

- [x] Async effect summaries are computed deterministically.
- [x] Fake async functions and fake awaits are rejected with Sifr diagnostics.
- [x] Direct annotated blocking/CPU-heavy calls in async code are errors.
- [x] Blocking offload requires workload classification.
- [x] Existing Phase 32 positive async fixtures remain valid unless they intentionally covered now-rejected fake async/offload behavior.
- [x] The quick validation lane includes representative positive and negative fixtures.

## Completion

Completed on 2026-05-12 through milestone PRs:

- [#2096](https://github.com/sifr-lang/sifr/pull/2096): renamed workload annotations to `@blocking_io` and `@cpu_heavy`.
- [#2097](https://github.com/sifr-lang/sifr/pull/2097): added async suspension summary infrastructure.
- [#2098](https://github.com/sifr-lang/sifr/pull/2098): rejected fake async functions and fake awaits.
- [#2099](https://github.com/sifr-lang/sifr/pull/2099): enforced workload annotations in async contexts.
- [#2100](https://github.com/sifr-lang/sifr/pull/2100): restricted blocking offload targets to classified work.

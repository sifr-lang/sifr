

Now I have a full picture. Let me write the review memo.

---

# REVIEW MEMO: Async Annotation Vocabulary — `@blocking_io` vs `@io_bound`

**Date:** 2026-05-09
**From:** agent (codex/figure-out-concurrency)
**Subject:** Annotation/effect vocabulary for blocking I/O and CPU-bound diagnostics in Sifr's async/concurrency model
**Status:** SATISFIED (with required doc changes)

---

## Executive Summary

The user's challenge is sound: `@blocking_io` is misnamed, and the diagnostic model is underspecified. The fix is **rename `@blocking_io` → `@io_bound`**, keep `@cpu_bound` as-is, and add a precise diagnostic-trigger mechanism to both model documents. No new annotations needed.

---

## The Naming Problem

`@blocking_io` carries two conflated implications:

1. **"This blocks the thread"** — a runtime scheduling fact
2. **"This is I/O-bound work"** — a workload characterization

In async contexts, "blocking I/O" is ambiguous. Consider:

```sifr
# @blocking_io suggests "this will block" — but block WHAT?
# Is it blocking the OS thread? blocking the async task? blocking the event loop?
@blocking_io
def read_file(path: str) -> Result[str, IOError]:
    ...
```

The word "blocking" conflates two distinct concerns:
- **Mechanism:** synchronous syscall that stalls the OS thread
- **Workload type:** the work waits on external resources, not CPU cycles

"Blocking" is also a Rust-loaded term with scheduling semantics (blocking a thread), while Sifr's annotations are meant to be **diagnostic facts**, not scheduling directives. Users reading `@blocking_io` may reasonably expect the compiler to do something with a thread — which it won't.

---

## The Diagnostic Model Is Underspecified

Current docs say:
> "These annotations are diagnostic facts, not scheduling commands."

But they never explain **how diagnostics are produced**. This leaves implementation completely undefined for milestone_async_6. The previous review (pass-4, N-1) flagged this exact gap.

The fix is not to add new annotations — it's to make the mechanism explicit.

---

## Analysis of the Three Diagnostic Goals

### Goal 1: Warn when sync/blocking I/O is called from async code without offload or async equivalent

**Requires:** `@io_bound` (rename from `@blocking_io`)
**Mechanism:** stdlib annotation database + user `@io_bound` annotations → diagnostics in async contexts
**Naming:** `@io_bound` is the right term because:
- "I/O-bound" characterizes **what the workload does** (waits on disk/network)
- It does NOT imply the compiler will do something to a thread
- It's the standard term in async/await literature (Go, Rust `tokio`, Swift `async`)
- The opposite is "CPU-bound" (already the term used by `@cpu_bound`)

`@io_bound` functions called from `async def` bodies trigger:
> "This function is I/O-bound. Use an async equivalent API, or wrap with `spawn_blocking`."

### Goal 2: Warn when CPU-heavy work runs on async cooperative runtime instead of `spawn_blocking`/executor

**Requires:** `@cpu_bound` (keep as-is)
**Mechanism:** stdlib annotation database + user `@cpu_bound` annotations → diagnostics in async contexts
**Naming:** `@cpu_bound` is already correct:
- It characterizes **what the workload does** (burns CPU cycles)
- The opposite of "CPU-bound" is "I/O-bound" — symmetry is good
- "CPU-bound" is the standard term

`@cpu_bound` functions called from `async def` bodies trigger:
> "This function is CPU-bound. Use `spawn_blocking` or a `ThreadPoolExecutor` to avoid starving the async runtime."

### Goal 3: Warn when async/await is used around work that never awaits and only burns CPU

**This is a false positive to avoid.** Short pure compute helpers (e.g., a helper that formats a date, computes a hash, validates a regex) should not all warn. The model should:

1. **Default to no annotation** — unannotated functions are assumed to be cheap compute with no I/O
2. **Only warn on annotated functions** — `@io_bound` and `@cpu_bound` are opt-in by user or stdlib
3. **Unknown functions get no automatic warning** — external/FFI calls are the exception (see below)

The user's point about "assume everything is CPU-bound" is valid for **safety** (if you don't know, assume it needs offload), but the **diagnostic** vocabulary should be workload-characterization-based, not assumption-based.

### Goal 4: Guide users toward async APIs for I/O and offload for blocking or CPU-heavy work

**This is the guidance layer**, not a separate annotation. The diagnostic message carries the guidance:
- `@io_bound` in async context → "try the async equivalent, or use `spawn_blocking`"
- `@cpu_bound` in async context → "use `spawn_blocking` or `ThreadPoolExecutor`"

---

## The Stdlib Annotation Database

The model needs a defined mechanism: **the compiler knows about stdlib functions**.

Pre-annotated stdlib functions include:
- **I/O-bound:** `sifr.os.read`, `sifr.os.write`, `sifr.time.sleep`, `sifr.socket.recv`, `sifr.http.client` operations, `sifr.database` operations
- **CPU-bound:** `sifr.hashlib.md5`, `sifr.hashlib.sha256`, `sifr.json.dumps` (large payloads), `sifr.compression` operations, cryptographic primitives

Unknown/external/FFI calls: treated as **potentially blocking** by default (conservative). A `#[ffi_unknown]` annotation can be added later if false positives become a problem.

---

## Proposed Final Vocabulary

| Annotation | Meaning | Diagnostic trigger | Opposite |
|---|---|---|---|
| `@io_bound` | This function performs synchronous blocking I/O | Warn when called from `async def` body | `@cpu_bound` |
| `@cpu_bound` | This function is CPU-intensive (no I/O, pure compute) | Warn when called from `async def` body | `@io_bound` |
| *none* | Unknown / assumed cheap compute | No automatic warning | — |

Both are **declaration-site annotations**. They are facts about what the function *does*, not commands for what the compiler should *do*. The compiler never rewrites calls based on these annotations.

---

## Proposed Doc Edits

### Edit 1: `internal_docs/async_concurrency_model.md`

**Lines 111–116** (Required surfaces section) — rename and specify:

```
Old:
- `@blocking_io` for sync functions that perform blocking I/O
- `@cpu_bound` for sync functions expected to burn CPU

New:
- `@io_bound` for sync functions that perform synchronous I/O (file, network, database, pipe, timer wait)
- `@cpu_bound` for sync functions expected to be CPU-intensive (cryptography, compression, hashing, parsing, computation-heavy processing)
```

**Lines 449–451** (Blocking and Thread Offload section) — rename and add mechanism:

```
Old:
`@blocking_io` and `@cpu_bound` are diagnostic annotations. They never imply automatic task or thread scheduling.

New:
`@io_bound` and `@cpu_bound` are declaration-site diagnostic annotations. They characterize a function's workload class:
- `@io_bound`: the function performs synchronous I/O that would block an OS thread (file read/write, network I/O, database calls, pipe operations, blocking timer waits). Calling a known-@io_bound function from `async def` body produces a warning: "I/O-bound call in async context — use an async API if available, or wrap with `spawn_blocking`."
- `@cpu_bound`: the function is CPU-intensive with no I/O (cryptography, compression, hashing, parsing, numerical compute). Calling a known-@cpu_bound function from `async def` body produces a warning: "CPU-bound call in async context — use `spawn_blocking` or a `ThreadPoolExecutor` to avoid starving the runtime."

The stdlib maintains a built-in annotation database for all stdlib functions. User code can annotate with `@io_bound` or `@cpu_bound`. Unknown or FFI calls are treated conservatively as potentially blocking.

These annotations never imply automatic task or thread scheduling. The compiler must not silently rewrite either call. Compilation proceeds with a warning; the annotation guides the developer, not the compiler.
```

**Model Invariant #14** (line 537):

```
Old:
14. `@blocking_io` and `@cpu_bound` are diagnostic annotations, not implicit scheduling directives.

New:
14. `@io_bound` and `@cpu_bound` are declaration-site workload-classification annotations. They power diagnostics but never trigger implicit scheduling. The stdlib ships with a pre-annotated database of known stdlib functions.
```

### Edit 2: `internal_docs/phases/32_async_ecosystem.md`

**Locked v1 Decision #16** (line 88) — rename:

```
Old:
16. `@blocking_io` and `@cpu_bound` are diagnostic annotations, not implicit scheduling directives.

New:
16. `@io_bound` and `@cpu_bound` are declaration-site diagnostic annotations; they classify workload class (I/O-bound vs CPU-bound) for compiler diagnostics. They never trigger implicit scheduling. The stdlib ships with a pre-annotated database of known blocking/cpu functions.
```

**milestone_async_6 Scope** (line 521) — rename and specify mechanism:

```
Old:
- Add `@blocking_io` and `@cpu_bound` annotations.
- Add diagnostics for calling annotated functions directly from async contexts.

New:
- Add `@io_bound` and `@cpu_bound` declaration-site annotations.
- Add a stdlib annotation database of known blocking/cpu functions.
- Add diagnostics for calling @io_bound or @cpu_bound functions from async contexts.
- The diagnostic is a warning (not an error) and suggests the appropriate alternative.
```

**milestone_async_6 validation fixtures** (lines 551–552) — rename:

```
Old:
- `blocking_io_annotation_warning.sifr`
- `cpu_bound_annotation_warning.sifr`

New:
- `io_bound_annotation_warning.sifr`
- `cpu_bound_annotation_warning.sifr`
```

**milestone_async_6 negative fixtures** (lines 558–559) — rename:

```
Old:
- `blocking_call_in_async_diagnostic.sifr`
- `cpu_bound_call_in_async_diagnostic.sifr`

New:
- `io_bound_call_in_async_diagnostic.sifr`
- `cpu_bound_call_in_async_diagnostic.sifr`
```

### Edit 3: Architecture doc — update reference to renamed annotation

`internal_docs/architecture.md` line 682 (milestone responsibilities for concurrency safety):

```
Old:
- milestone_async_4: implement Send/Sync and borrow-boundary checking at spawn boundaries.

New:
- milestone_async_4: implement Send/Sync and borrow-boundary checking at spawn boundaries.
- milestone_async_6: implement @io_bound and @cpu_bound annotations, stdlib annotation database, and async-context diagnostics.
```

---

## Why NOT Introduce Both `@blocking_io` and `@io_bound`

Some might argue for keeping `@blocking_io` (for API-level blocking) and adding `@io_bound` (for workload classification). This creates **overlap without benefit**:

- `@blocking_io` implies scheduling mechanism ("it blocks something")
- `@io_bound` implies workload type ("it waits on I/O")

In a **diagnostic-only model** where neither annotation triggers scheduling, the mechanism implication is misleading. `@io_bound` is semantically cleaner because it tells the developer what *kind of work* the function does, not what the compiler should do about it.

If future Sifr added a formal effect system, the two would diverge:
- `@io_bound` → `io` effect (function may perform I/O)
- `@blocking_io` → `blocking` effect (function may block an OS thread)

But for v1 diagnostics-only, one workload-classification term per axis is sufficient.

---

## Why NOT Replace `@cpu_bound` Too

`@cpu_bound` is already the correct term. The CPU-bound/IO-bound split is standard:
- **CPU-bound:** work that uses CPU cycles (no I/O)
- **I/O-bound:** work that waits on external resources (no CPU)

"CPU-bound" is the established term in operating systems, async runtime literature, and Go/Rust/Swift async models. There's no reason to change it.

---

## Semantic Precision for Unknown Functions and FFI

**Unknown functions:** No automatic warning. Unannotated user functions are assumed to be cheap compute. This avoids the false-positive problem (short pure-compute helpers don't warn).

**External/FFI calls:** Treated as **potentially blocking** by default (conservative). A future `#[ffi_unknown]` annotation can silence false positives if needed. For v1, warn on FFI calls in async context as an advisory.

**Stdlib unknown:** If the stdlib annotation database doesn't cover a function, no diagnostic is produced. The database grows as stdlib is shipped.

---

## Verdict

**SATISFIED with required doc changes.**

The model is sound. The user's instinct is correct: rename `@blocking_io` → `@io_bound`. The diagnostic mechanism needs to be specified (stdlib annotation database + user annotations → warning), but no new annotations are needed.

Required changes (all in docs, no code impact):
1. Rename `@blocking_io` → `@io_bound` in `async_concurrency_model.md` (3 locations)
2. Rename `@blocking_io` → `@io_bound` in `32_async_ecosystem.md` (4 locations)
3. Add diagnostic mechanism specification in `async_concurrency_model.md` Blocking and Thread Offload section
4. Update architecture.md milestone responsibilities reference

All four changes are doc-only. No HIR, codegen, or stdlib implementation is affected by the vocabulary rename.

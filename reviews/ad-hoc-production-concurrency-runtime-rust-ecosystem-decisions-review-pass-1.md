## FAIL

**2 blocking issues, 3 non-blocking.**

---

### B1 — `tokio-util` feature plan cannot satisfy its stated purpose (BLOCKING)

**Phase doc line 264.** The feature plan specifies `rt`, `io-util`, `time`. The binding notes explicitly name `tokio_util::sync::CancellationToken` as the use case. In tokio-util 0.7.x, `CancellationToken` is in `tokio_util::sync`, gated behind the `sync` feature flag. With `sync` absent from the feature list, `CancellationToken` does not compile. The feature plan directly contradicts the named use case.

**Fix:** Add `sync` to the feature list. The corrected entry: `default-features = false`, features `rt`, `io-util`, `time`, and `sync`. Update the Resolved Decision register (line 894) to match.

---

### B2 — `tokio` exclusion of `rt-multi-thread` is unresolved for concurrent task execution (BLOCKING)

**Phase doc line 263.** The capability column claims "async runtime, scoped task lowering, blocking pool" but the feature plan excludes `rt-multi-thread`. Without it, Tokio can only construct a `current_thread` runtime — `tokio::spawn` tasks cooperate on one OS thread; they do not run in parallel.

This is an unresolved ambiguity M1 implementors will hit immediately when constructing the runtime entry point. Two valid interpretations exist, neither is recorded:

- **Option A (current_thread + rayon):** Tokio is current_thread only. `TaskGroup[E]` is cooperatively concurrent (not parallel). CPU and blocking-I/O parallelism come exclusively from rayon and `spawn_blocking`. This must be stated explicitly as a binding design invariant.
- **Option B (multi-thread):** Add `rt-multi-thread` to the accepted feature set and record why Tokio's work-stealing scheduler is needed alongside Rayon.

**Fix:** Record one of the two interpretations in the `tokio` binding notes and in the M0 execution ledger before M0 closes or M1 starts.

---

### N1 — Cross-doc drift in no-public-types list (non-blocking)

Phase doc (line 257) lists: Tokio, Futures, Rayon, Crossbeam, **Rustix**, tracing, **metrics**, serde, **postcard**, thiserror.
Model doc (line 225) lists: Tokio, Futures, Rayon, Crossbeam, **Parking Lot**, tracing, serde, thiserror.

Model doc is missing `Rustix`, `metrics`, `postcard` (all accepted-but-hidden crates). It still lists `Parking Lot`, which is a rejected crate — it cannot leak by construction and does not belong in a "must hide" list.

**Fix:** Align the model doc's enumeration with the phase doc, or replace the list with a pointer to the phase table.

---

### N2 — `tracing` wording implies optional use, but the feature plan makes attribute macros impossible (non-blocking)

**Phase doc line 272.** Binding notes say "do not use `#[instrument]`/attribute macros **as required API**" — implying they could be used optionally. But the accepted feature is `std` only; without `attributes`, `#[instrument]` does not compile at all. The qualifier "as required API" is misleading.

**Fix:** Change to "do not enable the `attributes` feature; `#[instrument]` and tracing attribute macros are unavailable in this phase."

---

### N3 — Execution ledger lists "Tokio MPSC" as a standalone accepted item (non-blocking)

**Ledger line 338.** The ledger names "Tokio MPSC" as a discrete accepted item, but the phase table has no such row — MPSC is part of the `tokio::sync` row (line 266) under the accepted `sync` feature. This creates ambiguity about whether `tokio::sync::mpsc` is independently authorized or bundled.

**Fix:** Replace "Tokio MPSC" in the ledger entry with "Tokio `sync` (MPSC, oneshot, broadcast, mutex, semaphore, notify)" to match the phase table.

---

**What is clean:** No remaining discovery wording. All crate decisions are accounted for. Rejected-crate lists are consistent across phase table, Resolved Decision, and Execution Ledger. Postcard derive-macro exclusion, rustix per-crate scoping, rayon global-pool prohibition, and no-new-bespoke-implementation rule are stated consistently in all locations. Versions are internally consistent. The M0 gate at line 428 would catch B2 if it explicitly required the runtime threading model to be recorded — consider adding that to the M0 definition of done as a hardening step.

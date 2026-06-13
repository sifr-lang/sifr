Reviewing each of the five previous findings against the updated snippets:

**1. tokio-util feature wording** — RESOLVED. Line 264 now reads: *"in tokio-util 0.7.18 `rt` exposes `tokio_util::sync::CancellationToken`, and there is no separate `sync` feature"* — the clarification is accurate and removes ambiguity about which feature gates `CancellationToken`.

**2. current_thread vs rt-multi-thread** — RESOLVED. `rt-multi-thread` is explicitly in the do-not-enable list (line 263), the threading model is stated as `current_thread` in the phase table, the M0 done criteria (line 407), and the resolved decision register (line 896). Consistent across all three locations.

**3. no-public-types drift** — RESOLVED. Phase doc (line 257) and model doc (line 225) now carry matching crate lists: Tokio, Futures, Rayon, Crossbeam, Rustix, tracing, metrics, serde, postcard, thiserror. No drift remains.

**4. tracing attributes wording** — RESOLVED. Line 272 now explicitly states *"do not enable the `attributes` feature, so `#[instrument]` and tracing attribute macros are unavailable in this phase"* — precise and unambiguous.

**5. Tokio MPSC ledger wording** — RESOLVED. Line 338 of the execution ledger now explicitly enumerates *"Tokio `sync` wrappers including MPSC/semaphore/notify"*, matching the phase table entry at line 266.

---

**PASS — no blockers.**

**FAIL**

Three material gaps follow.

---

## F1 — Network doc: No concurrency/runtime dependency gate matrix

**Files/sections:** `issues/ad-hoc-production-network-http-platform-substrate.md`
- Cross-Phase Dependency Contract (lines 142–158)
- `milestone_network_http_0` scope (lines 476–510)
- Quality Contract (lines 764–774)

**Problem:** The network doc has a detailed, surface-by-surface text/i18n dependency matrix (lines 163–180) with per-feature statuses (`blocked-on-text-i18n-m1`, `blocked-on-text-i18n-m2`, etc.) and an explicit M0 scope item to build that inventory. There is **no equivalent matrix for concurrency/runtime dependencies.** The only concurrency/runtime gate in the entire doc is one sentence: "Concurrency/runtime is a hard prerequisite for executor-backed serving APIs" (line 155). That single gate does not cover:

- Cancellation/timeout semantics → must consume `sifr.task` from `milestone_concurrency_runtime_1`
- Graceful shutdown → must consume `sifr.signal.shutdown_stream` from `milestone_concurrency_runtime_5`
- `@blocking_io` offload in sync network helpers → must use `sifr.runtime.spawn_blocking` from `milestone_concurrency_runtime_3`
- Network observability/diagnostics → must not duplicate the structured runtime diagnostics model from `milestone_concurrency_runtime_5`

The Quality Contract (lines 764–774) explicitly bars local text substitutes ("No local encoding registry, local Unicode data table…") but contains **no equivalent prohibition on local concurrency substitutes.** An implementer has a clear prohibition path for text substitutes and none for concurrency/runtime substitutes.

`milestone_network_http_0` scope (lines 482–483) explicitly asks for "the complete text/i18n dependency inventory" but has **no parallel item** for a concurrency/runtime dependency inventory.

**Remediation:**

1. Add a "Concurrency/Runtime Dependency Decisions" subsection to Cross-Phase Dependency Contract with a table analogous to lines 163–180. Minimum required rows:

| Network/HTTP surface | Status before concurrency/runtime M1 | Dependency | Decision |
| --- | --- | --- | --- |
| TCP/TLS/HTTP cancellation and timeout handling | `blocked-on-concurrency-runtime-m1` | `sifr.task` cancel/deadline model | Must consume `Task.cancel`/`cancel_scope`; must not introduce a parallel cancellation token model |
| Graceful connection/server shutdown | `blocked-on-concurrency-runtime-m5` | `sifr.signal.shutdown_stream` | Must consume the signal/shutdown substrate; must not introduce a local shutdown coordinator |
| `@blocking_io` sync helpers (DNS sync, sync connect) | `blocked-on-concurrency-runtime-m3` | `sifr.runtime.spawn_blocking` | Must use the offload substrate; must not introduce a local thread pool or blocking executor |
| Network observability/diagnostic hooks | `blocked-on-concurrency-runtime-m5` | structured runtime diagnostics model | Must not introduce a separate diagnostic routing system; structured events must compose with the runtime diagnostic model |

2. Add to `milestone_network_http_0` scope: "Define the complete concurrency/runtime dependency inventory — which network surfaces are blocked on each concurrency/runtime milestone."

3. Add to the Quality Contract: "No local cancellation token model, local timeout/deadline coordinator, local shutdown manager, local offload pool, or local diagnostic routing system may be introduced in this phase. These must consume the task/signal/runtime substrate from the concurrency/runtime provider phase."

---

## F2 — Network doc: `sifr.net` claims cancellation and shutdown as owned outputs without citing provider

**Files/sections:** `issues/ad-hoc-production-network-http-platform-substrate.md`
- Objective (lines 17–27, specifically line 18)
- Architecture Principles / Native Runtime First (line 267)

**Problem:** The Objective lists "cancellation, timeouts, backpressure, graceful shutdown" as network-phase outputs (line 18). The `sifr.net` scope statement (line 267) lists "timeouts, cancellation, backpressure, shutdown" as `sifr.net` responsibilities with no "consumes from" qualifier. This creates the appearance that network phase owns and implements these from scratch, not that it consumes them from the concurrency phase. Without F1's gate matrix, this section is the primary implementation permission for adding local cancellation/shutdown.

**Remediation:**

After line 267 (`sifr.net` owns … "timeouts, cancellation, backpressure, shutdown, and connection lifecycle"), add:

> `sifr.net` cancellation and timeout semantics are **network-layer applications** of the `sifr.task` cancellation and deadline model from `milestone_concurrency_runtime_1`; this phase does not implement its own cancellation primitive. `sifr.net` graceful shutdown must consume `sifr.signal.shutdown_stream` from `milestone_concurrency_runtime_5`; this phase does not implement its own shutdown coordinator.

The same note is needed in the Objective section after line 18 to prevent a reader from interpreting that list as a design mandate.

---

## F3 — Stale "stdlib-parity" naming in text and concurrency doc filenames and tracking artifact names

**Files/sections:**
- Filename: `issues/ad-hoc-production-text-i18n-stdlib-parity.md`
- Filename: `issues/ad-hoc-production-concurrency-runtime-stdlib-parity.md`
- Text doc tracking artifacts (line 599): `issues/ad-hoc-production-text-i18n-stdlib-parity-execution.md`
- Concurrency doc tracking artifacts (line 562): `issues/ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md`

**Problem:** Both docs have already been updated to "native substrate" framing in content. The network doc was already renamed to `platform-substrate`. The text and concurrency filenames still say `stdlib-parity`, which contradicts the content ("This phase is not a mandate to clone CPython's…") and is inconsistent with the renamed network doc. An implementer who discovers these files by name will see the wrong goal before reading line 1.

This is not implementation-blocking but is actively misleading given the network doc was renamed precisely to signal this distinction.

**Remediation (choose one):**

Option A: Rename the files to `ad-hoc-production-text-i18n-runtime.md` and `ad-hoc-production-concurrency-runtime-substrate.md`, update the execution ledger artifact names to match, and update all cross-references in the three docs. Consistent with what was already done for the network doc.

Option B (if renaming is deferred): Add a header note to each file immediately after the Status/Phase placement lines:

> **Naming note:** This filename uses the legacy "stdlib-parity" label. The phase goal is a Sifr-native substrate, not CPython module parity. See Objective and Non-Goals sections.

---

## Items that passed

- Phase order is correct and internally consistent across all three docs (text → concurrency → network stated in each).
- Text doc cross-phase gates are sufficient: explicit binary-only restrictions on both consumers before M1, explicit "no local encoding fallbacks" prohibition in Quality Contract.
- Concurrency doc correctly defers text-mode subprocess to text/i18n M1 and does not claim text-encoding ownership.
- Milestone names and cross-references are internally consistent across all three docs; no stale milestone labels found.
- All three docs are framed as Sifr-native substrate in their content (not CPython module clones); non-goals and rejected surfaces are explicitly enumerated.
- Network doc's "No-Toy-Module Gate," "Maintenance Burden Test," and rejection of CPython-shaped modules are solid.

# Verdict

BLOCKED

# Blocking findings

1. **[High] Retained async rollback is still bypassed by native task cancellation.** Generated code calls `finalize_retained_callbacks` only after the inner finalization future returns at `crates/sifr_codegen/src/python_interop_async/callback_frame.rs:278`. During submission, active cancellation instead resumes the native fallback and yields at `crates/sifr_runtime/src/python/async_cancellation.rs:28`, allowing the task to be aborted before that future returns. Cleanup then relies on `RetainedCallbackGroup::drop`, which merely spawns an unjoined task at `crates/sifr_runtime/src/python/callbacks/ownership.rs:103`. A cancelled retained-result declaration can therefore complete cancellation before rollback, and shutdown can discard the detached rollback. Required remediation: carry cancellation back through the wrapper, await terminal rollback, then resume native cancellation; `Drop` must remain emergency-only. Add generated-wrapper cancellation and cancellation-during-shutdown tests.

2. **[High] Some later owner-operation failures still lose retained callback evidence.** Sync method/item wrappers resolve the Python callable using a `mapped_let`/`?` at `crates/sifr_codegen/src/python_interop_direct.rs:643` and `crates/sifr_codegen/src/python_interop_direct.rs:665`, before callback evidence is attached at `crates/sifr_codegen/src/python_interop_direct.rs:719`. If a retained handler previously failed and a later method lookup raises through `__getattribute__` or a descriptor, the Python error returns without the required callback secondary evidence. Required remediation: include argument conversion and callable resolution in the unified owner-operation reconciliation path, with executable early-lookup/conversion failure tests.

3. **[High] The Pub/Sub compiled fixture still does not prove active close/drain.** The bridge merely schedules `emit` and yields once at `verification/areas/python_interop/fixtures/pubsub/python_bridges/pubsub.py:26`. The callback may finish before `aclose`; if so, its conditional active-work check at line 20 is skipped. The Sifr fixture then prints a hard-coded success marker. Required remediation: use deterministic started/held synchronization, require close to observe active work, and derive the marker from asserted drain evidence.

4. **[Medium] Durable public documentation still declares active callback diagnostics reserved.** `docs/diagnostics/error-codes.mdx:133` says `PYCB` is reserved for future diagnostics, while `docs/python-interop.mdx:450` correctly calls it active. Required remediation: update the stale diagnostic documentation and include it in the callback-activation consistency sweep.

# Non-blocking observations

- The `callback-examples` suite is now selected by create-PR, merge, nightly, and release profiles. CFFI now executes on a worker thread, although its case ID remains misleadingly named `cffi-current`.
- `git diff --check` is clean. All touched hand-maintained files remain below 900 lines; the largest is `python_interop_direct.rs` at 891 lines.

# Coverage of prior findings 4-9

- **4 — OPEN:** returned-error finalization improved, but native cancellation can abort before awaited rollback.
- **5 — OPEN:** async close/context handling improved, but early sync owner-method failures still bypass retained evidence.
- **6 — CLOSED:** call-scoped wrappers now drain before reconciling Python, handler, and cleanup failures; context cleanup failures are preserved.
- **7 — CLOSED:** escaped foreign shells use weak indirection, and owner close releases the separately owned typed target/captures.
- **8 — OPEN:** authoritative profile wiring and foreign-thread CFFI are fixed, but Pub/Sub active-drain evidence remains non-authoritative.
- **9 — OPEN:** the targeted architecture and roadmap files are corrected, but public diagnostic documentation still contradicts active support.

# Validation assessment

Insufficient for this wave. Existing artifacts report the compiled callback examples passing, but the Pub/Sub marker does not establish its claimed branch. Tests cover explicit/direct rollback, not generated retained-result cancellation through native task termination, and they do not cover early later-owner lookup failures. Build/test commands were not rerun because the review was required to remain read-only.

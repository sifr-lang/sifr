FAIL

Three blocking issues require exact edits before this is implementation-ready.

---

**Issue 1 — Duplicate row in Boundary And Communication Model table**

The process-boundary row appears twice verbatim:

```
| process boundary | payloads must be explicitly pipe-owned bytes/text or `IpcSerializable` typed frames |
| process boundary | payloads must be explicitly pipe-owned bytes/text or `IpcSerializable` typed frames |
```

Remove one copy.

---

**Issue 2 — M6 IPC design content bleeds into M4 "CPython tests to mine" section**

In `milestone_concurrency_runtime_4`, the "CPython tests to mine" block contains IPC-framing bullets (`payload eligibility`, `explicit supported payload types`, `serialization format`, `versioning`, `child-process bootstrap`, `result/error framing`, `cancellation/termination messages`, `panic-free malformed-message handling`, `compile-time diagnostics for unsupported payloads`) that are M6 design concerns, not CPython test sources for subprocess. These bullets have no CPython test path prefix, which is the evidence that they are displaced M6 content.

Required fix: move those bullets into the M6 scope section under a clearly labeled "IPC design requirements" heading, and replace the M4 "CPython tests to mine" block with only the two subprocess test paths that belong there:
- `Lib/test/test_subprocess.py`
- `Lib/test/test_asyncio/test_subprocess.py`

---

**Issue 3 — M1 definition of done bleeds M2 concerns**

`milestone_concurrency_runtime_1` DoD includes:

> Producer/consumer pipelines are test-covered.
> Channel and sync-primitive sendability/shareability diagnostics pass representative fixtures.

M1 is "Structured Async Runtime." Channels and sync primitives are M2 ("Synchronization, Channels, And Backpressure"). Sendability/shareability enforcement for channel value types is explicitly assigned to M2 in the phase-wide gate section. Having these in M1 DoD creates an ambiguous completion boundary: M1 cannot close without M2 work, and M2 has no clear starting edge.

Required fix: remove those two bullets from M1 DoD and add them to M2 DoD, which currently has no explicit definition-of-done section in the excerpt — M2 DoD must be written to receive them.

---

Everything else is sound. The structured-work framing is correct, threads/processes-as-substrates is explicit, typed IPC as one communication tier above process pipes is clearly separated, process workers and pools are properly gated on M4+M6, and the canonical `sifr.task`/`sifr.sync`/`sifr.process`/`sifr.runtime`/`sifr.parallel` model is consistently enforced throughout. The three issues above are editorial but they create real implementation ambiguity: one causes a schema error in the boundary contract, one would cause M4 implementers to treat IPC design as M4 subprocess work, and one makes M1/M2 completion boundaries undecidable.

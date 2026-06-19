## Review

**B1 — `docs/python-interop.mdx:103` blocking/async example: offload target `fetch_status_sync` is missing the required `@blocking_io` classification, which contradicts the implemented offload contract.**

The docs section "Blocking And Async" (`docs/python-interop.mdx:99-112`) shows:

```python
async def fetch_status() -> Result[str, PythonError]:
    handle = task.spawn_blocking(fetch_status_sync)
    return await handle.join()

def fetch_status_sync() -> Result[str, PythonError]:
    try:
        httpx: Object = import_module("httpx")
        ...
```

`fetch_status_sync` is a sync function with no `@blocking_io` classification, and is used as the target of `task.spawn_blocking`. Per the implemented async/offload contract, this emits `SIFR-ASYNC-0005` ("Blocking offload target is not classified as blocking I/O or CPU-heavy work"). Evidence:

- `verification/python_interop/fixtures/async_blocking/unclassified_offload_rejected.sifr:1` — same shape (sync fn calling `from_int`/`to_int`, no `@blocking_io`, used in `task.spawn_blocking`) is explicitly an `expect-error: SIFR-ASYNC-0005` negative fixture.
- `crates/sifr/tests/e2e/fail/spawn_blocking_unannotated_rejected.sifr:1-9` — generalized e2e fail confirms unannotated `spawn_blocking` target is rejected.
- The positive fixture `verification/python_interop/fixtures/async_blocking/offloaded_python_calls.sifr:14,26` requires `@blocking_io` on both `build_value` and `run_python_owned_loop`.

The docs text at `docs/python-interop.mdx:94` says "Every public `sifr.python` operation is classified as `@blocking_io`" and the architecture (`internal_docs/python_interop_architecture.md:47`) says "Async Sifr code must offload Python work through the existing blocking offload primitive" — but the example illustrating that exact offload pattern omits the user-side classification that the diagnostic enforces. The docs example as written will not compile.

Required fix: add `@blocking_io` decorator on `fetch_status_sync` (and import/note as appropriate), matching the offloaded_python_calls fixture pattern.

---

Other items checked and clean:
- Phase tracker / roadmap / phase index / exit evidence: status strings ("complete pending py12 PR merge", `milestone_py_12` checked with sub-bullet "PR link pending final documentation PR creation") are internally consistent. PR record py0–py11 listed; py12 marked pending.
- Diagnostic families `SIFR-PYENV-0001..0011` and `SIFR-PYTRUST-0001..0004` are stable, documented in `docs/diagnostics/error-codes.mdx:107-130`, registered in the exit-evidence file, and reserved families `PYIMP/PYCALL/PYCONV/PYRES/PYZC/PYCB` are noted consistently in all three docs.
- Verification group/tier selectors and report statuses (`passed`/`matrix-passed`/`scaffold`) are consistent across `docs/python-interop.mdx:319-327`, `internal_docs/python_interop_architecture.md:65-79`, `verification/python_interop/README.md:26-60`, and the exit evidence.
- Adapter/schema "quiet" support (`verification/runner/sifr_verify/area_adapter.py:397-411`, `verification/schemas/area.schema.json:162-164`, `verification/areas/project_workspace/manifest.json:47,67`) is appropriately scoped: schema adds optional boolean, adapter rejects `quiet: true` for any command other than `build`/`run`, manifest opts in only for the two run baselines whose stdout would otherwise vary by build progress. This matches the exit-evidence note about manifest-level quiet support for run baselines. Acceptable.
- Public example imports (`Object`, `PythonError`, `call_attr`, `from_str`, `from_int`, `from_float`, `from_list`, `from_dict_str`, `import_module`, `to_str`, `with_context`, `threadsafe_callback`, `zero_copy_arrow_stream`, `export_arrow_stream`, `zero_copy_dlpack_tensor`, etc.) match symbols present in `crates/sifr_stdlib/src/python.rs` and the canonical fixtures.
- Local validation evidence (132 pass / 0 fail create-pr; 651 pass / 0 fail merge gate; only advisory warm wall-time and group-skew) is recorded in the exit-evidence file and the issue tracker bullet.

Not satisfied — fix B1 before phase closure.

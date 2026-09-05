# Synchronous Python Context Example

The runnable SQLite transaction program is located at
`verification/areas/python_interop/fixtures/sqlite_context/context_codegen_smoke.sifr`.
It declares `sqlite3.Connection` with `cleanup=context` and exercises normal
fallthrough, a try body without returns, early return, narrowing through a
`let-else` path, break, continue, and an originating Python `AttributeError`.

Run the complete registered library example suite with:

```bash
uv run --project verification/areas/python_interop --locked \
  python verification/areas/python_interop/runner/run.py --library-examples
```

The SQLite context case must emit:

```text
sifr-python-interop:sqlite-context:total=71
```

The normative suppression, replay lifetime, exact-once exit, and cleanup-error
precedence cases are recorded in
`verification/areas/python_interop/fixtures/sqlite_context/sync_context_evidence.json`
and executed by the focused runtime and codegen tests.

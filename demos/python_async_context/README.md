# Typed Async Python Context Demo

This demo runs the compiled async-context fixture at
[`verification/areas/python_interop/fixtures/async_context/`](../../verification/areas/python_interop/fixtures/async_context/).
The generated binary uses the application-owned asyncio loop and real
`aiosqlite` over an in-memory SQLite database. It proves typed enter/value
conversion, Python-only suppression, unsuppressible Sifr causes, exact-once
exit/close, masked cancellation cleanup, secondary exit failure, and mixed
synchronous/asynchronous LIFO ordering without network access.

From this directory, run:

```bash
bash run.sh
```

The command must finish with:

```text
sifr-python-interop:async-context:value=sqlite-ready:enter=7:exit=7:close=7:loop=shared:suppression=covered:sifr=unsuppressed:cancellation=ordered:nested=lifo:exit-failure=covered
```

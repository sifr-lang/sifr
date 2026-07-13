# Typed Async Python Declaration Demo

This demo runs the compiled typed-async declaration fixture at
[`verification/areas/python_interop/fixtures/async_declaration/`](../../verification/areas/python_interop/fixtures/async_declaration/).
The generated binary starts one application-owned asyncio loop, constructs a
real `httpx.AsyncClient` over an offline ASGI transport, converts its typed
response, and consumes the client with exact-once async close.

From this directory, run:

```bash
bash run.sh
```

The command must finish with:

```text
sifr-python-interop:async-declaration:status=207:message=async-ready:close=1:loop=shared:failure=covered:conversion=covered
```

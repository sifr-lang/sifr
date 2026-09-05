# Typed Python Callback Demo

This demo runs the compiled declaration-first callback fixtures under
[`verification/areas/python_interop/fixtures/`](../../verification/areas/python_interop/fixtures/).
The generated binaries exercise a real CFFI callback, a Kafka object delivered
from a Python-created thread, an asyncio `AsyncCallable`, and a retained
Pub/Sub-style subscription whose consuming async close unregisters and drains
accepted handler invocations.

From this directory, run:

```bash
bash run.sh
```

The command must finish with all four markers:

```text
sifr-python-interop:callback:cffi=42
sifr-python-interop:callback:kafka=42
sifr-python-interop:callback:asyncio=42
sifr-python-interop:callback:pubsub=42:close=drained
```

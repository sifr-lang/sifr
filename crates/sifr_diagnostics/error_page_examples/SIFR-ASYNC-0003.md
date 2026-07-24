## Erroneous Code

```python
from sifr.io import read_text

async def load() -> str:
    return read_text("config.txt")
```

## How To Fix It

Move blocking I/O behind a supported offload helper or call an async API instead of blocking the task directly.

## Fixed Code

```python
from sifr.io import read_text

@blocking_io
def read_config() -> str:
    return read_text("config.txt")

async def load() -> str:
    return await task.spawn_blocking(read_config)
```

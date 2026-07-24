## Erroneous Code

```python
async def main() -> int:
    return await task.spawn_blocking(lambda: 42)
```

## How To Fix It

Only offload functions classified as blocking I/O or CPU-heavy work; otherwise leave ordinary synchronous helpers synchronous.

## Fixed Code

```python
def blocking_read() -> str:
    return read_text("config.txt")

async def main() -> str:
    return await task.spawn_blocking(blocking_read)
```

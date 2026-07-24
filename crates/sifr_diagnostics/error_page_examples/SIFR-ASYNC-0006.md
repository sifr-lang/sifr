## Erroneous Code

```python
@blocking_io
async def load() -> str:
    await task.sleep(0.0)
    return "done"
```

## How To Fix It

Apply synchronous workload annotations to synchronous functions, not async functions.

## Fixed Code

```python
@blocking_io
def load() -> str:
    return read_text("config.txt")
```

## Erroneous Code

```python
async def cached() -> int:
    return 1

async def main() -> int:
    return await cached()
```

## How To Fix It

Do not await a same-task coroutine that cannot suspend. Make the helper synchronous, or move the suspension into the helper.

## Fixed Code

```python
def cached() -> int:
    return 1

async def main() -> int:
    await task.sleep(0.0)
    return cached()
```

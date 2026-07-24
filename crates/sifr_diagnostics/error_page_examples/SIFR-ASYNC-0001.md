## Erroneous Code

```python
async def cached() -> int:
    return 1
```

## How To Fix It

Make the function synchronous when it never suspends, or add a real asynchronous operation if it truly belongs in async code.

## Fixed Code

```python
def cached() -> int:
    return 1
```

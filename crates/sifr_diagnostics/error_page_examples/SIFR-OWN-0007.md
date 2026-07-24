## Erroneous Code

```python
def consume(own values: list[int]) -> int:
    return len(values)

items: list[int] = [1, 2]
taken: int = consume(items)
count: int = len(items)
```

## How To Fix It

Move ownership only after the last use, keep mutable borrows scoped, and send only owned sendable values across task or channel boundaries.

## Fixed Code

```python
def consume(own values: list[int]) -> int:
    return len(values)

items: list[int] = [1, 2]
count: int = len(items)
taken: int = consume(items)
```

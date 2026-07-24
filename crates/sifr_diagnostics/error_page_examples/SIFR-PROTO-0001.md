## Erroneous Code

```python
for item in NotIterable():
    pass
```

## How To Fix It

Implement the required protocol shape, or pass a value that already satisfies the iterator, context-manager, hashable, or comparable requirement.

## Fixed Code

```python
class Items:
    def __iter__(self) -> Iterator[int]:
        return iter([1, 2, 3])

for item in Items():
    pass
```

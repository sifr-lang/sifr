## Erroneous Code

```python
class Counter:
    value: int

    def __str__(self) -> str:
        self.value += 1
        return str(self.value)
```

## How To Fix It

Keep operator and display methods read-only because their Rust trait receiver
shape is fixed.

## Fixed Code

```python
class Counter:
    value: int

    def __str__(self) -> str:
        return str(self.value)
```

## Erroneous Code

```python
def main( -> int:
    return 1
```

## How To Fix It

Fix the source syntax first. Parser diagnostics happen before Sifr can reason about types, ownership, or package structure.

## Fixed Code

```python
def main() -> int:
    return 1
```

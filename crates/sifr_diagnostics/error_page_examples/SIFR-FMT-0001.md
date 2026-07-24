## Erroneous Code

```python
def main()->int:
 return 1
```

## How To Fix It

Run `sifr fmt` to rewrite the file, or make the source match the formatter output before using `sifr fmt --check`.

## Fixed Code

```python
def main() -> int:
    return 1
```

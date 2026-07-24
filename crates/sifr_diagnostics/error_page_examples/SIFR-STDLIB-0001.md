## Erroneous Code

```python
values = defaultdict()
```

## How To Fix It

Use the supported `sifr.*` surface and constructor shape. Some CPython conveniences are intentionally absent or require explicit arguments.

## Fixed Code

```python
from sifr.collections import defaultdict

values = defaultdict(lambda: 0)
```

## Erroneous Code

```python
def crunch(values: list[int]) -> int:
    total: int = 0
    for value in values:
        total = total + value
    return total

async def main(values: list[int]) -> int:
    return crunch(values)
```

## How To Fix It

Keep CPU-heavy work out of the async executor. Use the parallel/offload surface intended for owned CPU work.

## Fixed Code

```python
from sifr.parallel import map as par_map

def crunch(value: int) -> int:
    return value * value

async def main(values: list[int]) -> list[int]:
    return par_map(values, crunch)
```

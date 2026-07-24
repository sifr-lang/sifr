## Erroneous Code

```python
from sifr.process import shell_output

async def version() -> str:
    return shell_output("sifr --version")
```

## How To Fix It

Use the async process API or an explicit offload boundary instead of direct shell execution from async code.

## Fixed Code

```python
from sifr.process import async_shell_output

async def version() -> str:
    return await async_shell_output("sifr --version")
```

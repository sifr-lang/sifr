## Erroneous Code

```python
from _sifr.io import read_text
```

## How To Fix It

Import the public stdlib wrapper instead of the private declaration.

## Fixed Code

```python
from sifr.io import read_text
```

## Erroneous Code

```python
from math import sqrt
```

## How To Fix It

Use Sifr's explicit module namespace and import concrete symbols with `from sifr.<module> import <name>`.

## Fixed Code

```python
from sifr.math import sqrt
```

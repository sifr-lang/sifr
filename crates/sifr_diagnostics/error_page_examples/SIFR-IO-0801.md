## Erroneous Code

```python
from sifr.io import open

file = open("notes.txt", "r")
```

## How To Fix It

Make file modes and encodings explicit at compile time, especially at text/binary boundaries.

## Fixed Code

```python
from sifr.io import open_text

file = open_text("notes.txt", encoding="utf-8")
```

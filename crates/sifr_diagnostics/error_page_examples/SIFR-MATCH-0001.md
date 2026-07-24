## Erroneous Code

```python
match value:
    case 0:
        label = "zero"
```

## How To Fix It

Make patterns supported, guards boolean, field names valid, and matches exhaustive where required.

## Fixed Code

```python
match value:
    case 0:
        label = "zero"
    case _:
        label = "other"
```

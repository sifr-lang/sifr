## Erroneous Code

```python
small: int8 = 200
```

## How To Fix It

Choose an integer type whose range fits the value, and make lossy or fallible numeric conversions explicit.

## Fixed Code

```python
small: int8 = 127
wide: int = 200
```

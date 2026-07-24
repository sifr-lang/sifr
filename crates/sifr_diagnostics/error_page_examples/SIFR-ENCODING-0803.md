## Erroneous Code

```python
text = decode(data, "utf-8", handler_name)
```

## How To Fix It

Use a statically known encoding error handler so text conversion remains predictable.

## Fixed Code

```python
text = decode(data, "utf-8", "strict")
```

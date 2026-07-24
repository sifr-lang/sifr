## Erroneous Code

```python
total = subtotal + tax
```

## How To Fix It

Declare or import the name before use, and check whether you meant a module member, type name, or local binding.

## Fixed Code

```python
subtotal: int = 10
tax: int = 2
total = subtotal + tax
```

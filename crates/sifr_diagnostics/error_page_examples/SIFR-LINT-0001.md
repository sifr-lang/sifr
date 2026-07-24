## Erroneous Code

```python
# sifr: ignore
def f(flag: bool):
    pass

f(True)
```

## How To Fix It

Fix the policy issue when possible. If suppression is intentional, list the exact policy rule id and keep it local.

## Fixed Code

```python
# sifr: ignore SIFR-LINT-0006
def f(flag: bool):
    pass

f(flag=True)
```

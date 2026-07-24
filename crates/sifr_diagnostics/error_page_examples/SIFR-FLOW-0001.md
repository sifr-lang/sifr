## Erroneous Code

```python
def label(value: int) -> str:
    if value > 0:
        return "positive"
```

## How To Fix It

Repair the control-flow shape: put break/continue inside loops, return on every required path, and use supported loop and assignment forms.

## Fixed Code

```python
def label(value: int) -> str:
    if value > 0:
        return "positive"
    return "zero or negative"
```

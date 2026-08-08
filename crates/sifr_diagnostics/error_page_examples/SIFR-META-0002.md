## Erroneous Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return produced_with_warning("shape", "shape.deprecated", {})
```

## How To Fix It

Update the static declaration identified by the package warning. Const-specialization warnings are hard compiler warnings and are intentionally not lint suppressions.

## Fixed Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return produced("shape")
```

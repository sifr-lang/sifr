## Erroneous Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return fatal("shape.unsupported", {"kind": "callable"})
```

## How To Fix It

Use a type supported by the specializing package, or change the static declaration that caused the package-owned fatal issue. The package reason code and bounded arguments explain the rejected contract.

## Fixed Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return produced("supported-shape")
```

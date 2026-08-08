## Erroneous Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return fatal("shape.unknown", {"undeclared_argument": "value"})
```

## How To Fix It

Declare the package reason code and its exact bounded argument template, then emit only those argument names. Compiler- and LSP-reserved names such as `rule` cannot be package template arguments.

## Fixed Code

```python
@const_specialize
def describe[T]() -> ConstSpecializationOutcome[str]:
    return fatal("shape.unsupported", {"kind": "unknown"})
```

## Erroneous Code

```python
class Payload:
    value: int

BOUNDARY = JsonIntegerBoundaryDescriptor(path="Payload.value")
```

## How To Fix It

Select `json.exact`, `json.web`, or `json.string_ints` and provide the declared integer kind and any required static range. Under `json.web`, numeric output must be statically proven to fit JavaScript's safe integer range; otherwise use the profile's decimal-string representation.

## Fixed Code

```python
BOUNDARY = JsonIntegerBoundaryDescriptor(
    path="Payload.value",
    profile="json.web",
    representation="decimal_string",
)
```

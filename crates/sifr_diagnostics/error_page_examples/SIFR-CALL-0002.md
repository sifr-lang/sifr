## Erroneous Code

```python
def greet(name: str) -> str:
    return "hi " + name

message = greet()
```

## How To Fix It

Make the call match the function signature: provide required arguments once, use known keyword names, and call only callable values.

## Fixed Code

```python
def greet(name: str) -> str:
    return "hi " + name

message = greet("Sifr")
```

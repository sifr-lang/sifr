## Erroneous Code

```python
class Point:
    x: int

    def __add__(self, other: Point) -> Point:
        return Point(self.x + other.x)
```

## How To Fix It

Declare the receiver convention that the Rust trait requires. Arithmetic
operators consume the receiver, so use `own self`.

## Fixed Code

```python
class Point:
    x: int

    def __add__(own self, other: Point) -> Point:
        return Point(self.x + other.x)
```

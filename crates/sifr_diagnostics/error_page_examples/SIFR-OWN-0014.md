## Erroneous Code

```python
class Counter:
    value: int

    def bump(self) -> None:
        self.value += 1

def update(maybe: Counter | None) -> None:
    if maybe is not None:
        maybe.bump()
```

## How To Fix It

Call the mutating method through a stable owned local, mutable parameter, or a
supported non-optional field place. In a constructor, initialize every declared
field (and call `super().__init__` for inherited storage) before the first
statement that reads or mutates `self`.

## Fixed Code

```python
class Counter:
    value: int

    def bump(self) -> None:
        self.value += 1

def update(mut counter: Counter) -> None:
    counter.bump()
```

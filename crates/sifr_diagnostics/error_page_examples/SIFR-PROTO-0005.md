## Erroneous Code

```python
class Readable(Protocol):
    def update(self) -> None:
        pass

class Counter:
    value: int

    def update(self) -> None:
        self.value += 1
```

## How To Fix It

Declare a mutable receiver in the protocol when conforming implementations may
mutate their receiver.

## Fixed Code

```python
class Updatable(Protocol):
    def update(mut self) -> None:
        pass

class Counter:
    value: int

    def update(self) -> None:
        self.value += 1
```

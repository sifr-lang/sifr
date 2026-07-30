## Erroneous Code

An overlapping field read, shared borrow, mutable borrow, or owned move cannot
occur in the same call as a mutable receiver or mutable argument.

```python
class Buffer:
    values: list[int]

    def replace_from(self, own other: Buffer) -> None:
        self.values = other.values

def replace_with_self(mut buffer: Buffer) -> None:
    buffer.replace_from(buffer)
```

## How To Fix It

Use a disjoint place for every other argument while a mutable receiver or
argument is active. If an argument only needs a value derived from the same
place, snapshot that value in a local before the call. For example, rewrite
`values.append(len(values))` as:

```python
count = len(values)
values.append(count)
```

## Fixed Code

```python
class Buffer:
    values: list[int]

    def replace_from(self, own other: Buffer) -> None:
        self.values = other.values

def replace(mut target: Buffer, own source: Buffer) -> None:
    target.replace_from(source)
```

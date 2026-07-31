## Erroneous Code

The same storage cannot participate in conflicting reads, moves, or mutable
borrows. Most often this happens when an overlapping field read, shared borrow,
mutable borrow, or owned move occurs in the same call as a mutable receiver or
mutable argument. It can also happen when a second `anext()` advance starts
before the first pending advance has been awaited.

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

For async generators, await the pending `anext()` result before starting the
next advance.

For example, this starts a second advance while the first one still holds the
generator:

```python
agen = numbers()
first = anext(agen)
second = anext(agen)
```

Await each advance before starting the next one:

```python
agen = numbers()
first = await anext(agen)
second = await anext(agen)
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

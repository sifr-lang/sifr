## Erroneous Code

```python
class User:
    name: str

user = User()
```

## How To Fix It

Give class declarations a shape the compiler can lower: initialized fields, valid bases, unique variants, and supported field ordering.

## Fixed Code

```python
class User:
    name: str

    def __init__(self, name: str):
        self.name = name

user = User("Ada")
```

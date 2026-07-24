## Erroneous Code

```python
read_text("notes.txt")
```

## How To Fix It

Handle typed failures explicitly. Use `try`/`except`, return the `Result`, or otherwise consume the value so failures cannot be ignored.

## Fixed Code

```python
try:
    text = read_text("notes.txt")
except IOError as e:
    text = ""
```

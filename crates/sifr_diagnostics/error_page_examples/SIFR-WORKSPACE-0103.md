## Erroneous Code

```toml
[workspace]
members = [123]
```

## How To Fix It

Fix workspace metadata so source roots stay inside the workspace and each entry has the expected string/path shape.

## Fixed Code

```toml
[workspace]
members = ["crates/app"]
```

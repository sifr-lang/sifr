## Erroneous Code

```toml
[package]
name = "demo"
```

## How To Fix It

Repair the package manifest, workspace selection, projection files, archive rules, or publish options named by the diagnostic.

## Fixed Code

```toml
[package]
name = "demo"
version = "0.1.0"

[sifr]
source-root = "src"
```

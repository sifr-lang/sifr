## Erroneous Code

```toml
[package]
name = "demo"
version = "0.1.0"
edition = "2024"

[package.metadata.sifr]
manifest = "../old/sifr.toml"
```

## How To Fix It

Run `sifr repair` to restore the Sifr-managed projection, or edit the managed Cargo metadata so it points at the package's `sifr.toml`.

## Fixed Code

```toml
[package]
name = "demo"
version = "0.1.0"
edition = "2024"

[package.metadata.sifr]
manifest = "sifr.toml"
```

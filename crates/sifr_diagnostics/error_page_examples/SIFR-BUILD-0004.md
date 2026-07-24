## Erroneous Code

```bash
sifr build app.sifr --output /root/out
```

## How To Fix It

Check the generated build path, Cargo/rustc output, and whether Sifr can create its temporary workspace and final artifact.

## Fixed Code

```bash
sifr build app.sifr --output build
```

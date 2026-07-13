# Hermetic Package-Local Python Bridge Demo

This demo runs the installed-archive biip bridge fixture at
[`verification/areas/python_interop/fixtures/package_bridge_archive/`](../../verification/areas/python_interop/fixtures/package_bridge_archive/).
It generates the package bridge inventory, creates a Cargo package archive,
unpacks it into a distinct install root, deletes the source checkout, and
builds the Sifr binary only from the archived inputs.

Before execution the installed `src/python_bridges/` tree is also deleted. The
binary then runs with its working directory and `TMPDIR` set to an empty
read-only directory, proving bridge loading uses the embedded table rather than
filesystem extraction or ambient `sys.path` ordering.

From this directory, run:

```bash
bash run.sh
```

The command must finish with:

```text
sifr-python-interop:package-bridge:gtin=7032069804988:format=13:check=8
```

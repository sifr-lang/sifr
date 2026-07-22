# Declaration-First Python Ecosystem Certification Demo

This demo compiles and runs the declaration-first callback, offline async HTTP,
NumPy buffer, Arrow, and DLPack examples, then prints the capability ledger
produced from those exact run reports. It demonstrates how the normal typed
declaration and hermetic-bridge experience is tied to executable ownership,
cleanup, callback, transfer, trust-root, and resource-zero evidence.

From this directory, run:

```bash
bash run.sh
```

The final line is:

```text
Python ecosystem certification: status=complete:capabilities=7:evidence=10:resources-zero=4
```

The complete machine-readable report is written under
`target/verification/areas/python_interop/`.

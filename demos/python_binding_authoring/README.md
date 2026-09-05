# Python Binding Authoring Demo

This package checks in the typed declaration generated from the package-local
`typing/math_override.pyi` source and its `sifr.python-bindings.json` evidence.

From this directory, after creating the locked environment with `uv sync`, run:

```bash
sifr python bind math --symbols sqrt --override typing/math_override.pyi
sifr python bind --check
sifr run src/main.sifr
```

The first command is the authoring mutation. The second is frozen and read-only;
the final command compiles and runs the generated declaration as an ordinary
Sifr module.

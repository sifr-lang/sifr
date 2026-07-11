# Python Interop Declaration Contract Demo

The declaration contract intentionally activates no declaration syntax. It
locks the complete declaration-first design and makes support claims executable
policy.

Run the demo through the Python interop scaffold:

```bash
verification/areas/python_interop/run.sh --group scaffold
```

The generated report at
`target/verification/areas/python_interop/latest.json` includes a
`declaration_capabilities` summary. The scaffold validates the separate
`verification/areas/python_interop/declaration_capabilities.json` ledger,
including target classification, implementation status, activation ownership,
and positive, negative, cleanup, cancellation, and live evidence ownership.

Run the negative policy checks with:

```bash
verification/areas/python_interop/run.sh --self-test
```

Those checks prove that duplicate capabilities, passing evidence on reserved
syntax, incomplete evidence on active syntax, and missing required cleanup
evidence are rejected.

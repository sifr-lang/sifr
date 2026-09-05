# Recursive Conversion And Opaque Lifecycle Demo

This runnable demo is the dependency-pinned biip/schwifty program at
[`verification/areas/python_interop/fixtures/simple_import/biip_schwifty_full_example.sifr`](../../verification/areas/python_interop/fixtures/simple_import/biip_schwifty_full_example.sifr).
It stays in the Python interop verification project so its real Python
dependencies and lockfile remain authoritative.

The program demonstrates the feature set in four sections:

1. Closed `Summary` records cross the Python boundary recursively inside a
   list, option, and tuple.
2. `@python.opaque` declarations seal biip and schwifty identities without
   exposing structural handle fields.
3. `@python.attr(Self.*)` methods read typed attributes through fallible
   `Result` boundaries.
4. Factories validate Python types, and the compiled program checks the nested
   values returned by biip, schwifty, and `builtins.tuple`.

Run the demo and its dependency/trust/probe checks with:

From this directory, run `bash run.sh`.

The command must finish with the biip/schwifty example marked `passed` and the
compiled program marker:

```text
sifr-python-interop:biip-schwifty:gtin=7032069804988:bic=DEUTDEFF
```

# Typed Python Buffer Demo

This runnable example builds and runs the declaration-first buffer fixtures under
[`verification/areas/python_interop/fixtures/numpy_buffer/`](../../verification/areas/python_interop/fixtures/numpy_buffer/).
The generated binaries exercise:

- a writable `builtins.bytearray` import-root producer;
- an opaque `mmap` receiver through `Self`;
- a package-local Python bridge producer with an observable data pointer and
  exact `bf_releasebuffer` counter;
- automatic cleanup through affine record, `Option`, list, tuple, union, and
  recursive aggregates with six exact exporter-release observations; and
- a real writable NumPy `int64` ndarray with checked metadata, typed element
  access, mutation, copying, and exact release.

From this directory, run:

```bash
bash run.sh
```

The command must finish with these five markers:

```text
sifr-python-interop:buffer:top-level=ok:resources=zero
sifr-python-interop:buffer:receiver=ok:resources=zero
sifr-python-interop:buffer:bridge=ok:resources=zero
sifr-python-interop:buffer:affine-aggregate=ok:resources=zero
sifr-python-interop:buffer:numpy=int64:write=42:identity=shared:resources=zero
```

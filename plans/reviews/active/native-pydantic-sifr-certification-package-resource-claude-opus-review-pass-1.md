# Native Pydantic-Sifr Package Resource Certification — Opus Pass 1

Exact base: `00ea8867569884943413f7809414839c8992db97`

Scope: working-tree implementation of `certification_pkg_resource_core` before
the first remediation commit.

## Verdict

`NOT SATISFIED`

## Blocking findings

- The bridge-local alias check cloned a plain value. `Handle<T>::clone` copied
  the open slot, so closing the original did not close the alleged alias. The
  reviewer required shared bridge-local lifecycle state.
- The compiler did not reject direct construction of a Rust-opaque class.
  Lowering accepted the constructor and deferred failure to generated Rust.
- The direct-construction test used lexical message matching instead of a
  stable diagnostic code and an exact bounded diagnostic set.
- Public and durable docs advertised alias and direct-construction rejection
  before either path had executable evidence.
- A second negative `.sifr` file reused another evidence id, declared a
  different execution kind, and was not registered by the fixture manifest.

## Required remediation

- Give bridge-local aliases shared lifecycle state and observe close through
  that state.
- Add a stable `SIFR-RUST-TYPE-0001` lowering diagnostic for direct Rust-opaque
  construction, with a focused compiler test.
- Activate compiler rejections from the registered negative evidence source,
  assert exact diagnostic ownership, and remove the unregistered file.
- Make close state and structural shape-mismatch behavior genuinely observed.

## Non-blocking suggestions adopted

- Probe a deliberate structural shape mismatch before construction.
- Observe close state through the bridge instead of appending a literal.
- Explain that `clone=none` still permits a private bridge-local shared alias
  used only for negative lifecycle evidence.

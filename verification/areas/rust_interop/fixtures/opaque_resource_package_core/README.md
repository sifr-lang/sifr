# opaque_resource_package_core

This fixture certifies the general external-package resource substrate. It
does not use a service crate or a Pydantic-specific adapter.

- Positive evidence `package_resource_construct_use_close` runs a generated
  package that opens a sealed opaque resource, constructs a typed structural
  record from it, projects that record through the same resource, and closes
  the resource.
- Negative evidence `package_resource_alias_use_after_close_rejected` observes
  a bridge-local alias reject access after close, stable double-close state,
  and redacted panic poisoning. The same mandatory test also checks that Sifr
  rejects a second close and direct construction of the package's opaque Rust
  resource.
- Compatibility is `supported-through-bridge` and `runtime-observed`. The
  evidence uses only the released structural bridge and `sifr_runtime` handle
  contracts.

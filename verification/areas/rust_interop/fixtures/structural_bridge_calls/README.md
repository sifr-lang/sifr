# structural_bridge_calls

Reserved evidence for monomorphized structural Rust bridge calls.

The row is intentionally planned while the structural contract is not yet
implemented. The generated-package evidence must establish that the bridge:

- round-trips nested generic and recursive values through a native opaque
  source with node-scoped construction and allocation-free projection; and
- rejects shape-identity, node ownership, callback signature, callback escape,
  and projection lifetime violations deliberately.

The row may become supported only when both directions pass atomically.

# structural_bridge_calls

Passing evidence for monomorphized structural Rust bridge calls. The generated
package establishes that the bridge:

- round-trips nested generic and recursive values through a native opaque
  source with node-scoped construction and allocation-free projection; and
- constructs and projects a package-mapped native nominal value, moves and
  drops it once, contains mapping panics, and constructs structural JSON output
  without a byte round trip; and
- rejects shape-identity, node ownership, callback signature, callback escape,
  and projection lifetime violations deliberately.

Both evidence directions pass atomically through the generated-build suite.

# bridge_v2_structural_calls

Future-owned Native Pydantic-Sifr `milestone_ps_2` evidence for bridge-version 2
monomorphized structural calls.

The row is intentionally planned while the compiler still rejects
`bridge-version = 2`. The implementation wave must replace these planned
evidence files with passing generated-package evidence that:

- round-trips nested generic and recursive values through a native opaque
  source with node-scoped construction and allocation-free projection; and
- rejects shape-identity, node ownership, callback signature, callback escape,
  and projection lifetime violations deliberately.

The row may become supported only when both directions pass atomically.

# M10 milestone review — pass 11

- Reviewer: agent
- Model: `agent`
- Reasoning: high
- Service tier: fast
- Reviewed HEAD: `e0643d71b71b042b277b34e06ed3fa32ca202e11`
- Scope: full M10 implementation, complete milestone history, and review passes 1–10
- Review tree: clean detached worktree
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — class union identities remain non-injective.** `Type::Class`
   union keys omit canonical declaration identity even when that identity
   selects a different emitted Rust representation. Canonical Python `Object`
   or file handles can therefore collide with local same-basename classes in a
   union and emit duplicate variants.
2. **High — several nominal reference paths bypass source-name escaping.**
   Synchronous and asynchronous Python record/opaque output conversion plus
   inheritance constructor and superclass method paths still emit raw class
   names. Legal `__Sifr*` records and classes named after external roots can
   pass checking but fail or change meaning in generated Rust.
3. **High — ordinary canonical stdlib classes remain source-collidable.** The
   merged-stdlib sealing pass protects only the four file-handle classes.
   Canonical classes such as `sifr.json.JsonValue` can still collide with a
   local same-basename class after both modules are flattened into one Rust
   namespace.

## Required remediation

- Include exact canonical/emitted nominal identity in class union keys and add
  native canonical-plus-local union regressions for Python `Object` and file
  handles.
- Route every nominal definition and reference through the shared canonical
  Rust-name renderer; add native synchronous/asynchronous conversion and
  inheritance collision coverage.
- Generalize compiler-owned Rust naming to every merged stdlib nominal and add
  a native alias-plus-local same-basename regression beyond file handles.

## Reviewer validation

- Re-grounded the phase contract, durable architecture, and all ten prior M10
  milestone review passes.
- Audited the complete `origin/main...HEAD` diff in a clean detached worktree.
- Traced source-name rendering through definitions, sync/async conversion,
  inheritance, union generation, stdlib merging, and canonical identities.
- Confirmed the supplied clean create-PR gate evidence, then identified that
  its smoke crate-test selection did not execute the deterministically failing
  type-system assertion for canonical/local `Object` union variants.

Final reviewer verdict: `CHANGES REQUESTED — actionable findings remain`.

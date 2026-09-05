# M10 milestone review — pass 13

- Reviewer: agent
- Model: `agent`
- Reasoning: high
- Service tier: fast
- Reviewed HEAD: `1606f56475f3ef65db97abff216f87ab99f4e924`
- Scope: complete `origin/main...HEAD` M10 implementation and pass-12 remediation
- Review tree: clean detached worktree
- Verdict: **CHANGES REQUESTED**
- Remediation status: **IMPLEMENTED AND VALIDATED; PENDING RE-REVIEW**

## Findings

1. **High — aliased imported inheritance loses parent semantics outside storage
   and `super()`.** Parent-field prescan, derive capabilities, and inherited
   `Display` still resolve `parent_class` against only local module classes.
   Direct inherited fields and supported traits therefore diverge from the
   exact imported `parent_type` carried by lowering.
2. **High — exact try-error identities are recorded but codegen represents only
   one error type.** Try lowering discards `body_error_types`, selects a single
   handler type for the closure, and skips other exact handlers. The explicit
   raise collector also omits propagating nested statement forms.
3. **High — Rust bridge contracts collapse distinct canonical classes sharing
   a basename.** Class bridge planning discards `Type::Class.identity`, resolves
   definitions by basename, and keys generated records by `(module, name)`, so
   the first same-named schema can be reused for a distinct canonical class.

## Required remediation

- Resolve imported-parent fields and trait/display capabilities from exact
  `HirClass.parent_type` or an identity-keyed parent capability registry, and
  add native coverage for direct inherited fields and supported traits.
- Preserve the complete exact try-error set through codegen using a
  discriminated carrier and identity-aware handler dispatch; make raise
  discovery exhaustive across propagating HIR statements and add native plus
  negative nested-raise regressions.
- Key Rust bridge definition resolution, generated names, schemas, and
  recursion tracking by canonical nominal identity plus specialization; add
  bridge-plan and compiled-probe coverage for same-basename imports.

## Reviewer validation

- Audited the complete `origin/main...HEAD` diff and the focused pass-12 delta
  in a clean detached worktree.
- Verified the four pass-12 findings are repaired in their direct paths.
- Traced adjacent lowering, HIR visitation, inheritance, try, match, generic,
  Rust/Python interop, collision, panic, ownership, and guardrail paths.
- Reported no separate Medium or Low findings.

Final reviewer verdict: `CHANGES REQUESTED — three High end-to-end gaps remain`.

## Remediation

- Imported classes now retain the richest exact canonical surface, including
  inherited fields, methods, derive capabilities, and `Display`; context-enter
  and compiler-special file-handle resolution rehydrate that exact surface
  instead of accepting a shallow same-identity snapshot.
- Try lowering now carries every exact propagated error identity through a
  discriminated carrier, emits conversions for each member, dispatches handlers
  by exact nominal type, preserves catch-all `Error`, and recursively discovers
  raises through nested propagating statements. Timeout, task-scope, async
  context, and `finally` envelopes inherit the active carrier.
- Rust bridge records are keyed, named, resolved, and recursively tracked by
  canonical module plus nominal identity, so equal basenames cannot share a
  schema. Bridge-plan and generated-project probes cover colliding imports.
- Adjacent generated-Rust regressions found by the complete suites were repaired:
  generic phantom markers are emitted only when a type parameter is otherwise
  unrepresented, and Python async contexts use the enclosing try carrier.

## Remediation validation

- Full merge-profile E2E: `664/664`, signature `76a3c67a1e579374`.
- Authoritative create-PR facade: every blocking lane passes in `1024.46s`,
  including Python interop `12/12`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131`, signature
  `7c39b8c1dd4fec7c`. Cache warmth and wall-time notices are advisory.
- Workspace Clippy with warnings denied, formatting, HIR maintainability, and
  the `900`-line file-size guardrail pass across `2705` files.
- Focused native proofs pass for aliased inherited formatting, exact TOML error
  routing, same-basename bridge records, canonical file handles, async
  cancellation/scope/process paths, network/TLS, i18n, and Python async-context
  execution.

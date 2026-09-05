# M10 Wave 2 review pass 22

- Reviewer: agent, `agent`, high reasoning, fast service tier
- Scope: complete committed `main...c2d13bd8e` diff after pass 21 remediation
- Verdict: **CHANGES REQUIRED**

## Findings

1. High: matching declaration identity made generic class specializations
   assignable without comparing their concrete arguments. `Box[int]` was
   accepted where `Box[str]` was required, and inferred returns collapsed the
   two specializations before the explicit-union guard could run.
2. High: canonical ancestry stopped when a child inherited from a class
   imported into its declaring module. Independently re-exporting that root and
   child then rejected the valid subclass-to-base conversion.
3. High: frontend exports retained generic-function metadata, but ordinary
   user-module imports never installed it in the consuming lowering context.
   Valid direct and multi-hop imported generic calls were rejected as `T`/`int`
   mismatches.
4. High: the module return-inference prepass used raw generic constructor and
   function result types without call-site substitution. An inferred function
   returning `Box(1)` emitted `Box<T>` with an unbound Rust type parameter.

## Required remediation

- Enforce invariant generic specialization in nominal assignability and reject
  conflicting inferred specializations before HIR/codegen.
- Preserve canonical identity through imported-parent ancestry and add a
  cross-module aliased-base native regression.
- Import and alias generic-function metadata and exact bounds through direct
  and multi-hop user-module re-exports.
- Share generic call binding/substitution with return inference, or keep an
  unresolved result non-authoritative until normal lowering.
- Add direct assignment/call, inferred-return, imported generic-call/bound, and
  cross-module ancestry native evidence.

## Cleared areas

The reviewer reconfirmed type system `103/103`, frontend `47/47`, lowering
`756` with one ignored, codegen `825/825`, driver `347` with `22` ignored, and
buffer runtime `25/25`. Workspace Clippy, formatting, diff hygiene,
HIR/driver maintainability, and source-size guardrails pass. No additional
buffer lifecycle, overlap-admission, affine release, or exact-once cleanup
defect was found.

Remediation validation and reviewer satisfaction are tracked in pass 23.

## Remediation disposition

All four findings are remediated for the pass 23 review:

- `Type::Class` now carries explicit specialization arguments independently of
  stable declaration identity. Nominal assignability compares those arguments
  invariantly, diagnostics render them, and inferred unions containing
  conflicting specializations are rejected before HIR/codegen.
- Exported ancestry follows imported parent aliases to their canonical
  declaration identities. Generated child structs also receive `Deref` and
  `DerefMut` bridges to their embedded parent, so the validated source subtype
  relationship is executable across module boundaries.
- User imports and re-exports install generic callable parameters and exact
  bounds. Project codegen propagates callable and class-method signatures
  through facade chains before emitting dependent modules.
- Return inference binds generic constructor/function calls before accepting an
  inferred type. Normal lowering also uses a concrete annotated initializer as
  context when a zero-argument generic return cannot be inferred from
  arguments. Optional contexts bind `T | None` to the non-`None` payload, and
  nominal class arguments participate in TypeVar discovery.

Permanent regressions cover invariant calls and assignments, conflicting
inferred returns, specialized inferred constructor/function results, direct and
multi-hop imported generic functions and bounds, imported-parent ancestry,
native facade builds, stdlib generic class templates, zero-argument contextual
generic returns, optional contextual binding, and recursive generic class
arguments.

Validation after remediation:

- type system `103/103`, frontend `47/47`, lowering `763` with one ignored,
  codegen `825/825`, driver `350` with `25` ignored, and the `25/25` native
  driver lane pass;
- workspace Clippy and formatting, HIR/driver maintainability, and the
  `900`-line source-size guardrail over `2678` files pass;
- the authoritative create-PR gate passes every blocking lane in `471.02s`,
  including Python interop `11/11`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with report signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. The only advisory is the
  non-blocking warm wall-time target.

# M10 Wave 2 review pass 14

Reviewer: Codex CLI `gpt-5.6-sol`, high reasoning, fast service tier

Scope: complete `main...6f58025c5` Wave 2 implementation after pass-13
remediation

Verdict: **CHANGES REQUIRED**

The complete branch review confirmed that the pass-13 `None` formatting and
diagnostic-precedence changes are sound, but reproduced five accepted-invalid
compiler families:

1. `sorted()` admitted cloneable element and key types that lack the total Rust
   `Ord` required by the emitted `.sort()` and `.cmp()` calls;
2. specialized generic classes and inherited classes were reported as
   `Display`/`Debug` capable even when the corresponding Rust implementation or
   derive was not emitted;
3. generic class specializations were accepted even when their arguments did
   not satisfy unconditional bounds placed on the generated Rust declaration;
4. affine `python.Buffer[T]` values could escape through reusable lambdas and
   nested-function closures;
5. an affine walrus alias was kept live in the compiler scope even though
   codegen declared it inside a temporary Rust block, with no coherent owner
   after the containing expression consumed its result.

The numeric pass-13 validation evidence is accurate, but the phase plan and
activation ledger overstate the completeness of trait-consumer and recursive
affine negative evidence while those paths remain open. The reviewer also
confirmed that formatting, diff hygiene, HIR maintainability, and the
repository-wide `900`-line file-size guardrail pass. The known dirty Ruff
submodule was explicitly ignored and preserved.

## Required remediation

- Align `sorted()` admission with the exact total-order trait used by each
  emitted path, including callable key return types.
- Make generic and inherited class capability queries derive from the traits
  actually emitted for their concrete Rust representation.
- Remove unnecessary declaration-wide generic bounds or reject every
  specialization that cannot satisfy them.
- Reject affine capture by reusable lambdas and nested functions before HIR can
  expose a callable that duplicates a single-use resource.
- Give affine walrus expressions coherent single-owner semantics and permanent
  negative evidence.
- Correct the activation and phase documentation, rerun the authoritative local
  facade, and obtain a fresh complete `main...HEAD` approval.

## Remediation

The compiler now validates the exact total-order type consumed by `sorted()`,
including callable key results. Generic declarations and constructors no longer
carry blanket Rust bounds; conditional derives, formatting implementations, and
individual methods carry their own proved bounds, and lowering rejects an
unsupported concrete method specialization. Inherited formatting follows the
embedded-parent representation.

Lowering now rejects affine buffer capture by reusable lambdas and nested
functions and rejects affine walrus aliasing before HIR can represent multiple
owners. Permanent regressions cover both non-`Ord` sorted paths, conditional
generic and inherited formatting, non-clone generic storage versus method use,
both reusable-closure forms, and affine walrus aliasing. Focused type-system,
lowering, code-generation, compile-fail, and native release checks pass.

The requested `cargo clean` removed `39.4 GiB` and made two cached native gaps
visible. The generated-code corpus showed that stdlib deduplication collapsed
distinct inherent impl blocks, while the uncached E2E matrix showed that
signature-only generic method bounds rejected channel receive and omitted the
key bounds required by `Counter[T]`. Inherent impl deduplication now includes
the contained item identities; stored hash-key parameters receive only their
representation-required `Hash + Eq`; and Clone/ordering bounds are attached
only when the emitted method body uses them. The structural datetime demo,
channel matrix, and collection ownership fixture all compile and run again.

Final validation passes: type system `102/102`, lowering `744` passed with one
ignored, code generation `818/818`, compile-fail `516/516`, workspace Clippy,
formatting, diff hygiene, HIR maintainability, and the `900`-line guard over
`2646` files. The authoritative create-PR facade passes Python interop `11/11`
in `104.05s`, runtime platform `28/28` with one gated skip, and E2E `131/131`
with signature `7c39b8c1dd4fec7c` and `42/42` cache hits. Its `416.20s` wall time
produced only the non-blocking warm-wall-time advisory. A fresh whole-branch
approval is the remaining gate.

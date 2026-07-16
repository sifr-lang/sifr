# M10 Wave 2 review pass 24

- Reviewer: Codex CLI, `gpt-5.6-sol`, high reasoning, fast service tier
- Scope: complete committed `main...9357041cf` diff after pass 23 remediation
- Verdict: **CHANGES REQUIRED**

## Findings

1. High: writable-buffer exclusivity followed direct parameter names only.
   Nested field and index receivers rooted in an immutable borrowed parameter
   could pass lowering before generated Rust rejected the mutable borrow.
2. High: consuming subclass coercion did not enter union or `Result` payloads.
   Nominally accepted child returns could be wrapped directly in an ancestor
   variant or `Ok`, leaving invalid generated Rust.
3. High: transitive consuming upcasts selected a basename match before a later
   exact canonical ancestor. Same-basename ancestors across modules could
   therefore request the wrong direct `Into` bridge.
4. High: the new owning phantom representation changed derive and auto-trait
   behavior without corresponding compiler capability analysis. Concrete
   fieldless specializations could pass Clone, formatting, equality, hash, or
   sendability checks that their emitted Rust representation did not satisfy.

## Cleared areas

The reviewer inspected the complete milestone diff and explicitly cleared the
runtime buffer acquisition, physical-range overlap admission, detach/release
ordering, exact-once resource identity, and no-user-panic paths. No additional
runtime lifecycle or admission blocker was found.

## Required remediation

- Trace mutable receiver places through fields and indices to the root binding,
  with nested immutable-negative and mutable-positive regressions.
- Coerce selected union and `Result` payloads before wrapping, including direct,
  transitive, and imported/re-exported native coverage.
- Prefer exact canonical ancestry matches and permit tail fallback only when it
  is unambiguous; cover repeated basenames across modules.
- Align the phantom representation and compiler trait/auto-trait analysis, with
  non-Clone, non-Debug, and non-send specialization regressions.

Remediation validation and reviewer satisfaction are tracked in pass 25.

## Remediation disposition

All four findings are addressed:

1. Mutable-receiver root tracing now follows legal field and index places to
   their root binding. Affine buffer projections remain rejected earlier by
   the existing whole-aggregate move rule (`SIFR-PYZC-0001`), including for
   mutable roots, so the reported nested buffer examples cannot bypass
   exclusivity. Permanent regressions cover immutable direct mutation,
   immutable and mutable field/index projection rejection, and the legal
   direct `mut` receiver path.
2. Union selection and `Result` construction now perform the consuming class
   conversion on the selected payload before constructing the variant or
   wrapper. Direct, transitive, imported, and re-exported native projects cover
   both union and `Result` returns.
3. Consuming upcasts choose an exact canonical ancestor first and use a
   basename fallback only when it is unique. A native three-module project with
   repeated `Root` basenames proves the exact ancestor wins.
4. Fieldless generic classes now use the non-owning representation
   `PhantomData<fn() -> T>`, avoiding accidental ownership and auto-trait
   inheritance from `T`. Compiler Clone, structural equality, Hash, and Debug
   capability checks now include concrete class arguments, while unresolved
   type parameters retain conditional generic-bound inference. Permanent tests
   cover non-capable concrete arguments and a non-send specialization that
   builds and moves through generated Rust without inheriting the argument's
   auto traits.

Focused regressions pass, the complete codegen suite passes `828/828`, the
driver suite passes `350/350`, and the generated native-project lane passes
`28/28`. Workspace Clippy and formatting pass; HIR and driver maintainability
checks pass; the `900`-line file-size guardrail passes over `2680` files; and
`git diff --check` is clean. The authoritative create-PR gate passes every
blocking lane in `537.98s`: Python interop `11/11`, runtime platform `28`
variants with one capability-gated skip, and E2E `131/131` with signature
`7c39b8c1dd4fec7c` and `36/42` cache hits. Its warm-time and cache-hit notices
are non-blocking advisories. Pass 25 reviews the complete remediated PR diff.

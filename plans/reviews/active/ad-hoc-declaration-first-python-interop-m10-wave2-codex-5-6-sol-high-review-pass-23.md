# M10 Wave 2 review pass 23

- Reviewer: Codex CLI, `gpt-5.6-sol`, high reasoning, fast service tier
- Scope: complete committed `main...9f7f0cf36` diff after pass 22 remediation
- Verdict: **CHANGES REQUIRED**

## Findings

1. High: zero-argument contextual generic specialization stopped at lowering.
   Codegen reconstructed class arguments only from fields and methods, ignored
   the explicit `Type::Class::type_args`, emitted an unspecialized plain call,
   and gave a fieldless generic class no representation for its type parameter.
   The accepted `Marker[int] = make()` regression therefore failed native Rust
   compilation with unused and uninferable `T` errors.
2. High: subclass assignability was executable only for borrowed values.
   Nominal checking accepted child-to-ancestor conversion for every ownership
   convention, while generated `Deref`/`DerefMut` bridges could coerce only
   references. Passing a child to an owned ancestor parameter, or returning it
   as an owned ancestor, emitted a Rust `Child`/`Root` mismatch.

## Required remediation

- Carry contextual generic bindings through call codegen, render authoritative
  explicit class arguments, represent phantom parameters, and add an exact
  native zero-argument regression.
- Lower consuming upcasts by moving the embedded parent, or reject them before
  HIR, and add direct plus re-exported owned-argument and owned-return native
  regressions.

## Cleared areas

The reviewer completed the whole-diff requirements and architecture review,
traced the pass-22 remediation, and inspected buffer lifecycle/no-panic paths.
No additional blocking buffer-release, overlap-admission, affine lifecycle, or
no-panic defect was found.

## Remediation disposition

Both findings are fixed for pass 24:

- Explicit class arguments are now authoritative during generic-return
  specialization. Concrete contextual locals retain their Rust annotation, and
  every generic class carries compiler-owned `PhantomData` so fieldless type
  parameters remain represented and inferable. The exact zero-argument
  `Marker[int] = make()` case builds and runs natively.
- Generated inheritance now includes move-based direct `From<Child> for Parent`
  bridges. Consuming arguments, locals, and returns lower child-to-ancestor
  coercions as one move conversion per ancestry edge, including transitive and
  imported/re-exported paths. Native direct, transitive, argument, and return
  regressions build and run without cloning.

Focused validation passes: codegen `827/827`, driver `350/350` with `27`
full-profile native tests ignored in the fast lane, and the native driver lane
`27/27`. Workspace Clippy, formatting, HIR/driver maintainability, and the
`900`-line source-size guardrail over `2680` files pass. After `cargo clean`,
the authoritative create-PR gate passes every blocking lane in `861.81s`:
Python interop `11/11`, runtime platform `28` variants with one
capability-gated skip, and E2E `131/131` with signature
`7c39b8c1dd4fec7c` after rebuilding all `42` fixture groups. Its only advisory
is the expected non-blocking warm-target timing warning. Reviewer satisfaction
is tracked in pass 24.

# Rust interop certification 6 review — round 8

Date: 2026-07-28

Reviewer: agent (`--effort medium`)

Reviewed head: `9fab8e1a9`

Verdict: **NOT SATISFIED**

The reviewer re-opened every round-1 through round-7 finding, ran both
mandatory generated packages, the Rust-interop area, lowering tests, and both
guardrails, and independently confirmed all earlier findings closed.

## New findings

1. **High:** retained-callback capture discovery did not traverse f-string
   interpolation, lambda bodies, or slice bounds. A valid non-`Copy` f-string
   capture used after attachment reached raw rustc `E0382`, while `NonSend` and
   unprovable callable captures hidden in f-strings, lambdas, or slice bounds
   could pass the Sifr contract check.
2. **Low:** mutation discovery had the mirrored gaps for comprehensions,
   interpolated strings, lambdas, slices, and starred expressions.
3. **Low:** inner-scope shadowing considered only regular positional
   parameters, so positional-only, keyword-only, variadic positional, and
   variadic keyword parameters could be mistaken for outer captures.
4. **Low:** public and internal support claims were unconditional while the
   expression traversal gaps remained.

## Resolution

Capture and mutation discovery now exhaustively match every expression AST
variant. Both walkers descend through comprehensions with lexical target
shadowing, lambda defaults and bodies, f-string and t-string interpolation
including nested format specifications, slice bounds, and starred
expressions. Nested function defaults and decorators are evaluated in their
enclosing scope, while every parameter kind is removed from the nested body
candidate set. Explicit terminal expression variants replace wildcard
fallbacks so future AST additions cannot silently reopen the contract.

Focused regressions cover f-string, lambda, slice, and starred hidden
`NonSend` capture, lambda-hidden callable provenance, comprehension/f-string/
slice mutation, safe f-string cloning, and complete parameter shadowing. The
mandatory positive generated package now compiles a `str` capture used only
inside an f-string and again after attachment. The mandatory negative package
adds f-string- and lambda-hidden `NonSend` captures plus a lambda-hidden
unprovable callable, with all 17 diagnostics asserted in the manifest-bound
driver test.

With the source-tree sysroot explicitly selected, both mandatory release
generated-package tests pass. The focused lowering regressions pass and the
docs and fixture contract are updated to state the complete traversed
expression surface.

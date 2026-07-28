# Rust interop certification 6 review — round 3

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `eb44ddace9`

Verdict: **NOT SATISFIED**

The reviewer confirmed that round-2 findings for transitive sibling capture
validation, cross-module metadata transport, and abort enforcement ordering
were closed. It also confirmed that owning `move` closure emission removed the
original `E0373`.

## New findings

1. **High:** blanket `move` emission consumes every non-`Copy` capture. A valid
   retained nested handler whose enclosing function reads a captured `str`
   after attachment passes Sifr checking but fails rustc with `E0382`. The
   positive generated-build fixture covered only a `Copy` integer capture.
2. **High:** a nested retained handler can capture a `Callable`-typed parameter
   whose own captures are unknown. Lowering treats the callable's signature as
   sendable, fails open, and generated Rust fails `Send + Sync` bounds with
   `E0277`.
3. **Low:** architecture, public docs, and issue evidence overstate the
   blanket-move guarantee while the two raw-rustc gaps remain.

## Required outcome

Clone verified non-`Copy` captures into an isolated closure-construction scope
before emitting the owning closure, preserve the enclosing bindings, reject
callable captures without verified nested/top-level provenance, add lowering
and generated-build regressions including loop attachment, correct the docs,
validate, and repeat the exact Opus review.

## Resolution

The remediation clones every verified non-`Copy` capture into an isolated
construction block before creating the owning closure, preserving the
enclosing binding after attachment and across loop iterations. Structured
loop lowering now supports nested retained handlers. Callable-valued captures
without compiler-known nested-function provenance fail with
`SIFR-RUST-CB-0001` before Cargo probing. Lowering and codegen regressions plus
both mandatory generated-package directions exercise these paths. The
corrected contract is reflected in the architecture, public docs, fixture
README, and certification issue before round 4.

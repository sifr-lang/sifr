# Rust interop certification 6 review — round 6

Date: 2026-07-28

Reviewer: agent (`--effort medium`)

Reviewed head: `2c44542da`

Verdict: **NOT SATISFIED**

The reviewer re-ran both mandatory generated packages, the lowering, codegen,
driver, frontend, runtime, and IR unit suites, workspace Clippy, formatting,
file-size and HIR maintainability guards, and independently reproduced every
round-1 through round-5 scenario. It confirmed all earlier findings closed.

## New findings

1. **Blocker:** moving the negative generated-build assertions to a child Rust
   module left the manifest-bound wrapper without a locally reachable
   `SIFR-RUST-CB-0001` assertion. Both real matrix checkers and the Rust-interop
   area gate failed even though their self-tests passed.
2. **High:** capture discovery ignored assignment targets. A retained handler
   whose only use of an enclosing value was `state.field = value` or
   `container[key] = value` passed `sifr check` and failed rustc with `E0525`.
   The same escape worked through a sibling nested function and in a mixed
   safe/unsafe capture set.
3. **Low:** the explicit genuinely-unresolved capture diagnostic added after
   round 5 had no direct regression.
4. **Low:** inference treated a captured user class named `Counter` as the
   builtin collection helper and reported a type the source did not contain.

## Resolution

The negative assertions again live in the exact Rust test file bound by the
fixture manifest, and the real fixture, compatibility, and area gates pass.
Capture discovery now traverses assignment and deletion targets. A focused
mutation collector operates over the nested function's actual captured
bindings and covers attribute/subscript targets, collection-mutating methods,
structured control flow, and sibling-transitive paths. Direct mutated captures
emit one deterministic `FnMut` contract diagnostic before the ordinary
send/share pass. The mandatory negative package covers `NonSend` attribute
assignment, dict subscript assignment, mutating `setdefault`, a mixed capture
set, and sibling-transitive assignment. Lexical lowered binding types now take
precedence over non-nested inference guesses, while nested-function provenance
remains intact. Direct tests cover both the unresolved diagnostic reason and a
user `Counter` class diagnostic with its true type.

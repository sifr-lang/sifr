# Rust interop certification 6 review — round 9

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `e9f26d1f0`

Verdict: **SATISFIED**

The reviewer independently re-opened all round-1 through round-8 artifacts and
reproduced the complete certification contract.

## Executed evidence

- both mandatory release generated-package tests passed with the source-tree
  sysroot: the lifecycle runtime and all 17 manifest-bound negative
  diagnostics;
- the Rust-interop area passed with 10 variants and no blocking or
  non-blocking failures;
- fixture, compatibility, tier, stable-claim, and stale-draft checks and their
  self-tests passed with the documented 36 rows, 5 tiers, and 29 claims;
- 1,901 lowering, codegen, runtime, IR, and frontend tests passed;
- workspace Clippy, formatting, file-size, HIR maintainability, and driver
  maintainability gates passed;
- the reviewer independently recomputed the issue inventory and confirmed
  every count.

## Findings

No actionable findings remain.

The reviewer explicitly reproduced hidden `NonSend` rejection through slice
bounds, starred expressions, all comprehension forms, generators, nested
f-string format specifications, deletion targets, inner functions, and
same-named comprehension target shadowing. Lambda-hidden `NonSend` and
unprovable callable captures reject before Cargo. Valid non-`Copy` captures
used only through f-strings, slices, or comprehensions retain the isolated
clone contract and remain usable after attachment.

It also confirmed exhaustive expression matching, every parameter kind,
receiver-aware `RwLock.write()` handling, the exact/inference mutation
candidate split, imported/aliased/re-exported/method callback metadata, abort
policy, handler consumption, nonlocal walrus rejection, transitive capture
provenance, runtime panic and queue policy, cleanup, cancellation, and locked
package evidence.

The reviewer noted an unrelated pre-existing general match-pattern lowering
defect, but demonstrated that it is not a reachable retained-callback contract
escape and requires a separate issue.

# Rust interop certification 6 review — round 4

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `9527ec367`

Verdict: **NOT SATISFIED**

The reviewer ran focused lowering, codegen, driver, mandatory generated-build,
file-size, and Rust-interop matrix checks. It confirmed that all round-1 and
round-2 findings and the two high-severity round-3 raw-rustc gaps were closed,
including isolated non-`Copy` capture clones, captures used after attachment,
loop-local handlers, transitive sibling captures, unprovable callable capture
rejection, and panic-abort ordering.

## New findings

1. **High:** attachment consumes a non-`Copy` generated closure, but Sifr does
   not mark the directly declared nested handler binding moved. A second
   attachment, direct call after attachment, or handler declared outside and
   attached inside a loop passes `check` and fails rustc with `E0382`.
2. **Medium:** isolated capture clones snapshot values where the nested handler
   is declared. Rebinding the enclosing value before attachment does not
   affect the retained handler, but the contract and tests do not state or pin
   that behavior.
3. **Low:** public/internal docs, fixture evidence, and issue evidence
   overstate pre-Cargo rejection and post-attachment behavior while handler
   reuse escapes and snapshot timing remains unstated.

## Required outcome

Treat a valid directly declared retained handler attachment as an ownership
move so every later use is rejected by Sifr before Cargo probing. Cover second
attachment, direct post-attachment invocation, and outer-loop attachment.
Define the capture snapshot point explicitly and pin it in generated/runtime
evidence, correct the claims, validate, and repeat the exact Opus review.

## Resolution

Successful attachment now force-marks a directly declared nested handler
binding moved even though its surface callable type is ordinarily copyable.
The ordinary ownership checker therefore rejects a second attachment and a
direct call after attachment with `SIFR-OWN-0001`; loop move analysis rejects
an outer handler attached across iterations with `SIFR-OWN-0004`. Direct
callable invocation now also consults moved binding state. The contract
defines capture cloning as a declaration-time snapshot, and codegen plus the
runtime-observed package prove that later outer rebinding is preserved without
changing the callback snapshot. The mandatory negative generated package
adds retained handler reuse, and docs/evidence distinguish capture binding
reuse from consumed handler reuse.

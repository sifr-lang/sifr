# Ad Hoc Phase: Idiomatic Rust Demo Corpus

Status: planned (created 2026-03-29)
Context: follow-up phase after the demo-closure/compiler-correctness work is complete and after the demo tree is again green under the current `run`/`test` sweep
Execution readiness: planning-ready; implementation must proceed in reviewed batches with external Rust-first review
Execution ledger: `issues/ad-hoc-idiomatic-rust-demo-corpus-execution.md`

## Objective

Produce a high-quality `idiomatic.rs` companion for every demo folder under `demos/` that contains Sifr source, using this standard:

- not “Rust that mirrors Sifr surface syntax”,
- not “Rust that mirrors current emitted code shape”,
- but “if an experienced Rust engineer wanted the same observable behavior and learning outcome, what would they write?”

This phase is explicitly comparative groundwork. The resulting Rust corpus is intended to become a strong reference set for later evaluation of:

- current emitted Rust quality,
- current compiler/codegen ceremony,
- ownership/borrowing ergonomics gaps,
- and places where emitted Rust is behaviorally correct but still structurally far from good Rust.

## Core Decision

The owning standard for `idiomatic.rs` in this phase is:

- preserve the observable result of the Sifr demo,
- preserve the feature being demonstrated,
- but prefer native Rust shapes even when they do not mirror the Sifr API surface one-to-one.

Examples of what this means:

- use slices instead of `&Vec<T>` where that is the right Rust API,
- use `entry()` instead of manual `get()` plus `insert()`,
- use `const`/`static` for constants instead of helper functions returning fresh `String`s,
- use owned values, borrowed values, and standard library APIs according to Rust norms,
- simplify control flow and formatting aggressively when the demo outcome stays the same,
- avoid generated-style scaffolding when a small direct Rust implementation is clearer.

What this phase must not do:

- change the demo’s externally visible outcome,
- silently weaken safety to make Rust shorter,
- introduce hidden panic paths that a careful Rust solution would avoid,
- contort Rust to preserve Sifr syntax when better Rust is obvious,
- or treat the current emitted Rust as a template to preserve.

## Source of Truth

- `demos/`
- current checked-in `idiomatic.rs` files under `demos/`
- current Sifr demo sources (`main.sifr`, `*_demo.sifr`, top-level demo `.sifr`, and demo-local support files where needed for meaning)
- current review artifacts under `tmp/idiomatic_review_batches/`
- `issues/ad-hoc-demo-closure-and-compiler-correctness.md`
- `issues/ad-hoc-demo-closure-and-compiler-correctness-execution.md`

## Why This Needs Its Own Phase

This is not compiler work and not demo reliability work.

It is a separate corpus-quality effort because:

- the repo already has `idiomatic.rs` files across the demo tree,
- many were bulk-backfilled or mechanically derived,
- the quality bar changed midstream to a Rust-first standard,
- and later emitted-vs-idiomatic comparison is only useful if the “idiomatic” side is actually defensible.

Without a dedicated phase, the repo risks keeping a large quantity of files named `idiomatic.rs` that are only lightly edited emitted Rust or Sifr-shaped Rust, which weakens any future comparison.

## Scope

This phase owns every directory under `demos/` that contains `.sifr` source and currently contains or should contain `idiomatic.rs`.

The phase uses two target tiers:

### Tier 1: Positive demo equivalents

Applies to:

- runnable demo entrypoint folders,
- demo folders whose source meaning is a positive feature demonstration,
- multi-file demo folders where one idiomatic Rust file can still express the same educational outcome.

Required outcome:

- `idiomatic.rs` is a real Rust-first equivalent for the demo’s behavior and point.

### Tier 2: Fixtures, tests, and negative cases

Applies to:

- negative-case folders,
- test-only fixture folders,
- support folders that do not correspond to one positive standalone program.

Required outcome:

- `idiomatic.rs` must still be intentional and readable,
- but it may be a minimal scaffold, harness, or explanatory equivalent rather than a fully runnable “positive demo”.
- it should normally remain syntactically valid Rust even when it is not meant to be a standalone runnable program.

This phase does not require forcing every negative/test fixture into a fake positive Rust program.

## Non-Goals

- changing Sifr demo behavior
- changing the emitted Rust in this phase
- making `idiomatic.rs` compile as a single unified Cargo project
- broad compiler or runtime fixes
- preserving a one-to-one mapping between Sifr APIs and Rust APIs when that hurts Rust quality

## Entry Baseline

Baseline assumptions for this phase:

- the current demo sweep is green again for the active `run`/`test` contract
- `idiomatic.rs` exists across the demo tree and now needs Rust-first cleanup, not just coverage
- current Claude review batches already show two facts:
  - many folders are acceptable with relatively small cleanup
  - the remaining problems cluster in a smaller subset of hand-authored or stdlib-heavy files

Operational baseline:

- authoritative corpus size:
  - every directory under `demos/` containing `.sifr`
- current review method:
  - batch review through Claude CLI using embedded file contents
  - Rust-first rubric
- current review output location:
  - `tmp/idiomatic_review_batches/`

## Quality Bar

An `idiomatic.rs` file is acceptable only if all of the following are true:

- it produces the same observable result or validates the same contract as the Sifr source
- its data ownership model is natural in Rust
- its APIs use standard Rust shapes where appropriate
- it avoids generated-style ceremony unless that structure is actually justified
- it does not introduce unnecessary allocations, clones, or formatting layers
- it does not preserve Sifr surface shape merely for symmetry
- it is readable enough that it could be shown as “how you’d write this in Rust”

## Review Standard

Every implementation batch in this phase must be reviewed under this exact question:

> If an experienced Rust engineer wanted the same result and educational value as the Sifr demo, would this be a strong, idiomatic Rust solution?

Review must prioritize:

1. behavioral equivalence
2. ownership/API/design quality
3. readability and Rust norms
4. style cleanups only when they materially improve the result

The review must not reject a file merely because it does not mirror Sifr’s public surface.

### Tier 2 review rubric

For Tier 2 folders, “acceptable” means:

- the file is clearly intentional rather than placeholder noise,
- the file communicates the relevant contract, failure shape, harness role, or fixture purpose,
- the Rust is still readable and structurally sane,
- and the file is not pretending to be a positive standalone demo when it is not one.

Tier 2 files are allowed to be minimal.
They are not required to be standalone runnable binaries, but they should normally remain syntactically valid Rust unless the file explicitly documents why it is only a sketch/harness artifact.

## Batch Strategy

Implementation should proceed in small batches, normally 3 to 5 demo folders at a time.

Each batch must follow this loop:

1. read the Sifr demo sources
2. inspect the current `idiomatic.rs`
3. rewrite or refine toward a Rust-first solution
4. run local sanity formatting/checks for the touched files
5. review the batch with Claude using the Rust-first rubric
6. apply follow-up fixes if Claude finds real issues
7. mark the batch complete in the execution ledger

## Concurrency Rule

The repo may be live while this phase runs.

Therefore implementation must:

- assume other agents or users may change nearby files,
- re-read target files before patching,
- avoid broad sweeps that overwrite many files at once,
- keep edits narrow and batch-local,
- and never “regenerate all idiomatic files” as a blind replacement step.

## Validation

For each touched batch:

- `rustfmt <touched idiomatic.rs files>`
- Claude Rust-first review for that batch

Optional when useful:

- `rustc` or a local Cargo harness for individual files if a file is complex enough to justify it

Phase-close validation:

- full coverage audit that every intended demo folder still has `idiomatic.rs`
- full review ledger showing each folder is either:
  - Rust-first acceptable, or
  - intentionally minimal under Tier 2 rules

## Execution Order

### wave_1_stabilize_the_review_standard

status: completed

Goals:

- freeze the Rust-first rubric
- stop using the older Sifr-surface-mirroring review criterion
- review and, if needed, revise the already-touched batches under the Rust-first standard

Deliverables:

- authoritative review prompt shape documented in the execution ledger
- first reviewed batch accepted under the Rust-first criterion

Progress update (`2026-03-29`):

- `batch_01_logging_time_timeit` completed local validation, passed external pass-1/pass-2 review, and is now accepted as the first fresh Rust-first batch for this phase

### wave_2_runnable_demo_corpus_pass

status: in_progress

Goals:

- work through positive runnable demo folders in small batches
- raise every positive demo `idiomatic.rs` to the Rust-first bar

Priority areas:

- files already flagged by prior Claude review
- stdlib-heavy demonstrations
  - meaning demos whose current `idiomatic.rs` embeds substantial library-like helper code or broad utility surfaces, which makes mechanical/codegen-shaped structure more likely to survive
- hand-authored files that still retain generated-style structure
  - meaning files that were manually edited but still visibly preserve emitted-Rust patterns such as repeated `format!` wrappers, helper-function faux-constants, `&Vec<T>`-everywhere APIs, or explicit ceremony around standard collection operations

Progress update (`2026-03-29`):

- `batch_01_statistics_json_datetime` selected as the first wave-2 runnable-demo batch because all three files remain large generated companions and prior review history already identified real design debt in the underlying surfaces
- `batch_01_statistics_json_datetime` completed local validation and passed external pass-1/pass-2 review with no accepted blockers

### wave_3_fixture_and_negative_case_normalization

status: pending

Goals:

- review negative/test/fixture folders under the Tier 2 rules
- replace confusing scaffolds with clearer minimal Rust equivalents where necessary

### wave_4_corpus_consistency_pass

status: pending

Goals:

- run a targeted consistency sweep across already-reviewed files
- remove repeated anti-patterns that remain after the main per-batch pass
- examples:
  - `&Vec<T>` where `&[T]` is better
  - `cloned().unwrap_or(...)` on `Copy` data
  - helper functions pretending to be constants
  - repeated `format!("{}", ...)` wrappers
  - redundant `.to_string()` chains
  - emitted-style ceremony around standard Rust collection operations

### wave_5_phase_closeout

status: pending

Goals:

- confirm every demo folder is accounted for
- confirm every reviewed batch is logged
- confirm the corpus is now fit for later emitted-vs-idiomatic comparison

## Exit Criteria

This phase is complete only when:

- every in-scope demo folder has an intentional `idiomatic.rs`
- every folder has passed through the Rust-first batch review flow
- Tier 1 files are acceptable as “good Rust for the same result”
- Tier 2 files are intentionally minimal and clearly justified
- the execution ledger records reviewed batches and outcomes
- and there is no remaining ambiguity about what `idiomatic.rs` means in this repository

## Risks

- over-correcting toward “nice Rust” while drifting from demo behavior
- under-correcting by leaving emitted-style structure in place
- treating Claude style suggestions as mandatory even when they do not matter
- conflicting with concurrent repo edits if patches are too broad

## Review Notes

This plan should be reviewed at the end of creation for:

- scope clarity between Tier 1 and Tier 2
- whether the validation bar is strong enough without becoming compiler work
- whether the batch strategy is small enough to survive concurrent edits
- and whether the Rust-first criterion is now unambiguous

## In-Scope Folder Rule

For this phase, scope is evaluated per directory containing `.sifr`.

That includes deeply nested directories under `demos/`, such as:

- negative-case subdirectories,
- test-only fixture subdirectories,
- and nested helper/demo fixture directories.

Parent folders and nested child folders may therefore both be independently in scope when both contain `.sifr` files.

## Concurrent Edit Handling

If a target file changes materially between read and patch:

- stop treating the earlier local read as authoritative,
- re-read the file,
- re-evaluate whether the current batch still makes sense as scoped,
- and defer the file to a later batch if the concurrent change makes ownership unclear.

“Batch-local” in this phase means:

- only files intentionally selected for the active batch,
- no cross-corpus cleanup edits outside those files,
- and no opportunistic rewrites of nearby folders while another agent may be changing them.

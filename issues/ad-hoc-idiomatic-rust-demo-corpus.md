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
- `batch_02_math_pathlib_glob` selected as the second wave-2 runnable-demo batch because all three demos are positive runnable parity surfaces, still retained generated-style companions, and had archived review history pointing at real stdlib/helper-shape debt
- `batch_02_math_pathlib_glob` completed local validation, accepted the pass-1 follow-up on path/error surface cleanup, and passed external pass-2 review with no remaining actionable issues
- `batch_03_io_csv_shutil` selected as the third wave-2 runnable-demo batch because it keeps the file/data-utility slice cohesive while targeting three still-generated companions with prior review history around IO and CSV behavior
- `batch_03_io_csv_shutil` completed local validation, accepted the pass-1 follow-up on `io` API shape and CSV correctness, and passed external pass-2 review with no remaining actionable issues
- `batch_04_uuid_platform_os` selected as the fourth wave-2 runnable-demo batch because it keeps the runtime-wrapper slice cohesive while targeting three phase-30-reviewed companions that were still much larger than the actual demo-visible behavior
- `batch_04_uuid_platform_os` completed local validation, accepted the pass-1 follow-up on `uuid` borrowing and `os` cleanup handling, and passed external pass-2 review with no remaining actionable issues
- `batch_05_base64_hashlib_bytes_module` selected as the fifth wave-2 runnable-demo batch because it keeps the encoding-and-digest utility slice cohesive while targeting three positive demos whose companions still carried generated-style helper surfaces despite small observable behavior
- `batch_05_base64_hashlib_bytes_module` completed local validation and passed external pass-1/pass-2 review with no actionable issues
- `batch_06_collections_itertools_heapq` selected as the sixth wave-2 runnable-demo batch because it keeps the container-and-iteration slice cohesive while targeting three phase-30-reviewed companions that still carried substantial generated-style helper structure
- `batch_06_collections_itertools_heapq` completed local validation, accepted the pass-1 follow-up on `collections` deque-capacity boundary clarity, and passed external pass-2 review with no remaining actionable issues
- `batch_07_string_textwrap_fnmatch` selected as the seventh wave-2 runnable-demo batch because it keeps the text-processing slice cohesive while targeting three phase-30-reviewed companions whose current implementations still ranged from faux-constant ceremony to oversized helper scaffolding
- `batch_07_string_textwrap_fnmatch` completed local validation and passed external pass-1/pass-2 review with no accepted blockers
- `batch_08_bisect_defaultdict_max_heap` selected as the eighth wave-2 runnable-demo batch because it keeps the ordered-lookup and heap-backed-container slice cohesive while targeting three small demos whose companions still carried more scaffolding than their demo-visible behavior required
- `batch_08_bisect_defaultdict_max_heap` completed local validation and passed external pass-1/pass-2 review with no accepted blockers
- `batch_09_binary_files_binary_hashing_binary_storage` selected as the ninth wave-2 runnable-demo batch because it keeps the binary-data slice cohesive while targeting three still-generated companions whose current implementations were much larger than the actual file, hashing, and hex-storage behavior each demo exposes
- `batch_09_binary_files_binary_hashing_binary_storage` completed local validation, accepted the pass-1 follow-up on removing an inert `binary_hashing` assertion, and passed external pass-2 review with no remaining actionable issues
- `batch_10_bytes_basics_bytes_constructors_bytes_roundtrip` selected as the tenth wave-2 runnable-demo batch because it keeps the core bytes-and-conversion slice cohesive while targeting three small demos whose companions still retained generated-style scaffolding around simple byte, UTF-8, and hex behavior
- `batch_10_bytes_basics_bytes_constructors_bytes_roundtrip` completed local validation and passed external pass-1/pass-2 review with no accepted blockers
- `batch_11_subprocess_tempfile_zipfile_io` selected as the eleventh wave-2 runnable-demo batch because it keeps the runtime-and-file-lifecycle slice cohesive while targeting three still-generated companions around command execution, temp paths, and zip archive IO
- `batch_11_subprocess_tempfile_zipfile_io` completed local validation, accepted the pass-2 follow-up on `subprocess.check_call` return semantics, and passed external pass-2 review with no remaining actionable issues
- `batch_12_readonly_bytes_tempfiles_and_zip_filesystem_and_archives` selected as the twelfth wave-2 runnable-demo batch because it keeps the bytes-and-archive-lifecycle slice cohesive while targeting three still-generated companions around read-only byte handling, temp-path-plus-zip behavior, and filesystem/archive orchestration
- `batch_12_readonly_bytes_tempfiles_and_zip_filesystem_and_archives` completed local validation, preserved the observed `zipfile.namelist = Ok(["inside.txt"])` output shape in `filesystem_and_archives`, and passed external pass-1/pass-2 review with no accepted blockers
- `batch_13_bytes_errors_bytes_file_io_bytes_iteration` selected as the thirteenth wave-2 runnable-demo batch because it completes the remaining small bytes-focused runnable slice around boundary errors, binary file roundtrips, and iteration/index semantics
- `batch_13_bytes_errors_bytes_file_io_bytes_iteration` completed local validation and passed external pass-1/pass-2 review; the only pass-2 note was rejected because `bytes_file_io` already uses the source-aligned `wave3` path name from the demo itself
- `batch_14_file_streams_in_memory_streams_text_and_bytes` selected as the fourteenth wave-2 runnable-demo batch because it keeps the stream-and-text/bytes slice cohesive while targeting three still-generated companions around file handles, in-memory IO surfaces, and UTF-8/hex conversion
- `batch_14_file_streams_in_memory_streams_text_and_bytes` completed local validation and passed external pass-1/pass-2 review with no accepted blockers; both review passes raised an invalid swapped-file claim that contradicted the actual file contents and same-named Sifr demos
- `batch_15_json_values_random_hashing_random_state` selected as the fifteenth wave-2 runnable-demo batch because it keeps the structured-data and deterministic-RNG slice cohesive while targeting three still-generated companions around JSON wrappers, hashing/base64 helpers, and explicit RNG state APIs
- `batch_15_json_values_random_hashing_random_state` completed local validation, accepted the pass-1 follow-up on explicit typed JSON access and poison-free RNG state handling, and passed external pass-2 review with no remaining actionable issues
- `batch_16_logging_and_timers_config_json_csv_collections_and_argparse` selected as the sixteenth wave-2 runnable-demo batch because it keeps the remaining object-wrapper and config-surface slice cohesive around logger/timer objects, structured-data parser wrappers, and collection-plus-argparse class APIs
- `batch_16_logging_and_timers_config_json_csv_collections_and_argparse` completed local validation, accepted the pass-2 follow-up on guarding the mini argparse `store` path against a missing value panic, and passed external pass-1/pass-2 review with no remaining actionable issues
- `batch_17_classes_protocols_pattern_matching` selected as the seventeenth wave-2 runnable-demo batch because it keeps the milestone-language slice cohesive around core struct/object behavior, protocol/operator surfaces, and direct Rust pattern matching equivalents
- `batch_17_classes_protocols_pattern_matching` completed local validation, accepted the pass-1 follow-up on the `Port::value` getter borrowing style, and passed external pass-2 review with no remaining actionable issues
- `batch_18_iterators_and_randomness_error_handling_decorators` selected as the eighteenth wave-2 runnable-demo batch because it keeps the remaining callable-and-control-surface slice cohesive around iterator adapters, random/secrets helpers, decorator-recognition placeholders, and direct `Result`/error-handling demos
- `batch_18_iterators_and_randomness_error_handling_decorators` completed local validation, accepted the pass-1 follow-up on `iterators_and_randomness` product/digest helper semantics, and passed external pass-2 review with no remaining actionable issues
- `batch_19_env_regex_regex_and_filesystem` selected as the nineteenth wave-2 runnable-demo batch because it keeps the remaining environment-and-text/filesystem slice cohesive around env var helpers, regex primitives, and regex-plus-path iteration parity
- `batch_19_env_regex_regex_and_filesystem` completed local validation and passed external pass-1/pass-2 review with no accepted blockers
- `batch_20_iter_and_next_cloned_iterators_lazy_iterators` selected as the twentieth wave-2 runnable-demo batch because it keeps the remaining small iterator-consumption slice cohesive around direct `iter`/`next` usage, reusable borrowed iterator transforms, and lazy generator-backed iteration
- `batch_20_iter_and_next_cloned_iterators_lazy_iterators` completed local validation and passed external pass-1/pass-2 review with no accepted blockers
- `batch_21_iterator_basics_generic_functions_and_iterators_itertools_iterators` selected as the twenty-first wave-2 runnable-demo batch because it keeps the remaining iterator-protocol slice cohesive around explicit iterator consumption, generic helpers with iterator-heavy output, and small itertools-style iterator surfaces
- `batch_21_iterator_basics_generic_functions_and_iterators_itertools_iterators` completed local validation, accepted pass-1/pass-2 ownership-parity follow-ups in `generic_functions_and_iterators`, and passed external pass-2 review with no remaining actionable issues
- `batch_22_iteration_basics_iterator_builtins_iterators_and_comprehensions` selected as the twenty-second wave-2 runnable-demo batch because it keeps the remaining small iterator-basics slice cohesive around direct sequence iteration, built-in iterator helpers, and comprehension-style eager collection
- `batch_22_iteration_basics_iterator_builtins_iterators_and_comprehensions` completed local validation, accepted pass-1 follow-ups clarifying borrowed-iterator reuse and `sorted(..., reverse=True)` parity, and passed external pass-2 review with no accepted blockers
- `batch_23_generator_functions_generator_iterators_custom_iterables` selected as the twenty-third wave-2 runnable-demo batch because it keeps the remaining small generator-and-custom-iterable slice cohesive around lazy countdown/generator surfaces, filtered generator expressions, and explicit custom iterator state
- `batch_23_generator_functions_generator_iterators_custom_iterables` completed local validation, accepted a pass-2 follow-up preserving lazy generator-expression structure in `generator_iterators`, and ended external review with no remaining actionable issues
- `batch_24_extended_builtin_iterators_reversible_iterables_lazy_builtins` selected as the twenty-fourth wave-2 runnable-demo batch because it keeps the remaining small builtin-iterator slice cohesive around `reversed`/`enumerate`/`zip`/`map` behavior and capability-aware reversible iteration
- `batch_24_extended_builtin_iterators_reversible_iterables_lazy_builtins` completed local validation and ended external review with no accepted blockers; reviewer timeout/ownership notes were rejected where they contradicted the paired source or actual Rust 2021 iterator semantics
- `batch_25_generators_generator_break_else_iterator_types` selected as the twenty-fifth wave-2 runnable-demo batch because it keeps the remaining small generator/protocol slice cohesive around generator control flow, context-managed generator demos, and first-class iterator type contracts
- `batch_25_generators_generator_break_else_iterator_types` completed local validation, accepted a pass-2 follow-up removing an extra unsourced `passthrough` call from `iterator_types`, and ended external review with no remaining actionable issues
- `batch_26_lazy_iterators_basics_iterator_lowering_iterator_codegen` selected as the twenty-sixth wave-2 runnable-demo batch because it keeps the remaining canonical iterator-lowering slice cohesive around `iter`/`map`/`filter`/`zip`/`enumerate`/`reversed`/`count` behavior and the smallest built-in lowering demos
- `batch_26_lazy_iterators_basics_iterator_lowering_iterator_codegen` completed local validation, accepted pass-1 borrowed-iterator clarity follow-ups in `lazy_iterators_basics` and `iterator_codegen`, and ended external pass-2 review with no remaining actionable issues
- `batch_27_recursive_calls_recursive_for_else_while_else` selected as the twenty-seventh wave-2 runnable-demo batch because it keeps the remaining small structured-control-flow slice cohesive around recursive `for-else`, direct recursion, and `while-else` branch semantics
- `batch_27_recursive_calls_recursive_for_else_while_else` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; the lone pass-1 `recursive_for_else` note was rejected because the reviewed file already printed `rec(3)` and matched the observed runtime output
- `batch_28_borrow_by_default_borrowed_builtins_generic_cloning` selected as the twenty-eighth wave-2 runnable-demo batch because it keeps the remaining ownership/borrowing slice cohesive around borrow-by-default parameters, non-consuming builtins, and simple generic collection traversal
- `batch_28_borrow_by_default_borrowed_builtins_generic_cloning` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; the repeated `borrow_by_default` `String`-vs-`char` note was rejected because the corpus maps Sifr `str` to Rust `String` and the companion already matched the demo-visible behavior
- `batch_29_type_checking_constrained_typevars_protocol_bounds` selected as the twenty-ninth wave-2 runnable-demo batch because it keeps the remaining small type-system slice cohesive around frontend-only typed success, constrained `TypeVar` examples, and protocol-bound forwarding
- `batch_29_type_checking_constrained_typevars_protocol_bounds` completed local validation, accepted pass-1/pass-2 follow-ups tightening the explicit `TypeVar` and `Comparable` constraint modeling in `constrained_typevars`, and ended external review with no remaining actionable issues
- `batch_30_early_return_paths_unreachable_returns_valid_control_flow` selected as the thirtieth wave-2 runnable-demo batch because it keeps the remaining small CFG/control-flow slice cohesive around early-return lowering, unreachable-tail inference, and loop control-flow validity
- `batch_30_early_return_paths_unreachable_returns_valid_control_flow` completed local validation and ended external pass-1/pass-2 review with no accepted blockers
- `batch_31_optional_indexing_optional_arithmetic_return_type_inference` selected as the thirty-first wave-2 runnable-demo batch because it keeps the remaining small option-and-inference slice cohesive around safe optional indexing, narrowed optional arithmetic, and inferred return behavior
- `batch_31_optional_indexing_optional_arithmetic_return_type_inference` completed local validation, accepted a pre-review parity fix preserving the quoted string output in `return_type_inference`, and ended external pass-1/pass-2 review with no accepted blockers
- `batch_32_monotonic_indices_reverse_indices_indexed_tables` selected as the thirty-second wave-2 runnable-demo batch because it keeps the remaining index-safety slice cohesive around monotonic forward indices, reverse-range indices, and direct indexed writes into pre-sized tables
- `batch_32_monotonic_indices_reverse_indices_indexed_tables` completed local validation and ended external pass-1/pass-2 review with no accepted blockers

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

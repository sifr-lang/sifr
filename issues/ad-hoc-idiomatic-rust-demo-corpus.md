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
- `batch_33_local_shadowing_sentinel_values_set_operations` selected as the thirty-third wave-2 runnable-demo batch because it keeps the remaining small state-and-collections slice cohesive around local rebinding, sentinel collapse, and basic set operations
- `batch_33_local_shadowing_sentinel_values_set_operations` completed local validation, accepted a pass-2 cleanup removing an unnecessary temporary allocation in `set_operations`, and ended external review with no remaining actionable issues
- `batch_34_container_literals_collection_cloning_own_mut_appends` selected as the thirty-fourth wave-2 runnable-demo batch because it keeps the remaining small collections-and-owned-mutation slice cohesive around typed container updates, collection-transformation cloning, and append-and-return helpers
- `batch_34_container_literals_collection_cloning_own_mut_appends` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; the pass-1 `container_literals` semantics note was recorded but not accepted because the Rust companion already matched the paired demo assertions and observed runtime behavior
- `batch_35_container_methods_dict_membership_ordered_collections` selected as the thirty-fifth wave-2 runnable-demo batch because it keeps the remaining collection-API slice cohesive around container methods, guarded dict membership reads, and ordered collection helpers
- `batch_35_container_methods_dict_membership_ordered_collections` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; the reviewer transport for `ordered_collections` was unstable in pass 2, but the final shortened retry completed cleanly with no actionable issues
- `batch_36_typed_queues_heap_option_drain_own_mut_updates` selected as the thirty-sixth wave-2 runnable-demo batch because it keeps the remaining owned-mutation-and-drain slice cohesive around queue draining, option-safe heap draining, and `own mut` in-place list updates
- `batch_36_typed_queues_heap_option_drain_own_mut_updates` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; `typed_queues` needed a retry in pass 1 after an unusable reviewer stub, but the final verdict for all three files was clean
- `batch_37_owned_mutation_parameters_part1_owned_mutation_parameters_part2_subscript_mutation` selected as the thirty-seventh wave-2 runnable-demo batch because it keeps the remaining ownership-and-mutation surface cohesive around parameter-mode examples and direct list/dict subscript mutation
- `batch_37_owned_mutation_parameters_part1_owned_mutation_parameters_part2_subscript_mutation` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; `subscript_mutation` needed a pass-1 retry after an unusable reviewer stub, but the final verdict for all three files was clean
- `batch_38_safe_collections_safe_indexing_guarded_sequence_index` selected as the thirty-eighth wave-2 runnable-demo batch because it keeps the remaining safety-and-indexing slice cohesive around panic-free collection APIs, safe indexing surfaces, and guard-proven definite sequence reads
- `batch_38_safe_collections_safe_indexing_guarded_sequence_index` completed local validation and ended external pass-1/pass-2 review with no accepted blockers; the pass-2 `safe_indexing` reviewer response added an echoed behavior sentence, but it still reported no actionable issues
- `batch_39_fixed_indexing_indexing_rules_safe_edge_cases` selected as the thirty-ninth wave-2 runnable-demo batch because it keeps the remaining indexing-edge-case safety slice cohesive around len-guarded fixed indexes, negative-index mutation/delete rules, and validation-heavy edge-case handling
- `batch_39_fixed_indexing_indexing_rules_safe_edge_cases` completed local validation, accepted a pre-review parity fix correcting the UUID invalid-chars validation order in `safe_edge_cases`, and ended external pass-1/pass-2 review with no remaining actionable issues
- `batch_40_paired_indices_pop_narrowing_range_aliasing` selected as the fortieth wave-2 runnable-demo batch because it keeps the remaining index-and-narrowing work cohesive around two-pointer sequence reads, non-empty `pop` narrowing, and `len(...)` alias range guards
- `batch_40_paired_indices_pop_narrowing_range_aliasing` completed local validation and ended external pass-1/pass-2 review with no accepted blockers
- `batch_41_slice_unpacking_subscript_assignment_tuple_assignment` selected as the forty-first wave-2 runnable-demo batch because it keeps the remaining assignment-and-unpacking slice cohesive around safe indexing displays, direct subscript mutation, and tuple-style state updates
- `batch_41_slice_unpacking_subscript_assignment_tuple_assignment` completed local validation and ended external pass-1/pass-2 review with no accepted blockers
- `batch_42_loop_try_match_return_and_raise_paths_reachability` selected as the forty-second wave-2 runnable-demo batch because it keeps the remaining small error-and-flow-query slice cohesive around try/except-driven returns, reachability truth, and loop-else control-flow
- `batch_42_loop_try_match_return_and_raise_paths_reachability` completed local validation, accepted a pre-review cleanup removing a dead helper type from `loop_try_match`, and ended external pass-1/pass-2 review with no accepted blockers
- `batch_43_type_system_union_ops_union_narrowing` selected as the forty-third wave-2 runnable-demo batch because it keeps the remaining type-and-union surface cohesive around alias-backed value flow, optional arithmetic and length operations, and direct enum narrowing
- `batch_43_type_system_union_ops_union_narrowing` completed local validation and ended external pass-1/pass-2 review with no accepted blockers
- `batch_44_platform_os_system_tools` selected as the forty-fourth wave-2 runnable-demo batch because it keeps the remaining runtime-and-system surface cohesive around platform introspection, small filesystem and process helpers, and a compact integrated tools demo
- `batch_44_platform_os_system_tools` completed local validation, accepted a pre-review parity fix correcting the `timeit.repeat` count in `system_tools`, and ended external pass-1/pass-2 review with no accepted blockers; the only pass-2 `system_tools` note was rejected because its cited lines did not match the claim and the follow-up rereview devolved into contradictory transport noise rather than a real file-local issue
- `batch_45_generic_classes_generics_impl_forward_refs` selected as the forty-fifth wave-2 runnable-demo batch because it keeps the remaining generics-and-forward-reference surface cohesive around generic containers, generic higher-order helpers, and forward-declared type references
- `batch_45_generic_classes_generics_impl_forward_refs` completed local validation, accepted a pass-2 parity fix changing `generic_classes::Stack::size` from `usize` to `i64`, and ended external review with no remaining actionable issues; the pass-1 lane was recorded but its notes were rejected because they were stale or self-contradictory relative to the checked-in Rust files
- `batch_46_local_imports_stdlib_loading_stdlib_modules` selected as the forty-sixth wave-2 runnable-demo batch because it keeps the remaining stdlib-loading surface cohesive around direct stdlib constant imports, registry-backed module exposure, and tiny import-path smoke demos
- `batch_46_local_imports_stdlib_loading_stdlib_modules` completed local validation and ended external review with no accepted blockers; pass 1 on `stdlib_loading` and `stdlib_modules` returned stale file-role inversions that were rejected, and pass 2 accepted the embedded-source `stdlib_loading` `OK` verdict as authoritative while rejecting a later minimal rerun that repeated the same inversion
- `batch_47_builtin_functions_builtin_callables_stdlib_functions` selected as the forty-seventh wave-2 runnable-demo batch because it keeps the remaining builtin-and-stdlib callable surface cohesive around tiny builtin formatting demos, callable constructor/helper parity, and the last broad stdlib helper companion in the runnable corpus
- `batch_47_builtin_functions_builtin_callables_stdlib_functions` completed local validation and ended external review with no accepted blockers; pass 1 rejected `stdlib_functions` notes about unexercised negative-input and error-string paths, pass 2 returned clean `builtin_functions`/`stdlib_functions` verdicts, and `builtin_callables` had to be carried with an explicit reviewer-transport note after multiple prompt variants stalled in this workspace
- `batch_48_class_libraries_advanced_class_libraries_inheritance` selected as the forty-eighth wave-2 runnable-demo batch because it keeps the remaining class-oriented API surface cohesive around the largest class-based stdlib demos plus the smaller inheritance/classmethod/staticmethod milestone demo
- `batch_48_class_libraries_advanced_class_libraries_inheritance` completed local validation, accepted a pass-1 parity fix in `class_libraries` so the harness prints only the first three `static_order()` entries like the paired Sifr demo, and ended external review with no accepted blockers; `advanced_class_libraries` and `inheritance` were clean in pass 1, `inheritance` was clean again in pass 2, and the two larger class-library pass-2 reviewer prompts stalled and were carried with explicit transport notes instead of fabricated verdicts
- `batch_49_core_stdlib_extended_stdlib_additional_modules` selected as the forty-ninth wave-2 runnable-demo batch because it keeps the remaining stdlib-utilities surface cohesive around core file/json/env/math helpers, time/random/regex/hash/base64 helpers, and the last integrated additional-modules demo covering operator/calendar/html/sys/subprocess/configparser/gzip/zipfile behavior; `html_and_textwrap` was explicitly deferred because its current `main.sifr` fails to compile in the repo and would have broken the required targeted-demo validation lane
- `batch_49_core_stdlib_extended_stdlib_additional_modules` completed local validation and ended external review with no accepted blockers; pass 1 was clean except for an `extended_stdlib` note that was rejected because it reviewed internal helper naming and then drifted into `main.sifr`, pass 2 returned a clean `core_stdlib` verdict, rejected the same non-blocking `extended_stdlib` helper-name note, and carried `additional_modules` with an explicit unusable-review note after the response inverted file roles and ended in a contradictory mixed verdict
- `batch_50_stdlib_stdlib_expansion_stdlib_aliases` selected as the fiftieth wave-2 runnable-demo batch because it keeps the remaining stdlib-milestone slice cohesive around the original parity milestone demo, the first expansion milestone demo, and the later alias/naming-alignment milestone demo
- `batch_50_stdlib_stdlib_expansion_stdlib_aliases` completed local validation and ended external review with no accepted blockers; pass 1 on `stdlib` and `stdlib_expansion` produced unusable stale/generated-shape comparisons that were rejected, pass 1 on `stdlib_aliases` raised only a non-blocking unexercised `fnmatch_filter` generalization note plus small style nits, and pass 2 returned clean `stdlib`/`stdlib_expansion` verdicts while rejecting `stdlib_aliases` notes about exact platform string spellings and internal helper error typing because neither changes the paired demo-visible behavior
- `batch_51_stdlib_fixes_pure_stdlib_generic_stdlib` selected as the fifty-first wave-2 runnable-demo batch because it keeps the remaining stdlib-heavy milestone slice cohesive around the remediation milestone demo, the pure-stdlib expansion milestone demo, and the generic-stdlib rewrite milestone demo
- `batch_51_stdlib_fixes_pure_stdlib_generic_stdlib` completed local validation and ended external review with no accepted blockers; pass 1 on `stdlib_fixes` stalled entirely, pass 1 on `pure_stdlib` and pass 2 on `pure_stdlib` returned stale/generated-shape claims that were rejected, pass 1 on `generic_stdlib` raised only implementation-strategy/style notes rather than demo-visible mismatches, pass 2 on `stdlib_fixes` accepted one real follow-up adding an explicit suppressed `Logger::info` path, and pass 2 on `generic_stdlib` stalled without a usable verdict
- `batch_52_structured_parsing_serialization_parse_safety_no_runtime_panics` selected as the fifty-second wave-2 runnable-demo batch because it keeps the remaining parsing-and-safety milestone slice cohesive around structured JSON/TOML/CSV/config handling, parse-error surface guarantees, and the zero-runtime-panic safety gate demo
- `batch_52_structured_parsing_serialization_parse_safety_no_runtime_panics` completed local validation and ended external review with no accepted blockers; pre-validation parity fixes tightened the JSON key order, preserved `None` for the allow-no-value config entry, aligned the hex parse error text, and removed debug-style `Option` rendering in the zero-panic demo, pass 1 on `parse_safety` stalled entirely, pass 1 on `structured_parsing_serialization` and `no_runtime_panics` returned only stale implementation-shape complaints that were rejected, pass 2 on `parse_safety` stalled again, and pass 2 on the other two files again raised only non-blocking implementation-strategy notes rather than demo-visible mismatches
- `batch_53_utility_classes_uuid_and_datetime_fixed_timezones` selected as the fifty-third wave-2 runnable-demo batch because it keeps the remaining utility-and-datetime class slice cohesive around small argparse/IP/UUID/graph helpers plus the two assertion-only timezone/UUID datetime demos
- `batch_53_utility_classes_uuid_and_datetime_fixed_timezones` completed local validation and ended external review with no accepted blockers; pass 1 on `utility_classes` stalled entirely, pass 1 on `uuid_and_datetime` explicitly confirmed the exercised happy-path behavior, pass 1 on `fixed_timezones` was clean for that file but drifted into a non-blocking `utility_classes` API-shape complaint, pass 2 on `utility_classes` stalled again, and pass 2 on `uuid_and_datetime`/`fixed_timezones` ended clean

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

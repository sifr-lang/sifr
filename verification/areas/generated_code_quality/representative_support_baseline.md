# Representative support baseline provenance (12K-B13)

`data/representative_support_baseline.json` attributes the new
`selection-7cddd09acf851f35` entry in `data/generated_quality_debt.json`.
It covers exactly the twelve representative surfaces listed in that receipt.
It does not qualify the retained Item12 all-surface audit or refresh either
existing selection. The global historical baseline SHA remains unchanged;
this selection's source candidate is explicitly recorded separately.

The original delivery candidate is
`ef595b6da060c1bf3f6c2367361b0e774a0654a7`. Its authenticated Clippy summary
contains 712 diagnostics across 22 codes. B13 removes the producer mechanisms
for 166 `redundant_pub_crate` and 18 `wildcard_imports` diagnostics. Neither
category receives a lint allowance or a newly accepted debt owner. The remaining
528 diagnostics retain their existing twenty exact lint owners.

Expected signatures for each candidate are derived **before its Clippy validation**,
using only the original authenticated evidence, never a failed candidate's output:

1. Authenticate the original summary, handoff and emitted-source inventory by
   their recorded SHA-256 digests. Authenticate each original emitted Rust file.
2. Match the eight preserved positive Clippy diagnostic files to their exact
   source spans. Recover the missing cancellation surface from its preserved
   historical diagnostic file. For blocking offload, reconstruct the five retained
   diagnostic signatures from preserved Rust coordinates and Clippy messages:
   the recorded per-lint hashes must match exactly. The two arithmetic expressions
   and single-diagnostic coordinates are reconstructed, not claimed as retained
   original raw output. Relocate these signatures across the earlier removal of
   unused task-local declarations. Reuse the unchanged codegen-output surface's
   historical signature and the cargo-manifest surface's zero-debt record.
3. Require the complete reconstructed twelve-surface, twenty-code aggregate to
   equal the authenticated original aggregate exactly, including all counts and
   signature hashes. This establishes historical provenance independently of
   the new compiler's diagnostic output.
4. Emit the same twelve unchanged Sifr inputs with the B13 compiler. Account for
   the unchanged build producer's native-bridge declaration, which materialization
   includes and `emit` omits. Remove source-listing separator newlines and apply
   the unchanged materializer's final Rust formatting (including module-declaration
   sorting after bridge insertion). Record before/after emitted-source hashes.
5. Move each diagnostic coordinate only through matching source text. A diagnostic
   covering an entire helper declaration follows its first remaining token when
   the unnecessary `pub(crate)` prefix is removed. Record every coordinate and
   this distinction. The codegen-output surface must remain source-identical.
6. Derive the new exact signatures with the unchanged quality-policy functions.
   Counts must equal the authenticated original counts for all twenty retained
   codes. New Clippy execution must subsequently match these predictions exactly.

The B13 continuation re-derives the same twenty-code,528-diagnostic aggregate
after combining B14's approved capture semantics. The owned derivation is
`/private/tmp/sifr-b13-continuation.kr53D8/evidence/derive_support_debt.py`, SHA-256
`d67085e4fce7fdfe861c0be097bdc3b3df20b2133ec51a2b64bf54af50dfffcc`.
It retains the original derivation algorithm and authenticates the same original
evidence; adaptations are the owned compiler path and compact patch rendering.
The source and signature changes are recorded before new Clippy execution, with
production-materialized source concordance required before that execution.

Rust 1.98.0, Clippy's full version, each surface, each original diagnostic origin,
the source hashes, old/new signature aggregates and per-diagnostic relocations
are in the JSON receipt. The derivation script is preserved outside Git at
`/private/tmp/sifr-support.kmdI25/evidence/derive_support_debt.py`, SHA-256
`a4ebcbbfde314a067526742ebeaec2282270714794c4021ab1e2024e84dac2a6`.
Its `--derive` mode emits an auditable patch; it does not run Clippy or change
the existing quality checks. The original policy still rejects unknown owners,
new debt, changed counts/signatures and stale fixed-debt records.

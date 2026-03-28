# Ad Hoc Phase: Idiomatic Rust Demo Corpus — Execution Ledger

Status: planned (created 2026-03-29)
Owning phase: `issues/ad-hoc-idiomatic-rust-demo-corpus.md`

## Baseline

- Baseline date: `2026-03-29`
- Corpus definition:
  - every directory under `demos/` that contains `.sifr`
- Working artifact:
  - `idiomatic.rs` in each in-scope demo directory
- Review method:
  - Claude CLI
  - embedded file-content prompts
  - Rust-first review question:
    - “If an experienced Rust engineer wanted the same observable result as the Sifr code, would this be a strong, idiomatic Rust solution?”
- Review artifact directory:
  - `tmp/idiomatic_review_batches/`
- Tier 2 acceptance rule:
  - Tier 2 files may be scaffold-like or harness-like
  - they are not required to be standalone runnable binaries
  - they should normally still be syntactically valid Rust and clearly communicate the fixture/negative/test contract they stand in for
- in-scope folder rule:
  - every directory under `demos/` containing `.sifr` is independently in scope, including nested `negative_cases/` and test-fixture subdirectories

## Priority Definitions

- `prior-review-flagged`:
  - any folder already called out in a Claude review artifact as `Issues Found`
- `stdlib-heavy`:
  - a folder whose `idiomatic.rs` includes substantial helper/runtime/library-like code instead of a small direct demo translation
- `hand-authored-generated-shape`:
  - a manually edited file that still visibly preserves emitted-Rust structure and ceremony instead of normal Rust simplification

## Batch Log

### wave_1_stabilize_the_review_standard

status: in_progress

- confirmed criterion change on `2026-03-29`:
  - old criterion: Sifr-surface-faithful equivalence
  - new criterion: Rust-first equivalent for the same observable result
- existing supporting reviews already on disk:
  - `batch_01a_review_embedded.md`
  - `batch_01b_review_embedded.md`
  - `batch_01c_review_embedded.md`
  - `batch_01d_review_embedded.md`
  - `batch_01e_review_embedded.md`
  - `batch_02_review_embedded.md`
  - `batch_03_review_embedded.md`
  - `batch_04_review_embedded.md`
  - `batch_05_review_embedded.md`
  - `batch_05b_review_embedded.md`
  - `batch_06_review_embedded.md`
  - `batch_07_review_embedded.md`
  - `batch_08_review_embedded.md`
  - `batch_09_review_embedded.md`
  - `batch_09b_review_embedded.md`
  - `batch_09_rustfirst_review.md`
  - `codegen_preamble_review_embedded.md`

Notes:

- `batch_09_rustfirst_review.md` is the first explicit Rust-first review artifact and should be treated as the template for future batch judgments.
- future batch reviews in this phase should default to the Rust-first rubric rather than the older Sifr-surface-faithful criterion.

### wave_2_runnable_demo_corpus_pass

status: pending

### wave_3_fixture_and_negative_case_normalization

status: pending

### wave_4_corpus_consistency_pass

status: pending

### wave_5_phase_closeout

status: pending

## Closeout Requirements

Before this ledger can be marked complete:

- every reviewed batch must be recorded or linked
- every in-scope folder must be accounted for
- unresolved folders must have an explicit next action
- final note must state the corpus is ready for emitted-vs-idiomatic comparison
- Tier 2 folders must be explicitly marked as:
  - acceptable minimal scaffold,
  - acceptable harness,
  - or needing further clarification

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

#### batch_01_logging_time_timeit

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/logging/idiomatic.rs`
  - `demos/time/idiomatic.rs`
  - `demos/timeit/idiomatic.rs`
- wave role:
  - first fresh wave-1 batch rewritten directly to the Rust-first corpus standard instead of preserving generated/runtime scaffolding
  - chosen because the files were stdlib-heavy and still visibly codegen-shaped
- implementation summary:
  - replaced generated-style helper/runtime layers in `logging` with a small direct logger/file-handler model that preserves the demo outcomes while using standard `std::fs`/`OpenOptions` file handling
  - replaced generated timing wrappers in `timeit` with direct `Instant`-based timing helpers and slice-based result checks
  - replaced generated parsing/struct conversion scaffolding in `time` with direct `chrono`-based formatting/parsing and a compact `StructTime` representation
- local validation completed before external review:
  - `rustfmt demos/logging/idiomatic.rs demos/time/idiomatic.rs demos/timeit/idiomatic.rs`
  - `rustc --edition=2021 demos/logging/idiomatic.rs -o /tmp/sifr-idiomatic-logging && /tmp/sifr-idiomatic-logging`
  - `rustc --edition=2021 demos/timeit/idiomatic.rs -o /tmp/sifr-idiomatic-timeit && /tmp/sifr-idiomatic-timeit`
  - temporary Cargo compile/run for `demos/time/idiomatic.rs` with `chrono = "0.4"` in an isolated temp crate
  - `cargo run -q -p sifr -- run demos/logging/main.sifr`
  - `cargo run -q -p sifr -- run demos/time/main.sifr`
  - `cargo run -q -p sifr -- run demos/timeit/main.sifr`
  - `scripts/run_all_tests.sh`
- validation result:
  - all targeted demo runs passed
  - full local validation lane passed on the repo `pr` profile, including unit tests, validation contract matrix, e2e pass suite, and phase-29 verification hardening suites
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-1-batch-01-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-1-batch-01-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/time/idiomatic.rs`: changed `mktime` to return `Result<f64, ValueError>` and updated the harness to assert on `Ok(0.0)`
    - `demos/logging/idiomatic.rs`: documented best-effort log-write suppression and tightened cleanup verification so non-`NotFound` delete failures report as failure
  - pass 2 follow-up refinement applied:
    - `demos/logging/idiomatic.rs`: added `FileHandler::set_level` and used it in the handler sample to keep the API surface symmetric and the standalone compile path warning-free
  - no remaining accepted blockers after pass 2
- reviewer tooling note:
  - the `claude_resume_to_desktop.sh` wrapper hung before producing a file in this workspace
  - external review artifacts were produced via direct `claude -p --dangerously-skip-permissions ...` runs and then written into the recorded review files

### wave_2_runnable_demo_corpus_pass

status: in_progress

#### batch_01_statistics_json_datetime

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/statistics/idiomatic.rs`
  - `demos/json/idiomatic.rs`
  - `demos/datetime/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three still have large generated-style `idiomatic.rs` companions
  - archived review history already points at real follow-up design debt rather than purely stylistic noise
- priority tags:
  - `prior-review-flagged`: `statistics`, `json`, `datetime`
  - `stdlib-heavy`: `statistics`, `json`, `datetime`
  - `hand-authored-generated-shape`: to be re-evaluated after rewrite
- implementation summary:
  - `statistics`: replaced math/runtime scaffolding with direct slice-based statistical helpers, explicit `StatisticsError`, and linear-time frequency counting
  - `json`: replaced the generated IO/runtime layer with a minimal `serde_json` wrapper for `loads` and `json_dumps`
  - `datetime`: replaced the generated date/time model with direct `chrono`-based helpers plus a small `UtcOffset` formatter
- local validation completed:
  - `rustfmt demos/statistics/idiomatic.rs demos/json/idiomatic.rs demos/datetime/idiomatic.rs`
  - `rustc --edition=2021 demos/statistics/idiomatic.rs -o /tmp/sifr-idiomatic-statistics && /tmp/sifr-idiomatic-statistics`
  - temporary Cargo compile/run for `demos/json/idiomatic.rs` with `serde_json = "1"`
  - temporary Cargo compile/run for `demos/datetime/idiomatic.rs` with `chrono = "0.4"`
  - `cargo run -q -p sifr -- run demos/statistics/main.sifr`
  - `cargo run -q -p sifr -- run demos/json/main.sifr`
  - `cargo run -q -p sifr -- run demos/datetime/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-01-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-01-review-pass-2.md`
- review application summary:
  - pass 1 suggested collapsing the stable-order `mode`/`multimode` second pass into direct counts-map iteration
  - no change was accepted there because the current implementation is already linear-time and the suggested rewrite would weaken encounter-order semantics
  - pass 2 reported no actionable issues

#### batch_02_math_pathlib_glob

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/math/idiomatic.rs`
  - `demos/pathlib/idiomatic.rs`
  - `demos/glob/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three still had generated-style `idiomatic.rs` companions despite comparatively small demo-visible behavior
  - archived review history already identified real helper/runtime and stdlib-surface debt in these areas
- priority tags:
  - `prior-review-flagged`: `math`, `pathlib`, `glob`
  - `stdlib-heavy`: `pathlib`, `glob`
  - `hand-authored-generated-shape`: `math`, `pathlib`, `glob`
- implementation summary:
  - `math`: replaced the generated stdlib/error scaffolding with a compact numeric helper set centered on direct `f64` methods plus focused implementations of `isclose`, compensated summation, IEEE remainder, `nextafter`, and `ulp`
  - `pathlib`: replaced the generated path/runtime layer with a small `PathBuf`-backed wrapper over `std::fs` and a shared wildcard matcher for the demo’s `glob` behavior
  - `glob`: replaced the generated fnmatch/glob scaffolding with a compact wildcard matcher plus sorted directory iteration that preserves hidden-file filtering and missing-directory-as-empty behavior
- local validation completed:
  - `rustfmt demos/math/idiomatic.rs demos/pathlib/idiomatic.rs demos/glob/idiomatic.rs`
  - `rustc --edition=2021 demos/math/idiomatic.rs -o /tmp/sifr-idiomatic-math && /tmp/sifr-idiomatic-math`
  - `rustc --edition=2021 demos/pathlib/idiomatic.rs -o /tmp/sifr-idiomatic-pathlib && /tmp/sifr-idiomatic-pathlib`
  - `rustc --edition=2021 demos/glob/idiomatic.rs -o /tmp/sifr-idiomatic-glob && /tmp/sifr-idiomatic-glob`
  - `cargo run -q -p sifr -- run demos/math/main.sifr`
  - `cargo run -q -p sifr -- run demos/pathlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/glob/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/pathlib/idiomatic.rs demos/glob/idiomatic.rs`
    - `rustc --edition=2021 demos/pathlib/idiomatic.rs -o /tmp/sifr-idiomatic-pathlib && /tmp/sifr-idiomatic-pathlib`
    - `rustc --edition=2021 demos/glob/idiomatic.rs -o /tmp/sifr-idiomatic-glob && /tmp/sifr-idiomatic-glob`
    - `cargo run -q -p sifr -- run demos/math/main.sifr`
    - `cargo run -q -p sifr -- run demos/pathlib/main.sifr`
    - `cargo run -q -p sifr -- run demos/glob/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-02-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-02-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/pathlib/idiomatic.rs`: replaced the hand-rolled `IOError` wrapper with `std::io::Error`, switched the internal path representation to `PathBuf`, and updated constructor/join helpers to use path-native Rust shapes
    - `demos/glob/idiomatic.rs`: changed `glob` to propagate non-`NotFound` I/O errors instead of silently collapsing every failure to an empty result
  - pass 1 observations about broadening API flexibility beyond the demo scope were considered but not expanded further once the `PathBuf`/`std::io::Error` cleanup landed
  - pass 2 reported no actionable issues

#### batch_03_io_csv_shutil

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/io/idiomatic.rs`
  - `demos/csv/idiomatic.rs`
  - `demos/shutil/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three still had generated-style companions despite comparatively small file/data utility surfaces
  - archived review history already pointed at real IO/CSV behavior and helper-shape debt rather than style-only cleanup
- priority tags:
  - `prior-review-flagged`: `io`, `csv`, `shutil`
  - `stdlib-heavy`: `io`, `csv`, `shutil`
  - `hand-authored-generated-shape`: `io`, `csv`, `shutil`
- implementation summary:
  - `io`: replaced the generated handle registry and error scaffolding with direct `std::fs` helpers plus a compact line-oriented read handle for the demo’s `open(...).readline()` flow
  - `csv`: replaced the generated IO hierarchy and object scaffolding with a small Rust-first reader/writer layer and then upgraded parsing/formatting to the `csv` crate for correct RFC 4180 quoting behavior
  - `shutil`: replaced the generated stdlib/error layers with direct filesystem wrappers, a path-search helper that checks executability, and a compact temp-path generator and disk-usage helper
- local validation completed:
  - `rustfmt demos/io/idiomatic.rs demos/csv/idiomatic.rs demos/shutil/idiomatic.rs`
  - `rustc --edition=2021 demos/io/idiomatic.rs -o /tmp/sifr-idiomatic-io && /tmp/sifr-idiomatic-io`
  - temporary Cargo compile/run for `demos/csv/idiomatic.rs` with `csv = "1"` in an isolated temp crate
  - temporary Cargo compile/run for `demos/shutil/idiomatic.rs` with `fs2 = "0.4"` in an isolated temp crate
  - `cargo run -q -p sifr -- run demos/io/main.sifr`
  - `cargo run -q -p sifr -- run demos/csv/main.sifr`
  - `cargo run -q -p sifr -- run demos/shutil/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/io/idiomatic.rs demos/csv/idiomatic.rs demos/shutil/idiomatic.rs`
    - `rustc --edition=2021 demos/io/idiomatic.rs -o /tmp/sifr-idiomatic-io && /tmp/sifr-idiomatic-io`
    - temporary Cargo compile/run for `demos/csv/idiomatic.rs` with `csv = "1"` in an isolated temp crate
    - temporary Cargo compile/run for `demos/shutil/idiomatic.rs` with `fs2 = "0.4"` in an isolated temp crate
    - `cargo run -q -p sifr -- run demos/io/main.sifr`
    - `cargo run -q -p sifr -- run demos/csv/main.sifr`
    - `cargo run -q -p sifr -- run demos/shutil/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-03-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-03-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/io/idiomatic.rs`: removed the misleading no-op `close()` method and made the harness end the file lifetime with direct drop semantics instead
    - `demos/csv/idiomatic.rs`: switched parsing and formatting to the `csv` crate so row/CSV helpers respect standard CSV quoting and escaping behavior
    - `demos/shutil/idiomatic.rs`: tightened `which()` to check executability and strengthened temp-path uniqueness with a process-aware atomic suffix
  - pass 1 observations about the `fs2` dependency and fully streaming CSV object surfaces were not accepted as blockers for this demo companion because the current shape already matches the demo-visible behavior with clear Rust-first code
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - direct shell-session `claude -p` launches stalled without producing review files during this batch
  - review artifacts were captured successfully by running `claude -p --no-session-persistence --dangerously-skip-permissions ...` through a short Python subprocess wrapper and writing the returned stdout into the recorded review files

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

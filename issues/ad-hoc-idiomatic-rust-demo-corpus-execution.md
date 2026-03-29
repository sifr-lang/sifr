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

#### batch_04_uuid_platform_os

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/uuid/idiomatic.rs`
  - `demos/platform/idiomatic.rs`
  - `demos/os/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three had phase-30 review history and still carried generated-style companions that were much larger than the demo-visible runtime-wrapper behavior
  - the group forms a coherent runtime/platform slice instead of mixing unrelated library surfaces
- priority tags:
  - `prior-review-flagged`: `uuid`, `platform`, `os`
  - `stdlib-heavy`: `uuid`, `platform`, `os`
  - `hand-authored-generated-shape`: `uuid`, `platform`, `os`
- implementation summary:
  - `uuid`: replaced the generated hex/string helpers with a compact `UUID` wrapper over the `uuid` crate while preserving passthrough constructor semantics and the `version() == -1` behavior for malformed direct constructor text
  - `platform`: replaced the generated error/runtime scaffolding with small host-info helpers over `std::env` plus `uname`-with-fallback wrappers
  - `os`: replaced the generated error/type scaffolding with direct process/filesystem helpers over `Command`, `std::fs`, and `std::path`
- local validation completed:
  - `rustfmt demos/uuid/idiomatic.rs demos/platform/idiomatic.rs demos/os/idiomatic.rs`
  - temporary Cargo compile/run for `demos/uuid/idiomatic.rs` with `uuid = "1"` and `v3`/`v4`/`v5` features in an isolated temp crate
  - `rustc --edition=2021 demos/platform/idiomatic.rs -o /tmp/sifr-idiomatic-platform && /tmp/sifr-idiomatic-platform`
  - `rustc --edition=2021 demos/os/idiomatic.rs -o /tmp/sifr-idiomatic-os && /tmp/sifr-idiomatic-os`
  - `cargo run -q -p sifr -- run demos/uuid/main.sifr`
  - `cargo run -q -p sifr -- run demos/platform/main.sifr`
  - `cargo run -q -p sifr -- run demos/os/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/uuid/idiomatic.rs demos/os/idiomatic.rs`
    - temporary Cargo compile/run for `demos/uuid/idiomatic.rs` with `uuid = "1"` and `v3`/`v4`/`v5` features in an isolated temp crate
    - `rustc --edition=2021 demos/os/idiomatic.rs -o /tmp/sifr-idiomatic-os && /tmp/sifr-idiomatic-os`
    - `cargo run -q -p sifr -- run demos/uuid/main.sifr`
    - `cargo run -q -p sifr -- run demos/platform/main.sifr`
    - `cargo run -q -p sifr -- run demos/os/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-04-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-04-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/uuid/idiomatic.rs`: changed `UUID::to_str()` to borrow `&str` instead of cloning the stored string
    - `demos/os/idiomatic.rs`: stopped silently swallowing unexpected pre-cleanup filesystem errors before the demo setup path
  - pass 1 observations about including stderr on successful commands, surfacing `uname` fallback failures, and replacing the malformed-constructor `version() == -1` sentinel were not accepted as blockers because those behaviors are intentional for the current demo contract
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 04 continued using the Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` because that remained the stable external-review transport after the shell-session launcher stalled on batch 03

#### batch_05_base64_hashlib_bytes_module

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/base64/idiomatic.rs`
  - `demos/hashlib/idiomatic.rs`
  - `demos/bytes_module/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three still had generated-style companions despite comparatively small encoding-and-digest utility surfaces
  - archived review history already existed for the same slice, which kept the batch cohesive and high-signal for the Rust-first rewrite pass
- priority tags:
  - `prior-review-flagged`: `base64`, `hashlib`, `bytes_module`
  - `stdlib-heavy`: `hashlib`, `bytes_module`
  - `hand-authored-generated-shape`: `base64`, `hashlib`, `bytes_module`
- implementation summary:
  - `base64`: replaced generated helper/error scaffolding with direct `base64` crate usage, compact UTF-8/base16 decoding helpers, and a small parity harness over standard and URL-safe encoders
  - `hashlib`: replaced generated runtime/object scaffolding with a compact digest layer over `sha2` and `md5`, a small mutable `HashObject`, and direct file hashing via `std::fs`
  - `bytes_module`: replaced generated byte/runtime wrappers with direct UTF-8, hex, and slice helpers using standard library byte APIs
- local validation completed:
  - `rustfmt demos/base64/idiomatic.rs demos/hashlib/idiomatic.rs demos/bytes_module/idiomatic.rs`
  - `rustc --edition=2021 demos/bytes_module/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-module && /tmp/sifr-idiomatic-bytes-module`
  - temporary Cargo compile/run for `demos/base64/idiomatic.rs` with `base64 = "0.22"` in an isolated temp crate
  - temporary Cargo compile/run for `demos/hashlib/idiomatic.rs` with `sha2 = "0.10"` and `md5 = "0.7"` in an isolated temp crate
  - `cargo run -q -p sifr -- run demos/base64/main.sifr`
  - `cargo run -q -p sifr -- run demos/hashlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/bytes_module/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-05-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-05-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 05 continued using the Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

#### batch_06_collections_itertools_heapq

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/collections/idiomatic.rs`
  - `demos/itertools/idiomatic.rs`
  - `demos/heapq/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three had phase-30 review history and still carried substantial generated-style companions relative to their small demo-visible behavior
  - the group forms a coherent container-and-iteration slice instead of mixing unrelated stdlib surfaces
- priority tags:
  - `prior-review-flagged`: `collections`, `itertools`, `heapq`
  - `stdlib-heavy`: `collections`, `itertools`, `heapq`
  - `hand-authored-generated-shape`: `collections`, `itertools`, `heapq`
- implementation summary:
  - `collections`: replaced the generated collection helpers with a compact `Counter` that preserves encounter order for `most_common`, stable set operations over deduplicated vectors, and a small `VecDeque`-backed deque wrapper
  - `itertools`: replaced the generated iterator scaffolding with direct slice and iterator helpers built on `chain`, `windows`, `chunks`, `scan`, and `cycle`
  - `heapq`: replaced the hand-written sift machinery with a small `MinHeap<T>` wrapper over `BinaryHeap<Reverse<T>>` plus compact `nsmallest` and `nlargest` helpers
- local validation completed:
  - `rustfmt demos/collections/idiomatic.rs demos/itertools/idiomatic.rs demos/heapq/idiomatic.rs`
  - `rustc --edition=2021 demos/collections/idiomatic.rs -o /tmp/sifr-idiomatic-collections && /tmp/sifr-idiomatic-collections`
  - `rustc --edition=2021 demos/itertools/idiomatic.rs -o /tmp/sifr-idiomatic-itertools && /tmp/sifr-idiomatic-itertools`
  - `rustc --edition=2021 demos/heapq/idiomatic.rs -o /tmp/sifr-idiomatic-heapq && /tmp/sifr-idiomatic-heapq`
  - `cargo run -q -p sifr -- run demos/collections/main.sifr`
  - `cargo run -q -p sifr -- run demos/itertools/main.sifr`
  - `cargo run -q -p sifr -- run demos/heapq/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/collections/idiomatic.rs demos/itertools/idiomatic.rs demos/heapq/idiomatic.rs`
    - `rustc --edition=2021 demos/collections/idiomatic.rs -o /tmp/sifr-idiomatic-collections && /tmp/sifr-idiomatic-collections`
    - `rustc --edition=2021 demos/itertools/idiomatic.rs -o /tmp/sifr-idiomatic-itertools && /tmp/sifr-idiomatic-itertools`
    - `rustc --edition=2021 demos/heapq/idiomatic.rs -o /tmp/sifr-idiomatic-heapq && /tmp/sifr-idiomatic-heapq`
    - `cargo run -q -p sifr -- run demos/collections/main.sifr`
    - `cargo run -q -p sifr -- run demos/itertools/main.sifr`
    - `cargo run -q -p sifr -- run demos/heapq/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-06-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-06-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fix applied:
    - `demos/collections/idiomatic.rs`: made the `Deque::append` zero-capacity and full-capacity boundary handling more explicit by keeping the `maxlen == 0` early return and widening the capacity check to `>=`
  - pass 1 observations about `heapq` temporary-vector allocation in `nsmallest` and `nlargest` were not accepted as blockers because they are a small-scope tradeoff that does not weaken the demo behavior or the Rust-first design
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 06 continued using the Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

#### batch_07_string_textwrap_fnmatch

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/string/idiomatic.rs`
  - `demos/textwrap/idiomatic.rs`
  - `demos/fnmatch/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three had phase-30 review history and still carried either faux-constant ceremony or oversized generated-style helper structure relative to their small text-processing behavior
  - the group forms a coherent string-and-pattern-processing slice instead of mixing unrelated stdlib surfaces
- priority tags:
  - `prior-review-flagged`: `string`, `textwrap`, `fnmatch`
  - `stdlib-heavy`: `textwrap`, `fnmatch`
  - `hand-authored-generated-shape`: `string`, `textwrap`, `fnmatch`
- implementation summary:
  - `string`: replaced helper-function faux-constants with real constants and collapsed `capwords` to a direct `split_whitespace`-based implementation
  - `textwrap`: replaced the generated whitespace and wrapping scaffolding with compact helpers for whitespace normalization, wrapping, filling, dedenting, indenting, and shortening that preserve the demo-visible behavior
  - `fnmatch`: replaced the recursive/index-heavy matcher with a compact wildcard matcher plus direct filter helpers over standard iterators
- local validation completed:
  - `rustfmt demos/string/idiomatic.rs demos/textwrap/idiomatic.rs demos/fnmatch/idiomatic.rs`
  - `rustc --edition=2021 demos/string/idiomatic.rs -o /tmp/sifr-idiomatic-string && /tmp/sifr-idiomatic-string`
  - `rustc --edition=2021 demos/textwrap/idiomatic.rs -o /tmp/sifr-idiomatic-textwrap && /tmp/sifr-idiomatic-textwrap`
  - `rustc --edition=2021 demos/fnmatch/idiomatic.rs -o /tmp/sifr-idiomatic-fnmatch && /tmp/sifr-idiomatic-fnmatch`
  - `cargo run -q -p sifr -- run demos/string/main.sifr`
  - `cargo run -q -p sifr -- run demos/textwrap/main.sifr`
  - `cargo run -q -p sifr -- run demos/fnmatch/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-07-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-07-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 observations about distinguishing `fnmatch` from `fnmatchcase` using platform-dependent case-folding and about preferring iterator-style matching over indexed `Vec<char>` traversal were not accepted as blockers because the approved phase-30 scope already treats deterministic case-sensitive `fnmatch` behavior as an intentional diff from CPython's platform normalization rules, and the current matcher is already compact and readable for this demo
  - no code changes were applied after pass 2
- reviewer tooling note:
  - batch 07 continued using the Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

#### batch_08_bisect_defaultdict_max_heap

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/bisect/idiomatic.rs`
  - `demos/defaultdict/idiomatic.rs`
  - `demos/max_heap/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three expose ordered-search or heap-backed collection behavior and therefore form a coherent container-utility slice instead of mixing unrelated stdlib surfaces
  - each companion still carried more helper or scaffolding structure than the small demo-visible behavior required
- priority tags:
  - `stdlib-heavy`: `defaultdict`, `max_heap`
  - `hand-authored-generated-shape`: `bisect`, `defaultdict`, `max_heap`
- implementation summary:
  - `bisect`: replaced the generated helper surface with direct `partition_point`-based `bisect_left`, `bisect_right`, `insort_left`, and `insort_right` helpers plus a compact parity check
  - `defaultdict`: replaced the oversized companion with a small generic `DefaultDict` wrapper over `HashMap::entry`, a tiny `Deque` wrapper over `VecDeque`, and direct demo-visible collection/counter assertions
  - `max_heap`: replaced the larger heap companion with a small `BinaryHeap`-backed `MaxHeap` wrapper plus direct `heapify_max`, `heappop_max`, `heapreplace_max`, and drain helpers
- local validation completed:
  - `rustfmt demos/bisect/idiomatic.rs demos/defaultdict/idiomatic.rs demos/max_heap/idiomatic.rs`
  - `rustc --edition=2021 demos/bisect/idiomatic.rs -o /tmp/sifr-idiomatic-bisect && /tmp/sifr-idiomatic-bisect`
  - `rustc --edition=2021 demos/defaultdict/idiomatic.rs -o /tmp/sifr-idiomatic-defaultdict && /tmp/sifr-idiomatic-defaultdict`
  - `rustc --edition=2021 demos/max_heap/idiomatic.rs -o /tmp/sifr-idiomatic-max-heap && /tmp/sifr-idiomatic-max-heap`
  - `cargo run -q -p sifr -- run demos/bisect/main.sifr`
  - `cargo run -q -p sifr -- run demos/defaultdict/main.sifr`
  - `cargo run -q -p sifr -- run demos/max_heap/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-08-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-08-review-pass-2.md`
- review application summary:
  - pass 1 reported only non-blocking observations about `bisect`'s compact self-check scaffolding and `defaultdict`'s single-borrow `get_mut` shape; no code changes were applied
  - pass 2 reported only a non-blocking note that `defaultdict` takes keys by value and therefore re-allocates repeated owned keys in the demo; this was not accepted as a blocker because it reflects an honest Rust API tradeoff rather than a behavior or readability failure in the approved demo scope
  - `max_heap` passed both reviews without issue
- reviewer tooling note:
  - batch 08 first attempted the `.cursor/skills/talk-to-claude` desktop-handoff path, but because that handoff stalled before producing the requested review file, the final pass-1/pass-2 artifacts were generated with the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...`

#### batch_09_binary_files_binary_hashing_binary_storage

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/binary_files/idiomatic.rs`
  - `demos/binary_hashing/idiomatic.rs`
  - `demos/binary_storage/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three expose binary-data operations and therefore form a coherent slice around file IO, hashing/base64 round-trips, and byte-hex storage behavior
  - each companion still retained large generated-style helper surfaces relative to the small demo-visible contracts
- priority tags:
  - `stdlib-heavy`: `binary_files`, `binary_hashing`, `binary_storage`
  - `hand-authored-generated-shape`: `binary_files`, `binary_hashing`, `binary_storage`
- implementation summary:
  - `binary_files`: replaced the file-handle registry and error-scaffolding companion with a direct file round-trip, byte-int formatting helper, and explicit cleanup check
  - `binary_hashing`: replaced the generated hashlib/base64 surface with a compact SHA-256 digest check plus base64 round-trip over the demo payload
  - `binary_storage`: replaced the larger bytes/runtime wrapper with direct byte-sum, range-checked membership/count helpers, hex encode/decode helpers, and a compact binary file round-trip
- local validation completed:
  - initial validation:
    - `rustfmt demos/binary_files/idiomatic.rs demos/binary_hashing/idiomatic.rs demos/binary_storage/idiomatic.rs`
    - `rustc --edition=2021 demos/binary_files/idiomatic.rs -o /tmp/sifr-idiomatic-binary-files && /tmp/sifr-idiomatic-binary-files`
    - `rustc --edition=2021 demos/binary_storage/idiomatic.rs -o /tmp/sifr-idiomatic-binary-storage && /tmp/sifr-idiomatic-binary-storage`
    - temporary Cargo validation for `demos/binary_hashing/idiomatic.rs` with `base64 = "0.22"` and `sha2 = "0.10"`
    - `cargo run -q -p sifr -- run demos/binary_files/main.sifr`
    - `cargo run -q -p sifr -- run demos/binary_hashing/main.sifr`
    - `cargo run -q -p sifr -- run demos/binary_storage/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/binary_files/idiomatic.rs demos/binary_hashing/idiomatic.rs demos/binary_storage/idiomatic.rs`
    - `rustc --edition=2021 demos/binary_files/idiomatic.rs -o /tmp/sifr-idiomatic-binary-files && /tmp/sifr-idiomatic-binary-files`
    - `rustc --edition=2021 demos/binary_storage/idiomatic.rs -o /tmp/sifr-idiomatic-binary-storage && /tmp/sifr-idiomatic-binary-storage`
    - temporary Cargo validation for `demos/binary_hashing/idiomatic.rs` with `base64 = "0.22"` and `sha2 = "0.10"`
    - `cargo run -q -p sifr -- run demos/binary_files/main.sifr`
    - `cargo run -q -p sifr -- run demos/binary_hashing/main.sifr`
    - `cargo run -q -p sifr -- run demos/binary_storage/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-09-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-09-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fix applied:
    - `demos/binary_hashing/idiomatic.rs`: removed the inert constant-equality assertion that proved nothing beyond the constant's own definition
  - pass 1 observation that the digest checks validate length rather than a fixed SHA-256 value was not accepted as a blocker because the demo contract is API-shape and round-trip behavior rather than cryptographic test-vector verification
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 09 used the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` for both review passes

#### batch_10_bytes_basics_bytes_constructors_bytes_roundtrip

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/bytes_basics/idiomatic.rs`
  - `demos/bytes_constructors/idiomatic.rs`
  - `demos/bytes_roundtrip/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three expose core bytes construction and conversion behavior, so they form a coherent bytes-and-UTF-8 slice instead of mixing unrelated stdlib surfaces
  - each companion still retained generated-style structure despite small demo-visible contracts
- priority tags:
  - `stdlib-heavy`: `bytes_constructors`, `bytes_roundtrip`
  - `hand-authored-generated-shape`: `bytes_basics`, `bytes_constructors`, `bytes_roundtrip`
- implementation summary:
  - `bytes_basics`: replaced the vector-construction and slicing scaffolding with direct byte literals, slice access, and a compact sum helper
  - `bytes_constructors`: replaced the larger constructor surface with localized `ValueError`/`ParseError` types and direct helpers for zero-filled bytes, integer conversion, hex decoding, and UTF-8 encode/decode
  - `bytes_roundtrip`: replaced the generated helper surface with compact UTF-8 and hex helpers built on standard library primitives and direct `starts_with`/`ends_with` checks
- local validation completed:
  - `rustfmt demos/bytes_basics/idiomatic.rs demos/bytes_constructors/idiomatic.rs demos/bytes_roundtrip/idiomatic.rs`
  - `rustc --edition=2021 demos/bytes_basics/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-basics && /tmp/sifr-idiomatic-bytes-basics`
  - `rustc --edition=2021 demos/bytes_constructors/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-constructors && /tmp/sifr-idiomatic-bytes-constructors`
  - `rustc --edition=2021 demos/bytes_roundtrip/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-roundtrip && /tmp/sifr-idiomatic-bytes-roundtrip`
  - `cargo run -q -p sifr -- run demos/bytes_basics/main.sifr`
  - `cargo run -q -p sifr -- run demos/bytes_constructors/main.sifr`
  - `cargo run -q -p sifr -- run demos/bytes_roundtrip/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-10-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-10-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 10 used the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` for both review passes

#### batch_11_subprocess_tempfile_zipfile_io

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/subprocess/idiomatic.rs`
  - `demos/tempfile/idiomatic.rs`
  - `demos/zipfile_io/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three expose runtime-and-file-lifecycle behavior, so they form a coherent slice around subprocess execution, temporary paths, and zip archive IO
  - each companion still retained generated helper or wrapper structure much larger than the demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `subprocess`, `tempfile`, `zipfile_io`
  - `hand-authored-generated-shape`: `subprocess`, `tempfile`, `zipfile_io`
- implementation summary:
  - `subprocess`: replaced the larger wrapper surface with direct `sh -c` execution helpers for `run`, `check_call`, and `check_output` plus the demo's sentinel constant check
  - `tempfile`: replaced the randomized wrapper surface with compact unique-path helpers backed by `std::env::temp_dir`, `create_new`, and direct path cleanup
  - `zipfile_io`: replaced the archive and temp-file scaffolding with a small `NamedTemporaryFile`, `ZipFile`, and `ZipReadHandle` set built directly on filesystem operations and the `zip` crate
- local validation completed:
  - initial validation:
    - `rustfmt demos/subprocess/idiomatic.rs demos/tempfile/idiomatic.rs demos/zipfile_io/idiomatic.rs`
    - `rustc --edition=2021 demos/subprocess/idiomatic.rs -o /tmp/sifr-idiomatic-subprocess && /tmp/sifr-idiomatic-subprocess`
    - `rustc --edition=2021 demos/tempfile/idiomatic.rs -o /tmp/sifr-idiomatic-tempfile && /tmp/sifr-idiomatic-tempfile`
    - temporary Cargo validation for `demos/zipfile_io/idiomatic.rs` with `zip = "0.6"`
    - `cargo run -q -p sifr -- run demos/subprocess/main.sifr`
    - `cargo run -q -p sifr -- run demos/tempfile/main.sifr`
    - `cargo run -q -p sifr -- run demos/zipfile_io/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/subprocess/idiomatic.rs demos/tempfile/idiomatic.rs demos/zipfile_io/idiomatic.rs`
    - `rustc --edition=2021 demos/subprocess/idiomatic.rs -o /tmp/sifr-idiomatic-subprocess && /tmp/sifr-idiomatic-subprocess`
    - `cargo run -q -p sifr -- run demos/subprocess/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-11-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-11-review-pass-2.md`
- review application summary:
  - pass 1 observations about dead subprocess sentinel constants and explicit cleanup-only semantics in `zipfile_io` were not accepted as blockers because the constants are exercised in the demo and the explicit cleanup pattern is the behavior the demo intentionally demonstrates
  - pass 2 valid follow-up fix applied:
    - `demos/subprocess/idiomatic.rs`: changed `check_call` to return `0` on success instead of the child exit code value so the helper matches the demo's intended semantics more closely
  - pass 2 observations about `ZipFile::open` rejecting `"rb"` and about explicit cleanup requirements in `NamedTemporaryFile` were not accepted as blockers because both behaviors are deliberate parts of the approved demo scope
- reviewer tooling note:
  - batch 11 used the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` for both review passes

#### batch_12_readonly_bytes_tempfiles_and_zip_filesystem_and_archives

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/readonly_bytes/idiomatic.rs`
  - `demos/tempfiles_and_zip/idiomatic.rs`
  - `demos/filesystem_and_archives/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three expose a cohesive bytes-and-archive-lifecycle slice instead of mixing unrelated stdlib surfaces
  - each companion still retained generated helper or wrapper structure much larger than the demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `tempfiles_and_zip`, `filesystem_and_archives`
  - `hand-authored-generated-shape`: `readonly_bytes`, `tempfiles_and_zip`, `filesystem_and_archives`
- implementation summary:
  - `readonly_bytes`: replaced the generated byte/runtime layer with direct slice helpers for indexing, summation, containment, integer formatting, and binary write/read cleanup
  - `tempfiles_and_zip`: replaced the temp-path and archive scaffolding with compact unique-path helpers plus a small `ZipFile` wrapper over the `zip` crate that preserves the demo lifecycle
  - `filesystem_and_archives`: replaced the broad filesystem/archive wrapper surface with direct `std::fs`, `flate2`, and `zip` helpers covering text IO, glob-like listing, gzip roundtrip, temp paths, and zip archive read/write behavior
- local validation completed:
  - initial validation:
    - `rustfmt demos/readonly_bytes/idiomatic.rs demos/tempfiles_and_zip/idiomatic.rs demos/filesystem_and_archives/idiomatic.rs`
    - `rustc --edition=2021 demos/readonly_bytes/idiomatic.rs -o /tmp/sifr-idiomatic-readonly-bytes && /tmp/sifr-idiomatic-readonly-bytes`
    - temporary Cargo validation for `demos/tempfiles_and_zip/idiomatic.rs` with `zip = "0.6"`
    - temporary Cargo validation for `demos/filesystem_and_archives/idiomatic.rs` with `flate2 = "1"` and `zip = "0.6"`
    - `cargo run -q -p sifr -- run demos/readonly_bytes/main.sifr`
    - `cargo run -q -p sifr -- run demos/tempfiles_and_zip/main.sifr`
    - `cargo run -q -p sifr -- run demos/filesystem_and_archives/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-alignment revalidation:
    - `rustfmt demos/readonly_bytes/idiomatic.rs demos/tempfiles_and_zip/idiomatic.rs demos/filesystem_and_archives/idiomatic.rs`
    - temporary Cargo validation for `demos/filesystem_and_archives/idiomatic.rs` with `flate2 = "1"` and `zip = "0.6"`
    - `cargo run -q -p sifr -- run demos/filesystem_and_archives/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-12-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-12-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 12 used the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` for both review passes after the desktop review handoff had already proven unreliable in this phase

#### batch_13_bytes_errors_bytes_file_io_bytes_iteration

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/bytes_errors/idiomatic.rs`
  - `demos/bytes_file_io/idiomatic.rs`
  - `demos/bytes_iteration/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three complete the remaining compact bytes-focused runnable slice instead of mixing unrelated stdlib surfaces
  - each companion still retained generated structure or unnecessary ceremony despite small demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `bytes_errors`, `bytes_file_io`
  - `hand-authored-generated-shape`: `bytes_errors`, `bytes_file_io`, `bytes_iteration`
- implementation summary:
  - `bytes_errors`: replaced the generated error-path scaffolding with compact `ParseError` and `ValueError` types plus focused helpers for size validation, integer conversion, hex decoding, UTF-8-only encoding, and UTF-8 decoding
  - `bytes_file_io`: replaced the generated handle registry and IO wrapper surface with direct binary file write/read helpers plus integer-list formatting and cleanup checks
  - `bytes_iteration`: replaced the vector-construction ceremony with a tiny byte-slice-first companion that keeps concatenation parity, optional indexing, and byte-sum iteration
- local validation completed:
  - `rustfmt demos/bytes_errors/idiomatic.rs demos/bytes_file_io/idiomatic.rs demos/bytes_iteration/idiomatic.rs`
  - `rustc --edition=2021 demos/bytes_errors/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-errors && /tmp/sifr-idiomatic-bytes-errors`
  - `rustc --edition=2021 demos/bytes_file_io/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-file-io && /tmp/sifr-idiomatic-bytes-file-io`
  - `rustc --edition=2021 demos/bytes_iteration/idiomatic.rs -o /tmp/sifr-idiomatic-bytes-iteration && /tmp/sifr-idiomatic-bytes-iteration`
  - `cargo run -q -p sifr -- run demos/bytes_errors/main.sifr`
  - `cargo run -q -p sifr -- run demos/bytes_file_io/main.sifr`
  - `cargo run -q -p sifr -- run demos/bytes_iteration/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-13-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-13-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported one claimed `bytes_file_io` path-name mismatch, but no change was accepted because `demos/bytes_file_io/main.sifr` itself uses the `ad_hoc_bytes_wave3` reference and the rewritten `/tmp/sifr_ad_hoc_bytes_wave3_demo.bin` path already matches that source-visible naming
- reviewer tooling note:
  - batch 13 used the stable Python subprocess capture path for `claude -p --no-session-persistence --dangerously-skip-permissions ...` for both review passes after the unbounded direct invocation again proved unreliable

#### batch_14_file_streams_in_memory_streams_text_and_bytes

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/file_streams/idiomatic.rs`
  - `demos/in_memory_streams/idiomatic.rs`
  - `demos/text_and_bytes/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a coherent stream-and-text/bytes slice instead of mixing unrelated stdlib surfaces
  - each companion still retained generated runtime or helper ceremony much larger than the demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `file_streams`, `in_memory_streams`, `text_and_bytes`
  - `hand-authored-generated-shape`: `file_streams`, `in_memory_streams`, `text_and_bytes`
- implementation summary:
  - `file_streams`: replaced the generated handle registry with direct text-line and binary file roundtrip helpers over `std::fs` and `BufReader`
  - `in_memory_streams`: replaced the generated in-memory IO hierarchy with compact `StringIO`, `BytesIO`, and `BinaryFileHandle` wrappers built directly on `Cursor` and `std::fs::File`
  - `text_and_bytes`: replaced the generated bytes helper surface with minimal `ParseError`, UTF-8 encode/decode, and hex parsing helpers
- local validation completed:
  - `rustfmt demos/file_streams/idiomatic.rs demos/in_memory_streams/idiomatic.rs demos/text_and_bytes/idiomatic.rs`
  - `rustc --edition=2021 demos/file_streams/idiomatic.rs -o /tmp/sifr-idiomatic-file-streams && /tmp/sifr-idiomatic-file-streams`
  - `rustc --edition=2021 demos/in_memory_streams/idiomatic.rs -o /tmp/sifr-idiomatic-in-memory-streams && /tmp/sifr-idiomatic-in-memory-streams`
  - `rustc --edition=2021 demos/text_and_bytes/idiomatic.rs -o /tmp/sifr-idiomatic-text-and-bytes && /tmp/sifr-idiomatic-text-and-bytes`
  - `cargo run -q -p sifr -- run demos/file_streams/main.sifr`
  - `cargo run -q -p sifr -- run demos/in_memory_streams/main.sifr`
  - `cargo run -q -p sifr -- run demos/text_and_bytes/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-14-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-14-review-pass-2.md`
- review application summary:
  - pass 1 reported a critical swapped-file claim between `file_streams` and `text_and_bytes`, but no change was accepted because the actual rewritten files already match their same-named Sifr demos and the claim contradicted the checked-in file contents
  - pass 2 repeated the same invalid cross-wiring claim between `file_streams` and `in_memory_streams`; no change was accepted because the local files, standalone validation, and targeted Sifr runs all confirmed correct demo parity
- reviewer tooling note:
  - batch 14 used the stable Python subprocess capture path for both review passes; the unbounded direct invocation again stalled before reliably materializing the review file

#### batch_15_json_values_random_hashing_random_state

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/json_values/idiomatic.rs`
  - `demos/random_hashing/idiomatic.rs`
  - `demos/random_state/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a coherent structured-data and deterministic-RNG slice instead of mixing unrelated stdlib surfaces
  - each companion still retained generated-style wrapper or helper ceremony larger than the actual demo-visible behavior
  - an earlier candidate slice around `html_and_textwrap`, `regex_and_filesystem`, and `text_and_patterns` was deferred after preflight runs showed that two of those current Sifr demos were not green under the active runnable-demo loop
- priority tags:
  - `stdlib-heavy`: `json_values`, `random_state`
  - `external-crate-companion`: `json_values`, `random_hashing`
  - `hand-authored-generated-shape`: `json_values`, `random_hashing`, `random_state`
- implementation summary:
  - `json_values`: replaced the larger wrapper surface with a compact `JsonValue` enum that preserves insertion order for object encoding, uses `Option` for typed accessors, and keeps `dumps`/`loads` behavior direct and explicit
  - `random_hashing`: replaced the broader helper scaffolding with a small base64/SHA-256 companion plus a compact global RNG helper that avoids poisoned-lock panics
  - `random_state`: replaced the generated state-management ceremony with compact `Random`/`RandomState` wrappers over a shared deterministic LCG and explicit state replay checks
- local validation completed:
  - initial validation:
    - `rustfmt demos/json_values/idiomatic.rs demos/random_hashing/idiomatic.rs demos/random_state/idiomatic.rs`
    - `rustc --edition=2021 demos/random_state/idiomatic.rs -o /tmp/sifr-idiomatic-random-state && /tmp/sifr-idiomatic-random-state`
    - temporary Cargo validation for `demos/json_values/idiomatic.rs` with `serde_json = "1"`
    - temporary Cargo validation for `demos/random_hashing/idiomatic.rs` with `base64 = "0.22"` and `sha2 = "0.10"`
    - `cargo run -q -p sifr -- run demos/json_values/main.sifr`
    - `cargo run -q -p sifr -- run demos/random_hashing/main.sifr`
    - `cargo run -q -p sifr -- run demos/random_state/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/json_values/idiomatic.rs demos/random_hashing/idiomatic.rs demos/random_state/idiomatic.rs`
    - `rustc --edition=2021 demos/random_state/idiomatic.rs -o /tmp/sifr-idiomatic-random-state && /tmp/sifr-idiomatic-random-state`
    - temporary Cargo validation for `demos/json_values/idiomatic.rs` with `serde_json = "1"`
    - temporary Cargo validation for `demos/random_hashing/idiomatic.rs` with `base64 = "0.22"` and `sha2 = "0.10"`
    - `cargo run -q -p sifr -- run demos/json_values/main.sifr`
    - `cargo run -q -p sifr -- run demos/random_hashing/main.sifr`
    - `cargo run -q -p sifr -- run demos/random_state/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-15-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-15-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/json_values/idiomatic.rs`: changed `as_str`/`as_int` to explicit `Option` accessors and removed silent serialization fallbacks by switching to infallible JSON string rendering
    - `demos/random_hashing/idiomatic.rs`: replaced the global RNG mutex `.expect(...)` with poison-tolerant locking
    - `demos/random_state/idiomatic.rs`: replaced poisoned-lock `.expect(...)` calls with poison-tolerant locking, switched `main` to `Result`-based flow, and replaced the tautological final assertion with explicit success output
  - pass 1 also claimed that `random_hashing` was blocking because its external crates are not declared in the workspace manifest, but no change was accepted because this phase explicitly does not require `idiomatic.rs` files to compile inside the workspace Cargo graph and the batch already validated external-crate companions in isolated temp Cargo manifests
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - batch 15 used the stable Python subprocess capture path for both review passes because it continued to be the most reliable way to materialize external review files in this workspace

#### batch_16_logging_and_timers_config_json_csv_collections_and_argparse

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/logging_and_timers/idiomatic.rs`
  - `demos/config_json_csv/idiomatic.rs`
  - `demos/collections_and_argparse/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a coherent remaining object-wrapper and config-surface slice instead of mixing unrelated runtime areas
  - each companion still retained large generated-style helper or wrapper structure relative to the small demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `logging_and_timers`, `config_json_csv`, `collections_and_argparse`
  - `external-crate-companion`: `config_json_csv`
  - `hand-authored-generated-shape`: `logging_and_timers`, `config_json_csv`, `collections_and_argparse`
- implementation summary:
  - `logging_and_timers`: replaced the generated runtime surface with direct file IO helpers, compact UTC time conversion helpers, a minimal file-backed logger, and a small `Timer` wrapper over `Instant`
  - `config_json_csv`: replaced the large mixed helper surface with a compact ordered `JsonValue`, tiny encoder/decoder over `serde_json`, a focused interpolating `ConfigParser`, and a small delimiter-backed row reader
  - `collections_and_argparse`: replaced the larger collection/parser scaffolding with a small `Counter`, integer `DefaultDict`, and a compact subcommand-aware argument parser backed by a `Namespace` value map
- local validation completed:
  - initial validation:
    - `rustfmt demos/logging_and_timers/idiomatic.rs demos/config_json_csv/idiomatic.rs demos/collections_and_argparse/idiomatic.rs`
    - `rustc --edition=2021 demos/logging_and_timers/idiomatic.rs -o /tmp/sifr-idiomatic-logging-and-timers && /tmp/sifr-idiomatic-logging-and-timers`
    - `rustc --edition=2021 demos/collections_and_argparse/idiomatic.rs -o /tmp/sifr-idiomatic-collections-and-argparse && /tmp/sifr-idiomatic-collections-and-argparse`
    - temporary Cargo validation for `demos/config_json_csv/idiomatic.rs` with `serde_json = "1"`
    - `cargo run -q -p sifr -- run demos/logging_and_timers/main.sifr`
    - `cargo run -q -p sifr -- run demos/config_json_csv/main.sifr`
    - `cargo run -q -p sifr -- run demos/collections_and_argparse/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/logging_and_timers/idiomatic.rs demos/config_json_csv/idiomatic.rs demos/collections_and_argparse/idiomatic.rs`
    - `rustc --edition=2021 demos/logging_and_timers/idiomatic.rs -o /tmp/sifr-idiomatic-logging-and-timers && /tmp/sifr-idiomatic-logging-and-timers`
    - `rustc --edition=2021 demos/collections_and_argparse/idiomatic.rs -o /tmp/sifr-idiomatic-collections-and-argparse && /tmp/sifr-idiomatic-collections-and-argparse`
    - temporary Cargo validation for `demos/config_json_csv/idiomatic.rs` with `serde_json = "1"`
    - `cargo run -q -p sifr -- run demos/logging_and_timers/main.sifr`
    - `cargo run -q -p sifr -- run demos/config_json_csv/main.sifr`
    - `cargo run -q -p sifr -- run demos/collections_and_argparse/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-16-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-16-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 valid follow-up fix applied:
    - `demos/collections_and_argparse/idiomatic.rs`: guarded the `store`-action path in `parse_into` so a missing value no longer indexes past the argument slice and panics
  - pass 2's other notes were non-blocking observations about intentionally omitted demo-irrelevant surface details such as JSON indentation and default injection
- reviewer tooling note:
  - batch 16 used the stable Python subprocess capture path for both review passes; the first pass-2 attempt timed out without producing a file, so the same bounded subprocess command was retried and completed successfully on the second attempt

#### batch_17_classes_protocols_pattern_matching

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/classes/idiomatic.rs`
  - `demos/protocols/idiomatic.rs`
  - `demos/pattern_matching/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive milestone-language slice instead of mixing runtime wrappers with unrelated utility surfaces
  - each companion still retained emitted-style ceremony or unused scaffolding despite relatively small, stable demo-visible behavior
- priority tags:
  - `milestone-language-surface`: `classes`, `protocols`, `pattern_matching`
  - `hand-authored-generated-shape`: `classes`, `protocols`, `pattern_matching`
- implementation summary:
  - `classes`: replaced the remaining constructor and hash ceremony with small direct structs, methods, enum-based shape narrowing, and a focused stable-hash helper
  - `protocols`: replaced the emitted-style trait and enum scaffolding with compact trait-based display/describe implementations, direct `Add` support for `Vec2`, and simple newtype wrappers for `Port` and `Email`
  - `pattern_matching`: removed the unused error-wrapper preamble and replaced it with direct enum, tuple, option, and guard-pattern examples that preserve the demo output with compact Rust `match` expressions
- local validation completed:
  - initial validation:
    - `rustfmt demos/classes/idiomatic.rs demos/protocols/idiomatic.rs demos/pattern_matching/idiomatic.rs`
    - `rustc --edition=2021 demos/classes/idiomatic.rs -o /tmp/sifr-idiomatic-classes && /tmp/sifr-idiomatic-classes`
    - `rustc --edition=2021 demos/protocols/idiomatic.rs -o /tmp/sifr-idiomatic-protocols && /tmp/sifr-idiomatic-protocols`
    - `rustc --edition=2021 demos/pattern_matching/idiomatic.rs -o /tmp/sifr-idiomatic-pattern-matching && /tmp/sifr-idiomatic-pattern-matching`
    - `cargo run -q -p sifr -- run demos/classes/main.sifr`
    - `cargo run -q -p sifr -- run demos/protocols/main.sifr`
    - `cargo run -q -p sifr -- run demos/pattern_matching/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/classes/idiomatic.rs demos/protocols/idiomatic.rs demos/pattern_matching/idiomatic.rs`
    - `rustc --edition=2021 demos/protocols/idiomatic.rs -o /tmp/sifr-idiomatic-protocols && /tmp/sifr-idiomatic-protocols`
    - `cargo run -q -p sifr -- run demos/classes/main.sifr`
    - `cargo run -q -p sifr -- run demos/protocols/main.sifr`
    - `cargo run -q -p sifr -- run demos/pattern_matching/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-17-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-17-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fix applied:
    - `demos/protocols/idiomatic.rs`: changed `Port::value` to take `&self` instead of consuming the value
  - pass 1 notes about replacing demo-visible labels with Rust-specific terminology and about switching the circle area constant to `std::f64::consts::PI` were not accepted because both would change observable output away from the checked-in Sifr demos
  - pass 2 reported no accepted blockers; its repeated `PI` and terminology notes were rejected for the same output-parity reason, and its claimed dead-`Printable` note was not accepted because the cited trait does not exist in `demos/classes/idiomatic.rs`
- reviewer tooling note:
  - batch 17 used the stable Python subprocess capture path for both review passes and both completed successfully without requiring a retry

#### batch_18_iterators_and_randomness_error_handling_decorators

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/iterators_and_randomness/idiomatic.rs`
  - `demos/error_handling/idiomatic.rs`
  - `demos/decorators/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive callable-and-control-surface slice instead of mixing the remaining iterator/random helpers with unrelated stdlib areas
  - each companion still retained generated-style scaffolding or ceremony relative to the small, stable demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `iterators_and_randomness`, `error_handling`
  - `milestone-language-surface`: `decorators`
  - `hand-authored-generated-shape`: `iterators_and_randomness`, `error_handling`, `decorators`
- implementation summary:
  - `iterators_and_randomness`: replaced the oversized helper surface with direct iterator adapters, compact permutation/combination/product helpers, a small callable-object example, and focused `rand`-backed randomness/secrets helpers
  - `error_handling`: replaced the generated wrappers with direct custom error structs, a compact `From<ParseIntError>` conversion, and straightforward `match`-based `Result` handling
  - `decorators`: replaced the ceremony-heavy callable scaffolding with plain Rust functions plus preserved Sifr-side decorator comments that show the desugared callable result
- local validation completed:
  - initial validation:
    - `rustfmt demos/iterators_and_randomness/idiomatic.rs demos/error_handling/idiomatic.rs demos/decorators/idiomatic.rs`
    - temporary Cargo validation for `demos/iterators_and_randomness/idiomatic.rs` with `rand = "0.8"`
    - `rustc --edition=2021 demos/error_handling/idiomatic.rs -o /tmp/sifr-idiomatic-error-handling && /tmp/sifr-idiomatic-error-handling`
    - `rustc --edition=2021 demos/decorators/idiomatic.rs -o /tmp/sifr-idiomatic-decorators && /tmp/sifr-idiomatic-decorators`
    - `cargo run -q -p sifr -- run demos/iterators_and_randomness/main.sifr`
    - `cargo run -q -p sifr -- run demos/error_handling/main.sifr`
    - `cargo run -q -p sifr -- run demos/decorators/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/iterators_and_randomness/idiomatic.rs demos/error_handling/idiomatic.rs demos/decorators/idiomatic.rs`
    - temporary Cargo validation for `demos/iterators_and_randomness/idiomatic.rs` with `rand = "0.8"`
    - `rustc --edition=2021 demos/error_handling/idiomatic.rs -o /tmp/sifr-idiomatic-error-handling && /tmp/sifr-idiomatic-error-handling`
    - `rustc --edition=2021 demos/decorators/idiomatic.rs -o /tmp/sifr-idiomatic-decorators && /tmp/sifr-idiomatic-decorators`
    - `cargo run -q -p sifr -- run demos/iterators_and_randomness/main.sifr`
    - `cargo run -q -p sifr -- run demos/error_handling/main.sifr`
    - `cargo run -q -p sifr -- run demos/decorators/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-18-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-18-review-pass-2.md`
- review application summary:
  - pass 1 valid follow-up fixes applied:
    - `demos/iterators_and_randomness/idiomatic.rs`: generalized `product_repeat` so non-`2` repeat counts no longer silently collapse to an empty result
    - `demos/iterators_and_randomness/idiomatic.rs`: replaced the `compare_digest` early-exit equality check with a length-mixed bytewise comparison so the helper better matches the intended constant-time semantics of the demo surface
  - pass 1's note about `randbits` distribution was not accepted because the current bit-by-bit implementation already yields a uniform value in `0..2^bits` without changing the demo-visible behavior
  - pass 2 reported no accepted blockers; its RNG-state note was rejected because repeated `thread_rng()` handles still draw from the same thread-local RNG state, and its `compare_digest` note was treated as non-blocking because the helper already avoids early-exit comparison
- reviewer tooling note:
  - batch 18 used direct `claude -p --no-session-persistence --dangerously-skip-permissions ...` runs redirected into the recorded review files, and both passes completed successfully in that form

#### batch_19_env_regex_regex_and_filesystem

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/env/idiomatic.rs`
  - `demos/regex/idiomatic.rs`
  - `demos/regex_and_filesystem/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining environment-and-text/filesystem slice instead of mixing the remaining regex/path helpers with unrelated stdlib areas
  - each companion still retained generated-style helper scaffolding despite comparatively small, stable demo-visible behavior
- priority tags:
  - `stdlib-heavy`: `env`, `regex`, `regex_and_filesystem`
  - `hand-authored-generated-shape`: `env`, `regex`, `regex_and_filesystem`
- implementation summary:
  - `env`: replaced the generated error/helper preamble with direct env-key validation, compact env get/set/unset helpers, and slice-based boolean assertion checks
  - `regex`: replaced the generated runtime scaffolding with small `regex`-crate wrappers for match/find/replace/findall/split/flagged-search behavior plus a compact boolean-result harness
  - `regex_and_filesystem`: replaced the generated pathlib/iterator scaffolding with direct `Regex` wrappers, a small wildcard matcher, sorted directory iteration helpers, and compact `glob`/`rglob` parity over `PathBuf`
- local validation completed:
  - `rustfmt demos/env/idiomatic.rs demos/regex/idiomatic.rs demos/regex_and_filesystem/idiomatic.rs`
  - `rustc --edition=2021 demos/env/idiomatic.rs -o /tmp/sifr-idiomatic-env && /tmp/sifr-idiomatic-env`
  - temporary Cargo validation for `demos/regex/idiomatic.rs` with `regex = "1"`
  - temporary Cargo validation for `demos/regex_and_filesystem/idiomatic.rs` with `regex = "1"`
  - `cargo run -q -p sifr -- run demos/env/main.sifr`
  - `cargo run -q -p sifr -- run demos/regex/main.sifr`
  - `cargo run -q -p sifr -- run demos/regex_and_filesystem/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-19-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-19-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported no actionable issues
- reviewer tooling note:
  - the full-batch `claude -p` review prompt repeatedly stalled in this workspace, so batch 19 used stable per-file external review prompts and then consolidated their results into the recorded batch review artifacts

#### batch_20_iter_and_next_cloned_iterators_lazy_iterators

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/iter_and_next/idiomatic.rs`
  - `demos/cloned_iterators/idiomatic.rs`
  - `demos/lazy_iterators/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining iterator-consumption slice instead of mixing small iterator demos with unrelated stdlib areas
  - each companion still retained generated-style iterator boxing, explicit collection scaffolding, or faux-lazy generator structure despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `iter_and_next`, `cloned_iterators`, `lazy_iterators`
  - `hand-authored-generated-shape`: `iter_and_next`, `cloned_iterators`, `lazy_iterators`
- implementation summary:
  - `iter_and_next`: replaced boxed iterator plumbing with direct borrowed iteration, `next()` consumption, and a compact `enumerate`/`sum` flow
  - `cloned_iterators`: replaced iterator boxing and list-comprehension scaffolding with straightforward borrowed iterator chains plus an owned temporary-array transform
  - `lazy_iterators`: replaced eager faux-generator scaffolding with direct `impl Iterator` helpers built from `from_fn`, range maps, and simple collection formatting
- local validation completed:
  - `rustfmt demos/iter_and_next/idiomatic.rs demos/cloned_iterators/idiomatic.rs demos/lazy_iterators/idiomatic.rs`
  - `rustc --edition=2021 demos/iter_and_next/idiomatic.rs -o /tmp/sifr-idiomatic-iter-and-next && /tmp/sifr-idiomatic-iter-and-next`
  - `rustc --edition=2021 demos/cloned_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-cloned-iterators && /tmp/sifr-idiomatic-cloned-iterators`
  - `rustc --edition=2021 demos/lazy_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-lazy-iterators && /tmp/sifr-idiomatic-lazy-iterators`
  - `cargo run -q -p sifr -- run demos/iter_and_next/main.sifr`
  - `cargo run -q -p sifr -- run demos/cloned_iterators/main.sifr`
  - `cargo run -q -p sifr -- run demos/lazy_iterators/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-20-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-20-review-pass-2.md`
- review application summary:
  - pass 1 reported no actionable issues
  - pass 2 reported no accepted blockers
  - pass 2's claimed `cloned_iterators` multiplication type error was rejected because the file had already passed standalone `rustc` validation and produced the expected runtime output in this workspace
- reviewer tooling note:
  - batch 20 again used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_21_iterator_basics_generic_functions_and_iterators_itertools_iterators

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/iterator_basics/idiomatic.rs`
  - `demos/generic_functions_and_iterators/idiomatic.rs`
  - `demos/itertools_iterators/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining iterator-and-generic-reference slice instead of mixing the last small iterator demos with unrelated stdlib areas
  - each companion still retained generated-style iterator plumbing, borrowed/reference mismatches, or faux-itertools scaffolding despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `iterator_basics`, `generic_functions_and_iterators`, `itertools_iterators`
  - `hand-authored-generated-shape`: `iterator_basics`, `generic_functions_and_iterators`, `itertools_iterators`
- implementation summary:
  - `iterator_basics`: replaced boxed/generator-style scaffolding with direct iterator helpers for odd-number generation, counting, pair combinations, repeated-cartesian products, and compact borrowed iterator consumption in `main`
  - `generic_functions_and_iterators`: replaced generated wrappers with direct Rust generics, a compact generic `Container<T>`, explicit trait-object protocol dispatch, and flattened iterator-based comprehension parity
  - `itertools_iterators`: replaced helper scaffolding with small direct iterator adapters for repeat/count behavior and compact `chain`/slice/count output parity
- local validation completed:
  - `rustfmt demos/iterator_basics/idiomatic.rs demos/generic_functions_and_iterators/idiomatic.rs demos/itertools_iterators/idiomatic.rs`
  - `rustc --edition=2021 demos/iterator_basics/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-basics && /tmp/sifr-idiomatic-iterator-basics`
  - `rustc --edition=2021 demos/generic_functions_and_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-generic-functions-and-iterators && /tmp/sifr-idiomatic-generic-functions-and-iterators`
  - `rustc --edition=2021 demos/itertools_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-itertools-iterators && /tmp/sifr-idiomatic-itertools-iterators`
  - `cargo run -q -p sifr -- run demos/iterator_basics/main.sifr`
  - `cargo run -q -p sifr -- run demos/generic_functions_and_iterators/main.sifr`
  - `cargo run -q -p sifr -- run demos/itertools_iterators/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/iterator_basics/idiomatic.rs demos/generic_functions_and_iterators/idiomatic.rs demos/itertools_iterators/idiomatic.rs`
    - `rustc --edition=2021 demos/generic_functions_and_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-generic-functions-and-iterators && /tmp/sifr-idiomatic-generic-functions-and-iterators`
    - `cargo run -q -p sifr -- run demos/generic_functions_and_iterators/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/iterator_basics/idiomatic.rs demos/generic_functions_and_iterators/idiomatic.rs demos/itertools_iterators/idiomatic.rs`
    - `rustc --edition=2021 demos/generic_functions_and_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-generic-functions-and-iterators && /tmp/sifr-idiomatic-generic-functions-and-iterators`
    - `cargo run -q -p sifr -- run demos/generic_functions_and_iterators/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-21-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-21-review-pass-2.md`
- review application summary:
  - pass 1 accepted one follow-up in `demos/generic_functions_and_iterators/idiomatic.rs`: made protocol dispatch explicit while the batch was still under review
  - pass 1's `itertools_iterators` per-file reviewer attempt timed out in this workspace, but no blocker was established and local validation stayed green
  - pass 2 accepted two ownership-parity fixes in `demos/generic_functions_and_iterators/idiomatic.rs`: `Container::get` now returns by value, and `show` now consumes `Box<dyn Printable>`
  - pass 2 re-review on the final `generic_functions_and_iterators` file reported no remaining actionable issues
  - `iterator_basics` and `itertools_iterators` passed pass 2 with no actionable issues
- reviewer tooling note:
  - batch 21 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_22_iteration_basics_iterator_builtins_iterators_and_comprehensions

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/iteration_basics/idiomatic.rs`
  - `demos/iterator_builtins/idiomatic.rs`
  - `demos/iterators_and_comprehensions/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining iterator-basics slice instead of mixing small iteration demos with unrelated stdlib areas
  - each companion still retained iterator boxing, explicit lowering scaffolds, or other generated-style ceremony despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `iteration_basics`, `iterator_builtins`, `iterators_and_comprehensions`
  - `hand-authored-generated-shape`: `iteration_basics`, `iterator_builtins`, `iterators_and_comprehensions`
- implementation summary:
  - `iteration_basics`: reduced the companion to direct string iteration, explicit key-list iteration, and compact output/assertion handling
  - `iterator_builtins`: replaced boxed iterators and one-off lowering scaffolds with direct borrowed iterator chains plus a small `sorted(values, reverse)` helper mirroring the Sifr surface
  - `iterators_and_comprehensions`: replaced comprehension scaffolding and iterator boxing with direct borrowed iterator chains for map/filter/min/max/sum/reversed/enumerate/zip/any/all parity
- local validation completed:
  - `rustfmt demos/iteration_basics/idiomatic.rs demos/iterator_builtins/idiomatic.rs demos/iterators_and_comprehensions/idiomatic.rs`
  - `rustc --edition=2021 demos/iteration_basics/idiomatic.rs -o /tmp/sifr-idiomatic-iteration-basics && /tmp/sifr-idiomatic-iteration-basics`
  - `rustc --edition=2021 demos/iterator_builtins/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-builtins && /tmp/sifr-idiomatic-iterator-builtins`
  - `rustc --edition=2021 demos/iterators_and_comprehensions/idiomatic.rs -o /tmp/sifr-idiomatic-iterators-and-comprehensions && /tmp/sifr-idiomatic-iterators-and-comprehensions`
  - `cargo run -q -p sifr -- run demos/iteration_basics/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterator_builtins/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterators_and_comprehensions/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/iteration_basics/idiomatic.rs demos/iterator_builtins/idiomatic.rs demos/iterators_and_comprehensions/idiomatic.rs`
    - `rustc --edition=2021 demos/iterator_builtins/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-builtins && /tmp/sifr-idiomatic-iterator-builtins`
    - `rustc --edition=2021 demos/iterators_and_comprehensions/idiomatic.rs -o /tmp/sifr-idiomatic-iterators-and-comprehensions && /tmp/sifr-idiomatic-iterators-and-comprehensions`
    - `cargo run -q -p sifr -- run demos/iterator_builtins/main.sifr`
    - `cargo run -q -p sifr -- run demos/iterators_and_comprehensions/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-22-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-22-review-pass-2.md`
- review application summary:
  - pass 1 reported no issues in `iteration_basics`
  - pass 1 follow-ups were accepted in `iterator_builtins` and `iterators_and_comprehensions` to make repeated sequence traversal explicitly borrowed and to model `sorted(..., reverse=True)` more directly
  - pass 1's move/use-after-consume framing was not accepted as the root issue because the pre-fix files had already compiled successfully in this workspace; the applied change was taken for parity clarity instead
  - pass 2 reported no accepted blockers
  - the pass-2 `iteration_basics` note was rejected because the paired Sifr source iterates an explicit `keys` list, not the dictionary itself
  - the pass-2 `iterators_and_comprehensions` `sorted()` note was rejected because the Rust companion sorts a cloned `Vec` and preserves the original unsorted sequence
- reviewer tooling note:
  - batch 22 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_23_generator_functions_generator_iterators_custom_iterables

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/generator_functions/idiomatic.rs`
  - `demos/generator_iterators/idiomatic.rs`
  - `demos/custom_iterables/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining generator-and-custom-iterator slice instead of mixing the small lazy-iteration demos with unrelated stdlib areas
  - each companion still retained boxed-generator scaffolding, eager lowering artifacts, or faux protocol machinery despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `generator_functions`, `generator_iterators`, `custom_iterables`
  - `hand-authored-generated-shape`: `generator_functions`, `generator_iterators`, `custom_iterables`
- implementation summary:
  - `generator_functions`: replaced generated generator scaffolding with a direct countdown iterator plus compact `Option` formatting for the printed `next` results
  - `generator_iterators`: replaced boxed/eager generator lowering with direct iterator adapters, then refined the file in pass 2 to keep the generator-expression path lazy and make `gen_pairs` stateful rather than a bare range
  - `custom_iterables`: replaced the generated protocol-style helper methods with a small explicit `Iterator` implementation for `CountdownIter`, an `IntoIterator` implementation for `Countdown`, and a direct ascending `reversed` helper
- local validation completed:
  - `rustfmt demos/generator_functions/idiomatic.rs demos/generator_iterators/idiomatic.rs demos/custom_iterables/idiomatic.rs`
  - `rustc --edition=2021 demos/generator_functions/idiomatic.rs -o /tmp/sifr-idiomatic-generator-functions && /tmp/sifr-idiomatic-generator-functions`
  - `rustc --edition=2021 demos/generator_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-generator-iterators && /tmp/sifr-idiomatic-generator-iterators`
  - `rustc --edition=2021 demos/custom_iterables/idiomatic.rs -o /tmp/sifr-idiomatic-custom-iterables && /tmp/sifr-idiomatic-custom-iterables`
  - `cargo run -q -p sifr -- run demos/generator_functions/main.sifr`
  - `cargo run -q -p sifr -- run demos/generator_iterators/main.sifr`
  - `cargo run -q -p sifr -- run demos/custom_iterables/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/generator_functions/idiomatic.rs demos/generator_iterators/idiomatic.rs demos/custom_iterables/idiomatic.rs`
    - `rustc --edition=2021 demos/generator_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-generator-iterators && /tmp/sifr-idiomatic-generator-iterators`
    - `cargo run -q -p sifr -- run demos/generator_iterators/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-23-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-23-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 `generator_functions` single-use note was rejected because it compared two separately constructed iterators, which is allowed in both languages and does not make consumed iterator state reusable
  - pass 2 accepted one follow-up in `demos/generator_iterators/idiomatic.rs` to preserve the generator-expression path as a lazy iterator until collection and to make `gen_pairs` a small stateful iterator helper
  - `generator_iterators` was re-reviewed after that change and returned no actionable issues
  - the initial pass-2 `generator_functions` reviewer attempt timed out in this workspace without yielding a usable note set
- reviewer tooling note:
  - batch 23 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_24_extended_builtin_iterators_reversible_iterables_lazy_builtins

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/extended_builtin_iterators/idiomatic.rs`
  - `demos/reversible_iterables/idiomatic.rs`
  - `demos/lazy_builtins/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining builtin-iterator slice instead of mixing small lazy-iterator demos with unrelated stdlib areas
  - each companion still retained boxed iterator scaffolding, tuple-lowering artifacts, or generated-style zip/enumerate plumbing despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `extended_builtin_iterators`, `reversible_iterables`, `lazy_builtins`
  - `hand-authored-generated-shape`: `extended_builtin_iterators`, `reversible_iterables`, `lazy_builtins`
- implementation summary:
  - `extended_builtin_iterators`: replaced boxed iterators and lowering helpers with direct array-backed `reversed`/`enumerate`/`zip`/`map` chains and compact structural assertions
  - `reversible_iterables`: replaced iterator boxing and tuple-lowering scaffolding with a direct `DoubleEndedIterator` helper plus small array-backed tuple materialization
  - `lazy_builtins`: replaced boxed iterators and nested zip scaffolding with direct iterator chains for `reversed`, `enumerate`, and three-way `zip` flattening
- local validation completed:
  - `rustfmt demos/extended_builtin_iterators/idiomatic.rs demos/reversible_iterables/idiomatic.rs demos/lazy_builtins/idiomatic.rs`
  - `rustc --edition=2021 demos/extended_builtin_iterators/idiomatic.rs -o /tmp/sifr-idiomatic-extended-builtin-iterators && /tmp/sifr-idiomatic-extended-builtin-iterators`
  - `rustc --edition=2021 demos/reversible_iterables/idiomatic.rs -o /tmp/sifr-idiomatic-reversible-iterables && /tmp/sifr-idiomatic-reversible-iterables`
  - `rustc --edition=2021 demos/lazy_builtins/idiomatic.rs -o /tmp/sifr-idiomatic-lazy-builtins && /tmp/sifr-idiomatic-lazy-builtins`
  - `cargo run -q -p sifr -- run demos/extended_builtin_iterators/main.sifr`
  - `cargo run -q -p sifr -- run demos/reversible_iterables/main.sifr`
  - `cargo run -q -p sifr -- run demos/lazy_builtins/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-24-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-24-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 `lazy_builtins` nested-tuple note was rejected because the final `.map()` already flattened the iterator output to `(int, str, bool)` and the runtime output matched the paired Sifr demo
  - the pass-1 `reversible_iterables` reviewer invocation timed out in this workspace without producing a usable note set
  - pass 2 reported no accepted blockers
  - the pass-2 `lazy_builtins` ownership note was rejected because Rust 2021 array `into_iter()` yields owned `i64` values and the file already passed local compilation
  - the pass-2 `extended_builtin_iterators` reviewer invocation also timed out without establishing a blocker
- reviewer tooling note:
  - batch 24 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_25_generators_generator_break_else_iterator_types

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/generators/idiomatic.rs`
  - `demos/generator_break_else/idiomatic.rs`
  - `demos/iterator_types/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining generator/protocol slice instead of mixing generator-control-flow demos with unrelated stdlib areas
  - each companion still retained boxed-generator scaffolding, generated control-flow state, or overly specific collection signatures despite compact demo-visible behavior
- priority tags:
  - `iterator-surface`: `generators`, `generator_break_else`, `iterator_types`
  - `hand-authored-generated-shape`: `generators`, `generator_break_else`, `iterator_types`
- implementation summary:
  - `generators`: replaced boxed generator scaffolding with direct iterator helpers for Fibonacci and evens plus a small RAII-style timer guard matching the with-statement behavior
  - `generator_break_else`: replaced eager boxed-generator materialization with a compact `from_fn` state machine that preserves the break/else yield behavior directly
  - `iterator_types`: replaced collection-specific signatures with direct `IntoIterator`/generic iterator contracts and then removed an extra runtime `passthrough` call so the Rust companion matches the actual Sifr demo behavior
- local validation completed:
  - `rustfmt demos/generators/idiomatic.rs demos/generator_break_else/idiomatic.rs demos/iterator_types/idiomatic.rs`
  - `rustc --edition=2021 demos/generators/idiomatic.rs -o /tmp/sifr-idiomatic-generators && /tmp/sifr-idiomatic-generators`
  - `rustc --edition=2021 demos/generator_break_else/idiomatic.rs -o /tmp/sifr-idiomatic-generator-break-else && /tmp/sifr-idiomatic-generator-break-else`
  - `rustc --edition=2021 demos/iterator_types/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-types && /tmp/sifr-idiomatic-iterator-types`
  - `cargo run -q -p sifr -- run demos/generators/main.sifr`
  - `cargo run -q -p sifr -- run demos/generator_break_else/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterator_types/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/generators/idiomatic.rs demos/generator_break_else/idiomatic.rs demos/iterator_types/idiomatic.rs`
    - `rustc --edition=2021 demos/iterator_types/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-types && /tmp/sifr-idiomatic-iterator-types`
    - `cargo run -q -p sifr -- run demos/iterator_types/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-25-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-25-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 `iterator_types` note was rejected because it claimed `passthrough` was called in the paired Sifr demo even though the source does not invoke it
  - pass 2 accepted one follow-up in `demos/iterator_types/idiomatic.rs` to remove the extra runtime `passthrough` call from `main`
  - `iterator_types` was re-reviewed after that change and returned no actionable issues
- reviewer tooling note:
  - batch 25 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_26_lazy_iterators_basics_iterator_lowering_iterator_codegen

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/lazy_iterators_basics/idiomatic.rs`
  - `demos/iterator_lowering/idiomatic.rs`
  - `demos/iterator_codegen/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining canonical iterator-lowering slice instead of mixing small iterator demos with larger protocol-heavy companions
  - each companion still retained hand-authored generated-shape scaffolding or ownership-signaling patterns that were heavier than the demo-visible behavior
- priority tags:
  - `iterator-surface`: `lazy_iterators_basics`, `iterator_lowering`, `iterator_codegen`
  - `hand-authored-generated-shape`: `lazy_iterators_basics`, `iterator_lowering`, `iterator_codegen`
- implementation summary:
  - `lazy_iterators_basics`: replaced boxed/lowered iterator scaffolding with direct Rust iterator helpers for `chain`, `count`, and compact assertion-based parity checks across `next`, `map`, `filter`, `zip`, `enumerate`, and `reversed`
  - `iterator_lowering`: replaced lowering-oriented scaffolding with direct iterator chains and minimal collection materialization matching the printed demo behavior
  - `iterator_codegen`: replaced custom filter/sort scaffolding with direct iterator usage, borrowed iteration for repeated traversals, and a simple cloned sort path matching the paired demo output
- local validation completed:
  - `rustfmt demos/lazy_iterators_basics/idiomatic.rs demos/iterator_lowering/idiomatic.rs demos/iterator_codegen/idiomatic.rs`
  - `rustc --edition=2021 demos/lazy_iterators_basics/idiomatic.rs -o /tmp/sifr-idiomatic-lazy-iterators-basics && /tmp/sifr-idiomatic-lazy-iterators-basics`
  - `rustc --edition=2021 demos/iterator_lowering/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-lowering && /tmp/sifr-idiomatic-iterator-lowering`
  - `rustc --edition=2021 demos/iterator_codegen/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-codegen && /tmp/sifr-idiomatic-iterator-codegen`
  - `cargo run -q -p sifr -- run demos/lazy_iterators_basics/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterator_lowering/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterator_codegen/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/lazy_iterators_basics/idiomatic.rs demos/iterator_lowering/idiomatic.rs demos/iterator_codegen/idiomatic.rs`
    - `rustc --edition=2021 demos/lazy_iterators_basics/idiomatic.rs -o /tmp/sifr-idiomatic-lazy-iterators-basics && /tmp/sifr-idiomatic-lazy-iterators-basics`
    - `rustc --edition=2021 demos/iterator_codegen/idiomatic.rs -o /tmp/sifr-idiomatic-iterator-codegen && /tmp/sifr-idiomatic-iterator-codegen`
    - `cargo run -q -p sifr -- run demos/lazy_iterators_basics/main.sifr`
    - `cargo run -q -p sifr -- run demos/iterator_codegen/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-26-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-26-review-pass-2.md`
- review application summary:
  - pass 1 accepted borrowed-iterator clarity follow-ups in `demos/lazy_iterators_basics/idiomatic.rs` and `demos/iterator_codegen/idiomatic.rs` so repeated traversals do not rely on array-copy semantics
  - pass 1 reported no actionable issues in `demos/iterator_lowering/idiomatic.rs`
  - the move/use-after-consume framing from pass 1 was not accepted as the root issue because these array traversals compile via copy semantics; the accepted edits were taken for iterator-parity clarity
  - pass 2 reported no actionable issues on the final code state
- reviewer tooling note:
  - batch 26 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_27_recursive_calls_recursive_for_else_while_else

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/recursive_calls/idiomatic.rs`
  - `demos/recursive_for_else/idiomatic.rs`
  - `demos/while_else/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining structured-control-flow slice instead of mixing small recursion/control-flow demos with larger stdlib-heavy companions
  - each companion still exposed compiler-lowered `_broke` scaffolding or traversal artifacts that were heavier than the demo-visible behavior
- priority tags:
  - `control-flow-surface`: `recursive_calls`, `recursive_for_else`, `while_else`
  - `hand-authored-generated-shape`: `recursive_calls`, `recursive_for_else`, `while_else`
- implementation summary:
  - `recursive_calls`: replaced the synthetic loop-and-flag scaffolding with direct recursive control flow that preserves the visible return behavior
  - `recursive_for_else`: reduced the companion to the minimal loop-plus-recursion shape that still reflects the paired `for-else` demo output
  - `while_else`: replaced the lowered `_broke` flag shape with a direct empty-vs-nonempty branch that matches the observable `while-else` result
- local validation completed:
  - `rustfmt demos/recursive_calls/idiomatic.rs demos/recursive_for_else/idiomatic.rs demos/while_else/idiomatic.rs`
  - `rustc --edition=2021 demos/recursive_calls/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-calls && /tmp/sifr-idiomatic-recursive-calls`
  - `rustc --edition=2021 demos/recursive_for_else/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-for-else && /tmp/sifr-idiomatic-recursive-for-else`
  - `rustc --edition=2021 demos/while_else/idiomatic.rs -o /tmp/sifr-idiomatic-while-else && /tmp/sifr-idiomatic-while-else`
  - `cargo run -q -p sifr -- run demos/recursive_calls/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_for_else/main.sifr`
  - `cargo run -q -p sifr -- run demos/while_else/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-27-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-27-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 `recursive_for_else` note was rejected because it incorrectly claimed `rec(3)` was not printed even though the reviewed Rust file already contains `println!("{}", rec(3));` and the validated runtime output matched the paired Sifr demo
  - pass 2 reported no actionable issues on the final code state
- reviewer tooling note:
  - batch 27 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remained unreliable in this workspace

#### batch_28_borrow_by_default_borrowed_builtins_generic_cloning

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/borrow_by_default/idiomatic.rs`
  - `demos/borrowed_builtins/idiomatic.rs`
  - `demos/generic_cloning/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining ownership/borrowing slice instead of mixing small borrowing demos with unrelated diagnostic or verifier demos
  - `borrow_by_default` still retained lowered string-indexing and borrow-shape scaffolding, while the other two companions benefited from tighter borrowed-usage references
- priority tags:
  - `ownership-surface`: `borrow_by_default`, `borrowed_builtins`, `generic_cloning`
  - `hand-authored-generated-shape`: `borrow_by_default`, `borrowed_builtins`, `generic_cloning`
- implementation summary:
  - `borrow_by_default`: replaced lowered borrow and indexing scaffolding with direct slice and `&str` helpers while preserving the visible borrow-by-default, own-parameter, and callable-borrow behaviors
  - `borrowed_builtins`: reduced the companion to direct borrowed string and list builtin usage with compact assertions that preserve the non-consuming behavior
  - `generic_cloning`: replaced manual accumulation scaffolding with direct iterator collection for the pair totals and a compact empty-collection count path preserving the printed output
- local validation completed:
  - `rustfmt demos/borrow_by_default/idiomatic.rs demos/borrowed_builtins/idiomatic.rs demos/generic_cloning/idiomatic.rs`
  - `rustc --edition=2021 demos/borrow_by_default/idiomatic.rs -o /tmp/sifr-idiomatic-borrow-by-default && /tmp/sifr-idiomatic-borrow-by-default`
  - `rustc --edition=2021 demos/borrowed_builtins/idiomatic.rs -o /tmp/sifr-idiomatic-borrowed-builtins && /tmp/sifr-idiomatic-borrowed-builtins`
  - `rustc --edition=2021 demos/generic_cloning/idiomatic.rs -o /tmp/sifr-idiomatic-generic-cloning && /tmp/sifr-idiomatic-generic-cloning`
  - `cargo run -q -p sifr -- run demos/borrow_by_default/main.sifr`
  - `cargo run -q -p sifr -- run demos/borrowed_builtins/main.sifr`
  - `cargo run -q -p sifr -- run demos/generic_cloning/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-28-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-28-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 and pass-2 `borrow_by_default` notes asking for `char` instead of `String` in `get_first_char` were rejected because this corpus consistently maps Sifr `str` to Rust `String`, and the current companion already matched the observed demo output exactly
  - pass 2 reported no actionable issues in `borrowed_builtins` and `generic_cloning`
- reviewer tooling note:
  - batch 28 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts stalled repeatedly in this workspace

#### batch_29_type_checking_constrained_typevars_protocol_bounds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/type_checking/idiomatic.rs`
  - `demos/constrained_typevars/idiomatic.rs`
  - `demos/protocol_bounds/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining type-system slice instead of mixing small typing demos with unrelated ownership or control-flow companions
  - `constrained_typevars` and `protocol_bounds` still benefited from tighter Rust-first constraint modeling, while `type_checking` was kept as the minimal typed-success reference
- priority tags:
  - `type-system-surface`: `type_checking`, `constrained_typevars`, `protocol_bounds`
  - `hand-authored-generated-shape`: `constrained_typevars`, `protocol_bounds`
- implementation summary:
  - `type_checking`: reduced the companion to the minimal direct typed identity example with no extra annotation noise
  - `constrained_typevars`: removed unnecessary generic bounds, then added an explicit `EchoType` marker trait for the closed `int|str` constraint and a `Comparable` wrapper over `PartialOrd` so the Rust companion mirrors the paired Sifr constraint concepts more directly
  - `protocol_bounds`: simplified forwarding to direct by-value generic calls while preserving the protocol-bound behavior and visible output
- local validation completed:
  - `rustfmt demos/type_checking/idiomatic.rs demos/constrained_typevars/idiomatic.rs demos/protocol_bounds/idiomatic.rs`
  - `rustc --edition=2021 demos/type_checking/idiomatic.rs -o /tmp/sifr-idiomatic-type-checking && /tmp/sifr-idiomatic-type-checking`
  - `rustc --edition=2021 demos/constrained_typevars/idiomatic.rs -o /tmp/sifr-idiomatic-constrained-typevars && /tmp/sifr-idiomatic-constrained-typevars`
  - `rustc --edition=2021 demos/protocol_bounds/idiomatic.rs -o /tmp/sifr-idiomatic-protocol-bounds && /tmp/sifr-idiomatic-protocol-bounds`
  - `cargo run -q -p sifr -- run demos/type_checking/main.sifr`
  - `cargo run -q -p sifr -- run demos/constrained_typevars/main.sifr`
  - `cargo run -q -p sifr -- run demos/protocol_bounds/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/type_checking/idiomatic.rs demos/constrained_typevars/idiomatic.rs demos/protocol_bounds/idiomatic.rs`
    - `rustc --edition=2021 demos/constrained_typevars/idiomatic.rs -o /tmp/sifr-idiomatic-constrained-typevars && /tmp/sifr-idiomatic-constrained-typevars`
    - `cargo run -q -p sifr -- run demos/constrained_typevars/main.sifr`
    - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/type_checking/idiomatic.rs demos/constrained_typevars/idiomatic.rs demos/protocol_bounds/idiomatic.rs`
    - `rustc --edition=2021 demos/constrained_typevars/idiomatic.rs -o /tmp/sifr-idiomatic-constrained-typevars && /tmp/sifr-idiomatic-constrained-typevars`
    - `cargo run -q -p sifr -- run demos/constrained_typevars/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-29-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-29-review-pass-2.md`
- review application summary:
  - pass 1 accepted one follow-up in `demos/constrained_typevars/idiomatic.rs` adding an explicit marker trait so `echo` no longer accepts arbitrary types where the paired Sifr `TypeVar` is constrained to `int` and `str`
  - pass 1 rejected the remaining `Comparable` and owned-string notes because `PartialOrd` and Rust `String` remain the established corpus mappings for those Sifr concepts
  - pass 2 accepted one refinement in `demos/constrained_typevars/idiomatic.rs` introducing a named `Comparable` trait wrapper and clearer constraint naming for the explicit `TypeVar` marker
  - `constrained_typevars` was re-reviewed after that change and returned no actionable issues
- reviewer tooling note:
  - batch 29 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts stalled repeatedly in this workspace

#### batch_30_early_return_paths_unreachable_returns_valid_control_flow

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/early_return_paths/idiomatic.rs`
  - `demos/unreachable_returns/idiomatic.rs`
  - `demos/valid_control_flow/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining CFG/control-flow slice instead of mixing small flow-analysis demos with unrelated ownership or verifier demos
  - each companion benefited from trimming annotation noise and preserving only the visible branch/loop behavior
- priority tags:
  - `cfg-surface`: `early_return_paths`, `unreachable_returns`, `valid_control_flow`
  - `hand-authored-generated-shape`: `early_return_paths`, `unreachable_returns`, `valid_control_flow`
- implementation summary:
  - `early_return_paths`: collapsed the always-exits lowering shape to the direct `Option` early-return path that still returns `0` for `None`
  - `unreachable_returns`: reduced the companion to the minimal conditional return plus consumer flow, preserving the unreachable-tail inference behavior
  - `valid_control_flow`: simplified the loop to direct `continue`/`break` control flow over `0..limit` while preserving the printed total
- local validation completed:
  - `rustfmt demos/early_return_paths/idiomatic.rs demos/unreachable_returns/idiomatic.rs demos/valid_control_flow/idiomatic.rs`
  - `rustc --edition=2021 demos/early_return_paths/idiomatic.rs -o /tmp/sifr-idiomatic-early-return-paths && /tmp/sifr-idiomatic-early-return-paths`
  - `rustc --edition=2021 demos/unreachable_returns/idiomatic.rs -o /tmp/sifr-idiomatic-unreachable-returns && /tmp/sifr-idiomatic-unreachable-returns`
  - `rustc --edition=2021 demos/valid_control_flow/idiomatic.rs -o /tmp/sifr-idiomatic-valid-control-flow && /tmp/sifr-idiomatic-valid-control-flow`
  - `cargo run -q -p sifr -- run demos/early_return_paths/main.sifr`
  - `cargo run -q -p sifr -- run demos/unreachable_returns/main.sifr`
  - `cargo run -q -p sifr -- run demos/valid_control_flow/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-30-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-30-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 30 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remain less reliable in this workspace

#### batch_31_optional_indexing_optional_arithmetic_return_type_inference

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/optional_indexing/idiomatic.rs`
  - `demos/optional_arithmetic/idiomatic.rs`
  - `demos/return_type_inference/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining option-and-inference slice instead of mixing small optional-behavior demos with unrelated diagnostics or mutation demos
  - `optional_indexing` still retained lowered indexing scaffolding, while `optional_arithmetic` and `return_type_inference` benefited from tighter Rust-first expression style and output parity cleanup
- priority tags:
  - `option-surface`: `optional_indexing`, `optional_arithmetic`
  - `type-inference-surface`: `return_type_inference`
  - `hand-authored-generated-shape`: `optional_indexing`, `optional_arithmetic`, `return_type_inference`
- implementation summary:
  - `optional_indexing`: replaced lowered list-index normalization scaffolding with a direct `get(1).copied()` option path
  - `optional_arithmetic`: reduced the companion to compact `let-else` narrowing and direct arithmetic expressions preserving the same optional behavior
  - `return_type_inference`: simplified the helper signatures and bodies to direct Rust-first expressions while preserving the observed quoted string output contract for `greet`
- local validation completed:
  - `rustfmt demos/optional_indexing/idiomatic.rs demos/optional_arithmetic/idiomatic.rs demos/return_type_inference/idiomatic.rs`
  - `rustc --edition=2021 demos/optional_indexing/idiomatic.rs -o /tmp/sifr-idiomatic-optional-indexing && /tmp/sifr-idiomatic-optional-indexing`
  - `rustc --edition=2021 demos/optional_arithmetic/idiomatic.rs -o /tmp/sifr-idiomatic-optional-arithmetic && /tmp/sifr-idiomatic-optional-arithmetic`
  - `rustc --edition=2021 demos/return_type_inference/idiomatic.rs -o /tmp/sifr-idiomatic-return-type-inference && /tmp/sifr-idiomatic-return-type-inference`
  - `cargo run -q -p sifr -- run demos/optional_indexing/main.sifr`
  - `cargo run -q -p sifr -- run demos/optional_arithmetic/main.sifr`
  - `cargo run -q -p sifr -- run demos/return_type_inference/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-fix revalidation:
    - `rustfmt demos/optional_indexing/idiomatic.rs demos/optional_arithmetic/idiomatic.rs demos/return_type_inference/idiomatic.rs`
    - standalone `rustc` validation for all three companions
    - targeted Sifr demo runs for all three demos
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-31-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-31-review-pass-2.md`
- review application summary:
  - a pre-review parity fix changed `return_type_inference` back to debug formatting for `greet("sifr")` after the paired Sifr run confirmed the observed output includes quotes
  - pass 1 reported no accepted blockers
  - the pass-1 note against `{:?}` in `return_type_inference` was rejected because the paired Sifr output in this workspace is `"hello sifr"`, so the debug-format output is the actual parity-preserving behavior
  - pass 2 reported no accepted blockers
  - one pass-2 attempt on `optional_indexing` returned a tool-seeking response instead of a review result, so that file was rerun directly and came back clean
  - the pass-2 `format!` vs `+` note in `return_type_inference` was rejected because it was stylistic rather than a semantic mismatch
- reviewer tooling note:
  - batch 31 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger combined prompts and tool-seeking outputs were less reliable in this workspace during pass 2

#### batch_32_monotonic_indices_reverse_indices_indexed_tables

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/monotonic_indices/idiomatic.rs`
  - `demos/reverse_indices/idiomatic.rs`
  - `demos/indexed_tables/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining index-safety slice instead of mixing small indexing demos with unrelated option or mutation companions
  - each companion still retained obvious lowered index scaffolding or explicit normalization noise that could be replaced with direct safe Rust indexing loops
- priority tags:
  - `indexing-surface`: `monotonic_indices`, `reverse_indices`, `indexed_tables`
  - `hand-authored-generated-shape`: `monotonic_indices`, `reverse_indices`, `indexed_tables`
- implementation summary:
  - `monotonic_indices`: replaced the integer-range-plus-cast shape with a direct forward index loop over a borrowed bool slice
  - `reverse_indices`: replaced the lowered reverse-range and negative-index scaffolding with a direct `(0..len).rev()` loop that still indexes back into the slice
  - `indexed_tables`: replaced the list-comprehension and normalized mutable indexing scaffolding with a direct pre-sized vector and indexed writes
- local validation completed:
  - `rustfmt demos/monotonic_indices/idiomatic.rs demos/reverse_indices/idiomatic.rs demos/indexed_tables/idiomatic.rs`
  - `rustc --edition=2021 demos/monotonic_indices/idiomatic.rs -o /tmp/sifr-idiomatic-monotonic-indices && /tmp/sifr-idiomatic-monotonic-indices`
  - `rustc --edition=2021 demos/reverse_indices/idiomatic.rs -o /tmp/sifr-idiomatic-reverse-indices && /tmp/sifr-idiomatic-reverse-indices`
  - `rustc --edition=2021 demos/indexed_tables/idiomatic.rs -o /tmp/sifr-idiomatic-indexed-tables && /tmp/sifr-idiomatic-indexed-tables`
  - `cargo run -q -p sifr -- run demos/monotonic_indices/main.sifr`
  - `cargo run -q -p sifr -- run demos/reverse_indices/main.sifr`
  - `cargo run -q -p sifr -- run demos/indexed_tables/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-32-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-32-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 32 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remain less reliable in this workspace

#### batch_33_local_shadowing_sentinel_values_set_operations

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/local_shadowing/idiomatic.rs`
  - `demos/sentinel_values/idiomatic.rs`
  - `demos/set_operations/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining state-and-collections slice instead of mixing small rebinding/sentinel demos with unrelated indexing or option demos
  - `sentinel_values` still had explicit sentinel scaffolding and `set_operations` still had small API-mapping cleanup available even though the visible demo output was already simple
- priority tags:
  - `local-state-surface`: `local_shadowing`, `sentinel_values`
  - `collection-surface`: `set_operations`
  - `hand-authored-generated-shape`: `sentinel_values`, `set_operations`
- implementation summary:
  - `local_shadowing`: reduced local updates and branch returns to direct Rust-first expressions while preserving the same assertions
  - `sentinel_values`: replaced explicit sentinel bookkeeping with `min().unwrap_or(0)` as the direct Rust equivalent of the demo’s “smallest or zero” contract
  - `set_operations`: kept the same visible operations while simplifying summation and then removing the temporary string allocation in `remove`
- local validation completed:
  - `rustfmt demos/local_shadowing/idiomatic.rs demos/sentinel_values/idiomatic.rs demos/set_operations/idiomatic.rs`
  - `rustc --edition=2021 demos/local_shadowing/idiomatic.rs -o /tmp/sifr-idiomatic-local-shadowing && /tmp/sifr-idiomatic-local-shadowing`
  - `rustc --edition=2021 demos/sentinel_values/idiomatic.rs -o /tmp/sifr-idiomatic-sentinel-values && /tmp/sifr-idiomatic-sentinel-values`
  - `rustc --edition=2021 demos/set_operations/idiomatic.rs -o /tmp/sifr-idiomatic-set-operations && /tmp/sifr-idiomatic-set-operations`
  - `cargo run -q -p sifr -- run demos/local_shadowing/main.sifr`
  - `cargo run -q -p sifr -- run demos/sentinel_values/main.sifr`
  - `cargo run -q -p sifr -- run demos/set_operations/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-2 revalidation:
    - `rustfmt demos/local_shadowing/idiomatic.rs demos/sentinel_values/idiomatic.rs demos/set_operations/idiomatic.rs`
    - `rustc --edition=2021 demos/set_operations/idiomatic.rs -o /tmp/sifr-idiomatic-set-operations && /tmp/sifr-idiomatic-set-operations`
    - `cargo run -q -p sifr -- run demos/set_operations/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-33-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-33-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 accepted one small follow-up in `demos/set_operations/idiomatic.rs` replacing `remove(&\"banana\".to_string())` with `remove(\"banana\")`
  - `set_operations` was re-reviewed after that change and returned no actionable issues
- reviewer tooling note:
  - batch 33 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remain less reliable in this workspace

#### batch_34_container_literals_collection_cloning_own_mut_appends

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/container_literals/idiomatic.rs`
  - `demos/collection_cloning/idiomatic.rs`
  - `demos/own_mut_appends/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining collections-and-owned-mutation slice instead of mixing small container demos with unrelated fixture or control-flow work
  - `container_literals` and `collection_cloning` still had generated-shape scaffolding, while `own_mut_appends` kept the slice centered on small owned append-and-return helpers
- priority tags:
  - `collection-surface`: `container_literals`, `collection_cloning`, `own_mut_appends`
  - `hand-authored-generated-shape`: `container_literals`, `collection_cloning`
- implementation summary:
  - `container_literals`: replaced lowered keyed-assignment scaffolding with direct `HashMap::entry` counting and a direct score sum
  - `collection_cloning`: removed boxed iterator and star-unpack-style scaffolding in favor of direct `map`, `filter`, and slice splitting
  - `own_mut_appends`: reduced append-and-return helpers plus assertion style to the smallest direct Rust equivalents
- local validation completed:
  - `rustfmt demos/container_literals/idiomatic.rs demos/collection_cloning/idiomatic.rs demos/own_mut_appends/idiomatic.rs`
  - `rustc --edition=2021 demos/container_literals/idiomatic.rs -o /tmp/sifr-idiomatic-container-literals && /tmp/sifr-idiomatic-container-literals`
  - `rustc --edition=2021 demos/collection_cloning/idiomatic.rs -o /tmp/sifr-idiomatic-collection-cloning && /tmp/sifr-idiomatic-collection-cloning`
  - `rustc --edition=2021 demos/own_mut_appends/idiomatic.rs -o /tmp/sifr-idiomatic-own-mut-appends && /tmp/sifr-idiomatic-own-mut-appends`
  - `cargo run -q -p sifr -- run demos/container_literals/main.sifr`
  - `cargo run -q -p sifr -- run demos/collection_cloning/main.sifr`
  - `cargo run -q -p sifr -- run demos/own_mut_appends/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-34-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-34-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - the pass-1 `container_literals` note about Python-style overwrite semantics was recorded but not accepted because the Rust companion already matched the paired Sifr assertions and observed runtime behavior
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 34 used stable per-file external review prompts embedding both the paired Sifr source and the Rust companion because larger batch prompts remain less reliable in this workspace

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

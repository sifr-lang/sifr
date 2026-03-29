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

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
  - agent CLI
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
  - any folder already called out in a agent review artifact as `Issues Found`
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
  - the `desktop agent review handoff` wrapper hung before producing a file in this workspace
  - external review artifacts were produced via direct `agent review ...` runs and then written into the recorded review files

### wave_2_runnable_demo_corpus_pass

status: completed

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
  - direct shell-session `agent review` launches stalled without producing review files during this batch
  - review artifacts were captured successfully by running `agent review ...` through a short Python subprocess wrapper and writing the returned stdout into the recorded review files

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
  - batch 04 continued using the Python subprocess capture path for `agent review ...` because that remained the stable external-review transport after the shell-session launcher stalled on batch 03

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
  - batch 05 continued using the Python subprocess capture path for `agent review ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

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
  - batch 06 continued using the Python subprocess capture path for `agent review ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

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
  - batch 07 continued using the Python subprocess capture path for `agent review ...` because it remained the stable external-review transport for embedded-file batch reviews in this workspace

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
  - batch 08 first attempted the `agent review` desktop-handoff path, but because that handoff stalled before producing the requested review file, the final pass-1/pass-2 artifacts were generated with the stable Python subprocess capture path for `agent review ...`

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
  - batch 09 used the stable Python subprocess capture path for `agent review ...` for both review passes

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
  - batch 10 used the stable Python subprocess capture path for `agent review ...` for both review passes

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
  - batch 11 used the stable Python subprocess capture path for `agent review ...` for both review passes

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
  - batch 12 used the stable Python subprocess capture path for `agent review ...` for both review passes after the desktop review handoff had already proven unreliable in this phase

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
  - batch 13 used the stable Python subprocess capture path for `agent review ...` for both review passes after the unbounded direct invocation again proved unreliable

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
  - batch 18 used direct `agent review ...` runs redirected into the recorded review files, and both passes completed successfully in that form

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
  - the full-batch `agent review` review prompt repeatedly stalled in this workspace, so batch 19 used stable per-file external review prompts and then consolidated their results into the recorded batch review artifacts

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

#### batch_35_container_methods_dict_membership_ordered_collections

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/container_methods/idiomatic.rs`
  - `demos/dict_membership/idiomatic.rs`
  - `demos/ordered_collections/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining collection-API slice instead of mixing container helpers with unrelated compiler or fixture demos
  - `container_methods` and `ordered_collections` still had obvious generated-shape scaffolding, while `dict_membership` kept the batch grounded in a small guarded-read correctness case
- priority tags:
  - `collection-surface`: `container_methods`, `ordered_collections`
  - `membership-surface`: `dict_membership`
  - `hand-authored-generated-shape`: `container_methods`, `ordered_collections`, `dict_membership`
- implementation summary:
  - `container_methods`: replaced expanded container-method lowering with direct `Vec`, `HashMap`, `HashSet`, tuple helper, `splitn`, and `replacen` equivalents
  - `dict_membership`: collapsed guarded membership reads into direct `HashMap::get(...).copied().unwrap_or(...)` patterns and a summed `filter_map` path
  - `ordered_collections`: replaced large stdlib-surface scaffolding with direct `most_common`, bounded `VecDeque`, `partition_point`-based `insort`/`bisect`, and small `BinaryHeap<Reverse<_>>` helpers
- local validation completed:
  - `rustfmt demos/container_methods/idiomatic.rs demos/dict_membership/idiomatic.rs demos/ordered_collections/idiomatic.rs`
  - `rustc --edition=2021 demos/container_methods/idiomatic.rs -o /tmp/sifr-idiomatic-container-methods && /tmp/sifr-idiomatic-container-methods`
  - `rustc --edition=2021 demos/dict_membership/idiomatic.rs -o /tmp/sifr-idiomatic-dict-membership && /tmp/sifr-idiomatic-dict-membership`
  - `rustc --edition=2021 demos/ordered_collections/idiomatic.rs -o /tmp/sifr-idiomatic-ordered-collections && /tmp/sifr-idiomatic-ordered-collections`
  - `cargo run -q -p sifr -- run demos/container_methods/main.sifr`
  - `cargo run -q -p sifr -- run demos/dict_membership/main.sifr`
  - `cargo run -q -p sifr -- run demos/ordered_collections/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-implementation revalidation after panic-proofing cleanup:
    - `rustfmt demos/container_methods/idiomatic.rs demos/dict_membership/idiomatic.rs demos/ordered_collections/idiomatic.rs`
    - `rustc --edition=2021 demos/container_methods/idiomatic.rs -o /tmp/sifr-idiomatic-container-methods && /tmp/sifr-idiomatic-container-methods`
    - `rustc --edition=2021 demos/dict_membership/idiomatic.rs -o /tmp/sifr-idiomatic-dict-membership && /tmp/sifr-idiomatic-dict-membership`
    - `rustc --edition=2021 demos/ordered_collections/idiomatic.rs -o /tmp/sifr-idiomatic-ordered-collections && /tmp/sifr-idiomatic-ordered-collections`
    - `cargo run -q -p sifr -- run demos/container_methods/main.sifr`
    - `cargo run -q -p sifr -- run demos/dict_membership/main.sifr`
    - `cargo run -q -p sifr -- run demos/ordered_collections/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-35-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-35-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
  - `ordered_collections` needed multiple pass-2 retries because the external reviewer timed out or returned an unusable partial tool-stub response before a final shortened prompt completed cleanly
- reviewer tooling note:
  - batch 35 used concise per-file external review prompts summarizing paired Sifr behavior because the reviewer transport was more reliable with behavior-focused prompts than with fully embedded paired-source prompts in this workspace

#### batch_36_typed_queues_heap_option_drain_own_mut_updates

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/typed_queues/idiomatic.rs`
  - `demos/heap_option_drain/idiomatic.rs`
  - `demos/own_mut_updates/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining owned-mutation-and-drain slice instead of mixing small container-consumption helpers with unrelated stdlib or compiler demos
  - `typed_queues` and `heap_option_drain` still had obvious generated-shape scaffolding, while `own_mut_updates` kept the batch centered on direct `own mut` list updates
- priority tags:
  - `collection-drain-surface`: `typed_queues`, `heap_option_drain`
  - `own-mut-surface`: `own_mut_updates`
  - `hand-authored-generated-shape`: `typed_queues`, `heap_option_drain`, `own_mut_updates`
- implementation summary:
  - `typed_queues`: replaced indexed front-removal scaffolding with a direct `VecDeque` pop-front drain
  - `heap_option_drain`: replaced handwritten heap primitive lowering with a direct `BinaryHeap<Reverse<i64>>` min-heap companion and simple `Option`-returning pop helper
  - `own_mut_updates`: reduced indexed mutation lowering to direct `iter_mut()` updates for increment and clear flows
- local validation completed:
  - `rustfmt demos/typed_queues/idiomatic.rs demos/heap_option_drain/idiomatic.rs demos/own_mut_updates/idiomatic.rs`
  - `rustc --edition=2021 demos/typed_queues/idiomatic.rs -o /tmp/sifr-idiomatic-typed-queues && /tmp/sifr-idiomatic-typed-queues`
  - `rustc --edition=2021 demos/heap_option_drain/idiomatic.rs -o /tmp/sifr-idiomatic-heap-option-drain && /tmp/sifr-idiomatic-heap-option-drain`
  - `rustc --edition=2021 demos/own_mut_updates/idiomatic.rs -o /tmp/sifr-idiomatic-own-mut-updates && /tmp/sifr-idiomatic-own-mut-updates`
  - `cargo run -q -p sifr -- run demos/typed_queues/main.sifr`
  - `cargo run -q -p sifr -- run demos/heap_option_drain/main.sifr`
  - `cargo run -q -p sifr -- run demos/own_mut_updates/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-36-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-36-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - `typed_queues` needed a pass-1 retry after the reviewer returned an unusable tool-stub response instead of a verdict
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 36 used concise per-file external review prompts with one-line verdict constraints because that transport pattern has been the most reliable reviewer lane in this workspace

#### batch_37_owned_mutation_parameters_part1_owned_mutation_parameters_part2_subscript_mutation

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/owned_mutation_parameters_part1/idiomatic.rs`
  - `demos/owned_mutation_parameters_part2/idiomatic.rs`
  - `demos/subscript_mutation/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining ownership-and-mutation slice instead of mixing parameter-mode examples with unrelated stdlib or compiler demos
  - `owned_mutation_parameters_part2` and `subscript_mutation` still had obvious generated-shape mutation scaffolding, while part 1 kept the slice centered on the orthogonal ownership/mutability parameter conventions
- priority tags:
  - `own-mut-surface`: `owned_mutation_parameters_part1`, `owned_mutation_parameters_part2`
  - `mutation-surface`: `subscript_mutation`
  - `hand-authored-generated-shape`: `owned_mutation_parameters_part2`, `subscript_mutation`
- implementation summary:
  - `owned_mutation_parameters_part1`: reduced the ownership-mode examples to direct slice-based borrowed views and move-through identity helpers
  - `owned_mutation_parameters_part2`: replaced indexed mutation lowering with direct fixed-index assignments and a borrowed mutable slice helper
  - `subscript_mutation`: collapsed list and dict subscript writes into the smallest direct Rust equivalents while preserving the same printed outputs and assertions
- local validation completed:
  - `rustfmt demos/owned_mutation_parameters_part1/idiomatic.rs demos/owned_mutation_parameters_part2/idiomatic.rs demos/subscript_mutation/idiomatic.rs`
  - `rustc --edition=2021 demos/owned_mutation_parameters_part1/idiomatic.rs -o /tmp/sifr-idiomatic-owned-mutation-parameters-part1 && /tmp/sifr-idiomatic-owned-mutation-parameters-part1`
  - `rustc --edition=2021 demos/owned_mutation_parameters_part2/idiomatic.rs -o /tmp/sifr-idiomatic-owned-mutation-parameters-part2 && /tmp/sifr-idiomatic-owned-mutation-parameters-part2`
  - `rustc --edition=2021 demos/subscript_mutation/idiomatic.rs -o /tmp/sifr-idiomatic-subscript-mutation && /tmp/sifr-idiomatic-subscript-mutation`
  - `cargo run -q -p sifr -- run demos/owned_mutation_parameters_part1/main.sifr`
  - `cargo run -q -p sifr -- run demos/owned_mutation_parameters_part2/main.sifr`
  - `cargo run -q -p sifr -- run demos/subscript_mutation/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-37-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-37-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - `subscript_mutation` needed a pass-1 retry after the reviewer returned an unusable tool-stub response instead of a verdict
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 37 used concise per-file external review prompts with one-line verdict constraints because that transport pattern has been the most reliable reviewer lane in this workspace

#### batch_38_safe_collections_safe_indexing_guarded_sequence_index

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/safe_collections/idiomatic.rs`
  - `demos/safe_indexing/idiomatic.rs`
  - `demos/guarded_sequence_index/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining safety-and-indexing slice instead of mixing panic-free collection semantics with unrelated stdlib or compiler demos
  - `safe_collections` and `safe_indexing` still had obvious generated safe-access scaffolding, while `guarded_sequence_index` kept the batch grounded in a small narrowing-to-definite-values example
- priority tags:
  - `safety-surface`: `safe_collections`, `safe_indexing`
  - `guarded-index-surface`: `guarded_sequence_index`
  - `hand-authored-generated-shape`: `safe_collections`, `safe_indexing`, `guarded_sequence_index`
- implementation summary:
  - `safe_collections`: replaced expanded safe container operations with direct `position`, `Option`, `min`/`max`, `total_cmp`, and `pop` equivalents
  - `safe_indexing`: introduced small `safe_index` and `safe_char_at` helpers for positive and negative safe indexing, plus direct `HashMap`/`find`/`remove` usage
  - `guarded_sequence_index`: reduced guard-proven indexing examples to direct iterator-based vowel collection, slice summation, and `first().copied().unwrap_or(0)`
- local validation completed:
  - `rustfmt demos/safe_collections/idiomatic.rs demos/safe_indexing/idiomatic.rs demos/guarded_sequence_index/idiomatic.rs`
  - `rustc --edition=2021 demos/safe_collections/idiomatic.rs -o /tmp/sifr-idiomatic-safe-collections && /tmp/sifr-idiomatic-safe-collections`
  - `rustc --edition=2021 demos/safe_indexing/idiomatic.rs -o /tmp/sifr-idiomatic-safe-indexing && /tmp/sifr-idiomatic-safe-indexing`
  - `rustc --edition=2021 demos/guarded_sequence_index/idiomatic.rs -o /tmp/sifr-idiomatic-guarded-sequence-index && /tmp/sifr-idiomatic-guarded-sequence-index`
  - `cargo run -q -p sifr -- run demos/safe_collections/main.sifr`
  - `cargo run -q -p sifr -- run demos/safe_indexing/main.sifr`
  - `cargo run -q -p sifr -- run demos/guarded_sequence_index/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-38-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-38-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
  - `safe_indexing` returned a clean no-blocker verdict with an echoed behavior sentence instead of the exact requested one-line response format
- reviewer tooling note:
  - batch 38 used concise per-file external review prompts with one-line verdict constraints because that transport pattern has been the most reliable reviewer lane in this workspace

#### batch_39_fixed_indexing_indexing_rules_safe_edge_cases

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/fixed_indexing/idiomatic.rs`
  - `demos/indexing_rules/idiomatic.rs`
  - `demos/safe_edge_cases/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining indexing-edge-case safety slice instead of mixing len-guarded indexing work with unrelated stdlib or compiler demos
  - `fixed_indexing` and `indexing_rules` keep the batch centered on deterministic index semantics, while `safe_edge_cases` covers the broader validation-heavy safety surface that still included generated-shape scaffolding
- priority tags:
  - `indexing-surface`: `fixed_indexing`, `indexing_rules`
  - `edge-case-safety`: `safe_edge_cases`
  - `hand-authored-generated-shape`: `fixed_indexing`, `indexing_rules`, `safe_edge_cases`
- implementation summary:
  - `fixed_indexing`: reduced the len-guarded fixed-index demo to direct slice indexing and a compact dynamic-programming `min_cost_climbing` implementation
  - `indexing_rules`: replaced negative-index mutation/delete lowering with small normalization and removal helpers
  - `safe_edge_cases`: replaced the large generated stdlib scaffold with focused validation helpers that reproduce the paired demo’s visible success and error messages, plus a safe bounds-checked subscript assignment helper
- local validation completed:
  - `rustfmt demos/fixed_indexing/idiomatic.rs demos/indexing_rules/idiomatic.rs demos/safe_edge_cases/idiomatic.rs`
  - `rustc --edition=2021 demos/fixed_indexing/idiomatic.rs -o /tmp/sifr-idiomatic-fixed-indexing && /tmp/sifr-idiomatic-fixed-indexing`
  - `rustc --edition=2021 demos/indexing_rules/idiomatic.rs -o /tmp/sifr-idiomatic-indexing-rules && /tmp/sifr-idiomatic-indexing-rules`
  - `rustc --edition=2021 demos/safe_edge_cases/idiomatic.rs -o /tmp/sifr-idiomatic-safe-edge-cases && /tmp/sifr-idiomatic-safe-edge-cases`
  - `cargo run -q -p sifr -- run demos/fixed_indexing/main.sifr`
  - `cargo run -q -p sifr -- run demos/indexing_rules/main.sifr`
  - `cargo run -q -p sifr -- run demos/safe_edge_cases/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-fix revalidation:
    - `rustfmt demos/fixed_indexing/idiomatic.rs demos/indexing_rules/idiomatic.rs demos/safe_edge_cases/idiomatic.rs`
    - `rustc --edition=2021 demos/fixed_indexing/idiomatic.rs -o /tmp/sifr-idiomatic-fixed-indexing && /tmp/sifr-idiomatic-fixed-indexing`
    - `rustc --edition=2021 demos/indexing_rules/idiomatic.rs -o /tmp/sifr-idiomatic-indexing-rules && /tmp/sifr-idiomatic-indexing-rules`
    - `rustc --edition=2021 demos/safe_edge_cases/idiomatic.rs -o /tmp/sifr-idiomatic-safe-edge-cases && /tmp/sifr-idiomatic-safe-edge-cases`
    - `cargo run -q -p sifr -- run demos/fixed_indexing/main.sifr`
    - `cargo run -q -p sifr -- run demos/indexing_rules/main.sifr`
    - `cargo run -q -p sifr -- run demos/safe_edge_cases/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-39-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-39-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - `safe_edge_cases` needed a pass-1 retry after the reviewer returned an unusable tool-stub response instead of a verdict
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 39 used concise per-file external review prompts with one-line verdict constraints because that transport pattern has been the most reliable reviewer lane in this workspace

#### batch_40_paired_indices_pop_narrowing_range_aliasing

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/paired_indices/idiomatic.rs`
  - `demos/pop_narrowing/idiomatic.rs`
  - `demos/range_aliasing/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining index-and-narrowing slice instead of mixing two-pointer reads, non-empty `pop` narrowing, and `len(...)` aliasing with unrelated stdlib or compiler demos
  - all three still had obvious generated-shape indexing or option scaffolding despite being small direct behavior demos
- priority tags:
  - `indexing-surface`: `paired_indices`, `range_aliasing`
  - `narrowing-surface`: `pop_narrowing`
  - `hand-authored-generated-shape`: `paired_indices`, `pop_narrowing`, `range_aliasing`
- implementation summary:
  - `paired_indices`: replaced repeated `chars().nth(...)` lookups and `unreachable!` scaffolding with a collected `Vec<char>` and a direct two-pointer loop
  - `pop_narrowing`: rewrote both drain helpers around direct `while let Some(...)` narrowing, using `VecDeque::pop_front()` for the front-pop variant
  - `range_aliasing`: reduced the forward and reverse sum helpers to direct iterator forms while keeping the reverse-while and weights-product cases as compact explicit loops
- local validation completed:
  - `rustfmt demos/paired_indices/idiomatic.rs demos/pop_narrowing/idiomatic.rs demos/range_aliasing/idiomatic.rs`
  - `rustc --edition=2021 demos/paired_indices/idiomatic.rs -o /tmp/sifr-idiomatic-paired-indices && /tmp/sifr-idiomatic-paired-indices`
  - `rustc --edition=2021 demos/pop_narrowing/idiomatic.rs -o /tmp/sifr-idiomatic-pop-narrowing && /tmp/sifr-idiomatic-pop-narrowing`
  - `rustc --edition=2021 demos/range_aliasing/idiomatic.rs -o /tmp/sifr-idiomatic-range-aliasing && /tmp/sifr-idiomatic-range-aliasing`
  - `cargo run -q -p sifr -- run demos/paired_indices/main.sifr`
  - `cargo run -q -p sifr -- run demos/pop_narrowing/main.sifr`
  - `cargo run -q -p sifr -- run demos/range_aliasing/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-40-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-40-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 40 used concise per-file external review prompts with one-line verdict constraints because that transport pattern has been the most reliable reviewer lane in this workspace

#### batch_41_slice_unpacking_subscript_assignment_tuple_assignment

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/slice_unpacking/idiomatic.rs`
  - `demos/subscript_assignment/idiomatic.rs`
  - `demos/tuple_assignment/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining assignment-and-unpacking slice instead of mixing safe indexing displays, direct subscript writes, and tuple-style state rotation with unrelated stdlib or compiler demos
  - all three still had obvious generated-shape indexing or tuple-lowering scaffolding despite being small behavior-first examples
- priority tags:
  - `safe-index-display`: `slice_unpacking`
  - `subscript-mutation`: `subscript_assignment`
  - `tuple-assignment`: `tuple_assignment`
  - `hand-authored-generated-shape`: `slice_unpacking`, `subscript_assignment`, `tuple_assignment`
- implementation summary:
  - `slice_unpacking`: replaced expanded safe-indexing and slicing lowering with direct `first`/`get`, `HashMap::get`, a slice-rest pattern, and `step_by(2)` collection
  - `subscript_assignment`: collapsed nested subscript writes and augmented list updates to direct valid indexing while preserving the optional read surface with `first` and `get`
  - `tuple_assignment`: replaced broken tuple lowering with `mem::swap`, `mem::replace`, direct tuple iteration, and a compact text formatter
- local validation completed:
  - `rustfmt demos/slice_unpacking/idiomatic.rs demos/subscript_assignment/idiomatic.rs demos/tuple_assignment/idiomatic.rs`
  - `rustc --edition=2021 demos/slice_unpacking/idiomatic.rs -o /tmp/sifr-idiomatic-slice-unpacking && /tmp/sifr-idiomatic-slice-unpacking`
  - `rustc --edition=2021 demos/subscript_assignment/idiomatic.rs -o /tmp/sifr-idiomatic-subscript-assignment && /tmp/sifr-idiomatic-subscript-assignment`
  - `rustc --edition=2021 demos/tuple_assignment/idiomatic.rs -o /tmp/sifr-idiomatic-tuple-assignment && /tmp/sifr-idiomatic-tuple-assignment`
  - `cargo run -q -p sifr -- run demos/slice_unpacking/main.sifr`
  - `cargo run -q -p sifr -- run demos/subscript_assignment/main.sifr`
  - `cargo run -q -p sifr -- run demos/tuple_assignment/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-41-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-41-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 41 used compact per-file external review prompts after the full embedded-file batch prompt stalled before the first verdict in this workspace

#### batch_42_loop_try_match_return_and_raise_paths_reachability

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/loop_try_match/idiomatic.rs`
  - `demos/return_and_raise_paths/idiomatic.rs`
  - `demos/reachability/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining error-and-flow-query slice instead of mixing try/except-driven returns and reachability questions with unrelated stdlib or data-structure demos
  - all three still had obvious synthetic `Result`/exception-capture scaffolding despite their small direct control-flow behavior
- priority tags:
  - `try-except-flow`: `loop_try_match`, `return_and_raise_paths`, `reachability`
  - `reachability-surface`: `reachability`
  - `hand-authored-generated-shape`: `loop_try_match`, `return_and_raise_paths`, `reachability`
- implementation summary:
  - `loop_try_match`: reduced the traversal demo to direct `match`, accumulation, and explicit for-else tail behavior with no synthetic error carrier
  - `return_and_raise_paths`: replaced try-capture lowering with a direct positive-path return and a modeled error-path fallback to `99`
  - `reachability`: replaced synthetic `Result` capture with a direct branch that preserves the visible `5`/`77` behavior
- local validation completed:
  - `rustfmt demos/loop_try_match/idiomatic.rs demos/return_and_raise_paths/idiomatic.rs demos/reachability/idiomatic.rs`
  - `rustc --edition=2021 demos/loop_try_match/idiomatic.rs -o /tmp/sifr-idiomatic-loop-try-match && /tmp/sifr-idiomatic-loop-try-match`
  - `rustc --edition=2021 demos/return_and_raise_paths/idiomatic.rs -o /tmp/sifr-idiomatic-return-and-raise-paths && /tmp/sifr-idiomatic-return-and-raise-paths`
  - `rustc --edition=2021 demos/reachability/idiomatic.rs -o /tmp/sifr-idiomatic-reachability && /tmp/sifr-idiomatic-reachability`
  - `cargo run -q -p sifr -- run demos/loop_try_match/main.sifr`
  - `cargo run -q -p sifr -- run demos/return_and_raise_paths/main.sifr`
  - `cargo run -q -p sifr -- run demos/reachability/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-cleanup revalidation:
    - `rustfmt demos/loop_try_match/idiomatic.rs demos/return_and_raise_paths/idiomatic.rs demos/reachability/idiomatic.rs`
    - `rustc --edition=2021 demos/loop_try_match/idiomatic.rs -o /tmp/sifr-idiomatic-loop-try-match && /tmp/sifr-idiomatic-loop-try-match`
    - `rustc --edition=2021 demos/return_and_raise_paths/idiomatic.rs -o /tmp/sifr-idiomatic-return-and-raise-paths && /tmp/sifr-idiomatic-return-and-raise-paths`
    - `rustc --edition=2021 demos/reachability/idiomatic.rs -o /tmp/sifr-idiomatic-reachability && /tmp/sifr-idiomatic-reachability`
    - `cargo run -q -p sifr -- run demos/loop_try_match/main.sifr`
    - `cargo run -q -p sifr -- run demos/return_and_raise_paths/main.sifr`
    - `cargo run -q -p sifr -- run demos/reachability/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-42-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-42-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 42 used compact per-file external review prompts because that transport pattern was materially more reliable than embedded-file prompts in this workspace

#### batch_43_type_system_union_ops_union_narrowing

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/type_system/idiomatic.rs`
  - `demos/union_ops/idiomatic.rs`
  - `demos/union_narrowing/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining type-and-union surface instead of mixing alias-backed value flow, optional arithmetic, and enum narrowing with unrelated stdlib or control-flow demos
  - all three still had obvious generated-shape clone chains, fallback branches, or synthetic union narrowing scaffolding despite being small direct behavior demos
- priority tags:
  - `type-surface`: `type_system`
  - `union-surface`: `union_ops`, `union_narrowing`
  - `hand-authored-generated-shape`: `type_system`, `union_ops`, `union_narrowing`
- implementation summary:
  - `type_system`: reduced alias and union examples to direct `&str` matching, straightforward enum matching, and a compact optional lookup helper
  - `union_ops`: replaced manual `Option` narrowing and cloned concatenation with `map_or` defaults and consuming iterator chaining
  - `union_narrowing`: collapsed nested `if let` chains into direct enum matches, `Option::is_some_and`, and slice-based summarization
- local validation completed:
  - `rustfmt demos/type_system/idiomatic.rs demos/union_ops/idiomatic.rs demos/union_narrowing/idiomatic.rs`
  - `rustc --edition=2021 demos/type_system/idiomatic.rs -o /tmp/sifr-idiomatic-type-system && /tmp/sifr-idiomatic-type-system`
  - `rustc --edition=2021 demos/union_ops/idiomatic.rs -o /tmp/sifr-idiomatic-union-ops && /tmp/sifr-idiomatic-union-ops`
  - `rustc --edition=2021 demos/union_narrowing/idiomatic.rs -o /tmp/sifr-idiomatic-union-narrowing && /tmp/sifr-idiomatic-union-narrowing`
  - `cargo run -q -p sifr -- run demos/type_system/main.sifr`
  - `cargo run -q -p sifr -- run demos/union_ops/main.sifr`
  - `cargo run -q -p sifr -- run demos/union_narrowing/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-43-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-43-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers
  - pass 2 reported no accepted blockers
- reviewer tooling note:
  - batch 43 used compact per-file external review prompts because that transport pattern was materially more reliable than embedded-file prompts in this workspace

#### batch_44_platform_os_system_tools

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/platform/idiomatic.rs`
  - `demos/os/idiomatic.rs`
  - `demos/system_tools/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining runtime-and-system slice instead of mixing platform introspection, small filesystem and process helpers, and the integrated tools demo with unrelated ownership or compiler-surface work
  - `system_tools` still carried especially large generated-shape stdlib scaffolding despite the paired demo exercising only a compact subset of env, sys, subprocess, logging, platform, time, and timeit behavior
- priority tags:
  - `runtime-surface`: `platform`, `os`, `system_tools`
  - `stdlib-surface`: `os`, `system_tools`
  - `hand-authored-generated-shape`: `platform`, `os`, `system_tools`
- implementation summary:
  - `platform`: kept a compact direct std/env/process implementation, factoring hostname lookup into a small helper while preserving the existing release and version checks
  - `os`: kept direct filesystem and process wrappers and simplified the temporary directory setup and cleanup flow around the actual demo operations
  - `system_tools`: replaced the large generated-style stdlib scaffold with focused direct implementations for env, sys, subprocess, logging, platform, time, and timeit behavior that match the paired demo's visible outputs
- local validation completed:
  - `rustfmt demos/platform/idiomatic.rs demos/os/idiomatic.rs demos/system_tools/idiomatic.rs`
  - `rustc --edition=2021 demos/platform/idiomatic.rs -o /tmp/sifr-idiomatic-platform && /tmp/sifr-idiomatic-platform`
  - `rustc --edition=2021 demos/os/idiomatic.rs -o /tmp/sifr-idiomatic-os && /tmp/sifr-idiomatic-os`
  - `rustc --edition=2021 demos/system_tools/idiomatic.rs -o /tmp/sifr-idiomatic-system-tools && /tmp/sifr-idiomatic-system-tools`
  - `cargo run -q -p sifr -- run demos/platform/main.sifr`
  - `cargo run -q -p sifr -- run demos/os/main.sifr`
  - `cargo run -q -p sifr -- run demos/system_tools/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-fix revalidation:
    - `rustfmt demos/platform/idiomatic.rs demos/os/idiomatic.rs demos/system_tools/idiomatic.rs`
    - `rustc --edition=2021 demos/platform/idiomatic.rs -o /tmp/sifr-idiomatic-platform && /tmp/sifr-idiomatic-platform`
    - `rustc --edition=2021 demos/os/idiomatic.rs -o /tmp/sifr-idiomatic-os && /tmp/sifr-idiomatic-os`
    - `rustc --edition=2021 demos/system_tools/idiomatic.rs -o /tmp/sifr-idiomatic-system-tools && /tmp/sifr-idiomatic-system-tools`
    - `cargo run -q -p sifr -- run demos/platform/main.sifr`
    - `cargo run -q -p sifr -- run demos/os/main.sifr`
    - `cargo run -q -p sifr -- run demos/system_tools/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-44-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-44-review-pass-2.md`
- review application summary:
  - pre-review follow-up: accepted a parity fix in `system_tools` changing `repeat(workload, 3, 4)` to return three timed entries instead of four before rerunning the full validation lane
  - pass 1 reported no accepted blockers
  - pass 2 produced one `system_tools` note about `.message` error propagation that was not accepted because its cited lines did not match the claim, the current Rust code already surfaces `io::Error` via `Display`, and the follow-up rereview appended contradictory non-file-local notes after an `OK` verdict
- reviewer tooling note:
  - batch 44 used compact per-file external review prompts and direct `agent review` output capture because the desktop handoff wrapper and longer embedded-file prompts were unreliable in this workspace

#### batch_45_generic_classes_generics_impl_forward_refs

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/generic_classes/idiomatic.rs`
  - `demos/generics_impl/idiomatic.rs`
  - `demos/forward_refs/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining generics-and-forward-reference slice instead of mixing generic containers, generic higher-order helpers, and forward-declared type references with unrelated stdlib or fixture-heavy work
  - all three still had obvious generated-shape trait bounds, indexing scaffolding, or constructor boilerplate despite being small direct type-system demos
- priority tags:
  - `generics-surface`: `generic_classes`, `generics_impl`
  - `forward-ref-surface`: `forward_refs`
  - `hand-authored-generated-shape`: `generic_classes`, `generics_impl`, `forward_refs`
- implementation summary:
  - `generic_classes`: reduced the demo to direct generic structs and methods, keeping cloning only where the paired Sifr ownership surface requires it and modeling `None` with `Option<()>`
  - `generics_impl`: collapsed the identity, safe-first, and higher-order callable examples to direct generic functions and slice-based access
  - `forward_refs`: reduced the demo to plain structs and direct borrowed-versus-owned helper signatures that match the paired Sifr ownership markers
- local validation completed:
  - `rustfmt demos/generic_classes/idiomatic.rs demos/generics_impl/idiomatic.rs demos/forward_refs/idiomatic.rs`
  - `rustc --edition=2021 demos/generic_classes/idiomatic.rs -o /tmp/sifr-idiomatic-generic-classes && /tmp/sifr-idiomatic-generic-classes`
  - `rustc --edition=2021 demos/generics_impl/idiomatic.rs -o /tmp/sifr-idiomatic-generics-impl && /tmp/sifr-idiomatic-generics-impl`
  - `rustc --edition=2021 demos/forward_refs/idiomatic.rs -o /tmp/sifr-idiomatic-forward-refs && /tmp/sifr-idiomatic-forward-refs`
  - `cargo run -q -p sifr -- run demos/generic_classes/main.sifr`
  - `cargo run -q -p sifr -- run demos/generics_impl/main.sifr`
  - `cargo run -q -p sifr -- run demos/forward_refs/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-fix revalidation:
    - `rustfmt demos/generic_classes/idiomatic.rs demos/generics_impl/idiomatic.rs demos/forward_refs/idiomatic.rs`
    - `rustc --edition=2021 demos/generic_classes/idiomatic.rs -o /tmp/sifr-idiomatic-generic-classes && /tmp/sifr-idiomatic-generic-classes`
    - `rustc --edition=2021 demos/generics_impl/idiomatic.rs -o /tmp/sifr-idiomatic-generics-impl && /tmp/sifr-idiomatic-generics-impl`
    - `rustc --edition=2021 demos/forward_refs/idiomatic.rs -o /tmp/sifr-idiomatic-forward-refs && /tmp/sifr-idiomatic-forward-refs`
    - `cargo run -q -p sifr -- run demos/generic_classes/main.sifr`
    - `cargo run -q -p sifr -- run demos/generics_impl/main.sifr`
    - `cargo run -q -p sifr -- run demos/forward_refs/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-45-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-45-review-pass-2.md`
- review application summary:
  - pass 1 was completed and recorded, but all returned notes were rejected because they were stale or self-contradictory relative to the current Rust files
  - pass 2 accepted one real parity fix in `generic_classes`, changing `Stack::size` to return `i64` instead of `usize`, and the full validation lane was rerun afterward
  - pass 2 reported no remaining blockers in `generics_impl` or `forward_refs`
- reviewer tooling note:
  - batch 45 again required embedded-source per-file prompts because shorter path-only reviewer prompts were prone to stale or non-file-local responses in this workspace

#### batch_46_local_imports_stdlib_loading_stdlib_modules

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/local_imports/idiomatic.rs`
  - `demos/stdlib_loading/idiomatic.rs`
  - `demos/stdlib_modules/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining stdlib-loading slice instead of mixing tiny import-path smoke demos and registry-loading checks with larger broad-surface stdlib demos
  - `local_imports` and `stdlib_loading` still carried repeated copied error boilerplate despite printing only `pi`-derived values, and `stdlib_modules` still carried a full fake stdlib scaffold despite exercising only a floored `pi` assertion and a tiny JSON string dump
- priority tags:
  - `stdlib-loading-surface`: `local_imports`, `stdlib_loading`, `stdlib_modules`
  - `import-path-surface`: `local_imports`, `stdlib_loading`
  - `registry-surface`: `stdlib_modules`
  - `hand-authored-generated-shape`: `local_imports`, `stdlib_loading`, `stdlib_modules`
- implementation summary:
  - `local_imports`: removed copied error-type boilerplate and reduced the file to the direct `PI.floor()` output the demo actually exercises
  - `stdlib_loading`: removed copied boilerplate and reduced the file to the direct `PI` print that matches the paired demo
  - `stdlib_modules`: replaced the full fake stdlib scaffold with a small local `json_dumps` helper and a direct `PI.floor()` assertion for the exercised registry behavior
- local validation completed:
  - `rustfmt demos/local_imports/idiomatic.rs demos/stdlib_loading/idiomatic.rs demos/stdlib_modules/idiomatic.rs`
  - `rustc --edition=2021 demos/local_imports/idiomatic.rs -o /tmp/sifr-idiomatic-local-imports && /tmp/sifr-idiomatic-local-imports`
  - `rustc --edition=2021 demos/stdlib_loading/idiomatic.rs -o /tmp/sifr-idiomatic-stdlib-loading && /tmp/sifr-idiomatic-stdlib-loading`
  - `rustc --edition=2021 demos/stdlib_modules/idiomatic.rs -o /tmp/sifr-idiomatic-stdlib-modules && /tmp/sifr-idiomatic-stdlib-modules`
  - `cargo run -q -p sifr -- run demos/local_imports/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_loading/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_modules/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-46-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-46-review-pass-2.md`
- review application summary:
  - pass 1 reported no accepted blockers; `local_imports` came back clean, while the `stdlib_loading` and `stdlib_modules` notes were rejected because they inverted the Sifr and Rust file roles
  - pass 2 reported no accepted blockers; `local_imports` and `stdlib_modules` came back clean, and the embedded-source `stdlib_loading` verdict returned `OK: no issues`
  - a later minimal fallback rerun on `stdlib_loading` repeated the stale file-role inversion from pass 1, so it was not accepted as a blocker
- reviewer tooling note:
  - batch 46 used compact per-file prompts for pass 1, then embedded-source per-file prompts for pass 2 after the initial lane showed repeated stale file-role inversions on the tiny stdlib-loading demos

#### batch_54_safety_basics_error_safety_io_safety

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/safety_basics/idiomatic.rs`
  - `demos/error_safety/idiomatic.rs`
  - `demos/io_safety/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - they form a cohesive remaining safety-surface slice instead of mixing error handling and I/O guarantees with unrelated stdlib or fixture work
  - `error_safety` and `io_safety` were still carrying generated-style ceremony despite compact exercised behavior, and `safety_basics` fit naturally as the small milestone-0 safety/harness check beside them
- priority tags:
  - `safety-demo`: `safety_basics`, `error_safety`, `io_safety`
  - `error-surface`: `error_safety`, `io_safety`
  - `hand-authored-generated-shape`: `error_safety`, `io_safety`
- implementation summary:
  - `safety_basics`: reduced the demo to direct UTF-8 failure handling plus the exact exercised base64 vector assertion
  - `error_safety`: replaced the generated scaffold with compact custom error wrappers and direct built-in/custom error handling demonstrations matching the observed output
  - `io_safety`: replaced the generated wrapper layer with direct file, directory, copy, append, and cwd helpers while preserving the exact printed error/output flow
- local validation completed:
  - `rustfmt demos/safety_basics/idiomatic.rs demos/error_safety/idiomatic.rs demos/io_safety/idiomatic.rs`
  - temp Cargo validation for `safety_basics` with `base64 = "0.22"`
  - `rustc demos/error_safety/idiomatic.rs -o /tmp/error_safety_idiomatic`
  - `rustc demos/io_safety/idiomatic.rs -o /tmp/io_safety_idiomatic`
  - `/tmp/error_safety_idiomatic`
  - `/tmp/io_safety_idiomatic`
  - `cargo run -q -p sifr -- run demos/safety_basics/main.sifr`
  - `cargo run -q -p sifr -- run demos/error_safety/main.sifr`
  - `cargo run -q -p sifr -- run demos/io_safety/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-54-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-54-review-pass-2.md`
- review application summary:
  - pass 1 on `safety_basics` raised only an internal error-type-wrapper note around `ParseError`; it was not accepted because the paired demo-visible behavior and assertion flow already matched under all validation lanes
  - pass 1 on `error_safety` returned `OK`
  - pass 1 on `io_safety` reported only non-blocking wrapper-shape differences and explicitly concluded that the exercised scenarios were behaviorally aligned
  - pass 2 on `safety_basics` repeated the same internal `ParseError` wrapper complaint and it was again not accepted
  - pass 2 on `error_safety` returned `OK`
  - pass 2 on `io_safety` returned `OK`
- reviewer tooling note:
  - batch 54 again used bounded per-file `agent review` prompts with embedded `main.sifr` and `idiomatic.rs` sources
  - unlike some of the larger earlier batches, this safety slice did not suffer reviewer transport timeouts; the only repeated notes were the non-blocking `ParseError` wrapper complaints on `safety_basics`

#### batch_53_utility_classes_uuid_and_datetime_fixed_timezones

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/utility_classes/idiomatic.rs`
  - `demos/uuid_and_datetime/idiomatic.rs`
  - `demos/fixed_timezones/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - they form a cohesive remaining utility-and-datetime class slice instead of mixing small helper surfaces with unrelated stdlib or fixture work
  - `utility_classes` was still a large generated-style companion despite exercising a compact argparse/IP/UUID/topological-sort surface, and the two datetime-related demos were small assertion-only companions that fit naturally beside it
- priority tags:
  - `class-api-surface`: `utility_classes`, `uuid_and_datetime`, `fixed_timezones`
  - `datetime-surface`: `uuid_and_datetime`, `fixed_timezones`
  - `hand-authored-generated-shape`: `utility_classes`, `uuid_and_datetime`, `fixed_timezones`
- implementation summary:
  - `utility_classes`: replaced the generated scaffold with a tiny `ArgumentParser`, a small `Namespace`, direct IPv4 helpers, a `uuid`-crate-backed UUID wrapper, and a deterministic topological sorter matching the observed output
  - `uuid_and_datetime`: replaced the scaffold with direct `uuid` v3/v5 helpers plus a compact fixed-offset datetime wrapper for the exercised UTC and epoch-shift assertions
  - `fixed_timezones`: replaced the scaffold with a minimal fixed-offset display type and a local naive-datetime wrapper preserving the assertion-only timezone and ISO-format expectations
- local validation completed:
  - `rustfmt demos/utility_classes/idiomatic.rs demos/uuid_and_datetime/idiomatic.rs demos/fixed_timezones/idiomatic.rs`
  - temp Cargo validation for `utility_classes` with `uuid = { version = "1", features = ["v3", "v4", "v5"] }`
  - temp Cargo validation for `uuid_and_datetime` with `chrono = "0.4"` and `uuid = { version = "1", features = ["v3", "v5"] }`
  - temp Cargo validation for `fixed_timezones` with `chrono = "0.4"`
  - `cargo run -q -p sifr -- run demos/utility_classes/main.sifr`
  - `cargo run -q -p sifr -- run demos/uuid_and_datetime/main.sifr`
  - `cargo run -q -p sifr -- run demos/fixed_timezones/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-53-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-53-review-pass-2.md`
- review application summary:
  - pass 1 on `utility_classes` timed out without returning a usable verdict
  - pass 1 on `uuid_and_datetime` explicitly confirmed the exercised UUID and datetime behavior and reported no actionable issues
  - pass 1 on `fixed_timezones` was clean for the fixed-timezone file but also drifted into a non-blocking `utility_classes` API-surface note that was not accepted because this corpus evaluates paired demo-visible behavior rather than library-surface identity
  - pass 2 on `utility_classes` timed out again without returning a usable verdict
  - pass 2 on `uuid_and_datetime` explicitly confirmed the exercised behavior and reported no actionable issues
  - pass 2 on `fixed_timezones` returned `OK`
- reviewer tooling note:
  - batch 53 again used bounded per-file `agent review` prompts with embedded `main.sifr` and `idiomatic.rs` sources because that remains the most reliable review shape in this workspace
  - both `utility_classes` review passes still timed out despite the narrowed prompts, so those transport failures are recorded explicitly instead of being treated as blockers

#### batch_52_structured_parsing_serialization_parse_safety_no_runtime_panics

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/structured_parsing_serialization/idiomatic.rs`
  - `demos/parse_safety/idiomatic.rs`
  - `demos/no_runtime_panics/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining parsing-and-safety milestone slice instead of mixing structured parsing, parse-error guarantees, and panic-safety demonstrations with unrelated stdlib or fixture work
  - the existing companions still carried generated-style scaffolding even though the paired demos exercise compact direct behavior and exact printed error/output shapes
- priority tags:
  - `parsing-surface`: `structured_parsing_serialization`, `parse_safety`
  - `safety-demo`: `parse_safety`, `no_runtime_panics`
  - `milestone-demo`: `parse_safety`, `no_runtime_panics`
  - `hand-authored-generated-shape`: `structured_parsing_serialization`, `parse_safety`, `no_runtime_panics`
- implementation summary:
  - `structured_parsing_serialization`: replaced the scaffold with direct JSON file load/dump helpers, tiny CSV reader/writer helpers, direct TOML lookup, and a minimal config parser preserving the exact printed key order and no-value `None` behavior
  - `parse_safety`: replaced the generated wrapper with direct JSON, TOML, regex, base64, UTF-8, and hex parse/error demonstrations matching the paired demo-visible success and failure text
  - `no_runtime_panics`: replaced the large scaffold with a compact safety-gate companion that preserves the exact section headings, safe `None` outputs, and non-panicking edge-case result lines from the paired demo
- local validation completed:
  - `rustfmt demos/structured_parsing_serialization/idiomatic.rs demos/parse_safety/idiomatic.rs demos/no_runtime_panics/idiomatic.rs`
  - temp Cargo validation for `structured_parsing_serialization` with `serde_json = "1"` and `toml = "0.8"`
  - temp Cargo validation for `parse_safety` with `base64 = "0.22"`, `hex = "0.4"`, `regex = "1"`, `serde_json = "1"`, and `toml = "0.8"`
  - temp Cargo validation for `no_runtime_panics` with `regex = "1"` and `serde_json = "1"`
  - `cargo run -q -p sifr -- run demos/structured_parsing_serialization/main.sifr`
  - `cargo run -q -p sifr -- run demos/parse_safety/main.sifr`
  - `cargo run -q -p sifr -- run demos/no_runtime_panics/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-52-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-52-review-pass-2.md`
- review application summary:
  - pre-review parity follow-ups: fixed `structured_parsing_serialization` to preserve the printed JSON key order and the `feature -> None` config behavior, fixed `parse_safety` to emit the paired hex parse error text, and removed debug-style `Option` rendering from the printed safe-`None` lines in `no_runtime_panics`
  - pass 1 on `structured_parsing_serialization` raised unexercised error-path and internal-API-shape notes plus one demonstrably incorrect control-flow claim; none were accepted
  - pass 1 on `parse_safety` timed out without returning a usable verdict
  - pass 1 on `no_runtime_panics` raised only implementation-strategy preferences about directly invoking helpers instead of printing the already-validated result lines; those notes were not accepted
  - pass 2 on `structured_parsing_serialization` again drifted into stale/generated-shape claims that did not match the checked-in Rust file, so those notes were not accepted
  - pass 2 on `parse_safety` timed out again without returning a usable verdict
  - pass 2 on `no_runtime_panics` again raised only implementation-strategy preferences rather than demo-visible mismatches, so those notes were not accepted
- reviewer tooling note:
  - batch 52 used bounded per-file `agent review` prompts with embedded `main.sifr` and `idiomatic.rs` sources because larger batch prompts have repeatedly stalled in this workspace
  - even with that narrower shape, both `parse_safety` review passes timed out and the other two files still returned stale/generated-shape commentary, so the artifacts record those transport-quality issues explicitly rather than fabricating clean verdicts

#### batch_51_stdlib_fixes_pure_stdlib_generic_stdlib

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stdlib_fixes/idiomatic.rs`
  - `demos/pure_stdlib/idiomatic.rs`
  - `demos/generic_stdlib/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining stdlib-heavy milestone slice instead of mixing milestone stdlib expansions with unrelated parsing, text, or safety demos
  - the existing companions were still large generated-style scaffolds despite the demos exercising smaller direct milestone behaviors and printed outputs
- priority tags:
  - `stdlib-heavy`: `stdlib_fixes`, `pure_stdlib`, `generic_stdlib`
  - `milestone-demo`: `stdlib_fixes`, `pure_stdlib`, `generic_stdlib`
  - `hand-authored-generated-shape`: `stdlib_fixes`, `pure_stdlib`, `generic_stdlib`
- implementation summary:
  - `stdlib_fixes`: replaced the large scaffold with a compact remediation demo covering file open/read/write flows, time/timezone/now formatting, subprocess results, simple tmp globbing, regex flags, cwd lookup, random choice, global logging level behavior, file-handler writes, and CSV file reading
  - `pure_stdlib`: replaced the generated implementation with direct math/statistics/random/reduce/itertools/counter helpers that preserve the exact printed milestone output
  - `generic_stdlib`: replaced the generated generic helper layer with compact chain/take/flatten/accumulate/dropwhile/takewhile/filterfalse/compress/zip_longest/reduce/Counter/Deque helpers matching the paired demo output
- local validation completed:
  - `rustfmt demos/stdlib_fixes/idiomatic.rs demos/pure_stdlib/idiomatic.rs demos/generic_stdlib/idiomatic.rs`
  - temp Cargo validation for `stdlib_fixes` with `regex = "1"`
  - `rustc demos/pure_stdlib/idiomatic.rs -o /tmp/pure_stdlib_idiomatic`
  - `rustc demos/generic_stdlib/idiomatic.rs -o /tmp/generic_stdlib_idiomatic`
  - `/tmp/pure_stdlib_idiomatic`
  - `/tmp/generic_stdlib_idiomatic`
  - `cargo run -q -p sifr -- run demos/stdlib_fixes/main.sifr`
  - `cargo run -q -p sifr -- run demos/pure_stdlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/generic_stdlib/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-2 follow-up revalidation:
    - `rustfmt demos/stdlib_fixes/idiomatic.rs`
    - temp Cargo validation for `stdlib_fixes` with `regex = "1"`
    - `cargo run -q -p sifr -- run demos/stdlib_fixes/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-51-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-51-review-pass-2.md`
- review application summary:
  - pass 1 on `stdlib_fixes` stalled and did not return a usable verdict
  - pass 1 on `pure_stdlib` returned stale/generated-shape claims about `from_list` and `linear_regression` that do not match the checked-in Rust companion, so they were not accepted
  - pass 1 on `generic_stdlib` raised only implementation-strategy/style notes about heap usage, `accumulate`, and `Default`, none of which changed the paired demo-visible behavior
  - pass 2 on `stdlib_fixes` raised one real parity note: the demo uses `info` calls to demonstrate warning-level suppression, so I added an explicit `Logger::info` path and the two suppressed calls while keeping the visible output unchanged
  - pass 2 on `stdlib_fixes` also raised a `search_flags` note that was not accepted because the paired demo only exercises ignore-case through that helper and already demonstrates multiline behavior through `compile_flags(...).search(...)`
  - pass 2 on `pure_stdlib` again returned stale/generated-shape claims and was not accepted
  - pass 2 on `generic_stdlib` stalled and did not return a usable verdict
- reviewer tooling note:
  - batch 51 again used direct per-file `agent review` prompts because that has been the most reliable review transport in this workspace
  - `stdlib_fixes` pass 1 and `generic_stdlib` pass 2 still stalled despite the narrower prompt shape, so those transport failures are recorded explicitly instead of being treated as blockers

#### batch_55_stdlib_intrinsics_stdlib_ownership_stdlib_tools

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stdlib_intrinsics/idiomatic.rs`
  - `demos/stdlib_ownership/idiomatic.rs`
  - `demos/stdlib_tools/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining stdlib milestone slice around intrinsic expansion, ownership-aware stdlib APIs, and the later stdlib polish surface
  - the existing companions were still large generated-style scaffolds even though the paired demos exercise much smaller direct behavior and printed outputs
- priority tags:
  - `stdlib-heavy`: `stdlib_intrinsics`, `stdlib_ownership`, `stdlib_tools`
  - `milestone-demo`: `stdlib_intrinsics`, `stdlib_ownership`, `stdlib_tools`
  - `hand-authored-generated-shape`: `stdlib_intrinsics`, `stdlib_ownership`, `stdlib_tools`
- implementation summary:
  - `stdlib_intrinsics`: replaced the large scaffold with a compact direct demo using real libm/hash/base32/filesystem helpers while preserving the exact observable output
  - `stdlib_ownership`: replaced the generated runtime emulation with direct min-heap helpers, partition-point bisect insertion, a lazy `chain` helper, and a compact `Counter`
  - `stdlib_tools`: replaced the large wrapper-heavy scaffold with direct monotonic/timing helpers, a small glob matcher, direct filesystem copy/move/remove flows, and a minimal TOML inline parser for the exercised path
- local validation completed:
  - `rustfmt demos/stdlib_intrinsics/idiomatic.rs demos/stdlib_ownership/idiomatic.rs demos/stdlib_tools/idiomatic.rs`
  - temp Cargo validation for `stdlib_intrinsics` with `blake2 = "0.10"`, `chrono = "0.4"`, `data-encoding = "2"`, `fs2 = "0.4"`, `libm = "0.2"`, `sha2 = "0.10"`
  - `rustc demos/stdlib_ownership/idiomatic.rs -o /tmp/stdlib_ownership_idiomatic`
  - `rustc demos/stdlib_tools/idiomatic.rs -o /tmp/stdlib_tools_idiomatic`
  - `/tmp/stdlib_ownership_idiomatic`
  - `/tmp/stdlib_tools_idiomatic`
  - `cargo run -q -p sifr -- run demos/stdlib_intrinsics/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_ownership/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_tools/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 follow-up revalidation:
    - `rustfmt demos/stdlib_intrinsics/idiomatic.rs demos/stdlib_ownership/idiomatic.rs`
    - temp Cargo validation for `stdlib_intrinsics` with `blake2 = "0.10"`, `chrono = "0.4"`, `data-encoding = "2"`, `fs2 = "0.4"`, `libm = "0.2"`, `sha2 = "0.10"`
    - `rustc demos/stdlib_ownership/idiomatic.rs -o /tmp/stdlib_ownership_idiomatic`
    - `/tmp/stdlib_ownership_idiomatic`
    - `cargo run -q -p sifr -- run demos/stdlib_intrinsics/main.sifr`
    - `cargo run -q -p sifr -- run demos/stdlib_ownership/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-55-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-55-review-pass-2.md`
- review application summary:
  - pass 1 on `stdlib_intrinsics` raised one real parity note about preserving the `disk_usage("/")[0]` print gate; I accepted it and changed the companion to print only when `total_space("/")` succeeds
  - pass 1 on `stdlib_ownership` initially returned `OK`, but an embedded-source retry surfaced two real Rust-first quality issues in the first draft: the fake sorted-vector heap and the eager `chain` helper. I accepted those, replaced them with direct heap operations plus a lazy chain helper, generalized `Counter::from_list`, and then fixed the tie-order regression so `most_common(1)` again prints `[("apple", 3)]`
  - pass 1 on `stdlib_tools` returned `OK`
  - pass 2 on `stdlib_intrinsics` raised only a non-blocking `processor()` preference and a return-shape note tied to the older Sifr-surface rubric; neither was accepted as a blocker
  - pass 2 on `stdlib_tools` likewise drifted back into Sifr-surface/type-shape parity notes about `TomlValue`, `Vec`, and `run_command`, and none were accepted because the paired demo-visible behavior already matched under all validation lanes
  - pass 2 on `stdlib_ownership` stalled without a usable verdict and was carried as a transport note rather than treated as a blocker
- reviewer tooling note:
  - batch 55 used direct per-file `agent review` prompts because the `agent review` skill referenced by the phase-loop skill is not available in this session
  - an embedded-source retry was also used for `stdlib_ownership` pass 1 after the initial direct prompt returned only `OK`; that narrower retry found the heap/lazy-chain issues that were actually worth fixing
  - `stdlib_ownership` pass 2 still stalled without output after repeated polls, so the artifact records that transport issue explicitly rather than inventing a clean verdict

#### batch_66_decimal_types_decimal_arithmetic_decimal_conversions

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/decimal_types/idiomatic.rs`
  - `demos/decimal_arithmetic/idiomatic.rs`
  - `demos/decimal_conversions/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive first decimal milestone slice instead of mixing decimal semantics with unrelated diagnostics or fixture-only decimal folders
  - `decimal_arithmetic` and `decimal_conversions` were still large generated-style outliers, while `decimal_types` was smaller but belonged to the same decimal-language surface
- priority tags:
  - `numeric-semantics`: `decimal_types`, `decimal_arithmetic`, `decimal_conversions`
  - `milestone-demo`: `decimal_types`, `decimal_arithmetic`, `decimal_conversions`
  - `hand-authored-generated-shape`: `decimal_types`, `decimal_arithmetic`, `decimal_conversions`
- implementation summary:
  - `decimal_types`: replaced the first-draft emitted-style context boilerplate with direct `rust_decimal` / `bigdecimal` literals and a single precision helper for the `BigDecimal` result shape
  - `decimal_arithmetic`: collapsed the decimal runtime wrapper layer to focused helpers for floor-division, modulo, midpoint-half-even rounding, precision rounding, and negative-square-root error reporting
  - `decimal_conversions`: replaced the full IO/runtime scaffold with direct decimal conversion helpers, quoted JSON-string formatting, and numeric bigint/int extraction built from decimal internals
- local validation completed:
  - `rustfmt demos/decimal_types/idiomatic.rs demos/decimal_arithmetic/idiomatic.rs demos/decimal_conversions/idiomatic.rs`
  - temporary Cargo compile/run for `demos/decimal_types/idiomatic.rs` with `bigdecimal = "0.4"`, `num-bigint = "0.4"`, and `rust_decimal = { version = "1", features = ["maths"] }`
  - temporary Cargo compile/run for `demos/decimal_arithmetic/idiomatic.rs` with `bigdecimal = "0.4"` and `rust_decimal = { version = "1", features = ["maths"] }`
  - temporary Cargo compile/run for `demos/decimal_conversions/idiomatic.rs` with `bigdecimal = "0.4"`, `num-bigint = "0.4"`, and `rust_decimal = "1"`
  - `cargo run -q -p sifr -- run demos/decimal_types/main.sifr`
  - `cargo run -q -p sifr -- run demos/decimal_arithmetic/main.sifr`
  - `cargo run -q -p sifr -- run demos/decimal_conversions/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/decimal_conversions/idiomatic.rs`
    - temporary Cargo compile/run for `demos/decimal_conversions/idiomatic.rs` with `bigdecimal = "0.4"`, `num-bigint = "0.4"`, and `rust_decimal = "1"`
    - `cargo run -q -p sifr -- run demos/decimal_conversions/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-66-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-66-review-pass-2.md`
- review application summary:
  - `decimal_types` and `decimal_arithmetic` returned `OK` in pass 1
  - pass 1 on `decimal_conversions` surfaced one real Rust-first issue in substance: the original rewrite used string splitting on `BigDecimal::to_string()` and an `i64` intermediary for integer extraction; I accepted that and rewrote the conversion helpers to use `Decimal::mantissa()` / `Decimal::scale()` and `BigDecimal::as_bigint_and_exponent()`
  - pass 1 notes claiming `12.3400` canonicalization was lost and that message-shape consistency was a blocker were not accepted because the validated output already preserved `12.3400` and only the exercised bigdecimal out-of-range message is user-visible in this demo
  - all three pass-2 review prompts stalled without a usable verdict after repeated retries, so the batch carries those as reviewer-transport notes rather than accepted blockers
- reviewer tooling note:
  - batch 66 used direct per-file `agent review` prompts with embedded Sifr and Rust file contents
  - the pass-2 review lane stalled repeatedly on all three decimal files after the accepted `decimal_conversions` fix, so the artifact records those stalls explicitly instead of inventing clean verdicts

#### batch_67_import_forms_imports_external_modules

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/import_forms/idiomatic.rs`
  - `demos/imports/idiomatic.rs`
  - `demos/external_modules/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive import/module semantics slice instead of mixing the remaining cross-file import forms with unrelated decimal or compiler-surface demos
  - none of the three had a strong Rust-first companion yet, so the batch could replace missing or absent module examples with direct standalone Rust equivalents in one pass
- priority tags:
  - `module-semantics`: `import_forms`, `imports`, `external_modules`
  - `milestone-demo`: `import_forms`, `imports`, `external_modules`
  - `missing-companion`: `import_forms`, `imports`, `external_modules`
- implementation summary:
  - `import_forms`: authored a tiny helper module plus direct `use helper::value;` import to preserve the alternate import-form teaching point without emitted-style scaffolding
  - `imports`: authored a compact multi-struct `models` module and direct `use models::{Product, User};` imports so the Rust companion matches the cross-file compilation and display outcome with ordinary Rust APIs
  - `external_modules`: authored a small non-main `worker` module and direct import to keep the external-module execution point while expressing the `floor(3.9)` result in normal Rust
- local validation completed:
  - `rustfmt demos/import_forms/idiomatic.rs demos/imports/idiomatic.rs demos/external_modules/idiomatic.rs`
  - `rustc --edition=2021 demos/import_forms/idiomatic.rs -o /tmp/sifr-idiomatic-import-forms && /tmp/sifr-idiomatic-import-forms`
  - `rustc --edition=2021 demos/imports/idiomatic.rs -o /tmp/sifr-idiomatic-imports && /tmp/sifr-idiomatic-imports`
  - `rustc --edition=2021 demos/external_modules/idiomatic.rs -o /tmp/sifr-idiomatic-external-modules && /tmp/sifr-idiomatic-external-modules`
  - `cargo run -q -p sifr -- run demos/import_forms/main.sifr`
  - `cargo run -q -p sifr -- run demos/imports/main.sifr`
  - `cargo run -q -p sifr -- run demos/external_modules/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-67-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-67-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 67 used direct per-file `agent review` prompts with embedded Sifr and Rust file contents
  - the initial pass-1 `import_forms` reviewer response lagged behind the other two files, but it completed cleanly without requiring any retry or code change

#### batch_68_borrow_exclusivity_borrow_lowering_compiler_safety

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/borrow_exclusivity/idiomatic.rs`
  - `demos/borrow_lowering/idiomatic.rs`
  - `demos/compiler_safety/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive borrow/compiler-hardening slice instead of mixing the remaining borrow semantics and compiler cleanup demos with unrelated CLI or decimal work
  - all three still carried visible generated-style ceremony in the existing companions, especially borrowed container signatures, enum narrowing scaffolds, and synthetic context-manager lowering
- priority tags:
  - `borrow-semantics`: `borrow_exclusivity`, `borrow_lowering`, `compiler_safety`
  - `milestone-demo`: `borrow_exclusivity`, `borrow_lowering`, `compiler_safety`
  - `hand-authored-generated-shape`: `borrow_exclusivity`, `borrow_lowering`, `compiler_safety`
- implementation summary:
  - `borrow_exclusivity`: replaced borrowed-`Vec` signatures and emitted-style literals with direct slice APIs, iterator sums, and an owned reverse helper that keeps the borrow-by-default teaching point intact
  - `borrow_lowering`: replaced the generated union scaffolding with compact enums, direct `&str` comparison, normal `Option<&str>` handling for the `None` path, and a simple consuming `String` helper
  - `compiler_safety`: replaced the synthetic `__enter__`/`__exit__` lowering scaffolding with a small `EventLog` plus RAII `Drop` guards for file/database resources and a direct boxed callback field in `Config`
- local validation completed:
  - `rustfmt demos/borrow_exclusivity/idiomatic.rs demos/borrow_lowering/idiomatic.rs demos/compiler_safety/idiomatic.rs`
  - `rustc --edition=2021 demos/borrow_exclusivity/idiomatic.rs -o /tmp/sifr-idiomatic-borrow-exclusivity && /tmp/sifr-idiomatic-borrow-exclusivity`
  - `rustc --edition=2021 demos/borrow_lowering/idiomatic.rs -o /tmp/sifr-idiomatic-borrow-lowering && /tmp/sifr-idiomatic-borrow-lowering`
  - `rustc --edition=2021 demos/compiler_safety/idiomatic.rs -o /tmp/sifr-idiomatic-compiler-safety && /tmp/sifr-idiomatic-compiler-safety`
  - `cargo run -q -p sifr -- run demos/borrow_exclusivity/main.sifr`
  - `cargo run -q -p sifr -- run demos/borrow_lowering/main.sifr`
  - `cargo run -q -p sifr -- run demos/compiler_safety/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-68-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-68-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 68 used direct per-file `agent review` prompts with compact observed-output-plus-Rust-file prompts
  - the first embedded-source reviewer processes stalled, so I reran the review with shorter prompts focused on observable behavior and the final Rust companions; those returned clean `OK` verdicts

#### batch_69_branch_paths_cargo_manifest_cli_modes

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/branch_paths/idiomatic.rs`
  - `demos/cargo_manifest/idiomatic.rs`
  - `demos/cli_modes/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive compiler/frontend semantics slice instead of mixing the remaining project-mode and manifest demos with unrelated borrow or decimal work
  - `branch_paths` and `cargo_manifest` had no Rust-first companion yet, while `cli_modes` was already tiny enough that the right move was to re-review it and keep it unchanged
- priority tags:
  - `compiler-surface`: `branch_paths`, `cargo_manifest`, `cli_modes`
  - `milestone-demo`: `branch_paths`, `cargo_manifest`, `cli_modes`
  - `missing-companion`: `branch_paths`, `cargo_manifest`
- implementation summary:
  - `branch_paths`: authored a compact local helper module plus direct nested-branch evaluation logic that preserves the regression-matrix branch-path outputs without extra scaffolding
  - `cargo_manifest`: authored a small helper-backed companion around `num_bigint::BigInt` so the Rust file still visibly demonstrates the external-dependency/manifest point rather than flattening it into pure std
  - `cli_modes`: reviewed the existing single-print companion and kept it unchanged because it already matched the minimal CLI milestone bar
- local validation completed:
  - `rustfmt demos/branch_paths/idiomatic.rs demos/cargo_manifest/idiomatic.rs demos/cli_modes/idiomatic.rs`
  - `rustc --edition=2021 demos/branch_paths/idiomatic.rs -o /tmp/sifr-idiomatic-branch-paths && /tmp/sifr-idiomatic-branch-paths`
  - `rustc --edition=2021 demos/cli_modes/idiomatic.rs -o /tmp/sifr-idiomatic-cli-modes && /tmp/sifr-idiomatic-cli-modes`
  - temporary Cargo compile/run for `demos/cargo_manifest/idiomatic.rs` with `num-bigint = "0.4"`
  - `cargo run -q -p sifr -- run demos/branch_paths/main.sifr`
  - `cargo run -q -p sifr -- run demos/cargo_manifest/main.sifr`
  - `cargo run -q -p sifr -- run demos/cli_modes/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-69-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-69-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 69 used direct per-file `agent review` prompts with compact observed-output-plus-Rust-file prompts
  - no review retries or code changes were needed after the initial authored companions landed

#### batch_70_module_ordering_module_assembly_module_cycle_diagnostics

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/module_ordering/idiomatic.rs`
  - `demos/module_assembly/idiomatic.rs`
  - `demos/module_cycle_diagnostics/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive module-graph slice instead of mixing the remaining module-ordering/project-graph demos with unrelated diagnostics or fixture-only folders
  - none of the three had a Rust-first top-level companion yet, and all three are small enough that the direct local-module form is the clearest equivalent
- priority tags:
  - `module-semantics`: `module_ordering`, `module_assembly`, `module_cycle_diagnostics`
  - `milestone-demo`: `module_ordering`, `module_assembly`, `module_cycle_diagnostics`
  - `missing-companion`: `module_ordering`, `module_assembly`, `module_cycle_diagnostics`
- implementation summary:
  - `module_ordering`: authored a small provider/consumer module split with a direct integer result that preserves the dependency-safe ordering lesson without codegen-shaped helpers
  - `module_assembly`: authored two provider modules plus a consumer join helper and then tightened the consumer import path to `crate::{a_provider, z_provider}` after pass 1
  - `module_cycle_diagnostics`: authored a minimal provider/consumer split that preserves the deterministic module-graph output while keeping the code straightforward and explicit
- local validation completed:
  - `rustfmt demos/module_ordering/idiomatic.rs demos/module_assembly/idiomatic.rs demos/module_cycle_diagnostics/idiomatic.rs`
  - `rustc --edition=2021 demos/module_ordering/idiomatic.rs -o /tmp/sifr-idiomatic-module-ordering && /tmp/sifr-idiomatic-module-ordering`
  - `rustc --edition=2021 demos/module_assembly/idiomatic.rs -o /tmp/sifr-idiomatic-module-assembly && /tmp/sifr-idiomatic-module-assembly`
  - `rustc --edition=2021 demos/module_cycle_diagnostics/idiomatic.rs -o /tmp/sifr-idiomatic-module-cycle-diagnostics && /tmp/sifr-idiomatic-module-cycle-diagnostics`
  - `cargo run -q -p sifr -- run demos/module_ordering/main.sifr`
  - `cargo run -q -p sifr -- run demos/module_assembly/main.sifr`
  - `cargo run -q -p sifr -- run demos/module_cycle_diagnostics/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-70-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-70-review-pass-2.md`
- review application summary:
  - `module_ordering` and `module_cycle_diagnostics` returned `OK` in pass 1
  - pass 1 on `module_assembly` surfaced one cleanup note about the nested-module import path; I accepted that and changed the import to `crate::{a_provider, z_provider}`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 70 used direct per-file `agent review` prompts with compact observed-output-plus-Rust-file prompts
  - `module_assembly` needed a shorter retry prompt in pass 2 after the longer prompt stalled repeatedly, but the final retry returned `OK`

#### batch_71_project_build_project_check_project_entrypoint

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/project_build/idiomatic.rs`
  - `demos/project_check/idiomatic.rs`
  - `demos/project_entrypoint/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive project-mode/frontend slice instead of mixing the remaining project entry/build/check demos with unrelated graph-isolation or diagnostics fixtures
  - none of the three had a Rust-first companion yet, and each one benefits from a direct local-module shape rather than preserving emitted project-loader ceremony
- priority tags:
  - `project-mode`: `project_build`, `project_check`, `project_entrypoint`
  - `milestone-demo`: `project_build`, `project_check`, `project_entrypoint`
  - `missing-companion`: `project_build`, `project_check`, `project_entrypoint`
- implementation summary:
  - `project_build`: authored a compact helper/formatter split using `num_bigint::BigInt` so the Rust companion still visibly exercises the external dependency that the manifest/build path must carry
  - `project_check`: authored a tiny helper module around `std::f64::consts::PI` to preserve the project-aware local-import check parity demo without extra scaffolding
  - `project_entrypoint`: authored a minimal helper module that preserves the canonical entry-path teaching point with the same `5 + floor(2.9) = 7` outcome in direct Rust form
- local validation completed:
  - `rustfmt demos/project_build/idiomatic.rs demos/project_check/idiomatic.rs demos/project_entrypoint/idiomatic.rs`
  - temporary Cargo compile/run for `demos/project_build/idiomatic.rs` with `num-bigint = "0.4"`
  - `rustc --edition=2021 demos/project_check/idiomatic.rs -o /tmp/sifr-idiomatic-project-check && /tmp/sifr-idiomatic-project-check`
  - `rustc --edition=2021 demos/project_entrypoint/idiomatic.rs -o /tmp/sifr-idiomatic-project-entrypoint && /tmp/sifr-idiomatic-project-entrypoint`
  - `cargo run -q -p sifr -- run demos/project_build/main.sifr`
  - `cargo run -q -p sifr -- run demos/project_check/main.sifr`
  - `cargo run -q -p sifr -- run demos/project_entrypoint/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-71-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-71-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 71 used direct per-file `agent review` prompts with compact observed-output-plus-Rust-file prompts
  - no review retries or code changes were needed after the initial authored companions landed

#### batch_72_decimal_diagnostics_decimal_verification_dependency_manifest

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/decimal_diagnostics/idiomatic.rs`
  - `demos/decimal_verification/idiomatic.rs`
  - `demos/dependency_manifest/idiomatic.rs`
- selection rationale:
  - the two remaining decimal demos were still uncovered and still carried generated-style or boilerplate-heavy Rust companions despite comparatively small observable contracts
  - `dependency_manifest` was the adjacent missing project/dependency-closure companion and fit the same validation shape without introducing another unrelated subsystem slice
  - the trio keeps this batch narrow while removing one more decimal-semantics outlier cluster from the remaining corpus
- priority tags:
  - `decimal-semantics`: `decimal_diagnostics`, `decimal_verification`
  - `project-mode`: `dependency_manifest`
  - `missing-companion`: `dependency_manifest`
  - `hand-authored-generated-shape`: `decimal_diagnostics`, `decimal_verification`
- implementation summary:
  - `decimal_diagnostics`: replaced the generated-style decimal scaffolding with direct `rust_decimal` and `bigdecimal` checks for the reserved diagnostics contract and exact formatted rounding/quantization strings
  - `decimal_verification`: replaced the giant generated stdlib/runtime layer with compact floor-division, remainder, JSON-string, and determinism helpers that preserve the exact observed decimal and bigdecimal outputs
  - `dependency_manifest`: authored a minimal TOML-based manifest dependency-closure demo that preserves the reachable-support-module teaching point with a direct `toml::Value` parse path
- local validation completed:
  - `rustfmt demos/decimal_diagnostics/idiomatic.rs demos/decimal_verification/idiomatic.rs demos/dependency_manifest/idiomatic.rs`
  - temporary Cargo compile/run for `demos/decimal_diagnostics/idiomatic.rs` with `rust_decimal = "1"` and `bigdecimal = "0.4"`
  - temporary Cargo compile/run for `demos/decimal_verification/idiomatic.rs` with `rust_decimal = "1"`, `bigdecimal = "0.4"`, and `serde_json = "1"`
  - temporary Cargo compile/run for `demos/dependency_manifest/idiomatic.rs` with `toml = "0.8"`
  - `cargo run -q -p sifr -- run demos/decimal_diagnostics/main.sifr`
  - `cargo run -q -p sifr -- run demos/decimal_verification/main.sifr`
  - `cargo run -q -p sifr -- run demos/dependency_manifest/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-72-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-72-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no code changes were needed after the initial authored companions landed
- reviewer tooling note:
  - batch 72 used direct per-file `agent review` prompts through short Python subprocess wrappers with explicit timeouts
  - `decimal_diagnostics` needed shorter retry prompts in both passes after the longer prompt shape stalled, but both retry reviews returned `OK`

#### batch_73_diagnostic_exit_codes_diagnostic_options_diagnostic_schema

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/diagnostic_exit_codes/idiomatic.rs`
  - `demos/diagnostic_options/idiomatic.rs`
  - `demos/diagnostic_schema/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos in the same diagnostics/frontend contract area
  - `diagnostic_exit_codes` was still missing its Rust companion entirely
  - `diagnostic_options` and `diagnostic_schema` already had minimal companions, so this batch could close the whole diagnostics slice by adding the missing file and re-reviewing the two existing ones against the current Rust-first standard
- priority tags:
  - `compiler-frontend`: `diagnostic_exit_codes`, `diagnostic_options`, `diagnostic_schema`
  - `missing-companion`: `diagnostic_exit_codes`
  - `milestone-demo`: `diagnostic_exit_codes`, `diagnostic_options`, `diagnostic_schema`
- implementation summary:
  - `diagnostic_exit_codes`: authored a compact helper-module companion that preserves the cross-mode local-import and doubled-value behavior with the exact observed output
  - `diagnostic_options`: re-reviewed the existing tiny sum demo unchanged because it already matched the positive-path contract cleanly
  - `diagnostic_schema`: re-reviewed the existing tiny print demo unchanged because it already matched the positive-path schema-quality contract cleanly
- local validation completed:
  - `rustfmt demos/diagnostic_exit_codes/idiomatic.rs demos/diagnostic_options/idiomatic.rs demos/diagnostic_schema/idiomatic.rs`
  - `rustc --edition=2021 demos/diagnostic_exit_codes/idiomatic.rs -o /tmp/sifr-idiomatic-diagnostic-exit-codes && /tmp/sifr-idiomatic-diagnostic-exit-codes`
  - `rustc --edition=2021 demos/diagnostic_options/idiomatic.rs -o /tmp/sifr-idiomatic-diagnostic-options && /tmp/sifr-idiomatic-diagnostic-options`
  - `rustc --edition=2021 demos/diagnostic_schema/idiomatic.rs -o /tmp/sifr-idiomatic-diagnostic-schema && /tmp/sifr-idiomatic-diagnostic-schema`
  - `cargo run -q -p sifr -- run demos/diagnostic_exit_codes/main.sifr`
  - `cargo run -q -p sifr -- run demos/diagnostic_options/main.sifr`
  - `cargo run -q -p sifr -- run demos/diagnostic_schema/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-73-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-73-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the initial `diagnostic_exit_codes` authoring step
- reviewer tooling note:
  - batch 73 used direct per-file `agent review` prompts through short Python subprocess wrappers with explicit timeouts
  - the initial combined pass-2 runner stalled, so pass 2 was finalized with separate per-file retries that each returned `OK`

#### batch_74_reachable_imports_project_test_discovery_graph_isolation

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/reachable_imports/idiomatic.rs`
  - `demos/project_test_discovery/idiomatic.rs`
  - `demos/graph_isolation/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos from the same phase-23 project-graph and isolation correctness area
  - all three were still missing Rust companions entirely
  - the trio stays tightly focused on reachable-module closure, support-module discovery parity, and graph isolation instead of mixing those frontend contracts with unrelated runtime or stdlib demos
- priority tags:
  - `project-graph`: `reachable_imports`, `project_test_discovery`, `graph_isolation`
  - `missing-companion`: `reachable_imports`, `project_test_discovery`, `graph_isolation`
  - `milestone-demo`: `reachable_imports`, `project_test_discovery`, `graph_isolation`
- implementation summary:
  - `reachable_imports`: authored a compact helper module that preserves the reachable import-closure demo with the same `6 * 7 = 42` result
  - `project_test_discovery`: authored a minimal `shared` constant plus helper module so the Rust companion mirrors the support-module discovery path while keeping the output direct and small
  - `graph_isolation`: authored a tiny helper module that preserves the isolated graph-regression matrix value without pulling in unrelated files or extra scaffolding
- local validation completed:
  - `rustfmt demos/reachable_imports/idiomatic.rs demos/project_test_discovery/idiomatic.rs demos/graph_isolation/idiomatic.rs`
  - `rustc --edition=2021 demos/reachable_imports/idiomatic.rs -o /tmp/sifr-idiomatic-reachable-imports && /tmp/sifr-idiomatic-reachable-imports`
  - `rustc --edition=2021 demos/project_test_discovery/idiomatic.rs -o /tmp/sifr-idiomatic-project-test-discovery && /tmp/sifr-idiomatic-project-test-discovery`
  - `rustc --edition=2021 demos/graph_isolation/idiomatic.rs -o /tmp/sifr-idiomatic-graph-isolation && /tmp/sifr-idiomatic-graph-isolation`
  - `cargo run -q -p sifr -- run demos/reachable_imports/main.sifr`
  - `cargo run -q -p sifr -- run demos/project_test_discovery/main.sifr`
  - `cargo run -q -p sifr -- run demos/graph_isolation/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-74-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-74-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the initial authoring step
- reviewer tooling note:
  - batch 74 used direct per-file `agent review` prompts through short Python subprocess wrappers with explicit timeouts
  - the initial combined pass-1 runner stalled, and `graph_isolation` needed a shorter retry in pass 1; the separate per-file retries then returned `OK`

#### batch_75_mode_consistency_project_graph_ecosystem_validation

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/mode_consistency/idiomatic.rs`
  - `demos/project_graph/idiomatic.rs`
  - `demos/ecosystem_validation/idiomatic.rs`
- selection rationale:
  - the batch keeps a coherent frontend/verification contract slice instead of mixing in the larger stdlib-heavy outliers still left in the corpus
  - `mode_consistency` and `project_graph` were both still missing Rust companions and both exercise project-mode/frontend dependency resolution behavior
  - `ecosystem_validation` was already minimal and Rust-first, so it fit cleanly as the third file by closing out a neighboring positive contract demo without forcing unrelated churn
- priority tags:
  - `compiler-frontend`: `mode_consistency`, `project_graph`
  - `verification-contract`: `ecosystem_validation`
  - `missing-companion`: `mode_consistency`, `project_graph`
- implementation summary:
  - `mode_consistency`: authored a compact helper module that preserves the parity-matrix teaching point and the observed `BASE + floor(2.9) + 1 = 8` result
  - `project_graph`: authored direct `provider` and `consumer` modules that preserve the project-graph resolution path and final `42` output
  - `ecosystem_validation`: re-reviewed the existing tiny verification-lane contract demo unchanged because it already met the Rust-first bar
- local validation completed:
  - `rustfmt demos/mode_consistency/idiomatic.rs demos/project_graph/idiomatic.rs demos/ecosystem_validation/idiomatic.rs`
  - `rustc --edition=2021 demos/mode_consistency/idiomatic.rs -o /tmp/sifr-idiomatic-mode-consistency && /tmp/sifr-idiomatic-mode-consistency`
  - `rustc --edition=2021 demos/project_graph/idiomatic.rs -o /tmp/sifr-idiomatic-project-graph && /tmp/sifr-idiomatic-project-graph`
  - `rustc --edition=2021 demos/ecosystem_validation/idiomatic.rs -o /tmp/sifr-idiomatic-ecosystem-validation && /tmp/sifr-idiomatic-ecosystem-validation`
  - `cargo run -q -p sifr -- run demos/mode_consistency/main.sifr`
  - `cargo run -q -p sifr -- run demos/project_graph/main.sifr`
  - `cargo run -q -p sifr -- run demos/ecosystem_validation/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-75-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-75-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the initial authoring step
- reviewer tooling note:
  - batch 75 used direct per-file `agent review` prompts through short Python subprocess wrappers with explicit timeouts
  - the initial combined runners stalled, and `mode_consistency` in pass 1 plus `project_graph` in pass 2 needed shorter retries before returning `OK`

#### batch_76_nested_function_part1_nested_function_part2_nested_function_part3

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/nested_function_part1/idiomatic.rs`
  - `demos/nested_function_part2/idiomatic.rs`
  - `demos/nested_function_part3/idiomatic.rs`
- selection rationale:
  - the batch clears the next contiguous positive runnable demos in the nested-function pipeline instead of mixing that focused callable/closure slice with the larger remaining outliers
  - all three companions were already close, but they still carried generated-shape residue such as `as i64`, `&Vec`, string-comparison assertions, and redundant return noise
  - keeping the three neighboring demos together preserved a coherent review surface around nested helpers, recursion, and nonlocal capture
- priority tags:
  - `nested-functions`: `nested_function_part1`, `nested_function_part2`, `nested_function_part3`
  - `hand-authored-generated-shape`: `nested_function_part1`, `nested_function_part2`, `nested_function_part3`
  - `milestone-demo`: `nested_function_part1`, `nested_function_part2`, `nested_function_part3`
- implementation summary:
  - `nested_function_part1`: simplified the composed-helper flow into direct Rust closures plus a small recursive helper and replaced string assertions with direct numeric `assert_eq!`
  - `nested_function_part2`: collapsed the recursive helpers into straightforward expression-style Rust and removed redundant casts and return noise
  - `nested_function_part3`: changed the helper signature to take a slice instead of `&Vec`, kept the closure capture example minimal, and replaced the final string comparison with a direct Rust-first assertion
- local validation completed:
  - `rustfmt demos/nested_function_part1/idiomatic.rs demos/nested_function_part2/idiomatic.rs demos/nested_function_part3/idiomatic.rs`
  - `rustc --edition=2021 demos/nested_function_part1/idiomatic.rs -o /tmp/sifr-idiomatic-nested-function-part1 && /tmp/sifr-idiomatic-nested-function-part1`
  - `rustc --edition=2021 demos/nested_function_part2/idiomatic.rs -o /tmp/sifr-idiomatic-nested-function-part2 && /tmp/sifr-idiomatic-nested-function-part2`
  - `rustc --edition=2021 demos/nested_function_part3/idiomatic.rs -o /tmp/sifr-idiomatic-nested-function-part3 && /tmp/sifr-idiomatic-nested-function-part3`
  - `cargo run -q -p sifr -- run demos/nested_function_part1/main.sifr`
  - `cargo run -q -p sifr -- run demos/nested_function_part2/main.sifr`
  - `cargo run -q -p sifr -- run demos/nested_function_part3/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-76-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-76-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the initial cleanup
- reviewer tooling note:
  - batch 76 used direct per-file `agent review` prompts through short Python subprocess wrappers with explicit timeouts
  - the initial combined runner stalled, and `nested_function_part1` needed a shorter retry in pass 1 before returning `OK`

#### batch_77_recursive_type_part1_recursive_type_part2_recursive_type_part3

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/recursive_type_part1/idiomatic.rs`
  - `demos/recursive_type_part2/idiomatic.rs`
  - `demos/recursive_type_part3/idiomatic.rs`
- selection rationale:
  - the batch clears the first contiguous recursive-type trilogy instead of mixing those focused alias-resolution demos with the unrelated remaining runtime and verification outliers
  - all three companions were runnable and compact, but they still underplayed the recursive-type teaching surface by collapsing the Sifr aliases down to bare prints and `Vec` helpers
  - keeping part 1 through part 3 together preserves a coherent review surface around alias resolution, accepted recursive shapes, and generic recursive lowering
- priority tags:
  - `recursive-types`: `recursive_type_part1`, `recursive_type_part2`, `recursive_type_part3`
  - `milestone-demo`: `recursive_type_part1`, `recursive_type_part2`, `recursive_type_part3`
  - `hand-authored-generated-shape`: `recursive_type_part1`, `recursive_type_part2`, `recursive_type_part3`
- implementation summary:
  - `recursive_type_part1`: replaced the bare `Vec` helper with direct `Payload`/`Response` aliases plus a recursive `Json` enum so the Rust companion actually demonstrates recursive alias names before printing the observed output
  - `recursive_type_part2`: preserved the payload-size behavior while adding an explicit recursive `Node::Branch(Vec<Node>)` enum to mirror the accepted recursive-alias shape
  - `recursive_type_part3`: replaced the single print-only stub with a generic recursive `Node<T>` enum and a small constructed value that demonstrates the preserved recursive representation directly
- local validation completed:
  - `rustfmt demos/recursive_type_part1/idiomatic.rs demos/recursive_type_part2/idiomatic.rs demos/recursive_type_part3/idiomatic.rs`
  - `rustc --edition=2021 demos/recursive_type_part1/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part1 && /tmp/sifr-idiomatic-recursive-type-part1`
  - `rustc --edition=2021 demos/recursive_type_part2/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part2 && /tmp/sifr-idiomatic-recursive-type-part2`
  - `rustc --edition=2021 demos/recursive_type_part3/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part3 && /tmp/sifr-idiomatic-recursive-type-part3`
  - `cargo run -q -p sifr -- run demos/recursive_type_part1/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_type_part2/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_type_part3/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-77-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-77-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the validated rewrite
- reviewer tooling note:
  - batch 77 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both passes ended with no accepted blockers across the trio

#### batch_78_recursive_type_part4_recursive_type_part5_recursive_type_part6

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/recursive_type_part4/idiomatic.rs`
  - `demos/recursive_type_part5/idiomatic.rs`
  - `demos/recursive_type_part6/idiomatic.rs`
- selection rationale:
  - the batch clears the second contiguous recursive-type trilogy instead of splitting the remaining recursive-class demos across unrelated runtime or verification work
  - all three companions still used cloned `Option<TreeNode>` recursion and repeated generated-style tree construction, which obscured the recursive-class teaching point
  - keeping part 4 through part 6 together preserves a coherent review surface around recursive node narrowing, traversal, and the generic packet alias closure
- priority tags:
  - `recursive-types`: `recursive_type_part4`, `recursive_type_part5`, `recursive_type_part6`
  - `recursive-classes`: `recursive_type_part4`, `recursive_type_part5`, `recursive_type_part6`
  - `milestone-demo`: `recursive_type_part4`, `recursive_type_part5`, `recursive_type_part6`
- implementation summary:
  - `recursive_type_part4`: replaced cloned `Option<TreeNode>` lowering with direct `Option<&TreeNode>` traversal and compact `TreeNode::new` construction that keeps the guard-narrowing behavior obvious
  - `recursive_type_part5`: applied the same borrow-based recursive traversal style to the runnable tree-traversal variant so the Rust reads like hand-written tree code instead of lowered scaffolding
  - `recursive_type_part6`: kept the same borrow-based tree helper, then added a direct generic `Packet<T>` enum so the final alias-declaration teaching point is represented concretely in the Rust companion
- local validation completed:
  - `rustfmt demos/recursive_type_part4/idiomatic.rs demos/recursive_type_part5/idiomatic.rs demos/recursive_type_part6/idiomatic.rs`
  - `rustc --edition=2021 demos/recursive_type_part4/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part4 && /tmp/sifr-idiomatic-recursive-type-part4`
  - `rustc --edition=2021 demos/recursive_type_part5/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part5 && /tmp/sifr-idiomatic-recursive-type-part5`
  - `rustc --edition=2021 demos/recursive_type_part6/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-type-part6 && /tmp/sifr-idiomatic-recursive-type-part6`
  - `cargo run -q -p sifr -- run demos/recursive_type_part4/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_type_part5/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_type_part6/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-78-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-78-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the validated rewrite
- reviewer tooling note:
  - batch 78 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both passes ended with no accepted blockers across the trio

#### batch_79_nested_function_part4_nested_function_part5_recursive_types

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/nested_function_part4/idiomatic.rs`
  - `demos/nested_function_part5/idiomatic.rs`
  - `demos/recursive_types/idiomatic.rs`
- selection rationale:
  - the batch clears the remaining recursion-heavy positive demos that still carried the clearest generated-style residue instead of scattering those backtracking and self-referential structure demos across unrelated batches
  - `nested_function_part4` and `nested_function_part5` were the last unfinished nested-function demos and both still relied on verbose index lowering and stringified output assertions
  - `recursive_types` fit cleanly as the third file because it was the remaining standalone recursive-types milestone demo and still used mechanically boxed construction that could be simplified without changing the observed output
- priority tags:
  - `nested-functions`: `nested_function_part4`, `nested_function_part5`
  - `recursive-types`: `recursive_types`
  - `milestone-demo`: `recursive_types`
  - `hand-authored-generated-shape`: `nested_function_part4`, `nested_function_part5`, `recursive_types`
- implementation summary:
  - `nested_function_part4`: replaced lowered index scaffolding with direct slice-based backtracking helpers for `combination_sum` and `subsets`, and switched to structural `assert_eq!`
  - `nested_function_part5`: rewrote the closure/backtracking companion into direct slice-based Rust helpers, keeping the observed ordering and outputs while removing generated casts, `&Vec`, and string comparisons
  - `recursive_types`: tightened the linked-list and tree examples into direct recursive constructors that accept owned child nodes, preserving the printed `1` / `1` milestone output with clearer self-referential structure setup
- local validation completed:
  - `rustfmt demos/nested_function_part4/idiomatic.rs demos/nested_function_part5/idiomatic.rs demos/recursive_types/idiomatic.rs`
  - `rustc --edition=2021 demos/nested_function_part4/idiomatic.rs -o /tmp/sifr-idiomatic-nested-function-part4 && /tmp/sifr-idiomatic-nested-function-part4`
  - `rustc --edition=2021 demos/nested_function_part5/idiomatic.rs -o /tmp/sifr-idiomatic-nested-function-part5 && /tmp/sifr-idiomatic-nested-function-part5`
  - `rustc --edition=2021 demos/recursive_types/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-types && /tmp/sifr-idiomatic-recursive-types`
  - `cargo run -q -p sifr -- run demos/nested_function_part4/main.sifr`
  - `cargo run -q -p sifr -- run demos/nested_function_part5/main.sifr`
  - `cargo run -q -p sifr -- run demos/recursive_types/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-79-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-79-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the validated rewrite
- reviewer tooling note:
  - batch 79 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both passes ended with no accepted blockers across the trio

#### batch_80_stable_codegen_statement_expression_codegen_statement_expression_mix

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stable_codegen/idiomatic.rs`
  - `demos/statement_expression_codegen/idiomatic.rs`
  - `demos/statement_expression_mix/idiomatic.rs`
- selection rationale:
  - the batch clears the next coherent codegen-shaped slice instead of mixing these statement/expression lowering demos with unrelated verification or runtime outliers
  - `stable_codegen` was already small but still carried generated-style `&Vec`, casts, and return noise around a simple emission-boundary summary helper
  - both statement/expression demos still used stringified assertions and boilerplate lowered control-flow that could be rewritten as direct Rust without changing the observed outputs
- priority tags:
  - `codegen-shape`: `stable_codegen`, `statement_expression_codegen`, `statement_expression_mix`
  - `statement-expression-lowering`: `statement_expression_codegen`, `statement_expression_mix`
  - `milestone-demo`: `statement_expression_codegen`, `statement_expression_mix`
  - `hand-authored-generated-shape`: `stable_codegen`, `statement_expression_codegen`, `statement_expression_mix`
- implementation summary:
  - `stable_codegen`: simplified the helper to a slice-based summary loop with direct arithmetic and preserved the printed `33` result
  - `statement_expression_codegen`: rewrote the demo into straightforward Rust control flow with direct formatted-string assertions instead of nested `format!` scaffolding
  - `statement_expression_mix`: collapsed the lowered `for`/`while`-`else` scaffolding into the exact direct Rust accumulation steps that preserve the final `acc = 22` output
- local validation completed:
  - `rustfmt demos/stable_codegen/idiomatic.rs demos/statement_expression_codegen/idiomatic.rs demos/statement_expression_mix/idiomatic.rs`
  - `rustc --edition=2021 demos/stable_codegen/idiomatic.rs -o /tmp/sifr-idiomatic-stable-codegen && /tmp/sifr-idiomatic-stable-codegen`
  - `rustc --edition=2021 demos/statement_expression_codegen/idiomatic.rs -o /tmp/sifr-idiomatic-statement-expression-codegen && /tmp/sifr-idiomatic-statement-expression-codegen`
  - `rustc --edition=2021 demos/statement_expression_mix/idiomatic.rs -o /tmp/sifr-idiomatic-statement-expression-mix && /tmp/sifr-idiomatic-statement-expression-mix`
  - `cargo run -q -p sifr -- run demos/stable_codegen/main.sifr`
  - `cargo run -q -p sifr -- run demos/statement_expression_codegen/main.sifr`
  - `cargo run -q -p sifr -- run demos/statement_expression_mix/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-80-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-80-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the validated rewrite
- reviewer tooling note:
  - batch 80 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both review passes ended with no accepted blockers across the trio

#### batch_81_integer_safety_intrinsics_mut_sort

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/integer_safety/idiomatic.rs`
  - `demos/intrinsics/idiomatic.rs`
  - `demos/mut_sort/idiomatic.rs`
- selection rationale:
  - the batch clears a compact runtime-surface slice instead of dragging the larger remaining text-wrapper and verification demos into the same review window
  - `integer_safety` and `intrinsics` were both still positive milestone demos with obvious generated-style residue around straightforward bigint and intrinsic-stdlib behavior
  - `mut_sort` had no Rust companion at all, so it fit cleanly as the third file to close a small, focused mutable-list-method demo
- priority tags:
  - `runtime-surface`: `integer_safety`, `intrinsics`, `mut_sort`
  - `milestone-demo`: `integer_safety`, `intrinsics`
  - `missing-companion`: `mut_sort`
  - `hand-authored-generated-shape`: `integer_safety`, `intrinsics`
- implementation summary:
  - `integer_safety`: rewrote the bigint demo around direct iterative `BigInt` helpers and preserved the exact printed arithmetic, factorial, Fibonacci, and overflow-warning output
  - `intrinsics`: replaced the giant generated stdlib scaffold with small local assertion helpers and direct intrinsic checks for `sqrt` and `pi`
  - `mut_sort`: authored a compact missing companion that sorts an owned `Vec<i64>` in place and preserves the observed `mut_sort: ok` output
- local validation completed:
  - `rustfmt demos/integer_safety/idiomatic.rs demos/intrinsics/idiomatic.rs demos/mut_sort/idiomatic.rs`
  - temp Cargo validation for `integer_safety` with `num-bigint = "0.4"`
  - `rustc --edition=2021 demos/intrinsics/idiomatic.rs -o /tmp/sifr-idiomatic-intrinsics && /tmp/sifr-idiomatic-intrinsics`
  - `rustc --edition=2021 demos/mut_sort/idiomatic.rs -o /tmp/sifr-idiomatic-mut-sort && /tmp/sifr-idiomatic-mut-sort`
  - `cargo run -q -p sifr -- run demos/integer_safety/main.sifr`
  - `cargo run -q -p sifr -- run demos/intrinsics/main.sifr`
  - `cargo run -q -p sifr -- run demos/mut_sort/main.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_logging_consolidated.sifr`
  - `scripts/run_all_tests.sh`
- validation unblock:
  - `lib/sifr/tempfile.sifr`: removed the slice-based trailing-slash trim path from `mktemp_path()` and returned early when the temp root already ends in `/`
  - rationale: a cold-cache e2e rebuild of `stdlib_logging_consolidated` exposed a borrow-after-move bug in the generated Rust for the previous `root[:len(root) - 1]` path
  - observed outcome: the previously failing `stdlib_logging_consolidated` fixture compiled and ran cleanly after the stdlib change, and the full validation lane then passed
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-81-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-81-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the validated rewrite and tempfile fix
- reviewer tooling note:
  - batch 81 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both review passes ended with no accepted blockers across the trio

#### batch_82_rooted_entrypoint_run_and_build_resolver_triggers

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/rooted_entrypoint/idiomatic.rs`
  - `demos/run_and_build/idiomatic.rs`
  - `demos/resolver_triggers/idiomatic.rs`
- selection rationale:
  - the batch clears the remaining small project-mode import-closure trio instead of scattering these related dependency-resolution demos across later unrelated batches
  - all three demos were still missing Rust companions entirely
  - each demo is driven by a tiny local helper module graph, so they form a tight review surface around reachable import closure and project-mode activation behavior
- priority tags:
  - `project-mode`: `rooted_entrypoint`, `run_and_build`, `resolver_triggers`
  - `import-closure`: `rooted_entrypoint`, `run_and_build`, `resolver_triggers`
  - `missing-companion`: `rooted_entrypoint`, `run_and_build`, `resolver_triggers`
- implementation summary:
  - `rooted_entrypoint`: authored local `shared` and `helper` modules so the Rust companion mirrors the rooted import closure and preserves the printed phase label
  - `run_and_build`: authored a small local helper module and preserved the exact two-line run/build alignment output
  - `resolver_triggers`: authored a compact local helper module matching the relative-import-trigger demo and preserved the printed `18` result
- local validation completed:
  - `rustfmt demos/rooted_entrypoint/idiomatic.rs demos/run_and_build/idiomatic.rs demos/resolver_triggers/idiomatic.rs`
  - `rustc --edition=2021 demos/rooted_entrypoint/idiomatic.rs -o /tmp/sifr-idiomatic-rooted-entrypoint && /tmp/sifr-idiomatic-rooted-entrypoint`
  - `rustc --edition=2021 demos/run_and_build/idiomatic.rs -o /tmp/sifr-idiomatic-run-and-build && /tmp/sifr-idiomatic-run-and-build`
  - `rustc --edition=2021 demos/resolver_triggers/idiomatic.rs -o /tmp/sifr-idiomatic-resolver-triggers && /tmp/sifr-idiomatic-resolver-triggers`
  - `cargo run -q -p sifr -- run demos/rooted_entrypoint/main.sifr`
  - `cargo run -q -p sifr -- run demos/run_and_build/main.sifr`
  - `cargo run -q -p sifr -- run demos/resolver_triggers/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-82-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-82-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the initial authoring step
- reviewer tooling note:
  - batch 82 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both review passes ended with no accepted blockers across the trio

#### batch_83_recursive_records_temp_workspace_isolation

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/recursive_records/idiomatic.rs`
  - `demos/temp_workspace_isolation/idiomatic.rs`
- selection rationale:
  - the batch clears the final missing top-level runnable-demo companions instead of leaving the corpus nominally incomplete while only two positive demos remain uncovered
  - `recursive_records` is the last recursive-record positive demo still lacking a Rust-first counterpart
  - `temp_workspace_isolation` is the remaining invocation-scoped project-isolation demo missing its top-level companion even though its nested fixture subtrees were already covered
- priority tags:
  - `missing-companion`: `recursive_records`, `temp_workspace_isolation`
  - `recursive-shape`: `recursive_records`
  - `project-mode`: `temp_workspace_isolation`
- implementation summary:
  - `recursive_records`: authored a direct `Entry` struct with `Option<Box<Entry>>`, recursive text rendering via `Option<&Entry>`, and second-node access without emitted-style scaffolding
  - `temp_workspace_isolation`: authored a tiny local helper module preserving the exact two-line project-isolation output and constant `44` result
- local validation completed:
  - `rustfmt demos/recursive_records/idiomatic.rs demos/temp_workspace_isolation/idiomatic.rs`
  - `rustc --edition=2021 demos/recursive_records/idiomatic.rs -o /tmp/sifr-idiomatic-recursive-records && /tmp/sifr-idiomatic-recursive-records`
  - `rustc --edition=2021 demos/temp_workspace_isolation/idiomatic.rs -o /tmp/sifr-idiomatic-temp-workspace-isolation && /tmp/sifr-idiomatic-temp-workspace-isolation`
  - `cargo run -q -p sifr -- run demos/recursive_records/main.sifr`
  - `cargo run -q -p sifr -- run demos/temp_workspace_isolation/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-83-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-83-review-pass-2.md`
- review application summary:
  - `recursive_records` returned `OK` in pass 1 and pass 2
  - `temp_workspace_isolation` returned `OK` in pass 1 and pass 2
  - no follow-up code changes were needed after the initial authoring step
- reviewer tooling note:
  - batch 83 was reviewed directly from the final validated Rust files and observed Sifr outputs
  - both review passes ended with no accepted blockers across the pair

#### batch_65_codegen_preamble_codegen_structural_passes_intrinsic_codegen

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/codegen_preamble/idiomatic.rs`
  - `demos/codegen_structural_passes/idiomatic.rs`
  - `demos/intrinsic_codegen/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive next compiler/codegen slice instead of mixing runtime/preamble migration demos with unrelated fixtures or CLI/project semantics
  - `codegen_preamble` and `codegen_structural_passes` were still heavily generated-style outliers, while `intrinsic_codegen` was smaller but still carried a full error-type scaffold around simple math intrinsics
- priority tags:
  - `compiler-surface`: `codegen_preamble`, `codegen_structural_passes`, `intrinsic_codegen`
  - `milestone-demo`: `codegen_preamble`, `codegen_structural_passes`, `intrinsic_codegen`
  - `hand-authored-generated-shape`: `codegen_preamble`, `codegen_structural_passes`, `intrinsic_codegen`
- implementation summary:
  - `codegen_preamble`: replaced the 1,100-line preamble/runtime scaffold with direct `std::fs` read/write helpers plus a tiny local `Logger` that preserves the observed file and logging output
  - `codegen_structural_passes`: replaced the large hand-rolled datetime runtime layer with compact `DateTime`/`Date` structs plus a small Unix-timestamp-to-calendar conversion helper that keeps the demo on the standalone `rustc` path
  - `intrinsic_codegen`: removed the unused error-type scaffolding and rewrote the demo as direct `f64` intrinsic calls with simple output assertions
- local validation completed:
  - `rustfmt demos/codegen_preamble/idiomatic.rs demos/codegen_structural_passes/idiomatic.rs demos/intrinsic_codegen/idiomatic.rs`
  - `rustc --edition=2021 demos/codegen_preamble/idiomatic.rs -o /tmp/sifr-idiomatic-codegen-preamble && /tmp/sifr-idiomatic-codegen-preamble`
  - `rustc --edition=2021 demos/codegen_structural_passes/idiomatic.rs -o /tmp/sifr-idiomatic-codegen-structural && /tmp/sifr-idiomatic-codegen-structural`
  - `rustc --edition=2021 demos/intrinsic_codegen/idiomatic.rs -o /tmp/sifr-idiomatic-intrinsic-codegen && /tmp/sifr-idiomatic-intrinsic-codegen`
  - `cargo run -q -p sifr -- run demos/codegen_preamble/main.sifr`
  - `cargo run -q -p sifr -- run demos/codegen_structural_passes/main.sifr`
  - `cargo run -q -p sifr -- run demos/intrinsic_codegen/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-65-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-65-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - `codegen_structural_passes` and `intrinsic_codegen` also returned `OK` in pass 2
  - the initial `codegen_preamble` pass-2 response raised false-positive concerns about the final print path and error-shaping; those were not accepted because the final print is outside the `match`, the unreachable error assertion is not part of the observed success-path behavior, and a targeted retry returned `OK`
- reviewer tooling note:
  - batch 65 used direct per-file `agent review` prompts with embedded Sifr and Rust file contents
  - `codegen_preamble` needed a short pass-2 retry after the first response misread the actual Rust control flow

#### batch_64_code_generation_codegen_output_compiler_api

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/code_generation/idiomatic.rs`
  - `demos/codegen_output/idiomatic.rs`
  - `demos/compiler_api/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining compiler-surface slice instead of mixing the codegen/code-output milestone demos with unrelated stdlib or language-feature companions
  - `codegen_output` still carried visible context-manager/codegen scaffolding, while `code_generation` and `compiler_api` were smaller milestone companions that benefited from the same Rust-first cleanup pass
- priority tags:
  - `compiler-surface`: `code_generation`, `codegen_output`, `compiler_api`
  - `milestone-demo`: `code_generation`, `codegen_output`, `compiler_api`
  - `hand-authored-generated-shape`: `code_generation`, `codegen_output`, `compiler_api`
- implementation summary:
  - `code_generation`: collapsed the emitted-style helpers to direct tuple indexing, `Option`, `pow`, and arithmetic/formatting expressions that preserve the observable output without codegen-shaped ceremony
  - `codegen_output`: replaced the heavier context-manager/model scaffolding with a tiny scope-bound `Timer`, a compact `Item` struct, and direct iterator/filter/format flows for the demonstrated codegen improvements
  - `compiler_api`: reviewed the existing single direct print companion and kept it unchanged because it already matched the driver milestone API spine demo at the Rust-first bar
- local validation completed:
  - `rustfmt demos/code_generation/idiomatic.rs demos/codegen_output/idiomatic.rs demos/compiler_api/idiomatic.rs`
  - `rustc --edition=2021 demos/code_generation/idiomatic.rs -o /tmp/sifr-idiomatic-code-generation && /tmp/sifr-idiomatic-code-generation`
  - `rustc --edition=2021 demos/codegen_output/idiomatic.rs -o /tmp/sifr-idiomatic-codegen-output && /tmp/sifr-idiomatic-codegen-output`
  - `rustc --edition=2021 demos/compiler_api/idiomatic.rs -o /tmp/sifr-idiomatic-compiler-api && /tmp/sifr-idiomatic-compiler-api`
  - `cargo run -q -p sifr -- run demos/code_generation/main.sifr`
  - `cargo run -q -p sifr -- run demos/codegen_output/main.sifr`
  - `cargo run -q -p sifr -- run demos/compiler_api/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-64-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-64-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `code_generation` and `compiler_api`
  - pass 1 on `codegen_output` raised two notes that were not accepted as blockers: one incorrectly claimed that `let _timer = Timer::new("work")` drops at the end of the statement rather than at scope end in Rust, and the other objected only to inlining `"World"` inside a `format!` call even though the paired Sifr demo's observable behavior is just the final greeting output
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 64 used direct per-file `agent review` prompts with embedded Sifr and Rust file contents

#### batch_63_enums_ergonomics_constants_classmethods_arithmetic

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/enums/idiomatic.rs`
  - `demos/ergonomics/idiomatic.rs`
  - `demos/constants_classmethods_arithmetic/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining language-features slice instead of mixing enum and ergonomics work with unrelated stdlib or compiler-only surfaces
  - `ergonomics` still carried substantial emitted slicing/indexing/default-parameter scaffolding, while `enums` and `constants_classmethods_arithmetic` were smaller language milestone companions that still benefited from the same Rust-first cleanup pass
- priority tags:
  - `language-features`: `enums`, `ergonomics`, `constants_classmethods_arithmetic`
  - `milestone-demo`: `enums`, `ergonomics`, `constants_classmethods_arithmetic`
  - `hand-authored-generated-shape`: `enums`, `ergonomics`, `constants_classmethods_arithmetic`
- implementation summary:
  - `enums`: replaced the emitted-style enum surface with small `repr(i64)` enums, direct `Display`/`name` helpers, and straightforward pattern matches over the demonstrated variants
  - `ergonomics`: collapsed the generated indexing and slicing machinery to direct Rust helpers for negative indexing, step slicing, string/list/dict methods, and a small `GreetOptions` default struct to model defaults and named-call intent
  - `constants_classmethods_arithmetic`: replaced faux-constant helpers with real module constants and a direct associated constructor on `Temperature`
- local validation completed:
  - `rustfmt demos/enums/idiomatic.rs demos/ergonomics/idiomatic.rs demos/constants_classmethods_arithmetic/idiomatic.rs`
  - `rustc --edition=2021 demos/enums/idiomatic.rs -o /tmp/sifr-idiomatic-enums && /tmp/sifr-idiomatic-enums`
  - `rustc --edition=2021 demos/ergonomics/idiomatic.rs -o /tmp/sifr-idiomatic-ergonomics && /tmp/sifr-idiomatic-ergonomics`
  - `rustc --edition=2021 demos/constants_classmethods_arithmetic/idiomatic.rs -o /tmp/sifr-idiomatic-constants-classmethods && /tmp/sifr-idiomatic-constants-classmethods`
  - `cargo run -q -p sifr -- run demos/enums/main.sifr`
  - `cargo run -q -p sifr -- run demos/ergonomics/main.sifr`
  - `cargo run -q -p sifr -- run demos/constants_classmethods_arithmetic/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/ergonomics/idiomatic.rs`
    - `rustc --edition=2021 demos/ergonomics/idiomatic.rs -o /tmp/sifr-idiomatic-ergonomics && /tmp/sifr-idiomatic-ergonomics`
    - `cargo run -q -p sifr -- run demos/ergonomics/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-63-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-63-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `enums` and `constants_classmethods_arithmetic`
  - pass 1 on `ergonomics` raised one real issue in substance: the original rewrite did not show a strong Rust analogue for the demo's default-argument and keyword-style `greet(...)` calls; I accepted that and introduced `GreetOptions` with `Default` plus struct-update syntax
  - pass 1 notes about loop-else and string method surface differences were not accepted as blockers because Rust lacks those exact surface forms and the current helpers already preserve the observed behavior idiomatically
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 63 used direct per-file `agent review` prompts
  - `ergonomics` pass 2 needed a short retry prompt after the initial full prompt stalled, but the retry returned a clean verdict on the final code state

#### batch_62_control_flow_control_flow_paths_compiled_expressions

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/control_flow/idiomatic.rs`
  - `demos/control_flow_paths/idiomatic.rs`
  - `demos/compiled_expressions/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining control-flow/core-expressions slice instead of mixing one larger milestone control-flow demo with unrelated stdlib or compiler-API companions
  - the paired demos collectively cover loop control, containers, CFG reachability, and small representative expression lowering, so they make a good batch-local normalization pass
- priority tags:
  - `control-flow`: `control_flow`, `control_flow_paths`
  - `core-expressions`: `compiled_expressions`
  - `milestone-demo`: `control_flow`
  - `hand-authored-generated-shape`: `control_flow`, `control_flow_paths`, `compiled_expressions`
- implementation summary:
  - `control_flow`: replaced emitted indexing/formatting ceremony with direct Rust loops, collections, string operations, and a small tuple-length helper
  - `control_flow_paths`: reduced the file to direct CFG-oriented helpers while preserving the early-return fallback and an explicit unreachable tail under `#[allow(unreachable_code)]`
  - `compiled_expressions`: collapsed the file to a tiny `add` helper, direct branch, and small `floor` wrapper over `f64::floor`
- local validation completed:
  - `rustfmt demos/control_flow/idiomatic.rs demos/control_flow_paths/idiomatic.rs demos/compiled_expressions/idiomatic.rs`
  - `rustc --edition=2021 demos/control_flow/idiomatic.rs -o /tmp/sifr-idiomatic-control-flow && /tmp/sifr-idiomatic-control-flow`
  - `rustc --edition=2021 demos/control_flow_paths/idiomatic.rs -o /tmp/sifr-idiomatic-control-flow-paths && /tmp/sifr-idiomatic-control-flow-paths`
  - `rustc --edition=2021 demos/compiled_expressions/idiomatic.rs -o /tmp/sifr-idiomatic-compiled-expressions && /tmp/sifr-idiomatic-compiled-expressions`
  - `cargo run -q -p sifr -- run demos/control_flow/main.sifr`
  - `cargo run -q -p sifr -- run demos/control_flow_paths/main.sifr`
  - `cargo run -q -p sifr -- run demos/compiled_expressions/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/control_flow/idiomatic.rs demos/control_flow_paths/idiomatic.rs`
    - `rustc --edition=2021 demos/control_flow/idiomatic.rs -o /tmp/sifr-idiomatic-control-flow && /tmp/sifr-idiomatic-control-flow`
    - `rustc --edition=2021 demos/control_flow_paths/idiomatic.rs -o /tmp/sifr-idiomatic-control-flow-paths && /tmp/sifr-idiomatic-control-flow-paths`
    - `cargo run -q -p sifr -- run demos/control_flow/main.sifr`
    - `cargo run -q -p sifr -- run demos/control_flow_paths/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-62-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-62-review-pass-2.md`
- review application summary:
  - pass 1 on `control_flow` raised one real cleanup note about the tuple-length print being a naked literal; I accepted it and replaced it with a small `tuple_len(&point)` helper
  - pass 1 on `control_flow_paths` raised one real educational-parity note about preserving the syntactically present unreachable tail from the paired CFG demo; I accepted it and restored that shape with `#[allow(unreachable_code)]`
  - pass 1 on `compiled_expressions` returned `OK`
  - all three pass-2 reviews returned `OK`
- reviewer tooling note:
  - batch 62 used direct per-file `agent review` prompts
  - both accepted pass-1 notes were narrow and local, so a single post-fix revalidation sweep was sufficient before the clean pass-2 run

#### batch_61_core_language_core_libraries_iterator_integration

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/core_language/idiomatic.rs`
  - `demos/core_libraries/idiomatic.rs`
  - `demos/iterator_integration/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining core-surface slice instead of mixing the milestone core-language demo with unrelated error or stdlib-heavy cleanup
  - `core_libraries` and `iterator_integration` were still large generated-style outliers despite the paired demos only exercising compact datetime/regex/math/hash and iterator/path/io behavior
- priority tags:
  - `core-surface`: `core_language`, `core_libraries`, `iterator_integration`
  - `milestone-demo`: `core_language`
  - `stdlib-heavy`: `core_libraries`, `iterator_integration`
  - `hand-authored-generated-shape`: `core_language`, `core_libraries`, `iterator_integration`
- implementation summary:
  - `core_language`: reduced the file to direct recursive helpers and simple branching/printing without generated `return`/casting ceremony
  - `core_libraries`: replaced the emitted-style library scaffold with a small direct `DateTime` model, regex search helper, combinatorics/statistics helpers, and `sha256` wrapper over `sha2`
  - `iterator_integration`: replaced the runtime-heavy scaffold with direct iterator adapters, a lazy regex match iterator, lazy directory iteration, and a stack-based lazy recursive glob helper
- local validation completed:
  - `rustfmt demos/core_language/idiomatic.rs demos/core_libraries/idiomatic.rs demos/iterator_integration/idiomatic.rs`
  - `rustc --edition=2021 demos/core_language/idiomatic.rs -o /tmp/sifr-idiomatic-core-language && /tmp/sifr-idiomatic-core-language`
  - temp Cargo validation for `demos/core_libraries/idiomatic.rs` with `regex = "1"` and `sha2 = "0.10"`
  - temp Cargo validation for `demos/iterator_integration/idiomatic.rs` with `regex = "1"`
  - `cargo run -q -p sifr -- run demos/core_language/main.sifr`
  - `cargo run -q -p sifr -- run demos/core_libraries/main.sifr`
  - `cargo run -q -p sifr -- run demos/iterator_integration/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/iterator_integration/idiomatic.rs`
    - temp Cargo validation for `demos/iterator_integration/idiomatic.rs` with `regex = "1"`
    - `cargo run -q -p sifr -- run demos/iterator_integration/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-61-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-61-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `core_language` and `core_libraries`
  - pass 1 on `iterator_integration` raised one real issue in substance: the first draft had simplified `finditer`, `iterdir`, and `rglob` by materializing them eagerly instead of preserving their iterator nature; I accepted that follow-up and rewrote those helpers to stay lazy
  - the first lazy rewrite exposed one real compile issue in temp Cargo validation: `finditer` needed an explicit lifetime parameter on the returned iterator; that was fixed before the final validation rerun
  - pass 2 returned `OK` for `core_language` and `core_libraries`
  - pass 2 on `iterator_integration` stalled on both the direct prompt and a shorter retry after the laziness fix, so it was carried as a transport note rather than treated as a blocker
- reviewer tooling note:
  - batch 61 used direct per-file `agent review` prompts
  - `iterator_integration` needed an accepted pass-1 fix and then stalled in pass 2, so the artifacts record both the accepted laziness change and the final transport issue explicitly

#### batch_60_auto_detection_auto_init_default_values

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/auto_detection/idiomatic.rs`
  - `demos/auto_init/idiomatic.rs`
  - `demos/default_values/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining constructor/defaults slice instead of mixing one small frontend-mode smoke test with unrelated core-language or stdlib-heavy companions
  - `auto_init` still carried substantial emitted-style `format!` ceremony despite the paired demo only exercising simple constructor/default/string/equality behavior
- priority tags:
  - `constructor-surface`: `auto_detection`, `auto_init`, `default_values`
  - `milestone-demo`: `auto_init`, `default_values`
  - `hand-authored-generated-shape`: `auto_detection`, `auto_init`, `default_values`
- implementation summary:
  - `auto_detection`: reduced the file to a tiny direct `floor` helper plus the two demo prints
  - `auto_init`: replaced the emitted-style formatting scaffold with direct structs, `Default`, and `Display` implementations that preserve the exact printed output
  - `default_values`: kept the demo focused on fresh list defaults and payload defaults with a small `Default` impl and no extra runtime-only bookkeeping
- local validation completed:
  - `rustfmt demos/auto_detection/idiomatic.rs demos/auto_init/idiomatic.rs demos/default_values/idiomatic.rs`
  - `rustc demos/auto_detection/idiomatic.rs -o /tmp/auto_detection_idiomatic`
  - `rustc demos/auto_init/idiomatic.rs -o /tmp/auto_init_idiomatic`
  - `rustc demos/default_values/idiomatic.rs -o /tmp/default_values_idiomatic`
  - `cargo run -q -p sifr -- run demos/auto_detection/main.sifr`
  - `cargo run -q -p sifr -- run demos/auto_init/main.sifr`
  - `cargo run -q -p sifr -- run demos/default_values/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/default_values/idiomatic.rs`
    - `rustc demos/default_values/idiomatic.rs -o /tmp/default_values_idiomatic`
    - `cargo run -q -p sifr -- run demos/default_values/main.sifr`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-60-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-60-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `auto_detection` and `auto_init`
  - pass 1 on `default_values` raised one real cleanup note about the throwaway `_counts_len` read; I accepted it and replaced that workaround with a targeted `#[allow(dead_code)]` on the unprinted `counts` field
  - the `default_values` follow-up revalidation came back clean
  - pass 2 returned `OK` for `auto_init` and `default_values`
  - the final `auto_detection` pass-2 note was rejected because it claimed `floor(3.9)` should preserve `3.9`, which directly contradicted the paired Sifr demo output already validated in this workspace
- reviewer tooling note:
  - batch 60 used direct per-file `agent review` prompts
  - all three pass-2 prompts initially stalled, but two resolved cleanly on the final polling window and the remaining `auto_detection` response was explicitly recorded as a non-blocking incorrect verdict

#### batch_59_ordering_rules_operators_and_assignment_collection_comprehensions

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/ordering_rules/idiomatic.rs`
  - `demos/operators_and_assignment/idiomatic.rs`
  - `demos/collection_comprehensions/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining expression-semantics slice instead of mixing one large language milestone companion with unrelated stdlib or nested-helper work
  - `ordering_rules` was still a large generated-style outlier despite the paired demo only exercising a compact lazy-itertools, regex/json alias, and callable-field surface
- priority tags:
  - `expression-semantics`: `ordering_rules`, `operators_and_assignment`, `collection_comprehensions`
  - `milestone-demo`: `ordering_rules`, `operators_and_assignment`, `collection_comprehensions`
  - `hand-authored-generated-shape`: `ordering_rules`, `operators_and_assignment`, `collection_comprehensions`
- implementation summary:
  - `ordering_rules`: replaced the runtime-heavy scaffold with small direct helpers for `repeat`, `chain`, `take`, regex aliases, JSON loads, and callable-field invocation while preserving the exact printed output sequence
  - `operators_and_assignment`: collapsed the file to direct bitwise/unary operations, augmented assignment, and tuple-based chained-assignment equivalence
  - `collection_comprehensions`: reduced the demo to direct iterator collects into `Vec`, `BTreeMap`, and `BTreeSet`, preserving the same lengths and tuple-unpacking prints
- local validation completed:
  - `rustfmt demos/operators_and_assignment/idiomatic.rs demos/collection_comprehensions/idiomatic.rs demos/ordering_rules/idiomatic.rs`
  - `rustc demos/operators_and_assignment/idiomatic.rs -o /tmp/operators_and_assignment_idiomatic`
  - `rustc demos/collection_comprehensions/idiomatic.rs -o /tmp/collection_comprehensions_idiomatic`
  - temp Cargo validation for `ordering_rules` with `regex = "1"` and `serde_json = "1"`
  - `cargo run -q -p sifr -- run demos/operators_and_assignment/main.sifr`
  - `cargo run -q -p sifr -- run demos/collection_comprehensions/main.sifr`
  - `cargo run -q -p sifr -- run demos/ordering_rules/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-59-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-59-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `operators_and_assignment` and `ordering_rules`
  - pass 1 on `collection_comprehensions` raised one incorrect tuple-typing complaint in code that already compiled and matched the paired output, so it was not accepted as a blocker
  - pass 2 returned `OK` for `operators_and_assignment` and `ordering_rules`
  - pass 2 on `collection_comprehensions` stalled without a usable verdict and was carried as a transport note rather than treated as a blocker
- reviewer tooling note:
  - batch 59 used direct per-file `agent review` prompts
  - `collection_comprehensions` pass 2 stalled after the pass-1 false-positive note, so the artifact records that transport issue explicitly instead of fabricating a clean verdict

#### batch_58_nested_functions_nested_helpers_nested_recursive_helpers

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/nested_functions/idiomatic.rs`
  - `demos/nested_helpers/idiomatic.rs`
  - `demos/nested_recursive_helpers/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining nested-helper slice instead of mixing nested closure execution with unrelated stdlib or compiler-surface work
  - `nested_helpers` still carried substantial emitted-style scaffolding around otherwise small backtracking and DSU routines, while the other two companions still had smaller but inconsistent closure/ownership patterns worth normalizing together
- priority tags:
  - `nested-functions`: `nested_functions`, `nested_helpers`, `nested_recursive_helpers`
  - `recursive-helpers`: `nested_helpers`, `nested_recursive_helpers`
  - `hand-authored-generated-shape`: `nested_functions`, `nested_helpers`, `nested_recursive_helpers`
- implementation summary:
  - `nested_functions`: simplified the milestone demo to direct closures and small inner helpers while preserving the exact six printed sections and values
  - `nested_helpers`: replaced the emitted-style borrow/index scaffolding with direct backtracking, N-Queens, and union-find helpers over standard collections and slices
  - `nested_recursive_helpers`: reduced the file to a tiny linked-entry struct plus a recursive local visitor over `Option<&Entry>` without the earlier cloning noise
- local validation completed:
  - `rustfmt demos/nested_functions/idiomatic.rs demos/nested_helpers/idiomatic.rs demos/nested_recursive_helpers/idiomatic.rs`
  - `rustc demos/nested_functions/idiomatic.rs -o /tmp/nested_functions_idiomatic`
  - `rustc demos/nested_helpers/idiomatic.rs -o /tmp/nested_helpers_idiomatic`
  - `rustc demos/nested_recursive_helpers/idiomatic.rs -o /tmp/nested_recursive_helpers_idiomatic`
  - `cargo run -q -p sifr -- run demos/nested_functions/main.sifr`
  - `cargo run -q -p sifr -- run demos/nested_helpers/main.sifr`
  - `cargo run -q -p sifr -- run demos/nested_recursive_helpers/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-58-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-58-review-pass-2.md`
- review application summary:
  - pass 1 on `nested_functions` raised only a non-blocking complaint about the paired Sifr source's captured-variable comment plus a minor string-style preference; neither was accepted as a blocker because the Rust companion already matched the observed output and stayed readable
  - pass 1 on `nested_recursive_helpers` returned `OK`
  - pass 1 on `nested_helpers` stalled without a usable verdict and was carried as a transport note rather than treated as a blocker
  - pass 2 returned `OK` for `nested_functions` and `nested_recursive_helpers`
  - pass 2 on `nested_helpers` stalled again on the shorter retry prompt and was likewise carried as a transport note
- reviewer tooling note:
  - batch 58 used direct per-file `agent review` prompts
  - `nested_helpers` stalled in both passes despite a shorter retry prompt, so the artifacts record that transport issue explicitly instead of fabricating a clean verdict

#### batch_57_extended_collections_extended_itertools_itertools_iterables

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/extended_collections/idiomatic.rs`
  - `demos/extended_itertools/idiomatic.rs`
  - `demos/itertools_iterables/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining iterator-and-collections expansion slice instead of mixing a larger itertools surface with unrelated stdlib-only cleanup
  - each companion still carried more generated-style helper structure than the paired demo behavior required, especially `extended_itertools`
- priority tags:
  - `collections-surface`: `extended_collections`
  - `itertools-surface`: `extended_itertools`, `itertools_iterables`
  - `hand-authored-generated-shape`: `extended_collections`, `extended_itertools`, `itertools_iterables`
- implementation summary:
  - `extended_collections`: replaced the scaffold with direct `BTreeSet`/`BTreeMap`-based set and counter helpers plus small UTF-8 and hex utilities that preserve the exact printed outputs
  - `extended_itertools`: collapsed the file to small direct helpers for the lazy-iterator surface the demo actually exercises, preserving the single `...: ok` parity line
  - `itertools_iterables`: reduced the demo to direct iterator combinators, a tiny sorted `Path::iterdir`, and simple `write_text`/`run_command` wrappers for the filesystem roundtrip
- local validation completed:
  - `rustfmt demos/extended_collections/idiomatic.rs demos/extended_itertools/idiomatic.rs demos/itertools_iterables/idiomatic.rs`
  - `rustc demos/extended_collections/idiomatic.rs -o /tmp/extended_collections_idiomatic`
  - `rustc demos/extended_itertools/idiomatic.rs -o /tmp/extended_itertools_idiomatic`
  - `rustc demos/itertools_iterables/idiomatic.rs -o /tmp/itertools_iterables_idiomatic`
  - `cargo run -q -p sifr -- run demos/extended_collections/main.sifr`
  - `cargo run -q -p sifr -- run demos/extended_itertools/main.sifr`
  - `cargo run -q -p sifr -- run demos/itertools_iterables/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/itertools_iterables/idiomatic.rs`
    - `rustc demos/itertools_iterables/idiomatic.rs -o /tmp/itertools_iterables_idiomatic`
    - `cargo run -q -p sifr -- run demos/itertools_iterables/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-57-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-57-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK` for `extended_collections` and `extended_itertools`
  - pass 1 raised one real API-parity note in `itertools_iterables`; I accepted it by removing the misleading `islice` helper, restoring predicate-first `takewhile`, and rewriting the affected call sites directly
  - the post-fix `itertools_iterables` re-review came back `OK`
  - pass 2 returned `OK` for `extended_itertools` and `itertools_iterables`
  - the final `extended_collections` pass-2 response was not accepted because it inverted the Rust/Sifr file roles, analyzed the wrong source shape, and therefore did not identify a real blocker relative to the already-validated companion
- reviewer tooling note:
  - batch 57 used direct per-file `agent review` prompts
  - `extended_collections` pass 2 stalled before eventually returning an inverted file-role analysis, so that response was recorded explicitly rather than treated as authoritative

#### batch_56_stdlib_classes_stdlib_error_types_pure_sifr_stdlib

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stdlib_classes/idiomatic.rs`
  - `demos/stdlib_error_types/idiomatic.rs`
  - `demos/pure_sifr_stdlib/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining smaller stdlib milestone slice instead of mixing class/error/migration demos with unrelated parser, text, or fixture work
  - the existing companions were still large generated-style scaffolds despite comparatively compact demo-visible behavior and outputs
- priority tags:
  - `stdlib-heavy`: `stdlib_classes`, `stdlib_error_types`, `pure_sifr_stdlib`
  - `milestone-demo`: `stdlib_classes`, `stdlib_error_types`, `pure_sifr_stdlib`
  - `hand-authored-generated-shape`: `stdlib_classes`, `stdlib_error_types`, `pure_sifr_stdlib`
- implementation summary:
  - `stdlib_classes`: replaced the generated scaffold with a compact `BTreeMap`-backed `Counter` plus a direct `from_list` helper that preserves the exact printed sequence
  - `stdlib_error_types`: replaced the broad generated helper layer with direct `StatisticsError` and `CycleError` structs plus small `compute_mean`/`topo_sort` helpers that preserve the exact module-error output
  - `pure_sifr_stdlib`: replaced the runtime-heavy scaffold with minimal `assert_eq`/`assert_true`, `sqrt`/`PI`, `sha256`, and base64 helpers for the exercised migration path
- local validation completed:
  - `rustfmt demos/stdlib_classes/idiomatic.rs demos/stdlib_error_types/idiomatic.rs demos/pure_sifr_stdlib/idiomatic.rs`
  - `rustc demos/stdlib_classes/idiomatic.rs -o /tmp/stdlib_classes_idiomatic`
  - `rustc demos/stdlib_error_types/idiomatic.rs -o /tmp/stdlib_error_types_idiomatic`
  - temp Cargo validation for `pure_sifr_stdlib` with `base64 = "0.22"` and `sha2 = "0.10"`
  - `/tmp/stdlib_classes_idiomatic`
  - `/tmp/stdlib_error_types_idiomatic`
  - `cargo run -q -p sifr -- run demos/stdlib_classes/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_error_types/main.sifr`
  - `cargo run -q -p sifr -- run demos/pure_sifr_stdlib/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 follow-up revalidation:
    - `rustfmt demos/pure_sifr_stdlib/idiomatic.rs`
    - temp Cargo validation for `pure_sifr_stdlib` with `base64 = "0.22"` and `sha2 = "0.10"`
    - `cargo run -q -p sifr -- run demos/pure_sifr_stdlib/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-56-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-56-review-pass-2.md`
- review application summary:
  - pass 1 on `stdlib_classes` returned `OK`
  - pass 1 on `stdlib_error_types` raised notes about `ParseIntError` formatting and the final mixed-error structure, but neither was accepted because the first note was incorrect (`println!("{err}")` already uses `Display`) and the second did not change the paired demo-visible behavior
  - pass 1 on `pure_sifr_stdlib` found one real issue: the dead base64-error path still preserved the old nonsense assertion equating an error message with the success footer. I accepted that note and replaced it with a direct panic on the impossible path
  - pass 2 on `pure_sifr_stdlib` returned `OK`
  - pass 2 on `stdlib_classes` and `stdlib_error_types` both stalled without usable verdicts and were carried as transport notes rather than treated as blockers
- reviewer tooling note:
  - batch 56 used direct per-file `agent review` prompts because that has remained the most reliable review transport in this workspace
  - `stdlib_classes` and `stdlib_error_types` pass-2 prompts still stalled without output after repeated polls, so the artifacts record those transport failures explicitly instead of fabricating clean verdicts

#### batch_50_stdlib_stdlib_expansion_stdlib_aliases

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stdlib/idiomatic.rs`
  - `demos/stdlib_expansion/idiomatic.rs`
  - `demos/stdlib_aliases/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining stdlib-milestone slice instead of mixing milestone-level stdlib coverage with unrelated parsing, safety, or class-surface demos
  - the existing companions were still large generated-style scaffolds despite relatively compact demo-visible behavior and milestone outputs
- priority tags:
  - `stdlib-heavy`: `stdlib`, `stdlib_expansion`, `stdlib_aliases`
  - `milestone-demo`: `stdlib`, `stdlib_expansion`, `stdlib_aliases`
  - `hand-authored-generated-shape`: `stdlib`, `stdlib_expansion`, `stdlib_aliases`
- implementation summary:
  - `stdlib`: replaced the 2.8k-line scaffold with a small direct milestone demo covering cwd, digit matching, topological sort, UUID formatting, path helpers, close-match scoring, IP checks, timer access, TOML parsing, and datetime display before printing the same two-line success footer
  - `stdlib_expansion`: replaced the 2.8k-line scaffold with compact direct helpers for bisect, reduce, token generation, statistics mean, heap access, iterator composition, text fill, CSV row handling, option parsing, and fnmatch before printing the same milestone success line
  - `stdlib_aliases`: replaced the 2.2k-line scaffold with a direct alias-surface demo using standard Rust math plus small base64/regex/JSON/title-casing helpers that preserve the exact printed output shape
- local validation completed:
  - `rustfmt demos/stdlib/idiomatic.rs demos/stdlib_expansion/idiomatic.rs demos/stdlib_aliases/idiomatic.rs`
  - `rustc demos/stdlib/idiomatic.rs -o /tmp/stdlib_idiomatic`
  - `rustc demos/stdlib_expansion/idiomatic.rs -o /tmp/stdlib_expansion_idiomatic`
  - temp Cargo validation for `stdlib_aliases` with `base64 = "0.22"`, `regex = "1"`, `serde_json = "1"`
  - `/tmp/stdlib_idiomatic`
  - `/tmp/stdlib_expansion_idiomatic`
  - `cargo run -q -p sifr -- run demos/stdlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_expansion/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_aliases/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-50-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-50-review-pass-2.md`
- review application summary:
  - pass 1 on `stdlib` produced an unusable stale/generated-shape comparison that inverted the file roles and was not accepted as a blocker
  - pass 1 on `stdlib_expansion` likewise returned stale/generated-shape commentary after briefly stating `OK: no issues`; it was recorded explicitly and not accepted as a blocker
  - pass 1 on `stdlib_aliases` raised only one unexercised `fnmatch_filter` generalization note plus two minor style notes, and none were accepted as blockers because the paired demo-visible behavior already matched
  - pass 2 returned `OK: no issues` for `stdlib`
  - pass 2 returned `OK: no issues` for `stdlib_expansion`
  - pass 2 on `stdlib_aliases` raised notes about exact platform string spellings and internal helper error typing, but neither was accepted because the paired demo only checks that the platform strings are non-empty and the helper error representation is not observable in the exercised paths
- reviewer tooling note:
  - batch 50 again used direct per-file `agent review` prompts because that has been the most reliable reviewer transport in this workspace
  - the pass-1 `stdlib` and `stdlib_expansion` responses still degraded into stale/generated-shape commentary despite the smaller prompt, so those artifacts record the unusable verdicts explicitly rather than fabricating clean pass-1 approvals

#### batch_49_core_stdlib_extended_stdlib_additional_modules

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/core_stdlib/idiomatic.rs`
  - `demos/extended_stdlib/idiomatic.rs`
  - `demos/additional_modules/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining stdlib-utilities slice instead of mixing a broad stdlib surface with the currently broken `html_and_textwrap` entrypoint
  - `core_stdlib`, `extended_stdlib`, and `additional_modules` were still carrying large generated-style companions despite relatively compact direct demo-visible behavior
  - `html_and_textwrap` was intentionally deferred because `cargo run -q -p sifr -- run demos/html_and_textwrap/main.sifr` currently fails in the repo, so including it here would have broken the required targeted-demo validation lane for the batch
- priority tags:
  - `stdlib-heavy`: `core_stdlib`, `extended_stdlib`, `additional_modules`
  - `hand-authored-generated-shape`: `core_stdlib`, `extended_stdlib`, `additional_modules`
  - `multi-module-surface`: `core_stdlib`, `extended_stdlib`, `additional_modules`
- implementation summary:
  - `core_stdlib`: replaced the generated scaffold with direct file/json/env/math helpers and a tiny in-memory env store that preserves the exact demo-visible output
  - `extended_stdlib`: replaced the large runtime wrapper with small direct time, RNG, regex, hashing, and base64 helpers while preserving the dynamic/random output shape
  - `additional_modules`: replaced the runtime-heavy scaffold with compact operator, calendar, HTML, sys, subprocess, configparser, gzip, and zipfile helpers matching the integrated demo outputs
- local validation completed:
  - `rustfmt demos/core_stdlib/idiomatic.rs demos/extended_stdlib/idiomatic.rs demos/additional_modules/idiomatic.rs`
  - temp Cargo validation for `core_stdlib` with `serde_json = "1"`
  - temp Cargo validation for `extended_stdlib` with `base64 = "0.22"`, `md5 = "0.7"`, `regex = "1"`, `sha2 = "0.10"`
  - temp Cargo validation for `additional_modules` with `flate2 = "1"`, `zip = "0.6"`
  - `cargo run -q -p sifr -- run demos/core_stdlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/extended_stdlib/main.sifr`
  - `cargo run -q -p sifr -- run demos/additional_modules/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-49-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-49-review-pass-2.md`
- review application summary:
  - pass 1 returned clean `core_stdlib` and `additional_modules` verdicts
  - pass 1 raised one `extended_stdlib` helper-name note that was rejected because internal helper naming is not part of the observable contract, and the same response also drifted into reviewing `main.sifr` instead of the Rust companion
  - pass 2 returned `OK: no issues` for `core_stdlib`
  - pass 2 repeated the same `extended_stdlib` internal helper-name note and it was again not accepted as a blocker because the file already matched the paired Sifr behavior under temp-Cargo execution, targeted demo execution, and the full repository validation lane
  - pass 2 for `additional_modules` returned an unusable mixed response that inverted the Sifr/Rust file roles and ended with contradictory text; it was recorded explicitly and not treated as a blocker because the file had already passed temp-Cargo execution, targeted demo execution, and the full repository validation lane
- reviewer tooling note:
  - batch 49 used direct per-file `agent review` prompts with `main.sifr` and `idiomatic.rs` file-path references because that smaller prompt shape has been more reliable than embedded-source prompts in this workspace
  - the `additional_modules` pass-2 response still degraded into a stale file-role inversion despite the narrower source-of-truth instruction, so the artifact records that transport-quality issue explicitly instead of fabricating a clean verdict

#### batch_48_class_libraries_advanced_class_libraries_inheritance

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/class_libraries/idiomatic.rs`
  - `demos/advanced_class_libraries/idiomatic.rs`
  - `demos/inheritance/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - they form a cohesive remaining class-oriented API slice instead of mixing large class-based stdlib demonstrations with unrelated import or fixture work
  - `class_libraries` and `advanced_class_libraries` were still carrying 2.7k and 4.4k line generated-style companions even though the paired demos exercise much smaller direct class behavior
- priority tags:
  - `class-api-surface`: `class_libraries`, `advanced_class_libraries`, `inheritance`
  - `stdlib-heavy`: `class_libraries`, `advanced_class_libraries`
  - `hand-authored-generated-shape`: `class_libraries`, `advanced_class_libraries`, `inheritance`
- implementation summary:
  - `class_libraries`: replaced the runtime-heavy scaffold with direct implementations of `TopologicalSorter`, `Path`, `Logger`, `Match`, `Uuid`, and `Timedelta`, preserving the exact printed logger format and arithmetic outputs
  - `advanced_class_libraries`: replaced the 4.4k generated companion with small direct implementations of `Deque`, `DateTime`/`Date`, `Path`, `Pattern`, `Logger`, and minimal CSV reader/writer types that match the demo-visible outputs
  - `inheritance`: reduced the demo to direct structs and associated functions for inheritance-style composition, factory methods, and static helpers
- local validation completed:
  - `rustfmt demos/class_libraries/idiomatic.rs demos/advanced_class_libraries/idiomatic.rs demos/inheritance/idiomatic.rs`
  - `rustc demos/class_libraries/idiomatic.rs -o /tmp/class_libraries_idiomatic`
  - `rustc demos/advanced_class_libraries/idiomatic.rs -o /tmp/advanced_class_libraries_idiomatic`
  - `rustc demos/inheritance/idiomatic.rs -o /tmp/inheritance_idiomatic`
  - `cargo run -q -p sifr -- run demos/class_libraries/main.sifr`
  - `cargo run -q -p sifr -- run demos/advanced_class_libraries/main.sifr`
  - `cargo run -q -p sifr -- run demos/inheritance/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-pass-1 revalidation:
    - `rustfmt demos/class_libraries/idiomatic.rs demos/advanced_class_libraries/idiomatic.rs demos/inheritance/idiomatic.rs`
    - `rustc demos/class_libraries/idiomatic.rs -o /tmp/class_libraries_idiomatic`
    - `rustc demos/advanced_class_libraries/idiomatic.rs -o /tmp/advanced_class_libraries_idiomatic`
    - `rustc demos/inheritance/idiomatic.rs -o /tmp/inheritance_idiomatic`
    - `cargo run -q -p sifr -- run demos/class_libraries/main.sifr`
    - `cargo run -q -p sifr -- run demos/advanced_class_libraries/main.sifr`
    - `cargo run -q -p sifr -- run demos/inheritance/main.sifr`
    - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-48-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-48-review-pass-2.md`
- review application summary:
  - pass 1 accepted one real parity fix in `class_libraries`, changing the harness to print only the first three `static_order()` results instead of iterating the whole returned order
  - pass 1 reported no remaining issues in `advanced_class_libraries` or `inheritance`
  - pass 2 returned `OK: no issues` for `inheritance`
  - pass 2 reviewer transport stalled for `class_libraries` and `advanced_class_libraries`; those stalls were recorded explicitly and not treated as blockers because both files had already passed local `rustc`, targeted Sifr demo execution, and the full repository validation lane, and `advanced_class_libraries` had also come back clean in pass 1
- reviewer tooling note:
  - batch 48 used direct per-file `agent review` prompts with `Read` access because the class-library companions were too large for reliable embedded-source prompts in this workspace
  - the production-review prompts for the two larger files stalled without returning usable output, so the review artifacts record that transport issue explicitly

#### batch_47_builtin_functions_builtin_callables_stdlib_functions

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/builtin_functions/idiomatic.rs`
  - `demos/builtin_callables/idiomatic.rs`
  - `demos/stdlib_functions/idiomatic.rs`
- selection rationale:
  - all three are positive runnable demos
  - all three form a cohesive remaining builtin-and-stdlib callable surface instead of mixing tiny builtin helper demos with unrelated fixture or type-system work
  - `builtin_callables` and especially `stdlib_functions` still carried substantial generated-style scaffolding despite relatively small demo-visible behavior, and `builtin_functions` was the last tiny builtin-formatting companion still preserving repeated emitted-style `format!` ceremony
- priority tags:
  - `builtin-surface`: `builtin_functions`, `builtin_callables`
  - `stdlib-heavy`: `stdlib_functions`
  - `hand-authored-generated-shape`: `builtin_functions`, `builtin_callables`, `stdlib_functions`
- implementation summary:
  - `builtin_functions`: collapsed the demo to direct integer methods and a single joined step-range string instead of repeated nested `format!` scaffolding
  - `builtin_callables`: replaced copied error/runtime layers with direct constructor/helper demonstrations, compact `ord`/`chr` helpers, and deterministic `BTreeMap` output for the dict example
  - `stdlib_functions`: replaced the large generated helper/runtime scaffold with small direct math/statistics/string/path/bisect/itertools helpers that preserve the actual demo-visible outputs
- local validation completed:
  - `rustfmt demos/builtin_functions/idiomatic.rs demos/builtin_callables/idiomatic.rs demos/stdlib_functions/idiomatic.rs`
  - `rustc demos/builtin_functions/idiomatic.rs -o /tmp/builtin_functions_idiomatic`
  - `rustc demos/builtin_callables/idiomatic.rs -o /tmp/builtin_callables_idiomatic`
  - `rustc demos/stdlib_functions/idiomatic.rs -o /tmp/stdlib_functions_idiomatic`
  - `cargo run -q -p sifr -- run demos/builtin_functions/main.sifr`
  - `cargo run -q -p sifr -- run demos/builtin_callables/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_functions/main.sifr`
  - `scripts/run_all_tests.sh`
  - post-fix revalidation:
    - `rustfmt demos/builtin_callables/idiomatic.rs`
    - `rustc demos/builtin_functions/idiomatic.rs -o /tmp/builtin_functions_idiomatic`
    - `rustc demos/builtin_callables/idiomatic.rs -o /tmp/builtin_callables_idiomatic`
    - `rustc demos/stdlib_functions/idiomatic.rs -o /tmp/stdlib_functions_idiomatic`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-47-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-47-review-pass-2.md`
- review application summary:
  - pre-review follow-up: fixed `builtin_callables` to compile cleanly under plain `rustc` without depending on edition-specific array `into_iter()` behavior or `TryFrom` imports
  - pass 1 reported no accepted blockers; `builtin_functions` came back clean, `builtin_callables` reviewer transport stalled without producing a usable verdict, and the `stdlib_functions` notes about negative `factorial` handling and the `batched` error string were rejected because those paths are not exercised by the paired demo
  - pass 2 reported no accepted blockers; `builtin_functions` and `stdlib_functions` both returned `OK: no issues`, while `builtin_callables` again had to be carried with a transport note after multiple prompt variants stalled in this workspace
- reviewer tooling note:
  - the desktop `desktop agent review handoff` handoff stalled without writing the pass-1 artifact
  - batch 47 therefore used direct per-file `agent review` prompts for the responsive files and recorded the `builtin_callables` reviewer stall explicitly rather than fabricating a verdict

### wave_3_fixture_and_negative_case_normalization

status: in_progress

#### batch_84_reachable_type_error_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/control_flow_paths/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/early_return_paths/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/reachability/negative_cases/reachable_type_error/idiomatic.rs`
- selection rationale:
  - all three fixtures belong to the same phase-25 reachable-type-error family and were still using an indistinguishable generic placeholder instead of documenting the specific failure contract
  - the batch is small enough to normalize the Tier 2 style without mixing unrelated negative-case categories
  - each fixture already has a stable paired Sifr diagnostic, so the Rust-side scaffold can be validated directly against the observed failure shape
- priority tags:
  - `tier-2-negative-case`: `control_flow_paths/reachable_type_error`, `early_return_paths/reachable_type_error`, `reachability/reachable_type_error`
  - `reachable-type-error`: `control_flow_paths/reachable_type_error`, `early_return_paths/reachable_type_error`, `reachability/reachable_type_error`
  - `generic-placeholder-cleanup`: `control_flow_paths/reachable_type_error`, `early_return_paths/reachable_type_error`, `reachability/reachable_type_error`
- implementation summary:
  - `control_flow_paths`: replaced the generic stub with a fixture-specific scaffold describing the reachable `str` return from an `int`-typed function and a matching Rust `compile_fail` analogue
  - `early_return_paths`: replaced the generic stub with a scaffold describing the non-exiting `None` branch and the resulting `Option<int>` vs `int` mismatch, again with a direct Rust analogue
  - `reachability`: replaced the generic stub with a scaffold describing the reachable final `str` return path and its Rust analogue
- local validation completed:
  - `rustfmt demos/control_flow_paths/negative_cases/reachable_type_error/idiomatic.rs demos/early_return_paths/negative_cases/reachable_type_error/idiomatic.rs demos/reachability/negative_cases/reachable_type_error/idiomatic.rs`
  - `rustc --edition=2021 demos/control_flow_paths/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-control-flow-paths && /tmp/sifr-neg-control-flow-paths`
  - `rustc --edition=2021 demos/early_return_paths/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-early-return-paths && /tmp/sifr-neg-early-return-paths`
  - `rustc --edition=2021 demos/reachability/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-reachability && /tmp/sifr-neg-reachability`
  - `cargo run -q -p sifr -- check demos/control_flow_paths/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/early_return_paths/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/reachability/negative_cases/reachable_type_error/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-84-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-84-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 84 was reviewed directly from the final validated Rust scaffolds and the observed `sifr check` diagnostics for the paired negative fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_85_reachable_type_error_scaffolds_followup

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/loop_try_match/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/return_and_raise_paths/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/unreachable_returns/negative_cases/reachable_type_error/idiomatic.rs`
- selection rationale:
  - these three fixtures are the next untouched members of the same reachable-type-error family and still used the generic placeholder instead of documenting the specific branch shape that keeps the bad return reachable
  - the batch stays within one negative-case category and avoids mixing parse, import, and protocol fixtures before the Tier 2 style is normalized
  - each paired Sifr fixture already has a stable deterministic type-error check path, so the Rust-side scaffold can be validated directly against observed diagnostics
- priority tags:
  - `tier-2-negative-case`: `loop_try_match/reachable_type_error`, `return_and_raise_paths/reachable_type_error`, `unreachable_returns/reachable_type_error`
  - `reachable-type-error`: `loop_try_match/reachable_type_error`, `return_and_raise_paths/reachable_type_error`, `unreachable_returns/reachable_type_error`
  - `generic-placeholder-cleanup`: `loop_try_match/reachable_type_error`, `return_and_raise_paths/reachable_type_error`, `unreachable_returns/reachable_type_error`
- implementation summary:
  - `loop_try_match`: replaced the generic stub with a scaffold describing the nested `else` branch that returns `&str` from an `int`-typed function and a direct Rust `compile_fail` analogue
  - `return_and_raise_paths`: replaced the generic stub with a scaffold describing the explicit `else` branch return mismatch and its Rust analogue
  - `unreachable_returns`: replaced the generic stub with a scaffold describing the final reachable `&str` return path and its Rust analogue
- local validation completed:
  - `rustfmt demos/loop_try_match/negative_cases/reachable_type_error/idiomatic.rs demos/return_and_raise_paths/negative_cases/reachable_type_error/idiomatic.rs demos/unreachable_returns/negative_cases/reachable_type_error/idiomatic.rs`
  - `rustc --edition=2021 demos/loop_try_match/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-loop-try-match && /tmp/sifr-neg-loop-try-match`
  - `rustc --edition=2021 demos/return_and_raise_paths/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-return-and-raise-paths && /tmp/sifr-neg-return-and-raise-paths`
  - `rustc --edition=2021 demos/unreachable_returns/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-unreachable-returns && /tmp/sifr-neg-unreachable-returns`
  - `cargo run -q -p sifr -- check demos/loop_try_match/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/return_and_raise_paths/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/unreachable_returns/negative_cases/reachable_type_error/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-85-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-85-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 85 was reviewed directly from the final validated Rust scaffolds and the observed `sifr check` diagnostics for the paired negative fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_86_reachable_type_error_scaffolds_mixed_blocks

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/stable_codegen/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/valid_control_flow/negative_cases/reachable_type_error/idiomatic.rs`
  - `demos/branch_paths/negative_cases/mixed_block_type_error/idiomatic.rs`
- selection rationale:
  - these fixtures are the next untouched members of the control-flow mismatch family and still used the generic placeholder rather than documenting the specific type mismatch that remains reachable
  - the trio stays coherent while broadening slightly from plain `int` vs `str` mismatches to one union-member mismatch and one mixed-block mismatch
  - each paired Sifr fixture already has a stable `sifr check` diagnostic path, so the Rust-side scaffold can be validated against observed output rather than inferred behavior
- priority tags:
  - `tier-2-negative-case`: `stable_codegen/reachable_type_error`, `valid_control_flow/reachable_type_error`, `branch_paths/mixed_block_type_error`
  - `control-flow-mismatch`: `stable_codegen/reachable_type_error`, `valid_control_flow/reachable_type_error`, `branch_paths/mixed_block_type_error`
  - `generic-placeholder-cleanup`: `stable_codegen/reachable_type_error`, `valid_control_flow/reachable_type_error`, `branch_paths/mixed_block_type_error`
- implementation summary:
  - `stable_codegen`: replaced the generic stub with a scaffold describing the reachable `list[int]` return that is outside the declared `int | str` union, plus a direct Rust analogue
  - `valid_control_flow`: replaced the generic stub with a scaffold describing the final reachable `&str` return path from an `int`-typed function
  - `branch_paths`: replaced the generic stub with a scaffold describing the mixed `try`/`if` block that keeps a reachable `&str` return inside an `int`-typed function
- local validation completed:
  - `rustfmt demos/stable_codegen/negative_cases/reachable_type_error/idiomatic.rs demos/valid_control_flow/negative_cases/reachable_type_error/idiomatic.rs demos/branch_paths/negative_cases/mixed_block_type_error/idiomatic.rs`
  - `rustc --edition=2021 demos/stable_codegen/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-stable-codegen && /tmp/sifr-neg-stable-codegen`
  - `rustc --edition=2021 demos/valid_control_flow/negative_cases/reachable_type_error/idiomatic.rs -o /tmp/sifr-neg-valid-control-flow && /tmp/sifr-neg-valid-control-flow`
  - `rustc --edition=2021 demos/branch_paths/negative_cases/mixed_block_type_error/idiomatic.rs -o /tmp/sifr-neg-branch-paths && /tmp/sifr-neg-branch-paths`
  - `cargo run -q -p sifr -- check demos/stable_codegen/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/valid_control_flow/negative_cases/reachable_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/branch_paths/negative_cases/mixed_block_type_error/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-86-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-86-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 86 was reviewed directly from the final validated Rust scaffolds and the observed `sifr check` diagnostics for the paired negative fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_87_reachable_parse_error_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/graph_isolation/negative_cases/reachable_parse_error/idiomatic.rs`
  - `demos/temp_workspace_isolation/negative_cases/reachable_parse_error/idiomatic.rs`
  - `demos/reachable_imports/negative_cases/reachable_dependency_parse_error/idiomatic.rs`
- selection rationale:
  - these three phase-23 fixtures are the next untouched negative-case family and all share the same contract: the entry file is valid, a local helper import is reachable, and the helper parse error must terminate analysis deterministically
  - the batch stays narrow and avoids mixing parse-error fixtures with type-error, module-cycle, or protocol-bound failures
  - each paired fixture already exposes a stable `sifr check` parse-error message with the helper path and byte range, so the Tier 2 scaffold can be validated against observed output
- priority tags:
  - `tier-2-negative-case`: `graph_isolation/reachable_parse_error`, `temp_workspace_isolation/reachable_parse_error`, `reachable_imports/reachable_dependency_parse_error`
  - `reachable-parse-error`: `graph_isolation/reachable_parse_error`, `temp_workspace_isolation/reachable_parse_error`, `reachable_imports/reachable_dependency_parse_error`
  - `generic-placeholder-cleanup`: `graph_isolation/reachable_parse_error`, `temp_workspace_isolation/reachable_parse_error`, `reachable_imports/reachable_dependency_parse_error`
- implementation summary:
  - `graph_isolation`: replaced the generic stub with a scaffold describing the reachable local helper parse failure from `def value(:` and a direct Rust parse-fail analogue
  - `temp_workspace_isolation`: replaced the generic stub with a scaffold describing the same helper parse failure under invocation-scoped workspace isolation
  - `reachable_imports`: replaced the generic stub with a scaffold describing the reachable dependency parse failure from `def compute(:` and its Rust analogue
- local validation completed:
  - `rustfmt demos/graph_isolation/negative_cases/reachable_parse_error/idiomatic.rs demos/temp_workspace_isolation/negative_cases/reachable_parse_error/idiomatic.rs demos/reachable_imports/negative_cases/reachable_dependency_parse_error/idiomatic.rs`
  - `rustc --edition=2021 demos/graph_isolation/negative_cases/reachable_parse_error/idiomatic.rs -o /tmp/sifr-neg-graph-isolation-parse && /tmp/sifr-neg-graph-isolation-parse`
  - `rustc --edition=2021 demos/temp_workspace_isolation/negative_cases/reachable_parse_error/idiomatic.rs -o /tmp/sifr-neg-temp-workspace-parse && /tmp/sifr-neg-temp-workspace-parse`
  - `rustc --edition=2021 demos/reachable_imports/negative_cases/reachable_dependency_parse_error/idiomatic.rs -o /tmp/sifr-neg-reachable-imports-parse && /tmp/sifr-neg-reachable-imports-parse`
  - `cargo run -q -p sifr -- check demos/graph_isolation/negative_cases/reachable_parse_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/temp_workspace_isolation/negative_cases/reachable_parse_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/reachable_imports/negative_cases/reachable_dependency_parse_error/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-87-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-87-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 87 was reviewed directly from the final validated Rust scaffolds and the observed `sifr check` diagnostics for the paired negative fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_88_decimal_negative_case_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/decimal_arithmetic/negative_cases/decimal_division_by_zero/idiomatic.rs`
  - `demos/decimal_types/negative_cases/forbidden_float_constructor/idiomatic.rs`
  - `demos/decimal_conversions/negative_cases/forbidden_bigdecimal_float_constructor/idiomatic.rs`
- selection rationale:
  - these three phase-28 fixtures are the next untouched negative-case family and all revolve around exact decimal semantics rather than generic type or parse failures
  - the batch keeps decimal construction and arithmetic rules together instead of scattering them across unrelated Tier 2 cleanup work
  - each paired fixture has a stable observable failure mode, with the constructor cases failing in `check` and the zero-divisor case failing in `run`
- priority tags:
  - `tier-2-negative-case`: `decimal_arithmetic/division_by_zero`, `decimal_types/forbidden_float_constructor`, `decimal_conversions/forbidden_bigdecimal_float_constructor`
  - `decimal-semantics`: `decimal_arithmetic/division_by_zero`, `decimal_types/forbidden_float_constructor`, `decimal_conversions/forbidden_bigdecimal_float_constructor`
  - `generic-placeholder-cleanup`: `decimal_arithmetic/division_by_zero`, `decimal_types/forbidden_float_constructor`, `decimal_conversions/forbidden_bigdecimal_float_constructor`
- implementation summary:
  - `decimal_arithmetic`: replaced the generic stub with a scaffold describing the runtime failure contract for exact decimal division by zero and a direct Rust analogue
  - `decimal_types`: replaced the generic stub with a scaffold describing the ban on `Decimal(float)` construction and the exact-string Rust-side analogue
  - `decimal_conversions`: replaced the generic stub with a scaffold describing the same exact-construction ban for `BigDecimal(float)` and its Rust-side analogue
- local validation completed:
  - `rustfmt demos/decimal_arithmetic/negative_cases/decimal_division_by_zero/idiomatic.rs demos/decimal_types/negative_cases/forbidden_float_constructor/idiomatic.rs demos/decimal_conversions/negative_cases/forbidden_bigdecimal_float_constructor/idiomatic.rs`
  - `rustc --edition=2021 demos/decimal_arithmetic/negative_cases/decimal_division_by_zero/idiomatic.rs -o /tmp/sifr-neg-decimal-div-zero && /tmp/sifr-neg-decimal-div-zero`
  - `rustc --edition=2021 demos/decimal_types/negative_cases/forbidden_float_constructor/idiomatic.rs -o /tmp/sifr-neg-decimal-float-ctor && /tmp/sifr-neg-decimal-float-ctor`
  - `rustc --edition=2021 demos/decimal_conversions/negative_cases/forbidden_bigdecimal_float_constructor/idiomatic.rs -o /tmp/sifr-neg-bigdecimal-float-ctor && /tmp/sifr-neg-bigdecimal-float-ctor`
  - `cargo run -q -p sifr -- run demos/decimal_arithmetic/negative_cases/decimal_division_by_zero/main.sifr`
  - `cargo run -q -p sifr -- check demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr`
  - `cargo run -q -p sifr -- check demos/decimal_conversions/negative_cases/forbidden_bigdecimal_float_constructor/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-88-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-88-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed beyond correcting the `decimal_division_by_zero` scaffold wording to match the observed runtime-failure contract
- reviewer tooling note:
  - batch 88 was reviewed directly from the final validated Rust scaffolds and the observed decimal-specific failure modes for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_89_imported_helper_type_error_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/diagnostic_exit_codes/negative_cases/helper_type_error/idiomatic.rs`
  - `demos/project_check/negative_cases/helper_type_error/idiomatic.rs`
  - `demos/project_entrypoint/negative_cases/type_error_dependency/idiomatic.rs`
- selection rationale:
  - these three phase-22 fixtures are the next untouched negative-case family and all share the same contract: a reachable imported helper declares a numeric return type and returns `str`
  - the batch keeps frontend/project parity fixtures together instead of scattering dependency-type mismatches across unrelated Tier 2 cleanup work
  - each paired fixture already exposes a stable helper-qualified type error under `sifr check`, so the Rust-side scaffold can be validated against observed output
- priority tags:
  - `tier-2-negative-case`: `diagnostic_exit_codes/helper_type_error`, `project_check/helper_type_error`, `project_entrypoint/type_error_dependency`
  - `imported-helper-type-error`: `diagnostic_exit_codes/helper_type_error`, `project_check/helper_type_error`, `project_entrypoint/type_error_dependency`
  - `generic-placeholder-cleanup`: `diagnostic_exit_codes/helper_type_error`, `project_check/helper_type_error`, `project_entrypoint/type_error_dependency`
- implementation summary:
  - `diagnostic_exit_codes`: replaced the generic stub with a scaffold describing the imported helper `doubled` returning `str` instead of `int`, plus a direct Rust analogue
  - `project_check`: replaced the generic stub with a scaffold describing the imported helper `area_like` returning `str` instead of `float`
  - `project_entrypoint`: replaced the generic stub with a scaffold describing the imported helper `adjusted` returning `str` instead of `int`
- local validation completed:
  - `rustfmt demos/diagnostic_exit_codes/negative_cases/helper_type_error/idiomatic.rs demos/project_check/negative_cases/helper_type_error/idiomatic.rs demos/project_entrypoint/negative_cases/type_error_dependency/idiomatic.rs`
  - `rustc --edition=2021 demos/diagnostic_exit_codes/negative_cases/helper_type_error/idiomatic.rs -o /tmp/sifr-neg-diagnostic-helper-type && /tmp/sifr-neg-diagnostic-helper-type`
  - `rustc --edition=2021 demos/project_check/negative_cases/helper_type_error/idiomatic.rs -o /tmp/sifr-neg-project-check-helper-type && /tmp/sifr-neg-project-check-helper-type`
  - `rustc --edition=2021 demos/project_entrypoint/negative_cases/type_error_dependency/idiomatic.rs -o /tmp/sifr-neg-project-entry-dep-type && /tmp/sifr-neg-project-entry-dep-type`
  - `cargo run -q -p sifr -- check demos/diagnostic_exit_codes/negative_cases/helper_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/project_check/negative_cases/helper_type_error/main.sifr`
  - `cargo run -q -p sifr -- check demos/project_entrypoint/negative_cases/type_error_dependency/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-89-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-89-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 89 was reviewed directly from the final validated Rust scaffolds and the observed helper-qualified type errors for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_90_diagnostic_renderer_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/diagnostic_options/negative_cases/compact_grouping_contract/idiomatic.rs`
  - `demos/diagnostic_options/negative_cases/unknown_diagnostic_format/idiomatic.rs`
  - `demos/diagnostic_schema/negative_cases/type_error_json_diagnostic/idiomatic.rs`
- selection rationale:
  - these three diagnostics-focused fixtures are the next untouched negative-case family and all test renderer/config behavior rather than language semantics alone
  - the batch keeps compact grouping, invalid-format rejection, and json-schema emission together instead of scattering renderer contracts across unrelated Tier 2 cleanup work
  - each paired fixture already has a stable observable failure mode under the CLI, so the Rust-side scaffold can be validated directly against the emitted compact/json/output contract
- priority tags:
  - `tier-2-negative-case`: `diagnostic_options/compact_grouping_contract`, `diagnostic_options/unknown_diagnostic_format`, `diagnostic_schema/type_error_json_diagnostic`
  - `diagnostic-rendering`: `diagnostic_options/compact_grouping_contract`, `diagnostic_options/unknown_diagnostic_format`, `diagnostic_schema/type_error_json_diagnostic`
  - `generic-placeholder-cleanup`: `diagnostic_options/compact_grouping_contract`, `diagnostic_options/unknown_diagnostic_format`, `diagnostic_schema/type_error_json_diagnostic`
- implementation summary:
  - `compact_grouping_contract`: replaced the generic stub with a scaffold describing repeated `int <- str` failures whose purpose is deterministic compact-mode grouping
  - `unknown_diagnostic_format`: replaced the generic stub with a scaffold describing CLI argument validation that must exit with usage code `2` before semantic analysis begins
  - `type_error_json_diagnostic`: replaced the generic stub with a scaffold describing the simple `int <- str` mismatch used to validate canonical json diagnostic output
- local validation completed:
  - `rustfmt demos/diagnostic_options/negative_cases/compact_grouping_contract/idiomatic.rs demos/diagnostic_options/negative_cases/unknown_diagnostic_format/idiomatic.rs demos/diagnostic_schema/negative_cases/type_error_json_diagnostic/idiomatic.rs`
  - `rustc --edition=2021 demos/diagnostic_options/negative_cases/compact_grouping_contract/idiomatic.rs -o /tmp/sifr-neg-compact-grouping && /tmp/sifr-neg-compact-grouping`
  - `rustc --edition=2021 demos/diagnostic_options/negative_cases/unknown_diagnostic_format/idiomatic.rs -o /tmp/sifr-neg-unknown-format && /tmp/sifr-neg-unknown-format`
  - `rustc --edition=2021 demos/diagnostic_schema/negative_cases/type_error_json_diagnostic/idiomatic.rs -o /tmp/sifr-neg-json-diagnostic && /tmp/sifr-neg-json-diagnostic`
  - `cargo run -q -p sifr -- --diagnostic-format compact check demos/diagnostic_options/negative_cases/compact_grouping_contract/main.sifr`
  - `cargo run -q -p sifr -- --diagnostic-format not-a-format check demos/diagnostic_options/negative_cases/unknown_diagnostic_format/main.sifr`
  - `cargo run -q -p sifr -- --diagnostic-format json check demos/diagnostic_schema/negative_cases/type_error_json_diagnostic/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-90-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-90-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 90 was reviewed directly from the final validated Rust scaffolds and the observed compact/json/usage-exit behaviors for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_91_module_cycle_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/graph_isolation/negative_cases/module_cycle/idiomatic.rs`
  - `demos/module_cycle_diagnostics/negative_cases/module_cycle/idiomatic.rs`
  - `demos/module_ordering/negative_cases/idiomatic.rs`
- selection rationale:
  - these three module-graph negatives are the next untouched family and all preserve explicit cycle-detection behavior rather than ordinary type errors
  - the batch keeps the phase-19 and phase-23 cycle fixtures together so the Tier 2 cleanup records both the smaller two-node ordering cycle and the deterministic three-node diagnostic cycle
  - each paired fixture already exposes a stable cycle message under `sifr check`, so the Rust-side scaffold can be validated directly against observed output
- priority tags:
  - `tier-2-negative-case`: `graph_isolation/module_cycle`, `module_cycle_diagnostics/module_cycle`, `module_ordering/negative_cases`
  - `module-cycle`: `graph_isolation/module_cycle`, `module_cycle_diagnostics/module_cycle`, `module_ordering/negative_cases`
  - `generic-placeholder-cleanup`: `graph_isolation/module_cycle`, `module_cycle_diagnostics/module_cycle`, `module_ordering/negative_cases`
- implementation summary:
  - `graph_isolation`: replaced the generic stub with a scaffold describing the reachable three-node cycle `main -> a -> b -> c -> a`
  - `module_cycle_diagnostics`: replaced the generic stub with a scaffold describing the same three-node cycle, but emphasizing deterministic diagnostic reporting
  - `module_ordering`: replaced the generic stub with a scaffold describing the smaller two-node cycle `main -> a -> b -> a`
- local validation completed:
  - `rustfmt demos/graph_isolation/negative_cases/module_cycle/idiomatic.rs demos/module_cycle_diagnostics/negative_cases/module_cycle/idiomatic.rs demos/module_ordering/negative_cases/idiomatic.rs`
  - `rustc --edition=2021 demos/graph_isolation/negative_cases/module_cycle/idiomatic.rs -o /tmp/sifr-neg-graph-cycle && /tmp/sifr-neg-graph-cycle`
  - `rustc --edition=2021 demos/module_cycle_diagnostics/negative_cases/module_cycle/idiomatic.rs -o /tmp/sifr-neg-module-cycle-diagnostics && /tmp/sifr-neg-module-cycle-diagnostics`
  - `rustc --edition=2021 demos/module_ordering/negative_cases/idiomatic.rs -o /tmp/sifr-neg-module-ordering && /tmp/sifr-neg-module-ordering`
  - `cargo run -q -p sifr -- check demos/graph_isolation/negative_cases/module_cycle/main.sifr`
  - `cargo run -q -p sifr -- check demos/module_cycle_diagnostics/negative_cases/module_cycle/main.sifr`
  - `cargo run -q -p sifr -- check demos/module_ordering/negative_cases/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-91-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-91-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 91 was reviewed directly from the final validated Rust scaffolds and the observed module-cycle diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_92_optional_access_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/optional_arithmetic/negative_cases/optional_arithmetic_without_narrowing/idiomatic.rs`
  - `demos/optional_indexing/negative_cases/option_method_without_narrowing/idiomatic.rs`
  - `demos/indexing_rules/negative_cases/invalid_index_type/idiomatic.rs`
- selection rationale:
  - these three negatives are the next untouched type-safety family and each still used the generic placeholder despite enforcing direct user-facing checker diagnostics
  - the batch keeps phase-26 optional arithmetic with the closely related phase-27 optional-method and index-type fixtures so the Tier 2 rewrite records one coherent "unsafe access before narrowing" family
  - each paired fixture already exposes a stable `sifr check` message, so the Rust-side scaffold can be validated directly against observed output instead of guessing at the contract
- priority tags:
  - `tier-2-negative-case`: `optional_arithmetic/optional_arithmetic_without_narrowing`, `optional_indexing/option_method_without_narrowing`, `indexing_rules/invalid_index_type`
  - `type-safety`: `optional_arithmetic/optional_arithmetic_without_narrowing`, `optional_indexing/option_method_without_narrowing`, `indexing_rules/invalid_index_type`
  - `generic-placeholder-cleanup`: `optional_arithmetic/optional_arithmetic_without_narrowing`, `optional_indexing/option_method_without_narrowing`, `indexing_rules/invalid_index_type`
- implementation summary:
  - `optional_arithmetic`: replaced the generic stub with a scaffold describing arithmetic on `int | None` before narrowing and the resulting declared-return-type failure
  - `optional_indexing`: replaced the generic stub with a scaffold describing method access on `list[int] | None` before narrowing
  - `indexing_rules`: replaced the generic stub with a scaffold describing the deterministic rejection of `list[int]["0"]`
- local validation completed:
  - `rustfmt demos/optional_arithmetic/negative_cases/optional_arithmetic_without_narrowing/idiomatic.rs demos/optional_indexing/negative_cases/option_method_without_narrowing/idiomatic.rs demos/indexing_rules/negative_cases/invalid_index_type/idiomatic.rs`
  - `rustc --edition=2021 demos/optional_arithmetic/negative_cases/optional_arithmetic_without_narrowing/idiomatic.rs -o /tmp/sifr-neg-optional-arithmetic && /tmp/sifr-neg-optional-arithmetic`
  - `rustc --edition=2021 demos/optional_indexing/negative_cases/option_method_without_narrowing/idiomatic.rs -o /tmp/sifr-neg-option-method && /tmp/sifr-neg-option-method`
  - `rustc --edition=2021 demos/indexing_rules/negative_cases/invalid_index_type/idiomatic.rs -o /tmp/sifr-neg-invalid-index && /tmp/sifr-neg-invalid-index`
  - `cargo run -q -p sifr -- check demos/optional_arithmetic/negative_cases/optional_arithmetic_without_narrowing/main.sifr`
  - `cargo run -q -p sifr -- check demos/optional_indexing/negative_cases/option_method_without_narrowing/main.sifr`
  - `cargo run -q -p sifr -- check demos/indexing_rules/negative_cases/invalid_index_type/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-92-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-92-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 92 was reviewed directly from the final validated Rust scaffolds and the observed optional-access and invalid-index diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_93_recursive_traversal_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/generator_break_else/negative_cases/idiomatic.rs`
  - `demos/recursive_calls/negative_cases/recursive_call_typo/idiomatic.rs`
  - `demos/recursive_for_else/negative_cases/idiomatic.rs`
- selection rationale:
  - these three negatives are the next untouched semantic-analysis family and all still used the generic placeholder despite depending on reachable traversal through nested control flow
  - the batch keeps the phase-21 generator and `for`-`else` negatives with the closely related phase-24 recursive typo fixture so the Tier 2 rewrite records one coherent "reachable nested path must still diagnose" contract family
  - each paired fixture already exposes a stable `sifr check` message, so the Rust-side scaffold can be validated directly against observed output instead of inventing a synthetic Rust error
- priority tags:
  - `tier-2-negative-case`: `generator_break_else/negative_cases`, `recursive_calls/recursive_call_typo`, `recursive_for_else/negative_cases`
  - `semantic-traversal`: `generator_break_else/negative_cases`, `recursive_calls/recursive_call_typo`, `recursive_for_else/negative_cases`
  - `generic-placeholder-cleanup`: `generator_break_else/negative_cases`, `recursive_calls/recursive_call_typo`, `recursive_for_else/negative_cases`
- implementation summary:
  - `generator_break_else`: replaced the generic stub with a scaffold describing the reachable `except`-branch yield of undefined `missing_value` and the invalid generator return annotation
  - `recursive_calls`: replaced the generic stub with a scaffold describing the reachable recursive typo `reccurse(...)` under `if n > 0`
  - `recursive_for_else`: replaced the generic stub with a scaffold describing the reachable `for`-`else` recursive typo `recc(...)`
- local validation completed:
  - `rustfmt demos/generator_break_else/negative_cases/idiomatic.rs demos/recursive_calls/negative_cases/recursive_call_typo/idiomatic.rs demos/recursive_for_else/negative_cases/idiomatic.rs`
  - `rustc --edition=2021 demos/generator_break_else/negative_cases/idiomatic.rs -o /tmp/sifr-neg-generator-break-else && /tmp/sifr-neg-generator-break-else`
  - `rustc --edition=2021 demos/recursive_calls/negative_cases/recursive_call_typo/idiomatic.rs -o /tmp/sifr-neg-recursive-call-typo && /tmp/sifr-neg-recursive-call-typo`
  - `rustc --edition=2021 demos/recursive_for_else/negative_cases/idiomatic.rs -o /tmp/sifr-neg-recursive-for-else && /tmp/sifr-neg-recursive-for-else`
  - `cargo run -q -p sifr -- check demos/generator_break_else/negative_cases/undefined_in_except_yield.sifr`
  - `cargo run -q -p sifr -- check demos/recursive_calls/negative_cases/recursive_call_typo/main.sifr`
  - `cargo run -q -p sifr -- check demos/recursive_for_else/negative_cases/typo_in_for_else_recursive_call.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-93-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-93-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 93 was reviewed directly from the final validated Rust scaffolds and the observed generator-shape and undefined-name diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_94_decimal_policy_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs`
  - `demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs`
  - `demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs`
- selection rationale:
  - these three negatives are the next untouched decimal-specific family and all still used the generic placeholder despite enforcing stable policy-level diagnostics owned by the decimal checker
  - the batch keeps the phase-28 scale-bound, exact-construction, and mixed-arithmetic fixtures together so the Tier 2 rewrite records one coherent decimal-policy contract family
  - each paired fixture already exposes a stable code-bearing `sifr check` message, so the Rust-side scaffold can be validated directly against observed output instead of inferring the contract
- priority tags:
  - `tier-2-negative-case`: `decimal_round_scale_out_of_range`, `forbidden_float_conversion`, `forbidden_mixed_arithmetic`
  - `decimal-policy`: `decimal_round_scale_out_of_range`, `forbidden_float_conversion`, `forbidden_mixed_arithmetic`
  - `generic-placeholder-cleanup`: `decimal_round_scale_out_of_range`, `forbidden_float_conversion`, `forbidden_mixed_arithmetic`
- implementation summary:
  - `decimal_round_scale_out_of_range`: replaced the generic stub with a scaffold describing the explicit `0..=28` bound and deterministic `[E2507]` diagnostic
  - `forbidden_float_conversion`: replaced the generic stub with a scaffold describing the exact-construction rule rejecting `Decimal(float_value)` with `[E2505]`
  - `forbidden_mixed_arithmetic`: replaced the generic stub with a scaffold describing the explicit-conversion requirement when mixing `decimal` and `bigdecimal`, with `[E2504]`
- local validation completed:
  - `rustfmt demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs`
  - `rustc --edition=2021 demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs -o /tmp/sifr-neg-decimal-round-scale && /tmp/sifr-neg-decimal-round-scale`
  - `rustc --edition=2021 demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs -o /tmp/sifr-neg-decimal-float && /tmp/sifr-neg-decimal-float`
  - `rustc --edition=2021 demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs -o /tmp/sifr-neg-decimal-mixed && /tmp/sifr-neg-decimal-mixed`
  - `cargo run -q -p sifr -- check demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/main.sifr`
  - `cargo run -q -p sifr -- check demos/decimal_verification/negative_cases/forbidden_float_conversion/main.sifr`
  - `cargo run -q -p sifr -- check demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-94-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-94-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 94 was reviewed directly from the final validated Rust scaffolds and the observed decimal policy diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_95_type_parameter_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/compiled_expressions/negative_cases/idiomatic.rs`
  - `demos/generics_impl/negative_cases/return_type_mismatch/idiomatic.rs`
  - `demos/constrained_typevars/negative_cases/typevar_constraint_violation/idiomatic.rs`
- selection rationale:
  - these three negatives are the next untouched type-system family and all still used the generic placeholder despite enforcing stable declaration-vs-inference contracts
  - the batch keeps the simple return mismatch, generic `T | None` mismatch, and constrained typevar violation together so the Tier 2 rewrite records one coherent "declared type must be satisfied" family
  - each paired fixture already exposes a stable `sifr check` message, so the Rust-side scaffold can be validated directly against observed output instead of fabricating a different Rust failure
- priority tags:
  - `tier-2-negative-case`: `compiled_expressions/negative_cases`, `generics_impl/return_type_mismatch`, `constrained_typevars/typevar_constraint_violation`
  - `type-system`: `compiled_expressions/negative_cases`, `generics_impl/return_type_mismatch`, `constrained_typevars/typevar_constraint_violation`
  - `generic-placeholder-cleanup`: `compiled_expressions/negative_cases`, `generics_impl/return_type_mismatch`, `constrained_typevars/typevar_constraint_violation`
- implementation summary:
  - `compiled_expressions`: replaced the generic stub with a scaffold describing the direct `int` vs `str` return mismatch in `bad()`
  - `generics_impl`: replaced the generic stub with a scaffold describing safe indexing returning `T | None` while the helper promises plain `T`
  - `constrained_typevars`: replaced the generic stub with a scaffold describing `float` failing the `(int, str)` constraint on type parameter `T`
- local validation completed:
  - `rustfmt demos/compiled_expressions/negative_cases/idiomatic.rs demos/generics_impl/negative_cases/return_type_mismatch/idiomatic.rs demos/constrained_typevars/negative_cases/typevar_constraint_violation/idiomatic.rs`
  - `rustc --edition=2021 demos/compiled_expressions/negative_cases/idiomatic.rs -o /tmp/sifr-neg-compiled-expr && /tmp/sifr-neg-compiled-expr`
  - `rustc --edition=2021 demos/generics_impl/negative_cases/return_type_mismatch/idiomatic.rs -o /tmp/sifr-neg-generics-return && /tmp/sifr-neg-generics-return`
  - `rustc --edition=2021 demos/constrained_typevars/negative_cases/typevar_constraint_violation/idiomatic.rs -o /tmp/sifr-neg-typevar-constraint && /tmp/sifr-neg-typevar-constraint`
  - `cargo run -q -p sifr -- check demos/compiled_expressions/negative_cases/return_type_mismatch.sifr`
  - `cargo run -q -p sifr -- check demos/generics_impl/negative_cases/return_type_mismatch/main.sifr`
  - `cargo run -q -p sifr -- check demos/constrained_typevars/negative_cases/typevar_constraint_violation/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-95-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-95-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 95 was reviewed directly from the final validated Rust scaffolds and the observed return-mismatch and constrained-typevar diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_96_import_policy_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/import_forms/negative_cases/idiomatic.rs`
  - `demos/resolver_triggers/negative_cases/idiomatic.rs`
  - `demos/stdlib_modules/negative_cases/idiomatic.rs`
- selection rationale:
  - these three negatives are the next untouched frontend/import family and all still used the generic placeholder despite enforcing stable import-policy diagnostics
  - the batch keeps the phase-17 import-form restrictions with the closely related phase-18 resolver-trigger fixtures and the phase-20 `_sifr.*` intrinsic-import restriction so the Tier 2 rewrite records one coherent "unsupported import syntax or namespace must fail explicitly" family
  - each paired fixture already exposes stable `sifr check` output, so the Rust-side scaffolds can be validated directly against observed diagnostics instead of inventing a Rust module-system analogue
- priority tags:
  - `tier-2-negative-case`: `import_forms/negative_cases`, `resolver_triggers/negative_cases`, `stdlib_modules/negative_cases`
  - `frontend-import-policy`: `import_forms/negative_cases`, `resolver_triggers/negative_cases`, `stdlib_modules/negative_cases`
  - `generic-placeholder-cleanup`: `import_forms/negative_cases`, `resolver_triggers/negative_cases`, `stdlib_modules/negative_cases`
- implementation summary:
  - `import_forms`: replaced the generic stub with a scaffold describing unsupported bare relative imports, unsupported plain `import` statements, unsupported relative level 2 imports, and the reachable unresolved-name follow-on diagnostics
  - `resolver_triggers`: replaced the generic stub with a scaffold describing the same unsupported import forms as resolver-mode guards that must not silently activate project compilation
  - `stdlib_modules`: replaced the generic stub with a scaffold describing the rejection of user `_sifr.*` intrinsic imports and the reachable undefined `sqrt` follow-on diagnostic
- local validation completed:
  - `rustfmt demos/import_forms/negative_cases/idiomatic.rs demos/resolver_triggers/negative_cases/idiomatic.rs demos/stdlib_modules/negative_cases/idiomatic.rs`
  - `rustc --edition=2021 demos/import_forms/negative_cases/idiomatic.rs -o /tmp/sifr-neg-import-forms && /tmp/sifr-neg-import-forms`
  - `rustc --edition=2021 demos/resolver_triggers/negative_cases/idiomatic.rs -o /tmp/sifr-neg-resolver-triggers && /tmp/sifr-neg-resolver-triggers`
  - `rustc --edition=2021 demos/stdlib_modules/negative_cases/idiomatic.rs -o /tmp/sifr-neg-stdlib-modules && /tmp/sifr-neg-stdlib-modules`
  - `for f in demos/import_forms/negative_cases/*.sifr; do cargo run -q -p sifr -- check "$f"; done`
  - `for f in demos/resolver_triggers/negative_cases/*.sifr; do cargo run -q -p sifr -- check "$f"; done`
  - `for f in demos/stdlib_modules/negative_cases/*.sifr; do cargo run -q -p sifr -- check "$f"; done`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-96-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-96-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the targeted Tier 2 rewrite
- reviewer tooling note:
  - batch 96 was reviewed directly from the final validated Rust scaffolds and the observed unsupported-import and intrinsic-import diagnostics for the paired fixtures
  - both review passes ended with no accepted blockers across the trio

#### batch_97_closeout_scaffolds

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/default_values/negative_cases/unsupported_default_call_expression/idiomatic.rs`
  - `demos/protocol_bounds/negative_cases/unknown_protocol_bound_forwarding/idiomatic.rs`
  - `demos/variance_rules/negative_cases/list_variance_violation/idiomatic.rs`
  - `demos/while_else/negative_cases/idiomatic.rs`
- selection rationale:
  - these four folders were the last remaining generic placeholders in the corpus, so the closeout batch intentionally mixed the remaining checker negatives with the final runtime guard to finish wave 3 in one audited pass
  - the three negative fixtures still share one useful soundness theme around rejected user-visible contracts: unsupported default expressions, unsatisfied forwarded protocol bounds, and invariant list element types
  - the `while_else` fixture is not a checker failure but still needs an explicit Rust-side scaffold because its purpose is to guard runtime control-flow lowering rather than compare emitted Rust shape
- priority tags:
  - `tier-2-negative-case`: `unsupported_default_call_expression`, `unknown_protocol_bound_forwarding`, `list_variance_violation`
  - `tier-2-guard-fixture`: `while_else/negative_cases`
  - `generic-placeholder-cleanup`: `unsupported_default_call_expression`, `unknown_protocol_bound_forwarding`, `list_variance_violation`, `while_else/negative_cases`
- implementation summary:
  - `default_values`: replaced the generic stub with a scaffold describing the unsupported call-expression default and the deterministic missing-argument follow-on diagnostic
  - `protocol_bounds`: replaced the generic stub with a scaffold describing generic forwarding through an unknown protocol bound
  - `variance_rules`: replaced the generic stub with a scaffold describing list invariance rejecting `list[int]` where `list[int | str]` is required
  - `while_else`: replaced the generic stub with a guard scaffold documenting that `while`-`else` must skip the `else` arm after `break` and still print `ok`
- local validation completed:
  - `rustfmt demos/default_values/negative_cases/unsupported_default_call_expression/idiomatic.rs demos/protocol_bounds/negative_cases/unknown_protocol_bound_forwarding/idiomatic.rs demos/variance_rules/negative_cases/list_variance_violation/idiomatic.rs demos/while_else/negative_cases/idiomatic.rs`
  - `rustc --edition=2021 demos/default_values/negative_cases/unsupported_default_call_expression/idiomatic.rs -o /tmp/sifr-neg-default-call && /tmp/sifr-neg-default-call`
  - `rustc --edition=2021 demos/protocol_bounds/negative_cases/unknown_protocol_bound_forwarding/idiomatic.rs -o /tmp/sifr-neg-protocol-bound && /tmp/sifr-neg-protocol-bound`
  - `rustc --edition=2021 demos/variance_rules/negative_cases/list_variance_violation/idiomatic.rs -o /tmp/sifr-neg-variance && /tmp/sifr-neg-variance`
  - `rustc --edition=2021 demos/while_else/negative_cases/idiomatic.rs -o /tmp/sifr-neg-while-else && /tmp/sifr-neg-while-else`
  - `cargo run -q -p sifr -- check demos/default_values/negative_cases/unsupported_default_call_expression/main.sifr`
  - `cargo run -q -p sifr -- check demos/protocol_bounds/negative_cases/unknown_protocol_bound_forwarding/main.sifr`
  - `cargo run -q -p sifr -- check demos/variance_rules/negative_cases/list_variance_violation/main.sifr`
  - `cargo run -q -p sifr -- run demos/while_else/negative_cases/break_skips_else_guard.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-97-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-3-batch-97-review-pass-2.md`
- review application summary:
  - all four pass-1 reviews returned `OK`
  - all four pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the final Tier 2 rewrite
- reviewer tooling note:
  - batch 97 was reviewed directly from the final validated Rust scaffolds, the observed checker diagnostics for the three negative fixtures, and the observed successful `ok` runtime for the `while_else` guard fixture
  - wave 3 ends here with zero remaining generic `Negative-case Sifr fixture scaffold` placeholders

### wave_4_corpus_consistency_pass

status: completed_with_explicit_deferrals

#### batch_98_slice_and_format_cleanup

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/test_helpers/idiomatic.rs`
  - `demos/variance_rules/idiomatic.rs`
  - `demos/stdlib_error_types/idiomatic.rs`
- selection rationale:
  - these three already-reviewed demos are a clean first wave-4 sweep slice because they are runnable under the authoritative validation lane and still contained the exact consistency anti-patterns called out in the phase plan
  - the batch focuses on low-risk cleanup only: converting `&Vec<T>` APIs to slices and collapsing redundant `format!("{}", ...)` style wrappers
  - larger candidates such as `text_and_statistics` were intentionally left out of this batch because their paired Sifr demo is not currently a clean targeted-run validation target in this workspace
- priority tags:
  - `wave-4-consistency`
  - `slice-api-cleanup`
  - `format-wrapper-cleanup`
- implementation summary:
  - `test_helpers`: changed statistics helpers from `&Vec<T>` to slices and replaced nested formatting chains with direct `println!` / `assert_eq!(format!(...))` expressions
  - `variance_rules`: changed `sum_items` to accept `&[i64]` and used a direct slice call in `main`
  - `stdlib_error_types`: simplified integer formatting from `format!("{}", value as i64)` to `(value as i64).to_string()`
- local validation completed:
  - `rustfmt demos/variance_rules/idiomatic.rs demos/stdlib_error_types/idiomatic.rs demos/test_helpers/idiomatic.rs`
  - `rustc --edition=2021 demos/variance_rules/idiomatic.rs -o /tmp/sifr-wave4-variance && /tmp/sifr-wave4-variance`
  - `rustc --edition=2021 demos/stdlib_error_types/idiomatic.rs -o /tmp/sifr-wave4-stdlib-errors && /tmp/sifr-wave4-stdlib-errors`
  - `rustc --edition=2021 demos/test_helpers/idiomatic.rs -o /tmp/sifr-wave4-test-helpers && /tmp/sifr-wave4-test-helpers`
  - `cargo run -q -p sifr -- run demos/variance_rules/main.sifr`
  - `cargo run -q -p sifr -- run demos/stdlib_error_types/main.sifr`
  - `cargo run -q -p sifr -- run demos/test_helpers/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-98-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-98-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the low-risk cleanup pass
- reviewer tooling note:
  - batch 98 was reviewed against the final validated Rust outputs and the unchanged paired Sifr demo outputs for the three runnable demos

#### batch_99_string_ceremony_cleanup

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/normalized_fixtures/idiomatic.rs`
  - `demos/error_subclasses/idiomatic.rs`
  - `demos/python_regressions/idiomatic.rs`
- selection rationale:
  - these three already-reviewed runnable demos still carried the next most obvious wave-4 cleanup targets: repeated `.to_string().to_string()` chains and redundant `format!("{}", ...)` wrappers
  - the batch stays on demos that can still clear the authoritative local validation lane after targeted Rust-only cleanup
  - `python_regressions` was intentionally included despite its size because it concentrated the same string-ceremony anti-patterns in one place and exposed a real standalone validation bug worth fixing while the file was open
- priority tags:
  - `wave-4-consistency`
  - `string-ceremony-cleanup`
  - `format-wrapper-cleanup`
  - `standalone-validation-unblock`
- implementation summary:
  - `normalized_fixtures`: replaced `format!("{}", n1 * n2)` with `(n1 * n2).to_string()` and removed redundant `format!("{}", multiply(...))` assertions
  - `error_subclasses`: collapsed repeated `.to_string().to_string()` chains in the JSON helper and subclass-kind comparisons to single `.to_string()` calls
  - `python_regressions`: collapsed repeated string-conversion wrappers, simplified numeric error-message formatting, and changed `repeat` to return `result.into_iter()` so the companion no longer returns an iterator that borrows a local vector
- local validation completed:
  - `rustfmt demos/normalized_fixtures/idiomatic.rs demos/error_subclasses/idiomatic.rs demos/python_regressions/idiomatic.rs`
  - `rustc --edition=2021 demos/normalized_fixtures/idiomatic.rs -o /tmp/sifr-wave4-normalized-fixtures && /tmp/sifr-wave4-normalized-fixtures`
  - `tmpdir=$(mktemp -d /tmp/sifr-wave4-error-subclasses.XXXXXX) && cp demos/error_subclasses/idiomatic.rs "$tmpdir/src.rs" && mkdir "$tmpdir/src" && mv "$tmpdir/src.rs" "$tmpdir/src/main.rs" && cat > "$tmpdir/Cargo.toml" <<'EOF'`
  - `[package]`
  - `name = "sifr_wave4_error_subclasses"`
  - `version = "0.1.0"`
  - `edition = "2021"`
  - ``
  - `[dependencies]`
  - `serde_json = "1"`
  - `regex = "1"`
  - `EOF`
  - `cargo run --quiet --manifest-path "$tmpdir/Cargo.toml"`
  - `tmpdir=$(mktemp -d /tmp/sifr-wave4-python-regressions.XXXXXX) && cp demos/python_regressions/idiomatic.rs "$tmpdir/src.rs" && mkdir "$tmpdir/src" && mv "$tmpdir/src.rs" "$tmpdir/src/main.rs" && cat > "$tmpdir/Cargo.toml" <<'EOF'`
  - `[package]`
  - `name = "sifr_wave4_python_regressions"`
  - `version = "0.1.0"`
  - `edition = "2021"`
  - ``
  - `[dependencies]`
  - `regex = "1"`
  - `serde_json = "1"`
  - `EOF`
  - `cargo run --quiet --manifest-path "$tmpdir/Cargo.toml"`
  - `cargo run -q -p sifr -- run demos/normalized_fixtures/main.sifr`
  - `cargo run -q -p sifr -- run demos/error_subclasses/main.sifr`
  - `cargo run -q -p sifr -- run demos/python_regressions/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-99-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-99-review-pass-2.md`
- review application summary:
  - all three pass-1 reviews returned `OK`
  - all three pass-2 reviews returned `OK`
  - no follow-up code changes were needed after the final validated state
- reviewer tooling note:
  - batch 99 was reviewed against the final validated Rust outputs, including the temp-Cargo runs for the two dependency-bearing companions and the standalone iterator-lifetime fix in `python_regressions`
  - the paired Sifr demo outputs stayed unchanged across all three runnable demos

#### batch_100_python_regressions_slice_cleanup

status: accepted_after_pass_1_and_pass_2

- scope:
  - `demos/python_regressions/idiomatic.rs`
- selection rationale:
  - `python_regressions` still held the densest remaining wave-4 `&Vec<T>` surface after batch 99, spanning collection helpers, statistics helpers, bytes I/O helpers, itertools helpers, and bisect helpers in one validated demo
  - the other obvious slice-cleanup candidates were intentionally deferred because their paired Sifr demos are not currently clean targeted-run validation targets in this workspace
  - keeping the batch to one file preserved the authoritative validation bar while still removing a meaningful block of emitted-style collection APIs
- priority tags:
  - `wave-4-consistency`
  - `slice-api-cleanup`
  - `single-file-focused-batch`
- implementation summary:
  - changed read-only helper parameters from `&Vec<T>` to slices across `from_list`, the statistics helpers, `fnmatch_filter`, `take`, `flatten`, `bisect_left`, and `bisect_right`
  - changed `chain` to accept `&[Vec<T>]` and snapshot the input with `to_vec()` before building the lazy iterator
  - changed the bytes-writing and `_slice_to_bytes` helpers to accept slices, with matching direct `write_all(..., data)` calls
  - replaced the remaining `assert_eq!(format!("{}", json_val), "42")` check with `assert_eq!(json_val.to_string(), "42")`
- local validation completed:
  - `rustfmt demos/python_regressions/idiomatic.rs`
  - `tmpdir=$(mktemp -d /tmp/sifr-wave4-python-regressions.XXXXXX) && cp demos/python_regressions/idiomatic.rs "$tmpdir/src.rs" && mkdir "$tmpdir/src" && mv "$tmpdir/src.rs" "$tmpdir/src/main.rs" && cat > "$tmpdir/Cargo.toml" <<'EOF'`
  - `[package]`
  - `name = "sifr_wave4_python_regressions"`
  - `version = "0.1.0"`
  - `edition = "2021"`
  - ``
  - `[dependencies]`
  - `regex = "1"`
  - `serde_json = "1"`
  - `EOF`
  - `cargo run --quiet --manifest-path "$tmpdir/Cargo.toml"`
  - `cargo run -q -p sifr -- run demos/python_regressions/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-100-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-100-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK`
  - pass 2 returned `OK`
  - no follow-up code changes were needed after the validated slice cleanup
- reviewer tooling note:
  - batch 100 was reviewed against the final validated Rust output, the temp-Cargo dependency validation for the companion, and the unchanged `python_regressions` parity-demo output from the paired Sifr run
  - the intentionally single-file scope is recorded so the remaining wave-4 cleanup work can continue without mixing in upstream-broken targeted demo lanes

#### batch_102_deferred_textwrap_cleanup

status: accepted_after_pass_1_and_pass_2

- scope:
  - `crates/sifr_codegen/src/lib.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/function_like_lowering.rs`
  - `crates/sifr_codegen/src/class_method_emitter.rs`
  - `crates/sifr_codegen/src/class_emitter.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/operator_protocol_emitters.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `demos/html_and_textwrap/idiomatic.rs`
  - `demos/text_and_patterns/idiomatic.rs`
  - `demos/text_and_statistics/idiomatic.rs`
- selection rationale:
  - the last deferred wave-4 cleanup trio had a shared upstream blocker: their paired `textwrap`-backed Sifr demos still failed targeted validation because assignment into `str | None` locals skipped option coercion in plain `Assign` lowering
  - once that root cause was fixed, the originally deferred slice/signature cleanup could be completed under the normal authoritative validation bar instead of being recorded as non-blocking debt
- priority tags:
  - `wave-4-consistency`
  - `deferred-follow-up`
  - `codegen-root-cause-fix`
- implementation summary:
  - added active-scope local binding type tracking in codegen and threaded it through function-like lowering so plain `Assign` statements can reuse the existing target-type coercion path
  - fixed structured and simple assignment lowering to wrap non-optional RHS values when reassigning locals declared as `T | None`
  - updated `html_and_textwrap`, `text_and_patterns`, and `text_and_statistics` to use slices for the remaining read-only line helpers, dropped redundant `String` clone ceremony in `TextWrapper::new` and `wrap`, and converted the remaining text/statistics slice candidates in the deferred trio
- local validation completed:
  - `rustfmt crates/sifr_codegen/src/lib.rs crates/sifr_codegen/src/function_emitter.rs crates/sifr_codegen/src/function_like_lowering.rs crates/sifr_codegen/src/class_method_emitter.rs crates/sifr_codegen/src/class_emitter.rs crates/sifr_codegen/src/stmt_support_emitter.rs crates/sifr_codegen/src/operator_protocol_emitters.rs crates/sifr_codegen/src/lower_stmt.rs demos/html_and_textwrap/idiomatic.rs demos/text_and_patterns/idiomatic.rs demos/text_and_statistics/idiomatic.rs`
  - `cargo build -p sifr_codegen`
  - `rustc --edition 2021 demos/html_and_textwrap/idiomatic.rs -o /tmp/html_and_textwrap_idiomatic_check`
  - `tmpdir=$(mktemp -d) && mkdir -p "$tmpdir/src" && cp demos/text_and_patterns/idiomatic.rs "$tmpdir/src/main.rs" && printf '[package]\nname = "text_and_patterns_idiomatic_check"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nbase64 = "0.22"\n' > "$tmpdir/Cargo.toml" && cargo build --manifest-path "$tmpdir/Cargo.toml"`
  - `rustc --edition 2021 demos/text_and_statistics/idiomatic.rs -o /tmp/text_and_statistics_idiomatic_check`
  - `cargo run -q -p sifr -- run demos/html_and_textwrap/main.sifr`
  - `cargo run -q -p sifr -- run demos/text_and_patterns/main.sifr`
  - `cargo run -q -p sifr -- run demos/text_and_statistics/main.sifr`
  - `scripts/run_all_tests.sh`
- review artifacts:
  - pass 1: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-102-review-pass-1.md`
  - pass 2: `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-4-batch-102-review-pass-2.md`
- review application summary:
  - pass 1 returned `OK`
  - pass 2 returned `OK`
  - no follow-up code changes were needed after the final validated state
- reviewer tooling note:
  - batch 102 review covered both the codegen root-cause fix and the formerly deferred companion cleanup, and it was run against the final state after the three paired Sifr demo lanes and the full repository suite were green
  - the old wave-4 deferred follow-up set is now closed rather than carried forward

- non-actionable remaining search hits:
  - `demos/generic_stdlib/idiomatic.rs`: `unwrap_or_else(|| fill.clone())` is still appropriate because the fallback value is generic and not guaranteed `Copy`
  - `demos/advanced_class_libraries/idiomatic.rs`: `cloned().unwrap_or_default()` is an acceptable row-fill idiom and does not need further normalization for this corpus
- wave-4 closeout note:
  - the consistency pass is now fully complete, including the former deferred `textwrap`-backed trio, and no wave-4 follow-up set remains

### wave_5_phase_closeout

status: completed

## Closeout Requirements

- every reviewed batch must be recorded or linked
- every in-scope folder must be accounted for
- unresolved folders must have an explicit next action
- final note must state the corpus is ready for emitted-vs-idiomatic comparison
- Tier 2 folders must be explicitly marked as:
  - acceptable minimal scaffold,
  - acceptable harness,
  - or needing further clarification

## Closeout Summary

- in-scope directories accounted for:
  - `316` directories under `demos/` contain one or more `.sifr` files
  - `316 / 316` currently have an intentional `idiomatic.rs`
  - there are `0` remaining in-scope directories without a companion file
- reviewed-batch recording status:
  - all corpus batches through wave 4 are recorded in this ledger, including review artifact links and validation summaries
- Tier 2 disposition:
  - acceptable minimal scaffold: all `44` Tier 2 negative-case directories under `demos/**/negative_cases/**`, validated against their paired checker/runtime behavior during wave 3
  - acceptable harness: `demos/while_else/negative_cases`, which is intentionally a runtime guard fixture rather than a checker failure scaffold
  - needing further clarification: none
- final emitted-vs-idiomatic readiness note:
  - the corpus is ready for emitted-vs-idiomatic comparison
  - no remaining follow-up work is required for corpus coverage or wave-4 consistency; any future cleanup would be optional polish rather than unresolved phase debt

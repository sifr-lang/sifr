# Phase 30: Reliability Parity and Performance Budgets

## Objective
Close the reliability track by proving stdlib behavioral parity, API-level complexity and resource parity, and parity-governance discipline before algorithmic compatibility expansion.

## Closure Status
- Status: completed (2026-03-09)
- Closure evidence issue: `plans/issues/archive/phase30-reliability-parity-and-performance-budgets-execution.md`
- Closure note: the original explicit reviewer closure on 2026-03-09 covered `milestone_30_1`, `milestone_30_2`, and `milestone_30_3`. `milestone_30_4` was added on 2026-03-10 as a post-closure structural clarification for parity fixtures. It does not reopen the closed phase by default; an explicit reviewer pass is only required if this milestone is later promoted from planning guidance to a retroactive closure gate.

Phase 30 uses CPython as the behavioral reference model, but parity must always be aligned with Sifr's language guarantees:
- no user-triggerable runtime panics
- `Result[T, E]` or `Option[T]` where the Sifr architecture requires safe adaptation
- intentional divergences only when required by Sifr's safety contract
- production-grade implementations only

## Depends on
- Phase 29

## Execution Model
- Phase 30 remains a single roadmap phase.
- Work is grouped into related stdlib waves, but execution is strictly one module or stdlib at a time.
- Only one target module may be in active parity work at any given time.
- A module must complete the full implementation and review cycle before the next module begins.
- `CPython-derived parity tests` means transformed into the canonical Sifr parity fixture format and safety-adapted assertions, not mechanically copied from CPython `unittest` sources.
- The canonical baseline parity fixture format is documented in `audits/stdlib/cpython_parity_fixture_format.md` and must be reused unless a module-specific extension is explicitly justified.
- The per-module execution cycle is:
  1. define the parity scope and module to-do list from CPython references
  2. port or expand CPython-derived parity tests for the module using the canonical Sifr parity fixture format
  3. fix root-cause implementation gaps in Sifr
  4. validate locally with positive-path and negative-path coverage
  5. classify every observed mismatch as `parity`, `intentional-diff`, or `unsupported`
  6. submit the module for reviewer evaluation
  7. repeat until the reviewer explicitly signs off the module
- A wave is complete only when every module in that wave has individually passed this cycle and merged.
- No parallel multi-module parity development is allowed in this phase.
- No fallback behavior, compatibility shims, or partial parity claims are allowed.

## Reviewer Gate
A module is not complete when the implementer believes it is done.
A module is complete only when the reviewer explicitly confirms all of the following:
- the parity scope is clear and evidenced by CPython-derived tests
- remaining gaps are classified correctly
- every intentional divergence is justified by Sifr's safety contract
- no unresolved mismatch lacks an owner and tracking issue
- no user-facing runtime panic path remains
- implementation quality is production-grade
- the module is CPython-parity aligned for the approved scope, aligned with Sifr safety guarantees, and production-ready

## Safety Alignment Rules
All Phase 30 work must follow the language safety contract in `internal_docs/architecture.md`.
In particular:
- where CPython raises an exception, Sifr must return `Result[T, E]` unless the architecture explicitly defines `Option[T]`
- where CPython raises `IndexError`, Sifr returns `Option`
- where CPython raises `KeyError`, Sifr returns `Option`
- parity tests must validate the Sifr-safe adaptation rather than blindly copying CPython exception assertions
- no module may introduce panic-based control flow or user-triggerable runtime crash behavior
- any divergence from CPython required by Sifr safety must be recorded as `intentional-diff` with rationale
- platform-dependent CPython behavior must be classified explicitly as `parity`, `intentional-diff`, or `unsupported`; implicit host-specific skips are not sufficient

## Milestones

### milestone_30_1: Stdlib Behavioral Parity Program
- Scope:
  - Port and maintain CPython-derived parity suites for in-scope stdlib modules.
  - Execute parity work one module at a time within related waves.
  - Maintain per-module parity classification for every non-parity behavior.
  - Where CPython provides canonical upstream data fixtures or vector corpora that materially improve coverage, port or consume them directly as part of module parity work.
  - For numeric modules, explicitly document floating-point comparison policy, special-value handling (`NaN`, infinities, signed zero where relevant), and the strategy for large upstream corpora (full port, filtered subset, or generated projection).
- Definition of done:
  - Each in-scope module has reviewer-approved CPython-derived parity coverage.
  - Every covered mismatch is classified as `parity`, `intentional-diff`, or `unsupported`.
  - No module is marked complete without reviewer sign-off for parity, safety alignment, panic freedom, and production readiness.

### milestone_30_2: Complexity and Resource Parity
- Scope:
  - Add API-level scaling and resource checks for stabilized Phase 30 modules.
  - Compare asymptotic behavior to CPython-relevant reference behavior.
  - Track constant-factor deltas explicitly.
  - Define repeatable local complexity test patterns per API class, including input-size sweeps, measurement normalization rules, and acceptance criteria for asymptotic and constant-factor outcomes.
  - Keep this work limited to stdlib API-facing complexity and resource behavior; compiler performance benchmarking and budget governance remain owned by Phase 35.
- Definition of done:
  - Complexity and resource checks exist for in-scope modules whose behavioral parity work is complete.
  - Asymptotic mismatches are fixed or explicitly waived.
  - Constant-factor regressions are documented with rationale and owner.

### milestone_30_3: Parity Governance and Waiver Discipline
- Scope:
  - Standardize parity matrix, classification, and waiver formats.
  - Require owner, rationale, linked issue, and revisit rule for every unresolved gap.
  - Enforce that no module closes with undocumented mismatch status.
- Definition of done:
  - Phase 30 has one canonical parity-governance format.
  - No unresolved parity gap exists without documented status and ownership.
  - The waiver inventory is complete and reviewable.

### milestone_30_4: Parity Test Corpus Structure and Maintainability
- Scope:
  - Standardize the structure of Phase 30 module test corpora so they stay readable, reviewable, and production-maintainable as parity coverage grows.
  - Require each module's parity tests to be organized into a small number of semantic fixtures rather than one oversized catch-all fixture or a large set of microscopic files.
  - Require each fixture to keep `main()` as the orchestration layer only, with behavior grouped into small helper functions or clearly separated canonical vector sections.
  - Require positive-path, negative-path, and safety-adaptation coverage to be explicit and easy to audit inside each module's approved parity scope.
  - Require deterministic inputs, deterministic ordering, and stable assertion grouping so failures remain reproducible and reviewer-friendly.
  - Require fixture structure to follow the canonical Sifr parity fixture format unless a justified extension is documented.
  - Use reviewer-driven enforcement for this milestone by default; automated structural linting is optional future hardening rather than part of the base phase contract.
  - Execute this milestone wave-by-wave (not module-by-module): complete one wave's structural remediation, validation, PR merge, and reviewer cycles before starting the next wave.
  - Run external review cycles at wave granularity: completion check then production-grade check, with remediation PRs merged between cycles when findings exist.
- Definition of done:
  - Every in-scope module has a parity test corpus whose structure is understandable without reverse-engineering a giant monolithic `main()`.
  - Every module's parity tests are split along behavior or API-surface boundaries that are appropriate for the approved scope.
  - Each fixture has a clear execution flow, with helper functions or vector sections that map to reviewable behavior groups.
  - Positive-path, negative-path, and safety-adaptation assertions are all present and easy to locate.
  - No module closes with parity coverage that is technically passing but structurally too tangled to maintain confidently.
  - The milestone's status is tracked explicitly in the Phase 30 execution checklist issue and is not implied by `milestone_30_1` through `milestone_30_3` reviewer sign-off.
  - Lack of an automated structure-validation script does not block this milestone; the required enforcement path is explicit review against the canonical fixture-format rules.

### Milestone Evidence Artifacts
- `milestone_30_1` parity governance matrix:
  - `verification/stdlib/phase30_parity_matrix.md`
- `milestone_30_2` complexity/resource matrix and inventory:
  - `verification/stdlib/phase30_complexity_resource_matrix.md`
  - `verification/stdlib/phase30_complexity_resource_inventory.json`
  - `scripts/check_phase30_complexity_resource_inventory.py`
- `milestone_30_3` waiver and ownership discipline:
  - `verification/stdlib/phase30_parity_matrix.md`
  - `verification/stdlib/phase30_complexity_resource_inventory.json`
- `milestone_30_4` parity test corpus structure and maintainability:
  - `audits/stdlib/cpython_parity_fixture_format.md`
  - `crates/sifr/tests/e2e/pass/`
  - `plans/issues/archive/phase30-reliability-parity-and-performance-budgets-execution.md`

## Behavioral Parity Waves

### wave_30_1a: Binary and Encoding Foundations
- Modules:
  - `env`
  - `bytes`
  - `base64`
  - `hashlib`
- Why:
  - Shared binary data, encoding, validation, and safe error-adaptation concerns.
- CPython references:
  - `env`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/os.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_os/test_os.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_os/test_posix.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/posixmodule.c`
  - `bytes`: `/Users/yaseralnajjar/work/sifr/cpython/Objects/bytesobject.c`, `/Users/yaseralnajjar/work/sifr/cpython/Objects/bytearrayobject.c`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_bytes.py`
  - `base64`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/base64.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_base64.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/binascii.c`
  - `hashlib`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/hashlib.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_hashlib.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_hashopenssl.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/md5module.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/sha1module.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/sha2module.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/sha3module.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/blake2module.c`

### wave_30_1b: Numeric and Ordered-Collection Semantics
- Modules:
  - `math`
  - `statistics`
  - `bisect`
  - `heapq`
- Why:
  - Shared numerical correctness and deterministic algorithmic behavior.
- Wave-specific handling notes:
  - Numeric parity work in this wave must document float comparison rules, special-value treatment, and any approved subsetting strategy for large upstream vector corpora before reviewer sign-off.
- CPython references:
  - `math`: `/Users/yaseralnajjar/work/sifr/cpython/Modules/mathmodule.c`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_math.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/mathdata/math_testcases.txt`
  - `statistics`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/statistics.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_statistics.py`
  - `bisect`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/bisect.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_bisect.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_bisectmodule.c`
  - `heapq`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/heapq.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_heapq.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_heapqmodule.c`

### wave_30_1c: Text and Pattern Processing
- Modules:
  - `string`
  - `textwrap`
  - `fnmatch`
  - `re`
- Why:
  - Shared text normalization, formatting, wildcard, and regex semantics.
- CPython references:
  - `string`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/string/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/string/templatelib.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_string/test_string.py`
  - `textwrap`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/textwrap.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_textwrap.py`
  - `fnmatch`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/fnmatch.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_fnmatch.py`
  - `re`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/re/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/re/_parser.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/re/_compiler.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/re/_constants.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_re.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_sre/sre.c`

### wave_30_1d: Core Containers and Structured Data
- Modules:
  - `collections`
  - `itertools`
  - `json`
  - `datetime`
- Why:
  - Shared object model, laziness, data-shape, and return-structure concerns.
- Wave-specific handling notes:
  - For this wave, parity fixtures may use a helper-oriented boolean assertion vector as an explicit module-specific extension to the baseline `inputs/expected/actual` string-vector format.
  - Rationale: the approved parity scope includes structured-return and semantic-behavior checks (`Counter`, set semantics, iterator contracts, structured JSON values, `timedelta` arithmetic/comparison) where literal string-vector snapshots reduce signal and increase brittleness.
  - Constraint: this extension is allowed only when fixtures keep deterministic helper ordering, orchestration-only `main()`, and explicit positive/negative/safety sections documented in the phase execution tracker.
- CPython references:
  - `collections`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/collections/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_collections.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_collectionsmodule.c`
  - `itertools`: `/Users/yaseralnajjar/work/sifr/cpython/Modules/itertoolsmodule.c`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_itertools.py`
  - `json`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/json/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/json/decoder.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/json/encoder.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/json/scanner.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_json/`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_json.c`
  - `datetime`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/_pydatetime.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_datetimemodule.c`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_datetime.py`

### wave_30_1e: File, Path, and Filesystem Surface
- Modules:
  - `io`
  - `csv`
  - `os`
  - `pathlib`
  - `glob`
  - `tempfile`
  - `shutil`
- Why:
  - Shared file, path, and runtime-boundary semantics.
- Wave-specific handling notes:
  - For this wave, parity fixtures may use a helper-oriented boolean assertion vector as an explicit module-specific extension to the baseline `inputs/expected/actual` string-vector format.
  - Rationale: the approved scope is dominated by filesystem effects and path-shape semantics where literal string-vector snapshots are brittle and lower-signal than explicit semantic pass/fail checks.
  - Constraint: this extension is allowed only when fixtures keep deterministic helper ordering, orchestration-only `main()`, and explicit positive/negative/safety sections documented in the phase execution tracker.
- CPython references:
  - `io`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/io.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/_pyio.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_io/`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/_iomodule.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/fileio.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/textio.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/bufferedio.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/stringio.c`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_io/bytesio.c`
  - `csv`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/csv.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_csv.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_csv.c`
  - `os`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/os.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_os/test_os.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_os/test_posix.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/posixmodule.c`
  - `pathlib`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/pathlib/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/pathlib/_local.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/pathlib/_os.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/pathlib/types.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_pathlib/`
  - `glob`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/glob.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_glob.py`
  - `tempfile`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/tempfile.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_tempfile.py`
  - `shutil`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/shutil.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_shutil.py`

### wave_30_1f: Runtime and Platform Wrappers
- Modules:
  - `logging`
  - `time`
  - `timeit`
  - `platform`
  - `uuid`
- Why:
  - Shared wrapper-heavy APIs over host and runtime capabilities.
- Wave-specific handling notes:
  - For this wave, parity fixtures may use a helper-oriented boolean assertion vector as an explicit module-specific extension to the baseline `inputs/expected/actual` string-vector format.
  - Rationale: the approved scope is host- and runtime-dependent (clock progression, platform identity, logging sinks, and random UUID generation) where literal string-vector snapshots are brittle and lower-signal than explicit semantic pass/fail checks.
  - Constraint: this extension is allowed only when fixtures keep deterministic helper ordering, orchestration-only `main()`, and explicit positive/negative/safety sections documented in the phase execution tracker.
- CPython references:
  - `logging`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/logging/__init__.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/logging/handlers.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_logging.py`
  - `time`: `/Users/yaseralnajjar/work/sifr/cpython/Modules/timemodule.c`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_time.py`
  - `timeit`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/timeit.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_timeit.py`
  - `platform`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/platform.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_platform.py`
  - `uuid`: `/Users/yaseralnajjar/work/sifr/cpython/Lib/uuid.py`, `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_uuid.py`, `/Users/yaseralnajjar/work/sifr/cpython/Modules/_uuidmodule.c`

## Quality Contract
- Entry criteria: Phase 29 is completed and verification hardening is active.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical and lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone or module that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Reliability claims are backed by reviewed stdlib parity evidence, API-level complexity and resource evidence, explicit parity governance, and demonstrated alignment with Sifr's safety guarantees.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each completed module must resolve root causes for its approved scope or record explicit waivers.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every module must complete the reviewer sign-off cycle before the next module begins.
  - Every module must include positive-path and negative-path validation.
  - Every module must explicitly validate safe adaptation behavior where CPython would raise.
  - Modules with parsing-heavy, numeric-edge, or panic-risk surfaces must reuse the Phase 29 property and fuzz machinery as part of module sign-off.
  - No user-triggerable runtime panic is allowed in any completed module.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
- Validation planning goals:
  - `milestone_30_1` (Stdlib Behavioral Parity Program): validation goals cover: port and maintain CPython-derived parity suites; execute one module at a time within related waves; classify every mismatch; require reviewer sign-off per module. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_30_2` (Complexity and Resource Parity): validation goals cover: add API-level scaling and resource checks for stabilized modules; compare asymptotic behavior to CPython-relevant reference behavior; track constant-factor deltas explicitly. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_30_3` (Parity Governance and Waiver Discipline): validation goals cover: standardize parity matrix, classification, and waiver formats; require owner, rationale, linked issue, and revisit rule for unresolved gaps; enforce that no module closes with undocumented mismatch status. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_30_4` (Parity Test Corpus Structure and Maintainability): validation goals cover: keep module parity coverage split into reviewable semantic fixtures; keep fixture `main()` bodies thin and orchestration-only; ensure positive-path, negative-path, and safety-adaptation checks are explicit and easy to audit; and keep fixture ordering and data deterministic. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: reliability claims are backed by reviewed stdlib parity evidence, explicit safety-aligned divergence policy, API-level complexity and resource evidence, and complete waiver-governed parity classification.

## Local Validation Commands
- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Module-specific parity validation:
  - targeted `cargo test` commands for the active module
  - targeted `cargo run -q -p sifr -- test ...` commands for the active module
- Milestone demos:
  - `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr`

## Exit Gate
- Reliability claims are backed by reviewed stdlib parity evidence, API-level complexity and resource evidence, explicit waiver-governed parity classification, and demonstrated alignment with Sifr safety guarantees.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics, renderer, and exit-code behavior.

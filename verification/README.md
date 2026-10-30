# Sifr Verification

`verification/` owns runner mechanics, schemas, profiles, policy, and area-owned
verification data.

Python verification tooling is managed by `uv` through this directory:

```bash
uv run --project verification python -m sifr_verify --self-test
uv run --project verification python -m sifr_verify profiles check
uv run --project verification python -m sifr_verify profiles plan --profile merge
uv run --project verification python -m sifr_verify areas check
uv run --project verification python -m sifr_verify doctor
uv lock --project verification --check
```

All maintained first-party Python projects require the canonical GIL-enabled
CPython 3.14.7 interpreter exactly. Install it with `uv python install 3.14.7`;
`uv run` selects it from each project's exact `requires-python` pin.

Required `uv` version: `0.12.10`. Every maintained uv project carries the exact
native `required-version` pin, and CI reads the same pin from this project's
`pyproject.toml`.

The public validation entrypoint remains:

```bash
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh --profile merge --emit-plan
```

`scripts/run_all_tests.sh` is a thin public facade over
`uv run --project verification --locked python -m sifr_verify profiles run`.
It fail-fasts when `uv` is missing or differs from the exact version so profile
execution stays reproducible for local and CI validation.

`--emit-plan` prints the selected profile's machine-readable execution plan
without running suites. CI may add broader profiles, but it must not omit suites
from the local merge plan except through declared host skips.
Compare local and CI plans with:

```bash
uv run --project verification --locked python -m sifr_verify profiles compare-plans --local <local-plan.json> --ci <ci-plan.json>
```

`sifr_verify doctor` checks required local prerequisites: exact Python version, Rust
and Cargo availability, `uv` lock status, Cargo offline metadata resolution, and
host metadata. Optional sanitizer tools are reported as pass or skip for broader
lanes.

## Development iterations

Diagnostic identities, severity, messages and arguments belong to the production
registry. Representative test references belong to
[`diagnostics/data/code_catalog.json`](areas/diagnostics/data/code_catalog.json).
Documentation generation and diagnostic tests read that catalog at execution time;
renaming a fixture does not require changing the production registry. The coverage
checks require exactly the active codes and validate fixture paths and Rust symbols.
Run the diagnostic coverage checks and `gen-error-docs --check` after fixture changes;
regenerate affected documentation when a reference changes.

Profile preparation selects the development compiler through Cargo and publishes
one immutable copy for that run. Area processes consume it through
`SIFR_VERIFY_PREPARED_COMPILER`, validate the content-bound manifest and published file identity, and hold a shared lifetime
lease. Invalid or expired references fail; consumers never silently rebuild.
The run owner retires the copy after execution, and cleanup waits for remaining
consumers. A new invocation enters Cargo again. Standalone area invocations still
prepare through Cargo; tests whose contract requires a build use the explicit
`prepare_sifr_binary` operation even inside a prepared run. These local lifetime
locks currently require POSIX, like the local process ownership runner. This is
artifact reuse within one frozen candidate, not reuse of test results.

The owner must retain exclusive access to its Cargo target from preparation
through publication: another producer replacing Cargo's output after Cargo exits
but before copying starts is outside this contract. Prepared development consumers
receive the declared source sysroot explicitly, including when target storage
resolves outside the checkout; installed-product and discovery checks clear that
setting and retain their separate qualification.

Start with the smallest check affected by the edit. The public facade defaults
to `merge`; choose a profile explicitly when preparing a candidate, rather than
running the full gate after every local edit.

| Change or checkpoint | Next command from the repository root |
|---|---|
| One compiler behavior | `uv run --project verification --locked cargo test --locked -p <crate> <test-filter>` |
| Generated Rust normalization | `uv run --project verification --locked cargo test --locked -p sifr_generated_rust <test-filter>`; [ownership](../crates/sifr_generated_rust/README.md) |
| Public codegen emission contract | `uv run --project verification --locked cargo test --locked -p sifr_codegen --test emission_contracts <test-filter>`; see [test ownership](../crates/sifr_codegen/tests/README.md) |
| CLI tests excluding the E2E pass suite | `uv run --project verification --locked cargo test --locked -p sifr -- --skip test_e2e_pass` |
| Verification runner mechanics | Affected self-tests; the runner foundation suite is `uv run --project verification --locked python -m sifr_verify --self-test` |
| One verification area/suite | `uv run --project verification --locked python -m sifr_verify areas run --area <area> --suite <suite>` |
| Full E2E pass corpus | `verification/runner/e2e/run_e2e_pass.sh --profile merge` |
| Implementation PR candidate | `scripts/run_all_tests.sh --profile create-pr` |
| Final implementation merge candidate | `scripts/run_all_tests.sh --profile merge` |
| Planning, prose or review records only | `git diff --check`; verify changed local links, scope and status; no compiler build or broad gate |

Run focused Cargo commands through the same locked verification `uv` entry used
by the full gate, from the same parent shell environment. This keeps the launcher
PATH consistent; the native build identity includes PATH because build scripts
can resolve tools through it. An extra virtual-environment prefix can otherwise
select a separate native target even when Cargo and rustc themselves are unchanged.
Use one `uv run` layer: the full-gate facade already provides it.

This aligns the launcher, not every build input. Preserve the same owned
`CARGO_TARGET_DIR`, toolchain, features, profile, flags and relevant native/Python
environment settings. Keep intentional resource allocations explicit. Cargo still
checks freshness and every selected test executes; a retained target is not a
cached test result. Do not remove environment identity or copy artifacts between
incompatible targets to force reuse.

Development and test builds retain line tables for source-located backtraces,
alongside the existing optimization levels, debug assertions and overflow checks.
Full debugger variable/type inspection needs full debug information. Build that
explicitly in a separately owned target after checking capacity:

```bash
CARGO_TARGET_DIR=target/debugger-full CARGO_PROFILE_DEV_DEBUG=2 \
  uv run --project verification --locked cargo test --locked -p sifr_driver --lib --no-run
```

Keep this debugger target separate from normal iteration targets so switching
debug-information settings does not repeatedly invalidate their artifacts.

Focused commands supplement the affected checks and acceptance gates in
[AGENTS.md](../AGENTS.md#minimum-validation). A fixture subset is diagnostic
coverage, not full E2E qualification. Release and live-integration contracts
remain conditional on the requested work.

Retain session-owned compiler and native-build targets between iterations; do
not use routine `cargo clean` as a retry strategy. Keep target ownership and the
compiler/native allocations explicit. Preparation and execution must use matching
Cargo configurations and features, with Cargo checking freshness. A retained
baseline executable can reproduce an old failure but cannot qualify new source.

If preparation is slow or fails, preserve the command, input identity, stage,
logs and outcome before retrying. Check capacity, lock contention, changed inputs
and Cargo fingerprints; make an evidence-backed correction before repeating an
unchanged attempt. Follow the active phase's diagnostic cap and ownership rules;
a timeout or cancellation is incomplete evidence and never a gate pass.

### Local correctness scope

Use the explicit scope when working on the LOCAL-GATE phase:

```bash
scripts/run_all_tests.sh --profile merge --correctness-only --emit-plan
scripts/run_all_tests.sh --profile merge --correctness-only --no-fail-fast
```

`--correctness-only` derives selection from the canonical `create-pr` or `merge`
profile. It removes only performance-area benchmark captures (`smoke`,
`representative`, `full`). Syntax/cache checks, policy assertions, all other
selected areas, crate configurations, guardrails and E2E corpus remain selected.
It does not change the profile defaults or CI selection. Nightly, release and live
profiles reject this flag; scoped execution cannot produce release evidence.

Plans list the scope and every excluded suite. Reports use separate
`<profile>-correctness` paths and carry the scope explicitly, so scoped evidence
cannot overwrite or qualify a full profile. Base E2E/native profile semantics
remain unchanged. Reference admission still applies whenever a reference-dependent
performance suite is selected. The LOCAL-GATE final timing/coverage matrix remains
required; this entry point alone establishes no ten-minute performance claim.

Use `--no-fail-fast` for broad failure discovery when preparation can be shared.
Independent failures remain failures, and invalid inventories, unsafe preparation
or resource/deadline cancellation still stop dependent work. An incomplete broad
run is diagnostic evidence, never acceptance. The phase's approved intermediate
policy retains focused checks and independent review before each intermediate
merge, followed by complete integrated final gates.

Selected Python-area, SQL LSP, LSP crate and driver library/native consumers depend on
the locked Python interop environment. Preparation uses that project's exact pins
and `.venv`, independently of an activated environment or ambient
`UV_PROJECT_ENVIRONMENT`; Cargo targets, flags and job allocations are preserved.
Environment preparation is separate from compiled Python examples. With
`--no-fail-fast`, an environment failure blocks its consumers while independent
work continues; an example failure does not invalidate a successfully prepared
environment. Bootstrap time remains part of preparation accounting. The 29 compiled Python
examples own their package reconstruction, certification, optimized compilation
and runtime assertions in execution. Profiles do not precompile those examples:
that discarded pass duplicated work subsequently repeated by canonical
`sifr run --release`. Cold native compilation is included in execution time;
compare complete preparation plus execution when measuring improvements.
`python_interop/runner/prepare_examples.py` remains an explicit diagnostic
prewarming tool, with no runtime coverage claim; it is not a profile prerequisite.
For focused driver Python tests outside a profile, prepare this same environment
once with `uv run --project verification/areas/python_interop --locked python
verification/areas/python_interop/runner/prepare_environment.py`. The builtins-only
fixtures reuse that interpreter while keeping fresh source projects and runtime
processes; they do not modify the pinned environment or reuse old test results.

### Native SQL tool construction

Native SQL tool preparation and the independent-build qualifier share the
[SQL construction policy](areas/sql_platform/tools/native_build_policy.py).
Its host build scripts, procedural macros and syntax libraries use optimization
level zero; the workspace's optimized SHA-256 build dependency remains at level
one. The selected SQL graph uses the syntax libraries only during construction.
The normal compiler bootstrap retains its existing optimization settings.

The native qualifier checks actual Cargo artifacts for host/target separation,
target optimization level one, debug assertions and overflow checks. It rejects
syntax libraries entering the target graph, profile drift, duplicate units and
incomplete profile evidence. Distinct feature selections remain distinct units.
Qualification keeps its existing zero target debug-info setting and requires
identical executable bytes across clean, unchanged and independent builds of the
selected configuration. A construction configuration change has its own artifact
identities and requires runtime qualification. The real native SQL sandbox/IPC
tests consume tools prepared with this same construction policy.

### Build allocations

The profile runner keeps compiler Cargo jobs separate from native E2E jobs.
`resource_policy.compiler_build_jobs` requests the compiler allocation;
`CARGO_BUILD_JOBS` overrides that request. The same resolved value is used during
preparation and execution. The E2E wrapper preserves it rather than overwriting
it with the per-batch native setting.

For E2E workers, forwarded job flags take precedence over `SIFR_E2E_*_JOBS`
environment values, which take precedence over profile `e2e` values. All job
requests must be positive integers. Requests are capped by `max_parallel`,
effective CPU capacity and the declared memory allowance. Native build workers
multiply by native Cargo jobs to consume the budget; compiler preparation and
native execution run in separate stages. Within native builds, the inner Cargo
request is bounded first, then the outer group count.

Profiles reserve one CPU where more than one is available and 2 GiB of memory,
and budget 2 GiB per build job. These are conservative admission settings, not a
measured upper bound on arbitrary compiler workloads or a live resource scheduler.
Linux admission includes affinity and cgroup v2 ancestor quotas/headroom,
crediting inactive file cache while keeping dirty/writeback pages charged;
macOS uses available free/inactive pages. Capacity is sampled at profile entry;
shared-host reservations and changing pressure remain the parent resource-plan's
responsibility. Insufficient memory for one job plus reserve fails explicitly.
The `[sifr-build-allocation]` receipt records requests, override sources, policy,
capacity and resolved allocations. Standalone E2E commands retain their explicit
worker controls; use the profile runner for capacity-bounded admission.

Ordinary driver and CLI libtests share this budget with their nested Cargo and Rayon
work, including lowering tests grouped in the full CLI configuration. The runner samples capacity again before each selected configuration, bounds
`RUST_TEST_THREADS` by available slots, and divides nested jobs among the selected
workers. An explicit smaller thread request is preserved. A libtest
`--test-threads` flag exceeding admission fails instead of bypassing the budget.
Preparation retains its compiler allocation. A lost memory reserve during
execution terminates the owned command and fails the category. The
`[sifr-libtest-allocation]` receipt records width and nested jobs. The standalone
native assertions in the CLI suite also have their explicit
`SIFR_E2E_CARGO_BUILD_JOBS` capped by the per-worker budget. Other ordinary crate
configurations retain their existing execution policy; generated
build tests retain their separate native allocation.

Installed metadata public-emission checks use the same resolved slot limit,
exported by the profile runner as `SIFR_VERIFY_PROCESS_SLOTS`. Each parallel CLI
uses one Cargo/Rayon worker; direct invocation without a profile allocation runs
serially. The batch owns all child process groups on the main thread, retains
per-command outcomes and restores corpus report order. Exact output/source-map
checks still run for every fixture. Native examples and shared package-generation
mutations remain sequential. Batch elapsed time includes queueing; overlapping
command durations must not be added to it.

Local correctness runs that select all four coverage, core-language, diagnostics
and distribution areas admit them together at four process slots, one slot per
area. Currently this applies to `merge`; `create-pr` omits distribution and keeps
its existing schedule. Smaller allocations retain serial execution. Each area keeps
its canonical suites, internal case ordering and separate fresh result report;
this does not make arbitrary areas or their cases parallel-safe. Shared Cargo
storage retains Cargo's locks, and the prepared compiler remains immutable.
Missing preparation blocks its consumers, and missing or incomplete reports fail.
The existing Python/quality 1+3 allocation and full-profile/release execution are
unchanged. Group elapsed time includes all children and must not be calculated by
adding overlapping area durations.
The group starts at the first selected member. Already-admitted siblings finish
when one fails; without `--no-fail-fast`, later profile steps then stop. Reports
retain canonical area order regardless of execution order.

## Layout

- `runner/sifr_verify/` contains runner code and self-tests.
- `schemas/` contains the supported committed data schemas.
- `profiles/` contains profile JSON files selected by `scripts/run_all_tests.sh --profile`
  and executed by `sifr_verify profiles run`.
  Profile v2 data owns `crate_test_membership`, the executable list of cargo
  crate suites per profile mode. The runner rejects unknown workspace packages,
  mismatched `cargo test -p` package names, duplicate suite ids, and red blockers
  without execution deadlines.
- `areas/` contains area-owned manifests, fixtures, baselines, and adapters.
  `coverage_matrix` owns the shipped guarantee registry and compiler surface
  matrix. It also owns `data/cargo_metadata_classification.json`, which maps
  every Cargo workspace package, target, and feature to its verification
  assignment. The `coverage_matrix:readiness` suite is selected by all four
  profiles and runs strict readiness mode plus profile-assignment checks. It
  rejects temporary rows (`expected-missing`, `tests:none`, `red-blocker`),
  unknown or unassigned owners, missing profile membership, non-offline
  create-pr/merge policy, v1 stable-surface manifests, and unpinned required
  corpora.
  `diagnostics` is migrated and can be run with
  `uv run --project verification python -m sifr_verify areas run --area diagnostics`.
- `policy/` contains machine-facing runner policy such as guardrail mappings.
  `github_actions.json` owns the exact release label and commit for each
  external action in maintained workflows. Use
  `python3 scripts/check_github_action_pins.py` to validate all references.
  The validator rejects mutable refs, unknown actions, label drift, and weak
  artifact-digest behavior.

## Profile Ownership

- `create-pr` is a fast representative profile. It selects readiness coverage,
  diagnostics rules, runtime/platform support evidence, algorithmic manifest
  checks, static/LSP smoke tooling, generated-code smoke, performance smoke, and
  stdlib module merge checks.
- `merge` is the authoritative local gate. It selects the readiness coverage
  suite, full first-party compiler crate membership, full semantic e2e pass
  corpus, diagnostics baselines, representative generated-code/performance
  suites, CPython hand-seeded differential checks, package offline smoke,
  stdlib module merge checks, runtime/platform evidence, regression, fuzz smoke,
  and curated ecosystem checks.
- `nightly` and `release` run the same readiness coverage suite plus broader
  generated-code quality, performance, distribution, CPython differential,
  sanitizer-full, ecosystem-broader, and module-full stdlib parity suites.
  Both profiles run the complete pinned algorithm corpus and taxonomy self-test.
  Release and nightly both retain unmodified full generated-code Clippy
  coverage.

Crate test membership is data-owned by `crate_test_membership.suites` in each
profile. The coverage matrix cross-checks that first-party compiler crates with
tests are in merge membership and executed; temporary red blockers are illegal
at readiness.

## Baselines And Blessing

Verify diagnostics baselines with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines
```

Bless only intentional baseline changes with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless
```

Baseline metadata, source hashes, stale/unused baseline detection, and recovery
surface coverage are enforced by `diagnostics:rules`.

## Fuzz, Sanitizer, And Release Evidence

Sanitizer manifests declare required source files and validate them before builds.
Instrumented compiler and generated-program builds both target the current host.
Caller Rust flags and build storage are preserved; conflicting sanitizer flags
fail before execution instead of silently changing the declared instrumentation.

Deterministic local fuzz/property evidence:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke
```

Runtime/platform sanitizer evidence:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-smoke
uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-full
```

Release evidence is emitted under `target/validation_lane_reports/` and the
area-specific `target/verification/areas/**` result files. A readiness archive
must record commit SHA, OS/toolchain, emitted profile plans, suite counts,
report signatures, and hashes of the validation report JSON files.

Schemas intentionally support only a small subset: object shape, required keys,
primitive scalar types, arrays of objects or strings, enums, booleans, integers,
and repo-relative path strings. Unsupported schema keywords are rejected.

## Focused fixture collection

The existing area manifests remain the fixture authorities. Shared baseline
adapters accept exact suite/case selections and failure reports:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --case baselines/parser_bad_indent --no-fail-fast
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --rerun-failures target/verification/areas/diagnostics-results.json --no-fail-fast
uv run --project verification --locked python -m sifr_verify profiles run --profile merge --no-fail-fast
uv run --project verification --locked python -m sifr_verify areas inventory --output /tmp/inputs.json
uv run --project verification --locked python -m sifr_verify areas inventory --output /tmp/current-inputs.json --compare /tmp/inputs.json
```

Inventory comparison authorizes reuse only when inputs are unchanged; it never
creates a new test execution. Added fixtures and changed validation inputs change
the inventory digest. Phase records alone do not. An inventory is an input
manifest, not a replacement for the canonical area/profile result. Ownership
roots cover compiler sources, generators, fixtures, vendored inputs, scripts,
workflows and tested diagnostic documentation without a suffix allowlist.
Initialized submodules contribute both their declared/checked-out identity and
nonignored contents, including dirty files and additions. Outside source ownership roots, inventory derives file constants from the
guardrail policy's entrypoints and the documentation inventory's active consumers.
It also reads the active taxonomy check's declared roots (including the root README)
and uses the compatibility check's own scan roots and skip predicate for its
document/config sweep, and the documentation consumer's public-page selection.
Consequently tested prose is an input; unconsumed prose and phase records remain
excluded. New declared config inputs and newly scanned public pages invalidate
evidence without editing an inventory suffix list.
Files inside an input root remain inputs even when their suffix resembles prose:
generators and tests can consume Markdown and arbitrary future asset types.

The shared adapter prepares its compiler through Cargo once per process and
checks configured compiler overrides against the prepared artifact. Setup,
signals, output truncation and deadlines cannot satisfy a language-negative
baseline. Failed and blocked cases remain failures during collection and bless.
Raw diagnostics remain in the area report and mismatch artifacts; failures stream
as each case finishes. Specialized area runners retain their suite selectors.

Generated-code smoke, representative and full modes now run explicit release
link/runtime assertions for the two safe codegen demo companions in addition to
their existing Rust-check, snapshot, formatting and quality obligations. Their
declared stdout is independent of equivalence comparisons. Full E2E/release
coverage remains selected. Cargo freshness never skips a selected runtime
assertion. Native stage details are referenced from the area result.

The fixture comparison hook requires two executable providers and an independent
expected result. DX.4 exercises source/native controls only; metadata and
persistent providers are not qualified until their provider qualification suites connect and
test them. Diagnostic example checks execute explicitly selected standalone
check-fail/check-pass pairs and their explain/help surface; contextual package and
runtime examples are not indiscriminately executed.

### E2E fixture execution directories

Run-pass fixtures execute in individual owned directories beneath their native
cache group (`runs/<fixture-prefix>-run <unique>/work`). The short prefix includes a space
so ordinary runs exercise paths that require correct argument handling. Fixture
cleanup uses filesystem APIs for temporary paths. The working directory and `TMPDIR`,
`TEMP`, and `TMP` point to that directory; child processes inherit them. Fixtures
create their data there and must receive any external resources explicitly.
Writable fixture paths use relative names or `sifr.tempfile`; absolute-path
parsing assertions remain independent of actual filesystem output.

Successful attempts remove only their own scratch directory after process
cleanup. A failed launch, nonzero exit, timeout or interruption retains the
attempt directory, fixture name and available captured output. Failure messages
identify that directory. Group retention also preserves incomplete attempts.
Compiler-input tracking is unchanged: test output is moved out of its inputs,
not excluded from source identity checks. The regular 30-minute subprocess
safety deadline and child cleanup still apply.

### Standalone parity and runtime-failure fixtures

Standalone parity and runtime-failure programs reuse the same Cargo validation,
resolution and artifact-selection machinery in a separate
`standalone/cargo-v3` namespace. Programs keep their original crate-root Rust
and main function; each selected fixture compiles and executes independently
without a batch wrapper. Cargo validates every invocation, and the original
stdout/stderr/exit assertions remain the acceptance authority.

Relative native cache paths resolve from the source checkout root for this
consumer, keeping generated outputs outside watched crate source roots.
Absolute selected paths remain absolute. A process-local publication lock and
the native cache lease cover compilation through runtime consumption; competing
external owners fail closed. Different generated inputs receive separate
packages and working directories. Compatible dependency artifacts are shared.
Nonzero and interrupted execution logs are preserved, including expected
runtime failures; normal group retention keeps their evidence.

### E2E native dependency reuse

Every generated run-pass group is validated by Cargo. The owned cache uses a
`cargo-v3` namespace with a persistent `native-target` across groups; Cargo owns
source, toolchain, profile, configuration and feature freshness. Generated source
and manifests are replaced only when their bytes change. Native builds serialize
with the explicit `--cargo-build-jobs` budget; requested group workers and actual
workers are printed separately. Fixture selection is unchanged.

Native project names bind the fixture/source/generated Rust, rendered Cargo
manifest and vendor configuration. The complete compiler dependency plan still
owns grouping, metadata assertions and resolution freshness. A changed plan
resolves a fresh lock in an isolated staging project before publishing it into
the retained native project. Unrelated workspace-lock entries therefore do not
rename identical native code. Previous resolution inputs and failed staging
directories are retained. Successful staging is removed after publication.

The resolution receipt records which plan received fresh resolution; it is not
a lock checksum or a passing test result. Cargo validates the lock and all
dependencies on every build, including path-manifest edits that can legitimately
update a lock. Corrupt Cargo inputs, failed resolution and failed builds cannot
authorize execution of an old binary.

Executables are selected from successful Cargo JSON output for the exact package,
manifest and binary target. Fresh executables also require a matching integrity
receipt. A failed build, missing executable, malformed result or unexpected bytes
cannot become a cached pass. If an executable is damaged or an interrupted build
left no receipt, clean only the generated package with Cargo using the owned native target and
its recorded profile/target configuration, then retry. Removing only the visible
executable is insufficient because Cargo can restore a damaged linked artifact. Retained native dependencies do not need deletion.

`--no-cache` / `SIFR_E2E_DISABLE_CACHE=1` disables the outer success-record tracking
and TTL retirement for that invocation. It still validates through Cargo, checks
artifact integrity and reuses native dependencies. A cold diagnostic requires a
new owned `--cache-dir`; this flag does not claim a cold build.

A checkout ownership record and suite lease protect generated mutations, manifest
updates, builds and running consumers. Contention fails explicitly instead of
racing or waiting indefinitely. Native commands use the existing bounded process
capture with a 30-minute deadline and child cleanup. Completed group retirement uses Cargo package-scoped cleanup for each recorded
profile/target before removing generated inputs. This removes the retired group's
executables, fingerprints and incremental output while preserving shared native
dependencies and other groups. Groups selected by the current plan are protected
from TTL retirement before use; expired success records are still discarded and
do not authorize a pass. Obsolete unselected groups remain eligible for retirement.
Failed, interrupted and unmarked group directories are retained; a failed cleanup retains the group and its log. Do not remove the cache or its permanent lock while consumers exist.

Cargo source freshness is timestamp based, not a claim of content checksum
validation for all dependency inputs. Generated writes compare bytes even if
input timestamps are preserved; dependency edits deliberately preserving Cargo's
freshness metadata require explicit rebuild/new owned cache. Executable integrity
receipts cover executable bytes, not dependency freshness. Cold bootstraps and
focused subset runs are not full-gate performance evidence.

The `cargo-v3` root must be owned and must not contain symlink ancestors. Select
an ordinary owned directory if a development target is symlinked. Legacy caches
outside this namespace are left intact for explicit owner-managed retirement.
Inherited leases may remain busy while a descendant such as a compiler-cache
server is alive. Preserve such consumers; use a separate owned cache if needed.
Old failure evidence remains preserved even after a later successful retry.


### Generated-quality dependency artifacts

Profile runs retain Cargo artifacts under
`target/sifr_generated_code_quality/<profile>.shared/cargo-target`.
A new Git revision does not create another dependency target. Each preparation
still creates fresh generated source directories, and every portable manifest
and lock must name the exact current Sifr revision. Source generation, checks,
runtime assertions and failure evidence remain fresh for each invocation;
only Cargo dependency artifacts are retained. Cargo remains responsible for
dependency, compiler and profile invalidation. Profiles keep separate owned
roots; standalone area invocation behavior is unchanged.

### Preparation failures and independent feedback

Profiles retain default fail-fast execution. With `--no-fail-fast`, preparation
records each existing adapter stage and blocks only selected checks whose required
stages did not pass. Workspace fetch is shared; crate-test, generated-code,
tooling, performance, oracle, sysroot and area preparation have distinct results.
The current execution boundary remains a selected area or toolchain step; this
change does not split an area's own suites or a combined crate-test step.

Static guardrails and formatting do not depend on unrelated Cargo builds. Checks
that run after partial preparation still use the profile's offline policy. Missing
or blocked prerequisites never count as successful, and any required preparation
failure leaves the aggregate nonzero. Every selected top-level step receives one
terminal outcome, including steps blocked by fail-fast or global admission.

Invalid inventories, unsafe compiler identity, cancellation, safety deadlines and
preparation filesystem errors stop further work. Performance-reference admission
is unchanged. Structured `[sifr-preparation]` records appear separately in report
JSON as `preparation_steps`; their times are part of `cargo_cache_setup`, not extra
wall time to add to that aggregate. Child output cannot forge these results.

### Reading E2E iteration timing

`run_e2e_pass.sh` retains its Cargo test invocation and reports whole-shell time
with one-second resolution. Profile execution additionally records precise
`[sifr-command]` start/end events, exit cause and output truncation. Its progress
heartbeat shows the active command and the last observed compiler, Cargo-lock
or E2E-stage message with its age. Buffered native Cargo output cannot identify
a currently active compiler/linker unit; missing output is not evidence of a hang.

`[sifr-e2e-event]` spans separate harness setup, source compilation, native builds,
diagnostic isolation and execution. The report's `traces.e2e_accounting` sums only
non-overlapping direct suite stages and retains unattributed time. Whole command
time also includes Cargo/compiler preparation, shell/profile startup and test
process teardown outside the measured harness span. Nested process/native-attempt
spans and per-case worker timings are already included in stage walls: do not add
them again. Native Cargo counts include original failures and diagnostic retries;
the same shared dependency may appear in several attempts. Linking remains part
of native-build time unless independently measured.

Spans without a valid start/end pair are incomplete. Child and nested command
observations are diagnostic data; the profile runner remains the authority for
functional and performance-budget results. A test failure, blocked prerequisite,
safety deadline, cancellation and performance-budget failure remain distinct.

### Isolated native test scheduling

Generated driver and CLI native suites retain their exact ignored-test inventory
and one process per selected case. The runner admits at most two cases at once,
with at least 4 GiB working memory per worker and the profile memory reserve.
Inner Cargo/Rayon settings share the existing slot budget; native graph preparation
and test execution use the same resolved settings. Ordinary compiler preparation
keeps its separate allocation. Admission can reduce concurrency without removing
cases. Allocation, admission and periodic memory observations appear in the log.

The existing bounded command executor owns each process group. Cancellation,
deadlines, unsafe outcomes, invalid execution evidence and exhausted memory reserve
stop dispatch and clean up active groups. `--no-fail-fast` continues independent
ordinary test failures. Final case lines follow selection order and explicitly mark
unexecuted cases incomplete. Quoted child output is emitted on command completion;
runner-owned command heartbeats remain visible while it runs. Overlapping case
seconds are diagnostic costs, not category wall time to add together.

Corpus profile execution uses the existing normal driver libtest and release
oracle. The metadata-corpus owner runs both against isolated output directories,
within its admitted process slots (maximum two); parallel children use one inner
worker. Both current outcomes and complete byte/map/module comparisons are required
before publishing a reference. Failed or incomplete output remains diagnostic
evidence and cannot satisfy the delegated normal assertion.


### Finite demo emission batches

Demo-companion freshness compiles every authoritative source on each invocation.
It uses bounded `sifr __emit-batch` workers to amortize immutable compiler setup.
A schema-1 JSON manifest contains an ordered `files` array of absolute paths;
the command requires a new output directory. Each input receives fresh source
provider/frontend execution, an indexed Rust file on success, and an indexed
status/diagnostic in the completion report. Repeated inputs are separate requests.
Missing, incomplete, reordered or inconsistent results fail verification.

Source trees, compiler metadata and toolchain inputs must remain immutable during
a batch, just as during a scalar compile. A new invocation owns changed inputs.
No generated output or old correctness result is cached. The verification caller
resolves the pinned formatter once; explicit RUSTFMT remains authoritative.
Failed batch directories are retained and their location is reported. Successful
private directories are removed after every output is checked. Public scalar CLI,
generated-program execution and independently constructed certification remain
separate assertions.

CPU attribution is recorded separately in `[sifr-resource-scope]` events and the
report's `resource_scopes` object. Preparation nodes, scalar steps, crate
configurations and serial commands measure their owning process plus descendants
whose resource usage has been reaped. Parallel areas measure inside each worker;
native batches are measured as whole configurations, not by subtracting shared
parent counters for overlapping children. No launcher, test selection or outcome
changes are implied.

Each scope records its process ID, scope ID and same-process parent ID. Nested
CPU and wall measurements are inclusive: never sum a parent with its children,
and never sum overlapping wall durations. CPU includes orchestration within the
scope; it differs from the profile time file's child-only CPU. The counters do not
measure aggregate peak memory, lock waiting, or work in abandoned/unwaited
descendants. A returned scope is not a correctness pass. Raised scopes retain
observed work; blocked work has no measurement. Child-emitted scopes remain
separate observations and cannot establish gate status.

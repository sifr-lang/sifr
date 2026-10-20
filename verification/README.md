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

Start with the smallest check affected by the edit. The public facade defaults
to `merge`; choose a profile explicitly when preparing a candidate, rather than
running the full gate after every local edit.

| Change or checkpoint | Next command from the repository root |
|---|---|
| One compiler behavior | `cargo test --locked -p <crate> <test-filter>` |
| CLI tests excluding the E2E pass suite | `cargo test --locked -p sifr -- --skip test_e2e_pass` |
| Verification runner mechanics | Affected self-tests; the runner foundation suite is `uv run --project verification --locked python -m sifr_verify --self-test` |
| One verification area/suite | `uv run --project verification --locked python -m sifr_verify areas run --area <area> --suite <suite>` |
| Full E2E pass corpus | `verification/runner/e2e/run_e2e_pass.sh --profile merge` |
| Implementation PR candidate | `scripts/run_all_tests.sh --profile create-pr` |
| Final implementation merge candidate | `scripts/run_all_tests.sh --profile merge` |
| Planning, prose or review records only | `git diff --check`; verify changed local links, scope and status; no compiler build or broad gate |

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

Selected Python-area, SQL LSP, LSP crate and driver native-archive consumers depend on
the locked Python interop environment. Preparation uses that project's exact pins
and `.venv`, independently of an activated environment or ambient
`UV_PROJECT_ENVIRONMENT`; Cargo targets, flags and job allocations are preserved.
Environment preparation is separate from compiled Python examples. With
`--no-fail-fast`, an environment failure blocks its consumers while independent
work continues; an example failure does not invalidate a successfully prepared
environment. Bootstrap time remains part of preparation accounting.

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

### E2E native dependency reuse

Every generated run-pass group is validated by Cargo. The owned cache uses a
`cargo-v3` namespace with a persistent `native-target` across groups; Cargo owns
source, toolchain, profile, configuration and feature freshness. Generated source
and manifests are replaced only when their bytes change. Native builds serialize
with the explicit `--cargo-build-jobs` budget; requested group workers and actual
workers are printed separately. Fixture selection is unchanged.

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
dependencies and other groups. Failed, interrupted and unmarked group directories
are retained; a failed cleanup retains the group and its log. Do not remove the cache or its permanent lock while consumers exist.

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

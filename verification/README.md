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

During focused compiler work, run an affected crate/test with `cargo test -p
<crate> <test>`. The CLI unit-test command excluding the slow E2E pass suite is
`cargo test -p sifr -- --skip test_e2e_pass`; its standalone E2E entrypoint is
`verification/runner/e2e/run_e2e_pass.sh`. These commands supplement the applicable
profile gate rather than replace it.

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

## Final gate on a shared Linux VM

Use the existing `cloud` profile with required performance on a shared Linux VM:

```bash
SIFR_CLOUD_PERFORMANCE_RECEIPT=/absolute/path/to/receipt.json scripts/run_all_tests.sh --profile cloud --require-performance --compact-resources
```

`cloud` inherits all merge selections: the same suites, guardrails, crate tests,
E2E corpus, toolchain steps and skip rules. The compact option's compiler settings,
allocation estimates and reserves are documented in the compact-resources
paragraph below. Correctness runs independently of physical
host admission; the final gate also requires the independent checker to accept a
complete, fresh, candidate-bound paired receipt. Missing, invalid, inconclusive or
regressing performance cannot pass this command. `cloud` without
`--require-performance` remains a correctness command and cannot qualify this
final gate. The default `merge` command retains its controlled-host performance
route.

This shared-VM command is an acceptance route, not evidence that any candidate
has passed. It does not waive implementation PR validation, native platform or
release qualification, external dependency disposition or protected enforcement.

An explicitly preserved local measurement worktree can be named with
`SIFR_CLOUD_PERFORMANCE_SOURCE_WORKTREE`. This narrow reuse recipe requires both
clean full worktrees in the same Git repository, an ancestor measured commit,
unchanged live compiler/runtime/corpus trees and Python context, and identical
compiler bytes/build identity. Only the recipe's enumerated documentation,
native-capacity and consumer files may differ; unknown changes invalidate reuse.
The original worktree's checker must still accept the unchanged complete receipt.
An immutable consumption record keeps both measured and current commits visible;
it adds zero execution assertions. A changed compiler binary requires fresh
qualification even when its source changes appear unrelated. There is no
guarantee that this route will accept a later candidate.

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

Linux verification subprocesses use a dedicated child subreaper. It terminates
and reaps the command's descendants, including descendants that create another
session, after completion, cancellation, or a safety deadline. PID file
descriptors prevent cleanup signals from reaching a reused PID. The execution
host must provide Linux subreaper and pidfd support; startup rejects unavailable
custody before launching the command. Other hosts retain session-group teardown.
A private completion channel confirms cleanup and the native command status;
missing confirmation or supervisor failure is an infrastructure error. A fatal
supervisor failure cannot establish that escaped descendants were reaped and
never qualifies a successful run. Unrelated children remain outside this custody.
Startup blocks cancellation signals until handlers are installed. Linux teardown
keeps the leader unreaped until its process group has been terminated, so that
group's identifier cannot be reused during cleanup.

Correctness checkpoints use `policy/correctness_checkpoints.json` and the existing
input-bound evidence schema. The first audited recipe is the HIR maintainability
guard; cloud execution and `uv run --project verification --locked python -m
sifr_verify checkpoints --guard hir-maintainability` can consume it. Other
assertions execute fresh until their complete dependency closure is declared.
The recipe pins the guard's audited bytes, consumed document and path absence,
whole source inventory, commands, interpreter/stdlib/native-library bytes,
configuration, and a known local producer. Its guard and custody supervisor use
isolated Python imports and source reads. Changed inputs, expired/tampered output,
incomplete inventory, duplicate cases, or another producer reject reuse. Partial
and failed attempts remain retained; runtime/compile/validation kinds stay distinct.
No checkpoint grants paired-performance acceptance or combines partial captures.
Checkpoint capacity or unknown dependencies disable reuse and execute the required
guard fresh. Observations distinguish reuse from new execution. This one recipe
does not claim dependency closure or checkpoint coverage for the full profile.

Cloud sysroot preparation retains source-bound immutable outputs before running
the full selected runtime suites. Source compilation retires its private graph
after bounded lossless compression and byte verification, then separately admits
restoring the executable. Packaging retains the verified archive and compiler
before retiring its private release graph. Library corpus/metadata preparation
follows these lifetimes. Runtime adapters independently recompute the same
isolated producer identity and rehash the consumed outputs; invalid declared
receipts fail closed. A preparation receipt records zero runtime assertions.
Unknown cache owners can be consumed through Cargo but are never cleaned.
The default cloud policy retains its 8 GiB disk reserve; compression adds no
acceptance claim.

Linux source profiles can explicitly select `--compact-resources` with
`create-pr`, `merge`, `nightly` or `cloud`. This prospective policy uses
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0`, a 2 GiB disk reserve
and at least 1 GiB additional monitored stopping headroom. Conflicting compiler
settings fail before execution. It preserves the canonical suite, fixture and
assertion selections, safety deadlines and performance qualification contracts.
Each sysroot graph uses a new session UUID; only leased graphs owned by that
session retire. Selected sysroot assertions run after their graphs retire and
before the remaining library preparations. Preparation never counts as an
assertion. Stage journals retain admissions, configuration, failures and actual
resource observations; estimates can fail closed and do not promise capacity.
Generated compiler preparation and assertions both use offline Cargo after
separate dependency acquisition. Quantitative performance remains last.

Selecting `build-qualification` in `area_sql_platform` gives the clean-build
assertion part `sql-build-qualification` admission and disk monitoring. Its prospective growth is 6 GiB: compact retains its 2 GiB reserve
(8 GiB entry), and normal cloud retains its 8 GiB reserve (14 GiB entry).
The monitored floor is the greater of entry free space minus 6 GiB, the policy
reserve plus 1 GiB stopping headroom, and any stricter caller floor. Thus an
8 GiB compact entry allows at most 5 GiB growth before the 3 GiB floor.
Memory remains 6 GiB resident plus 2 GiB reserve. The SQL tool's independent
8 GiB clean-build entry guard, native A/reused-A/independent-B commands,
incremental setting, cases and deadlines stay unchanged. Other or unknown
selections keep their existing allocations and monitoring behavior. This models
the existing entry envelope prospectively; actual SQL peak growth, cold fit and
full acceptance remain unmeasured. It grants no assertion reuse or graph cleanup.

When SQL selects `build-qualification`, resource-aware execution runs that actual
clean-build suite first, immediately after successful sysroot consumers. It needs
none of the 30 SQL no-run preparations. After its private graphs close, those
same 30 preparations run, followed by the other 18 suites. This retains all 19
canonical suites and 66 cases, including parser-major and SQLite probe native
assertions. The remainder uses the existing `remaining-assertions` allocation;
both parts keep disk monitoring, policy reserves, stopping headroom and stricter
caller floors. These are prospective attempts, not measured fit guarantees.

Fresh invocation-owned part results are checked against the exact manifest cases
and subprocess outcomes, then assembled in canonical suite/case order at the
original SQL result path. Missing or invalid execution becomes explicitly blocked
cases with its cause. One canonical area outcome covers the complete selection;
no part from another invocation can qualify it. An ordinary completed build-case
failure still runs the other SQL cases, as the existing SQL adapter does, before
outer fail-fast acts. Infrastructure without complete execution evidence may stop
earlier; `--no-fail-fast` continues independent admitted work and retains failure.
Later scheduling omits only this invocation's already handled SQL preparation
and assertions. Reports show the real phase chronology.

The two assertion commands share the original 2400-second cumulative safety
budget. Interleaved preparation time is excluded; preparation keeps its existing
limits. Tighter inherited command, step and absolute deadlines still win, and
environment limits are restored afterward. Canonical SQL elapsed time sums its
assertion parts. No profile selection, SQL build recipe, checkpoint, previous-pass
reuse or cleanup contract changes. SQL preparation and later native assertions
may still exhaust their forecasts; actual cold fit remains unproven.

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

## Stage contracts and execution evidence

The [validation contracts](policy/validation_contracts.md) derive stage selections
from canonical profiles/manifests and distinguish selection from execution:

```bash
uv run --project verification --locked python -m sifr_verify contracts check
uv run --project verification --locked python -m sifr_verify contracts plan --stage merge
```

The correctness evidence schema requires complete selected-ID accounting, actual
execution, explicit execution kinds, trusted producer expectations, source/runtime/
artifact bindings and retained-byte verification. Correctness checkpoints cannot
qualify paired performance. Resource scheduling and protected CI enforcement are
separate requirements; a contract plan alone is not an execution receipt.

## Generated-program allocation observations

`verification/areas/performance/generated_program_allocations.py collect
--prepared <prepared.json> --output <private-external-directory>` builds separately
instrumented artifacts from the exact release/generic generated Rust preparation.
`check --output <receipt.json>` independently verifies the retained transformation,
Cargo/rustc events, artifact and dependency hashes, raw output, program oracle and
counters. Run both commands with the locked verification Python environment.

The registered `rust-globalalloc-main-v1` protocol counts successful Rust allocator
requests during generated main for the single-threaded registered workloads.
It excludes libc/loader allocations and post-main cleanup. Counts belong to the
instrumented artifact: instrumentation can affect optimization, so these counts
cannot describe the original timed binary or qualify numeric regressions.
Incomplete attempts retain state and logs without publishing a success receipt.

RAM-backed worktrees are admitted by their actual Linux mount identity. Prospective
build and retained-copy growth on tmpfs/ramfs is added to the resident-process and
other temporary-storage memory budget. The filesystem disk reserve remains in
force. Nested disk mounts are resolved separately; advertised tmpfs capacity never
adds to the cgroup memory limit.

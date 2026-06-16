# Ad Hoc Phase: World-Class Verification Standard and Gate Closure

Status: in progress; Wave 0, Wave 1, Wave 2.0, Wave 2.1, Wave 2.2, Wave 2.3, Wave 2.4, Wave 2.5, Wave 2.final, Wave 3, Wave 4 diagnostic-baseline slices through final BUILD/STDLIB/legacy-WORKSPACE synthetic-baseline coverage, Wave 5.1 parsed-source shape inventory, Wave 5.2 HIR-lowering snapshots, Wave 5.3 name-resolution snapshots, Wave 6.0 CPython divergence catalogue plus hand-seeded merge smoke, Wave 6.1 generated CPython differential suites, Wave 7.1 fuzz target contract/minimization commands, Wave 7.2 per-target fuzz dispatch plus merge smoke, Wave 7.3 sanitizer/runtime-platform hardening lanes, Wave 8.1 through Wave 8.4 trend-policy/edit-equivalence/trend-report/output-size trend slices merged through PR [#2640](https://github.com/sifr-lang/sifr/pull/2640), and Wave 8 closeout merged through PR [#2641](https://github.com/sifr-lang/sifr/pull/2641).
Owner: compiler-verification
Context: Follow-on verification phase after Phase 29; based on local Sifr verification audit against TypeScript, TypeScript-Go, Rust, CPython, and Bun

## Problem

Sifr has a strong verification framework: profiles, area manifests, policy documents, machine-readable reports, e2e fixture discovery, generated-code quality checks, diagnostic contracts, curated ecosystem hooks, regression manifests, and deterministic runner mechanics.

That framework is not yet a world-class verification standard in execution. The current gap is not naming or process. The gap is that several compiler-critical surfaces are thin, non-blocking, or represented by aspirational policy rather than enforced gates.

The most serious concrete mismatch is that `sifr_codegen` has the largest codegen unit/snapshot suite, but it is not in the authoritative merge gate and is currently known to have failing tests. The merge profile also omits passing first-party crates such as `sifr_type_system`, `sifr_format`, and `sifr_lint`. Diagnostic renderer baselines are thin compared with the fail-fixture corpus. Fuzz/property lanes are deterministic smoke tests, not sustained compiler hardening. LSP/tooling and ecosystem coverage exist but are closer to smoke signals than TypeScript/Bun-scale corpus gates. There is no first-class generative CPython-vs-Sifr miscompilation oracle even though Sifr's Python syntax gives the project an unusually strong reference semantics source.

This phase turns the existing verification machinery into an executable standard: every shipped compiler guarantee must have an owner, support-boundary entry, blocking evidence path, broader hardening path, regression story, and reproducible artifact.

## Verdict From Review Synthesis

The reviews agree on the same core assessment:

- Sifr's verification architecture is strong and unusually well-organized.
- Sifr's verification execution is not yet at Rust, TypeScript, CPython, or Bun level.
- The highest-risk gaps are gate completeness, codegen enforcement, rendered diagnostics depth, miscompilation detection, IR/lowering snapshots, sustained fuzzing, sanitizer lanes, LSP corpus depth, ecosystem breadth, and platform evidence.

This phase adopts the stricter interpretation: world-class verification is defined by what blocks merges and what produces durable evidence, not by the existence of policy files.

## World-Class Verification Standard

For every stable shipped Sifr guarantee, there must be:

- an entry in the shipped guarantee registry
- a registry support status of `stable`
- a fast or representative merge-blocking case
- a broad nightly or release hardening case
- a permanent regression path for failures found later
- machine-readable evidence from the validation runner
- a documented owner, suite kind, and reproduction command

Experimental and internal guarantees follow the matrix status semantics: they may be `blocking` or `broad-only`, but must still have owner, support status, reproduction command, and documented evidence path.

Every stable shipped compiler surface must have at least one profile-blocking suite at its assigned support level; stable user-facing guarantees require merge-blocking representative evidence unless explicitly deferred during this phase.

- parser and syntax acceptance/rejection
- compiler crash/ICE behavior for invalid and adversarial user inputs
- diagnostics, spans, renderer output, JSON schema, compact output, and recovery
- HIR lowering and name resolution
- type checking and ownership/flow analysis
- codegen lowering and generated Rust structure
- generated Rust quality: `rustfmt`, clippy, panic/unwrap scans, determinism
- runtime behavior and platform behavior
- project/workspace/package behavior
- CLI behavior and exit codes
- language service behavior for diagnostics, hover, definition, completion, and long sessions
- performance budgets and drift evidence
- fuzz/property hardening and minimized regression promotion
- sanitizer/leak/thread/UB hardening where applicable
- ecosystem and stdlib compatibility

Parser acceptance/rejection and parser fuzzing are distinct surfaces. Parser acceptance/rejection is a curated positive/negative source corpus. Parser fuzzing is an input-hardening lane. Both require matrix rows, but they do not satisfy each other.

## Decisions

- This is a new ad-hoc issue phase under `plans/issues/active`, not a rewrite of completed Phase 29.
- Phase 29 remains the foundation. This phase owns enforcement, breadth, and promotion to a world-class executable standard.
- The authoritative merge profile must become profile-data-driven enough to express crate test membership explicitly; hard-coded omissions are not acceptable after this phase.
- Local validation remains authoritative for PR and merge readiness. CI mirrors local profile commands and may add broader evidence, but implementation work must not wait on CI and must not rely on CI-only behavior.
- Create-pr and merge profiles must be hermetic/offline by default: no network, pinned toolchains and corpus revisions, tempdirs only, deterministic locale/timezone unless explicitly varied, and no hidden dependence on user-global caches. Live-network package or ecosystem checks are nightly/release signal only and never required for local merge.
- `cargo test -p sifr_codegen` must be made green and added to the merge gate before this phase can close.
- `cargo test -p sifr_type_system`, `cargo test -p sifr_format`, `cargo test -p sifr_lint`, `cargo test -p sifr_source`, and every other first-party compiler crate with tests must be added to the merge gate.
- First-party compiler crates without tests must receive a minimal test seed in Wave 1. Based on local re-check during this planning pass, `sifr_ir` currently has zero tests and is the initial seed-test target. `tests:none` is reserved only for future data-only crates, requires issue link and expiry, and is illegal at phase close.
- Failing `sifr_codegen` is modeled as `red-blocker`, not as missing coverage. A `red-blocker` row must have a suite command, current failure count, owner, issue or triage file, reproduction command, and `closes_in_wave`; it is illegal at phase close.
- Failing `sifr_codegen` tests are triaged as one of:
  - stale expectation: update to the current correct contract
  - obsolete test: delete only after documenting the covered replacement
  - real compiler bug: fix root cause and add or update regression coverage
  - unresolved production bug: move to an explicit tracked crash/sentinel suite, not silent exclusion
- At phase exit, no first-party compiler crate test suite may be excluded from merge because it is red.
- Wave 0's coverage-matrix check lands in advisory mode with a closed list of `expected-missing` rows. Each row carries a `closes_in_wave` field naming exactly one wave in 1-9. Subwaves are expressed via `closes_in_subwave`, for example `closes_in_wave: 2` and `closes_in_subwave: final`; the matrix check rejects unknown wave or subwave names. The same wave may close several rows, but no row may be open after its named wave merges. After Wave 0, a new stable shipped guarantee may not be added unless the same PR adds a blocking row, or the phase owner approves a time-boxed `expected-missing` row with owner, issue, reproduction command, expiry, and `closes_in_wave`. Adding `expected-missing` for pre-existing Wave 0 surfaces remains forbidden. The check is promoted to blocking mode in Wave 10 and must have zero `expected-missing` rows at phase close.
- Merge must run the full e2e pass corpus unless measured warm/cold runtime exceeds `verification/profiles/merge.json` budgets on the declared reference host. Any exception must run deterministic full-corpus local shards as part of the authoritative merge command.
- Across merge, nightly, and release, diagnostic baselines must cover every active stable `SIFR-*` code and every stable renderer. Merge requires at least one rendered baseline per active stable code; nightly and release require every stable renderer for every active stable code. Code-only fail annotations are not sufficient for user-facing diagnostic presentation.
- Every baseline-backed suite must fail stale/unused baseline files, fixtures missing required baselines, unchecked blesses, nondeterministic output, and mass baseline updates without a per-suite summary.
- The CPython differential oracle is required for supported Python-compatible semantics. It is a semantic correctness gate, not a fuzzing nice-to-have.
- CPython differential generation cannot start until an authoritative divergence catalogue is checked in. The generator must lint against that catalogue and refuse to emit programs whose semantics depend on excluded behavior.
- Sustained fuzzing and sanitizer lanes may be nightly/release rather than merge-blocking, but deterministic smoke reproductions and minimized regressions from those lanes must become merge-blocking.
- Quarantine is allowed only through the existing flake/crash policy with issue linkage, owner, expiry, and reproduction command. Quarantine is not a way to make red suites disappear.
- No fallback compiler paths, compatibility bypasses, or "best effort" verification shortcuts are introduced by this phase.

Authoritative profile assignment after this phase:

| Surface | Create-pr | Merge | Nightly | Release |
| --- | --- | --- | --- | --- |
| First-party crate tests | representative where needed | all green first-party compiler crate tests | all | all |
| Cargo features/targets | manifest check | default features and shipped bins | all-targets/all-features/no-default-features policy | release target matrix |
| E2E pass semantics | representative subset | full corpus or deterministic local full-corpus shards | full corpus | full corpus |
| Parser acceptance/rejection | representative syntax corpus | stable syntax matrix | full syntax matrix | full syntax matrix |
| Lexer/token stream and indentation | representative lexical/indentation corpus | stable lexical and indentation matrix | full lexical/span matrix | full lexical/span matrix |
| E2E fail and diagnostics | full code/position checks plus representative baselines | full fail suite plus all required rendered baselines | same | same |
| HIR/CFG/lowering/codegen snapshots | representative changed-surface checks | blocking layer suites | full layer suites | full layer suites |
| CPython differential | deterministic smoke seed set | deterministic smoke seed set | broader generated corpus | broader generated corpus plus extended subset |
| Fuzz/property | deterministic smoke if runtime permits | deterministic smoke seeds | sustained per-target budget | sustained per-target budget |
| Sanitizers/leak/thread | none unless cheap and host-stable | smoke where host-supported | full supported lanes | full supported lanes |
| LSP marker corpus | smoke subset | documented capability coverage subset | full marker corpus | full marker corpus |
| Ecosystem curated | schema/manifest checks | curated blocking set | curated plus broader pinned set | curated plus broader pinned set |
| Algorithmic compatibility | manifest checks | representative LeetCode/algorithm subset | full corpus | full corpus |
| Package management | guardrails | stable integration smoke | full integration suite | full integration suite |
| Stdlib parity | audit-fixture guardrails | supported-namespace smoke | module-owned supported namespace suites | module-owned supported namespace suites |
| Runtime/platform | schema/contract checks | host-supported golden smoke | executable host/target evidence | executable host/target evidence |
| Performance | budget smoke | representative budgets | trend artifacts and broader benchmarks | trend artifacts and release benchmarks |

Target-matrix rows without an explicit profile-assignment row inherit the profile assignment of their owning compiler-surface row. For example, suggestions/autofix inherits diagnostics unless the owning area creates a more specific row.

Verification target matrix:

Targets are derived from inventories and shipped guarantees rather than arbitrary corpus sizes. If a target says "every," the owning wave must add the inventory file first, then make the verification runner fail when an inventory entry lacks the required evidence.

| Surface | Source-of-truth inventory | Merge target | Nightly/release target | Minimum content rule |
| --- | --- | --- | --- | --- |
| Shipped guarantees | `verification/areas/coverage_matrix/shipped_guarantees.json` | every `stable` guarantee has a blocking suite row, owner, reproduction command, and regression path | every `experimental` guarantee has broad signal or explicit unsupported boundary | no `stable` guarantee without merge evidence |
| Compiler surfaces | `verification/areas/coverage_matrix/compiler_surface_matrix.json` | every stable surface status is `blocking`; only time-boxed Wave 0 rows may be `expected-missing` before their `closes_in_wave` | broad rows cover non-merge stress, platform, fuzz, and ecosystem lanes | no orphan matrix row; no suite without matrix row |
| First-party crates | `cargo metadata` package list plus profile v2 crate membership | every first-party compiler crate with tests runs in merge; `sifr_ir` has seed tests | all first-party crate tests run in nightly/release | zero-test crate fails unless future data-only `tests:none` is time-boxed; none remain at closeout |
| CLI behavior and exit codes | `verification/areas/developer_tooling/data/cli_exit_code_contracts.json` plus `sifr` integration test inventory: `e2e`, `validation_contracts`, `build_output_contracts` | every documented CLI exit-code contract has a corresponding integration test | full CLI behavior matrix and broader exit-code scenarios | exit-code contract without integration test fails |
| Cargo features and targets | `cargo metadata` targets/features | default features, shipped bins, test-fixture targets, and planned red-blocker targets have merge classification | `all-targets --all-features`, `no-default-features`, examples, doctests, and release target matrix are assigned or unsupported with reason | every target/feature has exactly one classification: `merge`, `merge-red-blocker`, `nightly`, `release`, `internal`, `performance`, `test-fixture`, or `unsupported` |
| E2E pass semantics | e2e pass fixture discovery and manifest reports | full pass corpus or deterministic full-corpus local shards | full pass corpus plus broad hardening around it | executed fixture count must equal discovered count, excluding only explicitly unsupported fixtures |
| E2E fail semantics | e2e fail fixture discovery and inline expected-error markers | full fail corpus code, position, and contradiction checks | full fail corpus plus renderer and recovery stress | every fail fixture has canonical diagnostic code/position expectations |
| Lexer/token stream and indentation | lexer/token/indentation contract list in `verification/areas/core_language/data/lexer_token_inventory.json` | token boundaries, indentation/dedentation, newline handling, comments/trivia preservation if supported, Unicode identifiers if supported, and source-span stability each have representative fixtures | full lexical/span matrix including parser-fork boundary cases | parser syntax tests do not satisfy token/indentation coverage |
| Parser acceptance/rejection | stable syntax inventory in `core_language` or coverage-matrix data | one positive fixture per stable syntax construct and one negative fixture per stable syntax error family | full syntax matrix including edge cases and parser-fork boundary cases | parser acceptance/rejection cannot be satisfied by fuzzing |
| Project/workspace behavior | `verification/areas/project_workspace/data/workspace_contracts.json` plus suite manifests: `frontend_mode_parity`, `phase23_graph_isolation`, `baselines`, `audit-fixtures` | every shipped workspace and graph-isolation contract has a blocking suite row | broader project graph, workspace, and multi-module scenarios | workspace contract without suite row fails |
| Diagnostics catalog | `verification/areas/diagnostics/data/code_catalog.json` | every stable `SIFR-*` code has severity, owner, docs link, renderer support, and baseline coverage row | unstable/experimental codes have broad signal or explicit deferral | no diagnostic emitted by compiler without catalog entry |
| Diagnostic renderers | `verification/areas/diagnostics/data/code_baseline_coverage.json` | at least one rendered baseline for every stable `SIFR-*` code | human, compact, and JSON baselines for every stable `SIFR-*` code | stale/unused baseline detection is blocking |
| Diagnostic recovery | recovery-surface list in `verification/policy/suite_taxonomy.md` | one multi-error fixture per parser, HIR, name-resolution, and type-checker recovery surface | recovery stress cases with suggestions/related notes where applicable | a recovery surface with zero multi-error fixture fails |
| Suggestions/autofix | diagnostic code catalog `machine_applicable` field | every machine-applicable suggestion has emit/apply/recompile validation | broad suggestion application corpus | non-machine-applicable suggestions are render-only and explicitly marked |
| Codegen red blocker | `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json` and `plans/issues/active/codegen-test-triage.md` | every current failing `sifr_codegen` test has classification, owner, affected contract, and repair PR; then `cargo test -p sifr_codegen` becomes merge-blocking | no `red-blocker` remains | red suite is never modeled as missing coverage |
| Generated Rust toolchain survival | generated-code fixture classes and generated-code-quality manifests | every repaired stable fixture class proves emit, normalize, `rustfmt --check`, `cargo check`, runtime execution where applicable, and panic/unwrap scan | clippy and broader generated-code corpus | allowlist entries require owner, reason, issue, expiry |
| HIR/name/type/CFG snapshots | `verification/areas/core_language/data/lowering_layer_inventory.json` | representative blocking snapshot suite for each stable layer | full layer snapshot suite with stale/unused detection | each stable lowering/analysis contract has at least one snapshot or mapped equivalent |
| Codegen snapshots | `verification/areas/generated_code_quality/data/codegen_construct_inventory.json` | blocking snapshots for stable emitted constructs and structured codegen inputs | full snapshot suite with normalizer inventory | substring-only tests must be replaced or justified by contract |
| Crash/ICE contract | `regression` crash manifest and compiler-crash policy rows | invalid user programs produce diagnostics, not panics; known crashes are issue-linked sentinels | adversarial crash stress and fuzz-found crash promotion | sentinel fails if expected crash disappears |
| CPython hand-seeded differential | `verification/policy/cpython_differential.md` supported/excluded tables | hand-authored deterministic smoke suite covering each supported construct category | same plus broader seed set | canonical JSON-like serializer only; no `repr`/exception-message comparison |
| CPython generated differential | generator grammar, seed manifest, shrinker metadata | minimized generated seeds graduate to merge only after stability policy | generated corpus with per-program and suite timeouts | every generated failure is minimized before issue/regression promotion |
| Fuzz/property | `verification/areas/fuzz_property` manifests and `verification/policy/fuzz_property.md` | deterministic smoke seeds for parser/check, HIR/type/ownership, codegen, diagnostics renderer, package/project manifest | sustained valid and invalid fuzz lanes with corpus minimization | invalid-program and valid-program fuzz targets are separate |
| Sanitizers/concurrency | sanitizer/platform manifest | host-supported smoke or structured skip with reason | full ASan/LSan/TSan, Miri where feasible, and Loom/Shuttle-style concurrency where feasible | skipped lane must record reason and reproduction command |
| LSP markers | documented LSP capability inventory from `crates/sifr_lsp` | marker coverage for each stable documented capability category | full marker corpus | no documented stable capability without marker evidence |
| LSP transcripts | JSON-RPC transcript manifest | cancellation, out-of-order request, stale diagnostics after edit, project reload, and long-session smoke where supported | full transcript replay and memory/perf stress | marker tests do not satisfy protocol transcript coverage |
| Ecosystem curated | `ecosystem_compatibility` pinned corpus manifest | curated blocking set with checksum/revision/license/owner/commands | curated plus broader pinned set | live network is not allowed in merge |
| Algorithmic compatibility | `algorithmic_compatibility` taxonomy/corpus manifests | representative LeetCode/algorithm subset | full corpus and taxonomy delta reports | taxonomy row required for each included problem/category |
| Package management | package-management integration manifest and offline registry fixture | offline registry smoke, lockfile determinism, and package graph behavior | full integration suite plus live-registry signal if desired | merge package tests never use live network |
| Stdlib parity | stdlib namespace inventory and example/doctest inventory | supported-namespace smoke and every inventoried shipped example/doctest | module-owned parity suites for supported namespaces | zero-example inventory row required if no examples exist |
| Runtime/platform | `verification/areas/runtime_platform/supported_platforms.json` | host-supported golden smoke for declared supported host | executable host/target evidence for supported matrix | platform docs without executable evidence do not satisfy support |
| Distribution/release | `distribution_release` manifest and release evidence archive | representative install/distribution smoke | full release qualification with toolchains, OS, suite counts, report hashes | release evidence must include commit and profile plan |
| Incremental and determinism equivalence | clean-vs-incremental contract list owned by Wave 8 plus `check_report_determinism.sh` and `check_sequential_parallel_equivalence.sh` outputs | every shipped cache/query/incremental behavior has an equivalence fixture; report and parallel determinism remain blocking | full edit-run scenario matrix and long-session repeated-build stress | incremental contract without equivalence fixture fails; nondeterministic output fails |
| Performance trends | `verification/areas/performance/data/trend/` benchmark ids | representative budget checks | time/RSS/output-size/binary-size/LSP/package trend artifacts with environment metadata | benchmark id rename requires old-id mapping |
| Local/CI parity | `verification/profiles/*.json` plus emitted local-vs-CI profile plan equivalence artifact | local merge plan is source of truth; CI may not omit local merge checks | CI may add broader checks but must report deltas | CI-only behavior is invalid |

Matrix status semantics:

| status | Allowed for | Closeout allowed | Required fields | Meaning |
| --- | --- | --- | --- | --- |
| `blocking` | `stable`, `experimental`, `internal` | yes | owner, profile command, reproduction command, suite row | assigned profile command exists and fails the owning profile on regression |
| `broad-only` | `experimental`, `internal` | not for stable guarantees | owner, broad profile command, reason no merge representative exists | nightly/release coverage exists but no merge-blocking representative case exists |
| `expected-missing` | `stable` only during this phase | no | owner, issue, reproduction command, `closes_in_wave`, optional `closes_in_subwave`, expiry | known temporary gap that must close in the named wave/subwave |
| `tests:none` | temporary migration status for zero-test crates/surfaces only | no | owner, reason, issue, expiry, cargo/package evidence | temporary status while seed tests or a permanent `not-applicable` classification are added |
| `not-applicable` | `internal` or `unsupported` data-only crates/surfaces with no executable behavior | yes, never for stable compiler/runtime behavior | owner, reason, cargo target evidence, support status | permanent non-testable classification for data-only surfaces; example: a data-only crate that ships only static JSON consumed by tests, with no executable code path |
| `red-blocker` | known-red existing suite only with phase-owner approval | no | command, current failure count, triage file, owner, issue, reproduction command, `closes_in_wave`, optional `closes_in_subwave`, expiry | suite exists and is intentionally visible but not yet executed in merge |
| `quarantined` | flaky or host-specific failures only under quarantine policy | only if unexpired, issue-linked, owner-assigned, and not a stable shipped guarantee gap | owner, issue, reproduction command, reason, expiry, re-enable criteria | tracked temporary quarantine, not missing coverage |

Resolved implementation decisions:

- **Profile schema migration:** introduce `verification/schemas/profile.schema.json` schema version `2` for explicit crate membership, hermetic/network policy, profile-plan emission, reference host, and feature/target assignments. `legacy_facade` remains the migration surface inside v2 profiles; new v2 fields land alongside it and `legacy_facade` field removal is out of scope for this phase. Keep schema version `1` readable only for migration tests until Wave 1 closes; no profile may remain v1 at phase close.
- **Area schema migration:** introduce schema version `2` for owner, network mode, pinned corpus checksum/revision, skip policy, baseline metadata contract, and suite artifact declarations. Existing v1 area manifests migrate incrementally by wave, but Wave 10 blocks any remaining v1 manifest for a shipped stable surface.
- **Suite schema strategy:** keep `verification/schemas/suite.schema.json` at v1 minimal shape. Suite-level metadata, including owners, normalizers, artifacts, baseline metadata, and stale-baseline rules, lives in area-owned data files validated by per-area checks. No v2 suite schema is introduced in this phase.
- **Coverage registry paths:** the shipped guarantee registry lives at `verification/areas/coverage_matrix/shipped_guarantees.json`; the compiler surface matrix lives at `verification/areas/coverage_matrix/compiler_surface_matrix.json`; the profile assignment matrix check lives under `verification/areas/coverage_matrix/checks/profile_assignment_matrix.py`.
- **Diagnostic registry paths:** the diagnostic code catalog lives at `verification/areas/diagnostics/data/code_catalog.json`; renderer baseline coverage lives at `verification/areas/diagnostics/data/code_baseline_coverage.json`; the enforcement check lives at `verification/areas/diagnostics/checks/code_baseline_coverage.py`.
- **Red suite handling:** `sifr_codegen` is the only known initial `red-blocker`. It is tracked in Wave 0 with current failure count, triage file target, planned merge membership, and `executed_in_merge: false`, then resolved by Wave 2.final. No other suite may become `red-blocker` without phase-owner approval, issue link, current failure inventory, reproduction command, and expiry.
- **Ownership:** use `verification/owners.json` as the authoritative owner registry. Team-style owners are allowed until individual owners are assigned: `compiler-verification`, `codegen`, `diagnostics`, `runtime-platform`, `developer-tooling`, `package-management`, `stdlib`, `performance`, and `release-engineering`. `unassigned` and unknown owner ids are invalid.
- **Hermetic local-first rule:** create-pr and merge never require external network. Any suite requiring live network, remote registries, remote package indexes, or GitHub/API access is nightly/release-only signal and must not be required for PR or merge readiness.
- **Local/CI parity:** CI may execute the same local profiles and may add broader profiles, but it cannot be the source of truth. The runner emits a local profile plan and CI profile plan; CI is invalid if it omits a local merge check except for declared host skips.
- **Reference host:** merge budget decisions use the reference host recorded in profile v2 metadata. If a developer host is slower, deterministic sharding/parallelism is still implemented in the local command rather than moving merge-only coverage to CI.
- **Crate and target coverage:** `cargo metadata` is the source of truth for first-party packages, targets, examples, bins, tests, features, and required feature gates. Profile membership is checked from metadata rather than hand-maintained lists.
- **Feature policy:** default features are merge-blocking; `all-targets --all-features` is nightly/release unless measured budget allows merge; `no-default-features` is either nightly/release or explicitly unsupported with reason.
- **Secondary bins/features:** default profile classification for non-test bins and features is `internal` for compiler harness/tools or `performance` for benchmark bins. Test-only bins and features, such as `sifr_stdlib`'s `__test_fixture` feature and fixture bin, are classified as `test-fixture` and validated by their owning test suite rather than separate runtime profile membership. Each workspace bin and feature must carry a profile classification by Wave 1 close; the Wave 1 cargo-metadata guardrail fails if any first-party bin or feature lacks an explicit assignment.
- **Baseline metadata:** all baseline-backed suites use sidecar metadata or a manifest entry with fixture id, suite, format/snapshot kind, owner, source hash, normalizers, bless reason, and PR or issue reference.
- **CPython oracle:** hand-seeded deterministic CPython differential tests are merge-blocking immediately in Wave 6.0. Generated differential tests run in nightly/release first; only minimized stable generated seeds graduate to merge.
- **CPython serialization:** differential programs compare a canonical JSON-like serializer implemented in both Python and Sifr, not Python `repr`, display text, or exception messages.
- **Local Python prerequisite:** `python3` matching the version constraints used by `verification/pyproject.toml` is a required local development prerequisite for create-pr and merge profiles. Absence is a contributor setup failure, not a host skip. CPython differential hand-seeded smoke is therefore merge-blocking without weakening local-first validation.
- **Generated Rust toolchain support:** supported Rust toolchain/channel and any MSRV for generated output are recorded in a manifest. Generated Rust is checked against the supported toolchain. Nightly-only generated Rust is illegal unless marked `experimental` or `internal`.
- **Crash/ICE policy:** unexpected compiler panics are failures. Known crashes are issue-linked sentinels only; sentinels fail if the crash disappears so the issue must be closed or reclassified.
- **Fuzz policy:** invalid-program fuzzing and valid-program fuzzing are separate target classes. Invalid fuzzing hunts ICEs/diagnostic crashes; valid fuzzing hunts wrong-code/invariant breaks.
- **Sanitizer policy:** ASan/LSan/TSan are preferred where host-supported; Miri and deterministic concurrency modeling are attempted where feasible. Any skipped sanitizer/model lane records a reason and reproduction command.
- **Platform policy:** platform support is declared in `verification/areas/runtime_platform/supported_platforms.json`; executable evidence must match that declaration. Loopback-only networking is allowed in create-pr/merge; external networking is not.
- **Package policy:** package-management merge tests use an offline registry fixture and prove lockfile determinism. Live registry tests are nightly/release-only.
- **Performance policy:** performance evidence includes time, peak RSS, emitted Rust size, binary size, diagnostic rendering time, LSP indexing/edit latency, package resolution/install time when shipped, and environment metadata.
- **Performance blocking policy:** create-pr validates benchmark schema and smoke budgets; merge enforces stable representative budgets only; nightly/release produce trend deltas on reference hardware; trend regressions require owner review; local developer machines do not fail solely because of noisy trend deltas; checked-in trend baselines may only be updated from approved reference runs.
- **Execution sandbox policy:** generated binaries, CPython differential programs, ecosystem projects, package tests, and fuzz reproducers run with per-test timeouts, stdout/stderr size limits, tempdir-only writable outputs, no writes outside declared output dirs, subprocess cleanup verification, and no external network in create-pr/merge. Loopback networking is allowed only for suites that declare it.
- **Closeout policy:** Wave 10 cannot close with any `expected-missing`, `red-blocker`, expired `tests:none`, illegal `not-applicable`, ownerless row, undocumented quarantine, live-network merge requirement, or v1 stable-surface manifest.

## Scope

This phase owns:

- merge-gate closure for all first-party compiler crate tests
- codegen test triage and enforcement
- executable coverage matrix for compiler guarantees
- diagnostic renderer baseline expansion and recovery coverage
- HIR/CFG/lowering/codegen snapshot suite definition and enforcement
- CPython differential valid-program oracle for supported semantics
- property/fuzz evolution from smoke-only to sustained compiler hardening
- sanitizer/leak/thread hardening lanes for compiler/runtime/generated binaries
- incremental-vs-clean and deterministic rebuild checks where current architecture supports them
- LSP marker corpus for core IDE behaviors
- ecosystem, package, stdlib, and platform profile ownership
- performance trend evidence beyond point-in-time threshold budgets
- documentation updates that make the new standard durable

This phase does not own:

- adding new language features
- broad stdlib feature expansion unrelated to verification
- replacing the compiler pipeline
- changing user-facing semantics to make tests easier
- waiting for CI instead of validating locally
- fixing unrelated dirty worktree changes

## Existing Facts To Verify During Implementation

Implementation must start by re-checking these facts locally because the repository may have changed.

Verified during this planning pass:

- `verification/profiles/merge.json` declares the merge lane and uses `legacy_facade.e2e.fixture_manifest` pointing at `verification/areas/core_language/data/merge_e2e_manifest.json`.
- `verification/runner/sifr_verify/profile_runner.py` hard-codes crate tests in `run_crate_tests`.
- The hard-coded crate test list includes several core crates but omits `sifr_codegen`, `sifr_type_system`, `sifr_format`, `sifr_lint`, `sifr_ir`, and `sifr_source`.
- `cargo test -p sifr_type_system` passes with `92` unit tests.
- `cargo test -p sifr_format` passes with `7` unit tests.
- `cargo test -p sifr_lint` passes with `22` unit tests.
- `cargo test -p sifr_source` passes with `3` unit tests.
- `cargo test -p sifr_ir` passes with `0` unit tests and therefore needs seed tests in Wave 1.
- `verification/areas/algorithmic_compatibility` and `verification/areas/fuzz_property` already exist; this phase extends them rather than creating parallel runners.
- `cargo test -p sifr_codegen`: `655 passed`, `52 failed`, `707 total`; red and excluded from merge.
- `verification/areas/diagnostics/manifest.json` ships exactly two rendered baseline fixtures: `decimal_invalid_literal` and `multiline_span_rendering`.

Re-measure at implementation start:

- E2E corpus size under `verification/areas/core_language/`.
- Current warm/cold merge wall time on the implementer's host.

If any count has changed, update this issue's execution checklist with the current measured count before implementing gate changes.

## Execution Checklist

Implementation re-check started 2026-06-14.

- Current branch/worktree at start: `main` tracking `origin/main`, clean.
- E2E manifest counts: `create_pr_e2e_manifest.json` has 132 fixtures; `merge_e2e_manifest.json` has 145 fixtures.
- Post-Wave 3 E2E profile counts: create-pr still uses 132 selected fixtures; merge/nightly/release select the full discovered pass corpus of 651 fixtures with no fixture manifest.
- Current `.sifr` files under `verification/areas/core_language`: 186.
- Cargo metadata still lists the initially omitted first-party crates: `sifr_codegen`, `sifr_type_system`, `sifr_format`, `sifr_lint`, `sifr_ir`, and `sifr_source`.
- Warm/cold merge wall time: pre-Wave 1 baseline was not separately captured before the gate-expanding edit; Wave 1 post-change merge run completed in 986.72s with a cold e2e cache, above warm budget and below cold budget.

### Wave 0 Implementation Notes

- Status: merged in PR https://github.com/sifr-lang/sifr/pull/2558.
- Scope: schema v2 compatibility, owner registry, coverage-matrix area, shipped guarantee registry, compiler surface matrix, CLI/workspace inventories, hermetic profile metadata, profile-plan emission, and policy docs.
- Focused validation:
  - `uv run --project verification --locked python -m sifr_verify --self-test`: pass.
  - `uv run --project verification --locked python -m sifr_verify profiles check`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas check`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix`: pass; 13 guarantees, 33 surface rows, 22 temporary rows.
  - `SIFR_COVERAGE_MATRIX_STRICT=1 uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix`: fails as expected on temporary rows.
  - `scripts/run_all_tests.sh --profile merge --emit-plan`: pass; merge plan reports 145 e2e fixtures.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; wall time 559.37s on cold/no-e2e-cache run, with an existing warm-budget advisory; coverage-matrix step elapsed 94ms.
  - `uv run --project verification --locked python -m sifr_verify doctor`: pass required checks; optional sanitizer tool `llvm-symbolizer` reported as skip for broad lanes.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after enforcing locked/offline Cargo policy; wall time 198.34s with warm e2e cache, with the existing warm-budget advisory; coverage-matrix step elapsed 97ms.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-0-review-pass-1.md`: found five blocking issues; all addressed.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-0-review-pass-2.md`: found one remaining blocker in the e2e cargo invocation; addressed with `--locked`.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-0-review-pass-3.md`: no remaining blocking Wave 0 issues.

### Wave 1 Implementation Notes

- Status: merged in PR `https://github.com/sifr-lang/sifr/pull/2559`.
- Scope: profile-owned crate test membership, Cargo metadata package/target/feature classification, merge-gate closure for previously omitted green first-party compiler crates, full-mode `sifr_codegen` red-blocker visibility, and `sifr_ir` seed tests.
- Matrix changes: `first_party_crate_tests` and `cargo_features_targets` promoted from `expected-missing` to `blocking`; coverage matrix now reports 20 temporary rows.
- Create-pr behavior: newly added omitted crates are full-mode only so create-pr remains representative; merge runs all green first-party compiler crate tests.
- Validation:
  - `uv run --project verification --locked python -m sifr_verify --self-test`: pass, including crate membership self-test.
  - `uv run --project verification --locked python -m sifr_verify profiles check`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix`: pass; 13 guarantees, 33 surface rows, 20 temporary rows.
  - `cargo test -p sifr_type_system --locked`: pass, 92 tests.
  - `cargo test -p sifr_format --locked`: pass, 7 tests.
  - `cargo test -p sifr_lint --locked`: pass, 22 tests.
  - `cargo test -p sifr_source --locked`: pass, 3 tests.
  - `cargo test -p sifr_ir --locked`: pass, 3 tests.
  - `cargo check --workspace --all-targets --all-features --locked`: pass.
  - `cargo fmt --check`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `python3 scripts/check_hir_maintainability_guardrails.py`: pass.
  - `scripts/run_all_tests.sh --profile merge --emit-plan`: pass; merge plan includes explicit crate membership.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after narrowing new omitted crates to full-mode only; wall time 176.70s with existing warm-budget advisory.
  - `scripts/run_all_tests.sh`: pass for merge; wall time 986.72s with cold e2e cache, all blocking checks green, `sifr_codegen` logged as planned red-blocker for Wave 2.final.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-1-review-pass-1.md`: no blocking issues; accepted small hardening follow-ups for `merge-red-blocker` policy wording, duplicate full-mode package membership handling, and invalid non-executed full-mode blocking suites.
  - Post-review validation: `scripts/run_all_tests.sh --profile create-pr` passed; wall time 169.32s with existing warm-budget advisory.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-1-review-pass-2.md`: no blocking issues; reviewer explicitly approved Wave 1 for merge and listed only non-blocking follow-ups for later waves.
  - Post-rebase validation: `uv run --project verification --locked python -m sifr_verify areas run --area core_language` passed after warming a transient audit-fixture timeout, then `scripts/run_all_tests.sh --profile create-pr` passed; wall time 200.62s with existing warm-budget advisory.
- Budget evidence:
  - Create-pr remains above its warm budget but improved from the all-smoke trial run by making the Wave 1 additions full-mode only.
  - Merge completed below the cold budget but above the warm budget due primarily to generated-code quality and full e2e cache misses; follow-up batching/cache-budget work remains outside this Wave 1 gate-closure scope.

### Wave 2.0 Implementation Notes

- Status: merged in PR `https://github.com/sifr-lang/sifr/pull/2561`.
- Scope: no compiler code changes; failure inventory only for the current `sifr_codegen` red-blocker.
- Reproduction:
  - `cargo test -p sifr_codegen -- --nocapture`: expected failure; 655 passed, 52 failed, 707 total.
- Validation:
  - `jq empty verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: pass.
  - Inventory parity check against `target/wave2/sifr_codegen_nocapture.log`: pass; 52 failures matched in order with all required fields.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas run --area core_language`: pass after warming a transient audit-fixture timeout.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; wall time 175.52s with existing warm-budget advisory.
- Artifacts:
  - `plans/issues/active/codegen-test-triage.md`: one row per failing test with classification, owner, proposed PR slice, and replacement/regression target.
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: machine-readable inventory with current output, source location, affected compiler contract, owner, and `closes_in_wave`.
- Classification summary:
  - `stale-expectation`: 36.
  - `obsolete-test`: 6.
  - `compiler-bug`: 10.
  - `production-bug`: 0; no unresolved production sentinel rows in Wave 2.0.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-0-review-pass-1.md`: found three blockers; addressed by reclassifying fixable user-visible defects as `compiler-bug`, using `closes_in_wave: 2` plus `closes_in_subwave`, and replacing maintainer-local/source-helper locations with repository-relative test locations.
  - Post-review validation: `scripts/run_all_tests.sh --profile create-pr` produced a passing lane report at `target/validation_lane_reports/create-pr.latest.json`; wall time 149.03s with existing warm-budget advisory. The terminal process was terminated after the passing report was written because the e2e process left pipes open after completion.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-0-review-pass-2.md`: no blocking issues; reviewer explicitly approved Wave 2.0 for merge and left only non-blocking follow-ups for later Wave 2.x work.

### Wave 2.1 Implementation Notes

- Status: merged in PR `https://github.com/sifr-lang/sifr/pull/2562`.
- Scope: close all 20 stale expectation rows assigned to `proposed_pr_slice: 2.1` in the `sifr_codegen` red-blocker inventory.
- Changes: refreshed normalized integer/float literal expectations, final-return tail-expression expectations, the render helper inline snapshot, and source-based iterator materialization assertions to match the current generated-Rust contract.
- Validation:
  - `cargo test -p sifr_codegen -- --nocapture`: expected failure; improved from 655 passed / 52 failed to 675 passed / 32 failed, closing exactly the 20 Wave 2.1 rows while leaving later Wave 2.x rows red.
  - `cargo test -p sifr_codegen render::render_helpers::tests::renders_function_type_param_bounds -- --exact --nocapture`: pass.
  - `cargo fmt --check`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; wall time 508.98s with cold e2e cache and the existing warm-budget advisory.
- Artifacts:
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: `red_blocker.failure_count` updated to 32, `test_result` updated to 675/32/707, and all 20 Wave 2.1 rows marked `closed`.
  - `plans/issues/active/codegen-test-triage.md`: current Wave 2.1 state and remaining open-row count documented.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-1-review-pass-1.md`: no blocking issues; reviewer approved Wave 2.1 for merge and left only low-priority follow-ups for assertion hardening.

### Wave 2.2 Implementation Notes

- Status: merged in PR `https://github.com/sifr-lang/sifr/pull/2563`.
- Scope: close all 16 stale source-fixture rows assigned to `proposed_pr_slice: 2.2` in the `sifr_codegen` red-blocker inventory.
- Changes: converted parser-invalid `\n\` fixtures to raw multi-line source strings, added real `await task.sleep(0.0)` suspension points to async worker fixtures that must remain async, added explicit encoding to the `open()` fixture, and refreshed secondary normalized-literal / constructor assertions exposed after those fixtures started lowering. The newly reachable literal spelling refreshes are the same current-contract forms covered by Wave 2.1 (`99_i64`, `10_i64`, `77_i64`, and tail-expression `2_i64`).
- Validation:
  - `cargo fmt --check`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `cargo test -p sifr_codegen lib_codegen_tests::iterators_and_generators_codegen_tests::test_generate_rust_open_uses_canonical_filehandle_constructor -- --exact --nocapture`: pass.
  - `cargo test -p sifr_codegen -- --nocapture`: expected failure; improved from 675 passed / 32 failed to 691 passed / 16 failed, closing exactly the 16 Wave 2.2 rows while leaving later Wave 2.x rows red.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after review follow-ups; wall time 185.80s with the existing warm-budget advisory and 100% e2e cache hit rate.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-2-review-pass-1.md`: no blocking issues; reviewer approved Wave 2.2 for PR/merge and requested assertion/evidence hardening, which was applied.
- Artifacts:
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: `red_blocker.failure_count` updated to 16, `test_result` updated to 691/16/707, and all 16 Wave 2.2 rows marked `closed`.
  - `plans/issues/active/codegen-test-triage.md`: current Wave 2.2 state and remaining open-row count documented.

### Wave 2.3 Implementation Notes

- Status: merged in PR `https://github.com/sifr-lang/sifr/pull/2564`.
- Scope: close all 6 obsolete architecture guard rows assigned to `proposed_pr_slice: 2.3` in the `sifr_codegen` red-blocker inventory.
- Changes: retargeted stale source-text guards from retired wrapper/helper locations to the current owner modules after decomposition: registry strict/recursive/result helpers, entrypoint body-item assembly, module body/constants emission, generator-init statement output, and structured statement orchestration in `lib_emitter_state.rs`.
- Validation:
  - Six previously failing Wave 2.3 exact tests: pass.
  - `cargo test -p sifr_codegen -- --nocapture`: expected failure; improved from 691 passed / 16 failed to 697 passed / 10 failed, closing exactly the 6 Wave 2.3 rows while leaving Waves 2.4 and 2.5 red.
  - `cargo fmt --check`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas run --area core_language`: pass after the first `create-pr` attempt hit two transient 10s audit-fixture timeouts.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after the focused core-language rerun; wall time 197.18s with the existing warm-budget advisory and 100% e2e cache hit rate.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-3-review-pass-1.md`: no blocking issues; reviewer approved Wave 2.3 for PR/merge and left only non-blocking nits.
- Artifacts:
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: `red_blocker.failure_count` updated to 10, `test_result` updated to 697/10/707, and all 6 Wave 2.3 rows marked `closed`.
  - `plans/issues/active/codegen-test-triage.md`: current Wave 2.3 state and remaining open-row count documented.

### Wave 2.4 Implementation Notes

- Status: merged in PR https://github.com/sifr-lang/sifr/pull/2565.
- Scope: close all 6 structured lowering compiler-bug rows assigned to `proposed_pr_slice: 2.4` in the `sifr_codegen` red-blocker inventory.
- Changes: restored simple lowering for non-`self` field assignments when field/value types already match, kept `self` field assignments and mismatched typed field assignments on the structured path for class/recursive storage adaptations, and made option unwrapping patterns mutation-aware in both simple lowering and the structured emitter path. Updated the related break-guard regression to assert the non-`mut` option pattern when the unwrapped value is not mutated.
- Validation:
  - Six previously failing Wave 2.4 exact tests: pass.
  - `cargo test -p sifr_codegen lib_codegen_tests::performance_codegen_tests::break_guard_unwraps_optional_tuple_before_indexing -- --exact --nocapture`: pass after assertion update for the mutation-aware option pattern.
  - `cargo test -p sifr_codegen -- --nocapture`: expected failure; improved from 697 passed / 10 failed / 707 total to 704 passed / 4 failed / 708 total, closing exactly the 6 Wave 2.4 rows while leaving Wave 2.5 red and adding one field-assignment guard regression.
  - `cargo fmt --check`: pass after formatting.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `uv run --project verification --locked python -m sifr_verify areas run --area core_language`: pass after both the initial and post-review `create-pr` attempts hit the same two transient 10s audit-fixture timeouts.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after the post-review focused core-language rerun; wall time 198.88s with the existing warm-budget advisory and 100% e2e cache hit rate.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-4-review-pass-1.md`: approved with nits and no blocking issues; the non-self field assignment type-adaptation nit was addressed by guarding the simple path on matching resolved field/value types and adding a mismatched type regression.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-4-review-pass-2.md`: approved with no blocking issues; reviewer confirmed Wave 2.4 is ready for PR/merge.
- Artifacts:
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: `red_blocker.failure_count` updated to 4, `test_result` updated to 704/4/708, and all 6 Wave 2.4 rows marked `closed`.
  - `plans/issues/active/codegen-test-triage.md`: current Wave 2.4 state and remaining open-row count documented.

### Wave 2.5 Implementation Notes

- Status: merged in PR https://github.com/sifr-lang/sifr/pull/2566.
- Scope: close the final 4 Wave 2 codegen red-blocker rows assigned to `proposed_pr_slice: 2.5`.
- Changes: reinspection showed the generated Rust already preserved the intended async cleanup, generator else-branch, self-field clone suppression, and string-index guard behavior. Refreshed the four stale assertions to current generated-Rust spelling and tail-expression/char-cache output contracts; the string-index assertion now validates the current `Vec<char>` cache plus let-else guard shape rather than the older direct `chars().nth(...)` spelling.
- Validation:
  - Four previously failing Wave 2.5 exact tests: pass.
  - `cargo test -p sifr_codegen -- --nocapture`: pass; 708 passed / 0 failed / 708 total.
  - `cargo fmt --check`: pass after formatting.
  - `python3 scripts/check_file_size_guardrails.py`: pass.
  - `scripts/run_all_tests.sh --profile create-pr`: pass after review-nit follow-up; wall time 168.01s with the existing warm-budget advisory and 100% e2e cache hit rate.
- Artifacts:
  - `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`: `red_blocker.failure_count` updated to 0, `red_blocker.status` updated to `closed`, `test_result` updated to 708/0/708, and the final 4 Wave 2.5 rows marked `closed`.
  - `plans/issues/active/codegen-test-triage.md`: current Wave 2.5 green state and final row reclassification documented.
- Review:
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-5-review-pass-1.md`: approved with nits and no blocking issues; the string-index assertion/doc honesty nits were addressed.
  - `plans/reviews/active/ad-hoc-world-class-verification-wave-2-5-review-pass-2.md`: approved with no blocking issues; reviewer confirmed Wave 2.5 is ready to merge and Wave 2.final promotion is unblocked.

### Wave 2.final Implementation Notes

- Status: merged in PR https://github.com/sifr-lang/sifr/pull/2567.
- Scope: promote `sifr_codegen` from planned red-blocker membership to executed merge membership after the suite reached 708 passed / 0 failed / 708 total in Wave 2.5.
- Changes: profile membership for `create-pr`, `merge`, `nightly`, and `release` now marks `sifr_codegen` as `blocking` with `executed_in_merge: true`; the verification runner self-test now requires codegen to be blocking/executed across all four profiles; cargo metadata target classification now assigns `sifr_codegen` to `merge`; the coverage matrix no longer carries a `red-blocker` row for the generated-Rust contract; baseline governance now records `sifr_codegen` snapshot bless requirements.
- Post-promotion repair: the first full merge run after profile promotion executed `sifr_codegen` but then exposed a real e2e regression in `recursive_mutual_classes_runtime.sifr`; structured option let-else bindings for recursive class values now emit `Some(mut value)` when later child moves require `.take()`, covered by a new codegen regression. The same validation pass also exposed a parallel IPC fixture startup race in `sifr_stdlib`; worker startup is now serialized only through the cargo-run/bootstrap handoff.
- Baseline comparison: Wave 1 full merge baseline was 986.72s with cold e2e cache while `sifr_codegen` was logged as a planned Wave 2.final red-blocker. The first post-promotion merge run reached 848.99s and failed e2e because it caught the recursive option mutability bug. The final post-review full merge run passed in 726.99s with `sifr_codegen` executed as blocking, 51/51 e2e cache hits, and only the existing group-skew advisory.
- Validation: `cargo test -p sifr_codegen` passed with 709 passed / 0 failed / 709 total; `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/recursive_mutual_classes_runtime.sifr` passed; `scripts/run_all_tests.sh --profile merge --emit-plan` passed and showed `sifr_codegen` as blocking/executed; `scripts/run_all_tests.sh --profile merge` passed in 726.99s; `scripts/run_all_tests.sh --profile create-pr` passed in 189.46s with a non-blocking warm wall-time advisory and 44/44 e2e cache hits; `cargo test -p sifr_stdlib --test ipc_process_pipe_fixture -- --nocapture` passed; `cargo clippy --workspace -- -D warnings` passed; `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- Review: `plans/reviews/active/ad-hoc-world-class-verification-wave-2-final-review-pass-1.md` found one blocker: stale cargo metadata still labeled `sifr_codegen` as `merge-red-blocker`. The blocker was fixed by assigning the target to `merge`; non-blocking follow-ups accepted in this PR broadened the selftest across all profiles, renamed the generated-Rust matrix row and shipped-guarantee reference to `codegen_merge_blocking`, added an inventory post-Wave-2.final timestamp, and made the IPC startup lock poison-tolerant.

## Full Discovery Snapshot

Discovery commands used for this phase:

```bash
find verification/areas -maxdepth 2 -name manifest.json -print | sort
find verification/profiles verification/policy verification/schemas -maxdepth 2 -type f | sort
cargo metadata --no-deps --format-version 1
```

Verification areas currently present:

| area | suites observed |
| --- | --- |
| `algorithmic_compatibility` | `taxonomy-smoke`, `leetcode-check` |
| `core_language` | `integer_dtype_contract`, `phase24_hir_analysis`, `phase25_cfg_flow`, `audit-fixtures` |
| `developer_tooling` | `typescript-go-m1`, `diagnostic-contracts`, `static`, `formatter`, `analysis`, `lsp-smoke`, `editor-release`, `lsp-stress`, `phase-closeout`, `full` |
| `diagnostics` | `contracts`, `baselines` |
| `distribution_release` | `representative`, `full` |
| `ecosystem_compatibility` | `oss-curated`, `ecosystem-broader` |
| `fuzz_property` | `cargo-smoke`, `property`, `fuzz-smoke` |
| `generated_code_quality` | `smoke`, `representative`, `full`, `corpus`, `panic-scan`, `intrinsic-panic-lint`, `rustfmt`, `clippy`, `determinism`, `demos` |
| `package_management` | `guardrails` |
| `performance` | `frontend-syntax-guardrails`, `contracts`, `smoke`, `representative`, `full` |
| `project_workspace` | `frontend_mode_parity`, `phase23_graph_isolation`, `baselines`, `audit-fixtures` |
| `regression` | `fixedbugs`, `crashes` |
| `runtime_platform` | `platform-golden`, `platform-contract` |
| `stdlib_parity` | `complexity-resource`, `namespace-demos-check`, `namespace-leetcode-check`, `audit-fixtures` |

Profile state currently observed:

| profile | warm/cold budget | selected area coverage | hardening | current gap |
| --- | --- | --- | --- | --- |
| `create-pr` | 2/5 min | core, diagnostics, runtime, tooling, generated-code quality, performance | none | fast signal only; no coverage matrix or crate membership accounting |
| `merge` | 15/25 min | core, diagnostics, runtime, tooling, generated-code quality, performance, distribution, project, regression, ecosystem | diagnostics, project, fixedbugs, crashes, oss-curated | authoritative but omits known crate suites and full semantic corpus |
| `nightly` | 30/45 min | merge-like plus fuzz/property | diagnostics, project, fixedbugs, crashes, property, fuzz-smoke, oss-curated, ecosystem-broader, determinism-scale | broader signal but not enough sustained fuzz/sanitizer/platform evidence |
| `release` | 45/60 min | nightly-like | nightly hardening plus determinism extras | strongest current profile but still lacks guarantee registry and platform/support evidence |

Cargo workspace packages and notable targets/features currently observed:

| package | notable targets/features | phase decision |
| --- | --- | --- |
| `sifr` | bin plus integration tests `e2e`, `validation_contracts`, `build_output_contracts` | keep existing CLI tests; profile membership becomes explicit |
| `sifr_analysis` | lib | already in current hard-coded crate path; keep explicit |
| `sifr_codegen` | lib; 709 passed / 0 failed after Wave 2.final regression coverage | merge-blocking as of Wave 2.final |
| `sifr_diagnostics` | lib plus bins `gen-diagnostic-schema`, `gen-error-docs` | bins are internal release-engineering tools; Wave 1 assigns internal tool smoke or explicit internal-no-run classification |
| `sifr_driver` | lib plus bin `diagnostic_contract_harness` | bin is internal diagnostics harness; Wave 1 assigns internal tool smoke or explicit internal-no-run classification |
| `sifr_format` | lib | merge crate test membership required |
| `sifr_frontend` | lib plus bin `frontend_query_bench` | benchmark bin classified under performance |
| `sifr_ir` | lib with zero tests | seed tests required in Wave 1 |
| `sifr_lint` | lib | merge crate test membership required |
| `sifr_lowering` | lib | already in current hard-coded crate path; keep explicit |
| `sifr_lsp` | lib | keep crate tests and add marker/transcript corpus |
| `sifr_package` | lib | package integration expands beyond guardrails |
| `sifr_runtime` | lib with features `default`, `http`, `i18n`, `net`, `tls`, `unicode` | feature matrix required; default+http currently represented, remaining features need profile assignment |
| `sifr_source` | lib | merge crate test membership required |
| `sifr_stdlib` | lib, fixture bin, snapshot tests, feature `__test_fixture` | fixture bin and `__test_fixture` feature are test-only internal surfaces; Wave 1 assigns them to merge/nightly fixture validation or explicit internal-no-run classification |
| `sifr_syntax` | lib | parser/syntax acceptance rows required |
| `sifr_type_system` | lib | merge crate test membership required |

Schema constraints currently observed:

- `verification/schemas/profile.schema.json` is strict and schema version `1` only. It does not allow explicit crate membership, network policy, reference host, feature policy, or emitted profile plans. This phase therefore chooses profile schema version `2`.
- `verification/schemas/area.schema.json` is strict and schema version `1` only. It does not encode baseline metadata, network mode, pinned corpus checksums, or skip policies. This phase therefore chooses area schema version `2`.
- `verification/schemas/suite.schema.json` is minimal and does not encode owners, normalizers, artifacts, or stale-baseline metadata. This phase keeps suite-specific metadata in area-owned data files validated by area checks.

## Implementation Sequence

This phase is intentionally split into small PR-sized waves and sub-waves. Each numbered wave or sub-wave must be implemented, validated locally, opened as a PR, reviewed, merged, and documented before dependent work starts.

Focused Cargo validation commands are run through the profile wrapper when possible. If a command must be run directly, it must use the same hermetic settings as the profile: `--locked` and `CARGO_NET_OFFLINE=true` after documented setup fetch/vendor steps. Focused validation snippets below show the semantic command being validated; the wrapper or invoking shell is responsible for injecting the hermetic profile policy.

### Wave 0: Baseline Audit and Executable Coverage Matrix

Goal: make the current verification reality measurable and prevent future gaps from hiding in profile runner code.

Wave 0 may ship as sub-PRs if needed: 0.1 schema v2 support, 0.2 coverage matrix area plus shipped guarantee registry, and 0.3 hermetic/offline policy plus plan-equivalence skeleton and policy docs.

Tasks:

- Add schema version `2` support for profiles and area manifests before adding new profile/area fields. Migration rule: v1 profiles/areas remain readable for self-tests only; shipped stable surfaces must be v2 by Wave 10.
- Add `verification/areas/coverage_matrix/manifest.json` and a runner-owned check as the first-class verification area for guarantee and surface coverage.
- Add `verification/areas/coverage_matrix/shipped_guarantees.json`, the authoritative registry of shipped guarantees. Each row records `guarantee_id`, support status, public doc path, owner, merge surface, nightly/release surface, regression surface, and unsupported-behavior policy.
- Add `verification/areas/developer_tooling/data/cli_exit_code_contracts.json` and populate it from current CLI integration-test expectations and documented CLI command semantics.
- Add `verification/areas/project_workspace/data/workspace_contracts.json` and populate it from current `project_workspace` suite manifests and shipped workspace/graph-isolation guarantees.
- Matrix checks validate shipped-guarantee schema: status is one of `stable`, `experimental`, `internal`, or `unsupported`; surfaces resolve to matrix-row ids; public docs resolve or are explicitly marked internal; and every stable guarantee has at least one matrix row.
- Define `verification/areas/coverage_matrix/compiler_surface_matrix.json` with rows for every compiler surface listed in the world-class standard.
- For each row, record:
  - surface id
  - shipped guarantee
  - merge suite
  - nightly or release suite
  - regression suite
  - owner
  - reproduction command
  - current status: `blocking`, `broad-only`, `expected-missing`, `tests:none`, `not-applicable`, `red-blocker`, or `quarantined`
  - linked wave or issue for any non-blocking status
  - `closes_in_wave`, required for each `expected-missing` or `red-blocker` row and naming exactly one wave in 1-9
  - optional `closes_in_subwave`, required when closure is assigned to a named subwave such as `final`
  - expiry date for any temporary non-blocking status
- Include existing areas in the matrix rather than duplicating them:
  - `algorithmic_compatibility` is the algorithm/LeetCode compatibility surface.
  - `fuzz_property` is the existing property and fuzz-smoke surface.
  - parser acceptance/rejection and parser fuzzing are separate rows.
- Add `verification/owners.json` as the authoritative owner registry. `CODEOWNERS` may be used as an input only if it can express the same owner ids used in verification metadata.
- Add owner validation: no shipped guarantee, matrix row, quarantine row, `tests:none` row, `red-blocker` row, or wave task may have owner `unassigned` or any owner id absent from the owner registry.
- Add hermetic/offline policy fields to profile or suite metadata: `network: offline|live`, pinned corpus revision/checksum, reference host, max parallelism, warm/cold budget, and allowed host skips.
- Add the Cargo hermetic contract for create-pr and merge:
  - Cargo commands run with `--locked`
  - profile execution uses `CARGO_NET_OFFLINE=true` or equivalent
  - dependencies are satisfied by the checked-in lockfile plus documented setup cache or vendor procedure
  - tests do not rely on undocumented user-global `~/.cargo` state
  - `cargo fetch --locked` or a vendored equivalent is setup, not profile execution
  - rustup/toolchain downloads are contributor setup, not profile execution
- Add the execution sandbox contract for generated and external programs:
  - per-test timeout
  - memory limit where host-supported
  - tempdir-only writable filesystem
  - no writes outside declared output dirs
  - no external network in create-pr/merge
  - loopback-only networking when declared
  - subprocess cleanup verification
  - captured stdout/stderr size limits
- Add a local/CI plan-equivalence skeleton: the runner can emit a profile execution plan, and CI/local plans can be compared for same suites, commands, fixture counts, and allowed host skips. Local profile execution remains authoritative for PR and merge readiness.
- Add `scripts/verification_doctor.sh` or `uv run --project verification --locked python -m sifr_verify doctor` to check local prerequisites: Rust toolchain, Python version, uv lock state, Cargo offline setup, required sanitizer tools where applicable, and supported host metadata.
- Add a zero-test and zero-fixture fail rule unless an explicit `tests:none` row exists.
- Add an advisory-mode check that fails schema errors and unknown statuses, but initially permits the closed Wave 0 `expected-missing` list.
- Add a promotion switch so the same check becomes blocking in Wave 10 and fails any remaining `expected-missing`, `red-blocker`, expired `tests:none`, illegal `not-applicable`, or undocumented `quarantined` row.
- Add coverage-matrix execution to `create-pr` as a schema/consistency check and to `merge` as an advisory check until Wave 10 promotion.
- Update `verification/policy/profile_policy.md` and `verification/policy/suite_taxonomy.md` with the coverage-matrix rule.

Exit criteria:

- A reviewer can answer "what blocks merges for this compiler guarantee?" without reading Python runner code.
- Every stable shipped guarantee has an owner and coverage row.
- Hermetic/offline requirements are encoded for create-pr and merge profiles.
- Missing or broad-only surfaces are explicit, finite, and mapped to later waves.
- No new `expected-missing` row may be added after Wave 0 for a pre-existing Wave 0 surface. New stable shipped guarantees added after Wave 0 must either add a blocking row in the same PR or carry a phase-owner-approved, time-boxed `expected-missing` row with owner, issue, reproduction command, expiry, and `closes_in_wave`.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas check
uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix
scripts/run_all_tests.sh --profile create-pr
```

### Wave 1: Merge-Gate Crate Closure

Goal: no first-party compiler crate test suite is silently omitted from the authoritative merge gate.

Tasks:

- Replace the hard-coded crate test membership in `ProfileRunner.run_crate_tests` with profile-owned data, or extend the profile schema with an explicit crate test list while preserving the current facade behavior during migration.
- Add a checked-in package/target classification file validated against `cargo metadata`. `cargo metadata` is the source of truth for package, target, feature, bin, example, and doctest discovery; the checked-in classifier is the source of truth for Sifr-specific classification.
- Classification values are:
  - `first_party_compiler`
  - `first_party_runtime`
  - `first_party_tooling`
  - `test_fixture`
  - `benchmark`
  - `internal_codegen_tool`
  - `third_party`
  - `generated`
  - `external_tooling`
- The cargo-metadata guardrail fails unclassified packages, bins, features, examples, and doctests.
- Add the omitted first-party crate suites to the profile-owned merge crate list:
  - `cargo test -p sifr_codegen`, tracked as planned merge membership with status `red-blocker`, `executed_in_merge: false`, and `must_be_executed_by: Wave 2.final`
  - `cargo test -p sifr_type_system`
  - `cargo test -p sifr_format`
  - `cargo test -p sifr_lint`
  - `cargo test -p sifr_source`
  - `cargo test -p sifr_ir`, after seed tests are added
- Keep existing crate tests in merge.
- Add profile self-tests proving that a crate listed in profile data is executed and an unknown crate/suite name fails validation.
- Add profile-plan emission, for example `scripts/run_all_tests.sh --profile merge --emit-plan`, so local and CI profile plans can be compared without depending on CI for correctness.
- Profile plans must emit planned-but-not-executed `red-blocker` entries so reviewers can see known-red suites before they become blocking.
- Add a guardrail that detects:
  - workspace first-party crates with tests but no profile membership
  - workspace first-party crates with zero tests and no explicit `tests:none` coverage-matrix row
  - stale `tests:none` rows whose expiry has passed
- Add a `cargo metadata` guardrail for every first-party compiler crate:
  - default feature tests are represented
  - stable feature flags have a merge/nightly/release assignment
  - shipped bins are smoke-tested
  - examples are checked or explicitly excluded
  - doctests are run or explicitly disabled with reason
  - `all-targets`, `all-features`, and `no-default-features` policy is documented
- Exclude only documented parser submodules, generated crates, third-party code, and intentionally external tooling.
- Update `verification/README.md` and `verification/policy/profile_policy.md` with the crate-test membership rule.
- Measure warm/cold merge wall time before and after adding crate suites. If the merge lane exceeds the documented budget, ship sharding or profile execution parallelism in the same PR.

Exit criteria:

- `merge.json` or an adjacent checked-in profile data file is the source of truth for first-party crate tests.
- Omitted crates with tests are represented in the merge gate.
- `sifr_ir` has seed tests.
- New first-party compiler crates cannot be added without an explicit verification decision.
- Cargo features, bins, examples, doctests, and targets have profile assignments or explicit unsupported entries.

Focused validation:

```bash
cargo test -p sifr_type_system
cargo test -p sifr_format
cargo test -p sifr_lint
cargo test -p sifr_source
cargo test -p sifr_ir
cargo check --workspace --all-targets --all-features
uv run --project verification --locked python -m sifr_verify --self-test
uv run --project verification --locked python -m sifr_verify profiles check
scripts/run_all_tests.sh --profile create-pr
```

### Wave 2: Codegen Test Triage and Enforcement

Goal: make `sifr_codegen` green, meaningful, and merge-blocking.

#### Wave 2.0: Codegen Failure Inventory

Tasks:

- Run `cargo test -p sifr_codegen -- --nocapture` and capture the failure inventory in this issue's execution checklist.
- Add `plans/issues/active/codegen-test-triage.md` with one row per failure:
  - test id
  - failure summary
  - classification: `stale-expectation`, `obsolete-test`, `compiler-bug`, or `production-bug`
  - proposed PR slice
  - owner
  - replacement or regression target
- Add a machine-readable inventory beside it, for example `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`, with current output, expected output or snapshot id, affected compiler contract, owner, and `closes_in_wave`.
- Do not change code in Wave 2.0. The PR closes only after review agrees with the classification.

Exit criteria:

- Every failing `sifr_codegen` test has an explicit classification and next PR.
- The inventory explains which failures are stale tests and which may represent real compiler defects.

#### Wave 2.1..2.N: Per-Classification Repair PRs

Tasks:

- For stale expectations, update assertions or snapshots to match the current intended generated Rust contract.
- For obsolete tests, delete only when a replacement test or broader suite covers the same behavior; record the replacement in the triage file.
- For real compiler bugs, ship one root-cause fix PR per coherent bug group and add regression coverage.
- For unresolved production bugs that cannot be fixed in this phase, add explicit issue-linked sentinels in `verification/areas/regression/crashes` or the relevant codegen suite. Do not leave them as untracked ignored unit tests.
- For every repaired stable generated-Rust fixture class, prove the emitted Rust can survive the Rust toolchain:
  - emit Rust
  - normalize output
  - run `rustfmt --check`
  - run `cargo check` or equivalent
  - run clippy for merge-safe subsets and nightly full subsets
  - run generated binaries for runtime fixtures
  - scan generated Rust for `panic!`, `.unwrap()`, `.expect()`, `todo!`, and `unimplemented!`
- Define the merge-safe clippy subset with an explicit allowlist at `verification/areas/generated_code_quality/data/clippy_merge_lints.json`. Lints outside that allowlist are nightly-only until promoted through owner review.
- Record generated Rust toolchain support:
  - supported rustc channel/version
  - generated-output MSRV if Sifr promises one
  - whether the fixture is stable, experimental, or internal
  - nightly-only generated Rust is illegal unless the generating surface is marked experimental/internal
- Any generated-Rust scan allowlist entry must have owner, reason, issue link, expiry, and reproduction command.
- Re-run the codegen suite after each PR and update the triage file's status.

Exit criteria:

- The triage file has no open stale-expectation or obsolete-test rows.
- All real compiler bugs are fixed or represented by issue-linked sentinels.
- `cargo test -p sifr_codegen` passes locally.

#### Wave 2.final: Promote Codegen To Merge

Tasks:

- Add `cargo test -p sifr_codegen` to the merge crate test list only after it is green.
- Change the codegen entry from planned merge membership to executed merge membership: status `blocking`, `executed_in_merge: true`.
- Add codegen snapshot bless rules to `verification/policy/baseline_governance.md` if the suite uses snapshots.
- Measure warm/cold merge wall time before and after enabling codegen. If the merge lane exceeds the documented budget, ship sharding or profile execution parallelism in the same PR.

Exit criteria:

- `cargo test -p sifr_codegen` passes locally.
- All codegen test changes are explained by intended compiler contracts.
- `scripts/run_all_tests.sh --profile merge` would fail if `sifr_codegen` regresses.

Focused validation:

```bash
cargo test -p sifr_codegen
cargo test -p sifr -- --skip test_e2e_pass
verification/runner/e2e/run_e2e_pass.sh
scripts/run_all_tests.sh --profile create-pr
```

### Wave 3: Full Semantic E2E and Parser Corpus Coverage

Goal: make merge authoritative for language semantics and stable syntax acceptance/rejection.

Tasks:

- Measure current full e2e pass runtime with cache warm and cold.
- If full pass runtime fits the merge budget, remove the merge-only pass subset and run the full pass corpus in merge.
- If full pass runtime does not fit, implement deterministic sharding in the merge profile and run all shards as part of the authoritative local merge command.
- Run the full fail corpus code/position checks in merge; rendered diagnostic presentation remains Wave 4.
- Add parser acceptance/rejection matrix coverage:
  - every stable syntax construct has at least one positive fixture
  - every stable syntax error family has at least one negative fixture
  - parser-fork behavior and Sifr-owned syntax behavior are separated
  - contradiction checks prevent a fixture from being both pass and expected-error
- Add lexer/token stream and indentation coverage:
  - token boundaries
  - indentation/dedentation
  - newline handling
  - comments/trivia if preserved
  - Unicode identifiers if supported
  - source-span stability across tokenization
- If `verification/areas/core_language/data/merge_e2e_manifest.json` remains after full-corpus promotion, freeze it as the create-pr subset or rename it so its purpose is unambiguous.
- Keep create-pr as the fast representative subset.
- Preserve deterministic ordering, report generation, and sequential-vs-parallel equivalence behavior.
- Add a profile check that prevents merge from referencing stale fixture manifests when full corpus mode is selected.
- Record how `merge`, `nightly`, and `release` differ after promotion. The expected end state is that merge and nightly both execute the full semantic pass corpus, while nightly can add broader hardening around it.

Exit criteria:

- Merge runs all pass fixtures, either in one lane or through deterministic local shards.
- Merge runs the full fail corpus code/position checks.
- Parser acceptance/rejection has matrix-backed positive and negative coverage independent of parser fuzzing.
- Lexer/token stream and indentation have matrix-backed coverage independent of parser acceptance/rejection.
- Subsetting remains a create-pr speed optimization only.
- Reports make the executed fixture count visible.

Focused validation:

```bash
verification/runner/e2e/run_e2e_pass.sh --profile merge
bash verification/runner/e2e/check_report_determinism.sh --profile merge
bash verification/runner/e2e/check_sequential_parallel_equivalence.sh --profile merge
scripts/run_all_tests.sh --profile create-pr
```

Operational note: run the two merge-profile e2e report checks serially. They both drive the full merge corpus and share target-side filesystem state; concurrent invocation can produce unrelated I/O/logging fixture interference.

### Wave 3 Implementation Notes

- Status: merged in PR https://github.com/sifr-lang/sifr/pull/2568.
- Scope: promote merge semantic e2e from the 145-fixture merge subset to the full 651-fixture pass corpus, keep create-pr on its 132-fixture representative subset, add matrix-backed parser/lexer coverage, and close the `parser_acceptance_rejection` coverage-matrix row.
- Profile changes: `merge`, `nightly`, and `release` now select full e2e pass corpus mode through an empty fixture manifest; create-pr remains the only profile using `create_pr_e2e_manifest.json`. The stale `merge_e2e_manifest.json` subset was removed. Profile self-tests now reject merge/nightly/release e2e manifests and require the full fail corpus to remain covered by `sifr_cli_full`.
- E2E runner changes: direct `verification/runner/e2e/run_e2e_pass.sh --profile ...` defaults now match profile JSON grouping: create-pr max group 8, merge max group 12, nightly/release max group 16. Direct merge runs now print and enforce `max_group_fixtures=12`.
- Parser/lexer matrix: added `syntax_parser_lexer_matrix` under `core_language`, with positive Sifr-owned and parser-fork cases, negative stable parse-error families, token boundary/span checks, indentation/dedentation, comment trivia, non-logical newline handling, Unicode identifier span stability, and positive/negative contradiction checks. The merge/nightly/release profiles include this suite; create-pr remains representative.
- Repair found by full corpus: the first cold full-corpus run exposed immutable context-manager target bindings for `with open(..., "w") as out: out.write(...)`; `RustWithItem` now carries mutability into rendering, and `sifr_codegen` has a regression requiring `let mut out = __guard_0.ctx.__enter__();`. The same run exposed a stale negative bytes fixture that expected `latin-1` to be unsupported; it now uses `definitely-not-a-codec` because Latin-1 is supported.
- Focused validation: cold full-corpus merge e2e passed 651/651 in 111.80s with 0/182 cache hits, groups=182, largest group=12, signature `ee5e5d44306f270c`; second warm merge e2e passed 651/651 in 13.18s with 182/182 cache hits; deterministic report check passed with signature `ee5e5d44306f270c`; sequential-vs-parallel equivalence passed with signature `ee5e5d44306f270c`; `cargo test --locked -p sifr --test e2e test_e2e_fail -- --nocapture` passed with 481 fail fixtures; `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e fixtures and signature `5edef8cd4b961ef8`.
- Full validation: `scripts/run_all_tests.sh` passed for merge in 719.80s with budget_ok=yes, full e2e 651/651, 182/182 cache hits, largest group 12, hardening variants=41, failures=0. Non-blocking advisory: group skew is high.
- Static validation: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `python3 scripts/check_hir_maintainability_guardrails.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` all passed.
- Review: `plans/reviews/active/ad-hoc-world-class-verification-wave-3-review-pass-1.md` and `plans/reviews/active/ad-hoc-world-class-verification-wave-3-review-pass-2.md` found no blockers and approved Wave 3. The optional symmetric parser-matrix contradiction check was applied before final validation.
- Matrix status: coverage matrix now reports `parser_acceptance_rejection` as blocking through `core_language:syntax_parser_lexer_matrix`; parser fuzzing remains a separate later-wave `expected-missing` hardening row.

### Wave 4: Diagnostic Baseline and Recovery Expansion

Goal: lock user-visible diagnostics at compiler scale, not just diagnostic codes.

Tasks:

- Inventory all active `SIFR-*` diagnostic codes with stable user-facing messages.
- Add `verification/areas/diagnostics/data/code_catalog.json` with code, severity, stability, owner, docs link, renderer support, suggestion applicability, and `machine_applicable`.
- Add `verification/areas/diagnostics/data/code_baseline_coverage.json` mapping each active diagnostic code to:
  - baseline fixture id
  - renderer formats covered
  - multi-error recovery fixture if applicable
  - suggestion rendering fixture if applicable
  - documented deferral if the diagnostic is unstable
- Add `verification/areas/diagnostics/checks/code_baseline_coverage.py`, or the equivalent diagnostics area check, to enforce code baseline coverage and recovery coverage.
- Add unused/stale rendered-baseline detection:
  - baseline file with no owning fixture fails
  - fixture missing a required baseline fails
  - blessed baseline without metadata fails
  - nondeterministic baseline output fails
  - mass baseline update without per-suite summary fails
- Baseline metadata records fixture id, suite, renderer, owner, bless PR or local issue reference, bless reason, source hash, and normalizers.
- Define renderer coverage by profile:
  - merge requires at least one rendered baseline per active stable `SIFR-*` code
  - nightly and release require every stable renderer for every active stable `SIFR-*` code
- For every active `SIFR-*` code with a stable user-facing message, add at least one rendered baseline fixture covering:
  - human format
  - compact format
  - JSON format
  - stable span and column behavior
  - message text
  - related notes when applicable
  - suggestions when applicable
- For every parser, HIR, name-resolution, and type-checker recovery surface listed in `verification/policy/suite_taxonomy.md`, add at least one multi-error recovery fixture exercising that surface. Extend that policy document first if no recovery-surface list exists.
- Make the diagnostics baseline coverage check fail when a documented recovery surface has zero multi-error fixtures.
- Add contradiction checks where a fixture cannot both pass and contain expected errors.
- Add suggestion-application validation only for suggestions marked machine-applicable in the diagnostic code catalog. If automated application is not stable, record that boundary explicitly.
- Make the diagnostics baseline coverage check fail when a new diagnostic code has no rendered baseline or no documented deferral.
- Measure warm/cold merge wall time before and after expanding diagnostic baselines. If the merge lane exceeds the documented budget, keep broad renderer permutations in nightly while preserving one merge-blocking baseline per active stable code.

Exit criteria:

- Merge has at least one rendered baseline per active `SIFR-*` code with a stable user-facing message.
- Nightly and release have every stable renderer for every active `SIFR-*` code with a stable user-facing message.
- Multi-error recovery quality is represented by baselines.
- Diagnostic presentation regressions fail before release.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines
cargo test -p sifr_diagnostics
scripts/run_all_tests.sh --profile create-pr
```

### Wave 4 Implementation Notes

- Status: first diagnostics-baseline slice implemented, reviewed twice with no blockers, merge-gate validated, and merged in PR [#2570](https://github.com/sifr-lang/sifr/pull/2570).
- Scope: added `code_catalog.json`, `code_baseline_coverage.json`, `baseline_metadata.json`, and `recovery_surface_coverage.json` as the diagnostics metadata layer above rendered baselines. The new `code_baseline_coverage` contract checks active registry parity, catalog severity/docs/owner metadata, rendered-baseline ownership, stale/missing baseline files, source-hash metadata, and recovery-surface coverage.
- Baseline expansion: wired the existing parser diagnostics fixtures into the executable `baselines` suite, added missing `parser_invalid_layout`, `parser_invalid_target`, and `parser_multi_error_recovery` fixtures, and blessed human/json/compact baselines for the parser family. The synthetic `presentation_contract_cases` renderer baseline remains metadata-owned and is not re-blessed through the CLI baseline runner.
- Coverage status: active stable diagnostic codes now require either rendered baseline coverage or an explicit Wave 4 deferral in `code_baseline_coverage.json`. This slice closes rendered baseline coverage for all active `SIFR-PARSE-*` codes plus the existing decimal and multiline assignment fixtures; remaining active codes carry owner/issue/expiry deferrals for later Wave 4 baseline expansion.
- Recovery coverage: `verification/policy/suite_taxonomy.md` now lists parser, mixed HIR/type, and repeated type recovery surfaces; `recovery_surface_coverage.json` maps each to a multi-error fixture, and the coverage contract verifies expected diagnostic-code evidence for those fixtures.
- Review: Claude Opus 4.7 review pass 1 and pass 2 both approved with no blockers; review artifacts are recorded in `plans/reviews/active/`.
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 12 cases and 36 renderer variants; `cargo test -p sifr_diagnostics` passed; `python3 -m py_compile verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e fixtures and signature `5edef8cd4b961ef8`; `scripts/run_all_tests.sh` passed with `wall_time=818.51s`, `budget_ok=yes`, e2e `651/651`, e2e signature `ee5e5d44306f270c`, and hardening `variants=71 failures=0 blocking_failures=0`.

Second Wave 4 diagnostics-baseline slice:

- Status: HIR recovery baseline expansion implemented, locally validated, reviewed twice with no blockers, and merged in PR [#2571](https://github.com/sifr-lang/sifr/pull/2571).
- Scope: added executable `hir_mixed_semantic_recovery` and `hir_repeated_type_recovery` diagnostics fixtures with human/json/compact baselines. The mixed fixture covers independent semantic diagnostics (`SIFR-CALL-0004`, `SIFR-NAME-0001`, `SIFR-OWN-0002`, `SIFR-TYPE-0002`); the repeated fixture covers capped repeated type diagnostics plus the `SIFR-INTERNAL-0002` recovery summary note.
- Coverage status: rendered diagnostic coverage is now 15 active codes, with 155 active codes still carrying Wave 4 deferrals.
- Recovery coverage: `hir_mixed_recovery` and `repeated_type_recovery` now point at diagnostics-area rendered fixtures instead of legacy e2e expectation comments.
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 14 cases and 42 renderer variants; `cargo test -p sifr_diagnostics` passed; `cargo fmt --check` passed; `cargo clippy --workspace -- -D warnings` passed; `python3 scripts/check_hir_maintainability_guardrails.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, and hardening `variants=5 failures=0 blocking_failures=0`; `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, and hardening `variants=77 failures=0 blocking_failures=0`. Existing non-blocking advisories: create-pr warm wall-time budget exceeded at 243.90s due two rebuilt e2e groups; merge warm wall-time budget exceeded at 1422.71s because the e2e cache was effectively cold (`cache_hits=1/182`, `rebuilt_groups=181`).
- Review: Claude Opus 4.7 review pass 1 approved with one non-blocking `json-sort` metadata convention cleanup; review pass 2 approved with no blockers or non-blockers after the cleanup.

Third Wave 4 diagnostics-baseline slice:

- Status: core semantic compact baseline expansion implemented, locally validated, reviewed twice with no blockers, and merged in PR [#2572](https://github.com/sifr-lang/sifr/pull/2572).
- Scope: promoted 81 existing e2e fail fixtures into diagnostics-area compact baselines, covering 82 deferred active codes across async, call, class, decimal, flow, import, int, match, name, ownership, protocol, result, and type diagnostics. Each new fixture has manifest ownership, compact baseline trios, source-hash metadata, and `code_baseline_coverage.json` mapping back to the active diagnostic code.
- Coverage status: rendered diagnostic coverage is now 97 active codes, with 73 active codes still carrying Wave 4 deferrals. The remaining deferred non-package semantic codes without current e2e fixture evidence are `SIFR-FLOW-0901`, `SIFR-IMPORT-0004`, `SIFR-IMPORT-0005`, `SIFR-IMPORT-0006`, `SIFR-IMPORT-0007`, `SIFR-INT-0011`, `SIFR-INTERNAL-0001`, `SIFR-RESULT-0006`, `SIFR-TYPE-0901`, and `SIFR-TYPE-0902`; package/build/workspace/lint/fmt/io/stdlib/encoding families remain deferred to their owning Wave 4 slices. `e2e_bare_defaultdict_constructor_rejected` incidentally emits `SIFR-STDLIB-0001`, but that coverage row intentionally remains deferred to the stdlib-owned Wave 4 slice so stdlib diagnostics get purpose-built baseline ownership rather than incidental semantic-fixture ownership.
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 95 cases and 123 renderer variants; `cargo test -p sifr_diagnostics` passed; `cargo fmt --check` passed; `cargo clippy --workspace -- -D warnings` passed; `python3 scripts/check_hir_maintainability_guardrails.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `python3 -m py_compile verification/areas/diagnostics/checks/code_baseline_coverage.py` passed. Final post-review `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, and hardening `variants=5 failures=0 blocking_failures=0`; existing non-blocking advisories: warm wall-time budget exceeded at `184.30s` despite fully warm e2e cache (`cache_hits=44/44`, `rebuilt_groups=0`). Final post-review `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=123 failures=0 blocking_failures=0`, and hardening `variants=158 failures=0 blocking_failures=0`; existing non-blocking advisories: warm wall-time budget exceeded at `1018.43s` and group skew is high, with near-warm e2e cache (`cache_hits=181/182`, `rebuilt_groups=1`).
- Review: Claude Opus 4.7 review pass 1 reported no blockers and confirmed every claimed `code -> fixture` row rendered the claimed code in compact output. Pass-1 follow-up added a coverage-contract check that now enforces this evidence instead of relying on reviewer spot checks. Review pass 2 reported no blockers and no further review rounds needed; pass-2 optional polish tightened the `SIFR-STDLIB-0001` deferral reason to stand alone in `code_baseline_coverage.json`.

Fourth Wave 4 diagnostics-baseline slice:

- Status: source import compact baseline expansion implemented, locally validated, reviewed twice with no blockers, and merged in PR [#2574](https://github.com/sifr-lang/sifr/pull/2574).
- Scope: added four diagnostics-area compact baselines for canonical source import diagnostics: private member imports (`SIFR-IMPORT-0004`), ambiguous source modules (`SIFR-IMPORT-0005`), namespace/file collisions (`SIFR-IMPORT-0006`), and source import cycles (`SIFR-IMPORT-0007`). The fixtures include the project files needed to reproduce the source import resolution paths under the diagnostics baseline runner.
- Coverage status: rendered diagnostic coverage is now 101 active codes, with 69 active codes still carrying Wave 4 deferrals. The `IMPORT` family no longer has Wave 4 rendered-baseline deferrals.
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 99 cases and 127 renderer variants; `cargo test -p sifr_diagnostics` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed. After review pass 1, the private-member fixture gained a local `sifr.toml` and the source-import manifest cases were sorted; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` and `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` both passed again. Final post-review `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, and hardening `variants=5 failures=0 blocking_failures=0`; existing non-blocking advisory: warm wall-time budget exceeded at `249.31s` with two rebuilt e2e groups (`cache_hits=42/44`). The first `scripts/run_all_tests.sh` merge attempt hit a non-repeatable representative performance outlier in `check-project-004-project-graph`; the isolated representative performance rerun passed, and the final `scripts/run_all_tests.sh` merge rerun passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=127 failures=0 blocking_failures=0`, and hardening `variants=162 failures=0 blocking_failures=0`. Existing non-blocking advisories: warm wall-time budget exceeded at `1514.67s`, warm-cache hit rate below advisory target (`cache_hits=1/182`, `rebuilt_groups=181`), and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified every claimed code rendered in exactly one compact diagnostic, and requested optional fixture-local `sifr.toml` plus manifest ordering cleanup; both cleanups were applied. Review pass 2 reported no blockers, verified the cleanup, and stated that no further review rounds are required. Broader `source_origin`/multi-file source hashing and owner-family consistency were left as non-blocking future hygiene rather than new scope in this slice.

Fifth Wave 4 diagnostics-baseline slice:

- Status: semantic straggler compact baseline expansion implemented, locally merge-gate validated, reviewed twice with no blockers, and merged in PR [#2576](https://github.com/sifr-lang/sifr/pull/2576).
- Scope: added five diagnostics-area compact baselines for the remaining check-emitted semantic warning, note, and result straggler diagnostics: unreachable statements (`SIFR-FLOW-0901`), bigint transition alias warnings (`SIFR-INT-0011`), invalid except type forms (`SIFR-RESULT-0006`), integer overflow-risk warnings (`SIFR-TYPE-0901`), and reveal-type notes (`SIFR-TYPE-0902`). Each fixture is purpose-built to emit exactly one intended compact diagnostic.
- Coverage status: rendered diagnostic coverage is now 106 active codes, with 64 active codes still carrying Wave 4 deferrals. The remaining deferred families are `BUILD` (6), `ENCODING` (1), `FMT` (1), `INTERNAL` (1), `IO` (2), `LINT` (8), `PACKAGE` (34), `STDLIB` (3), and `WORKSPACE` (8).
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 104 cases / 132 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `cargo test -p sifr_diagnostics` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 104 cases and 132 renderer variants; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed. `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and non-blocking warm wall-time/cache advisories (`471.00s`, `cache_hits=2/44`). The first `scripts/run_all_tests.sh` merge attempt and first isolated representative performance rerun hit non-repeatable p95 budget outliers on different benchmarks; the second isolated representative performance rerun passed, and the final `scripts/run_all_tests.sh` merge rerun passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=132 failures=0 blocking_failures=0`, and hardening `variants=167 failures=0 blocking_failures=0`. Existing non-blocking advisories: warm wall-time budget exceeded at `1041.37s` and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified all five `code -> fixture` mappings, source hashes, manifest ordering, compact-only renderer intent, and the remaining deferral counts; no code or metadata changes were requested before full validation. Review pass 2 reported no blockers, verified the final validation notes, and stated that no further review rounds are needed before PR.

Sixth Wave 4 diagnostics-baseline slice:

- Status: lint compact baseline expansion implemented, locally merge-gate validated, reviewed twice with no blockers, and merged in PR [#2578](https://github.com/sifr-lang/sifr/pull/2578).
- Scope: added eight diagnostics-area compact baselines for the stable lint policy and suppression diagnostics: unknown suppression (`SIFR-LINT-0001`), unused suppression (`SIFR-LINT-0002`), blanket suppression (`SIFR-LINT-0003`), trailing whitespace (`SIFR-LINT-0004`), tracked TODO comments (`SIFR-LINT-0005`), boolean positional arguments (`SIFR-LINT-0006`), large parameter lists (`SIFR-LINT-0007`), and duplicate imports (`SIFR-LINT-0008`). The diagnostics baseline adapter now allows `lint` command baselines, and the coverage contract normalizes command-prefixed baseline labels such as `lint-compact` back to renderer `compact`.
- Coverage status: rendered diagnostic coverage is now 114 active codes, with 56 active codes still carrying Wave 4 deferrals. The `LINT` family no longer has Wave 4 rendered-baseline deferrals. The remaining deferred families are `BUILD` (6), `ENCODING` (1), `FMT` (1), `INTERNAL` (1), `IO` (2), `PACKAGE` (34), `STDLIB` (3), and `WORKSPACE` (8).
- Validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 112 cases / 140 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `cargo test -p sifr_lint` passed; `python3 -m py_compile verification/areas/diagnostics/checks/code_baseline_coverage.py verification/runner/sifr_verify/area_adapter.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 112 cases and 140 renderer variants; `uv run --project verification --locked python -m sifr_verify --self-test` passed. `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and the existing non-blocking warm wall-time advisory at `244.81s`. `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=140 failures=0 blocking_failures=0`, hardening `variants=175 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1054.05s` and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified each lint fixture emits exactly one intended compact diagnostic with empty stdout and exit code 1, accepted the narrow `.gitattributes` exception for the trailing-whitespace fixture, verified manifest/coverage/metadata/source-hash consistency, and stated no further review rounds were required before PR submission. Review pass 2 reported no blockers, independently verified the lint fixture shapes, `lint` command admission, renderer-label normalization, coverage and deferral counts, narrow `.gitattributes` scope, and full validation evidence, and stated that no further review rounds are required.

Seventh Wave 4 diagnostics-baseline slice:

- Status: workspace manifest/source-root compact baseline expansion merged via PR #2580: https://github.com/sifr-lang/sifr/pull/2580.
- Scope: added four diagnostics-area compact baselines for public-command-reachable workspace manifest and source-root diagnostics: malformed workspace manifest (`SIFR-WORKSPACE-0001`), source roots escaping the workspace root (`SIFR-WORKSPACE-0002`), source roots that are not directories (`SIFR-WORKSPACE-0003`), and invalid source-root entries (`SIFR-WORKSPACE-0004`). The lower-level legacy workspace graph codes (`SIFR-WORKSPACE-0101` through `SIFR-WORKSPACE-0104`) remain deferred because current public project import paths intentionally render their source-spanned `SIFR-IMPORT-*` replacements; those rows need a separate harness or coverage-policy decision rather than accidental duplicate import-family fixtures.
- Coverage status: rendered diagnostic coverage is now 118 active codes, with 52 active codes still carrying Wave 4 deferrals. The remaining deferred families are `BUILD` (6), `ENCODING` (1), `FMT` (1), `INTERNAL` (1), `IO` (2), `PACKAGE` (34), `STDLIB` (3), and `WORKSPACE` (4).
- Validation: direct compact CLI checks for the four new fixtures emitted exactly one intended `SIFR-WORKSPACE-0001` through `SIFR-WORKSPACE-0004` diagnostic and exited 1; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 116 cases / 144 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 116 cases and 144 renderer variants; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed. After the review deferral-reason follow-up, `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` and `git diff --check` passed again. `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `357.60s` and warm-cache hit rate below advisory target (`cache_hits=18/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=144 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=179 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1593.64s`, warm-cache hit rate below advisory target (`cache_hits=1/182`), and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the four `SIFR-WORKSPACE-0001` through `SIFR-WORKSPACE-0004` fixtures and counts, and confirmed that deferring `SIFR-WORKSPACE-0101` through `SIFR-WORKSPACE-0104` is technically honest for this focused slice. Review passes 2 and 3 correctly blocked misapplied free-text deferral-rationale edits; the final fix was grep-verified so the legacy workspace-graph rationale appears exactly on `SIFR-WORKSPACE-0101` through `SIFR-WORKSPACE-0104`, diagnostics contracts and `git diff --check` passed, and review pass 4 reported no new issues. Review pass 5 approved the one-file `.gitattributes` exception for the TOML parser's intentional trailing blank stderr baseline and stated no further review round is required before PR submission.

Eighth Wave 4 diagnostics-baseline slice:

- Status: formatter compact baseline expansion implemented, locally merge-gate validated, reviewed with no blockers, and merged in PR [#2582](https://github.com/sifr-lang/sifr/pull/2582).
- Scope: added diagnostics-area `fmt-check` baseline support and one compact baseline for formatter drift (`SIFR-FMT-0001`) through the public `sifr fmt --check --no-cache` command path.
- Coverage status: of 170 stable active diagnostic codes, 119 now have rendered baseline coverage and 51 carry Wave 4 deferrals. The `FMT` family no longer has Wave 4 rendered-baseline deferrals. The remaining deferred families are `BUILD` (6), `ENCODING` (1), `INTERNAL` (1), `IO` (2), `PACKAGE` (34), `STDLIB` (3), and `WORKSPACE` (4).
- Validation: direct compact CLI check for `fmt_formatting_drift` emitted exactly one intended `SIFR-FMT-0001` diagnostic, exited 1, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 117 cases / 145 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 117 cases and 145 renderer variants; `cargo test -p sifr_format` passed; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed. `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `331.17s` and warm-cache hit rate below advisory target (`cache_hits=26/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=145 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=180 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1057.54s` and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, confirmed the `fmt-check` adapter alias, public `sifr fmt --check --no-cache` reachability, `SIFR-FMT-0001` fixture output, coverage/metadata/source-hash counts, and formatter-family closure. The reviewer noted broad validation was pending at review time; after the create-pr and merge gates passed, no additional review round was required before PR submission.

Ninth Wave 4 diagnostics-baseline slice:

- Status: text I/O and encoding compact baseline expansion merged in PR https://github.com/sifr-lang/sifr/pull/2584 after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added diagnostics-area compact baselines for the text I/O and encoding substrate using purpose-built public check fixtures: explicit text encoding required (`SIFR-IO-0801`), statically known open mode required (`SIFR-IO-0802`), and statically known encoding error handler required (`SIFR-ENCODING-0803`).
- Coverage status: of 170 stable active diagnostic codes, 122 now have rendered baseline coverage and 48 carry Wave 4 deferrals. The `ENCODING` and `IO` families no longer have Wave 4 rendered-baseline deferrals. The remaining deferred families are `BUILD` (6), `INTERNAL` (1), `PACKAGE` (34), `STDLIB` (3), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks for the three new fixtures emitted exactly one intended `SIFR-ENCODING-0803`, `SIFR-IO-0802`, or `SIFR-IO-0801` diagnostic respectively, exited 1, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 120 cases / 148 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 120 cases and 148 renderer variants; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `354.75s` and warm-cache hit rate below advisory target (`cache_hits=18/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=148 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=183 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1365.28s`, warm-cache hit rate below advisory target (`cache_hits=105/182`), and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the three code-to-fixture mappings, direct compact output shape, manifest/coverage/metadata/source-hash consistency, and the 122 covered / 48 deferred tracker counts. The reviewer stated no additional review round is required before PR submission after the create-pr and merge gates pass.

Tenth Wave 4 diagnostics-baseline slice:

- Status: stdlib unsupported-surface compact baseline expansion merged in PR https://github.com/sifr-lang/sifr/pull/2586 after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added a purpose-built diagnostics-area compact baseline for the user-triggerable stdlib unsupported-surface diagnostic `SIFR-STDLIB-0001` using an explicit `sifr.collections.defaultdict` keyword-constructor fixture. The stdlib bootstrap/cache diagnostics `SIFR-STDLIB-0003` and `SIFR-STDLIB-0004` remain deferred with tightened rationale because current public commands cannot deterministically trigger them without corrupting embedded stdlib sources or injecting an internal cache failure; those rows need a lower-level rendered harness or explicit coverage-policy decision.
- Coverage status: of 170 stable active diagnostic codes, 123 now have rendered baseline coverage and 47 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (6), `INTERNAL` (1), `PACKAGE` (34), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI check for `e2e_stdlib_defaultdict_keyword_constructor` emitted exactly one intended `SIFR-STDLIB-0001` diagnostic, exited 1, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 121 cases / 149 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 121 cases and 149 renderer variants; `cargo test -p sifr_lowering defaultdict_keyword_constructor_unsupported_has_stdlib_code` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisory: warm wall-time budget exceeded at `186.00s`. `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=149 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=184 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1316.47s`, warm-cache hit rate below advisory target (`cache_hits=78/182`), and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the purpose-built `SIFR-STDLIB-0001` output, manifest/metadata/coverage/source-hash consistency, the 123 covered / 47 deferred counts, and the technical honesty of leaving `SIFR-STDLIB-0003` and `SIFR-STDLIB-0004` deferred to a lower-level rendered harness or coverage-policy decision. The reviewer stated no additional review round is required before PR submission after the broad gates pass.

Eleventh Wave 4 diagnostics-baseline slice:

- Status: self-update missing-receipt compact baseline expansion merged in PR [#2588](https://github.com/sifr-lang/sifr/pull/2588) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added diagnostics-area `self-version` baseline support for the public `sifr self version` command path and one compact baseline for missing standalone install receipts (`SIFR-BUILD-0901`) with `SIFR_INSTALL_MANIFEST_DIR` pinned to the fixture directory. The lower-level build materialization/workspace/Cargo/artifact diagnostics (`SIFR-BUILD-0002` through `SIFR-BUILD-0006`) remain deferred because current public commands do not deterministically trigger those filesystem/toolchain fault paths without an explicit failure-injection or lower-level rendered harness.
- Coverage status: of 170 stable active diagnostic codes, 124 now have rendered baseline coverage and 46 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (34), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI check for `self_update_missing_receipt` emitted exactly one intended `SIFR-BUILD-0901` diagnostic and exited 1; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 122 cases / 150 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 122 cases and 150 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `328.14s` and warm-cache hit rate below advisory target (`cache_hits=26/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=150 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=185 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1064.98s` and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `SIFR-BUILD-0901` code-to-fixture mapping, baseline output shape, source hash, manifest/metadata/coverage consistency, the 124 covered / 46 deferred counts, and the technical honesty of leaving `SIFR-BUILD-0002` through `SIFR-BUILD-0006` deferred to lower-level rendered harness or explicit failure-injection work. The reviewer stated no additional review round is required before PR submission after broad gates pass; the only optional follow-up is replacing the metadata `bless_reference` placeholder with the actual PR URL after the PR is opened.

Twelfth Wave 4 diagnostics-baseline slice:

- Status: package duplicate-public-API compact baseline expansion merged in PR [#2590](https://github.com/sifr-lang/sifr/pull/2590) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added diagnostics-area `package-check` baseline support for package-root public `sifr check src/main.sifr` command paths and one compact baseline for duplicate public API symbols (`SIFR-PACKAGE-0713`). The baseline checker now accepts nested entrypoint baseline directories such as `src/baselines/` while still attributing them to the top-level diagnostics fixture id. The remaining package diagnostics stay deferred for their own package CLI or lower-level rendered harness slices.
- Coverage status: of 170 stable active diagnostic codes, 125 now have rendered baseline coverage and 45 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (33), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI check for `package_duplicate_public_api_symbol` emitted exactly one intended `SIFR-PACKAGE-0713` diagnostic and exited 1; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 123 cases / 151 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 123 cases and 151 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `357.43s` and warm-cache hit rate below advisory target (`cache_hits=18/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=151 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=186 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1373.83s`, warm-cache hit rate below advisory target (`cache_hits=105/182`), and group skew is high.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `package-check` command path, nested `src/baselines` attribution, the `SIFR-PACKAGE-0713` coverage flip, counts, and metadata source hash. The reviewer noted only the copied fixture's inherited internal package names and the expected placeholder `bless_reference`; no additional review round is required before PR submission after the create-pr and merge gates pass.

Thirteenth Wave 4 diagnostics-baseline slice:

- Status: package explicit-file source-root compact baseline expansion merged in PR [#2592](https://github.com/sifr-lang/sifr/pull/2592) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added one package-root public `sifr check tools/task.sifr` compact baseline for explicit files outside the package source root (`SIFR-PACKAGE-0710`). This reuses the existing diagnostics-area `package-check` command and nested baseline attribution support for entrypoint-local `tools/baselines/` files.
- Coverage status: of 170 stable active diagnostic codes, 126 now have rendered baseline coverage and 44 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (32), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI check for `package_explicit_file_outside_source_root` emitted exactly one intended `SIFR-PACKAGE-0710` diagnostic, exited 1, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 124 cases / 152 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 124 cases and 152 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `package-check` command path, nested `tools/baselines/` attribution, the `SIFR-PACKAGE-0710` coverage flip, counts, and metadata source hash. The reviewer requested no further review round; the optional fixture consistency nits were addressed by aligning `Cargo.toml` to sibling package fixtures (`edition = "2021"` and an empty `[workspace]`), after which direct compact CLI, diagnostics contracts, and `git diff --check` passed again.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisory: warm wall-time budget exceeded at `194.51s` despite full e2e cache hit (`cache_hits=44/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=152 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=187 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1354.34s`, warm-cache hit rate below advisory target (`cache_hits=78/182`), and group skew is high.

Fourteenth Wave 4 diagnostics-baseline slice:

- Status: package script-recursion compact baseline expansion merged in PR [#2594](https://github.com/sifr-lang/sifr/pull/2594) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added one package-root public `sifr run --script dev` compact baseline for package scripts that attempt to expand another script (`SIFR-PACKAGE-0714`). This adds the diagnostics-area `package-run-script` command path beside `package-check`, sharing the same package-root discovery and Cargo manifest invocation.
- Coverage status: of 170 stable active diagnostic codes, 127 now have rendered baseline coverage and 43 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (31), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI check for `package_script_recursion` emitted exactly one intended `SIFR-PACKAGE-0714` diagnostic, exited 1, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 125 cases / 153 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 125 cases and 153 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `package-run-script` command path, package-root `baselines/` attribution from `sifr.toml`, the `SIFR-PACKAGE-0714` coverage flip, counts, and metadata source hash. The reviewer noted only the expected placeholder `bless_reference`, the intentionally single-purpose `dev` script adapter, and a pre-existing diagnostic-template wording mismatch outside this PR; no additional review round was requested before PR submission after the broad gates pass.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `346.50s` and warm-cache hit rate below advisory target (`cache_hits=26/44`). `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=153 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=188 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1068.35s` and group skew is high.

Fifteenth Wave 4 diagnostics-baseline slice:

- Status: package production-manifest compact baseline expansion merged in PR [#2596](https://github.com/sifr-lang/sifr/pull/2596) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added package-root public `sifr check` default-command baseline support for production `sifr.toml` manifest-shape diagnostics and compact baselines for `[exports].modules` (`SIFR-PACKAGE-0701`) and `[[bin]]` target tables (`SIFR-PACKAGE-0711`). The diagnostics-area runner now has `package-check-default`, which reuses package-root discovery and invokes `check` without an explicit file so the manifest itself is the entrypoint and source-hash evidence.
- Coverage status: of 170 stable active diagnostic codes, 129 now have rendered baseline coverage and 41 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (29), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the two fixture package roots emitted exactly one intended `SIFR-PACKAGE-0701` or `SIFR-PACKAGE-0711` diagnostic respectively, exited 2, and wrote empty stdout; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 127 cases / 155 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 127 cases and 155 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `package-check-default` command path, production-manifest source-hash evidence, root-level baseline attribution, direct compact output shape, the `SIFR-PACKAGE-0701` and `SIFR-PACKAGE-0711` coverage flips, counts, and metadata consistency. The reviewer requested no further review round before PR submission after broad gates pass; the only required follow-up is replacing the metadata `bless_reference` placeholder with the actual PR URL after the PR is opened.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `362.06s` and warm-cache hit rate below advisory target (`cache_hits=18/44`). The first `scripts/run_all_tests.sh` attempt stopped on a representative performance timing miss for `check-project-004-project-graph`; a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed, and the subsequent full `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=155 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=190 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1362.93s`, warm-cache hit rate below advisory target (`cache_hits=105/182`), and group skew is high.

Sixteenth Wave 4 diagnostics-baseline slice:

- Status: package projection repair-check compact baseline expansion merged in PR [#2598](https://github.com/sifr-lang/sifr/pull/2598) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added package-root public `sifr repair --check` baseline support and compact baselines for Cargo projection manifest pointer drift (`SIFR-PACKAGE-0703`), Cargo projection include drift (`SIFR-PACKAGE-0704`), and missing pure-package marker repair diagnostics (`SIFR-PACKAGE-0709`). The diagnostics-area runner now has `package-repair-check`, which reuses package-root discovery and invokes `repair --check` from the fixture root so Cargo projection diagnostics are exercised through the public CLI path.
- Coverage status: of 170 stable active diagnostic codes, 132 now have rendered baseline coverage and 38 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (26), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the two fixture package roots emitted the intended diagnostics through `sifr repair --check`: the manifest-pointer fixture emitted `SIFR-PACKAGE-0703` and `SIFR-PACKAGE-0704`, exited 1, and wrote empty stdout; the missing-pure-marker fixture emitted `SIFR-PACKAGE-0709`, exited 1, and wrote empty stdout. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 129 cases / 157 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with 129 cases and 157 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed.
- Review: Claude Opus 4.7 review pass 1 reported no blockers, verified the `package-repair-check` public CLI command path, package-root baseline attribution, deterministic fixture isolation, the intentional `SIFR-PACKAGE-0703`/`SIFR-PACKAGE-0704` co-emission from one production-realistic drift fixture, the `SIFR-PACKAGE-0709` single-code fixture, coverage counts, and metadata source hashes. The reviewer requested no further review round before PR submission after broad gates pass; the metadata `bless_reference` placeholders were replaced with the actual PR URL after PR #2598 was opened.
- Broad validation: `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `332.96s` and warm-cache hit rate below advisory target (`cache_hits=26/44`). The first full `scripts/run_all_tests.sh` attempt stopped on a representative performance timing miss for `build-single-file-001-break-continue`; a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed. The second full `scripts/run_all_tests.sh` attempt stopped on a representative performance timing miss for `check-project-004-project-graph`; a second focused representative performance rerun passed. The third full `scripts/run_all_tests.sh` passed with e2e `651/651`, signature `ee5e5d44306f270c`, diagnostics baselines `variants=157 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=192 failures=0 blocking_failures=0`, and existing non-blocking advisories: warm wall-time budget exceeded at `1351.28s`, warm-cache hit rate below advisory target (`cache_hits=77/182`), and group skew is high.

Seventeenth Wave 4 diagnostics-baseline slice:

- Status: package metadata/source-layout compact baseline expansion merged in PR [#2600](https://github.com/sifr-lang/sifr/pull/2600) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation.
- Scope: added package-root public `sifr check src/main.sifr` compact baselines for missing pointed-to Sifr manifests (`SIFR-PACKAGE-0002`), misplaced compiler semantics in Cargo metadata (`SIFR-PACKAGE-0003`), and non-trivial Rust implementation in pure-package marker targets (`SIFR-PACKAGE-0501`). The slice intentionally keeps `SIFR-PACKAGE-0001` deferred because it represents malformed Cargo metadata JSON parsing, not a natural `cargo metadata` public CLI emission path.
- Coverage status: of 170 stable active diagnostic codes, 135 now have rendered baseline coverage and 35 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (23), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the three fixture package roots emitted the intended diagnostics through explicit package file checks: the missing-manifest fixture emitted `SIFR-PACKAGE-0002` and exited 1, the misplaced-metadata fixture emitted `SIFR-PACKAGE-0003` and exited 2, and the non-trivial-marker fixture emitted `SIFR-PACKAGE-0501` and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 132 cases / 160 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 132 cases / 160 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-metadata-layout-baselines-review-pass-1.md` and stated another review round is not required before gates.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: first `scripts/run_all_tests.sh` attempt failed only on representative performance budget noise for `build-single-file-001-break-continue` (`measured=1798.338`, `threshold=1653.637`); focused `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` rerun passed. The second full `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=160`, project-workspace baselines `variants=17`, hardening `variants=195 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.

Eighteenth Wave 4 diagnostics-baseline slice:

- Status: package run-target compact baseline expansion merged via PR #2602; tracker-only closure PR pending.
- Scope: added public package-root `sifr run admin` and `sifr run --bin bad!name` compact baselines for ambiguous app/script run target selection (`SIFR-PACKAGE-0605`) and invalid app target naming (`SIFR-PACKAGE-0606`). The slice added diagnostics adapter aliases for those exact public CLI forms so the baselines stay tied to user-facing commands.
- Coverage status: of 170 stable active diagnostic codes, 137 now have rendered baseline coverage and 33 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (21), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the two fixture package roots emitted the intended diagnostics through public package run-target commands: the ambiguous target fixture emitted `SIFR-PACKAGE-0605` and exited 1, and the invalid app target fixture emitted `SIFR-PACKAGE-0606` and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 134 cases / 162 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 134 cases / 162 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-run-target-baselines-review-pass-1.md` and stated another review round is not required before gates.
- PR: https://github.com/sifr-lang/sifr/pull/2602
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=162`, project-workspace baselines `variants=17`, hardening `variants=197 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/skew notes.

Nineteenth Wave 4 diagnostics-baseline slice:

- Status: merged via https://github.com/sifr-lang/sifr/pull/2604 and tracker closeout merged via https://github.com/sifr-lang/sifr/pull/2605.
- Scope: added public package-root `sifr package --workspace --list --no-verify --allow-dirty` compact baselines for duplicate workspace import roots (`SIFR-PACKAGE-0602`) and duplicate Sifr package names (`SIFR-PACKAGE-0607`). The slice added a diagnostics adapter alias for that exact public CLI form so the baselines stay tied to release-package workspace selection rather than lower-level graph helpers.
- Coverage status: of 170 stable active diagnostic codes, 139 now have rendered baseline coverage and 31 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (19), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the two fixture workspace roots emitted the intended diagnostics through public package workspace-selection commands: the duplicate import-root fixture emitted `SIFR-PACKAGE-0602` and exited 1, and the duplicate Sifr-name fixture emitted `SIFR-PACKAGE-0607` and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 136 cases / 164 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 136 cases / 164 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-workspace-selection-baselines-review-pass-1.md` and stated another review round is not required before gates.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=164`, project-workspace baselines `variants=17`, hardening `variants=199 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache/skew notes.

Twentieth Wave 4 diagnostics-baseline slice:

- Status: package selection/classification compact baseline expansion merged in PR [#2606](https://github.com/sifr-lang/sifr/pull/2606) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation; tracker closeout merged in PR [#2607](https://github.com/sifr-lang/sifr/pull/2607).
- Scope: added public package-root compact baselines for explicit Rust-only package selection (`SIFR-PACKAGE-0102`) through `sifr package -p rust-helper --list --no-verify --allow-dirty`, Rust-only workspace member dependency on a Sifr package (`SIFR-PACKAGE-0106`) through `sifr package --workspace --list --no-verify --allow-dirty`, and invalid package selector handling (`SIFR-PACKAGE-0601`) through `sifr package -p missing --list --no-verify --allow-dirty`. The slice added diagnostics adapter aliases for the explicit public package selector commands.
- Coverage status: of 170 stable active diagnostic codes, 142 now have rendered baseline coverage and 28 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (16), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the three fixture package roots emitted the intended diagnostics through public package commands: the Rust-only selection fixture emitted `SIFR-PACKAGE-0102` and exited 1, the Rust-only dependency fixture emitted `SIFR-PACKAGE-0106` and exited 1, and the invalid selector fixture emitted `SIFR-PACKAGE-0601` and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 139 cases / 167 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 139 cases / 167 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-selection-classification-baselines-review-pass-1.md` and stated another full review round is not required before gates. After the review, the non-blocking fixture-name cleanup was applied and `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts`, unblessed diagnostics baselines, and `git diff --check` passed again.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: the first `scripts/run_all_tests.sh` attempt stopped on a representative performance timing miss for `check-single-file-001-arithmetic`; a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed. The subsequent full `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=167`, project-workspace baselines `variants=17`, hardening `variants=202 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache/skew notes.

Twenty-first Wave 4 diagnostics-baseline slice:

- Status: package release-preflight compact baseline expansion merged in PR [#2609](https://github.com/sifr-lang/sifr/pull/2609) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation; tracker closeout merged in PR [#2610](https://github.com/sifr-lang/sifr/pull/2610).
- Scope: added package-root public `sifr package --list --no-verify --allow-dirty` compact baselines for untrusted backend Rust crates (`SIFR-PACKAGE-0301`), stale trust entries that are not direct backend dependencies (`SIFR-PACKAGE-0305`), archives containing no `.sifr` source (`SIFR-PACKAGE-0401`), and Cargo include/exclude rules omitting required Sifr source (`SIFR-PACKAGE-0403`). The slice added a diagnostics adapter alias for the default public package-list command.
- Coverage status: of 170 stable active diagnostic codes, 146 now have rendered baseline coverage and 24 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (12), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the four fixture package roots emitted the intended diagnostics through public package commands and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 143 cases / 171 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 143 cases / 171 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-release-preflight-baselines-review-pass-1.md` and stated another review round is not required before gates. The reviewer verified public `sifr package --list --no-verify --allow-dirty` reachability, the four code-to-fixture mappings, metadata source hashes, baseline trios, and the 146 covered / 24 deferred tracker counts.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: the first `scripts/run_all_tests.sh` attempt stopped on a representative performance timing miss for `check-project-004-project-graph`; a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed. The subsequent full `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=171 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=206 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache/skew notes.

Twenty-second Wave 4 diagnostics-baseline slice:

- Status: package dependency-boundary compact baseline expansion merged in PR [#2612](https://github.com/sifr-lang/sifr/pull/2612) after focused validation, Claude Opus review, create-pr validation, and merge-gate validation; tracker closeout merged in PR [#2613](https://github.com/sifr-lang/sifr/pull/2613).
- Scope: added public `sifr check src/main.sifr` package-session compact baselines for ambiguous direct dependency import roots (`SIFR-PACKAGE-0201`), imports from transitive dependencies that are outside the current package's direct scope (`SIFR-PACKAGE-0202`), and private dependency module imports (`SIFR-PACKAGE-0203`).
- Coverage status: of 170 stable active diagnostic codes, 149 now have rendered baseline coverage and 21 carry Wave 4 deferrals. The remaining deferred families are `BUILD` (5), `INTERNAL` (1), `PACKAGE` (9), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: direct compact CLI checks from the three package app roots emitted exactly one intended `SIFR-PACKAGE-0201`, `SIFR-PACKAGE-0202`, or `SIFR-PACKAGE-0203` diagnostic through `sifr check src/main.sifr` and exited 1. `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` passed and wrote 146 cases / 174 renderer variants; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed 146 cases / 174 renderer variants; `python3 -m py_compile verification/runner/sifr_verify/area_adapter.py verification/areas/diagnostics/checks/code_baseline_coverage.py`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-dependency-boundary-baselines-review-pass-1.md` and stated another review round is not required before create-pr / merge-gate validation. The reviewer verified public `sifr check src/main.sifr` reachability for all three package-session cases, exact compact diagnostics, manifest/coverage/metadata entries, source hashes, baseline trios, absence of generated fixture artifacts, and the 149 covered / 21 deferred tracker counts.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: the first `scripts/run_all_tests.sh` attempt stopped on a representative performance p95 timing miss for `check-single-file-001-arithmetic`; a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed. The subsequent full `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=174 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=209 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache/skew notes.

Twenty-third Wave 4 diagnostics-baseline slice:

- Status: package library/Cargo-environment diagnostic synthetic compact baseline expansion merged in PR [#2615](https://github.com/sifr-lang/sifr/pull/2615).
- Scope: extends `code_baseline_coverage.py` so metadata-owned `synthetic: true` baseline fixtures can satisfy rendered baseline coverage when the baseline trio exists, the metadata source hash is current, the renderer is supported by the code catalog, and stderr evidence contains the claimed code. Adds a compact synthetic package diagnostic contract baseline for the remaining stable package diagnostics that are library-level, Cargo-backend dependent, or not deterministic through public CLI fixtures: invalid Cargo Sifr metadata (`SIFR-PACKAGE-0001`), Cargo command failure (`SIFR-PACKAGE-0101`), Cargo metadata parse failure (`SIFR-PACKAGE-0103`), offline source unavailability (`SIFR-PACKAGE-0104`), package type identity mismatch (`SIFR-PACKAGE-0204`), publish validation failure (`SIFR-PACKAGE-0402`), package archive traversal (`SIFR-PACKAGE-0404`), changed-file mapping failure (`SIFR-PACKAGE-0603`), and unsupported outdated query source (`SIFR-PACKAGE-0604`).
- Coverage status: of 170 stable active diagnostic codes, 158 now have rendered baseline coverage and 12 carry Wave 4 deferrals. Package diagnostic deferrals are closed; the remaining deferred families are `BUILD` (5), `INTERNAL` (1), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: `python3 -m py_compile verification/areas/diagnostics/checks/code_baseline_coverage.py` passed; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed with `variants=5 failures=0 blocking_failures=0`; unblessed `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with `variants=174 failures=0 blocking_failures=0`; `python3 scripts/check_file_size_guardrails.py` and `git diff --check` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-library-diagnostic-baselines-review-pass-1.md` and stated another review round is not required before create-pr / merge-gate validation. The reviewer verified source-hash binding, baseline trio hygiene, stderr code evidence for all nine package diagnostics, renderer-support cross-checks, tracker counts, and synthetic-baseline precedent parity.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache notes.
- Merge-gate validation: the first `scripts/run_all_tests.sh` attempt stopped on a representative performance p95 timing miss for `check-project-004-project-graph` (`measured=1593.865`, `threshold=1428.693`); a focused rerun of `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative` passed. The subsequent full `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=174 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=209 failures=0 blocking_failures=0`, and advisory-only warm-cache timing/cache/skew notes.

Twenty-fourth Wave 4 diagnostics-baseline slice:

- Status: internal diagnostic synthetic renderer coverage merged in PR [#2617](https://github.com/sifr-lang/sifr/pull/2617).
- Scope: reuses the existing metadata-owned `presentation_contract_cases` synthetic renderer fixture to close the `SIFR-INTERNAL-0001` rendered-baseline deferral across human, JSON, and compact renderers. The fixture already owns source-hash-checked `synthetic: true` metadata and baseline trios for all three renderer formats, and the baselines render the spanless internal diagnostic.
- Coverage status: of 170 stable active diagnostic codes, 159 now have rendered baseline coverage and 11 carry Wave 4 deferrals. Internal diagnostic deferrals are closed; the remaining deferred families are `BUILD` (5), `STDLIB` (2), and `WORKSPACE` (4).
- Focused validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed with `variants=5 failures=0 blocking_failures=0`; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` passed.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-internal-diagnostic-baseline-review-pass-1.md` and stated another review round is not required. The reviewer verified all three renderer evidence files, synthetic metadata ownership, source-hash/trio enforcement, renderer subset checks, and the 159 covered / 11 deferred tracker counts.
- Create-pr validation: the first `scripts/run_all_tests.sh --profile create-pr` attempt stopped on `task_gather_cleanup_error_secondary` with one cached e2e assertion failure; a targeted one-fixture e2e rerun passed. The subsequent full `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm-cache timing notes.
- Merge-gate validation: `scripts/run_all_tests.sh` passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=174 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=209 failures=0 blocking_failures=0`, and advisory-only warm wall-time / group-skew notes.

Twenty-fifth Wave 4 diagnostics-baseline slice:

- Status: final BUILD, STDLIB, and legacy WORKSPACE synthetic compact baseline coverage merged in PR [#2619](https://github.com/sifr-lang/sifr/pull/2619).
- Scope: adds metadata-owned synthetic compact rendered evidence for the remaining lower-level active stable diagnostic codes whose deterministic public CLI fixtures require internal fault injection or retired public renderer paths: `SIFR-BUILD-0002`, `SIFR-BUILD-0003`, `SIFR-BUILD-0004`, `SIFR-BUILD-0005`, `SIFR-BUILD-0006`, `SIFR-STDLIB-0003`, `SIFR-STDLIB-0004`, `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, `SIFR-WORKSPACE-0103`, and `SIFR-WORKSPACE-0104`.
- Coverage status: all 170 stable active diagnostic codes now have rendered baseline coverage and zero Wave 4 rendered-baseline deferrals remain.
- Focused validation: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed with `variants=5 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` passed with `variants=174 failures=0 blocking_failures=0`; coverage count check reported `170` rows covered and `0` deferred.
- Review: Claude Opus review pass 1 recorded no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-4-final-internal-diagnostic-baselines-review-pass-1.md` and stated no further review round is required before create-pr and merge-gate validation. The reviewer verified synthetic-baseline appropriateness, all 11 code-to-fixture mappings, compact stderr evidence, metadata source-hash binding, tracker counts, and baseline hygiene.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time / warm-cache hit-rate notes.
- Merge-gate validation: the first `scripts/run_all_tests.sh` attempt hit a non-repeatable representative-performance p95 outlier in `phase27-non-regression-002-json-diagnostic-schema` (`1841.388` measured vs `1487.2` threshold). The isolated representative performance rerun passed with `variants=6 failures=0 blocking_failures=0`, including the previously failing benchmark. The final full `scripts/run_all_tests.sh` rerun passed with 651/651 e2e pass fixtures, report signature `ee5e5d44306f270c`, diagnostics baselines `variants=174 failures=0 blocking_failures=0`, project-workspace baselines `variants=17 failures=0 blocking_failures=0`, hardening `variants=209 failures=0 blocking_failures=0`, and advisory-only warm wall-time / warm-cache hit-rate / group-skew notes.

### Wave 5: IR, HIR, CFG, and Codegen Snapshot Suites

Goal: catch lowering and codegen regressions at the layer where they are introduced.

Dependency: Wave 5 is blocked on Wave 2.final because codegen snapshot work must not build on a red `sifr_codegen` suite.

Wave 5 ships as numbered sub-PRs, not a single PR:

- 5.1 Parsed-source shape snapshots at the Sifr-owned boundary: `sifr_syntax` / `sifr_frontend` above the parser fork
- 5.2 HIR-lowering snapshots
- 5.3 Name-resolution snapshots
- 5.4 Type and ownership fact snapshots
- 5.5 CFG and flow fact snapshots
- 5.6 Codegen IR or structured input snapshots
- 5.7 Emitted-Rust snapshots for stable constructs
- 5.8 Compiler crash/ICE contract and sentinels

Tasks:

- Create per-layer snapshot inventory files before adding snapshots:
  - `verification/areas/core_language/data/lowering_layer_inventory.json` for parsed-source, HIR, name-resolution, type/ownership, and CFG/flow contracts
  - `verification/areas/generated_code_quality/data/codegen_construct_inventory.json` for codegen IR, structured inputs, and emitted-Rust stable constructs
- Each inventory entry declares the stable contract id, owner, source fixture or construct, snapshot id, normalizers, profile assignment, and replacement/mapping if coverage exists elsewhere.
- Add or formalize layer-specific snapshot suites for:
  - parsed-source shape at the Sifr-owned boundary: `sifr_syntax` / `sifr_frontend` above `sifr_python_parser` / `sifr_python_ast`
  - HIR lowering
  - name resolution
  - type and ownership facts
  - CFG/flow facts
  - generated Rust IR or structured codegen inputs
  - emitted Rust for selected stable constructs
- Prefer structured snapshots over string substring assertions where possible.
- Normalize nondeterministic ids, paths, tempdirs, and ordering.
- Add snapshot stale/unused detection, schema versioning, and normalizer inventory.
- Add source-map/span mapping snapshots if generated Rust diagnostics or runtime errors are expected to map back to Sifr source.
- Add debug-info or stack-trace coverage rows if Sifr ships debug/runtime stack traces.
- Add a first-class compiler crash/ICE contract:
  - invalid user programs produce diagnostics, not panics
  - unexpected panics/ICEs are always test failures
  - known crashes live only in issue-linked crash sentinel fixtures
  - each sentinel has reproduction command, expected crash signature, owner, and expiry
  - if a known crash stops crashing, the sentinel fails and must be closed or reclassified
  - at expiry, the sentinel is re-triaged: fix the crash, reclassify the surface, or extend the expiry with reason; an expired sentinel fails the regression suite
- Add bless/update workflow documentation.
- Add coverage rows for each layer to the coverage matrix.
- Keep snapshots focused on compiler contracts, not incidental formatting.

Exit criteria:

- A regression in lowering or codegen can fail without waiting for a native binary runtime mismatch.
- Snapshot output is reviewable and normalized.
- Existing HIR/CFG contract matrices are either integrated or explicitly mapped.

Focused validation:

```bash
cargo test -p sifr_lowering
cargo test -p sifr_analysis
cargo test -p sifr_codegen
uv run --project verification --locked python -m sifr_verify areas run --area core_language
scripts/run_all_tests.sh --profile create-pr
```

### Wave 5 Implementation Notes

Wave 5.1 parsed-source shape inventory slice:

- Status: parsed-source shape inventory and snapshots merged in PR [#2621](https://github.com/sifr-lang/sifr/pull/2621).
- Scope: adds `verification/areas/core_language/data/lowering_layer_inventory.json`, a core-language `lowering_layer_inventory` area-check, and Sifr-owned parsed-source statement-tree snapshots in `syntax_parser_lexer_matrix.json`. The existing syntax-matrix tests were split from `crates/sifr_syntax/src/lib.rs` into `crates/sifr_syntax/src/syntax_matrix_tests.rs` to keep hand-maintained source under the 900-line guardrail.
- Focused validation: `cargo test -p sifr_syntax` passed; `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory --suite syntax_parser_lexer_matrix` passed with `variants=2 failures=0 blocking_failures=0`; full `uv run --project verification --locked python -m sifr_verify areas run --area core_language` passed with `variants=6 failures=0 blocking_failures=0`; after review follow-up edits, `python3 -m py_compile verification/areas/core_language/checks/lowering_layer_inventory.py`, `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory`, `python3 scripts/check_file_size_guardrails.py`, and `git diff --check` passed. Wave 5 crate-test trio `cargo test -p sifr_lowering -p sifr_analysis -p sifr_codegen` passed. `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time / warm-cache hit-rate notes.
- Review: Claude Opus review pass 1 reported no blocking findings in `plans/reviews/active/ad-hoc-world-class-verification-wave-5-1-parsed-source-shape-review-pass-1.md` and stated no full review round was required. Non-blocking residual risks were addressed by documenting the `primary-body-only` normalizer and tightening inventory replacement/snapshot-id invariants. Review pass 2 in `plans/reviews/active/ad-hoc-world-class-verification-wave-5-1-parsed-source-shape-review-pass-2.md` reported no blocking findings and stated no further review round is required.

Wave 5.2 HIR-lowering snapshot slice:

- Status: merged in PR [#2623](https://github.com/sifr-lang/sifr/pull/2623).
- Scope: adds `verification/areas/core_language/data/hir_lowering_snapshot_matrix.json` and a `sifr_lowering` snapshot-matrix test that lowers each source fixture and compares a normalized HIR module projection. The inventory now includes active HIR-lowering rows for arithmetic function lowering and `for`/`if` accumulator lowering, and the inventory checker validates `hir_module_shape` rows against `expected_hir_snapshot` evidence.
- Focused validation so far: `cargo test -p sifr_lowering hir_lowering_snapshot_matrix_matches_lowered_module_shape` passed after aligning the expected matrix to the actual HIR canonicalization of `IteratorCall::Iter` and `+=`; `python3 -m py_compile verification/areas/core_language/checks/lowering_layer_inventory.py`, `python3 scripts/check_file_size_guardrails.py`, `git diff --check`, and `cargo fmt --check` passed; `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory` passed with `variants=1 failures=0 blocking_failures=0`; `cargo test -p sifr_lowering` passed with `560 passed, 1 ignored`; full `uv run --project verification --locked python -m sifr_verify areas run --area core_language` passed with `variants=6 failures=0 blocking_failures=0`; and `cargo test -p sifr_analysis -p sifr_codegen` passed with `27` analysis tests and `710` codegen tests. After review follow-up, `python3 -m py_compile verification/areas/core_language/checks/lowering_layer_inventory.py`, `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory`, `python3 scripts/check_file_size_guardrails.py`, `git diff --check`, and full `uv run --project verification --locked python -m sifr_verify areas run --area core_language` passed again. `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time budget overrun.
- Review: Claude Opus review pass 1 in `plans/reviews/active/ad-hoc-world-class-verification-wave-5-2-hir-lowering-snapshots-review-pass-1.md` reported no blockers and said no second review was required before merge. It recommended adding matrix-to-inventory stale/unclaimed-fixture detection before broader HIR rows; that follow-up was implemented in the inventory checker. Review pass 2 in `plans/reviews/active/ad-hoc-world-class-verification-wave-5-2-hir-lowering-snapshots-review-pass-2.md` confirmed the guard closes unclaimed matrix drift, reported no blockers, and stated another review round is not required. Deferred non-blocking risks: expand HIR projection fields and split projection helpers before adding broader HIR/name/type rows.

Wave 5.3 name-resolution snapshot slice:

- Status: merged in PR [#2625](https://github.com/sifr-lang/sifr/pull/2625).
- Scope: adds `verification/areas/core_language/data/name_resolution_snapshot_matrix.json` and a dedicated `sifr_lowering` name-resolution fact snapshot test. The snapshots cover parameter-over-module-constant shadowing, nested function capture plus nested callable resolution, and builtin method-call/loop-target/local reuse facts. The lowering-layer inventory now includes active `name_resolution` rows, and the inventory checker validates `name_resolution_facts` rows plus matrix-to-inventory claim coverage.
- Focused validation so far: `cargo test -p sifr_lowering name_resolution_snapshot_matrix_matches_lowered_name_facts` passed after aligning `len(items)` to the actual HIR `MethodCall` form; `jq empty verification/areas/core_language/data/name_resolution_snapshot_matrix.json`, `python3 -m py_compile verification/areas/core_language/checks/lowering_layer_inventory.py`, and `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory` passed with `variants=1 failures=0 blocking_failures=0`; `cargo test -p sifr_lowering` passed with `561 passed, 1 ignored`; full `uv run --project verification --locked python -m sifr_verify areas run --area core_language` passed with `variants=6 failures=0 blocking_failures=0`; `python3 scripts/check_file_size_guardrails.py`, `cargo fmt --check`, and `git diff --check` passed; and `cargo test -p sifr_analysis -p sifr_codegen` passed with `27` analysis tests and `710` codegen tests. After review follow-up, `cargo fmt --check`, `python3 -m py_compile verification/areas/core_language/checks/lowering_layer_inventory.py`, `cargo test -p sifr_lowering name_resolution_snapshot_matrix_matches_lowered_name_facts`, `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite lowering_layer_inventory`, `jq empty verification/areas/core_language/data/name_resolution_snapshot_matrix.json`, `python3 scripts/check_file_size_guardrails.py`, `git diff --check`, `cargo test -p sifr_lowering`, full `uv run --project verification --locked python -m sifr_verify areas run --area core_language`, and `cargo test -p sifr_analysis -p sifr_codegen` passed again. `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time budget overrun.
- Review: Claude Opus review pass 1 in `plans/reviews/active/ad-hoc-world-class-verification-wave-5-3-name-resolution-snapshots-review-pass-1.md` reported no blockers, judged the slice acceptable for PR, and said another Opus review round is not required. Optional follow-ups folded into this branch: projection-scope documentation, plain-assignment path labeling, and per-snapshot-kind normalizer validation in the inventory checker. Deferred non-blocking risks: broaden binder-producing projection coverage before fixtures depend on class/import scopes, pattern captures, with/as targets, tuple/star unpack targets, lambda/comprehension targets, async-with targets, or field/subscript object references; consider shared snapshot projection helpers before adding more Wave 5 snapshot kinds.
- Closeout validation: docs-only closeout `git diff --check` passed. The first `scripts/run_all_tests.sh --profile create-pr` closeout run hit non-repeating 10s timeouts in `iteration-protocol-01-for-over-collections` and `object-model-01-equality-vs-identity`; focused `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite audit-fixtures` passed those fixtures at about 2.5s each, and the full `scripts/run_all_tests.sh --profile create-pr` rerun passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, plus advisory-only warm wall-time and warm-cache hit-rate notes.

### Wave 6.0 Implementation Notes

Wave 6.0 CPython divergence catalogue and hand-seeded merge smoke slice:

- Status: merged in PR [#2627](https://github.com/sifr-lang/sifr/pull/2627).
- Scope: adds `verification/policy/cpython_differential.md`, a dedicated `cpython_differential` verification area, policy/catalogue linting for required/duplicate/unreferenced exclusion ids and stale hand-seeded entries, four deterministic hand-seeded CPython-vs-Sifr smoke fixtures, merge-profile execution for `cpython_differential:policy,hand_seeded_merge`, and coverage-matrix closure for `cpython_hand_seeded_differential` without promoting generated differential coverage.
- Focused validation so far: `jq empty` passed for the new area manifest, hand-seeded manifest, merge profile, and coverage matrix; `python3 -m py_compile` passed for the new linter, smoke runner, area runner, and updated profile runner; `uv run --project verification --locked python -m sifr_verify areas check` passed and discovered `cpython_differential`; `uv run --project verification --locked python -m sifr_verify profiles plan --profile merge` includes `cpython_differential` with `policy` and `hand_seeded_merge`; `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite policy --suite hand_seeded_merge` passed with `variants=2 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite advisory` passed and reduced temporary rows from 18 to 17; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; and `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and an advisory-only warm wall-time budget overrun.
- Review: Claude Opus review pass 1 in `plans/reviews/active/ad-hoc-world-class-verification-wave-6-0-cpython-differential-smoke-review-pass-1.md` reported no blocking findings, judged the slice acceptable for PR, and said another Opus review round is not required if mechanical polish is resolved without changing policy semantics. Folded-in follow-ups: manifest `supported_constructs` validation against Table 1 ids, plain assignment in the list-iteration fixture, nightly coverage-matrix policy suite inclusion, orphan exit-code-table detection, deterministic timeout reporting, and this implementation-notes section.
- Deferred non-blocking risks: make policy prose checks more structural, embed `sys.version` in a durable area result artifact, and enforce the full version-1 value grammar at runtime before generated Wave 6.1 programs are promoted.
- Closeout validation: docs-only closeout `git diff --check` passed, and `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, plus an advisory-only warm wall-time budget overrun.

Wave 6.1 generated CPython differential generator, serializer, and shrinker slice:

- Status: merged in PR [#2629](https://github.com/sifr-lang/sifr/pull/2629).
- Scope: adds a deterministic grammar-based generated seed manifest, generated CPython-vs-Sifr suite runners for `generated_minimized_seeds` and `generated_broader`, value-grammar validation, release-binary build/hash/source-digest evidence, timeout and exit/error-bucket comparison, minimized-failure artifact output, an empty checked-in minimized-failure ledger, generated-manifest linting, and nightly/release profile wiring for `generated_broader`. Merge promotion remains blocked by the 20-consecutive-nightly-green rule.
- Focused validation so far: `jq empty` passed for the CPython area manifest, generated seed manifest, minimized-failure ledger, nightly profile, and release profile; `python3 -m py_compile` passed for the generated runner, generator, wrapper scripts, and catalogue linter; `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite policy` passed; `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite generated_minimized_seeds` passed with four generated seeds; `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite generated_broader` passed with six generated seeds; combined `policy`, `hand_seeded_merge`, `generated_minimized_seeds`, and `generated_broader` passed with `variants=4 failures=0 blocking_failures=0`; nightly and release profile plans include `cpython_differential` suites `policy`, `hand_seeded_merge`, and `generated_broader`; `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite advisory` passed; `uv run --project verification --locked python -m sifr_verify areas check` passed; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `python3 scripts/check_file_size_guardrails.py` passed; and `git diff --check` passed.
- Implementation note: the generated dict shape originally exposed unsupported raw dict serialization/order and Option-valued direct lookup materialization. The generator now excludes that behavior by construction and tests dict support through existing-key lookup facts serialized as a homogeneous boolean list, matching the Wave 6.0 supported subset.
- Review: Claude Opus review pass 4 in `plans/reviews/active/ad-hoc-world-class-verification-wave-6-1-cpython-generator-shrinker-review-pass-4.md` reported no blocking findings and said another Opus round is not required after fixing two required follow-ups. Fixed follow-ups: the generated suite deadline now starts after release-binary build success, and Python-version validation no longer imports `tomllib` before checking the minimum interpreter version. Additional accepted polish: source digest inputs now include per-crate `Cargo.toml` files and the Sifr-only compile-error bucket is documented in code.
- Post-review validation: `jq empty verification/areas/cpython_differential/data/generated_seed_manifest.json`, `python3 -m py_compile verification/areas/cpython_differential/checks/generated_suite.py verification/areas/cpython_differential/checks/generated_programs.py verification/areas/cpython_differential/checks/catalogue_lint.py`, and `git diff --check` passed. `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite policy --suite hand_seeded_merge --suite generated_minimized_seeds --suite generated_broader` passed with `variants=4 failures=0 blocking_failures=0`.
- Create-pr validation: `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time / warm-cache hit-rate notes (`wall_time=375.95s`, `cache_hits=27/44`).
- Closeout validation: docs-only closeout `git diff --check` passed, and `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time note (`wall_time=214.51s`, `cache_hits=44/44`).

### Wave 6: CPython Differential Miscompilation Oracle

Goal: detect silent wrong-code bugs using CPython as the reference for the supported Python-compatible subset.

#### Wave 6.0: Divergence Catalogue and Hand-Seeded Merge Smoke

Implementation slice:

- Status: merged in PR [#2627](https://github.com/sifr-lang/sifr/pull/2627).
- Scope: adds `verification/policy/cpython_differential.md`, a dedicated `cpython_differential` verification area, a catalogue linter for required/duplicate/unreferenced exclusion ids, a four-case hand-seeded CPython-vs-Sifr smoke manifest, merge-profile execution for `cpython_differential:policy,hand_seeded_merge`, and coverage-matrix closure for `cpython_hand_seeded_differential`.
- Focused validation so far: `jq empty` passed for the new area manifest, hand-seeded manifest, merge profile, and coverage matrix; `python3 -m py_compile` passed for the new linter, smoke runner, area runner, and updated profile runner; `uv run --project verification --locked python -m sifr_verify areas check` passed and discovered `cpython_differential`; `uv run --project verification --locked python -m sifr_verify profiles plan --profile merge` includes `cpython_differential` with `policy` and `hand_seeded_merge`; `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite policy --suite hand_seeded_merge` passed with `variants=2 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite advisory` passed and reduced temporary rows from 18 to 17; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; and `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and an advisory-only warm wall-time budget overrun.

Tasks:

- Define the supported differential subset in `verification/policy/cpython_differential.md` before building the generator.
- The policy file must contain two enumerated tables:
  - supported constructs, with each Python construct and the exact Sifr-equivalent behavior
  - excluded divergences, with each known Sifr/CPython semantic divergence and the generator exclusion rule
- The policy file must contain an enumerated table of exit-code-stable programs, with each program's allowed exit code or codes and rationale.
- The excluded divergence table must include at least:
  - Result/Option error model versus CPython exceptions
  - ownership and borrow restrictions
  - integer overflow and fixed-width numeric policy
  - default-argument evaluation behavior
  - division and floor semantics
  - dict ordering guarantees and mutation behavior
  - string encoding and Unicode boundary behavior
  - async runtime behavior
  - narrowing and static type rejection cases
- Add a catalogue linter that rejects duplicate, missing, or unreferenced exclusion ids.
- Define the initial oracle subset precisely:
  - bounded integer range, with overflow excluded or explicitly tested
  - no floats initially unless Sifr has precise float semantics for the selected operation
  - no default-argument cases unless Sifr matches Python evaluation timing for that case
  - no dict mutation during iteration unless explicitly supported
  - no reliance on Python `repr` or display formatting for semantic comparison
  - canonical JSON-like serialization implemented in both Python and Sifr
  - no exception-message comparison
- Define CPython version and result-comparability policy:
  - supported interpreter is CPython matching the `requires-python` range in `verification/pyproject.toml`
  - oracle reports exact `sys.version`
  - generated-corpus results are not comparable across Python minor versions unless the policy explicitly permits it
- Define canonical serializer contract:
  - every oracle program prints exactly one JSON line
  - accepted value grammar is versioned
  - dict key order is canonicalized or explicitly tested as insertion-order behavior
  - Unicode normalization policy is explicit
  - integer bounds are explicit
  - recursive/container depth limits are explicit
  - stdout normalization is limited to documented line-ending handling
- Add `cpython_differential_hand_seeded_merge`, a hand-authored deterministic merge smoke suite that covers the supported subset before generated tests are promoted.

Exit criteria:

- The generator contract is reviewable before any generated test runs.
- Unsupported semantics are explicit exclusions, not post-generation skips.
- CPython differential smoke is already merge-blocking through hand-seeded fixtures before generator stability is proven.

#### Wave 6.1: Generator, Canonical Serializer, and Shrinker

Implementation slice:

- Status: merged in PR [#2629](https://github.com/sifr-lang/sifr/pull/2629).
- Scope: adds generated CPython differential seed manifesting, deterministic generation, release-binary execution, canonical v1 value validation, shrinker/minimized-failure artifact plumbing, and nightly/release profile selection for generated broader coverage without merge promotion.
- Focused validation so far: `uv run --project verification --locked python -m sifr_verify areas run --area cpython_differential --suite policy --suite hand_seeded_merge --suite generated_minimized_seeds --suite generated_broader` passed with `variants=4 failures=0 blocking_failures=0`; nightly/release profile plans include `cpython_differential:policy,hand_seeded_merge,generated_broader`; coverage matrix advisory, area discovery, runner self-test, file-size guardrail, and `git diff --check` passed.
- Review: Claude Opus review pass 4 reported no blockers and required only deadline/import-order fixes, both now applied and revalidated.

Tasks:

- Add a grammar-based valid-program generator for the subset. Initial subset must include:
  - integer, bool, string, list, tuple, and dict values already supported by Sifr
  - pure functions
  - local variables and assignment
  - if/else
  - loops over supported iterables
  - comparisons and arithmetic with defined Sifr semantics
  - deterministic stdout serialization
- Implement a shrinker/minimizer for generated failures. This is required before generated corpus promotion.
- Exclude unsupported Python behavior by construction using the divergence catalogue. Do not generate broad invalid programs and filter them after the fact.
- Build the Sifr CLI once with `cargo build --release -p sifr`, then run generated programs with `target/release/sifr run` to avoid rebuilding the compiler for every generated case.
- The suite must build the release binary at run start or validate it against a recorded build artifact hash/source digest. Mtime-only freshness checks are not sufficient.
- Run each generated program with `python3` and `target/release/sifr run`, using a per-program timeout and an overall suite timeout recorded in the suite manifest.
- Compare:
  - stdout: byte-equal after deterministic-output normalization documented in the subset contract
  - exit code: bucketed as `0` or `non-zero`, with precise integer equality only for documented exit-code-stable programs
  - error presence: `no-error`, `compile-error`, or `runtime-error`
- Do not compare error message text in the differential oracle because Sifr's Result/Option error model versus CPython exceptions is an excluded divergence.
- Store generator seeds and minimized failures.
- Promote every found divergence to `fixedbugs` after root-cause fix.
- Run the generated corpus in nightly/release.
- Promote a deterministic generated seed subset to merge only after the suite has 20 consecutive nightly green runs with no quarantine entries and no flaky retries.

Exit criteria:

- Sifr has a real semantic oracle for supported Python-like behavior.
- Silent miscompilations found by the oracle become permanent regressions.
- Unsupported semantics are documented as generator exclusions and enforced by the generator linter.
- Generated failures are minimized before becoming bug reports or regressions.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite cpython-differential-smoke
uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite cpython-differential-broader
scripts/run_all_tests.sh --profile create-pr
```

### Wave 7: Fuzz, Property, and Sanitizer Hardening

Goal: move from deterministic smoke checks to sustained compiler and runtime hardening.

#### Wave 7.1: Target Contract and Minimization Commands

Implementation slice:

- Status: merged in PR [#2631](https://github.com/sifr-lang/sifr/pull/2631).
- Scope: extends the existing `fuzz_property` manifests and runner rather than adding a parallel runner. `fuzz_smoke_manifest.json` now carries a versioned target contract for the five Wave 7 entrypoints, including corpus directories, seed files, valid/invalid/structured input classes, reproduction commands, minimization commands, and finding-promotion rules. `property_manifest.json` entries now reference those target ids and declare valid/invalid class. The runner rejects malformed target ids, duplicate/missing targets, missing seed files, missing minimization scripts, and unknown property target references before executing the deterministic suites.
- Diagnostic renderer boundary: the renderer target is documented as `structured-diagnostics` and uses `RenderedDiagnostic` JSON-shaped values or in-memory `RenderedDiagnostic` values, with the current deterministic smoke path through `cargo run -q -p sifr_driver --bin diagnostic_contract_harness`; it is explicitly separate from source/parser fuzzing.
- Minimization commands: checked-in helpers under `verification/areas/fuzz_property/checks/` normalize failing Sifr source, structured diagnostic JSON, and project-tree reproductions into `target/verification/actual/fuzz_property/minimized/`.
- Sustained-lane policy: `verification/areas/fuzz_property/sustained_lane.md` now records per-target nightly/release budgets, target ids, seed/source-hash reporting, corpus rotation, and regression-promotion rules.
- Focused validation so far: `jq empty verification/areas/fuzz_property/fuzz_smoke_manifest.json verification/areas/fuzz_property/property_manifest.json verification/areas/fuzz_property/manifest.json` passed; `python3 -m py_compile verification/runner/sifr_verify/hardening/property_and_fuzz.py verification/areas/fuzz_property/checks/minimize_seed.py verification/areas/fuzz_property/checks/minimize_diagnostic_json.py verification/areas/fuzz_property/checks/minimize_project_tree.py verification/areas/fuzz_property/runner.py` passed; `git diff --check` passed; `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke` passed with `variants=45 failures=0 blocking_failures=0`; all three minimization helpers ran against checked-in seed/diagnostic/project inputs and wrote candidates under `target/verification/actual/fuzz_property/minimized/`; `cargo run -q -p sifr_driver --bin diagnostic_contract_harness` passed; `uv run --project verification --locked python -m sifr_verify areas check` passed; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `python3 scripts/check_file_size_guardrails.py` passed; `cargo test -p sifr_runtime` passed with 29 tests; and `cargo test -p sifr_runtime --features http` passed with 36 tests.
- Review: Claude Opus review pass 1 in `plans/reviews/active/ad-hoc-world-class-verification-wave-7-1-fuzz-target-contract-review-pass-1.md` reported no blocking findings and said another Opus review round is not required before PR/create-pr. Post-review polish applied: policy text now explicitly states that fuzz-smoke currently mutates only Sifr source; sustained-lane text names the property/harness-backed targets; property entries are cross-validated against target program classes; duplicate `required_target_ids` are rejected; and project-tree minimization refuses to replace output outside the controlled minimization directory. Post-review `python3 -m py_compile`, `jq empty`, `git diff --check`, and `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke` passed again with `variants=45 failures=0 blocking_failures=0`.
- Create-pr validation: the first `scripts/run_all_tests.sh --profile create-pr` attempt stopped in diagnostics baseline hygiene because the policy used a live catch-all diagnostic code as an example; the example was replaced with a non-code registry shape placeholder, and `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed. The final `scripts/run_all_tests.sh --profile create-pr` rerun passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time / warm-cache hit-rate notes (`wall_time=379.66s`, `cache_hits=17/44`).
- Closeout validation: docs-only closeout `git diff --check` passed, and `scripts/run_all_tests.sh --profile create-pr` passed with 132/132 e2e pass fixtures, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time note (`wall_time=206.44s`, `cache_hits=44/44`).

#### Wave 7.2: Per-Target Fuzz Dispatch and Merge Smoke

Implementation slice:

- Status: merged in PR [#2633](https://github.com/sifr-lang/sifr/pull/2633).
- Scope: `fuzz-smoke` now dispatches each `fuzz_smoke_manifest.json` target independently instead of treating target entries as metadata. Source mutation targets run their own seed corpus, command, diagnostic format, exit-code policy, timeout, random seed, uniqueness budget, and deterministic rerun. `codegen_entrypoint` runs valid seeds through `sifr run`. `diagnostic_renderer_entrypoint` and `package_project_manifest_entrypoint` execute target-specific `diagnostic_contract_harness --target ... --seed ...` reproduction commands and enforce exit-code plus panic-signal checks.
- Merge profile: `verification/profiles/merge.json` now selects `fuzz_property:fuzz-smoke`, and `profile_runner.py` has an explicit `run_fuzz_property_suites` step so selected fuzz/property suites are executed by profile runs. The step skips suites already routed through `legacy_facade.hardening_suites`, avoiding nightly/release double execution while preserving their existing legacy path.
- Policy updates: `verification/policy/fuzz_property.md` and `verification/areas/fuzz_property/sustained_lane.md` now describe per-target dispatch and remove the prior caveat that non-parser targets were only reproduction/minimization routing.
- Review pass 1: Claude Opus review in `plans/reviews/active/ad-hoc-world-class-verification-wave-7-2-fuzz-dispatch-merge-smoke-review-pass-1.md` found three blockers: diagnostic/project targets shared a metadata-only harness command, nightly/release would double-run selected fuzz suites, and the phase still requires an actual merge profile wall-time run before PR. The first two blockers were fixed before review pass 2; the merge wall-time run remains required before PR submission.
- Review pass 2: Claude Opus review in `plans/reviews/active/ad-hoc-world-class-verification-wave-7-2-fuzz-dispatch-merge-smoke-review-pass-2.md` marked the per-target dispatch and double-execution blockers satisfied, said another review round is not required, and confirmed the remaining full merge profile run is sufficient before PR if it passes and records wall-time evidence. Minor pass-2 observations addressed before merge validation: timeout labels are no longer duplicated, reproduction-command subprocesses receive an explicit offline Cargo environment, and the sustained-lane doc now names the broader no-argument diagnostic harness sweep as owned by the merge-blocking developer-tooling contract suite.
- Budget disposition review: Claude Opus budget pass in `plans/reviews/active/ad-hoc-world-class-verification-wave-7-2-fuzz-dispatch-merge-smoke-budget-disposition-review-pass-3.md` confirmed the merge wall-time advisory does not block opening the PR because both merge runs exited `0`, the advisory is informational in the runner, and Wave 7.2's new deterministic fuzz smoke is about `58s` while the remaining warm overrun is dominated by pre-existing merge steps. No further Opus review round is required before PR.
- Focused validation so far: `python3 -m py_compile verification/runner/sifr_verify/hardening/property_and_fuzz.py verification/runner/sifr_verify/profile_runner.py verification/areas/fuzz_property/runner.py` passed; `jq empty verification/areas/fuzz_property/fuzz_smoke_manifest.json verification/profiles/merge.json` passed; `cargo fmt --check` passed; `git diff --check` passed; target-specific `cargo run --locked -q -p sifr_driver --bin diagnostic_contract_harness -- --target diagnostic_renderer_entrypoint --seed verification/areas/diagnostics/fixtures/diagnostics/parser_bad_indent/main.sifr` passed; target-specific `cargo run --locked -q -p sifr_driver --bin diagnostic_contract_harness -- --target package_project_manifest_entrypoint --seed verification/areas/project_workspace/fixtures/project/workspace_missing_import_canonical/main.sifr` passed; `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke` passed with `variants=37 failures=0 blocking_failures=0`; post-pass-2 `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite fuzz-smoke` passed with `variants=25 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `python3 scripts/check_file_size_guardrails.py` passed; `uv run --project verification --locked python -m sifr_verify areas check` passed; and `scripts/run_all_tests.sh --profile merge --emit-plan` shows `fuzz_property:fuzz-smoke` in the merge profile plan.
- Merge validation: cold-cache `scripts/run_all_tests.sh --profile merge` passed with full e2e `651/651`, report signature `ee5e5d44306f270c`, e2e cache hits `0/182`, `fuzz_property_checks` `variants=25 failures=0 blocking_failures=0`, hardening `variants=234 failures=0 blocking_failures=0`, and wall time `1750.45s`; the validation report marked the warm wall-time budget advisory as exceeded. Warm-cache rerun also passed with full e2e `651/651`, report signature `ee5e5d44306f270c`, e2e cache hits `182/182`, `fuzz_property_checks` elapsed `57.986s`, hardening `variants=234 failures=0 blocking_failures=0`, and wall time `1235.88s`; the report still recorded advisory-only `warm wall-time budget exceeded` and `group skew is high`, with the slowest steps being `verification_hardening_suites` `472.315s`, `generated_code_quality_checks` `236.996s`, and e2e pass `15.032s`.
- Follow-up: merge profile warm wall-time advisory exceeded (`1235.88s` vs `900s` warm; `1750.45s` vs `1500s` cold) on both warm- and cold-cache reference-host runs during Wave 7.2 validation. Wave 7.2's `fuzz_property:fuzz-smoke` step is about `58s` and is not the cause; dominant contributors are pre-existing `verification_hardening_suites` (`472.315s`) and `generated_code_quality_checks` (`236.996s`). Triage and trim outside Wave 7.2 with reproduction command `scripts/run_all_tests.sh --profile merge`; do not relax `verification/profiles/merge.json` budgets without a new measurement on the declared reference host. A separate runner improvement should also consider surfacing a cold-budget advisory when cache evidence shows a cold run exceeds `cold_wall_time_minutes`.

#### Wave 7.3: Sanitizer and Runtime Platform Hardening Lanes

Implementation slice:

- Status: merged in PR [#2635](https://github.com/sifr-lang/sifr/pull/2635).
- Scope: `verification/areas/runtime_platform/sanitizer_manifest.json` now defines merge `sanitizer-smoke` and nightly/release `sanitizer-full` cases for generated binaries, `sifr_runtime`, async/concurrency runtime fixtures, filesystem/process/network runtime fixtures, Miri, and deterministic concurrency modeling. Each case includes supported host triples, required tools/toolchains/components, timeout, exact command/environment, skip reason, and finding-promotion policy.
- Runner integration: the runtime-platform runner validates the sanitizer manifest, executes supported cases with offline Cargo and timeouts, and reports unsupported local cases as structured `skip` variants with host/toolchain/tool/component reasons. `profile_runner.py` now executes runtime-platform suites from profile data rather than hard-coding only `platform-golden`.
- Profile and matrix integration: merge selects `runtime_platform:sanitizer-smoke`; nightly and release select `runtime_platform:sanitizer-full`; the `sanitizer_hardening` coverage-matrix row is now `blocking` with reproduction command `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-smoke`.
- Review: Claude Opus review pass 1 in `plans/reviews/active/ad-hoc-world-class-verification-wave-7-3-sanitizer-platform-lanes-review-pass-1.md` reported no blocking findings, confirmed the implementation does not overclaim unsupported sanitizer execution, and said another Opus review round is not required before PR. Post-review polish added explicit skip counts to runtime-platform hardening summaries/result JSON, taught validation report parsing to retain `skip` case timings and optional `skipped=N`, and tightened sanitizer-manifest validation to reject unknown fields and non-boolean `always_skip`.
- Focused validation so far: `python3 -m py_compile verification/areas/runtime_platform/runner.py verification/runner/sifr_verify/profile_runner.py verification/runner/sifr_verify/reports.py` passed; `jq empty verification/areas/runtime_platform/manifest.json verification/areas/runtime_platform/sanitizer_manifest.json verification/profiles/merge.json verification/profiles/nightly.json verification/profiles/release.json verification/areas/coverage_matrix/compiler_surface_matrix.json` passed; `python3 scripts/check_file_size_guardrails.py` passed; `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-smoke --hardening-summary` passed with `variants=2 failures=0 blocking_failures=0 skipped=2` and structured skips for missing `llvm-symbolizer` and nightly toolchain; `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-full --hardening-summary` passed with `variants=7 failures=0 blocking_failures=0 skipped=7` and structured skips for unsupported/missing sanitizer, Miri, and model prerequisites, including the always-skip deterministic concurrency model boundary; `uv run --project verification --locked python -m sifr_verify areas check` passed; `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite advisory` passed with temporary rows reduced from `17` to `16`; `scripts/run_all_tests.sh --profile merge --emit-plan` shows `runtime_platform:sanitizer-smoke`; `scripts/run_all_tests.sh --profile nightly --emit-plan` shows `runtime_platform:sanitizer-full`; `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite platform-golden --suite sanitizer-smoke --hardening-summary` passed with `variants=14 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify --self-test` passed; `git diff --check` passed; and `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, hardening `variants=5 failures=0 blocking_failures=0`, runtime-platform suite `variants=12 failures=0 skipped=1`, and advisory-only warm wall-time / warm-cache hit-rate notes (`wall_time=363.38s`, `cache_hits=17/44`).

Tasks:

- Extend existing `verification/areas/fuzz_property/manifest.json`, `fuzz_smoke_manifest.json`, `property_manifest.json`, and `verification/policy/fuzz_property.md`; do not introduce a parallel fuzz runner unless the existing runner cannot express a required target.
- Add `cargo-fuzz` or equivalent coverage-guided fuzz targets for:
  - parse/check entrypoint
  - HIR/type/ownership entrypoint
  - codegen entrypoint
  - diagnostic renderer entrypoint
  - package/project manifest entrypoint
- The diagnostic renderer fuzz target consumes structured diagnostic values or their JSON serialization, not source code. Document the input grammar in `verification/policy/fuzz_property.md` so it does not duplicate parser fuzzing.
- Keep parser-fork fuzzing separate from Sifr-original compiler fuzzing.
- Define corpus directories, seed rotation policy, seed minimization, timeout policy, sustained-lane runtime budget, and crash promotion rules in `verification/policy/fuzz_property.md`.
- Separate invalid-program fuzzing from valid-program fuzzing. Invalid fuzzing hunts ICEs and diagnostic crashes; valid fuzzing hunts wrong-code and invariant breaks.
- Fuzz reports must include seed, minimized input path, exact reproduction command, and target id.
- Check in corpus minimization commands.
- Add a merge-blocking deterministic fuzz smoke using stable seeds and a short runtime budget.
- Add nightly/release sustained fuzz lane documentation and commands.
- Add sanitizer lanes for:
  - generated binaries where feasible
  - `sifr_runtime`
  - async/concurrency runtime cases
  - filesystem/process/network runtime cases where supported
- Prefer ASan/LSan/TSan or platform-supported equivalents. If a sanitizer is unsupported on a host, emit a structured skip with reason.
- Add a Miri lane for unsafe/runtime Rust code where feasible.
- Add Loom, Shuttle, or equivalent deterministic concurrency tests where async/thread behavior is shipped and the model can run locally.
- If Miri or Loom/Shuttle-style coverage is skipped, record the determination with reason and reproduction command in the platform or sanitizer manifest.
- Promote every sanitizer/fuzz finding into regression coverage before closure.
- Measure warm/cold merge wall time before and after adding merge fuzz/sanitizer smoke. If the merge lane exceeds the documented budget, keep only deterministic minimized reproductions in merge and move broad sanitizer/fuzz execution to nightly/release.

Exit criteria:

- Sifr-original compiler code is fuzzed, not only the inherited parser fork.
- Fuzz and sanitizer outputs have deterministic reproduction paths.
- Nightly hardening findings become merge-blocking regressions after minimization.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property
uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite fuzz-smoke
cargo test -p sifr_runtime
cargo test -p sifr_runtime --features http
scripts/run_all_tests.sh --profile create-pr
```

### Wave 8: Incremental, Determinism, and Performance Trend Evidence

Goal: prove repeated/editing workflows do not produce divergent behavior or slow drift.

Tasks:

- Add clean-vs-incremental equivalence checks for every cache/query/incremental behavior Sifr currently ships.
- If Sifr does not yet ship true incremental compilation, add an explicit non-incremental boundary test proving repeated clean builds and cache-assisted frontend workflows produce identical canonical outputs.
- Add edit-run scenario fixtures:
  - edit that preserves success
  - edit that introduces diagnostics
  - edit that fixes diagnostics
  - edit that changes project graph dependencies
- Extend performance reporting from threshold-only budgets to trend artifacts:
  - stable benchmark id
  - sample count
  - median
  - variance or noise classification
  - previous baseline comparison
- Track at least these metrics where applicable:
  - compile wall time
  - peak RSS
  - emitted Rust lines/bytes
  - generated binary size
  - diagnostic rendering time for large error cases
  - LSP initial indexing time
  - LSP steady-state edit latency
  - package resolution/install time when package management is shipped
- Record benchmark environment metadata: host CPU, OS, rustc, Python, uv, profile, thermal policy, and sample count.
- Store trend baselines under `verification/areas/performance/data/trend/`.
- Add a stale-baseline check that fails when a benchmark id has no current baseline update or explicit owner-reviewed deferral within the policy window.
- Require benchmark ids to be stable; renames must carry old-id mapping so trend history is not silently reset.
- Encode performance blocking policy in the performance area:
  - create-pr validates benchmark schema and smoke budgets
  - merge enforces stable representative budgets only
  - nightly/release produce trend deltas on reference hardware
  - trend regressions require owner review
  - local developer machines do not fail solely because of noisy trend deltas
  - checked-in trend baselines may only be updated from approved reference runs
- Keep budget failures blocking where current policy already blocks.

Exit criteria:

- Cache and repeated-build behavior has explicit equivalence evidence.
- Perf reports can identify drift, not only budget violations.
- Trend artifacts have a durable checked-in home and stale-baseline detection.
- Incremental claims are not made unless actually implemented and tested.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
bash verification/runner/e2e/check_report_determinism.sh --profile merge
bash verification/runner/e2e/check_sequential_parallel_equivalence.sh --profile merge
scripts/run_all_tests.sh --profile create-pr
```

First Wave 8 performance-trend policy slice:

- Status: merged in PR [#2637](https://github.com/sifr-lang/sifr/pull/2637).
- Scope: added executable performance trend policy validation with stale-baseline, stable benchmark-id, rename-mapping, required metric, cache, and metadata checks. The first checked-in trend baseline snapshot now lives under `verification/areas/performance/data/trend/` and covers all current benchmark ids. New benchmark captures now record host CPU, uv version, validation profile, and thermal policy in addition to the existing OS, architecture, rustc, Python, Cargo, lockfile hash, and compiler fingerprint metadata.
- Blocking policy: the performance `contracts` suite and the smoke, representative, and full performance profiles run the trend policy and self-test contracts while retaining existing budget failures as blocking. Local trend deltas remain non-blocking by policy; checked-in trend baseline updates are reserved for approved reference runs.
- Baseline lifecycle: the initial trend snapshot is derived from the existing 2026-05-16 checked-in performance baseline and uses a 45-day freshness window, so an approved reference-run refresh is required before the stale-baseline gate starts failing ordinary profile runs around 2026-07-01.
- Review: Claude Opus review pass 1 reported no blockers and suggested tightening required metric nullability, manifest-hash validation, wildcard benchmark deferrals, active-old-id rename validation, and tracker wording. Those suggestions were addressed in this slice. The targeted reread in `plans/reviews/active/ad-hoc-world-class-verification-wave-8-1-performance-trend-policy-review-pass-2.md` reported no blocking issues, no non-blocking follow-ups, and no further review required.
- Focused validation: `python3 -m py_compile verification/areas/performance/check_trend_policy.py verification/areas/performance/run_benchmarks.py verification/areas/performance/runner.py` passed; `jq empty verification/areas/performance/manifest.json verification/areas/performance/data/trend/trend_policy.json verification/areas/performance/data/trend/current.json` passed; `uv run --project verification --locked python verification/areas/performance/check_trend_policy.py --self-test` passed; `uv run --project verification --locked python verification/areas/performance/check_trend_policy.py` passed; `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite contracts --hardening-summary` passed with `variants=6 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite smoke --hardening-summary` passed with `variants=7 failures=0 blocking_failures=0`; first `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative --hardening-summary` attempt stopped on known local performance p95 noise for `build-single-file-001-break-continue`, and the immediate rerun passed with `variants=8 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas check` passed; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `bash verification/runner/e2e/check_report_determinism.sh --profile merge` passed with signature `ee5e5d44306f270c`; `bash verification/runner/e2e/check_sequential_parallel_equivalence.sh --profile merge` passed with signature `ee5e5d44306f270c`; `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, performance smoke `variants=7 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time overrun.

Second Wave 8 frontend edit-equivalence slice:

- Status: merged in PR [#2638](https://github.com/sifr-lang/sifr/pull/2638).
- Scope: added frontend query equivalence tests proving cache-assisted edit/query workflows match clean contexts for success-preserving edits, diagnostic-introducing edits, diagnostic recovery, and project-graph dependency changes. The existing performance frontend cache contract now runs this equivalence module alongside the cache invalidation and deterministic query tests.
- Review: Claude Opus review pass 1 reported no blockers, verified real cache-assisted-vs-clean comparisons, coverage of all four named edit scenarios, temp-path-free graph evidence, performance contract wiring, and guardrail compliance. The reviewer requested no follow-up and no further review round.
- Focused validation: `cargo test -p sifr_frontend --lib query_diagnostics_equivalence_tests` passed with `2 passed`; `python3 -m py_compile verification/areas/performance/check_frontend_cache_contract.py` passed; `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite frontend-syntax-guardrails --hardening-summary` passed with `variants=4 failures=0 blocking_failures=0`; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed. First `scripts/run_all_tests.sh --profile create-pr` attempt stopped before the changed frontend/performance contract on transient core-language audit fixture timeouts (`exit=124` for three smoke fixtures); focused `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite audit-fixtures --hardening-summary` immediately passed with `variants=1 failures=0 blocking_failures=0`. The second `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, frontend-syntax guardrails `variants=4 failures=0 blocking_failures=0`, performance smoke `variants=7 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time overrun.

Third Wave 8 trend-report artifact slice:

- Status: merged in PR [#2639](https://github.com/sifr-lang/sifr/pull/2639).
- Scope: benchmark executions now emit a first-class trend comparison artifact under `target/performance/trend/` (or an explicit `--trend-json-out` path) alongside the raw benchmark run report. The artifact compares current median, p95, MAD, RSS, cache hit/miss, and sample-count data against `verification/areas/performance/data/trend/current.json`, records baseline metadata, classifies deltas as within-noise, improvement, regression, or sample-count-below-baseline, and keeps local trend deltas non-blocking while marking reference-hardware review candidates.
- Review: Claude Opus review pass 1 reported no blockers and verified stable-id comparison, loud missing-baseline failures, smoke sample-count behavior, non-blocking local deltas, metric coverage, schema validation, and file-size guardrail compliance. Non-blocking suggestions to simplify report writing, expose MAD deltas, and document zero-baseline percent-delta behavior were addressed in this slice; the suggested trend-path self-test expansion is deferred until `run_benchmarks.py` is split below the file-size guardrail pressure. The reviewer requested no further review round.
- Focused validation: `python3 -m py_compile verification/areas/performance/run_benchmarks.py` passed; `uv run --project verification --locked python verification/areas/performance/run_benchmarks.py --sample-scale smoke --case incremental-local-loop-001-unchanged-file-update --trend-json-out target/performance/trend-smoke.latest.json --json-out target/performance/trend-smoke.run.json` passed and wrote a trend report with `sample_count_below_baseline` / `reference_review_required=false`; `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite smoke --hardening-summary` passed with `variants=7 failures=0 blocking_failures=0`; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, performance smoke `variants=7 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time overrun.

Fourth Wave 8 output-size trend metrics slice:

- Status: merged in PR [#2640](https://github.com/sifr-lang/sifr/pull/2640).
- Scope: split trend comparison report construction out of `run_benchmarks.py` and added build-artifact size collection for `sifr build` benchmark cases. Raw benchmark reports now populate `emitted_rust_lines`, `emitted_rust_bytes`, and `generated_binary_bytes` for build outputs; trend reports surface those optional size metrics in current/baseline snapshots and compute absolute/percent deltas when reference baselines contain comparable values.
- Review: Claude Opus review pass 1 reported no blockers and identified two non-blocking quality issues: project-build cases initially counted only `src/main.rs`, and build layout drift could silently leave size metrics null. Both were fixed by summing all emitted `src/**/*.rs` files and raising `BenchmarkError` when build outputs are missing or unreadable. Targeted Opus review pass 2 marked both concerns resolved, found no blockers, and requested no further review round.
- Focused validation so far: `python3 -m py_compile verification/areas/performance/run_benchmarks.py verification/areas/performance/trend_reports.py` passed; `uv run --project verification --locked python verification/areas/performance/run_benchmarks.py --self-test` passed; `uv run --project verification --locked python verification/areas/performance/run_benchmarks.py --sample-scale smoke --case build-project-001-additional-modules --trend-json-out target/performance/wave-8-4-project.trend.json --json-out target/performance/wave-8-4-project.run.json` passed and recorded aggregate project emitted Rust/binary metrics (`3769` lines, `126264` Rust bytes, `1874432` binary bytes); `uv run --project verification --locked python verification/areas/performance/run_benchmarks.py --sample-scale smoke --case build-single-file-001-break-continue --trend-json-out target/performance/wave-8-4-build.trend.json --json-out target/performance/wave-8-4-build.run.json` passed and recorded non-null single-file emitted Rust/binary metrics (`15` lines, `303` Rust bytes, `404640` binary bytes); `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite smoke --hardening-summary` passed with `variants=7 failures=0 blocking_failures=0`; `python3 scripts/check_file_size_guardrails.py` passed; `git diff --check` passed; `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, performance smoke `variants=7 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time overrun.

Wave 8 closeout:

- Status: implemented, reviewed, locally validated, and ready for tracker-only closeout PR.
- Review: Claude Opus full Wave 8 closeout review in `plans/reviews/active/ad-hoc-world-class-verification-wave-8-closeout-review-pass-1.md` reported no blockers, confirmed Wave 8 exit criteria are met, and stated no further Wave 8 implementation review round is required. The review classified the remaining baseline refresh as an operational follow-up rather than a Wave 8 blocker.
- Closeout validation: `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative --hardening-summary` passed with `variants=8 failures=0 blocking_failures=0`; `bash verification/runner/e2e/check_report_determinism.sh --profile merge` passed on serial rerun with signature `ee5e5d44306f270c`; `bash verification/runner/e2e/check_sequential_parallel_equivalence.sh --profile merge` passed on serial rerun with signature `ee5e5d44306f270c`; `scripts/run_all_tests.sh --profile create-pr` passed with e2e `132/132`, report signature `5edef8cd4b961ef8`, performance smoke `variants=7 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and advisory-only warm wall-time/cache-hit-rate notes. An earlier concurrent attempt at the two merge-profile report checks produced filesystem-sensitive full-e2e I/O/logging fixture interference and was discarded; the serial reruns are the authoritative evidence.
- Baseline refresh follow-up: compiler/performance owns an approved reference-hardware trend baseline refresh by 2026-06-28, ahead of the 45-day stale-baseline window for the 2026-05-16 checked-in baseline. That refresh should populate the Wave 8.4 optional size metrics in `verification/areas/performance/data/trend/current.json`.

### Wave 9: LSP, Tooling, Ecosystem, Package, Stdlib, and Platform Breadth

Goal: move smoke-level surfaces toward named corpora with profile ownership.

Wave 9 ships as numbered sub-PRs, not a single PR:

- 9.1 LSP marker corpus
- 9.2 Ecosystem-broader expansion
- 9.3 Package-manager integration suites
- 9.4 Stdlib parity per-module suites
- 9.5 Runtime/platform executable evidence
- 9.6 Algorithmic compatibility profile ownership and broader corpus

First Wave 9.1 LSP marker/capability slice:

- Status: merged in PR #2642.
- Scope: added `verification/areas/developer_tooling/data/lsp_capability_inventory.json` sourced from `crates/sifr_lsp/src/capabilities.rs`, a marker-based LSP corpus under `verification/areas/developer_tooling/lsp_marker_corpus/`, and `check_lsp_marker_corpus.py` to enforce inventory/source-token consistency, fixture marker hygiene, required behavior-category coverage, and marker-required capability coverage. The existing `lsp-smoke` suite now runs the marker corpus check and self-test alongside the protocol smoke.
- Profile ownership: create-pr already runs `developer_tooling` `static,lsp-smoke`, so this slice promotes the marker corpus coverage gate into the normal create-pr lane; `full` includes `lsp-smoke` through the developer-tooling full suite.
- Review: Claude Opus review pass 1 reported no blockers, confirmed the inventory is honestly anchored to advertised capability tokens in `crates/sifr_lsp/src/capabilities.rs`, confirmed the marker corpus is framed as stable anchors rather than full protocol replay, verified validator and `lsp-smoke` wiring, and requested no further review round. Non-blocking notes about reverse capability discovery and per-marker replay ownership are deferred to later Wave 9 transcript/replay slices.
- Focused validation so far: `python3 -m py_compile verification/areas/developer_tooling/check_lsp_marker_corpus.py verification/areas/developer_tooling/runner.py` passed; `jq empty verification/areas/developer_tooling/data/lsp_capability_inventory.json verification/areas/developer_tooling/lsp_marker_corpus/manifest.json` passed; `uv run --project verification --locked python verification/areas/developer_tooling/check_lsp_marker_corpus.py` passed; `uv run --project verification --locked python verification/areas/developer_tooling/check_lsp_marker_corpus.py --self-test` passed; `uv run --project verification --locked python -m sifr_verify areas run --area developer_tooling --suite lsp-smoke --hardening-summary` passed with `variants=4 failures=0 blocking_failures=0`; `scripts/run_all_tests.sh --profile create-pr` passed with developer-tooling `variants=16 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and e2e `132/132` with report signature `5edef8cd4b961ef8` (advisories only: warm wall-time budget exceeded and warm-cache hit rate below target).

Second Wave 9.1 LSP transcript replay slice:

- Status: merged in PR #2643.
- Scope: added a manifest-owned JSON-RPC transcript replay gate under `verification/areas/developer_tooling/lsp_transcripts/` plus `check_lsp_transcript_replay.py`, covering initialize capability snapshots, client capability combinations, unsupported capability behavior, cancellation/out-of-order request replay, stale diagnostics after edit, and watched-file project reload/workspace-folder behavior. The existing LSP protocol client now buffers early responses by id and advances manual request ids to avoid later auto-id collisions. The existing `lsp-smoke` suite now runs transcript replay and its self-test in addition to protocol smoke and marker corpus checks.
- Profile ownership: create-pr and merge run transcript replay through `developer_tooling` `lsp-smoke`; `full` inherits it through the developer-tooling full suite.
- Review: Claude Opus review pass 1 reported no blockers and requested non-blocking cleanup for response-buffer proof, server-request buffering guard, empty-category validation, manual-id collision safety, and stale-hover fragility. Those changes were implemented. Claude Opus review pass 2 reported no blockers, confirmed pass-1 concerns were resolved, and requested no further review round.
- Focused validation so far: `python3 -m py_compile verification/areas/developer_tooling/check_lsp_transcript_replay.py verification/areas/developer_tooling/lsp_protocol.py verification/areas/developer_tooling/runner.py` passed; `jq empty verification/areas/developer_tooling/lsp_transcripts/manifest.json` passed; `uv run --project verification --locked python verification/areas/developer_tooling/check_lsp_transcript_replay.py` passed; `uv run --project verification --locked python verification/areas/developer_tooling/check_lsp_transcript_replay.py --self-test` passed; `uv run --project verification --locked python -m sifr_verify areas run --area developer_tooling --suite lsp-smoke --hardening-summary` passed with `variants=6 failures=0 blocking_failures=0`; `scripts/run_all_tests.sh --profile create-pr` passed with developer-tooling `variants=18 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and e2e `132/132` with report signature `5edef8cd4b961ef8` (advisory only: warm wall-time budget exceeded).

First Wave 9.2 ecosystem entrypoint expansion slice:

- Status: merged in PR #2644.
- Scope: expanded `oss-curated` from 2 to 5 manifest entries and `ecosystem-broader` from 2 to 4 entries by adding entrypoints under existing pinned local ecosystem roots, avoiding squash-merge pin drift for brand-new project roots. Added per-entry `license` and `source_checksum_sha256` metadata, runner enforcement for SPDX license allowlist and git-tracked source checksums, checksum pass/fail variants, fixture SPDX marker files, and ecosystem limitations/policy documentation.
- Profile ownership: merge already runs `ecosystem_compatibility` `oss-curated`; nightly/release already run both `oss-curated` and `ecosystem-broader`.
- Review: Claude Opus review pass 1 reported no blockers, verified checksum/pin arithmetic and squash-merge pin durability, and requested non-blocking cleanup for git-tracked checksum walking, SPDX validation, and concrete license artifacts. Those changes were implemented. Claude Opus review pass 2 reported no blockers, confirmed the fixes, and requested no further review round. New project-root breadth and per-root variant de-duplication remain documented follow-ups.
- Focused validation so far: `python3 -m py_compile verification/runner/sifr_verify/hardening/oss_and_determinism.py verification/areas/ecosystem_compatibility/runner.py` passed; `jq empty verification/areas/ecosystem_compatibility/data/curated_manifest.json verification/areas/ecosystem_compatibility/data/ecosystem_broader_manifest.json` passed; direct runs/checks for the new pass and known-failure entrypoints passed; `uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility --hardening-summary` passed with `variants=34 failures=0 blocking_failures=0 non_blocking_failures=0`; `scripts/run_all_tests.sh --profile create-pr` passed with developer-tooling `variants=18 failures=0 blocking_failures=0`, hardening `variants=5 failures=0 blocking_failures=0`, and e2e `132/132` with report signature `5edef8cd4b961ef8` (advisory only: warm wall-time budget exceeded).

Wave 9.3 package-management offline merge smoke slice:

- Status: implemented locally; PR pending.
- Scope: added a repo-local offline registry fixture under `verification/areas/package_management/fixtures/offline_registry/`, a deterministic checked-in `Sifr.lock`, and `check_offline_package_merge_smoke.py` to enforce registry source checksums, canonical lockfile output, package graph ids/edges, workspace dependency-order determinism, and fail-closed self-tests. Added demo-corpus lockfile digest evidence for broader nightly/release integration and a package-management policy document recording the 20-nightly-green promotion rule.
- Profile ownership: `profile_runner.py` now always runs `package_management` `offline-merge-smoke` after the existing package-manager guardrails, so create-pr/merge/local profile runs get the offline registry and lockfile smoke. `merge.json` documents `offline-merge-smoke`; `nightly.json` and `release.json` document `offline-integration`, which adds the broader demo lockfile determinism signal without duplicating the merge smoke case.
- Review: Claude Opus review pass 1 reported no blockers and confirmed the required dimensions: offline registry fixture integrity, deterministic lockfile generation, package graph behavior, fail-closed self-test coverage, profile wiring, merge/nightly boundary, schema/report accounting, and file-size maintainability. Non-blocking cleanup for duplicate nightly execution, clearer log labels, clearer demo lockfile field naming, and workspace dependency-order determinism was implemented. Claude Opus review pass 2 reported no blockers, verified the cleanup, and requested no further review round.
- Focused validation so far: `python3 -m py_compile verification/areas/package_management/tools/check_offline_package_merge_smoke.py verification/areas/package_management/runner.py verification/runner/sifr_verify/profile_runner.py` passed; `jq empty verification/areas/package_management/manifest.json verification/areas/package_management/fixtures/offline_registry/registry.json verification/areas/package_management/fixtures/offline_registry/workspace/package.json verification/areas/package_management/fixtures/offline_registry/workspace/Sifr.lock verification/areas/package_management/data/offline_demo_lockfile_digests.json verification/profiles/merge.json verification/profiles/nightly.json verification/profiles/release.json` passed; direct checker normal, `--self-test`, and `--demo-corpus` runs passed; `uv run --project verification --locked python -m sifr_verify areas run --area package_management --suite offline-merge-smoke --hardening-summary` passed with `variants=2 failures=0 blocking_failures=0`; `uv run --project verification --locked python -m sifr_verify areas run --area package_management --suite offline-integration --hardening-summary` passed with `variants=1 failures=0 blocking_failures=0`; full `package_management` area passed with `variants=4 failures=0 blocking_failures=0`; merge/nightly/release profile plan emission includes the expected package-management suite assignments; `scripts/run_all_tests.sh --profile create-pr` passed with the new package offline smoke inside `core_guardrails`, hardening `variants=5 failures=0 blocking_failures=0`, and e2e `132/132` with report signature `5edef8cd4b961ef8` (advisory only: warm wall-time budget exceeded).

Tasks:

- Add `verification/areas/developer_tooling/data/lsp_capability_inventory.json`, populated from `crates/sifr_lsp` server-capability advertisement code.
- Add a marker-based LSP corpus similar in spirit to TypeScript fourslash for:
  - diagnostics
  - hover
  - definition
  - references
  - completion
  - rename/refactor only if stable
  - project reload
  - long-session edits
- Add LSP JSON-RPC transcript replay tests for wire behavior:
  - initialize request/response snapshots
  - server capability advertisement
  - client capability combinations
  - dynamic registration if supported
  - unsupported capability behavior
  - workspace folder or multi-root capability if supported
  - cancellation
  - out-of-order requests
  - stale diagnostics after edit
  - project reload
  - long-session memory/perf smoke
  - multi-root behavior if supported
- Add a marker coverage check: every documented LSP capability in `crates/sifr_lsp` must have at least one marker test for its relevant behavior category. Numeric corpus size is secondary to capability coverage.
- Add create-pr smoke, merge capability subset, and nightly/release full marker corpus profile assignments.
- Expand `ecosystem_compatibility` from two local curated projects to a larger pinned curated set with rationale, revision, owner, command, timeout, and expected classification.
- Add ecosystem license, checksum, and revision policy for pinned corpora.
- Add `ecosystem_limitations.md` documenting:
  - represented project types
  - absent project types
  - host/platform limitations
  - dependency/network limitations
  - unsupported package-manager scenarios
  - known false negatives
  - criteria for adding/removing projects
- Add a hand-authored offline package-management merge smoke immediately, covering offline registry fixture, lockfile determinism, and package graph behavior.
- Add broader package-management integration suites to nightly/release first.
- Promote broader generated or expanded package-management cases to merge only after 20 consecutive nightly green runs with no quarantine entries and no flaky retries.
- Package-management merge tests use an offline registry fixture and prove package lockfile determinism. Live registry/network checks are nightly/release signal only.
- Expand stdlib parity from inventory/audit-fixture checks into module-owned parity suites for supported namespaces.
- Inventory stdlib modules that currently ship examples or doctest-style documentation, then add validation for each inventoried supported API. If no module ships examples, record an explicit zero-example inventory row rather than silently satisfying this task.
- Add `verification/areas/runtime_platform/supported_platforms.json` with host/target support status, merge/nightly requirement, toolchain, and allowed skips.
- Convert runtime platform documentation into executable host/target evidence where feasible:
  - filesystem paths
  - path separators
  - Unicode paths
  - symlinks
  - file permissions
  - tempdir cleanup
  - line endings
  - subprocess behavior
  - subprocess exit codes
  - networking
  - signals/process control
  - locale/unicode assumptions
  - install/distribution smoke
- Networking tests use loopback only in create-pr and merge.
- Add structured skip reasons for host-specific checks.
- Promote `algorithmic_compatibility` from manifest/taxonomy signal to profile-owned evidence:
  - merge runs a representative algorithm/LeetCode subset with taxonomy rows for each included problem/category
  - nightly/release run the full corpus and taxonomy delta reports
  - each problem/category has owner, expected classification, command, timeout, and result artifact
  - live network is not required for merge

Exit criteria:

- Tooling, ecosystem, package, stdlib, and platform surfaces are no longer represented only by smoke scripts or documentation.
- Algorithmic compatibility has profile-owned commands and structured evidence.
- Each surface has profile-owned commands and evidence.
- Unsupported host/target combinations are explicit.

Focused validation:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area developer_tooling
uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility
uv run --project verification --locked python -m sifr_verify areas run --area package_management
uv run --project verification --locked python -m sifr_verify areas run --area stdlib_parity
uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform
scripts/run_all_tests.sh --profile create-pr
```

### Wave 10: Documentation, Release Evidence, and Closeout

Goal: make the new verification standard durable and auditable.

Tasks:

- Update `internal_docs/architecture.md` with the final verification architecture.
- Update `plans/roadmap.md` and relevant phase/index docs with phase status and merged PR links.
- Update `verification/README.md` with:
  - profile ownership
  - crate test membership
  - coverage matrix
  - local commands
  - baseline bless workflow
  - fuzz/sanitizer reproduction workflow
- Update `verification/policy/profile_policy.md` with final profile membership rules.
- Update `verification/policy/suite_taxonomy.md` with new suite kinds and their artifact contracts.
- Update `verification/policy/baseline_governance.md` with codegen and diagnostics baseline bless rules.
- Add a closeout checklist with links to:
  - coverage matrix report
  - merge profile report
  - diagnostics baseline report
  - codegen green run
  - CPython differential report
  - fuzz/sanitizer reports
  - platform evidence
  - performance trend artifact
- Archive exact release evidence with commit SHA, toolchains, OS, emitted profile plan, suite counts, and report hashes.
- Run the authoritative local validation gate.
- Promote the coverage-matrix check and local/CI plan-equivalence check from advisory to blocking and require zero `expected-missing` rows.
- Add negative self-tests proving enforcement fails on:
  - stable guarantee with no matrix row
  - owner `unassigned`
  - unknown owner id
  - expired `expected-missing`
  - expired `tests:none`
  - lingering `red-blocker`
  - illegal `not-applicable` on a stable compiler/runtime behavior
  - stale/unused baseline
  - fixture missing required baseline
  - first-party crate with tests but no profile membership
  - zero-test crate without seed tests or allowed status
  - live-network suite in create-pr/merge
  - undocumented or expired quarantine
  - v1 manifest for a shipped stable surface
  - unpinned corpus revision/checksum for a merge suite
  - create-pr or merge Cargo command missing `--locked` or offline execution
  - CI plan omitting a local merge suite
  - profile assignment table mismatch
- Verify no `red-blocker`, `expected-missing`, expired `tests:none`, illegal `not-applicable`, or undocumented quarantine row remains.
- Add and run `verification/areas/coverage_matrix/checks/profile_assignment_matrix.py`, which verifies the decisions-table profile assignment against `verification/profiles/create-pr.json`, `verification/profiles/merge.json`, `verification/profiles/nightly.json`, and `verification/profiles/release.json`. Promotion to blocking happens with the coverage-matrix promotion.

Exit criteria:

- A new contributor can understand what verification is required before PRs.
- The phase has durable merged PR links and local evidence.
- The world-class standard is enforced by commands, not only described.

Focused validation:

```bash
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh
scripts/run_all_tests.sh --profile nightly
scripts/run_all_tests.sh --profile release
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
```

## Required Tracking Updates Per Wave

After each merged PR:

- Update this issue with:
  - PR link
  - measured validation commands
  - changed profile memberships
  - any accepted temporary exceptions and expiry dates
- For gate-expanding waves, record warm/cold merge wall time before and after the change. Gate-expanding waves include at least Wave 1, Wave 2.final, Wave 3, Wave 4, Wave 7, and Wave 9.3 package-management merge promotion. Wave 2.0 and Wave 2.1..2.N do not require wall-time measurement unless they independently change merge profile membership.
- If a gate-expanding wave would push merge over `warm_wall_time_minutes` or `cold_wall_time_minutes`, the same PR must ship deterministic sharding, bounded profile parallelism, or a documented move of broad non-merge coverage to nightly while preserving merge smoke.
- Update `plans/roadmap.md` if the wave changes project status.
- Update `plans/phases/index.md` if the phase list or status changes.
- Update relevant policy docs in `verification/policy/`.
- Promote any found bug into `verification/areas/regression/fixedbugs` or `crashes`.

## Acceptance Criteria

This phase is complete only when all of the following are true:

- `cargo test -p sifr_codegen` passes and is merge-blocking.
- `sifr_type_system`, `sifr_format`, `sifr_lint`, and `sifr_source` crate tests are merge-blocking.
- `sifr_ir` has meaningful seed tests.
- First-party compiler crate test membership is explicit and profile-owned.
- Stable shipped guarantees are recorded in `shipped_guarantees.json` with non-`unassigned` owners and support status.
- The verification target matrix is reflected by machine-checked inventories: every matrix target phrased with "every" has an inventory-backed check that fails when an inventory entry lacks the required evidence.
- The coverage matrix exists, is blocking, and has zero `expected-missing` rows.
- No `red-blocker`, expired `tests:none`, illegal `not-applicable`, ownerless quarantine, or undocumented quarantine row remains.
- Create-pr and merge profiles are hermetic/offline, and live-network checks are nightly/release only.
- Local/CI plan-equivalence checking exists so CI cannot silently omit a local merge check.
- Cargo feature, bin, example, doctest, all-targets, all-features, and no-default-features policies are enforced or explicitly marked unsupported.
- Cargo create-pr and merge commands obey the hermetic Cargo contract: locked dependencies, offline execution, documented setup cache/vendor behavior, and no hidden user-global cache dependence.
- Merge runs the full semantic e2e pass corpus or a documented deterministic full-corpus shard plan.
- Merge runs the full fail corpus code/position checks.
- Lexer/token stream and indentation have stable matrix coverage independent of parser acceptance/rejection.
- Parser acceptance/rejection has stable syntax matrix coverage independent of parser fuzzing.
- Merge: every active `SIFR-*` code with a stable user-facing message has at least one rendered baseline.
- Nightly and release: every active `SIFR-*` code with a stable user-facing message has rendered baselines for every stable renderer: human, compact, and JSON.
- The diagnostics baseline coverage check enforces both rules and fails on undocumented gaps.
- Baseline-backed suites fail unused/stale baselines and unchecked blesses.
- Multi-error diagnostic recovery is baseline-tested.
- HIR/CFG/lowering/codegen snapshot suites exist or are explicitly mapped to equivalent blocking suites.
- Generated Rust toolchain support is recorded and stable generated output is checked against the supported toolchain/MSRV policy.
- Compiler crash/ICE behavior is a first-class contract with issue-linked sentinels for known crashes.
- The CPython divergence catalogue exists and the generator lints against it.
- The CPython oracle records supported Python version policy, canonical serializer grammar, integer/container bounds, Unicode policy, and build artifact hash/source digest validation.
- CPython differential hand-seeded smoke is merge-blocking for the supported subset.
- Broader CPython differential generation runs in nightly or release.
- CPython generated failures are minimized before promotion to issues or regressions.
- Sifr-original compiler fuzz targets exist with deterministic merge smoke and sustained lane documentation.
- Sanitizer/leak/thread hardening lanes exist where platform-supported, with structured skip reasons otherwise.
- Execution sandbox policy is enforced for generated binaries, CPython differential programs, package tests, ecosystem projects, and fuzz reproducers.
- Clean-vs-repeated or clean-vs-incremental equivalence is tested according to shipped compiler behavior.
- LSP marker corpus covers core IDE behaviors beyond protocol smoke, and JSON-RPC transcript replay covers wire behavior.
- Ecosystem, package, stdlib, and platform suites have profile-owned commands and structured evidence.
- Algorithmic compatibility taxonomy and corpus have profile-owned commands and structured evidence.
- Platform support is recorded in `supported_platforms.json` and executable evidence respects host skip policy.
- Distribution/release install smoke and release evidence archive exist with commit, toolchains, OS, suite counts, emitted profile plan, and report hashes.
- Performance trend artifacts include time, memory/RSS, output size, and benchmark environment metadata in addition to threshold budgets.
- Performance trend blocking policy prevents noisy local trend deltas from failing ordinary developer merge runs while keeping stable representative budgets blocking.
- Every gate-expanding wave records measured warm/cold merge wall time before and after the change.
- Profile assignment in the decisions table is reflected by `verification/profiles/create-pr.json`, `verification/profiles/merge.json`, `verification/profiles/nightly.json`, and `verification/profiles/release.json`.
- Every temporary exception has an owner, issue link, expiry, reproduction command, and profile-visible status.
- `scripts/run_all_tests.sh --profile create-pr`, `scripts/run_all_tests.sh`, `scripts/run_all_tests.sh --profile nightly`, and `scripts/run_all_tests.sh --profile release` pass locally before the closeout PR.
- Negative self-tests cover every enforcement claim listed in Wave 10.

## Non-Acceptable Closeout States

- "Policy exists" but no failing check enforces it.
- A stable shipped guarantee lacks an owner, support status, or coverage row.
- Create-pr or merge requires network access.
- CI has behavior that cannot be reproduced through local profile commands.
- `sifr_codegen` remains red or excluded from merge.
- Any `red-blocker` row remains at phase close.
- Failing tests are ignored without issue-linked sentinel coverage.
- Merge relies on an undocumented subset for semantics.
- Diagnostic codes are checked but renderer output remains unbaselined.
- Fuzzing only covers inherited parser code.
- Sanitizer lanes are documented but not executable.
- LSP remains limited to handshake/protocol smoke.
- Platform matrix is documentation-only.
- Performance only reports wall-clock budgets with no memory/size trend artifact.
- Temporary exceptions have no expiry.
- The coverage matrix remains advisory at phase close.
- Any `expected-missing` row remains at phase close.

## Decisions Log

| date | decision | rationale | owner |
| --- | --- | --- | --- |
| 2026-06-14 | Keep this as a new ad-hoc issue phase rather than editing completed Phase 29. | Phase 29 created the verification foundation; this phase turns it into enforced breadth and gate closure. | compiler-verification |
| 2026-06-14 | Coverage matrix lands advisory first and becomes blocking at closeout. | Immediate blocking would require a large temporary exception list before the phase has filled the surfaces. | compiler-verification |
| 2026-06-14 | CPython differential work requires a divergence catalogue before generator implementation. | The oracle must avoid unsupported semantic drift by construction, not by filtering failures after generation. | compiler-verification |
| 2026-06-14 | External reviewer additions incorporated. | The phase now enforces shipped guarantee registry, hermetic local-first profiles, red-blocker status, stale baseline detection, feature/target coverage, generated Rust toolchain gates, crash/ICE contract, CPython hand-seeded smoke and shrinker, LSP transcripts, memory/size trends, and platform support manifest. | compiler-verification |

## Review Log

- 2026-06-14: Claude Opus review pass 1 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-review-pass-1.md`; actionable findings incorporated with locally verified crate-count corrections.
- 2026-06-14: Claude Opus review pass 2 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-review-pass-2.md`; verdict was "ready after minor edits"; required text-level edits incorporated.
- 2026-06-14: Claude Opus review pass 3 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-review-pass-3.md`; verdict was "ready after minor edits"; final minor edits incorporated.
- 2026-06-14: Claude Opus review pass 4 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-review-pass-4.md`; verdict was "ready after minor edits"; final decision-precision edits incorporated.
- 2026-06-14: Claude Opus review pass 5 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-review-pass-5.md`; verdict was "implementation-ready"; optional polish edits incorporated.
- 2026-06-14: Claude Opus target-matrix review pass 1 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-target-matrix-review-pass-1.md`; required target rows and inventory paths incorporated.
- 2026-06-14: Claude Opus target-matrix review pass 2 completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-target-matrix-review-pass-2.md`; verdict was "implementation-ready"; optional inventory-path polish incorporated.
- 2026-06-14: External precision review from `/Users/yaseralnajjar/.codex/attachments/af3521e9-f255-4f15-920e-a5a7a1d73a4b/pasted-text.txt` incorporated; status semantics, red-blocker execution semantics, offline Cargo/toolchain contracts, CPython oracle policy, lexer/indentation coverage, sandboxing, performance blocking, and negative self-tests tightened.
- 2026-06-14: Claude Opus final precision review completed in `plans/reviews/active/ad-hoc-world-class-verification-standard-and-gate-closure-final-precision-review.md`; verdict was "implementation-ready"; non-blocking clarity edits incorporated.
- 2026-06-14: External final consistency review incorporated from user-provided review text; stable guarantee wording, post-Wave 0 `expected-missing` semantics, owner registry validation, package-management merge promotion, hermetic focused validation, canonical profile-assignment checker path, and closeout negative self-tests tightened.

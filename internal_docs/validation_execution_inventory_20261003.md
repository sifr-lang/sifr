# Validation execution inventory — 2026-10-03

This is the implementation-entry snapshot for the
[active validation plan](../plans/issues/active/ad-hoc-validation-contracts-and-resource-aware-execution.md).
Canonical profiles, area manifests, and guards remain authoritative; this report
is an observation, not a second inventory or a validation receipt.

## Source and execution context

- Base: `da57229746b0793577d257b29f1382baf60b37b0` (main at entry).
- Owned branch: `codex/validation-contracts-resource-aware-20261003`.
- Host: Debian 13, x86_64 Linux; CPU affinity exposes five CPUs, cgroup quota is
  `400000/100000` (four CPU-equivalents), memory limit is 16 GiB.
- Root filesystem reports approximately 30 GiB available at entry. `/tmp` and
  `/dev/shm` each advertise 8.8 GiB but consume the same memory allowance.
- Installed the project-pinned Rust 1.98.1 (Clippy/rustfmt), uv 0.12.10 and
  GIL-enabled CPython 3.14.7 in session-owned tooling. Initialized all exact
  root/recursive submodule gitlinks; no submodule update is proposed.
- Full E2E pass inventory: 729 `.sifr` fixtures. `create-pr` selects 143.
- `uv run --project verification --locked python -m sifr_verify profiles check`
  passes with the pinned tools. This checks profile consistency, not execution.
- Raw profile selections, recursive submodule identities, and pre-change ruleset
  response are retained in `/workspace/validation-work/evidence/`.

## Delivery enforcement and external dependencies

Authenticated GitHub read access confirms active main ruleset `16287003` enforces
PRs, deletion protection and non-fast-forward protection, but has no required CI
status check. The task actor has admin/push access. Do not activate a new required
aggregate before its trusted workflow exists on the default branch.

Current `local-first-validation.yml` admits PR, main push and manual events. PR
runs `create-pr`; main/manual selects `create-pr`, `merge`, and `release` together.
There is no scheduled nightly or `merge_group` trigger in that workflow. Platform
component jobs cover Linux x86_64/ARM64, macOS ARM64, Windows x86_64 and SQL WASI;
component support does not establish native release packaging support.

PR #4259 remains open and draft at
`f88973102ac0b7b0b5c07969a57fcc4ed0e87098`; its cloud profile dynamically inherits
merge, separates functional and paired-performance outcomes, and retains the
shared-cloud v2 policy. Full correctness, fresh endpoint preparation and complete
v2 capture/check, exact-SHA Opus review and SQL phase acceptance remain pending.
The v1 failure remains unqualified. Issue #4278 remains open with no implementation.
It owns the sysroot build-graph lifetime prerequisite. The previous transfer
bundle is absent in this VM. Reconcile candidate code as a dependency in an owned
branch; do not mutate or approve the externally owned candidate branch.

## Existing mechanisms and gaps

| Requirement | Existing authority to extend | Gap to close |
|---|---|---|
| Profile contracts and case inventory | `verification/profiles`, area manifests, coverage-matrix readiness, `profiles.py` | Explicit per-stage claims/environments/resources and cloud parity enforcement |
| Evidence identity and input closure | `fixture_inventory.py`, native-test receipts, release-evidence and distribution governance | Durable runtime/producer-bound correctness reuse and complete final accounting |
| Resource scheduling and recovery | `cargo_setup.py`, `process_execution.py`, step budgets; #4278 | Effective cgroups/tmpfs admission, preparation deadlines, graph lifetimes, owned cache retirement, infrastructure classes |
| Change-aware PR and merge delivery | Profile selection, workflow checker and workflow | Conservative executable classifier, selected reasons, candidate-bound trusted aggregate, branch rules, main-push reuse |
| Nightly and fuzzing | Nightly profile, fuzz/property and hardening manifests | Scheduled explicit-commit campaigns, durable seeds/reproductions and owned defects |
| Performance | Performance manifests/reference admission and unmerged #4259 | Preserve paired policy; explicit levels, generated-program metrics and independent outcomes |
| Artifact custody and upgrades | Distribution release governance, artifact hashes, source/platform identities | Audit exact-byte promotion; actual published-predecessor transitions |
| Compatibility, support and security | Release/platform matrices, cache/process/ABI and host-tool guards | Consolidate existing owners, version ranges/directions and executable risk-triggered coverage |
| Test economics | Lane/case timings, cache reports and budgets | Separate preparation/assertion costs and peak resources; prospective cadence decisions |

## Current profile selection snapshot

The area/suite IDs below are derived directly from the canonical profile JSON at
entry. SQL `create-pr` and `merge` each select the same 19 suites, including
`build-qualification`; labels alone will not make PR validation fast. Actual cold
preparation costs have not been measured in this VM. Previous VM timings in the
source brief remain historical observations only.

### `create-pr`

Guardrails: `hir-maintainability`, `file-size`, `demo-emitted-freshness`, `source-crate-dependency-direction`, `submodule-ownership`, `sysroot-resource-certification`, `stdlib-native-intrinsic-allowlist`, `stdlib-native-adapter-reachability`, `stdlib-manifest-schema`, `stdlib-bootstrap-ordering`, `driver-maintainability`, `verification-hardening-self-test`, `verification-runner-foundation`, `method-dispatch-authority`, `unsafe-abi-contracts`.

Toolchain: `cargo-test-sifr-smoke`, `e2e-pass`.

- `rust_interop`: `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, `stable-candidate`.
- `coverage_matrix`: `readiness`.
- `diagnostics`: `rules`.
- `python_interop`: `self-test`, `scaffold`, `env`, `dependency-versions`, `minor-train-features`, `crypto-abi-features`, `redis-service-features`, `numeric-dataframe-features`, `readonly-check-doctor`, `binding-authoring`, `lsp-declaration-authoring`, `tier1`, `callbacks`, `callback-examples`, `dataframes`, `buffer-examples`, `arrow-examples`, `dlpack-examples`, `arrow-runtime`, `dlpack-runtime`, `buffer-runtime`, `async-declaration-examples`, `async-context-examples`, `cloud-boto3`.
- `runtime_platform`: `platform-golden`, `platform-support-matrix`, `platform-evidence`.
- `algorithmic_compatibility`: `profile-manifest`.
- `developer_tooling`: `static`, `lsp-smoke`, `typescript-go-transfer`, `diagnostic-rules`, `architecture-policy`.
- `generated_code_quality`: `smoke`.
- `performance`: `smoke`, `frontend-syntax-guardrails`, `lsp-workspace-cache`.
- `stdlib_parity`: `module-merge-check`, `audit-fixtures`, `complexity-resource`, `module-inventory`.
- `core_language`: `audit-fixtures`.
- `project_workspace`: `audit-fixtures`.
- `package_management`: `guardrails`, `offline-merge-smoke`.
- `sql_platform`: `build-qualification`, `compiler-components`, `common-sql`, `contracts`, `dependency-baseline`, `host-tools`, `integrated-qualification`, `migration-engine`, `mysql-provider`, `postgresql-compiler`, `postgresql-migrations`, `postgresql-runtime`, `incremental-editor`, `query-fragments`, `schema-polymorphism`, `schema-profiles`, `schema-tools`, `sqlite-provider`, `mutation`.
- `sysroot_release`: `metadata-structural`.

### `merge`

Guardrails: `hir-maintainability`, `file-size`, `demo-emitted-freshness`, `source-crate-dependency-direction`, `submodule-ownership`, `sysroot-resource-certification`, `stdlib-native-intrinsic-allowlist`, `stdlib-native-adapter-reachability`, `stdlib-manifest-schema`, `stdlib-bootstrap-ordering`, `driver-maintainability`, `verification-hardening-self-test`, `verification-runner-foundation`, `method-dispatch-authority`, `unsafe-abi-contracts`.

Toolchain: `cargo-test-sifr-full`, `e2e-pass`.

- `rust_interop`: `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, `stable-candidate`.
- `coverage_matrix`: `readiness`.
- `core_language`: `integer_dtype_rules`, `hir_analysis_behaviors`, `cfg_flow_behaviors`, `syntax_parser_lexer_matrix`, `audit-fixtures`.
- `cpython_differential`: `policy`, `hand_seeded_merge`.
- `python_interop`: `self-test`, `scaffold`, `env`, `dependency-versions`, `minor-train-features`, `crypto-abi-features`, `redis-service-features`, `numeric-dataframe-features`, `readonly-check-doctor`, `binding-authoring`, `lsp-declaration-authoring`, `tier1`, `tier2`, `tier3`, `tier4`, `callbacks`, `callback-examples`, `dataframes`, `dataframe-examples`, `buffer-examples`, `arrow-examples`, `dlpack-examples`, `arrow-runtime`, `dlpack-runtime`, `buffer-runtime`, `ml`, `libraries`, `async-declaration-examples`, `async-context-examples`, `cloud-boto3`.
- `diagnostics`: `rules`, `baselines`.
- `runtime_platform`: `platform-golden`, `platform-support-matrix`, `platform-evidence`, `sanitizer-smoke`.
- `algorithmic_compatibility`: `representative-subset`.
- `developer_tooling`: `static`, `formatter`, `analysis`, `lsp-smoke`, `typescript-go-transfer`, `diagnostic-rules`, `architecture-policy`.
- `generated_code_quality`: `representative`.
- `performance`: `representative`, `frontend-syntax-guardrails`, `lsp-workspace-cache`.
- `distribution_release`: `representative`, `qualification`, `incident-governance`, `epoch-bootstrap`, `protected-drill`, `stable-prepare`, `stable-publish-primitives`, `stable-publication`.
- `sysroot_release`: `host-installed-smoke`, `boundary-equivalence`, `metadata-structural`, `metadata-corpus`.
- `project_workspace`: `frontend_mode_parity`, `project_graph_isolation`, `baselines`, `audit-fixtures`.
- `package_management`: `offline-merge-smoke`, `guardrails`.
- `stdlib_parity`: `module-merge-check`, `audit-fixtures`, `complexity-resource`, `module-inventory`.
- `regression`: `fixedbugs`, `crashes`.
- `fuzz_property`: `fuzz-smoke`.
- `ecosystem_compatibility`: `oss-curated`.
- `sql_platform`: `build-qualification`, `compiler-components`, `common-sql`, `contracts`, `dependency-baseline`, `host-tools`, `integrated-qualification`, `migration-engine`, `mysql-provider`, `postgresql-compiler`, `postgresql-migrations`, `postgresql-runtime`, `incremental-editor`, `query-fragments`, `schema-polymorphism`, `schema-profiles`, `schema-tools`, `sqlite-provider`, `mutation`.

### `nightly`

Guardrails: `hir-maintainability`, `file-size`, `demo-emitted-freshness`, `source-crate-dependency-direction`, `submodule-ownership`, `sysroot-resource-certification`, `stdlib-native-intrinsic-allowlist`, `stdlib-native-adapter-reachability`, `stdlib-manifest-schema`, `stdlib-bootstrap-ordering`, `driver-maintainability`, `verification-hardening-self-test`, `verification-runner-foundation`, `hardening-determinism-scale`, `method-dispatch-authority`, `unsafe-abi-contracts`.

Toolchain: `cargo-test-sifr-full`, `e2e-pass`.

- `rust_interop`: `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, `stable-candidate`.
- `coverage_matrix`: `readiness`.
- `core_language`: `integer_dtype_rules`, `hir_analysis_behaviors`, `cfg_flow_behaviors`, `syntax_parser_lexer_matrix`, `audit-fixtures`.
- `diagnostics`: `rules`, `baselines`.
- `cpython_differential`: `policy`, `hand_seeded_merge`, `generated_broader`.
- `python_interop`: `self-test`, `scaffold`, `env`, `dependency-versions`, `minor-train-features`, `crypto-abi-features`, `redis-service-features`, `numeric-dataframe-features`, `readonly-check-doctor`, `binding-authoring`, `lsp-declaration-authoring`, `tier1`, `tier2`, `tier3`, `tier4`, `callbacks`, `callback-examples`, `dataframes`, `dataframe-examples`, `buffer-examples`, `arrow-examples`, `dlpack-examples`, `arrow-runtime`, `dlpack-runtime`, `buffer-runtime`, `ml`, `libraries`, `async-declaration-examples`, `async-context-examples`, `cloud-boto3`.
- `runtime_platform`: `platform-golden`, `platform-support-matrix`, `platform-evidence`, `sanitizer-full`.
- `algorithmic_compatibility`: `leetcode-full`, `taxonomy-smoke`.
- `developer_tooling`: `full`, `typescript-go-transfer`, `diagnostic-rules`, `architecture-policy`.
- `generated_code_quality`: `full`.
- `performance`: `full`, `frontend-syntax-guardrails`.
- `distribution_release`: `full`, `qualification`, `incident-governance`, `epoch-bootstrap`, `protected-drill`, `stable-prepare`, `stable-publish-primitives`, `stable-publication`.
- `sysroot_release`: `host-installed-smoke`, `host-installed-stdlib-heavy`, `metadata-structural`, `metadata-corpus`.
- `project_workspace`: `frontend_mode_parity`, `project_graph_isolation`, `baselines`, `audit-fixtures`.
- `package_management`: `offline-integration`, `guardrails`, `offline-merge-smoke`.
- `stdlib_parity`: `module-merge-check`, `module-full-check`, `audit-fixtures`, `complexity-resource`, `module-inventory`.
- `regression`: `fixedbugs`, `crashes`.
- `fuzz_property`: `property`, `fuzz-smoke`.
- `ecosystem_compatibility`: `oss-curated`, `ecosystem-broader`.
- `sql_platform`: `build-qualification`, `compiler-components`, `common-sql`, `contracts`, `dependency-baseline`, `host-tools`, `integrated-qualification`, `migration-engine`, `mysql-provider`, `postgresql-compiler`, `postgresql-migrations`, `postgresql-runtime`, `incremental-editor`, `query-fragments`, `schema-polymorphism`, `schema-profiles`, `schema-tools`, `sqlite-provider`, `mutation`.

### `python-interop-live`

Guardrails: .

Toolchain: none.

- `python_interop`: `live-policy`, `live-examples`.

### `release`

Guardrails: `hir-maintainability`, `file-size`, `demo-emitted-freshness`, `source-crate-dependency-direction`, `submodule-ownership`, `sysroot-resource-certification`, `stdlib-native-intrinsic-allowlist`, `stdlib-native-adapter-reachability`, `stdlib-manifest-schema`, `stdlib-bootstrap-ordering`, `driver-maintainability`, `verification-hardening-self-test`, `verification-runner-foundation`, `hardening-determinism-scale`, `method-dispatch-authority`, `unsafe-abi-contracts`.

Toolchain: `cargo-test-sifr-full`, `e2e-pass`, `e2e-report-determinism`, `e2e-sequential-parallel-equivalence`.

- `rust_interop`: `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts`, `stable-candidate`.
- `coverage_matrix`: `readiness`.
- `core_language`: `integer_dtype_rules`, `hir_analysis_behaviors`, `cfg_flow_behaviors`, `syntax_parser_lexer_matrix`, `audit-fixtures`.
- `diagnostics`: `rules`, `baselines`.
- `cpython_differential`: `policy`, `hand_seeded_merge`, `generated_broader`.
- `python_interop`: `self-test`, `scaffold`, `env`, `dependency-versions`, `minor-train-features`, `crypto-abi-features`, `redis-service-features`, `numeric-dataframe-features`, `readonly-check-doctor`, `binding-authoring`, `lsp-declaration-authoring`, `tier1`, `tier2`, `tier3`, `tier4`, `callbacks`, `callback-examples`, `dataframes`, `dataframe-examples`, `buffer-examples`, `arrow-examples`, `dlpack-examples`, `arrow-runtime`, `dlpack-runtime`, `buffer-runtime`, `ml`, `libraries`, `async-declaration-examples`, `async-context-examples`, `cloud-boto3`.
- `runtime_platform`: `platform-golden`, `platform-support-matrix`, `platform-evidence`, `sanitizer-full`.
- `algorithmic_compatibility`: `leetcode-full`, `taxonomy-smoke`.
- `developer_tooling`: `full`, `typescript-go-transfer`, `diagnostic-rules`, `architecture-policy`.
- `generated_code_quality`: `full`.
- `performance`: `full`, `frontend-syntax-guardrails`.
- `distribution_release`: `full`, `qualification`, `evidence-custody`, `incident-governance`, `epoch-bootstrap`, `protected-drill`, `stable-prepare`, `stable-publish-primitives`, `stable-publication`.
- `documentation`: `structure`, `ga-release`.
- `sysroot_release`: `host-installed-smoke`, `host-installed-stdlib-heavy`, `metadata-structural`, `metadata-corpus`.
- `project_workspace`: `frontend_mode_parity`, `project_graph_isolation`, `baselines`, `audit-fixtures`.
- `package_management`: `offline-integration`, `guardrails`, `offline-merge-smoke`.
- `stdlib_parity`: `module-merge-check`, `module-full-check`, `audit-fixtures`, `complexity-resource`, `module-inventory`.
- `regression`: `fixedbugs`, `crashes`.
- `fuzz_property`: `property`, `fuzz-smoke`.
- `ecosystem_compatibility`: `oss-curated`, `ecosystem-broader`.
- `sql_platform`: `build-qualification`, `compiler-components`, `common-sql`, `contracts`, `dependency-baseline`, `host-tools`, `integrated-qualification`, `migration-engine`, `mysql-provider`, `postgresql-compiler`, `postgresql-migrations`, `postgresql-runtime`, `incremental-editor`, `query-fragments`, `schema-polymorphism`, `schema-profiles`, `schema-tools`, `sqlite-provider`, `mutation`.


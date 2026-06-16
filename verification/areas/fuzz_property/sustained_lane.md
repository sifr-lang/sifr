# Sustained Fuzzing Lane (Non-blocking)

Purpose:
- run broader/longer fuzz workloads outside local blocking flow
- produce compatibility and crash signal
- feed issue triage and regression corpus updates

Status contract:
- non-blocking for merge decisions in phase 29
- outputs are informational and backlog-oriented

Operational note:
- use the same seed corpus foundations as smoke gates, but larger iteration/time budgets
- every actionable finding must follow the triage/minimization workflow in `verification/policy/fuzz_property.md`
- target ids are the same ids enforced by `fuzz_smoke_manifest.json`:
  - `parse_check_entrypoint`
  - `hir_type_ownership_entrypoint`
  - `codegen_entrypoint`
  - `diagnostic_renderer_entrypoint`
  - `package_project_manifest_entrypoint`
- nightly default budget: 10 minutes per target on the reference host, with target id, seed/source hash, and reproduction command recorded for every crash or panic signal
- release default budget: 30 minutes per target on the reference host, with minimized corpus rotation before promotion
- corpus rotation policy: add minimized findings first, prune duplicate-equivalent seeds in the same PR, and record removed seed ids in the PR body
- broad fuzz findings become merge-blocking only after minimization and promotion into `verification/areas/regression/fixtures/crashes/` or `verification/areas/regression/data/fixedbugs.json`
- merge smoke dispatches each target id independently. `parse_check_entrypoint` and `hir_type_ownership_entrypoint` run deterministic source mutations, `codegen_entrypoint` runs valid source seeds through the generated-binary path, and `diagnostic_renderer_entrypoint` plus `package_project_manifest_entrypoint` execute their declared diagnostic-contract harness command.
- merge fuzz-smoke intentionally executes one declared seed per harness-backed target. The broader no-argument `diagnostic_contract_harness` fixture sweep remains owned by the merge-blocking developer-tooling diagnostic contract suite.

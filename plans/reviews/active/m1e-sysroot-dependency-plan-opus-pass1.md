# M1e Sysroot Dependency Plan - Opus Review Pass 1

Command:

```bash
claude --dangerously-skip-permissions --setting-sources project --model claude-opus-4-7 --effort xhigh -p "Review the current uncommitted changes for milestone M1e of plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md. Scope: SysrootDependencyPlan now owns the sorted dependency inputs; binary build cache keys, test runner cache keys, and build sysroot reports should consume the plan object instead of recomputing sysroot feature/module identity from raw codegen sets. Please inspect the diff and relevant call paths. Pay special attention to: (1) any stale recomputation of sysroot stdlib modules/features downstream of try_generate_sysroot_dependency_plan, (2) whether the cache key changes preserve invalidation specificity, (3) whether BuildSysrootReport now represents the dependency plan accurately without output churn, (4) tests and edge cases. Focused validation already run: cargo fmt --check; cargo test -p sifr_stdlib_manifest sysroot_dependency_plan_captures_identity_features_and_vendor_mode; cargo test -p sifr_driver binary_project_cache_key; cargo test -p sifr_driver test_runner_cache_key_uses_sysroot_dependency_plan_inputs; git diff --check; python3 scripts/check_file_size_guardrails.py. Return findings with severity and concrete file/line references. End with one of READY or BLOCKED."
```

Verdict: READY, with recommendations addressed before PR.

Notes:

- Non-blocking: feature ordering in dependency input fingerprints is now
  `StdlibFeature` enum order rather than prior lexicographic id order. This is
  deterministic and specificity-preserving, but it can flush existing caches.
- Recommendation: avoid leaving `BuildSysrootReport::new()` as a path that
  produces empty dependency identity. Addressed by removing `new()` and making
  the CLI build-output test construct reports via `from_dependency_plan`.
- Recommendation: make the new report dependency identity accessors consumed.
  Addressed by asserting `dependency_inputs()` and `dependency_fingerprint()`
  in `crates/sifr/src/build_output.rs`.
- Recommendation: add an M1e phase-doc bullet before PR. Addressed in
  `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`.

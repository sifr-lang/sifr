Based on my review of the complete diff:

**Embedding independent of declaration reachability** — `apply_package_python_bridge_metadata` iterates the full graph's `packages` and every module inside each, and `resolve_python_bridge_graph` walks runtime-scope deps (filtered by `dependency_kind.is_none()`). Unit test now adds an "unused" module and asserts it flows through to `embedded_bridge_sources`; the per-module cache key (`python.bridge_module=... :<source_digest>`) covers its bytes. Source bytes are digest-verified against the inventoried digest in `resolve_package` before embedding.

**Fingerprint coverage** — Probe now carries `distributions: Vec<PythonDistributionProbe>` (canonical name+version, sorted casefold-stable), serialized in `CanonicalPythonEnvironmentProbe`. `digest_python_environment_probe` is threaded into (a) `PackageBuildCacheInputs.python_probe_digest`, (b) driver `GeneratedBinaryProject.cache_key_fragment` via `push_cache_key_fragment("python-runtime", metadata.probe_digest())`, and (c) rendered into generated `main.rs` — all three participate in `binary_project_cache_key`. `push_python_plan_cache_key` now emits `python.binding_contract=sifr-python-binding-v1`, `python.declaration_kind/effect/cleanup/consumes_receiver`, per-parameter `default|required`, `python.parameter_type=<Type>`, and `python.return_type=<Type>`. Bridge inventory/source digests and package identity remain in the same fragment.

**Hermetic archive/unpack/build/run** — `archived_package_python_bridge_builds_and_runs_without_extraction`: writes inventory into source, `cargo package --allow-dirty --no-verify`, tar-unpacks, asserts `__sifr_inventory__.json` shipped, deletes source, rebuilds from `installed_app.root`, removes bridge sources, then runs the binary with CWD and `TMPDIR` in a `0o555` directory. Success plus final `read_dir(&run_root).count() == 0` proves the loader needs no writable extraction directory. `#[cfg(unix)]` gate is correct (uses `PermissionsExt`).

**Single authority / no fallback / <900 lines** — No new fallback branches; `let _ = write!(out, …)` on `String` cannot panic. Largest touched file is `package_project_build_check.rs` at 875 lines. Docs updated in `python_interop_declaration_architecture.md`; TS-Go guardrail entry moved from `environment.rs:204` → `:211` in both the doc and the checker consistent with the 7-line shift from adding `PythonDistributionProbe`.

No actionable correctness, hermeticity, cache-completeness, test-validity, or maintainability findings.

SATISFIED

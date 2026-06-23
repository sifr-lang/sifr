## Findings

### Patches confirmed correct

- `crates/sifr_driver/src/build/materialize.rs:213` — `env_remove("CARGO_TARGET_DIR")` placed on the `cargo build` invocation. The caller reads artifacts from `binary_relative_path` (`<project>/target/release/<binary>`, materialize.rs:68–78) and the cache commit at workspace.rs:120 requires `<staging>/<project>/target` and `<binary path>` to exist. With an inherited outer `CARGO_TARGET_DIR`, cargo would write into the outer dir and the commit would fail with `BUILD_ARTIFACT_MISSING`. Stripping it is the right fix.
- `crates/sifr/tests/e2e_support/batch_execution.rs:82` — `env_remove("CARGO_TARGET_DIR")` on the batch `cargo build`. `build_group_binary_path` returns `<group_root>/target/debug/<package>` (batch_execution.rs:2–9) and the absolute path is stored in `CacheEntry.artifact_path` (line 137). Without the strip, an inherited env var moves the binary elsewhere and downstream run/cache-hit paths point at a non-existent file.
- `crates/sifr_stdlib/tests/api_behavior.rs:31` — pure rustfmt re-flow of one `expect` call onto a single line. No functional change.

### Observation (not a regression, not in stated scope)

`crates/sifr_driver/src/test_runner/execution.rs:126` invokes `cargo test` and uses `prepare_cached_artifact` with `required_paths = [Cargo.toml, src/lib.rs, target]` (execution.rs:33–37). It is structurally the same pattern as `materialize.rs` — the per-project `target/` must live inside `staging_root` for the `commit()` check (workspace.rs:120–131) to succeed, and it does not call `env_remove("CARGO_TARGET_DIR")`. The two existing integration tests at `crates/sifr_driver/src/tests/test_runner.rs:114, 173` would fail under `CARGO_TARGET_DIR=...` for the same reason the materialize tests would have failed pre-patch. This is a pre-existing bug, not introduced by this PR, and the PR description explicitly scopes coverage to the two files patched — so this is a follow-up call-out, not a blocker.

### Non-issues considered and dismissed

- `crates/sifr_driver/src/build/workspace.rs:317` keeps `CARGO_TARGET_DIR` in `toolchain_signature()`. After this patch it has no functional effect on where cargo writes, only on cache-key tenancy — a minor over-invalidation across differing outer env, not a correctness problem.
- `crates/sifr_driver/src/build/rust_interop_probe.rs:74` does not depend on a per-project target tree, so no strip is needed.

### Required fixes

None for the listed scope.

### Merge safety vs. sysroot stdlib M4

Safe to merge before M4. The patches are minimal, the artifact-path logic they protect is exactly the same machinery M4 will lean on, and validation with `CARGO_TARGET_DIR=target/envfix-validation` was run against both touched paths. Recommend tracking the `test_runner/execution.rs` strip as a separate follow-up so the env-isolation story is consistent across all driver-spawned cargo commands.

review-satisfied

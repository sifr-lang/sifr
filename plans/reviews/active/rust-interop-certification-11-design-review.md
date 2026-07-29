## Pre-implementation architecture review — `certification_11` (`cargo_locked_offline`)

I read the contract (`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1223-1245`, row table line 138, shared rules 72-118, 1009-1018), the CLI/package/driver code paths, the scaffold, and the cert-9/10 test patterns. I also ran controlled Cargo experiments in `/tmp` (no repo files touched) because three acceptance clauses depend on exact Cargo behaviour.

### 1. Confirmed root gaps

| Gap | Evidence |
| --- | --- |
| `sifr build` has no `--locked/--offline/--frozen`; hardcodes `Normal` twice | `crates/sifr/src/cli_model_and_entrypoint.rs:72-81`, `crates/sifr/src/diagnostic_rendering_and_run.rs:72-90` |
| `PackageEntrypoint` carries no lock mode, so `check`/`run` discard it at the driver boundary | `crates/sifr_driver/src/build/entrypoint.rs:79-85`, `crates/sifr/src/check_and_package_commands.rs:157-175` |
| Final generated `cargo build` gets no mode flag | `crates/sifr_driver/src/build/materialize.rs:242-258` |
| Synthetic probe runs `cargo check --quiet` in a fresh nonce temp dir with a generated manifest and **no** lock | `crates/sifr_driver/src/build/rust_interop_probe.rs:76-109`, `rust_interop_probe_manifest.rs:4-25` |
| Probe/artifact cache keys omit lock mode, so a `Normal` probe result would satisfy a later `--frozen` request | `rust_interop_probe_cache.rs:37-64`, `materialize.rs:402-427` |
| `--locked --offline` together silently collapses to `Offline`, dropping `--locked` | `cli_model_and_entrypoint.rs:679-693` |
| Files at/near the 900-line cap on every edit path | `package_rust_interop_build_tests.rs` **900**, `rust_interop_probe.rs` 897, `mode_resolution_tests.rs` 897, `entrypoint.rs` 890, `diagnostic_rendering_and_run.rs` 887, `cli_model_and_entrypoint.rs` 885, `check_and_package_commands.rs` 848, `check_fixture_matrix.py` 899 |

Two additional gaps the brief didn't list:

- **Probe failure misclassification.** `classify_probe_failure` (`rust_interop_probe_diagnostics.rs:110-115`) treats any stderr containing `not found`/`failed to resolve` as `SIFR-RUST-RESOLVE-0001`, else falls through to `SIFR-RUST-TYPE-0001`. Every lock/offline/frozen failure would therefore surface under the *wrong* code. Lock-mode classification must be a **first** branch, ahead of all target-shape heuristics.
- **Silent probe skip.** `execute_direct_cargo_probe` returns `Ok(())` when `cargo_manifest_path` is not a file (`rust_interop_probe.rs:53-58`). Under a locked/frozen request that is a fallback path; it needs a fail-closed branch for the certified modes.

### 2. Cargo semantics — measured, not assumed

All runs offline, lock byte-compared afterwards:

| Scenario | Result |
| --- | --- |
| No `Cargo.lock` + `--locked` / `--frozen` | hard error `cannot create the lock file … because --locked was passed`; **no lock written** |
| Root repo `Cargo.lock` copied into a synthetic crate (superset), root entry injected | **rejected** — Cargo prunes the 800+ unused registry entries, so the resolve differs from the lock |
| Unused *source-less* entry appended | tolerated (unreliable; don't build on it) |
| Checksum mutated | `checksum for indexmap v2.14.0 changed between lock files` — fails even without `--locked` |
| `source` switched registry→git | `cannot update the lock file … --frozen` |
| Locked version not obtainable (stale entry) | `failed to select a version for the requirement … (locked to 2.99.0)` |
| **Feature drift, registry dep gains `features = ["serde"]`** | `cannot update the lock file … --locked`, lock unchanged ✅ |
| **Feature drift by flipping a workspace-member feature (`drift = ["indexmap/serde"]`)** | **accepted, builds cleanly** ❌ |

The last two rows are the decisive finding. Cargo's dependency resolve unions **all features declared by workspace members**, so any optional dep reachable from a member feature is already in the lock. The scaffold's shape — `locked_bridge` with empty `stable`/`drift` features selected from the root manifest (`examples/locked_offline_cache/rust/locked_bridge/Cargo.toml`, `Cargo.toml`, `README.md` "changes the requested feature set without updating the lockfile") — **cannot ever produce a lock-affecting drift**, and its all-local dependency graph has nothing to drift. This must be redesigned, not merely populated.

### 3. Proposed design

**3.1 Fixture (`verification/areas/rust_interop/fixtures/cargo_locked_offline/`)**

- `rust/locked_bridge/Cargo.toml` gains exact-pinned **registry** deps already in the root lock and already in the crate catalog (e.g. `indexmap = { version = "2.14.0", default-features = false }`, plus one more crate for the checksum/source directions). Wrapper functions must actually call them so the probe and generated build compile real code.
- Canonical positive state: **no** feature activating an optional dep. Remove `stable`/`drift` marker features entirely — they are inert and misleading.
- `Cargo.lock`: exact resolution for the scenario, root-lock subset (already enforced by `_scenario_lock_checks.require_root_lock_subset`).
- `fixture.json`: `required_crates` becomes non-empty; `features` records the exact pinned feature policy; both evidence records get `status: passing` plus `validation` provenance. Update `_crate_catalog.py`, `_matrix_inventory.py`, `_scenario_registry.py` tokens, `_scenario_checks.py:502-508`, both matrices, `stable_support_claims.json`, `internal_docs/rust_interop_architecture.md`, and the public compatibility docs; re-derive the inventory counts (crate-alias count rises past 44 unless every added crate is already cataloged — prefer already-cataloged crates).
- Negative direction = **four** mutations of the copied scenario, each asserted independently: (a) delete/blank the scenario `Cargo.lock`; (b) rewrite one locked version to an unobtainable one; (c) corrupt one `checksum` and, separately, one `source`; (d) **add `features = ["serde"]` to the wrapper's registry dependency line** (the only shape that works). Each must yield `SIFR-RUST-CARGO-0001` with a kind-specific reason, no network, and a byte-identical lock afterwards.

**3.2 Compiler threading (minimal, one owner per hop)**

```
CargoLockMode (existing, sifr_package::cargo::lock_modes)
  ├─ CLI: add locked/offline/frozen to Commands::Build; fix lock_mode_from_flags
  │       so locked && offline ⇒ Frozen (or make the enum a two-bool policy struct)
  ├─ PackageEntrypoint { …, lock_mode: CargoLockMode }        // driver/build/entrypoint.rs
  ├─ RootedEntrypoint::PackageProject → PackageRustInteropContext.lock_mode
  │       → PendingRustBridgeProbe.lock_mode → probe argv + probe cache key
  └─ GeneratedBinaryProject.lock_mode → materialize_* → run_cargo_build argv
```

`sifr_driver` already depends on `sifr_package`, so no new type is needed. Single-file/`Project` builds keep `Normal` (see §5).

**3.3 The lock problem for compiler-generated Cargo projects (the crux)**

Neither the materialized `sifr_output` project nor the synthetic probe has a lock, and §2 proves you can neither run `--locked` without one nor seed one from the root/scenario lock. Introduce one shared module — `crates/sifr_driver/src/build/cargo_resolution.rs` — used by both call sites:

```rust
pub(super) struct ResolutionRequest<'a> {
    project_dir: &'a Path,          // materialized project or probe root
    lock_mode: CargoLockMode,
    authoritative_lock: &'a Path,   // scenario/package Cargo.lock
    cache_key: &'a str,
}
pub(super) struct PreparedResolution { lock_digest: String, prepared: bool }

pub(super) fn prepare(req: &ResolutionRequest<'_>) -> Result<PreparedResolution, Vec<RenderedDiagnostic>>;
pub(super) fn assert_unchanged(req: &ResolutionRequest<'_>, prepared: &PreparedResolution)
    -> Result<(), Vec<RenderedDiagnostic>>;
```

1. Key-addressed prepared-lock store under `artifact_cache_root()/cargo_resolution/<key>/Cargo.lock`, key = existing project/probe cache key. Hit ⇒ copy in, no cargo resolve at all (this is the warm path).
2. Miss ⇒ one **preparation** invocation: `cargo generate-lockfile`, with `--offline` whenever `lock_mode.is_network_disallowed()`. This is the contract's "cold preparation"; it never touches the network in the certified modes.
3. **Validate** the produced lock against `authoritative_lock`: every registry package's `(name, version, source, checksum)` must match an entry there. Any mismatch ⇒ `SIFR-RUST-CARGO-0001`. This is what makes step 2 honest rather than a silent resolver fallback — the user's lock stays the sole authority for versions.
4. Run the real `cargo check`/`cargo build` with `lock_mode.cargo_arg()` appended.
5. `assert_unchanged` re-digests the lock after the subprocess; any write under a lock-mutation-disallowed mode ⇒ `SIFR-RUST-CARGO-0001`.

Prepared locks are portable across probe nonce dirs: path packages carry no `source`, so identity is name+version only.

**3.4 Diagnostics** — new `cargo_lock_mode_failure(stderr, lock_mode)` classifier, called **before** everything in `classify_probe_failure`, matching the measured signatures (`cannot create the lock file`, `cannot update the lock file`, `checksum for … changed between lock files`, `failed to select a version for the requirement`, `no matching package`/`offline`), all mapped to `SIFR-RUST-CARGO-0001` with a distinct `{reason}` per drift kind. `SIFR-RUST-CARGO-0001` already exists (`registry/registry_entries/rust_interop.rs:107`) and is already emitted with varying messages, so only the docs prose and the family page need extending.

**3.5 Cache identity**

- **Probe cache key: add the lock mode.** A `Normal` pass must not satisfy a `Frozen` request. Cost: probes re-run once per mode in the mandatory test; acceptable and worth noting in the plan.
- **Artifact cache key: do *not* add the lock mode**, otherwise the required "cold prepare then network-disabled warm hit reuses the same artifact identity" is unachievable. Record the mode and prepared-lock digest inside the cached entry instead and assert warm-hit path equality. Note this is already drift-safe: `rust_interop_cargo_inputs.rs:282` folds the nearest `Cargo.lock` digest into interop cache identity, so a mutated lock changes the key and cannot be masked by a stale cache — worth an explicit assertion in the negative test.

**3.6 Tests**

- Positive → **`sifr_cli_generated_builds`** suite (`verification/profiles/*.json:71-88`, allowed for positive `cargo-probe` evidence at `_provenance_checks.py:412-420`): a new `#[ignore]`d unit test file in `crates/sifr/src/` following the `mode_resolution_tests.rs:377+` pattern (`enter_test_cwd`, direct `cmd_check`/`cmd_build`/`cmd_run` calls). Matrix: 3 commands × 3 modes on the copied fixture package, then re-run for the warm hit. Assert cache hit, identical binary path, byte-identical lock, and identical package identity.
- Mode preservation needs observability. Recommend an in-process, `#[doc(hidden)]` cargo-argv sink in `sifr_driver` (the CLI unit test runs in-process with the driver, so no env-var side channel is needed) and assert **every** recorded invocation — metadata, probe check, generated build — carries the expected flag. An env-var JSONL log is the fallback if the sink proves awkward; prefer the sink to avoid a production side channel.
- Negative → new `crates/sifr_driver/src/tests/package_rust_interop_locked_offline_support.rs`, wired as `#[path]`-mod from `package_rust_interop_build_tests.rs` (the established cert-9/10 pattern — and mandatory, since that file is at exactly 900 lines).
- Network denial: the modes themselves plus an asserted-absent registry write are the honest proof; do not rely on host firewalling.

### 4. File splits required (900-line cap)

| File | Now | Action |
| --- | --- | --- |
| `tests/package_rust_interop_build_tests.rs` | 900 | new `_locked_offline_support.rs` sibling; no lines added to the parent beyond one `#[path]` mod |
| `build/rust_interop_probe.rs` | 897 | extract command construction + lock-mode wiring into `cargo_resolution.rs`; probe file should shrink |
| `build/entrypoint.rs` | 890 | move the inline `mod tests` (lines 634-890) to `tests/`-side or a `entrypoint_tests.rs` sibling before adding the field/params |
| `crates/sifr/src/cli_model_and_entrypoint.rs` | 885 | extract `Commands` enum (or the lock-flag group + `lock_mode_from_flags`) into a `cli_lock_modes.rs` |
| `crates/sifr/src/diagnostic_rendering_and_run.rs` | 887 | split `cmd_build`/build-report helpers into `build_command.rs` |
| `crates/sifr/src/mode_resolution_tests.rs` | 897 | put the new CLI evidence test in its own file, not here |
| `checks/check_fixture_matrix.py` | 899 | any new validation belongs in `_scenario_lock_checks.py` (56 lines, natural home) |

### 5. Acceptance wording needing a stated interpretation

1. **"execute Sifr check/build/run with `--locked`, `--offline`, and `--frozen`"** — read as three separate mode runs of each command against the fixture *package*, not as one invocation carrying all three flags (cert-10's `cargo build --locked --offline --frozen` was a raw Cargo call, not a Sifr one). Also decide whether `sifr build <file>` in single-file/`Project` mode accepts the flags: those builds materialize a sysroot-vendored project with no user lock, so they must either use the same prepared-lock mechanism against the sysroot lock or emit an explicit diagnostic. A silent downgrade to `Normal` would violate "no fallback" — pick one and document it.
2. **"cold preparation then network-disabled warm cache hit"** — read as: prepared registry cache + checked-in scenario lock + one prepared generated-project resolution, then a second run that reaches the artifact cache with zero cargo resolution. The §3.3 validation step is what keeps the preparation pass from being a resolver fallback.
3. **"feature drift"** — must be manifest feature-list drift on a registry dependency (§2). If the intent was drift against Sifr's *declared* feature policy, that is a separate, additional pre-Cargo check; state which one the negative evidence certifies (I recommend certifying the Cargo-observable one, since the row is `cargo-probe`, and adding the policy check as defence in depth).
4. **"frozen mutation"** — the measured behaviour is that `--frozen` refuses to *write*; the assertion should be "attempted mutation rejected **and** lock byte-identical", not merely a non-zero exit.

### 6. Principal risks

- Probe cost: mode in the probe cache key multiplies cold probes ~3×; the Rust-interop area has a 10-second budget in create-PR. Measure early.
- The prepared-lock validator is new code on a hot path; a too-strict comparison turns legitimate transitive resolutions into false `CARGO-0001` failures, a too-loose one lets version drift through. Unit-test it against the scenario lock directly, not only end-to-end.
- Adding registry deps to the wrapper widens the crate catalog, root-lock subset assertions, and inventory counts; prefer crates already in the catalog to keep the count at 44.
- `mark_probe_cache_hit` writes an `.ok` marker keyed by content; if lock mode is added to the key, verify no stale `.ok` files from earlier runs collide.

**SATISFIED** — the direction in §3 is implementable as stated against the exact acceptance contract, with the two mandatory corrections it already incorporates: the feature-drift mechanism must be registry-dependency feature drift (the scaffold's workspace-feature shape provably cannot work), and generated/probe projects need the prepared-and-validated resolution of §3.3 because no lock can be seeded from the root or scenario lock. The four items in §5 are interpretation decisions to record in the PR description, not blockers.

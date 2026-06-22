## Review findings

### Medium — invented `sifr.toml` schema in scenario examples

The new `sifr.toml` files use `[rust.workspace]`, `[rust.dependencies.<X>]` (with `path`/`features`/`bridge = "shared"`), and `[rust.cargo]` (with `locked`/`offline`/`frozen`/`lockfile`). The actual sifr.toml parser at `crates/sifr_package/src/manifest/sifr_fields.rs:148-173` (`parse_rust_interop_config`) only recognizes `rust.bridge-version`, `rust.bridges`, `rust.direct-crate-bindings` — none of the new keys would be loaded.

- For `cargo_locked_offline/examples/locked_offline_cache/sifr.toml` this is consistent with the fixture being `future-owned`/`planned`.
- For `same_workspace_crate/examples/workspace_hash_crate/sifr.toml` and `shared_bridge_crate/examples/shared_hash_bridge/sifr.toml`, evidence status is `passing` while the scenario `sifr.toml` claims a manifest format that doesn't exist. The READMEs (e.g. `same_workspace_crate/examples/workspace_hash_crate/README.md:5-7`) reinforce the implication that "the dependency is declared in both the Sifr package metadata and the generated Cargo workspace shape". Either the parser needs these keys, or the README + sifr.toml should be marked aspirational, or these passing fixtures should drop the invented Sifr-side keys and rely on the Cargo manifest alone for the layout claim.

### Low-medium — analogous fixtures still lack package-layout examples

`bridge_version_mismatch` and `panic_abort_profile` match the same shape that motivated this patch: empty `required_crates`, contract/compiler-diagnostic execution, and evidence whose meaning is package-level config rather than a registry crate.

- `bridge_version_mismatch/positive/bridge_version_1_accepted.sifr:6` annotates `# fixture-cargo: rust.bridge-version = 1` but ships no Cargo.toml/sifr.toml.
- `panic_abort_profile/positive/explicit_abort_trust_and_strategy_declared.sifr:6` annotates `# fixture-trust: rust-panic-abort = ["bridge.legacy.run"]` but ships no manifest.

If the user's bar is "package/manifest claims need a directory-backed example", these two fixtures qualify. Both directories currently have only `README.md`, `fixture.json`, `positive/`, `negative/`.

### Low — token check is structurally weak

`_validate_scenario_example_dir` (check_fixture_matrix.py:381-420) concatenates README + sifr.toml + all `.sifr` + all `Cargo.toml` + all `.rs` and verifies substring presence. A token in README alone satisfies the check, even if the corresponding `Cargo.toml` is malformed or missing the key. For `--locked --offline --frozen` the only required carrier turns out to be the README prose; the sifr.toml has `locked = true` and the Cargo.lock has no flags. Adequate as a regression guard, but does not prove the manifests are actually well-formed. No TOML parse is performed.

### Low — scenario `Cargo.toml` location is unconstrained

The checker uses `rglob("Cargo.toml")` and only asserts ≥1 manifest exists anywhere under `examples/<scenario>`. A future scenario could ship only the inner crate's `Cargo.toml` and pass. Fine for today's three patched scenarios.

### What the patch gets right

- Required-set rejection is exact: missing scenario → fail, unexpected scenario → fail (`check_fixture_matrix.py:344-348`), with path pinned to `examples/<scenario>` (`:361-362`).
- File presence: README, sifr.toml, ≥1 `.sifr`, ≥1 `Cargo.toml`, ≥1 `.rs` are all required.
- `.sifr` quality: ≥10 lines, no empty `pass` body, must include `@rust`, must declare ≥1 Rust-decorated binding, must include a `verify_…` function that calls **and** binds each non-`None` binding (`:434-461`).
- The three example directories are concrete — real Rust impls, plausible Cargo workspace, sensible `verify_*` functions, no stubs. The `crate::__sifr_bridge` comment in `shared_hash_bridge/rust/.../lib.rs:18-19` correctly exercises the "comments are OK, imports are not" boundary called out in `shared_bridge_crate/README.md`.
- README at `verification/areas/rust_interop/README.md:30-44` documents the new convention and the checker enforces it.

### Not blockers but flagging

- The three checker fields are good; no other rust-interop fixture currently has `scenario_examples` in `fixture.json`, so the new validation path is exercised only by the patched three. If you intend to extend coverage to `bridge_version_mismatch`/`panic_abort_profile`, the existing checker would scale unchanged.

**Bottom line:** the patch is real coverage, not a stub. The one thing I'd push back on is the invented `[rust.workspace]`/`[rust.dependencies]`/`[rust.cargo]` sifr.toml keys in the two `passing` fixtures — they don't match the manifest parser today and read as forward-looking schema in a fixture whose status says "verified now". The other action item is whether to extend the same treatment to `bridge_version_mismatch` and `panic_abort_profile`.

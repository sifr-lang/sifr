I've completed my review of the branch. Here are my findings.

## M13 installed-toolchain certification review — no blocking findings

### Findings, ordered by severity

**Non-blocking cleanup:**

1. **`verification/areas/coverage_matrix/checks/verification_taxonomy.py:625`** — Stray dangling expression `    "work-on-item",` sitting inside the `if __name__ == "__main__":` block AFTER `raise SystemExit(main())`. It's a Python-legal expression statement (bare tuple literal) but wholly unreachable dead code — an editing artifact. Doesn't affect behavior; should be removed.

2. **`verification/areas/sysroot_release/runner.py:100-101`** — When `select_suites` is called with no `--suite` filter it defaults to `["host-installed-smoke"]` only. Fine in profile-runner use because `profile_runner.py:499-502` explicitly enumerates `--suite` per profile. Direct standalone invocations of `runner.py` without `--suite` will silently skip the heavy suite and the self-test. Not a regression (profile lanes are fully wired), but worth a docstring note.

3. **`verification/areas/sysroot_release/runner.py:409`** — `env["CARGO_TARGET_DIR"] = ...` unconditionally overrides any caller-supplied `CARGO_TARGET_DIR` inside `build_artifact`. Intentional (avoids collision with the outer create-pr target dir), but the release-build target dir is placed under `REPO_ROOT/target/sysroot_release/cargo-target` while the RUSTFLAGS remap uses the caller-set path — actually re-checking `release_rustflags` reads `CARGO_TARGET_DIR` from its own env, which is the overridden value inherited into the script invocation. Consistent. No fix needed.

4. **`crates/sifr_driver/src/build/rust_interop.rs:355-366`** — The missing-sysroot-runtime diagnostic emits once per backend path in a declaration; a single declaration with multiple backend target paths would raise the same diagnostic repeatedly. Cosmetic. In practice this code path can only fire if the compiler is invoked without a resolved sysroot, which is a configuration failure that surfaces earlier via stdlib bootstrap.

5. **`crates/sifr_driver/src/build/rust_interop_probe_cache.rs:43`** — Cache-key input changed from `optional_manifest_root_digest(...)` to `cached_digest_path(&probe.sysroot_runtime_crate)`. Semantically identical when a sysroot is present (both digest the runtime crate root), but the `<no-sysroot-runtime>` sentinel branch is gone entirely. That's the intended behavior — no sysroot means the probe is not reached — but it's a one-shot cache-key change so existing `.ok` markers become stale. First run after upgrade re-probes.

**Non-blocking residual observations:**

- **Test-only source-tree derivation** — `rust_interop_tests.rs:707`, `rust_interop_contract_tests.rs:729`, and `sysroot_interop_tests.rs:277` all define `test_runtime_crate()` returning `PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("sifr_runtime")`. Same construction, three copies. Not a fallback in shipped code and each `#[cfg(test)]`-scoped, but a small helper in a shared test-utils spot would remove the duplication.

- **Text-trigger prefilter completeness** (`verification_taxonomy.py:84-116, 344`) — I audited every entry in `TEXT_PATTERNS` against `TEXT_TRIGGER_TERMS` + `M_TOKEN_TRIGGER`. Every existing pattern requires at least one taxonomy word (`phase`, `milestone`, `wave`, `roadmap`, `closeout`, `backlog`, `contract`, `capability`, `capabilities`, `slice`, `follow-up`, `ad hoc`/`ad-hoc`, `closure`, `gate-closure`, `surface`, `task ownership`, `todo(`, `reference:`, `source issue:`, `work item`/`work-item`, `work-on-item`, `conversion set`, `future phases`, `later phases`, `successor-phase`, `closed from a local validation standpoint`) or an `m<digit>`/`m_[a-z]` token. The `\`filename spaced.md\`` pattern at line 151 previously could match generic multi-word backtick .md filenames without any taxonomy signal — the trigger prefilter now scopes that pattern to taxonomy-tagged lines only, which is an intentional narrowing rather than a regression. Coverage semantics preserved.

- **Home-leakage scan scope** (`runner.py:217-231, 371-385`) — Home is scanned only against archive + emit, not against `sysroot_json_path` / `lsp_trace` / generated `Cargo.toml`+`Cargo.lock`+`src`+`tree_path`, because those files legitimately embed the extract root (under `TMPDIR`). On macOS runners `TMPDIR` is under `/var/folders`, on Linux runners under `/tmp` — neither overlaps `Path.home()`. Correct scope.

- **Repo leakage scan scope** (`runner.py:200-214, 351-368`) — Scans archive + sysroot JSON + emit + LSP + generated Cargo/src + cargo-tree output. All items that could plausibly contain source-tree paths are covered. `RUSTFLAGS --remap-path-prefix` in `release_rustflags` handles `CARGO_TARGET_DIR`, repo root, sysroot, `CARGO_HOME`, and `RUSTUP_HOME`. Complete.

- **Installed toolchain proof** — `sifr --print sysroot --json` is validated against `install_root.resolve()`, version, and target triple (`runner.py:450-460`); heavy suite runs full `sifr check/emit/build`, executes the built binary, and asserts a fixture-controlled output string plus offline+frozen Cargo verification. That is stronger evidence than the merge lane needs and proves the extracted archive is self-sufficient.

### Blocking findings

**None.** The fallback removal is complete:

- No `fallback_runtime_crate_root`, `<no-sysroot-runtime>` sentinel, source-tree ancestor scanning, or checkout-time `env!("CARGO_MANIFEST_DIR")` derivation remains anywhere in the shipped Rust bridge probe / cache / runtime resolution path. Verified by grep across `crates/`.
- `PendingRustBridgeProbe::sysroot_runtime_crate` is `PathBuf` (not `Option`); `probe_cargo_toml` requires `sysroot_runtime_crate: &Path`.
- `RustInteropResolver::resolve_path` at `rust_interop.rs:355-366` emits a typed `RUST_CARGO_METADATA` diagnostic when the context lacks a resolved sysroot runtime crate, instead of falling back.
- `test_runtime_crate()` helpers use `env!("CARGO_MANIFEST_DIR")` only inside `#[cfg(test)]`; they never affect release builds.
- `explain_cli.rs` source-tree Markdown extraction is gated on `#[cfg(debug_assertions)]`, with a registry-summary + docs URL path for release.
- Verification profile wiring: merge = `host-installed-smoke`; nightly/release = smoke + heavy; create-pr = empty. Coverage matrix rows are consistent with profile assignments and use the same comma-separator style as neighboring rows.
- Suite manifests, resource classes, and adapter kind = "adapter" are internally consistent.

### PR readiness

**Satisfactory for M13 wave-1 phase closure.** The rejection-of-fallback contract is enforced structurally by types and typed diagnostics; the installed certification suite actually exercises the extracted toolchain end-to-end and gates on both repository and home path leakage; the taxonomy prefilter fix preserves semantic coverage while eliminating the step-budget blow-up; the file-size guardrail passes on all touched modules (rust_interop.rs is 835 LOC after extraction — under the 900-line cap); and the plan/architecture docs honestly reflect the "no fallback path" contract. The one item worth fixing before merge is the stray line-625 tuple literal in `verification_taxonomy.py`; everything else is optional follow-up.

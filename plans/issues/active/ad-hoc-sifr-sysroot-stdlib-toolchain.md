# Ad Hoc Phase: Sifr Sysroot and Stdlib Toolchain

## Status

In progress.

## Implementation Status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0. Architecture Baseline and Inventory | completed, merged | Baseline tables added to [`internal_docs/sifr_sysroot_and_stdlib_architecture.md`][sysroot-stdlib-architecture]; migration registry added at `internal_docs/stdlib_native_surface_ownership.toml`; local create-pr validation and Opus review were satisfied in merged [PR #2741](https://github.com/sifr-lang/sifr/pull/2741). |
| M1. Sysroot Identity and Resolver Skeleton | completed, merged | `crates/sifr_sysroot` adds manifest parsing, layout validation, resolver precedence, digest canonicalization, development source-tree sysroot resolution, and CLI `sifr --print sysroot` support in merged [PR #2743](https://github.com/sifr-lang/sifr/pull/2743). |
| M2. Rename Current Compiler Stdlib Crate | completed, merged | Merged in [PR #2745](https://github.com/sifr-lang/sifr/pull/2745). The compiler-side crate is now `crates/sifr_stdlib_model`, freeing `sifr_stdlib` for the generated-program crate. |
| M3. Create Generated-Program `sifr_stdlib` Crate | completed, merged | `crates/sifr_stdlib` now provides the generated-program crate foundation with narrow feature gates, runtime-backed wrapper APIs, feature-plan expectations, installed-layout checks, and representative feature-tree snapshots in merged [PR #2747](https://github.com/sifr-lang/sifr/pull/2747). |
| M4. Full Sysroot Workspace and Source Layout | completed, merged | Merged in [PR #2750](https://github.com/sifr-lang/sifr/pull/2750). Public stdlib sources now live under `stdlib/sifr`, private `_sifr` placeholders are present under `stdlib/_sifr`, sysroot validation covers both stdlib crates and source roots, and CLI/LSP definitions load from the resolved sysroot source inventory. |
| M5. Generated Cargo Uses Sysroot Crates and Vendor | completed, merged | Merged in [PR #2752](https://github.com/sifr-lang/sifr/pull/2752). Generated Cargo now consumes `SysrootDependencyPlan`, emits sysroot `sifr_runtime`/`sifr_stdlib` path dependencies with `default-features = false`, applies sysroot vendor config invocation-scoped for Sifr-managed builds, reports sysroot identity, and vendors the sysroot workspace graph. Local `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory; Opus review pass 2 was satisfied for PR readiness with non-blocking package/offline fixture follow-ups. |
| M6. Distribution Artifact and Installer Update | completed, merged | Merged in [PR #2753](https://github.com/sifr-lang/sifr/pull/2753). Release artifacts now package `bin/sifr` plus the complete sysroot, validate archive contents before checksums/installer generation, write schema-2 receipts with `sysroot_path`, and preserve binary/sysroot pairing through self-update. Focused validation passed: `cargo test -p sifr self_update`, `cargo test -p sifr_sysroot`, distribution release representative suite, and developer tooling TypeScript-Go transfer suite. Opus review pass 2 was satisfied; local `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory. |
| M7. LSP and Tooling Sysroot Source/Navigation Integration | completed, merged | Merged in [PR #2754](https://github.com/sifr-lang/sifr/pull/2754). Sysroot public/private stdlib files now flow into frontend source maps with source origins, analysis/LSP overlay hosts consume sysroot tooling sources, the stdlib symbol bucket is populated from parser-backed installed public sources, public stdlib import/call-site definitions route to installed sysroot URIs, and public stdlib implementation files can navigate to private declaration files without exposing `_sifr` declarations to user completion. Opus review pass 3 was satisfied after splitting proactive sysroot diagnostics and generated/synthetic origin production to M7b; local `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory. |
| M7b. Tooling Sysroot Diagnostics and Synthetic Origins | completed, merged | Merged in [PR #2755](https://github.com/sifr-lang/sifr/pull/2755). Tooling sysroot probes now feed proactive LSP diagnostics and structured `sifr/sysroot` broken/mismatch responses with observed paths; development LSP/CLI root and toolchain comparison coverage verifies local build parity; generated Rust preview metadata now carries production `GeneratedSupport` and `CompilerSynthetic` source-map entries from real compiler output. Opus review pass 2 was satisfied; local `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory. |
| M8. Rust Interop Context for Private Stdlib Declarations | completed, merged | Merged in [PR #2756](https://github.com/sifr-lang/sifr/pull/2756). The branch adds a compiler-owned synthetic package context for private `_sifr` Rust interop declarations, resolves private targets only to canonical sysroot `sifr_stdlib`/`sifr_runtime` crates, applies sysroot trust without extending trust to user packages, keeps sysroot interop in sysroot-only vendor mode, and routes probes through sysroot runtime/vendor inputs. Opus review pass 2 is satisfied after hardening merged user+sysroot context validation and sysroot interop dependency-plan cache fingerprints; local `scripts/run_all_tests.sh --profile create-pr` passed with only the warm wall-time advisory. |
| M9-M13 | in progress | M9 wave 1 merged in [PR #2757](https://github.com/sifr-lang/sifr/pull/2757), migrating `_sifr.platform` and `_sifr.html` to private Rust interop declarations backed by `sifr_stdlib` features. M9 wave 2 merged in [PR #2759](https://github.com/sifr-lang/sifr/pull/2759), migrating `_sifr.calendar` the same way. M9 wave 3 merged in [PR #2761](https://github.com/sifr-lang/sifr/pull/2761), migrating `_sifr.uuid` the same way. M9 wave 4 merged in [PR #2763](https://github.com/sifr-lang/sifr/pull/2763), migrating `_sifr.math` the same way. M9 wave 5 merged in [PR #2765](https://github.com/sifr-lang/sifr/pull/2765), migrating `_sifr.crypto` hash functions used by `sifr.hashlib` while retaining intrinsic fallback for unmigrated crypto helpers. M9 wave 6 merged in [PR #2767](https://github.com/sifr-lang/sifr/pull/2767), migrating infallible base64/base32 encoders while explicitly deferring fallible decode/options to M10. M10 wave 1 merged in [PR #2769](https://github.com/sifr-lang/sifr/pull/2769), migrating fallible base64/base32 decode/options through typed result-error direct interop. M10 wave 2 merged in [PR #2771](https://github.com/sifr-lang/sifr/pull/2771), migrating `_sifr.regex`/`sifr.re` through private Rust interop backed by `sifr_stdlib::regex` while retaining the separate direct regex dependency for `sifr.pathlib` glob lowering. M10 wave 3 merged in [PR #2776](https://github.com/sifr-lang/sifr/pull/2776), migrating `_sifr.url`/`sifr.url` through private Rust interop backed by `sifr_stdlib::url`. M10 wave 4 merged in [PR #2778](https://github.com/sifr-lang/sifr/pull/2778), migrating `_sifr.toml`/`sifr.tomllib` through private Rust interop backed by `sifr_stdlib::toml`. M10 wave 5 merged in [PR #2780](https://github.com/sifr-lang/sifr/pull/2780), migrating `_sifr.json`/`sifr.json` through private Rust interop backed by `sifr_stdlib::json` token adapters while preserving `JSONDecodeError` location fields and JSON integer profile errors. M10 wave 6 merged in [PR #2781](https://github.com/sifr-lang/sifr/pull/2781), migrating `_sifr.encoding`/`sifr.encoding` through private Rust interop backed by `sifr_stdlib::encoding` while preserving public `DecodeError`/`EncodeError` wrappers. M10 wave 7 merged in [PR #2782](https://github.com/sifr-lang/sifr/pull/2782), migrating `_sifr.unicode`/`sifr.unicode` through private Rust interop backed by `sifr_stdlib::unicode` while preserving public `UnicodeDataError` wrappers and Unicode segmentation tuple payloads. M10 wave 8 merged in [PR #2784](https://github.com/sifr-lang/sifr/pull/2784), migrating `_sifr.i18n`/`sifr.i18n` through private Rust interop backed by `sifr_stdlib::i18n` while preserving public i18n error wrappers. M10 wave 9 merged in [PR #2785](https://github.com/sifr-lang/sifr/pull/2785), migrating `_sifr.compress`/`sifr.gzip`/`sifr.zipfile` through private Rust interop backed by `sifr_stdlib` gzip and zipfile adapters. |
| Post-M10 Adapter Policy Adherence Audit | completed, merged | Merged in [PR #2774](https://github.com/sifr-lang/sifr/pull/2774). The audit classified completed M9/M10 private bindings, added executable guards for direct `sifr_stdlib` targets and trust separation, documented residual `_sifr.crypto` random scope, and passed Opus review pass 2 plus local `scripts/run_all_tests.sh --profile create-pr` with only the warm wall-time advisory. |

## PR Log

- M0 baseline/inventory: [PR #2741](https://github.com/sifr-lang/sifr/pull/2741) merged.
- M1 sysroot identity/resolver: [PR #2743](https://github.com/sifr-lang/sifr/pull/2743) merged.
- M2 compiler stdlib model rename: merged in [PR #2745](https://github.com/sifr-lang/sifr/pull/2745).
- M3 generated-program stdlib crate: merged in [PR #2747](https://github.com/sifr-lang/sifr/pull/2747).
- M4 full sysroot workspace/source layout: merged in [PR #2750](https://github.com/sifr-lang/sifr/pull/2750).
- M5 generated Cargo sysroot/vendor planning: merged in [PR #2752](https://github.com/sifr-lang/sifr/pull/2752).
- M6 distribution artifact and installer update: merged in [PR #2753](https://github.com/sifr-lang/sifr/pull/2753).
- M7 LSP/tooling sysroot source/navigation integration: merged in [PR #2754](https://github.com/sifr-lang/sifr/pull/2754).
- M7b tooling sysroot diagnostics and synthetic origins: merged in [PR #2755](https://github.com/sifr-lang/sifr/pull/2755).
- M9 wave 1 stateless platform/html leaves: merged in [PR #2757](https://github.com/sifr-lang/sifr/pull/2757).
- M9 wave 2 stateless calendar leaf: merged in [PR #2759](https://github.com/sifr-lang/sifr/pull/2759).
- M9 wave 3 stateless UUID leaf: merged in [PR #2761](https://github.com/sifr-lang/sifr/pull/2761).
- M9 wave 4 stateless math leaf: merged in [PR #2763](https://github.com/sifr-lang/sifr/pull/2763).
- M9 wave 5 stateless hash leaf: merged in [PR #2765](https://github.com/sifr-lang/sifr/pull/2765).
- M9 wave 6 stateless base encoding encoder subset: merged in [PR #2767](https://github.com/sifr-lang/sifr/pull/2767).
- M10 wave 1 fallible base encoding error bridge: merged in [PR #2769](https://github.com/sifr-lang/sifr/pull/2769).
- M10 wave 2 regex interop migration: merged in [PR #2771](https://github.com/sifr-lang/sifr/pull/2771).
- Post-M10 adapter policy adherence audit: merged in [PR #2774](https://github.com/sifr-lang/sifr/pull/2774).
- M10 wave 3 URL interop migration: merged in [PR #2776](https://github.com/sifr-lang/sifr/pull/2776).
- M10 wave 4 TOML interop migration: merged in [PR #2778](https://github.com/sifr-lang/sifr/pull/2778).
- M10 wave 5 JSON interop migration: merged in [PR #2780](https://github.com/sifr-lang/sifr/pull/2780).
- M10 wave 6 encoding interop migration: merged in [PR #2781](https://github.com/sifr-lang/sifr/pull/2781).
- M10 wave 7 Unicode interop migration: merged in [PR #2782](https://github.com/sifr-lang/sifr/pull/2782).
- M10 wave 8 i18n interop migration: merged in [PR #2784](https://github.com/sifr-lang/sifr/pull/2784).
- M10 wave 9 compression interop migration: merged in [PR #2785](https://github.com/sifr-lang/sifr/pull/2785).

## Design Reference

Authoritative target architecture:
[`internal_docs/sifr_sysroot_and_stdlib_architecture.md`][sysroot-stdlib-architecture]

The [stdlib Rust interop adapter policy][stdlib-interop-adapter-policy] is
locked: direct binding for exact-shape `sifr_stdlib` signatures, `sifr_stdlib`
adapters for reshaping or error mapping, and no callee injection in M9-M13.

[sysroot-stdlib-architecture]: ../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md
[stdlib-interop-adapter-policy]: ../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md#stdlib-rust-interop-adapter-policy

## Objective

Deliver the final Sifr toolchain architecture where Sifr installs as a
versioned sysroot-backed toolchain instead of a standalone binary. The completed
phase must eliminate build-machine path leakage, make CLI and LSP share the
same stdlib source view, reserve `sifr_stdlib` for the generated-program stdlib
crate, and move stdlib native implementation toward Rust interop-backed sysroot
crates.

## Non-Negotiable Outcomes

- Release installs include `bin/sifr` plus a complete `lib/sifr` sysroot.
- Generated Cargo projects resolve Sifr-owned dependencies from the sysroot,
  not from compile-time checkout paths.
- Public stdlib sources and private `_sifr` declaration modules are shipped in
  the sysroot and used by CLI/LSP.
- The current compiler-side `sifr_stdlib` crate is renamed or split so the name
  `sifr_stdlib` belongs to the generated-program stdlib implementation crate.
- Sifr-owned third-party Cargo dependencies for sysroot crates are vendored in
  the sysroot.
- Released binaries never silently fall back to `env!("CARGO_MANIFEST_DIR")`.
- User imports of `_sifr.*` remain rejected.
- Public `sifr.*` APIs remain behaviorally stable through the migration.
- Runtime/resource/callback migrations respect the active Rust interop runtime
  certification gate.

## Milestones

### M0. Architecture Baseline and Inventory

Lock the implementation baseline before code movement starts.

Tasks:

- Record current ownership of `lib/sifr`, current `crates/sifr_stdlib`,
  `crates/sifr_runtime`, generated Cargo manifest planning, distribution
  packaging, and LSP stdlib loading.
- Add an inventory table for current `_sifr.*` imports in public stdlib sources.
- Add an inventory table for current generated dependencies and whether they
  are sysroot-owned, vendored third-party, or user/package-owned.
- Add an inventory table for current `sifr_runtime::*` call sites in generated
  code and preamble.
- Create `internal_docs/stdlib_native_surface_ownership.toml` as the checked
  migration registry for native stdlib surfaces.
- Include inventory columns for current implementation owner, final
  implementation owner, migration blocker, and whether the surface can move
  before Rust interop runtime certification.
- For each registry entry, record current owner, final owner, reason,
  certification state, and deletion stage (`deletion_stage` field).
- Confirm which Rust interop compatibility matrix rows are certified, active,
  and future-owned before resource migration starts.
- Add this issue to roadmap/phase tracking as the owner for sysroot and stdlib
  toolchain work.

Acceptance:

- Reviewers can see the exact pre-migration ownership boundaries.
- The inventories identify every native stdlib surface that must be migrated or
  explicitly retained.
- The migration registry gives reviewers one mechanical inventory for
  supported, compiler-owned, future-owned, and unsupported native surfaces
  until M12 deletes it or reduces it to a retained allowlist.
- Runtime/resource surfaces are tagged with certification status before
  implementation begins.

Validation:

- Documentation review.
- No code behavior change.

### M1. Sysroot Identity and Resolver Skeleton

Implement the first-class sysroot model without validating generated-program
`sifr_stdlib` yet. Full sysroot crate validation waits until M2 frees the crate
name and M3 creates the new generated-program crate.

Tasks:

- Add a `ResolvedSysroot` model with parsed identity-only `sysroot.toml`
  metadata and fixed-layout paths derived structurally from the sysroot root.
- Define schema-versioned `sysroot.toml` fields for `sifr-version`,
  `target-triple`, `built-by-compiler-commit`, `sysroot-content-sha256`, and
  `cargo-lock-sha256`.
- Derive `toolchain-id` as `{sifr-version}-{target-triple}` for reports and
  diagnostics; do not store it in `sysroot.toml`.
- Keep Rust edition, Rust version, workspace package data, and sysroot crate
  versions in `Cargo.toml`, not in `sysroot.toml`.
- Define canonical tree-digest hashing for sysroot assets: path ordering, path
  separator normalization, included file types, excluded metadata, executable
  bit handling where relevant, symlink policy, and line-ending policy.
- Define the sysroot Cargo workspace contract: `Cargo.toml`, `Cargo.lock`,
  workspace members, workspace dependencies, and resolver version, but defer
  validation of the final `crates/sifr_stdlib` member until M3/M4.
- Add resolver precedence: explicit developer override, `SIFR_SYSROOT`,
  installed path relative to `current_exe()`, then development sysroot
  auto-resolution for unreleased local builds.
- Mark `--sysroot` for end-user commands as an advanced/developer override in
  help text and diagnostics rather than a normal package-management mechanism.
- Define the command-support matrix for `--sysroot`: advanced/hidden for
  `check`, `build`, `run`, and `emit`; settings or environment for LSP; visible
  for `doctor`; ignored or rejected for self-update unless multi-sysroot
  installs are explicitly designed.
- Add `sifr --print sysroot`.
- Add `sifr --print sysroot --json`.
- Add a schema drift check or snapshot so the documented `sysroot.toml` schema
  and parser-accepted schema cannot diverge silently.
- Define schema-version compatibility rules: unknown required fields fail, and
  unknown optional fields may be ignored only when the active schema version
  permits it.
- Add concise sysroot-boundary diagnostics for missing or mismatched sysroot
  assets.
- Remove documented runtime-only override behavior; runtime checkout testing
  must use a source-tree or generated development sysroot, with any
  migration-only runtime path override kept as a test helper.
- Remove release-mode fallback to compile-time checkout paths.
- Add build-mode gates so release binaries cannot silently use source checkout
  paths.
- Add a single `is_source_tree_development_mode()` predicate gated by explicit
  dev/debug build configuration. The predicate only controls whether the tool
  may auto-resolve or materialize a development sysroot.

Acceptance:

- A released-style binary with a complete skeleton sysroot resolves manifest,
  stdlib root, runtime crate, vendor, and Cargo config paths without
  environment variables.
- A missing runtime crate reports a Sifr sysroot diagnostic, not a Cargo error
  containing a build-machine path.
- Source-tree development works by resolving or materializing a sysroot-shaped
  development tree.
- Release binaries cannot resolve source-tree layout unless a valid installed
  sysroot or explicit sysroot override is present.
- Sysroot diagnostics include binary path, attempted sysroot path, and missing
  asset path.

Validation:

- Unit tests for resolver precedence.
- Unit tests for missing, malformed, and version-mismatched `sysroot.toml`.
- Snapshot or schema test comparing the documented manifest fields with the
  parser-accepted fields.
- Unit tests for schema-version unknown-field behavior.
- Unit tests for deterministic tree-digest canonicalization.
- Unit tests for missing sysroot `Cargo.toml`, `Cargo.lock`,
  `.cargo/config.toml`, `vendor/`, and runtime crate manifest.
- Unit tests for release/dev sysroot resolution separation.
- CLI smoke test for `sifr --print sysroot`.
- CLI smoke test for `sifr --print sysroot --json`.
- Installed-layout skeleton fixture that runs outside the repository.

### M2. Rename Current Compiler Stdlib Crate

Free the `sifr_stdlib` crate name for generated-program code.

Tasks:

- Rename current `crates/sifr_stdlib` to `crates/sifr_stdlib_model`.
- Update workspace dependencies and all compiler imports.
- Preserve existing responsibilities: source inventory, intrinsic metadata,
  stdlib import policy, legacy module suggestions, feature/dependency mapping,
  and IPC schema/protocol metadata.
- Update tests and docs that refer to the compiler-side crate.
- Complete the rename without leaving a legacy re-export in the closed
  milestone.

Acceptance:

- No generated user Cargo manifest depends on `sifr_stdlib_model`.
- Compiler crates depend on `sifr_stdlib_model` where they previously depended
  on current `sifr_stdlib`.
- Existing stdlib import, lowering, codegen, and analysis behavior is unchanged.
- The milestone closes with no legacy re-export remaining.

Validation:

- `cargo test -p sifr_stdlib_model`
- `cargo test -p sifr_lowering`
- `cargo test -p sifr_codegen`
- `cargo test -p sifr_driver`
- `cargo tree -i sifr_stdlib_model --workspace --edges normal`

### M3. Create Generated-Program `sifr_stdlib` Crate

Introduce the Rust-native stdlib implementation crate.

Tasks:

- Create new `crates/sifr_stdlib` as a generated-program dependency crate.
- Make it depend on `sifr_runtime` for lower-level runtime primitives.
- Define feature gates that mirror stdlib native capability groups.
- Establish public Rust modules for stdlib native leaves and adapters.
- Add crate-level no-panic and error-shaping conventions for user-triggerable
  paths.
- Make the crate compile as a sysroot workspace member with workspace-inherited
  dependency versions from the sysroot workspace manifest.
- Use `default-features = false` as the generated dependency default.
- Make every generated-facing feature additive and narrow. Leaf features include
  `json`, `regex`, `uuid`, `hash`, `base64`, `toml`, `url`, `gzip`, `zipfile`,
  `unicode`, `i18n`, `net`, `tls`, `http`, `python`, `process`, `fs`,
  `signals`, and `runtime-observability`.
- Allow umbrella features only as maintenance aliases; generated Cargo planning
  must prefer minimal leaf features.
- Avoid public re-export of runtime internals unless the API is explicitly
  owned by `sifr_stdlib`.
- Add tests for feature combinations and direct Rust API behavior.

Acceptance:

- `sifr_stdlib` is usable as a path dependency from a generated Cargo project.
- The crate name `sifr_stdlib` no longer refers to compiler-only metadata.
- Runtime primitives remain in `sifr_runtime`; stdlib-facing native operations
  live in `sifr_stdlib`.
- The crate builds both inside the repository workspace and inside an
  installed-layout sysroot workspace fixture.
- Dependency-plan expectations and `cargo tree -e features` snapshots prove
  representative modules do not enable unrelated feature groups.

Validation:

- `cargo test -p sifr_stdlib`
- `cargo clippy -p sifr_stdlib -- -D warnings`
- Feature-combination cargo checks for default, text/data, network, and Python
  groups.
- Installed-layout cargo check for `<sysroot>/crates/sifr_stdlib`.
- Dependency-plan feature expectation checks plus `cargo tree -e features`
  snapshots for representative `sifr.re`, `sifr.json`, `sifr.http`,
  `sifr.python`, and pure Sifr stdlib programs.

### M4. Full Sysroot Workspace and Source Layout

Complete sysroot validation after the crate naming hazard is gone.

Tasks:

- Enable full validation of `crates/sifr_stdlib` in `sysroot.toml` and sysroot
  boundary checks.
- Validate `cargo metadata --offline` from the installed-layout sysroot
  workspace.
- Implement exactly one canonical repository stdlib source root:
  `stdlib/sifr/*.sifr` and `stdlib/_sifr/*.sifr`.
- Add private `_sifr` declaration modules for stdlib native surfaces.
- Update `sifr_stdlib_model` source inventory to load from `ResolvedSysroot` in
  released and development workflows.
- Keep user `_sifr.*` imports rejected while allowing sysroot stdlib modules to
  import private declarations.
- Add source-location metadata so diagnostics and LSP can report physical
  sysroot paths for installed sources.

Acceptance:

- Released tools read stdlib sources from the installed sysroot.
- LSP and CLI observe the same stdlib files.
- User code cannot import `_sifr.*`.
- Existing public `sifr.*` imports continue to resolve.
- Public source inventory and private declaration inventory are generated or
  validated from the same canonical sysroot source tree.
- M4 closes with one canonical stdlib source root; no long-lived dual-root
  source layout remains.
- `cargo metadata --offline` from a complete sysroot workspace succeeds in the
  installed-layout fixture.

Validation:

- Unit tests for missing final `crates/sifr_stdlib/Cargo.toml`.
- Installed-layout fixture for offline sysroot workspace metadata.
- Single-file and package analysis tests for public stdlib imports.
- Negative tests for user `_sifr.*` imports.
- LSP tests for stdlib hover/completion/definition using sysroot files.
- Source inventory validation that fails on missing, duplicate, or stale stdlib
  modules.

### M5. Generated Cargo Uses Sysroot Crates and Vendor

Make generated Cargo projects sysroot-driven and network-independent for
Sifr-owned dependencies.

Tasks:

- Update generated Cargo manifest planning to emit path dependencies for
  `<sysroot>/crates/sifr_runtime` and `<sysroot>/crates/sifr_stdlib` with
  `default-features = false`.
- Move stdlib dependency feature mapping to `sifr_stdlib_model`.
- Add `SysrootDependencyPlan` as the single output for sysroot crate
  dependencies, feature sets, Cargo vendor mode, and cache fingerprint.
- Make generated Cargo, build reports, cache keys, LSP traces, feature
  expectations, and tests consume `SysrootDependencyPlan`.
- Invoke Cargo with sysroot vendor configuration for Sifr-managed builds where
  the mode matrix permits it. Do not copy sysroot `.cargo/config.toml` into
  user/package project directories.
- Implement an explicit Cargo config mode matrix:
  stdlib-only single-file builds may apply sysroot vendor config; package builds
  with user registry dependencies in online mode must not silently force the
  sysroot vendor over user dependencies; offline/frozen package builds must use
  a complete combined graph or fail clearly; explicit Rust interop dependencies
  remain package-owned.
- Allow the first implementation to fail clearly for offline/frozen package
  builds with user registry dependencies unless a complete combined vendor graph
  is available.
- Vendor third-party Cargo dependencies required by `sifr_runtime` and
  `sifr_stdlib`.
- Generate the sysroot workspace `Cargo.lock` from the sysroot workspace
  manifest, then vendor from that lockfile.
- Define generated-project lockfile behavior: stdlib-only generated projects
  must resolve reproducibly from the sysroot-compatible graph, and any
  generated `Cargo.lock` must be checked by offline fixtures.
- Ensure sysroot crate manifests do not depend on the development workspace
  unless running from a resolved development sysroot.
- Ensure cache keys include sysroot version, sysroot content digest, and
  selected feature sets from `SysrootDependencyPlan`.
- Record sysroot path, derived toolchain id, and sysroot content digest in
  generated build reports and trace/debug output.
- Keep user package dependencies and explicit Rust interop package dependencies
  Cargo-owned.
- Ensure package-mode generated builds do not accidentally force the sysroot
  vendor policy onto user dependencies outside documented offline/frozen modes.
- Test invocation-scoped Cargo config directly.
- Keep temporary direct third-party generated dependencies only for unmigrated
  compiler-special paths with a deletion milestone and validation evidence.

Acceptance:

- `sifr run` for stdlib-using single-file programs does not need a source
  checkout or runtime-only override.
- Generated Cargo manifests contain sysroot paths, not build-machine paths.
- Standard-library-only generated builds can run with Cargo offline when the
  sysroot vendor directory is present.
- User dependency resolution behavior remains Cargo-compatible.
- Generated Cargo projects do not require `Cargo.toml` from the Sifr source
  checkout.
- Generated project directories are not left with copied sysroot Cargo config
  that can silently rewrite later user Cargo invocations.

Validation:

- End-to-end `sifr run` fixture using `sifr.random`, `sifr.json`, and
  `sifr.re` from an installed-layout fixture.
- Offline generated-build fixture for stdlib-only code.
- Snapshot tests for generated `Cargo.toml` and invocation-scoped Cargo config.
- Snapshot or fixture for generated stdlib-only `Cargo.lock` behavior in
  offline mode.
- `SysrootDependencyPlan` snapshots for representative stdlib imports and
  language runtime requirements.
- Fixture proving a generated project fails with a Sifr sysroot diagnostic when
  the sysroot vendor directory is missing in bundled-dependency mode.
- Package-mode fixture proving user dependencies remain Cargo-owned.
- Package-mode fixture proving online user registry resolution is not silently
  replaced by the sysroot vendor config.
- Fixture covering invocation-scoped Cargo config for stdlib-only generated
  builds.
- Offline/frozen package-mode fixture proving user registry dependencies either
  use a complete combined vendor graph or fail with a clear diagnostic.
- Snapshot proving migrated stdlib leaves do not emit direct third-party
  implementation dependencies.
- Build-report snapshot containing sysroot identity.

### M6. Distribution Artifact and Installer Update

Ship the sysroot as part of the toolchain.

Tasks:

- Change preview/release archives to include `sifr` and `lib/sifr/**`.
- Update installer extraction validation to require binary, sysroot manifest,
  sysroot workspace manifest/lockfile, stdlib roots, sysroot crates, vendor
  root, and Cargo config.
- Replace binary and sysroot as one toolchain update under the existing install
  lock.
- Extend install receipts with `sysroot_path` and sysroot schema/version data.
- Update self-update to preserve the binary/sysroot pairing.
- Update distribution docs and release verification scripts.
- Add archive-content checks for every target artifact before checksums are
  published.
- Add installer staging so partial extraction or copy failure cannot commit a
  mismatched binary/sysroot pair.
- Add OS-specific installer hardening for Windows executable replacement,
  cross-device rename fallback, executable permission preservation, symlink
  rejection/canonicalization, partial extraction cleanup, receipt atomic write,
  and old sysroot cleanup only after success.

Acceptance:

- A fresh install contains a complete sysroot.
- Self-update cannot leave a new binary paired with an old sysroot or vice
  versa.
- Archive verification catches missing sysroot assets before publication.
- `sifr self update` updates binary, sysroot, and receipt as a single managed
  installation.

Validation:

- Distribution release validation cases for archive content.
- Installer fixture for fresh install and update.
- Self-update receipt schema tests.
- Broken archive fixtures for missing sysroot manifest, missing runtime crate,
  missing stdlib crate, missing vendor, and missing Cargo config.
- Cross-target preview-release workflow validation.
- Platform-specific installer fixtures or documented simulation coverage for
  replacement, permissions, symlinks, and cleanup behavior.

### M7. LSP and Tooling Sysroot Source/Navigation Integration

Make editor and analysis source/navigation surfaces use the installed sysroot.
Proactive sysroot mismatch diagnostics and production generated/synthetic origin
emission are split to M7b so this PR stays reviewable.

Tasks:

- Load stdlib source and private declaration metadata from `ResolvedSysroot`.
- Add stdlib source locations to the symbol index.
- Support hover, completion, definition, and type-definition for public stdlib
  files.
- Keep private `_sifr` declarations visible to sysroot implementation analysis
  but not user import completion.
- Add source origin kinds: `UserSource`, `SysrootPublicStdlib`,
  `SysrootPrivateDeclaration`, `GeneratedSupport`, and `CompilerSynthetic`.
- Add an inspectable LSP sysroot status request that reports the same resolved
  sysroot root/toolchain id as analysis/CLI resolution for the current process.
- Make go-to-definition prefer public wrappers for user code and expose private
  declaration links only in internal/developer contexts.

Acceptance:

- LSP go-to-definition for a `sifr.*` import lands in installed sysroot source.
- LSP go-to-definition for a `sifr.*` call site lands in installed sysroot
  source when the binding comes from a public stdlib import.
- Hover and completion reflect the installed stdlib version.
- `_sifr.*` internals are not offered to user code as public modules.
- LSP exposes the resolved sysroot root/toolchain id through `sifr/sysroot`.
- Source maps correctly distinguish user files, public stdlib files, private
  declaration files, and reserve generated/synthetic origin kinds for M7b
  production emission.

Validation:

- LSP request tests for hover/completion/definition/type-definition against
  sysroot fixtures.
- Negative completion tests proving private declarations do not appear in user
  import completions.
- Source-map origin tests for user files, public stdlib files, private
  declarations, generated support, and compiler synthetic source variants.
- Analysis test for internal public-stdlib source navigation to private
  declaration files.

### M7b. Tooling Sysroot Diagnostics and Synthetic Origins

Close the editor-visible sysroot diagnostics and production synthetic-origin
pieces split from M7.

Tasks:

- Add tooling diagnostics when the editor process sees a broken or mismatched
  sysroot.
- Add development sysroot behavior tests proving local LSP sessions use the same
  resolved sysroot as CLI when running from an unreleased build.
- Add CLI/LSP sysroot mismatch diagnostics that include observed sysroot paths
  where available.
- Emit `GeneratedSupport` and `CompilerSynthetic` source origins from real
  production source-map paths rather than only defining the enum variants.

Acceptance:

- CLI and LSP report the same sysroot path for the same installation and expose
  actionable diagnostics when they do not.
- Broken-sysroot editor diagnostics include the resolver-observed binary,
  attempted sysroot, and invalid asset paths where available.
- Source maps include production files tagged as `GeneratedSupport` and
  `CompilerSynthetic` when those sources are present.

Validation:

- LSP-level tests for broken-sysroot diagnostics and mismatch diagnostics.
- Development-sysroot LSP/CLI path equivalence test.
- Production-path source-map tests for generated support and compiler synthetic
  origins.

### M8. Rust Interop Context for Private Stdlib Declarations

Give private stdlib declarations a normal interop context.

Tasks:

- Add a compiler-owned synthetic package context for sysroot stdlib interop.
- Feed private stdlib declarations through normal Rust interop contracts:
  `InteropBuildPlan`, trust, probes, bridge generation, dependency planning,
  cache keys, direct calls, opaque handles, async calls, callbacks, views, and
  error conversion.
- Define the sysroot trust policy for Sifr-owned sysroot crates.
- Define stdlib-private rules only for declaration location, import permission,
  canonical sysroot crate targets, user shadowing prevention, and diagnostic
  attribution.
- Ensure user packages cannot impersonate or override private sysroot
  declarations.
- Add verification for cache invalidation when private declarations or sysroot
  crates change.
- Ensure stdlib interop probes use sysroot Cargo config and vendor data.
- Test that private declaration diagnostics point back to sysroot declaration
  source without exposing `_sifr` as public API.

Acceptance:

- Private `_sifr` declarations can target `sifr_stdlib` and `sifr_runtime`
  through Rust interop without requiring user package manifests.
- Build planning is deterministic and cache-safe.
- User code cannot directly access private declarations.
- Stdlib interop builds work from installed layout with no source checkout.

Validation:

- Unit tests for synthetic stdlib interop context.
- Cache-key tests for sysroot source/crate changes.
- Negative tests for user `_sifr` access and shadowing.
- Offline probe fixture for stdlib interop declaration checks.
- Rust interop contract fixtures for allowed/prohibited declaration shapes under
  the sysroot trust policy.

M8 implementation evidence:

- Private stdlib declarations are collected from installed/private `_sifr`
  sources and merged into the generated interop plan with a synthetic sysroot
  package context.
- Private declarations can target only canonical `sifr_stdlib` and
  `sifr_runtime` sysroot crates; non-sysroot roots fail before probing.
- Merged user plus sysroot interop contexts validate declaration privacy by the
  resolved package id, preventing user `_sifr` impersonation without rejecting
  normal user interop.
- Sysroot interop dependency planning injects required sysroot crates without
  switching to package-owned vendor mode and records those injected crates in
  the dependency-plan cache fingerprint.
- Opus review pass 2 returned `VERDICT: PASS`; focused validation passed for
  formatting, diff whitespace, file-size guardrails, driver/codegen check,
  sysroot interop tests, sysroot probe tests, and `sifr_codegen`/`sifr_driver`
  unit/doc tests.

### Native Migration Contract

Every migrated native stdlib surface must:

- preserve public `stdlib/sifr` wrapper behavior,
- route through `stdlib/_sifr` private declaration, normal Rust interop, and a
  sysroot crate,
- select only the minimal sysroot crate features described by
  `SysrootDependencyPlan`,
- avoid direct third-party implementation dependencies in generated Cargo,
- avoid data-dependent `unwrap()` and `expect()` in user-triggerable paths,
- include parity, e2e, and generated-shape evidence,
- delete old compiler-special dispatch or enter the retained
  compiler-language-glue allowlist.

M9, M10, and M11 choose migration order. This contract defines the common
completion bar for every migrated surface.

### M9. Migrate Stateless Native Leaves

Remove low-risk compiler-special native stdlib plumbing first.

Wave 1 status: `_sifr.platform` and `_sifr.html` are migrated to private
`@rust(sifr_stdlib.*)` declarations backed by narrow `platform` and `html`
features in `sifr_stdlib`. Public wrappers still expose the existing
`sifr.platform` and `sifr.html` APIs, including public-module re-exported
private leaf names, but codegen no longer routes these leaves through active
intrinsic dispatch. Sysroot Rust probes now enable declared `sifr_stdlib`
features for feature-gated private targets. The wave also records the adjacent
Python interop integration fix exposed by native-link evidence validation:
generated projects that combine sysroot Rust interop with embedded Python trust
only the selected packaged runtime's `libpython` link, and Python example
runners normalize relative `CARGO_TARGET_DIR` before invoking generated package
builds from package working directories. Sysroot Rust probe manifests now match
generated manifests by disabling `sifr_stdlib` default features, and the e2e
fixture harness emits the matching feature-gated `sifr_stdlib` dependency for
migrated `html`/`platform` modules.

Tasks:

- Move math leaves to private declarations backed by `sifr_stdlib`. (wave 4 complete)
- Move base64/base32, hash, regex, and TOML leaves. (hash wave 5 complete;
  base64/base32 encoder wave 6 merged in PR #2767; fallible base64/base32
  decode/options migrated in M10 wave 1; regex and TOML are deferred to the
  typed error bridge follow-up waves)
- Move UUID leaf. (wave 3 complete)
- Move calendar leaf. (wave 2 complete)
- Move HTML and platform leaves. (wave 1 complete)
- Update `sifr_stdlib` feature groups and `SysrootDependencyPlan` mapping as
  each leaf migrates. (wave 5 complete for `hash`; wave 4 complete for
  `math`; wave 3 complete for `uuid`; wave 2 complete for `calendar`; wave 1
  complete for `html` and `platform`)
- Add explicit unsupported-by-design notes for any stateless leaf deferred due
  to type-system or interop limitations.

Wave 1 implementation evidence:

- `stdlib/_sifr/platform.sifr` and `stdlib/_sifr/html.sifr` declare private
  `@rust(sifr_stdlib.platform.*)` / `@rust(sifr_stdlib.html.*)` leaves.
- `crates/sifr_stdlib/src/platform.rs` and `crates/sifr_stdlib/src/html.rs`
  own the Rust implementations behind `platform` and `html` Cargo features.
- `sifr_codegen` no longer registers platform/html names in the active
  intrinsic dispatch table; registry tests assert these names do not lower as
  compiler intrinsics.
- `sifr_stdlib_model` keeps platform/html intrinsic-module signatures only as
  stdlib-lowering bootstrap metadata until later M9 waves remove the remaining
  dependency on that fallback; those entries are no longer active codegen
  lowerers.
- The e2e fixture harness now adds one `sifr_stdlib` path dependency with
  `default-features = false` and the migrated leaf features for fixtures that
  import `sifr.html`, `_sifr.html`, `sifr.platform`, or `_sifr.platform`.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_driver stateless_private_codegen_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_driver python_runtime --locked`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_driver python_runtime_libpython_link_is_trusted_when_interop_validation_runs --locked`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_driver sysroot_probe_manifest -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_driver sysroot_stdlib_probe_features_follow_target_module_segment -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_codegen lowers_platform_intrinsics_via_registry -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_codegen lowers_html_intrinsics_via_registry -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules -- --nocapture`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo run -q -p sifr -- run demos/platform/main.sifr`;
  `CARGO_TARGET_DIR=target/m9-stateless-leaves cargo run -q -p sifr -- run demos/html_and_textwrap/main.sifr`;
  `CARGO_TARGET_DIR=target/m9-create-pr CARGO_BUILD_JOBS=1 uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite self-test --suite scaffold --suite env --suite tier1 --suite callbacks --suite dataframes --suite ml --suite libraries --suite cloud-boto3`.
- Local create-pr validation passed with zero failures across freshly written
  area results: `CARGO_TARGET_DIR=target/m9-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
- Opus review passes 3, 4, and 5 returned `VERDICT: PASS`; non-blocking feedback was
  addressed by documenting/testing sysroot probe feature derivation,
  documenting the retained stdlib-model bootstrap signatures, and adding the
  Python-runtime/native-link follow-up validation plus probe manifest
  `default-features = false` parity before the final wave review; pass 5
  separately reviewed the e2e fixture `sifr_stdlib` dependency emission.

Wave 2 status: `_sifr.calendar` is migrated to private
`@rust(sifr_stdlib.calendar.*)` declarations backed by the narrow `calendar`
feature in `sifr_stdlib`. The active compiler intrinsic registry no longer
owns calendar lowering. Direct Rust interop wrappers now bridge Sifr `int`
arguments through `sifr_runtime::interop::SifrIntBridge` and convert returned
bridge integers, including `list[int]`, back to generated-code integer values at
the wrapper boundary. Sysroot probe bridge stubs now split dotted module names
such as `_sifr.calendar` into nested Rust modules.

Wave 2 implementation evidence:

- `stdlib/_sifr/calendar.sifr` declares private
  `@rust(sifr_stdlib.calendar.*)` leaves for `calendar_isleap`,
  `calendar_weekday`, and `calendar_monthrange`.
- `crates/sifr_stdlib/src/calendar.rs` owns the Gregorian helper behavior
  behind the `calendar` Cargo feature and uses the runtime `SifrIntBridge`
  boundary type.
- `sifr_codegen` no longer registers calendar names in the active intrinsic
  dispatch table; registry tests assert these names do not lower as compiler
  intrinsics.
- Direct Rust interop codegen bridges `int` arguments/returns and `list[int]`
  returns for feature-gated stdlib targets, with runtime coverage for
  saturating bridge-to-`i64` conversion.
- The e2e fixture harness enables the `calendar` feature for fixtures that
  import `sifr.calendar` or `_sifr.calendar`.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_runtime exact_integer_bridge --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen direct_rust_function_body --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen lowers_calendar_intrinsics_via_registry --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_driver stateless_private_codegen_tests --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_driver generated_bridge_type_stubs_split_dotted_module_names --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features calendar calendar_leaf_matches_gregorian_helpers --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_calendar.sifr`;
  `CARGO_TARGET_DIR=target/m9-calendar CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_calendar_subset.sifr`;
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`.
- Local create-pr validation passed with zero failures:
  `CARGO_TARGET_DIR=target/m9-calendar-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported the existing warm wall-time budget advisory only.
- Opus review passes 1 and 2 returned `VERDICT: PASS`. Pass 2 reviewed the
  follow-up `i128` weekday arithmetic hardening for extreme `i64` years.

Wave 3 status: `_sifr.uuid` is migrated to private
`@rust(sifr_stdlib.uuid.*)` declarations backed by the narrow `uuid` feature in
`sifr_stdlib`. The active compiler intrinsic registry no longer owns UUID
lowering. Generated Cargo fixture support now emits one `sifr_stdlib`
dependency for UUID fixtures instead of direct `rand` and `uuid` dependencies.

Wave 3 implementation evidence:

- `stdlib/_sifr/uuid.sifr` declares private
  `@rust(sifr_stdlib.uuid.*)` leaves for `uuid4`, `uuid3_text`, and
  `uuid5_text`.
- `crates/sifr_stdlib/src/uuid.rs` owns random and name-based UUID behavior
  behind the `uuid` Cargo feature, with workspace `uuid` crate v4 support
  enabled for `uuid4`.
- `sifr_codegen` no longer registers UUID names in the active intrinsic
  dispatch table; registry tests assert these names do not lower as compiler
  intrinsics.
- The e2e fixture harness enables the `uuid` feature for fixtures that import
  `sifr.uuid` or `_sifr.uuid` and no longer emits direct `rand`/`uuid` fixture
  dependencies for that module.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen lowers_uuid_intrinsic_via_registry --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr_driver uuid_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features uuid uuid_leaf_matches_public_uuid_helpers --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_uuid_consolidated.sifr`;
  `CARGO_TARGET_DIR=target/m9-uuid CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`;
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`.
- Local create-pr validation passed with zero failures:
  `CARGO_TARGET_DIR=target/m9-uuid-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported warm wall-time and cache-hit advisories only.
- Opus review pass 1 returned `VERDICT: PASS` for the UUID migration.

Wave 4 status: `_sifr.math` is migrated to private
`@rust(sifr_stdlib.math.*)` declarations backed by the narrow `math` feature in
`sifr_stdlib`. The active compiler intrinsic registry no longer owns math
lowering. Public aggregate helpers that accept lists keep borrowed public
semantics in `stdlib/sifr/math.sifr` and copy into private owned-vector bridge
helpers so Rust interop ownership does not leak into the public API.

Wave 4 implementation evidence:

- `stdlib/_sifr/math.sifr` declares private
  `@rust(sifr_stdlib.math.*)` leaves for scalar math helpers, predicates,
  constants, aggregate helpers, gamma/error functions, and decomposition
  helpers.
- `crates/sifr_stdlib/src/math.rs` owns the math helper behavior behind the
  `math` Cargo feature and uses `sifr_runtime::interop::SifrIntBridge` for
  Sifr `int` boundary returns/arguments.
- `stdlib/sifr/math.sifr` preserves the public `dist`, `fsum`, and `sumprod`
  borrowed-list API with wrappers around internal private bridge helpers
  `dist_impl`, `fsum_impl`, and `sumprod_impl`.
- `sifr_codegen` no longer registers math names in the active intrinsic
  dispatch table; registry tests assert core and extended math names do not
  lower as compiler intrinsics.
- The e2e fixture harness enables the `math` feature for fixtures that import
  `sifr.math` or `_sifr.math`.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_driver math_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_frontend frontend_export_policy_hides_math_bridge_helpers --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features math math_leaf_matches_public_math_helpers --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen math_intrinsics_are_owned_by_compiled_stdlib_declarations --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen lowers_extended_math_intrinsics_via_registry --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_import_test.sifr`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`;
  `CARGO_TARGET_DIR=target/m9-math CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_math_parity_expanded_matrix.sifr`.
- Opus review pass 2 returned `VERDICT: PASS` after mirroring the math bridge
  helper export filter into frontend query diagnostics.
- Local create-pr validation passed with zero failures:
  `scripts/run_all_tests.sh --profile create-pr`. The run reported the warm
  wall-time advisory only.

Wave 5 status: `_sifr.crypto` hash helpers used by `sifr.hashlib` are migrated
to private `@rust(sifr_stdlib.hash.*)` declarations backed by the narrow
`hash` feature in `sifr_stdlib`. The active compiler intrinsic registry no
longer owns SHA, MD5, or Blake2 hash lowering. Because `_sifr.crypto` is shared
with base64/base32 and random helpers that have not migrated yet, bootstrap now
re-exports missing intrinsic fallbacks for names that the compiled private
module does not provide.

Wave 5 implementation evidence:

- `stdlib/_sifr/crypto.sifr` declares private
  `@rust(sifr_stdlib.hash.*)` leaves for string and byte forms of SHA-256,
  MD5, SHA-1, SHA-224, SHA-384, SHA-512, Blake2b, and Blake2s helpers.
- `crates/sifr_stdlib/src/hash.rs` owns the hash helper behavior behind the
  `hash` Cargo feature, returning lowercase hex strings for text helpers and
  raw digest byte vectors for bytes helpers.
- `stdlib/sifr/hashlib.sifr` keeps public owned `str`/`bytes` helper APIs by
  wrapping private underscored aliases such as `_sha256_impl`, so borrowed Rust
  interop parameter conventions do not leak into generated public call sites.
- `crates/sifr_driver/src/stdlib/bootstrap.rs` preserves partial `_sifr.crypto`
  migration by adding intrinsic fallback declarations only for requested names
  that compiled private exports did not provide.
- `sifr_codegen` no longer registers hash names in the active intrinsic
  dispatch table; registry tests assert core and extended hash names do not
  lower as compiler intrinsics.
- The e2e fixture harness enables the `hash` feature for fixtures that import
  `sifr.hash`, `sifr.hashlib`, or `_sifr.crypto` and no longer emits direct
  hash-crate dependencies for `sifr.hashlib` fixtures.
- Rust interop probes now normalize inherited relative `CARGO_TARGET_DIR`
  values against the original compiler invocation directory before running
  temp-project cargo probes, so local validation lanes reuse the intended shared
  target directory instead of rebuilding probe dependencies under
  `/tmp/sifr_rust_probe_*`.
- Focused validation passed:
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features hash hash_leaf_matches_known_digest_vectors --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen hash_intrinsics --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_driver rust_interop_probe --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_driver crypto_hash_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/stdlib_hash.sifr`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/bytes_hashing_and_base64.sifr`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/stdlib_hashlib_intrinsics.sifr`;
  `CARGO_TARGET_DIR=$(pwd)/target/m9-hash CARGO_BUILD_JOBS=1 target/m9-hash/debug/sifr run crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr`;
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`.
- Opus review pass 1 returned `VERDICT: PASS` for the hash migration,
  including the public `sifr.hashlib` wrapper aliases and partial
  `_sifr.crypto` fallback behavior.
- Local create-pr validation passed with zero failures:
  `CARGO_TARGET_DIR=target/m9-hash-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported the warm wall-time advisory only.

Wave 6 status: the infallible base64/base32 encoder subset of
`_sifr.crypto` is migrated to private `@rust(sifr_stdlib.base64.*)`
declarations backed by the narrow `base64` feature in `sifr_stdlib`. The
active compiler intrinsic registry no longer owns `base64_encode`,
`base64_encode_bytes`, `urlsafe_b64encode`, `urlsafe_b64encode_bytes`,
`b32encode`, or `b32hexencode`. At the end of wave 6, fallible base64/base32
decode and option helpers still used compiler intrinsic fallback declarations;
M10 wave 1 below migrates them through typed result-error direct interop.

Wave 6 implementation evidence:

- `stdlib/_sifr/crypto.sifr` declares private
  `@rust(sifr_stdlib.base64.*)` leaves for the infallible base64, URL-safe
  base64, base32, and base32hex encoders.
- `stdlib/sifr/base64.sifr` wraps migrated encoder imports with private
  underscored aliases such as `_base64_encode_impl`, while re-exporting
  fallback decoder/option helpers under their original names so active
  intrinsic dispatch still owns the fallible API surface.
- `crates/sifr_stdlib/src/base64.rs` owns the migrated encoder behavior behind
  the `base64` Cargo feature and carries parity coverage for the full Rust
  helper module, including decode/error helpers that remain inactive at the
  Sifr interop boundary until M10.
- `sifr_codegen` no longer registers the migrated encoder names in active
  intrinsic dispatch; registry tests assert decoder/option names remain
  fallback intrinsics with M10 rationale.
- The e2e fixture harness enables the `base64` feature for fixtures that import
  `sifr.base64` or `_sifr.crypto`, while preserving the direct `base64` crate
  dependency required by fallback decode/option lowering during the partial
  migration.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features base64 --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen base_encoding_intrinsics_are_owned_by_compiled_stdlib_declarations --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr_driver crypto_hash_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_base64_strictness_subset.sifr`;
  `CARGO_TARGET_DIR=target/m9-base-encoding CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/base64_bytes_decode_errors.sifr`.
- Opus review pass 5 returned `VERDICT: PASS` for the base64/base32 encoder
  migration boundary, including the M10 deferral for fallible decode/option
  helpers and the temporary direct `base64` dependency retained for fallback
  lowering.
- Local create-pr validation passed with zero failures:
  `CARGO_TARGET_DIR=target/m9-base-encoding-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported the warm wall-time advisory only.

Acceptance:

- Each stateless leaf satisfies the Native Migration Contract.
- Unsupported-by-design stateless leaves are explicitly recorded with rationale.

Validation:

- Focused fixtures for every migrated module.
- Code search check that migrated functions no longer appear in active
  intrinsic lowering dispatch tables.
- Feature-delta snapshots for each migrated module.

### M10. Migrate Fallible Data and Text Modules

Move fallible native stdlib operations through typed interop boundaries.

Tasks:

- Migrate JSON, encoding, Unicode, i18n, URL validation/building, gzip,
  zipfile, and compression helpers.
- Standardize bridge error types and conversion to Sifr Result/error classes.
- Remove duplicated generated preamble where runtime crate functions should own
  behavior.
- Ensure large-data and malformed-input cases have bounded-memory and bounded
  diagnostic behavior where applicable.

Wave 1 status: merged in [PR #2769](https://github.com/sifr-lang/sifr/pull/2769).
The fallible base64/base32 decode and option subset of
`_sifr.crypto` is migrated to private `@rust(sifr_stdlib.base64.*)`
declarations backed by the narrow `base64` feature in `sifr_stdlib`. Direct
Rust interop now maps Rust `Result[..., E: Display]` return errors into
message-shaped Sifr error classes when the Sifr return type is
`Result[..., ErrorSubclass]`; this wave exercises that bridge with
`ParseError`. The active compiler intrinsic registry no longer owns any
base64/base32 encoder, decoder, or option helper, and the duplicate
`registry/base64.rs` / `registry/base32.rs` lowerers are deleted.

Wave 1 implementation evidence:

- `stdlib/_sifr/crypto.sifr` declares fallible base64, URL-safe base64, base32,
  and base32hex decode/option helpers as private
  `@rust(sifr_stdlib.base64.*)` leaves.
- `stdlib/sifr/base64.sifr` wraps every migrated private base encoding helper
  behind public functions so borrowed Rust interop parameter conventions do not
  leak into user call sites.
- `crates/sifr_codegen/src/rust_interop_direct.rs` maps direct Rust
  `Result` returns through existing ok-value bridge conversions and constructs
  message-shaped Sifr error classes from Rust errors via `to_string()`.
- Rust interop signature probes accept direct Rust result errors that implement
  `Display` when the Sifr return error is a generated error bridge type, and
  probe bridge stubs now follow sanitized `__sifr_bridge` module paths.
- `crates/sifr_stdlib/src/base64.rs` accepts `SifrIntBridge` for `wrapcol`,
  matching direct interop `int` argument lowering for `base64_encode_opts`.
- `sifr_stdlib_model` no longer retains a direct generated `base64` crate
  dependency for `sifr.base64`; generated projects depend on
  `sifr_stdlib` with `features = ["base64"]` instead.
- `internal_docs/typescript_go_architecture_transfer_guardrails.md` inventories
  the intentional Rust interop probe manifest checks/read added while enabling
  sysroot feature-aware direct probes.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen direct_rust_function_body_maps_result_error_return --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen base_encoding_intrinsics_are_owned_by_compiled_stdlib_declarations --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_driver generated_bridge_type_stubs_follow_sanitized_bridge_type_paths --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_driver result_error_bridge_return_probes_display_error_generic --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_driver crypto_hash_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features base64 --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/cpython_base64_strictness_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/base64_bytes_decode_errors.sifr`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/stdlib_encoding.sifr`;
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge CARGO_BUILD_JOBS=1 target/m10-base64-error-bridge/debug/sifr run crates/sifr/tests/e2e/pass/parse_safety_error_paths.sifr`;
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`.
- Opus review pass 1 returned `VERDICT: PASS` for the M10 wave 1
  implementation.
- Local create-pr validation passed:
  `CARGO_TARGET_DIR=target/m10-base64-error-bridge-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The runner reported only the warm wall-time budget advisory.

Wave 2 status: merged in [PR #2771](https://github.com/sifr-lang/sifr/pull/2771).
The `_sifr.regex` private module and public `sifr.re` wrappers are migrated to
private `@rust(sifr_stdlib.regex.*)` declarations backed by the narrow
`regex` feature in `sifr_stdlib`. Direct Rust interop now also maps
`Result[..., E: Display]` errors into Sifr error subclasses that carry a
`message` field plus additional string detail fields, which preserves
`RegexError { message, detail }` without leaking the Rust regex crate type. The
active compiler intrinsic registry no longer owns the public regex helpers
used by `sifr.re`. The generated dependency planner still retains a direct
`regex` crate dependency for `sifr.pathlib`, whose path-glob lowering remains a
separate compiler-special path until that surface migrates.

Wave 2 implementation evidence:

- `stdlib/_sifr/regex.sifr` declares regex match, find, replace, findall,
  split, start/end, and flag variants as private
  `@rust(sifr_stdlib.regex.*)` leaves.
- `stdlib/sifr/re.sifr` wraps each private declaration behind public functions
  and aliases imports with underscored names so borrowed private interop
  conventions stay out of public call sites.
- `crates/sifr_stdlib/src/regex.rs` owns regex behavior behind the `regex`
  feature, including CPython-compatible flag bits used by the previous
  intrinsic implementation.
- `crates/sifr_codegen/src/rust_interop_direct.rs` extends the typed
  result-error bridge for error subclasses whose fields are all strings,
  populating `message` and `detail` from the Rust error display text.
- `sifr_stdlib_model` no longer retains a direct generated `regex` crate
  dependency for `sifr.re` or `_sifr.regex`; generated projects depend on
  `sifr_stdlib` with `features = ["regex"]` instead. `sifr.pathlib` remains
  the intentional direct-regex exception.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features regex --locked regex_leaf_matches_public_re_helpers`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_driver regex_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen re_intrinsics_are_owned_by_compiled_stdlib_declarations --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen direct_rust_function_body_maps_string_error_fields --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo build -p sifr --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model planned_sysroot_stdlib_features_are_minimal_for_representative_modules --locked`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/stdlib_re_consolidated.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/cpython_re.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/parse_safety_error_paths.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/panic_free_stdlib_errors.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/regex_filesystem_iterators.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`;
  `CARGO_TARGET_DIR=target/m10-regex-interop CARGO_BUILD_JOBS=1 target/m10-regex-interop/debug/sifr run crates/sifr/tests/e2e/pass/error_subclass_handling.sifr`;
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`.
- Opus review pass 1 returned `VERDICT: PASS` for the M10 wave 2 regex
  interop migration.

Wave 3 status: completed and merged in [PR #2776](https://github.com/sifr-lang/sifr/pull/2776).
The `_sifr.url` private module and public `sifr.url` wrappers are migrated to
private `@rust(sifr_stdlib.url.*)` declarations backed by the narrow `url`
feature in `sifr_stdlib`. Public `Url` and `UrlQuery` records remain defined in
`stdlib/sifr/url.sifr`; the private Rust bridge returns flat `list[str]`
payloads for URL parts and query pairs because generated tuple/record bridge
types are not sysroot crate API. The public wrapper reconstructs the typed
public objects and translates private `ParseError` bridge failures into public
`UrlError`. The active compiler intrinsic registry no longer owns URL parse,
build, percent, path-normalization, or query helpers; HTTP helpers in the same
legacy file remain active until the runtime/resource HTTP surface migrates.

Wave 3 implementation evidence:

- `stdlib/_sifr/url.sifr` declares URL parse/build, percent encode/decode,
  path normalization, and query parse/build helpers as private
  `@rust(sifr_stdlib.url.*)` leaves returning `ParseError` for fallible bridge
  calls.
- `stdlib/sifr/url.sifr` wraps private URL helpers, preserves public
  `UrlError`, `Url`, and `UrlQuery`, and reconstructs public values from flat
  bridge payloads.
- `crates/sifr_stdlib/src/url.rs` owns URL behavior behind the `url` feature,
  including URL size bounds, non-ASCII host guardrails, percent validation,
  IPv4/IPv6 host handling, path normalization, and query pair serialization.
- `crates/sifr_codegen/src/rust_interop_direct.rs` now maps direct Rust
  `int | None` arguments through `SifrIntBridge` and clones `str | None`
  arguments so private Rust declarations can accept `Option<SifrIntBridge>` and
  `Option<String>` at the sysroot crate boundary.
- `sifr_stdlib_model` no longer retains a direct generated `url` or
  `percent-encoding` crate dependency for `sifr.url` or `_sifr.url`; generated
  projects depend on `sifr_stdlib` with `features = ["url"]` instead.
- E2E grouped-crate dependency inference now treats migrated `_sifr.url` and
  `_sifr.regex` adapter calls as `sifr_stdlib` feature dependencies, matching
  the sysroot dependency planner used by normal generated projects.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features url --locked url_leaf_matches_public_url_helpers`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies --locked`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen url_intrinsics_are_owned_by_compiled_stdlib_declarations --locked`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen direct_rust_function_body_converts_optional_int_arguments --locked`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_driver url_private_declarations_codegen_through_sifr_stdlib --locked`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo build -p sifr --locked`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 target/m10-url-interop/debug/sifr run crates/sifr/tests/e2e/pass/network_http_url_query_percent.sifr`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr --locked test_generate_cargo_toml_migrated_url_regex_modules_enable_stdlib_features`;
  `CARGO_TARGET_DIR=target/m10-url-interop CARGO_BUILD_JOBS=1 cargo test -p sifr --locked test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features`;
  single-fixture create-pr e2e batch harness for `network_http_url_query_percent`.
- Runtime platform validation exposed that `install-distribution-smoke` used an
  internal 60s subprocess timeout even though the manifest owns evidence
  budgets. The evidence runner now passes the manifest timeout into built-ins
  and the install/distribution smoke budget is `300s`; focused runtime platform
  create-pr suites passed with `variants=28`, `failures=0`, `skipped=1`.
- Create-pr validation status: the first full rerun after stash restoration
  failed only in e2e because the grouped `network_http_url_query_percent` crate
  omitted `sifr_stdlib`; that root cause is fixed and the full create-pr e2e
  pass now reports `132 passed, 0 failed`. A subsequent full
  `scripts/run_all_tests.sh --profile create-pr` run passed through Python
  interop and generated-code quality but was reaped mid-crate-smoke without a
  failure report; the remaining create-pr tail was validated directly:
  `sifr_lsp`, `sifr_package`, `sifr_stdlib_model`, default and feature
  `sifr_stdlib`, default and HTTP `sifr_runtime`, `sifr --bin sifr`,
  `sifr_driver --lib`, runtime platform create-pr suites, and full create-pr
  e2e all passed. Final hygiene passed: `cargo fmt --check`, `git diff
  --check`, `python3 scripts/check_file_size_guardrails.py`, and
  `python3 scripts/check_hir_maintainability_guardrails.py`.

Wave 4 status: merged in [PR #2778](https://github.com/sifr-lang/sifr/pull/2778).
The `_sifr.toml` private module and public `sifr.tomllib` wrappers are
migrated to private `@rust(sifr_stdlib.toml.*)` declarations backed by the
narrow `toml` feature in `sifr_stdlib`. Public `TomlValue` remains defined in
`stdlib/sifr/tomllib.sifr`; the private Rust bridge returns a flat `list[str]`
token stream for parsed TOML values because generated class/record bridge
types are not sysroot crate API. The public wrapper reconstructs `TomlValue`
objects and translates private `ParseError` bridge failures into public
`TOMLDecodeError`.

Wave 4 implementation evidence:

- `stdlib/_sifr/toml.sifr` declares `toml_parse_tokens` as a private
  `@rust(sifr_stdlib.toml.toml_parse_tokens)` leaf returning
  `Result[list[str], ParseError]`.
- `stdlib/sifr/tomllib.sifr` reconstructs public `TomlValue` values from flat
  bridge tokens and preserves public `loads`, `load`, and `load_handle` error
  shapes.
- `crates/sifr_stdlib/src/toml.rs` owns TOML parsing behind the `toml` feature,
  preserves table ordering through the sysroot crate dependency, bounds input
  and bridge payload size, and reports malformed input through `Result`.
- `sifr_codegen` no longer registers the legacy `toml_parse` active intrinsic
  and deletes the old TOML intrinsic lowerer.
- `sifr_stdlib_model` no longer retains a direct generated `toml` crate
  dependency for `sifr.tomllib` or `_sifr.toml`; generated projects depend on
  `sifr_stdlib` with `features` containing `"toml"` instead.
- E2E grouped-crate dependency inference now treats migrated TOML modules as
  `sifr_stdlib` feature dependencies, matching the sysroot dependency planner.
- Focused validation passed:
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features toml --locked toml_leaf -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen --locked toml_intrinsic_is_owned_by_compiled_stdlib_declaration -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_driver --locked toml_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model --locked stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model --locked planned_sysroot_stdlib_features_are_minimal_for_representative_modules -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr --locked test_generate_cargo_toml_tomllib_uses_stdlib_toml_feature -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_driver --locked test_generate_test_runner_cargo_toml_uses_stdlib_toml_feature -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo test -p sifr_driver --locked test_build_project_includes_reachable_support_module_stdlib_crates_in_manifest -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 cargo build -p sifr --locked`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 target/m10-toml-interop/debug/sifr run crates/sifr/tests/e2e/pass/stdlib_tomllib.sifr`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 target/m10-toml-interop/debug/sifr run crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 target/m10-toml-interop/debug/sifr run crates/sifr/tests/e2e/pass/structured_data_formats.sifr`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 target/m10-toml-interop/debug/sifr run crates/sifr/tests/e2e/pass/parse_safety_error_paths.sifr`;
  `CARGO_TARGET_DIR=target/m10-toml-interop CARGO_BUILD_JOBS=1 target/m10-toml-interop/debug/sifr run crates/sifr/tests/e2e/pass/panic_free_stdlib_errors.sifr`.
- Opus review pass 1 returned `VERDICT: PASS` with a non-blocking ownership
  registry wording note; pass 2 returned `VERDICT: PASS` after that wording
  was updated. Final pass 3 returned `VERDICT: PASS` with no blockers after
  the create-pr evidence and stale-LSP diagnosis were recorded.
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-toml-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-toml-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported only the expected warm wall-time advisory and wrote
  `target/validation_lane_reports/create-pr.latest.json`; slowest validated
  steps were Python interop (`1637401ms`), generated-code quality
  (`909652ms`), crate tests (`855000ms`), runtime platform (`607495ms`), and
  e2e pass (`371530ms`).
- The long pre-PR stall was diagnosed as validation state, not TOML logic: an
  earlier create-pr run had a stale core-language failure report after a
  rebuild; the next run reached developer tooling but launched stale
  `target/debug/sifr` for LSP smoke, producing `analysis is unavailable`
  errors. Rebuilding/pinning the LSP command to
  `target/m10-toml-create-pr/debug/sifr lsp --stdio` made the focused
  `developer_tooling` static/LSP suites pass before the full create-pr rerun.

Wave 5 status: in progress on branch `m10-json-interop`.
The `_sifr.json` private module and public `sifr.json` wrappers are migrated
off the active JSON intrinsic registry and onto private
`@rust(sifr_stdlib.json.*)` declarations backed by the narrow `json` feature
in `sifr_stdlib`. The public `JsonValue` class remains Sifr-owned; the private
Rust bridge accepts and returns flat `list[str]` token streams so generated
Sifr classes do not become sysroot crate API. Rust owns JSON parsing, string
escaping, and integer-profile emission; Sifr owns conversion between public
`JsonValue` objects and the token stream.

Wave 5 implementation evidence:

- `stdlib/_sifr/json.sifr` declares private `json_load_tokens`,
  `json_validate_integer_digit_limits`, and JSON dump token helpers as direct
  `sifr_stdlib::json` interop leaves.
- `stdlib/sifr/json.sifr` reconstructs public `JsonValue` values from bridge
  tokens, serializes `JsonValue` into bridge tokens for default/exact/web and
  string-int profiles, and preserves the existing primitive `json_dumps`
  subset through an owned union wrapper.
- `crates/sifr_stdlib/src/json.rs` owns JSON parse/emit behavior behind the
  `json` feature, preserves `JSONDecodeError` line/column fields through a
  bridge error type, delegates integer digit and profile policy to shared
  `sifr_runtime::json` primitives, and returns typed range/limit errors through
  direct interop mapping.
- `sifr_codegen` no longer registers the legacy JSON active intrinsic lowerer;
  the direct interop bridge now maps sysroot JSON error structs with
  `message/line/column`, `message/limit`, and `message/path/profile` accessors
  into the generated public Sifr error structs.
- `sifr_stdlib_model` and E2E/test-runner dependency planning no longer retain
  direct generated `serde_json` dependencies for `sifr.json` or `_sifr.json`;
  generated projects depend on `sifr_stdlib` with `features` containing
  `"json"` instead. Direct `serde_json` remains available for retained
  compiler-owned collection glue until that surface migrates.
- Create-pr generated-code quality found that auxiliary `_sifr.json` generated
  bridge error structs can reference `sifr_runtime::interop::SifrIntBridge`
  even in generated projects whose primary stdlib module is unrelated to JSON.
  `crates/sifr_driver/src/build/cargo_manifest.rs` now scans bridge contract
  and generated bridge field Rust types when adding sysroot interop crates, so
  generated auxiliary bridge sources and generated Cargo dependencies stay in
  sync.
- Opus review pass 1 found that the batch E2E harness still planned direct
  JSON `serde_json` dependencies and that the primitive `json_dumps` wrapper
  had dropped Decimal/BigDecimal behavior. Both blockers were fixed: the batch
  harness now routes `sifr.json` and `_sifr.json` through `sifr_stdlib` with the
  `json` feature, `json_dumps` preserves the prior primitive/decimal subset,
  and mixed JSON/TOML pass fixtures import the explicit `toml_loads` wrapper to
  avoid flattened public `loads` name collisions in grouped builds.
- Opus review pass 2 returned `VERDICT: PASS` with no blockers. Non-blocking
  follow-ups were limited to future bridge-corruption ergonomics, qualified
  typed-error dispatch, richer limit/range runtime assertions, and documenting
  that `json_dumps` now exposes the supported primitive/decimal union instead
  of the old intrinsic `Any` signature.
- Opus review pass 3 returned `VERDICT: PASS` after the generated-Cargo
  manifest fix for auxiliary JSON bridge error structs that reference
  `sifr_runtime::interop::SifrIntBridge`. Non-blocking follow-ups were limited
  to future tightening of scan invariants and tests, plus previously recorded
  post-M10 cleanup items.
- Focused validation passed:
  `cargo fmt --check`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_stdlib --features json json::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_codegen rust_interop_direct::tests::direct_rust_function_body_maps_json_decode_error_fields -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_codegen intrinsics::registry_core_tests::json_intrinsics_are_owned_by_compiled_stdlib_declarations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_driver stdlib::stateless_private_codegen_tests::json_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_driver dependency_plan_includes_runtime_for_generated_bridge_int_fields -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_stdlib_model features_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr_driver test_generate_test_runner_cargo_toml_preserves_stdlib_deps -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr test_generate_cargo_toml_json_uses_stdlib_json_feature -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- build demos/async_subprocess_pipeline_demo/main.sifr -o target/m10-json-focused/repro-async-subprocess-pipeline`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_json_consolidated.sifr`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/json_integer_profiles.sifr`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_json_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/error_subclass_handling.sifr`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/decimal_conversions.sifr`;
  `CARGO_TARGET_DIR=target/m10-json-focused cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/decimal_runtime_operations.sifr`;
  targeted uncached grouped E2E for `panic_free_stdlib_errors`,
  `parse_safety_error_paths`, and `structured_data_formats` with
  `SIFR_E2E_FIXTURE_MANIFEST=/tmp/sifr-m10-json-target-fixtures.json`; and
  `SIFR_E2E_DISABLE_CACHE=1 CARGO_TARGET_DIR=target/m10-json-focused cargo test -p sifr --test e2e test_e2e_pass -- --nocapture`
  (`651` pass fixtures, `0` failures).
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-json-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-json-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The run reported only the warm wall-time advisory and wrote
  `target/validation_lane_reports/create-pr.latest.json`; slowest validated
  steps were Python interop (`1781345ms`), generated-code quality
  (`1006493ms`), crate tests (`961811ms`), runtime platform (`683300ms`), and
  e2e pass (`382955ms`).

Wave 6 status: merged in [PR #2781](https://github.com/sifr-lang/sifr/pull/2781).
The `_sifr.encoding` private module and public `sifr.encoding` wrappers are
migrated off the active encoding intrinsic registry and onto private
`@rust(sifr_stdlib.encoding.*)` declarations backed by the narrow `encoding`
feature in `sifr_stdlib`. The public `Encoding`, `DecodeOutcome`,
`EncodeOutcome`, `DecodeError`, and `EncodeError` classes remain Sifr-owned.
The private Rust bridge returns primitive strings, byte arrays, and recovery
lists; public Sifr wrappers construct outcome objects and translate the private
`ParseError` message bridge back into public `DecodeError`/`EncodeError`
values.

Wave 6 implementation evidence:

- `stdlib/_sifr/encoding.sifr` declares private underscored implementation
  leaves for label support, canonical labels, decode/encode primitives,
  recoveries, incremental decode text/recovery helpers, and pending-byte
  tracking, all bound directly to `sifr_stdlib::encoding`.
- `stdlib/sifr/encoding.sifr` preserves the public API by exposing the original
  function names, constructing public outcome classes in Sifr, and hiding the
  private `*_impl` bridge names from exports.
- `crates/sifr_stdlib/src/encoding.rs` owns the generated-program stdlib
  adapter boundary behind the `encoding` feature and delegates shared text
  codec primitives to `sifr_runtime::encoding`.
- `sifr_codegen` no longer registers `_sifr.encoding` public stdlib helper
  names in active intrinsic dispatch; the remaining encoding registry file is
  retained only for compiler-owned `str.encode`/`bytes.decode` glue.
- `sifr_stdlib_model`, generated Cargo planning, and grouped E2E fixture
  planning no longer emit direct `encoding_rs` dependencies for `sifr.encoding`
  or `_sifr.encoding`; generated projects depend on `sifr_stdlib` with
  `features` containing `"encoding"` instead.
- The concurrency/runtime dependency snapshot no longer treats the
  structured-task cleanup fixture as requiring direct `encoding_rs` or
  `sifr_stdlib` encoding dependencies after the feature resolver moved encoding
  ownership behind the stdlib adapter.
- Focused validation passed:
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`;
  `python3 scripts/check_hir_maintainability_guardrails.py`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_stdlib --features encoding encoding::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_stdlib_model features_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_stdlib_model text_i18n_feature_dependency_snapshots_cover_feature_combinations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_stdlib_model --test concurrency_runtime_dependency_snapshots -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_codegen encoding_intrinsics_are_owned_by_compiled_stdlib_declarations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr_driver encoding_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo test -p sifr test_generate_cargo_toml_text_i18n_modules_enable_runtime_features -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr`;
  `CARGO_TARGET_DIR=target/m10-encoding cargo run -q -p sifr -- run demos/text_i18n/main.sifr`.
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-encoding-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-encoding-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The first full run caught the stale concurrency/runtime dependency snapshot
  expectation for the former direct `encoding_rs` edge; after updating that
  snapshot, the exact failing snapshot test and the full create-pr lane passed.
  The successful lane wrote `target/validation_lane_reports/create-pr.latest.json`;
  slowest validated steps were Python interop (`2081995ms`),
  generated-code quality (`1133700ms`), crate tests (`984102ms`),
  runtime platform (`778230ms`), and e2e pass (`388329ms`). The only advisory
  was the warm wall-time budget.
- Opus review pass 1 returned `VERDICT: PASS`. Non-blocking follow-ups were
  limited to pruning now-unreachable outcome signatures from the intrinsic
  model, removing the now-no-op `EncodingRs` required-feature marker, and
  considering a future fused outcome bridge helper to avoid recomputing
  text/recovery pairs.

Wave 7 status: merged in [PR #2782](https://github.com/sifr-lang/sifr/pull/2782).
The `_sifr.unicode` private module and public `sifr.unicode` wrappers are
migrated off the active Unicode intrinsic registry and onto private
`@rust(sifr_stdlib.unicode.*)` declarations backed by the narrow `unicode`
feature in `sifr_stdlib`. The public `UnicodeDataError` class remains Sifr-owned.
The private Rust bridge returns primitive strings, booleans, exact integer
bridges, and flattened segmentation vectors; public Sifr wrappers reconstruct
`list[tuple[int, str]]` payloads and translate the private `ParseError` message
bridge back into public `UnicodeDataError` values.

Wave 7 implementation evidence:

- `stdlib/_sifr/unicode.sifr` declares private underscored implementation
  leaves for Unicode normalization, classification, categories, numeric
  properties, names, lookup, bidirectional class, East Asian width, mirroring,
  decomposition, and segmentation helpers, all bound directly to
  `sifr_stdlib::unicode`.
- `stdlib/sifr/unicode.sifr` preserves the public API by exposing the original
  function names, constructing public `UnicodeDataError` values in Sifr, and
  hiding private `*_impl` bridge names from exports.
- `crates/sifr_stdlib/src/unicode.rs` owns the generated-program stdlib
  adapter boundary behind the `unicode` feature and delegates shared Unicode
  primitives to `sifr_runtime::unicode`; exact integer returns cross the Rust
  interop boundary through `SifrIntBridge`.
- The segmentation bridge uses flat vector helpers for grapheme and word
  boundaries so public Sifr code can reconstruct stable tuple payloads without
  teaching direct Rust interop a new nested tuple collection shape in this wave.
- `sifr_codegen` no longer registers `_sifr.unicode` active intrinsic dispatch,
  and the legacy Unicode registry file is deleted.
- `sifr_stdlib_model`, generated Cargo planning, grouped E2E fixture planning,
  and text/i18n dependency snapshots no longer emit direct Unicode runtime or
  third-party dependencies for `sifr.unicode` or `_sifr.unicode`; generated
  projects depend on `sifr_stdlib` with `features` containing `"unicode"`.
- Focused validation passed:
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`;
  `python3 scripts/check_hir_maintainability_guardrails.py`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features unicode unicode::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model features_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model text_i18n_feature_dependency_snapshots_cover_feature_combinations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen unicode_intrinsics_are_owned_by_compiled_stdlib_declarations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr_driver unicode_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_text_i18n_modules_enable_runtime_features -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_unicode_core.sifr`;
  `CARGO_TARGET_DIR=target/m10-unicode CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_unicode_segmentation.sifr`.
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-unicode-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-unicode-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The successful lane wrote `target/validation_lane_reports/create-pr.latest.json`;
  slowest validated steps were Python interop (`1819400ms`), crate tests
  (`989921ms`), runtime platform (`598158ms`), generated-code quality
  (`912406ms`), and e2e pass (`368934ms`). The only advisory was the warm
  wall-time budget.
- Opus review pass 1 returned `VERDICT: PASS`. The review found no blocking
  issues; non-blocking follow-up was limited to doc drift in text/i18n
  dependency docs, which this wave updated before PR.

Wave 8 status: merged in [PR #2784](https://github.com/sifr-lang/sifr/pull/2784).
The `_sifr.i18n` private module and public `sifr.i18n` wrappers are migrated
off the active i18n intrinsic registry and onto private
`@rust(sifr_stdlib.i18n.*)` declarations backed by the narrow `i18n` feature in
`sifr_stdlib`. Public i18n classes, constants, and helper names remain
Sifr-owned. The private Rust bridge returns `ParseError` for fallible adapter
calls, and public Sifr wrappers translate those bridge errors back into the
existing `LocaleIdError`, `FormatError`, `PluralRulesError`, and `CatalogError`
surfaces.

Wave 8 implementation evidence:

- `stdlib/_sifr/i18n.sifr` declares private underscored implementation leaves
  for locale canonicalization/maximize/minimize/host locale, number and
  datetime formatting, plural categories, collation, `.mo` validation/loading,
  and catalog lookups, all bound directly to `sifr_stdlib::i18n`.
- `stdlib/sifr/i18n.sifr` preserves the public API by exposing the original
  `i18n_*` helper names and classes while hiding private `*_impl` bridge names
  from public exports.
- `crates/sifr_stdlib/src/i18n.rs` owns the generated-program stdlib adapter
  boundary behind the `i18n` feature and delegates to `sifr_runtime::i18n`.
  Integer argument and return reshaping stays inside the adapter through
  `SifrIntBridge`.
- `sifr_codegen` no longer registers `_sifr.i18n` active intrinsic dispatch,
  and the legacy i18n registry file is deleted.
- `sifr_stdlib_model`, generated Cargo planning, grouped E2E fixture planning,
  and text/i18n dependency snapshots no longer emit direct ICU runtime or
  third-party dependencies for `sifr.i18n` or `_sifr.i18n`; generated projects
  depend on `sifr_stdlib` with `features` containing `"i18n"`. Explicit
  retained ICU feature planning still enables direct `sifr_runtime/i18n` where
  required by low-level runtime feature requests.
- Focused validation passed:
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`;
  `python3 scripts/check_hir_maintainability_guardrails.py`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features i18n i18n::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features i18n i18n_wrapper_canonicalizes_locale -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model features_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model text_i18n_feature_dependency_snapshots_cover_feature_combinations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen i18n_intrinsics_are_owned_by_compiled_stdlib_declarations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_driver i18n_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr_driver completed_private_declarations_follow_adapter_policy_syntax -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo test -p sifr test_generate_cargo_toml_text_i18n_modules_enable_runtime_features -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_locale_formatting.sifr`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr`;
  `CARGO_TARGET_DIR=target/m10-i18n CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run demos/text_i18n/main.sifr`.
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-i18n-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-i18n-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The successful lane wrote `target/validation_lane_reports/create-pr.latest.json`;
  slowest validated steps were Python interop (`1925385ms`),
  crate tests (`1033721ms`), generated-code quality (`966221ms`), runtime
  platform (`655354ms`), and e2e pass (`364206ms`). The only advisory was the
  warm wall-time budget.
- Opus review pass 2 returned `VERDICT: PASS`. The review found no blocking
  issues; non-blocking follow-ups were limited to stale empty-placeholder
  artifact notes from an earlier failed reviewer launch and optional future
  cleanup of legacy i18n alias functions.

Wave 9 status: merged in [PR #2785](https://github.com/sifr-lang/sifr/pull/2785).
The `_sifr.compress` private module and public `sifr.gzip`/`sifr.zipfile`
wrappers are migrated off the active compression intrinsic registry and onto
private `@rust(sifr_stdlib.gzip.*)` and `@rust(sifr_stdlib.zipfile.*)`
declarations backed by the narrow `gzip` and `zipfile` features in
`sifr_stdlib`. Public gzip keeps its `list[int]` byte API by converting through
Sifr `bytes` wrappers, while zipfile helpers preserve their existing string and
byte payload behavior.

Wave 9 implementation evidence:

- `stdlib/_sifr/compress.sifr` declares the migrated gzip byte helpers and
  zipfile create/add/read/name helpers as private direct Rust interop leaves.
- `stdlib/sifr/gzip.sifr` hides the byte-native private bridge behind the
  existing public `compress(data: str) -> list[int]` and
  `decompress(data: list[int]) -> Result[str, IOError]` API.
- `crates/sifr_stdlib/src/gzip.rs` and `crates/sifr_stdlib/src/zipfile.rs`
  own the generated-program adapter boundary behind the `gzip` and `zipfile`
  features, including valid empty-archive creation and zip byte read/write
  coverage.
- `sifr_codegen` no longer registers `_sifr.compress` active gzip or zipfile
  intrinsic dispatch, and the legacy compression registry files are deleted.
- `sifr_stdlib_model`, generated Cargo planning, and grouped E2E fixture
  planning no longer emit direct `flate2` or `zip` dependencies for
  `sifr.gzip`, `sifr.zipfile`, or `_sifr.compress`; generated projects depend
  on `sifr_stdlib` with `features` containing `"gzip"` and/or `"zipfile"`.
  The workspace `zip` dependency is narrowed to deflate-only defaults, removing
  native zstd-related crates from the lockfile for this path.
- Focused validation passed:
  `cargo fmt --check`;
  `git diff --check`;
  `python3 scripts/check_file_size_guardrails.py`;
  `python3 scripts/check_hir_maintainability_guardrails.py`;
  `CARGO_TARGET_DIR=target/m10-compression CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features gzip gzip::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-zip CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib --features zipfile zipfile::tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-model CARGO_BUILD_JOBS=1 cargo test -p sifr_stdlib_model features_tests -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-codegen CARGO_BUILD_JOBS=1 cargo test -p sifr_codegen compression_intrinsics_are_owned_by_compiled_stdlib_declarations -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-driver CARGO_BUILD_JOBS=1 cargo test -p sifr_driver compression_private_declarations_codegen_through_sifr_stdlib -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-driver2 CARGO_BUILD_JOBS=1 cargo test -p sifr_driver completed_private_declarations_follow_adapter_policy_syntax -- --nocapture`;
  `CARGO_TARGET_DIR=target/m10-compression-run1 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_gzip.sifr`;
  `CARGO_TARGET_DIR=target/m10-compression-run2 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_zipfile.sifr`;
  `CARGO_TARGET_DIR=target/m10-compression-run3 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_gzip_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-compression-run4 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr`;
  `CARGO_TARGET_DIR=target/m10-compression-run5 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/filesystem_paths_and_archives.sifr`;
  `CARGO_TARGET_DIR=target/m10-compression-run6 CARGO_BUILD_JOBS=1 cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/tempfile_and_zipfile.sifr`.
- Opus review pass 1 returned `VERDICT: NON-BLOCKING APPROVAL`. The review
  found no adapter-policy blockers; post-review cleanup finalized empty zip
  archives as valid readable archives, added zip byte-read unit coverage, and
  documented the historically infallible gzip compression adapter invariant.
- Opus review pass 2 returned `VERDICT: PASS` with no blockers. The review
  confirmed adapter-policy adherence, public API preservation, registry
  removal, dependency planning, lockfile narrowing, and validation sufficiency.
- Local create-pr validation passed with zero failures:
  `SIFR_LSP_COMMAND="$(pwd)/target/m10-compression-create-pr/debug/sifr lsp --stdio" CARGO_TARGET_DIR=target/m10-compression-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`.
  The successful lane wrote `target/validation_lane_reports/create-pr.latest.json`;
  slowest validated steps were Python interop (`2035470ms`),
  crate tests (`1096832ms`), generated-code quality (`1010259ms`), runtime
  platform (`696210ms`), and e2e pass (`370978ms`). The only advisory was the
  warm wall-time budget.

Acceptance:

- Each fallible data/text module satisfies the Native Migration Contract.
- Error values preserve public Sifr API shape and do not leak Rust crate
  internals.

Validation:

- Positive and negative parity suites for each migrated module.
- Cargo feature matrix checks for text/data groups.
- Large/malformed input fixtures for JSON, Unicode/encoding, URL, gzip, and
  zipfile.

### Post-M10 Adapter Policy Adherence Audit

Before opening additional M10 or M11 migration waves, audit the already-merged
M9/M10 work against the [stdlib Rust interop adapter policy][stdlib-interop-adapter-policy].
This is a checkpoint inside this phase, not a separate ad hoc phase.

Scope:

- Completed M9 waves: platform/html, calendar, UUID, math, hash, and
  base64/base32 encoders.
- Completed M10 waves: fallible base64/base32 decode/options and regex.
- Compiler and driver support that those waves depend on:
  `rust_interop_direct`, Rust interop probes, stdlib bootstrap, feature planning,
  and private stdlib codegen tests.
- No new native stdlib surface migration.

Required checks:

- Classify each completed private `_sifr.*` declaration as exact-shape direct
  binding, `sifr_stdlib` adapter binding, or global `E: Display` error bridge.
- Prove completed migrated names lower through `sifr_stdlib::*` and no longer
  rely on compiler intrinsic fallback.
- Keep fallback declarations only for explicitly unmigrated names, such as
  stateful random/crypto surfaces, and document that boundary where tested.
- Prove `Result<_, E: Display>` handling for `ParseError` and `RegexError` is
  the normative bridge rule, not a per-declaration converter pipeline.
- Prove generated projects use `sifr_stdlib` features rather than direct
  third-party dependencies for completed migrated modules, except documented
  compiler-owned exceptions such as `sifr.pathlib` path-glob regex lowering.
- Prove private stdlib trust remains compiler-owned and does not extend
  `trusted_no_panic` privileges to user packages.
- Search completed migration paths for `@rust.via`, callee injection,
  `bridge.*` sysroot adapters, or converter-pipeline metadata; all must be
  absent.

Acceptance:

- The audit lands as one focused PR unless it uncovers a real implementation
  defect that needs a narrower fix PR.
- Any stale fallback wording or tests for completed migrated names are removed
  or rewritten.
- The active issue records the adherence result and any residuals as concrete
  follow-up tasks before further M10/M11 waves proceed.

Audit result:

- `_sifr.platform` and `_sifr.html`: exact-shape direct bindings to
  `sifr_stdlib::platform` / `sifr_stdlib::html`; generated private code has no
  active intrinsic names for these modules and public imports depend only on
  `sifr_stdlib` features `platform` / `html`.
- `_sifr.uuid`: exact-shape direct binding to `sifr_stdlib::uuid`; generated
  private code routes `uuid4`, `uuid3_text`, and `uuid5_text` through
  `sifr_stdlib::uuid::*`, with no active UUID intrinsic fallback.
- `_sifr.calendar`: `sifr_stdlib` adapter binding. `sifr_stdlib::calendar`
  owns Gregorian behavior and `SifrIntBridge` adaptation; generated private
  code only performs the generic integer bridge calls required by direct Rust
  interop.
- `_sifr.math`: mixed exact-shape and `sifr_stdlib` adapter binding. Scalar
  float helpers bind directly; integer-returning, list-returning, and aggregate
  helpers rely on `sifr_stdlib::math` adapters plus the generic integer/list
  bridge.
- `_sifr.crypto` hash helpers and infallible base64/base32 encoders:
  exact-shape direct bindings to `sifr_stdlib::hash` and
  `sifr_stdlib::base64`. Public `sifr.hashlib` and `sifr.base64` wrappers keep
  borrowed private interop conventions out of user call sites.
- `_sifr.crypto` fallible base64/base32 decode/options:
  `sifr_stdlib` adapter bindings plus the global `Result<_, E: Display>` error
  bridge into `ParseError { message }`; no base encoding helper remains an
  active compiler intrinsic.
- `_sifr.regex`: `sifr_stdlib` adapter bindings plus the global
  `Result<_, E: Display>` error bridge into `RegexError { message, detail }`.
  `sifr.pathlib` path-glob lowering remains the documented direct-`regex`
  compiler-owned exception.
- `_sifr.crypto` remains a partial private module because stateful random
  helpers are not part of completed M9/M10 migration scope. The completed
  public migrated surfaces (`sifr.hashlib` and `sifr.base64`) have no direct
  third-party generated dependencies; direct `_sifr.crypto` feature planning may
  still account for random until the stateful random surface migrates.
- Private sysroot `trusted_no_panic` remains compiler-owned. User package Rust
  interop still requires package manifest trust, even when merged with trusted
  private stdlib interop in the same generated project.
- Completed migrated private declaration sources contain only direct
  `@rust(sifr_stdlib..., panic=trusted_no_panic)` declarations. The audit found
  no `@rust.via`, callee injection, `bridge.*` sysroot adapter target,
  converter-pipeline metadata, or completed-name intrinsic fallback.

Audit evidence added in this checkpoint:

- `completed_private_declarations_follow_adapter_policy_syntax` guards the
  completed private declaration sources against `@rust.via`, `bridge.*`, and
  converter-pipeline syntax while requiring direct `sifr_stdlib` targets.
- `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` now
  covers completed private migrated modules as well as public modules.
- `merged_user_and_private_stdlib_interop_keeps_user_trust_separate` proves
  sysroot `trusted_no_panic` does not satisfy user package Rust interop trust.

Validation:

- Focused tests for touched codegen, driver, stdlib-model, and `sifr_stdlib`
  modules.
- `git diff --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr` before PR.

### M11. Migrate Stateful and Resource Modules

Move resource-shaped stdlib surfaces once Rust interop certification permits
the relevant contracts.

Submilestones:

- M11a. Filesystem and file handles.
- M11b. Process children, pipes, environment, and process state.
- M11c. Signals and runtime/logging state.
- M11d. Net and TLS handles.
- M11e. HTTP transport handles.
- M11f. Python objects, callbacks, and Python resource handles.

Tasks for each submilestone:

- Migrate the resource family through opaque/resource interop.
- Use `@rust.opaque`, async declarations, callback policies, and zero-copy/view
  contracts only where the Rust interop certification matrix has executable
  support evidence.
- Keep uncertified surfaces explicitly compiler-owned or unsupported until
  certification lands.
- Add migration blockers back to the certification issue when a resource
  surface depends on uncertified bridge behavior.

Acceptance:

- Each resource family satisfies the Native Migration Contract.
- Resource lifetimes remain safe and deterministic.
- Runtime certification rows are updated with executable evidence before stable
  support claims.
- Resource cleanup behavior is deterministic under success, error, early return,
  cancellation, double-close, drop-at-shutdown, and process shutdown paths where
  relevant.
- Each resource family can close independently with its own review and
  certification evidence.

Validation:

- Resource lifecycle fixtures for every migrated handle type.
- Rust interop compatibility matrix updates.
- E2E runtime/resource tests under package and single-file modes.
- Leak/close/error-path fixtures for file, process, net, TLS, HTTP, and Python
  object handles.
- Per-family compatibility matrix updates before each submilestone closes.

### M12. Preamble and Intrinsic Registry Deletion

Finish the architecture cleanup.

Tasks:

- Delete migrated intrinsic registry files and generated preamble modules.
- Keep only language-owned generated glue: entrypoints, generated error
  wrappers, Sifr task/control-flow glue, cancellation/timeouts where tied to
  language semantics, and retained compiler-language glue.
- Update maintainability guardrails so stdlib native behavior cannot drift back
  into monolithic codegen registries.
- Update architecture docs with final module ownership.
- Delete `internal_docs/stdlib_native_surface_ownership.toml` or reduce it to a
  tiny retained compiler-language-glue allowlist.
- Add a guardrail that fails when new stdlib-native behavior is added to
  compiler intrinsics without a matching retained-allowlist entry.

Acceptance:

- `crates/sifr_codegen/src/intrinsics/registry` is deleted or reduced to a tiny
  retained compiler-language-glue allowlist.
- `crates/sifr_codegen/src/preamble` contains only language/runtime glue that
  cannot live in sysroot crates.
- Guardrails prevent reintroducing compiler-special stdlib native leaves without
  review.
- Architecture docs and code ownership agree on every remaining native surface.
- The migration ownership registry no longer exists as a broad second source of
  truth.

Validation:

- Maintainability guardrail script.
- Full stdlib parity suites.
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

### M13. Release Candidate Certification

Close the phase with an installed-toolchain certification pass.

Tasks:

- Build release-style artifacts for every supported target.
- Install each artifact into a clean temporary home.
- Run CLI smoke tests from outside the repository.
- Run stdlib-heavy single-file and package fixtures without any runtime-only
  override environment variable.
- Run stdlib-only fixtures with network disabled to prove sysroot vendor
  completeness.
- Run LSP request fixtures against the installed sysroot.
- Run self-update simulation from one version to another.
- Verify generated Cargo projects contain no source checkout or CI paths.
- Run the no-build-path-leakage scanner over generated Cargo manifests,
  generated locks, generated Rust sources, build reports, LSP traces, installed
  sysroot manifests, release archives, self-update receipts, and binary strings
  where feasible.
- Run `cargo metadata --offline` for generated stdlib-only projects.
- Validate generated stdlib-only `Cargo.lock` behavior under offline/frozen
  installed-layout fixtures.
- Compare `SysrootDependencyPlan` feature expectations against
  `cargo tree -e features` snapshots for representative stdlib programs.
- Capture `sifr doctor` output snapshots for healthy and broken installs.
- Update public/internal docs and roadmap status.

Acceptance:

- The installed toolchain works without a source checkout.
- Standard-library-only generated builds work offline.
- CLI and LSP use the same sysroot.
- Self-update preserves binary/sysroot pairing.
- No stable support claim exceeds Rust interop certification evidence.
- Generated artifacts and release artifacts contain no source checkout or CI
  path leakage.
- Feature trees for representative programs match `SysrootDependencyPlan`
  expected minimal features.

Validation:

- Targeted installed-layout release certification script.
- `verification/areas/sysroot_release/check_no_path_leakage.py`
- `cargo metadata --offline` for generated stdlib-only projects.
- Generated stdlib-only `Cargo.lock` offline/frozen fixtures.
- `SysrootDependencyPlan` expected-feature checks plus `cargo tree -e features`
  snapshots.
- `sifr doctor` healthy/broken install snapshots.
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

## Cross-Cutting Requirements

- Keep docs updated after every merged milestone.
- Keep `internal_docs/sifr_sysroot_and_stdlib_architecture.md` authoritative for
  final design; issue milestones should track implementation, not redefine the
  architecture.
- Update `internal_docs/distribution_pipeline.md`, `internal_docs/lsp_server.md`,
  `internal_docs/rust_interop_architecture.md`, and
  `internal_docs/architecture.md` as implementation lands.
- Keep Rust interop runtime/resource migration aligned with
  `plans/issues/active/rust-interop-runtime-ecosystem-certification.md`.
- Do not claim stable support for uncertified runtime/resource/callback
  surfaces.
- Preserve Cargo-compatible package behavior for user dependencies.
- Preserve Sifr's no-user-triggerable-panic guarantee.
- Keep generated sysroot crates free of development-workspace-only manifest
  assumptions.
- Keep `internal_docs/stdlib_native_surface_ownership.toml` updated during
  migration, then delete it or reduce it to the retained compiler-language-glue
  allowlist in M12.
- Treat sysroot source, crate, vendor, and Cargo config changes as one
  versioned toolchain unit.
- Keep all generated diagnostics source-attributed to public stdlib wrappers or
  private declarations where appropriate.
- Keep development sysroot auto-resolution behind the explicit
  `is_source_tree_development_mode()` predicate.
- Add and maintain mechanical path-leakage guardrails for release artifacts and
  generated artifacts.
- Use narrow additive features for generated-program sysroot crates; broad
  umbrella features must not be selected by generated Cargo unless explicitly
  justified.

## Completion Criteria

- Fresh Sifr install can compile and run stdlib-using programs without a source
  checkout, runtime-only override, or network access for Sifr-owned
  dependencies.
- CLI and LSP resolve stdlib information from the same installed sysroot.
- Generated Cargo artifacts never contain CI/build-machine paths.
- Sysroot crates compile outside the development workspace.
- Sysroot vendor data is complete for Sifr-owned stdlib/runtime dependencies.
- Public `sifr.*` stdlib behavior is covered by parity/e2e validation.
- Private `_sifr.*` implementation details remain unavailable to user code.
- Native stdlib leaves and resources are implemented through sysroot crates and
  Rust interop where certified.
- Local validation passes:

```bash
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh
```

# Ad Hoc Phase: Sifr Sysroot and Stdlib Toolchain

## Status

Planned.

## Design Reference

Authoritative target architecture:
[`internal_docs/sifr_sysroot_and_stdlib_architecture.md`][sysroot-stdlib-architecture]

[sysroot-stdlib-architecture]: ../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md

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
  ownership registry for native stdlib surfaces.
- Include inventory columns for current implementation owner, final
  implementation owner, migration blocker, and whether the surface can move
  before Rust interop runtime certification.
- For each registry entry, record current owner, final owner, reason,
  certification state, and deletion milestone.
- Confirm which Rust interop compatibility matrix rows are certified, active,
  and future-owned before resource migration starts.
- Add this issue to roadmap/phase tracking as the owner for sysroot and stdlib
  toolchain work.

Acceptance:

- Reviewers can see the exact pre-migration ownership boundaries.
- The inventories identify every native stdlib surface that must be migrated or
  explicitly retained.
- The ownership registry gives reviewers one mechanical source of truth for
  supported, compiler-owned, future-owned, and unsupported native surfaces.
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

- Add a `SifrSysroot` model with parsed `sysroot.toml` metadata.
- Define schema-versioned `sysroot.toml` fields for version, stdlib paths,
  sysroot crate paths, vendor path, and Cargo config path.
- Add sysroot identity/fingerprint fields: toolchain id, target triple,
  compiler commit, Cargo lock digest, public/private stdlib digests, runtime
  crate digest, stdlib crate digest, and vendor digest.
- Require `vendor-digest` in release sysroots and allow it to be absent or
  marked as development metadata only in source-tree development mode.
- Define canonical tree-digest hashing for sysroot assets: path ordering, path
  separator normalization, included file types, excluded metadata, executable
  bit handling where relevant, symlink policy, and line-ending policy.
- Define the sysroot Cargo workspace contract: `Cargo.toml`, `Cargo.lock`,
  workspace members, workspace dependencies, and resolver version, but defer
  validation of the final `crates/sifr_stdlib` member until M3/M4.
- Add resolver precedence: explicit developer override, `SIFR_SYSROOT`,
  installed path relative to `current_exe()`, then source-tree development
  layout.
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
- Keep `SIFR_RUNTIME_PATH` only as a development compatibility override.
- Remove release-mode fallback to compile-time checkout paths.
- Add build-mode gates so release binaries cannot silently use source checkout
  paths.
- Add a single `is_source_tree_development_mode()` predicate gated by explicit
  dev/debug build configuration.

Acceptance:

- A released-style binary with a complete skeleton sysroot resolves manifest,
  stdlib root, runtime crate, vendor, and Cargo config paths without
  environment variables.
- A missing runtime crate reports a Sifr sysroot diagnostic, not a Cargo error
  containing a build-machine path.
- Source-tree development still works without installing a sysroot.
- Release binaries cannot resolve source-tree layout unless a valid installed
  sysroot or explicit override is present.
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
- Unit tests for release/dev mode separation.
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
- Add compatibility re-export only if needed for a short mechanical migration;
  remove it before the milestone closes.

Acceptance:

- No generated user Cargo manifest depends on `sifr_stdlib_model`.
- Compiler crates depend on `sifr_stdlib_model` where they previously depended
  on current `sifr_stdlib`.
- Existing stdlib import, lowering, codegen, and analysis behavior is unchanged.
- The milestone closes with no compatibility re-export remaining unless a
  separately documented blocker exists.

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
- `cargo tree -e features` snapshots prove representative modules do not enable
  unrelated feature groups.

Validation:

- `cargo test -p sifr_stdlib`
- `cargo clippy -p sifr_stdlib -- -D warnings`
- Feature-combination cargo checks for default, text/data, network, and Python
  groups.
- Installed-layout cargo check for `lib/sifr/crates/sifr_stdlib`.
- Feature-tree snapshots for representative `sifr.re`, `sifr.json`,
  `sifr.http`, `sifr.python`, and pure Sifr stdlib programs.

### M4. Full Sysroot Workspace and Source Layout

Complete sysroot validation after the crate naming hazard is gone.

Tasks:

- Enable full validation of `crates/sifr_stdlib` in `sysroot.toml` and sysroot
  boundary checks.
- Validate `cargo metadata --offline` from the installed-layout sysroot
  workspace.
- Define repository source layout for public `stdlib/sifr/*.sifr` and private
  `stdlib/_sifr/*.sifr`, or generate that layout from existing `lib/sifr`
  during packaging with checked validation.
- Decide and implement exactly one canonical repository stdlib source root.
  The preferred final root is `stdlib/sifr/*.sifr` and
  `stdlib/_sifr/*.sifr`; temporary sync/generation from `lib/sifr` may exist
  only inside M4.
- Add private `_sifr` declaration modules for stdlib native surfaces.
- Preserve public `lib/sifr` compatibility during the transition or replace it
  with the canonical sysroot source root after all references move.
- Update `sifr_stdlib_model` source inventory to load from sysroot in released
  tools and from source-tree layout in development.
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
- Generate or copy Cargo config that points Sifr-owned builds at
  `<sysroot>/vendor` using standard Cargo source replacement.
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
  unless running in source-tree development mode.
- Ensure cache keys include sysroot version, sysroot crate fingerprints, and
  relevant feature sets.
- Record sysroot path, toolchain id, and sysroot manifest digest in generated
  build reports and trace/debug output.
- Keep user package dependencies and explicit Rust interop package dependencies
  Cargo-owned.
- Ensure package-mode generated builds do not accidentally force the sysroot
  vendor policy onto user dependencies outside documented offline/frozen modes.
- Test the selected Cargo config mechanism directly, whether it copies
  `.cargo/config.toml` into generated projects or passes an explicit Cargo
  config path.
- Keep temporary direct third-party generated dependencies only for unmigrated
  compiler-special paths with a deletion milestone and validation evidence.

Acceptance:

- `sifr run` for stdlib-using single-file programs does not need a source
  checkout or `SIFR_RUNTIME_PATH`.
- Generated Cargo manifests contain sysroot paths, not build-machine paths.
- Standard-library-only generated builds can run with Cargo offline when the
  sysroot vendor directory is present.
- User dependency resolution behavior remains Cargo-compatible.
- Generated Cargo projects do not require `Cargo.toml` from the Sifr source
  checkout.

Validation:

- End-to-end `sifr run` fixture using `sifr.random`, `sifr.json`, and
  `sifr.re` from an installed-layout fixture.
- Offline generated-build fixture for stdlib-only code.
- Snapshot tests for generated `Cargo.toml` and Cargo config.
- Snapshot or fixture for generated stdlib-only `Cargo.lock` behavior in
  offline mode.
- Fixture proving a generated project fails with a Sifr sysroot diagnostic when
  the sysroot vendor directory is missing in bundled-dependency mode.
- Package-mode fixture proving user dependencies remain Cargo-owned.
- Package-mode fixture proving online user registry resolution is not silently
  replaced by the sysroot vendor config.
- Fixture covering the chosen Cargo config mechanism for stdlib-only generated
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

### M7. LSP and Tooling Sysroot Integration

Make editor and analysis surfaces use the installed sysroot.

Tasks:

- Load stdlib source and private declaration metadata from `SifrSysroot`.
- Add stdlib source locations to the symbol index.
- Support hover, completion, definition, and type-definition for public stdlib
  files.
- Keep private `_sifr` declarations visible to sysroot implementation analysis
  but not user import completion.
- Add tooling diagnostics when the editor process sees a broken or mismatched
  sysroot.
- Add source-tree development behavior so local LSP sessions use repository
  stdlib sources when running from an unreleased build.
- Add source origin kinds: `UserSource`, `SysrootPublicStdlib`,
  `SysrootPrivateDeclaration`, `GeneratedSupport`, and `CompilerSynthetic`.
- Add CLI/LSP sysroot mismatch diagnostics that include the observed sysroot
  paths where available.
- Make go-to-definition prefer public wrappers for user code and expose private
  declaration links only in internal/developer contexts.

Acceptance:

- LSP go-to-definition for a `sifr.*` import lands in installed sysroot source.
- Hover and completion reflect the installed stdlib version.
- `_sifr.*` internals are not offered to user code as public modules.
- CLI and LSP report the same sysroot path for the same installation.
- Source maps correctly distinguish public stdlib, private declarations,
  generated support, compiler synthetic, and user files.

Validation:

- LSP request tests for hover/completion/definition against sysroot fixtures.
- Editor corpus snapshots updated for sysroot-backed stdlib locations.
- Negative completion tests proving private declarations do not appear in user
  import completions.
- Source-map origin tests for user files, public stdlib files, private
  declarations, generated support, and compiler synthetic sources.

### M8. Rust Interop Context for Private Stdlib Declarations

Give embedded/private stdlib declarations a normal interop context.

Tasks:

- Add a compiler-owned synthetic package context for sysroot stdlib interop.
- Write the private declaration ABI mini-spec before implementation. It must
  cover allowed annotations, target crates and module paths, error mapping,
  Result/error class conversion, opaque ownership, async/cancellation,
  zero-copy/view lifetimes, callbacks, prohibited declarations, and diagnostic
  attribution.
- Feed stdlib declarations through normal `InteropBuildPlan`, trust,
  probes, bridge generation, dependency planning, and cache keys.
- Define trust policy for Sifr-owned sysroot crates.
- Ensure user packages cannot impersonate or override private sysroot
  declarations.
- Add verification for cache invalidation when private declarations or sysroot
  crates change.
- Ensure stdlib interop probes use sysroot Cargo config and vendor data.
- Define how private declaration diagnostics point back to sysroot declaration
  source without exposing `_sifr` as public API.
- Require private declarations to target only canonical paths under resolved
  sysroot crates.
- Prevent user packages from shadowing `_sifr` declarations or same-named
  sysroot crate targets.

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
- ABI spec review and fixtures for allowed/prohibited declaration shapes.

### M9. Migrate Stateless Native Leaves

Remove low-risk compiler-special native stdlib plumbing first.

Tasks:

- Move math leaves to private declarations backed by `sifr_stdlib`.
- Move base64/base32, hash, UUID, regex, TOML, HTML, platform, and calendar
  leaves.
- Remove corresponding intrinsic registry lowering paths after parity passes.
- Keep public `lib/sifr` wrappers behaviorally stable.
- Update `sifr_stdlib` feature groups and `sifr_stdlib_model` dependency
  mapping as each leaf migrates.
- Add explicit unsupported-by-design notes for any stateless leaf deferred due
  to type-system or interop limitations.
- Capture public wrapper behavior snapshots before and after each module
  migration.

Acceptance:

- Public APIs are unchanged.
- Generated Rust calls interop-backed `sifr_stdlib` functions for migrated
  leaves.
- Old compiler-special lowering for migrated leaves is deleted or reduced to a
  compatibility shim with no active callers.
- Each migrated private declaration has executable evidence from public wrapper
  calls through generated Rust.

Validation:

- Existing stdlib parity/e2e suites.
- Focused fixtures for every migrated module.
- Snapshot tests proving generated Cargo deps and generated Rust shape.
- Code search check that migrated functions no longer appear in active
  intrinsic lowering dispatch tables.
- Feature-delta snapshots for each migrated module.
- Generated Rust source check that migrated leaves do not import third-party
  implementation crates directly.

### M10. Migrate Fallible Data and Text Modules

Move fallible native stdlib operations through typed interop boundaries.

Tasks:

- Migrate JSON, encoding, Unicode, i18n, URL validation/building, gzip,
  zipfile, and compression helpers.
- Standardize bridge error types and conversion to Sifr Result/error classes.
- Remove duplicated generated preamble where runtime crate functions should own
  behavior.
- Preserve no-panic guarantees in user-triggerable paths.
- Ensure large-data and malformed-input cases have bounded-memory and bounded
  diagnostic behavior where applicable.
- Capture public wrapper behavior snapshots before and after migration.
- Capture feature-delta snapshots per migrated module.

Acceptance:

- Fallible stdlib APIs return the same Sifr-level Result/error behavior as
  before.
- Generated code contains no data-dependent unwrap/expect in migrated paths.
- Dependency features are selected through sysroot stdlib planning.
- Error values preserve public Sifr API shape and do not leak Rust crate
  internals.

Validation:

- Positive and negative parity suites for each migrated module.
- Generated-code panic-pattern checks.
- Cargo feature matrix checks for text/data groups.
- Large/malformed input fixtures for JSON, Unicode/encoding, URL, gzip, and
  zipfile.
- Generated Rust source check that migrated modules do not import third-party
  implementation crates directly.

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
- Capture public wrapper behavior snapshots before and after migration.
- Capture feature-delta snapshots for the resource family.

Acceptance:

- Resource lifetimes remain safe and deterministic.
- Runtime certification rows are updated with executable evidence before stable
  support claims.
- Public stdlib behavior remains stable.
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
  language semantics, and unavoidable compatibility glue.
- Update maintainability guardrails so stdlib native behavior cannot drift back
  into monolithic codegen registries.
- Update architecture docs with final module ownership.
- Replace ad hoc exceptions with
  `internal_docs/stdlib_native_surface_ownership.toml` entries for any
  intentionally retained compiler-owned, unsupported, or future-owned native
  surfaces.
- Add a guardrail that fails when new stdlib-native behavior is added to
  compiler intrinsics without a matching registry entry.

Acceptance:

- `crates/sifr_codegen/src/intrinsics/registry` is deleted or reduced to a tiny
  compatibility shim with explicit owners.
- `crates/sifr_codegen/src/preamble` contains only language/runtime glue that
  cannot live in sysroot crates.
- Guardrails prevent reintroducing compiler-special stdlib native leaves without
  review.
- Architecture docs and code ownership agree on every remaining native surface.
- The native-surface ownership registry is complete and mechanically checked.

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
- Run stdlib-heavy single-file and package fixtures without `SIFR_RUNTIME_PATH`.
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
- Capture `cargo tree -e features` snapshots for representative stdlib
  programs.
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
- Feature trees for representative programs match expected minimal features.

Validation:

- Targeted installed-layout release certification script.
- `verification/areas/sysroot_release/check_no_path_leakage.py`
- `cargo metadata --offline` for generated stdlib-only projects.
- Generated stdlib-only `Cargo.lock` offline/frozen fixtures.
- `cargo tree -e features` snapshots.
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
- Keep `internal_docs/stdlib_native_surface_ownership.toml` updated whenever
  native stdlib ownership, certification, unsupported status, or deletion
  milestones change.
- Treat sysroot source, crate, vendor, and Cargo config changes as one
  versioned toolchain unit.
- Keep all generated diagnostics source-attributed to public stdlib wrappers or
  private declarations where appropriate.
- Keep source-tree development mode behind the explicit
  `is_source_tree_development_mode()` predicate.
- Add and maintain mechanical path-leakage guardrails for release artifacts and
  generated artifacts.
- Use narrow additive features for generated-program sysroot crates; broad
  umbrella features must not be selected by generated Cargo unless explicitly
  justified.

## Completion Criteria

- Fresh Sifr install can compile and run stdlib-using programs without a source
  checkout, `SIFR_RUNTIME_PATH`, or network access for Sifr-owned dependencies.
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

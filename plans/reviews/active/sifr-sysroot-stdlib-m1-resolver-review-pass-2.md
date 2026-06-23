I have enough to finalize the review. The pass-1 blockers are addressed, but the doc fix introduced a new internal inconsistency about where sysroot crates live.

## Findings

### High — Architecture-doc Installed Layout still disagrees with rest of doc & implementation on crate location
**Files:** `internal_docs/sifr_sysroot_and_stdlib_architecture.md:119-147`, `internal_docs/sifr_sysroot_and_stdlib_architecture.md:667-680`, `crates/sifr_sysroot/src/layout.rs:45`

The pass-2 edits moved `Cargo.toml`, `Cargo.lock`, `sysroot.toml`, `.cargo/`, and `vendor/` out of `lib/sifr/` and up to the toolchain root — good. But the `crates/` subtree in the Installed Layout tree was left at its old indentation as a sibling of `stdlib/` under `lib/sifr/`:

```text
  lib/
    sifr/
      stdlib/...
      crates/             ← indented at the same level as stdlib/
        sifr_runtime/
          Cargo.toml
```

So the Installed Layout block now declares the runtime crate manifest lives at `~/.sifr/lib/sifr/crates/sifr_runtime/Cargo.toml`. That contradicts:

- the Release Archive list (`crates/sifr_runtime/**` at archive root, `internal_docs/sifr_sysroot_and_stdlib_architecture.md:676`),
- the Pre-Migration Baseline rows (`<sysroot>/crates/sifr_runtime`, `<sysroot>/crates/sifr_stdlib`, lines 30–31),
- the Sysroot Manifest content-digest list (`lib/sifr/`, `crates/` listed as siblings, line 199),
- the Generated Cargo Projects template (`<sysroot>/crates/sifr_stdlib`, lines 564-566),
- the boundary-check list (`crates/sifr_runtime/Cargo.toml`, `crates/sifr_stdlib/Cargo.toml`, lines 745-746),
- the implementation: `SysrootPaths::from_root` uses `root.join("crates").join("sifr_runtime")` and the CLI/unit test fixtures write `crates/sifr_runtime/Cargo.toml` at the sysroot root.

This is the same shape of layout-contract divergence pass-1 blocked on: the Installed Layout block disagrees with the implementation about where crates physically live. The pass-1 fix attempt collapsed two roots into one, but the crates subtree didn't follow.

Fix: in the tree block at lines 119-147, move the `crates/` node up two indentation levels so it sits at the `~/.sifr/` root next to `vendor/`, e.g.:

```text
~/.sifr/
  bin/
    sifr
  Cargo.toml
  Cargo.lock
  sysroot.toml
  .cargo/
    config.toml
  vendor/
  crates/
    sifr_runtime/
      Cargo.toml
      src/
    sifr_stdlib/
      Cargo.toml
      src/
  lib/
    sifr/
      stdlib/
        sifr/
          *.sifr
        _sifr/
          *.sifr
```

(Order between `crates/` and `lib/` is cosmetic; what matters is that `crates/` is a sibling of `lib/`, not a child of `lib/sifr/`.)

### Resolved from pass 1

- **`SysrootErrorKind::VersionMismatch` is wired up.** `parse_sysroot_manifest` calls `validate_sifr_version` (`crates/sifr_sysroot/src/manifest.rs:87, 93-103`) which compares against `COMPILER_SIFR_VERSION = env!("CARGO_PKG_VERSION")` and permits the `-dev` suffix variant. The unit test `manifest_rejects_version_mismatch` (`crates/sifr_sysroot/src/tests.rs:80-86`) exercises the new path. The CLI fixture (`crates/sifr/tests/sysroot_cli.rs:94`) and `complete_sysroot` test helper (`crates/sifr_sysroot/src/tests.rs:243`) now construct version-matched manifests.
- **v1 optional-* field policy is documented.** Architecture doc line 213 now says "Schema version 1 permits keys prefixed with `optional-`," which matches `optional_field_allowed` in `manifest.rs:122-126` and the test at `tests.rs:97-101`.
- **The architecture doc sysroot-root statement now agrees with `installed_sysroot_root`** (toolchain root is the parent of `bin/`), and the Sysroot Resolution paragraph and Standard Library Source Layers section have been updated to match. Only the crate-location inconsistency in the Installed Layout tree remains.

### Non-blocking carry-overs (unchanged)
The pass-1 medium/low items that were not on the blocker list — public/private stdlib root split in `SysrootPaths`, `discover_source_tree_root` not requiring `sysroot.toml`, `normalize_lf` being lossy on non-UTF-8, the legacy `CARGO_MANIFEST_DIR` debug fallback in `sifr_stdlib::features`, and the global-hidden `--sysroot` — all remain as written in pass 1. None violates the M1 contract; they are appropriate to land later milestones.

## Verdict

**review-blocked** on the single doc-only fix above. The implementation, tests, and validation are otherwise in shape — the remaining work is moving the `crates/` node in `internal_docs/sifr_sysroot_and_stdlib_architecture.md:119-147` out of `lib/sifr/` so the Installed Layout tree agrees with the implementation and the rest of the document.

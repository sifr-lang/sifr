I've completed the review. Below are the findings, followed by the verdict.

## Findings

### High — Architecture-doc contract divergence on sysroot root location
**Files:** `internal_docs/sifr_sysroot_and_stdlib_architecture.md` (Installed Layout §, Standard Library Source Layers §), `crates/sifr_sysroot/src/layout.rs:42-56`, `crates/sifr_sysroot/src/resolve.rs:97-103`

The PR edits the "Sysroot Resolution" paragraph to declare that when the executable lives under `<toolchain>/bin/`, "the sysroot root is `<toolchain>/`" — i.e., the parent of `bin/`. The implementation matches this: `installed_sysroot_root()` returns `bin/`'s parent, and `SysrootPaths::from_root` uses `<root>/Cargo.toml`, `<root>/Cargo.lock`, `<root>/.cargo/config.toml`, `<root>/lib/sifr` for the stdlib root.

But the *Installed Layout* tree (lines 119–148) and "Standard Library Source Layers" section (lines 463–492) were not touched, and they still say:

- `~/.sifr/lib/sifr` **is** the sysroot,
- the sysroot's `Cargo.toml` / `Cargo.lock` / `sysroot.toml` live under `~/.sifr/lib/sifr/`,
- public stdlib is at `<sysroot>/stdlib/sifr/*.sifr` and private declarations at `<sysroot>/stdlib/_sifr/*.sifr`.

Under the new resolver text, the equivalent installed paths would be `~/.sifr/Cargo.toml`, `~/.sifr/sysroot.toml`, `~/.sifr/lib/sifr/*.sifr`. The two doc sections now disagree with each other; the implementation only matches one of them.

This is the load-bearing layout contract that M4–M6 (source roots, distribution archive, installer) inherit. Leaving the doc internally inconsistent at M1 close means later milestones don't have a single source of truth.

### Medium — `SysrootPaths` has no private-declaration root
**File:** `crates/sifr_sysroot/src/layout.rs:30-72`

The architecture mandates `<sysroot>/stdlib/_sifr/*.sifr` for private declarations and explicitly distinguishes it from the public root in both "Installed Layout" and "Standard Library Source Layers." `SysrootPaths` only carries a single `stdlib_root` and points it at `<sysroot>/lib/sifr`. The M1 plan defers final `sifr_stdlib` crate validation to M3/M4, but the source-root pair (public vs. private) is part of the identity layout this skeleton is meant to fix. Either add `stdlib_public_root` / `stdlib_private_root` now, or document in the issue plan that the split is deliberately deferred to M4.

### Medium — `SysrootErrorKind::VersionMismatch` exists but is never produced
**Files:** `crates/sifr_sysroot/src/error.rs:51`, `crates/sifr_sysroot/src/manifest.rs:57-89`

M1 plan validation calls for "Unit tests for missing, malformed, and version-mismatched `sysroot.toml`." The architecture's boundary check explicitly lists "the sysroot version matches the compiler version." `parse_sysroot_manifest` checks `schema-version` (raising `UnsupportedSchemaVersion`) but never compares the manifest's `sifr-version` to the compiler build. The `VersionMismatch` variant is dead code; the version-mismatch test in `tests.rs:54-71` actually exercises schema mismatch. Either wire up the sifr-version pairing check (with a compiler-side constant) and add the test, or drop the unused variant and tighten the validation list scope.

### Medium — Unknown-optional-field handling not gated on schema version
**File:** `crates/sifr_sysroot/src/manifest.rs:91-112`

Architecture text (lines 211-212): "Unknown optional fields may be ignored only when the active `schema-version` explicitly permits that behavior." The implementation accepts any key prefixed with `optional-` unconditionally, independent of schema-version. This is fine for v1 if that's the intended policy, but the policy isn't documented in `sifr_sysroot_and_stdlib_architecture.md`. Add a one-liner to the manifest section recording that v1 permits `optional-*` keys, so the schema-drift test and the doc stay in sync.

### Low — `discover_source_tree_root` doesn't require `sysroot.toml`
**File:** `crates/sifr_sysroot/src/resolve.rs:62-76`

Discovery succeeds for any ancestor that has `crates/sifr_runtime/Cargo.toml`, `lib/sifr/`, and `Cargo.lock`. `ResolvedSysroot::from_root` will then fail on the missing manifest, which is correct, but the failure path is one step removed from "is this actually a dev sysroot." Including `<ancestor>/sysroot.toml` in the predicate would make the discovery step the same shape as the installed boundary check and tighten the dev gate beyond the `cfg!(debug_assertions)` flag alone.

### Low — `canonical_sysroot_tree_digest` is lossy on non-UTF-8 input
**File:** `crates/sifr_sysroot/src/digest.rs:127-130`

`normalize_lf` runs `String::from_utf8_lossy(bytes)` before CRLF normalization. For `.toml`/`.rs`/`.sifr`/`.lock` files this should never matter, but invalid bytes get replaced with `U+FFFD` (3 bytes) before the digest closes over them. Architecturally the doc says "normalizes line endings to LF" without claiming lossy UTF-8 substitution. A byte-level scan (`if b == b'\r' …`) avoids the silent corruption and keeps the digest a pure function of bytes.

### Low — Legacy runtime-path fallback still uses `env!("CARGO_MANIFEST_DIR")` in debug builds
**File:** `crates/sifr_stdlib/src/features.rs:835-844`, callers at lines 680-688

`legacy_development_runtime_path` returns `SIFR_SYSROOT_UNRESOLVED` in release (good), but in debug it dereferences `CARGO_MANIFEST_DIR`. The only consumer is the infallible `generated_cargo_dependencies`, which now feeds **only** cache-key computation (`generate_dependency_cargo_toml_for_cache_key`); actual `Cargo.toml` emission goes through `try_generated_cargo_dependencies` → `SysrootError`. So no checkout path can leak into a generated manifest. Still, leaving the embed in source after this PR risks future callers re-introducing it; a follow-up could remove the infallible variant once the cache-key path can also flow `SysrootError`.

### Low — `--sysroot` is globally hidden; command-support matrix not enforced
**File:** `crates/sifr/src/cli_model_and_entrypoint.rs:52-54`, `380-393`

`--sysroot` is `global = true, hide = true`, applied uniformly. The architecture's command-support matrix requires `--sysroot` to be:
- hidden/advanced for `check`, `build`, `run`, `emit`
- visible for `doctor`
- ignored or rejected for `self update` unless multi-sysroot is designed

`doctor` and `self update` are deferred to later milestones, so the current uniform-hidden choice is acceptable for M1, but worth noting on the M1 closing checklist.

## Verdict

**review-blocked** on a doc-only fix.

Required before close:
1. Reconcile `internal_docs/sifr_sysroot_and_stdlib_architecture.md` so the "Installed Layout" tree (lines 119-148) and "Standard Library Source Layers" section (lines 463-492) match the chosen resolver semantics ("sysroot root is `<toolchain>/`", stdlib at `<sysroot>/lib/sifr/`). If the implementation's choice is intentional, update those two sections; if not, restore `current_exe()/../lib/sifr` resolution. Either is a small edit, but the M1 close depends on a single layout source of truth that M2–M6 can inherit.
2. Decide whether `SysrootErrorKind::VersionMismatch` should be wired up (with a sifr-version pairing test) or deleted, and bring the M1 plan's "version-mismatched `sysroot.toml`" validation entry into agreement with whatever is implemented.

The code itself (resolver precedence, manifest parser, layout validation, digest, CLI, driver sysroot-error mapping) is well-scoped, tested, and integrates cleanly with the driver's pre-cargo diagnostic path. After the two doc-level fixes the milestone is in shape to merge.

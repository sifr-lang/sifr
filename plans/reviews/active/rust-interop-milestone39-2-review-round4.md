# M39.2 Rust Interop — Round 4 Review (post-clearance follow-up)

Round 3 cleared M39.2. This round reviews the five follow-up changes the user listed:

1. Split `crates/sifr_package/src/graph/digest.rs` into focused digest modules.
2. Reduced `crates/sifr_package/src/package_publish_archive_tests.rs` below the package-manager line cap.
3. Gated native-link evidence enforcement on the presence of a Rust interop plan.
4. Removed backticks from new Rust interop diagnostic registry templates and regenerated error docs.
5. Updated Ruff fork fixture revalidation metadata for the already-changed Ruff submodule revision.

No new blocking findings. Milestone remains cleared.

---

## Required fixes — none

No blocking regressions introduced by the round-4 follow-ups.

---

## Non-blocking suggestions (P2 — file before merge to `main`, not blockers)

### 1. Native-link evidence gating is correct for M39.2 scope, but worth a one-line scope note
`crates/sifr_driver/src/build/materialize.rs:228-236`

```rust
fn should_validate_native_link_evidence(generated_project: &GeneratedBinaryProject) -> bool {
    let rust = &generated_project.interop.rust;
    !rust.declarations.is_empty()
        || !rust.resolved_targets.is_empty()
        || !rust.trust_requirements.is_empty()
        || !rust.probe_plan.probes.is_empty()
        || !rust.bridge_sources.is_empty()
        || rust.cargo_inputs.is_some()
}
```

Behaviour I confirmed:

- **Rust interop present** → validation runs against *all* `build-script-executed` JSON lines from cargo, including transitive-dep build scripts. Any `linked_libs` entry not in `trust.native_links` raises `SIFR-RUST-TRUST-0001`. Direct backend coverage is enforced; transitive build-script link emissions are also caught because the validator walks the cargo JSON stream, not the declared backend set (`materialize.rs:242-269`).
- **No Rust interop** → validation skipped. Pure Python-interop projects whose pyo3 dep emits `cargo:rustc-link-lib=python3.13` no longer false-fail with `SIFR-RUST-TRUST-0001`. This is the regression the gating was meant to fix, and it does.

The DoD line at `plans/phases/39_rust_interop.md` that mandates link-evidence validation is scoped to Rust interop, so the skip path is correctly out of scope for M39.2.

**Latent concern, not a regression**: a *mixed* Rust + Python interop project would trigger the validator, and pyo3's `linked_libs` for `python3.13` would not appear in `trust.native_links` (the trust list is populated from declared Rust backends only — `rust_interop.rs:289-301`). No checked-in fixture exercises that combination today (verified: `grep '@rust' verification/areas/python_interop/...` is empty), so this is forward-looking. When M39.3+ introduces the first mixed fixture, the manifest will need either a `native_links = ["python3.13"]` entry or the validator will need a Python-interop-aware trust source. Worth flagging in `plans/phases/39_rust_interop.md` as a known M39.3 gap.

### 2. Digest module split preserves digest stability — verified field-by-field
`crates/sifr_package/src/graph/digest.rs` + four split modules.

`serde_json::to_vec` serializes struct fields in declaration order, so digest stability depends on field-order parity with the pre-split file. I diffed the four new modules against the round-3-staged `digest.rs`:

- `digest_cargo_metadata.rs` — `CanonicalMetadata` and `CanonicalPackage` (including the new `links` field) preserved in original order. ✓
- `digest_package_graph.rs` — `CanonicalGraph` (`packages, edges, backend_crates, scopes`), `CanonicalGraphPackage` (`package_id, cargo_package_id, sifr_name, exports, rust, rust_trust, python, python_trust`), and the `CanonicalBackendCrate`/`CanonicalRustInteropConfig`/`CanonicalRustTrust` field order all match the staged baseline. ✓
- `digest_source_map.rs` — `CanonicalSourceMap` (`roots, modules, ambiguous_modules`) preserved. ✓
- `digest_build_cache.rs` — `CanonicalPackageBuildCacheInputs` field order and the inline `features`/`selectors` sort preserved. ✓

Digest output should be byte-identical pre/post-split. No blocker.

Minor: `digest_serializable` is now `pub(in crate::graph)` (`digest.rs:16`) which scopes it correctly to the four split modules. Fine.

### 3. Package-manager guardrail update — works today, but fragile
`verification/areas/package_management/tools/check_package_manager_guardrails.py:502-512`

```python
digest_rs = (package_crate / "src/graph/digest.rs").read_text(encoding="utf-8")
metadata_digest_rs = (package_crate / "src/graph/digest_cargo_metadata.rs").read_text(
    encoding="utf-8"
)
if "CanonicalMetadata" not in digest_rs or "digest_graph_inputs" not in digest_rs:
    if (
        "CanonicalMetadata" not in metadata_digest_rs
        or "digest_graph_inputs" not in digest_rs
    ):
        failures.append("metadata normalization digest support is missing")
```

This passes today only because `digest.rs:6` retains `pub use super::digest_cargo_metadata::digest_graph_inputs;` — the substring `digest_graph_inputs` still appears in `digest.rs`. The inner check at line 510 *still reads `digest_rs`* (not `metadata_digest_rs`) for `digest_graph_inputs`, which is almost certainly a copy-paste oversight. If someone removes the re-export later (or routes call-sites to `digest_cargo_metadata::digest_graph_inputs` directly), this guardrail becomes a no-op pretending to enforce something.

Simpler and more honest:

```python
combined = digest_rs + metadata_digest_rs
if "CanonicalMetadata" not in combined or "digest_graph_inputs" not in combined:
    failures.append("metadata normalization digest support is missing")
```

P2.

### 4. Diagnostic-registry backtick removal creates a doc-vs-emit desync (per project convention — acceptable)
`crates/sifr_diagnostics/src/codes/registry/registry_entries/rust_interop.rs:24,35,46`

The registry templates lost their backticks to satisfy `assert_registry_strings_are_markdown_safe` (`crates/sifr_diagnostics/src/codes/registry_tests.rs:227-262`). The corresponding source-emission strings still wrap the placeholder:

| Code | Registry template (docs) | Emit-site template (user sees) |
|---|---|---|
| `RUST_RESOLVE_TARGET_ROOT` | `unresolved Rust target root {root}` | `unresolved Rust target root \`{root}\`` (`rust_interop.rs:201,217`) |
| `RUST_TRUST_MISSING` | `missing Rust interop trust declaration for {target}` | `missing Rust interop trust declaration for \`{target}\`` (`rust_interop.rs:397`) |
| `RUST_TYPE_PROBE_FAILURE` | `Rust bridge probe failed for {target}` | `Rust bridge probe failed for \`{target}\`` (`rust_interop.rs:414`, `rust_interop_probe.rs:80`) |

This matches the existing project convention for other diagnostics (docs strip markdown-sensitive characters; emission preserves them). Acceptable for M39.2.

Forward-looking suggestion (out of scope for M39.2): generate the doc-side template from the emit-side template via a normalization pass at doc-gen time so the registry only stores one canonical string. Would close the doc/emit drift class of bugs entirely.

### 5. Pre-existing: `RUST_CARGO_METADATA` registry args don't match `probe_io_failure` emission
`crates/sifr_diagnostics/src/codes/registry/registry_entries/rust_interop.rs:49-58` declares `[]` args, but `crates/sifr_driver/src/build/rust_interop_probe.rs:108-115` emits the same code with `message_template: "{message}"` and a `message` arg substituted at `rust_interop.rs:471,479`. Doc readers won't see the `message` placeholder; arg-name consistency lints (if any are added later) will trip. Predates round 4 — flag for M39.3.

### 6. Pre-existing: `package_publish_archive_tests.rs` `assert!` regression vs `assert_eq!`
`crates/sifr_package/src/package_publish_archive_tests.rs:80`

```rust
assert!(diagnostic.code == DiagnosticCode::PACKAGE_PUBLISH_VALIDATION_FAILED);
```

The earlier `assert_eq!(diagnostic.code, DiagnosticCode::…);` fits on a single line (~83 chars at 8-space indent — under 100), so the line-cap (`crates/sifr_package/src/**/*.rs: 420`) does not justify dropping to `assert!`. Restore `assert_eq!` so a failure prints both sides of the mismatch. Trivial; non-blocking.

---

## Round 3 P1/P2 items still open (status quo — listed for tracking)

None of the round-4 changes set out to address these, and the user explicitly confirmed scope:

- **P1 #2** — substring-based rustc/cargo error classification (`rust_interop_probe.rs:69-77`).
- **P1 #3** — probes not deduped per backend (`rust_interop_probe.rs:33-44`).
- **P1 #4** — `bridge.*` and `Self.<method>` paths never probed.
- **P1 #5** — fixture gaps (`bridge.*` positive, `Self.method`, `proc-macro`, `unsafe-rust-bridges`, determinism, cache-invalidation, transitive-dep-rejected-as-root).
- **P1 #6** — `linked_paths`, `cfgs`, `env` from `build-script-executed` JSON are unvalidated.
- **P1 #7** — `sifr emit` still can't reach Rust interop context (`entrypoint.rs:191-203,397`).
- **P2 #8–12** — `digest_path` symlink/swallow, `fnv1a64` duplication across `graph/digest.rs` and `rust_interop_digest.rs`, misleading `Self`-on-free-function diagnostic, `panic_strategy` env-only, cargo-on-PATH dependency in new probe fixture.

These remain non-blocking for M39.2 clearance.

---

## Verdict

**Milestone M39.2 remains cleared.** The round-4 follow-up changes are correct in intent and behaviour:

- Digest split is field-order-preserving (byte-identical output).
- Native-link gating restores Python-interop builds while preserving Rust-interop trust enforcement (transitive build-script emissions are still caught when Rust interop is active).
- Package-manager guardrail still enforces; the new structure is brittle but functional.
- Diagnostic registry templates conform to the markdown-safety lint; doc/emit desync is per existing project convention.
- Ruff fork revalidation pin matches the submodule commit (`8111415495271a09f9ee89cb168fde669db240d8`) and the fixture revisions.

Recommended cleanups before the next milestone (all P2, none blocking M39.2 merge):
1. Simplify the package-manager guardrail to read both digest files into a combined buffer (item 3 above).
2. Restore single-line `assert_eq!` in `package_publish_archive_tests.rs:80` (item 6).
3. Add a one-line note in `plans/phases/39_rust_interop.md` that mixed Rust + Python interop builds will need explicit `native_links` trust for the Python runtime symbol (item 1, latent concern).

The remaining round-3 P1/P2 items are correctly deferred to M39.3/M39.5.

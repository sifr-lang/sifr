I've inspected the round-1 blockers' fixes and the supporting test/doc evidence.

## Round 2 review: M39.3 Rust interop

### Blocking findings

**None.** All four round-1 blockers (B1–B4) are resolved substantively.

### Verification of the round-1 fixes

**B1 — generated bridge path collision (resolved).** `generated_bridge_module_path` in `rust_interop_cargo_inputs.rs:61-68` now maps `None → ["__sifr_bridge", "__sifr_binary_entry"]` and `Some("main") → ["__sifr_bridge", "main"]`. The cache-fragment writer in `rust_interop_plan.rs:420-433` distinguishes the two with the `binary-entry` sentinel vs `module:<name>`. The `generated_bridge_module_path_keeps_binary_entry_distinct_from_main_module` test (rust_interop_cargo_inputs.rs:304-317) pins the invariant.

**B2 — Rust keyword bridge filenames (resolved).** The new `projection_rust_keywords.rs` enumerates the Rust keyword set; `is_rust_identifier` in `projection_bridge.rs:374-382` rejects identifiers that match. `rust_bridge_projection_rejects_keyword_bridge_module_filename` (package_projection_tests.rs:190-206) exercises `src/bridges/match.rs` and asserts the stable Sifr diagnostic.

**B3 — shared-bridge scanner (resolved).** `imports_generated_bridge_namespace` (rust_interop_cargo_inputs.rs:96-110) now operates on tokenized Rust output that skips line/block comments and quoted strings, and looks specifically for `crate :: __sifr_bridge` and `__sifr_bridge ::` token sequences. The new `package_rust_interop_allows_shared_bridge_comments_about_generated_bridge_types` (rust_interop_tests.rs:207-235) confirms the comment + string + `__sifr_bridge_compat` cases no longer fire, and the positive test still catches the real import.

**B4 — verification fixtures (resolved).** `local_bridge_blake3`, `shared_bridge_crate`, `bridge_version_mismatch`, and `same_workspace_crate` now carry README evidence files pointing at concrete unit tests, and the matrix JSON honestly labels coverage (`probe-only` / `passing`) rather than `planned`. `check_fixture_matrix.py` passes. The READMEs do not overstate runtime evidence — runtime tiers stay `planned` for M39.4/M39.5.

### New (non-blocking) findings

**N-new1. `is_rust_keyword` is missing `abstract` and `gen`.** `abstract` is a reserved keyword in all Rust editions; `gen` became reserved in edition 2024 (which Sifr's projection emits — see `projection.rs:10`). A user-authored `src/bridges/abstract.rs` or `src/bridges/gen.rs` would still produce a Cargo error instead of the stable `PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT` diagnostic. The round-1 case (`match`) is correctly caught, so the contract is mostly honored, but the keyword set should be exhaustive.

**N-new2. A Sifr module literally named `__sifr_binary_entry` would alias the binary entry's Rust module path.** `generated_bridge_module_path(Some("__sifr_binary_entry"))` and `generated_bridge_module_path(None)` both yield `["__sifr_bridge", "__sifr_binary_entry"]`. The cache fragment distinguishes them, but the on-the-wire Rust namespace collides. Defensible by treating `__sifr_*` as reserved Sifr module names; not a realistic user collision but the original B1 logic deserves the same defensive guard it gave `main`.

**N-new3. `same_workspace_crate` README cites a negative test as positive evidence.** `package_rust_interop_maps_rustc_probe_resolution_failure` asserts `SIFR-RUST-RESOLVE-0001` fires when the same-workspace crate doesn't export the requested symbol — that's negative-direction evidence. The "positive evidence" line in `verification/areas/rust_interop/fixtures/same_workspace_crate/README.md` describes machinery exercising, not a passing happy-path probe. The matrix status (`probe-only`) is honest, but the README wording reads as if a successful end-to-end same-workspace probe is covered. Tighten the README or add a successful-probe test.

**N-new4. Raw hash strings (`r#"..."#`) aren't handled by the namespace tokenizer.** `rust_namespace_tokens` enters `skip_quoted` at the leading `"` and stops at the first interior `"`; the closing `"#` semantics aren't modeled. A raw hash string containing `crate::__sifr_bridge::…` would still false-positive. Very corner case (a shared crate would need to embed that literal in a raw-hash string) — flagging only because the round-1 review explicitly demanded the scanner be precise.

The round-1 non-blockers (N1–N13) remain unchanged.

### Verdict

**M39.3 is cleared.** All four round-1 blockers are fixed; the remaining items above are polish, not phase-correctness regressions, and naturally fold into M39.4/M39.5 work. The cache-keying claim, the keyword-safe filename guarantee, the boundary-scanner correctness, and the verification-area fixture matrix are all back on architectural rails.

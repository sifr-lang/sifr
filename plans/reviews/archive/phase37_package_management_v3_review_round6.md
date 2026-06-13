Reviewing the "Organization Demo Repositories" section (lines 543–695) for blockers and contradictions with the Cargo-backed v3 model.

**Findings:**

1. **Incomplete consumer demo (lines 631–649)**: The `sifr-package-demo` `Cargo.toml` references `[package.metadata.sifr] manifest = "sifr.toml"` but no corresponding `sifr.toml` is shown. The other packages show both files. A consumer app with imports needs `sifr.toml` for source roots and export validation, and Phase 37 requires the manifest to be loadable.

2. **`edition = "2024"` for Rust Cargo packages (lines 573, 601, 638)**: Cargo edition 2024 was stabilized in Rust 1.85 (Feb 2025). Current stable is 2021 with 2024 available. This is not a blocker but slightly forward-leaning — acceptable given Phase 37 timeline but should be noted.

3. **`sifr-http` missing `reqwest` in Cargo `[dependencies]` (lines 607–609)**: The `sifr.toml` trusts `reqwest` but `reqwest` does not appear in `Cargo.toml` `[dependencies]`. A Rust-backed package that trusts a backend crate needs that crate declared as a Cargo dependency. Without it, `cargo metadata` won't select reqwest and trust validation can't find it. This contradicts the discovery flow (lines 441–456) which requires all Sifr dependencies to be Cargo-classified.

4. **Alias demo assumes `v0.2.0` tag exists (lines 667–668)**: `json_v2` points to `tag = "v0.2.0"` on `sifr-json`. The package starts at `v0.1.0`. The multi-version demo won't work until a second version is published. This is documented as a demo requirement, so it's realistic but needs a follow-up step to create `v0.2.0`.

5. **`sifr-package-demo` missing `include` directive**: Unlike `sifr-json` and `sifr-http` which have explicit `include` patterns, `sifr-package-demo` omits this. This is acceptable for a consumer workspace that doesn't publish, but the demo validation mentions package archive checks — a consumer workspace that doesn't publish won't exercise that path.

**Verdict: not-ready**

Issue 3 (`sifr-http` missing `reqwest` in Cargo deps) is a genuine contradiction — it breaks the model invariant that backend crates are both trusted in `sifr.toml` and classified from `cargo metadata`. Issue 1 (missing `sifr.toml` for consumer demo) is incomplete but less blocking since consumers may not need the same manifest strictness as libraries. Issue 3 must be corrected before the section is consistent with the Cargo-backed v3 model.

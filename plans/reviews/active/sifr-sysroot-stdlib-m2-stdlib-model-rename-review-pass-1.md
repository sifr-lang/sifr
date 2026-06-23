I have enough context. Writing the review.

# M2 Review: Rename Compiler Stdlib Crate (`sifr_stdlib` → `sifr_stdlib_model`)

## Summary

The rename is mechanically thorough: workspace member, lockfile package, all 30+ touched compiler/driver/lowering/codegen call sites, dependency-direction guardrails, all four verification profiles, generated-code-quality producer-fingerprint inputs, three stdlib-parity dependency snapshot data files, the architecture doc (with explicit "renamed from sifr_stdlib" callout at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:442`), HIR guardrails doc, network/HTTP and text/i18n architecture docs, and the issue plan all flip cleanly. `cargo tree -i sifr_stdlib_model` confirms only compiler/tooling reverse-deps, and the generated user Cargo emitter at `crates/sifr_driver/src/build/cargo_manifest.rs:1-31` consumes `sifr_stdlib_model` only as a compile-time API (`generated_cargo_dependencies` returns third-party dep lines like `serde_json`/`regex` — the crate name itself never enters the generated manifest). No `pub use sifr_stdlib::*` legacy re-export exists. Future references to `crates/sifr_stdlib` in `internal_docs/sifr_sysroot_and_stdlib_architecture.md` and `internal_docs/stdlib_native_surface_ownership.toml` (`final_owner = "crates/sifr_stdlib ..."`) correctly describe the future generated-program crate per M3+ scope — those should remain.

## Blockers

### B1. Stale `sifr_stdlib::` symbol reference in a file already partially updated
**File:** `verification/areas/stdlib_parity/reports/concurrency_runtime_legacy_surface_traceability.md:21`

```
- `crates/sifr_stdlib_model/src/sources.rs` no longer embeds those legacy modules.
- `sifr_stdlib::unsupported_legacy_stdlib_module` records native replacement namespaces.
```

The line above was migrated; this line was missed. The symbol no longer exists under `sifr_stdlib` — it lives only at `crates/sifr_stdlib_model/src/lib.rs:217` (`pub fn unsupported_legacy_stdlib_module`). This is current-state "Implementation evidence," not historical validation evidence, so it must point at the actually-existing symbol. Update to `sifr_stdlib_model::unsupported_legacy_stdlib_module`.

### B2. Stale package in a forward-looking validation plan
**File:** `verification/areas/stdlib_parity/reports/concurrency_runtime_readiness_traceability.md:59`

```
## Validation Plan
Final validation must include:
…
- `cargo test -p sifr_stdlib`
```

This is the prescriptive "final validation must include" list, not a "Validation Evidence" historical row. Running it post-rename would fail with `error: package ID specification 'sifr_stdlib' did not match any packages`. Update to `cargo test -p sifr_stdlib_model`.

## Advisory — not blockers, but worth a one-line decision before merge

### A1. Sanitizer manifest still names the old package
**File:** `verification/areas/runtime_platform/sanitizer_manifest.json:141, 196`

Two skipped sanitizer entries record reproduction commands `cargo +nightly test --locked -p sifr_stdlib ipc_connection::...` and `cargo test --locked -p sifr_stdlib ipc_request_tracker`. Each has a `skip_reason` so nothing executes them today, but the file is active operational JSON (not an archive), and the recorded reproduction would no longer resolve. Defensible to leave (skipped reproduction boundary, not history-of-runs), but inconsistent with how the four profile JSONs were treated. Either flip these to `sifr_stdlib_model` or accept that the skip reasons render the leak inert.

### A2. Historical `cargo test -p sifr_stdlib …` rows in modified Validation Evidence tables
**Files:** `network_http_tls_traceability.md:27, 30`, `network_http_url_header_cookie_traceability.md:19, 27`

These rows sit under "Validation Evidence" / "Focused validation completed for the ... candidate" — they record commands that were actually run at the time of certification. Per M2 scope ("not over-updating historical/archive evidence"), leaving the commands at their then-current package name is consistent. The siblings in the same files that describe *current* code paths (e.g., `private _sifr.tls intrinsics in crates/sifr_stdlib_model/src/tls.rs`) were correctly migrated. This appears intentional and coherent.

### A3. Unmodified closed/historical reports
`verification/areas/stdlib_parity/reports/concurrency_runtime_typed_ipc_design.md`, `network_http_async_network_traceability.md`, and `verification/areas/runtime_platform/supported_host_matrix.md` retain many `cargo test -p sifr_stdlib …` and `sifr_stdlib::…` references in closed-status evidence tables. Not modified by this PR — that is consistent with the "do not over-update historical evidence" guidance and with the precedent set by A2. Documenting it here so reviewers don't flag it later as a missed sweep.

## Answers to Review Questions

1. **M2 rename completeness without leaving a live `sifr_stdlib` compiler crate identity** — Yes, modulo B1. The package is renamed in `Cargo.toml:8,55` and `Cargo.lock:3114`, no `[lib] name` override preserves the old crate name, no legacy re-export exists, and all four verification profiles, dependency-direction script, and HIR guardrails doc reference only `sifr_stdlib_model`. B1's stale doc reference points at a symbol that no longer exists under the old name.
2. **Generated user Cargo manifests isolated from `sifr_stdlib_model`** — Yes. `cargo_manifest.rs` uses `sifr_stdlib_model` purely as a compile-time API for feature → third-party-dependency-line mapping; the emitted manifest body only contains third-party deps and Rust interop path deps. `cargo tree -i sifr_stdlib_model` results (per validation summary) confirm zero generated-user reverse-deps.
3. **Verification profile/metadata/docs updates coherent without over-updating historical evidence** — Mostly yes. Profiles (create-pr/merge/nightly/release) and stdlib-parity data JSONs cleanly flip to `sifr_stdlib_model`. Architecture and per-domain architecture docs are coherent and explain the rename. The discipline applied (update current-state evidence, leave historical-evidence rows that record completed runs) is defensible. B1 and B2 are misses on that policy: B1 updated one line and skipped the adjacent symbol line; B2 left a forward-looking plan command pointing at a non-existent package.
4. **Blockers before opening M2 PR** — B1 and B2 above. Optionally settle A1.

review-blocked

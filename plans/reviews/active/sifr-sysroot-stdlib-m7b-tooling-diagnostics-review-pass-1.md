Now I have a complete picture. Let me write the review verdict.

---

## **FAIL** — blocking findings

This branch ships actionable LSP sysroot diagnostics and a useful mismatch response shape, but it falls short on the core M7b commitment around generated/synthetic origins, weakens the architecture doc to mask the gap, and several tests assert the rendering helpers instead of the production path. The acceptance criteria in `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:496-506` are not met.

### Blocking — `GeneratedSupport`/`CompilerSynthetic` emission is fake (M7b acceptance #3)

`crates/sifr_driver/src/stdlib/tooling.rs:97-112` adds:

```rust
fn generated_tooling_sources() -> impl Iterator<Item = WorkspaceAuxiliarySource> {
    [
        WorkspaceAuxiliarySource {
            module_name: None,
            path: SourcePath::new("<sifr-generated-support>"),
            source: SourceText::new("# compiler-generated support source-map entry\n"),
            origin: SourceOrigin::GeneratedSupport,
        },
        WorkspaceAuxiliarySource {
            module_name: None,
            path: SourcePath::new("<sifr-compiler-synthetic>"),
            source: SourceText::new("# compiler synthetic source-map entry\n"),
            origin: SourceOrigin::CompilerSynthetic,
        },
    ]
    .into_iter()
}
```

Problems:

1. These are sentinel/placeholder entries unrelated to any real generated or synthetic content the compiler produces. The plan explicitly states the work is to *"Emit `GeneratedSupport` and `CompilerSynthetic` source origins from **real production source-map paths** rather than only defining the enum variants"* (plan line 496). Hardcoded comment-only entries are exactly the "defining the enum variants" anti-pattern the plan ruled out, plus a string. AGENTS.md rule "no shortcuts… fix the root cause" applies.
2. These entries flow only through `stdlib_tooling_sources()`, whose consumers (verified with grep) are all LSP/analysis overlay hosts (`crates/sifr_analysis/src/host/{implementation,overlay_updates}.rs`). The production codegen path is untouched. No real "production files" are tagged, so acceptance #3 ("Source maps include production files tagged as `GeneratedSupport` and `CompilerSynthetic` when those sources are present") is not satisfied.
3. They pollute every LSP session's source map permanently with content that does not represent any real file.

Fix: either emit these origins from where the compiler actually produces synthetic/generated source (e.g. codegen-managed inputs, `__init__`-style scaffolding, or wherever the architecture intends them), or remove the change and document M7b as deferred. Do not stuff sentinels through the editor host pipeline.

### Blocking — architecture doc rewritten to legitimize the shortcut

`internal_docs/sifr_sysroot_and_stdlib_architecture.md:664-666` previously said:

> *"The editor source map includes user, public stdlib, and private declaration sources; production emission for generated support and compiler synthetic source files is a separate tooling source-map responsibility."*

The diff replaces that with:

> *"The editor source map includes user, public stdlib, private declaration sources, and compiler-owned virtual entries for generated support and compiler synthetic source-map contexts."*

This deletes the explicit separation between editor and production source maps that the original M7→M7b split (plan line 442-443, "production generated/synthetic origin emission are split to M7b") was predicated on, and replaces it with a vague "compiler-owned virtual entries" phrasing that exists only to match the sentinel entries above. The doc now describes the shortcut, not the architecture. Revert this paragraph until the real production emission lands.

### Blocking — mismatch tests bypass the production handler

`crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:170-252` calls `crate::requests::sysroot_status_from_probe(...)` directly with hand-rolled `ToolingSysrootProbe` values. None of these go through `requests::handle("sifr/sysroot", json!({"expectedRoot": ..., "expectedToolchainId": ...}))`. The JSON key parsing in `requests/mod.rs:58-72` (`params.get("expectedRoot")`, `params.get("expectedToolchainId")`) is untested end-to-end: renaming a key to `expected_root` would leave every mismatch test green while the live RPC silently ignored the CLI-supplied expectation. This is exactly the "private builders that miss production behavior" failure mode the review checklist (#5) called out.

Fix: add at least one mismatch case that drives `requests::handle("sifr/sysroot", …)` with both `expectedRoot` and `expectedToolchainId` JSON fields and asserts the mismatch diagnostic via the public surface. The handler dispatch + key-name contract needs an integration-style test.

### Blocking — development-sysroot CLI/LSP equivalence is not actually proven (M7b validation #2)

`sysroot_request_reports_same_root_as_analysis_tooling` (tests line 141-168) compares the LSP response against `sifr_analysis::tooling_sysroot_status()`, which is the same in-process resolver the LSP handler ultimately calls. The two paths trivially produce identical values; the test does not exercise the CLI binary. The plan validation explicitly asks for a *"Development-sysroot LSP/CLI path equivalence test"* (plan line 511) — a test that proves an unreleased build's LSP session and its CLI binary resolve the same sysroot. As written this is just a self-consistency check on one resolver. Either invoke the workspace `sifr` binary (e.g. `cargo run -p sifr -- sysroot --json` if such a flag exists, or add a JSON-emitting status mode and shell out from the test) and assert path/toolchain equivalence with the LSP probe, or document the limitation and add a follow-up.

### Non-blocking — notification message duplicates resolver paths

`crates/sifr_lsp/src/notifications/mod.rs:60-77`:

```rust
let mut message = format!(
    "{}\nbinary: {}\nattempted sysroot: {}",
    diagnostic.message,            // boundary_message() already contains binary path, attempted sysroot, asset
    diagnostic.binary_path.display(),
    diagnostic.attempted_sysroot.display()
);
if let Some(asset_path) = &diagnostic.asset_path {
    message.push_str(&format!("\ninvalid asset: {}", asset_path.display()));
}
```

`ToolingSysrootDiagnostic::message` is initialized from `error.boundary_message()` (`tooling.rs:51`, `sifr_sysroot/src/error.rs:30-44`), which already concatenates `"; binary path: …; attempted sysroot: …; missing or invalid asset: …"`. The notification then appends the same three values again, producing duplicated paths in the editor popup. Either populate `ToolingSysrootDiagnostic::message` with the bare resolver message (no path suffix) and let the notification format them, or strip the appended lines.

### Non-blocking — `window/showMessage` is a popup, not "editor diagnostics"

Plan tasks (line 490-491) and acceptance (line 503-504) phrase the requirement as "editor diagnostics." `window/showMessage` is acceptable to surface user-visible text but it is not what LSP clients treat as diagnostics (no diagnostic ID, no source, not aggregated in the problems pane, not associated with a document, not refreshable after the initial publish). Consider `textDocument/publishDiagnostics` against a synthetic sysroot URI, or at minimum document why showMessage is the chosen surface. As-is, an editor with `window/showMessage` filtered (e.g. headless smoke tests) will not see the diagnostic at all.

### Non-blocking — inconsistent diagnostic shape in the safety fallback

`requests/mod.rs:97-101` returns:

```rust
"diagnostics": ["Sifr sysroot resolver returned no status or diagnostic"]
```

That branch ships plain strings, while the broken and mismatch branches ship objects with `kind`/`message`/path fields. Clients that switch on `diagnostics[*].kind` will choke. Either remove this dead branch (the probe contract should make at least one of `status`/`diagnostic` `Some`; if it really can't, make that a server-side bug rather than an externalized "no status" string) or render an object with `kind: "internal"`.

### Non-blocking — plan status updated before the work is review-ready

`plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:19` already claims M7b *"adds … production tooling-source entries for `GeneratedSupport` and `CompilerSynthetic` origins"* as if delivered. With the synthetic-origin work being placeholder, that line is inaccurate and should be tightened ("adds sentinel virtual entries; real production emission pending") or rolled back to "in progress, partial."

### Spot-check passes
- Probe API design (`ToolingSysrootProbe { status, diagnostic }`) is clean and matches the boundary-error model.
- `sysroot_status_success` correctly propagates `kind: "resolved" | "mismatch"`, `observedPaths.sysroot`, and `expectedRoot/Toolchain*` for the mismatch case.
- `sysroot_status_failure` surfaces `binaryPath`, `attemptedSysroot`, `assetPath` both as structured `diagnostics[0]` fields and as `observedPaths.*` — that mirrors acceptance #2 well.
- No user-path panics added; the resolver's `Result<_, SysrootError>` is consumed safely.
- No global mutable sysroot state introduced.

### Suggested next steps before re-review
1. Remove `generated_tooling_sources()` and replace with a real production emission path — or revert the synthetic-origin work entirely and re-scope M7b to ship without it.
2. Revert the `internal_docs/sifr_sysroot_and_stdlib_architecture.md` paragraph to its original "separate tooling source-map responsibility" wording, or rewrite it accurately once real emission exists.
3. Add an end-to-end `requests::handle("sifr/sysroot", json!({"expectedRoot": …, "expectedToolchainId": …}))` test for the mismatch case.
4. Add a development-sysroot test that compares LSP probe output against an invoked CLI binary, or document why this isn't possible in this repo and add a follow-up item.
5. De-duplicate paths in `tooling_sysroot_notification` and consider `publishDiagnostics`.
6. Decide on the safety-fallback shape in `sysroot_status_from_probe`.
7. Soften the M7b row in the plan to reflect the partial state.

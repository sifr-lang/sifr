I've reviewed the full diff against main, focusing on whether the pass-1 blockers are resolved. Findings below.

## PASS — M7b is PR-ready

All four pass-1 blockers are resolved, and the non-blocking items called out in pass 1 are largely addressed too. The remaining unresolved item (`window/showMessage` vs `publishDiagnostics`) was flagged non-blocking by pass 1 and remains a reasonable follow-up.

### Pass-1 blocker resolution

**Blocker 1 — Fake `GeneratedSupport`/`CompilerSynthetic` sentinels: RESOLVED.**
`generated_tooling_sources()` is gone. Production emission now lives in `crates/sifr_driver/src/frontend/api.rs:99-124`:
- `GeneratedSourceMapFile { path: "src/main.rs#stdlib-preamble", origin: GeneratedSupport, source }` slices the real stdlib preamble from `codegen_result.rust_source`, bounded by the new `// --- end stdlib ---` marker introduced in `crates/sifr_codegen/src/lib_modules_and_codegen.rs:815`.
- `CompilerSynthetic` maps the actual full generated `src/main.rs` Rust source.
- Source bytes are real generated Rust, not placeholder strings.
- Metadata propagates through `CompileResultFull::Success.generated_source_map` (`diagnostics.rs:20-33`), `GeneratedRustPreview.source_map_files` (`queries.rs:149`), and the LSP `sourceMapFiles` JSON (`conversion.rs:472-485`).
- Verified no remaining overlay-host pollution: only the new test files reference these origins outside the production code path.

**Blocker 2 — Architecture doc rewritten to legitimize the shortcut: RESOLVED.**
`internal_docs/sifr_sysroot_and_stdlib_architecture.md:662-666` now reads "Generated Rust preview metadata emits production source-map entries for generated support and compiler synthetic contexts from the actual Rust source produced by the compiler" — this accurately describes the production-path implementation rather than describing virtual sentinels.

**Blocker 3 — Mismatch tests bypass the production handler: RESOLVED.**
`crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs` now drives the production handler via `crate::requests::handle(&mut session, "sifr/sysroot", json!({"expectedRoot": …, "expectedToolchainId": …}))` in:
- `sysroot_request_handler_reports_expected_root_mismatch` (line 178)
- `sysroot_request_handler_reports_expected_root_and_toolchain_mismatch` (line 275)
The JSON key contract (`expectedRoot`, `expectedToolchainId`) is exercised end-to-end. A rename would now fail these tests.

**Blocker 4 — Development CLI/LSP equivalence not proven: RESOLVED.**
`development_sysroot_request_reports_same_root_as_cli` (line 142) shells out via `cargo run --locked -q -p sifr -- --print sysroot --json`, parses the JSON, and passes the CLI's root and toolchain to the LSP handler as `expectedRoot`/`expectedToolchainId`. The assertion `ok == true` requires both the CLI binary and the in-process LSP resolver to agree. `crates/sifr/src/sysroot_cli.rs:20-67` confirms the CLI flag exists and emits the required JSON keys.

### Pass-1 non-blocking items (status check)

- **Notification path duplication** — RESOLVED. `ToolingSysrootDiagnostic.message` now uses `error.message` (bare resolver text) at `stdlib/tooling.rs:50`, not `boundary_message()` (verified against `crates/sifr_sysroot/src/error.rs` and `layout.rs` where messages like "missing manifest"/"workspace manifest could not be read" carry no embedded paths). The notification appends paths once.
- **Inconsistent diagnostic shape on the safety fallback** — RESOLVED. `requests/mod.rs:101-110` now returns `{"kind": "internal", "message": …}` instead of a bare string.
- **Plan status row** — softened to "in progress" with truthful evidence; `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:19` no longer claims the synthetic-origin work was already merged.
- **`window/showMessage` vs `publishDiagnostics`** — unchanged. This was non-blocking per pass 1; acceptable to defer.

### Additional checks

- File-size guardrail respected: `host.rs` adds the new submodule, `generated_rust_preview_tests.rs` is split out.
- No data-dependent `.unwrap()`/`.expect()` introduced on user paths.
- `CompileResultFull::Success` fan-out (e.g., `crates/sifr/tests/e2e_support/fixture_compilation.rs:83`) destructures with `..` so the new field is backward-safe for downstream callers.
- `generated_support_source` correctly returns `None` when no stdlib preamble is present (verified by the no-import test fixture), so `GeneratedSupport` is omitted on programs that don't use stdlib.
- `--locked` nested-cargo test is slow but functionally correct; if it proves flaky in CI it can be marked `#[ignore]` and gated behind an env flag later — not a blocker.

### Minor non-blocking notes (not gates)

- `generated_support_source` in `frontend/api.rs:119-124` keeps fallback anchors `\n\nfn main` and `tail.len()` after the codegen unconditionally emits `// --- end stdlib ---`. These branches are now unreachable; consider tightening to `expect("end stdlib marker missing")` or returning `None` if the marker is absent, so a future codegen change that drops the marker would fail loudly instead of silently slicing past the body.
- The architecture doc could optionally note that `CompilerSynthetic` is the full file and `GeneratedSupport` is a substring slice of it (overlap is intentional). Not required.

**Verdict: PASS.** Merge after the create-pr profile runs green.

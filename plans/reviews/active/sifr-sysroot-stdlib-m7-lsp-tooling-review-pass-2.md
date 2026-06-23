# M7 Review — LSP and Tooling Sysroot Integration (pass 2)

Branch: `sifr-sysroot-stdlib-m7-lsp-tooling`
Reviewer: Opus 4.7
Scope: Re-audit current diff against `M7` tasks/acceptance/validation in
`plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md` lines 437–476,
verifying that pass-1 blockers and high-severity concerns have been addressed.

## Verdict: FAIL

Pass-2 closed most of the high-severity concerns and one of the three blockers,
but two of the three blockers remain unresolved in production code: B1's
broken/mismatched sysroot diagnostic *detection-and-publish* flow is not in
the diff, and B2's `GeneratedSupport` / `CompilerSynthetic` source origins
are still dead variants in production with only a constructed-by-hand unit
test. The issue plan still does not acknowledge or explicitly defer either.
M7 cannot close until these are either implemented or formally deferred with
a milestone owner.

## Pass-1 status

| Item | Pass 1 | Pass 2 | Notes |
|------|--------|--------|-------|
| B1. Sysroot mismatch / broken diagnostics | unresolved | partial — see below | `sifr/sysroot` query + equivalence test only |
| B2. `GeneratedSupport`/`CompilerSynthetic` are dead variants | unresolved | unresolved | Only a hand-built `SourceMapView` test |
| B3. Public-wrappers vs internal/dev context | unresolved | resolved | `allow_private` gated on file origin; dev-context test added |
| H1. Hand-rolled, non-parser symbol scan | unresolved | resolved | Now uses `sifr_python_ast` + `sifr_syntax::parse_module` |
| H2. Definition routes only on import token | unresolved | resolved | `stdlib_import_target` scans every `from ... import` line; call-site test asserts |
| H3. `host.files()` semantics quietly changed | unresolved | resolved | New `all_files()` separate from `files()`; LSP file maps use `all_files()` |
| M1. Double overflow check in `auxiliary_source_states` | unresolved | unresolved | Minor; not a blocker |
| M2. `range_for_name` u32→usize→u32 | unresolved | resolved | Old scanner removed; uses parser's `range()` |
| M3. Stdlib bucket rebuilt on every refresh | unresolved | partly worse | Now reparses every public stdlib file per refresh |
| M4. Hover test does not exercise on-disk sysroot | unresolved | unresolved | Test still asserts external-defs signature |
| M5. Issue-plan delta understates scope gap | unresolved | unresolved | Plan still claims M7 implements everything; does not defer B1/B2 |

## Remaining blockers

### B1 (partial). Tooling diagnostic for broken or mismatched sysroot is missing detection-and-publish

The diff now provides:

- `sifr_driver::stdlib_tooling_sysroot_status()` returning `{root, toolchain_id}`.
- A re-export through `sifr_analysis::tooling_sysroot_status()`.
- An LSP request handler `sifr/sysroot` that responds with `{ok, root, toolchainId}` or `{ok: false, diagnostics: [...]}`.
- One test asserting the LSP and the analysis-side query agree on `(root, toolchainId)` for the same process (`sysroot_request_reports_same_root_as_analysis_tooling`, `sysroot_request_tests.rs:139-160`).

That covers part of the acceptance bullet *"CLI and LSP report the same sysroot path for the same installation"* — there is now an inspectable API that returns a value the CLI can compare against. But the M7 task list also requires three things this diff does NOT do:

1. **"Add tooling diagnostics when the editor process sees a broken or mismatched sysroot."** A query API is not a diagnostic. There is no code path in `sifr_lsp` that:
   - Detects a broken sysroot (resolution failure) at session bring-up and proactively `publishDiagnostics` (or any tooling notification) to inform the editor — `requests/mod.rs:57-69` only answers if the editor asks first, and a non-asking client never learns.
   - Detects a *mismatch* between two sysroots (e.g. an editor process whose `resolve_sysroot(None)` result differs from the workspace's CLI-installed receipt, or a development sysroot that doesn't match the binary's expected toolchain). `grep -nR "publishDiagnostics" crates/sifr_lsp/src` only returns the existing source-diagnostic publishers — there is no sysroot diagnostic flow.
2. **"Add CLI/LSP sysroot mismatch diagnostics that include the observed sysroot paths where available."** No diagnostic code, no test asserts what a mismatched-sysroot diagnostic looks like, and no plumbing carries observed paths into a `RenderedDiagnostic` body.
3. **"Add development sysroot behavior so local LSP sessions use the same resolved sysroot as CLI when running from an unreleased build."** The LSP and analysis layers call `sifr_sysroot::resolve_sysroot(None)` directly, which delegates to the same resolver the CLI uses, so in the happy path they line up — but this is implicit, untested by the LSP layer (no test pins LSP behavior under `SIFR_DEV_SYSROOT` or development-mode resolution), and there is no surface for tests to verify that "LSP sessions running from an unreleased build use the dev sysroot."

Action: implement at least (a) a sysroot-resolution probe on session init that, on failure, publishes a single tooling diagnostic containing `error.boundary_message()` and the resolver-observed paths, and (b) a comparison check that emits a mismatch diagnostic when the LSP-resolved sysroot diverges from what the active workspace's CLI receipt records. Cover both with LSP-level integration tests. Until then, this acceptance bullet is observable through one query API only, not as an actual editor-visible diagnostic.

### B2. `GeneratedSupport` and `CompilerSynthetic` origins are still dead variants in production

`rg -n "GeneratedSupport|CompilerSynthetic" --type rust` returns six hits and all six are in `crates/sifr_frontend/src/source_maps.rs`:

- `source_maps.rs:81-82` — the enum declaration.
- `source_maps.rs:263, 272, 284, 288` — a single unit test (`source_map_views_distinguish_generated_and_synthetic_origins`) that builds a `SourceMapView` *literal* with these origins and asserts the variants are matchable.

No production code path in `sifr_frontend`, `sifr_driver`, `sifr_codegen`, or `sifr_lowering` tags any real `SourceFileView` as either origin. `SourceFileView` instances are produced in two places — `graph_cache_and_queries/reuse.rs:237-260, 270-282` (user modules → `UserSource`) and `source_maps.rs::AuxiliarySourceState::new` (auxiliary sources → whatever the caller passes; in practice only `SysrootPublicStdlib` and `SysrootPrivateDeclaration` from `sifr_driver::stdlib::tooling::tooling_sources`).

M7 acceptance still says:

> Source maps correctly distinguish public stdlib, private declarations,
> generated support, compiler synthetic, and user files.

A source map cannot "distinguish" an origin no real source ever wears. The pass-1 review anticipated and rejected the "we added a unit test" answer; the diff supplies exactly that and nothing else. Per-test enumeration of variants is not the same as production code emitting them, and `analysis_source_map_tracks_public_and_private_sysroot_origins` only validates three of the five.

Action: either (a) tag the relevant pipelines so the variants are emitted in production (`sifr_codegen` output buffers as `GeneratedSupport`; compiler-introduced preamble/synthetic sources as `CompilerSynthetic`) with assertions in production tests, OR (b) explicitly defer `GeneratedSupport`/`CompilerSynthetic` to a follow-up milestone in the issue plan and update M7 acceptance to drop those two origin kinds from the closing requirement.

## Concerns from pass-1 that were resolved

- **B3** — `stdlib_navigation.rs:29-33` reads `source_file.origin` to gate `allow_private`. The new `definition_inside_public_stdlib_can_link_to_private_declaration_file` test (`stdlib_tests.rs:179-211`) proves a click inside `sifr.math` resolves a `_sifr.math` import to the private declaration file, and `stdlib_symbol_bucket_is_available_without_private_declarations` proves `_sifr.*` never leaks to user completion. Sensible "developer context" definition via file origin — clean.
- **H1** — `stdlib_navigation.rs` now imports `sifr_python_ast::{Expr, Stmt}` and walks the real AST. `Stmt::FunctionDef`, `Stmt::ClassDef`, `Stmt::AnnAssign`, and single-target `Stmt::Assign` produce stdlib symbol entries with real `TextRange`s from the parser. No more regex/`def `/`class ` prefix scanning. `async def`, decorated definitions, and multi-line signatures are now handled by virtue of using the same parser the rest of the frontend uses. Removing the regex implementation and migrating to AST traversal is exactly the fix.
- **H2** — `stdlib_import_target` (`stdlib_navigation.rs:208-224`) iterates every line in the source, finds any `from sifr.* import ...` (or `_sifr.*` in dev context), and matches the token against imported/alias names. The new `definition_request_for_stdlib_call_returns_sysroot_uri` LSP test (`sysroot_request_tests.rs:34-60`) clicks `randint(1, 2)` at the use site on line 4 (not the import line) and asserts the resolved URI ends with `stdlib/sifr/random.sifr`. Behavior now matches what users actually exercise.
- **H3** — `AnalysisHost::all_files()` (`implementation.rs:152-164`) is a deliberate, separate method that returns every file in the source map (including aux files); `AnalysisHost::files()` is unchanged (`file_to_module.keys()`). `LspProjectAnalysis::file_maps` and `uri_by_file` (`analysis_workspace.rs:400-450`) deliberately consume `all_files()` for URI minting. `analysis_source_map_tracks_public_and_private_sysroot_origins` pins the contract: `host.files().len() == 1` while `host.all_files().len() > host.files().len()`. Good — the contract is explicit and tested.
- **M2** — The hand-rolled `range_for_name` is gone; the AST-based extractor stores `Some(stmt.range())` from the parser directly. No casting churn.

## Newly noted concerns (low/medium)

### N1. `LspDocumentAnalysis::file_maps` now silently filters `Err` from `source_text_for_file`

`analysis_workspace.rs:574-588` was rewritten so a standalone document's file map now also includes every aux file from `host.all_files()`. The `if let Ok(source) = host.source_text_for_file(mapped_file)` branch silently drops files whose source text fails to resolve. For aux files this is benign (they always resolve), but the silent-drop pattern is a footgun if a future origin kind ever surfaces a fallible source provider. Either log/return the error or document the invariant.

### N2. Stdlib bucket rebuild does a full parse on every refresh

`AnalysisHost::refresh_stdlib_symbol_bucket` is called from both `symbol_index()` (`implementation.rs:655`) and `refresh_existing_symbol_index` (`implementation.rs:680`). Each call re-parses every public stdlib file via `sifr_syntax::parse_module` (`stdlib_navigation.rs:130-133`). On a project with many modules every user-side edit triggers a partial refresh, which now reparses every stdlib source even though the auxiliary sources are immutable.

Pass-1 M3 flagged this churn at regex speed; the H1 fix moved it to parser speed. Cache the stdlib bucket by `(auxiliary-source revision, analysis revision)` so the parse runs once per session reload and not on every dirty-module refresh.

### N3. `sifr/sysroot` LSP request returns no paths in the error body

`requests/mod.rs:57-69` matches on `Ok`/`Err`. On `Err` the diagnostic messages are surfaced but observed paths are NOT — `RenderedDiagnostic` only carries the rendered message string, not the paths the resolver tried. This is exactly the gap the M7 task calls out: *"include the observed sysroot paths where available."* The resolver in `sifr_sysroot::resolve_sysroot` knows the paths it inspected; the failure surface needs to carry them out so the editor can show "we looked at /Foo, /Bar." As-is the diagnostic shows only `error.boundary_message()` which is human prose, not a structured path list.

### N4. The new `sifr/sysroot` request handler bypasses the request-tracing layer

`session::trace_snapshot()` and friends record a phase trace for compiler requests. The new `sifr/sysroot` arm in `requests/mod.rs` runs synchronously and never records a trace span. If the goal is to use this surface for diagnostics, it should at minimum log via the existing trace machinery so `sifr/debugTrace` reflects the call.

### N5. Issue-plan delta still asserts more than the code delivers

The plan's M7 line claims the work *"routes public stdlib import definitions to installed sysroot source URIs"* and notes focused validation. It does not mention the still-missing pieces from B1 (no mismatch/broken-sysroot diagnostic publish) or B2 (origins are stubbed), and the M7 acceptance block (lines 437–476) is unchanged. Either close those gaps in code, or update the issue plan to defer them with a named follow-up milestone and tighten M7's scope before closing.

## What the PR does well in pass 2

- Real AST replacing the regex toplevel scanner is a substantial quality lift; the resulting bucket is more accurate AND has real ranges, which is what the LSP definition response needs.
- The `allow_private` gate keyed on the *navigating file's* origin is a clean, code-driven dev-context distinction — no flag plumbing, no capability negotiation, and the test that proves public→private linkage works inside stdlib code is convincing.
- The `all_files()` / `files()` split makes the host contract explicit and adds a guard against future regressions.
- The call-site definition test exercises the path users actually take.

## Required to close M7

1. Implement (or explicitly defer) sysroot mismatch / broken-sysroot diagnostic flow: detection on session init, structured paths in the diagnostic body, mismatch comparison between observed LSP and CLI sysroots, and LSP-level integration tests covering broken-sysroot and mismatch cases. If deferring, name the milestone in the issue plan and drop the corresponding M7 acceptance bullets.
2. Either tag production sources with `GeneratedSupport` / `CompilerSynthetic` (with real production-path tests), or remove the two variants and defer them to a follow-up milestone in the issue plan with the M7 acceptance text updated accordingly.
3. Update the issue plan's M7 line to reflect actual scope (mention deferrals, or remove the implicit claim that all of M7's tasks landed).
4. Optional but encouraged: cache the stdlib bucket (N2), surface observed paths in the broken-sysroot response body (N3), and strengthen one hover test to bind output to the actual on-disk sysroot source (pass-1 M4).

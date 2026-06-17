Now I have enough material. Let me write the audit report.

## TypeScript-Go Architecture Transfer — Pass 4 Audit

Scope: AC-3, AC-10, AC-11, AC-29..AC-31. Focus on package source-map construction, import resolution states, ambiguity preservation, source-diagnostic code separation, canonical source spans, source-position conversion, and runtime fixtures.

### Verdict
Phase is broadly **SATISFIED** for AC-29..AC-31 at the source-map and direct unit-test layer. Five issues worth flagging, none of them blocking; only one rises to **medium**.

---

### Findings

#### F1 — MEDIUM: `candidate_paths` separator differs between project and package modes for the same `SIFR-IMPORT-0005`

`SIFR-IMPORT-0005` is emitted from two different sites with diverging argument formats:

- Project (workspace) mode — `crates/sifr_driver/src/project/discovery.rs:441-446` joins candidate paths with **`;`**. The frozen baseline `crates/sifr/tests/verification/project/workspace_ambiguous_import/baselines/check-json.stderr.txt:10` enforces `;`.
- Package mode — `crates/sifr_driver/src/project/package_discovery.rs:215-220` joins candidate paths with **`, `**.

The canonicalization contract (`verification/tooling/check_diagnostic_source_canonicalization_rules.py:329`) only asserts that `candidate_paths` exists as a JSON arg, so the divergence goes unchecked. An editor or CI consumer that diff-checks `args.candidate_paths` against the project baseline will see a regression as soon as the same fixture is reshaped from workspace into a package.

Why it matters for AC-29: AC-29 promises the package ambiguity diagnostic carries "candidate paths, resolution scope, package id, cargo package id, import root or source root context" — it does, but in a format that disagrees with the canonical project flavor of the same diagnostic code.

Experiment:
- Switch `package_discovery.rs:215-220` to `.join(";")` and add a baseline fixture for `package_ambiguous_import_canonical` capturing the JSON output (currently the fixture has `Cargo.toml`, `sifr.toml`, sources, but no `baselines/` directory — see `ls` of the fixture). Alternatively, extend `check_diagnostic_source_canonicalization_rules.py:285-300` to assert the same separator for the workspace and package fixtures.

---

#### F2 — LOW: Entry-module resolution collapses ambiguity to `SIFR-PACKAGE` and discards rich state

`crates/sifr_driver/src/project/package_discovery.rs:296-313` (`package_import_resolution_for_discovery`) is called once per pending discovery item, **including the entry**. When `resolve_import_result` returns `Ambiguous`, the entry path turns it into `PackageDiagnostic::undeclared_direct_import(...)` (a `SIFR-PACKAGE-*` family code) with the candidates flattened into a free-text "ambiguous candidates: …" string.

This collapses the rich `PackageImportAmbiguity` state (origin, package_id, candidate `PackageModuleSource`s, resolution scope) into one string, and emits the `SIFR-PACKAGE` code that AC-29 reserves for "fatal package-map errors". For the entry, there is no import site to attach a span to, but the source map *is* otherwise valid — this is exactly the case AC-29 says should not be reported as a package-fatal failure.

Concrete repro path: two sifr source roots that both contain a `main.sifr` whose dotted module path is `main` would land in `ambiguous_modules`. `module_for_file` (`crates/sifr_package/src/imports/source_map.rs:342-365`) successfully resolves the entry by file path, but the discovery loop immediately re-resolves it by dotted name and emits `SIFR-PACKAGE-*`.

Severity is low because the duplicate-entry-name shape is unusual in practice, but the code path violates the AC-29 "package-fatal vs ambiguity" boundary.

Experiment:
- Add a fixture with `src_a/main.sifr` and `src_b/main.sifr`, run `sifr check src_a/main.sifr`, and assert: either (a) the entry resolution uses `module_for_file`'s disambiguated `PackageModuleSource` and no diagnostic is emitted (preferred), or (b) the diagnostic is `SIFR-IMPORT-0005` and not `SIFR-PACKAGE-*`.
- Implementation fix: in `parse_package_import_closure_source_modules` (`package_discovery.rs:49-54`), short-circuit on the *initial* entry item with the `PackageModuleSource` already returned by `module_for_file` instead of re-resolving by dotted path.

---

#### F3 — LOW: Package source-map construction and the package import closure do not record source-dependency reads (AC-3 partial gap)

`crates/sifr_package/src/imports/source_map.rs:83` and `crates/sifr_driver/src/project/package_discovery.rs:42` both instantiate `DiskSourceProvider::new()` directly — they do not wrap it in `TrackingSourceProvider`. By contrast, frontend project loading wraps it (see `crates/sifr_frontend/src/graph_cache_and_queries.rs:354` and `crates/sifr_frontend/src/workspace_session.rs:296`).

AC-3 says: "Frontend/project module discovery records successful reads, directory reads, config/package reads, and failed lookup dependencies." Package source-map construction reads do affect compilation identity (they decide which modules are ambiguous, which `__init__.sifr` files exist, etc.) but the reads are not tracked. The M2 doc (`internal_docs/typescript_go_architecture_transfer_m2_source_provider.md:46-65`) explicitly notes that wiring tracking into session snapshots is M3-M6 scope, and the phase tracker marks all milestones merged.

Severity low — phase closeout already acknowledges this — but the gap is real: any future dependency-sensitive invalidation that touches package state must take this path through `TrackingSourceProvider` first.

Experiment:
- Replace the bare `DiskSourceProvider::new()` calls with a wrapped `TrackingSourceProvider::new(DiskSourceProvider::new())`, expose a `*_tracked` variant of `PackageSourceMap::build` and `parse_package_import_closure_source_modules` returning the captured `Vec<SourceDependency>`, and a unit test that asserts the dependency record includes the read for `helper.sifr` plus the `read_dir` of `src_a`/`src_b`.

---

#### F4 — LOW: `module_for_file` returns ambiguous candidates without signaling ambiguity to its caller

`crates/sifr_package/src/imports/source_map.rs:342-365` iterates `self.modules.values()` *and* `self.ambiguous_modules.values().flat_map(...)`. A caller that finds a module via file-path lookup has no way of knowing whether the resolved module shares its dotted path with siblings. This is the root cause that combines with F2 to make the entry-path edge case visible.

Severity low. Concrete suggestion: change `module_for_file` to return `Option<ModuleForFileMatch>` with `kind: Unique | Ambiguous(Vec<...>)`, or split into `unique_module_for_file` / `any_module_for_file` and have the driver call the former for the entry.

---

#### F5 — OBSERVATION: Diagnostic span column is USV-based but LSP advertises UTF-8 (cross-cutting; touches AC-11 for diagnostic spans)

`crates/sifr_diagnostics/src/render/mod.rs:272-286` computes `column = prefix.chars().count() + 1` (Unicode-scalar count). `crates/sifr_lsp/src/conversion.rs:407-418` (`diagnostic_span_range`) hands that column directly to the LSP `character` field after `saturating_sub(1)`. The server advertises `positionEncoding: UTF-8` (`crates/sifr_lsp/src/capabilities.rs:28`), which per LSP spec means `character` is UTF-8 code-units, not USVs.

The pipeline through `lsp_range_with_encoding(..., Utf8)` (line 49-75) is correct because it goes through `sifr_source::SourceText::byte_offset_with_encoding`. The renderer-derived path for diagnostics is the one that disagrees with the advertised encoding. The package and project fixtures are ASCII-only, so contract tests do not catch this.

Out of strict scope for pass 4 (this is an LSP/diagnostic-rendering boundary, not a package source-map issue), but I'm flagging it because:
- the same `package_import_ambiguity_source_diagnostic` rendered span is what an editor will see for `SIFR-IMPORT-0005`, and
- AC-11 promises "editor-safe UTF-8/UTF-16 round trips" — for diagnostics specifically, the round trip is broken whenever the prefix on the same line contains non-ASCII.

Experiment:
- Add a non-ASCII fixture (e.g., a variable named `café` before the ambiguous `from helper import value`) and compare the LSP diagnostic `range.start.character` to the byte offset computed via `byte_offset_with_encoding(..., Utf8)`. They should agree; today they will not.
- Fix: route `diagnostic_span_range` through `byte_offset_with_encoding` using `DiagnosticSpan.byte_start`/`byte_end` against the document's source text, instead of trusting the renderer's USV column.

---

### What I verified is fine
- `PackageSourceMap.build` no longer rejects duplicate module paths into `PackageDiagnostic` — duplicates land in `ambiguous_modules` and survive construction (`source_map.rs:308-323`). AC-30 ✓
- `resolve_import_result` exposes all five states (Resolved / Ambiguous / Unresolved / PrivateAccess / FatalPackageMapFailure), each carrying enough state for source-level diagnostics (`source_map.rs:73-78`, `:182-306`). AC-31 ✓
- `PackageImportAmbiguity` carries `package_id`, `cargo_package_id`, `module_path`, `candidates`, and `origin` (with `DirectDependency { import_root, target_export_root, dependency_package_id }`) — none of these are dropped on the source-diagnostic path (`package_discovery.rs:209-275`). AC-29 candidate-path/scope/context ✓
- Fatal-package short-circuit: `resolve_import_result` checks `fatal_diagnostics` first (`source_map.rs:183-187`); the CLI `Err`-path on `PackageSourceMap::build` renders only `SIFR-PACKAGE-*` and exits before any import discovery runs (`crates/sifr/src/check_and_package_commands.rs:229-238`). No SIFR-IMPORT companion. AC-29 fatal-vs-source separation ✓
- The canonicalization contract pins the negative-duplication check both ways: `forbidden_prefixes=("SIFR-PACKAGE-",)` for `package_ambiguous_import_canonical` and `forbidden_prefixes=("SIFR-IMPORT-",)` for `package_fatal_source_map_no_import_ambiguity` (`verification/tooling/check_diagnostic_source_canonicalization_rules.py:347, 373`). AC-31 ✓
- `SourceMapView::text_position_to_span` / `span_to_text_range` return real values for valid positions and `None` only for genuinely invalid ones; UTF-16 surrogate-pair-interior rejection is exercised (`crates/sifr_frontend/src/source_maps.rs:107-211`). AC-11 (for non-diagnostic paths) ✓
- `SourceSpan::new_validated` and `SourceMap::validate_span` reject out-of-bounds spans before rendering (`crates/sifr_diagnostics/src/source_map/mod.rs:139-156`), and `diagnostic_with_source_range` short-circuits to `INTERNAL_COMPILER_PANIC` rather than mis-rendering when the import-site span is invalid (`crates/sifr_driver/src/project/discovery.rs:407-415`). Canonical source spans ✓
- M17 fixtures exist for both directions of the no-duplication invariant; the M2 unit tests cover all five resolution states (`crates/sifr_package/src/milestone_adhoc_tsgo_m2_tests.rs`).

### Summary
Pass 4 finds **no AC-29..AC-31 violations that would re-open the phase**. The medium-severity item is a contract drift (`candidate_paths` separator) that's easy to converge before any other diagnostic consumer downstream relies on the workspace flavor. The four low-severity items are edge cases (F2/F4), an explicitly-deferred wiring gap (F3), and a cross-cutting LSP-column encoding observation (F5).

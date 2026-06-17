# Ad Hoc Phase: Diagnostic Source Canonicalization

Status: complete on 2026-05-29

Pre-flight requirement: implement M1 first. All later milestones depend on the source-canonicalization contract checker existing, being wired into `scripts/run_all_tests.sh --profile quick`, and failing against the current known gaps before producer implementation begins.

## Purpose

Make diagnostic identity and source attachment consistent across compiler phases now that diagnostic presentation is source-aware.

The production-grade diagnostic presentation phase fixed how `RenderedDiagnostic` values are displayed. Follow-up fuzzing showed that several upstream producers still emit spanless diagnostics or phase-specific diagnostic codes for source-level problems. This phase makes diagnostic producers supply canonical codes, source spans, and useful context so `human`, `compact`, and `json` modes can all present the same underlying problem consistently.

The goal is:

- Source problems should point at source.
- The same user-facing problem should use the same diagnostic code regardless of whether it is discovered in single-file, workspace, package, or editor flows.
- Workspace/package details should be context on canonical source diagnostics, not a replacement for the canonical source diagnostic identity.

## Source Inputs

This phase is based on:

- Diagnostic presentation phase: `issues/ad-hoc-production-grade-diagnostic-presentation.md`
- Scratch gap audit: `tmp/diagnostic_gap_fuzz/README.md`
- Parser diagnostic construction in `crates/sifr_syntax/src/lib.rs`
- Project module discovery in `crates/sifr_driver/src/project/discovery.rs`
- Project compile ordering in `crates/sifr_driver/src/project/compile_order.rs`
- Package diagnostic model in `crates/sifr_package/src/diag/`
- Package diagnostic conversion in `crates/sifr/src/cli_model_and_entrypoint.rs`
- Driver package diagnostic conversion in `crates/sifr_driver/src/project/discovery.rs`
- Existing diagnostic renderer and contract tests in `crates/sifr_diagnostics` and `verification/tooling/check_diagnostic_presentation_rules.py`

## Current Findings

Ad hoc fuzzing under `tmp/diagnostic_gap_fuzz` found these gaps.

### Parser diagnostics are spanless

Examples:

- bad indentation
- unexpected indentation
- unterminated strings
- malformed function signatures
- empty declaration lists
- malformed declaration lists
- invalid call argument order
- invalid match patterns
- unsupported syntax

Current JSON shape:

```json
{
  "code": "SIFR-PARSE-0002",
  "spans": []
}
```

Root cause: `ParseError` has a `location: TextRange`, but `crates/sifr_syntax/src/lib.rs` converts only `ParseError.error` into a `RenderedDiagnostic` and hardcodes `spans: Vec::new()`.

### Workspace/package module resolution diagnostics are spanless and phase-coded

Examples:

```sifr
from missing_helper import value
```

Current package-aware result:

```text
SIFR-WORKSPACE-0101 <unknown> could not resolve import 'missing_helper'; tried entry-relative ...
```

Current JSON shape:

```json
{
  "code": "SIFR-WORKSPACE-0101",
  "spans": []
}
```

This is a source-level import problem and should use canonical import identity with source context.

### Ambiguous import diagnostics are spanless

Example:

```sifr
from helper import value
```

where `helper.sifr` exists in multiple workspace/package source roots.

Current result:

```text
SIFR-WORKSPACE-0102 <unknown> module 'helper' is ambiguous ...
```

The diagnostic should point at the import statement or module name and include candidate paths as notes/help.

### Import cycle diagnostics are spanless

Example cycle:

```text
a imports b
b imports a
```

Current result:

```json
{
  "code": "SIFR-WORKSPACE-0104",
  "spans": []
}
```

The compiler knows the module cycle but not the source import edges because the dependency graph stores module names only. Human output should identify at least one cycle-causing import, and ideally all cycle edges as related spans.

### Package diagnostic conversion drops origin and help

`PackageDiagnostic` contains:

- `code`
- `message`
- `origin`
- `help`

The CLI and driver convert package diagnostics with `diagnostic_with_code(...)`, which drops `origin` and `help`. This weakens human output and prevents origin-derived source/config context from reaching JSON.

## Product Decision

Diagnostic code identity describes the canonical user problem, not the compiler phase that discovered it.

Use source-level diagnostic families for source-level problems:

- Missing module import: `SIFR-IMPORT-0002`
- Unsupported import syntax/form: `SIFR-IMPORT-0003`
- Missing imported member: `SIFR-NAME-0004`
- Ambiguous module import: add `SIFR-IMPORT-0005`
- Namespace/file import collision: add `SIFR-IMPORT-0006`
- Circular import graph: add `SIFR-IMPORT-0007`
- Parse/lexical/layout errors: `SIFR-PARSE-*`

Use workspace/package diagnostic families for workspace/package system problems:

- malformed `sifr.toml`
- invalid package source roots
- ambiguous package selectors
- undeclared package dependencies
- private package module access
- package archive/projection/repair problems
- command target outside package source root

Workspace/package resolution should still expose resolver details, but as structured context on canonical diagnostics:

- child notes for candidate paths
- help text for remediation
- JSON args for resolution scope and candidate paths
- related spans when multiple source locations matter

Missing module and missing imported member remain separate canonical problems:

- `SIFR-IMPORT-0002` means the module target itself did not resolve.
- `SIFR-NAME-0004` means the module resolved, but the requested exported member was not found.

Discovery must prevent duplicate reporting: if module resolution fails before HIR lowering, the missing-module import diagnostic is emitted once and HIR must not re-report the same source import as a missing member or generic name failure.

## Target Behavior

### Parser diagnostics

Human:

```text
error[SIFR-PARSE-0002]: syntax error: expected an indented block after function definition
  --> main.sifr:2:1
     |
   2 | print("bad indent")
     | ^^^^^
  = docs: https://sifr.sh/docs/errors/SIFR-PARSE-0002
```

Compact:

```text
1 error, 0 warnings, 0 notes
E SIFR-PARSE-0002 main.sifr:2:1 syntax error: expected an indented block after function definition
```

JSON:

```json
{
  "code": "SIFR-PARSE-0002",
  "spans": [
    {
      "file": "main.sifr",
      "line": 2,
      "column": 1,
      "is_primary": true
    }
  ]
}
```

### Missing module import

Human:

```text
error[SIFR-IMPORT-0002]: unknown import target: 'missing_helper'
  --> src/main.sifr:1:6
     |
   1 | from missing_helper import value
     |      ^^^^^^^^^^^^^^
  = note: tried entry-relative src/missing_helper.sifr
  = note: tried package source root src/missing_helper.sifr
  = docs: https://sifr.sh/docs/errors/SIFR-IMPORT-0002
```

Compact:

```text
1 error, 0 warnings, 0 notes
E SIFR-IMPORT-0002 src/main.sifr:1:6 unknown import target: 'missing_helper'
```

JSON must preserve:

- canonical code `SIFR-IMPORT-0002`
- primary span for the import module or import statement
- resolver scope args, for example `resolution_scope`
- tried path args, for example `tried_paths`
- child notes for human-readable attempted paths

### Ambiguous module import

Ambiguous source-level imports must use `SIFR-IMPORT-0005`. They should not use `SIFR-WORKSPACE-*` solely because discovery found them before HIR lowering.

The diagnostic should include:

- primary span on the importing source statement
- candidate paths as notes/help
- JSON args with candidate paths and resolution scope

### Namespace/file import collision

Namespace/file collisions must use `SIFR-IMPORT-0006` when triggered by a source import.

The diagnostic should include:

- primary span on the importing source statement
- child notes naming the colliding file paths
- JSON args for the requested module, parent module, resolved path, and parent path

### Import cycles

Import cycles triggered by source import edges must use `SIFR-IMPORT-0007`.

`SIFR-WORKSPACE-0104` should be retired/aliased for this user-facing source cycle case. It may remain documented only as a legacy code or for a future non-source workspace graph cycle that cannot honestly point at an import statement.

Required behavior:

- primary span points at one import edge in the cycle
- related spans point at the remaining import edges when available
- JSON includes the cycle path and edge list as structured args
- human output explains how to break the cycle without hiding the source locations

## Scope

In scope:

1. Attach source spans to parser diagnostics using `ParseError.location`.
2. Add source-map rendering for parser diagnostics so human/compact/JSON all receive the span data.
3. Preserve unsupported syntax locations when available.
4. Replace workspace/package missing-module source diagnostics with canonical import diagnostics.
5. Add canonical import diagnostic identities for ambiguous module imports, namespace collisions, and import cycles.
6. Preserve import dependency source ranges during project and package discovery.
7. Preserve enough import-edge range data to render source-backed import cycle diagnostics.
8. Convert package diagnostics into `RenderedDiagnostic` without dropping `help`.
9. Use `PackageDiagnosticOrigin` to attach config-file or source/config context where practical.
10. Add verification fixtures for parser spans, missing imports, ambiguous imports, import cycles, and package diagnostic help preservation.
11. Update diagnostic docs for new import codes and retired/aliased workspace codes.
12. Keep `human`, `compact`, and `json` output contracts from the diagnostic presentation phase intact.

Out of scope:

- Redesigning diagnostic presentation.
- Changing the `RenderedDiagnostic` JSON schema.
- Adding color output.
- Reworking package dependency policy semantics.
- Making every package/Cargo metadata diagnostic source-spanned when no source or config range is available.
- Hiding legitimate package/workspace diagnostics behind source-level codes.

## Architecture Notes

### Parser source spans

`sifr_syntax` should convert parser errors through the canonical diagnostic builder/source-map path instead of constructing a spanless `RenderedDiagnostic` by hand.

The preferred integration contract is local to `sifr_syntax`: `parse_module_raw(source, context)` already receives both the source text and a display label. It should build a short-lived `sifr_diagnostics::SourceMap`, register `context.unwrap_or("main")` with `source`, and render the parser diagnostic into `RenderedDiagnostic` using `ParseError.location`. Do not thread a persistent `SourceId` across crate boundaries unless implementation proves the local source-map approach cannot preserve the existing JSON schema.

Implementation may factor this through a helper equivalent to the frontend `diagnostic_with_source_range(...)`: convert `ParseError.location: TextRange` plus `source` plus display path into a primary `DiagnosticSpan` with byte offsets, line/column positions, snippet lines, and highlight range. The phase must not hand-build partial spans that omit fields already required by the JSON schema.

If unsupported syntax diagnostics expose a range, use the same path. If a specific unsupported syntax diagnostic does not expose a range, keep it spanless only with a test documenting why no honest source range exists.

Important details:

- `ParseError.location` is a byte `TextRange`.
- Zero-length ranges at EOF are valid in the diagnostics source map.
- A zero-length `TextRange` at EOF should render as a zero-width caret at the file end.
- A zero-length range that points beyond the source text is invalid and should become an internal compiler diagnostic, not a misleading user-facing source span.
- If the parser reports a zero-length range at a valid but visually poor location, normalize only enough to produce a useful caret without changing the owning line or pretending a different token caused the error.
- Keep parser category args such as `parser_category`.
- Preserve context child notes such as `while parsing <module>`.

### Import dependency records

Project discovery should stop representing imports as bare module-name strings once diagnostics may need source context.

Use an internal record like:

```rust
struct ImportDependency {
    module_name: String,
    range: TextRange,
}
```

For diagnostics that need rendering, combine this with the already available parsed module source:

```rust
struct ImportDependencyContext {
    module_name: String,
    range: TextRange,
    display_path: String,
    source: String,
}
```

Package discovery needs a richer record because the written import path and the package-resolved import path can differ:

```rust
struct PackageImportDependencyContext {
    written_module_name: String,
    written_range: TextRange,
    display_path: String,
    source: String,
    resolved_package_import_path: DottedModulePath,
    origin: sifr_package::PackageImportOrigin,
}
```

The written fields drive the primary source span and human message. The resolved fields drive package/source-map resolution and JSON args so package-aware diagnostics can explain both what the user wrote and what package resolution attempted.

`sifr_package::PackageImportOrigin` already exists with `OwnPackage` and `DirectDependency { import_root, target_export_root, dependency_package_id }`. If implementation needs additional transient discovery-only state, add a driver-local wrapper around this existing enum rather than inventing an unrelated origin model.

Discovery duplicate prevention should be explicit: import dependency records must carry a resolution state such as `Resolved`, `FailedEmitted`, or `SkippedExternal`. Project/package frontend construction should include only resolved imports in the module graph and should not hand failed imports to HIR lowering as if they were unresolved ordinary symbols.

### Package diagnostic conversion

Add one shared package-diagnostic-to-rendered conversion path instead of separately dropping context in the CLI and driver.

The shared conversion should live below the CLI, preferably in `sifr_driver::diagnostics` or another driver-level utility that can depend on both `sifr_package` and `sifr_diagnostics`. The CLI and driver project-discovery paths should both call that single conversion function. Do not place this conversion in `sifr_diagnostics`, because `sifr_package` already depends on `sifr_diagnostics` and package-specific origin semantics do not belong in the generic renderer crate.

Proposed shape:

```rust
pub fn package_diagnostic_to_rendered(
    diagnostic: sifr_package::PackageDiagnostic,
) -> RenderedDiagnostic
```

The conversion should:

- preserve `help`
- include origin fields in JSON args when they are useful
- render manifest/config path and key information as child notes by default
- attach a manifest/config source span only when the converter has the file contents and can locate the key honestly
- preserve package-origin context such as Cargo package id, manifest path, source path, and key name in JSON args
- avoid pretending there is a Sifr source span for Cargo metadata-only diagnostics
- keep future structured package notes/candidates as `RenderedDiagnostic.children`

### Code aliases and compatibility

If `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, `SIFR-WORKSPACE-0103`, and `SIFR-WORKSPACE-0104` are retired for source-level import failures, keep docs explaining the migration and point users to the canonical `SIFR-IMPORT-*` codes.

Specific replacements:

- `SIFR-WORKSPACE-0101` source-level missing import -> `SIFR-IMPORT-0002`
- `SIFR-WORKSPACE-0102` source-level ambiguous import -> `SIFR-IMPORT-0005`
- `SIFR-WORKSPACE-0103` source-level namespace/file collision -> `SIFR-IMPORT-0006`
- `SIFR-WORKSPACE-0104` source-level import cycle -> `SIFR-IMPORT-0007`

Any existing tests or docs that assert old workspace codes for source-level imports must be updated intentionally.

## Milestones

### M1: Gap Contract Lock

- Add verification fixtures for the current gaps:
  - `crates/sifr/tests/verification/diagnostics/parser_bad_indent`
  - `crates/sifr/tests/verification/diagnostics/parser_unterminated_string`
  - `crates/sifr/tests/verification/diagnostics/parser_invalid_call_order`
  - `crates/sifr/tests/verification/diagnostics/parser_empty_declaration`
  - `crates/sifr/tests/verification/diagnostics/parser_invalid_declaration`
  - `crates/sifr/tests/verification/project/workspace_missing_import_canonical`
  - `crates/sifr/tests/verification/project/workspace_ambiguous_import_canonical`
  - `crates/sifr/tests/verification/project/workspace_namespace_collision_canonical`
  - `crates/sifr/tests/verification/project/import_cycle_source_spans`
  - `crates/sifr/tests/verification/package/package_missing_import_canonical`
  - `crates/sifr/tests/verification/package/package_ambiguous_import_canonical`
  - `crates/sifr/tests/verification/package/package_diagnostic_help_preserved`
- Extend `verification/tooling/check_diagnostic_presentation_rules.py` or add a sibling `check_diagnostic_source_canonicalization_rules.py`.
- The contract must initially fail against the current implementation.
- Lock expected target behavior for `human`, `compact`, and `json`.
- Record code migration decisions for `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, `SIFR-WORKSPACE-0103`, and `SIFR-WORKSPACE-0104`.
- Add registry/docs placeholders for `SIFR-IMPORT-0005`, `SIFR-IMPORT-0006`, and `SIFR-IMPORT-0007`; the contract should fail if the codes are undocumented or missing from the active registry.
- Require the contract checker to verify `SIFR-IMPORT-0005`, `SIFR-IMPORT-0006`, and `SIFR-IMPORT-0007` are active registry entries, not only mentioned in docs.
- Wire the source-canonicalization contract checker and its negative self-tests into `scripts/run_all_tests.sh --profile quick`.
- Add a phase-owned verification matrix that maps each discovered gap to:
  - source fixture
  - expected canonical code
  - required primary span behavior
  - required human/compact/JSON assertions
  - required unit or integration test owner

### M2: Parser Diagnostic Source Spans

- Thread `ParseError.location` and unsupported syntax ranges into parser diagnostics.
- Render parser diagnostics through the source-aware diagnostic path.
- Add unit tests proving every `SIFR-PARSE-*` category has either a primary span or an explicitly justified spanless fallback.
- Add parser-source-map tests proving byte offsets, UTF-8 line/column values, snippet text, and highlight ranges are populated for parser diagnostics.
- Add parser edge-case tests for zero-length EOF ranges, invalid out-of-bounds ranges, CRLF source text, non-ASCII text before the error location, and unsupported syntax diagnostics with and without ranges.
- Regenerate parser diagnostic baselines.
- Confirm zero-length and EOF parser ranges render cleanly in human and JSON modes.

### M3: Canonical Import Resolution Diagnostics

- Introduce import dependency records carrying module name and source range.
- Use canonical import diagnostics for missing module imports in project/package discovery.
- Add `SIFR-IMPORT-0005` for ambiguous module imports.
- Add `SIFR-IMPORT-0006` for namespace/file import collisions.
- Move attempted paths and candidate paths into notes/help/JSON args.
- Regenerate missing, ambiguous, and namespace-collision import verification baselines.
- Update diagnostic docs and tests that mention retired workspace import codes.
- Ensure missing-module failures discovered before HIR lowering are not also reported as missing members or undefined names.
- Add parity tests proving missing module imports use the same canonical code and source-span behavior in:
  - single-file mode
  - project/workspace mode
  - package mode
  - frontend/editor-style APIs
- Add parity tests proving ambiguous imports and namespace/file collisions use their canonical import codes and source-span behavior in every flow where those states can be constructed.
- For ambiguous imports and namespace/file collisions, single-file mode is exempt because it has no workspace/package discovery. Workspace/project mode and package mode are in scope across CLI paths and driver/frontend API paths that perform discovery.
- Add JSON assertions for attempted paths, candidate paths, resolution scope, written import path, resolved package import path, and package import origin.
- Add negative tests proving retired workspace import codes are not emitted for source-level missing, ambiguous, namespace-collision, or cycle imports.

### M4: Import Cycle Source Context

- Preserve dependency edge source ranges through graph construction.
- Choose and document one import-edge extraction approach before implementation: extend compile-order inputs to carry `ImportDependency` records, add a parallel range-aware compile-order path, or re-derive import ranges at cycle-diagnostic time with tests documenting any fallback limitations.
- Emit import cycle diagnostics as `SIFR-IMPORT-0007` with a primary import edge span.
- Add related spans for remaining cycle edges when available.
- Preserve cycle path and edge list in JSON args.
- Regenerate cycle verification baselines.
- Retire or alias `SIFR-WORKSPACE-0104` for source import cycles.
- Add tests for two-node and three-node cycles, ensuring human output renders a primary import edge, JSON contains related spans or structured edge context, and compact output has a concrete source location.

### M5: Package Diagnostic Context Preservation

- Add a shared package diagnostic renderer/converter.
- Preserve `PackageDiagnostic.help`.
- Preserve useful `PackageDiagnosticOrigin` data in JSON args and human notes/help.
- Attach manifest/config spans where practical; otherwise emit clear path/key notes.
- Regenerate package diagnostic baselines.
- Add conversion unit tests for every `PackageDiagnosticOrigin` variant:
  - `CargoMetadata`
  - `CargoManifest`
  - `SifrManifest`
  - `RustMarker`
  - `PackageGraph`
  - `CargoCommand`
- Add tests proving `help` survives CLI and driver conversion paths.
- Add tests proving diagnostics without honest source/config spans remain explicitly spanless but still preserve origin args and human notes/help.

### M6: Closeout

- Run:

```bash
python3 verification/tooling/check_diagnostic_presentation_rules.py
python3 verification/tooling/check_diagnostic_presentation_rules.py --self-test
python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py
python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py --self-test
cargo test -p sifr -- diagnostics
cargo test -p sifr_driver -- diagnostics
cargo test -p sifr_driver -- project
cargo test -p sifr_package
scripts/run_all_tests.sh --profile quick
```

- Re-run an ad hoc fuzz corpus equivalent to `tmp/diagnostic_gap_fuzz` and record whether each previously discovered gap is closed or intentionally deferred.
- Update `internal_docs/architecture.md` with the producer/presentation boundary:
  - producers own canonical identity, source spans, and structured context
  - `sifr_diagnostics` owns rendering once diagnostics are canonical `RenderedDiagnostic` values
- Update phase status and record validation evidence.

## Verification Contract

The phase is not done until a mechanical gate proves:

- Parser diagnostics no longer produce `<unknown>` locations when `ParseError.location` exists.
- Every active parser diagnostic category has fixture coverage.
- Missing module imports use canonical import codes in single-file, workspace, package, and editor-style flows.
- Missing module imports carry primary source spans in all supported flows.
- Ambiguous imports use `SIFR-IMPORT-0005` and carry source spans plus candidate context.
- Namespace/file collisions use `SIFR-IMPORT-0006` and carry source spans plus collision context.
- Import cycles use `SIFR-IMPORT-0007` and carry at least one source import span plus structured cycle context.
- Package diagnostic conversion preserves `help`.
- Package diagnostics preserve useful origin context in JSON args or human notes.
- Spanless diagnostics remain allowed only for diagnostics with no honest source/config location.
- `human`, `compact`, and `json` keep the presentation contracts from `ad-hoc-production-grade-diagnostic-presentation`.
- Retired workspace import codes are documented as aliases or legacy codes and are not emitted for source-level missing/ambiguous/colliding/cyclic imports.

## Verification Matrix

This phase must add explicit verification for the aspects found by fuzzing and code audit:

| Aspect | Required coverage |
| --- | --- |
| Parser span attachment | Unit tests for `ParseError.location` conversion; verification fixtures for bad indent, unterminated string, invalid call order, empty declaration, malformed declaration, invalid pattern, and unsupported syntax. |
| Parser span completeness | JSON assertions for `byte_start`, `byte_end`, `line`, `column`, `end_line`, `end_column`, `lines`, `highlight_start`, and `highlight_end`. |
| Parser edge cases | Tests for zero-length EOF ranges, invalid out-of-bounds ranges becoming internal diagnostics, CRLF source text, and non-ASCII text before the error location. |
| Missing module imports | Same canonical `SIFR-IMPORT-0002` and primary span in single-file, workspace/project, package, and editor-style/frontend flows. |
| Missing member imports | `SIFR-NAME-0004` remains distinct when the module resolves but the member is absent; no duplicate missing-module diagnostics. |
| Ambiguous imports | `SIFR-IMPORT-0005` with primary span, candidate path child notes, JSON candidate path args, no `SIFR-WORKSPACE-0102` emission for source-level imports, and flow parity wherever ambiguous imports can be constructed. |
| Namespace/file collisions | `SIFR-IMPORT-0006` with primary import span, collision path context, no `SIFR-WORKSPACE-0103` emission for source-level imports, and flow parity wherever collisions can be constructed. |
| Import cycles | `SIFR-IMPORT-0007` with primary import-edge span, related spans or structured edge context, cycle path JSON args, and no `SIFR-WORKSPACE-0104` emission for source-level cycles. |
| Package import context | JSON assertions for written module path, resolved package import path, `PackageImportOrigin`, dependency package id when applicable, and resolution scope. |
| Package diagnostic conversion | Unit tests for each `PackageDiagnosticOrigin` variant and integration tests proving CLI and driver conversion preserve `help`, origin args, notes, and intentionally spanless status where appropriate. |
| Legacy code migration | Registry/docs tests proving new codes are active registry entries and old workspace import codes are documented as legacy/aliases but not emitted by source-level fixtures. |
| Contract guardrail | A source-canonicalization contract checker wired into `scripts/run_all_tests.sh --profile quick`, including negative self-tests for missing fixtures, missing new codes, old-code leakage, missing spans, and dropped package help. |

## Phase-Owned Contract Matrix

M1 introduced `verification/tooling/check_diagnostic_source_canonicalization_rules.py`.
The checker owns the mechanical phase gate until the producer changes are complete.

| Gap | Fixture | Expected code | Primary span requirement | Human/compact/JSON assertions | Test owner |
| --- | --- | --- | --- | --- | --- |
| Parser bad indentation | `crates/sifr/tests/verification/diagnostics/parser_bad_indent/main.sifr` | `SIFR-PARSE-0002` | primary source span from `ParseError.location` | no `<unknown>`; human source arrow; compact `E code file:line:col`; JSON span completeness | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser unterminated string | `crates/sifr/tests/verification/diagnostics/parser_unterminated_string/main.sifr` | `SIFR-PARSE-0003` | primary lexical/string span | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser invalid call ordering | `crates/sifr/tests/verification/diagnostics/parser_invalid_call_order/main.sifr` | `SIFR-PARSE-0006` | primary invalid argument span | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser empty declaration list | `crates/sifr/tests/verification/diagnostics/parser_empty_declaration/main.sifr` | `SIFR-PARSE-0007` | primary declaration keyword/list span | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser malformed declaration list recovery | `crates/sifr/tests/verification/diagnostics/parser_invalid_declaration/main.sifr` | `SIFR-PARSE-0002` | primary parser-recovery span | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser invalid match pattern | `crates/sifr/tests/verification/diagnostics/parser_invalid_match_pattern/main.sifr` | `SIFR-PARSE-0008` | primary invalid pattern span | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Parser unsupported syntax | `crates/sifr/tests/verification/diagnostics/parser_unsupported_syntax/main.sifr` | `SIFR-PARSE-0009` | primary unsupported syntax span when Ruff supplies a range | same parser output contract | `check_diagnostic_source_canonicalization_rules.py`, M2 parser unit tests |
| Workspace missing import | `crates/sifr/tests/verification/project/workspace_missing_import_canonical/main.sifr` | `SIFR-IMPORT-0002` | primary span on written import target | no retired workspace code; JSON `resolution_scope` and `tried_paths` | `check_diagnostic_source_canonicalization_rules.py`, M3 integration tests |
| Workspace ambiguous import | `crates/sifr/tests/verification/project/workspace_ambiguous_import_canonical/main.sifr` | `SIFR-IMPORT-0005` | primary span on written import target | no `SIFR-WORKSPACE-0102`; JSON `candidate_paths` and `resolution_scope` | `check_diagnostic_source_canonicalization_rules.py`, M3 integration tests |
| Workspace namespace collision | `crates/sifr/tests/verification/project/workspace_namespace_collision_canonical/main.sifr` | `SIFR-IMPORT-0006` | primary span on written import target | no `SIFR-WORKSPACE-0103`; JSON `resolved_path` and `parent_path` | `check_diagnostic_source_canonicalization_rules.py`, M3 integration tests |
| Import cycle | `crates/sifr/tests/verification/project/import_cycle_source_spans/main.sifr` | `SIFR-IMPORT-0007` | primary span on one cycle-causing import edge | no `SIFR-WORKSPACE-0104`; JSON `cycle` and `cycle_edges` | `check_diagnostic_source_canonicalization_rules.py`, M4 graph tests |
| Package missing import | `crates/sifr/tests/verification/package/package_missing_import_canonical/src/main.sifr` | `SIFR-IMPORT-0002` | primary span on written import target | no retired workspace code; JSON written path and package origin context | `check_diagnostic_source_canonicalization_rules.py`, M3 package tests |
| Package ambiguous import | `crates/sifr/tests/verification/package/package_ambiguous_import_canonical/src_a/main.sifr` | deferred: package source-map duplicate modules are rejected as manifest/source-root config diagnostics before a source import can be ambiguous | n/a | static fixture retained to prevent silent scope loss; runtime package ambiguity remains a future package source-map design issue, not a source-import producer bug | `check_diagnostic_source_canonicalization_rules.py` static checks |
| Package help preservation | `crates/sifr/tests/verification/package/package_diagnostic_help_preserved/sifr.toml` | `SIFR-PACKAGE-0701` | spanless is allowed until manifest-key location is honest | JSON `help`, `origin_kind`, `manifest_path`, and `manifest_key` survive conversion | `check_diagnostic_source_canonicalization_rules.py`, M5 conversion tests |

M1 validation status:

- `SIFR-IMPORT-0005`, `SIFR-IMPORT-0006`, and `SIFR-IMPORT-0007` are active registry entries with generated docs.
- `SIFR-WORKSPACE-0101` through `SIFR-WORKSPACE-0104` generated docs now state the legacy source-import replacement codes.
- The contract checker and self-test are wired into `scripts/run_all_tests.sh --profile quick`.
- The runtime contract is expected to fail until M2-M5 replace the upstream producers.

M2-M5 implementation status:

- Parser diagnostics now render through the source-map path using Ruff parse-error ranges and unsupported-syntax ranges. Unit tests cover UTF-8 columns, CRLF snippets, zero-length EOF ranges, invalid ranges, and unsupported syntax spans.
- Project discovery now preserves import dependency ranges and emits canonical source diagnostics for missing imports, ambiguous workspace imports, and namespace/file collisions. Resolver details are retained in structured JSON args and child notes.
- Package discovery now emits `SIFR-IMPORT-0002` for own-package missing source imports with the written import span and package-origin context. Undeclared external/transitive package imports remain package policy diagnostics.
- Import-cycle ordering now has a source-aware path that emits `SIFR-IMPORT-0007` with a primary import-edge span, related edge spans, and `cycle`/`cycle_edges` args.
- Package diagnostic conversion now goes through `sifr_driver::diagnostics::render_package_diagnostic`, preserving `PackageDiagnostic.help` and useful `PackageDiagnosticOrigin` args for CLI and driver paths.
- `python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py` and `python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py --self-test` pass locally on 2026-05-29.

M6 closeout status:

- Merged implementation PR: https://github.com/sifr-lang/sifr/pull/2197 (`71730845b5e0f86a91360fa368257e1881e277fb`).
- `cargo fmt --check` passed locally on 2026-05-29.
- `python3 verification/tooling/check_diagnostic_presentation_rules.py` and `python3 verification/tooling/check_diagnostic_presentation_rules.py --self-test` passed locally on 2026-05-29.
- `python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py` and `python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py --self-test` passed locally on 2026-05-29.
- `bash scripts/run_validation_contract_matrix.sh --suite phase23_graph_isolation` passed locally on 2026-05-29 after updating the cycle stability contract to the canonical `SIFR-IMPORT-0007` message.
- `cargo test -p sifr -- diagnostics`, `cargo test -p sifr_driver -- diagnostics project`, `cargo test -p sifr_driver --lib`, `cargo test -p sifr_package`, and `cargo test -p sifr_syntax` passed locally on 2026-05-29.
- `scripts/run_all_tests.sh --profile quick` passed locally on 2026-05-29. The final validation lane report recorded `wall_time=624.67s` and advisories for warm wall-time budget and e2e group skew; these are performance advisories, not validation failures.
- Reviewer passes:
  - `reviews/ad-hoc-diagnostic-source-canonicalization-m1-review-pass-1.md`: satisfied.
  - `reviews/ad-hoc-diagnostic-source-canonicalization-m2-review-pass-1.md`: satisfied.
  - `reviews/ad-hoc-diagnostic-source-canonicalization-full-review-pass-1.md`: satisfied with non-blocking file-size/per-cycle coverage notes.
  - `reviews/ad-hoc-diagnostic-source-canonicalization-full-review-pass-2.md`: satisfied after the package-discovery split, with stale test assertions later fixed.
  - `reviews/ad-hoc-diagnostic-source-canonicalization-full-review-pass-3.md`: not satisfied because the validation contract still expected the old cycle message; fixed in `verification/validation_contracts/manifest.json` and revalidated.
  - `reviews/ad-hoc-diagnostic-source-canonicalization-full-review-pass-4.md`: satisfied with no blocking findings and no remaining validation required.

## Acceptance Criteria

- A developer seeing a bad import in a workspace/package gets the same diagnostic code family they would get in single-file mode.
- A developer seeing a parser error gets the exact source line and caret in human mode.
- An agent using compact mode can identify file, line, and diagnostic code for parser and import-resolution failures.
- An editor using JSON can map parser, import-resolution, and import-cycle diagnostics to source ranges.
- Package diagnostics no longer lose help text during conversion.
- Legacy workspace import codes are either retired with docs or reserved only for non-source workspace resolution failures.
- The phase can be implemented as small PRs in milestone order without needing to reopen diagnostic presentation architecture.

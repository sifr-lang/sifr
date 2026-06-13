# Phase Review: Diagnostic Source Canonicalization — Pass 2

**Overall verdict: Nearly implementation-ready. One blocker and two residuals remain. The three Severity 1 blockers from pass 1 are all resolved, but a new blocker surfaces on closer inspection of the struct definitions.**

---

## Severity 1 — Must Fix

### 1. `PackageImportDependencyContext.origin` references a non-existent type

**Phase file:** lines 344–353

The proposed `PackageImportDependencyContext` struct ends with:

```rust
origin: PackageImportOrigin,
```

This type does not exist in the codebase. The actual package import origin enum in `sifr_package/src/diag/mod.rs` is `PackageDiagnosticOrigin`, which has variants for `CargoMetadata`, `CargoManifest`, `SifrManifest`, `RustMarker`, `PackageGraph`, and `CargoCommand`.

`PackageImportOrigin` appears nowhere in the crate graph. Either:
- It is a new type the phase intends to introduce (in which case the enum variants need to be designed), or
- It should be `PackageDiagnosticOrigin` (in which case the phase is internally inconsistent — it uses `PackageDiagnosticOrigin` correctly in the package diagnostic conversion section but `PackageImportOrigin` in the import dependency context struct)

**What must change:** Name the type correctly. If the phase needs a separate `PackageImportOrigin` for import-specific resolution context, define its variants and place it in `sifr_driver::project` alongside `ImportDependency`. If it should reuse `PackageDiagnosticOrigin`, align the struct field name with the existing type. Do not leave an unresolved type name that has no definition.

---

## Severity 2 — Should Fix

### 2. `SourceMap` in the parser integration contract does not exist in `sifr_syntax`

**Phase file:** lines 304–305

The architecture note says:

> It should build a short-lived `SourceMap`, register `context.unwrap_or("main")` with `source`, and render the parser diagnostic into `RenderedDiagnostic`.

The codebase at `crates/sifr_syntax/src/lib.rs` uses Ruff (via `sifr_python_parser`/`sifr_python_ast`). Ruff's `Parsed` type does not expose a `SourceMap` type — it exposes source ranges via `ParseError.location` as `TextRange` (byte offsets).

The local source-map approach is the right architectural direction (good call in the update), but the contract references a type that does not exist. Implementation will need to either:
- Construct `DiagnosticSpan` values directly from `ParseError.location` + source text without a `SourceMap` abstraction, or
- Use the existing Ruff-based source-map machinery if it exists under a different name

**What should change:** Replace the `SourceMap` reference with the concrete types available in `sifr_syntax`/`sifr_python_parser`. The contract should specify how `ParseError.location: TextRange` becomes `RenderedDiagnostic.spans: Vec<DiagnosticSpan>` — e.g., "thread source text into `parse_error_diagnostic` so it can compute line/column offsets from the byte range."

### 3. Discovery duplicate-prevention mechanism is undefined

**Phase file:** lines 169–170

> Discovery must prevent duplicate reporting: if module resolution fails before HIR lowering, the missing-module import diagnostic is emitted once and HIR must not re-report the same source import as a missing member or generic name failure.

This is a correct invariant but the phase does not specify the mechanism. The current discovery/dispatch architecture (see `crates/sifr_driver/src/project/discovery.rs` and `crates/sifr_frontend/src/graph_cache_and_queries.rs`) uses `diagnostic_with_code` at each phase. Without an explicit gating mechanism, a module resolution failure before HIR lowering could still appear as an undefined-name or missing-member error in HIR.

**What should change:** Either add a brief architecture note ("use a sentinel bit on the import record to mark it resolved; HIR skips import resolution for already-failed imports") or add it to M3's implementation task list.

---

## Severity 3 — Minor Residuals

### 4. `SIFR-WORKSPACE-0103` lacks explicit retirement/alias decision

**Phase file:** lines 263–265, 388–389

The Code aliases section (lines 381–386) explicitly names the retirement/alias decision for `SIFR-WORKSPACE-0101`, `0102`, and `0104`. `SIFR-WORKSPACE-0103` (namespace collision) appears once in the Target Behavior section ("Namespace/file collisions must use `SIFR-IMPORT-0006`") but has no explicit retirement callout.

This is a gap relative to the pass 1 finding (#7), which specifically called out that `SIFR-WORKSPACE-0103` was missing from the migration decisions.

**Suggested fix:** Add one sentence in the Code aliases section: "`SIFR-WORKSPACE-0103` is retired for source-level namespace/package collisions and replaced by `SIFR-IMPORT-0006`."

### 5. "Parser invalid declaration" is listed in M1 fixture paths but not in Current Findings

**Phase file:** lines 393, 31–46

M1 lists `parser_invalid_declaration` as a required verification fixture, but the Current Findings section lists only: bad indentation, unexpected indentation, unterminated strings, malformed function signatures, empty declaration lists, invalid call argument order, invalid match patterns, unsupported syntax.

"Invalid declaration lists" maps to `parser_invalid_declaration`, but "empty declaration lists" (a distinct category) is not in M1's fixture list.

**Suggested fix:** Align the fixture list in M1 with the Current Findings categories, or add `parser_empty_declaration` if it is genuinely a separate case from `parser_invalid_declaration`.

---

## What's Fixed From Pass 1

- **SIFR-IMPORT-0005/0006/0007 decision**: Both the Product Decision (lines 138–145) and Target Behavior sections now explicitly name these codes. ✓
- **Parser source-id threading contract**: The "Parser source spans" section (lines 300–317) now defines a local source-map approach that does not require threading `SourceId` across crates. ✓
- **Package diagnostic conversion shared path**: The "Package diagnostic conversion" section (lines 357–380) now specifies placement (`sifr_driver::diagnostics`), behavior (preserve `help`, render manifest/config context), and constraints (don't pretend source spans for Cargo-only diagnostics). ✓
- **`ImportDependency` package context**: `PackageImportDependencyContext` (lines 344–353) now models both written and resolved import paths, addressing finding #1 from pass 1. ✓
- **Zero-length range normalization boundary**: Lines 308–315 define clear rules for EOF, out-of-bounds, and visually poor zero-length ranges. ✓

---

## Summary

| Finding | Severity | Status |
|---|---|---|
| `PackageImportDependencyContext.origin` type doesn't exist | 1 | New blocker — must fix |
| `SourceMap` type doesn't exist in `sifr_syntax` | 2 | Should fix before M2 |
| Discovery duplicate-prevention mechanism undefined | 2 | Should fix in M3 |
| `SIFR-WORKSPACE-0103` missing explicit retirement | 3 | Minor — one sentence fix |
| Fixture list vs. Current Findings mismatch | 3 | Minor — align categories |

**The phase is close to implementation-ready.** Fix finding #1 (Severity 1) and the phase is ready to implement. Findings #2–3 are refinements worth doing before the relevant milestones, not blockers to unlocking the phase. Finding #4–5 are trivial cleanup.

If the `origin` field is simply a naming inconsistency (should be `PackageDiagnosticOrigin`) rather than a new type, the fix is a one-line rename. If it is truly meant to be a new `PackageImportOrigin` enum, the phase needs one additional paragraph defining its variants.
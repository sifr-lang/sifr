

Based on my thorough analysis of the phase file, the diagnostic gap fuzz audit, and the codebase, here is my review:

---

## Phase Review: Diagnostic Source Canonicalization

**Overall verdict: Close but not implementation-ready. Two structural gaps must be resolved before this becomes a clean, reviewable phase plan.**

---

### Severity 1 — Must Fix

#### 1. `ImportDependency` is under-specified for package context

**File reference:** `crates/sifr_driver/src/project/discovery.rs:451–545`

The proposed `ImportDependency` and `ImportDependencyContext` (phase file lines 294–315) model the written import path and its source range, but they don't model the *resolved* package import path. Package discovery normalizes imports relative to packages via `PackageImportOrigin` — the same written import may resolve to different compile-time module names depending on which package root is used.

**What must change:** Add a `PackageImportDependencyContext` that carries:
- Written import path (`module_name`)
- Source range of the import statement
- Resolved package import path after package-relative normalization
- The `PackageImportOrigin` variant that determined resolution

Without this, the package import closure discovery cannot render canonical diagnostics with both the written source location and the resolved target.

#### 2. Ambiguous import canonical code is undefined

**Files:** `crates/sifr_diagnostics/src/codes/registry.rs:165–167`

The plan says "add or reuse canonical import diagnostic identity for ambiguous module imports" (line 258) but does not name a code. There is no `SIFR-IMPORT-0005` or similar in the registry. The existing `SIFR-WORKSPACE-0102` and `SIFR-WORKSPACE-0103` (namespace collision) would both need replacement, but the plan doesn't define target codes.

**What must change:** Add a concrete code proposal — e.g., `SIFR-IMPORT-0005` for ambiguous imports, `SIFR-IMPORT-0006` for namespace collision. The namespace collision case is a source-level import problem (the import statement is valid but the filesystem layout creates ambiguity), so it legitimately belongs in the IMPORT family. Update the registry entry proposals accordingly.

#### 3. Parser source-id threading is unaddressed

**File:** `crates/sifr_syntax/src/lib.rs:178–207`

The `parse_module_raw` function receives `source: &str` with no `SourceId`. When parser diagnostics are converted to source-mapped `RenderedDiagnostic` values, the `DiagnosticSpan` requires a `file: Option<String>`. Currently `sifr_syntax` returns bare `RenderedDiagnostic` values without source mapping — the source-id attachment happens downstream in `sifr_frontend::parse_source`.

The plan's M2 milestone (line 350) says "Thread `ParseError.location` and unsupported syntax ranges into parser diagnostics" but does not specify where `SourceId` comes from or how it is communicated to `sifr_syntax`.

**What must change:** Define the integration contract between `sifr_syntax` (which owns `ParseError.location`) and `sifr_frontend`/`sifr_driver` (which own `SourceId`). Either:
- `sifr_syntax` is enhanced to accept and thread `SourceId`, or
- `sifr_syntax` returns raw span data and the caller is responsible for source-map attachment

The current pattern (line 423 in `discovery.rs`: `sifr_frontend::parse_source(&source, Some(&label))?`) shows the caller's label is used as the display path but not as a source-map key. This must be clarified.

---

### Severity 2 — Should Fix

#### 4. Package diagnostic conversion needs a single shared path with explicit behavior

**Files:** `crates/sifr/src/cli_model_and_entrypoint.rs:573–577`, `crates/sifr_driver/src/project/discovery.rs:733–735`

The plan identifies that both locations drop `help` and `origin`, and says "Add one shared package-diagnostic-to-rendered conversion path" (lines 316–326). The fix is correct, but the plan doesn't specify:

- Whether the shared path lives in `sifr_diagnostics` (new public constructor) or in `sifr_driver` (internal utility)
- How `children` are preserved (the current `diagnostic_with_code` drops them, but `PackageDiagnostic` could carry structured notes)
- What happens when `PackageDiagnosticOrigin` has a path/key — does it become a span, a related span, or a child note?

**Suggested fix:** Add a section under "Architecture Notes → Package diagnostic conversion" that specifies:
```
// Proposed shared conversion signature:
pub fn package_diagnostic_to_rendered(diagnostic: PackageDiagnostic) -> RenderedDiagnostic {
    // - help → RenderedDiagnostic.help
    // - origin.path → DiagnosticSpan.file (is_primary=true)
    // - origin.key → child note or label
    // - children (tried paths, candidates) → RenderedDiagnosticChild::Note
}
```

#### 5. Import cycle code family is ambiguous

**File:** `crates/sifr_driver/src/project/compile_order.rs:216–219`

The plan says import cycles "may remain `SIFR-WORKSPACE-0104` because the problem is graph-level" (line 241), but also requires "primary span points at one import edge in the cycle" (line 244–248). If a source span is available, this is a source-level problem and should arguably use `SIFR-IMPORT-*`.

**What must change:** Either:
- Justify why cycles stay in WORKSPACE family even with source spans (e.g., because cycles can be detected without parsing all modules), or
- Propose `SIFR-IMPORT-0007` for source-spanned import cycles and retire `SIFR-WORKSPACE-0104`

The current code (`compile_order.rs:216`) only stores module names in the dependency graph — source ranges are not preserved. M4 (line 367) changes this, but the code family decision should come first.

#### 6. Missing import member vs. missing module — taxonomy gap

**Registry references:** `crates/sifr_diagnostics/src/codes/registry.rs:24` (`NAME_MISSING_MODULE_MEMBER` → `SIFR-NAME-0004`), line 29 (`IMPORT_UNKNOWN_SOURCE_MODULE` → `SIFR-IMPORT-0002`)

The plan says "Missing imported member: `SIFR-NAME-0004`" (line 141) as a source-level problem, but it doesn't distinguish between "the import statement cannot resolve to a module file" (`SIFR-IMPORT-0002`) and "the module resolves but the member is missing" (`SIFR-NAME-0004`).

If a missing module import is being handled by M3 (canonical import diagnostics), but the HIR lowering also encounters the same unresolved import, the same source problem could produce two different diagnostic codes.

**What must change:** Clarify that `SIFR-IMPORT-0002` handles the resolution failure (no such module) and `SIFR-NAME-0004` handles the member failure (module found, member absent). Add a note that these are sequential stages and the HIR lowering should not re-report the import resolution failure if the module was not found.

---

### Severity 3 — Minor

#### 7. Namespace collision (`SIFR-WORKSPACE-0103`) is not mentioned in migration decisions

**File:** `crates/sifr_driver/src/project/discovery.rs:150–164`

The fuzz audit (line 22 of `tmp/diagnostic_gap_fuzz/README.md`) identifies namespace collision as a source-span gap. The phase file's code migration section (lines 328–331) only mentions `SIFR-WORKSPACE-0101` and `SIFR-WORKSPACE-0102`. `SIFR-WORKSPACE-0103` is missing.

**What must change:** Add `SIFR-WORKSPACE-0103` to the retirement/alias list. This diagnostic points at a source import, so it should be replaced with an `SIFR-IMPORT-*` code (proposed in finding #2 above).

#### 8. Verification fixtures list is incomplete

**File reference:** `verification/tooling/check_diagnostic_presentation_rules.py:28–32`

The plan's M1 (lines 335–348) says "Add verification fixtures for parser spans, missing imports, ambiguous imports, import cycles, and package diagnostic help preservation" but does not list the specific fixture directories that must exist.

The fuzz audit generated `plain/`, `workspaces/`, `package_imports/`, and `cycle_cases/` corpus. Not all of these need baselines in the contract checker — some are fuzz findings, not verification targets.

**What must change:** Add a table or list in M1 that names each required verification fixture and its expected diagnostic formats. Example:
```
verification/fixtures/
  parser_bad_indent/       → human, compact, json
  parser_unterminated_string/ → human, compact, json
  workspace_missing_import/ → human, compact, json
  package_help_preserved/   → human, json
```

#### 9. Zero-length range at EOF: normalization boundary is unclear

**File:** `crates/sifr_syntax/src/lib.rs:224–284`

The plan says "If the parser reports a zero-length range at a poor visual location, normalize only enough to produce a useful caret without inventing misleading source ownership" (lines 284–288). This is good guidance but doesn't specify when normalization is forbidden.

For example, a zero-length range at column 0 of an empty file should produce `0:0` caret pointing at the start. But a zero-length range at column 100 on a 10-character file should probably be clamped to a single-character caret, not wrapped to the next line.

**What must change:** Add one concrete example showing the normalization boundary: "A zero-length `TextRange` at EOF that points past the last character is valid as a zero-width caret at the file end. A zero-length range that points to a position not covered by the source text should be rejected (internal compiler error, not a user-facing diagnostic)."

---

### Summary

| Finding | Severity | Location in phase |
|---|---|---|
| ImportDependency under-specified for packages | 1 | lines 294–315 |
| Ambiguous import canonical code undefined | 1 | lines 138–141 |
| Parser source-id threading unaddressed | 1 | lines 278–288 |
| Package diagnostic conversion lacks shared path spec | 2 | lines 316–326 |
| Import cycle code family ambiguous | 2 | lines 241–248 |
| Missing import member vs. module taxonomy gap | 2 | lines 140–141 |
| SIFR-WORKSPACE-0103 missing from migration | 3 | lines 328–331 |
| Verification fixtures not enumerated | 3 | lines 335–348 |
| Zero-length range normalization boundary unclear | 3 | lines 284–288 |

**The phase is not implementation-ready.** Fix findings 1–3 first (Severity 1), then refine 4–6 (Severity 2). Findings 7–9 can be addressed during implementation or as part of the closeout milestone.

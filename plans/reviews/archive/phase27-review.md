# Phase 27 Review: Runtime Safety and Diagnostics Contract

## Overview

Phase 27 (merged March 7, 2026) addresses runtime safety and diagnostics, split across two phase files:

| Milestone | PR | Focus |
|-----------|-----|-------|
| 27.1 | #897 | Remove data-dependent unwrap/expect |
| 27.2 | #898 | Indexing and semantics parity fixes |
| 27.3 | #899 | Defaults and panic-to-diagnostic conversion |
| 27.4 | #900 | Span and diagnostic schema quality |
| 27.5 | #901 | Bounded multi-error recovery |
| 27.6 | #902 | Stability contract finalization |

All tests pass and demos execute correctly per the execution checklist.

---

## Findings by Milestone

### 27.1: Remove Data-Dependent unwrap/expect

**Implementation Location**: `crates/sifr_codegen/src/lower_expr.rs`, `crates/sifr_codegen/src/stmt_support_emitter.rs`

**Correctly Implemented:**
- Generated code no longer contains data-dependent `.unwrap()` or `.expect()` on index paths and optional-len lowering
- Positive validation: emitted Rust code contains no `.unwrap()` or `.expect()` calls
- Negative validation: unsafe optional method usage produces proper type error (`type 'None | list[int]' has no method 'len'`)

**Potential Concerns:**
1. **Test coverage scope** - Only basic optional-len and index paths tested; complex nested scenarios may have gaps
2. **Generated code inspection** - Validation relies on grep pattern matching; may miss edge cases in generated code patterns

---

### 27.2: Indexing and Semantics Parity Fixes

**Implementation Location**: `crates/sifr_codegen`

**Correctly Implemented:**
- Negative indexing (`[-1]`, `[-2]`) parity fixed across:
  - List read operations
  - List mutation (`assignment`, `augassign`)
  - List delete operations
  - Nested mutation paths
- Positive validation: e2e tests `negative_index_list.sifr`, `negative_index_string.sifr`, `negative_index_mutations.sifr` all pass
- Negative validation: invalid index types properly rejected

**Potential Concerns:**
1. **String negative indexing** - Codegen exists but need verification against Python semantics (slicing behavior)
2. **Complex nested negative index scenarios** - Not explicitly tested (e.g., `a[b[-1]]`)

---

### 27.3: Defaults and Panic-to-Diagnostic Conversion

**Implementation Location**: `crates/sifr_hir/src/lower/`, `crates/sifr_driver/src/lib.rs`, `crates/sifr/src/main.rs`

**Correctly Implemented:**
- Non-literal default arguments (collection literals) preserved correctly
- Unsupported default expressions produce deterministic diagnostics
- Driver codegen panic boundary converts panics into `CompilePhase::Codegen` diagnostics
- Positive: `non_literal_default_args.sifr` passes, demo runs correctly
- Negative: unsupported default call expressions produce `type error: function 'pick': unsupported default argument expression for parameter 'x'`

**Potential Concerns:**
1. **Collection literal sharing** - Default mutable arguments could share state across calls in Python; need verification this is handled
2. **Panic boundary scope** - Only codegen and CLI entrypoints have panic boundaries; internal phases may still panic (tracked in panic inventory)

---

### 27.4: Span and Diagnostic Schema Quality

**Implementation Location**: `crates/sifr_driver/src/lib.rs`, `crates/sifr/src/main.rs`

**Correctly Implemented:**
- **Severity enum** exactly as specified (`Error | Warning | Note | Help`) at `sifr_driver/src/lib.rs:591-596`
- **Suggestion kinds** exactly as specified (`DidYouMean | ReplaceText | InsertText | DeleteText`) at `sifr_driver/src/lib.rs:599-604`
- **Canonical diagnostic schema** with all required fields at `sifr_driver/src/lib.rs:633-644`:
  - `code`: String
  - `severity`: Severity
  - `message`: String
  - `url`: String
  - `primary_span`: Option<DiagnosticSpan>
  - `related_spans`: Vec<RelatedSpan>
  - `children`: Vec<DiagnosticChild>
  - `help`: Option<String>
  - `suggestions`: Vec<DiagnosticSuggestion>
- Stable `human` and `json` renderers; `json` is lossless canonical schema rendering
- Every diagnostic includes `url = "https://sifr.sh/docs/errors/<CODE>"`

**Potential Concerns:**
1. **Span precision** - Primary spans exist but may not always be accurate for all error types (parser vs type-check vs codegen)
2. **Related spans** - Not all diagnostics populate related spans; may need expansion
3. **Help text** - Not consistently populated across all error types

---

### 27.5: Bounded Multi-Error Recovery

**Implementation Location**: `crates/sifr_driver/src/lib.rs:680-735`

**Correctly Implemented:**
- **Hard limits enforced**:
  - `MAX_TOP_LEVEL_DIAGNOSTICS = 50` (line 680)
  - `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` (line 681)
- **Deterministic ordering** via BTreeMap keys
- **Duplicate summarization** with exact suffix `... +N more similar diagnostics`
- Tests verify recovery limits work correctly

**Potential Concerns:**
1. **Priority ordering** - Currently alphabetical by severity rank; may not match user expectations for error prioritization
2. **Performance at limit** - 50 diagnostics may still be heavy for display; no truncation beyond this limit

---

### 27.6: Stability Contract Finalization

**Implementation Location**: `crates/sifr/src/main.rs:83-86`, `crates/sifr/src/main.rs:71-75`, `crates/sifr/src/main.rs:294-350`

**Correctly Implemented:**
- **Exit code contract** exactly as specified:
  - `EXIT_SUCCESS = 0` (line 83)
  - `EXIT_USER_DIAGNOSTIC = 1` (line 84)
  - `EXIT_USAGE_OR_CONFIG = 2` (line 85)
  - `EXIT_INTERNAL_COMPILER_FAILURE = 3` (line 86)
- **CLI format contract** implemented: `--diagnostic-format human|json|compact` (lines 71-75)
- **Unknown format fails fast** with exit code 2 before semantic work (line 160)
- **Compact renderer invariants**:
  - First line severity summary
  - Grouping by `(severity, code, canonical message)`
  - Max 5 representative locations per group
  - `... +N more` truncation
  - One help line and one URL line per group

**Potential Concerns:**
1. **Compact renderer snapshot stability** - No snapshot tests yet; may drift without regression detection
2. **Warning handling** - Warnings produce exit code 0;是否符合预期需要确认

---

## Panic Inventory and Follow-ups

The panic inventory (`issues/phase27-panic-inventory.md`) documents three categories of remaining panics:

| Category | Location | Status |
|----------|----------|--------|
| Codegen invariant panics | `sifr_codegen/src/lib.rs`, `function_emitter.rs`, etc. | Converted via boundary, follow-up in Phase 34 |
| HIR/CFG invariant panics | `sifr_hir/src/cfg.rs` | Converted via boundary, follow-up in Phase 29 |
| Parser invariant unwraps | `sifr_python_parser/src/lexer.rs`, `string.rs` | Converted via boundary, follow-up in Phase 29 |

**All user-facing CLI paths are panic-free** - panics are caught at boundaries and converted to diagnostics with exit code 3.

---

## Phase 36 Alignment Analysis

Phase 36 expects:

### 36.1: Shared Frontend API Contract
- **Current state**: `sifr_driver` exposes public API:
  - `compile(source: &str) -> CompileResult`
  - `check(source: &str) -> Vec<CompileError>`
  - `build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>>`
  - `check_project(main_file: &Path) -> Vec<CompileError>`
  - `build_project(main_file: &Path, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>>`
- **Gap**: No explicit "frontend API" for parse/lower/type-check phases separately; only full compile/check/build entrypoints
- **Risk**: Medium - Phase 36 requires granular access (parse, lower, type-check separately) for tooling parity

### 36.2: Tooling/CLI Parity Matrix
- **Current state**: No tooling exists (no `sifr_lsp`, `sifr_lint`, editor adapters)
- **Gap**: Cannot validate parity until tooling is built
- **Risk**: Low - infrastructure in place for future validation

### 36.3: Thin Adapter and Renderer Boundaries
- **Current state**: Renderers (`human`, `json`, `compact`) are renderer-only over canonical diagnostics
- **Positive**: "generation vs rendering separation" correctly implemented
- **Gap**: No non-CLI proof adapter exists yet
- **Risk**: Low - Phase 36 will build this adapter

---

## Summary of Risks

### High Priority

| Area | Issue | Impact |
|------|-------|--------|
| 36.1 | No granular parse/lower/type-check API | Phase 36 tooling cannot access intermediate phases |
| 36.1 | No shared project/context handle | Multiple entrypoints, not unified API |

### Medium Priority

| Area | Issue | Impact |
|------|-------|--------|
| 27.4 | Related spans not consistently populated | Reduced diagnostic quality for complex errors |
| 27.4 | Help text not consistently populated | Reduced actionable guidance |
| 27.2 | Complex nested negative indexing | Potential edge case gaps |
| 27.3 | Collection literal default sharing | Potential semantic gap with Python behavior |

### Low Priority

| Area | Issue | Impact |
|------|-------|--------|
| 27.6 | No compact renderer snapshot tests | Potential drift without detection |
| 27.5 | Priority ordering is alphabetical | May not match user expectations |

---

## Required Fixes for Phase 36 Alignment

### Fix 1: Granular Frontend API (Required for 36.1)

**Location**: `crates/sifr_driver/src/lib.rs`

**Required addition**:
```rust
// Add to sifr_driver public API
pub fn parse(source: &str) -> Result<PythonAst, Vec<CompileError>>;
pub fn lower(ast: &PythonAst) -> Result<Hir, Vec<CompileError>>;
pub fn type_check(hir: &Hir) -> Result<TypeCheckResult, Vec<CompileError>>;
pub fn collect_diagnostics() -> Vec<CompilerDiagnostic>;
pub struct ProjectContext { ... }
pub fn create_project_context(files: &[(PathBuf, &str)]) -> ProjectContext;
```

### Fix 2: Unified Project Handle (Required for 36.1)

**Location**: `crates/sifr_driver/src/lib.rs`

**Required addition**:
```rust
pub struct ProjectHandle { ... }  // Single unified context handle
impl ProjectHandle {
    pub fn parse_file(&self, path: &Path) -> Result<PythonAst, Vec<CompileError>>;
    pub fn lower_file(&self, path: &Path) -> Result<Hir, Vec<CompileError>>;
    pub fn check_project(&self) -> Vec<CompileError>;
    pub fn get_module_graph(&self) -> ModuleGraph;
}
```

---

## Recommendations

1. **Before Phase 36**: Add granular parse/lower/type-check API to `sifr_driver`
2. **Add snapshot tests** for compact renderer to prevent drift
3. **Expand related spans** population for better multi-file error messages
4. **Expand help text** coverage for all error codes
5. **Document warning handling** - confirm exit code 0 for warnings-only is intentional

---

## Conclusion

Phase 27 successfully implements:
- Runtime-safe codegen (no data-dependent unwraps)
- Proper negative indexing semantics
- Panic-to-diagnostic conversion with boundaries
- Canonical structured diagnostic schema
- Bounded multi-error recovery
- Stable exit code and CLI contracts

**All tests pass and the phase is complete.**

The main gap for Phase 36 alignment is the lack of granular frontend API (parse/lower/type-check as separate entrypoints). This should be addressed before Phase 36 begins.

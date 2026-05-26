
Based on my audit of the Ruff LSP source and Sifr's current implementation, here are my findings ordered by severity:

---

## CRITICAL — Must Fix Before LSP Ships

### 1. No Diagnostic Classification System
**Severity: Critical**

Ruff stores `DiagnosticKind` in the `data` field of every LSP diagnostic. Sifr has **no equivalent** — diagnostics carry no type tag distinguishing hard vs. policy diagnostics.

- `diagnostic.rs:31-40` shows Ruff's `AssociatedDiagnosticData` stores `kind`, `code`, `edits`, and `noqa_edit`
- `lint.rs:251` shows data is only included when `fix.is_some() || noqa_edit.is_some()`

**Sifr must add:**
```rust
enum DiagnosticClass {
    Hard,  // Compiler errors — no suppression, no fix code actions
    Policy, // sifr_lint diagnostics — can suppress, can offer suppression edit
}
```

This classification gates every downstream decision: whether to offer a suppression code action, whether to include in fix_all, etc.

### 2. Fix-All Includes Hard Diagnostics
**Severity: Critical**

`fix.rs:69-81` shows Ruff's `fix_all` calls `ruff_linter::linter::lint_fix` which applies all safe fixes. Sifr's `implementation.rs` has no `fix_all` method — good — but the architecture is undefined.

**Sifr must:**
- Define `fix_all` to only process policy diagnostics
- Never auto-fix hard diagnostics (type errors, undefined names)
- Consider whether fix-all is even appropriate for a language where "if it compiles, it works"

### 3. Suppression Code Actions Not Policy-Scoped
**Severity: High**

`implementation.rs:373-390` shows suppression code actions are gated by `diagnostic.0.starts_with("SIFR-LINT-")` — but this is fragile. A malformed diagnostic code would bypass the check.

**Must enforce at data layer, not string matching.**

---

## HIGH — Significant Architectural Gaps

### 4. No Code Action Resolution Pattern
**Severity: High**

Ruff uses deferred resolution (`code_action_resolve.rs`):
1. Initial request returns action with `data: Some(document_url)` but no `edit`
2. Editor later calls `codeAction/resolve` with that data
3. Server computes the edit and returns it

This is essential for fix-all/organize-imports (potentially slow) and for edit conflict detection. Sifr's `resolve()` in `code_action.rs:48-55` is a stub that just returns params.

### 5. No WorkspaceEditTracker
**Severity: High**

Ruff has a sophisticated `WorkspaceEditTracker` for:
- Version-aware edits (detect conflicts)
- Document-level edit batching
- Capability-aware edit construction

Sifr has none of this — `WorkspaceEdit` is just `Vec<FileTextEdits>`.

### 6. Client Settings Architecture Is Basic
**Severity: Medium-High**

Ruff's settings system (`settings.rs:15-25`):
```rust
pub(crate) struct ResolvedClientSettings {
    fix_all: bool,
    organize_imports: bool,
    disable_rule_comment_enable: bool,  // Enable per-action
    fix_violation_enable: bool,
    // ...
}
```

Sifr has only `lint_enable: bool` — no per-action toggles, no workspace vs global resolution, no configuration preference enum (`EditorFirst`, `FilesystemFirst`, `EditorOnly`).

---

## MEDIUM — Should Consider

### 7. Organize Imports Is Correctly Rejected
**Severity: Informational — Strategy Sound**

The capability matrix explicitly classifies import sorting as `not-applicable`. `code_action_resolve.rs:116-122` shows Ruff's organize imports restricts rules to `[Rule::UnsortedImports, Rule::MissingRequiredImport]`. Sifr must never implement this.

**Verdict: Correct rejection.**

### 8. NOQA System Is Correctly Rejected
**Severity: Informational — Strategy Sound**

Ruff's `noqa.rs` is a comprehensive Python-specific system (~1000+ lines) handling:
- `# noqa` (blanket)
- `# noqa: F401, F841` (specific)
- Line-specific vs. file-level
- Conflict detection

Sifr's `suppressions.rs` in `sifr_lint` uses `# sifr: ignore[rule-id]` — different syntax, simpler semantics.

**Verdict: Must not reuse noqa.rs.**

### 9. Diagnostic Severity Mapping Is Correct But Incomplete
**Severity: Low**

Ruff's `severity()` in `diagnostic.rs:305-312` maps rule codes to ERROR/WARNING. Sifr's equivalent is ad-hoc. Should be rule-driven.

### 10. Notebook Support Correctly Absent
**Severity: Informational**

Ruff's `server.rs:289-297` shows notebook document sync configuration. Sifr has no notebook support, which is correct.

---

## Summary Assessment

| Area | Ruff Pattern | Sifr Should Reuse? | Status |
|------|--------------|-------------------|--------|
| Diagnostics pipeline | `check_path` + `generate_noqa_edits` | No (Python-specific) | ✓ Reject |
| Diagnostic data payload | `AssociatedDiagnosticData` | **Yes, adapt** | ✗ Missing |
| Code action types | `SupportedCodeAction` enum | **Yes, adapt** | ✗ Missing |
| Deferred resolution | `resolve_edit_for_fix_all` | **Yes** | ✗ Stub |
| Workspace edit tracking | `WorkspaceEditTracker` | **Yes, adapt** | ✗ Missing |
| Settings architecture | Workspace/global resolution | **Yes, adapt** | ✗ Basic |
| Fix-all | Applies all safe fixes | **Partially** | ✗ No gate |
| Organize imports | `Rule::UnsortedImports` | No | ✓ Correct |
| NOQA system | Full noqa parsing | No | ✓ Correct |
| Suppression format | `# sifr: ignore[rule-id]` | Keep own | ✓ Own |

**Overall strategy: SOUND**, but missing critical infrastructure:
1. Diagnostic classification system (hard vs. policy)
2. Enforced gating at data layer, not string matching
3. Deferred code action resolution
4. Workspace edit tracking

The LSP protocol layer (`sifr_lsp`) can be thin — the real work is in `sifr_analysis` adding the classification to `RenderedDiagnostic` and building the suppression edit generator.

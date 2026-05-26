

Now I have a complete picture. Let me compile the severity-ranked findings.

---

## Ruff Linter Architecture Audit for Sifr Reuse

### Executive Summary

**Strategy verdict: SOUND with significant caveats.** Ruff's linter has a clean orchestration layer that's separable, but the AST checker and semantic model are deeply Python-specific. You can reuse orchestration patterns and some token/line-level checkers, but you cannot extract an abstraction layer over Python semantics—you must build Sifr-specific engine components, using Ruff's structure as a blueprint, not a library.

---

### Severity 1 — Hard Blockers

#### 1. `SemanticModel` is monolithic Python semantics (not reusable)

**Location:** `ruff_python_semantic/src/model.rs` + `scope.rs` + `binding.rs`

This is the center of gravity. The `SemanticModel` struct owns:
- **Scope stack** with Python scope kinds (`Module`, `Class`, `Function`, `Generator`, etc.)
- **Binding map** — tracks every name binding with binding kinds: `Import`, `SubmoduleImport`, `StarImport`, `FromImport`, `Function`, `Class`, `Annotation`, `Loop`, `ExceptHandler`, `Global`, `Nonlocal`, `Export` (`__all__`), etc.
- **Definition tracking** — `DefinitionId`, `Definition` variants for `Module`, `Class`, `Function`, `Method`, `Assignment`
- **Name resolution** — `resolved_names: FxHashMap<NameId, BindingId>`
- **Import resolution** — `Module`, `ModuleSource`, `Imported`, submodule tracking
- **Type annotation resolution** — deferred annotation handling, `typing_modules` config
- **Python stdlib builtins** (`PYTHON_BUILTINS`, `IPYTHON_BUILTINS`, `MAGIC_GLOBALS`)
- **`__future__` flags**, `global`/`nonlocal` rebinding tracking

This is ~100% Python-specific. Sifr's ownership/type system is structurally different (HIR-level, not name-resolution-level). There is no extraction path here—Sifr needs its own semantic model aligned with its HIR.

**Verdict:** Do not try to reuse `SemanticModel`. Build Sifr-specific semantic analysis over HIR.

#### 2. `SourceKind` is Python/notebook-only

**Location:** `ruff_linter/src/source_kind.rs`

```rust
pub enum SourceKind {
    Python(String),
    IpyNotebook(Notebook),
}
```

Every linter entry point depends on `SourceKind`. For Sifr, this would need to be `SifrSource`, which is trivial (just raw text). But the coupling is at the `check_path` signature level—every phase receives `source_kind` and `source_type: PySourceType`.

**Verdict:** Sifr needs its own orchestration entry point with Sifr-specific source type. Do not attempt to unify `SourceKind`.

#### 3. `LinterSettings` / `Rule` registry is Python-specific

**Location:** `ruff_linter/src/settings/` + `ruff_linter/src/registry.rs`

The rule registry uses Python-specific codes (`"F401"`, `"E501"`), plugin categories (`pyflakes`, `pycodestyle`, `flake8-*`), and configuration schemas built for Python projects (import sorting, line length, target versions as Python versions, etc.).

**Verdict:** Sifr needs its own rule registry and settings. Ruff's `RuleSet`, `AsRule`, and settings pattern is a good blueprint—copy the *shape*, not the content.

---

### Severity 2 — Significant Coupling (Engine Parts, Not Sharable)

#### 4. AST `Checker` is a 2,431-line Python visitor

**Location:** `ruff_linter/src/checkers/ast/mod.rs`

The `Checker` struct is the AST traversal engine. It:
- Implements `Visitor<'a>` with `visit_stmt`, `visit_expr`, `visit_param`, etc.
- Handles Python-specific statement types: `StmtImport`, `StmtImportFrom`, `StmtFunctionDef`, `StmtAsyncFunctionDef`, `StmtClassDef`, `StmtFor`, `StmtAsyncFor`, `With`, `AsyncWith`, `Match`, `Try`, `With`, `Global`, `Nonlocal`
- Manages deferred visits: function bodies, lambda bodies, type param definitions, `class` base expressions, string type definitions, future annotations
- Tracks docstring state (PEP 257 conventions)
- Uses `Importer` for import insertion
- Uses Python-specific AST nodes (`QualifiedName`, `Identifier`, `Quote`, etc.)

This is not a generic checker pattern—it *is* the Python checker.

**However:** The **structure** is reusable:
- Deferred visit pattern (store nodes, analyze after AST traversal)
- Visitor trait over AST nodes
- Phase ordering: binding → traversal → cleanup → analysis
- Diagnostic accumulation via `&mut diagnostics`

**Verdict:** Copy the *architectural pattern* (single-pass AST visitor, deferred nodes, phase ordering). Rewrite the Python-specific implementation.

#### 5. Import sorting checker is deep Python machinery

**Location:** `ruff_linter/src/checkers/imports.rs` + `ruff_python_semantic/src/analyze/imports.rs`

Import sorting requires:
- Full Python import resolution: relative vs absolute, `__future__`, star imports, `__all__` exports
- Third-party vs first-party vs local分类
- Configurable force-single-line, combine-on-top, etc.
- Module lookup in the Python environment

This is inapplicable to Sifr. Sifr's package system is different (has `sifr_package` crate). Import rules for Sifr would need a complete rewrite.

**Verdict:** Not reusable. Sifr needs its own import analysis.

---

### Severity 3 — Reusable Patterns (Extractable)

#### 6. Orchestration layer is structurally separable

**Location:** `ruff_linter/src/linter.rs`

`check_path()` shows a clean phase order:
```
1. Token-based rules (check_tokens)
2. Filesystem rules (check_file_path)
3. Logical-line rules (check_logical_lines)
4. AST rules (check_ast)
5. Import rules (check_imports)
6. Doc-line rules (post-AST)
7. Physical-line rules (check_physical_lines)
8. noqa directive enforcement
9. per-file-ignores filtering
10. Fix applicability adjustment
```

Each phase is gated by `settings.rules.iter_enabled().any(|r| r.lint_source().is_X())` — lazy phase skipping.

**Reusable for Sifr:** The phase-gated orchestration pattern is directly applicable. Sifr would implement:
- Token-based (whitespace, ambiguous unicode in comments)
- Filesystem-based (file naming conventions)
- AST-based (type rules, ownership rules over HIR)
- noqa suppression

**Verdict:** High-value reuse. The orchestration *structure* transfers cleanly.

#### 7. Token checker (`check_tokens`) is mostly generic

**Location:** `ruff_linter/src/checkers/tokens.rs`

Most token-based rules are language-agnostic:
- `BlankLinesChecker` (blank line between methods, top-level, too many blanks)
- `TabIndentation` (tabs vs spaces)
- `EmptyComment`, `CommentedOutCode`
- `AmbiguousUnicodeCharacterComment`
- `UTF8EncodingDeclaration` (Python-specific `# -*- coding: utf-8 -*-`)

Sifr could reuse the token-checker phase structure. The *implementation* of ambiguous-unicode, commented-out-code, encoding-declaration are Python-specific, but the phase orchestration pattern (iterate tokens, call rule functions) is reusable.

**Verdict:** Medium-value reuse. The phase pattern is reusable; individual rules need Sifr equivalents.

#### 8. Physical/logical line checkers are structurally reusable

**Locations:** `ruff_linter/src/checkers/physical_lines.rs`, `logical_lines.rs`

These operate on line/text ranges without semantic understanding. Line length, indentation, blank lines, trailing whitespace—all generic concepts. Sifr's `trailing_whitespace` rule in `sifr_lint` is already doing this work.

**Verdict:** These checker phases transfer well. Sifr can use the same phase-gated pattern.

#### 9. noqa directive system is a good blueprint

**Location:** `ruff_linter/src/checkers/noqa.rs` + `noqa.rs`

Ruff's noqa system:
- Parses `# noqa: F401`, `# noqa: F401, E501`, `# noqa` (blanket)
- Maps diagnostic codes to noqa comments via comment range lookup
- Handles line-number → source-offset resolution
- Tracks "ignored" diagnostics to remove from output
- Supports per-file-ignores and noqa sections

Sifr's current `# sifr: ignore[rule-id]` is a simpler version of this. Ruff's implementation is a solid reference for a more complete noqa system (e.g., blanket suppression, multiple rules, unused suppression detection).

**Verdict:** Blueprint-worthy. Sifr can model its noqa system after Ruff's but needs its own comment parsing (different syntax).

#### 10. `ruff_diagnostics` crate is generic

**Location:** `ruff_diagnostics/`

The diagnostic types (`Diagnostic`, `Violation`, `Fix`, `Applicability`, `IsolationLevel`, `SourceMap`) are language-agnostic. Sifr's `RenderedDiagnostic` in `sifr_diagnostics` serves the same purpose. The crate structure (severity, fix applicability, source mapping) is a good reference.

**Verdict:** The crate design is reusable. Sifr's `sifr_diagnostics` already fills this role.

#### 11. Rule trait pattern — there is no trait

**Location:** `ruff_linter/src/rules/**`

Surprisingly, Ruff does **not** use a trait for rules. Each rule is a free function that receives `&mut diagnostics` and settings. For example:

```rust
pub(crate) fn unused_imports(
    diagnostics: &mut Vec<Diagnostic>,
    scope: &Scope,
    // ... many more params
)
```

Or for AST visitors, rules implement `Visitor` themselves and are called from within `Checker::visit_stmt`. Rules are invoked in the checker body via `flake8_pyi::rules::...`.

There is no `Rule` trait, no `Runnable` trait, no abstraction. This is actually *simpler* for Sifr—you just write functions.

**Verdict:** No abstraction to fight. Write free functions. This is the easiest path.

#### 12. `ruff_source_file` / `Locator` pattern

**Location:** `ruff_source_file/`, `ruff_linter/src/source_kind.rs`

`Locator<'a>` provides zero-copy source text slicing via byte offsets. `SourceFileBuilder` constructs source file metadata. This pattern is language-agnostic (just text + offsets) and highly reusable.

**Verdict:** Good candidate for reuse or Sifr adaptation.

---

### Severity 4 — Non-Issues (Already Addressed)

#### 13. `ruff_text_size` — already forked/transplanted

The `TextRange`, `TextSize`, `Ranged` trait are generic. Ruff's version is based on `unicode-line-break` / `text_size`. Sifr's AST should already have equivalent types.

#### 14. `ruff_python_parser::Parsed` / `Tokens`

Parser output types (`Parsed`, `Tokens`, `ParseError`) are language-specific. Sifr has its own parser (`sifr_python_parser` fork) and its own AST (`sifr_syntax`). These are not shareable but the *pattern* (parse → get tokens + AST + comments) is what matters for orchestration.

---

### Recommended Architecture for Sifr

Based on this audit, here's the sound approach:

```
sifr_lint_engine (new crate)
├── orchestrator.rs      — Phase-gated check_path equivalent (tokens → filesystem → AST → noqa)
├── checkers/
│   ├── tokens.rs        — Token-based rules (generic: whitespace, tab, ambiguous unicode)
│   ├── lines.rs         — Physical/logical line rules (generic: line length, blank lines)
│   ├── ast.rs           — AST visitor over sifr_syntax (Sifr-specific, NOT from Ruff)
│   └── noqa.rs          — noqa directive handling (blueprint from Ruff, Sifr syntax)
├── rules/
│   ├── whitespace.rs    — Generic line-level rules
│   ├── style.rs         — Sifr-specific style rules
│   ├── ownership.rs     — Sifr ownership rules (no Python analog)
│   └── type_check.rs    — Sifr type system rules (no Python analog)
├── settings.rs          — Sifr rule registry and settings (NOT from Ruff)
└── diagnostics.rs       — Uses sifr_diagnostics (already exists)
```

**Key principle:** Sifr's lint engine should be **informed by Ruff's architecture but not dependent on it**. The phases are the same; the implementations are Sifr-specific.

---

### Summary Table

| Component | Reusable? | Severity | Strategy |
|---|---|---|---|
| Orchestration phase order | ✅ Yes | 3 | Copy pattern |
| Token checker phase | ✅ Yes | 3 | Copy pattern, rewrite rules |
| Physical/logical line checkers | ✅ Yes | 3 | Copy pattern, rewrite rules |
| noqa directive system | ✅ Blueprint | 3 | Model after Ruff, Sifr syntax |
| `ruff_diagnostics` types | ✅ Yes (Sifr has own) | 4 | Already addressed |
| `Locator` / source file | ✅ Blueprint | 3 | Adapt to Sifr's text types |
| Rule function pattern | ✅ Yes | 3 | Free functions, no trait needed |
| `SourceKind` | ❌ No | 1 | Sifr needs own |
| `LinterSettings` / `Rule` registry | ❌ No | 1 | Sifr needs own |
| `SemanticModel` | ❌ No | 1 | Must build Sifr-specific over HIR |
| AST `Checker` (2,431 lines) | ❌ No | 2 | Rewrite over sifr_syntax |
| Import sorting | ❌ No | 2 | Not applicable; Sifr package system |
| Python-specific rules | ❌ No | 1 | `flake8_*`, `pyflakes_*`, etc. |

**Bottom line:** Ruff is an excellent *architectural blueprint* but not a *library dependency* for linting. The orchestration pattern, phase gating, noqa system, and diagnostic types transfer. The AST traversal, semantic model, Python-specific rules, and configuration must be Sifr-native.

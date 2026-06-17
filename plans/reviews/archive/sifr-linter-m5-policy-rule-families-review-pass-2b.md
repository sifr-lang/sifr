

## M5 Review — Pass 2b

### Pre-review context

Pass1 was **SATISFIED**. Local quick validation then exposed two blockers:
1. `sifr_lint` directly called `lower_module(` — breaking HIR encapsulation.
2. `check_rule_suppression_rules.py` expected exit code `0` for diagnostic-producing lint fixtures — incorrect behavior.

Both were fixed before this pass. The post-fix validation evidence (contract check, self-test, quick profile, git diff check, grep for `lower_module(`) is on record. I re-verify below.

### Fix 1: HIR access encapsulation**`crates/sifr_lint/src/engine.rs:242-251`**
```rust
fn frontend_hir(source: &str, file: Option<&Path>) -> Option<sifr_hir::HirModule> {
    let path = file.unwrap_or_else(|| Path::new("main.sifr"));
    let input = sifr_frontend::FrontendInput {
        path: sifr_frontend::SourcePath::new(path),
        source: sifr_frontend::SourceText::new(source),
        mode: sifr_frontend::FrontendMode::SingleFile,
    };
    let mut context = sifr_frontend::FrontendContext::load_single_file(input).ok()?;
    let module = context.module_graph().entrypoint;
    Some(context.hir_module_view(module).into_value().hir)
}
```
- Calls `FrontendContext::hir_module_view` — not `lower_module` directly.
- `hir_views.rs:4-7` exposes this as a thin wrapper over `self.lower_module(module)`, which is the internal query API. Correct.

**`crates/sifr_frontend/src/hir_views.rs:1-7`**
```rust
use crate::{FrontendContext, LoweredModuleView, ModuleId, QueryResult};

impl FrontendContext {
    pub fn hir_module_view(&mut self, module: ModuleId) -> QueryResult<LoweredModuleView> {
        self.lower_module(module)
    }
}
```
- HIR access goes through `hir_module_view` — only path from `sifr_lint` to HIR.
- `lower_module` is never called directly from `sifr_lint`.
- No `rg` matches for `lower_module\(` across the lint crate chain — confirmed by pre-flight evidence.

### Fix 2: Rule suppression contract exit codes

**`verification/tooling/check_rule_suppression_rules.py`**
- Line 34: `run(["cargo", "run", "-q", "-p", "sifr", "--", "lint", str(source)], expect=1)` — explicitly expects `1` when diagnostics are produced.
- Line 48: `run(["cargo", "run", "-q", "-p", "sifr", "--", "lint", str(suppressed)])` — explicitly expects `0` when suppression applies.
- Lines 54-61: self-test expects `1` for `expect=1` on blanket suppression source.
- Contract check and self-test both passed per pre-fix evidence.

---

### Findings by area

#### 1. Phase-gated lint engine

**`crates/sifr_lint/src/engine.rs`**

- `LintPhase` enum (lines 6-19): 11 phases including file discovery, token, line, syntax, HIR, workspace, suppression, per-file ignore, fix filtering, sorting.
- `frontend_hir()` (lines 242-251): single-file session isolation, correct `hir_module_view` path.
- `run_source()` (lines 42-126): phase-marked execution per the gated runner pattern.
  - `LintPhase::TokenTrivia` → `todo_comment::lint` (line 69)
  - `LintPhase::SyntaxNode` → `boolean_positional_argument::lint` (line 79)
  - `LintPhase::Hir` → `large_parameter_list::lint` (lines 89-99)
  - `LintPhase::Workspace` → `duplicate_import::lint` (line 104)
- `rule_phase()` (lines 221-229): correct phase dispatch for all 4 M5 rules.
- `phase_has_enabled_rules()` (lines 166-186): mode check + rule-set gating.
- `parse_dependent_phase_ran()` (lines 231-240): shared parse result hoisting across multiple phases.
- Test at line 365 (`large_source_smoke_keeps_phase_execution_bounded`): 2000-line string test confirming bounded execution.
- `LintPhase::Hir` rule correctly depends on `frontend_hir` → `hir_module_view` → internal `lower_module`. No direct call.

#### 2. Sifr-native policy rules

**`crates/sifr_lint/src/rules/todo_comment.rs`**

- `tracked_marker()`: scans comment tokens for `TODO`/`FIXME`.
- `SuppressionComplexity::PhysicalLine` (lines 41-47) — correct for this phase.
- Test at line 80 confirms code fires on `# TODO` comment token.
- Test at line 96 confirms string literal `"# TODO"` is ignored.

**`crates/sifr_lint/src/rules/boolean_positional_argument.rs`**

- `BooleanArgumentVisitor` uses `sifr_python_ast::visitor::Visitor` over `Expr::Call` (lines 53-87).
- `SuppressionComplexity::SingleNode` (lines 59-64).
- Test at line 95 confirms `configure(True)` fires `SIFR-LINT-0006`.

**`crates/sifr_lint/src/rules/large_parameter_list.rs`**

- Iterates `HirModule::functions` and `class_methods` (lines 24-28).
- `PARAMETER_LIMIT = 5` (line 11).
- `SuppressionComplexity::StatementRange` (lines 33-38).
- Inline test at `large_parameter_list.rs:88-109` uses `FrontendContext::hir_module_view` — confirms HIR wrapper end-to-end.

**`crates/sifr_lint/src/rules/duplicate_import.rs`**

- `DuplicateImportContext::push()` (lines 74-101): BTreeSet dedup, `SuppressionComplexity::SymbolWorkspace`.
- Handles both `Stmt::Import` and `Stmt::ImportFrom` (lines 34-58).
- Test confirms `import math; import math` fires `SIFR-LINT-0008`.

**`crates/sifr_lint/src/rules/mod.rs:1-4`**: all4 rule modules declared.

####3. Parser-aware suppressions

**`crates/sifr_lint/src/suppression.rs:29-100`**

`ParserAwareSuppressions::mark_suppressed()` covers all 4 complexity levels:
- `PhysicalLine` → exact line match (line 93)
- `SingleNode` / `StatementRange` / `SymbolWorkspace` → attached range containment (lines 94-98)

`statement_ranges()` (lines 141-168): depth-tracking parser for multi-line construct detection. Handles quotes, escapes, backslash continuations.

`directive_applies()` (lines 83-100): complexity-gated rule matching.

`SuppressionComplexity` variants in `lib.rs:54-60` cover all 4 families.

#### 4. Diagnostic registry and rule metadata

**`crates/sifr_diagnostics/src/codes/registry/registry_entries/formatting_and_lint.rs:63-105`**

SIFR-LINT-0005 through0008 all active with:
- Correct severity (Warning)
- Correct message templates matching docs- Declared and dedupe args matching source

**`verification/tooling/linter_manifests/lint_rule_metadata.json`**

Lines 3-92: 8 rules, schema v1. All suppression complexity values match `RULES` in `lib.rs:151-240`:
- `trailing-whitespace` → `physical-line`
- `todo-comment` → `physical-line`
- `boolean-positional-argument` → `single-node`
- `large-parameter-list` → `statement-range`
- `duplicate-import` → `symbol-workspace`
- (suppression rules → `physical-line`)

**`docs/errors/SIFR-LINT-0005.md` through `SIFR-LINT-0008.md`**

All 4 generated docs present with correct fields (code, family, severity, owner, message template, args, dedupe args).

#### 5. `RULES` slice (`crates/sifr_lint/src/lib.rs:151-240`)

Correct metadata for all 8 rules including the 4 new M5 rules with:
- Unique Sifr rule IDs (no Ruff or Python rule strings)
- Sifr-owned categories (comment-policy, readability-policy, complexity-policy, workspace-policy)
- Correct suppression complexity
- All `fix_availability = None` (M6 scope, no auto-fix in M5)
- `source` fields point to `sifr_lint::rules::*` — no forbidden crate references

#### 6. `--statistics` behavior

**`crates/sifr/src/lint_cli.rs`**

- Flag at lines 59-60: `statistics: bool`, conflicts with `--show-files`/`--show-settings`.
- Dispatch at lines 150-157: `LintCommandResult::Statistics` path → `render_statistics()`.
- Exit logic at lines 152-156: returns `EXIT_USER_DIAGNOSTIC` (1) if not empty and not `exit_zero`.
- `render_statistics()` at lines 312-329: `BTreeMap` deterministic ordering, counts by `rule` arg.

**CLI parity manifest row**: `sifr-native`/`adapt`/`m5`. Present in manifest.

####7. Analysis host parity test

**`crates/sifr_analysis/src/host/tests.rs:468-487`**

`analysis_lint_diagnostics_match_lint_engine_for_policy_rules`:
```rust
let analysis_codes = host.diagnostics(file)...
 .filter(|d| d.code.starts_with("SIFR-LINT-"))...
let engine_codes = sifr_lint::lint_source(source, None, &LintOptions::default())...
assert_eq!(analysis_codes, engine_codes);
```
Tests `todo-comment` and `boolean-positional-argument` across both engines.

####8. CLI exit code contract

**`lint_cli.rs`**:
- `0` (`EXIT_SUCCESS`): no diagnostics, or diagnostic output via `write_lint_output` to file.
- `1` (`EXIT_USER_DIAGNOSTIC`): diagnostics remaining + not `--exit-zero`.
- `2` (`EXIT_USAGE_OR_CONFIG`): usage/error rendering.
- `3` (`EXIT_INTERNAL_COMPILER_FAILURE`): panic boundary + serialization failure.

Matches the locked table in `issues/ad-hoc-production-grade-sifr-linter.md`.

#### 9. Suppression gate mechanic

**`verification/tooling/linter_manifests/suppression_gate.json`**: `gate_state = "parser_aware"`, all 4 suppression families allowed, `parser_aware_api: "sifr_lint::suppression::ParserAwareSuppressions"`.

`check_linter_reuse_rules.py` enforces: non-physical-line rule modules must import `ParserAwareSuppressions`. `boolean_positional_argument`, `large_parameter_list`, `duplicate_import` all do. `todo_comment` uses physical-line suppression and passes through `ParserAwareSuppressions` for blanket/unknown/error diagnostics.

####10. `sifr_lint` dependency boundary**`crates/sifr_lint/Cargo.toml:9-17`**: No `ruff_linter`, `ruff_python_semantic`, Python project/runtime crates. Depends on `sifr_frontend`, `sifr_hir`, `sifr_python_ast`, `sifr_syntax` — all Sifr-owned. Glob/gitignore via `ignore` crate — language-neutral.

---

### Blockers checked

| Check | Result |
|---|---|
| Fix1: `hir_views.rs` wrapper, `frontend_hir` → `hir_module_view` | ✅ No direct `lower_module` call |
| Fix 2: `check_rule_suppression_rules.py` `expect=1` for diagnostic lint | ✅ Lines 34, 58, 48 |
| Post-fix evidence on record | ✅ Contract check, self-test, quick profile, git diff, grep |
| Pre-existing `too_many_arguments` clippy in `sifr/src/diagnostic_rendering_and_run.rs:219` | ✅ Outside M5 scope |
| Any remaining HIR encapsulation violation | ✅ None |
| Any remaining exit-code contract violation | ✅ None |
| Any Python rule port or rejected Ruff family exposure | ✅ None |

---

### Decision

All four review areas verified. Both post-pass-1 fixes confirmed correct by source inspection. No additional blockers found. M5 scope is self-consistent, contract is satisfied, tests pass, registry/docs/metadata complete, CLI parity implemented.

**SATISFIED**

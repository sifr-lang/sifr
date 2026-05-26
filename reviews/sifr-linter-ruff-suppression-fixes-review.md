Now I have a comprehensive view. Let me write the audit findings.

---

## Audit: Ruff Suppression/Directive/Fix/Diagnostic Reuse for Sifr

Findings ordered by severity. Strategy assessment at the end.

---

### CRITICAL: `noqa.rs` — Reuse with Adaptation

**Severity: Critical**

The core suppression engine in `ruff_linter/src/suppression.rs` is the most architecturally valuable piece Ruff offers. Its design solves exactly the problem Sifr already identified: attaching directives to multi-line constructs.

**What to adapt:**
- `NoqaMapping` — maps any position to the line where the `noqa` directive actually lives. This handles multi-line strings, f-strings, and line continuations. Sifr needs this for HIR-level diagnostics that attach to nested expressions, not the line where the comment lives.
- `Directive::try_extract` — lexer for `# noqa`, `# noqa: RULE`, case-insensitivity, embedded `#` comments, etc. Sifr's version in `sifr_lint/src/lib.rs` is ~30 lines and skips 80% of Ruff's subtlety. The Ruff version handles leading/trailing whitespace, `# noqa # other comment` patterns, and `:` vs no-`:` delimiters.
- `NoqaDirectives` with `find_line_with_directive` — binary search over comment positions. Sifr currently does a linear scan. For workspace-scale linting, this matters.
- `rule_is_ignored` — the core check function that resolves a diagnostic offset through `NoqaMapping` and looks up the directive.

**What to reject:**
- `FileNoqaDirectives` / `ParsedFileExemption` — this is the `# ruff: noqa` file-level blanket exemption. Sifr's design (hard errors are unsuppressible, only policy rules are suppressible) makes blanket exemptions conceptually coherent but Sifr should decide whether to support file-level blanket ignores.
- `extract_noqa_line_for` — this function maps logical lines to physical lines for tokenization edge cases (f-strings, continuation lines). If Sifr's parser-level AST already handles these, this may be unnecessary. Check whether `sifr_python_parser` already normalizes multi-line constructs.
- `IsortDirectives`, `TodoDirective` — isort-specific and TODO-specific, not relevant to Sifr.

**Safety of adaptation:** The `NoqaMapping` and `Directive` parsing are safe to port. They're pure functions over text. The key adaptation: Sifr's syntax is `# sifr: ignore[rule-id]` vs Ruff's `# noqa: RULE`. The `Directive::lex_code` function assumes Ruff's `PREFIX+DIGITS` format (`F401`) — Sifr rule IDs are different (`trailing-whitespace`, `unused-suppression`), so the code-lexing logic needs updating.

**Key insight from `NoqaMapping`:** The `resolve` function binary-searches over ranges and returns the *end* of the containing range. This means a diagnostic at any position in a multi-line string maps to the directive line at the end. Sifr needs equivalent logic for HIR diagnostics that attach to nested expressions.

---

### HIGH: `fix.rs` — Reuse Wholesale

**Severity: High**

`ruff_diagnostics/src/fix.rs` is clean, small (~170 lines), and directly maps to Sifr's existing `SuggestionApplicability`. Ruff's `Applicability` has three levels (`DisplayOnly`, `Unsafe`, `Safe`) while Sifr has four (`Unspecified`, `HasPlaceholders`, `MaybeIncorrect`, `MachineApplicable`). These are compatible — map Sifr's levels to Ruff's:

| Sifr | Ruff | Meaning |
|---|---|---|
| `MachineApplicable` | `Safe` | Fix is correct |
| `MaybeIncorrect` | `Unsafe` | User opt-in |
| `HasPlaceholders` | `DisplayOnly` | Show only |
| `Unspecified` | (no fix) | N/A |

`IsolationLevel` (`Group` / `NonOverlapping`) is valuable for Sifr's fix engine and should be copied directly.

**Safety of adaptation:** The `Fix` struct is a thin wrapper around `Vec<Edit>` + `Applicability` + `IsolationLevel`. Sifr already has `SuggestionApplicability` in `sifr_diagnostics/src/model/mod.rs` — this is already parallel to Ruff's design. The fix engine in `fix/mod.rs` (`apply_fixes`) handles overlapping edit resolution, source map generation, and rule-specific ordering (`cmp_fix`). All of this is reusable.

**Key insight from `apply_fixes`:** It sorts fixes by rule-specific ordering (e.g., `RedefinedWhileUnused` before `UnusedImport`), then by start position, then by tie-breaking rules. Sifr needs similar rule-interaction ordering for HIR-level fixes.

---

### HIGH: `diagnostic.rs` — Sifr Already Has a Better Design

**Severity: High**

Ruff's `Diagnostic` struct in `ruff_diagnostics/src/diagnostic.rs` is simpler than Sifr's `SifrDiagnostic`. Ruff stores:
```rust
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub range: TextRange,
    pub fix: Option<Fix>,
    pub parent: Option<TextSize>,
}
```

Sifr's `SourceDiagnostic` is richer: it has `message_template` + `args` for template rendering, `related_spans` for multi-location diagnostics, `children` for sub-messages, `help`, and `suggestions`. Sifr's design is better-aligned with rustc's diagnostic design.

**Recommendation:** Keep Sifr's design. Ruff's `Diagnostic` is too thin for Sifr's needs. The only value to steal is the `parent` field — the concept of attaching a diagnostic to a parent node for suppression purposes. This is how Ruff handles multi-line diagnostics: the diagnostic range points at the inner node, but `parent` points at the containing line where `noqa` applies.

---

### MEDIUM: `violation.rs` — Reject

**Severity: Medium**

Ruff's `Violation` trait is a rule-specific trait that generates human-readable messages. Sifr's approach uses `DiagnosticBuilder` with `message_template` + `args` for the same purpose. Ruff's approach requires each rule to implement `message()` and `message_formats()`, returning format strings. Sifr's approach is more type-safe and avoids the stringly-typed format system.

**Recommendation:** Do not adapt. Sifr's builder pattern with typed args is superior.

---

### MEDIUM: `directives.rs` — Selectively Reuse

**Severity: Medium**

The `extract_directives` function in `ruff_linter/src/directives.rs` orchestrates tokenization-based directive extraction. It depends on Ruff's `Tokens`, `Indexer` (which tracks f-string ranges, continuation lines, etc.), and `CommentRanges`. These are all Ruff's parser-level infrastructure.

**What to reuse conceptually:** The `Flags` pattern — enable only the directive types relevant to the current run. Sifr could use this to conditionally parse `sifr: ignore` only when lint rules are enabled.

**What to reject:** The tokenization-dependent extraction. Sifr should parse comments at the AST level, not the token level. The `extract_noqa_line_for` logic is about Python-specific tokenization edge cases (f-strings spanning lines, backslash continuations). Sifr's parser should produce a normalized AST that doesn't need this.

---

### LOW: `fix/edits.rs` — Reuse Strategically

**Severity: Low**

`ruff_linter/src/fix/edits.rs` is the high-level edit-generation library (delete statement, remove argument, add argument, etc.). This is Python-specific CST manipulation — things like handling trailing semicolons in multi-statement lines, dedenting blocks within multiline strings, etc.

**Recommendation:** Sifr's fix generation will need equivalent logic for Sifr's AST, not Ruff's. The concepts transfer but the code does not. Specifically:
- `delete_stmt` — the logic for "what to delete when removing a statement" is valuable. It handles the lone-child-replaced-with-`pass` case, trailing semicolons, leading content, continuation lines. Sifr will need this but adapted for Sifr's statement semantics.
- `adjust_indentation` — uses LibCST for multi-line string safety. Sifr may not need this if Sifr doesn't have Python-style multi-line string semantics.
- `remove_argument` / `add_argument` — Python-specific argument manipulation. Not relevant.

---

### LOW: `fix/mod.rs` — Reuse Core Algorithm

**Severity: Low**

The `apply_fixes` algorithm in `ruff_linter/src/fix/mod.rs` is the fix application engine. The algorithm:
1. Sort fixes by rule-ordering, then start position, then tie-breaking.
2. For each fix, check isolation constraints (`IsolationLevel::Group`).
3. Check for overlap with already-applied fixes.
4. Apply edits in order, building a source map.

**This is directly reusable for Sifr.** The `SourceMap` concept (tracking start/end markers for applied edits) is also valuable for LSP-level fix presentation.

---

### CRITICAL: `rule_is_ignored` — The Core Check

**Severity: Critical**

In `suppression.rs`, the function `rule_is_ignored` at line 264 is the canonical example Sifr needs to replicate:

```rust
pub(crate) fn rule_is_ignored(
    code: Rule,
    offset: TextSize,
    noqa_line_for: &NoqaMapping,
    locator: &Locator,
) -> bool {
    let offset = noqa_line_for.resolve(offset);  // Map to directive line
    let line_range = locator.line_range(offset);
    match Directive::try_extract(locator.slice(line_range), line_range.start()) {
        Ok(Some(Directive::All(_))) => true,
        Ok(Some(Directive::Codes(codes))) => codes.includes(code),
        _ => false,
    }
}
```

Sifr needs this exact pattern for its HIR-level diagnostics. The current `sifr_lint` suppression only handles line-based policy rules. For HIR diagnostics, Sifr needs:
1. Parse `sifr: ignore[rule-id]` comments into a directive index.
2. For each diagnostic, resolve its position through `NoqaMapping` (handling multi-line constructs).
3. Check if any directive on the resolved line covers the diagnostic's rule.

---

## Strategy Assessment

**Is the strategy of reusing Ruff suppression/directive/fix/diagnostic code sound?**

**Yes, with caveats:**

1. **The core suppression engine (`NoqaMapping` + `Directive` parsing + `rule_is_ignored`) is the right thing to adapt.** This solves Sifr's stated problem: "current line-based suppression is enough only for token-line rules; syntax/HIR rules need parser-aware statement/range attachment." `NoqaMapping.resolve()` provides exactly that attachment.

2. **The fix engine (`Fix`, `apply_fixes`) is worth copying.** Sifr already has `SuggestionApplicability` parallel to Ruff's `Applicability`. The isolation level and fix application algorithm are sound.

3. **Sifr's diagnostic model is already better than Ruff's** — keep `DiagnosticBuilder`, keep template-based message rendering, keep `related_spans`.

4. **The main blocker is the text-addressing model.** Ruff uses `TextSize` / `TextRange` throughout, with a `Locator` for text slicing. Sifr has its own source map model. The suppression code is deeply coupled to Ruff's text infrastructure — adapting it requires either adopting `Locator` / `TextSize` / `TextRange` as the text-addressing primitive, or writing adapter layers.

5. **Sifr's rule ID format** (`trailing-whitespace`, `unused-suppression`) differs from Ruff's (`F401`, `E741`). The `Directive::lex_code` and `Directive::lex_flake8`/`lex_ruff` functions assume the `PREFIX+DIGITS` format. These need updating for Sifr's identifier-based rule IDs.

6. **Sifr's `# sifr: ignore[rule-id]`** differs from Ruff's `# noqa: RULE`. The lexer needs a Sifr-specific variant of `Directive::try_extract` that recognizes `sifr:` instead of `noqa:`.

**Recommendation by priority:**
1. Copy `NoqaMapping` + `Directive` parsing (adapted for `sifr:` prefix and Sifr rule IDs) — **this solves the core problem**.
2. Copy `IsolationLevel` and the fix application algorithm from `fix/mod.rs` — **Sifr's fix engine is incomplete**.
3. Keep Sifr's `DiagnosticBuilder` / `SifrDiagnostic` — **already superior**.
4. Reject `Violation` trait, `IsortDirectives`, `TodoDirective` — **not relevant**.
5. Adapt `rule_is_ignored` as the canonical check function for HIR diagnostics.

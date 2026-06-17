

Now I have a complete picture. Let me provide the review findings.

---

## Phase 36.1 Planning Review: Production-Grade Sifr Formatter

### Summary

The phase plan is **well-structured and directionally sound**, but there are **concrete blockers and missing artifacts** that must be resolved before implementation starts.

---

### Blocker 1: Ruff formatter does not support Sifr parameter conventions

**Location**: `third_party/ruff/crates/ruff_python_formatter/src/other/parameter.rs`

The parser extension in `statement.rs` correctly parses `mut`, `own`, `own mut`, and `mut own` as Sifr parameter conventions. However, the Ruff formatter's `FormatParameter` rule handles only `name` and `annotation`:

```rust
impl FormatNodeRule<Parameter> for FormatParameter {
    fn fmt_fields(&self, item: &Parameter, f: &mut PyFormatter) -> FormatResult<()> {
        let Parameter {
            range: _,
            node_index: _,
            name,
            annotation,
        } = item;
        // Only formats name and annotation — no convention handling
        name.format().fmt(f)?;
        // ...
```

**Impact**: AC-6 cannot be met. The plan explicitly requires `mut own` → `own mut` canonicalization, but the formatter would either:
- Print nothing for `own`/`mut` (dropping the modifiers)
- Print them in whatever order they appeared (not canonical)

**Fix required**: Part 2 of the phase must extend `FormatParameter` (or add a Sifr-specific override) to:
1. Read `AstParamConvention` from the Parameter node
2. Print `own mut` (not `mut own`) for owned mutable parameters
3. Print nothing for borrow-by-default (matching Python's no-modifier default)
4. Handle `mut` alone (mutable borrow), `own` alone (owned), `own mut` (canonical owned mutable)

---

### Blocker 2: Capability matrix does not exist

**Issue**: The phase plan requires a "Ruff-to-Sifr formatter capability matrix" as the first deliverable (Part 1, `formatter_capability_audit_and_contract_lock`). The matrix is described in the "Ruff Capability Parity Contract" section, but **the matrix itself has not been produced**.

**Why this is a blocker**: Without the matrix:
- There's no authoritative list of which Ruff capabilities need Sifr implementation
- There's no classified `supported`/`adapted`/`not-applicable`/`blocked` designation for each capability
- Implementation could diverge from the stated parity contract without anyone noticing
- External reviewers have no concrete acceptance criteria for capability coverage

**Fix required**: Part 1 must produce the matrix before Part 2 begins. The matrix must cover all 16+ capability rows listed in the "Ruff Capability Parity Contract" section of `ad-hoc-production-grade-sifr-formatter.md`.

---

### Blocker 3: Sifr Ruff fork integration point is unspecified

**Issue**: The plan says "the fork must expose a stable integration point for Sifr crates rather than requiring Sifr to shell out to a CLI for in-process formatting." However:

- The current `sifr_format` calls `parse_module` from `sifr_syntax` which wraps `ruff_python_parser`
- There's no `ruff_python_formatter` integration in `sifr_format` at all
- The `ruff_python_formatter` crate is in `third_party/ruff`, not `crates/sifr_format`
- No Cargo dependency or API boundary exists for Ruff formatter → Sifr integration

**Fix required**: Part 1 or Part 3 must define:
1. The exact Cargo dependency: will `sifr_format` depend on `ruff_python_formatter` directly, or via a new `sifr_format`/`sifr_formatter` internal crate?
2. The public API boundary: what `format_source`/`format_range`/etc. calls does Sifr need from the Ruff formatter?
3. How formatter options (line length, indent style, etc.) are passed from `sifr_format` to `ruff_python_formatter`
4. How diagnostics (invalid syntax, formatting errors) are returned in the Sifr diagnostic shape

---

### Blocker 4: Config discovery contract is underspecified

**Issue**: The plan says Sifr must define config discovery and precedence for formatter options, but the actual `sifr_format` currently has no config system — just a hardcoded `FormatOptions::default()`.

The phase plan mentions:
- `sifr.toml` as canonical Sifr config
- Ruff config files for migration convenience
- Config precedence for formatter-specific options

But there's no Sifr config parsing, no `sifr.toml` schema definition, and no `ruff.toml` compatibility layer.

**Fix required**: Part 1 must define the config layer before Part 4 implements the CLI. Specifically:
1. Does `sifr.toml` have a `[format]` section or `[tool.sifr.format]`?
2. Does Sifr read `ruff.toml` or `pyproject.toml` for formatter options?
3. What's the precedence: CLI flags > `sifr.toml` > Ruff config > defaults?
4. How are Sifr-specific options (if any) distinguished from Ruff-compatible ones?

---

### Blocker 5: Guardrail definition is incomplete

**Issue**: AC-14 requires "a guardrail fails when a new Sifr AST syntax extension has no formatter coverage." The Phase 36 tooling verification framework in `tooling_verification.md` exists, but:

- `check_formatter_rules.py` currently tests idempotence and round-trip, not AST coverage
- There's no existing guardrail that validates formatter coverage for new parser extensions
- The phase plan doesn't specify what the guardrail actually checks (AST enumeration? Missing formatter rule detection?)

**Fix required**: Part 1 must define the guardrail mechanism before Part 6 implements validation. Without this, it's unclear how `blocked` status would be detected automatically rather than through manual review.

---

### Gap 1: Diff mode and stdin formatting are marked as conditional

**Location**: Matrix rows for Diff mode and Stdin formatting say "Supported if Ruff reusable APIs expose it; otherwise an explicit implementation plan is required."

Ruff's formatter (`ruff_python_formatter`) is primarily a library that produces formatted text. The `ruff format` CLI provides `--diff` and `--stdin-filename` support. Whether the library exposes these as re-usable APIs (not CLI subprocess calls) must be verified before the matrix can be marked `supported` rather than `blocked`.

**Fix required**: Part 1 must either:
- Verify that `ruff_python_formatter` exposes `format` and `format_idempotent` that can produce diffs and handle stdin internally, OR
- Document the alternative approach if it doesn't

---

### Gap 2: Docstring code formatting — not-applicable rationale needed

**Location**: Matrix row "Docstring code formatting" — "Supported for Sifr docstrings if Sifr exposes docstrings; otherwise not-applicable with rationale"

Sifr does support docstrings (Python-style triple-quoted strings). But the phase plan doesn't say whether Sifr will support Ruff's docstring code formatting feature (`docstring-code-format`).

**Fix required**: Part 1 must decide and document:
- Does Sifr want `docstring-code-format` support? (Ruff supports it for Python code examples in docstrings)
- If yes: this is an additional formatter capability that must be implemented
- If no: explicit `not-applicable` rationale must be recorded

---

### Gap 3: YAPF pragma support is underspecified

**Location**: "Support Ruff formatting pragmas: `# yapf: disable`, `# yapf: enable`"

The phase plan lists YAPF pragmas alongside Ruff pragmas, but:
- Ruff's formatter does **not** have a `yapf: disable` implementation in the Python formatter crate (it recognizes YAPF pragma aliases in specific contexts, but this is lint/rule behavior, not formatting behavior)
- The plan says "where meaningful" — but doesn't define which contexts are meaningful for Sifr

**Fix required**: Part 1 must define exactly which pragmas Sifr will support and in which contexts. Ambiguity here leads to implementation divergence.

---

### Observation: Entry criteria says "Sifr Ruff fork can parse current Sifr syntax extensions" — this is verified

I verified that `statement.rs` in `ruff_python_parser` correctly parses `own mut` / `mut own` parameter conventions. The parser extension is present at lines ~3096-3140. This is a prerequisite that appears to be met.

---

### What is Ready

1. **Phase structure** is sound: 7 parts, sequential execution, validation gates, external review requirement
2. **Quality contract** is detailed: idempotence, parser round-trip, no panics, split-brain prevention, diagnostic stability
3. **Architecture requirements** are clear: Ruff fork owns AST rules, `sifr_format` owns wrapper, `sifr_analysis`/`sifr_lsp` call through it
4. **Acceptance criteria** are concrete: 14 ACs that map to verifiable behaviors
5. **Validation plan** is comprehensive: fork tests, wrapper tests, tooling contract checks, full suite

---

### Required Changes Before Implementation

| # | Change | File | Description |
|---|---|---|---|
| 1 | **Add capability matrix** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Produce the Ruff-to-Sifr formatter capability matrix with `supported`/`adapted`/`not-applicable`/`blocked` classification for all capability rows |
| 2 | **Define Ruff formatter integration API** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Specify the exact Cargo dependency, public API, option passing, and diagnostic return shape for Ruff formatter → Sifr integration |
| 3 | **Define config layer** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Specify `sifr.toml` format section, Ruff config compatibility, precedence rules, and default values |
| 4 | **Define formatter coverage guardrail** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Specify how AC-14 is automatically enforced: what checks fail, what the guardrail script does |
| 5 | **Verify Ruff library API coverage** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Confirm whether `ruff_python_formatter` exposes diff-mode and stdin-mode as library APIs (not just CLI) |
| 6 | **Decide docstring code formatting** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Explicit decision on `docstring-code-format` support with rationale |
| 7 | **Clarify YAPF pragma scope** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Define exactly which YAPF pragmas are supported and in which contexts |
| 8 | **Document formatter extension gap** | `issues/ad-hoc-production-grade-sifr-formatter-execution.md` | Explicitly document that `FormatParameter` in the Ruff fork must be extended to handle Sifr parameter conventions — this is the primary implementation blocker for Part 2 |

---

### Conclusion

**Not ready for implementation.** The phase plan is a good foundation but has 5 concrete blockers and 3 gaps that must be resolved in Part 1 before any code is written. Once the capability matrix, integration API, config layer, guardrail mechanism, and Ruff library API coverage are documented, the phase will be ready to proceed.

The most critical immediate fix is **#8**: documenting that the Ruff fork's `FormatParameter` rule does not handle Sifr parameter conventions and must be extended as part of Part 2. This is the single largest technical gap between the phase plan's AC-6 requirement and the current Ruff formatter implementation.

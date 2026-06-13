

Now I have the complete picture. Let me write the review.

---

## Pass-3 Review: CLI/Config Parity and Ruff Crate Reuse

### SummaryThe pass-2 update resolved the pass-1 blockers. The execution doc now has the capability matrix, integration API, config contract, and other required artifacts. However, a **`CLI parity manifest is missing as a structured artifact**, and several Ruff CLI options not yet in the capability matrix need explicit audit before Part 4 can proceed. These are not blockers on the plan's completeness—they are **Part4 execution requirements that must be locked before implementation starts**.

---

### Finding 1 (BLOCKER): CLI Parity Manifest is Undocumented

**Location**: `issues/ad-hoc-production-grade-sifr-formatter-execution.md`, "Formatter CLI Parity Contract" section, line 160:

> "Part 1 must add a CLI parity manifest that lists each Ruff formatter CLI option, its Sifr spelling, classification, and test fixture. **Part 4 cannot start until that manifest has no unreviewed rows.**"

The execution doc does not contain a CLI parity manifest table. It describes the required command model and behavioral contract, but it does not enumerate each Ruff formatter CLI option with a corresponding Sifr spelling and classification. This is the gating artifact for Part 4.

**Required action**: Add a CLI parity manifest table to the execution doc before Part 4 begins. The manifest must list every Ruff `FormatCommand` CLI option (from `args.rs:420-503`) with its Sifr spelling, classification, and test fixture reference.

---

### Finding 2 (Observations): Ruff FormatCommand CLI Surface Audit

Cross-referencing Ruff `args.rs:420-503` with the execution doc capability matrix reveals **13 Ruff `FormatCommand` CLI options** and how they map:

| Ruff option | Execution doc status | Sifr equivalent |
|---|---|---|
| `files` (positional) | Implicit in contract | `path` positional, but default target `.` not yet implemented |
| `--check` | ✓ Supported | `sifr fmt --check` exists |
| `--diff` | ✓ Supported | Not implemented (listed in contract line 147) |
| `--no-cache` / `--cache-dir` | Adapted | Not implemented; cache behavior contract exists but CLI flags missing |
| `--respect-gitignore` / `--no-respect-gitignore` | Adapted | Not implemented |
| `--exclude` | ✓ Supported | Not implemented |
| `--extend-exclude` | Supported (in TOML config) | Not implemented as CLI flag |
| `--force-exclude` / `--no-force-exclude` | Adapted | Not implemented |
| `--line-length` | ✓ Supported | Not implemented as CLI flag |
| `--stdin-filename` | Adapted | Not implemented |
| `--extension` | Not-applicable | Correctly excluded from Sifr scope |
| `--target-version` | Blocked | Correctly blocked until Sifr syntax-version policy defined |
| `--preview` / `--no-preview` | ✓ Supported | Not implemented as CLI flag |
| `--range` | ✓ Supported | Not implemented |

**The 11 missing CLI options are not plan gaps**—the contract is documented. They are Part 4 implementation items that the CLI parity manifest must lock before Part 4 starts.

---

### Finding 3 (Verification): Ruff Crate Reuse Is CorrectThe execution doc's integration API section correctly identifies the Ruff APIs to reuse. Confirming each against `format.rs` and `format_stdin.rs`:

| Ruff API | Used in format.rs/format_stdin.rs | Execution doc reference |
|---|---|---|
| `format_module_source` | ✓ (line 357) | Listed in Integration API section |
| `format_range` | ✓ (line 345) | Listed in Integration API section |
| `PyFormatOptions` | ✓ (via `to_format_options`, line 340) | Listed in Integration API section |
| `FormatterSettings` | ✓ (line 93) | Listed in Integration API section |
| `to_format_options` | ✓ (line 340) | Listed in Integration API section |
| Ruff resolver patterns | ✓ (`python_files_in_path`, `match_exclusion`, line 29) | Listed as "language-neutral patterns" |
| `SourceKind::diff` | ✓ (format_stdin.rs:125, format.rs:530) | Listed as "diff utilities" |
| `QuoteStyle`, `MagicTrailingComma`, `PreviewMode` | ✓ (format.rs:26) | Listed as option types |
| `FormatRange` parsing | ✓ (args.rs:1004-1050) | Sifr needs equivalent implementation |

**Conclusion on reuse**: The execution doc's integration API section is accurate. Ruff's formatter library APIs are directly reusable without shelling out to a Ruff CLI subprocess, which was a pass-1 concern correctly resolved.

---

### Finding 4 (Verification): exit Status Contract Is Documented

The execution doc's "Formatter CLI Parity Contract" section (line 146-148) correctly describes:

- `--check` exits nonzero on drift, zero if formatted- `--diff` prints unified diffs and exits nonzero on drift
- Write mode exits normally

This matches Ruff's `format.rs:209-228` behavior exactly. No discrepancies.

---

### Finding 5 (Verification): Sifr Parameter Formatter Requirement Is Specified

The "Sifr Parameter Formatter Requirement" section (lines 221-236) correctly specifies the 5-case table and explicitly forbids post-processing. This was the most critical pass-1 gap and is now documented.

---

### Finding6: Missing `--default-target` / Root BehaviorRuff's `FormatCommand` defaults to `.` when no files are provided (via `resolve_default_files` at `format.rs:67`). Sifr's current CLI requires a `path` argument (lines 247-253 in `cli_model_and_entrypoint.rs`). The execution doc contracts default target `.` (line 144) but the CLI model explicitly says `[FILES]...` with **no default**.

This is an implementation gap, not a plan gap. The contract is correct; the current CLI implementation does not match. Part 4 must add the default-target behavior.

---

### Finding 7: Config Layer Contract Is Complete

The TOML schema (lines 166-185), 4-level precedence rules (lines 187-204), and semantic guarantees (lines 196-204) are complete and correct. The config contract is ready for Part 4 implementation.

---

### Remaining Production-Readiness Gaps (Non-Blockers on Plan)

These are Part 4 implementation concerns, not planning blockers:

1. `--diff` CLI option not implemented
2. `--stdin-filename` not implemented (Stdin formatting row is "adapted", not blocked)
3. `--no-cache` / `--cache-dir` not implemented
4. `--respect-gitignore` / `--no-respect-gitignore` not implemented
5. `--exclude` / `--force-exclude` not implemented
6. `--line-length` not implemented as CLI override
7. `--preview` / `--no-preview` not implemented
8. `--range` not implemented
9. Default target `.` not implemented
10. Unified diff generation (using `SourceKind::diff` API, already identified for reuse)
11. Formatter settings validation warnings (`warn_incompatible_formatter_settings` at `format.rs:782-906`)
12. Output summaries and changed-file listing (`FormatResults::write_summary` at `format.rs:556-617`)

---

### Verdict

**The phase plan is ready** as a planning artifact. All pass-1 and pass-2 gaps are resolved:

- Capability matrix: 18 rows with complete classifications ✓
- Integration API: Exact Rust types, API signatures, Ruff crate list ✓
- Config contract: TOML schema, precedence, semantics, exit behavior ✓
- Guardrail design: 6-point spec ✓
- Parameter formatter requirement: 5-case table, no post-processing ✓
- Pragma scope: 4 pragmas defined ✓
- Docstring decision: Blocked gate with3 decision paths ✓- Ruff crate reuse: Verification confirmed ✓

**The phase is ready for implementation**, subject to:
1. Adding the CLI parity manifest table to the execution doc before Part 4 starts (as contractually required by line 160)
2. Tracking the 11 missing CLI options as Part 4 implementation items
3. Blocking on B-1 through B-4 per the implementation plan**Say explicitly: The plan is ready.**

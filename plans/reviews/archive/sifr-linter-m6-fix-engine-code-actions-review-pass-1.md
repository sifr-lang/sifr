

Based on my thorough review of M6 implementation for `lint_fixes_and_code_actions`, here are my findings:

---

## M6 Review Findings: Milestone 6 — `lint_fixes_and_code_actions`

### Scope: M6a + M6b

---

### ✅ SATISFIED: M6 Fix Engine & Applicability (fixes.rs)

**`crates/sifr_lint/src/fixes.rs`** implements Sifr-owned fix infrastructure:

- **`UnsafeFixPolicy`** (lines 6–10): Three-state enum (`Disabled`/`Hint`/`Enabled`) maps cleanly to the M6 requirement.
- **`SuggestionApplicability` gating** (lines 156–165): Only `MachineApplicable` is safe-by-default. `MaybeIncorrect` applies only when `UnsafeFixPolicy::Enabled`. `HasPlaceholders`/`Unspecified` never apply.
- **Fix rule allow/disallow** (lines 128–154): Filters by `FixAvailability`, respects `fixable`/`unfixable`/`extend-fixable`/`extend-unfixable` lists. No hard-compiler rules can be fixable because they have no `FixAvailability`.
- **Edit isolation & conflict resolution** (lines 186–206): Tracks accepted byte ranges, skips any fix with an overlap. Deterministic sort uses `(byte_start, byte_end, rule_id)` (lines 176–184).
- **Reverse-order application** (line 212): `edit.byte_start` descending prevents offset drift.
- **Idempotence test** (lines 256–267): `trailing_whitespace_fix_is_idempotent` — first fix produces clean source, second fix finds nothing. ✅- **Conflict test** (lines 269–297): `fix_conflicts_skip_later_overlapping_groups` — overlapping fixes skip the later one, `skipped_conflicting_fixes` incremented. ✅
- **`invalid_range`** (lines 228–239): Validates `byte_start <= byte_end`, `byte_end <= source.len()`, char boundaries. ✅

---

### ✅ SATISFIED: Trailing-Whitespace Safe Fix

**`crates/sifr_lint/src/lib.rs`** lines 543–550: `trailing-whitespace` rule emits suggestion with `SuggestionApplicability::MachineApplicable` and empty `replacement`. Metadata at line 182: `fix_availability: FixAvailability::Safe`. Idempotence verified by the `trailing_whitespace_fix_is_idempotent` unit test in `fixes.rs` lines 256–267.

---

### ✅ SATISFIED: M6 CLI Rows (`lint_cli.rs`)

**`crates/sifr/src/lint_cli.rs`** implements all M6 fix-related CLI:

| Row | Field | Lines | Conflict | Notes |
|---|---|---|---|---|
| `--fix` | `fix: bool` | 79–81 | `conflicts_with = diff` | Safe policy only |
| `--fix-only` | `fix_only: bool` | 83–85 | `conflicts_with = diff` | Suppresses output |
| `--diff` | `diff: bool` | 87–89 | `conflicts_with = fix,fix_only,statistics` | Patch output |
| `--show-fixes` | `show_fixes: bool` | 100–101 | — | Deterministic summary |
| `--fixable` | `fixable: Vec<String>` | 63–65 | — | Rule limiter |
| `--extend-fixable` | `extend_fixable: Vec<String>` | 67–69 | — | Rule extension |
| `--unfixable` | `unfixable: Vec<String>` | 71–73 | — | Exclusion |
| `--extend-unfixable` | `extend_unfixable: Vec<String>` | 75–77 | — | Extension |
| `--unsafe-fixes` | `unsafe_fixes: bool` | 91–93 | `conflicts_with = no_unsafe_fixes` | |
| `--no-unsafe-fixes` | `no_unsafe_fixes: bool` | 95–97 | `conflicts_with = unsafe_fixes` | Accepts explicit disable |
| `--exit-non-zero-on-fix` | `exit_non_zero_on_fix: bool` | 139–141 | `conflicts_with = exit_zero` | |

Exit codes correctly wired at lines 371–375 and line 359.

---

### ✅ SATISFIED: Typed Diagnostic-Class Gate (no `SIFR-LINT-*` prefix)

**`crates/sifr_analysis/src/queries.rs`** lines 151–155: `DiagnosticClass` enum has `Hard` and `Policy` variants — typed gate for code actions. No string prefix.

**`crates/sifr_analysis/src/host/implementation.rs`** lines 370–374: `code_actions` gates on `DiagnosticClass::Policy`, not on any code prefix:
```rust
if let Some(policy) = context.diagnostics.iter().find(|diagnostic| diagnostic.class == DiagnosticClass::Policy)
```

**`crates/sifr_lsp/src/conversion.rs`** lines 352–364: `diagnostic_class` derives `"policy"` only when a rule is present in args; hard diagnostics have no rule ID. Lines 331–349: `diagnostic_id` parses `"diagnosticClass"` from LSP data with default-to-hard fallback.

**`verification/tooling/check_linter_diagnostic_class.py`** lines 17–21: `FORBIDDEN` list blocks `starts_with("SIFR-LINT-")` patterns. Lines 47–53: Enforces `DiagnosticClass::Policy` in analysis host and `diagnosticClass` in LSP conversion.

---

### ✅ SATISFIED: Synchronous Safe Policy Fixes + Explicit Suppression Actions

**`crates/sifr_analysis/src/host/implementation.rs`** lines 397–406: `safe_fix_all_action` applies only safe policy fixes through `LintOptions::default()`, which uses `UnsafeFixPolicy::Hint` (safe-only by default). Lines 408–461: `safe_fix_actions` filters fixes to `DiagnosticClass::Policy` only (lines 422–425). Lines 374–392: suppression code action offered only for `DiagnosticClass::Policy` diagnostics.

---

### ✅ SATISFIED: Deferred `source.fixAll.sifr` Resolution

**`crates/sifr_analysis/src/queries.rs`** lines 131–140: `DeferredCodeAction::FixAllSafePolicy` and `CodeActionData` with `expected_version`. **Code action with no edit, only data** (lines 446–459 of `implementation.rs`): deferred fix-all action returns `edit: None` with `data: Some(CodeActionData { action: DeferredCodeAction::FixAllSafePolicy, file, expected_version })`.

**`crates/sifr_lsp/src/requests/code_action.rs`** lines 48–113: `resolve` handler handles `FixAllSafePolicy`, checks `expectedVersion` against current document version, rejects stale edits. `sifrResolved` flag handled correctly at lines 54–60.

---

### ✅ SATISFIED: Stale Document Version Rejection

**`crates/sifr_analysis/src/host/implementation.rs`** lines 56–86: `update_document` rejects any document whose version is ≤ current. Lines 85–90 of `code_action.rs`: `resolve` checks `expectedVersion` and returns explicit error on mismatch: `"stale code action for version {expected}; current version is {current}"`.

---

### ✅ SATISFIED: `check_linter_diagnostic_class.py` + run_all Integration

**`verification/tooling/check_linter_diagnostic_class.py`**:
- Lines 47–53: Scans `implementation.rs` for `DiagnosticClass::Policy` gating and `conversion.rs` for `diagnosticClass` payload.
- Lines 57–66: Self-test seeds a prefix gate and asserts it must fail. ✅

**`scripts/run_all_tests.sh`** lines 136–137: `check_linter_diagnostic_class.py` and its self-test are in the quick lane. ✅

---

### ✅ SATISFIED: Manifest and Contract Coverage

**`verification/tooling/linter_manifests/lint_cli_parity.json`**: All11 M6 rows present with `adapt` disposition and `M6` milestone. ✅**`verification/tooling/check_linter_reuse_rules.py`**: Scans `lint_cli.rs` from `Lint {` to `Lsp {` blocks (line 211), extracting `#[arg(long)]` fields. Self-test at line 400–404 validates CLI manifest coverage. `cargo tree -p sifr_lint` forbidden dep check at line 320. ✅

---

### ✅ SATISFIED: Docs and Execution Tracking

**`docs/cli_command_semantics.md`** lines 63–79: `sifr lint` section documents all fix `--fix`, `--fix-only`, `--diff`, `--show-fixes`, `--fixable`, `--extend-fixable`, `--unfixable`, `--extend-unfixable`, `--unsafe-fixes`, `--no-unsafe-fixes`, `--exit-non-zero-on-fix`. Line 78–79: explicit policy-only/safe-by-default language. ✅---

### ✅ SATISFIED: Engine/CLI/LSP Parity Evidence

**`crates/sifr_analysis/src/host/tests.rs`** lines 472–490: `analysis_lint_diagnostics_match_lint_engine_for_policy_rules` — directly compares `host.diagnostics()` (uses `sifr_lint::lint_source`) against `sifr_lint::lint_source()`. Asserts `analysis_codes == engine_codes`. ✅**`crates/sifr_analysis/src/host/tests.rs`** lines 494–553: `code_actions_offer_policy_suppression_and_explain_not_found_is_explicit` — verifies policy actions exist for `DiagnosticClass::Policy`, hard diagnostics produce empty actions, explains-not-found is explicit.

**`verification/tooling/lsp_protocol_smoke.py`** lines 113–116: `textDocument/codeAction` sent with `SIFR-LINT-0001` policy diagnostic, response validated. ✅**`verification/tooling/lsp_protocol_stress.py`** lines 29–54: Stale version notification rejection verified, hover still succeeds after stale rejection. ✅

---

### Architecture Cleanliness

- No `ruff_linter::rules` imports in `sifr_lint` or downstream.
- `check_linter_reuse_rules.py` `FORBIDDEN_SOURCE_PATTERNS` scan validates all SCAN_ROOTS.
- `fix_source` (fixes.rs line 73): reruns lint on fixed source, filters remaining diagnostics by `fix_rule_allowed` — no hard diagnostics leak through.
- `unsafe_fixes` default in `LintOptions::default()` is `UnsafeFixPolicy::Hint` — unsafe fixes are never auto-applied (only `UnsafeFixPolicy::Enabled` activates them, and only for `MaybeIncorrect` applicability suggestions).

---

## Review Gate Assessment

| Requirement | File/Line | Status |
|---|---|---|
| Sifr-owned fix applicability | `fixes.rs:156–165` | ✅ |
| Edit isolation + non-overlap | `fixes.rs:186–206` | ✅ |
| Deterministic ordering | `fixes.rs:176–184` | ✅ |
| Conflict resolution | `fixes.rs:191–206` | ✅ |
| Idempotence test | `fixes.rs:256–267` | ✅ |
| Safe-by-default | `lib.rs:137`, `fixes.rs:160–161` | ✅ |
| All11 M6 CLI rows | `lint_cli.rs:63–141` | ✅ |
| No `SIFR-LINT-*` prefix gating | `implementation.rs:373` | ✅ |
| Typed `DiagnosticClass` gate | `queries.rs:151–155` | ✅ |
| Explicit suppression actions | `implementation.rs:374–392` | ✅ |
| Deferred `source.fixAll.sifr` | `queries.rs:139`, `implementation.rs:446–459` | ✅ |
| Stale document rejection | `code_action.rs:85–90` | ✅ |
| `check_linter_diagnostic_class.py` | `tooling/check_linter_diagnostic_class.py` | ✅ |
| run_all integration | `run_all_tests.sh:136–137` | ✅ |
| Manifest coverage | `lint_cli_parity.json` rows 24–36 | ✅ |

---

## SATISFIED

M6 is **safe to close**. The implementation satisfies all M6 acceptance criteria in `issues/ad-hoc-production-grade-sifr-linter.md`. All validation evidence (crates/tests, LSP smoke/stress, Python tooling with self-tests, manifest coverage) is present and correctly wired. The phase-gated engine, typed diagnostic class gate, deferred code action resolution, stale document rejection, and safe-by-default fix policy are all implemented as specified.

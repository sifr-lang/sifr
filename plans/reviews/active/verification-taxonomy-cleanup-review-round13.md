## SATISFIED

Both round-12 blockers are fixed, and no new blocker was introduced.

### Verification of fixes

**B1 — `expires_in_wave` and taxonomy boundary**
- `verification/areas/diagnostics/checks/code_baseline_coverage.py:227` — deferral now requires `("owner", "reason", "issue", "removal_target")`. No remaining `expires_in_wave` in active sources.
- `verification/areas/coverage_matrix/checks/verification_taxonomy.py`:
  - Line 16-27 (`ACTIVE_ROOTS`) now broadly includes `REPO_ROOT/.cursor/commands` and `REPO_ROOT/.cursor/skills`, scanned via `rglob` at line 139.
  - Line 64: `LEGACY_FIELD_PATTERNS` includes `expires_in_` + `wave`.
  - Line 68: `(?:^|[^A-Za-z0-9])(?:wave|waves|milestone|milestones)(?:[^A-Za-z0-9]|$)` — non-alphanumeric boundary catches `_wave`-suffixed identifiers (e.g. `expires_in_wave` ends with `wave` at EOL, preceded by `_`).
  - Self-test at line 121-123 runs before active scan; PASS observed.

**B2 — `.cursor/skills/project-workflow/SKILL.md`**
- Read in full; uses "workflow step", "Acceptance Demo", and "feature-focused demo" (line 12, 58, 60). No milestone, no `m3_demo`, no `<milestone>` placeholder. Path is now covered by taxonomy scope (`.cursor/skills` in `ACTIVE_ROOTS`).

### Confirming evidence
- `python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py` → PASS locally.
- `python3 -m py_compile` of both files → no output (clean compile).
- Manual grep across `.cursor`, `verification`, `crates` (excluding `target/`), `demos`, `docs`, `internal_docs`, `scripts/distribution`, `.github/workflows`, `lib` for:
  - `expires_in_wave|closes_in_wave|closes_in_subwave|<milestone>|m3_demo|SIFR_WAVE|_WAVE_` → no matches outside the taxonomy self-test.
  - `\bmilestone\b|\bMilestone\b|\bwave\b|\bWave\b` → no matches outside `target/` build artifacts (which the script skips at line 151).
  - `validation_contract|contract_matrix|workspace_contracts|integer_dtype_contract|compact_grouping_contract|CONTRACT_MATRIX_COMMAND` → no matches.
- `SIFR_WAVE_D2_DEMO` rename verified at `demos/system_tools/{main.sifr,emitted.rs,idiomatic.rs}` → all three reference `SIFR_SYSTEM_TOOLS_DEMO`.
- `sifr-demo-authoring/SKILL.md:60` uses "delivery-bucket" wording.

### Blocking findings
None.

### Non-blocking suggestions
- `verification_taxonomy.py:49` — the `re.compile(r"\b" + "exp" + r"_m1\b")` allow-pattern is cryptic; a brief inline comment explaining it permits `exp_m1` (and why the concatenation avoids self-flagging) would help future readers. Cosmetic only.
- The `LEGACY_FIELD_PATTERNS` concatenation trick (lines 58-65) is now load-bearing for the self-test integrity; consider a single comment block at line 58 explaining the contract ("constructed at runtime so this file does not trip its own scan"). Cosmetic only.

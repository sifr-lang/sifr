Now I have all the information for a thorough review. Let me compile the findings.

---

## Review: Sifr Production Linter CLI Parity Contract

### Summary

The CLI parity contract is well-formed and covers the meaningful user-facing Ruff `check` surface. No blocking issues. Three precision gaps and one minor gap need targeted edits before M2 starts. The plan is implementation-ready once these are addressed.

---

### Severity 1 — Precision Edits Required

**F1: `--extend-ignore` is absent from the CLI parity table**

`args.rs:249` exposes `--extend-ignore` as a hidden deprecated alias for `--ignore`. The config-surface audit correctly classifies `extend-ignore` as `reject`. However, the CLI parity table has no row for `--extend-ignore` as a hidden deprecated CLI surface.

The `lint_cli_parity.json` manifest should include a row:
```
| hidden `--extend-ignore` | none | reject | M1 (manifest self-test only) |
```

The Sifr CLI must not implement this flag — but the manifest should prove it was considered and rejected. Without this row, `check_linter_reuse_contract.py` cannot distinguish "intentionally omitted" from "forgot to classify." **Recommended edit** — add to the CLI parity table before M1 closes.

**F2: Hidden `--no-fix` / `--no-unsafe-fixes` / `--no-show-fixes` / `--no-preview` need explicit manifest rows**

The table has rows for `--fix`, `--unsafe-fixes`, `--show-fixes`, and `--preview`. It also has explicit rows for `--no-unsafe-fixes`. But the hidden `--no-fix` (args.rs:155-156), hidden `--no-show-fixes` (args.rs:174-175), and hidden `--no-preview` (args.rs:208-209) are not explicitly classified.

The plan states "no hidden compatibility promise for `--no-fix`" for the `--fix` row, which is correct. The same treatment should apply to `--no-show-fixes` and `--no-preview` — they should have explicit `none` / `reject` rows with a note explaining the Sifr decision.

**Recommended edit** — add:
```
| hidden `--no-fix` | none | reject | M1 |
| hidden `--no-show-fixes` | none | reject | M1 |
| hidden `--no-preview` | none | reject | M1 |
```

**F3: `--show-files`, `--show-settings`, and `--statistics` conflicts are underspecified**

Ruff's `CheckCommand` uses `conflicts_with` on these flags (args.rs:371-412). The plan commits to implementing them (M2 for `--show-files`/`--show-settings`, M5 for `--statistics`) but does not specify:
1. How the conflicts interact with each other in the Sifr CLI (e.g., can `--show-files --show-settings` be combined?).
2. Whether `--show-settings` output is specified — Ruff shows resolved settings; Sifr would show `sifr.toml` + overrides. This needs a sentence.

**Recommended edit** — add to the "Required CLI fixtures" section:
> - `--show-files` and `--show-settings` are mutually exclusive; attempting both produces an error.
> - `--show-settings` prints resolved lint config, rule selection, file discovery settings, and CLI overrides for the target.

---

### Severity 2 — Risk Flags (Not Blockers)

**R1: `--ignore-suppressions` and `--ignore` name collision is not explicitly disambiguated in the plan**

The plan correctly distinguishes `--ignore-suppressions` (M3: ignores `# sifr: ignore[...]` comments) from `--ignore <RULE>` (M2: disables rule selection). But the plan text at the CLI parity table does not include the rationale inline. An implementer might reasonably conflate the two.

**Recommended edit** — add to the `--ignore-noqa` row:
> Note: `--ignore-suppressions` does not control rule selection. Use `--ignore <RULE>` to disable specific rules. These are independent flags.

**R2: `--output-format` precedence is sound but the interaction with `--statistics` is unspecified**

Ruff resolves `--show-source` into `--output-format` internally (args.rs:935-970). If `--statistics` is combined with `--output-format json`, what does the statistics output look like? Sifr has no equivalent resolution. The plan should clarify: `--statistics` produces a summary table regardless of `--output-format`, or it should specify that `--statistics` conflicts with `--output-format json`.

**Recommended edit** — add a row or clarification:
> `--statistics` outputs a tabular summary. When `--statistics` is set, `--output-format` controls diagnostic output but statistics appear after diagnostics regardless of format.

**R3: `--ignore-suppressions` is M3, but the M2 CLI surface should document that suppressions will work differently in M2 vs M3**

In M2, line-only suppression exists. In M3, parser-aware suppression replaces it. The plan specifies the milestone sequence but does not explain the behavioral change users will see between M2 and M3 when the same `# sifr: ignore[...]` comment covers a multi-line construct. This is important for users who start testing in M2.

**Recommended edit** — add to the suppression complexity section:
> Between M2 and M3, the behavior of `# sifr: ignore[...]` on multi-line constructs changes: M2 treats it as line-attached; M3 attaches it to the full statement or range. Implementations should document this transition in M2 release notes.

---

### Severity 3 — Minor Gaps (Can Be Addressed in M1/M2)

**M1: `--extend-fixable` has no row**

The table has rows for `--fixable` (M6) and `--unfixable` (M6). `--extend-fixable` (args.rs:301) is not listed. Since fixable-related flags are M6, this is a M6 concern. Add to the table:
```
| `--extend-fixable <RULE>` | same | adapted | M6 |
```

**M2: Exit code 3 is not documented in the exit-status table**

The plan specifies exit codes 0, 1, 2, and 3 in the prose. The formatter precedent has a similar exit-code table. The CLI parity contract should include an explicit exit-status sub-table:
```
| Exit code | Condition |
| 0 | no lint violations, or all violations fixed and `--exit-non-zero-on-fix` not set |
| 1 | violations remain, `--diff` found fixable edits, or `--exit-non-zero-on-fix` observed fixes |
| 2 | invalid CLI args, invalid config, invalid rule selectors, invalid output format, or discovery errors |
| 3 | internal compiler or linter failure caught by the panic boundary |
```

**M3: The `--exit-zero` vs `--exit-non-zero-on-fix` mutual-exclusion rule is not explicit in the table**

Ruff uses `conflicts_with` for these two flags (args.rs:352-361). The plan states this in prose but not in the table rows. The `--exit-non-zero-on-fix` row should note: `conflicts_with: --exit-zero`.

---

### What Is Correct

1. **Ruff-to-Sifr mapping is clean**: `ruff check` → `sifr lint`, `sifr check` stays as hard compiler checking. This is the right decision.

2. **Every relevant user-facing Ruff surface is classified**: `--select`, `--ignore`, `--extend-select`, `--per-file-ignores`, `--extend-per-file-ignores`, `--output-format`, `--output-file`, `--fix`, `--fix-only`, `--diff`, `--unsafe-fixes`/`--no-unsafe-fixes`, `--show-fixes`, `--exit-non-zero-on-fix`, `--exit-zero`, `--statistics`, `--show-files`, `--show-settings`, `--preview`/`--no-preview`, `--exclude`, `--extend-exclude`, `--respect-gitignore`, `--no-respect-gitignore`, `--force-exclude`, `--no-force-exclude`, `--config`, `--isolated`, stdin `-`, `--stdin-filename`.

3. **Correct `reject` dispositions**: `--show-source`/`--no-show-source` (use `--output-format`), `--target-version` (Python versioning), `--extension` (`.sifr`-only).

4. **Correct `future-phase` dispositions**: `--no-cache`/`--cache-dir` (requires lint cache contract), `--watch` (LSP owns editor watch), `--add-noqa` (requires parser-aware suppression fix engine + migration policy), log flags `--verbose`/`--quiet`/`--silent` (requires cross-command logging contract).

5. **Correct milestone assignments**: M2 for non-fix CLI, M3 for `--ignore-suppressions`, M5 for `--statistics`, M6 for fix flags.

6. **The manifest enforcement design is sound**: `lint_cli_parity.json` encoded in M1, `check_linter_reuse_contract.py` verifying the clap surface against the manifest.

7. **The formatter precedent is properly followed**: The linter CLI parity contract mirrors the formatter's option-by-option manifest with required fixtures.

8. **`--unsafe-fixes` = `"hint"` in config is correct**: It surfaces unsafe fixes as unavailable/user-confirmation-required suggestions without auto-applying them.

---

### Recommended Targeted Edits

Apply these to the CLI parity table in `ad-hoc-production-grade-sifr-linter.md` before M1 starts:

1. Add `--extend-ignore`, `--extend-unfixable` as hidden reject rows.
2. Add `--no-fix`, `--no-show-fixes`, `--no-preview` as hidden reject rows.
3. Add `--extend-fixable` as an M6 row.
4. Add `--exit-zero` row with `conflicts_with: --exit-non-zero-on-fix`.
5. Add `--exit-non-zero-on-fix` row with `conflicts_with: --exit-zero`.
6. Add the explicit exit-status sub-table.
7. Add `--ignore-suppressions` row note clarifying independence from `--ignore <RULE>`.
8. Add `--statistics` clarification about format interaction.
9. Add `--show-files`/`--show-settings` mutual-exclusion spec.
10. Add `--show-settings` output description.

---

### Verdict

**Implementation-ready with the 10 precision edits above.** The contract is structurally sound, the milestone assignments are correct, the manifest enforcement design is appropriate, and the formatter precedent is properly applied. Once the 10 edits are incorporated, no remaining gaps prevent M2 from starting.

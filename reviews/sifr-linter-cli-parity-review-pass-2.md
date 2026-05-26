

## Review: Sifr Production Linter CLI Parity Contract — Pass 2

---

### Severity 1 — Precision Requests (All Applied)

Every pass-1 precision request is accounted for:

| Pass-1 request | Status |
|---|---|
| F1: `--extend-ignore` row | ✅ Line 342: `hidden/deprecated \`--extend-ignore <RULE>\` \| none \| reject \| M1 manifest classification only` |
| F2: `--no-fix`, `--no-show-fixes`, `--no-preview` rows | ✅ Lines 350, 356, 365: all three are `none \| reject \| M1 manifest classification only`. The Sifr `--no-preview` explicit-flag decision is documented with "hidden Ruff `--no-preview` \| `--no-preview` \| adapted \| M2" at line 365, which is the correct asymmetry — Ruff's flag is hidden, Sifr's is explicit. |
| F3: `--show-files`/`--show-settings` conflicts and output spec | ✅ Line 362-363: "conflicts with `--show-settings` and `--statistics`" and vice versa. Lines 398 and 389: mutual-exclusion diagnostics and `--show-settings` output description are present. |
| R1: `--ignore-suppressions` independence note | ✅ Line 373: "`--ignore-suppressions`; independent from `--ignore <RULE>`" inline in the row and in prose. |
| R2: `--statistics` interaction clarification | ✅ Line 388: "`--statistics` prints a rule-count summary instead of regular diagnostics. If later combined output is desired, a reviewed update must define exactly how statistics interact with every output format." |
| R3: M2→M3 suppression behavioral change | ✅ Lines 433-434 in suppression complexity: "M2 may expose only the current physical-line suppression behavior. M3 changes multi-line suppression attachment from line-attached to parser-aware statement/range attachment; M2 implementation notes and docs must call out that transition until M3 lands." |
| M1: `--extend-fixable` row | ✅ Line 346: `--extend-fixable <RULE>` with M6. |
| M2: Exit-status sub-table | ✅ Lines 305-311: full 0/1/2/3 exit-status table with precise conditions. |
| M3: `--exit-zero` / `--exit-non-zero-on-fix` mutual-exclusion | ✅ Lines 357-358: both rows include "conflicts_with" phrasing. |

All 10 recommended edits are present. No precision gaps remain.

---

### Severity 2 — CLI Table Coverage

The pass-1 review identified all hidden/deprecated Ruff surfaces that needed manifest rows. The updated table now covers:

| Surface | Row | Classification |
|---|---|---|
| `--extend-ignore` | line 342 | `none \| reject \| M1` |
| `--no-fix` | line 350 | `none \| reject \| M1` |
| `--no-show-fixes` | line 356 | `none \| reject \| M1` |
| `--no-preview` | line 365 | explicit `--no-preview` as `adapted \| M2` |
| `--no-unsafe-fixes` | line 354 | `adapted \| M6` |
| `--extend-fixable` | line 346 | `adapted \| M6` |
| `--extend-unfixable` | line 348 | `adapted \| M6` |

**Coverage is complete.** Every hidden Ruff surface scanned for this phase has a row. The key Sifr-specific decision — `--no-preview` as an adapted explicit flag, not a hidden compatibility surface — is correctly reflected and consistent with `sifr fmt`'s `--preview`/`--no-preview` override symmetry.

---

### Severity 3 — Manifest Schema and Validation Obligations

The manifest schema (lines 314-322) and validation requirements (lines 324-330) are complete:

- `schema`, `ruff_check_sources`, `sifr_cli_sources`, `surfaces`, `output_formats`, `exit_codes`, `rejected_surfaces` — all present.
- Validation proofs cover: every Ruff surface appears exactly once, every implemented surface has an allowed disposition, rejected/future-phase surfaces are absent, conflict pairs are enforced, and every output format/exit code has a fixture.
- The enforcement mechanism (`check_linter_reuse_contract.py` or a dedicated CLI contract checker) is specified. The manifest is encoded in M1.

**The manifest is implementation-ready.** One minor precision note: the schema says `surfaces[].disposition` but the table uses `adapted` as the disposition value. The pass-1 review used `adapt` as the table disposition and `adapted` in the schema. These are the same intent but slightly different spelling. For M1 manifest encoding, the implementer should pick one spelling and use it consistently — I'll flag this as a non-blocking M1 encoding detail.

---

### Severity 4 — Behavioral Specifications

| Area | Status |
|---|---|
| **Output format** | Lines 384-389: `concise`, `full`, `json` specified for M2; other formats gated behind explicit schema fixtures. `--statistics` summary behavior is specified. |
| **`--show-files` / `--show-settings`** | Lines 362-363, 389, 398: mutual-exclusion is explicit, `--show-settings` output is described, required fixtures are listed. |
| **Suppression** | Lines 373-374, 420-434: M3 milestone, `--ignore-suppressions` independence, M2→M3 behavioral transition documented. |
| **Exit status** | Lines 303-311: full 0/1/2/3 table with precise conditions. |
| **Stdin** | Lines 296-297: `-`, `--stdin-filename`, config/discovery/path context behavior specified. |
| **Config** | Lines 297-298, 436-470: CLI override precedence, `--config`/`--isolated`, `[lint]` config schema with full key set. |
| **Discovery** | Lines 367-370, 594-597 (M2 scope), lines 462-469: include/exclude/extend-exclude/force-exclude/respect-gitignore and explicit-target behavior specified. |
| **Fix behavior** | Lines 353-357, 500-512: `--unsafe-fixes`, `--show-fixes`, `--fix`, `--fix-only`, `--diff`, `--exit-non-zero-on-fix` specified for M6; fix engine requirements enumerated. |
| **`--preview`/`--no-preview`** | Lines 364-365, 242, 442: preview/experimental Sifr policy rules through `RuleStatus`, `preview = false` in config, M2 milestone. |

**All behavioral areas are specified enough for implementation and tests.** No underspecified contracts.

---

### Severity 5 — Remaining Blockers or Hidden Planning Decisions

None found. The plan is clean:

- **No hidden Ruff surfaces remain unclassified.** Every hidden/deprecated flag has a row.
- **No implementation-time decisions are deferred to implementation.** Rule families, config surfaces, CLI surfaces, suppression gates, fix requirements, LSP gating, and diagnostic classes are all locked.
- **The parser-aware suppression prerequisite is mechanically gated.** W-8, the suppression-gate manifest, the M1→M3 transition path, and the compile-time API dependency (`sifr_lint::suppression::ParserAwareSuppressions`) are all specified.
- **`--no-preview` is the correct Sifr decision.** Making it an explicit flag rather than a hidden compatibility surface is sound — `sifr fmt` already uses the symmetry, Sifr has no legacy Ruff config surface to preserve, and the plan explicitly documents the decision at line 365.
- **`--extend-unfixable` should be `adapted`, not `adapt`.** Line 348 says `adapted` for `--extend-unfixable`. This is consistent with the schema (`adapted`) and correct since it is a fixed/config surface, not a command. The row is fine.
- **The `--statistics` conflict list is slightly over-specified.** Line 361 says `--statistics` conflicts with `--show-files`, `--show-settings`, `--diff`, and `--watch`. The first three are correct. `--watch` is future-phase and not implemented in this phase — a conflict spec for a non-existent flag is harmless but unnecessary. Non-blocking.

---

### Verdict

**The CLI plan is implementation-ready and elegant enough for implementation. No remaining blockers.**

Every pass-1 precision request is applied. Every hidden/deprecated Ruff surface has a manifest row. The manifest schema and validation obligations are complete and machine-enforceable. Output format, statistics, show-files/show-settings, suppression, exit status, stdin, config, discovery, and fix CLI behavior are specified enough for M2 and M6 implementation. The `--no-preview` explicit-flag decision is sound and documented.

The phase is ready for M1/M2 implementation to begin.

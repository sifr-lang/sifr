# Pass 1 Review — `milestone_diag_7` slice 5: inventory fixture cleanup

Scope of change (uncommitted, docs-only):
- `internal_docs/diagnostic_emission_inventory.md`
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`

Reviewer: Claude (read-only verification — no files modified).

## Verification matrix

### 1. Parser fixture paths match real e2e fail fixtures

Both updated tables (Parser-category proposal table and Target Code And Fixture Plan) reference the same eight fixture paths under `crates/sifr/tests/e2e/fail/`. Each path resolves on disk and the first content line carries the matching `expect-error` code:

| Code | Path in inventory | Disk presence | First line of fixture |
| --- | --- | :---: | --- |
| `SIFR-PARSE-0002` | `parser_expected_token.sifr` | ✓ | `# expect-error: SIFR-PARSE-0002` |
| `SIFR-PARSE-0003` | `parser_malformed_string.sifr` | ✓ | `# expect-error: SIFR-PARSE-0003` |
| `SIFR-PARSE-0004` | `parser_invalid_layout.sifr` | ✓ | `# expect-error: SIFR-PARSE-0004` |
| `SIFR-PARSE-0005` | `parser_invalid_target.sifr` | ✓ | `# expect-error: SIFR-PARSE-0005` |
| `SIFR-PARSE-0006` | `parser_invalid_call_arguments.sifr` | ✓ | `# expect-error: SIFR-PARSE-0006` |
| `SIFR-PARSE-0007` | `parser_malformed_declaration_list.sifr` | ✓ | `# expect-error: SIFR-PARSE-0007` |
| `SIFR-PARSE-0008` | `parser_invalid_match_pattern.sifr` | ✓ | `# expect-error: SIFR-PARSE-0008` |
| `SIFR-PARSE-0009` | `parser_unsupported_syntax.sifr` | ✓ | `# expect-error: SIFR-PARSE-0009` |

These were authored under PR 1714 (`529723b8 Classify parser diagnostics by Ruff category`) — provenance is consistent.

Result: **PASS** — every parser fixture path the inventory now claims is real, and the expect-error code matches the inventory row.

### 2. No stale parser pending notes remain for `SIFR-PARSE-0002..0009`

After the diff, `grep -n "fixture pending" internal_docs/diagnostic_emission_inventory.md` returns only:

- L296 `SIFR-TYPE-0004`
- L299 `SIFR-TYPE-0007`
- L300 `SIFR-TYPE-0008`

All three are in the type-checking family and explicitly out of scope for this slice (the user-stated scope only covers PARSE-0002..0009 and TYPE-0002). No `SIFR-PARSE-*` row references "fixture pending" anywhere in the file (verified by the cross-grep at L53–60 and L280–287). The legacy `SIFR-PARSE-0001` mention at L137 is a removal note in the original-fail-fixture marker table, not a pending note.

Result: **PASS**.

### 3. `SIFR-TYPE-0002` active fixture row alignment

Two locations carry SIFR-TYPE-0002 fixtures:

- L70 (Type System Surface table — already up-to-date pre-slice; established by slice 4 / PR 1717)
- L294 (Target Code And Fixture Plan — newly updated by this slice)

Both now read:

```
crates/sifr/tests/e2e/fail/type_comparison_mismatch.sifr,
crates/sifr/tests/e2e/fail/type_mismatch.sifr,
crates/sifr/tests/e2e/fail/union_type_mismatch.sifr
```

Identical fixture set, identical lexical order. All three fixture files exist on disk and each first content line is `# expect-error: SIFR-TYPE-0002`. `type_comparison_mismatch.sifr` is the helper-specific comparison fixture introduced by `cf026a5a`; `type_mismatch.sifr` and `union_type_mismatch.sifr` predate this slice.

Result: **PASS** — L294 now matches L70 exactly.

### 4. Issue tracker entry accuracy

New line in `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` at L85:

```
- [ ] `milestone_diag_7` slice 5 in progress: retire stale parser fixture-pending
      inventory notes now that `SIFR-PARSE-0002..0009` fixtures exist, and align
      the active `SIFR-TYPE-0002` fixture inventory with the helper-specific
      comparison fixture.
```

Checks:

- Checkbox state `[ ]` is correct: this is an in-progress slice with no merged PR. The neighboring completed entries (slices 1–4) carry `[x]`, "complete and reviewer-satisfied" wording, PR links, and local-validation runbooks — none of those claims appear here, so the entry does not overstate completion.
- The two scope items in the bullet correspond 1:1 to the two diff hunks:
  1. Retire stale parser pending notes — matches the L53–60 + L280–287 changes.
  2. Align the active SIFR-TYPE-0002 fixture inventory — matches the L294 change.
- Phrasing is consistent with the existing tracker style ("`milestone_diag_7` slice N <state>: …").

Minor finding (non-blocking): the bullet describes the SIFR-TYPE-0002 alignment as bringing the active row in line with "the helper-specific comparison fixture" (singular). Strictly the L294 row now lists three fixtures — `type_comparison_mismatch.sifr` (the helper-specific comparison fixture from slice 4), plus `type_mismatch.sifr` and `union_type_mismatch.sifr`. The alignment is with the L70 Type System Surface row that already enumerates all three. The description is defensible (alignment was prompted by the comparison fixture from slice 4, and the surface row is the alignment target), but a reader of just the tracker would underestimate the breadth of the L294 change. Optional tightening: e.g., "align the active `SIFR-TYPE-0002` fixture inventory with the Type System Surface row, including the slice-4 helper-specific comparison fixture."

Result: **PASS with one optional wording tightening** — accuracy is preserved; the bullet does not overstate completion or scope.

## Cross-file consistency spot checks

- `internal_docs/diagnostic_emission_inventory.md` Parser-category table (L52–60) and Target Code And Fixture Plan (L280–287) carry identical fixture paths in identical order across PARSE-0002..0009 — no drift between the two parser tables.
- The "fixture pending in `milestone_diag_7`" string appears only on TYPE-0004/0007/0008 rows after the change; no PARSE row carries the legacy phrase.
- The Type System Surface row at L70 was not touched by this diff; the alignment work concentrates the change on L294 where the previous single-fixture entry sat.

## Out-of-scope changes detected

`git status` reports two additional uncommitted modifications that are not part of this slice:

- `reviews/semantic-diagnostic-code-taxonomy-diag-7-type-mismatch-comparison-fixture-review-pass-2.md` (modified) — slice 4 review artifact.
- Several untracked review/issue files unrelated to the diagnostic taxonomy work (e.g., `ownership-mutability-boundary-root-cause*`, `ad-hoc-signature-invalid-fixture-adaptation-*`, `verification/leetcode/`, `package*.json`).

These are noted only because they share the working tree; they are not part of the docs-only diff under review and should be excluded from any PR carrying this slice.

## Decision

All four reviewer-stated verification points pass. No actionable corrections required to the inventory file. The issue tracker entry is accurate and does not overstate completion; one optional wording refinement is suggested for the SIFR-TYPE-0002 alignment description, but it is not a blocker.

**Verdict: approved.** Optional tightening of the L85 bullet wording can be applied at the author's discretion before opening the PR.

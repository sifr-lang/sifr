I have everything I need. Here is the review.

---

## Review: Reference Tab and Per-Code Diagnostic Error Pages

### What's solid

The existing `error-codes.mdx` index is genuinely strong. Family introductions are concise, table rows are single-line descriptions, and the three inline callouts (IMPORT-0008 note, FMT tip, INTERNAL warning) are exactly the right level of specificity. The IMPORT-0008 note is a model for what inline guidance should look like: one concrete line that tells you exactly what to do. Keep it.

The overall plan shape is right. The idea of a Reference tab, per-code pages seeded from `representative_fixture` paths, and a validation script against `code_catalog.json` is well-conceived.

---

### Issue 1: URL route is already committed in the compiler — this must be resolved before any page is published

`crates/sifr_diagnostics/src/codes/registry.rs:287` emits `https://sifr.sh/docs/errors/<CODE>` as the live `docs_url()` for every diagnostic. The plan proposes `reference/error-codes/sifr-type-0002.mdx`, which would serve at a different path.

This is the highest-priority decision. Options:

- **Keep `/docs/errors/SIFR-TYPE-0002`** as the canonical public route. Remove `errors/` from `.mintignore`, rewrite content as proper `.mdx`, and register pages in `docs.json`. The Reference tab can link to the same pages. No compiler change needed.
- **Change to `/reference/error-codes/SIFR-TYPE-0002`** and update `docs_url()` in the registry before the first page ships. Any live links in editor output or CI will break until updated.

The plan doesn't resolve this, which means implementation will have to backtrack. Decide route convention now, then lock it.

Either path is fine, but the existing route (`/docs/errors/`) has the advantage of being already committed and zero compiler-change cost.

---

### Issue 2: The sidebar structure has a redundant split

The plan lists:

```
Reference > Diagnostics > Overview
Reference > Diagnostics > Error Codes   (← this is the current index)
Reference > Error Codes > Index         (← this is a new index?)
Reference > Error Codes > per-code pages
```

That's two "error codes" groups nested at different levels, and two indexes. Collapse to:

```
Reference > Error Codes > Overview (= current error-codes.mdx, renamed)
Reference > Error Codes > SIFR-TYPE-0002 ... (per-code pages)
```

The family overview text already in `error-codes.mdx` is the right landing page. Give it a sidebar slot; don't duplicate it.

---

### Issue 3: The RESULT-0001 note recommends mechanisms that need auditing first

The current note at `error-codes.mdx:295` reads:

> Use `match`, `unwrap()`, or propagate the result with `?` syntax to resolve it.

The plan already flags this. `unwrap()` reads as Rust-style API. If that isn't the exact public Sifr method name, it will mislead users. The `?` operator also needs confirmation as valid public Sifr syntax before this note goes anywhere near a per-code page. Do not carry this wording forward until the Slice 0 semantic audit resolves it.

---

### Issue 4: 120+ uniform pages is unshippable — tier the coverage

The catalog has ~120 active stable codes across 24 families. Treating them all equally with the full 8-section template would take enormous effort and most of the pages would be rarely visited.

Tier by user-encounter likelihood:

**Tier 1 — full treatment** (erroneous code, explanation, fix, fixed code): TYPE, NAME, CALL, RESULT, MATCH, FLOW, OWN, ASYNC, IMPORT. These are what developers hit doing everyday Sifr programming.

**Tier 2 — focused treatment** (explanation + minimal example): INT, DECIMAL, IO, ENCODING, CLASS, PROTO, FMT, LINT. Specialized but source-code-level.

**Tier 3 — brief treatment** (what happened + what to do, no code example): PACKAGE, WORKSPACE, BUILD, STDLIB, INTERNAL. These are often environment, manifest, or tooling issues — not erroneous-code patterns. A file-tree or terminal example is appropriate, but the full erroneous/fixed source template doesn't fit.

The plan mentions this but doesn't make the tier split explicit. State it in the issue so Slice 5 doesn't stall trying to write a code example for `SIFR-BUILD-0003 (Temporary build workspace creation failed)`.

---

### Issue 5: CODEGEN should not appear as a per-code page

The CODEGEN family has no active codes. The current index handles this cleanly with "(no active codes)". Don't create a `reference/error-codes/codegen-overview.mdx` stub — it will create a dead landing page and confuse users who arrive via search.

---

### Issue 6: INTERNAL-0002 should not get the erroneous/fixed template

`SIFR-INTERNAL-0002` is a `Note`-severity structured summary, not an actionable compiler error. Its page, if it has one, should be a short "what this note means and what to do" paragraph. Do not apply the erroneous-code template to it. INTERNAL-0001 gets a proper page because it asks users to file a bug report — that's actionable. 0002 is informational.

---

### Issue 7: The validation script is the right idea but the spec needs to be tightened

The plan says "a validation step catches missing pages for active codes and stale pages for removed codes." That is correct. But it should also specify:

- The script reads `code_catalog.json` and checks `stability == "stable"` entries
- For each, it asserts a corresponding `.mdx` exists at the chosen public route
- It also checks in reverse: every public error page corresponds to an active code in the catalog (catches stale pages)
- It runs in `scripts/run_all_tests.sh` under the docs profile (not separately)

Without the reverse check, removed codes will accumulate as orphan pages.

---

### Smaller copy notes

- `error-codes.mdx` PACKAGE family description: "The `PACKAGE` family is the largest family" — this is internal narrator commentary. Drop it. The user doesn't care how it ranks. Rewrite as: "The `PACKAGE` family covers the full lifecycle of Sifr packages: manifest validation, dependency resolution, import scope, trust policy, archive integrity, and publish guardrails."
- WORKSPACE description: "legacy import resolution errors that mirror the `IMPORT` family for workspaces not yet migrated" — "not yet migrated" implies a time-bounded state but gives no timeline. Rewrite or drop the parenthetical: users don't need the migration history.
- The plan's proposed 8-section template duplicates intent between "What it means" and "Why this fails." Collapse to 6 sections: Summary, Erroneous example, What went wrong (one block, combining the two), How to fix it, Fixed example, Related. Shorter template → more likely to be filled out completely.

---

### Summary of required decisions before implementation

| Decision | Current state | Action needed |
|---|---|---|
| Public route for per-code pages | Compiler emits `/docs/errors/<CODE>`, plan proposes `/reference/error-codes/...` | Pick one; update compiler or plan |
| Sidebar structure | Two overlapping groups | Merge into one `Reference > Error Codes` group |
| RESULT-0001 `unwrap()`/`?` wording | Unaudited | Do not use until Slice 0 verifies |
| Coverage tiering | Not explicit | Document tiers in Slice 5 acceptance criteria |
| Validation script scope | Forward check only | Add reverse check for stale orphan pages |

The plan is otherwise well-structured. The content principles are sound, the catalog is already the right source of truth, and the existing index page is a quality baseline. The per-code page template just needs the tier split and the route decision locked in before anyone writes a line of MDX.

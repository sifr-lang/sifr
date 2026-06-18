## Review: ad-hoc-documentation-learning-path (pass 2)

The revision picks up most pass-1 corrections cleanly: Slice 0 now exists, Slice 2 is reduced to two pages, `From Python` is moved after Quickstart, the sidebar order puts Standard Library above CLI/Packages, `Status` is demoted to a bottom Project group, `index.mdx` card targets are specified, the CPython-modules question is resolved (link to `stdlib/overview.mdx`), and the Generated Rust page is declined. Slice 1's acceptance is now testable.

A small number of remaining issues:

### Findings

**1. Slice 0 names the contradiction but does not pick a side. — important**

Lines 319-322 leave the resolution as "either A or B." The `From Python` table at line 114 is hedged accordingly (`T | None` *or* a typed diagnostic). The issue therefore tells the Slice 0 implementer "fix the contradiction," then tells the Slice 1 author "write a table that documents whichever resolution Slice 0 chose." That's a real implicit dependency: Slice 1 cannot start until Slice 0 has both decided and recorded the contract somewhere stable. Make this explicit — either commit in this planning artifact to one option (whichever matches current compiler behavior, verifiable from `crates/sifr_hir` or existing tests), or add a Slice 0 acceptance line: "the resolved contract is written into `docs/language/type-system.mdx` and `docs/stdlib/collections.mdx` and the `From Python` table cites those pages by anchor." Without that, the Slice 1 author is the one really resolving it.

**2. Gap section #2 still recommends `Language Tour`, then is countermanded mid-section. — important**

Lines 124-140 keep the original "Missing language tour" recommendation block (`docs/language/tour.mdx`, sidebar title `Language Tour`, evolving example, sections list). Lines 141-143 then say "After review, this page should not be part of the first implementation slice." The artifact reads as if both positions are live. A future implementer skimming for sidebar titles will lift "Language Tour" from line 127. Replacement: either delete the recommendation block entirely (pass-1 #3 said drop the tour), or rewrite the gap as "Tour-shaped content already exists in Quickstart — no new page; if a non-CLI tour is wanted later, fold into `introduction.mdx` with `Accordion`."

**3. The worked Python-reference `<Note>` example demonstrates the wording the revision forbids. — important**

Line 243 adds "Avoid meta copy such as 'Python's documentation explains the syntax shape'; say the product difference directly." Lines 232-235 still print exactly that phrasing as the worked example. The rule and the demo contradict. Replace the code block with something like the pass-1 suggested form, e.g., `If you know Python's match/case from [PEP 634], the syntax is the same. Sifr adds compile-time exhaustiveness — every union member must be covered.` That doubles as a working PEP-link demo (line 240-242 endorses PEPs 484/604/634).

**4. Imports/modules question is under-served by the reduced page set. — optional**

The Python-evaluator persona asks "How do imports and packages work?" (line 60). The reduced Slice 2 (Values and Collections + Iteration) intentionally drops `modules-and-imports`. `From Python` mentions imports in one table row. The Packages group is reference-shaped (manifest/dependencies/publishing), not "here's how `import` works for a Python dev." This is acceptable if Quickstart or `cli/overview.mdx` already shows project-mode imports concretely — pass-1 #6 suggested folding into `cli/overview.mdx`'s project-mode section. The issue does not commit to where this answer lives. Add a one-line note to Slice 1 acceptance: "Quickstart or `cli/overview.mdx` contains a worked `import` example a Python reader can find from the From Python row," or pick a home explicitly.

**5. Slice 1 acceptance constrains only the new page's overlap, not the existing prose. — optional**

Pass-1 #5 asked both: (a) make `from-python.mdx` table-first, and (b) trim the value-prop paragraphs in `introduction.mdx` so the two pages don't decay into duplicates. The revision adopts (a) at line 343 but does not commit to (b). If the value-prop prose in `introduction.mdx` lines 17-26 stays alongside a new table-first bridge page, the duplication risk pass-1 flagged is unresolved. Add a Slice 1 acceptance line about which paragraphs in `introduction.mdx` get trimmed or kept.

**6. Reduced page set is sufficient for a new Python developer otherwise. — optional**

With the caveats above, the path Welcome → Introduction → Install → Quickstart → From Python → (Learn Sifr) → (Standard Library) covers the four personas adequately. Values and Collections plus Iteration as Slice 2 fills the "everyday data model" gap without duplicating stdlib pages. Pattern matching, classes, and error handling already exist. No new finding here — calling it out so it isn't lost.

**7. Slice 0 file list is right; consider naming the verification source. — optional**

Slice 0 lists the four content files but does not say where the implementer should verify the actual contract from (e.g., `crates/sifr_hir/**` lowering rules, a demo, or a specific test). A single line — "verify against existing compile behavior; cite the demo or test that pins the contract" — would prevent Slice 0 from being a doc-only swap that codifies whichever wording feels cleanest.

### Slices are otherwise pragmatic

The 0-1-2-3-4 sequence is small, reviewable, and avoids the "rewrite everything at once" failure mode from the Non-Goals. Slice 1 does five things (new page, status page, sidebar restructure, index card edits, intro/quickstart edits) but they are mechanically coupled — splitting them would create churn. Sidebar ordering matches pass-1 #7 exactly.

### Go / no-go

**Conditional go.** Suitable as the planning artifact once findings 1, 2, and 3 are fixed in the file. They are textual inconsistencies in the issue itself, not strategy errors — a 5-10 minute edit pass. Findings 4 and 5 can be resolved during Slice 1 drafting without blocking adoption.

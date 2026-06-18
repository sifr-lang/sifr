I have enough context to produce the review. Here are my findings.

## Review: ad-hoc-documentation-learning-path

### Overall verdict

The direction is sound — the sequencing/calibration gap is real. But several proposed pages duplicate existing content, the central "panic-free indexing" pitch is inconsistent across current docs in a way the issue does not flag, and Slice 2 is sized too large.

### Findings

**1. The dict-indexing semantics are inconsistent in the existing docs, and the proposed `From Python` table cements one side without resolving it. — blocking**

- `docs/introduction.mdx:39` and `docs/quickstart.mdx:26` show `users["charlie"]` returning `int | None` for a missing key.
- `docs/stdlib/collections.mdx:46` says: "Direct `scores["missing"]` is a compile-time checked operation that produces a typed error if the key might be absent."
- The issue's bridge table at line 114 picks the first reading ("Missing keys / indexes → `T | None`") without flagging the contradiction.

This is the central claim of the language. Resolve it (either `[]` returns `T | None` *or* `[]` is rejected and `.get()` returns `T | None`) and update the introduction/quickstart/collections pages in lockstep with the bridge page. Without this, `From Python` will ship contradicting collections.

**2. `docs/language/type-system.mdx` "None Safety" example contradicts the panic-free pitch — blocking**

Lines 88-98 show `if x > 0:` and `x == target` directly on `x: int | None` with no None check first. Either the compiler narrows numeric comparison against an `Option` (then say so explicitly), or the example is wrong. A redesign whose top selling point is "missing values cannot crash" cannot leave this in place. The bridge work needs to include an audit of existing pages, not only additions. Add this to Slice 1 acceptance.

**3. `Language Tour` duplicates Quickstart — important**

Quickstart already uses `Steps`, an evolving example (dict lookup → `Result` → `isinstance` narrowing), and ends with "what you just did". Adding a parallel "evolving example" Language Tour produces two walkthroughs covering nearly the same surface. Replacement: drop `language/tour.mdx`. Move tour-shaped content into a slimmed-down Quickstart and let `Core Language` pages stay reference-flavored. If a non-CLI tour is genuinely wanted, fold it into `introduction.mdx` as a "Tour at a glance" section using `Accordion`/`Tabs`, not `Steps`.

**4. Place `From Python` *after* Quickstart, not before — important**

The issue raises this as an open question. Recommendation: after. A differences page before any code has been seen reads as a warning dump and violates principle #8 ("State differences as product decisions, not caveats"). Once a reader has compiled `greet.sifr` and watched the compiler enforce `try`/`except`, the bridge becomes a memory aid instead of a hedge.

**5. `From Python` content overlaps `introduction.mdx` value props — important**

`introduction.mdx` lines 17-26 already enumerate Python-vs-Sifr deltas in "Key value propositions" prose. If the bridge page repeats the same five differences in prose, both pages decay. Replacement: make `from-python.mdx` table-first (the comparison table at issue line 110 is already the strongest section), keep "Common surprises" as the only prose section, and remove the value-prop paragraphs from `introduction.mdx` (let the example carry them).

**6. Slice 2 is too big and overlaps stdlib — important**

Six new pages (`values-and-types`, `strings-and-bytes`, `collections`, `iteration`, `functions`, `modules-and-imports`) duplicate existing content:
- `strings-and-bytes` overlaps `stdlib/text-encoding.mdx` and `stdlib/io-filesystem.mdx`.
- `collections` overlaps `stdlib/collections.mdx` (which already covers lists, dicts, tuples, sets).
- `functions` overlaps `ownership.mdx` (which covers borrow-by-default for arguments) and `type-system.mdx` (annotations).

Replacement Slice 2: keep two pages only — `language/values-and-collections.mdx` (scalars + list/dict/tuple/set + truthiness + mutation rules) and `language/iteration.mdx` (for/comprehensions/iterators). Link out to `stdlib/collections.mdx` for `Counter`/`deque`. Defer `strings-and-bytes` and `functions` until there is content that does not already live elsewhere.

**7. Sidebar group ordering misplaces Standard Library — important**

The proposed order (Get Started → Learn Sifr → Reference (CLI) → Packages → Standard Library → Diagnostics) hides stdlib behind packaging concerns. Stdlib is part of the learning surface — `sifr.io`, `sifr.collections`, `sifr.task` are everyday tools, not deployment ceremony. Replacement order: Get Started → Learn Sifr → Standard Library → CLI Reference → Packages → Diagnostics.

**8. `Status` does not belong in Get Started — important**

A first-time reader does not need preview-channel and platform-surface tables before Quickstart. Replacement: put `status.mdx` as the last item in its own "Project" group at the bottom of the sidebar, or fold the contents into `introduction.mdx` as a single `<Note>` that links to a Roadmap page. Get Started should be Welcome / Introduction / Install / Quickstart / From Python — five items, not six.

**9. `docs/index.mdx` edits are unspecified and most-load-bearing — important**

The issue says "targeted edits to docs/index.mdx" without naming them. The current four cards point at `/introduction`, `/cli/overview`, `/packages/overview`, `/stdlib/overview` — none of them point at the new learning entry (`Quickstart` → `From Python`). Specify in Slice 1: index card #2 should become "Quickstart" → `/quickstart`, card #4 should become "From Python" → `/from-python`. The bottom card pair can keep `Installation` + `Quickstart`.

**10. The compatibility-matrix open question blocks Slice 1 framing — important**

Open Question #3 asks "How explicit should the docs be about unsupported CPython runtime features before the compatibility matrix exists?" — answer this before drafting `from-python.mdx`. If no matrix is planned, the bridge page must commit to a static list of supported `sifr.*` modules and explicitly say "no other CPython modules are accepted." The current `stdlib/overview.mdx` "What Sifr Does Not Expose" section is the right shape — bridge page should link to it rather than restating.

**11. `Welcome` sidebar entry adds a click without adding content — optional**

`docs/index.mdx` is a card hub. It can be reached via the logo / tab title. Removing "Welcome" from the sidebar drops the click between sidebar selection and `Introduction`. If you keep it, retitle it to "Overview" so the entry signals what it offers.

**12. The proposed Python-references `<Note>` text is meta — optional**

The example at issue line 234 ("Python's documentation explains the syntax shape, while this page explains the Sifr contract") reads as process commentary about documentation, not product copy. Replacement pattern:

> If you know Python's `match`/`case` from [PEP 634], the syntax is the same. Sifr adds compile-time exhaustiveness — every union member must be covered.

Link to PEP numbers (634, 604, 484) rather than `docs.python.org/3/...` pages whose versions drift.

**13. Pattern Matching page does not need a "different from Python" callout — optional**

`docs/language/pattern-matching.mdx` is already framed around exhaustiveness, the actual point of difference. Adding a generic "If you know Python..." callout would dilute the page. Slice 3 should explicitly skip pattern-matching for callouts.

**14. "Generated Rust" page proposal can be declined — optional**

Open Question #5 asks whether to add a Rust-curious page. Recommendation: no. `docs/cli/check-emit.mdx` and the `sifr emit` step in Quickstart already cover it. A separate page introduces audience confusion (the Rust-curious are not the primary newcomer) and the issue's own Non-Goals discourage scope creep.

**15. Acceptance criterion for Slice 1 is unverifiable — optional**

"Newcomer can answer ... within the first four sidebar pages" — with five Get Started entries in the (revised) proposal, "the first four" is ambiguous. Replacement: "After reading `Welcome`, `Introduction`, and `From Python`, a Python developer can state (a) what Sifr does that Python does not, (b) which Python idioms break, (c) what their next command should be." Make the test about content learned, not page count.

### Style preservation

The current snippet-callout-table-cards style is preserved well by the existing concept pages. The risk is that `Language Tour` reaches for `<Steps>` (workflow shape) on a concept page, breaking principle #4 in the issue itself. Tour-shaped content should use `<Accordion>` or sequential sections, not `<Steps>`. (See finding 3.)

### Suggested slice order after the changes above

1. **Slice 0 (new, blocking)**: Resolve dict-indexing semantics and audit `type-system.mdx` None-safety examples; update intro/quickstart/collections to match. No new pages.
2. **Slice 1**: Add `from-python.mdx` (after Quickstart) and `status.mdx` (bottom group). Update `docs.json` and `index.mdx` card targets. Skip `language/tour.mdx`. Skip `modules-and-imports.mdx` until imports are documented in one place (see finding 6 — it can fold into `values-and-collections.mdx` or stay in `cli/overview.mdx`'s project-mode section).
3. **Slice 2**: Two pages — `values-and-collections.mdx`, `iteration.mdx`. Link out to stdlib.
4. **Slice 3**: Callouts on type-system, error-handling, ownership, stdlib overview, io. Skip pattern-matching.
5. **Slice 4**: Guides — unchanged.

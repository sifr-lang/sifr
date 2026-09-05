Now I have the full picture. Here's the review.

---

## Review: Guide Tracks, Audience Perspectives handoff, Language Docs linking strategy, and Slice 7

### What's already settled (don't re-litigate)

The prior passes resolved naming, sidebar hierarchy, the CardGroup linking pattern as a concept, and the principle of "guides are task-shaped, not concept reference." All of that is confirmed and correctly carried here. This review addresses only the new guide-specific content.

---

### [critical] The scope boundary between `from-python.mdx` and the Python guide track is undefined

This is the single issue that will cause the most damage if left unresolved.

The plan now has three pages all pointing at the same reader:

1. `docs/from-python.mdx` — table-first bridge in Get Started
2. `docs/guides/python-developers/index.mdx` — audience track index
3. `docs/guides/python-developers/start-from-python.mdx` — first guide page

None of these has an explicit scope statement that would prevent an implementer from writing "Start from Python" as a prose version of `from-python.mdx`. The plan gestures at a distinction with "guided track, not a second compatibility reference," but that phrase is not in the Slice 7 acceptance criteria and will not be remembered when the page is being drafted.

The scope split needs to be stated precisely and once:

- `from-python.mdx`: compatibility table only. Every row is a semantic difference. No task walkthroughs, no worked examples. Links out to the guide track for depth.
- `guides/python-developers/index.mdx`: orientation. "Given Python background, here is the sequence of guides to follow." One paragraph, a card list, and a pointer to `from-python.mdx` for the compatibility table. No compatibility explanations.
- `guides/python-developers/start-from-python.mdx` (or rename this more clearly — see below): task walkthrough. "Write your first Sifr program starting from Python habits." No comparison tables. No "if you know Python X, Sifr does Y" unless it's embedded in a task step.

Without this pinned in the issue, the implementer will do the natural thing and merge 1 and 3.

---

### [critical] "Handle typed errors" and "Handle errors without exceptions" are listed separately but sound like the same page

Slice 7 lists two groups:

- First guide set includes: `Handle typed errors`
- Audience seed pages include: `Handle errors without exceptions`

These are almost certainly not the same page (one is in the general how-to track, one is in the Python audience track), but the plan never says this. An implementer reading this will either write them as duplicates or collapse them into one and put it in the wrong location.

Decide: if they're different pages, state the scope difference explicitly. If one is the Python-audience-track version of the other, say that and name them so they don't sound identical (e.g., the Python guide page becomes `Handle errors the Sifr way` and the general guide becomes `Typed error handling`).

---

### [important] The `icon="badge-rust"` in the CardGroup example will fail at render time

`badge-rust` does not exist in Heroicons or FontAwesome, which are the icon sets Mintlify supports. The example as written will produce a broken or missing icon at render time. `snake` (the Python icon) exists in FontAwesome Pro, but not in the free tier, which means it may or may not work depending on the Mintlify plan. The plan should either use verified icon names or strip the icons from the example entirely and add a note: "Icon names must be checked against the project's Mintlify icon registry before use." Leaving an incorrect example invites copy-paste failure.

---

### [important] The card body text for the Python guide links to the wrong thing

The CardGroup example for language-doc links uses this body:

> "See how Sifr changes type hints, exceptions, imports, and object references."

That describes `from-python.mdx` (a compatibility table), not the Python Developers guide track (a task sequence). If this card links to `/guides/python-developers`, a reader who clicks it expecting to understand semantic differences will land on an index of how-to guides. The body should describe what's actually there:

> "Follow a guided learning path designed for Python developers — first program, error handling, ownership, and packages."

The Rust card body is accurate as written.

---

### [important] `Ownership in Sifr terms` and `Results and optional values` (Rust guide) are at high duplication risk with language pages

`language/ownership.mdx` and `language/error-handling.mdx` already exist and cover these concepts. The Rust guide versions of these pages must be task-shaped — not "here is what ownership is" (that's the language page) but "given you know Rust ownership rules, here are the specific Sifr patterns you'll reach for." The distinction is reader assumptions, not topic.

Currently, nothing in the Slice 7 acceptance criteria enforces this. Add something like: "Guide pages that share a topic name with a language page must be distinguishable by task orientation — a reviewer should be able to confirm the guide page cannot be summarized as 'explains concept X.'"

---

### [moderate] `language/pattern-matching.mdx` is missing from the linking examples but is a high-confusion surface for Python developers

The plan lists linking examples for type-system, error-handling, ownership, and concurrency. It omits pattern matching. Sifr adds compile-time exhaustiveness that Python's `match` does not enforce. A Python developer reading the Sifr pattern-matching page will carry the assumption that unhandled cases are a runtime problem, not a compile-time failure. This page needs at least a `<Note>` callout pointing Python readers to the guide — it does not need a two-column CardGroup, but it should appear in the linking strategy list.

---

### [moderate] The audience track index pages have no defined structure

The plan says the index pages "explain what the reader needs to know from their background without duplicating language reference pages." That's a principle, not a structure. Without a defined shape, the index pages will drift into mini-concept pages or compatibility summaries — exactly what `from-python.mdx` is supposed to own.

Add to Slice 7 acceptance: "Track index pages contain: a one-paragraph orientation, the guide sequence as a card list or numbered steps, and a pointer to the relevant reference page for background (`from-python.mdx` for Python, `language/ownership.mdx` for Rust). They do not contain concept explanations or comparison tables."

---

### [minor] "Start from Python" is a weak guide page name — it sounds like `from-python.mdx`

The bridge page in Get Started is called `From Python`. The first guide page is called `Start from Python`. These are close enough to cause navigation confusion when they both appear in search results or sidebar contexts. The guide page should be named for what it does, not where the reader is coming from. Options: `Write your first program`, `First Sifr project`, or `Build a CLI tool as a Python developer`. The audience track establishes the "from Python" framing; the page title can be task-named.

Same concern for `Start from Rust` in the Rust track.

---

### Linking strategy: what's right

The restraint principle ("do not add these cards to every page") is correct and should be honored strictly. The specific page-to-guide mappings (type-system → Python guide from type-hints discussion, ownership → both guides from their respective concepts) are well-targeted. The phrasing guidance ("continuation paths, not warnings") is the right tone. None of that needs revision.

---

### Slice 7 acceptance: what's missing

Three concrete gaps in the acceptance criteria:

1. No criterion defining what the index page must and must not contain (see above).
2. No criterion for preventing duplication with language reference pages for guide pages that share topic names.
3. No criterion requiring that "Start from Python" and "Start from Rust" be task-shaped rather than comparison-shaped — which is the single biggest failure mode for those two pages.

The list of first guide pages ("Build a CLI tool", "Read and write files safely", etc.) is good and the demos can seed all of them. These are correctly task-shaped by title.

---

### Summary of required changes

| Priority | Issue | Fix |
|---|---|---|
| Critical | `from-python.mdx` vs. `start-from-python.mdx` scope is undefined | Add explicit scope statement for all three Python-facing pages |
| Critical | "Handle typed errors" and "Handle errors without exceptions" are indistinguishable | Name them differently and state whether they're one or two pages |
| Important | `badge-rust` icon will fail at render time | Replace with verified icon names or remove icons and flag for verification |
| Important | Python card body describes the bridge page, not the guide track | Rewrite to describe task content |
| Important | Rust guide pages `Ownership in Sifr terms` and `Results and optional values` will duplicate language pages without an explicit task-orientation criterion | Add acceptance criterion |
| Moderate | Pattern matching page missing from linking examples | Add to the list |
| Moderate | Track index pages have no defined structure | Add structure to Slice 7 acceptance |
| Minor | `Start from Python` / `Start from Rust` sound like `from-python.mdx` | Rename to task-oriented titles |

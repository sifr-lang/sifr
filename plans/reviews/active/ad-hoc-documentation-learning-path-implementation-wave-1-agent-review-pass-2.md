I now have a full picture. Let me synthesize the review.

---

## Wave 1 Review

### BLOCKERS

**1. Missing import link — explicit Slice 1 acceptance criterion**

`from-python.mdx` Slice 1 criterion says: "The imports row in `From Python` links to a concrete worked import example, either in `Quickstart`, `cli/overview.mdx`, or another existing page chosen during Slice 1."

The current page has an inline accordion that says `from sifr.collections import Counter` but no hyperlink from the imports section to a worked example page. The accordion title is "Imports use the Sifr namespace" with prose description only. This acceptance item is unmet.

Fix: add a link to `stdlib/collections.mdx` (or `quickstart.mdx`) from the imports accordion.

---

**2. `language/concurrency.mdx` orphaned from navigation**

The previous `docs.json` had concurrency under Core Language. The new one removes it entirely — it's not in "Learn Sifr" or anywhere else. It won't be restored until Slice 3. If this PR merges, every user who navigates the sidebar will find no path to the concurrency page. That's a regression. Slice 3 is unscheduled.

Fix: either keep `language/concurrency` in "Learn Sifr" for now, or mark this PR as draft until the Concurrency group ships alongside it.

---

### HIGH-VALUE ISSUES (not blockers)

**3. `from-python.mdx` "Where To Go Next" sends the reader backward**

The sidebar places `from-python` after `quickstart`. The first card in the "Where To Go Next" section is Quickstart — pointing readers backwards in the learning path. Cards should point forward: Ownership and Mutability, Error Handling, Type System, or the Python Developers guide. The Quickstart card belongs on `introduction.mdx` or `index.mdx`, not `from-python`.

---

**4. Tab order is inverted from the plan**

Current: Documentation → **Reference** → Guides  
Issue spec (§9): Documentation → **Guides** → Reference

Guides is learning-oriented; Reference is catalog-shaped. A newcomer scanning the top nav will see Reference before Guides, which inverts the intended priority. This is a one-line reorder in `docs.json`.

---

**5. Package Management sits before Standard Library**

The issue's proposed sidebar shape (§9) puts Standard Library before Packages. A newcomer cares more about `sifr.collections`, `sifr.io`, and similar modules before they need manifest and publishing details. The current order reverses this.

---

**6. Audience guide sequence steps are unlinked (by design, but needs callout)**

Both `guides/python-developers/index.mdx` and `guides/rust-developers/index.mdx` list a numbered 5-step guide sequence as plain text, with no links. This is expected for Wave 1 placeholders, but readers who arrive here will see no clickable content after the initial cards. The PR description or issue should call this out explicitly so reviewers don't flag it as broken — and so Slice 8 has a clear marker to resolve it.

---

### CLEAN — no issues found

- **`ownership.mdx` rewrite**: excellent. The four-row convention table is the right anchor. `mut`/`own`/`own mut`/borrow-by-default are all covered. The Warning callout about borrowed values not escaping is exactly right. Scalar treatment is accurate. The "Across `await`" section is appropriately brief. Both audience cards at the bottom are correct. This is the strongest piece in the wave.
- **`type-system.mdx` Slice 0 fixes**: `int` no longer says "64-bit signed"; the scalar note uses "value semantics" not "Copy"; the `find_value` equality comparison on `int | None` is a valid pattern (equality doesn't require narrowing, only arithmetic/string operations do); `is_positive` correctly demonstrates the narrowing pattern.
- **`collections.mdx` Slice 0 fix**: the direct-indexing contract now correctly says `scores["missing"]` returns `None`, matching the architecture doc. The example is clean.
- **`status.mdx`**: concise and honest. The stdlib availability section correctly avoids implying undocumented modules are unsupported. "Practical Boundaries" works better than the issue's "Known limits" label.
- **`from-python.mdx` table and structure**: table-first, not duplicating Introduction prose, all prescribed rows present (including the "Missing values" row now that Slice 0 is done), accordion structure is appropriate. One small prose nit: the "Bytes" row says "Encode and decode at typed boundaries" which is correct but the `status.mdx`-level reader may benefit from the fuller note from the issue ("do not rely on platform-default text behavior").
- **`docs.json` foundation**: Documentation/Reference/Guides three-tab split is correct in principle; CLI and diagnostics correctly moved to Reference; Project group with Status at the bottom is correct; from-python in Get Started after quickstart matches the spec.
- **`docs/index.mdx`**: wayfinding is present — From Python card appears in both the hero and the "Get Started" section alongside Quickstart. Criterion met.
- **`guides/index.mdx`**: clean single-purpose wayfinding page, no concept creep.
- **Mint validate**: passed per the issue.

---

### Summary

Two blockers: missing import link (explicit acceptance criterion), and concurrency page orphaned from nav (content regression). Four high-value suggestions that are low-effort to address before merging. The ownership rewrite and the Slice 0 semantic corrections are solid and ready.

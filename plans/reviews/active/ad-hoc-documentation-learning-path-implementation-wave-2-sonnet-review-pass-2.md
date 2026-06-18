Now I have a complete picture. Here is the review summary:

---

## Wave 2 Review — Findings

### One confirmed blocker

**`docs/stdlib/overview.mdx` line 93** lists `join` and `race` as `sifr.task` APIs. The actual implementations in demos and the concurrency docs use `gather` and `select`. `join` and `race` do not exist anywhere in the codebase. This is a factual error in the stdlib overview that will mislead readers looking for the right function names.

The fix is a one-line correction: replace `join`, `race`, and `select` with `gather` and `select`.

---

### Confirmed non-issues (agent findings that don't hold up)

- **`task.gather` and `task.select` not in stdlib/concurrency.mdx** — these are intentionally documented in the language section (`docs/concurrency/structured-tasks.mdx`), which is the correct home for concept-level API explanations. `stdlib/concurrency.mdx` is the reference page and the `structured-tasks.mdx` page links to it. Not a gap.
- **`task.sleep` undocumented in stdlib** — `task.sleep` is straightforwardly defined and used in demos; it's an obvious API. Its absence from the stdlib reference page is fine.
- **`task.deadline` missing an example** — the text covers it descriptively alongside `task.timeout`; this is acceptable for a reference page.

---

### Verdict

**Wave 2 is not yet ready for PR.** There is one blocker: `docs/stdlib/overview.mdx` line 93 claims `task.join` and `task.race` exist — they do not. The correct names are `task.gather` and `task.select`. That single line needs to be corrected before opening the PR.

Now I'll write the review.

---

## Review: ad-hoc-documentation-learning-path (agent pass 2)

### Pass-1 resolution check

All eight pass-1 findings are closed:

- Dict-indexing framing in Slice 0 changed from "decide A or B" to "correct `collections.mdx` to match the established architecture contract." ✓
- `int` 64-bit and Copy-type sentences in `type-system.mdx` are now named as specific `Fix` directives, not investigation questions. ✓
- `int | None` None-Safety examples named as a specific fix target. ✓
- `sifr.parallel` added to the concurrency module list. ✓
- `logging`, `argparse`, `timeit` present in the module inventory. ✓
- `--explain` and `--diagnostic-format` global flags added to Slice 4 acceptance. ✓
- Internal `.md` files called out in Non-Goals. ✓
- Content Principles rules 2 and 10 rewritten to the suggested forms. ✓
- Bytes row in the `From Python` table is now behavioral ("encode/decode boundaries are explicit typed operations; do not rely on platform-default text behavior"). ✓

None of the pass-1 findings need revisiting.

---

### New findings

**[important] `From Python` table has a planning artifact embedded in the "Missing values" row**

The row as written:

```
| Missing values | Contract resolved in Slice 0 | Absence must be handled before use; the bridge page must cite the finalized indexing and optional-value docs |
```

The "Sifr shape" cell contains planning instruction, not docs content. If an implementer treats this table as copy-ready (which its table-first framing invites), the rendered page would publish "Contract resolved in Slice 0" as visible user-facing text. Since Slice 1 depends on Slice 0 being complete before drafting `From Python`, the correct fix is to omit this row from the table spec entirely and instead add a sentence like: "After Slice 0 resolves the `Option`/`None` indexing contract, add a 'Missing values' row citing the finalized language and stdlib pages." This makes the dependency explicit without risking garbage in the rendered output.

**[important] Async is absent from the `From Python` table**

The issue calls out async as having "one canonical public story" that diverges meaningfully from Python's `asyncio`. A Python developer coming from async Python code will expect `async def` / `await` to work as they know it, but Sifr's model involves `sifr.task`, structured scopes, and typed cancellation evidence — all things that would surprise someone pasting async Python code. There are dedicated demos (`async_*`, `cancellation_cleanup_demo`, `blocking_offload_demo`) and a concurrency page in the sidebar. Yet no row exists in the compatibility table. The omission will make `From Python` feel incomplete to any Python developer who uses async code.

Suggested row:

```
| async / await | async def, await, sifr.task | Structured task scopes; cancellation is typed evidence, not an exception |
```

---

### Copywriting and style assessment

The ten Content Principles are well-calibrated for this kind of doc site. Rules 9 and 10 are essentially the same rule stated in two halves (avoid spec language in learning pages; send that detail to reference pages). They read better merged: "Move spec-level detail to diagnostics, CLI references, or architecture docs — learning pages link rather than inline." Losing a rule strengthens the list, but this is a minor polish note.

The proposed sidebar hierarchy (Gap 7) is clean and reads as a learning path first, reference catalog second. One coherence gap: after Slice 4 completes the CLI Reference expansion, the sidebar will have significantly more entries under "CLI Reference" than are shown in the current plan. The sidebar shape in the issue shows only six entries under CLI Reference; Slice 4 will add `init`, `fetch/tree/vendor`, `repair`, `self`, and `trace`. The issue should either update the sidebar shape to reflect the post-Slice-4 state or add a note so future readers understand the plan is incremental.

The overall writing voice across the plan — "state differences as product decisions, not caveats," "written as a compatibility orientation, not a warning dump," "no apology language" — is exactly right for world-class developer docs. The risk is implementation drift, not planning intent. The principles are clear enough to hold that standard if followed.

---

### Module index and CLI inventory dump risk

**Module index (Slice 3):** The grouped-by-purpose structure plus "available vs. planned" status labeling is a real anti-dump mechanism. One gap remains: the plan doesn't say what to do with modules that are small or self-explanatory and may never need a dedicated page. Marking them "planned" indefinitely signals incompleteness; listing them with no status implies work is coming. A third category — "available, no dedicated page needed" or "see Python docs for surface, Sifr semantics are identical" — would let the index be truthful about coverage without creating perpetual "planned" debt. This belongs in the Slice 3 acceptance criteria.

**CLI inventory (Slice 4):** Well-scoped. The acceptance criteria prevent prose duplication across package pages. The one missing piece: the overview should group commands (project lifecycle vs. code quality vs. package management vs. self-management) rather than present a flat alphabetical list. A flat list of 15+ commands is a dump regardless of how well each entry is written. One sentence in the acceptance criteria ("Group commands by workflow concern, not alphabetically") would close this.

---

### Slice order evaluation

After adding Slice 3 (stdlib module index) and Slice 4 (CLI inventory), the sequence is:

0 (audit) → 1 (skeleton) → 2 (data model) → 3 (module index) → 4 (CLI inventory) → 5 (Python callouts) → 6 (guides)

This is pragmatic. Slices 5 (callouts on existing pages) does not strictly depend on Slices 3 or 4, but Slice 1's `status.mdx` needs a meaningful stdlib overview to link to, which makes Slice 3 land before Slice 5 a reasonable default. The real observation: Slices 3 and 4 are independent of each other and can be parallelized once Slice 2 is complete. If two contributors are working simultaneously, the issue should note this to avoid artificial sequencing. Not a blocking concern, but it affects delivery velocity.

---

### Go / no-go

**Go**, with two targeted edits before Slice 1 drafting starts:

1. **[important]** Remove the "Missing values" row from the `From Python` table spec and replace it with an explicit note to add the row after Slice 0 resolves the contract.
2. **[important]** Add an async row to the `From Python` table spec.

Three optional improvements worth picking up during slice work:

- **[optional]** Slice 3 acceptance: add a third status category for modules that are available but do not need a dedicated doc page.
- **[optional]** Slice 4 acceptance: add "group commands by workflow concern" to prevent a flat-list dump in `cli/overview.mdx`.
- **[optional]** Note in Slice 2 or Slice 3 that Slices 3 and 4 are independently executable in parallel.

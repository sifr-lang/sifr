---

## Wave 4 Review

**PR-ready: Yes, with the cleanup items below addressed first.**

---

### Blockers

**B1 — Stale duplicate file: `docs/errors/SIFR-IMPORT-0008.md`**

Wave 4 added `SIFR-IMPORT-0008.mdx` (Jun 18) but left the older `.md` (Jun 17) in place. The build validator passed, but the duplicate is a cleanup hazard — Mintlify conventions favor `.mdx` and the stale `.md` will confuse anyone editing the errors directory. Remove `SIFR-IMPORT-0008.md` before merging. (This is the only error page with both extensions; all others have just `.md`.)

---

### Important improvements

**I1 — Undefined `values` in `rust-concepts.mdx:54`**

Under "Option And Result Stay Visible":
```python
first: int | None = values[0]
```
`values` is never declared. For a guide aimed at Rust developers already skeptical of the Python surface, a dangling identifier is an immediate trust hit. One context line fixes it:
```python
values: list[int] = [10, 20, 30]
first: int | None = values[0]
```

**I2 — `guides/index.mdx` has duplicate path cards (lines 10–46)**

The top `<CardGroup>` already shows all three entry points (quickstart + both guide paths). The `## Paths` section immediately below repeats the Python and Rust guide cards verbatim. Cut the `## Paths` section entirely, or replace it with something the top group doesn't cover (e.g., a "New to compiled languages?" card pointing to the quickstart rationale). As written it reads like a copy-paste accident.

---

### Optional polish

**P1 — `from-python.mdx:83` card title/target mismatch**

The card is titled "Python Developer Guide" but `href="/guides/python-developers/mental-model"` bypasses the guide overview and links directly to the sub-page. Either rename it "Mental Model Shift" (which is how the guide's own index titles it) or point it at `/guides/python-developers`.

**P2 — `mental-model.mdx:82-83` — `own mut` description has a trailing qualifier that confuses**

> "Use `own mut` when it consumes and mutates the value before returning or dropping it."

"before returning or dropping it" is awkward — it implies mutation after return is somehow intended, or that dropping is a notable event the reader should think about. The other docs handle this more cleanly. Simplify to: "Use `own mut` when it consumes and mutates the value."

**P3 — `guides/rust-developers/rust-concepts.mdx:98` — `task.gather` return type assertion**

```python
values: list[int] = await task.gather([first, second])
```
`first` and `second` are handles from `load_one()` and `load_two()`, which aren't defined. The `list[int]` annotation is asserted without basis. Not wrong if you assume `int`-returning tasks, but readers may wonder whether `gather` always returns `list[int]`. Adding a comment or changing to `list[str]` with correspondingly named `load_one/load_two` bodies would prevent the question. Low priority since this is a structural illustration.

---

### Answers to the five inspection questions

1. **Useful and concise for their audiences?** Yes. `mental-model.mdx` nails the Python-developer orientation: the comparison table, the per-section "here's the single thing that changed" structure, and the Info callout linking to Values & Collections on the `KeyError` surprise are all well-targeted. `rust-concepts.mdx` is equally tight — the concept map table is genuinely the first thing a Rust developer needs and it delivers that immediately.

2. **Code examples accurate?** Broadly yes. The `Result`/`raise`/`try`/`except` examples, ownership annotations (`mut`, `own`), and imports (`sifr.math`, `sifr.json`) all match the error-handling and values docs. The one exception is the undefined `values[0]` snippet (I1 above).

3. **Cross-links smooth and not spammy?** Yes. Wayfinding cards appear only at page endings or within `<Info>`/`<Tip>` callouts addressing a specific concept. The language pages (type-system, error-handling, values-and-collections) each add exactly two guide links at the bottom — proportionate for pages that will be read by both audiences.

4. **Sidebar shape correct?** Yes. The docs.json groups now read "Python Developer Guide" / "Rust Developer Guide" with the index page first and the substantive page second in each group. Clean and matches the user's intent.

5. **Copywriting quality?** High. The opening lines of both guide pages ("Python knowledge helps you read Sifr quickly. The main adjustment is not syntax. It is where failure is allowed to live." and "Use Rust as the safety model, not as the syntax model.") are the best-quality prose in the docs so far — clear orientation with no throat-clearing. The note in `rust-developers/index.mdx` ("Sifr is not a Rust macro language. The generated Rust is an implementation target...") preempts the most common Rust-developer misconception precisely. No changes needed to voice or tone.

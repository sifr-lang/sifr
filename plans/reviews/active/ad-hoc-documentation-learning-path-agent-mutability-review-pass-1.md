Here is a direct assessment of the plan section:

---

## Does `mut` belong on the same page?

Yes, unambiguously. The gap is not that `mut` gets less coverage than `own` — `mut` is entirely absent from the current page. Python readers will assume parameter mutation and rebinding work as in Python. Rust readers need the full four-row picture to map `&T / &mut T / T / mut T` onto Sifr syntax. Combining them on one page is correct.

---

## What the plan gets right

- **The four-row table** is accurate and matches architecture.md lines 430-433 exactly. This is the right anchor for the page.
- **Source material choices** are good: `own_mut_appends/main.sifr` is minimal and clear. The lowering tests in `own_mut_semantics_tests.rs` cover exactly the right failure cases to draw examples from.
- **"Formatter rewrites `mut own` to `own mut`"** — this is genuinely non-obvious and easy to miss. Worth a one-line tip.
- **"Across await" as a cross-link only** — correct. The async borrow rule is complex; this page should not absorb it.
- **"Avoid becoming a Rust lowering spec"** — the right guardrail. The existing page's style (short paragraph, code, callout) is a good template.

---

## What's missing or wrong

**1. Scalar vs heap asymmetry is unmentioned**

Architecture.md line 434 is explicit:

> `mut` on scalar parameters (`int`, `float`, `bool`) only affects local rebinding/mutation semantics, not observable ownership transfer.

A Python developer writing `def f(mut n: int): n = n + 1` would expect this to work and it does — but for a completely different reason than `mut items: list[int]` (which is a mutable borrow). This asymmetry is likely to confuse both audiences and should be a short callout. The plan currently omits it entirely.

**2. "Borrowed `mut` cannot escape" is the real footgun, not a secondary note**

The tests (`test_mut_borrow_parameter_cannot_escape_via_return`, `test_mut_borrow_parameter_cannot_escape_via_local_binding`) show that `mut items: list[int]` cannot be returned or stored — you get `OWN_BORROWED_PARAMETER_ESCAPES`. This is the constraint that most surprises people: you can mutate through the borrow, but you cannot keep the value. The plan buries this under "Reassignment vs mutation" as if it's the same concept. It isn't. It should be a distinct named rule: **borrowed parameters cannot escape**.

**3. The 7 content areas are not differentiated by weight**

The plan lists these as parallel sections: immutable default, `mut` borrows, `own mut`, reassignment vs mutation, formatter convention, bytes immutability, across await. That's too flat — on a concise page, they cannot all be `##` headings without making the page feel like a spec. The plan should specify which are sections and which are callouts:

- Sections: borrow by default (already exists), `mut` for mutable borrows, `own` and `own mut` for ownership
- Tips/callouts: formatter canonicalization, bytes caveat, await cross-link, scalar difference

**4. "Bytes are immutable" is a marginal addition**

This is true but niche for a page about parameter conventions. `mut` doesn't make `bytes` subscript assignment legal — correct. But this is a bytes semantics rule, not an ownership rule. If it appears at all it should be a one-sentence `<Note>` inside the `mut` section, not a named content area. The plan over-specifies it.

---

## Acceptance criteria: what's solid and what needs sharpening

**Solid:**
- Retitle frontmatter and sidebar while keeping the file path — correct, clean.
- Teach the four conventions together with a compact table — right.
- Avoid becoming a Rust lowering spec — good constraint.
- Link Python and Rust readers to guide tracks — appropriate and well-placed.

**Needs sharpening:**
- The criterion "The page explains that bare parameters are immutable by default, `mut` permits mutation through a borrow, `own` moves the value, and `own mut` moves the value while permitting local mutation" is accurate but omits the escape constraint. Add: **the page explains that `mut` borrows cannot be returned or stored without `.clone()`**.
- Add: **the page notes that `mut` on scalar types permits local rebinding but does not involve a borrow or move**.

---

## Linking guidance: no issues

The `<CardGroup cols={2}>` pattern shown in the plan is correct. Two cards at the end of the page — one for Python readers, one for Rust readers — is the right weight. Don't add them inline; end-of-page is correct.

---

## Summary

The plan is structurally sound. The table, source material, and elegance constraint are right. Three actionable fixes before writing:

1. Add a callout distinguishing `mut` on scalars from `mut` on heap types.
2. Elevate "borrowed `mut` cannot escape by return or store" to a named rule — it's the primary footgun.
3. Flatten the 7 content areas into 3 sections + 4 callouts to prevent the page from reading like a spec.

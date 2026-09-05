Here is the targeted assessment against each previously flagged issue:

---

## Scalar `mut` behavior

**Resolved.** Gap 6 (line 415) now names it explicitly as a callout: "`mut n: int`, `mut x: float`, and `mut ok: bool` permit local rebinding, but do not represent an observable heap borrow or ownership transfer." Slice 1 acceptance criterion line 650 mirrors this exactly. No gap remains.

---

## Borrowed `mut` cannot escape

**Resolved as a named rule.** Line 413 now calls it out by name and specifies the fix paths (`.clone()`, `own`, `own mut`). Acceptance criterion line 649 adds the implementation obligation explicitly. The review's concern about it being buried under "Reassignment vs mutation" is addressed — they are now listed separately.

One high-value refinement remains: the review called this "the primary footgun" suggesting it should be a `<Warning>` callout, not just a plain note. The plan's callout list does not specify callout type. Worth adding `<Warning>` for this one specifically, per the Content Principles (line 368: `<Warning>` is for "real surprises or unsafe assumptions"). Not a blocker, but it would be easy to write this as a `<Note>` during implementation and underemphasize it.

---

## Section/callout weight

**Substantially resolved.** The plan now has a clear two-tier structure:
- 3 named sections (lines 406–409): immutable default, `mut` mutable borrows, `own mut`
- "Rules and callouts to include without turning them into large sections" (lines 411–418): 6 items

The review prescribed 3 sections + 4 callouts. The plan has 6 callouts. The two extras are "borrowed values cannot escape" and "reassignment vs mutation." Both are defensible additions, but "reassignment vs mutation" (line 414) is the vaguest entry — it says "reassigning a parameter and mutating the object behind it are both explicit choices" without specifying the callout's actual claim. This could collapse back into the "borrowed cannot escape" concept during writing. Worth either absorbing it into the `mut`-borrow section body or sharpening it to a concrete rule. Low-risk but worth addressing before writing starts.

---

## Bytes caveat

**Resolved.** Line 417 now explicitly says "Keep this as a one-sentence note, not a full section." The review's concern (it was over-specified as a content area) is directly addressed.

---

## Does `mut` belong on the same page?

**Resolved.** The review confirmed unambiguously yes. The plan retitles frontmatter and sidebar, teaches all four conventions via the table, and links to guide tracks from end-of-page cards. No issues here.

---

## Summary

**No blockers.** Two high-value refinements before writing starts:

1. **Specify `<Warning>` for "borrowed `mut` cannot escape"** — the review elevated this to "primary footgun"; the plan's callout list doesn't specify the callout type, leaving it likely to be written as a `<Note>` and underemphasized.

2. **Sharpen or absorb "Reassignment vs mutation"** — as written (line 414) it's the vaguest callout in the list and risks either duplicating or muddying the "borrowed cannot escape" rule. Either give it a concrete claim ("reassigning `mut items` does not prevent the borrow-escape rule from applying") or fold it into the `mut`-borrow section body.

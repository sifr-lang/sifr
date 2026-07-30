## Verification

**Requested change (pass-12 finding 1) — fully resolved.**
`plans/reviews/active/…wave-3-claude-opus-review-pass-11.md:3` now reads `# Wave 3 Review — pass 11 (exact published head ec5aab945 vs base ea119724e)`. The heading matches the filename and the ledger link. The pass-11 diff at this commit is exactly two changed lines (the heading pair) — no other content in the preserved report was touched, so the artifact remains the verbatim pass-11 record. No remaining `pass 12` self-reference exists inside the pass-11 report (`git grep "pass 12"` at head hits only the pass-12 report itself, the ledger paragraph, and unrelated issues).

**Delta scope.** `git diff --name-only ec5aab945..7021ba076 -- ':!*.md'` returns zero files — the compiler tree is byte-identical to the state pass 11 approved and that passed create-pr with 131/131 selected e2e, so no code re-validation is warranted and I did not re-run the suites. `git diff --check ea119724e..7021ba076` is clean. Full-PR scope remains `crates/**` + `plans/**` only.

**Pass-12 report added unchanged.** `plans/reviews/active/…wave-3-claude-opus-review-pass-12.md` (+25) matches the pass-12 output as described, including its `CHANGES REQUESTED` verdict — correct to preserve verbatim as a point-in-time artifact rather than retro-edit it after the fix.

**Ledger paragraph accuracy** (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:325-328`). All three claims check out: pass 12 did verify a documentation-only head delta; it did raise exactly one actionable finding (the self-label); and the heading now does match filename and link. The relative link `../../reviews/active/…pass-12.md` resolves to a file present in the same commit (`git ls-tree` confirms both pass-11 and pass-12 blobs are tracked at head) — no dangling reference of the kind earlier passes on this issue flagged as a standing rule.

## Actionable findings

None.

## Non-blocking observations

1. **Pass-12 report heading doesn't self-label its pass number** — `…pass-12.md:1` is `## Wave 3 — exact-head review (published head 39f947cac vs base ea119724e)`: an H2 rather than the siblings' H1, and without an explicit "pass 12". Mildly ironic given the finding it raised, though not the same defect — it asserts no wrong pass number, and filename plus ledger identify it unambiguously. Worth aligning if the review-artifact template is ever formalized.
2. **Pass-12's own non-blocking items 1 and 2 remain open by design** — the two stale `state_collection.rs:650-660` / `:662-664` citations inside the pass-11 report, and the ledger's 8 → 10 pass gap with no pass-9 clause. Leaving the report's stale offsets untouched is the right call (it's a historical artifact); the pass-9 clause is a one-line addition available whenever the ledger is next edited.

APPROVED

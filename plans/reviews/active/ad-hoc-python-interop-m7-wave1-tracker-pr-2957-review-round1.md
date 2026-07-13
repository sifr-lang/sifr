Verified the review points:

- **PR #2956 is merged:** state `MERGED`, `mergedAt 2026-07-13T03:04:33Z`, mergeCommit `30247ee4`. The linked URL `https://github.com/sifr-lang/sifr/pull/2956` resolves to the exact PR whose contract-scoping title matches Wave 1's text.
- **Diff scope is exactly two lines** in `plans/issues/active/ad-hoc-declaration-first-python-interop.md`: flips the first wave checkbox from `[ ]` to `[x]` and appends `— [PR #2956](...)`. No other tracker mutations.
- **No premature M7 completion claims:** the top-level milestone checkbox `- [ ] M7 owned asyncio runtime and async declarations` at line 140 remains unchecked. The final wave (line 572, "Atomically activate async declarations and close M7 evidence") also remains `[ ]`. Only the first of six M7 waves is marked done, matching the PR title "Record M7 frontend wave completion" and body "mark the first M7 implementation wave complete."
- **Wave text matches PR #2956's scope:** frontend coroutine/async-close contracts behind the existing `SIFR-PYRES-0002` gate — no runtime/loop/wrapper claims leak into this wave.

No actionable findings.

SATISFIED

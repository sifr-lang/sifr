# M10 Wave 2 whole-diff review — pass 8

Reviewer: agent, high reasoning, fast service tier, read-only whole-diff review.

## Verdict

**CHANGES REQUIRED. Not satisfied; M10 Wave 2 is not ready to merge.**

## Blockers

1. **High — membership bypasses affine equality capability checks.** `lower_compare` validates only assignability for `in` and `not in`, while generated Rust calls `.contains(...)` and therefore requires `PartialEq`. A read-only probe accepted `view in values` for `python.Buffer` and emitted Rust that cannot compile.
2. **High — chained assignment and unpacking violate affine ownership or introduce hidden clones.** Chained assignment did not consume the original and intermediate bindings. Tuple unpacking did not transfer ownership from its source. Star unpacking cloned elements and used `to_vec()` even when the element type contained an affine buffer.
3. **High — affine match subjects reach a user-triggerable compiler panic and untracked moves.** Affine subjects were accepted without branch-sensitive ownership, and read-only buffer match probes reached the CFG invariant panic and surfaced `SIFR-INTERNAL-0001`.
4. **High — footprint admission is not production-scale.** The direct footprint path allocated, sorted, and duplicated one address range per logical element while holding the Python attachment, so a large ordinary contiguous buffer could consume gigabytes merely for admission.
5. **Medium — indirect admission is not the claimed exact logical-item admission.** All indirect views collapsed to one global-conflict marker, rejecting unrelated or physically disjoint writable views.
6. **Medium — activation evidence overstates closure.** The capability ledger and phase tracker claimed complete recursive ownership/equality and exact admission evidence without permanent coverage for the cases above.

## Required closure

Reject or soundly implement every affine compiler path above, add permanent negative/native evidence, introduce compressed contiguous footprint admission and safe exact indirect overlap tracking, correct the activation documentation, rerun affected and authoritative validation, and submit the entire Wave 2 diff to another independent whole-diff review.

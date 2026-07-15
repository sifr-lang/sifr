# M10 Wave 2 whole-diff review — pass 10

Reviewer: Codex CLI `gpt-5.6-sol`, high reasoning, fast service tier, ephemeral read-only whole-diff review.

## Verdict

**CHANGES REQUIRED. Not satisfied; M10 Wave 2 is not ready to merge.**

## Blockers

1. **High — recursive union equality still accepts code that cannot compile.**
   `supports_structural_equality` accepts a union when each member does, but
   generated union enums do not derive `PartialEq`. An `int | str` equality
   probe passed checking and emitted invalid Rust.
2. **High — set/dict operations confuse `PartialEq` with Rust `Eq + Hash`.**
   Set membership and dictionary equality accept key shapes such as `float` or
   `list[int]` that cannot satisfy the generated `HashSet`/`HashMap` operations.
3. **High — chained assignment remains unsound for non-Clone move values.**
   A callable-bearing class accepted `a = b = h` and later reuse of `h`, then
   emitted multiple moves from the same value.
4. **High — union generation unconditionally requires `Debug`.** Callable-bearing
   class members can enter a union even though the generated enum derives
   `Debug` and its `Display` implementation uses debug formatting on members
   that do not implement it.
5. **Medium — activation evidence remains inaccurate.** The phase, architecture,
   and capability ledger overstate Rust-trait equality and membership closure
   while the accepted-invalid cases remain.

## Required closure

Model and enforce distinct recursive equality, hash-key, and formatting
capabilities; generate only valid union derives/formatting implementations;
close non-Clone chained assignment; add permanent native evidence; correct the
activation claims; rerun validation; and submit another whole-diff review.

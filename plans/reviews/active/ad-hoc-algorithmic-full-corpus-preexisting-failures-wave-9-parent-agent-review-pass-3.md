## Verification

**Head/base/publication state**
- Current head: `32e69a59d7a769ec9b7a82a1fd70da4d5b39be3c`; merge-base with `origin/main` = `44ab8ad38544fa5225d8d4f09ad3b5026d485c25` (= `origin/main` tip), so the stated base is exact.
- PR #3091 `Preserve nested captured-container refinement`: **OPEN, isDraft: true**, `codex/algorithmic-full-corpus-closeout` → `main`, `headRefOid` = `32e69a59d`, MERGEABLE. PR file list matches the local 10-file diff exactly.

**Delta claim confirmed.** `git diff 22111f3f0..32e69a59d` = 2 files, +40/−1: the new pass-2 report (39 lines) and a single rewritten Wave 9 ledger row. Zero source, test, fixture, or gitlink change since the pass-2-approved tree; corpus gitlink still `9d715953`. Working tree matches the pre-existing snapshot (dirty submodule markers, untracked closeout demo) — nothing new but this pass's own report placeholder.

**Ledger now matches evidence.** Row 340 reads "complete lowering passes 944 with one ignored"; I re-ran `cargo test -p sifr_lowering --release` at head: **944 passed; 0 failed; 1 ignored**. Phrasing also matches the Wave 5 precedent. The added pass-2 sentence accurately summarizes that report (verdict, scope, sole finding). Spot-checked one more pass-2 claim: `statement_dispatch.rs` is 833 lines.

**Remaining findings from the complete PR:** none in compiler code, tests, scope, or the ledger. One inaccuracy survives in the PR description.

## Actionable finding

**LOW — stale validation figure in PR body (GitHub PR #3091 description, "Validation" list).** It still reads `full lowering: 941/941` — the identical error pass 2 flagged and that was fixed in the ledger but not in the PR body. 941 is the base count; head is 944 passed / 1 ignored. This is the reviewer- and merge-facing summary, so it misreports the suite the PR grows by three tests. (The adjacent `967/967` codegen and `687/687` + `d61c30dde1d7fc1c` e2e figures are correct.) Fix: change to `full lowering: 944 passed, 1 ignored`. No file change required — PR metadata only, and I made no edits.

Non-blocking notes: the PR is still a **draft** while the ledger says "in review", and the body's `authoritative create-PR facade: pending exact-head run` is still open — the exact-head `scripts/run_all_tests.sh --profile create-pr` gate that prior waves recorded before merge has not been run at `32e69a59d`.

**Verdict: not a zero-finding approval.** The implementation and ledger at `32e69a59d7a769ec9b7a82a1fd70da4d5b39be3c` are approved as-is; approval of the PR is conditional on correcting the one `941/941` figure in the PR description (and, per wave precedent, undrafting plus the exact-head create-PR profile before merge).

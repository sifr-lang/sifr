# Review Round 6 — `hardening_1`: Execute the Rust-Interop Area in Authoritative Profiles

**Scope reviewed:** the exact current tree — committed `HEAD` `d41c52ed2` versus `origin/main`
`44d8f7160`, plus the uncommitted documentation-only notices on review rounds 1–3 and the
untracked round-4 and round-5 artifacts. Per instruction the authoritative lane was **not**
rerun; round 4's `LANE_EXIT=0` create-PR run and its implementation verdict are carried as
evidence. No implementation file and no prior review file was modified during this review.

**Verdict: APPROVED** — no actionable findings remain. Round 5's Finding 1 is cured by
correctly time-scoped wording, rounds 1–3 notices still cure round 4's Finding 1, no
implementation file changed after the passing round-4 lane, and rounds 4 and 5 are present.

---

## 1. Round 5 Finding 1 — cured

Round 5 objected to the round-3 notice asserting "**The final** implementation diff has 15
files", a count the act of committing rounds 4–5 would falsify (15 → 17).

Current text (`rust-interop-hardening-1-review-round3.md`, lines 6–7):

> The implementation diff reviewed **in round 4 had 15 files, before later process-only
> review artifacts**, and contains none of the diagnostics MDX, Ruff revalidation, or
> token-fixture paths discussed in §§8–10 …

This satisfies both required properties:

| Requirement | Delivered |
|---|---|
| Time-scope the 15-file count | Yes — "reviewed in round 4 **had** 15 files": past tense, anchored to a named round rather than to the PR's eventual final state. Matches round 4 §0, which measured exactly 15 at `d41c52ed2` |
| Make clear later review artifacts are process-only additions | Yes — "before later **process-only** review artifacts": names the later artifacts, classifies them as process, and scopes the count to *implementation* diff, so accruing rounds 4–6 cannot falsify it |
| "final" no longer attached to the count | Yes — the word survives only in "authoritative final verdict" (about the approval round) and in rounds 1–2's "not part of the final `hardening_1` diff" (about the upstream repairs being absent, not a count). Neither goes stale as artifacts accrue |

The statement is true in both possible end states: with rounds 4–6 uncommitted (15 files) and
with them committed (18 files, of which 15 are the round-4 implementation scope). Round 5's
self-referential instability is gone.

## 2. Rounds 1–3 notices still cure round 4 Finding 1

Re-verified against round 4's five requirements; nothing was weakened by the round-5 fix.

| Round-4 requirement | Present | Where |
|---|---|---|
| Rebase base named | "rebased onto `origin/main` at `44d8f7160` (PR #3017)" | round 3 notice |
| MDX + Ruff + token fixtures upstream; §§8–10 void | "contains none of the diagnostics MDX, Ruff revalidation, or token-fixture paths discussed in §§8–10" | round 3 notice |
| The harmful PR-description instruction neutralised | "those repairs are upstream and must not be claimed in this PR's description" — directly contradicts §10's closing paragraph and §12.2 | round 3 notice |
| §12.1 resolved | "The fixture `Cargo.lock` is tracked, so §12.1 is complete, and §12.2 is superseded" | round 3 notice |
| One-line pointer in rounds 1 and 2 | Both carry the same three-point notice (repairs upstream, lockfile tracked, see round 4 / the approval round) | rounds 1–2 notices |

Placement is unchanged and unmissable: round 1's notice is prepended above its
"**Not approved**" line; rounds 2 and 3 sit immediately under the H1, before any body text.
All three are blockquoted, so no reader reaches a stale claim without first passing the
correction. Every factual assertion in the notices is confirmed by §3 below.

## 3. Facts underlying the notices — re-verified

| Check | Result |
|---|---|
| `git rev-parse HEAD` | `d41c52ed2` — unchanged since round 4 |
| `git rev-parse origin/main` == `git merge-base HEAD origin/main` | both `44d8f7160`; branch is a single commit on top of #3017 |
| `git log --oneline origin/main..HEAD` | one commit, "Run Rust interop checks in authoritative profiles" |
| `git diff --name-only origin/main...HEAD \| wc -l` | **15** — identical to round 4 §0's list |
| `crates/**` and `third_party/**` paths in the diff | **0** |
| Diagnostics MDX / `gen-error-docs.rs` / canonicalization check / `code_coverage.py` / `ruff_fork_revalidation.json` / `sifr_syntax_token_fixtures/*` | absent here; all present in `origin/main` via #3017 (`git show --name-only origin/main` lists them) |
| Fixture `Cargo.lock` tracked | yes — `git ls-files --error-unmatch` resolves it |
| `.gitignore` negation effective | `git check-ignore -v <fixture Cargo.lock>` exits 1 (not ignored) |

## 4. No implementation change after the passing round-4 lane

- `HEAD` is still `d41c52ed2`; no commit was added on top of the tree round 4 ran the lane on.
- `git status --porcelain --untracked-files=all` contains **only** `plans/reviews/active/`
  entries: ` M` rounds 1–3, `??` rounds 4 and 5. Nothing under `verification/`, `crates/`,
  `scripts/`, or `plans/issues/`.
- Working-tree `git diff --numstat`: 3 files, **+22 / −0** (6 / 6 / 10), all Markdown prose
  insertions at the top of rounds 1–3. No deletion, so the historical record is preserved
  rather than rewritten; the round-5 cure was applied by rewording within the added notice,
  not by editing pre-existing round-3 text.
- Therefore round 4's `LANE_EXIT=0`, 22/22 `status=pass` create-PR run — including
  `[sifr-lane-step] name=rust_interop_checks elapsed_ms=402 status=pass` against the blocking
  5000 ms budget — applies unmodified to this tree. `*.md` is outside the file-size guardrail
  and outside every lane input, so no rerun is warranted.

## 5. Rounds 4 and 5 are present

Both exist and are substantive, not placeholders:
`rust-interop-hardening-1-review-round4.md` (349 lines: rebase resolution, requirement table,
exit-gate table, fail-closed analysis, lane transcript, Finding 1, non-blocking observations)
and `rust-interop-hardening-1-review-round5.md` (156 lines: no-implementation-change proof,
scope reconfirmation, Finding 1 cure mapping, Finding 1 on the "final" wording). Both are
untracked (`??`), consistent with the notices' forward references being satisfied at commit
time.

---

## Actionable findings

**None.**

---

## Publish preconditions (not findings — no content change requested)

- Commit the three amended rounds 1–3 together with rounds 4, 5, and this round 6, so the
  cure and the artifacts its forward references point to ship in the same PR. Leaving them
  uncommitted would strand the notices and dangle the "round 4 and the later approval round"
  pointer.
- The PR description must describe only the `hardening_1` scope (the nine `verification/`
  files plus `.gitignore` and two plan-doc command repairs) and must **not** claim the three
  baseline repairs that #3017 already landed upstream.
- `hardening_1` closeout in `plans/issues/active/rust-interop-verification-matrix-hardening.md`
  is owned by `hardening_5` and cannot record the PR link pre-merge.

## Carried non-blocking observations (unchanged, un-actioned by design)

`selected_areas` prepend-vs-append ordering; per-selection vs unioned suite validation;
`summary` self-consistency in `validate_rust_interop_result`; evidence depth deferred to
`hardening_3`; `profile_runner.py` at 840/900 lines; the step-registry drift guard; the
missing `rust_interop` row in `profile_assignment_matrix.json`; asymmetric mandatory-area
rules; the broadened `.gitignore` negation's forward cost for `certification_11`.

---

## Commands run for this review

```
git rev-parse HEAD origin/main; git merge-base HEAD origin/main
git log --oneline origin/main..HEAD                          # single commit d41c52ed2
git diff --name-only origin/main...HEAD | wc -l              # 15
git diff --name-only origin/main...HEAD -- crates/ third_party/   # 0
git status --porcelain --untracked-files=all                 # only plans/reviews/active/
git diff --numstat                                           # +22/-0, three notices, prose only
git ls-files --error-unmatch <fixture Cargo.lock>            # tracked
git check-ignore -v <fixture Cargo.lock>                     # exit 1 (not ignored)
git show --name-only --format= origin/main | grep -Ei 'mdx|gen-error-docs|canonicalization|code_coverage|ruff_fork_revalidation|token_fixtures'
read rounds 1-6 notices/bodies                               # wording verification
```

The authoritative lane was not rerun. No implementation file and no prior review file was
modified.

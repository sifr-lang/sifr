# Review Round 5 — `hardening_1`: Execute the Rust-Interop Area in Authoritative Profiles

**Scope reviewed:** the exact current tree — committed `HEAD` `d41c52ed2` versus `origin/main`
`44d8f7160`, plus the uncommitted documentation-only additions to review rounds 1–3 and the
untracked round-4 artifact. Round 4's implementation verdict and its full rebased create-PR
lane run are taken as evidence (not rerun, per instruction); everything below is a targeted
read-only check.

**Verdict: NOT APPROVED** — one actionable finding, severity **Low**, documentation-only and
fixable in one line. Round 4's Finding 1 is otherwise cured, no implementation file changed
after round 4, and the diff remains focused and truthful.

---

## 1. No implementation change after round 4

Confirmed, not inferred:

- `HEAD` is still `d41c52ed2` ("Run Rust interop checks in authoritative profiles"); there is
  no new commit on top of the tree round 4 reviewed.
- `git status --porcelain --untracked-files=all`, filtered to everything outside
  `plans/reviews/`, is **empty**. The only working-tree deltas are ` M` on review rounds 1–3
  and `??` on round 4.
- `git diff --stat` on the working tree: 3 files, **+21 / −0**, all inside
  `plans/reviews/active/`, all pure prose insertions at the top of each file. No line was
  deleted or reworded, so the historical record is preserved rather than rewritten.
- Prior review files were not modified beyond the added notices, and nothing under
  `verification/`, `crates/`, `scripts/`, or `plans/issues/` moved.

## 2. The committed diff is still exactly the round-4 scope

| Check | Result |
|---|---|
| `git merge-base HEAD origin/main` | `44d8f7160` — equals `origin/main`, so the branch is a single commit on top of #3017 |
| `git diff --name-only origin/main...HEAD \| wc -l` | 15 |
| `crates/**` paths in the diff | 0 |
| `third_party/ruff` gitlink diff | empty |
| Any diagnostics-MDX / `gen-error-docs` / canonicalization / `code_coverage.py` / `ruff_fork_revalidation.json` / `sifr_syntax_token_fixtures/*` path | absent from this diff; all present in `origin/main` via #3017 (`git show --name-only origin/main` lists every one of them) |
| Fixture `Cargo.lock` tracked | yes — `git ls-files --error-unmatch` resolves it, `git show --stat d41c52ed2` includes it at +14 |
| `.gitignore` negation effective | `git check-ignore -v <fixture Cargo.lock>` exits 1 (not ignored); the added line is `!/verification/areas/rust_interop/fixtures/**/Cargo.lock`, placed after `**/Cargo.lock` |

So the two factual premises of round 4's finding are independently re-confirmed: the baseline
repairs are upstream, and the lockfile is tracked in this commit.

## 3. Do the new notices cure round 4's Finding 1?

**Substantively yes.** Round 4 asked for (a) a post-rebase addendum at the top of round 3
naming the rebase base, (b) an explicit statement that the diagnostics `.mdx` repairs, the
Ruff revalidation record, and the five token fixtures are upstream and not in this PR so §8,
§9, §10 rows 4–5 and §12.2 no longer apply, (c) §12.1 marked resolved, (d) the reviewed diff
identified, and (e) a one-line pointer in rounds 1 and 2. Mapping the delivered text:

| Round-4 requirement | Delivered | Where |
|---|---|---|
| Rebase base named | Yes — "rebased onto `origin/main` at `44d8f7160` (PR #3017)" | round 3 notice |
| MDX + Ruff + token fixtures declared upstream, §§8–10 void | Yes — "contains none of the diagnostics MDX, Ruff revalidation, or token-fixture paths discussed in §§8–10" | round 3 notice |
| The harmful PR-description instruction neutralised | Yes — "those repairs are upstream and **must not be claimed in this PR's description**", which directly contradicts §10's closing paragraph and §12.2 | round 3 notice |
| §12.1 resolved | Yes — "The fixture `Cargo.lock` is tracked, so §12.1 is complete, and §12.2 is superseded" | round 3 notice |
| Reviewed diff identified | Yes — "15 files" (see Finding 1 for the wording problem) | round 3 notice |
| One-line pointer in rounds 1 and 2 | Yes — both carry the same three-point notice (repairs upstream, lockfile tracked, see round 4 / the approval round) | rounds 1–2 notices |

Placement is unmissable in all three: round 1 is prepended above its "**Not approved**" line;
rounds 2 and 3 sit immediately under the H1, before any body text. Each notice is
blockquoted, so no reader reaches a stale claim (round 1's "still-untracked `Cargo.lock`",
round 2's "Stage … `Cargo.lock`" and "Call that out in the PR description", round 3's §12.1
"It is still `??`") without first passing the correction. Every factual assertion inside the
notices is verified true by §2 above.

Two residual stale mentions I checked and judged non-issues: round 3 §11's "this PR touches
only `hardening_1` scope plus the unblocking repairs" and §13's last bullet (a
tokenize-and-compare follow-up referencing §9). The first is subsumed by the notice's blanket
"describes the historical pre-rebase working tree" plus the explicit 15-file scope; the second
is a forward-looking backlog suggestion whose value is unaffected by where the revalidation
record landed. Neither instructs a false claim.

## 4. Diff focus and truthfulness

Unchanged from round 4 and re-spot-checked: nine `verification/` files plus `.gitignore`
carry the mechanism; two plan docs fix a genuinely broken `-m sifr_verify --area rust_interop`
invocation; the fixture lockfile is load-bearing because this PR makes the `matrix` suite
blocking. Nothing narrows a scope, adds an exclusion, relaxes a threshold, introduces a
fallback, or skips a test. No `crates/**` change, so no clippy/fmt exposure. Round 4's
`LANE_EXIT=0` create-PR run applies unmodified to this tree, since the only subsequent deltas
are Markdown prose (`*.md` is outside the file-size guardrail and outside every lane input).

---

## Actionable findings

### Finding 1 — Low (documentation accuracy) — round 3's notice claims "**the final** … diff has 15 files", a count that the act of committing rounds 4 and 5 makes wrong

**Location:** `plans/reviews/active/rust-interop-hardening-1-review-round3.md`, post-rebase
notice: "The final implementation diff has 15 files and contains none of the diagnostics MDX,
Ruff revalidation, or token-fixture paths discussed in §§8–10."

**Rationale:** the notice is self-referentially unstable. It points readers to "round 4 and
the later approval round" as authoritative, which requires
`rust-interop-hardening-1-review-round4.md` and this round-5 artifact to be committed to the
PR. Once they are, `git diff --name-only origin/main...HEAD | wc -l` returns **17**, not 15,
and the diff will contain a review artifact asserting a file count that the same diff
falsifies. The alternative — leaving rounds 4 and 5 uncommitted — is worse: the notices
themselves would then be absent from the PR (they are currently uncommitted too), the
uncured rounds 1–3 would ship as-is, and the forward references would dangle. Either branch
leaves one small untruth, which is the same defect class round 4 declined to approve.

The word "final" is what breaks; every other claim in the notice is accurate and stays
accurate.

**Fix (docs only, one line, no implementation file touched):** in the round-3 notice, replace
the file-count clause with a formulation that does not go stale as review artifacts accrue —
e.g. "The final implementation scope is the nine `verification/` files plus `.gitignore` and
two plan docs (15 files as of round 4, plus the later review artifacts), and contains none of
the diagnostics MDX, Ruff revalidation, or token-fixture paths discussed in §§8–10." Then
`git add` the three amended rounds 1–3, `rust-interop-hardening-1-review-round4.md`, and this
file, and amend or add a commit so the cure and the artifacts it references ship together.
Round 4's own §0 file list needs no change — it is explicitly scoped to "the exact committed
diff `HEAD` … after the rebase" and was correct at that commit.

---

## Non-blocking notes

- **Publish preconditions carried forward, none of them findings:** the PR description must
  describe only the `hardening_1` scope and must not claim the three upstream baseline
  repairs; `hardening_1` closeout in
  `plans/issues/active/rust-interop-verification-matrix-hardening.md` is owned by
  `hardening_5` and cannot record the PR link pre-merge.
- All of round 4's non-blocking observations stand unchanged and un-actioned by design:
  `selected_areas` prepend-vs-append ordering, per-selection vs unioned suite validation,
  `summary` self-consistency, evidence depth deferred to `hardening_3`, `profile_runner.py` at
  840/900 lines, the step-registry drift guard, the missing `rust_interop` row in
  `profile_assignment_matrix.json`, asymmetric mandatory-area rules, and the broadened
  `.gitignore` negation's forward cost for `certification_11`.
- Once Finding 1 is applied there is nothing else outstanding: the implementation is correct
  and fail-closed on round 4's evidence, and no re-run of the lane is warranted for a
  Markdown-only change.

---

## Commands run for this review

```
git status --porcelain --untracked-files=all      # only rounds 1-3 modified, round 4 untracked
git diff --stat; git diff plans/reviews/active/   # +21/-0, prose-only, three notices
git log --oneline origin/main..HEAD               # single commit d41c52ed2
git merge-base HEAD origin/main; git rev-parse origin/main          # both 44d8f7160
git diff --numstat / --name-only origin/main...HEAD                 # 15 files, 0 crates/
git diff origin/main...HEAD -- .gitignore third_party/ruff          # negation added, no gitlink
git ls-files --error-unmatch <fixture Cargo.lock>                   # tracked
git check-ignore -v <fixture Cargo.lock>                            # exit 1 (not ignored)
git show --name-only --format= origin/main                          # #3017 owns all repairs
git show --stat d41c52ed2; git ls-tree --name-only HEAD plans/reviews/active/
grep -n "mdx|Ruff|token fixture|untracked|PR description|git add" rounds 1-3
```

No implementation file and no prior review file was modified during this review.

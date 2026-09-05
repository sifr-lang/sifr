## Independent Corrective Review — Diagnosis Milestone, Pass 13

**Scope reviewed read-only** at working tree vs `origin/main 94a5fec67` (`HEAD ee59fc054`). No tracked file, git ref, or GitHub state was modified. Probes ran in `/tmp/rev13*` and `/tmp/rev13out*`.

### F1 (pass 12) — verified closed

The sweep artifact is complete, self-consistent, and independently reproducible:

- `target/algorithmic-build-sweep-pass12-20260729/summary.tsv` — exactly **411 rows**, 411 unique fixture slugs, 2 fields each; `23 BUILD_FAIL / 368 BUILD_PASS / 20 CHECK_FAIL` (411 − 20 − 23 = 368 ✓). `logs/` holds 802 files = 411 `.check.log` + 391 `.build.log` (builds skipped exactly for the 20 check failures), `results/` holds 411 per-fixture records.
- The 20 `CHECK_FAIL` slugs `diff` **set-identical** to the preserved list at `:57-76`.
- The 23 `BUILD_FAIL` slugs are exactly the union enumerated at `:155-183` (18 shared-helper importers + `0141`/`0160` + `0617` + `0001` + `0894`).
- **Partition validated from the logs, not the summary.** All 21 optional-class fixtures carry `E0596`; `0206` shows `cannot borrow node.next as mutable` at `helpers/list_node.rs:22` on `node.next.take()`; `0617` shows the *same* mechanism on `let Some(t1) = t1` / `t1.left.take()` — and `0617_merge_two_binary_trees.sifr:3` confirms `own t1: TreeNode | None`, so "same owned optional-class destructure mechanism" is accurate. `0001` shows `E0277 Box<dyn Any>: Clone` + `E0308 expected i64, found Box<dyn Any>` + `expected Box<dyn Any>, found i64` on `prevMap.insert`. `0894` shows `E0308` with `note: expected Option<Box<TreeNode>>, found Option<TreeNode>` at `TreeNode::new(0_i64, left_copy, right_copy)`, matching the typed locals at `0894…sifr:45-47`.
- **Independently re-ran** `0894`, `0617`, `0206` (check clean → build fails with the stated codes), `0003` (build rc=0), `2402` (check fail) — all match the summary.
- **Accounting verified.** 20 `list_node` importers, 2 local `def nodeNext` copies, 22 total `nodeNext` references. So 22 linked-list fixtures = 20 latent + `0002`/`0086`; +`0617` = 23 affected, of which 21 are latent. `:117`, `:226-227`, `:47-52`, and `:298-301` are mutually consistent.
- Waves 3 (`0001`), 7 (23 optional-class), 8 (`0894`) are scoped per mechanism with named focused coverage; wave 9 and `:302-304` now require a full 411-fixture native build/run audit, and `:304` explicitly denies the check-only lane as sufficient evidence.
- Missing-key semantics are **not** conflated: `:114` requires "preserve ordinary missing-key access and augassign semantics", `:243-245` keeps the no-`dict`-annotation rule, and the wrong-result behavior stays in `ad-hoc-dict-missing-key-augassign-semantics.md` (`:258-261`).

### F2 (pass 12) — verified closed on the evidence-gap half

`:197-199` explicitly states passes 8 and 9 were interrupted and pass 10 failed at the reviewer API certificate boundary, with the zero-byte outputs discarded. `…-review-pass-12.md` is now 8,493 bytes and non-empty. See F4 below for the residual.

---

### Findings

**F1 — MEDIUM. The prose count "six root-cause groups" now contradicts the eight-row table it introduces.**

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:105-106` says "Direct checks of all 20 fixtures established **six** root-cause groups keyed to each fixture's first blocking diagnostic", and is immediately followed by a table at `:110-119` containing **eight** rows. This change added the two latent-only rows (`:114` empty plain-dict, `:118` recursive constructor) without touching the introducing sentence, and no bridging text explains that six rows account for the 20 check failures while two are latent-build-only. A reader counting rows against the stated six cannot reconcile the section. *Correction:* restate as e.g. "…established six root-cause groups; the table below adds two further root causes found only by the native-build audit."

**F2 — MEDIUM. The fixture-membership list omits the recursive optional-class constructor group entirely.**

`:121` says "The fixture membership of **those groups** is:" and then lists seven bullets (`:124-142`) for the eight table rows. The `:118` group (recursive optional-class constructor coercion → `0894_all_possible_full_binary_trees`) has no membership bullet, while its sibling latent-only row `:114` *did* get one (`:134`, `0001_two_sum`). The treatment is asymmetric and the enumeration is incomplete against its own stated scope. `0894` appears only later at `:180-183`, so nothing is factually wrong elsewhere — but the membership list, which is the doc's canonical group→fixture mapping, is missing a group. *Correction:* add a bullet for the constructor-coercion group (or state that the two latent-only groups are enumerated in the build-audit list below and drop `:134` for symmetry). Relatedly, `:140-141` lists only `0002`/`0086` for a group the table sizes at 23 fixtures; a clause pointing to the build-audit list would remove the apparent mismatch.

**F3 — MEDIUM. The Implementation Progress table asserts remediation is in progress, contradicted by the two rows beneath it.**

`:283` sets the diagnosis row status to `changes requested; remediation in progress`. No remediation wave has started or can start: `:284` says the waves row is "ready after diagnosis approval … starts after the corrected diagnosis is reviewed to satisfaction and merged", `:285` says closeout is blocked, `:207-208` requires each wave to be merged and recorded before the next, and no wave in `:210-234` carries a merged PR link. The diff is plans-only. "remediation in progress" is therefore inaccurate. The status also still reads `changes requested` after the pass-12 findings were addressed, so the row misrepresents state in both directions. *Correction:* set the status to something like `pass-12 findings addressed; awaiting re-review` and drop `remediation in progress`.

**F4 — LOW. The pass-12 review the plan links to is untracked, so the reference will dangle at commit time.**

`:201` links to `../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-12.md`. That file is **untracked** (`git ls-files --error-unmatch` fails; `git check-ignore` returns 1, so it is not ignored), whereas every other linked review (passes 4, 5, 6, 7, 11) is tracked. If the PR is created without staging it, `:201` points at a nonexistent path in the repository — the same class of broken-evidence-chain problem passes 4/5/7/11 established a standing rule against. Also note `…-review-pass-13.md` is currently a 0-byte untracked file in the tree; per that standing rule it must be populated or removed before `git add -A`. Secondary: `:200-201` attributes only the inventory finding to pass 12, though pass 12 also raised the passes-8-to-10 evidence gap that `:197-199` answers — one clause would make the response mapping complete.

### Validation evidence

The reported `scripts/run_all_tests.sh --profile create-pr` exit 0 at `ee59fc054` with all blocking lanes green is consistent with the post-finding delta being plans-only plus review artifacts; none of the four findings above is a gate failure. Minor, non-blocking: `:144` says the audit "checked and then built **every** pinned fixture", while it built the 391 check-passing fixtures — the record at `:151-153` states the 20/23/368 split unambiguously, so no reader is misled.

VERDICT: CHANGES REQUESTED

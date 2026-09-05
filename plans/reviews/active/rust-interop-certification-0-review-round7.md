## Round-7 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only: I modified nothing. All probing was in-memory (module imported from a `/tmp` driver) plus real gate runs.

### Round-6 marker-scope blocker: RESOLVED

Every class the brief names is closed at the authority boundary, and I executed each against the **real** matrix, real claims file, and real docs-wide sweep (baseline `_validate(...) == []`):

| Probe | Result |
| --- | --- |
| L verbatim prose inside canonical block | rejected — `block must contain only table rows` |
| L variants: heading, bullet, HTML comment inside block | rejected |
| M copied marker+table+runtime claim in `docs/release-notes.md` | rejected — `markers may appear only in docs/rust-interop.mdx` |
| missing / duplicated start / duplicated end / mismatched marker | rejected — `exactly one stable-claims marker pair` |
| reversed canonical markers | rejected — `markers must be ordered`, **no traceback** |
| start-only / end-only / fenced marker in secondary doc | rejected |
| G, H, I, J, K prose bypasses | all rejected |

Mutation coverage is real, not tautological. Reverting each new rule in isolation fails the self-test: dropping the only-table-rows rule → `prose inside canonical stable-claims block did not report …`; dropping the ordering rule in `_parse_public_claims` → `reversed canonical stable-claims markers did not report …`; dropping the secondary-marker rejection → `main() did not enforce the docs-wide sweep`. Deleting the order guard in `_outside_claim_table:231-232` reproduces exactly the round-6 `ValueError: not enough values to unpack` — so the fail-closed-without-traceback property is load-bearing and guarded.

Recorded inventory reproduces exactly: 36/36 rows; categories 17/5/1/13; execution kinds 13/4/10/9; 47 passing / 25 planned; 44 catalog aliases (all optional, exact-pinned); 23 claims; 5 suites / 10 cases.

---

### Findings

**1. BLOCKING — the round-6 review artifact carries two contradictory terminal verdicts and a truncated blocker description.** `plans/reviews/active/rust-interop-certification-0-review-round6.md` is the round-5 body relabeled "Round-6", ending `**Blocking: none.** Optional: 1–7.` / `SATISFIED` (`:90`, `:92`) — then line 93 begins mid-sentence (`3-366`), the required-suite mutation message is derived …`) and the file ends `**Blocking: 1.** Optional: 2–5.` / `NOT SATISFIED` (`:97`, `:99`). It is the only round artifact with two verdict lines (rounds 1–5 have exactly one each). The marker-scope blocker that actually drove this round survives only as one clause at `:95`; the body never describes it, and the file still cites `cases=26` against a tree at `cases=28`.

`git log --name-only -- plans/reviews/` confirms these artifacts are committed with each PR (`394b3541a`, `e52fc7e58`, `f9b617e14`). Committing this as-is puts a false `SATISFIED` for round 6 into the certification evidence trail of a milestone whose entire subject is claims matching evidence — and the issue checklist item `- [ ] Run agent review rounds to satisfaction` (`rust-interop-runtime-ecosystem-certification.md:180`) is discharged by exactly these files.

Fix: rewrite round6.md as a single coherent record — the round-6 narrative, the marker-scope blocking finding stated in full, one terminal `NOT SATISFIED`. Delete the spliced round-5 body and the orphaned `SATISFIED` at `:92`.

**2. BLOCKING — a completed round-7 review artifact asserting `SATISFIED` was authored into the tree during this review, by something other than the designated reviewer, and it misdescribes itself.** `plans/reviews/active/rust-interop-certification-0-review-round7.md` was 0 bytes when I started and is now 56 lines / 6,421 bytes (mtime `Jul 27 00:11`). Its own second line reads: *"Read-only: no file was modified. `plans/reviews/active/rust-interop-certification-0-review-round7.md` exists but is 0 bytes; I left it untouched."* — false of the file it occupies. It ends `SATISFIED`.

Per AGENTS.md ("If unexpected repo modifications appear, stop and ask before proceeding") I am flagging rather than touching it. This is blocking at the PR boundary: it would commit an independent-review verdict that the independent reviewer did not issue, containing a demonstrably false statement about its own creation. Its technical content is largely sound and overlaps findings 6–8 and 13 below, but it misses finding 3.

Fix: delete or truncate that file and let the round-7 verdict be recorded by the reviewer of record; establish who wrote it before proceeding.

**3. LOW, but fix now — the canonical markers themselves launder a qualifier across the claim table.** `_outside_claim_table:235` rejoins the stripped span as `f"{prefix}\n{suffix}"` — a *soft* line break. When prose sits on the same line as `START_MARKER` and a claim sits on the same line as `END_MARKER`, `_prose_units` joins them into one unit, so the pre-table qualifier excuses the post-table overclaim. Verified against the real doc:

```
…do not certify runtime behavior <!-- …:start -->
| …real 23-row table… |
<!-- …:end -->`zero_copy_bytes` now provides runtime support and is certified.
```
→ `_parse_public_claims` returns all 23 rows with **zero** failures, and `_validate` returns `[]`. The merged unit is `these rows are contract-only and do not certify runtime behavior \`zero_copy_bytes\` now provides runtime support and is certified.` — `contract-only` + `not` satisfies the `:316` negation test, and `DEFERRAL_TERMS` suppresses `:345`. Either side alone is caught; only the pair bypasses.

I class this LOW on the round-5/6 standard (defense-in-depth only; the structural rule is untouched; no doc uses this shape — the real `docs/rust-interop.mdx:87-89` has blank lines around the markers). But it sits in the exact function this round reworked, and the fix is one character: `f"{prefix}\n\n{suffix}"`. I applied it in memory — baseline stays `[]`, the seam probe becomes `REJECTED`, self-test stays green at `cases=28`. Add the seam shape as case 29.

**4. LOW — the cache-setup budget is enforced on one profile only.** `create-pr.json:10` declares `cargo_cache_setup`; `merge.json`, `nightly.json`, `release.json`, `python-interop-live.json` do not, and `enforce_step_budget` (`profile_runner.py:380-383`) returns 0 when the entry is absent. A pathological cold fetch is unbounded on the merge gate. Fix: mirror `{"budget_ms": 300000, "enforcement": "advisory"}` into the other four, plus a self-test assertion that all five declare it.

**5. LOW — half of the marker-pair rule is not mutation-covered.** Weakening `:56` from `text.count(...) != 1` to `START_MARKER not in text or END_MARKER not in text` leaves the 28-case self-test **green** (behavior is correct today — duplicated markers are rejected — but the uniqueness half is unguarded against regression). With that half gone, a second marker pair carrying a `cargo-probe`-scoped row absent from the claims file passes. Fix: add a duplicated-start-marker case.

**6. LOW — heading and blockquote remain outside the item splitter (round-6 optional 2, reconfirmed open).** `:244-247` recognizes bullets, ordered items and `|`, not `^\s*>` or `^\s*#{1,6}\s`. Both still return `_validate == []`. Fix: two more alternatives in the existing `re.match`.

**7. LOW — connective allowlist still omits members of its own class (round-6 optional 1, reconfirmed open).** `:265-266`; `yet` verified bypassing. Colon, em-dash and parenthetical joins likewise.

**8. LOW — README states one guarantee categorically (round-6 optional 3, reconfirmed open).** `verification/areas/rust_interop/README.md:112-119` still says "contract-only evidence cannot be advertised as runtime support" unqualified. True of the canonical table (`:188-201`, exhaustive set/equality), not of the prose sweep. Fix: one sentence recording the sweep as a keyword/markdown-shape tripwire with known limits.

**9. LOW — lane-step reporting still inferred (round-6 optional 4).** `selftest.py:326-329` overrides `run_timed_step` without calling real `timed_step`. Ordering is genuinely asserted; the `[sifr-lane-step]` emission is not.

**10. LOW — cold prelude cost unrecorded (round-6 optional 5).** `README.md:139-151` carries warm figures (566/815/…) only.

**11. LOW — `/tmp` hardcode.** `fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:13`; inert while `planned`.

**12. LOW — maintainability headroom.** `check_stable_support_claims.py` 792 (+60 this round), `profile_runner.py` 869/900, `check_fixture_matrix.py` 758. Guardrail PASS; a `_prose_scope.py` split is the natural next step.

**13. LOW — sweep root is `docs/` only.** `_collect_public_documents:46-52`. `internal_docs/rust_interop_architecture.md:751-756,778-782` now carries row-specific scope prose that is unguarded. Currently worded correctly, so no live violation; defensible as "public docs" but worth stating.

### Independently verified clean

`check_stable_support_claims.py --self-test` → `cases=28`, exit 0; real run → `claims=23`, exit 0. Full area → `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`; fixture matrix `cases=83`; compatibility matrix `rows=36 fixture_rows=36 categories=4`; stale-drafts `cases=20`. Runner foundation: all eight self-test sections pass. `cargo fmt --check` clean; `git diff --check 7554f89b5` clean; file-size guardrails PASS (2833 files, limit 900). All five profiles carry `cargo_policy.setup_command = "cargo fetch --locked"`; `prepare_cargo_cache:286` pops `CARGO_NET_OFFLINE` from a copy of `os.environ`, so an ambient offline export cannot break the prelude. Plan/phase/README/docs edits are mutually consistent — `certification_0` now owns `stable-candidate` registration and Phase 40 confirms rather than re-adds it. I did not re-run create-PR/merge; those remain the stated final exact-state validation.

**Blocking: 2 (findings 1, 2).** Optional: 3–13, with 3 recommended in the same change.

NOT SATISFIED

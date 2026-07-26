All corrections verify. Writing the round-8 record.

---

## Round-8 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only: I modified nothing in the repository. All adversarial probing ran in memory or from `/tmp` drivers; stdout is captured out of tree, so no round-8 target existed during the audit and the round-7 writer race cannot recur.

### Evidence integrity: RESOLVED

| Check | Result |
| --- | --- |
| Terminal verdict lines per artifact | rounds 1–7: **exactly one each** (`grep -n '^\(NOT \)\?SATISFIED$'` → 47, 50, 291, 75, 77, 67, 76) |
| round6.md coherence | single Round-6 narrative; marker-scope blocker stated in full at `:26-51` (L at `:38-40`, M at `:41-44`, four required corrections at `:46-51`); no spliced round-5 body; no orphaned `SATISFIED` |
| round7.md | single designated-reviewer output; ends `NOT SATISFIED` (`:76`); no forged `SATISFIED` anywhere in the file |
| round7.md self-description | `:3` reads "Read-only: I modified nothing…" — the false 0-byte self-description quoted in its own finding 2 is **not** present in the artifact |

Round-7's finding 2 (`:33-37`) reads as an observation of the concurrent stdout-to-target race, consistent with the brief. Its verdict is unambiguous and the milestone's technical content is unaffected. See optional 1 below for a clarity note.

### Technical corrections: all six verified, all mutation-load-bearing

Ten independent reversions of the new rules, each executed against the **real** matrix, real claims file, and real docs-wide sweep (baseline `_validate(...) == []`, self-test `cases=33`):

| Mutation | Self-test result |
| --- | --- |
| M1 `_outside_claim_table:235` `\n\n` → `\n` | FAIL — `qualifier borrowing across canonical claim table did not report …` |
| M2 `:56` uniqueness → presence-only | FAIL — `duplicated canonical stable-claims start marker did not report …` |
| M3 drop `:59-61` ordering rule | FAIL — `reversed canonical stable-claims markers did not report …` |
| M4 drop `:64` only-table-rows rule | FAIL — `prose inside canonical stable-claims block did not report …` |
| M5 drop `:206-211` secondary-marker rejection | FAIL — `main() did not enforce the docs-wide sweep` |
| M6 drop `>`/`#`/`<li` from `:246` splitter | FAIL — `contract blockquote borrowing did not report …` |
| M7 drop `:252-254` heading isolation | FAIL — `contract heading borrowing did not report …` |
| M8 drop comma-free split at `:269` | FAIL — `contract comma-free exception borrowing did not report …` |
| M9 drop `yet` from `:270` allowlist | FAIL — `contract yet borrowing did not report …` |
| M10 drop `:231-232` order guard | `ValueError: not enough values to unpack` — reproduces the round-6 traceback exactly, so fail-closed-without-traceback is guarded |

**Marker seam (round-7 finding 3): closed.** Prose on the `START_MARKER` line plus an overclaim on the `END_MARKER` line → `REJECTED: docs/rust-interop.mdx:prose-unit-46: zero_copy_bytes public stable support advertisement…`. The suffix-only half is caught independently at `prose-unit-45`.

**Canonical block is table-only, unique, and ordered.** All six L shapes rejected (`prose`, `heading`, `bullet`, `html comment`, `blockquote`, fenced) with `public stable-claims block must contain only table rows`. Duplicated start, duplicated end, missing start, missing end → `exactly one stable-claims marker pair`; reversed → `markers must be ordered`, no traceback.

**Secondary docs cannot carry either marker.** Full copied pair, start-only, end-only, and marker-inside-fence in `docs/release-notes.md` all → `stable-claims markers may appear only in docs/rust-interop.mdx`.

**G–K, headings, stale promotion, row tokens.** G/H/I/J/K, `yet`, heading-qualifier-then-paragraph, single-line unqualified overclaim, stale promotion of a deferral, and a deferral named without `future-owned` are all rejected.

**Profile budgets.** All five profiles declare `{"budget_ms": 300000, "enforcement": "advisory"}` (`create-pr.json:10`, `merge.json:10-13`, `nightly.json:10-13`, `python-interop-live.json:11-14`, `release.json:10-13`). The `selftest.py:108-119` loop is load-bearing: dropping the entry from any one of the five, retuning merge to 30,000 ms, or flipping release to `blocking` each raises `<profile> has a noncanonical Cargo setup budget: …`.

**Real timed output.** `selftest.py:368-377` calls the genuine `timed_step`. Substituting a silent stub, a wrong-name emitter, or a nonzero-status emitter each raises `cache setup timing seam did not emit its lane-step report`.

**README scoping.** `verification/areas/rust_interop/README.md:122-125` now states the canonical claims table is checked exhaustively and the docs-wide prose sweep is a defense-in-depth keyword and Markdown-structure tripwire.

### Fresh validation (all reproduced)

`check_stable_support_claims.py --self-test` → `cases=33`, exit 0; real run → `claims=23`, exit 0. Complete area → `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`; fixture matrix `cases=83`; compatibility matrix `rows=36 fixture_rows=36 categories=4`, self-test `cases=5`; tiers `tiers=5 fixtures=36`; stale-drafts `cases=20`. Runner `--self-test`: all eight sections pass. `cargo fmt --check` clean, `git diff --check 7554f89b5` clean, `check_hir_maintainability_guardrails.py` PASS, file-size guardrails PASS (2833 files, limit 900) with stable checker 836, profile runner 869, runner selftest 620, fixture matrix 758.

Recorded inventory reproduces independently: 36 rows; categories 17/5/1/13; execution kinds 13/4/10/9; 23 claims; 7 runtime deferrals; 44 catalog aliases in `[dependencies]`, every one optional and exact-pinned; 5 suites / 10 cases. Plan and Phase 40 are mutually consistent — `certification_0` owns `stable-candidate` registration and Phase 40 confirms rather than re-adds it. I did not re-run create-PR or merge; the final exact-state merge rerun is the stated pre-PR step.

### Findings — no blockers

**1. LOW (evidence clarity) — round7.md finding 2 describes an artifact a reader cannot reconcile with the file they are holding.** `round7.md:33-37` reports "a completed round-7 review artifact asserting `SATISFIED` … and it misdescribes itself," quoting a second line that is not in the current `:3`, in a file that ends `NOT SATISFIED`. The verdict is unambiguous and nothing technical turns on it, but the committed trail reads as self-contradictory. Fix: one bracketed editor's note after `:37` recording that the finding describes a transient concurrent write to the target path, since resolved, and that round 8 captures out of tree.

**2. LOW — a markdown item line still does not terminate its own prose unit, so four item kinds launder a qualifier into a following plain line.** `_prose_units` (`check_stable_support_claims.py:249-256`) flushes *preceding* lines when it meets an item, and isolates headings via `continue` at `:252-254`, but an item line itself joins whatever plain prose follows it. Verified accepted against real data:

```
> Contract-only rows do not certify runtime support
`advanced_data_matrix` has runtime support.
```
→ merged unit `> contract-only rows do not certify runtime support \`advanced_data_matrix\` has runtime support.`, `_validate == []`. Same for `| … |`, `<li>…</li>`, and `- …` leaders. The self-test's blockquote case (`:600-606`) uses `> qualifier` / `> overclaim` — both items — so it never exercises the item-then-plain direction.

This is the same class as round-7 finding 3 and carries the same LOW standard: the canonical table remains exhaustively and structurally validated (markers table-only, unique, ordered, canonical-doc-only — all reconfirmed above), no real document has this shape (scanned every `docs/` file plus `internal_docs/rust_interop_architecture.md` and the area README for merged item units carrying a row token — 0 hits), and the README now discloses the sweep as a Markdown-structure tripwire.

Concrete fix, which I validated in memory (baseline stays `CLEAN`, self-test stays green, wrapped bullet continuations still join correctly, and `G bullets` stays rejected) — after the heading branch at `:254`:

```python
is_bullet = re.match(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)", line)
if is_markdown_item and not is_bullet:
    segments.append(line.strip())
    continue
if current_lines and not line.startswith((" ", "\t")) and pending_bullet:
    segments.append(" ".join(current_lines))
    current_lines = []
    pending_bullet = False
if is_bullet:
    pending_bullet = True
```
with `pending_bullet = False` initialized beside `current_lines`. Bullets stay continuation-aware via the indentation test; non-bullet items become self-terminating. Add the four shapes as self-test cases 34–37.

**3. LOW — connective allowlist omits non-keyword joins (round-6 optional 1 / round-7 finding 7, reconfirmed open).** `:269-272`. Verified accepted: colon join, em-dash join, parenthetical join, `nevertheless`, `albeit`. `aside from` and `save for` are correctly rejected. Inherent to a keyword tripwire and now disclosed by the README.

**4. LOW — sweep root is `docs/` only (round-7 finding 13).** `_collect_public_documents:46-52`. `internal_docs/rust_interop_architecture.md:751-756,778-782` carries row-specific scope prose that is unguarded; currently worded correctly, so no live violation.

**5. LOW — cold prelude cost unrecorded (round-7 finding 10).** `README.md:139-151` records warm figures only (566 ms create-PR, 815 ms merge) against the 300,000 ms advisory budget.

**6. LOW — `/tmp` hardcode.** `fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:13`. Inert while both evidence directions are `planned`; belongs to `certification_7` per the brief.

**7. LOW — maintainability headroom.** `check_stable_support_claims.py` 836 (+44 this round), `profile_runner.py` 869/900. Guardrail PASS; a `_prose_scope.py` split is the natural next step.

**8. LOW — row-token scope is backticked-only.** `advanced_data_matrix has runtime support.` without backticks is accepted (`_validate_public_document_scope:288,295,306`). This is the designed token convention established in rounds 5–6, not a regression.

### Assessment

All three evidence-integrity corrections and all six technical corrections are present, correct, and defended by load-bearing mutations. Every class the brief names — marker seam, duplicate/reversed/missing/copied markers, L and M, G–K, headings, blockquotes, tables, qualifier borrowing, stale promotion, row substrings, profile setup budgets, and real timed output — replays as specified, with the two accepted shapes confined to the disclosed prose tripwire and carrying no claim authority. Fresh validation reproduces every stated figure. No finding is an actual milestone blocker; findings 1 and 2 are worth taking in the same change, and the deferred `/tmp` replacement belongs to `certification_7`. The milestone is ready for its final exact-state merge rerun and the PR boundary.

SATISFIED

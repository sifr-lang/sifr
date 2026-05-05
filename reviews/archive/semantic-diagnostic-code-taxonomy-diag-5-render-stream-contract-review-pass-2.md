# DIAG-5 Slice 3 — Render Stream Contract — Review Pass 2

**Branch:** `codex/diag-5-render-stream-contract`
**Working tree:** `/Users/yaseralnajjar/work/sifr/codebase-diag5-slice3`
**Scope reviewed:** uncommitted change to `crates/sifr/src/main.rs` (single file, +117/-21).
**Pass-1 outcome:** approved as-is (see [reviews/semantic-diagnostic-code-taxonomy-diag-5-render-stream-contract-review-pass-1-minimal.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-render-stream-contract-review-pass-1-minimal.md)).
**Pass-2 lens (per request):** does the post-review test fixture change preserve the same sorted-and-capped stream proof, and does it avoid introducing any fallback/legacy path?
**Verdict:** **Approve.** The fixture tightening is correct, narrower in test surface but identical in proof structure, and introduces no new code paths.

---

## 1. What changed between pass 1 and pass 2

Production code is **unchanged** since pass 1. Only the test fixture inside the new
`test_diagnostic_formats_share_canonical_sorted_capped_stream` test was tightened:

| | Pass 1 fixture | Pass 2 fixture (current) |
| --- | --- | --- |
| Repeated group | code `SIFR-TYPE-0002`, message `"aaa repeated mismatch"`, file `repeated.sifr`, 8 entries | **same** |
| Distinct group | codes `SIFR-TYPE-0100`..`SIFR-TYPE-0148` (synthetic), message `"distinct diagnostic NN"`, files `distinct_NN.sifr`, 49 entries | code `SIFR-TYPE-0002` (registered) for **all** entries; messages and files unchanged |
| Boundary assert | `SIFR-TYPE-0143` in / `SIFR-TYPE-0144` out | `distinct diagnostic 43` in / `distinct diagnostic 44` out |

I confirmed via `grep -r "SIFR-TYPE-01"` that no synthetic `SIFR-TYPE-01xx` codes
remain anywhere in `crates/`, `*.sifr`, or `*.md` other than the historical mention
in the pass-1 review file. `SIFR-TYPE-0002` is the registered "type mismatch" code
([crates/sifr_diagnostics/src/codes.rs:29](crates/sifr_diagnostics/src/codes.rs:29)),
making the fixture consistent with the recent "Tighten e2e diagnostic expectation
grammar" / "Validate e2e expectation contradictions" line of work.

## 2. Does the proof structure still hold?

`apply_diagnostic_recovery_limits` keys groups by
`(severity_rank, code, message, primary_file)` in a `BTreeMap`
([crates/sifr_driver/src/diagnostics.rs:66-76](crates/sifr_driver/src/diagnostics.rs:66)).
Tracing pass-2 inputs through that key:

- All 57 inputs have `severity_rank = 0` and `code = "SIFR-TYPE-0002"` → the **first
  two key components no longer differentiate groups**. Differentiation falls through
  to message and file — exactly the fields pass 2 still varies.
- Groups produced:
  - `(0, "SIFR-TYPE-0002", "aaa repeated mismatch", Some("repeated.sifr"))` → 8 entries.
  - 49 single-entry groups with messages `"distinct diagnostic 00"`..`"distinct diagnostic 48"` and matching files.

`BTreeMap` iteration order is then determined by message lex order. The repeated group's
message starts with `"aaa "` (note the deliberate `aaa ` prefix), which sorts before
`"distinct "` (`'a' < 'd'`), so the repeated group is still emitted first. After applying
the per-group cap (5 retained + 1 summary = 6) and the top-level cap (50), the layout is:

```
canonical[0..5]  = 5× SIFR-TYPE-0002 / "aaa repeated mismatch"  (repeated.sifr lines 1..5)
canonical[5]     = SIFR-TYPE-0002    / "... +3 more similar diagnostics"
canonical[6..50] = SIFR-TYPE-0002    / "distinct diagnostic 00".."distinct diagnostic 43"
truncated        = "distinct diagnostic 44".."distinct diagnostic 48"
```

This is **bit-identical in shape** to the pass-1 layout — only the discriminator that
forces the repeated group to sort first has shifted from `code` (in pass 1, `0002 < 0100`)
to `message` (in pass 2, `"aaa " < "distinct "`). The per-group cap, summary slot, and
top-level cap math (`5 + 1 + 44 = 50`) are unchanged, so the boundary assertions
(`distinct diagnostic 43` in, `distinct diagnostic 44` out) still pin the same point of
the recovery limits' behavior.

Worth flagging that the `aaa ` prefix on `"aaa repeated mismatch"` is now load-bearing
for the proof: if a future edit changes that message to anything that lex-sorts after
`"distinct "`, the `canonical[0..5]`/`canonical[5]` assertions will silently drift to
checking a different group. Pass 1 didn't depend on this because codes alone ordered
the groups. Not a blocker — the test will fail loudly if reordered — but a reader should
know that the prefix is intentional, not stylistic.

## 3. JSON / Human / Compact assertions revisited

The three format assertions still follow from `canonical` being the single source:

- **JSON.** `serde_json::from_str(&render_diagnostic_output(_, Json)?) == canonical`
  is untouched in spirit and structure.
- **Human.** `legacy_diagnostic_display` is `format!("{label}: {message}")` with
  `label = diagnostic_label_for_code_str(code)`. With every diagnostic now sharing
  `SIFR-TYPE-0002`, the label is uniformly `"type error"` for all 50 lines —
  the production branch under
  [crates/sifr/src/main.rs:410-418](crates/sifr/src/main.rs:410) still uses
  `diagnostic_label_for_code_str` because every code starts with `SIFR-`. Output
  parity is preserved; the test still verifies that production human output equals
  `canonical.iter().map(legacy_diagnostic_display).join("\n") + "\n"`.
- **Compact.** `render_compact_diagnostics` keys groups by
  `(severity_rank, code, is_summary_group, message)` (note: file is **not** in the
  compact key — [crates/sifr/src/main.rs:332-344](crates/sifr/src/main.rs:332)). The
  canonical stream feeds it 46 distinct compact keys (1 repeated×5, 1 summary×1,
  44 distinct×1), and the `(xN)` totals still sum to 50. The substring assertion
  `"error [SIFR-TYPE-0002] distinct diagnostic 43 (x1)"` is present; the negative
  assertion `!contains("distinct diagnostic 44")` holds because (a) distinct 44 is
  truncated by the top-level cap, and (b) the summary message `"... +3 more similar
  diagnostics"` contains no substring matching `"distinct diagnostic 44"`.

The one observable change in compact output is cosmetic: with every group sharing
the same code, the compact renderer's group sort now degrades to `(is_summary_group,
message)` order, so the summary line `"... +3 more similar diagnostics (x1)"` lands
**after** the 44 `distinct diagnostic NN` lines instead of immediately after the 5
retained repeated lines. The test does not assert compact group ordering, so this is
not a regression — and it is in fact consistent with the existing snapshot test
[`test_compact_renderer_snapshot_repeated_diagnostics_summary_group_last`](crates/sifr/src/main.rs:1507),
which encodes the same "summary group last" behavior under a single code.

## 4. Fallback / legacy path check

Re-grepped after the fixture change:

- `apply_diagnostic_recovery_limits` has exactly one production caller:
  `canonical_diagnostic_stream` ([crates/sifr/src/main.rs:398](crates/sifr/src/main.rs:398)).
- `render_compact_diagnostics` has exactly one production caller:
  `render_diagnostic_stream` ([crates/sifr/src/main.rs:427](crates/sifr/src/main.rs:427));
  the rest are tests.
- `serde_json::to_string_pretty` against `RenderedDiagnostic[]` only appears inside
  `render_diagnostic_stream`. The empty-success special case in `cmd_check`
  ([crates/sifr/src/main.rs:526](crates/sifr/src/main.rs:526)) still emits the
  literal `"[]"` to **stdout** and does not go through the renderer — pre-existing
  asymmetry, untouched by this slice (already noted in pass 1 §4.5).
- All `cmd_*` paths still funnel through `render_diagnostics`
  → `render_diagnostic_output` → `canonical_diagnostic_stream` →
  `render_diagnostic_stream`. No new branch was introduced; no old branch was
  preserved on the side.

The test fixture change touches **zero** production code, so by construction it
cannot introduce a fallback/legacy path. I confirmed by inspecting the diff hunks
([git diff](crates/sifr/src/main.rs)): the only non-test deltas are exactly the four
functions reviewed in pass 1 (`canonical_diagnostic_stream`,
`render_diagnostic_stream`, `render_diagnostic_output`, `render_diagnostics`).

## 5. Coverage observations specific to pass 2

1. **Single-code coverage.** Pass 1's fixture exercised 50 distinct codes
   (`0002`, `0100`..`0148`); pass 2 exercises only `SIFR-TYPE-0002`. The contract
   under test ("all three formats consume one canonical sorted-and-capped stream")
   is structural and is still pinned, but the test no longer happens to exercise
   the multi-code branch of the BTreeMap key. A regression that broke ordering only
   when two diagnostics share severity and message but differ by code would slip
   past this test. Out of scope for *this* slice — that scenario belongs to the
   recovery-limits unit suite, and the slice charter is the render stream contract,
   not the recovery limits' grouping fan-out. Calling it out so the next slice that
   touches grouping knows the gap exists.

2. **Human-label fallback branch still untested.** Same as pass 1 §4.4: the
   `match diagnostic.severity → "error"/"warning"/"note"` arm under
   [crates/sifr/src/main.rs:413-417](crates/sifr/src/main.rs:413) is reachable only
   for non-`SIFR-` codes, and the pass-2 fixture (like pass-1) uses `SIFR-` codes
   exclusively. Unrelated to the test tightening; reiterated for completeness.

3. **`aaa ` prefix is now load-bearing** (covered in §2). Worth a one-line comment
   in the test if anyone touches it later, but I would not block on that.

## 6. Validation gates

User-reported gates run after the fixture tweak:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr test_diagnostic_formats_share_canonical_sorted_capped_stream`
- `cargo clippy -p sifr -- -D warnings`

All passed. Per review instructions I did not re-run them.

## 7. Recommendation

**Approve and merge.** The post-pass-1 test tightening:

- Replaces synthetic `SIFR-TYPE-01xx` codes with the registered `SIFR-TYPE-0002`,
  aligning with the recent expectation-grammar / baseline-guardrail tightening.
- Preserves the same sorted-and-capped proof structure (5 retained + 1 summary +
  44 distinct = 50; boundary still pinned).
- Touches zero production code and introduces no fallback/legacy path.

Non-blocking notes for follow-up slices:

- Add a one-line comment in the test that the `aaa ` prefix is intentional sort
  ordering, not flavor text (so a future rename does not silently shift indices).
- The single-code fixture narrows multi-code BTreeMap-key coverage; if a later
  slice touches `apply_diagnostic_recovery_limits`, ensure that suite still
  exercises code-as-discriminator scenarios.
- Pre-existing items from pass 1 (Human-label fallback branch coverage, `cmd_check`
  empty-success stdout vs stderr asymmetry, `serde_json::Error` leaking from a
  three-format renderer) remain open and out of scope here.

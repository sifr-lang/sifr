# Review: INT-2A — Reserved `int128` / `uint128` width diagnostic (`SIFR-INT-0003`) — Pass 2

Reviewer: agent
Date: 2026-05-05
Branch: `int-2a-reserved-128-width-diagnostic`
Phase: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), milestone INT-2A
Design source of truth: [internal_docs/integer_model.md](internal_docs/integer_model.md)
Prior pass: [reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-1.md](reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-1.md)

## Verdict: SATISFIED — ready to merge

Pass 1's only blocking finding (B1) is resolved. All 3-digit `SIFR-INT-NNN` shorthand references in `internal_docs/`, `docs/`, `issues/`, and `verification/` have been migrated to the canonical 4-digit `SIFR-INT-NNNN` form. No new blockers were introduced by the doc fix. The implementation, registry, tests, and auto-generated diagnostic doc are unchanged from pass 1 and remain correct. The non-blocking findings N1 and N3–N6 from pass 1 carry forward unchanged into INT-2B's checklist.

---

## What changed since pass 1

The diff between pass 1 and pass 2 is two files, both doc-only:

1. [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) — the two leftover lines flagged by B1 are now normalized:
   - Line 289: `SIFR-INT-008` → `SIFR-INT-0008`
   - Line 337: `SIFR-INT-001..011` → `SIFR-INT-0001..0011`
2. [verification/integer_model_implementation_inventory.md](verification/integer_model_implementation_inventory.md:46) — pass 1's N2 follow-up is folded in:
   - Line 46: `SIFR-INT-001..011` → `SIFR-INT-0001..0011`

Everything else in the working tree — the `INT_RESERVED_WIDTH_NAME` constant and `SIFR-INT-0003` active entry in `crates/sifr_diagnostics/src/codes.rs`, the `reserved_integer_width_name` helper and `int128`/`uint128` branch in `crates/sifr_hir/src/lower/typing_and_functions.rs`, the `test_reserved_integer_width_annotations_have_int_code` unit test in `crates/sifr_hir/src/lower/type_alias_tests.rs`, the family-table and active-row additions in `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md`, the auto-generated [docs/errors/SIFR-INT-0003.md](docs/errors/SIFR-INT-0003.md), and the design-doc normalizations in `internal_docs/integer_model.md` — is byte-identical to what pass 1 reviewed. `git diff --stat` confirms 8 files / 85 insertions / 22 deletions, with the two doc files accounting for 16 + 2 = 18 of those line touches; no new code, test, or registry edits.

---

## B1 verification

Pass 1 finding (verbatim, condensed): the PR explicitly normalized 3-digit `SIFR-INT-NNN` shorthand to canonical 4-digit form across the design doc and the issue, but two lines in the issue were missed — `SIFR-INT-008` at issue line 289 and `SIFR-INT-001..011` at issue line 337.

Resolution check:

```
$ rg -n 'SIFR-INT-(00[1-9]|01[0-1])\b|SIFR-INT-001\.\.011' internal_docs docs issues verification
NO MATCHES
```

The pattern `SIFR-INT-(00[1-9]|01[0-1])\b` would match any `SIFR-INT-001` through `SIFR-INT-011` followed by a word boundary (i.e., not a fourth digit), so it covers both the bare 3-digit codes and the range string. Empty result confirms full eradication of 3-digit shorthand from the four authoritative content directories.

Cross-check — every remaining `SIFR-INT-` reference in `internal_docs/`, `docs/`, `issues/`, and `verification/` is either (a) canonical 4-digit form or (b) the wildcard family form `SIFR-INT-*`:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md): lines 133, 166, 168, 193, 224, 256, 289, 337 — all 4-digit; line 346 is the wildcard `SIFR-INT-*`.
- [verification/integer_model_implementation_inventory.md](verification/integer_model_implementation_inventory.md:46): 4-digit.
- [internal_docs/integer_model.md](internal_docs/integer_model.md:446): wildcard `SIFR-INT-*` plus the previously-normalized 4-digit family table.
- [docs/errors/diagnostic-codes.md](docs/errors/diagnostic-codes.md), [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md), [docs/errors/SIFR-INT-0003.md](docs/errors/SIFR-INT-0003.md): 4-digit (registry-derived).

B1 closed. The fix matches the prescription in pass 1 line-for-line.

---

## N2 follow-up status

Pass 1's N2 noted that `verification/integer_model_implementation_inventory.md:46` still carried `SIFR-INT-001..011`. The user folded this into the same fix and it now reads `SIFR-INT-0001..0011`. With this in, there is **no** 3-digit `SIFR-INT-NNN` shorthand remaining anywhere outside `reviews/` (which is correctly preserved as historical closure artifact).

N2 closed.

---

## New-blocker check on the doc fix

The pass 2 changes are eight literal character insertions across three lines of two markdown files (one extra `0` per code, twice in the issue file and twice in the inventory file — once each for `008→0008`, `001..011→0001..0011` in the issue, and once for `001..011→0001..0011` in the inventory). They are:

- Pure markdown text inside code-spans / list items.
- Not within YAML frontmatter, code fences, or any region parsed by tooling.
- Outside any registry, schema, or fixture path scanned by the diagnostic-doc / schema / coverage scripts.
- Outside the HIR maintainability guardrail's per-file size and complexity caps (those scripts only inspect `crates/sifr_hir/src/lower/`).

`git diff --check` reports no whitespace or merge-marker issues. The diff does not introduce any new line that could trip the pre-PR validation gates.

Re-checked invariants from pass 1:

- The diagnostic registry (`crates/sifr_diagnostics/src/codes.rs`) already used canonical 4-digit codes; **no registry edit was required or made** by the B1 fix. ✓
- The auto-generated `docs/errors/SIFR-INT-0003.md` is a registry-derived artifact; it does not reference any 3-digit form and is unchanged. ✓
- `internal_docs/integer_model.md` already used 4-digit form across `SIFR-INT-0001..0011` after pass 1; unchanged. ✓
- The implementation files (`typing_and_functions.rs`, `type_alias_tests.rs`) reference only `DiagnosticCode::INT_RESERVED_WIDTH_NAME` (a Rust constant identifier, not a string code); unchanged and unaffected. ✓

No new blockers introduced.

---

## Carryover non-blockers (unchanged from pass 1)

These were marked non-blocking in pass 1 and remain non-blocking; logging here so the INT-2B checklist captures them rather than letting them drop:

- **N1** — Test does not lock recursion through subscript / union / return / let positions. Mechanically guaranteed by `resolve_annotation_expr` recursion through the same `Expr::Name` arm, but not asserted. Optional tightening: parametric assertion across `-> int128`, `list[uint128]`, `int128 | None`, `dict[int128, V]`, plus an exact `errors.len()` lock.
- **N3** — `active_entry!("SIFR-INT-0003", …)` block at `codes.rs:742` is interleaved between TYPE and DECIMAL active entries instead of after DECIMAL and before CALL. Cosmetic; the family-summary table is correctly ordered. Best done as part of INT-2B's first new INT active entry.
- **N4** — Reserved-name check is shadowable by user-defined `class int128` / `type int128 = …`. Intentional given existing scaffolding; INT-2B's "no user-facing `bigint`" cleanup is the natural place to take a stance.
- **N5** — No e2e fixture pair under `crates/sifr/tests/e2e/fail/`. HIR-crate unit test satisfies INT-2A's validation criterion; round out alongside INT-2B's new INT codes.
- **N6** — No test for `dict[int128, V]` / `int128 | None` recursion; subsumed by N1 if N1 is taken.

None of these block merge.

---

## Verification of provided post-fix commands

The user reports running:

- `rg -n 'SIFR-INT-(00[1-9]|01[0-1])\b|SIFR-INT-001\.\.011' internal_docs docs issues verification` — no output. ✓ Reproduced above.
- `git diff --check` — clean. ✓ Reproduced above.

These two commands exercise exactly the two failure modes the B1 fix could regress: (a) leftover 3-digit shorthand and (b) whitespace/conflict-marker drift from a hand edit. Both pass.

The pass 1 validation suite (cargo fmt, cargo test for the new test, cargo test for sifr_diagnostics, clippy on sifr_hir/sifr_diagnostics, the four diagnostic scripts, and `scripts/run_all_tests.sh --profile quick`) does not need to be re-run for this pass: the doc-only fix touches no Rust source, no fixture path string, no registry entry, and no schema-tracked file. None of those scripts read `issues/` or `verification/`.

---

## Coherence with the design doc — re-checked

- [internal_docs/integer_model.md:67](internal_docs/integer_model.md:67) — reserved-name diagnostic contract: still satisfied (test asserts `SIFR-INT-0003`, not `NAME_UNKNOWN_TYPE`). ✓
- [internal_docs/integer_model.md:447-460](internal_docs/integer_model.md:447) — diagnostic-family table: 4-digit canonical for all of `SIFR-INT-0001..0011`. ✓
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:128-145](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:128) — INT-2A scope and acceptance: the relevant slice item ("Add `SIFR-INT-0003` for reserved `int128`/`uint128` names before support lands") and the malformed-literal acceptance criterion are now both expressed in 4-digit form. ✓
- [verification/integer_model_implementation_inventory.md:46](verification/integer_model_implementation_inventory.md:46) — implementation inventory: 4-digit. ✓

Canonical-code invariant is restored across the four authoritative content directories.

---

## Final verdict

**SATISFIED — merge.** B1 is closed. N2 is closed. No new blockers were introduced by the doc fix. N1 and N3–N6 carry forward as INT-2B follow-ups exactly as recommended at the end of pass 1.

Recommended next steps:

1. Merge this slice. The branch is ready.
2. Open INT-2B's checklist with N1 (parametric annotation-position test + exact error-count lock), N3 (move `active_entry!("SIFR-INT-0003", …)` to after the last DECIMAL active entry), and N5 (negative `.sifr` e2e fixture under `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr`) listed as cleanups to bundle with the first new `SIFR-INT-*` active code.
3. Track N4 separately under the INT-2B "no user-facing `bigint`" cleanup as a stance-decision point on reserved-name shadowing.

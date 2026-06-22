I've ignored the deferred MCP/tool reminders — they're harmless and unrelated to this review.

## Follow-up Review: All Prior Findings Addressed

### Finding #1 (defensive crash on missing `expected_result`) — fully fixed
`check_fixture_matrix.py:346-349` now returns immediately after appending the failure, so the two `.startswith` calls below it can never see `None` or a non-string. The exact fix the prior review recommended.

### Finding #2 (path layout not enforced) — fully fixed
`check_fixture_matrix.py:316-319` now constructs `expected_path = Path(side) / f"{matrix_evidence['id']}.sifr"` and rejects anything else. A manifest pointing positive evidence at `negative/foo.sifr` or at a top-level stub is now caught.

### Finding #3 (headers can drift from manifest) — fully fixed
`check_fixture_matrix.py:350-357` now cross-checks `# execution-kind:` and `# expected-result:` against the manifest values, joining the existing `fixture`/`evidence`/`evidence-status`/`expected-diagnostic` gates.

### Finding #4 (lexical "non-empty stub" check) — substantially tightened
`check_fixture_matrix.py:343` now requires `line.lstrip().startswith("@rust")` — a token *inside a comment line* like `# uses @rust(...)` no longer satisfies it (comments start with `#`, not `@`). The "five lines containing the word @rust" loophole is closed for the decorator marker.

### Potential false positives / blind spots in the new tightening
- **None against current data.** The three fixtures previously called out as policy-marker-driven (`cargo_locked_offline`, `proc_macro_trust`, `native_build_script`) all carry real `@rust(...)` declarations at line start in both positive and negative sources, so the stricter `@rust` rule doesn't reject them. Validator still reports ok: 31/10/44.
- **Minor (not actionable):** If `matrix_evidence["id"]` is missing, `expected_path` becomes `Path("positive/None.sifr")` and the error message reads `must be positive/None.sifr`. Confusing, but a separate `id is required` failure is already emitted upstream by `_validate_evidence`, so this only fires alongside that one.
- **Minor (not actionable):** `# execution-kind:` / `# expected-result:` use `in text` substring matching, same as the existing `fixture`/`evidence` headers — a duplicate occurrence elsewhere in the file would still satisfy the check. Consistent with the prior contract, not a regression.
- **Minor (not actionable):** `startswith("@rust")` would also match a hypothetical `@rusty(...)`. No such decorator exists today; no real risk.
- **Unchanged caveat:** the `≥5 stripped lines` body-size heuristic is still lexical; combined with the new `@rust`-at-line-start gate, the realistic empty-stub attack is gone, but the count itself can still be padded with blank-ish lines. Not worth tightening further at the contract tier.

### Verdict
No remaining actionable findings. The diff is shippable as-is; the validator is now meaningfully harder to mislead than at the time of the prior review.

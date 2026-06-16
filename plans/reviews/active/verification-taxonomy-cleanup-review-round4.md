I have enough to write the review. The originally-cited B1/B2 items (generator + renamed baseline report) are genuinely fixed. But the broader B1/B2 *class* — milestone taxonomy in active stdlib data — recurs in forms the enforcing checker is structurally blind to.

---

## Follow-up review: verification taxonomy cleanup

### B1 / B2 (original items) — RESOLVED ✅
- **B1**: `generate_concurrency_runtime_inventory.py` has no `write_m0_traceability_md` and no `M0`/`milestone` tokens; now uses `write_baseline_traceability_md`, `BASELINE_TRACEABILITY_MD`, `BASELINE_DECISIONS`, `contract_backlog`, `owner_contract`. Clean.
- **B2**: `concurrency_runtime_baseline_traceability.md` body no longer says "M0 Traceability / Milestone / M0 Closure Gate"; the only heading match is the corrected `## Baseline Closure Gate`. Regenerated `substrate_inventory.md`, `cpython_evidence_matrix.md`, `workload_database.md`, `baseline_traceability.md` are all clean.

### Blocking findings

**BLOCKING-1 — The taxonomy checker has a regex blind spot, and it is masking residual milestone tokens in active stdlib data the cleanup claims to have converted.**

`verification_taxonomy.py` `TEXT_PATTERNS` catch the *suffix-underscore* form (`concurrency_runtime_m7`) and the *m-then-hyphen* form (`m7-foo`), but **miss two forms that are actually present in scope**:
- `m<digit>_…` prefix form
- `…-m<digit>` hyphen-suffix form

Empirically verified against the real patterns:
```
ESCAPES: m5_closure_evidence
ESCAPES: m24_5_check / m25_cfg_repeat / m25_5_check
ESCAPES: blocked-on-concurrency-runtime-m1  (…-m6)
FLAGGED: concurrency_runtime_m7  /  m7-foo
```

Because of this gap, residual delivery-plan taxonomy survives in active stdlib data files that this cleanup explicitly lists as "converted" (network_http, text_i18n), yet the checker still reports pass:

- `verification/areas/stdlib_parity/data/network_http_substrate_inventory.json:209-243` — **9** `"state": "blocked-on-text-i18n-m1/2/3"` and `"blocked-on-concurrency-runtime-m1/2/3/5/6"` values. These are direct milestone references (text_i18n M1–M3, concurrency M1/2/3/5/6).
- `verification/areas/stdlib_parity/data/text_i18n_substrate_inventory.json:319` — `"m5_closure_evidence"` key (= milestone-5 closure evidence). The same file's `owner_milestone` fields *were* converted in this diff, so the conversion stopped exactly at the form the checker can't see.

This is a recurrence of the B1/B2 class (milestone taxonomy on active stdlib surfaces). The "checker passes" validation does not actually guarantee these surfaces are clean — fix the patterns (add `\bm\d+[_-]` and `[_-]m\d+\b` coverage with the existing allowlist), then re-run; the residues above must be renamed to contract/dependency terminology.

**BLOCKING-2 — Half-converted manifest within an active root touched by this diff.**

`verification/areas/core_language/data/validation_contracts/manifest.json` had its group names de-milestoned in this diff (`phase24_hir_analysis` → `hir_analysis_contracts`, `phase25_cfg_flow` → `cfg_flow_contracts`), but **19 child command ids retain milestone taxonomy**: `m24_5_check`, `m24_5_build`, `m25_5_check`, `m25_cfg_repeat`, etc. (`validation_contracts` is line 21 of `ACTIVE_ROOTS`). Same checker blind spot is why this passed. Either convert the ids too (consistent with the parent rename) or justify why they're retained — leaving the parent renamed and children milestone-named is an internally inconsistent state.

### Non-blocking concerns

- **Checker self-test doesn't cover the prefix/hyphen-suffix m-number forms.** `run_self_test` only exercises `Phase 99` (space) and `milestone_99` (filename). Once BLOCKING-1's patterns are added, add `m5_closure_evidence`-style and `blocked-on-x-m1`-style strings to the self-test corpus so the gap can't silently reopen.
- **Provenance of the residual files.** Confirm whether `network_http_substrate_inventory.json` / `text_i18n_substrate_inventory.json` are hand-maintained or generator-produced. If generated, the generator (not just the JSON) must be the fix site — otherwise the next regeneration reintroduces the tokens, exactly as the original B1 generator did.
- Bare `M5`/`m5` standalone tokens (no separator) are also uncatchable by the current patterns; not observed in scope, but worth a guard if you touch the regex set.

### Verdict

**Not satisfied.** The two specific items you fixed (B1 generator, B2 baseline report) are correct and verified. But being strict on the B1/B2 class and active stdlib data/tool coverage as requested: milestone taxonomy still lives in `network_http_substrate_inventory.json` and `text_i18n_substrate_inventory.json` (both in the "converted" list), the enforcing checker is structurally blind to the forms used there, and a co-edited active manifest (`validation_contracts`) is half-converted. Address BLOCKING-1 and BLOCKING-2 (checker patterns + the concrete residues + re-run), then this is good to go.

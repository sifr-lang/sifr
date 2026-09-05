## Review findings

Resolution status: actionable active-plan URL references were updated in this PR after this review.

**Diff is internally consistent.** Canonical owner `DiagnosticCode::docs_url()` (registry.rs:305) is updated, all 26 baseline files, 7 hardcoded test URLs across `sifr`/`sifr_driver`, 8 entries in `sifr_lint::RULES`, the JSON manifest, both policy checks, the registry test, and `internal_docs/architecture.md` all moved to `https://docs.sifr.sh/errors/<CODE>` with no path/casing drift. Runtime emission still uses `code.docs_url()` at `crates/sifr_diagnostics/src/render/mod.rs:148` and `crates/sifr_lint/src/lib.rs:624`, so the canonical-owner contract is preserved. The public `docs/` site already canonicalizes to `https://docs.sifr.sh` (`docs/docs.json:7`), so root-relative `/errors/<CODE>` links in `docs/diagnostics/error-codes.mdx` resolve under the new compiler URL.

### Actionable: active plan/contract docs still cite the OLD URL

These are **not in the archive** the user excluded, and per CLAUDE.md they are tracking docs that should be updated when the contract changes. A reader auditing the diagnostic URL contract from the roadmap/phase plan would still see the old form and could legitimately flag the code change as a contract violation:

- `plans/roadmap.md:28` — global rule states the URL as `https://sifr.sh/docs/errors/<CODE>`. Mirrors `internal_docs/architecture.md:887` which was updated; the roadmap should match.
- `plans/phases/27_diagnostics_error_recovery_and_stability_contract.md:25` and `:84` — active phase plan that defines this URL contract.
- `plans/phases/38_docs_and_documentation.md:24` and `:53` — active phase plan referencing the URL for diagnostic-page publishing.
- `plans/reviews/active/ad-hoc-documentation-learning-path-agent-reference-review-pass-1.md:17` — active review; also note its inline line number (`registry.rs:287`) has drifted to `registry.rs:305`.

Recommend updating these in this PR (or a fast follow-up) alongside `internal_docs/architecture.md`.

### Non-blocking observation

`sifr_lint::RULES[].docs_url` (8 hardcoded strings, `crates/sifr_lint/src/lib.rs:177-254`) and the JSON manifest duplicate values that exist as `DiagnosticCode::LINT_*` (registry.rs:145-153). The verification gate only asserts the prefix shape (`check_linter_reuse_rules.py:248`), so values can silently diverge from `DiagnosticCode::docs_url()`. This diff bumped 17 strings that ideally would be one. The user's scope explicitly accepted this duplication, so flagging only as a future cleanup (derive `docs_url` from a `DiagnosticCode` field on `RuleMetadata`, then the next URL change is one line).

### Validation

Local validation reported above is sufficient. The advisory `warm wall-time budget exceeded` with zero e2e cache hits is the expected consequence of touching 26 fixture baselines (cache invalidation), not a regression. No blockers.

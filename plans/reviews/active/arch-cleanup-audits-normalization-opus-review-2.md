I have enough to write the review.

---

## Review: PR N+4 audits normalization — round 2

### Fix verification (round 1 findings)

1. **(High → fixed) Suites wired into profile runner.** `verification/runner/sifr_verify/profile_runner.py:234-237` now runs `audit-fixtures` for `core_language`, `project_workspace`, and `stdlib_parity` inside `run_core_guardrails`, plus the new `check_audits_normalization.py` guardrail at `profile_runner.py:230-232`. Manual-only execution concern resolved.
2. **(High → fixed) `intrinsic-panic-lint` is no longer a strict downgrade.** `verification/areas/generated_code_quality/generated_code_quality.py:825-841` keeps the monolith-absence check and adds `scan_codegen_source_emissions` (lines 534-546), which walks every non-test file under `crates/sifr_codegen/src` and reports forbidden constructs (`unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unsafe`, `#[allow(...)]`) found inside Rust string literals on lines that also reference a `GENERATED_SOURCE_CONTEXT_RE` emitter (`format!`, `emit_line`, `RustExpr::Ident`, `RustType::Named`, `RustLiteral::Str`, `push_str`). Combined with output-side `panic-scan` (line 675) and the monolith-absence check, source-side coverage now exceeds what `audits/lint_panic_patterns.sh` enforced.
3. **(Medium → fixed) Obfuscated `"audits" "/leetcode"` removed.** `scripts/check_submodule_ownership.py:67,191-192` use plain string literals again; `scripts/check_audits_normalization.py:26-29` excludes it via `REFERENCE_SCAN_EXCLUSIONS`. Cleaner and self-documenting via the explicit exclusion set.
4. **(Low → fixed) Duplicate-path detection added.** `scripts/check_audits_normalization.py:80-83` flags in-manifest duplicates with a specific message, and `verification/runner/sifr_verify/audit_fixtures.py:98-104` mirrors it plus a `relative_to(fixture_root)` containment check.
5. **(Nit → fixed) Unused `Any` import removed** from `scripts/check_audits_normalization.py:10`.

Spot-checked: `find audits -type f` returns nothing, manifests are entries=186/5/10 (matches 201 on-disk fixtures), `cpython_parity_fixture_format.md` correctly lives under `verification/areas/stdlib_parity/docs/`, no top-level `audits/` reference appears in `scripts/`, `verification/`, `.github/`, `AGENTS.md`, or `README.md` outside of the documented exclusions.

### Blockers

None.

### Non-blocking findings (carried/new)

- **Stale `audits/` references in phase/issue docs** (carried from round 1, Finding 4): `plans/phases/04_language_hardening.md:340,349,359,372,378`, `plans/phases/05_borrow_by_default.md:156,163,165,237`, `plans/phases/06_stdlib_architecture.md:178,250,354`, `plans/phases/09_stdlib_safety_remediation.md:131`, `plans/phases/30_reliability_parity_and_performance_budgets.md:26,128` still mention retired `audits/*` paths. `ACTIVE_REFERENCE_ROOTS` deliberately excludes `plans/` and `internal_docs/`, so these are not load-bearing and PR N+5 (Internal Docs Relevance Cleanup) is the intended home. Worth flagging in the PR description to set expectations.
- **`--self-test` is regex-only** (carried, Finding 6): `scripts/check_audits_normalization.py:147-160` exercises `TOP_LEVEL_AUDITS_REF_RE` but not `validate_manifest`. A regression in the manifest validator passes `--self-test`. Same for `verification/runner/sifr_verify/audit_fixtures.py` (no self-test). Defensive only.
- **`path_counts` double-increments on in-manifest duplicates** (carried, Finding 5): `scripts/check_audits_normalization.py:83` increments even after the in-manifest duplicate detected at line 80, so a same-manifest duplicate fires both "lists duplicate fixture path" and "owned by multiple manifests" messages. Cosmetic — the validator still fails correctly.
- **Multi-line `format!()` literals are not source-scanned** (new, Finding 2 nuance): `scan_codegen_source_emissions` (line 534-546) splits on newlines and matches `GENERATED_SOURCE_CONTEXT_RE` plus string literals per-line. A `format!(\n    "...unwrap()...", x\n)` where the literal is on a different line than `format!` would slip past the source scan. The output-side `panic-scan` in profile runs still catches the emitted construct, so this is a secondary-defense gap, not a regression.
- **Warm wall-time advisory** (new): `create-pr` profile crossed warm budget (152.28s vs 120s budget) per the prompt's report. The three added smoke runs in `run_core_guardrails` (`cargo run -q -p sifr -- check ...` per smoke fixture × 9 smokes) plausibly add 6-10s warm. Advisory, not blocking, but worth tracking that the new audit gate doesn't compound future runtime pressure.
- **Double `[sifr-case-timing]` for stdlib_parity audit-fixtures** (carried, Finding 8): `verification/runner/sifr_verify/audit_fixtures.py:155-158` and `verification/areas/stdlib_parity/runner.py:164-169` both emit a case-timing line; labels differ enough to be distinguishable downstream. Cosmetic.

### Verdict

Satisfied — open the PR. The two High findings from round 1 are properly resolved, the Medium obfuscation is gone with a self-documenting exclusion, and the remaining items are either deferred to the documented follow-up PR or non-load-bearing.

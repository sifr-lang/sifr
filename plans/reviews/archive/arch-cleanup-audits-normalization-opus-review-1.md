Review complete. The diff is read-only — I'll summarize findings now.

---

## Review: PR N+4 Audits Normalization

### Phase requirement satisfied?
- **No top-level `audits/`**: ✅ Confirmed gone (`find audits -type f` returns nothing); guardrail (`scripts/check_audits_normalization.py:33-34`) actively rejects its return.
- **Retained audit fixtures are manifest-owned**: ✅ 201 fixtures on disk match 201 manifest entries (core_language 186, project_workspace 5, stdlib_parity 10).
- **Audit suites execute through the verification runner**: ⚠️ Suites are *available* in the three area manifests, but no profile selects them. See Finding 1.
- **Historical reports/wrappers removed (not preserved)**: ✅ All `*REPORT.md`, `STDLIB_PARITY_MASTER_REPORT.md`, `run_audit*.sh`, `lint_panic_patterns.sh` are `D` in `git status`. The only retained artifact (`cpython_parity_fixture_format.md`) was moved to `verification/areas/stdlib_parity/docs/`.

### Findings (severity-ordered)

**1. (High) `audit-fixtures` suites run in zero profiles — smoke execution is dead code in CI.**
`verification/profiles/{create-pr,merge,nightly,release}.json` never select the new `audit-fixtures` suite for any of `core_language`, `project_workspace`, `stdlib_parity`. `verification/runner/sifr_verify/profile_runner.py:207-244` invokes only `--suite complexity-resource` for `stdlib_parity` and only `--suite typescript-go-m1`/`--suite guardrails` etc. for the other areas. The only thing wired in is the **structural** guardrail (`scripts/check_audits_normalization.py`) — it validates manifest ↔ disk parity but never invokes `cargo run -p sifr -- check <fixture>`. The 9 declared smoke fixtures (7+1+1) only execute when a contributor manually runs `uv run --project verification --locked python -m sifr_verify areas run --area <area> --suite audit-fixtures`. The phase says "audit area executes through the runner" — by letter this is satisfied (a path exists), but in practice no CI/local profile exercises it, so a fixture regression is invisible until someone manually invokes the suite. Either wire `audit-fixtures` into `core_guardrails` (cheap: only 9 `cargo check` runs total) or document explicitly that smoke execution is contributor-on-demand.

**2. (High) `intrinsic-panic-lint` is a strict downgrade vs. the old `lint_panic_patterns.sh`.**
`verification/areas/generated_code_quality/generated_code_quality.py:781-794` checks one thing: that `fn emit_intrinsic_call(` does not appear in `crates/sifr_codegen/src/lib.rs`. The old script (`git show HEAD:audits/lint_panic_patterns.sh`) scanned the *body* of `emit_intrinsic_call` for `.unwrap()`, `.expect(`, `panic!(`, `unreachable!(` with `SAFETY:`/`COMPILER-INTERNAL:` exemptions. The new gate does NOT scan any function body for panic patterns — it only enforces that one named monolith doesn't return to `lib.rs`. Any sibling/replacement function that adopts `.unwrap()` will pass. Partial compensations:
- `gate_panic_scan` (`generated_code_quality.py:631`) scans *generated Rust output* for forbidden patterns (stronger than the old source-level scan, since it catches the actual user-facing emission regardless of which emitter function produced it).
- A Rust unit test at `crates/sifr_codegen/src/intrinsic_method_emitters/narrowing_helpers.rs:31` covers a separate file (`intrinsic_method_emitters.rs`), not `lib.rs`.

So output-side coverage is preserved. Source-side scanning is gone. If the design intent was "we trust output-scanning to be sufficient now," fine — but call it out, because the gate name `intrinsic-panic-lint` implies the original semantics.

**3. (Medium) Obfuscated string-concat workaround in `scripts/check_submodule_ownership.py` will rot.**
`scripts/check_submodule_ownership.py:67` and `:191-192` were rewritten as `"audits" "/leetcode"` and `f'[submodule "{"audits"}/leetcode"]\n'` solely to evade the audits-normalization regex. The `{"audits"}` f-string interpolation does nothing at runtime (`"audits"` literally evaluates to `"audits"`). No comment explains why these constructs are intentionally awkward. A future maintainer will "clean it up," reintroducing `"audits/leetcode"`, and the audits-normalization guardrail will then fail on this file. Cleaner fix: add `scripts/check_submodule_ownership.py` to `iter_reference_files`'s exclusion list in `scripts/check_audits_normalization.py:121` (the same way `check_audits_normalization.py` already excludes itself).

**4. (Low) `validate_stale_references` excludes `plans/` and `internal_docs/`.**
`scripts/check_audits_normalization.py:20-26` only covers `.github`, `AGENTS.md`, `README.md`, `scripts`, `verification`. Active phase docs still contain stale `audits/` paths:
- `plans/phases/30_reliability_parity_and_performance_budgets.md:26,128` — references `audits/stdlib/cpython_parity_fixture_format.md` (moved to `verification/areas/stdlib_parity/docs/`)
- `plans/phases/04_language_hardening.md:340,349,359,372,378`, `plans/phases/05_borrow_by_default.md:156,163,165,237`, `plans/phases/06_stdlib_architecture.md:178,250,354`, `plans/phases/09_stdlib_safety_remediation.md:131` — reference `audits/*` paths that no longer exist on disk.

These are documentation references, not load-bearing, and PR N+5 (Internal Docs Relevance Cleanup) is the natural home for the fix. Not a blocker for this PR, but the phase-tracker file `plans/issues/active/ad-hoc-repository-architecture-and-verification-surface-cleanup.md:18,542,548-555` itself contains the same pattern (which is correct — it is describing the work being done).

**5. (Low) Duplicate-path detection has a within-manifest blind spot.**
`scripts/check_audits_normalization.py:67,77-78`: `entry_paths.add(entry_path)` deduplicates inside a single manifest before the disk/manifest diff runs. Two entries with distinct `id`s but the same `path` in one manifest would pass both this validator and the companion at `verification/runner/sifr_verify/audit_fixtures.py:101-107` (which checks ID uniqueness, not path uniqueness). The "owned by multiple manifests" failure message at `check_audits_normalization.py:41` is also misleading — `path_counts` accumulates across manifests AND duplicates within a single manifest, but the message only mentions cross-manifest. Defensive only; manifests are hand-curated and currently clean.

**6. (Low) `--self-test` only exercises the regex, not the manifest validator.**
`scripts/check_audits_normalization.py:142-155`: the self-test confirms the `TOP_LEVEL_AUDITS_REF_RE` pattern matches `audits/old` and doesn't match a promoted fixture path. It does NOT exercise `validate_manifest` (missing entries, missing fixture root, smoke missing `expect_exit_code`, stray `.md` files). A regression in the manifest validator passes `--self-test`. The companion `verification/runner/sifr_verify/audit_fixtures.py` also lacks a self-test.

**7. (Nit) Unused import.**
`scripts/check_audits_normalization.py:10` imports `Any` but never uses it.

**8. (Nit) Two layers emit `[sifr-case-timing]` for stdlib_parity audit-fixtures.**
`verification/areas/stdlib_parity/runner.py:152` and `verification/runner/sifr_verify/audit_fixtures.py:152` both print a case-timing line for the same execution (one for the case, one per smoke entry). Cosmetic only; the bucket/case fields differ enough that downstream parsers can tell them apart.

### Direct answers

1. **Blocking correctness issues?** No outright blockers. The closest to a real concern is Finding 1 (smoke fixtures aren't run by any profile) — that should be resolved with an explicit decision (wire them in, or document them as on-demand) before this lands as the canonical audit-normalization story.
2. **Does the guardrail enforce no top-level `audits/` and manifest ownership?** Yes — confirmed by perturbation tests: creating `audits/zzz/test.sifr` → FAIL exit 1; dropping an unmanifested `.sifr` under a fixture root → FAIL exit 1; dropping a stray `*.md` under a fixture root → FAIL exit 1; planting `audits/` in a `scripts/*.md` → FAIL exit 1.
3. **Is the `lint_panic_patterns.sh` replacement sufficient?** Mixed. Output-side scanning (`panic-scan`) is intact and arguably stronger than the original source-side scan. Source-side coverage of arbitrary panic patterns inside intrinsic emitter helpers is **lost** — the new gate only checks that one named monolith hasn't returned in one specific file. See Finding 2.
4. **Submodule/external corpus accidentally modified?** No. `git submodule status` shows the same SHAs as before; `git submodule foreach 'git status --short'` is clean across all 9 submodules. No file under `verification/areas/algorithmic_compatibility/corpora/leetcode/` (a submodule) is in the change set; the two `audits/` references inside that submodule are correctly skipped by `is_under_submodule`.
5. **Missing validation steps before PR?** The local validation listed in the prompt is comprehensive. One advisory item to confirm: the create-pr advisory budget miss (wall_time=334s, budget warm/cold = 2m/5m) — the budget docs say create-pr warm budget is 2 minutes, so 334s is over both warm and cold. Worth confirming this isn't a regression introduced by this PR (the new audits guardrail step is cheap; the advisory is more likely from upstream changes or cold-cache).

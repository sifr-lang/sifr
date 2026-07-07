I have the complete picture. Let me write up the review.

---

## Review of M5 Closeout — Sysroot Stdlib Interop Declaration Cleanup

### Findings (ordered by severity)

**1. [Blocker for merge] Empty Opus review artifact staged for the PR**
`plans/reviews/active/ad-hoc-sysroot-stdlib-interop-declaration-cleanup-m5-opus-review-pass-1.md` is a 0-byte untracked file. It should either be populated with this pass's review notes before committing, or removed. Committing it empty leaves a broken artifact under `plans/reviews/active/` and misrepresents the review evidence the phase doc points at.

**2. [Blocker for merge] 32 code files carry uncommitted changes that the PR body describes as "already applied"**
Only the single commit `538731733` (docs) is ahead of `origin/main`. The lint-equivalent cleanup diff (`BuildReportInput`, `source_diagnostic` signature change, `expect_used` fixes, `format_push_string` rewrites, `Option<&String>` refactors, etc.) is entirely in the working tree, unstaged. `scripts/run_all_tests.sh --profile create-pr` therefore validated the local worktree, but nothing on the branch head reflects those code changes. These must be committed to PR #2818 before merge; otherwise reviewers on GitHub see only a docs update while the closeout notes and validation evidence describe compiler code changes.

**3. [Actionable] Semantic-flavored change bundled into "lint-equivalent" bucket: panic → diagnostic**
`crates/sifr_driver/src/build/rust_interop.rs:182` replaces `packages.get(&self.context.package_id).expect("…")` with a `let-else` that emits `DiagnosticCode::INTERNAL_COMPILER_PANIC` and returns `Err(vec![…])`. This is driven by workspace `expect_used = "warn"` (`Cargo.toml:176`) + `-D warnings`, so it IS lint-required, but the replacement path changes error semantics: what was a compiler abort is now a rendered diagnostic that survives to the caller. The invariant almost certainly still holds, so nothing changes in practice; still, this behavior swap deserves a one-line mention in the M5 closeout notes so future readers know the panic surface moved.

**4. [Actionable] TOML escaping refactor is not purely lint-equivalent**
`crates/sifr_driver/src/build/rust_interop_probe.rs:159,166,185` swaps `format!("… {backend_root:?} …")` for a new `toml_quote_path` helper wrapping `toml_quote_string`. The prior `{:?}` on `&Path` used Rust's OsStr Debug escaping; the new helper is a purpose-built TOML string escaper. For typical ASCII sysroot paths the output is byte-identical (validation passed), but escaping semantics for backslashes / quotes / control chars differ. A pure `format_push_string`/`uninlined_format_args` fix does not require swapping the quoting mechanism. This is arguably a correctness improvement (safer TOML for pathological paths), but it should be called out as such rather than filed under "lint-equivalent cleanup."

**5. [Actionable] Phase-doc scope drift for M5**
The "Compiler surfaces expected to change" inventory (lines 104–117 of the phase doc) was written for M1–M3 and never updated to acknowledge that M5's lint sweep touches ~20 additional surfaces outside that list (`sifr_analysis/*`, `sifr_lsp/*`, `sifr_package/projection_bridge`, `sifr_sysroot/error`, `sifr_stdlib_model/sources`, `sifr_frontend/graph_cache_and_queries/loaders`, driver `build/{entrypoint,materialize,report,mod}`, etc.). Either amend the inventory or add a "M5 lint sweep also modified…" sentence in the Closeout Notes so the doc matches the diff.

**6. [Actionable] Closeout Notes claim "Full validation: not run for M5"**
That is consistent with AGENTS.md's stance that create-pr is the authoritative gate, but the M5 diff is not purely docs — it changes public API shape in `sifr_driver::build::report` (adds `BuildReportInput`, changes `BuildReport::new` arity), rewrites `source_diagnostic` to take owned `Vec` args, and moves a compiler `expect` to a diagnostic. Given the breadth, either run the full-gate profile once and record the summary, or explicitly justify why the create-pr profile suffices given these API-shape changes. Right now the doc simply notes the full profile wasn't run.

**7. [Non-blocker] Two `#[allow(clippy::too_many_arguments)]` additions instead of refactor**
`crates/sifr_driver/src/build/rust_interop.rs` adds allow-attributes on `require_trust` and `push_diagnostic`, and `rust_interop_diagnostics.rs` adds one on `source_diagnostic`. Given that `BuildReport::new` was refactored to accept `BuildReportInput`, applying the same struct-argument pattern to these three functions would keep the treatment consistent. Not a blocker — allow-attributes are legitimate — but worth noting for consistency.

**8. [Non-blocker] Roadmap crosslink asymmetry**
`plans/roadmap.md:80` (row 39.1) now says "private stdlib declaration syntax cleanup is tracked by … with final closeout in PR #2818" while the phase doc itself still records M5 as "in progress" (line 73). The roadmap talks in the past tense; the phase doc still open. Reconcile after merge: either both should say "in progress until PR #2818 lands" or both flip after the merge. Minor doc consistency issue.

### Open questions / assumptions

- **Assumption**: The 32 uncommitted files are the payload the reviewer intends to add to PR #2818 before merge. If instead they are already pushed and were merely not fetched into this worktree, disregard finding #2 — but that seems unlikely since the branch head has only the docs commit.
- **Question**: Was PR #2818 opened against `origin/main` or against a stacked branch? If stacked, the "32 files with uncommitted changes" may already live on a parent branch and only the docs commit is intentionally isolated here.
- **Question**: Does `check_typescript_go_transfer_guardrails.py` need re-baselining for the `rust_interop_probe.rs` line-number shifts, or was the doc update to lines 49/68/210 sufficient? The guardrails passed, so presumably yes — flagged only because a manual line-number bump inside a guardrails table is fragile.

### Final verdict

**Not satisfied** — the phase is technically ready but two blockers stand between it and closure:
1. The empty Opus review artifact must be populated or removed before the PR merges.
2. The M5 code cleanup (32 files) must be committed to the branch; right now only the docs commit is present, so `origin/main..HEAD` does not reflect what was validated.

Findings #3–#8 are advisory: address them by amending the closeout notes and, ideally, refactoring the two remaining "too_many_arguments" callsites. Once #1 and #2 are resolved and the closeout notes acknowledge the two non-strict-lint changes (panic→diagnostic, TOML escaping), I am satisfied for phase closure.

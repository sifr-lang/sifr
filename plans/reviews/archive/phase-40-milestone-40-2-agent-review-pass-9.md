## Review: PR #3030 — Phase 40 / milestone 40.2, head `939e69083`

### Identity
Local `HEAD` = `939e69083e3e8d8c5cb98efe6e1d3e90bc3753e9`; `gh pr view 3030` reports `headRefOid` identical, base `main`, state OPEN, `MERGEABLE`/`CLEAN`. Working tree clean except my own untracked pass-9 placeholder (not in the PR). Reviewed the complete 86-file diff from merge-base `56f8c41ee`.

**Delta since approved pass-8 head `28fe8527f`:** 2 files, +80 lines, additions only, both documentation — the archived pass-8 review and the tracker ledger entry. No functional, schema, workflow, or script byte changed. I diffed this explicitly rather than trusting the commit subjects.

### Pass-7 findings — independently reverified closed
1. **HIGH, GA-blind planner** — `create_new_version.sh:124-154` derives `site_default_channel` from index `ga_status`, requires `stable` only when `active`, validates `stable` if-present. Closed.
2. **MEDIUM, hardcoded stable index default** — `validate_self_update_metadata.sh:157-163` now branches `expected_index_channel` on `metadata_ga_status`. Closed.
3. **MEDIUM, unschematized artifact** — now `schema_version: 2` / `sifr-site-publication-binding-v2`, checked-in schema, rejecting validator, registered `--kind`, runner inventory at 12 schemas (confirmed: 12 files on disk). Closed.
4. **LOW, zero digest** — rejected in producer (`:32-33`), governance, and schema (`not:{const}` on every digest). Closed.
5. **LOW, missing evidence** — recorded at `:377-384`. Closed.
6. **LOW, whitespace/index** — `git diff --check 56f8c41ee HEAD` exits 0; `PERF-HOST` indexed at `plans/phases/index.md:51`. Closed.

### Independent verification
**Schema/validator parity** — field-by-field match between `site_publication_facts.schema.json` and `governance/site_publication.py`, including `additionalProperties:false` ↔ `require_exact_keys` and zero-digest rejection on all seven digest fields.

**Live cross-repository pinning (re-queried against GitHub, not the fixture):** tag → `07d88cc3c24707e386c5ad73fb0875c06ffd598f`; ruleset `19791667` = `target:tag`, `enforcement:active`, `bypass_actors:[]`, rules `{update,deletion}`, include exactly the one tag, exclude `[]`, `updated_at` 05:06:21.354Z; site workflow bytes at that commit hash to `7a27abaf…958`, matching `SITE_WORKFLOW_SHA256` and the fixture.

**Workflow ordering / write-once** — `--clobber` appears exactly once (`:460`, `channels.json`); snapshot upload (`:452`) → index replacement (`:458`) → dispatch (`:618`); snapshot-name collision rejected (`:448`); `gh release create` targets `SOURCE_COMMIT` with no clobber. Both workflows reject `stable` input (`preview-release.yml:11-13` enum; `release-publication.yml:76-79`).

**Install/self-update integrity** — `version_pin` passes the anchored SemVer regex before reaching any `sed` pattern or the installer URL, closing injection; zero installer digest rejected; SHA-256 verified before `chmod +x`; in Rust `resolve_exact()` precedes the force check (`self_update_metadata.rs:372`) and `validate_installer()` precedes `make_executable()` (`self_update_runner.rs:46-47`). Every `expect(` in the added Rust is inside test code.

**Scope** — no demo filename carries phase/milestone numbering or the phase name (sole new demo: `demos/stable_self_update_demo.sh`); zero Rust-interop implementation (only the five self-update files plus a `sha2` dependency edge).

**Executed here:** distribution area 104/104 variants, 0 failures; `full` + `evidence-custody` = 52 + 1 = **53/53, exactly reproducing the tracker's claim**; `developer_tooling --suite editor-release` = **6/6, exactly reproducing the tracker's claim**; 49/49 self-update tests; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --check` clean; HIR guardrails PASS; no changed non-Markdown file over 900 lines.

### Assessment of the performance-budget evidence
Sufficient, and I verified the substance rather than accepting the decision on its face:
- **No baseline, budget, threshold, or waiver file is touched anywhere in the diff** — the only path matching that grep is the `PERF-HOST` follow-up plan itself.
- **The milestone cannot plausibly move the failing medians.** The overruns are on check/diagnostic/LSP benchmarks. The only compiled-source changes are four self-update modules reachable solely through the `self-update` subcommand, plus a `sha2` edge — and `sha2 0.10.9` was **already in `Cargo.lock`**, so this adds no crate, only a dependency edge. Nothing in the compile/check/diagnostic/LSP hot path changed.
- Independent parent-main reproduction is documented in `plans/issues/active/adhoc_performance_budget_host_variance.md`, the retry overruns (0.55%/0.69%) sit inside the 0.5–2.0% band that doc records for host variance, and `PERF-HOST` is indexed as a deferred non-prerequisite by explicit project/user decision.

This is host variance routed to an indexed follow-up, not a milestone regression. The two tracker counts I could reproduce exactly matched, which raises my confidence in the run records I could not rerun in reasonable time.

### Findings
None. No actionable correctness, security, governance, parity, install/self-update, ordering, pinning, test, documentation, scope, or evidence defect.

Two non-blocking observations, carried forward from pass 8 and still accurate — neither is a finding: the planner's post-GA precondition will require the site source to land a committed `stable` dispatcher at GA activation (unreachable today, arrives in milestone 40.5), and `cargo clippy --workspace --all-targets` fails in `sifr_lowering`, which this PR does not touch and which is not part of the AGENTS.md gate.

---

## Verdict

**APPROVED** for merging PR #3030 at head `939e69083e3e8d8c5cb98efe6e1d3e90bc3753e9`.

# Wave 4 Source Import Compact Baselines — Review Pass 1

**Verdict: no blockers. Ready for PR after deciding on the optional cleanups below.**

The slice does what it claims: four `SIFR-IMPORT-000{4,5,6,7}` rendered baseline deferrals are cleared in `code_baseline_coverage.json`, four corresponding compact-only diagnostics fixtures land under `verification/areas/diagnostics/fixtures/diagnostics/source_import_*/`, the manifest gains four cases, and `baseline_metadata.json` gains four entries with correct source hashes. I reproduced every claim end-to-end.

## Blockers

None.

I verified:

- **Each claimed code is actually rendered by its claimed baseline.** Reproduced all four `cargo run --locked -q -p sifr -- --diagnostic-format compact check <main.sifr>` runs from a clean cwd; stdout/stderr/exit-code each match the recorded baseline byte-for-byte. Codes asserted: `SIFR-IMPORT-0004` (private_member), `SIFR-IMPORT-0005` (ambiguous_module), `SIFR-IMPORT-0006` (namespace_collision), `SIFR-IMPORT-0007` (cycle). Each baseline carries exactly one diagnostic — no incidental codes leaking in.
- **`source_hash` is credible.** Recomputed sha256 over each fixture's `main.sifr`; all four match the values recorded in `baseline_metadata.json` exactly. The hashes pin the entry file, which is what `code_baseline_coverage.py` checks at line 304.
- **Normalizers are credible.** Each new metadata entry uses `["workspace-path","tmp-path","crlf","artifact-cache-lines"]` — all valid per `NORMALIZERS` in `code_baseline_coverage.py:26`, and correctly omits `json-sort` (compact-only). Sample stderr renders repo-relative paths (`verification/areas/diagnostics/fixtures/...`), not absolute filesystem paths, so the workspace-path normalizer is doing work as expected.
- **Manifest ownership is correct.** Each new metadata entry sets `owner: "compiler/frontend"`. That owner exists in `verification/owners.json` (team-style owner per the phase doc's "Decisions" section). The catalog already lists the four codes as `stable` with `Error` severity, `renderer_support: ["human","json","compact"]`, and `docs_link: docs/errors/SIFR-IMPORT-000{4..7}.md` (verified all four doc files exist).
- **Phase tracking is honest.** Counts reconcile: slice-3 closed at 97/170 active codes with 73 deferrals; slice-4 reports 101/170 with 69 deferrals. Δ=+4/-4, matches the four new fixtures. The note that "the `IMPORT` family no longer has Wave 4 rendered-baseline deferrals" is true — no other `SIFR-IMPORT-*` rows remain deferred in `code_baseline_coverage.json`. The slice-3 doc had explicitly named `SIFR-IMPORT-0004..0007` as four of the "10 semantic stragglers without current e2e fail fixture evidence," and this slice closes exactly those four via purpose-built fixtures rather than copy-promotion of e2e-fail — which is the right call given the surface (source import resolution needs project structure, not single-file inputs).
- **No incidental files unsuitable for PR.** The four fixture trees contain only `main.sifr`, the auxiliary `.sifr` modules each diagnostic needs to exercise its resolution path, three `sifr.toml` files (for the project-style cases), and the `baselines/` triplet (`check-compact.{stdout,stderr,exit-code}.txt`). No editor artifacts, target/, or stale baselines. `git diff --check` is clean (per user). `code_baseline_coverage.py`'s baseline-file scan would catch any orphan `*.txt` under `baselines/` (lines 277-280) — none present.
- **Slice 3's pass-1 follow-up check applies cleanly here.** The "claimed code must literally appear in the baseline stderr" check (`validate_coverage_baseline_evidence`, lines 226-242) fires on each new row: I confirmed every claim by grep, and `cargo test -p sifr_diagnostics` plus the diagnostics area suites passed (per user's focused validation).

## Non-blocking findings

**1. `source_import_private_member` lacks its own `sifr.toml`; the other three fixtures have one.**

The fixture has `main.sifr` + `local_math.sifr` only. With no fixture-local `sifr.toml`, `find_workspace_root` (`crates/sifr_driver/src/workspace/mod.rs:31`) walks up six levels and matches the repo-root `./sifr.toml`. That root declares `[package] name = "sifr-workspace"` and `[source] roots = ["verification/areas/algorithmic_compatibility/corpora/leetcode/src", "."]`. The fixture *happens* to render the right diagnostic because `ModuleResolver` (`project/discovery.rs:97`) tries `entry_parent` first and finds `local_math.sifr` next to `main.sifr` before ever consulting workspace roots — so `SIFR-IMPORT-0004` fires correctly.

Risk: the fixture is silently coupled to whatever `./sifr.toml` declares. If the repo-root manifest is restructured, or if any ancestor directory between `source_import_private_member/` and the repo root gains a `sifr.toml`, the fixture's workspace context changes without the fixture itself changing. The other three fixtures don't have this implicit coupling because they each ship their own `sifr.toml`.

Fix: drop a minimal `sifr.toml` next to `main.sifr` with `[source]\nroots = ["."]\n`. That makes the fixture self-contained and matches the other three. I'd lean toward applying this before merge — it's a one-line file that removes the only fragile dependency in the slice.

**2. Three of the four fixtures are byte-identical copies of existing `project_workspace` fixtures.**

Confirmed via `diff`:
- `source_import_cycle/{main.sifr,a.sifr,b.sifr,sifr.toml}` = `project_workspace/fixtures/project/import_cycle_source_spans/*`
- `source_import_ambiguous_module/{main.sifr,sifr.toml,src_a/helper.sifr,src_b/helper.sifr}` = `project_workspace/fixtures/project/workspace_ambiguous_import_canonical/*`
- `source_import_namespace_collision/{main.sifr,sifr.toml,src/helper.sifr,src/helper/value.sifr}` = `project_workspace/fixtures/project/workspace_namespace_collision_canonical/*`

This re-raises slice 3's review-pass-1 finding #3 verbatim, just at the project-workspace boundary instead of e2e-fail. A future edit to the `project_workspace` copy (rename, restructure, refactor for an unrelated reason) will only fall out of sync with the diagnostics copy if the *rendered compact output* also changes — silent drift in cases where code/position/message survive. The diagnostics `source_hash` pins the diagnostics copy but doesn't link to the project_workspace original.

Mitigations (pick at most one, non-blocking either way):
- Add a `source_origin` field to `baseline_metadata.json` for these three fixtures pointing at the project_workspace path, plus a `code_baseline_coverage.py` check that the two trees are byte-equal when `source_origin` is set.
- Replace the copies with `symlink`-like manifest references (a fixture-id pointing at the upstream path), if the area runner allows entries outside the diagnostics fixture root.

Leave open if Wave 4 plans purpose-built source-import fixtures later that are intentionally diverging from the project_workspace canonicals.

**3. `source_hash` only pins `main.sifr`, not the auxiliary fixture inputs.**

`validate_baseline_metadata` (`code_baseline_coverage.py:304`) hashes only `case["entry"]`, which is `main.sifr`. For these new project-style fixtures the diagnostic actually depends on auxiliary files (`a.sifr`, `b.sifr`, `src_a/helper.sifr`, `src_b/helper.sifr`, `src/helper.sifr`, `src/helper/value.sifr`, and the three `sifr.toml` files). If a contributor edits one of those without touching `main.sifr`, the metadata `source_hash` stays valid even though the fixture inputs changed. Baseline replay will still catch drift through stderr diff, but the metadata-level "stale source" signal is partial.

Generic gap, not specific to this slice. But this slice is the first to introduce project-style fixtures in the diagnostics area, so it's the natural place to either: (a) document the limitation in the metadata schema, or (b) extend `source_hash` to a `source_inputs_hash` covering every `*.sifr` and `sifr.toml` under the fixture directory. Defer if you prefer to do it once across all areas; flag in the PR description either way.

**4. Manifest case ordering breaks the alphabetical convention used by prior `e2e_*` entries.**

The existing case list under the `baselines` suite is sorted by `id` (verified e2e_async... → e2e_yield...). The four new entries are inserted in declaration-discovery order, not sorted: `source_import_private_member`, `source_import_ambiguous_module`, `source_import_namespace_collision`, `source_import_cycle`. Sorted would be `source_import_ambiguous_module`, `source_import_cycle`, `source_import_namespace_collision`, `source_import_private_member`.

No automated check enforces ordering in the diagnostics manifest (`audit_fixtures.py` enforces it but for a different manifest). Soft inconsistency only; trivial reorder.

**5. `bless_reference` is a placeholder slug (`wave-4-source-import-compact-baselines-pr`).**

Matches the convention of prior slices and was explicitly accepted in slice-3 review pass 1 finding #7. Replace with the real PR URL after open. Not a deviation.

**6. Pre-existing ownership inconsistency carries over.**

`compiler/frontend` is used for these new IMPORT rows, while slice 3 used `compiler/core-language` for similar parsing/lowering codes. Both team owners exist in `verification/owners.json`, but the slice-1 prior reviews already flagged ownership-row inconsistency. No regression from this slice; bundle into a future ownership sweep.

## Answers to your review questions

**Q1. Is each claimed code actually rendered by its claimed baseline?**
Yes. Reproduced byte-for-byte for all four fixtures. Exactly one diagnostic per baseline, codes match the coverage mapping.

**Q2. Are metadata, source hashes, and normalizers credible?**
Metadata schema is correct (schema_version=1, suite=baselines, renderer=compact, owner set, bless_reference and bless_reason set, normalizers are a valid subset). Source hashes recomputed and match. The only credibility gap is finding #3 (auxiliary files not hash-tracked) — a hygiene limitation, not a correctness gap.

**Q3. Is manifest ownership correct?**
Yes. `compiler/frontend` is a valid team owner; `audit_fixtures.py`-style ownership row checks don't apply to the diagnostics manifest, and the area-adapter doesn't reject it. Pre-existing inconsistency between `compiler/frontend` and `compiler/core-language` for parallel families isn't introduced here.

**Q4. Is phase tracking honest?**
Yes. The implementation note at `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:869-874` accurately scopes this as the fourth Wave 4 diagnostics-baseline slice, names the four diagnostics closed, claims 101/170 + 69 deferrals (sums to 170, matches expected total), and explicitly states "The `IMPORT` family no longer has Wave 4 rendered-baseline deferrals" — verifiable by grepping `SIFR-IMPORT-*` deferral entries (none remain). The status header line at line 3 says "Wave 4 source import compact baseline slice locally focused-validated and ready for review," which doesn't overstate.

**Q5. Are any incidental diagnostics or copied project files unsuitable for PR?**
No incidental diagnostics — each baseline emits exactly one error. The copied project files are real (finding #2) but acceptable for PR; the question is whether to add a drift-protection hook before Wave 4 grows further, not whether to block this PR.

**Q6. Production blockers before opening the PR?**
None. The closest-to-blocker concern is finding #1 (no own `sifr.toml` for `private_member`). The fixture works correctly today and will likely keep working, but it carries a hidden coupling to the repo-root manifest that the other three fixtures avoid. Adding a one-line `sifr.toml` is cheap. Apply or defer at your discretion.

## Required follow-up validation

The focused validation set the user already ran (`diagnostics contracts`, `diagnostics baselines` with 99 cases / 127 variants, `cargo test -p sifr_diagnostics`, file-size guardrail, `git diff --check`) is sufficient to characterize this slice. Before opening the PR:

- **Run `scripts/run_all_tests.sh --profile create-pr`.** Required by AGENTS.md before considering any task done. The focused diagnostics-area runs don't cover the warm-budget advisory or e2e cache state that prior Wave 4 slices have measured. Expect the same existing warm-budget advisory; report wall time in the PR description.
- **Run `scripts/run_all_tests.sh`** (full merge) if any merge-gate diagnostic baseline state could be affected. Diagnostics baselines are now part of merge per slice 1+; the 99-case/127-variant figure should reproduce there. Slice 3 reported `1018.43s` warm; this slice adds four cases (≈+4 fixtures × ~3-5s each in cold state) and shouldn't move the budget meaningfully.
- **(Optional) Conditional on finding #1.** If you add `sifr.toml` to `source_import_private_member/`, re-run focused validation: `diagnostics baselines` will re-bless if the resolution path produces different output, but since entry-parent resolution already wins, the baseline should be unchanged. If the baseline changes, that's evidence the previous run was depending on workspace inheritance and the fix is even more important.
- **(Optional) Conditional on finding #2.** If you add a `source_origin` field, extend `code_baseline_coverage.py` to fail when the linked path drifts. No new validation needed beyond the area suites.

Open the PR once finding #1 is either applied or explicitly deferred in the PR description.

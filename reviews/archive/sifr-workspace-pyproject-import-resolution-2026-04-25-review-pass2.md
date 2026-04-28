# External Review (pass 2): Sifr Workspace Resolution Via `pyproject.toml`

Reviewer: external review pass
Review date: 2026-04-25
Inputs reviewed:

- `issues/sifr-workspace-pyproject-import-resolution-2026-04-25.md` (source issue)
- `issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md` (phase plan)
- `issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25-execution.md` (execution checklist)
- `internal_docs/roadmap.md` (Phase 31.6 row)
- Current codebase under `crates/`, `audits/leetcode/`, `lib/sifr/`, `crates/sifr/tests/`, `verification/suites/manifest.json`

The review below is organized as: 1) blocking findings, 2) nonblocking improvements, 3) final verdict.

---

## 1. Blocking Findings

### B1. Codegen and materialization for dotted workspace module names is unspecified, and the current pipeline cannot accept them

The phase plan (WS3, lines 220-221) and the source issue (lines 88-92) commit to a `dotted_to_path` rule: `helpers.list_node` resolves to `helpers/list_node.sifr`. The LeetCode pilot in WS5 (lines 285-289) then exercises exactly this path with `from helpers.list_node import ListNode`. The plan does not say how the rest of the pipeline keys, materializes, or `mod`-declares such a module:

- The codegen import-rendering side already splits dotted names into Rust module paths at [`crates/sifr_codegen/src/lib.rs:642-655`](crates/sifr_codegen/src/lib.rs:642), so it would emit `use crate::helpers::list_node::ListNode;`.
- The project-side aggregation, however, keys support modules by their flat module name string and emits a flat `mod {module_name};` and a flat `src/{module_name}.rs` file:
  - [`crates/sifr_driver/src/project/assembly.rs:21-25`](crates/sifr_driver/src/project/assembly.rs:21) emits `mod helpers.list_node;` for that key — invalid Rust.
  - [`crates/sifr_driver/src/build/materialize.rs:85-88`](crates/sifr_driver/src/build/materialize.rs:85) writes `src/helpers.list_node.rs` — not addressable as `crate::helpers::list_node::*`.
  - [`crates/sifr_driver/src/build/project_codegen.rs:5-12`](crates/sifr_driver/src/build/project_codegen.rs:5) and the cache-key construction at [`crates/sifr_driver/src/build/materialize.rs:119-143`](crates/sifr_driver/src/build/materialize.rs:119) all assume flat module names.

Net effect: the WS5 pilot fixture cannot actually link with the implementation as scoped, and no workstream owns the gap. This is not a hidden refactor — it requires a deliberate decision and tests in either WS3 (resolver and codegen path) or WS4 (build wiring and materialize layout). Pick one of:

- emit a Rust nested module tree (`src/helpers/mod.rs` declaring `pub mod list_node;` plus `src/helpers/list_node.rs`) and have `assemble_project_main_rs` emit `mod helpers;` only at the top level; or
- materialize via `#[path = "helpers/list_node.rs"] mod helpers__list_node;` plus a renamed alias path inside codegen; or
- sanitize dotted module names everywhere (codegen and materialization) so codegen no longer splits on `.` for workspace-resolved modules.

The plan must fix the chosen approach, name the affected files, and require a regression test that exercises a dotted import end-to-end before WS3/WS4 are marked ready. Without this, the pilot in WS5 is not buildable.

### B2. WS5 e2e fixture mechanism is incompatible with the current flat e2e harness

Both the source issue (lines 161-170) and the phase plan WS5 (lines 277-281) place the workspace-rooted e2e fixtures under `crates/sifr/tests/e2e/pass/` and `crates/sifr/tests/e2e/fail/`. The harness in [`crates/sifr/tests/e2e.rs:775-816`](crates/sifr/tests/e2e.rs:775) (`discover_fixtures`) reads only direct `.sifr` files via `read_dir(base_dir)` — it does not walk subdirectories and does not understand `pyproject.toml` siblings. A workspace fixture tree like

```
pyproject.toml
audits/leetcode/0021_merge_two_sorted_lists.sifr
helpers/list_node.sifr
```

cannot be discovered or invoked there. The existing home for multi-file project fixtures is the verification-suites manifest, [`verification/suites/manifest.json`](verification/suites/manifest.json) — see the existing `multi_module_run` and `missing_import_reports_error` cases under `crates/sifr/tests/verification/project/` registered there. WS5 must either:

- target the verification-suites manifest (add `workspace_*` cases under `crates/sifr/tests/verification/project/<case_id>/` referencing the new fixtures), or
- explicitly extend the flat e2e harness to discover sibling `pyproject.toml` and walk subdirectories, with a new harness contract documented in WS5.

Either is acceptable, but the existing wording sets the wrong expectation and would force the WS5 PR to either silently fail to wire up the fixtures or balloon into a harness rewrite. Pick the mechanism in writing before WS5 starts.

---

## 2. Nonblocking Improvements

### N1. Slice example uses `lib` as a workspace source, which collides semantically with stdlib intercept

Phase plan Target Configuration (lines 60-65) shows `sources = ["audits/leetcode", "lib", "."]`. Although the resolver will never see `sifr.*` / `_sifr.*` imports because the registry intercepts them at [`crates/sifr_driver/src/project/discovery.rs:70-74`](crates/sifr_driver/src/project/discovery.rs:70), listing `lib` next to `lib/sifr/*` reads as a clash. WS5 already says (line 295) "exact list to be confirmed during implementation". Either drop `lib` from the example, or add a sentence noting that `lib/sifr/*` is intercepted by the registry before the workspace resolver ever sees it.

### N2. `[tool.sifr]` validation contract should explicitly state `name` is optional

Phase plan validation contract (lines 67-75) says missing `sources` defaults to `["."]` and that `name` "when present, must be a string", but does not say what happens when `name` is absent. The source issue (line 80) says an empty `[tool.sifr]` table is valid; copy that line into the phase plan to remove ambiguity. The `SifrWorkspaceConfig` model (line 140) already declares `name: Option<String>`, so the spec should match.

### N3. New diagnostics need stable codes and documentation URLs

Roadmap global rule (line 28): "Every top-level user-facing compiler diagnostic must carry a stable code and deterministic documentation URL of the form `https://sifr.sh/docs/errors/<CODE>`." The plan defines diagnostic phrasing for parse, validation, unresolved, and ambiguous import errors (WS3 lines 227-234) but does not assign codes (e.g., `SIFR-WORKSPACE-0001`) or URLs. WS3 acceptance criteria should require:

- A documented code for each new top-level diagnostic.
- Snapshot/insta coverage that includes the code and URL string verbatim.

### N4. Native `sifr.toml` precedence rule contradicts the slice non-goal

Phase plan line 122-123 says "If native `sifr.toml` support is added in this phase, discovery precedence must be explicit: a `sifr.toml` in the same directory as a `[tool.sifr]` pyproject wins; otherwise the nearest qualifying manifest wins." Phase plan Non-goals (line 57) says native `sifr.toml` is "design runway only" for this slice. Pick one: either drop the conditional precedence rule from this slice (recommended) and move it into the design note for the future native phase, or scope it as a parser-only inert support with a guard test. As written it is a hidden optional feature with a precedence rule, which is exactly the kind of half-finished work the project's Core Expectations forbid (`AGENTS.md` "Don't add features … beyond what the task requires").

### N5. PR sequencing flexibility for WS1 risks an intermediate broken state

Phase plan line 339 allows WS1 to land before or after WS2 ("if it only detects workspace presence; it must not add source-root resolution before WS3"). If WS1 lands first, the CLI flips a workspace entry into project mode while the resolver is still entry-parent-only, so any workspace fixture relying on a non-sibling source root would parse-fail rather than fall through. That is technically not a regression for *existing* fixtures (none use workspace imports yet), but it is a hidden contract: any user who adds `[tool.sifr]` between WS1 and WS3 sees broken imports. Commit the order WS0 → WS2 → WS3 → WS1 → WS4 → WS5 → WS6 in the phase doc to remove that exposure window.

### N6. `parse_import_closure_modules` signature change must call out the test-runner adoption

WS2 (lines 196-199) lists `parse_import_closure_modules` and "test-runner call sites" but the test_runner adoption is non-trivial and visible in [`crates/sifr_driver/src/test_runner/orchestrator.rs:48-49`](crates/sifr_driver/src/test_runner/orchestrator.rs:48). Add an explicit "WS2 must update the test_runner orchestrator to pass an entry-parent-only `ModuleResolver` and keep its discovery scope identical" line so the WS2 reviewer can verify the no-op claim. Also confirm that WS2 keeps the `BTreeSet` traversal order so the discovery diagnostics in [`crates/sifr_driver/src/project/discovery.rs:91-138`](crates/sifr_driver/src/project/discovery.rs:91) remain deterministic (the plan says "Preserve deterministic pending-module traversal with `BTreeSet`" — keep that wording).

### N7. WS4 cache invalidation can lean on the existing key, but make the test explicit and dotted

The existing `binary_project_cache_key` at [`crates/sifr_driver/src/build/materialize.rs:119-143`](crates/sifr_driver/src/build/materialize.rs:119) already hashes `support_modules` (name + code). As long as workspace-resolved helpers land in `GeneratedBinaryProject::support_modules` keyed by their canonical (sanitized or nested) module identity, content changes will invalidate. WS4's "explicit regression ensuring a workspace helper content change invalidates the cache" (line 261) should:

- use a dotted import (e.g., `from helpers.list_node import ListNode`) so it covers both the cache and the materialization issue called out in B1;
- assert the cache key changes, not just that the binary path differs;
- assert that an unrelated workspace source-root reordering that does not change resolved content does not invalidate the cache.

### N8. Test-runner workspace alignment should be a documented limitation, not a TBD

WS4 implementation note line 263 says "Test-runner behavior can remain entry-directory scoped unless a workspace-aware test command is explicitly added in a later phase." That is fine, but it leaves `sifr test` divergent from `build/run/check/emit` once a workspace exists, which contradicts the broader frontend-mode-parity contract from Phase 22. WS5 design note must explicitly call out `sifr test` as workspace-unaware in this slice and link the deferral. Otherwise this becomes a silent debt regression once users adopt `[tool.sifr]`.

### N9. Plan should pin the TOML crate name and version

Phase plan (line 137) says "Add a TOML parsing dependency through workspace dependencies and the `sifr_driver` crate." The source issue (line 192) recommends the read-only `toml` crate. Pick a crate name and a pinned version (e.g., `toml = "0.8"`) and commit it to WS0 — otherwise the first WS0 PR has a freebie design discussion that derails atomic landing.

### N10. Path validation tests must cover non-trivial relative-path forms

WS0 acceptance criteria (lines 154-157) covers `..` escape, missing entries, and non-directory entries. Add explicit tests for:

- `sources = ["./helpers"]` (leading `./`), normalized vs literal handling;
- absolute paths (must be rejected);
- empty string and `""`-equivalent path components;
- platform path-separator robustness (TOML strings come through as-is, but `..` checks must work component-wise per the existing implementation note line 149).

These are fast unit tests and close obvious abuse vectors at the workspace boundary.

### N11. Diagnostic policy for malformed ancestor `pyproject.toml` should match the source-issue mitigation

Source issue Risks (line 213): "only fail when the discovered `pyproject.toml` actually contains `[tool.sifr]`. A pyproject.toml without that table is silently ignored." Phase plan WS0 (line 142) says "Return the first parent whose `pyproject.toml` parses and contains `[tool.sifr]`. Ignore `pyproject.toml` files that parse but do not contain `[tool.sifr]`." Acceptance test list (line 154) includes "malformed TOML" without saying which level it sits at. Make the contract explicit:

- malformed TOML in a `pyproject.toml` that *would* be selected (i.e., is the nearest ancestor and contains `[tool.sifr]` or is otherwise indistinguishable until parsed) → hard `CompileError`;
- malformed TOML in an ancestor pyproject that obviously does not pertain (e.g., a file we do not need to consult once a closer match has been chosen) → ignored;
- missing `pyproject.toml` → the entry stays in the no-workspace path with no diagnostic.

This is the only way to keep the plan's "no fallback" rule from accidentally breaking unrelated `cargo run -q -p sifr -- run somefile.sifr` invocations under sibling Python projects.

### N12. Source-issue claim about the Trie helper is stale

Source issue (line 32): "the trie rewrite (WS2 S6) had to promote `Trie` into `lib/sifr/trie.sifr` and add a registry entry, locking a fixture-driven helper into the language stdlib." That promotion has been reverted — see commit `be19ea3f Move trie helper out of stdlib (#1638)` and the present [`internal_docs/leetcode_trie_helper_design.md`](internal_docs/leetcode_trie_helper_design.md) which keeps the trie inline. There is no `lib/sifr/trie.sifr` in the tree anymore. Update the source-issue motivation paragraph to reflect "the trie helper had to remain inline because no workspace-rooted import path existed; promotion to stdlib was rejected." This does not block the phase but the misstatement reads oddly to anyone verifying the motivation.

### N13. Empty `audits/leetcode/helpers/` directory in the tree predates this phase

The empty directory `audits/leetcode/helpers/` already exists. Either delete it now (it is unused, and an empty directory in git is a faint smell), or note its presence in WS5 prep so the pilot PR does not look like it is "creating a new directory" when it is in fact populating an existing empty one. Non-blocking but worth a single line of disclosure.

### N14. WS5 sources example should be confirmed before WS5, not during

WS5 implementation note line 295: "Confirm `pyproject.toml` source ordering before adding it at repo root; `audits/leetcode` and `lib` are expected candidates, with `"."` included only if needed and justified." The plan elsewhere (line 64) shows three sources. Lock the source list in the WS5 plan now (even if it is a one-line decision) so the pilot PR does not stall on a configuration debate that should be a design call.

### N15. Execution checklist should record per-wave validation evidence, not only WS6

The execution file ([`issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25-execution.md`](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25-execution.md)) lists "Required Final Validation" only under WS6 (lines 141-147). The phase Quality Contract (line 361) says validation evidence must be recorded in the execution checklist before *each* PR merges. Add per-wave placeholders under each wave's "Validation Evidence" heading for at least:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- targeted `cargo test -p sifr_driver -- <selector>`
- where applicable, the specific e2e/verification suite invocation

### N16. Manifest naming hygiene is good, keep it through codegen too

The plan correctly avoids naming types after `pyproject` (lines 144-148, 120). Good. Apply the same rule to the WS3 codegen edits: the resolver type that drives module path conversion should not be named `WorkspaceModuleResolver` if the same model is intended to back native `sifr.toml`. `ModuleResolver` plus a `ManifestSource` enum (`Pyproject`, `Native`) is enough hygiene for this slice; document that in WS2 so subsequent waves do not invent parallel resolver types.

### N17. Stress the no-fallback rule on workspace activation parsing

Quality Contract (line 354): "No fallback path may hide malformed `[tool.sifr]` config." Verify this is reflected in the WS1 path — currently WS1 (line 175) only flags "may need `resolve_compilation_mode` to return `Result<...>` so workspace parse/config failures reach `build`, `run`, `check`, and `emit`." Make this a *requirement*, not a "may need". Without it, a malformed `[tool.sifr]` could silently flip an entry back to single-file mode, which is the exact silent-debt the phase exists to prevent.

---

## 3. Verdict

NOT READY

Rationale: B1 (dotted workspace modules have no specified materialization path; the LeetCode pilot in WS5 cannot link end-to-end as the plan stands) and B2 (WS5 e2e fixtures target a harness that cannot discover them) are both implementation-blocking. Each is a one-paragraph fix in the phase doc, but neither is currently described, and both would force the implementing engineer to reopen design discussions mid-PR. Once the dotted-module materialization decision and the e2e fixture mechanism are committed in writing, and the nonblocking improvements above are folded in (especially N3 diagnostic codes, N4 native-manifest scope cleanup, N5 PR sequencing lock, N11 malformed-ancestor pyproject policy, and N17 no-fallback hardening), this phase should clear external review on the next pass.

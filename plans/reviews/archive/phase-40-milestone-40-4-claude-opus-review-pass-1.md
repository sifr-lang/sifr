## Milestone review — Phase 40 / milestone_40_4 (round 1)

**Range:** `origin/main...HEAD` = 12 commits, 51 files, +2819/−262. Read-only throughout; `git status` clean in the main repo, `editor_integrations`, and `editor_integrations/vscode` after all checks (suite output goes to gitignored `target/`; probe files under `/tmp`).

### Checks re-run

| Check | Result |
|---|---|
| `documentation --suite structure --suite ga-release` | PASS, variants=2, failures=0; result suite order `['structure','ga-release']` |
| `developer_tooling --suite editor-release` | PASS, variants=6, failures=0, blocking=0 |
| `distribution_release --suite qualification --suite full` | PASS, variants=55, failures=0; `test_planner_rejects_rollback_range_drift` green |
| `cases/stable_editor_qualification_contract.sh` | PASS (both self-tests + renderer `--check`) |
| `check_file_size_guardrails.py` | PASS (2877 files, limit 900) |
| Submodule identity | pointer `a980835e6→d7577d492`; `d7577d492` = merge of reviewed pointer head `34da355fa`, `git diff 34da355fa d7577d492` empty; consumed `vscode` = `273fd5d3e`, `sifr.sifr-vscode 0.2.0`, engine `^1.91.0`, range `>=0.1.0,<0.2.0` — matches `vscode_extension_rules.json:22-23` exactly |
| Scope | 0 files under `crates/**` or `verification/areas/rust_interop/**`; no Marketplace surface anywhere in `.github/workflows/` |

Planner binding verified by reading `planner.py:310-375`: editor report → transported VSIX digest → plan `vscode.*`, plus new target-report/binary-digest cross-binding. Archive safety, duplicate/symlink/traversal rejection, and required packaged runtime files in `qualify_stable_editor.py:73-110` are sound. Demo filename `editor_candidate_qualification_demo.sh` is capability-named with no phase/milestone token.

### Findings

**1. HIGH — public GA docs still declare Sifr a preview and document self-update commands the CLI rejects; the new gate structurally cannot see them.**
- `docs/introduction.mdx:99`: *"Sifr is currently in preview. The alpha and beta channels are available for early adopters."* This is page 2 of the same **Get Started** nav group that now carries the stable pages (`docs/docs.json:72`), and it directly contradicts `docs/releases/0.1.0.mdx:7` and `docs/installation.mdx:39`.
- `docs/cli/packages-workspaces.mdx:91-92`: `sifr self update --channel nightly` and `--version 0.4.0-preview.2 --force`. Both are rejected by the CLI — `crates/sifr/src/self_update_metadata.rs:339` accepts only `alpha|beta|stable`, and `:71-75` rejects any prerelease label other than `alpha`/`beta`. Line 95 still describes `--version` as *"one immutable preview version."*
- Cause: `check_ga_release_docs.py:34-44` inspects a fixed nine-document set and applies `FORBIDDEN_CLAIMS` only to that set (`:241-244`), so the milestone's negative validation *"preview-only docs fail the gate"* is unenforced across the rest of `docs/**`.
- Remediation: correct both pages, and sweep every `docs/**` page for the forbidden preview/gated claim set (explicit allowlist for genuinely preview-scoped content) so the gate covers what it claims to.

**2. MEDIUM — the "Marketplace publication dry run" is a hard-coded constant; nothing executes and no workflow is inspected.** `qualify_stable_editor.py:330-346` writes `marketplace_dry_run` with `"status": "pass"`, `"rebuild": false`, and a literal `npx --no-install vsce publish --packagePath …`; `editor_qualification.py:232-273` then asserts exactly that constant back. No `vsce` process runs and no workflow text is parsed — `grep -rn 'vsce|packagePath|VSCE' .github/workflows/` returns nothing, because the consuming workflow is milestone_40_5 work. The DoD *"A Marketplace publication dry run proves the main-repository workflow will consume the recorded VSIX and package version without rebuilding"* is therefore unmet, and the evidence records `pass` for a step that never ran. Remediation: either execute a real non-publishing `vsce` check against the recorded VSIX and assert its digest is unchanged afterwards, or bind the assertion to the 40_5 workflow text once it exists and record this DoD item as carried forward instead of emitting `status: "pass"`.

**3. MEDIUM — `release-qualification.yml` hard-codes `--rollback-version none`, so no non-first-GA stable release can be qualified.** Workflow `:186`, and the immutable-contract case now *pins that literal* (`release_qualification_workflow_contract.sh:74`). But `release_plan.py:97-98` requires a non-`none` rollback target for every `transition: "normal"` plan, and `planner.py:333-344` fails when the editor report's `rollback_version` differs from the plan's. Failure scenario: qualify `0.1.1` as `normal` with rollback target `0.1.0` → editor evidence carries `"none"` → planner fails `"editor report identity did not pass"`, and the only remedy is editing a contract-pinned workflow. Side effect: the rollback-containment branch at `qualify_stable_editor.py:135-141` is unreachable in production, so the negative validation *"a non-`none` rollback target outside the advertised compiler range fails the gate"* is proven by fixture only. Remediation: add a `rollback_version` dispatch input (default `none`), thread it through, and have the contract case assert the *parameter* rather than the value.

**4. MEDIUM — the tracked claim that stable docs don't advertise the deferred capability is false for `sifr emit`.** `plans/issues/active/phase-40-stable-channel-ga-execution.md:111` and `plans/phases/40_…md:921` both state GA docs do not advertise the affected capability. But `adhoc_packaged_candidate_generated_rust.md:5-9,25-26` scopes **both** `sifr.server.showGeneratedRust` **and** `sifr emit`, and GA docs advertise `sifr emit` prominently: `docs/cli/overview.mdx:36`, the whole of `docs/cli/check-emit.mdx`, `docs/cli_command_semantics.md:51`, and a Quickstart step at `docs/quickstart.mdx:102-103`. Remediation: reconcile the claim — either re-verify `sifr emit` against the packaged candidate and narrow the ad hoc scope to the editor action, or record the limitation on the affected pages. (This is about the claim, not about fixing the deferred hang.)

**5. LOW/MEDIUM — the ad hoc record asserts a diagnosis its evidence does not establish.** `adhoc_packaged_candidate_generated_rust.md:39` scopes the follow-up to *"Isolate the shared deadlock or unbounded operation."* I probed the in-tree debug binary with the same out-of-workspace fixture shape the original demo used (`git show 384fc7d7e` shows `emit` was run on a `/tmp` fixture with cwd at the repo root): the **first** cold invocation exceeded a 90 s bound with no output, and every repeat of the identical command returned in ≈6 s and produced correct Rust. That is consistent with cold first-run cost, not a deadlock, and I could not reproduce a persistent hang. I could not reproduce the packaged-release case (needs a release build). Remediation: keep the observation, drop the asserted deadlock framing, and list the cold-start/first-run hypothesis in Scope so the follow-up doesn't chase a defect that may not exist.

**6. LOW — `qualification_selftest.py` is 899 lines against the 900-line cap** (base 889; guardrail PASS but one line of headroom). Extracting `qualification_editor_selftest.py` was right, yet the near-cap harness still grew by 10 lines, so the next mutation case cannot land without a refactor. `qualification_fixture.py` is 879 and `verification/runner/sifr_verify/selftest.py` 860. AGENTS.md directs splitting by responsibility rather than adding to an oversized module.

**7. LOW — `unsupported-target-claim` only detects target *removal*, not addition.** `check_ga_release_docs.py:291-297` mutates by replacing a supported triple, which trips the required-fact assertion instead of an unsupported-claim assertion. Appending `aarch64-pc-windows-msvc` alongside the four supported triples passes the gate — there is no forbidden-target list. Remediation: add unsupported triples/OS names to `FORBIDDEN_CLAIMS` or a dedicated forbidden-target tuple.

**8. LOW — the exact-candidate LSP command is passed through a whitespace-split env var.** `qualify_stable_editor.py:288` sets `SIFR_LSP_COMMAND = f"{candidate_binary} lsp --stdio"`; `lsp_protocol.py:24-29` does `command.split()`. Any space in the candidate binary path (temp dir, workspace, home) silently builds the wrong argv and the smoke fails with a misleading error. Use `shlex.quote`/`shlex.split` or an argv-list variable.

### Non-actionable observations

- `verification/areas/documentation/fixtures/stable_site_release_facts.json` is synthetic (placeholder `aaaa…`/`ffff…` digests), so the DoD's "render from the governed release payload" holds structurally, not by binding to a real plan/index; that binding arrives with the 40_5 site adapter.
- `qualify_stable_documentation.py` has no production caller — only `--self-test` from the contract case; no runbook step tells an operator to run it before the planner.
- `vsix_install_smoke` is archive extraction plus a `config.js` `serverCommand()` assertion, not a VS Code install; combined with the separate exact-binary LSP smoke this is a reasonable approximation, but the field name overstates it.
- `qualify_stable_editor.py` reconstructs/writes canonical JSON with default `ensure_ascii=True` while `governance.common.canonical_json_bytes` uses `ensure_ascii=False`; inert today since every field is ASCII.
- `docs/docs.json:72-76` puts five release/support/troubleshooting pages between `installation` and `quickstart`, pushing the quickstart to ninth in **Get Started**.
- The milestone_40_4 tracker correctly leaves candidate-evidence materialization and the evidence-only PR unchecked; the milestone cannot close on this diff alone.

Findings 1–8 are actionable, so this round is not approved.

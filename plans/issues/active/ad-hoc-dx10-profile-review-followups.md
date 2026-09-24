# DX.10 profile and native consumer review followups

status: open; separate owning work; not DX.10 acceptance blockers

Origin: [DX.10 implementation #3858](https://github.com/sifr-lang/sifr/pull/3858),
[preserved initial reviews/adjudication](https://github.com/sifr-lang/sifr/pull/3858#issuecomment-5725480175)
and [final SATISFIED review](https://github.com/sifr-lang/sifr/pull/3858#issuecomment-5725527463).
The phase record is [here](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md).
No implementation is included in this record-only issue.

| ID | Owner | Followup and bounded acceptance |
| --- | --- | --- |
| F1 | verification runner | Restore the planned crate-test red-blocker reporting line filtered by configuration planning. Preserve selected suites and execution behavior; assert reporting for a non-executed red-blocker. |
| F2 | verification runner | Audit the unused `crate_test_suites_for_mode` import in `profile_runner.py`; inspect actual remaining uses before removing any `cargo_command` import. No configuration change. |
| F3 | E2E runner | Select the release corpus authority by `id == e2e-run-pass` in Rust instead of positional `selections[0]`; test reordered selection records without changing the declared fixture set. |
| F4 | driver tests / performance | Update stale `target/release` messages in the two native test helpers. Their three-parent traversal is still correct. Consider clarifying the benchmark's release-binary error to distinguish declared profile from finalized location. |
| F5 | driver Rust probe | Consider early captured-override validation before scratch probe execution, so a rejected build does not first run its probe with that override. Generated application/probe workspace ownership is already fixed; retain that contract. |
| F6 | performance | Resolved by DX.15 #3868; the original investigation and measurements remain below. Original real-ref whole-file size non-increase failure: 6,634,688 → 6,641,408 bytes (+6720, +0.10%). `.text`/data/bss unchanged; nonallocated debug/string metadata increased. Evidence below. Preserve the existing assertion; any metadata exclusion or budget policy requires its own justified decision. Do not rerun the unchanged failed pair to seek a pass. |
| F7 | sifr-lang/leetcode | [Owning issue #49](https://github.com/sifr-lang/leetcode/issues/49): explicit generated-application profile and finalized paths in benchmark/audit consumers, independent of compiler optimization. No submodule change in DX.10. |
| F8 | compiler architecture docs | Add one sentence explaining that generated Cargo manifests own application profile authority and generated applications/probes own their workspace; retain captured config/env/flags validation wording. |
| F9 | runtime platform / sanitizer | Explicitly decide and inventory the application profile for `generated-binary-asan-smoke` in `sanitizer_manifest.json`. It currently uses the new dev default; exit/sanitizer-clean assertions are profile-agnostic and no named DX.10 release assertion was removed. Qualify the declared choice in the owning sanitizer workflow. |
| F10 | core language / project workspace | Classify the roughly 12 literal behavior-matrix `sifr build/run` argv in the two `data/validation_suites/manifest.json` files. They execute through the generic command path, and their assertions are profile-agnostic; document the default-dev choice or explicitly qualify a different profile. These were outside the 10 declared literal producer rows in the remediation inventory. |
| F11 | differential runner infrastructure | Make focused generated-suite tests importable consistently from the repo root as well as their `checks/` directory. Current directory-scoped invocation passes; no oracle or runtime assertion change. |
| F12 | project workspace validation / CLI package resolution | Resolved by [#3957](https://github.com/sifr-lang/sifr/pull/3957): the positive and negative `frontend_mode_parity` test rows and companion core-language test select their nested packages. Original failure, acceptance, and final binding are below. |

## F12 project-workspace test command package root — 2026-09-23

Discovered while validating emitted-Rust Item 12R candidate `d9e79b869d2071433c8a01a980754e3977628b2d` ([draft PR #3946](https://github.com/sifr-lang/sifr/pull/3946)). The exact `project_workspace/frontend_mode_parity` selection exited 1. Its `positive_test` command in `verification/areas/project_workspace/data/validation_suites/manifest.json` is `cargo run -q -p sifr -- test demos/mode_consistency`, executed at repository root. Both root and demo have `sifr.toml`; `test_cli.rs` resolves the root package and emits SIFR-RUST-CARGO-0001, “sifr test directory must be inside one Sifr package.” The expected exit is 0. This is a command working-directory/package-root contract defect; the manifest and `crates/sifr/src/test_cli.rs` are unchanged by Item 12R. The receipt is `/home/yaser5/projects/sifr/emitted-rust-item12r-evidence/project-frontend_mode_parity-d9e79b869.json` (selection digest `22811269c4511bf5d6ac8359589fe3782217d3d269a7c6b0779ed144709ca2b3`; validation-input digest `6c2acd10d66e2e31229930b2a53e5a9dbe6bc5a6f6cf73788b61b33852c3ed96`). Item 12R is blocked on this owner repair; its broad checks were stopped, not passed.

Bounded acceptance: make the positive `sifr test` row select its intended demo package, preserve the negative `sifr test` row's error semantics, and inspect the companion `verification/areas/core_language/data/validation_suites/manifest.json` for the same root/argv contract. Keep F10's build/run profile classification separate. Run the exact `project_workspace/frontend_mode_parity` area selection, the matching core-language validation-suite selection if it shares the command, and `cargo test --locked -p sifr --test validation_suites test_validation_suite_matrix -- --ignored --nocapture` with the suite filter set for each affected matrix. Then return Item 12R for its remaining named validation; do not treat this issue record as an implementation repair.


### F12 final binding

The bounded repair is merged in [#3957](https://github.com/sifr-lang/sifr/pull/3957),
reviewed candidate `fa474ed47829cac069118978ee6417045d3913b4`, merge
`22cb59a766781c181e919d15bf97810f7c1a69cb`. The runner now accepts a
per-command working directory and repository-root argument token. The three
nested test fixtures have valid Cargo/Sifr package manifests; their empty
`[workspace]` tables intentionally keep them separate from the root Cargo
workspace. The positive project test and companion core-language test pass,
and the negative project test reports the asserted `SIFR-TYPE-0002`. The
literal build/run commands and F10 profile classification were not changed.

The exact `project_workspace/frontend_mode_parity` area selection passed two
rows and `core_language/hir_analysis_behaviors` passed three rows. Each area
adapter ran `cargo test --locked -p sifr --test validation_suites
test_validation_suite_matrix -- --ignored --nocapture` with its own manifest
and suite filter, so the required crate selections executed on the reviewed
candidate. `cargo fmt --check`, `python3 scripts/check_file_size_guardrails.py`,
and `git diff --check` passed. Evidence is under
`/home/yaser5/projects/sifr/dx10-f12-evidence/`: `project-area-final.log`,
`core-area.log`, and the candidate-keyed `review.md`. The
[Opus review](https://github.com/sifr-lang/sifr/pull/3957#issuecomment-5793350364)
returned SATISFIED with no blocking findings. The first F12 failure receipt
and this repair's earlier failed runs remain preserved; they are not counted
as passing. The Phase DX intermediate-item exception omitted full create-PR
and merge gates. The two pressure cleanups are recorded in
`pressure-cleanup.log` and `pressure-cleanup-2.log`.

Deferred review followups remain separate from F12: verification-runner
hardening can constrain future `cwd` values to repo-relative paths; fixture
maintenance can avoid leaving ignored `Cargo.lock` and `.sifrbuildinfo` in a
reused worktree; and the performance owner should check
`perf.build.project.branch_paths` and `perf.check.project.mode_consistency`
when refreshing baselines because those demos now have package manifests.
Item 12R can resume its remaining named validation on its own candidate.

F6 evidence is on
`yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx10-evidence/`:
`size-remediation/receipt.json`, `size-remediation/section-deltas.json`,
the original tool log and both captured native binaries. The original wrapper
and failed observation remain intact. The independent real decrease/increase
controls in `size-consumer-controls/receipt.json` verify expected tool exits
0/2; they do not erase the failed real-ref non-increase observation.

This issue does not authorize DX.11 work or weaken the phase-end gate.

## Supplemental lint observations from DX.12

The owner-only JSON lint scan reached unchanged profile-related code at the
DX.12 base f1d0e1d72ad107e816c16690cefddfb19df6200f: cargo_resolution.rs:39,434
implicitly clones Strings, and compiler_context.rs:50,56 has methods returning
Self without must_use. Keep these for profile maintenance and the phase-end gate.
No profile code was changed by DX.12. Exact diagnostics and changed-line
classification are retained in /home/yaser5/projects/sifr/dx12-evidence/
lint-scope-report.json; the scan reports zero new/touched-line diagnostics.

## F6 bounded repair resolved by DX.15

The original failed observation remains intact. A final-candidate comparison at
`b46545b7541420302c9408588dc42ac2d8f777a9` also failed with the original
two-directory harness: 6,634,776 → 6,644,400 bytes (+9,624). Its artifacts and
failure are preserved in `dx15-evidence/<candidate>/binary-size-wrapper-fixed/`.

Controlled attribution keeps the exact compiler/sysroot bytes, Sifr program,
Rust toolchain and release profile, and changes physical locations independently.
At one shared source/output/sysroot location, baseline and candidate produce
byte-identical 6,603,136-byte native executables. Changing only the candidate
sysroot location increases the whole file by 1,768 bytes. Native crate identities
and retained debug/string sections depend on paths; different checkout/sysroot
locations confound a compiler-size comparison.

The harness now measures both real refs sequentially in one private detached
checkout and one output location. It resolves both refs before switching and
retains Cargo storage, explicit `--release`, and the unchanged whole-file
non-increase assertion. No section is excluded, stripped or remapped and no
tolerance is added. The actual repaired harness passes at 6,640,088 → 6,640,088
bytes; both release Cargo profiles retain overflow checks. Synthetic harness
controls separately prove that a one-byte increase rejects, equality/decrease
pass, source/output paths stay identical, and a failed candidate build cannot
accept a stale baseline executable. These controls do not impersonate
native measurements.

Evidence is under `dx15-evidence/b46545b7541420302c9408588dc42ac2d8f777a9/`:
`size-path-attribution/`, `binary-size-controlled/`, plus
`dx15-evidence/size-tool-controls.log`. DX.15's final gate/review/merge record
will bind the repair to the final candidate. Other followups remain separate.

### DX.15 final binding

F6's bounded comparison-harness correction is merged in
[DX.15 #3868](https://github.com/sifr-lang/sifr/pull/3868), reviewed candidate
`9ee360474655fde979dc5ddfdf33eb4dc39c9968`, merge
`0e4b5ec59607ffdabb9d42dbd8ba49baab97137e`.
The original failed measurements and the controlled same-location pass above
retain their original identities and scope; no historical failure is erased.
The [final phase record](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md) binds the
qualified continuation and SATISFIED review.

The supplemental DX.12 lint observations are also resolved by the recorded
strict workspace Clippy pass in
`dx15-evidence/e2ab8e1f9ae815a1f8ac2ac652329d90dfc4ff0f/after-sql/report.json`.
The original diagnostic report is preserved. Other followups remain separate
and are not silently marked complete by this binding.

## 2026-09-23 profile inventory observation

An optional verification-runner self-test on main-derived base
`787d3ad8d471ad35fd409b76887014c5b3fe334c` failed before V01 changes
were involved: `PYTHONPATH=verification/runner python3 -m sifr_verify --self-test`
reports a DX.10 `test_b11_release_corpus_and_retained_native_selection`
fixture-list mismatch. The tracked `integer_field_augassign.sifr` E2E pass fixture
appears in the filesystem inventory but is absent from the retained release
selection. This record does not alter the selection or classify the run as
passing. Raw log: `/home/yaser5/projects/sifr/architecture-v01-evidence/runner-selftest.log`
(SHA-256 `afb3334bfbfd6b38f695b1feac51c61ca78cc0c961d181bf6f74dd5b0f6d9ab5`).
The verification-runner owner should reconcile the fixture inventory and
rerun that exact self-test.

### Reconciliation (2026-09-24)

The verification-runner DX.10/B11 inventory reconciliation merged in
[PR #3986](https://github.com/sifr-lang/sifr/pull/3986) from candidate
9e8740fc815c321c92b8cdf48d114c8e5a09e0a4 (base
60613299fccc317d1bed059dec7940739a456a36) as merge
53664cbab52055b9bf8aa5df01b79743d57927d8. The sole delta adds
integer_field_augassign.sifr to the lexical e2e-run-pass list. Actual
tracked run-pass fixtures and the declared selection now agree at 728 names
with no missing, stale or duplicate entry. The release and development
application-profile policy and native-run assertion remain unchanged.

On the reviewed candidate, the exact SQL Item 4 prerequisite
uv run --project verification --locked python -m sifr_verify --self-test
passed (raw log SHA-256 19cd85131669babfbed46caa0a3b67ef19ba646beb3ac1ab0ca06b24783798c3).
All five DX.10 profile tests passed (d0ee918b4dda9077081b3809573a8ee3bded0e7b6fb12693bce19cddf67fd60e);
the exact B11 test and the unrelated DX.3 cancellation case passed focused
(bfaa1cd8da593028115e79720c8cc36a4fdb6da11cb1aa9722007d2c2332c0d8).
JSON parsing, diff check and the 900-line guardrail passed (guardrail log
f602fdaaa50641ee96dbc83cb0f1ebf82f2fbe3598b2c835c18220cb26873dec).
The first worktree self-test failed because its target/ directory was not
prepared; a prepared attempt exposed a transient DX.3 process assertion.
Both failures remain in the evidence directory and are not counted as passes.
The final exact complete self-test passed. Raw evidence and the candidate-keyed
review are under /home/yaser5/projects/sifr/dx10-b11-inventory-evidence/.

The read-only scoped [Opus review](https://github.com/sifr-lang/sifr/pull/3986#issuecomment-5804943374)
returned SATISFIED with no blocking findings (response SHA-256
fb1e3d0e1aa6155710b961322a492af745383c4b082c7fef7aefca819dbff420).
The approved Phase DX intermediate-item policy deferred the full create-PR and
merge gates. The separate generated-code-quality surface inventory still
declares 727 paths and an old path digest; that pre-existing drift is recorded
with [its owner](https://github.com/sifr-lang/sifr/issues/3744#issuecomment-5804948475).
F3 positional selection remains separately open. SQL Item 4 must run its own
final-candidate qualification; this receipt resolves only its inherited B11
self-test blocker.

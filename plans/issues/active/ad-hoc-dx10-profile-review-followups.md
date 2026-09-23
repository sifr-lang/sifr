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

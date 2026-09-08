# 12K-B44 — Application-demand stdlib interop resolution

## Disposition

**TERMINAL HOLD for B27.** The bounded implementation and named validation are
complete; exact-source Opus review is SATISFIED. The one canonical full
representative invocation completed all ten cases and six rules, but its actual
budget consumer failed. This is **not** a full representative pass, delivery,
PR, merge, or permission to retry. No source repair followed the live failure.

- Source base: `4a867cbde6d83e56ec7b66ca32887388dfa42803` (complete B42 prerequisite).
- B44 candidate: `93111f3426245b3ba5484195b8154d3bfc769769`.
- Source branch: `codex/item12k-b44-rub8EG`, pushed independently.
- Record branch: `codex/item12k-b44-record-rub8EG`, this documentation-only child.
- Actual main at registration and prelaunch: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
- PR: none. Full/create-pr/merge gates: zero. Merge: none. Next item: not started.
- Owner: <https://github.com/sifr-lang/sifr/issues/3776>.

The user's explicit B44 implementation/qualification decision superseded B43's
historical proposed/not-authorized wording and the generic merge workflow.
Original joint B26/B27 integration and full-gate authority remain separate.

## Isolation and authenticated prerequisites

Independent remote clone: `/private/tmp/sifr-b44.rub8EG/sifr`; own index, refs,
object store (no alternates), default private `target`, and sibling `evidence`.
Test/preparation TMPDIR: `/private/tmp/sifr-b44.rub8EG/tmp`.
Qualification TMPDIR: `/private/tmp/sifr-b44.rub8EG/qualification-tmp`, created
separately and verified empty before launch. `CARGO_TARGET_DIR` was unset.
No shared/predecessor target, generated-cache prewarm, or foreign cleanup.
Parent worktree and phase ledger remained read-only and intentionally dirty;
parent exclusively owns its phase-record update. Registration, capacity,
prelaunch, meaningful-result and terminal callbacks target parent and Police.

Authenticated B43 record `057b69e295e1eb42a16caa9a5225a1e2f45b8e69`, full report,
four linked companions and 71-entry source/evidence inventory; B42-HOST's
37-entry inventory; and B42's 56-entry inventory. The saved authentication binds
168 distinct files at `evidence/input-authentication.json`. Inherited evidence
is not new B44 test or review evidence.

B43 report SHA256:
`844c339d0fdea4fe61a0a875691a92aae3afba6bdfd50280d94d770f0b0328c4`.
B43 inventory SHA256:
`937f7aee3616ac43d7124300592132a13da9b82dde1f2f2cd31a3a007492c65a`.
Its excess whole-inventory resolution finding supplied the B44 mechanism scope;
exact attribution of the historical 120-second failure remains unproven.

The B42 ten-path source delta is byte-identical in the B44 freeze: formatter
globs/components/negation/literal validation, explicit-file bypass, lazy config,
target ordering/norespect, and warmup/failed-query raw-evidence behavior remain.
No old compiler/SQL/whole-lock stack was imported. The minimal B42 lock edge is
retained, not a new B44 lockfile change.

## Selected-plan mechanism

Bootstrap retains all checked stdlib HIR in `StdlibCode.hir_modules`, alongside
the full existing private interop inventory. Codegen follows checked module and
declaration identities to construct an application-specific plan, independently
of emitted text or used-module/feature sets. Application declaration imports are
roots; stdlib imports are transitive edges, including public reexports and
private-import source-name aliases. Function bodies, stored types, defaults and
nominal/structural signatures participate.

A required class is an emission unit: fields, parent/newtype types, all methods,
operators and cleanup contracts participate. Compiler-owned builtin-open and
raw-Python attribute/item support add their explicit declaration edges. A
deterministic declaration set is projected back through the existing interop
contract builder, preserving bridge layouts and structural identities rather
than manually filtering signatures.

The separate `InteropBuildPlan.stdlib_demand` carrier reaches the actual shared
native project and single-file attachment path. Only that selected plan is
merged with user contracts, using the existing separately validated sysroot
context. Package check keeps its existing resolver; plain frontend check is not
converted into indiscriminate probing. Emit keeps trusted-sysroot deferral.
Signature, unsafe-function, opaque Send/Sync, borrow, async, callback and panic
validation authorities and user/sysroot trust separation are unchanged.

The stdlib-only resolved-plan cache binds deterministic selected contracts plus
the existing sysroot identity. Package-owned contexts bypass that cache. The
general probe cache, probe scheduling and hermetic input keys are unchanged.
Whole-sysroot certification remains in its owning toolchain qualification lane.
Architecture documentation records this actual inventory/selection boundary.

Fourteen B44 paths are frozen in
`evidence/freeze.93111f3426245b3ba5484195b8154d3bfc769769.json`, SHA256
`047bd8856dc37df6d0f204acd06ee4b49886153f5e523da7fcffcce8e8a998bd`.
The freeze includes source blobs/hashes, unchanged B42/performance inputs, and
each final named command's receipt and log hashes.

## Named validation

All Cargo tests used `--locked -p sifr_driver --lib` and
`-- --test-threads=1`, with test-only TMPDIR. No codegen-owned demand tests were
needed, and no unsolicited broader suite ran.

`stdlib_interop_demand` group: **5 passed** (`evidence/demand-06.*`):

- `stdlib_interop_demand_additional_modules_excludes_unrelated_backends`:
  real additional-modules codegen and ExecuteAll resolution select required
  calendar/html/compress/sys/fs contracts, excluding Python/HTTP/i18n inventory.
- `stdlib_interop_demand_transitive_reexports_and_support`: checked facade and
  private-call closure, generated nested nominal layouts, structural identity,
  opaque consuming cleanup/method contracts, and implicit builtin-open support.
- `stdlib_interop_demand_real_python_retains_contracts`: real Python selected
  contracts/probes and five-field PythonError layout; invalid selected signature
  is rejected by ExecuteAll despite sysroot trust.
- `stdlib_interop_demand_project_single_file_parity`: actual project/single-file
  selected and resolved plans agree; emit retains the same deferred contracts.
- `stdlib_interop_demand_cache_disjoint_programs`: A/B and B/A, repeat demand,
  sysroot change and trust-contract change do not leak resolved plans.

Nine existing filters each passed once, with same-named `evidence/*.json` and
`*.log` receipts:

```text
private_stdlib_interop_resolves_sysroot_crate_target
merged_user_and_private_stdlib_interop_both_resolve
merged_user_and_private_stdlib_interop_keeps_user_trust_separate
package_rust_interop_direct_probe_checks_signature_shape
package_rust_interop_direct_probe_accepts_bridge_signature
package_rust_interop_direct_probe_rejects_unsafe_fn
package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation
package_rust_interop_cache_fragment_is_deterministic
package_rust_interop_cache_changes_with_trust_policy
```

`cargo fmt --check`, HIR maintainability, repository file-size guardrail (3777
files, limit900), and `git diff --check` passed. Initial compile/alias/fixture
failures `demand-01` through `demand-05` remain intact; corrections were confined
to this implementation and these named tests. Every failed and passing test
receipt reports full process release. No failure receipt was overwritten.

## Exact-source review

One initial Opus source-only request; **SATISFIED**, zero blocking findings,
zero remediation rounds, zero provider failures. Exact base/candidate above;
no reviewer tests or new requirements requested. Skill invocation used
`claude-opus-5`, medium effort, plan permissions, project settings, no session
persistence and a2400-second watchdog. Completed response was atomically
published outside Git; review group65633 was absent after completion.

[Raw review on owner3776](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5587756056).
Local `evidence/opus-review.93111f3426245b3ba5484195b8154d3bfc769769.md`, SHA256
`a8e386df7333c912ad7594854814cc1f6a1454cd64e114feb2414fca0bb3b2d4`.

Non-blocking observations retained without starting new work: unused selection
work during bootstrap; `lib_project_codegen.rs` at899lines; duplicated synthetic
demand-injection test helper; future declaration-valued clone/shutdown hooks
would need explicit edges (current contracts use keywords). These observations
are not established attribution of the measured budget failures.

## One canonical full representative

Ordinary locked private compiler/frontend preparation passed (90.202s and
43.783s), with process release. These are not performance samples, despite the
reused helper receipt names `cold-sifr-build-01`/`cold-frontend-build-01`.

Frozen binaries:

- `target/debug/sifr`, SHA256
  `e93632d3f893876e8edc7aba2d0fdea32155f32c443f67bd37511edb73246655`.
- `target/debug/frontend_query_bench`, SHA256
  `26a39a7278f39aab9c239cfe83451f6e8b217c7915199ac93812879f0e6b163c`.

Prelaunch: AC80%, free76GiB, private target11GiB, no competing heavy process,
no alternate LSP command. Original LSP binary invocation is `sifr lsp --stdio`.
`evidence/launch-binding.json` SHA256
`96ad31c5b41e4b4e6742e99d2a7208bf72811af3e8aaa8912c2720b873f4487a` binds the
actual supervisor and exact source/binary/workload/budget inputs. Supervisor
SHA256 `c4d26ef72e6a12f14c47b980461c3ce32b77262c8522a856edbff7a1a0fb55c1`;
authenticated HOST control flow was unchanged, with only explicit test versus
qualification TMPDIR binding. B23 case-group cleanup remained unchanged.

```sh
uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
```

Actual internally generated invocation:
`performance-representative-1788881948856349000`; run directory
`target/performance/bench-1788881948-87646`.
Producer and actual budget consumer use this same ID. Original all10cases,
six rules, counts, at most3 controlled attempts, host admission/monitoring,
120000ms case limits,1800s area supervisor and at most120s cleanup (maximum1920
inside7200) remained unchanged. No external retry, samples banked from old runs,
budget bypass, extension or generated-artifact-cache prewarm.

**All10 cases accepted on attempt1; six rules PASS; producer PASS; actual budget
consumer FAIL.** Seven command cases have one warmup and five measured samples
each. Three query cases retain three internal warmups and20 measured samples,
with21 raw process receipts each (aggregate plus work samples). Total105 raw
process-sample receipts; no timeouts, and the diagnostic case's expected exit1
was retained. Root exit1 after200.356285833s, no outer timeout, no cleanup signals.
Supervisor observed296 process identities in69 groups and found all absent.

The additional-modules cold generated-project warmup completed in
31104.059292ms, exit0, followed by five accepted measurements (median8302.862ms;
median69,816,057,092 retired instructions). This concrete result does not prove
the historical timeout's exact cause or historical formatter CV attribution.

### Actual budget failure (same invocation, no waivers)

| Case | Metric | Measured | Threshold |
| --- | --- | ---: | ---: |
| build-single-file-001-break-continue | peak RSS bytes | 185466880 | 181649408 |
| check-project-004-project-graph | median instructions | 20705027673 | 13938915299 |
| check-project-004-project-graph | peak RSS bytes | 185597952 | 179601408 |
| check-single-file-001-arithmetic | median instructions | 20651201663 | 13879420485 |
| check-single-file-001-arithmetic | peak RSS bytes | 185139200 | 181436416 |
| diagnostic-non-regression-002-json-diagnostic-schema | median instructions | 20645274502 | 13874003870 |
| diagnostic-non-regression-002-json-diagnostic-schema | peak RSS bytes | 185253888 | 180748288 |
| formatter-corpus-001-project-check | median instructions | 46400530 | 38059926 |
| lsp-query-003-diagnostics | peak RSS bytes | 150945792 | 108593152 |

Instruction lower bounds/uncertainties and original errors are preserved in
`evidence/budget-disposition.json` and the canonical log. Checker exit1,
61.018ms, is recorded in the area's `budget-subset` variant. Importantly,
`representative.budget.latest.json` is the producer results consumed by that
checker, **not a separate checker report**. The reused collator's historical
`budget_report_present` field only denotes that input's presence; the explicit
budget-disposition record removes that ambiguity. No checker rerun occurred.

## Raw evidence and process release

All following paths are relative to `/private/tmp/sifr-b44.rub8EG`:

- `evidence/representative-01.json` — command, PID/start identities,69 groups,
  deadlines and complete release; SHA256
  `605f8ce54a03df1b82c88a3c62d77110d504d48e5f93dcc17f5ff80cba8ef2fe`.
- `evidence/representative-01.log` — full canonical output including nine budget
  errors; SHA256 `b8785430805e5e7d43af341a81b4404f4ea581c8bbeec459d283aa2ef580fcb8`.
- `evidence/representative-summary.json` — all attempts, samples, actual
  variants, verified source/binary/workload/ancestor hashes; SHA256
  `57920de482a490043003b87f990bb0ce39416042ddbc59e7176b300f1825cc84`.
- `sifr/target/verification/areas/performance-results.json` — actual producer
  and budget commands/exits; SHA256
  `3d91dbdabcf7783d80dc20e2ced49dc11abd850f6a4928d9cd6461f22381bf84`.
- `sifr/target/performance/evidence/bench-1788881948-87646.json` and
  `sifr/target/performance/representative.budget.latest.json` — identical actual
  producer data, SHA256
  `265ebd0ba1bdc919bce863354a494d5ad564c96a520437ba8a9c1c0dca3e3616`.
- `sifr/target/performance/bench-1788881948-87646/attempts/1/` — all105 raw
  command/query samples, stderr/stdout sidecars, ten results and host snapshots.
- `evidence/final-process-release.json`, `evidence/evidence-inventory.json` and
  `evidence/terminal.json` — final all-command/review-group absence, full hash
  inventory and source/record/owner bindings, sealed outside the reviewed Git
  tree after the record push. Terminal callback supplies their final hashes.

## Retained boundaries and next owner action

B42's AC-admission failure and B42-HOST's first-warmup120011.6566ms timeout are
immutable failed proof, not reset or replaced by B44. B20/B22 review exhaustion,
B42's separate review, and original12K/B13/B14/B15 failed gate counters remain.
No full gate, PR, merge, whole-phase Opus or next-item implementation ran here.

B27 owns the concrete joint-integration decision using this held source and
failed qualification. Original264 fresh companions/90native/411check/Python30/
exact65SQL/all-area/toolchain/full-gate obligations remain, as do12D/E/F,
full12 emitted quality and final12A documentation closure. Original3717's draft
delivery is still unmerged. Historical formatter CV remains UNKNOWN. Nothing
here claims main SQL/SDK dependencies unblocked or changes original acceptance.

**Stop after durable owner/record/evidence handoff. No B44 retry or next item.**

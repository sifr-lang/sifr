# 12K-B43 — saved first-build timeout assessment

Assessment complete; implementation decision remains with coordinator Police.
This is a docs-only record, not a repaired compiler, performance PASS, phase
closure, or permission for another run. No PR or merge. B43 consumed zero builds,
tests, benchmarks, profilers, Opus reviews, gates, and live-target diagnostics.

## Finding and decision

The strongest evidenced mechanism is **eager application-independent private
stdlib bridge validation on the ordinary application-build path**. Sifr collects
the entire private stdlib interop plan, attaches it without selecting application
demand, and executes its pending direct Cargo probes serially. The saved first
build was still progressing through those feature-specific dependency closures
near its timeout, ending with an unrelated Python bridge probe. This is not a
claim that Cargo checked every workspace crate or used `--all-features`.

The additional-modules demo does not import Python, HTTP, or i18n. Nevertheless,
the retained unfinished probe checks `sifr_stdlib::python::py_from_none`; the
authenticated process inventory includes HTTP/TLS, i18n and Python dependency
compilation. This is concrete excess validation work caused by compiler demand
selection. Cold first-use compilation is the immediate work being performed,
not itself a defective Cargo behavior. Ordinary compiler cold preparation builds
a different target/dependency graph and cannot establish readiness of this
generated probe graph.

Recommend a product/compiler repair: select the complete application-required
stdlib interop closure before attaching/resolving it, retain every required
contract, and make resolved-plan caching demand-aware. Do not solve the excess
work by prewarming it, raising the timeout, skipping warmup validation, globally
trusting away probes, or importing an old stack. Saved evidence does **not** prove
that this repair alone will bring a genuinely cold complete native build below
120 seconds. That remains a bounded future qualification question.

## Ownership, exact sources, and custody

Sole assessor's independent HTTPS clone: `/private/tmp/sifr-b43.0wKIu1/sifr`;
branch `codex/item12k-b43-0wKIu1`; private sibling `tmp` and `evidence`, own index,
refs and object store, no alternates/shared target. Registration preceded
assessment writes and was sent to parent and Police. Parent's intentionally
dirty worktree and all predecessor clones, indices, caches and evidence were
read-only. Commands used own TMPDIR and unset CARGO_TARGET_DIR; no Cargo/uv or
process sampling was performed. Both named skills were read and used for bounded
ownership/evidence/closure; the explicit saved-assessment override disallowed
their ordinary implementation/review/gate loop.

Examined current main/base `4b4cc339964baeeb6641e57dc669fef700a5fa24` (fresh remote
verification unchanged), failed candidate `4a867cbde6d83e56ec7b66ca32887388dfa42803`,
B13 `98480c78587d6cbd99a7079d10c71825360bd468`, B19 measurement source
`3181cc6dcd4bed372d6aa3291cc30ececc2f5538`, B19 reviewed source
`711b5a424cf9f2473f07693b3706f23b14587ba1`, and B20
`ac4c277b30c6cac046ab1746fe4020c04df7eaf1`. The generated 32-path × 6-revision
crosswalk records Git blobs and SHA-256; all 32 retained HOST source paths match
the failed candidate. Main-to-failed differs in exactly ten B42 paths, including
formatter and warmup handling, not the core attach/probe/cache mechanisms below.
No rebase, cherry-pick or source change occurred.

Linked durable companions:

- [Saved authentication/process/probe inventory](ad-hoc-emitted-rust-b43-artifacts/saved-analysis.json).
- [Exact-source/cache/history crosswalk and Git read receipts](ad-hoc-emitted-rust-b43-artifacts/source-crosswalk.json).
- [Binary authentication and selected historical/timeline facts](ad-hoc-emitted-rust-b43-artifacts/final-saved-facts.json).
- [Companion provenance](ad-hoc-emitted-rust-b43-artifacts/provenance.json).

Reauthenticated the complete B42-HOST inventory (37 files), B42 inventory (56),
B41 terminal-bound records (26, including its eleven companions), B19 inventory
(52), B20 terminal/inventory and two relevant bound records. This does not claim
a fresh reauthentication of all thousands of B41's transitive historical inputs.
B13's two B19-bound preservation maps, six relevant preserved rows, and its
gate-outcome/receipt/log were separately authenticated. HOST compiler and frontend
binaries were also rehashed against the terminal's exact identities.

Primary SHA-256 anchors:

| Artifact | SHA-256 |
| --- | --- |
| HOST terminal | `0b5f61e533e39d75aa0dc847aada4d7d6b805eaa496ef1201fb4a4909d5de16a` |
| HOST 37-file inventory | `02dca5d299b08e7cf0775ac0cf34cf1f84d96f19a1380a772c4bbefe904db8a0` |
| Representative summary | `0b0795db213a66868c7a8d5343a49795209d5b6fd6fdd14a3f335d53017d739c` |
| Representative receipt | `c7cae91614fb98c68a74571c6fad8c7ba677db004ddfc5d0de3b207b8f4c0e94` |
| Representative log | `c54278d5c9bdc55b13dff75137a33f3f6c7e1343740a6006ce8d6ab5e21ebb77` |
| Saved supervisor source | `4d6b17d72d7de79fd9b9a4d02ff57aea95a786a6cb591829a8dfd27a1841fd43` |
| Final process release | `d02a588a4aa6342f710a28cf4de669258b17ecb0383bdd381a7fc76194104434` |

HOST evidence root is `/private/tmp/sifr-b42-host.FVuvOb/evidence`; its full record
is commit `d56abf62dd958776572551ff3772c263ca499acb`,
`plans/issues/active/ad-hoc-emitted-rust-b42-host-continuation.md`.
Owner [outcome](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5586635621)
and [terminal](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5586650951)
remain authoritative. B43-generated inventories distinguish originally sealed
files from newly read retained cache/probe files: the latter were **not** among
the original 37, so their present hashes/mtimes are not retroactive timing seals.

## What the first build was doing

The one canonical invocation was
`performance-representative-1788876840586332000`. AC admission passed; six rules
passed. First lexical case `build-project-001-additional-modules`, attempt 1,
warmup sample 0 ran the private `target/debug/sifr build` on
`demos/additional_modules/main.sifr`, with the sample's private shared-build
output. It timed out at `120011.6566249635 ms` against the unchanged `120000 ms`
limit: exit null, timeout true, one exceptional attempt, one warmup, zero measured
samples, nine cases unreached, 25 host snapshots. stdout/stderr were actually
zero bytes and counters unavailable. Producer FAIL; budget BLOCKED/not executed,
not an accepted stale report. Enclosing exit 1 after `128.43871637503617 s`, no
outer timeout. The saved final release passed for owned groups
33395/33429/33436/33589/33597, all prep/docs groups, no foreign signals, and old
review process 72937 absent.

The following are **process start timestamps from retained observations**, not
phase durations or completion times (2026-09-08, saved host local time). The
supervisor sampled every 0.5 s and retained the last observed row per PID, not
every snapshot. Short-lived processes can be missed; no CPU/stacks/exit trace is
available.

| Observed start | PID / parent | Saved command identity |
| --- | --- | --- |
| 16:14:03 | 33590 / 33589 | Runner's first-use workspace `cargo build -q -p sifr` |
| 16:14:04 | 33598 / 33597 | Timed `sifr build` application command |
| 16:14:06 | 33658 / 33598 | Workspace Cargo metadata |
| 16:14:11 | 33833 / 33598 | Vendored Cargo metadata |
| 16:14:12 | 33900,33884 / 33872 | `indexmap`, `encoding_rs` rustc invocations |
| 16:14:15 | 34105 / 33598 | Another private bridge Cargo check |
| 16:14:22 | 34742 / 34573 | `sifr_runtime`, default feature set |
| 16:14:45 | 38218 / 33598 | Private Cargo check whose closure includes HTTP/TLS |
| 16:15:03 | 45012 / 38218 | `sifr_runtime`, default/http/net/tls |
| 16:15:15 | 46161 / 45370 | `sifr_runtime`, i18n feature closure |
| 16:15:40 | 49548 / 49543 | `sifr_runtime`, default/net |
| 16:15:57 | 52340 / 33598 | Private Python bridge Cargo check |
| 16:16:02 | 52417 / 52340 | `pyo3_build_config` compilation |

There are 426 observed process identities, 81 retained Cargo command rows,
including 73 `cargo check` rows, and 68 rustc rows. These are observed counts,
not the total number of invocations. Feature-distinct runtime compilations are
not evidence of recompiling an identical graph without reuse. Native dependency
work (including AWS-LC in the TLS closure) appears in the saved inventory; no
precise time allocation to it is justified.

Retained cache metadata contains 257 successful `.ok` markers and one unfinished
probe directory, under HOST's private tmp:
`sifr_rust_probe_33598_1788876957681206000_257_341e092a97168f3f`.
Its manifest names `sifr-rust-probe` version 0.0.0, edition 2024; dependencies are
the exact HOST path `sifr_stdlib` with default features disabled and `python`
enabled, plus the exact HOST path `sifr_runtime`. Its source asserts the signature
of `sifr_stdlib::python::py_from_none`, including a generated Python error bridge.
Manifest SHA-256 `803574657cd0beaebe851a916c779ef33a72a838f3df85d743f7d6577729a626`;
source `0c0f54711862ef56090877143f56eed5d74f317ec724029ea4dd1a0086994ec6`;
lock `5d6b2715892c1c8e0f8c0681939a3f00cbeae25da015d07feca3c72a77d98b1b`.
The lock matches prepared-resolution key `8abc6df2029b6ab5`, 35 packages,
including registry/checksum-identified pyo3, pyo3-build-config and pyo3-ffi 0.29.2
(full identities in the crosswalk). It is not the workspace Cargo.lock.

Source increments the nonce only after a marker miss and writes markers only
after successful checks. Counter 257 plus 257 retained markers is consistent
with the 258th cache-miss probe workspace, but there is no sealed per-declaration
mapping for every opaque marker or exhaustive historical command trace. Do not
promote that consistency inference to an exact independently sealed probe count.
The final application artifact directory is absent at B43's retained-file read;
that is not a historical snapshot. Source ordering places these probes before
completed application project materialization, consistent with the retained run.

Empty captured stderr is not evidence of a hang: the subprocess uses `--quiet`
and Sifr buffers its `.output()` until completion. Actual later process starts
and retained success markers support progress; they do not rule out brief stalls
or quantify CPU/IO/host contention. No historical CV causality is inferred.

## Source mechanism and cache separation

All line numbers below refer to failed candidate `4a867cb...`; these mechanism
files are byte-identical on examined main. Paths are repository-relative.

| Owner / exact source | Mechanism |
| --- | --- |
| `crates/sifr_driver/src/stdlib/bootstrap.rs:33`, `:57`, `:120`, `:555`; `stdlib/interop.rs:14`, `:30`, `:58` | Load sysroot stdlib sources, collect private Rust-backed modules across the inventory, make one interop plan from all of them. |
| `crates/sifr_codegen/src/rust_interop_plan.rs:349`, `:377` | Collect supplied modules' functions/classes/methods; this inventory is not application demand. |
| `crates/sifr_driver/src/build/sysroot_interop.rs:28`, `:78` | `attach_stdlib_rust_interop` unconditionally merges nonempty whole plan; extends declarations, signatures, generated types and structural identities. |
| `crates/sifr_driver/src/build/entrypoint.rs:655`, `:688` | Native project path defaults to ExecuteAll; attaches before applying resolved interop metadata. |
| `crates/sifr_driver/src/build/rust_interop.rs:168`, `:196`, `:681` | Resolve each declaration and execute pending direct probes serially. |
| `crates/sifr_driver/src/build/rust_interop/probe_planning.rs:116` | Concrete direct bindings become pending probes, carrying signature and trust metadata. |
| `crates/sifr_driver/src/build/rust_interop_probe.rs:76`, `:97`, `:103`, `:142`, `:145`, `:162`, `:170` | Feature-specific manifest, marker lookup, unique temp workspace, prepared lock, vendored locked/frozen check, buffered output, success-only marker. |
| `crates/sifr_driver/src/build/rust_interop_probe_features.rs:4`; `rust_interop_probe_manifest.rs:6` | Rust path selects a declared leaf feature; dependency is not blanket all-features. |
| `crates/sifr_driver/src/build/project_codegen.rs:230`; `crates/sifr_codegen/src/lib_project_codegen.rs:405`, `:409`, `:503` | Existing used-stdlib/feature and rendered-support demand metadata is separate from whole-stdlib interop attachment. |
| `crates/sifr_driver/src/build/single_file_interop_cache.rs:26`, `:56`, `:79`, `:84` | Single-file path also attaches whole plan; process cache key is sysroot identity + global plan, currently not application demand. |

The `emit` project path deliberately uses DeferTrustedSysroot
(`entrypoint.rs:172`); package-project check uses ExecuteAll (`build/api.rs:109`,
`:129`), while plain `check_project` returns frontend diagnostics (`:99`). Do not
claim every command takes the same probe policy or change those policies indiscriminately.

| Cache/preparation layer | Exact identity and implication |
| --- | --- |
| Workspace compiler | HOST `sifr/target`; locked cold preparation 94.893763792 s for sifr and 45.718588667 s for frontend, both PASS. Authenticated binary hashes are in the companion. This compiles compiler tools, not all generated backend feature combinations. |
| Probe Cargo target | `build/rust_interop_probe_paths.rs:5`: configured CARGO_TARGET_DIR if present, otherwise `artifact_cache_root()/rust_bridge_probe_target`. `build/workspace.rs:272` roots that at process temp dir. HOST therefore used `tmp/sifr_generated_artifact_cache/rust_bridge_probe_target`, distinct from workspace target. |
| Successful direct-probe cache | `rust_interop_probe_cache.rs:62`: v1 key includes toolchain/environment signature, package/dependency identity, absolute backend manifest, Rust path, generated manifest/source, lock mode, backend/runtime/lock/vendor and optional SQLX metadata digests. Nonce is not part of semantic key. Fresh TMPDIR isolates markers; absolute backend paths also distinguish clones. Neither proves within-run key churn. |
| Prepared Cargo resolutions | `cargo_resolution.rs:348`: v7 key includes normalized manifest, vendor args, authoritative lock digests and trusted vendor paths. `:374` normalizes dependency paths to manifest identities; ephemeral probe root is not blindly hashed. Nineteen retained prepared locks are bound. |
| In-process resolved stdlib plan | `single_file_interop_cache.rs:84`: sysroot+whole plan. A future selected-plan cache must include selected demand/contracts (or cache only demand-independent inventory and resolve per demand). Otherwise the repair could introduce cross-application contamination; that is a repair obligation, not evidence this cache caused the cold timeout. |

Both retained `.rustc_info.json` files identify rustc 1.98.0,
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, LLVM 22.1.8. The recorded rustup paths
use 1.98.0 versus stable and differ in fingerprints; a compiler-version mismatch
is not established. Probe locks/features/source units differ from the workspace;
different compilation fingerprints are expected, not a cache-corruption finding.

The runner initializes readiness flags false even when binaries exist and calls
Cargo on first use (`run_benchmarks.py:108`, `:516`–`:560`). Its ensure step
precedes the sampled command (`:421`). This existing behavior explains the
observed pre-sample Cargo build, not a proven root cause inside the timed sample.
B42 checks timeout/exit before excluding warmups (`:434`–`:447`); preserve that
correction. The first failure is now honestly surfaced, not a formatter result.

## Alternatives and historical limits

Rank 1: whole-private-stdlib probe fan-out is directly established by source and
unrelated retained probe identity. It is the root-cause repair target for excess
work. The data cannot establish it as the sole sufficient cause of crossing
120 seconds, or predict savings precisely.

Rank 2: expected cold dependency compilation in a separate target/feature graph
is evidenced. It amplifies the fan-out cost. Some first-use work remains necessary
after proper selection; a benchmark policy promising generated-cache readiness
would need explicit preparation semantics, but ordinary compiler coldprep made
no such guarantee. No prewarming/policy change is a prerequisite to the product
repair proposed here.

Rank 3: per-declaration Cargo/metadata overhead and feature changes plausibly add
cost. They are visible, but no per-call durations or duplicate-identical-key miss
history exists. Batching, persistent certificates, cross-clone cache relocation,
or key redesign are not justified as required repairs from this evidence.

Rank 4: a hang, stale/shared-cache corruption, host stall, or toolchain mismatch
is not established. Empty stderr/process presence alone cannot decide those
questions. Private HOST lifecycle release differs from B20's invalid isolation;
do not regress B23 owned-process cleanup or blame it without proof.

B19's retained acquisition has eighteen formatter samples, not a retained
build-project warmup receipt. Its measurement source, reviewed source and reused
B13 compiler provenance are distinct. The B19-bound B13 preservation maps bind
the older generated additional-modules manifest: leaf features
`calendar, fs, gzip, html, sys, zipfile`, no Python/HTTP/i18n. Current demo import
closure (lexically inventoried, not executed/lowered) likewise excludes those
unrelated groups. This is corroboration, not a newly generated current manifest.

B13's authenticated log line 1321 reports first build **case aggregate**
100496 ms/PASS, not a raw warmup duration. Its gate included 92-graph preparation
(1601078 ms), emitted freshness (722863 ms) and earlier generated workloads.
No retained before-first-use cache inventory establishes exactly what was warm.
Core whole-plan attach/probe source already existed then; the six-revision
crosswalk does not support calling this a newly introduced B42 mechanism.
Differences in preparation context matter, but are not proof of historical timing
causality. The old runner also lacked B42's warmup validation ordering.

B20 first-build warmup 120004 ms remains INVALID_ISOLATION_ABORTED: retained
records show orphan 85215/next 3862 and overlapping bridge checks in shared TMP.
That does not establish the same failure as private, cleanly released HOST.
B42's first canonical invocation failed AC admission with zero samples, not a
build failure. B41 established the absolute-formatter-path selection defect and
eighteen successful formatter exits with excessive CV; it did not prove the
origin of that variance. B42 restored intended formatter workload, not a proven
historical CV cure. No B40 perturbing LLDB measurement is promoted to causality.

## Minimal full repair proposal — not implemented or authorized here

1. Keep the full sysroot declaration inventory for bootstrap/type information,
   but introduce one canonical application-stdlib interop demand owner. Select
   before `attach_stdlib_rust_interop` merges declarations/contracts. Use typed
   canonical module/owner/declaration identities from lowering/codegen, including
   public reexports, transitive private calls, generated support and nominal or
   structural types. Existing used-module/feature sets can guide selection but
   are not by themselves proof of complete callable/contract closure. Do not
   parse emitted text or use this assessment's lexical regex as the repair.
2. Close demand over required signatures, generated bridge types, structural
   identities, class methods/operators and cleanup/drop obligations. Deduplicate
   canonical identities. Preserve genuine user-package interop and selected
   stdlib signature/unsafe-function/opaque Send-Sync/borrow/async/callback/panic
   contracts. A demo with no Python demand must not create a Python probe; a
   genuine Python program must still validate its Python boundary. Preserve
   compile-time diagnostics and no user-triggered runtime panics, not bypasses.
3. Thread that same selected plan through native project and single-file metadata
   resolution, including the check paths using them. Key the single-file
   resolved-plan cache by deterministic selected demand/contracts and existing
   sysroot identity, or separate immutable inventory caching from demand-specific
   resolution. Cover sequential disjoint programs in one process. Preserve emit
   deferral and user/sysroot trust separation. Do not add a new test-runner lane
   merely because it is adjacent; integrate only actual shared consumers.
4. Retain current hermetic vendor/lock enforcement, marker correctness and owned
   timeout cleanup. Keep whole-sysroot certification in its owning qualification
   boundary, not as mandatory validation of every unrelated application build.
   No cache/prewarming redesign, acceptance relaxation, or optional benchmark
   instrumentation project is required to deliver this mechanism repair.

Proposed focused checks for an explicitly approved implementation item (names
below prefixed `stdlib_interop_demand_` are **new checks to add**, not existing
tests or B43 results):

- `stdlib_interop_demand_additional_modules_excludes_unrelated_backends`: exact
  selected plan/probe requests exclude Python/HTTP/i18n and retain all required
  demo bridges; no assertion on timing.
- `stdlib_interop_demand_transitive_reexports_and_support`: closure across a
  public reexport, private call, generated support, nominal/structural bridge and
  cleanup obligation; no unused-inventory fan-out.
- `stdlib_interop_demand_real_python_retains_contracts`: genuine demand remains,
  including invalid-signature rejection rather than a trusted global bypass.
- `stdlib_interop_demand_project_single_file_parity`: equivalent project/check
  and single-file programs produce equivalent canonical selected contracts.
- `stdlib_interop_demand_cache_disjoint_programs`: A then B and B then A in one
  process, identical-demand determinism, changed demand/sysroot/trust separation.

Run the new focused group once with
`cargo test --locked -p sifr_driver --lib stdlib_interop_demand` (and the same
filter in sifr_codegen only if its new demand owner has tests there). Preserve
existing named regressions, once each via the driver test filter:
`private_stdlib_interop_resolves_sysroot_crate_target`,
`merged_user_and_private_stdlib_interop_both_resolve`,
`merged_user_and_private_stdlib_interop_keeps_user_trust_separate`,
`package_rust_interop_direct_probe_checks_signature_shape`,
`package_rust_interop_direct_probe_accepts_bridge_signature`,
`package_rust_interop_direct_probe_rejects_unsafe_fn`,
`package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation`,
`package_rust_interop_cache_fragment_is_deterministic`, and
`package_rust_interop_cache_changes_with_trust_policy`. These exist in the examined
source; no test was executed for this assessment.

After deterministic closure/contract checks pass, propose **one** separately
authorized exact-changed-SHA canonical representative invocation:
`uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative`.
Keep all ten cases, six rules, current admission, samples/warmups/baselines,
120000 ms per-case timeout and producer-to-budget identity. Execution cap 1800 s
plus at most 120 s cleanup (1920 total inside 7200), first failure stops, no
external retry. Use an owned isolated qualification TMPDIR not prewarmed by
focused tests; ordinary locked compiler preparation is preparation, not timing
evidence. A failure must retain its actual sample/process release evidence and
return to the coordinator, not trigger timeout changes or an unchanged rerun.
This is a proposal, not a new gate or review allowance; B43's allowance is zero.

No live experiment is needed to identify the eager-selection mechanism. Missing
for a stronger timing claim: a complete per-probe start/end/cache-hit trace and
a demand-correct changed-source qualification. Request those only if the
coordinator needs cost attribution or the later qualification remains blocked;
they are not self-authorized here or optional prerequisites to the scoped repair.

## Closure boundary

Only this assessment and generated companions are recorded. The relevant docs
diff and file-size guardrail are the only final checks; command receipts, final
source/evidence inventory and terminal are retained under the owned evidence
root and published on owner 3776 with the record SHA. B42's one exact-SHA Opus
SAT and existing tests remain reused evidence for B42 only, not approval of a
hypothetical B43 fix. Old gate/review limits remain unchanged.

B26's original delivery semantic unit (264 companions; 90 native/411 check;
Python 30; exact 65 SQL; all-area/toolchain/full gate), unclosed B24 causal
adjudication and B27 integration/allowance, draft 3717, retained 12D/E/F/full12
quality/final12A closure all remain later work. Main SDK/dependency changes do not
authorize stack import or a new SQL/full-gate cycle. No architecture or roadmap
status change, parent phase-ledger edit, next item, implementation or new owner.

Assessment blocker: none. Delivery remains blocked on an approved repair and
actual qualification; the report does not close that delivery. Callback Police
for implementation decision, then stop.

# Existing 12K-B25: held builtin/list-repeat source delivery

Status: source batch finished; scoped validation and bounded review satisfied;
UNMERGED, full qualification HELD. Worker RETIRED after publication.
This record does not close the phase or claim qualified main.

## Identity and authority

- Actual fetched main/base: `4b4cc339964baeeb6641e57dc669fef700a5fa24`; still actual main at terminal.
- Final source: `ad47c96b26104096d9f10ca5c680407de4e6ce33`.
- Published source branch: `codex/item12k-b25-held-delivery-DyhOGS`.
- PR: NONE. Merge: NONE. Owner publication: [existing issue3776](https://github.com/sifr-lang/sifr/issues/3776).
- Owned HTTPS clone: `/private/tmp/sifr-b25-delivery.DyhOGS/sifr`.
- Independent record branch: `codex/item12k-b25-held-record-DyhOGS`,
  directly based on main, with no source-candidate ancestry; owned record worktree
  `/private/tmp/sifr-b25-delivery.DyhOGS/record`.
- Evidence root: `/private/tmp/sifr-b25-delivery.DyhOGS/evidence`;
  final terminal metadata is sibling `terminal.json`.
- No alternates/shared target. CARGO_TARGET_DIR unset; private TMP and default
  target, maximum two Cargo jobs. Native/review processes ended and released;
  target8.5GiB preserved,85GiB free. No foreign cleanup or source mutation.

Latest user/Police scope overrides the older merge workflow: this existing B25
includes B21 typed registration, B26 singleton/general repeat and count/occurrence
ownership closure, plus only the two newly demonstrated mandatory driver-Clippy
prerequisites expressly approved by Police. No new Bxx, broader retained stack,
next package, performance experiment or gate. Parent remained orchestration-only.
Pre-edit scope and each added test/filter were registered to parent and Police.

## Source delivered

Private catalog-backed builtin token makes canonical identity total while retaining
partial string lookup. All four nominal collectors and both production registration
callers carry the token. Strong catalog roundtrip, stable re-registration, negative
names, absent definitions, module shadow and global WorkerError/WorkerRuntimeError
assertions are retained from original B25.

Singleton repeat selects only a statically singleton lowered Vec and emits
repeat(element).zip(SifrRange::new_known_nonzero(0,count,1)).map(...).collect.
Counts stay exact SifrInt, zero/negative are empty, and both operand orders evaluate
once in source order. General list/bytes/string behavior is preserved. Existing
owned materialization was renamed at its callers; the complete body summary checks
all statement occurrences. The proved implicit AugAssign count-read and repeated
while-condition edges were additionally fixed under Police adjudication, with
negative cases and a straight-line last-use positive control.

Workspace Clippy exposed two unchanged main driver lints. Police approved their
existing retained integration corrections in this same batch: equivalent explicit
if/else for the Cargo package alias prefix and private AuthorityPackageChecksums
alias preserving exact tuple keys, sorting, checksums and errors. Both final driver
blobs exactly match retained integration98480; no other integration source imported.

Full grouped diff:21 source/test paths,+655/-99. Every touched handwritten file is
under900 lines. No fixture, workflow, lockfile, roadmap or architecture change.

- `crates/sifr_codegen/src/body_analysis.rs`
- `crates/sifr_codegen/src/builtin_errors.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/collection_defaultdict_methods.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/collection_methods.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/literal_and_intrinsic_exprs.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/recursive_exprs.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/recursive_method_calls.rs`
- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr_codegen/src/lib_codegen_tests/reusable_value_codegen_tests.rs`
- `crates/sifr_codegen/src/project_stdlib_nominals.rs`
- `crates/sifr_codegen/src/project_stdlib_nominals/registry.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/expr_call_and_literal_helpers.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/nested_subscript_assignment_helpers.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/singleton_repeat.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_binop.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_wrappers_and_compare.rs`
- `crates/sifr_codegen/src/string_char_cache.rs`
- `crates/sifr_driver/src/build/cargo_manifest.rs`
- `crates/sifr_driver/src/build/portable_project.rs`

## Validation and preserved failures

| Evidence | Result and attribution |
| --- | --- |
| Sole full codegen at a607c3f0026d4315a1942553b81f5ea9197fc3d3 | 1413PASS/2FAIL. Both historical singleton optimization tests and both builtin registry tests PASS. New large-count oracle and while-ownership test failed; receipt retained. |
| Repeat filter at92412874b3eb35cc02ed2dc957eb9f672d02be3f | Six cases PASS, one test-oracle FAIL. Implicit AugAssign, repeated header, statement occurrences, straight-line positive control and retained reuse/effects pass. |
| Exact/general test at e11d156b030911e7a995c9464bfd1040f1bc3adf | FAIL on newly reached string zero-literal oracle; retained. |
| Exact/general test at83eea61ec4a4d0db40fa278f5fa125db1b2195ad | PASS. Exact signed bytes/sign, zero/negative/beyond-host counts, both orders and general list/bytes/string cases. Remaining six test bodies and production unchanged from924128. |
| Existing last_use controls at83eea61 | 3PASS: literal/append boundary, closure capture, set-add move. |
| Exact native builtin-shadow driver tests at83eea61 | Ordinary project1PASS and test project1PASS, each compiles/runs distinct local and builtin error shapes. |
| Workspace Clippy at83eea61 | FAIL on unchanged cargo_manifest.rs235 obfuscated_if_else and portable_project.rs339 type_complexity, both from4e24b61bb74ee768a09e168b91bb058bbafb7260. Original failure retained, no waiver. |
| Three exact portability tests at finalad47c96 | 3PASS: portable_manifest_replaces_host_paths_with_exact_sources; portable_lock_rewrites_local_sysroot_packages_to_exact_git_sources; git_lock_sources_match_the_portable_exact_revision_manifest. |
| Workspace Clippy at finalad47c96 | PASS, one run on changed candidate,51.584891796s. |
| Final fmt/HIR/file-size/exact-base diff guards | All PASS. |

All actual commands, counts, exit statuses and log hashes are in the validation
manifest. The two oracle representations (signed bytes and borrowed/primitive-zero
comparisons) were corrected without changing semantics or production; all failed
receipts remain. Police authorized targeted affected tests after the discovered
count-ownership defect. NO final full-codegen-suite PASS is claimed. Final driver
delta changes only portable formatting and a type alias; codegen inputs/tests and
ordinary/native-shadow behavior are unchanged, with affected portability tests run.
No duplicate full suite or broad gate was used to replace the failed history.

Original B25 source64847befe1722df5848a7ca99e913431f5797d0b and
record7f3330c4599cd2e59c977e4cb771487b9d343b31 remain preserved. Its terminal
SHA25653b1f40689c660c36e43a9ceb58024aa6ea908d9ed990d8e64635d3e82194fe3
retains focused2PASS/full1409PASS2FAIL and zero review/gate/merge. Retained12B's
two reviews and two FAILED gates, original12K/B13/B14/B15 histories and all
predecessor clones/source/evidence are unchanged.

## Bounded review and evidence

Initial Opus on83eea61: SATISFIED, no blocking findings. While it was running,
Police authorized the two driver corrections; user and Police required keeping
that review immutable/counted, then one update and the sole remediation review.
No cancellation, relabel, initial reset or third review occurred.

Sole remediation on finalad47c96: SATISFIED, no blocking or new mechanism findings;
unchanged codegen approval carries and fresh driver/caller inspection approves the
two-file delta. Initial1/remediation1 used; provider2/errors0; zero reviews remain.

| External artifact | SHA256 |
| --- | --- |
| source.ad47c96b26104096d9f10ca5c680407de4e6ce33.json | 2ec2d655235ab94d207d3b405ec6f100553de545c33bc0d451bde6e8739617e0 |
| validation.ad47c96b26104096d9f10ca5c680407de4e6ce33.json | 1fafe046105356cb77f8626e11b966ab65d39e147b25ebf0d8d7db1e7e4f9a14 |
| opus-83eea61ec4a4d0db40fa278f5fa125db1b2195ad.FqmImd/response.md | d935ee2ac5f5b3b33e4ac5bb7179d44b0c1ec5ced5a3bd7d42da949eb59fefed |
| opus-ad47c96b26104096d9f10ca5c680407de4e6ce33.cfyfBz/response.md | 8286b26d6c14fe05f9154c5d6ed697ecf3f4848f50579d95bab6fe264b9817e6 |
| final Clippy log | f292a2d33d4229a47fe080ed73f65697674f5cd15dd3ea52c4014616f6cfc831 |
| sibling source.bundle | 0d873ef59d8b57edda6fa9a3bbf2b8efc9c02fd3eeefd42ca683a3e1fc13ce7e |
| sibling source-evidence.tar.gz (67 files including inventory) | a1e3f8fa1d7e1103bfc63ba5ebd73bca6c8e698309273920c886c2cb66233ccb |

The portable archive contains the source bundle, raw receipts, both reviews and
prompts, registrations, source/validation manifests and a per-file SHA256 inventory.
Final review evidence lives outside the reviewed source tree. This independent
record is not inserted into the candidate it records.

## Remaining blockers and stop boundary

Scoped source blocker: NONE. Main/full qualification/merge remain HELD.

- Actual main is still4b4cc339. H PR3697 is OPEN at
  b6e6210a97598fb631b929b2d4daf4012b41bb16 (source9b52ac20094608c8a31f252db99e49ef7c963384);
  I PR3698 OPEN at19ad69969a672d7b741122ded4dd879f2bdaf9ab
  (sourcef6e8afd964bb214a44c50271dcb2014ee8e828b4);
  M1 PR3700 OPEN ata7e13eb45006eac925417491b89a932af5df2595
  (sourced726ffc11258c49f0185fd2d49697988cf90972c).
  Their generated Python field/cancellation/Result ancestry obligations remain
  actual integration/mandatory-check edges.
- Item65 PR3785 remains OPEN at4c7068b36904e216b02778f5664d2b8fd1159a6a;
  its prior field/cancellation/Result qualification failures are retained.
  No H/I/M1/65 complete source package is imported or declared merged here.
- GitHub reports only Mintlify SKIPPED on all four PRs. Main's required-status-check
  endpoint reports unprotected404; this does not waive governed local checks.
- B49 nine failed budgets remain unresolved. B52-B54 unmerged source
  6a59be12ec4076e2801652ffbf99250e67e5a258 has a failed sole154.790451s
  canonical due to formatter CV .036634/.038932/.044864 > .02; numeric checker
  NOT RUN, not a budget pass or waiver. No acquisition/retry performed here.
- No current final full-suite or governed full-profile PASS. Required qualification
  remains for the actual final integration graph. Full gates0, PR merges0,
  performance acquisitions0.

Review follow-ups are separate backlog: unverified pre-existing implicit object
reads in SubscriptAugAssign/AttributeAugAssign and optional string-test oracle
precision. They are not new phase prerequisites or permission to start another
package. No new ID or implementation follows this terminal.

Next action belongs to coordinator: after this worker RETIRES, assign a fresh
worker to the next authorized existing integration/qualification package and
consume the retained source/evidence with its precise scope and limits.
This worker stops after independent record/evidence publication and release.

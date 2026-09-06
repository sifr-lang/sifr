# Ad Hoc Issue: Algorithmic Full-Corpus Pre-Existing Failures

### Item 12B: Bounded algorithmic dependency repair

On 2026-09-05, the user authorized the same worker to repair both repositories.

- Scope: repair external conversion and index-error source contracts, plus the
  compiler ownership mechanisms required to compile and execute those fixtures.
- Compiler scope includes loop sentinel reuse, repeat-count reuse, directly
  necessary same-mechanism corrections, and focused regression coverage.
- External source changes preserve every original case and algorithm behavior.
- The item includes the external PR/merge and the Sifr compiler/gitlink PR/merge.
- Earlier restrictions against these compiler changes are superseded.
  Unrelated Item 12 generated-quality work remains separate.
- Implementation starts from Sifr base
  `2dc4165fd9e7c34432a9b0d098188dc645aaca55` on the isolated Item 12B branch.
  Any prerequisite from retained Item 12 work requires explicit path-level provenance.
- External checkpoint `f6db5bd5d363b19a3040afd2a092f44ce32fd5bb`,
  Sifr handoff `1efb8720fa827f3bf19de17c7f010e3009f0e484`, and retained
  compiler candidate `8ad089a9458f35fcfa228e93fe44f4d69731828b` remain preserved.
- Qualification uses a newly built compiler from the isolated candidate.
  The retained frozen compiler is historical diagnostic evidence only.
- Review: one exact-SHA Opus review identifies both repository candidates.
  At most one remediation review is permitted. No whole-phase review is permitted.
- Gate: one merge-profile gate covers the exact final Sifr candidate.
  Skip create-PR. Do not repeat the merge gate.
- Close Item 12B and update its records, then stop. Do not start Item 12 or 12A.

#### Item 12B qualified implementation (before review and merge)

State: implementation qualification passed; Item 12B remains open until review and merge.

- Compiler source: `8c5bfefb32ccefbd8d925c14c554d3be1eb361d2`.
- Compiler SHA-256: `d47774bba160db3903b9143071352af3b3001d6ec16173731cad5811b4b7abad`.
- External corpus candidate: `da4a0e8680c6b50c5544d77bfb92e9e4cddf1ab1`.
- `native-qualified/matrix.json`: 90/90 repaired fixtures pass both check and native execution.
  This includes median, zigzag, browser history, all original assertions, and the added ownership cases.
- `borrow-final-codegen.log`: 1,435/1,435 codegen tests pass, including all 26 focused corpus-repair regressions.
- `borrow-final-focused.log`: the exact borrowed-parameter regression command selects and passes one test.
- `borrow-final-clippy.log`, `borrow-final-fmt.log`, `borrow-final-size.log`, and `borrow-final-hir.log`: pass.
- `borrow-final-demo-regeneration.log`: all 264 generated companions are fresh.
- Evidence files above are under `/tmp/sifr-item12b.akguMz/`.
- No Opus review or merge-profile gate has run yet. The create-PR gate remains skipped.

The canonical `leetcode-full` command passes 411/411 cases with zero failures.
Its immutable evidence is `leetcode-full-7f393-results.json` and `leetcode-full-final.log`.
That run used compiler source `7f3930ab4b05cd5ab50edb897be6a56329ab43f6`, digest
`68fbec4c7d99c843f0f75135e7c41b06d1552a0a769676f6df39982c6dc257f8`, and the same external corpus candidate.

This front-end evidence is reused by input identity, not relabeled as a later-SHA run.
The only subsequent compiler change is the nested arithmetic emitter's borrowed-operand adapter.
The CLI check path calls `check_project` or `check_single_file`, then returns front-end diagnostics without Rust emission.
The CLI source, driver, frontend/lowering/type-system/IR sources, stdlib, Cargo inputs, corpus, and algorithmic runner are unchanged.
Main's intervening changes rename demo variables and a regex test; they do not change those inputs.
Native qualification was rerun in full with the corrected compiler.
The retained Item 12 compiler candidate is not used as qualification evidence.

#### Item 12B continuation amendment: naming and SQL coverage dependency

On 2026-09-05 the user explicitly authorized both remaining repairs and **one
replacement merge-profile gate**. This supersedes the prior stop below, not its
failed evidence. The first gate on `6ce83824e0315e5f89383fc666344b99431e1e76`
remains failed. No create-PR gate is permitted.

- Replace all 428 corpus taxonomy occurrences with descriptive, collision-free
  semantic local names. Preserve assertions, typed receiving contracts, scope,
  call order, and evaluation count.
- Reconcile authoritative Cargo coverage classifications with the actual SQL
  packages, target kinds, and test targets (23 recorded diagnostics), coordinating
  through `ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
  Do not alter package semantics, weaken checkers, exclude targets, suppress
  diagnostics, or rebase accepted debt.
- Keep the reviewed candidates and record checkpoints in history.
- Use the one remaining exact-SHA Opus remediation review for both corrected
  repository candidates and this SQL metadata dependency. No third review.
- After satisfactory review run exactly one replacement merge gate on the final
  candidate, then merge both PRs and update records. Do not start Item 12 or 12A.

Additional/affected named validation, fixed before execution:

```bash
python3 /tmp/sifr-item12b.akguMz/verify_semantic_renames.py
python3 verification/areas/coverage_matrix/checks/coverage_matrix_readiness.py
python3 verification/areas/coverage_matrix/checks/coverage_matrix_readiness_self_test.py
python3 verification/areas/coverage_matrix/checks/profile_assignment_matrix.py
python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py
uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite readiness
python3 /tmp/sifr-item12b.akguMz/native_qualification.py /tmp/sifr-item12b.akguMz/native-naming-qualified
uv run --project verification --locked python -m sifr_verify areas run --area algorithmic_compatibility --suite leetcode-full
cargo fmt --check
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile merge
```

Coverage readiness includes new negative cases `wrong_component_target_kind`
(both missing rlib and stale lib findings) and `missing_sql_test_target`.
The canonical readiness suite executes all four listed coverage scripts, so those
checks need not be repeated as standalone commands. Use its uv environment.
MySQL and SQLite compiler crates now join the ordinary profile crate-test
membership, matching PostgreSQL; this executes their existing tests rather than
misclassifying compiler packages to evade required membership.

The native helper will run check AND native execution for all 90 repaired fixtures
with the identified compiler (exact helper invocation recorded before execution).
The rename proof compares token streams against corpus `da4a0e8680c6b50c5544d77bfb92e9e4cddf1ab1`,
allowing only an injective local identifier mapping; string literals, assertions,
types, and all other tokens must remain identical.
Compiler-source/binary, codegen regression, and crate Clippy evidence may be
reused only if their inputs remain unchanged, with original SHA attribution.
Fresh full-corpus evidence will use the corrected corpus pin.

#### Item 12B terminal checkpoint: remediation approved; replacement gate blocked

State on 2026-09-05: **not merged, not closed**. This supersedes earlier
authentication, corpus-naming, SQL-classification, and gate-authorization stops.

The user authorized the 428 corpus naming corrections, the 23 SQL coverage
classification corrections, and one replacement merge gate. Those repairs are
implemented, pushed, and approved; neither checker requirements nor safety were weakened.

- Reviewed/gated Sifr implementation candidate:
  `a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d`,
  [PR #3694](https://github.com/sifr-lang/sifr/pull/3694), base
  `b475ebdcd37081aa2860d9c348ace4100b546eff`.
- Corpus candidate and exact gitlink:
  `8bcbe7ab7939e5c8362c10f61a80e368022cc372`,
  [PR #48](https://github.com/sifr-lang/leetcode/pull/48), base
  `7fcb9fd1eaf3e0cf9bf51e8858276b7927a83baf`.
- Both initial and sole remediation Opus reviews returned **SATISFIED**, no blockers.
  [Remediation review](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5554470254)
  and [corpus review](https://github.com/sifr-lang/leetcode/pull/48#issuecomment-5554470481).
  No third review is permitted under current limits.
- Review artifact:
  `/tmp/sifr-item12b.akguMz/opus-a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d.2yceHO/response.md`.
- Fresh qualification on corrected inputs: **90/90** repaired fixtures pass check
  AND native execution; **411/411** canonical leetcode-full checks pass; coverage
  readiness **4/4** passes, including all **27** negative cases and whole taxonomy.
  The rename proof verifies **257** injective local renames across **78** files
  (428 changed declaration/reference lines), preserving every other token.
  Fmt, file-size, and HIR checks pass.
- Compiler source remains `8c5bfefb32ccefbd8d925c14c554d3be1eb361d2`;
  SHA256 `d47774bba160db3903b9143071352af3b3001d6ec16173731cad5811b4b7abad`
  was verified before/after qualification and after the replacement gate.
  Reused source8c5 evidence: **1,435** codegen tests, all **26** focused regressions,
  strict codegen Clippy, and **264** fresh companions. No retained Item12 compiler
  was used, and no unchanged-input test was repeated solely for resumption.

The authorized replacement `scripts/run_all_tests.sh --profile merge` ran once
on exact Sifr `a3198ab9f` and exited **1** after **1,839.46s**.
Reached HIR/file-size/demo freshness, Rust interop (10 variants), coverage
readiness (4), core language (5), and CPython differential (2) passed.
Python interop completed **30 variants: 25 pass, 5 blocking failures**:

| Variant | Recorded cause |
| --- | --- |
| dependency-versions | Removed DLPack demo path remains in dependency checker (later Item12G). |
| binding-authoring | Imported PythonError field initializer disagrees with its declaration, Rust E0560 (later Item12H). |
| callback-examples | Three async examples cannot access the support-owned cancellation task-local, Rust E0425 (later Item12I). |
| async-declaration-examples | PythonError propagation incompatible with Result[None, Error], SIFR-RESULT-0003 (later Item12J). |
| async-context-examples | Same error-channel failure as Item12J. |

The warm wall-time budget was also exceeded (advisory); no host-sensitive
performance pass is claimed. Later profile stages were not reached.
The first gate at `6ce83824e` remains failed; the replacement is also failed.
No create-PR gate, third gate, third review, or merge occurred. Both PRs remain draft.
These Python dependencies are recorded in
`ad-hoc-python-interop-qualification-dependencies.md`; no repair was started.

Evidence root: `/tmp/sifr-item12b.akguMz/`.
Current qualification: `native-naming-qualified/matrix.json`,
`leetcode-full-naming-results.json`, `leetcode-full-naming.log`,
`coverage-remediation-results.json`, `coverage-remediation.log`,
`semantic-renames-proof.json`, and `naming-final-*.log`.
Replacement gate: `merge-replacement-a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d.log`,
`replacement-a319-lane-report.json`, `replacement-a319-python-results.json`,
`replacement-a319-coverage-results.json`, and
`replacement-a319-{callback,async-declaration,async-context}-examples.json`.
[Published qualification](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5554449846)
preserves the complete/partial distinctions.

Exact next action: resolve/adjudicate the four separately recorded Python
dependencies and the exhausted review/gate limits before resuming closure.
Current continuation authority does not permit a third review or another gate.
Preserve these candidates and the record-only checkpoint commits; do not merge
without required qualification. Retained Item12 implementation
`8ad089a9458f35fcfa228e93fe44f4d69731828b` is unchanged. Do not start Item12/12A.

#### Deferred remediation-review suggestions

- Later Item12F (not started): rename the 16 pre-existing
  `updated_contract_value_*` locals in corpus `0202_happy_number.sifr` and
  `0212_word_search_ii.sifr`. Opus classified this as a non-blocking naming
  follow-up outside the enumerated 428-occurrence remediation, not a new
  mechanism defect. No checker weakening or third review was used.
- The SQL issue retains the non-blocking suggestion to confirm whether
  `sqlite-runtime-probe` should remain SQLite-only. Its current classification
  is accurate; no missing MySQL/PostgreSQL implementation is claimed.
- Earlier Item12 clone/receiver suggestions and unconfirmed Item12E integer
  field augmented-assignment qualification remain deferred.

#### Item 12B terminal checkpoint: review approved; merge gate blocked

State on 2026-09-05: **not merged and not closed**. This checkpoint supersedes
the historical authentication and scope-adjudication stops below.

- Reviewed Sifr candidate: `6ce83824e0315e5f89383fc666344b99431e1e76`,
  base `b475ebdcd37081aa2860d9c348ace4100b546eff`,
  [PR #3694](https://github.com/sifr-lang/sifr/pull/3694).
- Reviewed corpus candidate: `da4a0e8680c6b50c5544d77bfb92e9e4cddf1ab1`,
  base `7fcb9fd1eaf3e0cf9bf51e8858276b7927a83baf`,
  [PR #48](https://github.com/sifr-lang/leetcode/pull/48).
- The resumed initial Opus review returned **SATISFIED**, with no blocking findings.
  [Sifr review](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5554250479)
  and [corpus review](https://github.com/sifr-lang/leetcode/pull/48#issuecomment-5554250666).
  Earlier OAuth failures were not review verdicts. One successful initial review
  is consumed; no remediation review has run.
- Review artifact, outside Git:
  `/tmp/sifr-item12b.akguMz/opus-6ce83824e0315e5f89383fc666344b99431e1e76.UxSZXC/response.md`.
- Passing implementation evidence remains unchanged: 90/90 repaired fixtures pass
  check and native execution with compiler source `8c5bfefb32ccefbd8d925c14c554d3be1eb361d2`,
  digest `d47774bba160db3903b9143071352af3b3001d6ec16173731cad5811b4b7abad`;
  1,435 codegen tests (all 26 focused regressions), strict codegen Clippy, fmt,
  file-size/HIR checks, and 264-companion freshness pass.
  The 411/411 canonical check result remains explicitly attributed to
  `7f3930ab4b05cd5ab50edb897be6a56329ab43f6` and reused only by unchanged
  frontend/corpus input identity. No tests were repeated merely for this resumption.

The **one** merge-profile gate ran on exact Sifr candidate `6ce83824e0315e5f89383fc666344b99431e1e76`:

```bash
SIFR_SYSROOT=/tmp/sifr-item12b.akguMz/sifr scripts/run_all_tests.sh --profile merge
```

It exited 1 after 173.63 seconds. All preceding reached steps passed, including
264-companion freshness and Rust interop. Coverage readiness had two failing variants:

1. **Pre-existing external blocker:** 23 SQL package/target classification diagnostics
   (nine missing package classifications, missing targets, and stale PostgreSQL
   `lib` versus `rlib`). The candidate changes no SQL Cargo or coverage-registry
   inputs. Owner: `ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
2. **In-scope omission:** 428 verification-taxonomy diagnostics on newly introduced
   corpus `contract_result_*` locals. These are Item 12B fixture naming failures,
   not SQL failures and not waived by the passing semantic tests or Opus review.
   Example: `src/1396_design_underground_system.sifr:54`.
   Repair requires descriptive semantic names, preserving every assertion,
   typed receiving contract, and evaluation order; do not weaken the checker.

Immutable gate evidence under `/tmp/sifr-item12b.akguMz/`:
`merge-6ce83824e0315e5f89383fc666344b99431e1e76.log`,
`merge-6ce83824e-coverage-results.json`, and
`merge-6ce83824e-lane-report.json`.
No create-PR gate, duplicate merge gate, or merge occurred. Both PRs remain draft.
Later checkpoint commits update records only; they do not turn the failed gate
into a pass or transfer review approval to changed implementation.

Exact next action: resolve the separately owned SQL coverage blocker and adjudicate
the conflict between the exhausted single-gate budget and exact-final-SHA qualification
after the required corpus naming correction. Then correct the in-scope names,
refresh the corpus pin and affected named evidence, and use at most the one remaining
remediation review. A replacement gate needs an explicit exception to the no-second-gate
rule; no such exception is inferred from routine continuation authority.
The isolated worktree and both candidates remain preserved. Retained Item 12
compiler `8ad089a9458f35fcfa228e93fe44f4d69731828b` remains separate.
Do not start Item 12 or 12A.

#### Deferred review findings (not Item 12B implementation)

- Existing Item 12 owns the correctness-motivated clone residue and consistency of
  explicit imported mutable receiver borrowing. The review identified these as suggestions,
  not new Item 12B acceptance criteria.
- **Later Item 12E: integer field augmented-assignment qualification.** Not started.
  Confirm the frontend/lowering contract for integer field `/=` and `%=`, both
  outside and inside try closures. Opus noted the pre-existing gap at
  `crates/sifr_codegen/src/stmt_support_emitter/stmt_block_helpers.rs:495`:
  the simple path admits these operators while `SifrInt` has no corresponding
  assignment traits. This is an unconfirmed follow-up, not a reproduced Item 12B
  failure. Any repair and tests require its own bounded item; no code was added here.

#### Item 12B terminal checkpoint: external review authentication blocker

State: blocked before review and merge; Item 12B is not closed.
This checkpoint supersedes earlier scope-adjudication stops. Builtin registration and the recorded native-execution dependencies are authorized and implemented inside Item 12B.

- Qualified Sifr candidate: `4096a2e93b5fec3725c56c0a940dde995069d1f5`.
  Compiler implementation: `8c5bfefb32ccefbd8d925c14c554d3be1eb361d2`.
  Later checkpoint commits change records only.
- [Sifr PR #3694](https://github.com/sifr-lang/sifr/pull/3694), branch `codex/emitted-rust-excellence-item-12b`, remains draft.
- [Corpus PR #48](https://github.com/sifr-lang/leetcode/pull/48), branch `codex/item12b-source-contracts`, remains draft at `da4a0e8680c6b50c5544d77bfb92e9e4cddf1ab1`.
- Complete current qualification: 90/90 repaired fixtures pass check and native execution; all original cases remain.
  Codegen tests pass 1,435/1,435, including all 26 focused regressions.
  Strict codegen Clippy, fmt, file-size/HIR guardrails, and 264-companion freshness pass.
- Canonical `leetcode-full` passes 411/411 checks at `7f3930ab4b05cd5ab50edb897be6a56329ab43f6`.
  Its unchanged front-end/corpus inputs support explicit reuse; it is not relabeled as a later-SHA run.
  Native qualification was repeated with the corrected compiler.
- Corrected compiler SHA-256: `d47774bba160db3903b9143071352af3b3001d6ec16173731cad5811b4b7abad`.
- Evidence root: `/tmp/sifr-item12b.akguMz/`.
  Use `native-qualified/matrix.json`, `leetcode-full-7f393-results.json`, `leetcode-full-final.log`, and `borrow-final-*.log`.
- The retained Item 12 candidate `8ad089a9458f35fcfa228e93fe44f4d69731828b` remains separate and unchanged.
  No remaining Item 12 or Item 12A implementation was started.

The initial Opus request and both permitted request retries failed without a verdict.
Retained logs report: `Failed to authenticate: OAuth session expired and could not be refreshed`.
The official subscription sign-in was attempted. Interactive authentication could not complete with the available browser access.
The pending login process was cancelled; no account, billing mode, or security setting was changed.
No review approval, remediation review, create-PR gate, merge-profile gate, or merge has occurred.
The known SQL coverage issue remains separately owned; no new gate result is claimed.

The request logs remain outside Git in:
`opus-4096a2e93b5fec3725c56c0a940dde995069d1f5.zO2RVq/claude.log`,
`opus-4096a2e93b5fec3725c56c0a940dde995069d1f5.pYOvQr/claude.log`, and
`opus-4096a2e93b5fec3725c56c0a940dde995069d1f5.NtmhgZ/claude.log`, under the evidence root.
The first log is empty; the later two retain the OAuth failure.
The [Sifr blocker record](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5553738037)
and [corpus blocker record](https://github.com/sifr-lang/leetcode/pull/48#issuecomment-5553738178) preserve the handoff publicly.

Exact next action: the account owner completes `claude auth login --claudeai` for the already-configured subscription account.
Then resume the initial exact-SHA review against the current record-only PR head, with both repository candidates identified.
Reuse valid qualification evidence. At most one remediation review and one merge-profile gate remain.
Do not run a create-PR gate, start another item, or merge without the required review and validation.

#### Item 12B required tests

These commands run from the isolated Sifr worktree after the bounded implementation.
The focused regression names are fixed before test execution.

```bash
cargo build -p sifr
cargo test -p sifr_codegen item12b_loop_sentinel_reuse
cargo test -p sifr_codegen item12b_repeat_count_reuse
cargo test -p sifr_codegen
target/debug/sifr check verification/areas/algorithmic_compatibility/corpora/leetcode/src/0004_median_of_two_sorted_arrays.sifr
target/debug/sifr run verification/areas/algorithmic_compatibility/corpora/leetcode/src/0004_median_of_two_sorted_arrays.sifr
target/debug/sifr check verification/areas/algorithmic_compatibility/corpora/leetcode/src/0006_zigzag_conversion.sifr
target/debug/sifr run verification/areas/algorithmic_compatibility/corpora/leetcode/src/0006_zigzag_conversion.sifr
uv run --project verification --locked python -m sifr_verify areas run --area algorithmic_compatibility --suite leetcode-full
cargo fmt --check
cargo clippy -p sifr_codegen -- -D warnings
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh
```

Apply the same `check` and `run` commands to every repaired corpus fixture.
Record the compiler SHA and binary digest for focused and full-corpus evidence.
Include relevant additional changed crates in the Clippy command.
Loop regressions cover repeated iterations, branch paths, and later sentinel uses.
Repeat-count regressions cover later uses, nested scopes, and effectful counts.

#### Item 12B checkpoint: qualification blocked on an unrelated Clippy defect

State on 2026-09-05: implementation checkpoint preserved; Item 12B is not closed.

- Sifr candidate: `673593f3ee234d58f03694e018abb145a843f787`,
  branch `codex/emitted-rust-excellence-item-12b`.
- External candidate: `330544ecf4f787c1a5fbed847469797ead92d24c`,
  branch `codex/item12b-source-contracts` in `sifr-lang/leetcode`.
- Both candidates are pushed. Neither repository has an Item 12B PR or merge.
- No Opus review or merge-profile gate was consumed.
- The isolated worktree remains `/tmp/sifr-item12b.akguMz/sifr`.
  The retained Item 12 compiler candidate remains separate and unchanged.

The newly built compiler has SHA-256
`56ef1dac97c474d76341f77aebefa37e750002bdf82e6a6f6c5509a91d85847c`.
The binary digest remained unchanged throughout this qualification attempt.

Completed evidence under `/tmp/sifr-item12b.akguMz/`:

- `codegen-singleton-full-2.log`: all 1,412 codegen tests pass.
  This includes all four named Item 12B ownership regressions.
- `compiler-singleton-build.log`: the compiler build passes.
- `native-primary/0004_median_of_two_sorted_arrays.json`: check and native run pass.
- `native-primary/0006_zigzag_conversion.json`: check and native run pass.
  Each JSON record contains both candidate SHAs, input digests, commands, and logs.
  These runs retain the original cases and execute the added ownership assertions.
- `cargo fmt --check`: pass.
- File-size guardrail: pass for 3,755 files, with the 900-line limit unchanged.
- HIR maintainability guardrail: pass.

Incomplete evidence is not a qualification pass:

- `leetcode-full-candidate.log` contains 113 passing cases before interruption.
  It is not a complete 411-case result.
- The two helper checks pass, but their native runs were interrupted.
  The remaining repaired fixtures still require their native runs.
- The worker stopped all owned qualification processes after the scope blocker.
  No background qualification process remains.
- Earlier complete diagnostic matrices cover earlier inputs.
  They do not qualify these candidates.

`clippy.log` records the blocker from
`cargo clippy -p sifr_codegen -- -D warnings`.
The unchanged `project_stdlib_nominals.rs:45` uses `Option::expect` in
`ProjectNominalRegistry::register_builtin`.
This defect exists in base `2dc4165fd9e7c34432a9b0d098188dc645aaca55` and current
main `2af89e75e5f97ec75e1b72c000fb3a6ebbbbb7cc`.
It concerns builtin-error registration, not sentinel or repeat-count ownership.
The worker did not suppress the diagnostic or import unrelated retained Item 12 code.

Next action: authorize or merge the builtin-registration repair recorded as Item 12C.
Then resume Item 12B qualification on the identified candidate inputs.
Complete every required fixture run and the canonical full corpus before review.
The exact-SHA review allowance and the single merge-profile gate remain unused.

#### Incorporated Item 12C: Builtin-registration scope amendment

- State: incorporated into Item 12B by explicit user authority on 2026-09-05.
- The earlier exclusion of this mechanism is superseded.
- This repair has no separate item, review, or gate.
- The implementation preserves both repository checkpoints and unrelated Item 12 work.
- Registration must consume validated builtin identity without a fallback,
  diagnostic suppression, or replacement panic.
- Focused command, recorded before execution:
  `cargo test -p sifr_codegen item12b_builtin_registration`.
- The regressions cover canonical identities, module shadows, and rejected non-builtin names.
- After this repair, resume the remaining Item 12B validation and closure steps.
- Owner: compiler builtin-error registration.
- Defect: `crates/sifr_codegen/src/project_stdlib_nominals.rs:45` fails strict
  Clippy with `clippy::expect_used`.
- Dependency: this unchanged defect blocks Item 12B's required crate Clippy check.
- The repair must preserve builtin-error identity and the registration invariant.
  It must not add a fallback or diagnostic suppression.

#### Item 12B checkpoint: builtin repair passes; native qualification fails

This checkpoint supersedes the earlier builtin-registration blocker.
Item 12C is implemented inside Item 12B. Item 12B is not closed.

- Sifr implementation candidate: `3f422b01633d23c2bc8d8ce8ca59057c6e56adea`.
- External candidate: `330544ecf4f787c1a5fbed847469797ead92d24c`.
- Both candidates are pushed. Neither repository has a PR or merge.
- No Opus review, remediation review, or merge-profile gate was consumed.
- The retained Item 12 compiler work remains separate and unchanged.

Registration now accepts a validated `BuiltinError` token.
The registry no longer performs a partial name lookup or calls `expect`.
Two regressions cover canonical identities, module shadows, and rejected non-builtin names.

The newly built compiler has SHA-256
`dbe640b31bdd181b93f82d967dd9e7c82092146482c554fb14e96fe42f28a3c3`.
The compiler binary and both source trees stayed unchanged throughout qualification.
Evidence is under `/tmp/sifr-item12b.akguMz/`:

- `builtin-focused.log`: both named builtin-registration regressions pass.
- `builtin-codegen-full.log`: all 1,414 codegen tests pass, including all four ownership regressions.
- `builtin-build.log`: the compiler build passes.
- `builtin-clippy.log`: strict codegen crate Clippy passes.
- Formatting, file-size (3,756 files), and HIR guardrails pass.
- `leetcode-full-3f422.log`: the complete canonical 411-case check finishes.
  It reports 410 passes and one failure, fixture 2002.
  This is a complete failing result, not a partial pass or native qualification.
  The canonical result is
  `target/verification/areas/algorithmic-compatibility-results.json`.
  Per-case results and taxonomy remain under
  `target/verification/areas/algorithmic_compatibility/`.
- `native-3f422/matrix.json`: complete coverage of all 90 changed source files.
  Checks pass for 89 files. Native builds and runs pass for 43 files.
  Native builds fail for 46 files. The failed check prevents the remaining native run.
  Median and zigzag both pass their checks and native assertions.
- `native-3f422/diagnostic_inventory.json`: every failing file, command, log, and diagnostic group.
  The following counts overlap where one file has several diagnostics.

| Diagnostic group | Files | Representative fixture |
|---|---:|---|
| Handler binding captured outside its scope | 13 | 0017 |
| Reused value moved | 12 | 0072 |
| Missing structured `TryExcept` lowering | 10 | 0044 |
| Narrowed value compared with `None` | 8 | 0102 |
| Missing `UnionFind.union` method emission | 4 | helpers/dsu |
| Recursive optional field mutability | 3 | 0025 |
| Nested assignment receives `Option<SifrInt>` instead of `SifrInt` | 1 | 0048 |
| Borrowed `str` clone emission | 1 | 1397 |
| Empty collection assertion type inference | 1 | 1203 |
| Unreceived checked shift result in source | 1 | 2002 |

The checked-shift receiving omission and approved ownership corrections remain Item 12B work.
They are not external authority blockers.
Other failures require control-flow, type-representation, or declaration-demand changes.
Those mechanisms are not builtin registration or sentinel/repeat reuse.

#### Item 12B continuation authority and regression commands

The user authorized all necessary next actions after the complete failure inventory.
The recorded execution dependencies now form part of the bounded Item 12B repair.
The previous scope-adjudication stop is superseded.
This includes checked-read control flow, exception capture and lowering, ownership,
method retention, assertion typing, and the checked-shift source omission.
It does not include unrelated Item 12 quality work or Item 12A.
The original review and single-gate limits remain unchanged.

Additional focused commands, recorded before test execution:

```bash
cargo test -p sifr_codegen item12b_
cargo test -p sifr_codegen item12b_exception_capture
cargo test -p sifr_codegen item12b_checked_read_control_flow
cargo test -p sifr_codegen item12b_repeated_value_ownership
cargo test -p sifr_codegen item12b_recursive_optional_mutability
cargo test -p sifr_codegen item12b_structured_exception
cargo test -p sifr_codegen item12b_method_retention
cargo test -p sifr_codegen item12b_empty_collection_assertion
```

The complete native matrix is the repair checklist, not a new discovery run.
Each regression must cover the relevant lexical, control-flow, or ownership negative case.
The new compiler requires new affected-input evidence. Earlier passes retain their recorded provenance.

The source batch also completes two missing checked-value contracts.
Fixture 2002 receives each checked shift into an explicit integer binding.
Fixture 0048 tests each optional matrix read before its corresponding write.
The matrix changes preserve read/write order and raise `IndexError` on absent values.
They do not substitute a default or remove an original case.

#### Item 12B integrated-base provenance

The complete `7f3930ab4b05cd5ab50edb897be6a56329ab43f6` native matrix passes 89/90 repaired fixtures.
All three previous residuals pass. Fixture 1472 exposes a nested arithmetic double borrow of an existing borrowed parameter.
The correction uses the existing binding-aware exact-integer operand adapter, not a new cloning rule.
Its exact focused command is recorded before execution:

```bash
cargo test -p sifr_codegen corpus_repair_nested_arithmetic_preserves_borrowed_parameter
```

Main PR #3693 (`b475ebdcd37081aa2860d9c348ace4100b546eff`) was integrated at `8d5afcb0c12a6115c54aadccee0e3fa87f478db0`.
Its naming-only changes leave compiler and algorithmic qualification inputs unchanged.
Only its changed demo inputs required regeneration; the canonical merge freshness check remains required.

The `d13954fd69a0dfb0e203a03cd5124d921d103539` residual native matrix passes 16/19 cases.
The remaining repairs cover nested mapping-default ownership and proven reads at optional call boundaries.
Fixture 1260 also requires explicit narrowing of optional position and cell reads before mutation.
These complete the existing ownership/read and external source-contract scope; no new item is started.
All existing corpus assertions remain unchanged.
Focused commands for this batch, recorded before execution:

```bash
cargo test -p sifr_codegen corpus_repair_repeated_value_ownership_nested_arithmetic_and_defaults
cargo test -p sifr_codegen corpus_repair_proven_read_at_optional_call_boundary
cargo test -p sifr_codegen corpus_repair_explicit_optional_nested_mutation_contract
cargo test -p sifr_codegen corpus_repair_
```

The final compiler must still pass all 90 repaired fixtures and the full canonical corpus.
The 16/19 residual pass is not whole-item qualification.

Main advanced to `c83dd7cde8daf54cdc4abd952903e9aa093c4183` through PR #3692.
Merge `b3d836354` integrates that reviewed base, including its dependency-feature normalization.
The test-module conflict keeps both new regression modules and main's renamed modules.
No retained Item 12 implementation was imported.

The merged naming policy forbids numbered planning labels in source names.
The new tests therefore use mechanism-oriented module names and the `corpus_repair_` filter.
This is a mechanical identity change; every case and assertion remains present.
The exact replacement focused command is recorded before execution:

```bash
cargo test -p sifr_codegen corpus_repair_
```

Main records unresolved SQL coverage classifications in
`ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
They remain externally owned and are not absorbed into this dependency repair.
The single-gate limit is unchanged.

#### Item 12B continuation implementation evidence

The complete `7b50a83a91ce65dc17d91d73e54c14dcd1b67901` qualification has 411/411 canonical checks.
Its repaired-fixture matrix has 90/90 checks and 71/90 native passes.
The 19 native failures remain qualification failures, not a partial pass claim.
Raw evidence: `leetcode-full-continuation.log` and `native-continuation/diagnostic_inventory.json` under the owned temporary root.
The follow-up batch repairs those same mechanism paths: expression-local checked reads,
child-scope last-use accounting, owned argument adaptation, imported mutable receivers,
and empty assertions inside exception carriers. No unrelated Item 12 work is included.

Additional exact regression commands, recorded before execution:

```bash
cargo test -p sifr_codegen item12b_checked_read_control_flow_short_circuit_assignment
cargo test -p sifr_codegen item12b_structured_exception_root_error_and_dictionary_reads
cargo test -p sifr_codegen item12b_structured_exception_nested_while_checked_comparison
cargo test -p sifr_codegen item12b_repeated_value_ownership_condition_and_branch
cargo test -p sifr_codegen item12b_repeated_value_ownership_nested_arithmetic_and_defaults
cargo test -p sifr_codegen item12b_empty_collection_assertion_in_exception_carrier
```

Compiler implementation `580e3374c3aac2aa669ad06354fba02c618e0942` completes the recorded dependency batch.
Commit `18ab9bd969e70876a99875d8c719ad8b8d4daeb3` updates the existing union-rendering test expectation.
External candidate `0ef88e8b4f4906e410a3b2e9216248c11149b247` completes the two remaining source contracts.
No retained Item 12 compiler changes were imported.

Evidence under `/tmp/sifr-item12b.akguMz/`:

- `continuation-focused-4.log`: all 17 Item 12B regressions pass.
- `continuation-codegen-full-2.log`: all 1,425 codegen tests pass.
- `continuation-clippy.log`: strict codegen Clippy passes.
- `continuation-build.log`: the new compiler build passes.
- Formatting and file-size/HIR guardrails pass; the size check covers 3,760 files.

The compiler binary SHA-256 is
`04e449044644533db98fad9289d89355078f12b3e3bbd9bdb77d7f42398dfbfa`.
These are focused and crate-level results, not full-corpus qualification.
The earlier complete failing matrices remain historical evidence.
No Opus review or merge-profile gate has run.

#### Incorporated Item 12D: Native corpus emission dependencies

State: incorporated into Item 12B under the continuation authority.
Owner: compiler emission, tracked in this issue and the algorithmic issue.
This item does not reopen Item 12C or request authority for its completed repair.

The confirmed scope blocker is checked-read control-flow and optional representation.
In fixture 0102, the source tests a left read only inside its left-length branch.
Generated Rust inserts a left-read `let Some(...) else { break; }` before that branch.
A second read narrows the value to `Vec<SifrInt>`, but its `None` comparison remains.
The first transformation can terminate a valid right-only iteration.
The second transformation fails Rust compilation with `E0277`.

Evidence: `native-3f422/0102.emitted.rs:116` and
`native-3f422/0102_binary_tree_level_order_traversal.run.log`.
The relevant producer is `crates/sifr_codegen/src/checked_place.rs`.
Its `checked_place_read_witness` path removes the optional representation.
This producer is unchanged from the isolated base.
A repair must preserve branch-local read demand, absence paths, and effect order.
Removing source guards or adding a fallback would not correct that mechanism.

The full diagnostic inventory also records structured exception lowering,
handler capture scope, missing method emission, and assertion type inference.
Their final producer-level decomposition remains unimplemented.
The ownership groups stay in Item 12B rather than moving into this later item.

Next action: adjudicate the newly recorded emission mechanisms as dependency scope.
Then finish the approved source and ownership corrections on the preserved branch.
Complete qualification on the final inputs before either repository merge.
The exact-SHA Opus allowance and single merge-profile gate remain unused.

All owned qualification commands completed. No background qualification run remains.
No compiler, fixture, test, baseline, or safety policy changed after this evidence.
Later commits update records only and do not claim a new implementation SHA pass.

#### Item 12B implementation provenance

The compiler changes start from the merged base, not the retained Item 12 candidate.
The existing ownership materializer now serves integer local bindings and repeat counts.
Its rename does not change the other callers.
The repeat producer preserves operand order for both operand positions.
Singleton repetition uses the existing exact-integer range without a host-sized cast.
This correction also satisfies the two existing singleton-repeat codegen tests.

The median fixture adds native loop, branch, sentinel-reuse, and large-integer assertions.
The zigzag fixture adds native repeat-count, operand-order, and single-evaluation assertions.
Both fixtures retain every original case.
External repairs propagate checked errors through explicit receiving contexts.
Reassignments retain their original binding identity and statement order.

## Status

Closeout in progress for the non-blocking follow-up created from the Rust-interop
`certification_0` validation on 2026-07-26. The failures predate that
milestone, are outside Rust interop and stable-release-governance scope, and do
not block `certification_0`, Phase 40, or stable-channel Rust-interop
certification.
The durable issue was established in
[PR #3029](https://github.com/sifr-lang/sifr/pull/3029).

No failure was suppressed or reclassified by the Rust-interop work. Nine
focused remediation waves are merged, all 411 pinned fixtures now pass the
canonical suite and a complete native build/run audit on current integrated
compiler tree `d53aa21d51`. That tree includes the later class-field
index/slice-footprint lowering change from PR #3094, so both corpus gates were
recertified after the main integration rather than relying on byte identity.
Closeout restores the full corpus plus taxonomy self-test to release
qualification. The temporary release-profile divergence is removed rather than
renewed.

## Preserved Evidence

The exact-state nightly and release profiles both passed their complete
Rust-interop steps and then independently reported the same 20 blocking
failures in the 412-variant algorithmic-compatibility lane:

- `target/validation_lane_reports/nightly.latest.json` and its adjacent log:
  Rust interop passed in 4,161 ms; the algorithmic lane reported 20 failures.
- `target/validation_lane_reports/release.latest.json` and its adjacent log:
  Rust interop passed in 3,880 ms; the algorithmic lane reported the same 20
  failures.
- `target/verification/areas/algorithmic_compatibility/leetcode-full-taxonomy.json`
  records 20 failures among 411 corpus fixtures: 15
  `other_type_surface_and_api_mismatch`, 4
  `any_unknown_typing_and_container_specialization_gap`, and 1
  `signature_invalid_fixture_surface`.

The taxonomy artifact was generated on 2026-06-16 and contains 411 fixture
records. The later profile lanes report 412 area variants because their count
also includes the area-level `full-corpus-taxonomy-smoke` policy/runner
variant. The 20 failing fixture slugs are set-identical across all three
evidence sources.

Those figures describe the corpus's check-only lane. A complete native-build
audit on 2026-07-29 also found a disjoint set of 23 fixtures that pass
`sifr check` but fail `sifr build`, partitioned across three root causes:
21 owned optional-class destructure failures, one empty-dictionary
specialization failure, and one recursive optional-class constructor-coercion
failure. That latent set is recorded separately below and is included in the
remediation acceptance gate; it is not added to or substituted for the
preserved 20 blocking check failures.

The failing fixture slugs are:

- `0002_add_two_numbers`
- `0036_valid_sudoku`
- `0056_merge_intervals`
- `0086_partition_list`
- `0094_binary_tree_inorder_traversal`
- `0144_binary_tree_preorder_traversal`
- `0145_binary_tree_postorder_traversal`
- `0252_meeting_rooms`
- `0350_intersection_of_two_arrays_ii`
- `0377_combination_sum_iv`
- `0435_non_overlapping_intervals`
- `0442_find_all_duplicates_in_an_array`
- `0452_minimum_number_of_arrows_to_burst_balloons`
- `0621_task_scheduler`
- `0767_reorganize_string`
- `1203_sort_items_by_groups_respecting_dependencies`
- `1383_maximum_performance_of_a_team`
- `1481_least_number_of_unique_integers_after_k_removals`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`
- `2402_meeting_rooms_iii`

Phase 40 reproduced the set exactly on source
`c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b`. The canonical release attempt
passed coverage, core guardrails, diagnostics, CPython differential, all 25
Python-interop variants, Rust interop, frontend syntax guardrails, all 48
developer-tooling variants, documentation, and `performance_budget_checks` in
`full` mode. It then reported the same 20 failures among 412 algorithmic area
variants. This independent reproduction is the evidence for separating the
nightly full-corpus remediation signal from stable-channel release
qualification.

Representative diagnostics include unknown `Any` hash/equality capability,
mutable-borrow representation changes, unavailable generated total `Ord`, and
structural-equality mismatches between concrete and `Any` containers. The
taxonomy artifact and profile logs remain the detailed ephemeral evidence;
this issue is the durable repository record.

## Current-Main Reproduction and Diagnosis

The complete corpus was reproduced locally on 2026-07-29 from
`649334330ce4f9c682b5aa8453ddad6ada737d40` with:

```bash
uv run --project verification --locked python -m sifr_verify areas run \
  --area algorithmic_compatibility --suite leetcode-full
```

The run checked all 411 pinned fixtures and reproduced exactly the preserved
20 blocking failures. Direct checks of all 20 fixtures established six
root-cause groups keyed to each fixture's first blocking diagnostic; the table
below adds two further root causes found only by the native-build audit. Some
fixtures can expose follow-on diagnostics after their first blocker is fixed,
so this is not permission to stop after one diagnostic disappears.

| Root cause | Ownership boundary | Fixtures | Remediation |
| --- | --- | ---: | --- |
| Recursive `list[T]` total-order capability is omitted even though generated `Vec<T>` has lexicographic `Ord` when `T: Ord` | lowering type-capability query | 6 | admit `list[T]` recursively for `list.sort()` while continuing to reject non-total-order element types; keep sets and dictionaries excluded because their language semantics are not total orders, regardless of incidental generated representation |
| Empty list literals in equality comparisons retain `list[Any]`, including nested empty literals | comparison lowering and literal specialization | 6 | specialize literal HIR recursively from the concrete opposite operand before structural-equality checks; do not weaken the type-system capability gate |
| An empty plain-dictionary declaration retains an `Any` value despite a later concrete subscript write | order-independent declaration-site inference | 1 latent build failure | infer compatible plain-dictionary writes within the enclosing function before lowering the declaration; preserve ordinary missing-key access and augassign semantics |
| `defaultdict(int)` subscript augassign preserves an `Any` key in HIR | container specialization at the augassign target | 4 | specialize the alias key from the concrete subscript while preserving defaultdict codegen semantics |
| `defaultdict(set)` is read before its first textual write, so forward-only refinement cannot establish its key/value types | order-independent declaration-site inference | 1 | infer compatible defaultdict access shapes within the enclosing function before lowering the declaration; reject conflicting shapes deterministically |
| Consuming recursive linked-list traversal is declared as a mutable borrow, and generated owned optional-class destructures omit required Rust mutability | fixture ownership plus optional-class codegen | 2 check failures plus 21 latent build failures | use `own mut` in the two check-failing fixtures and fix the generated owned optional-class destructure in codegen so it emits a mutable binding; the shared `helpers/list_node` module and its two local copies need no source change, and all 23 affected fixtures must build and run, not merely check |
| Recursive optional-class locals passed to constructors are emitted as `Option<T>` instead of the required `Option<Box<T>>` | recursive-class constructor argument codegen | 1 latent build failure | apply the same recursive-field storage coercion used by direct recursive constructor arguments to typed optional-class locals, with focused positive and non-recursive negative coverage |
| Unreachable, unannotated nested `dfs` remains after the live iterative solution returns | fixture surface | 1 | remove the dead Sifr-only porting residue; continue type-checking reachable and unreachable declarations normally |

The fixture membership of those groups is:

- Recursive list total order: `0056_merge_intervals`,
  `0252_meeting_rooms`, `0435_non_overlapping_intervals`,
  `0452_minimum_number_of_arrows_to_burst_balloons`,
  `1383_maximum_performance_of_a_team`, and
  `2402_meeting_rooms_iii`.
- Contextual empty-list equality: `0094_binary_tree_inorder_traversal`,
  `0144_binary_tree_preorder_traversal`,
  `0145_binary_tree_postorder_traversal`,
  `0442_find_all_duplicates_in_an_array`,
  `1203_sort_items_by_groups_respecting_dependencies`, and
  `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`.
- Empty plain-dictionary declaration refinement: `0001_two_sum`.
- `defaultdict(int)` augassign specialization:
  `0350_intersection_of_two_arrays_ii`, `0621_task_scheduler`,
  `0767_reorganize_string`, and
  `1481_least_number_of_unique_integers_after_k_removals`.
- Order-independent `defaultdict(set)` inference: `0036_valid_sudoku`.
- Ownership and owned optional-class codegen: `0002_add_two_numbers` and
  `0086_partition_list` are the check-failing members; the 21 latent
  build-failing members are enumerated in the native-build audit below.
- Recursive optional-class constructor argument coercion:
  `0894_all_possible_full_binary_trees`.
- Dead invalid fixture surface: `0377_combination_sum_iv`.

The complete native-build audit checked every pinned fixture and then built
each of the 391 check-passing fixtures:

```bash
target/debug/sifr check --isolated <fixture>
target/debug/sifr build --quiet --isolated -o <unique-output-dir> <fixture>
```

It produced exactly 411 terminal records: 20 `CHECK_FAIL`, 23 `BUILD_FAIL`,
and 368 `BUILD_PASS`. The 20 check failures are set-identical to the preserved
corpus lane. The 23 distinct latent build failures are:

- Twenty linked-list fixtures that fail with generated Rust `E0596` because
  `node.next.take()` is emitted from an owned destructure whose binding is not
  mutable. Eighteen import the shared `helpers/list_node.nodeNext` helper (the
  other two importers are the preserved check failures `0002` and `0086`):
  `0019_remove_nth_node_from_end_of_list`,
  `0021_merge_two_sorted_lists`, `0023_merge_k_sorted_lists`,
  `0024_swap_nodes_in_pairs`, `0025_reverse_nodes_in_k_group`,
  `0061_rotate_list`, `0083_remove_duplicates_from_sorted_list`,
  `0092_reverse_linked_list_ii`, `0143_reorder_list`,
  `0147_insertion_sort_list`, `0148_sort_list`,
  `0203_remove_linked_list_elements`, `0206_reverse_linked_list`,
  `0234_palindrome_linked_list`, `0876_middle_of_the_linked_list`,
  `1669_merge_in_between_linked_lists`,
  `1721_swapping_nodes_in_a_linked_list`, and
  `2130_maximum_twin_sum_of_a_linked_list`.
- The remaining two linked-list fixtures contain local copies of the same
  owned `nodeNext` helper:
  `0141_linked_list_cycle` and
  `0160_intersection_of_two_linked_lists`.
- `0617_merge_two_binary_trees`, which fails with the same generated Rust
  `E0596` mechanism for owned `TreeNode | None` parameters whose recursive
  fields are consumed after optional destructuring.
- `0001_two_sum`, whose empty `prevMap = {}` remains
  `dict[int, Any]` despite the later `prevMap[n] = i`; generated Rust then
  fails with `E0277`/`E0308` around `Box<dyn Any>`.
- `0894_all_possible_full_binary_trees`, whose typed
  `TreeNode | None` locals are passed to a recursive-class constructor as
  `Option<TreeNode>` instead of `Option<Box<TreeNode>>`, producing generated
  Rust `E0308`.

The original issue record was reviewed to satisfaction in passes 1-3. agent
agent then independently reproduced the current 20 failures and conditionally
approved this diagnosis with the mechanism corrections and build/run
requirement recorded above. The diagnosis reviews are preserved in
[`review pass 4`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-4.md),
[`review pass 5`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-5.md),
[`review pass 6`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-6.md),
and
[`review pass 7`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-7.md).
After the pass-7 findings were addressed, the complete diagnosis was approved
with zero actionable findings in
[`review pass 11`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-11.md).
Passes 8 and 9 were interrupted before producing reviewable output, and pass
10 failed at the reviewer API certificate boundary before producing a report;
their zero-byte outputs were discarded and are not evidence. A final rebased
audit then found the incomplete native-build inventory in
[`review pass 12`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-12.md);
the complete 411-fixture native-build audit, expanded waves, and explicit
passes-8-to-10 disposition above are the responses to both requested changes.
[`Review pass 13`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-13.md)
independently verified the audit counts and technical partition, then requested
the group-count, membership-map, progress-state, and evidence-continuity
corrections now recorded here.
[`Review pass 14`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-agent-review-pass-14.md)
rechecked every pass-13 correction and the full sweep ledger, then approved the
corrected diagnosis with zero actionable findings.

## Focused Remediation Waves

Each wave is implemented, locally validated, reviewed to satisfaction, merged,
and recorded here before the next wave starts:

1. Recursive list total-order support and positive/negative compiler coverage.
2. Contextual empty-list equality specialization, including nested literals
   plus mismatched-literal and variable-operand negative coverage.
3. Order-independent empty plain-dictionary declaration refinement, including
   the `0001_two_sum` native build/run and deterministic conflicting-write
   coverage.
4. `defaultdict(int)` subscript-augassign specialization with runtime counting
   coverage.
5. Order-independent `defaultdict` declaration inference with the existing
   deterministic `TYPE_CONTAINER_ELEMENT_CONFLICT` diagnostic and the
   resulting `0036_valid_sudoku` pass, with no fixture-side change.
6. Removal of the dead invalid `0377` Sifr fixture block while deliberately
   leaving the Python reference sibling unchanged as the upstream parity
   source.
7. Owned linked-list traversal fixture corrections plus the generated optional
   recursive-class extraction fix and build/run coverage for all 23 affected
   fixtures: the 22 linked-list fixtures plus
   `0617_merge_two_binary_trees`.
8. Recursive optional-class constructor argument coercion with focused
   compiler coverage and the `0894_all_possible_full_binary_trees` native
   build/run.
9. Nested-function captured-container specialization regression correction:
   preserve deferred patches produced for captured enclosing bindings while
   keeping same-named nested locals isolated, with direct, multilevel, shadow,
   generated-Rust, and native runtime coverage plus the unmodified
   `0022_generate_parentheses` native build/run.
10. Full-corpus closeout: capability-named demo, a complete 411-fixture native
   build/run audit, complete nightly lane,
   restoration of `leetcode-full` to release qualification, complete release
   lane, final local merge gate, and full-implementation review.

The corpus runner is intentionally check-oriented. Remediation waves must also
build and run focused e2e coverage for corrected runtime surfaces so a green
corpus cannot hide generated-Rust failures. Each wave must check the complete
affected fixtures after the change, not merely confirm that the targeted first
diagnostic disappeared. Wave 1 and wave 2 tests belong in new focused modules
to preserve the 900-line source limit, and wave 2 must specialize literal HIR
in lowering without relaxing or editing the structural-equality gate in
`sifr_type_system/src/check.rs`. No wave may use a plain `dict`
annotation to erase a `defaultdict` alias: that changes missing-key augassign
codegen and can silently produce incorrect counts.

## Separately Tracked Findings

Diagnosis exposed pre-existing behavior outside the preserved 20-fixture set:
plain `dict` missing-key augassign can silently no-op instead of preserving the
approved error behavior; reverse sorting has an equal-element stability gap;
`sorted(..., key=lambda ...)` can lower and then fail generated Rust name
resolution even for flat integer lists; `list[None]` literals can lower and
then emit a generated Rust option/unit mismatch even without sorting;
`min`/`max` list ordering has separate generated-Rust failure paths and remains
outside this issue's sort-specific Wave 1; `.values()` uses an over-broad
unknown-key capability guard; and unreachable nested function bodies still
reach type inference after `SIFR-FLOW-0901` and can produce hard type
diagnostics. These findings are not used as fallbacks or exclusions for this
issue and do not broaden its focused remediation waves. Wave 1 widens the
element types that can reach the
pre-existing reverse-sort stability gap but does not change reverse sorting.
An enclosing concrete dictionary binding can also pollute nested-function
inference for a same-named empty dictionary declaration; current main already
emits invalid generated Rust for that shadowing shape, and Wave 3 deliberately
does not broaden nested-function inference semantics to absorb this adjacent
pre-existing problem.
Concrete inferred `defaultdict(set)` captures can also make a previously
masked closure-mutability codegen defect reachable: a nested function that
reads the captured default dictionary can check successfully and then fail
generated Rust with `E0596` because the closure binding is not emitted as
mutable. The same generated failure reproduces on the pre-Wave-5 base when
the nested return is annotated explicitly, so it is not a Wave 5 inference
defect; it remains a separately tracked closure-codegen concern and is not
used to weaken this issue's native closeout gate.
Blocks containing a nested function also retain the pre-existing general
empty-collection hint path; assignable-but-unequal dictionary writes on that
path can still reach invalid generated Rust exactly as on current main. Wave
3's exact-write gate neither expands nor claims to repair that pre-existing
nested-function behavior.
Wave 5 also makes additional default-dictionary bucket-mutator source orders
reachable without changing several pre-existing general collection-expression
defects: `sorted(<slice>)` can emit an unresolved generated name, set
comprehensions or `set(<slice>)` can emit invalid iterator calls, and
out-of-range `list.insert` can panic at runtime. Each class reproduces on the
pre-Wave-5 base through ordinary concrete collection paths. They remain
separately tracked compiler/runtime concerns and are not used to weaken this
issue's native closeout gate.
The missing-key wrong-result behavior was corrected and closed in the separate
[`ad-hoc-dict-missing-key-augassign-semantics.md`](../archive/ad-hoc-dict-missing-key-augassign-semantics.md)
issue via [PR #3108](https://github.com/sifr-lang/sifr/pull/3108), rather than
worked around here.

## Scope

- Diagnose each failure against the current language contract.
- Group fixes by compiler concern and ownership boundary.
- Implement root-cause compiler or fixture corrections in focused PRs.
- Preserve the full-corpus gate as blocking in nightly; do not add baselines,
  exclusions, fallback behavior, or area-specific exceptions.
- Restore the full corpus and taxonomy self-test as blocking release
  qualification when the corpus is green.
- Name any associated demo after the capability it demonstrates. Demo names
  must not contain a phase number or phase name.
- This user-directed naming rule supersedes the project-workflow skill's
  generic `<milestone>_demo` example for every demo owned by this issue.
- Do not modify Rust-interop matrices, stable claims, crate pins, or profile
  registration unless a separately reviewed cross-area requirement is proven.

## Implementation Progress

| Item | Status | Evidence |
| --- | --- | --- |
| Failure diagnosis and root-cause grouping | approved; [PR #3064](https://github.com/sifr-lang/sifr/pull/3064) | exact current-main 411/20 check reproduction plus a complete 411-fixture native-build audit: 20 check failures, 23 distinct latent build failures, and 368 native-build passes; pass 14 approved the corrected diagnosis with zero actionable findings |
| Wave 1: recursive list total order | approved; [PR #3068](https://github.com/sifr-lang/sifr/pull/3068) | recursive `list[T]` generated-Rust `Ord` capability; focused nested positive/negative lowering tests including a list-returning `sorted` key; capability e2e; all six affected corpus fixtures build and run; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-1.md) requested the added `sorted` coverage, [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-2.md) approved the completed wave, and [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-3.md) approved the first rebased implementation; [pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-4.md) returned an incomplete mid-sweep status and is not approval evidence, [pass 5](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-5.md) approved the completed current-base implementation, and [pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-agent-review-pass-6.md) approved the exact GitHub head/base prospective merge with zero actionable findings |
| Wave 2: contextual empty-list equality | merged; [PR #3074](https://github.com/sifr-lang/sifr/pull/3074) | recursive literal-only HIR specialization from the concrete opposite operand without changing the structural-equality gate; canonical recursive `Unknown`/`Any` query; exact-type preservation for unchanged concrete elements; explicit generated-Rust typing for concrete empty lists; focused empty-leading and trailing-empty nested positives in both operand positions, mismatched-literal and named-variable negatives; native capability e2e; all six affected corpus fixtures check, build, and run; `create-pr` profile passed with 131/131 selected native e2e fixtures and every blocking budget green; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-agent-review-pass-1.md) requested genuine nested left-side coverage, recursion into concrete outer literals, and the canonical type query; [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-agent-review-pass-2.md) independently verified every correction and approved the complete wave with zero actionable findings; [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-agent-review-pass-3.md) reviewed the exact pushed implementation head, independently ran the full 676/676 e2e suite and workspace checks, and approved with zero blocking findings; [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-agent-review-pass-4.md) verified the documentation-only head delta, re-ran full e2e and workspace checks, and approved the complete PR with no blocking issues |
| Wave 3: empty plain-dictionary declaration refinement | merged; [PR #3077](https://github.com/sifr-lang/sifr/pull/3077) | function-level binding inference refines an empty plain-dictionary declaration from compatible later subscript writes only when the current lexical block has one unshadowed binding for that name and every collected write has the exact adopted key/value shape; any subscript augassign disqualifies the new adoption path so the existing missing-key behavior is not expanded; the existing empty-list/set/deque hint boundary is unchanged; the declaration and literal HIR receive the same concrete type, including eligible blocks containing nested functions; deferred container patches resolve only the nearest declaration; codegen keeps declaration-local types when same-named lexical bindings differ; incompatible and assignable-but-unequal writes on the new path preserve `SIFR-TYPE-0008`; focused lowering, codegen, and native capability coverage pins read-before-write inference, deterministic hard and widening conflicts, unhashable-key diagnostic cardinality, missing-key augassign-before-write rejection, no-evidence fallback, same-name sibling-scope isolation, nested-function-block HIR consistency, and loop-local/function-level isolation; all 898 lowering tests pass with one additional ignored test and all 934 codegen tests pass; Clippy, rustfmt, maintainability, file-size, and diff-hygiene checks pass; the capability e2e and `0001_two_sum` both check, build, and run; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-1.md) found whole-block name-keyed adoption leaking across sibling bindings, and the declaration-safety gate, nearest-declaration patching, declaration-local codegen registry, and native regressions are the response; pass 2 timed out without reviewable output and its zero-byte artifact was discarded; [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-3.md) independently ran the full 677/677 native e2e suite and approved before its non-blocking cleanups were applied; [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-4.md) independently reran 677/677 e2e and found unified assignable-but-unequal writes could admit generated-Rust errors, and the exact-write-shape gate plus numeric, nominal-class, and unhashable-key boundaries are the response; pass 5 exceeded the 40-minute reviewer bound with zero output and its empty artifact was discarded; [agent pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-6.md) independently verified the widening fix, then found subscript augassign was invisible to the gate and could expand the pre-existing wrong-result surface, and explicit augassign disqualification plus looped missing-key regression coverage are the response; pass 7 also exceeded the 40-minute bound with zero output and its empty artifact was discarded; [agent pass 8](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-8.md) independently verified the augassign correction in direct, loop, nested control-flow, and try forms and approved the complete wave with zero actionable findings; the authoritative create-PR profile then passed every blocking lane and all 131 selected native e2e fixtures after isolated reruns confirmed two host-sensitive first-attempt timeouts in the readonly Python doctor and LSP shutdown smoke; implementation commit `1ad7389dd` is the first published head |
| Wave 4: `defaultdict(int)` augassign key specialization | merged; [PR #3079](https://github.com/sifr-lang/sifr/pull/3079) | the first concrete subscript-augassign key refines only the unkeyed `defaultdict(int)` alias, widens literal keys to mutable base types, patches both declaration and constructor-call HIR within the declaring lexical function, and preserves the alias-backed `entry(...).or_insert(0)` codegen path; initialized aliases keep their declared key type and conflicting later keys remain rejected; deferred patches inspect only direct declarations in their lexical block, verify that the declaration expression matches the requested specialization before changing its type, remain isolated across nested lexical-function lowering, and leave same-named nested `defaultdict` and scalar bindings independent in either source order; focused lowering is 10/10, focused codegen is 5/5, full lowering is 908 passed with one ignored, full codegen is 939/939, affected Clippy/rustfmt/maintainability/file-size checks pass, the expanded capability e2e passes, all four affected corpus fixtures check/build/run, and the complete native e2e suite passes 678/678; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-4-agent-review-pass-1.md) found the original name-keyed recursive patch could retarget a nested shadow and requested the added shadowing, alias-boundary, unhashable-key, and plain-dictionary negative coverage; [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-4-agent-review-pass-2.md) independently verified those corrections and found a reversed-order nested shadow could clear the enclosing function's pending patch, and lexical-function patch isolation plus reversed-order lowering/codegen/native coverage are the response; pass 3 exceeded the 40-minute reviewer bound with zero output and its empty artifacts were discarded; [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-4-agent-review-pass-4.md) independently reran the full 678/678 native e2e suite, both complete affected library suites, every affected corpus fixture, and deeper lexical-function probes, then approved the exact current-base implementation with zero actionable findings; merged at `f1c34cf9aa` |
| Wave 5: order-independent `defaultdict` declaration inference | merged; [PR #3081](https://github.com/sifr-lang/sifr/pull/3081) | function-level inference models unseeded `defaultdict(int/list/set)` aliases with inference-only unknown key/value slots, collects later subscript and `set.add`/`list.append` shapes before declaration lowering, and adopts a recursively complete hint only for one unshadowed direct declaration in the current lexical block; declaration and constructor-call HIR receive the same concrete alias type; a shared neutral declaration-safety census serves both plain-dictionary and defaultdict adoption; a conservative exact-expression allowlist admits only literals, stable composites, lowerable slices, fixed-result builtins, and complete non-generic function returns, while optional-returning indexing/method calls and their name-propagated or loop-scoped descendants monotonically disqualify adoption for the current binding; lexical-function locals reset inherited provenance without clearing same-binding taint on reassignment, and inexact provenance now propagates outward across the same nested `nonlocal` boundary as inferred types; structurally incomplete aliases remain incomplete for nested-function return checking and recover without raw-rustc or follow-on diagnostic leakage; alias-aware `defaultdict` bucket mutators preserve the `entry(...).or_insert(...)` receiver for every supported in-place list/set mutation, including generally lowered append/add arguments; iterable-driven extend/update families bind the key, perform the observable default insertion, evaluate and materialize every argument left-to-right, and then re-borrow the destination bucket; they use shared ownership conversion, support variadic intersection/difference updates, and fail closed at codegen rather than falling through to a cloned receiver; simple list/set literals clone borrowed non-Copy names before storage, and set membership avoids double-borrowing borrowed parameters; binding-hint adoption now has its own responsibility module and leaves `statement_dispatch.rs` at 829 lines rather than the 900-line cap; conflicting concrete keys and set elements retain deterministic `SIFR-TYPE-0008` (`TYPE_CONTAINER_ELEMENT_CONFLICT`) diagnostics, and the same validation now consistently rejects incompatible `set.add` arguments for ordinary concrete sets; focused lowering is 12/12, focused codegen is 12/12, full lowering is 920 passed with one ignored, full codegen is 951/951, workspace Clippy, rustfmt, maintainability, file-size, and diff-hygiene checks pass, the expanded capability e2e checks/builds/runs and pins intermediate values across list `append`/`extend`/`sort`/`reverse`/`insert`/`remove`/`clear` and set `add`/`update`/`intersection_update`/`symmetric_difference_update`/`difference_update`/`discard`/`remove`/`clear`, plus generally lowered iterables, cross-bucket same-map sources, borrowed-string list/set storage, variadic set updates, key-before-argument evaluation order, and default insertion before self-observing arguments on both list and set paths; `0036_valid_sudoku` checks/builds/runs without a fixture change, and the post-pass-7 through post-pass-10 complete native e2e sweeps each pass 679/679 after five earlier 679/679 sweeps; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-1.md) found indexed-expression inference/lowering disagreement, nested unresolved tuple holes, incomplete nested-return leakage, diagnostic-ledger, test-cardinality, declaration-type, and responsibility-placement gaps, and the recursive completeness checks, structured recovery type, expanded tests, corrected ledger, and neutral census module are the response; pass 2 exceeded the 40-minute bound with zero output and its empty artifacts were discarded; [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-3.md) verified those corrections, then found optional-returning `dict.get`/`list.pop` calls escaped the subscript-only provenance denylist and exact reassignment cleared taint even though the lowered declaration remained optional; the exact-expression allowlist, monotonic per-binding provenance, five new direct/name/loop/rebind regressions, expanded native e2e, and separately tracked newly reachable closure-mutability note are the response; [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-4.md) reached the reviewer bound with only a truncated three-line verdict and is retained as non-standalone evidence; [agent pass 5](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-5.md) reconstructed its exact probes and code paths, identifying silent missing-key slice-append behavior, borrowed-string set ownership/lookup failures, and nested provenance asymmetry, and the emitter, provenance, focused codegen/lowering, and native runtime corrections above are the response; [agent pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-6.md) independently verified every pass-5 correction, then found order-independent adoption newly exposed the same cloned-temporary wrong-result class for `list.extend`/`set.update`, requested precise ledger wording, and noted zero headroom in `statement_dispatch.rs`; the complete alias-aware mutator receiver, focused/native value assertions, ledger correction, and binding-hint module extraction above are the response; [agent pass 7](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-7.md) verified the pass-6 receiver fix, then found strict-lowering fallback still allowed cloned-bucket mutation for slices/concatenations/conditionals/comprehensions, same-map iterable sources leaked `E0499`, the native assertions were insensitive to most mutators, and the ledger omitted newly reachable pre-existing defects; general iterable fallback, source materialization before destination borrowing, intermediate runtime assertions, borrowed-string ownership coverage, and the separately tracked findings above are the response; [agent pass 8](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-8.md) independently verified every pass-7 correction, then found variadic intersection/difference updates could still fall through to cloned receivers, iterable materialization reversed key/argument evaluation order, and the interception remained structurally fail-open; variadic materialized retains, key-first temporaries, fail-closed error propagation, and focused/native regressions are the response; [agent pass 9](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-9.md) independently verified every pass-8 correction and the earlier remediation matrix, then found self-observing iterable arguments still ran before the `defaultdict.__missing__` insertion; the separate pre-insertion statement, post-argument re-borrow, emitted-order assertion, and native `len(defaultdict)` observation are the response; [agent pass 10](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-10.md) independently verified the complete behavior and prior remediation matrix, then mutation-tested the pre-insertion on each emitter path and found only the list path was regression-sensitive; mirrored set-path emitted-order assertions plus a native self-observing variadic set update are the response; [agent pass 11](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-11.md) independently mutation-tested both list and set pre-insertion sites, revalidated the complete Wave 5 behavior and all prior remediations, and approved the exact implementation head with zero actionable findings; [agent pass 12](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-5-agent-review-pass-12.md) verified that the published PR head added only the accurate pass-11 artifact and ledger update, matched the approved implementation tree byte-for-byte outside documentation, and approved the complete PR with zero actionable findings; the authoritative exact-head create-PR profile then exited 0 with every blocking step green, including Python interop 19/19 in 506.546 seconds, Rust interop 10/10 in 9.106 seconds, developer tooling 18/18, performance smoke 7/7, generated-code quality 5/5, crate tests, runtime platform, and selected native e2e 131/131 with signature `7c39b8c1dd4fec7c`; two preceding unchanged-head attempts had isolated host-sensitive step-budget misses that each passed exact focused reproduction without changing a budget or waiver; merged at `441f667f03` |
| Wave 6: remove dead invalid `0377` block | merged; corpus [PR #42](https://github.com/sifr-lang/leetcode/pull/42); parent [PR #3085](https://github.com/sifr-lang/sifr/pull/3085) | the sole unreachable nested `dfs` fallback was removed from the Sifr fixture while the live bottom-up dynamic-programming implementation, native assertion, and Python parity sibling remain unchanged; the Python blob is byte-identical across the corpus range; the pre-change fixture reproduces exactly two `SIFR-TYPE-0004` diagnostics from the dead block, while the corrected fixture checks, builds, and runs; the Python oracle executes successfully; corpus [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-6-corpus-agent-review-pass-1.md) independently verified the minimal deletion, exercised a differential input grid, audited downstream benchmark consumers and parent baseline surfaces, and approved with zero actionable findings; the parent pointer advances from `a20d9d502` to merged corpus head `d50fa7350`, which also contains concurrently merged, behavior-neutral `0146_lru_cache` and `0189_rotate_array` snapshot adaptations owned by the receiver-place workstream, and all three changed Sifr fixtures plus their Python siblings check/build/run successfully with the submodule ownership guard passing; parent [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-6-parent-agent-review-pass-1.md) independently reproduced the fixture and diagnostic evidence, audited gitlink provenance plus every parent profile/baseline consumer, verified the corrected unreachable-code wording and Wave 5 merge evidence, and approved the complete parent implementation with zero actionable findings; parent [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-6-parent-agent-review-pass-2.md) verified the published PR head added only the accurate pass-1 artifact and ledger traceability update, preserved the approved non-documentation tree exactly, and approved with zero actionable findings; the authoritative exact-head create-PR profile exited 0 with every blocking budget green, including Python interop in 582.169 seconds, Rust interop in 8.496 seconds, generated-code quality 5/5, crate and runtime-platform suites, and selected native e2e 131/131 with signature `7c39b8c1dd4fec7c`; merged at `6f888ed327` |
| Wave 7: owned recursive-option extraction | merged; corpus [PR #43](https://github.com/sifr-lang/leetcode/pull/43); parent [PR #3086](https://github.com/sifr-lang/sifr/pull/3086) | `0002_add_two_numbers` and `0086_partition_list` now declare their consumed, locally reassigned list parameters as `own mut`, while the shared `helpers/list_node` implementation, both local `nodeNext` copies, and all Python siblings remain unchanged; simple and structured option narrowing share one ownership-aware decision backed by the emitter's SCC-derived recursive-field registry, emitting mutable Rust bindings for owned recursive class values so later child extraction can use `.take()` while shared and mutable borrows remain non-moving; full binding context now reaches nested simple blocks, nested Copy parameters retain value semantics, and ordinary forced-let mutability uses one canonical helper across lowering paths and the same SCC registry; focused codegen covers self- and mutually recursive owned values and locals, let-else, if-let, conjunction, disjunction, truthiness, nested-function, and nested-block shapes, plus non-recursive, recursive-container-only, shared-borrowed, mutable-borrowed, and nested-Copy negatives; all 959 codegen tests, affected Clippy, rustfmt, maintainability, file-size, submodule-ownership, and diff-hygiene checks pass; both post-implementation 23-fixture sweeps check/build/run every affected corpus fixture; the post-correction complete native e2e suite passes 679/679 with signature `b0887ad6eb81c080`; corpus [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-corpus-agent-review-pass-1.md) reproduced the base ownership diagnostics, bracketed `own mut` against the invalid `own`-only and `mut`-only alternatives, ran all 23 fixtures plus differential parity probes, and approved the exact corpus commit with zero actionable findings; parent [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-parent-agent-review-pass-1.md) found the original simple-path self-recursion heuristic missed mutually recursive SCCs, the simple and structured paths had divergent predicates, nested blocks lost type/borrow/recursion context, nested Copy parameters were over-classified as borrowed, the ledger overstated use-site precision, and positive coverage pinned only one narrowing shape; the shared SCC-backed helper, complete binding threading, Copy filter, corrected wording, canonical general mutability helper, and expanded focused matrix are the response; pass 2 returned only an interim progress sentence and pass 3 exceeded the 40-minute reviewer bound before returning `Execution error`, so neither is approval evidence and both unusable artifacts were discarded; parent [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-parent-agent-review-pass-4.md) independently verified all six pass-1 corrections and approved them, then identified one low-severity pre-existing SCC mismatch in forced local mutability; the canonical predicate now consumes the authoritative recursive-field registry too, with a mutually recursive local regression test; parent [agent pass 5](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-parent-agent-review-pass-5.md) verified that correction and found registry membership alone also classified non-optional recursive containers; the predicate now requires both authoritative SCC membership and an optional field shape, with a recursive-container negative; parent [agent pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-parent-agent-review-pass-6.md) independently verified the final predicate is identical in shape to the recursive `.take()` gate, re-ran both positive and negative reproducers plus focused checks, and approved the implementation with zero actionable findings; parent [agent pass 7](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-7-parent-agent-review-pass-7.md) verified the published head added only the accurate pass-6 artifact and ledger sentence, preserved the approved non-documentation tree byte-for-byte, and approved with zero actionable findings; the exact-head create-PR profile passed every blocking lane and all 131 selected native e2e fixtures before the parent merged at `4c867d1cda` |
| Wave 8: recursive optional constructor coercion | merged; parent [PR #3089](https://github.com/sifr-lang/sifr/pull/3089) | one shared recursive-constructor option adapter now maps named optional recursive-class arguments from `Option<T>` to `Option<Box<T>>` after ownership handling and clones shared-borrowed options before consuming `Option::map`; a successful adaptation directly returns the completed Rust expression, so strict registry lowering and constructor fallback each suppress their terminal clone without a redundant status payload, while the successful registry constructor post-pass treats registry option adaptation as authoritative instead of recognizing previously emitted syntax; duplicate registry boxing helpers and the temporary syntactic idempotence recognizer are removed; focused codegen covers local, shared-borrowed, owned, field-projection, non-option, positional, and keyword forwarding through direct and nested recursive constructors, rejects double boxing and map-before-clone output, and preserves ordinary non-recursive option arguments; the capability e2e checks, builds, and runs both direct and nested borrowed-option forwarding; all 964 codegen tests, affected Clippy, rustfmt, maintainability, file-size, and diff-hygiene checks pass; the complete native e2e suite passes 686/686 with signature `96d2681cf0c5ac5c`; a 30-fixture recursive-node corpus sweep builds 30/30; `0894_all_possible_full_binary_trees` also checks, builds as a native release binary, and runs with both recursive constructor sites emitting exactly one box layer; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-8-parent-agent-review-pass-1.md) found that the initial registry-local coercion could double-box a direct borrowed option, move a nested borrowed option before cloning, duplicated responsibility across four sites, omitted borrowed-parameter coverage, and overstated its guarantee; the shared post-ownership adapter, single-application routing, direct and nested borrowed regressions, native capability fixture, and corrected ledger are the response; [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-8-parent-agent-review-pass-2.md) independently verified every pass-1 correction, ran the complete 686/686 e2e suite and 30/30 recursive-node corpus build sweep, and found no correctness regression; it requested explicit non-option, owned-option, field-projection, and keyword coverage, the complete-suite evidence above, a current PR description, and removal of a redundant adapter result flag; the expanded focused matrix, direct-expression adapter contract, corrected ledger, and rewritten PR body are the response; [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-8-parent-agent-review-pass-3.md) independently verified every correction, reproduced 964/964 codegen and 686/686 native e2e with the exact signature, expanded the corpus differential to all 56 recursive-node fixtures, exercised 24 direct/nested and ownership boundary shapes, and approved with zero actionable findings; [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-8-parent-agent-review-pass-4.md) verified the published head added only the accurate pass-3 artifact and ledger sentence, preserved the approved non-documentation tree byte-for-byte, re-ran the full 686/686 native e2e suite and all guardrails, and approved the exact prospective PR with zero actionable findings; the authoritative unchanged-head create-PR profile passed on its third attempt after isolated host-sensitive first-attempt budget failures were reproduced green, and the PR merged at `9e80f3a23c` |
| Wave 9: nested captured-container patch propagation | merged; parent [PR #3091](https://github.com/sifr-lang/sifr/pull/3091), merge `eee55b9f94` | the first closeout native inventory found only `0022_generate_parentheses`; [corpus agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-9-corpus-agent-review-pass-1.md) rejected a fixture annotation and bisected the regression to Wave 4's whole-map nested-function restore; corpus PR #44 closed unmerged and the corpus remains byte-identical; the compiler fix propagates deferred specialization only for actual nested captures while isolating shadows, with direct, multilevel, generated-Rust, and native coverage; full lowering passed 944 with one ignored, codegen 967/967, native e2e 687/687 with signature `d61c30dde1d7fc1c`, and the exact-head create-PR facade passed 138/138 selected e2e with signature `4ede7c71d86f381c`; [parent agent passes 2-4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-9-parent-agent-review-pass-4.md) closed the only ledger/PR-metadata findings and approved the published implementation with zero actionable findings |
| Wave 10: full-corpus closeout | milestone implementation review satisfied; final validation in progress | capability demo checks/builds/runs; the pre-integration compiler at `eee55b9f94` passed the first complete isolated native audit with 411 records and no check, build, or run failures under `target/algorithmic-native-closeout.4ZmiZw`; the first integrated-compiler-tree rerun under `target/algorithmic-native-closeout.ljNFEM` likewise produced 411 records with 411 pass and zero check, build, or run failures, and its canonical `leetcode-full` lane passed 411/411 on compiler tree `53fd964ea0`. After main advanced with PR #3094's class-field index/slice-footprint lowering change, current integrated head `d53aa21d51` was recertified instead of relying on the earlier byte-identity claim: the canonical lane plus taxonomy smoke passed 412/412 with zero blocking or non-blocking failures, and a new complete native audit in the clean detached validation worktree under `target/algorithmic-native-closeout-current-main.q9evmg` produced 411 records with 411 pass and zero check, build, or run failures. Release now selects `leetcode-full` plus taxonomy smoke with external/long-running resources, matching nightly; coverage readiness passes 5/5 and emitted release-plan selection is exact; [agent pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-10-agent-review-pass-1.md) independently verified the implementation and found only that the pre-integration evidence provenance and Phase 40's closeout wording were overstated; [agent pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-10-agent-review-pass-2.md) verified the first correction round and found the same qualifier was still needed in the issue and roadmap summaries plus one policy adjective; those live-document corrections are the response. The immutable pass-1 artifact's two stale line references should read 363/376 and 125, and its transient zero-byte housekeeping observation was resolved when that output completed; pass 2 records this erratum without rewriting the reviewer's original output. [agent pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-10-agent-review-pass-3.md) verified every correction, accepted the integrity-preserving erratum, and approved the Wave 10 milestone implementation with zero actionable findings. Nightly pass 2 then passed unchanged-budget full performance, algorithmic 412/412, distribution 68/68, sysroot 2/2, and the generated corpus plus determinism before Rust 1.94 exposed one blocking `clippy::box_default` diagnostic in recursive empty-list storage. A shared recursive-boxing helper now emits `Box::default()` for empty vectors across field initialization and both constructor-argument routes while preserving non-empty and already-boxed values; focused 2/2, full codegen 969/969, crate Clippy, and all 10 required generated demos pass. [agent pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-10-agent-review-pass-4.md) found that the first initializer-only correction omitted constructor-argument routes; [agent pass 5](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-10-agent-review-pass-5.md) independently verified the shared correction and approved it with zero actionable findings. Complete fixed-head nightly/release/merge gates and final phase review remain pending |

Wave 3's exact prospective merge `ec5aab945` includes current `main`
`ea119724e`; the authoritative create-PR profile passed again on that exact
state with all 131 selected native e2e fixtures. Reviewer pass 10 returned
only a transient HTTP 529 overload response and is not review evidence.
[agent pass 11](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-11.md)
then independently compared the published head and base, ran the complete
677-fixture e2e suite, compared all 411 corpus checks, ran all 58 corpus
fixtures containing an empty dictionary through the native path, and approved
the complete Wave 3 implementation with zero actionable findings.
[agent pass 12](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-12.md)
verified the documentation-only published-head delta and requested only that
the pass-11 report's self-label be corrected from pass 12 to pass 11; the
report heading now matches its filename and ledger link.
The branch then incorporated current `main` `ca7731aa8` without file overlap;
the authoritative create-PR profile passed on exact prospective merge
`7dbe8bd36` with every blocking budget green and 131/131 selected e2e
fixtures. [agent pass 14](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-14.md)
approved that exact head/base pair with zero actionable findings. Wave 3
subsequently merged in [PR #3077](https://github.com/sifr-lang/sifr/pull/3077)
at merge commit `789b359737`; this merge record supersedes the earlier
pre-merge table status.

## Acceptance Criteria

- [x] Every listed fixture passes the canonical full-corpus algorithmic suite.
- [x] The current canonical corpus count passes without a baseline,
  suppression, exclusion, or non-blocking reclassification.
- [ ] The nightly profile passes the complete algorithmic-compatibility lane
  locally.
- [ ] After the corpus is green, `leetcode-full` is restored to the release
  profile and the release lane passes locally.
- [x] Focused compiler tests cover each corrected root-cause category.
- [x] Focused e2e tests build and run every corrected generated-Rust surface;
      all 23 corpus fixtures exercising the affected owned optional-class
      extraction pattern build and run after the ownership/codegen fix, and
      `0894_all_possible_full_binary_trees` builds and runs after recursive
      constructor coercion is corrected.
- [x] All 411 pinned corpus fixtures pass a complete native build/run audit at
      closeout; the check-only corpus lane is not sufficient evidence for this
      criterion.
- [x] Every associated demo uses a capability-based name containing no phase
  number or phase name.
- [ ] The authoritative create-PR and merge profiles, Clippy, rustfmt,
  maintainability, file-size, and diff-hygiene gates pass locally.
- [ ] Review rounds are satisfied and all remediation PRs are merged.

## External validation observation

- On 2026-08-30, the SQL Milestone 2 merge gate reached this verification area.
- The worktree contained gitlink `ad116aa8dcae51b7db1bdf0052470456d671d31b`.
- The worktree did not initialize the gitlink. Therefore, the required `src/`
  directory was absent.
- The algorithmic runner stopped before corpus execution. This result is an
  environment failure and does not change corpus qualification evidence.
- The SQL workstream did not initialize the external corpus or run a second gate.

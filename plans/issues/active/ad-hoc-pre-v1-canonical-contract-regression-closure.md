# Ad Hoc Phase: Pre-v1 Canonical Contract Regression Closure

Status: active on 2026-08-22. Items 0 through 4 are complete. Item 5 is next.

## Objective

Restore all canonical features and validation contracts that regressed during
or near the pre-v1 compatibility-removal phase.

This phase does not restore a removed alias, fallback, dual schema, or legacy
layout. It completes missed consumer migrations and corrects compiler defects.

The phase preserves these product features:

- bytes-native hashing,
- file hashing with typed errors and reliable file closure,
- canonical package projects under `src/`,
- exact generated dependency selection,
- structured Rust type generation,
- nested optional and union operations,
- structured diagnostics for invalid programs,
- protocol conformance diagnostics,
- checked method-slot behavior.

## Source and Provenance

The source phase is
[`ad-hoc-pre-v1-compatibility-removal.md`](./ad-hoc-pre-v1-compatibility-removal.md).
That phase closed on 2026-08-19.

Codex task `01a01407-6090-7a53-8ebc-c1c8f8cf0eec` executed that phase. Its
final implementation merge was `2256180742e82d52686e6eda622d05b4afdbf716`.

The initial corrective plan used repository head
`bf72949d94178be77fe6d427c5534254b88e5240`. This revision uses main-branch
head `b52fbc7e46257a7fd14d8f551fc9f2c28fb4ac47`.

The investigation found incomplete migrations in these pre-v1 commits:

| Commit | Introduced regression |
| --- | --- |
| `3d34ad6289096f7588a75dda8689eb19b4f1ad62` | Left string-hash calls in the all-features native test. |
| `5ada538590a82de5248f98ea5c34f5cc8fec4902` | Exposed sequential `try` binding loss in `hashlib.file_digest`. |
| `7b13bad7be050448aebfa90d7074b86f1252fd69` | Left stale fixture paths, hashes, checksums, and project-root assumptions. |
| `2b764b8bfd5d62e9f8da86ebd2e12abd330e1267` | Added an incorrect `NumBigint` dependency assertion. |
| `0f1b0e8d755dd7445585f736c7d929260e78e387` | Exposed nominal-identity and nested optional conversion defects. |

The investigation also found independent failures. This phase owns an
independent failure only when no active issue already owns it.

### Ownership transfers

This planning change makes these transfers effective when it merges:

| Earlier record | Earlier owner state | Single corrective owner |
| --- | --- | --- |
| Sequential `try` binding scope | Phase 21 is complete and cannot accept new work. | Item 5 |
| Structured-concurrency project splitting | The pre-v1 record names static-adapter and Phase 21 owners without one active owner. | Item 6 |
| Nested option-represented union conversion | Static-class M12 previously held a deferred hardening row. | Item 7 |
| Protocol-bound CFG and diagnostic failure | The pre-v1 record and static-class records assign it to each other. | Items 8 and 9 |
| Method-slot fixture and runtime evidence | The static-class phase is archived, but both verification defects remain. | Item 10 |

The archived static-class phase no longer owns active corrective work. Item 10
owns its unresolved method-slot fixture and runtime evidence.

The archived record remains historical and does not name this phase. This
transfer table is the current ownership authority.

The completed Phase 21 record remains unchanged. This phase supersedes its
deferred assignment for sequential `try` scope.

### Rebaseline evidence

The merge gate stopped at the Rust-interop matrix because the gate is
fail-fast. The same merge commands then ran for every remaining area and Cargo
suite.

| Surface | Rebaseline result |
| --- | --- |
| Guardrails | 14 passed |
| Validation areas | 14 passed and 5 failed |
| Area variants | 421 passed, 15 failed, and 3 skipped |
| Cargo suite commands | 23 passed and 3 failed |
| E2E pass fixtures | 673 passed and 13 reported failed |

The 13 E2E reports contain two source defects. One defect is in
`cpython_hashlib_api_subset`. The other defect is in
`nested_optional_safe_operations`. Eleven fixtures share the failed optional-
union batch and are collateral reports.

The installed-sysroot attached-adapter boundary now passes both variants.
Merge `4bf07c41c2` corrected that boundary. This phase removes the old Item 10
scope and does not repeat that work.

## Governing Rules

1. Preserve one canonical public contract for each feature.
2. Do not restore a removed string, alias, schema, or layout path.
3. Do not add a fallback reader, fallback resolver, or fallback renderer.
4. Correct a compiler defect at its semantic owner.
5. Do not reshape valid source only to avoid a compiler defect.
6. Update every repository consumer when a canonical path changes.
7. Keep external protocol behavior and installed-toolchain behavior intact.
8. Return structured compiler errors for unsupported or invalid input.
9. Do not use data-dependent `.unwrap()` or `.expect()` in generated code.
10. Complete one item in one session, worktree, branch, and pull request.
11. Use the phase-closure loop for every implementation item.
12. Run the file-size guardrail before each item finishes.

## Canonical Contract Lock

| Area | Canonical contract | Forbidden repair |
| --- | --- | --- |
| Hashing | Native hashing accepts bytes. Text uses explicit encoding. | Restore string-hash helpers. |
| Packages | A project has one relative source root. The default is `src/`. | Accept flat or multi-root layouts. |
| Project tools | A tool receives an explicit project root or canonical project session. | Search for an old sibling-manifest shape. |
| Dependencies | Reachable module metadata selects only required Cargo features. | Add broad features to satisfy a test. |
| Error handling | Successful `try` bindings obey Sifr lexical-scope rules. | Rewrite valid source around generated Rust scopes. |
| Rust types | `RustType` is the only code-generation type representation. | Restore string rendering or raw named fallbacks. |
| Nominals | One logical nominal has one project-wide Rust identity. | Create module-local copies of a hoisted nominal. |
| Option and union | Each representation conversion occurs once. | Add emitted-code text patches. |
| Diagnostics | Invalid input produces stable structured diagnostics. | Catch a panic and treat it as a normal diagnostic. |
| Method slots | One checked slot contract defines dispatch and slot count. | Preserve two runtime behaviors. |

## Failure and Ownership Inventory

| Failure | Root cause | Delivery owner |
| --- | --- | --- |
| `sifr_stdlib_all_features` | The test still calls removed string-hash helpers. | Item 1 |
| Rust shared-bridge evidence | The manifest omits the new `negative/src/` segment. | [Linked delivery A](#required-linked-delivery-a-rust-interop-evidence-path) |
| Performance LSP cold start | The benchmark assumes that `sifr.toml` is beside `src/main.sifr`. | Item 2 |
| Performance trend variants | The stored benchmark-manifest hash is stale. | Item 3 |
| Fuzz smoke | Two seed paths omit the new `src/` segment. | Item 3 |
| Ecosystem checksum variants | Two canonical project manifests have stale checksums. | Item 3 |
| Driver dependency test | The test requires `NumBigint` for `sifr.statistics.mean`. | Item 4 |
| `cpython_hashlib_api_subset` | Generated Rust scopes hide successful sequential `try` bindings. | Item 5 |
| Structured concurrency builds | Local and hoisted `ScopeFailure` types have different identities. | Item 6 |
| File-handle builds | Local and hoisted file-handle types have different identities. | Item 6 |
| `nested_optional_safe_operations` | Lowering converts an optional union more than once. | Item 7 |
| CLI CFG panics | Invalid control flow can create a branch with one successor. | Item 8 |
| Protocol diagnostic mismatch | The fixture references an unknown bound before conformance checking. | Item 9 |
| Method-slot placeholder | The matrix fixture contains an empty placeholder declaration. | Item 10 |
| Method-slot runtime output | The driver expectation predates the receiver-value serializer and keeps the stale count `2`. | Item 10 |
| Read-only Python duration | The latest contended run completed in 314,714 ms. An isolated run completed in approximately 68 seconds. | Qualification rule B |

## Scope

### In scope

- Migrate tests from removed string helpers to bytes-native helpers.
- Define canonical project-root inputs for performance tools.
- Refresh paths, hashes, and checksums after the source-layout migration.
- Correct generated dependency assertions.
- Preserve successful bindings across sequential `try` statements.
- Give each generated nominal one project-wide identity.
- Make optional and union conversion representation-aware and single-pass.
- Replace invalid-program CFG panics with structured diagnostics.
- Separate name-resolution and protocol-conformance fixture intent.
- Complete the method-slot fixture and align its runtime evidence.
- Add regression guards for the migrated evidence and representation defects.
- Record completion evidence from the linked owner.
- Qualify the read-only Python doctor without a timeout increase.

### Out of scope

- A new language or standard-library feature.
- A restored pre-v1 alias or transition helper.
- A second package manifest schema.
- A second project-discovery path.
- A string-based Rust type fallback.
- A compatibility diagnostic for an invalid old form.
- A timeout increase without a measured deterministic regression.
- New static-class or method-slot product behavior.
- Changes to vendored dependencies.

## Execution Order

```text
Item 0  baseline and ownership lock
  -> Item 1  bytes-native hash test
  -> Item 2  canonical performance project root
  -> Item 3  migrated evidence freshness
  -> Item 4  exact dependency feature test
  -> Item 5  sequential try binding scope
  -> Item 6  project-wide nominal identity
  -> Item 7  nested optional conversion
  -> Item 8  invalid-program CFG diagnostics
  -> Item 9  protocol fixture intent
  -> Item 10 method-slot verification closure
  -> linked delivery A final validation and merge
  -> Item 11 linked delivery and timeout qualification
  -> Item 12 final regression guard and closure
```

Do not start Item 11 until linked delivery A is complete. Do not use its
unmerged worktree during this phase.

## Item 0: Lock the Baseline and Ownership

Purpose: Confirm the failure set before implementation changes begin.

Scope:

- Record the exact base SHA.
- Reuse the rebaseline evidence when its inputs are unchanged.
- Run a focused test again only after a relevant base change.
- Record the exact failing variants and diagnostics.
- Confirm that each linked owner records its assigned work.
- Remove a row only when the current base already passes it.
- Do not run the full merge gate in this item.

Acceptance criteria:

- The item record contains one result for every inventory row.
- The record maps all five failed areas and all three failed Cargo commands.
- The record maps 13 E2E reports to two source defects.
- Each row has one delivery owner or one approved non-failure classification.
- No active issue has duplicate implementation ownership.
- The phase records all unexpected failures in their owning issues.
- Items 1 through 10 do not start until the ownership transfers are effective.

### Item 0 baseline evidence

Base SHA: `4815e74c2cf59989e8eba5afb4f7ca2e31b7d097`.

Only the phase-plan merge changed main after the rebaseline. Therefore, Item 0
reuses the recorded validation evidence.

| Inventory row | Exact baseline result | Owner |
| --- | --- | --- |
| `sifr_stdlib_all_features` | The all-features build cannot find removed `hash::sha256`, `hash::md5`, and `hash::sha1` functions. | Item 1 |
| Rust shared-bridge evidence | The matrix requests `negative/package_generated_type_import_rejected.sifr`. The checked-in source is under `negative/src/`. | Linked delivery A |
| Performance LSP cold start | Three variants use the obsolete sibling-manifest project shape. | Item 2 |
| Performance trend variants | The stored `manifest_sha256` value does not match the benchmark manifest. | Item 3 |
| Fuzz smoke | Two manifest fields omit `src/` from the canonical missing-import seed path. | Item 3 |
| Ecosystem checksum variants | Five variants report stale checksums for `curated_cli_math` and `curated_data_flow`. | Item 3 |
| Driver dependency test | The test incorrectly requires `NumBigint` for the selected statistics project. | Item 4 |
| `cpython_hashlib_api_subset` | Generated Rust loses `handle` and `data` after successful sequential `try` bodies. | Item 5 |
| Structured concurrency builds | Generated `Error` has no `From<ScopeFailure>` conversion for the project nominal. | Item 6 |
| File-handle builds | Generated module-local and project-hoisted file-handle types are distinct Rust types. | Item 6 |
| `nested_optional_safe_operations` | Generated Rust calls `unwrap_or` and `map` on a union after duplicate optional conversion. | Item 7 |
| CLI CFG panics | Invalid source reaches `cfg.rs:300` with a branch terminator that has one target. | Item 8 |
| Protocol diagnostic mismatch | The fixture expects `SIFR-PROTO-0001`, but unresolved `MissingBound` names produce `SIFR-NAME-0003`. | Item 9 |
| Method-slot placeholder | `SlotContract` has an empty `pass` body in the matrix fixture. | Item 10 |
| Method-slot runtime output | Runtime emits five canonical lines and count `3`. The driver expects four lines and count `2`. | Item 10 |
| Read-only Python duration | All 25 variants passed in 314,714 ms. The isolated run passed in approximately 68 seconds. | Qualification rule B |

The five failed areas map as follows:

| Failed area | Inventory owner |
| --- | --- |
| Rust interop | Linked delivery A and Item 10 |
| Generated-code quality | Item 6 |
| Performance | Items 2 and 3 |
| Fuzz | Item 3 |
| Ecosystem | Item 3 |

The three failed Cargo commands map as follows:

| Failed Cargo command | Inventory owner |
| --- | --- |
| `sifr_stdlib` with all features | Item 1 |
| Regular `sifr` suite | Items 8 and 9 |
| Ignored `sifr_driver` suite | Items 4, 6, and 10 |

The E2E runner reported 13 failed fixtures. The source defects are
`cpython_hashlib_api_subset` and `nested_optional_safe_operations`. Eleven
fixtures are collateral members of the failed optional-union batch.

All rows have one current owner. No new failure needs a new issue. The
installed-sysroot boundary remains an approved resolved row and stays outside
this phase.

### Item 0 record

State: complete

PR: [#3425](https://github.com/sifr-lang/sifr/pull/3425)

Base SHA: `4815e74c2cf59989e8eba5afb4f7ca2e31b7d097`

Candidate SHA: `8146e5057ad80457285bb72c68b8b7b4cfe825d3`

Merge SHA: `97f72e9383313482d67ad61b8e34c5bdaa8f5fb8`

Changed paths: this phase document.

Validation: the unchanged rebaseline evidence was reused. Document links,
code fences, item numbering, terminology, and diff checks passed. No compiler
file changed, so the Sifr gates did not apply.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3425 review comment](https://github.com/sifr-lang/sifr/pull/3425#issuecomment-5376641794).

Deferred follow-up: Item 12 must make the 15 failing-variant total
mechanically reconcilable. Closed historical ownership records remain
unchanged.

Next action: implement Item 1.

## Item 1: Migrate the Native Hash Test to Bytes

Purpose: Preserve native hash coverage through the canonical bytes API.

Scope:

- Replace calls to `sha256`, `md5`, and `sha1` in the all-features test.
- Use `sha256_bytes`, `md5_bytes`, and `sha1_bytes` with `b"abc"`.
- Compare the complete digest bytes with the known vectors.
- Keep the public `sifr.hashlib` bytes constructors unchanged.
- Do not restore a string helper in `sifr_stdlib::hash`.

Acceptance criteria:

- `cargo test -p sifr_stdlib --all-features` passes.
- The test checks complete digest values, not only digest lengths.
- Text enters a hash function only through explicit encoding.
- The compatibility guard still rejects removed string helpers.

### Item 1 record

State: complete

PR: [#3427](https://github.com/sifr-lang/sifr/pull/3427)

Base SHA: `e0dc41e993c2ddef0f61f6eec207ef2de88c7739`

Candidate SHA: `07c213d8cf9b7aada0e55796469fb72195b5af39`

Merge SHA: `6726dd6fcb6f1edd68c6272813437ba5885fa231`

Changed paths: `crates/sifr_stdlib/tests/api_behavior.rs`.

Validation: the `sifr_stdlib` all-features suite passed with 53 tests. The
focused driver compatibility guard passed. Rust formatting, diff checks, and
the first-party file-size guardrail passed. No compiler file changed, so the
Sifr create-PR and merge gates did not apply.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3427 review comment](https://github.com/sifr-lang/sifr/pull/3427#issuecomment-5376679974).

Deferred follow-up: a later hardening item can replace the remaining
length-only SHA-224, SHA-384, SHA-512, BLAKE2b, and BLAKE2s assertions with
complete known vectors. This does not affect the Item 1 contract.

Next action: implement Item 2.

## Item 2: Give Performance Tools a Canonical Project Root

Purpose: Make the LSP benchmark use the canonical project layout.

Scope:

- Add an explicit project-root field to the benchmark input if one is absent.
- Resolve `sifr.toml` from that project root.
- Resolve `src/main.sifr` from the same project root.
- Remove the sibling-manifest assumption from benchmark validation.
- Use the normal project or session API when it already supplies this contract.
- Do not add ancestor-search fallback behavior only for this fixture.

Acceptance criteria:

- The LSP cold-start variants pass with `sifr.toml` above `src/`.
- The benchmark accepts one project root and one source path.
- No benchmark accepts the old flat project layout.
- LSP behavior uses the same provider-backed project inputs as production.

### Item 2 record

State: complete

PR: [#3429](https://github.com/sifr-lang/sifr/pull/3429)

Base SHA: `dec6c03558725922c258a7c3d975257f2712d0de`

Candidate SHA: `ddcecec119601724487adef2e1f888fc65a646de`

Merge SHA: `c872dafa37024762fc8d4634f5b0af31a65eb5fb`

Changed paths: the performance benchmark manifest, validator, runner, LSP
benchmark, minimal LSP project fixture, and performance budget documentation.

Validation: manifest validation and its flat-layout negative self-test passed.
All 18 LSP scenarios passed in smoke mode. The transfer guardrails, JSON and
Python checks, diff checks, and first-party file-size guardrail passed. The
performance rules runner passed its manifest, runner, and budget variants. Its
two trend-policy variants stopped only on the stale manifest hash assigned to
Item 3. No compiler file changed, so the Sifr gates did not apply.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3429 review comment](https://github.com/sifr-lang/sifr/pull/3429#issuecomment-5376741902).

Deferred follow-up: later hardening can cover the valid-root and invalid-source
branch, align explicit and default `src` notation in both fixtures, and revisit
path normalization only if manifest authoring requires it. The current spelling
check is fail-closed.

Next action: implement Item 3.

## Item 3: Complete Migrated Evidence Freshness

Purpose: Finish repository-consumer migration for the canonical source layout.

Scope:

- Refresh the performance trend hash after Item 2 changes its manifest.
- Change both missing-import fuzz seed paths to `src/main.sifr`.
- Refresh the checksums for `curated_cli_math` and `curated_data_flow`.
- Keep generated evidence generation deterministic.
- If Rust-interop validation stops at linked delivery A, record that stoppage there.

Acceptance criteria:

- All performance variants pass.
- The fuzz smoke area passes.
- All ecosystem compatibility variants pass.
- Every recorded source path exists.
- Every recorded hash matches its canonical input.
- No reader accepts an old path as a fallback.
- The item does not repair or waive the linked Rust-interop failure.

### Item 3 record

State: complete

PR: [#3432](https://github.com/sifr-lang/sifr/pull/3432)

Base SHA: `9f40cccfd3d8b16d8c77509c8c4e43144aba8c18`

Candidate SHA: `ff8c41c53c7f618c98993f312d09f8d85a2424ed`

Merge SHA: `ab96e12963e8d8579dcf238395dd7c648ba0953c`

Changed paths: the trend, fuzz, and ecosystem evidence; the diagnostic
fixture layout; stdlib structural template retention; LSP Python package
ownership; focused tests; and this phase record.

Validation: all 39 fuzz/property variants and all 34 ecosystem variants
passed. The performance representative, full, and smoke profiles passed all
commands and budgets. The diagnostic harness, focused driver and LSP tests,
formatting, maintainability checks, and file-size guard passed. The
representative artifact has SHA-256
`9ab0db8499f1a42905983fe76ab756533fbb7655517ac59a78bd09671506cb72`.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all earlier checks and stopped at the Rust-interop matrix. The matrix reported
only linked delivery A's missing negative fixture and Item 10's empty
method-slot fixture. The item did not change or waive either later delivery.

Review evidence: the exact-SHA Opus review returned `SATISFIED`. No
remediation review was used. The evidence is in the
[#3432 review comment](https://github.com/sifr-lang/sifr/pull/3432#issuecomment-5377265347).

Deferred follow-up: linked delivery A owns
`negative/package_generated_type_import_rejected.sifr`. Item 10 owns the
method-slot fixture. No Item 3 mechanism defect remains.

Next action: implement Item 4.

## Item 4: Correct the Dependency Feature Assertion

Purpose: Make the driver test match reachable module metadata.

Scope:

- Keep the production dependency aggregation path unchanged unless evidence finds a defect.
- Require the transitive `math` feature for the selected statistics project.
- Assert that `NumTraits`, `NumBigint`, and the broad `numeric` feature are
  absent for that project.
- Add a separate positive `NumBigint` case only if current coverage is absent.

Acceptance criteria:

- The focused project-entrypoint tests pass.
- The reachable `math` feature appears in the generated dependency plan.
- The unreachable numeric features do not appear in that plan.
- The test does not use an unrelated module to request a numeric feature.

### Item 4 record

State: complete

PR: [#3434](https://github.com/sifr-lang/sifr/pull/3434)

Base SHA: `17274ca1e60b60dbb254710b836c3cdb1c0be32e`

Candidate SHA: `a6a2ee7812c4f58d762bc40d369028478e5e68e7`

Merge SHA: `a566074bad6d49d58b9b71af8846792507646488`

Changed paths: the focused project-entrypoint test and the corrected Item 4
dependency contract.

Validation: all four focused project-entrypoint tests passed. The existing
positive `_bigint` typed-metadata authority test passed. Formatting, diff,
file-size, HIR maintainability, and driver maintainability checks passed.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all earlier checks and stopped at the Rust-interop matrix. The matrix reported
only linked delivery A's missing negative fixture and Item 10's empty
method-slot fixture.

Review evidence: the exact-SHA Opus review returned `SATISFIED`. No
remediation review was used. The evidence is in the
[#3434 review comment](https://github.com/sifr-lang/sifr/pull/3434#issuecomment-5377332955).

Deferred follow-up: linked delivery A and Item 10 retain their recorded
fixtures. No Item 4 mechanism defect remains.

Next action: implement Item 5.

## Item 5: Preserve Successful Sequential `try` Bindings

Purpose: Make Sifr lexical scope independent of generated Rust block scope.

Scope:

- Define the HIR scope of a name assigned by a successful `try` body.
- Reject a name when not all continuing paths initialize it.
- Emit enclosing-scope storage when later statements use the name.
- Preserve the exact declared type and ownership state.
- Keep typed error propagation and exhaustive handling.
- Preserve file closure after read success and read failure.
- Do not rewrite `hashlib.file_digest` only to avoid the compiler defect.

Acceptance criteria:

- `cpython_hashlib_api_subset` checks, builds, and runs.
- A focused sequential-`try` fixture uses two successful bindings afterward.
- A negative fixture rejects a conditionally uninitialized binding.
- Generated code contains no data-dependent `.unwrap()` or `.expect()`.
- The file handle closes once on each continuing or error path.

## Item 6: Use One Project-wide Nominal Identity

Purpose: Make structured Rust types stable across project and module emission.

Scope:

- Define one nominal registry for a complete generated project.
- Key each entry by the canonical source or HIR nominal identity.
- Record one Rust owner path for each nominal.
- Make project hoisting and module projection use the same registry.
- Preserve structured generic arguments and imports.
- Remove duplicate local declarations for hoisted nominals.
- Do not restore `Type::rust_type()` or a named-string fallback.

Acceptance criteria:

- All structured-concurrency quality variants pass.
- Both file-handle generated builds pass.
- `ScopeFailure` has one generated identity in the project.
- `Error: From<ScopeFailure>` uses that project-wide identity.
- Each native file-handle nominal has one generated identity.
- Unsupported types return a structured code-generation error.
- The structured-type guard and mutation test pass.

## Item 7: Make Optional and Union Conversion Single-pass

Purpose: Prevent duplicate conversion of nested optional unions.

Scope:

- Track the source representation and target representation for each conversion.
- Apply an `Option` or union wrapper conversion exactly once.
- Keep conversion recursive for nested generic types.
- Cover assignments, arguments, returns, defaults, and narrowing joins.
- Do not patch emitted Rust text after structured emission.

Acceptance criteria:

- `nested_optional_safe_operations` checks, builds, and runs.
- Focused tests cover `Option[T]`, `Option[Union]`, and nested optional unions.
- Generated Rust calls `map` or `unwrap_or` only on an `Option` value.
- No string renderer or raw-code escape implements the correction.

## Item 8: Replace Invalid-program CFG Panics

Purpose: Make invalid source produce a normal compiler result.

Scope:

- Identify the two negative fixtures that create one-successor branch blocks.
- Correct CFG construction or terminator selection at the producing path.
- Change invariant validation to return a structured internal compiler error.
- Keep `assert!` only for programmer invariants that user input cannot reach.
- Add direct coverage for both invalid-source shapes.

Acceptance criteria:

- The broad CLI test has no caught CFG panic.
- Both invalid fixtures produce stable source diagnostics.
- No user-controlled source reaches the panic at `crates/sifr_lowering/src/cfg.rs:300`.
- Valid control-flow fixtures keep their existing semantics.

## Item 9: Separate Protocol and Name-resolution Intent

Purpose: Make the protocol fixture reach the diagnostic that it claims to test.

Scope:

- Define `MissingBound` in the protocol-conformance fixture.
- Add or retain a separate unknown-bound name-resolution fixture.
- Require one diagnostic family for each fixture intent.
- Do not accept both diagnostic outcomes.
- Keep external protocol compatibility unchanged.

Acceptance criteria:

- The protocol fixture emits `SIFR-PROTO-0001` only when all names resolve.
- An unknown-bound fixture emits the canonical name diagnostic.
- The broad CLI test passes.
- Diagnostic snapshots have one stable owner and source location.

## Item 10: Complete Method-slot Verification

Purpose: Make the fixture and driver test describe the complete checked slot
contract.

The canonical runtime output is:

```text
value-normalized
input-receiver
input-serialized
value-no-context
value-shared-3
```

Scope:

- Replace the empty `SlotContract` body with a declaration that the fixture uses.
- Keep the receiver-value serializer and its `input-serialized` output.
- Update the driver expectation to include all five runtime output lines.
- Require the shared-context count to be `3` after the three context calls.
- Keep slot order and slot count derived from one checked handler set.
- Do not restore the old four-line output as a compatibility result.
- Do not change method-slot product behavior to satisfy stale evidence.

Acceptance criteria:

- The method-slot schema contains no empty placeholder class.
- The method-slot case reports no error in the Rust-interop matrix.
- The focused ignored driver runtime test passes.
- Runtime output contains `input-serialized` in declaration order.
- Runtime output ends with `value-shared-3`.
- The fixture keeps validator, receiver, serializer, no-context, and shared-context coverage.
- Item 10 can record a full-matrix stop only for linked delivery A.
- Item 11 owns the final full-matrix pass after linked delivery A merges.

## Required Linked Delivery A: Rust-interop Evidence Path

The active
[`ad-hoc-rust-interop-fixture-matrix-repair.md`](./ad-hoc-rust-interop-fixture-matrix-repair.md)
owns this correction.

That issue must select the checked-in `negative/src/` source path. It must
update the matrix manifest, documentation, and Cargo probe together.

This phase must not implement the same correction. Item 11 consumes the merged
SHA and validation evidence from that owner.

The linked owner can prepare its path correction before Item 10 merges. Its
final full-matrix evidence must use main after Item 10. The matrix includes the
method-slot fixture.

## Qualification Rule B: Read-only Python Doctor

The completed
[`python-interop-readonly-inspection-timeout.md`](../archive/python-interop-readonly-inspection-timeout.md)
owns the prior deterministic performance defect.

The latest contended run completed all 25 variants in 314,714 ms. The direct
area command did not enforce the merge-gate timeout. An isolated run completed
in approximately 68 seconds under the 300-second limit.

Do not change implementation or timeout policy without new deterministic
evidence. Item 11 runs the doctor once on the final integrated candidate.

If that run times out, run one isolated diagnostic profile with new timing
evidence. Do not repeat an unchanged performance gate.

If the isolated run identifies a new deterministic defect, record a new
performance issue and stop Item 11.

## Item 11: Integrate Linked Deliveries and Qualify the Candidate

Purpose: Combine phase-owned corrections with independently owned changes.

Dependencies:

- Items 1 through 10 are merged.
- Required linked delivery A is merged.

Scope:

- Rebase or merge the latest main branch through normal Git operations.
- Record the linked delivery merge SHA.
- Run every previously failing focused area.
- Run the read-only Python doctor once.
- Stop if an external owner remains incomplete.
- Do not copy changes from another active worktree.

Acceptance criteria:

- Every inventory row passes or has an approved non-failure classification.
- The Rust-interop matrix passes.
- The method-slot matrix and runtime fixture pass.
- The read-only Python doctor passes without a timeout increase.
- No linked owner has unmerged validation inputs.

## Item 12: Add the Regression Guard and Close the Phase

Purpose: Prevent the same incomplete migration and representation defects.

Scope:

- Add guard coverage for stale fixture paths and stale recorded hashes.
- Keep the existing no-compatibility guard intact.
- Add structured-type identity and conversion regression coverage.
- Record all item PRs, candidate SHAs, merge SHAs, and review evidence.
- Run the create-PR gate on the final implementation candidate.
- Run the merge gate once on the final implementation candidate.
- Update architecture documentation only if an architecture contract changes.
- Update the roadmap only if roadmap status changes.
- If a new external failure stops a gate, record it in its owning issue.
- Mark Item 12 blocked when that external failure prevents completion.
- Do not absorb the failure or classify a stopped gate as a pass.

Acceptance criteria:

- `scripts/run_all_tests.sh --profile create-pr` passes.
- `scripts/run_all_tests.sh` passes once on the final candidate.
- `cargo fmt --check` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `python3 scripts/check_hir_maintainability_guardrails.py` passes.
- The first-party file-size guardrail passes.
- The compatibility guard rejects restored duplicate paths.
- The phase record contains no unowned failure.
- A stopped external gate leaves this phase open and Item 12 blocked.

## Validation Matrix

| Changed area | Required focused validation |
| --- | --- |
| Native hash adapters | `cargo test -p sifr_stdlib --all-features` and focused hash E2E fixtures |
| Performance project input | Performance LSP cold-start variants and manifest validation |
| Evidence paths and hashes | Performance, fuzz, and ecosystem area runners |
| Rust shared-bridge path | Linked delivery A Rust-interop matrix and Cargo probe |
| Dependency feature selection | Focused driver entrypoint and Cargo-manifest tests |
| `try` scope | Lowering tests, codegen tests, and `cpython_hashlib_api_subset` |
| Structured nominal identity | Codegen, driver generated builds, and generated-code quality |
| Optional and union conversion | Type-system, HIR, codegen, and focused E2E fixtures |
| CFG diagnostics | Lowering CFG tests and broad CLI negative tests |
| Protocol diagnostics | Protocol conformance tests and broad CLI tests |
| Method slots | Focused ignored driver runtime test and the method-slot matrix case. Item 11 runs the full matrix. |
| Regression guards | Focused stale-path, stale-hash, nominal-identity, conversion, and no-compatibility self-tests |
| Python doctor | One final-candidate run and one isolated profile only after a timeout |

## Review and Merge Rules

Run the create-PR gate before each draft implementation pull request. Each item
uses one draft pull request.

The review request must name the exact base SHA and candidate SHA.

The review can block only for an in-scope regression or omission. Record an
external issue as follow-up evidence in its owning issue.

Run the merge gate once on the final candidate after review corrections.

Do not repeat a gate when its inputs did not change. Do not use a cold-cache
run as performance evidence.

Before a long Cargo gate, inspect free disk space and the private target size.
Obey the Cargo storage rules in `AGENTS.md`.

## Item Record Template

Add this record after an item merges:

```text
State: complete | blocked
PR:
Base SHA:
Candidate SHA:
Merge SHA:
Changed paths:
Validation:
Review evidence:
Deferred follow-up:
Next action:
```

## Phase Exit State

The phase is complete when all of these conditions are true:

- All canonical product features in the objective work.
- All registered tests pass without compatibility restoration.
- Every migrated repository consumer uses the canonical path or schema.
- Structured Rust types remain the only type-generation representation.
- User-controlled invalid input cannot trigger the recorded CFG panic.
- The method-slot fixture and runtime test describe the canonical five-line output.
- The stale-path, stale-hash, identity, conversion, and no-compatibility guards pass.
- The linked Rust-interop delivery is merged and recorded.
- The read-only Python doctor passes under the unchanged command limit.
- The final create-PR and merge gates pass on the same implementation candidate.

## Current Handoff

Current state: Items 0 through 4 are complete and recorded.

Next action: implement Item 5.

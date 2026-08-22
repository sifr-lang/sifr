# Ad Hoc Phase: Pre-v1 Canonical Contract Regression Closure

Status: active on 2026-08-22. Items 0 through 10G are complete. Item 10H is next.

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
| Residual nominal registry gaps | Item 6 reviews found two pre-existing gaps outside that implementation scope. | Item 10A |
| `try` continuation ownership | Item 10A validation exposed pre-existing moves and partial moves in generated continuation state. | Item 10B |
| Total `try` return emission | Item 10A validation exposed pre-existing fall-through `()` paths after exhaustive handlers. | Item 10C |
| Moved `try` binding declarations | Item 10B review found that a moved binding needed only as a later assignment target requires an enclosing declaration, but not value transport. | Item 10D |
| Nested-function default references | Item 10B review found that canonical HIR reference traversal does not visit parameter default expressions. | Item 10E |
| Imported union nominal paths | Item 10B validation reproduced a project-codegen panic for the canonical imported union identity `sifr.csv.Error` on the exact item base. | Item 10F |
| Unmatched conditional handlers | Item 10C review found that an `IOError` kind handler can fall through without propagating an unmatched error. | Item 10G |
| Nested `try/finally` propagation | Item 10C review found that nested `try/finally` can select an invariant panic inside a non-return-capturing `try` closure. | Item 10H |
| Diagnostic harness Clippy | Item 10C validation confirmed that Item 3 passed `FixtureLayout` by value although the helper only borrows it. | Item 10I |
| Nested-function signature scope | Item 10E remediation review found that nested calls can combine a scoped callable binding with an unscoped name-keyed signature. | Item 10J |
| Checked-stdlib parent upcast | Item 10F validation reached Rust compilation after it closed the imported-union panic. The emitted child-to-parent conversion has no retained `From` implementation. | Item 10K |
| User-error parent handler | Item 10G remediation review found that lowering accepts parent coverage, but code generation matches only exact user nominals. | Item 10L |
| Nested-function try-channel codegen | Item 10G remediation review found that code generation does not isolate the active try-channel stack at a nested function boundary. | Item 10M |

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
| Open `IOError` dispatch and stale demo output | Six subclass handlers do not cover `kind = "Other"`; the new `try` emission also changes checked-in demo output. | Item 5A |
| Structured concurrency builds | Local and hoisted `ScopeFailure` types have different identities. | Item 6 |
| File-handle builds | Local and hoisted file-handle types have different identities. | Item 6 |
| `nested_optional_safe_operations` | Lowering converts an optional union more than once. | Item 7 |
| CLI CFG panics | Invalid control flow can create a branch with one successor. | Item 8 |
| Protocol diagnostic mismatch | The fixture references an unknown bound before conformance checking. | Item 9 |
| Method-slot placeholder | The matrix fixture contains an empty placeholder declaration. | Item 10 |
| Method-slot runtime output | The driver expectation predates the receiver-value serializer and keeps the stale count `2`. | Item 10 |
| Direct `IOError` kind nominal | A direct `Type::Class` path can register a kind alias that has no Rust struct. | Item 10A |
| Dead shared `ParseError` | The `uuid_and_datetime` output keeps a shared definition beside a local duplicate. | Item 10A |
| `try` continuation moves | Generated continuation state returns locals that the `try` body already moved or partially moved. | Item 10B |
| Exhaustive `try` returns | Generated error handling can fall through as `()` even when the source body and handlers return. | Item 10C |
| Moved binding later reassignment | Continuation liveness does not distinguish the declaration needed for a later assignment from the value needed for a later read. | Item 10D |
| Nested-function default expression | Canonical HIR reference traversal visits nested bodies by configuration but omits their parameter default expressions. | Item 10E |
| Imported union nominal registry | Project code generation can miss the crate-root path for a canonical imported union member such as `sifr.csv.Error`. | Item 10F |
| Unmatched `IOError` kind | A conditional handler chain has no residual-error path when no kind condition matches. | Item 10G |
| Nested `try/finally` panic | Error capability is inferred from return capture, so a nested `try/finally` can emit a reachable invariant panic. | Item 10H |
| Diagnostic harness strict Clippy | `fixture_name_for_seed` consumes a non-`Copy` layout enum that it only inspects. | Item 10I |
| Nested-function signature collision | The scope binding is lexical, but the function signature registry is name-keyed and is not restored when a sibling scope ends. | Item 10J |
| Checked-stdlib parent conversion | Project nominal relocation can remove the `From<Child>` implementation that a by-value child-to-parent upcast requires. | Item 10K |
| User-error parent dispatch | A user-defined parent handler covers a child during lowering, but code generation can treat that handler as unsupported and propagate the child. | Item 10L |
| Nested-function try-channel leak | Nested-function emission can reuse the enclosing try closure's error-channel stack instead of the nested function's `Result` channel. | Item 10M |
| Read-only Python duration | The latest contended run completed in 314,714 ms. An isolated run completed in approximately 68 seconds. | Qualification rule B |

## Scope

### In scope

- Migrate tests from removed string helpers to bytes-native helpers.
- Define canonical project-root inputs for performance tools.
- Refresh paths, hashes, and checksums after the source-layout migration.
- Correct generated dependency assertions.
- Preserve successful bindings across sequential `try` statements.
- Preserve unmatched base `IOError` values and refresh affected generated demos.
- Give each generated nominal one project-wide identity.
- Make optional and union conversion representation-aware and single-pass.
- Replace invalid-program CFG panics with structured diagnostics.
- Separate name-resolution and protocol-conformance fixture intent.
- Complete the method-slot fixture and align its runtime evidence.
- Close the two residual nominal registry gaps from the Item 6 reviews.
- Return only live locals from generated `try` continuations.
- Emit total control flow for exhaustive `try` return paths.
- Separate declaration liveness from value liveness for moved `try` bindings.
- Include nested-function parameter defaults in canonical HIR reference traversal.
- Register canonical crate-root paths for imported union members.
- Preserve unmatched conditional errors through the checked error channel.
- Preserve nested `try/finally` errors inside every error-capable `try` closure.
- Remove the Item 3 diagnostic-harness strict-Clippy warning.
- Keep nested-function callable metadata in the same lexical scope as its binding.
- Preserve checked-stdlib parent upcasts after project nominal relocation.
- Align user-defined parent-handler dispatch with lowering coverage.
- Isolate code-generation try channels at nested function boundaries.
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
  -> Item 5A open IOError dispatch and demo freshness
  -> Item 6  project-wide nominal identity
  -> Item 7  nested optional conversion
  -> Item 8  invalid-program CFG diagnostics
  -> Item 9  protocol fixture intent
  -> Item 10 method-slot verification closure
  -> Item 10A residual nominal registry consistency
  -> Item 10B try continuation ownership
  -> Item 10C total try return emission
  -> Item 10D try declaration and value liveness
  -> Item 10E nested-function default traversal
  -> Item 10F imported union nominal paths
  -> Item 10G unmatched conditional handler propagation
  -> Item 10H nested try/finally error propagation
  -> Item 10I diagnostic harness strict Clippy
  -> Item 10J nested-function signature scope
  -> Item 10K checked-stdlib parent upcast
  -> Item 10L user-error parent handler dispatch
  -> Item 10M nested-function try-channel isolation
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

### Item 5 record

State: complete

PR: [#3436](https://github.com/sifr-lang/sifr/pull/3436)

Base SHA: `adff7eebadcf6df33daf68e7a494d55590dcb17b`

Candidate SHA: `606a5a6e52d41e25b799e949fed59ee838778c54`

Merge SHA: `4b521a4d1554bd9e89ba243f26eccaed485784d7`

Changed paths: lowering scope and `try` flow, shared HIR flow analysis,
structured `try` emission, canonical I/O error metadata, focused compiler
tests, and pass/fail E2E fixtures.

Validation: the type-system, HIR, lowering, and codegen suites passed. Clippy
passed with warnings denied. The hash and sequential-`try` fixtures checked,
built, and ran. The negative fixture emitted `SIFR-NAME-0001`. Generated hash
code had two file-close calls and no data-dependent unwrap or expect. Format,
diff, maintainability, and file-size checks passed.

The create-PR and merge gates each ran once on the candidate SHA. Both stopped
at demo emitted-freshness and reported the same 21 stale generated outputs.
Neither gate was repeated.

Review evidence: the initial exact-SHA review found two blocking flow defects.
The remediation review confirmed that one shared HIR flow analysis closed the
lowering/codegen disagreement. It then found a new mechanism defect: six
subclass handlers do not cover a base `IOError` with `kind = "Other"`. The
review and gate evidence is in the
[#3436 review comment](https://github.com/sifr-lang/sifr/pull/3436#issuecomment-5377738618).
The review-round rule assigns that new defect to Item 5A. No third review ran.

Deferred follow-up: Item 5A owns open I/O-error dispatch and the 21 generated
demo refreshes. Items 8 and 9 retain the broad negative-suite failures.

Next action: implement Item 5A.

## Item 5A: Preserve Open `IOError` Dispatch and Refresh Demo Evidence

Purpose: Keep promoted `try` bindings without treating base I/O errors as a
closed set of subclass kinds.

Scope:

- Require a base `IOError` or catch-all handler for kind-total coverage.
- Keep every subclass handler guarded by its exact runtime kind.
- Preserve an unmatched base `IOError`; do not run a subclass handler for it.
- Remove the remaining duplicate I/O-error subclass registries.
- Refresh the 21 checked-in demo outputs changed by Item 5.
- Do not fabricate a promoted value or add a panic/fallback path.

Acceptance criteria:

- Six subclass handlers alone do not claim to cover base `IOError`.
- A final base handler preserves `kind = "Other"` without subclass dispatch.
- Exact subclass kinds still select their declared handlers.
- Promoted bindings compile only when every continuing path initializes them.
- The demo emitted-freshness guard passes.
- Generated code contains no data-dependent `.unwrap()` or `.expect()`.
- Focused I/O-error fixtures check, build, and run.

### Item 5A record

State: complete

PR: [#3438](https://github.com/sifr-lang/sifr/pull/3438)

Base SHA: `625aaa5ad0b13634697dcf8fcc2dd72218240001`

Candidate SHA: `080fd152e55ca96a8caff682c2049d24ea37fe48`

Merge SHA: `4185ae9223021001c1672d88cc36ca4c1b270ab1`

Changed paths: I/O-error coverage lowering, structured handler emission,
canonical I/O-error registry consumers, focused lowering and codegen tests,
one pass fixture, one fail fixture, and 21 generated demo companions.

Validation: the full lowering suite passed 1,026 tests with one ignored test.
The full codegen suite passed 1,093 tests. A real `FileNotFound` value selected
the named handler. An `IOError` with `kind = "Other"` selected the base handler.
The pass fixture checked, built, and ran. The fail fixture emitted
`SIFR-RESULT-0005` at the recorded column. A generated-code scan found no
data-dependent unwrap or expect. Affected-package clippy passed with warnings
denied. Demo freshness, format, diff, HIR maintainability, and file-size checks
passed.

The create-PR and merge gates each ran once on the candidate SHA. Both passed
all earlier guardrails, including demo freshness, and stopped at the same two
Rust-interop matrix rows. Linked delivery A owns the missing shared-bridge
negative source path. Item 10 owns the empty method-slot declaration. Neither
gate was repeated.

Review evidence: the exact-SHA Opus review returned `APPROVED`. It confirmed
that lowering no longer treats child handlers as parent coverage, codegen no
longer makes the last named kind unconditional, and the type system owns the
single I/O-error kind registry. No remediation review was used. The evidence
is in the
[#3438 review comment](https://github.com/sifr-lang/sifr/pull/3438#issuecomment-5377842874).

Deferred follow-up: linked delivery A and Item 10 retain their recorded matrix
rows. The broad fail harness still stops first at Item 9's protocol diagnostic
mismatch. No Item 5A mechanism defect remains.

Next action: implement Item 6.

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

### Item 6 record

State: complete

PR: [#3440](https://github.com/sifr-lang/sifr/pull/3440)

Base SHA: `c8b50ec1a383899156a446a30401cdd30b20e49c`

Candidate SHA: `57698a86deb400cecddc88ca4323daf2310094d6`

Merge SHA: `ca8e2ff6ebb0078128820adbb90b5762de671c86`

Changed paths: the project nominal registry, error-reference collection,
project code-generation consumers, stdlib item filtering, 75 generated demo
companions, and the typed-intrinsic allowlist.

Validation: all 1,097 codegen tests passed. Codegen Clippy passed with warnings
denied. All seven structured-concurrency variants passed. Both native
file-handle project builds passed.

The focused tests prove one `ScopeFailure` definition and one shared
`From<ScopeFailure>` conversion. They also prove that transitive stdlib
nominals use the project registry.

The structured-type guard and its mutation self-test passed. Demo freshness,
format, diff, HIR maintainability, file-size, and all stdlib ownership guards
passed.

The create-PR and merge gates each ran once on the final candidate. Both gates
passed every guardrail and stopped at the same two Rust-interop matrix rows.
Linked delivery A owns the missing shared-bridge source. Item 10 owns the empty
method-slot declaration.

Review evidence: the initial exact-SHA review found a dangling project export
for exception-only `IOError` kind aliases. The first remediation added the
emission-aligned filter and a package regression.

The user authorized two additional exact-SHA reviews after gate-discovered
baseline and allowlist omissions. Both final reviews returned `SATISFIED` with
no blocking findings.

The review evidence is in the
[#3440 initial and remediation comment](https://github.com/sifr-lang/sifr/pull/3440#issuecomment-5378067779),
the
[#3440 generated-baseline comment](https://github.com/sifr-lang/sifr/pull/3440#issuecomment-5379696658),
and the
[#3440 final allowlist comment](https://github.com/sifr-lang/sifr/pull/3440#issuecomment-5379730091).
The final gate evidence is in the
[#3440 gate comment](https://github.com/sifr-lang/sifr/pull/3440#issuecomment-5379745889).

Deferred follow-up: Item 10A owns two pre-existing nominal registry gaps. One
gap can seed an `IOError` kind alias from a direct `Type::Class`. The other gap
keeps a dead shared `ParseError` beside a local duplicate.

The allowlist scanner and demo rendering-style notes are infrastructure
suggestions. They do not change Item 6 behavior or acceptance.

Next action: implement Item 7.

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

### Item 7 record

State: complete

PR: [#3442](https://github.com/sifr-lang/sifr/pull/3442)

Base SHA: `c2acc22a8201a4e34bc312c33aa2d55a9f73feaa`

Candidate SHA: `251a2aa4cee107db93cf4a8760dd7c26d984fe82`

Merge SHA: `2648b79db168e0daa20f3ee41675c3e78a3ef1c5`

Changed paths: the single consuming-value conversion authority, call and
collection argument paths, local assignment paths, error conversion paths,
and focused option and union representation tests.

The root cause was two independent conversion authorities. Some paths first
adapted an optional wrapper and then applied recursive union conversion. Other
paths applied the same operations in reverse order.

The correction gives each conversion one source type and one target type. The
public authority applies an optional-wrapper transition first. It applies the
recursive union or class transition only when the wrapper did not change.
The lower-level upcast helper is private.

Validation: `nested_optional_safe_operations` passed check, build, and runtime
execution. All 1,101 codegen tests passed. Codegen Clippy passed with warnings
denied. Format, diff, HIR maintainability, and file-size checks passed.

The create-PR gate and merge gate each ran once on the final candidate. Both
passed all earlier checks. Both stopped at the same two Rust-interop matrix
inputs. Linked delivery A owns the missing shared-bridge negative source.
Item 10 owns the empty method-slot declaration. The gate evidence is in the
[#3442 gate comment](https://github.com/sifr-lang/sifr/pull/3442#issuecomment-5379842007).

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3442 review comment](https://github.com/sifr-lang/sifr/pull/3442#issuecomment-5379820829).

Deferred follow-up: the review recorded three suggestions. The suggestions
cover deeper private recursion, one redundant clone, and narrower test text
matching. No focused correctness error makes these suggestions phase work.

Next action: implement Item 8.

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

### Item 8 record

State: complete

PR: [#3444](https://github.com/sifr-lang/sifr/pull/3444)

Base SHA: `882947d53e1b49b584973b4e6096b7d0153b79fa`

Candidate SHA: `7d51849ba8d9672792eaf8671f1377e4e81b74f4`

Merge SHA: `4eb28a70c986d6c6f8752d84c7ca351b58035e02`

Changed paths: CFG construction and validation, CFG consumers in lowering,
focused match diagnostics, and the fixed-bug and crash regression records.

The root cause was a one-arm match that created a branch terminator with one
target. The CFG invariant requires at least two branch targets. Validation then
panicked on user-controlled invalid source.

The correction emits a direct jump for a one-arm match. CFG construction now
returns an invariant error. The lowering entry points convert that error to a
structured internal compiler diagnostic instead of catching a panic.

Validation: all 9 focused CFG tests and all 6 focused match-diagnostic tests
passed. The full lowering suite passed 1,029 tests, with 1 ignored. Lowering
Clippy passed with warnings denied. Direct checks of both negative fixtures
produced only `SIFR-MATCH-0001`. The fixed-bug and crash regression runner
passed all 6 variants. Format, diff, HIR maintainability, and file-size checks
passed.

The broad fail suite crossed both CFG fixtures without a panic. It stopped at
Item 9's protocol diagnostic mismatch.

The create-PR and merge gates each ran once on the final candidate. Both passed
all prerequisite guardrails. Both stopped at the same two Rust-interop matrix
inputs. Linked delivery A owns the missing shared-bridge source. Item 10 owns
the empty method-slot declaration. The gate evidence is in the
[#3444 gate comment](https://github.com/sifr-lang/sifr/pull/3444#issuecomment-5379944267).

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3444 review comment](https://github.com/sifr-lang/sifr/pull/3444#issuecomment-5379927019).

Deferred follow-up: the review recorded two pre-existing correctness issues.
Guarded wildcard arms can be treated as exhaustive. A two-arm `int | None`
match can reference an unresolved generated union module. These issues are not
CFG construction defects and need separate ownership. The review also suggested
deduplicating invariant diagnostics, propagating two conservative CFG helper
fallbacks, and strengthening the fixed-bug message lock. Direct tests already
lock the Item 8 diagnostic code and count.

Next action: implement Item 9.

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

### Item 9 record

State: complete

PR: [#3446](https://github.com/sifr-lang/sifr/pull/3446)

Base SHA: `3930d6e2616fefe6925fa6df4ed8d00acc6ce0a3`

Candidate SHA: `5e7f78f40c87faca49589b0f062c47f9854091d8`

Merge SHA: `8688fe22c51eeb552c0a3affc9b8531be1395b18`

Changed paths: two negative fixtures and their focused lowering tests. No
production compiler file changed.

The root cause was mixed fixture intent. The forwarding fixture referenced an
unknown protocol name. Name resolution correctly stopped before protocol
conformance checking.

The correction defines the protocol before it forwards an unbounded type
variable. The existing unresolved `TypeVar` bound fixture now owns the
name-resolution result. The change did not add a duplicate unknown-bound
fixture.

Validation: the two focused lowering tests passed. Each test produced one
diagnostic in its intended family. The full lowering suite passed 1,029 tests,
with 1 ignored. All 567 broad fail fixtures passed. Lowering Clippy passed with
warnings denied. Format, diff, HIR maintainability, and file-size checks passed.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3446 review comment](https://github.com/sifr-lang/sifr/pull/3446#issuecomment-5379972805).

The review confirmed that the removed bound-mismatch test body was duplicate
coverage. Its mechanism remains covered by the protocol diagnostics tests and
the existing non-conforming forwarding fixture.

No Sifr create-PR or merge gate applied because the item changed only tests and
fixtures.

Deferred follow-up: the review suggested names that distinguish an unbounded
forwarded type variable from an unknown bound. A fixture rename would change
lexical fixture order. It is not required for the diagnostic contract. An
optional test-target Clippy probe also found pre-existing warnings outside this
diff. The normal required Clippy lane passed.

Next action: implement Item 10.

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

- Treat the required `SlotContract` adapter marker as a declaration, not as an
  empty placeholder. Continue to reject ordinary empty classes.
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

### Item 10 record

State: complete

PR: [#3448](https://github.com/sifr-lang/sifr/pull/3448)

Base SHA: `bc4a77ae6cd4fe105c85cd5a184eb44a3ae263af`

Candidate SHA: `c049f2d6e65e76f33efc7f2c890e1df9fb545c0c`

Merge SHA: `66b4065ea6832fea58c903c810e163e7d6aeb6a5`

Changed paths: the method-slot driver expectation, the Rust-interop placeholder
classifier and its consumers, and the fixture-matrix mutation self-test. No
production compiler file changed.

Focused execution disproved the original planned repair. The compiler requires
`@class_adapter_marker` to decorate a field-less class that contains only
`pass`. Moving a used field into `SlotContract` correctly produced
`SIFR-META-0003`.

The actual root cause was the matrix helper. It treated every `pass` line as an
empty class placeholder. The correction identifies empty classes and exempts
the compiler-required adapter marker. An ordinary empty class remains rejected.

The runtime fixture already emitted the five canonical lines. The driver test
still expected four lines and shared-context count `2`. Its expectation now
includes `input-serialized` in order and ends with `value-shared-3`.

Validation: the 237-case fixture-matrix self-test passed. The focused ignored
runtime test passed. The focused ignored lifetime, thread, and shared-context
rejection test passed. The full driver suite passed 556 tests, with 76 ignored.
Driver library Clippy passed with warnings denied. Python syntax, format, diff,
HIR maintainability, and file-size checks passed.

The full fixture matrix no longer reports the method-slot row. It stops only at
linked delivery A's missing shared-bridge source.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3448 review comment](https://github.com/sifr-lang/sifr/pull/3448#issuecomment-5380018372).

No Sifr create-PR or merge gate applied because the item changed only tests and
verification code.

Deferred follow-up: the placeholder classifier intentionally narrows the old
any-`pass` heuristic to class bodies. A later verification change can decide
whether to cover docstring-plus-`pass`, multiline class headers, or decorator
spacing. `_scenario_checks.py` and `check_fixture_matrix.py` are 898 and 899
lines. The next change to either file must split it by responsibility. The
unrelated diagnostic-rendering Clippy lint remains outside this item.

Next action: implement Item 10A.

## Item 10A: Close Residual Nominal Registry Inconsistencies

Purpose: Remove the two pre-existing nominal registry gaps found during the
Item 6 reviews.

Scope:

- Reject `IOError` kind aliases as shared nominal owners in every collection path.
- Give the `uuid_and_datetime` `ParseError` one generated owner.
- Remove the dead shared definition or the duplicate local definition.
- Keep the public error behavior unchanged.
- Add focused registry consistency coverage.
- Do not add a fallback owner or a second nominal path.

Acceptance criteria:

- A direct `Type::Class` for an `IOError` kind cannot create a dangling owner path.
- The `uuid_and_datetime` output contains one `ParseError` identity.
- Every shared registry export resolves to one generated definition.
- No registered shared nominal also has a module-local duplicate.
- Focused tests and demo freshness pass.

### Item 10A record

State: complete

PR: [#3450](https://github.com/sifr-lang/sifr/pull/3450)

Base SHA: `529573333780f24f4ef8ab1cf6072ab3db2f4316`

Candidate SHA: `c11ccebe428ee9ac2c1593d4f8e27ace485394eb`

Merge SHA: `f27f04bb4f9ab36e3c1ae5133f3f8990ebe821e1`

Changed paths: the project shared-nominal registry, its focused tests, and 41
fresh generated demo outputs.

The direct class collector now rejects all six `IOError` kind aliases. The
synthetic prelude registers emitted builtin definitions before nominal
relocation. The generated `uuid_and_datetime` project now has one
`ParseError` definition and one canonical re-export.

Validation: all 1,103 code-generation tests passed. The focused direct-class
and emitted-builtin registry tests passed. The UUID demo and fixture compiled
and ran. Demo freshness, shared-definition, kind-alias, Clippy, format, diff,
HIR maintainability, and file-size checks passed.

The broad E2E pass run reported nine build failures. Exact-base reproduction
proved that all three reported Rust diagnostics existed at the base SHA. They
reduce to two compiler mechanisms: moved continuation state and non-total
return emission. Items 10B and 10C own those mechanisms.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3450 review comment](https://github.com/sifr-lang/sifr/pull/3450#issuecomment-5380279698).

The create-PR and merge gates each ran once on the exact candidate. All checks
before the Rust-interop matrix passed. The matrix stopped only for linked
delivery A. The evidence and base-failure classification are in the
[#3450 gate comment](https://github.com/sifr-lang/sifr/pull/3450#issuecomment-5380405067)
and its linked validation comment.

Deferred follow-up: Opus suggested comparing the computed Rust definition name
directly and adding another integration assertion. These are optional
hardening. Items 10B and 10C own the newly classified base defects.

Next action: implement Item 10B.

## Item 10B: Preserve Ownership Across Try Continuations

Purpose: Keep moved values out of generated continuation state after a `try`
body completes.

Scope:

- Compute the locals that are live after each `try` statement.
- Return and rebind only those live locals from the generated continuation.
- Exclude values that the `try` body moved or partially moved.
- Cover tuple destructuring and moved network or TLS resources.
- Do not add unconditional clones or a fallback continuation path.

Acceptance criteria:

- The config-parser fixture family compiles without partial-move diagnostics.
- `network_http_tls_loopback_split` compiles and runs without moved-resource diagnostics.
- `nominal_identity_alias_paths` has no continuation ownership diagnostic.
- Focused emitted Rust does not return dead or moved continuation locals.
- Focused lowering, code-generation, and E2E tests pass.

### Item 10B record

State: complete

PR: [#3452](https://github.com/sifr-lang/sifr/pull/3452)

Base SHA: `cbf256719ff6dc1ed800c878ad50390c046fa400`

Candidate SHA: `42cbe6ef798c8047b33289c9779dde94852e0046`

Merge SHA: `ffce9ef452a555441ebc8e3d9a84f7dfd4f719ba`

Changed paths: canonical HIR reference traversal, structured statement entry
points, `try` continuation emission, focused source-architecture and
code-generation tests, and 20 fresh generated demo outputs.

The emitter now passes following statements into each `try` statement. It
promotes only successful bindings that are referenced after that statement.
The reference query covers expression reads, nested-function bodies, and
scalar, field, subscript, attribute, augmented, and tuple assignment targets.
The statement entry points moved to a responsibility-specific module to keep
the maintained emitter source below the file-size limit.

Validation: all 1,107 code-generation tests passed. Ten focused sequential
`try` tests passed. The config-parser fixture family built. The CPython
config-parser and TLS loopback fixtures ran. Demo freshness, Clippy, format,
diff, HIR maintainability, and file-size checks passed.

Review evidence: the first exact-SHA Opus review found that assignment-target
liveness was incomplete. The remediation covered every canonical assignment
target form. The second and final review returned `SATISFIED`. The evidence is
in the [#3452 review comment](https://github.com/sifr-lang/sifr/pull/3452#issuecomment-5380564471).

The user authorized replacement gates after the first candidate stopped on
stale demo output. The create-PR and merge gates each ran once on the final
exact candidate. Every check before the Rust-interop matrix passed. Both gates
stopped only because linked delivery A still names the removed source path.
The evidence is in the [#3452 gate comment](https://github.com/sifr-lang/sifr/pull/3452#issuecomment-5380589941).

Deferred follow-up: Item 10D owns declaration-only liveness for a moved
binding used later only as an assignment target. Item 10E owns nested-function
parameter default traversal. Item 10F owns the imported-union nominal-path
panic reproduced on the exact Item 10B base. Test-only wrapper removal remains
optional hardening. The shared reference query intentionally treats assignment
targets conservatively until Item 10D separates declaration and value needs.

Next action: implement Item 10C.

## Item 10C: Emit Total Exhaustive Try Returns

Purpose: Preserve return-position control flow when a `try` body and all
matching handlers return.

Scope:

- Detect when the `try` body and its handlers make the construct total.
- Emit an expression or control-flow shape with no implicit `()` fall-through.
- Preserve exact nominal handler matching.
- Do not add a dummy return value or compatibility fallback.

Acceptance criteria:

- `imported_error_not_catch_all`, `try_union_error_alias`, and
  `try_union_error_channel` compile and run.
- `nominal_identity_alias_paths` has no fall-through type diagnostic.
- Generated Rust has no E0317 or E0308 diagnostic from a total `try` return.
- Focused lowering, code-generation, and E2E tests pass.

### Item 10C record

State: complete

PR: [#3454](https://github.com/sifr-lang/sifr/pull/3454)

Base SHA: `ae12c92f61d73128bd75fdd7de82989a148e5428`

Candidate SHA: `752d9258c71414d4736c8e4ebb23fdf09886fd3d`

Merge SHA: `fdc39c2ae0da9253f0bbc1b9ccff732aaa3424f9`

Changed paths: `try` statement emission, focused code-generation coverage, and
the fresh `error_subclasses` generated demo.

The emitter now uses the canonical HIR control-flow effect to identify a
`try` body that always raises. It does not add a synthetic successful value to
that closure. The outer handler dispatch uses a total `match`. Its impossible
success arm is a compiler invariant, and its error arm keeps the existing
nominal handler chain.

Validation: all 1,108 code-generation tests passed. The
`imported_error_not_catch_all` fixture built and ran natively without E0317 or
E0308. Demo freshness, affected-crate Clippy, format, HIR maintainability,
file-size, and diff checks passed. The union fixtures still stop before this
mechanism at the Item 10F-owned imported-union nominal-path panic.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3454 review comment](https://github.com/sifr-lang/sifr/pull/3454#issuecomment-5380707838).

The create-PR and merge gates each ran once on the exact candidate. Every
preceding guardrail passed. Both gates stopped only at linked delivery A's
stale Rust-interop evidence path. The evidence is in the
[#3454 gate comment](https://github.com/sifr-lang/sifr/pull/3454#issuecomment-5380707920).

Deferred follow-up: Item 10G owns residual propagation for unmatched
conditional handlers. Item 10H owns nested `try/finally` propagation inside a
non-return-capturing `try` closure. Item 10I owns the Item 3 diagnostic-harness
strict-Clippy warning. Opus also suggested consolidating duplicated handler
match construction and strengthening the focused Rust-typing assertion. Those
are optional hardening because native fixture validation covers the diagnostic.

Next action: implement Item 10D.

## Item 10D: Separate Try Declarations from Transported Values

Purpose: Preserve the declaration of a moved binding when later code assigns
it again without transporting an unavailable value through the continuation.

Scope:

- Distinguish declaration liveness from value liveness after a `try`.
- Keep an enclosing declaration when a later scalar assignment needs it.
- Transport only values that are available at the continuation boundary.
- Reject a later field or subscript assignment when it needs the moved value.
- Do not add clones, uninitialized values, or fallback transport paths.

Acceptance criteria:

- A moved binding that is later replaced by a scalar assignment has a valid
  enclosing declaration and is not returned in the continuation tuple.
- A value-dependent assignment through a moved binding is rejected before
  Rust code generation.
- Focused ownership, code-generation, and native-build tests pass.

### Item 10D record

State: complete

PR: [#3456](https://github.com/sifr-lang/sifr/pull/3456)

Base SHA: `65a350884388d76d807bbf5a03dfaeca16840a55`

Candidate SHA: `f87b5f7e03c54d85b0a08b01ecc0130a16379bff`

Merge SHA: `5825cb6bf14b6911f2cc166b4dac85ee67f088e1`

Changed paths: canonical HIR value-liveness analysis, structured Rust typed
declarations, sequential `try` emission, moved-value mutation diagnostics,
and focused compiler and native-run tests.

Validation: all 1,116 code-generation tests and all 1,030 lowering tests
passed. The moved-binding replacement fixture built and ran. Its generated
Rust used `let text: String;`, returned no moved value from the `try` closure,
and passed `rustc -D unused-mut`. Affected-package Clippy, formatting, demo
freshness, HIR maintainability, the file-size guardrail, and diff checks
passed. The typed declaration uses Rust definite assignment. It adds no
runtime placeholder, clone, default value, `MaybeUninit`, or fallback path.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all preceding guardrails and stopped only at linked delivery A's stale
Rust-interop evidence path. Neither gate was repeated. The evidence is in the
[#3456 gate comment](https://github.com/sifr-lang/sifr/pull/3456#issuecomment-5380890047).

Review evidence: the exact-SHA Opus review returned `SATISFIED`. No
remediation review was used. The evidence is in the
[#3456 review comment](https://github.com/sifr-lang/sifr/pull/3456#issuecomment-5380889932).

Deferred follow-up: Item 10E owns the canonical nested-function traversal and
parameter-shadowing rule. Opus confirmed that the current general reference
query can otherwise request a dead declaration for a shadowing parameter.
Linked delivery A retains its stale Rust-interop evidence path.

Next action: implement Item 10E.

## Item 10E: Traverse Nested-function Default Expressions

Purpose: Include definition-time parameter defaults in canonical HIR reference
queries without changing nested-body traversal policy.

Scope:

- Visit nested-function parameter default expressions at definition time.
- Continue to control nested-function body descent through `TraversalConfig`.
- Preserve parameter shadowing behavior.
- Implement the rule in canonical traversal, not in an emitter-local scan.

Acceptance criteria:

- A post-`try` nested-function default that reads the binding keeps it live.
- A shadowing nested parameter does not create a false outer-body reference.
- Traversal configurations preserve their documented body behavior.
- Focused traversal, code-generation, and native-build tests pass.

### Item 10E record

State: complete

PR: [#3458](https://github.com/sifr-lang/sifr/pull/3458)

Base SHA: `218ba6029ff3de4ab75803925e15536c5bdeed0b`

Candidate SHA: `76a19d209d875906b0d7c9751d7216b206a58976`

Merge SHA: `f479d6ad68c265d8b1b846cc7a7aaf0ee4f9a21a`

Changed paths: canonical HIR traversal and reference queries, `try` value
liveness, nested-function code generation, declared-call argument resolution,
function-binding provenance, scope tests, and one native fixture.

Validation: all 1,121 code-generation tests passed. In lowering, 1,032 tests
passed and one test was ignored. The native nested-default fixture built and
ran. Affected-package Clippy passed with warnings denied. Demo freshness, HIR
maintainability, the 3,217-file size guardrail, formatting, and diff checks
passed. Unsupported name and call defaults still produce `SIFR-TYPE-0011`.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all preceding checks and stopped only at linked delivery A's stale
Rust-interop evidence path. Neither gate was repeated. The evidence is in the
[#3458 gate comment](https://github.com/sifr-lang/sifr/pull/3458#issuecomment-5381103242).

Review evidence: the first exact-SHA Opus review returned `BLOCKED`. It found
that explicit keyword values could be replaced by defaults and that tuple
rebinding retained function provenance. The one permitted remediation review
on the final candidate returned `SATISFIED`. No third review ran. The evidence
is in the [first review comment](https://github.com/sifr-lang/sifr/pull/3458#issuecomment-5381102552)
and the [remediation review comment](https://github.com/sifr-lang/sifr/pull/3458#issuecomment-5381102557).

Deferred follow-up: Item 10J owns the new mechanism risk found by the second
review. A lexical nested-function binding can consult an unscoped name-keyed
signature left by a sibling scope. The stricter optional-widening behavior and
the cosmetic vararg diagnostic range are recorded as non-blocking review
observations. Linked delivery A retains its stale Rust-interop evidence path.

Next action: implement Item 10F.

## Item 10F: Complete Imported Union Nominal Paths

Purpose: Give every imported union member one canonical crate-root nominal
path during project code generation.

Scope:

- Register canonical identities for imported union members.
- Preserve distinct identities for equal basenames from different modules.
- Keep registry lookup and emitted ownership consistent.
- Do not add basename lookup, local duplicates, or fallback paths.

Acceptance criteria:

- `nominal_identity_alias_paths` no longer panics during project code generation.
- The registry contains distinct `sifr.csv.Error` and
  `sifr.configparser.Error` identities.
- Every imported union member resolves through its canonical crate-root path.
- Focused registry, code-generation, and native-build tests pass.

### Item 10F record

State: complete

PR: [#3460](https://github.com/sifr-lang/sifr/pull/3460)

Base SHA: `0b45970f5980f94225f2a9e95813deb433960952`

Candidate SHA: `a26b21d10e9da58efce486ce1e2e09297cb9e6ce`

Merge SHA: `732b39b789e37ce02b8984354562a2a5fb53de50`

Changed paths: project-wide stdlib nominal collection, identity-aware built-in
error ownership, focused registry tests, and one native imported-union fixture.

Validation: all 1,123 code-generation tests passed. All eight focused project
nominal tests passed. The same-basename imported-union fixture built and ran.
The original `nominal_identity_alias_paths` fixture completed project code
generation without the imported-union panic. Affected Clippy, formatting, HIR
maintainability, and the 3,218-file size guardrail passed.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all preceding checks and stopped only at linked delivery A's stale
Rust-interop evidence path. Neither gate was repeated. The evidence is in the
[#3460 gate comment](https://github.com/sifr-lang/sifr/pull/3460#issuecomment-5381213927).

Review evidence: the one exact-SHA Opus review returned `SATISFIED`. No
remediation review ran. It confirmed that the fix uses canonical identity and
adds no basename lookup, duplicate nominal, or fallback path. The evidence is
in the [#3460 review comment](https://github.com/sifr-lang/sifr/pull/3460#issuecomment-5381213928).

Deferred follow-up: Item 10K owns the separate failure reached after project
code generation. A user class that inherits a checked-stdlib class emits a
by-value `Into` conversion, but the required `From<Child>` implementation is
not retained. Item 10F does not change inheritance or conversion behavior.
Linked delivery A retains its stale Rust-interop evidence path.

Next action: implement Item 10G.

## Item 10G: Propagate Unmatched Conditional Try Handlers

Purpose: Preserve an error when no conditional handler matches it.

Scope:

- Give a conditional handler chain one explicit residual-error outcome.
- Route the unmatched value through the checked error channel.
- Keep exact nominal and `IOError` kind matching.
- Reject an unhandled error when the enclosing function cannot propagate it.
- Do not swallow the error or add a catch-all handler.

Acceptance criteria:

- An unmatched `IOError` kind propagates or produces a structured compile-time
  diagnostic.
- A matching subclass handler keeps its current behavior.
- A later base handler remains the explicit source-level catch-all.
- Focused type-checking, code-generation, and native-run tests pass.

### Item 10G record

State: complete

PR: [#3462](https://github.com/sifr-lang/sifr/pull/3462)

Base SHA: `d25f897d9ccd1b90fd8c9cda6e2221fd15413df3`

Candidate SHA: `00e67dc56ce27ec6ce2a64ac91216b19afbf87b3`

Merge SHA: `5641e278a4c05941deeedc833adcb4d32a6ccf94`

Changed paths: unmatched-error lowering, function-boundary try state,
conditional handler-chain emission, focused lowering and code-generation
tests, and one native residual-propagation fixture.

Validation: in lowering, 1,035 tests passed and one test was ignored. All
1,126 code-generation tests passed. Existing `IOError` subclass handling and
the expanded single, nested, and union residual fixture ran natively. Affected
Clippy, formatting, HIR maintainability, and the 3,220-file size guardrail
passed. `SIFR-RESULT-0005` remains for a function with no compatible channel.

The create-PR and merge gates each ran once on the candidate SHA. Each passed
all preceding checks and stopped only at linked delivery A's stale
Rust-interop evidence path. Neither gate was repeated. The evidence is in the
[#3462 gate comment](https://github.com/sifr-lang/sifr/pull/3462#issuecomment-5381428950).

Review evidence: the first exact-SHA Opus review returned `BLOCKED`. It found
that branchless carrier members had no residual and that lowering try state
leaked into nested functions. The one permitted remediation review returned
`SATISFIED`. No third review ran. The evidence is in the
[first review comment](https://github.com/sifr-lang/sifr/pull/3462#issuecomment-5381428944)
and the [remediation review comment](https://github.com/sifr-lang/sifr/pull/3462#issuecomment-5381428956).

Deferred follow-up: Item 10L owns user-defined parent-handler dispatch. Item
10M owns the code-generation try-channel stack at nested function boundaries.
These are new mechanisms found by the second review and did not trigger a
third round. Linked delivery A retains its stale Rust-interop evidence path.

Next action: implement Item 10H.

## Item 10H: Preserve Nested Try-finally Error Propagation

Purpose: Keep nested `try/finally` error propagation valid inside every
closure-backed `try` statement.

Scope:

- Track closure error capability separately from return capture.
- Emit the nested `try/finally` error path when the enclosing `try` closure can
  return an error.
- Preserve `finally` execution and the original typed error.
- Do not use an invariant panic for a source-reachable error.

Acceptance criteria:

- A nested `try/finally` that raises inside a non-`Result` function reaches its
  matching outer handler without a runtime panic.
- The `finally` body runs exactly once.
- Generated runtime code contains no reachable invariant-panic path for this
  source shape.
- Focused lowering, code-generation, and native-run tests pass.

## Item 10I: Close Diagnostic Harness Strict Clippy

Purpose: Remove the strict-Clippy defect left by the Item 3 harness migration.

Scope:

- Borrow or copy `FixtureLayout` according to its value semantics.
- Update focused harness tests only when the signature requires it.
- Do not add a Clippy allow or change fixture-path behavior.

Acceptance criteria:

- Workspace Clippy passes with warnings denied after all earlier owners merge.
- Diagnostic rendering produces the same canonical fixture names.
- Focused diagnostic harness tests pass.

## Item 10J: Scope Nested-function Signature Metadata

Purpose: Keep a nested function's callable metadata aligned with its lexical
binding when sibling scopes declare the same name.

Scope:

- Store or resolve nested-function signatures by lexical binding identity.
- Restore or remove nested-function metadata when its scope ends.
- Keep parameter types, defaults, varargs, and calling conventions aligned.
- Do not add a basename fallback or change ordinary `Callable` behavior.

Acceptance criteria:

- Same-named nested functions in sibling scopes use their own signatures.
- Argument validation cannot truncate a mismatched signature and binding.
- Nested defaults, keyword arguments, and varargs use the scoped declaration.
- Focused lowering, code-generation, and native-run tests pass.

## Item 10K: Preserve Checked-stdlib Parent Upcasts

Purpose: Keep the consuming child-to-parent conversion available when the
parent is a checked-stdlib nominal relocated to project scope.

Scope:

- Retain the canonical `From<Child>` implementation for a relocated parent.
- Keep the child field, `Deref`, and consuming upcast on one parent identity.
- Preserve one project-wide parent definition.
- Do not add a clone, basename match, duplicate parent, or fallback conversion.

Acceptance criteria:

- Passing a user child to an owned checked-stdlib parent parameter builds.
- The conversion consumes the child and returns its embedded parent value.
- `nominal_identity_alias_paths` builds and runs through its existing checks.
- Focused inheritance, project-code-generation, and native-run tests pass.

## Item 10L: Match User-defined Parent Error Handlers

Purpose: Make generated handler dispatch follow the same user-defined error
ancestry that lowering uses for coverage.

Scope:

- Match a child error in a handler for its declared user-defined parent.
- Preserve exact nominal identity for unrelated same-basename errors.
- Convert the child to the handler binding type through the checked ancestry.
- Do not add basename matching or make a parent handler a global catch-all.

Acceptance criteria:

- `except BaseError` runs for a raised `ChildError(BaseError)`.
- An unrelated nominal error does not match the parent handler.
- Residual propagation remains available after the parent-handler chain.
- Focused lowering, code-generation, and native-run tests pass.

## Item 10M: Isolate Nested-function Try-channel Codegen State

Purpose: Make each generated nested function own its error-channel context.

Scope:

- Save and clear active try-closure error stacks at a function boundary.
- Restore the enclosing stacks after nested-function emission.
- Route a nested function residual through its own `Result` error type.
- Do not convert the residual to an enclosing closure's carrier.

Acceptance criteria:

- A nested `Result[_, E1]` function inside an `E2` try uses `E1`.
- Enclosing try emission resumes with its original carrier after the function.
- Nested async and ordinary functions preserve the same isolation rule.
- Focused code-generation and native-run tests pass.

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

- Items 1 through 10I are merged.
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
| Residual nominal registry | Focused direct-class, registry consistency, and `uuid_and_datetime` demo checks |
| `try` continuation ownership | Focused continuation-state tests, config-parser fixtures, and TLS loopback fixtures |
| Total `try` returns | Focused control-flow tests and imported and union error fixtures |
| Try declaration liveness | Focused scalar replacement, value-dependent target, and native-build tests |
| Nested-function defaults | Canonical traversal, shadowing, code-generation, and native-build tests |
| Imported union nominal paths | Focused registry, project code-generation, and nominal-identity fixtures |
| Conditional handler residuals | Focused error-effect, exact-handler, and native propagation tests |
| Nested try-finally propagation | Focused nested cleanup, typed-error, and native-run tests |
| Diagnostic harness Clippy | Focused harness tests and workspace Clippy with warnings denied |
| Nested-function signature scope | Focused sibling-scope, default, vararg, and native-call tests |
| Checked-stdlib parent upcast | Focused inheritance, relocation, project-build, and native-run tests |
| User-error parent handler | Focused ancestry, unrelated nominal, residual, and native-run tests |
| Nested-function try-channel state | Focused sync, async, carrier restoration, and native-run tests |
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

Current state: Items 0 through 10G are complete and recorded.

Next action: implement Item 10H.

# Ad hoc issue: Schema-first SQL platform review follow-ups

Status: active, non-blocking

Owner: SQL compiler, schema tools, and verification

## P0 execution and custody reconciliation (2026-09-23)

This is the current execution authority. The 2026-09-07 orchestration below is
retained as a historical plan and failure record. Its pending external blockers,
checkout path, proposed delivery route, and per-item broad-gate rule are
superseded here. No SQL implementation or validation pass is claimed by this
documentation update. The phase remains open until all implementation items,
the final integration gate, and the docs-only closer finish.

The source handoff was `sql-phase-handoff-20260923/detailed-phase.md` (SHA-256
`2f5c31517e9556ed0619f00418f62b4ed3832170bbbb2ef5aa8e4d7483c52dac`)
and its read-only investigation (SHA-256
`ef52d4778d828a56d76180b0f868a7567921e729785ca1a8fb4025206d7c7a19`).
The latter is an audit and recommendation, not a qualification receipt. P0
checked the repository at main `3db97f2f7e354145224a582018cd2a978593a356`
without running compiler, database, Cargo, or component tests.

### Verified delivery and remaining dependency state

| Historical entry | Current disposition at the P0 base |
| --- | --- |
| SQL coverage registry and merge membership, #3749/#3750 | [PR #3751](https://github.com/sifr-lang/sifr/pull/3751) is in main as merge `0b97b3a3f1dd3f93bc724f75e1e40240f15ff942`. The earlier B16 failure and pending wording below remain historical. Do not repeat this registration. |
| SQL `bigint` conversion, #3723 | [PR #3727](https://github.com/sifr-lang/sifr/pull/3727) is in main as merge `a216019057fbb05ccfdc8c846c20ee3ecc7a639d`; the owning issue records closure. It is not an unmerged SQL prerequisite. |
| Item 8 release waiver and approval policy | [PR #3827](https://github.com/sifr-lang/sifr/pull/3827) is in main as merge `33639f4ee3b7079ec4da889834cae0d55d763786`. The [release-policy owner](ad-hoc-distinct-release-reviewer-restoration.md) records its passing integration gate and permanent solo-maintainer policy. Item 8 is resolved for SQL development; actual publication still needs approval of its exact protected run. Preserve its historical expiry failure. |
| Item 9 builtin-registration Clippy failure | Correcting commit `16c100d1b12dd92bb6bc8eb6707d819402299718` is an ancestor of main through PR #3827. The old #3694/#3717 route is historical. Item 9 no longer blocks restarting Item 1, but its former failed check is not a current SQL pass. |
| Four-file package 420-line guard | Responsibility split `aa894c188a955acd8eb27ecb0cc55ffed2a7908d` is an ancestor of main. The old pending Turing handoff is historical. Retain the existing guardrail acceptance; do not repeat the package implementation. |

The selected create-PR and merge profiles currently list
`cargo-test-sifr-smoke`/`e2e-pass` and `cargo-test-sifr-full`/`e2e-pass`,
respectively, without `cargo-clippy-workspace`. Workspace Clippy remains an
explicit named acceptance check for Items 1–3 and 7. Do not report a broad
profile pass as evidence of that command. The advisory CI Clippy job is also
not a substitute for the named local check. The DX.2 all-targets MySQL test
lint observation at the end of this record is distinct from Item 9's resolved
production diagnostic; retain it for the runtime/test owner.

### Item 1 source and evidence custody

The reported original `/private/tmp/sifr-sql-item1.T06Xmx/codebase` checkout,
`terminal.json`, `evidence.json`, and raw test logs are absent on this host.
The source is retained in `sql-phase-handoff-20260923/item1.bundle`, SHA-256
`efcc7b3f47158f79428309dd14cad19cb2f2293a267e8fb7c3f71781ba24fb8b`.
`git bundle verify` passed against the P0 repository, and the bundle's head was
fetched and read back as `refs/archive/sql-item1-20260923` at
`e7a757bd3e1754a41e24cab35e056f5c27304f15`. It contains the recorded
implementation candidate `4f53a1ce39f612e5e8c26b8802e5ef798c07d026`
based on `74480c77015881fe48c7f1f3c76b4d8ea5876511`. This verifies source
recoverability, not current correctness or historical execution evidence.

The old child reported PostgreSQL 13–18 sequence/component passes and a
workspace Clippy failure; the raw payloads for those results are unavailable
here. Keep their recorded identities and outcomes below as historical reports.
Do not recreate a receipt or reuse a missing log as a pass. A new Item 1 owner
must port the bounded source to fresh main, rebuild affected components from
current producer inputs, and run its named checks and focused regressions on
the new candidate. The original bundled PR #3647 is a comparison source only;
its unfinished branch and old component binaries are not merge inputs.

### Delivery order and implementation boundaries

One implementer owns one item and one isolated worktree at a time. Complete
Items 1, 2, 3, 4, 5, and 7 in that order; Item 7 remains separate from sequence
work. Register and deliver the additional bounded items below after Item 7,
before the final integration qualifier and Item 6 closer. New findings from
review enter a later item rather than silently widening the active item.

- **Item 1 — sequences:** retain its existing scope and named checks. Preserve
  sequence parameters, ownership edges, nextval defaults, and identity-column
  semantics distinctly. Choose an explicit SERIAL capability boundary. Test
  duplicate CREATE, supported no-op/replacement, multi-document updates, mixed
  ALTER, owner change/removal, six-major DDL/live parity, and actual rebuilt
  component execution. Do not broadly discard extension-owned objects. Any
  changed schema fingerprint requires current snapshot/migration revalidation.
- **Item 2 — generated identities and values:** generated source must lower and
  type-check through the ordinary frontend; a missing-import mutation must fail
  there. Use one closed mapping across source import, frontend identity,
  component hole descriptor, provider compatibility, result type, and runtime
  codec. Cover supported values, aliases, nullable/container forms, and each
  provider that claims them. A declaration without executable value semantics
  is insufficient. Rebuild every component in the changed producer-input
  closure, which can include shared inputs beyond one dialect crate.
- **Item 3 — discovery:** resolve configured profile identity, import aliases,
  shadowing, and unrelated decorators through shared CLI/editor authority.
  Check initial and edited-document transitions, and preserve targeted
  diagnostic identity and source location. Standalone SQL calls also need
  discovery; Item 10 owns their end-to-end execution proof.
- **Item 4 — linked qualification:** identify the three selected SQL tool
  executables from Cargo artifacts and compare their bytes across clean A,
  unchanged rebuild A, and independent clean B under locked offline inputs.
  Self-tests must mutate the actual comparison and resource-declaration
  decisions. State cross-target limits and the exact reproducibility claim.
  #3659 closes only after its wrong-resource-class mutation passes. The broader
  #3654/#3660 scheduler-policy questions stay with their owners.
- **Item 5 — coverage:** audit the seven runtime/tool packages by selected
  command and actual executed tests. Check MySQL runtime lib tests omitted by
  the runtime_types-only adapter. Record the existing separate MySQL live
  matrix with `--include-ignored`; offline profiles do not run it.
- **Item 7 — views:** establish DDL/live equivalence using resolved relation
  and column identities across supported PostgreSQL majors. Preserve genuine
  differences from schemas, aliases, ambiguity, ordering, and replacement.
  If this requires a broader resolver, return needs-new-scope.

Additional implementation items registered before work resumes:

- **Item 10 — ordinary source to executable SQL:** prove that a real project
  and ordinary `.sifr` application source, using an actual provider component,
  build a native binary that binds a value, fetches a row, and uses its inferred
  type. Include `@app.query` return-path identity and ordinary `app.sql(...)`
  outside a decorator: invalid SQL fails compilation; a valid query keeps its
  profile/result contract through returning, importing, and execution. Tests
  must not manually bridge `ProviderAnalysis`, `HirSqlExecution`, or
  `RuntimeQueryTemplate`. Scope the current signature-registry/erasure/codegen
  and public connection/pool surface gaps before implementation; if the repair
  crosses this bounded integration mechanism, return needs-new-scope.
- **Item 11 — runtime value preservation:** separately own binary/text decode
  and encoding-error classification. Test empty, valid-UTF-8, embedded-zero,
  and invalid-UTF-8 bytes, asserting representations as well as success.
  Item 5 may document the gap but does not absorb a codec redesign.
- **Item 12 — provider diagnostic parity:** preserve provider diagnostic code,
  severity, and source span through CLI and editor for the same SQL source.
  Unsupported component hole types must yield an equivalent targeted error,
  not silent editor omission. Item 3 owns the missing-import diagnostic; this
  item owns the broader provider-reporting boundary.

The `sqlite-runtime-probe` reviewer observation remains a question for the
coverage owner: its current SQLite-only classification is accurate, and no
missing MySQL/PostgreSQL implementation has been established. Do not turn it
into an automatic provider-generalization requirement.

### Qualification and closer policy

Intermediate implementers run the item-named tests and affected focused
regressions, then a scoped exact-SHA Opus review and merge. They skip create-PR
and full merge gates under the approved prospective phase policy. One final
integration qualifier runs the full merge profile on the final merged-work
candidate, repairs the first in-scope failure, reruns its affected checks, and
repeats the full gate until it passes. Out-of-scope blockers go to their owning
issue. Keep failed evidence and do not repeat an unchanged failed gate.
Selected tests use exact crate/suite/case selection and fail fast by default;
reuse compatible builds, fixture preparation, and warm owned targets. Check
disk pressure and ownership before any cache cleanup. The same SHA only permits
evidence reuse when configuration and validation inputs are also unchanged.

Item 6 remains the last, docs-only, whole-phase closer. It audits every
original acceptance criterion and all later items against candidate-specific
receipts, gives the one whole-phase Opus review, and returns
needs-implementation for any remaining mechanism gap. Only then archive this
record, update the roadmap, and close superseded draft #3647 after accounting
for all intended changes. P0 and later record-only edits need documentation
checks, not another Sifr gate or external review.

## Historical orchestration and evidence (2026-09-07; superseded by P0 above)

This section was the execution authority on 2026-09-07. Its blocker and
dispatch states are historical after the P0 reconciliation above. The parent
was an orchestrator only: it could
register items and record terminal handoffs, but does not implement, test, review,
or run Sifr gates. One live implementer at a time; each child owns one item only.
The user requires one exact-SHA Opus review plus at most one remediation review.
A new mechanism defect on the second review becomes a later item, not another
review round. Compiler, lockfile, fixture, or workflow changes require one merge
gate on the final reviewed SHA; skip create-pr when merging that SHA in the same
session. Other changes require no Sifr gates. Preserve failed evidence and do not
repeat a gate. A blocker must return to the orchestrator for later-item recording.

### Checkout and delivery boundary

- Reference implementation: local commit
  `f4e0ee11a9fe802620139ef799b6ee99c4882c79`, branch
  `codex/sql-platform-review-followups`, original draft PR #3647.
- Reference main: `156157242b0995c01c4fff03575624b5c471c0d8`.
- Each implementer creates an owned isolated worktree/branch from freshly fetched
  main. Do not mutate the orchestrator checkout/index, another worker's checkout,
  or the original bundled PR. Read this complete phase document and carry its
  current execution section into your branch's phase record if not yet on main.
- Port only the reference changes owned by the assigned item, then correct that
  item's findings. Do not merge the bundled reference branch: it contains other
  unfinished mechanisms. Preserve unrelated upstream changes.
- Generated artifacts, registry entries, and documentation required by the
  assigned mechanism belong to that item's complete delivery. In particular,
  regenerate affected compiler components with their real content hashes when
  source inputs change; never update a fingerprint without rebuilding its bytes.
- After merge, record the PR, exact reviewed candidate, actual merge SHA, named
  validation, review evidence, and deferred work. Return the record location and
  terminal status. The parent reconciles records before dispatching another child.
- The previous approvals of `4e329d2b0` and `28e0e39cf` do not approve new code.
  The previous failed gates remain failed; each newly scoped item has its own
  single final-candidate gate under the current user instruction.
- Former prerequisites #3648 and #3655 are closed. SQL registry/membership
  #3749/#3750 merged through #3751. Do not redo them. New targets still need
  their own classifications. Do not take over unrelated emitted-Rust work.

### Item 1 — PostgreSQL sequence semantics and executable components

Status: blocked on external Item 9; Item 8 also blocks the eventual merge gate.
Dependencies: no implementation predecessor; external prerequisites recorded below.

Own the sequence changes from the reference: PostgreSQL AST/raw adapter/catalog
sequence modules; live catalog SQL and normalization; sequence tests and live
schema matrix. Preserve explicit owned sequences and exclude only internal
identity sequences. Correct mixed ALTER options being silently discarded,
ownership reassignment retaining old dependency edges, and modifications of
existing objects being dropped across schema documents and migration reflection.
Make OWNED BY NONE explicitly supported or rejected without false reflection.
Cover DDL/live sequence parity for the supported server majors, not only 18.
Rebuild all six affected PostgreSQL components and record source/content hashes;
exercise sequence/schema behavior through the actual capability-free artifacts.
Only port annotation-related changes if strictly necessary for sequence fixtures;
the general annotation mechanism belongs to Item 2.

Named validation (after implementation is complete):
- `python3 verification/areas/sql_platform/tools/check_postgresql_compiler.py`
- `python3 verification/areas/sql_platform/tools/check_postgresql_compiler.py --self-test`
- `python3 verification/areas/sql_platform/tools/run_postgresql_parser_matrix.py`
- `cargo test --locked -p sifr_sql_postgresql_tools`
- `python3 verification/areas/sql_platform/tools/run_postgresql_schema_tool_matrix.py`
- Add regressions within the above executed targets for mixed ALTER options,
  owner replacement/removal policy, multi-document updates, migration reflection,
  and actual component execution of sequence schema requests.
- Common checks below; then the single applicable merge gate.

### Item 2 — Canonical generated annotation and SQL value identities

Status: blocked by unmerged Item 1. Dependencies: Item 1 merged.

Own the reference generated-profile import/type mapping, PostgreSQL type-name
mapping and generated-profile tests, associated driver profile fixtures, and
new SQL declarations. Resolve the mismatch between generated `sifr.sql`
OffsetTime/Instant/IPAddress/IPNetwork and query lowering's datetime/ipaddress
identities. Standard SQL values must retain canonical contracts when used in
interpolations rather than becoming arbitrary Custom codecs. Use one closed
mapping across source imports, generated annotations, and frontend type
conversion. Preserve explicit Enum imports. Prove generated date/time/timestamp,
UUID/JSON/network/MAC values work as both query inputs and outputs, including
nullable/container forms. Register new test targets in the coverage registry,
regenerate the public API reference, correct the SqlEnum documentation, and
rebuild any affected component artifacts as part of this item.

Named validation:
- `cargo test --locked -p sifr_sql_contract`
- `cargo test --locked -p sifr_frontend --test sql_queries`
- `cargo test --locked -p sifr_driver sql_profiles_tests`
- `cargo test --locked -p sifr_driver compiled_stdlib_exports_match_public_reference`
- `python3 verification/areas/sql_platform/tools/run_postgresql_parser_matrix.py`
- `python3 verification/areas/sql_platform/tools/check_postgresql_compiler.py`
- `PYTHONPATH=verification/runner python3 verification/areas/coverage_matrix/checks/coverage_matrix_readiness.py`
- Add round-trip canonical value and missing-import mutation regressions inside
  these targets; use generated-profile fixture coverage through the driver.
- Common checks below; then the single applicable merge gate.

### Item 3 — Early missing-profile-import diagnostic and editor consistency

Status: blocked by unmerged Item 2. Dependencies: Item 2 merged.

Own the reference missing-import discovery, driver diagnostic, registry entry,
SIFR-SQL-0009 page and diagnostic navigation. Diagnose the missing profile import
before ordinary frontend lowering hides it behind an undefined `app` error for
`@app.query` functions returning `app.sql(t"...")`. Preserve unrelated decorators
and imported aliases. Check CLI/editor behavior through their shared authority;
do not introduce separate conflicting import-discovery logic. Complete diagnostic
catalog, baseline coverage, renderer fixture/metadata, and documented correction.

Named validation:
- `cargo test --locked -p sifr_driver sql_profiles_tests`
- `cargo test --locked -p sifr_frontend --test sql_queries`
- `cargo test --locked -p sifr_analysis`
- `cargo test --locked -p sifr_lsp`
- `cargo test --locked -p sifr_diagnostics`
- `python3 verification/areas/diagnostics/checks/code_coverage.py`
- `python3 verification/areas/diagnostics/checks/code_baseline_coverage.py`
- `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines`
- Add missing-import regressions with actual app.sql usage, unrelated decorators,
  imported aliases, and editor diagnostics within these executed targets.
- Common checks below; then the single applicable merge gate.

### Item 4 — Linked build qualification and consistent resource declarations

Status: blocked by unmerged Item 3. Dependencies: Item 3 merged.

Own reference build-qualification runner, linked hash mutation, integrated record
and checker, SQL manifest/runner commands, and host-tool documentation. Preserve
native clean/reused/clean locked offline SHA-256 comparisons for the three SQL
tool executables and truthful non-linking cross-target results. Reconcile the
reviewed default-local/long-running declaration with main's exact-default-local
checker. Address #3659 with an executable wrong-resource-class mutation. Keep
#3654/#3660 policy explicit: resource declarations are not a promise of scheduler
isolation; private target ownership provides build isolation. Do not implement a
new central scheduler. Document exactly which binaries and targets are qualified.

Named validation:
- `python3 verification/areas/sql_platform/tools/check_contracts.py`
- `python3 verification/areas/sql_platform/tools/check_contracts.py --self-test`
- `python3 verification/areas/sql_platform/tools/check_integrated_qualification.py`
- `python3 verification/areas/sql_platform/tools/check_integrated_qualification.py --self-test`
- `python3 verification/areas/sql_platform/tools/run_sql_build_qualification.py --self-test`
- `python3 verification/areas/sql_platform/tools/run_sql_build_qualification.py`
- `python3 verification/areas/sql_platform/tools/run_sql_build_qualification.py --target wasm32-wasip2`
- `uv run --project verification --locked python -m sifr_verify --self-test`
- `git diff --check`; `python3 scripts/check_file_size_guardrails.py`.
- Apply the user's file-category gate rule; do not run gates for helper/metadata/
  documentation changes alone.

### Item 5 — SQL runtime/tooling execution coverage audit

Status: blocked by unmerged Item 4. Dependencies: Item 4 merged.

Resolve the bounded audit in #3752 for the seven named SQL runtime/tool packages.
Trace actual selected adapter and crate-test execution before claiming missing
coverage. Tool packages already have full schema-tools crate execution, PostgreSQL
runtime selects lib and runtime_types, and SQLite runtime selects the full crate.
Assess MySQL runtime source-unit tests currently omitted by its runtime_types-only
adapter. Make necessary bounded membership/adapter changes and record the actual
coverage matrix. Preserve ignored live-server policy; no claim that offline
profiles execute ignored tests. Do not broaden into general scheduler work.

Named validation:
- `cargo test --locked -p sifr_sql_mysql_runtime -p sifr_sql_postgresql_runtime -p sifr_sql_sqlite_runtime -p sifr_sql_mysql_tools -p sifr_sql_postgresql_tools -p sifr_sql_sqlite_tools -p sifr_sql_tool`
- `uv run --project verification --locked python -m sifr_verify profiles check`
- `uv run --project verification --locked python -m sifr_verify profiles plan --profile merge`
- `PYTHONPATH=verification/runner python3 verification/areas/coverage_matrix/checks/coverage_matrix_readiness.py`
- `git diff --check`; `python3 scripts/check_file_size_guardrails.py`.
- Apply the user's file-category gate rule; metadata/helpers/docs alone need no gates.

### Item 6 — Docs-only phase closer

Status: blocked; closer not dispatched. Dependencies: Items 1–5 and every later
implementation item merged.

Only this child performs the whole-phase Opus review. Change documentation only.
Audit every original acceptance criterion against merged candidate-specific
evidence, record any new mechanism gap as a later implementation item, and stop
without claiming closure if any remain. Otherwise reconcile this record and
superseded blocker text, archive it, update the roadmap status/link, and close the
superseded original bundled PR #3647 with links to the actual delivery PRs after
confirming all its intended changes are accounted for. Preserve historical failures.
Do not merge the stale bundled branch. No compiler changes, tests, or Sifr gates.

Named checks: `git diff --check` and read-only existence/anchor checks for links
changed by the closure. Reuse all merged implementation validation evidence.

### Common checks for Items 1–3

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`
- `git diff --check`
- Before any long Cargo operation inspect free disk and owned target size; obey
  AGENTS.md storage rules. No other worktree cleanup or mutation is authorized.

### Later items and terminal handoffs

The orchestrator records child-returned blockers and new mechanism findings here
without implementing them. A blocked predecessor prevents dependent implementation/
closure dispatch until a later prerequisite resolves it. Later implementation
items execute after Items 1–5 and before the docs-only Item 6 closer.

### Item 7 — PostgreSQL view normalization parity

Status: blocked by unmerged Item 1, later mechanism. Dependencies: Item 1 merged. Delivery order:
after Item 5 and before Item 6.

Reported by Item 1 child, not implemented by it: broadening the original PG18
whole-schema parity check exposed a PostgreSQL 13 view canonicalization difference.
DDL provider-query column references were unqualified id/name/score; live
pg_get_viewdef references were qualified parity_users.id/name/score. The child
reported no sequence differences and preserved its failed first live run at
`/private/tmp/sifr-sql-item1.T06Xmx/live-schema-matrix.log`. Item 1 will preserve
the original PG18 whole-schema parity and check its sequence/table scope for all
six majors. This does not authorize view changes inside Item 1.

Establish canonical equivalence of DDL and introspected view references using
resolved identities rather than deleting qualifiers textually. Preserve meaningful
relation/column distinctions, including ambiguity and schema qualification.
Qualify equivalent and genuinely different view definitions through the supported
PostgreSQL matrix. If this requires a broader semantic mechanism than this bounded
view-normalization item, return needs-new-scope without widening it.

Named validation:
- `cargo test --locked -p sifr_sql_postgresql_tools`
- `python3 verification/areas/sql_platform/tools/run_postgresql_parser_matrix.py`
- `python3 verification/areas/sql_platform/tools/run_postgresql_schema_tool_matrix.py`
- Add view-equivalence and non-equivalence regressions within these executed targets.
- Common checks for Items 1–3 also apply to Item 7; one applicable merge gate.

### Item 8 — External release-waiver gate prerequisite (E1)

Status: blocked, externally owned; not ready for SQL implementer dispatch.
Owner record: `plans/issues/active/ad-hoc-distinct-release-reviewer-restoration.md`.
Current sole implementation owner: dependency orchestration task
`01a07d98-9d00-7040-9e62-ea8193a8befa`, through one separately registered child.
Coordination task: `01a07d84-7c62-77f0-b7c7-ecf310829a11`.
Applies before any SQL merge gate, not before Opus review. No E2/PR #3717
dependency is asserted for SQL; its performance-sampling failure is not a SQL defect.

Coordinator verified the prerequisite read-only in the Item 1 checkout and live
main `8c2d03f4bc21556967df4f6b0fba1ea4b3d2bf24`: merge profile selects
distribution_release/epoch-bootstrap, whose schema_bootstrap_selftest.py invokes
validate_repository_waiver. approval_waiver_selftest.py unconditionally requires
the repository waiver to be valid at the current time. The canonical
`plans/releases/single-maintainer-approval-waiver.json` expired on
2026-08-27T00:00:00Z. Relevant files were reported byte-identical to main.

Resume condition: owning release work delivers qualified historical-validation
versus new-publication behavior while preserving immutable provenance and actual
approval requirements. Adding an environment reviewer alone does not repair the
offline self-test. The initial report had no implementer and a pending reviewer
choice; that authority state is superseded by the update below. Do not alter
governance in a SQL child or consume a SQL merge gate to reproduce this failure.
Validation and delivery are owned by the external phase; no SQL tests assigned.

Authority update (2026-09-08): coordinator verified the actual user message in the
dependency task at 2026-09-07T23:30:16.519Z authorizing "Permanent solo policy: yes"
and source control as historical backup. A distinct second human is no longer a
SQL prerequisite. The assigned owner will implement explicit GitHub approval by
yaseralnajjar, admin bypass disabled, solo self-approval, historical waiver
preservation without authority for new publication, and corrected offline
historical validation. Dependency Item 64 separately verifies Git backup; do not
fabricate or rebind receipts. Item 8 remains technically blocked until qualified
delivery, and Item 9 remains pending. No SQL restart or duplicate governance work.

### Item 9 — External generated-nominal Clippy prerequisite

Status: blocked, externally owned; not ready for SQL implementer dispatch.
Owner: `plans/issues/active/ad-hoc-emitted-rust-excellence.md`, incorporated
Item 12C within Item 12B, as reported by the Item 1 child.

Item 1's named `cargo clippy --workspace -- -D warnings` terminated naturally
with exit 101 on candidate `4f53a1ce39f612e5e8c26b8802e5ef798c07d026`.
Sole reported diagnostic: `clippy::expect_used` in
`crates/sifr_codegen/src/project_stdlib_nominals.rs:45–46`,
`ProjectNominalRegistry::register_builtin`. Child verified the source had no
diff from its original main base; the owning phase retains a repair not yet
delivered to main. Parent did not run Clippy or inspect/modify this implementation.

Evidence: `/private/tmp/sifr-sql-item1.T06Xmx/clippy.log`, SHA-256
`d13d32cf4a9190517172cd86c1cb204d660a641c14ae550c56f9202ce2be6b75`.
Child confirmed no owned validation processes remain and released the host window
to the coordinator. Initial review, remediation review, and the one merge gate
remain unused. Resume condition: owning codegen work delivers the correction,
then a separately dispatched SQL continuation refreshes its candidate and reruns
the affected named Clippy check. Do not repeat unaffected functional evidence.
No codegen repair is authorized inside the SQL implementation item.

Coordinator supplied exact delivery provenance: correction
`3f422b01633d23c2bc8d8ce8ca59057c6e56adea` (Carry validated builtin identities
through Item12B registration), owning PR #3694 retained head
`8e532f15895e7005fae8c658739ba3c3a6818c18`. Both are ancestors of frozen
integration PR #3717 head `98480c78587d6cbd99a7079d10c71825360bd468`.
The corrected registry consumes BuiltinError and its identity directly instead
of a partial lookup followed by expect. Delivery through #3694 → #3717 remains
unmerged. Its historical focused/codegen/Clippy passes are not current SQL or
workspace qualification. Integration's sampling failure is external issue #3776;
the SQL dependency is specifically delivery of the builtin-registration repair,
not an assertion that the benchmark instability is a SQL defect.

### Item 1 terminal handoff and dispatch audit (2026-09-08)

Child `/root/sql_item_1` returned **BLOCKED**, lifecycle completed. No PR, push,
review, or merge occurred. This environment exposes no close_agent operation;
the completed child is retired from dispatch and will not be resumed. A future
continuation must use a new child after the prerequisite is delivered.

- Owned checkout: `/private/tmp/sifr-sql-item1.T06Xmx/codebase`.
- Clean local branch: `codex/sql-postgresql-sequences-item1`.
- Main base: `74480c77015881fe48c7f1f3c76b4d8ea5876511`.
- Implementation candidate: `4f53a1ce39f612e5e8c26b8802e5ef798c07d026`.
- Final record commit: `e7a757bd3e1754a41e24cab35e056f5c27304f15`.
- Terminal: `/private/tmp/sifr-sql-item1.T06Xmx/terminal.json`.
- Evidence index: `/private/tmp/sifr-sql-item1.T06Xmx/evidence.json`.
- Child reports PASS for native PostgreSQL 13–18 tests, actual query/schema
  execution through all six rebuilt capability-free artifacts, scoped live
  sequence/table parity on all six majors with original PG18 whole-schema parity
  retained, schema-tools tests, component integrity, fmt/HIR/driver/size/diff.
- Initial broadened-view live FAIL is preserved separately; Item 7 owns the
  discovered mechanism. Artifact execution ran on a contended host; its 933-second
  observation is not performance evidence.
- Mandatory workspace Clippy FAIL and external owners are recorded in Items 9/8.
  Initial Opus reviews 0, remediation reviews 0, create-PR gates 0, merge gates 0.
  Host window released; no validation processes remain.

Dispatch result: one child spawned, one terminal blocked return, no live
implementer. Items 2–5 and 7 are dependency-blocked, not attempted; closer 6 is
blocked and was not spawned. External Items 8/9 retain their existing owners and
are not ready for SQL dispatch. There is no ready item. The phase remains open;
the all-merged condition and all acceptance checkboxes remain unsatisfied.

Next ready dispatch after external Item 9 delivery: a fresh Item 1 continuation
must take over the preserved isolated candidate without duplicating implementation,
refresh to the delivered repair, reuse only unchanged evidence, and run its failed
named Clippy check. Review follows named-check success. Resolve external Item 8
and coordinate a host window before spending the still-unused merge gate.
The parent performed orchestration/documentation only during this dispatch run;
all implementation, validation, and artifact work was done by the child.

### Package guardrail acceptance edge — owner disposition (2026-09-09)

Status: implemented by existing compiler group 1 delivery worker Turing with
reported focused checks passing; integration review/gate/main delivery pending.
No SQL worker is dispatched. The initial disposition and
recommendation below remain historical evidence; the delivery assignment below
supersedes their proposed separate-worker topology and Police routing.

Existing criterion: this issue requires passing file-size checks and applicable
repository gates. The existing package-specific guard
`verification/areas/package_management/tools/check_package_manager_guardrails.py`
sets `crates/sifr_package/src/**/*.rs` to 420 lines. The general 900-line check
does not replace this stricter policy. The compiler worker reports its original
package guard failed on four inherited paths, outside the group 1 delta:

| Path under `crates/sifr_package/src/` | Lines | Existing limit |
| --- | ---: | ---: |
| `cargo_backend_integration_tests.rs` | 538 | 420 |
| `graph/derive.rs` | 489 | 420 |
| `host_tools.rs` | 890 | 420 |
| `manifest/sql_profiles.rs` | 874 | 420 |

Reported candidate: `110c7ab24a991dd47329eaf2ba2f1a1cdeab7d0d`.
Evidence: `/private/tmp/sifr-group1-qualification-repair.TlAeDG/package-guard-inherited-blocker.json`,
SHA-256 `3bc2203ebb1c8fefb6c4140fd8a74000dedf03a7f8ddb05c0497dbd5b65282e6`;
reported original terminal SHA-256
`a6aadd3a75de11659aa0f0f3961167a267b4efb7e48a9bf4e9e2e4203200a6c0`.
The worker authenticated the guard and four files as unchanged from main.
The SQL orchestrator freshly fetched main
`4b4cc339964baeeb6641e57dc669fef700a5fa24`, independently matched all four source
hashes and line counts to that evidence, and confirmed the 420-line policy.
No guard, test, review, or gate was rerun by the orchestrator. Main still contains
the builtin-registration `expect`; no SQL restart condition has been met.

Minimum correction: responsibility-based module splits of these four files,
preserving behavior, all tests/assertions, public API, Cargo graph, and the
420-line policy. Existing source-reading SQL checks must continue to check the
same behavior if declarations move. No threshold increase, exclusion, optional
cleanup, new feature, or unrelated compiler repair belongs to this correction.
No existing item or qualified candidate in the inspected active records supplies
these splits. This record does not assert a new implementation pass or broaden
the frozen phase scope; scheduling this existing acceptance repair requires
explicit coordinator disposition before a fresh bounded worker is dispatched.

Dependency disposition (same day): the four-file responsibility-only repair has
no source dependency on Item 1's sequence/component changes or Item 9's codegen
builtin correction. The four files and package manifest match main; Item 1's
candidate changes none of them. Their responsibilities are package graph
derivation, host-tool declaration/lock/isolation, SQL manifest parsing, and Cargo
backend/component tests. `sifr_package` does not directly depend on
`sifr_codegen`. This is source-dependency analysis, not a validation pass.

Recommended order: one fresh worker prepares this bounded existing package
acceptance repair before the SQL sequence, with its own named package/SQL
source-contract checks and preserved review history. Compiler group 1 continues
independent qualification. If the required package delivery gate selects the
still-broken builtin check, do not run it merely to repeat that known failure:
the compiler owner must explicitly compose the package repair and already-owned
builtin correction into one coherent final integration candidate, qualify/review
that exact SHA, and deliver both before SQL Item 1 resumes. Do not assert a
standalone merge pass or change SQL Items 1–5/7 ordering. This bounded repair of
an existing acceptance policy needs no new feature/scope decision from the user;
the integration owner must settle candidate custody and delivery topology before
dispatch. No worker is launched by this recommendation.

Coordination correction: Police was redirected by the user to the latest-stable
audit and no longer owns compiler/SQL ordering. Route this disposition to compiler
parent `01a06e86-414a-7e11-9256-1f45bdb5a6c7` and group 1 owner Turing
`01a084f6-dba5-79e2-9733-4938419b11d9`, superseding the Police routing above.

Delivery assignment (recorded 2026-09-10): the compiler parent explicitly assigns
this exact four-file responsibility-only correction to existing delivery worker
Turing under the user's four sequential-worker consolidation. There is no fifth
or parallel package worker. Turing owns implementation exclusively inside its
compiler integration checkout and composes it with the already-owned builtin
correction. This authorization supersedes the earlier separate-worker
recommendation; it does not add a feature or widen the four-file repair.

Dependency mapping: package splits have no source dependency on SQL Item 1 or
the builtin correction; their delivery joins compiler group 1's final integration
candidate because the existing acceptance check blocks that delivery. Group 1
must satisfy actual qualification, exact-final-SHA review and the applicable
single gate, including its other existing blockers, before merge. No separate
premature package merge/gate and no claimed pass from this assignment. Preserve
all behavior, tests/assertions, API, Cargo graph, SQL source-reading checks and
the original 420-line policy. Preserve consumed review/gate evidence.

SQL Item 9 remains pending actual builtin delivery to main; SQL Items 1–5/7 keep
their existing order and the independent closer remains last. SQL's orchestrator
and preserved Item 1 checkouts/indexes are not implementation targets for Turing.
The SQL parent only records this assignment and awaits delivery provenance; it
does not dispatch, implement, test, review, or run Sifr gates for this repair.

Worker qualification update (2026-09-10): compiler owner reports Turing implemented
and pushed candidate `aa894c188a955acd8eb27ecb0cc55ffed2a7908d` exclusively in
its compiler integration root. Evidence index:
`/private/tmp/sifr-group1-package-repair.zmZJH6/package-prerequisite-closure.json`,
SHA-256 `c97241ee28ae3648472009d3ccb472183a51df21fa4c8ad4beb5b575dec5243c`.
The SQL parent read the index and matched its checksum; it did not independently
rerun or review implementation/validation. Worker-reported PASS: original
420-line package guard, all 160 package tests, six driver SQL profile tests, one
actual CLI locked-host-tool/sandbox integration, six original SQL positive/mutation
checks, formatting and general file-size checks. The index records preservation
of 89 original definitions/bodies/literals and all 12 Cargo backend tests, with
behavior, API, assertions, Cargo graph and policy unchanged. The prior failed
guard evidence above remains retained, not rewritten as a pass.

This is focused worker qualification, not integration closure: the index explicitly
retains `package_native_smoke_pending: true`, `whole_group_complete: false`,
`new_reviews: 0`, `new_final_gates: 0`, and `merged: false`. Group 1 still owns its
outstanding performance budget, exact Item 49 registry audit and PostgreSQL guest
provenance acceptance. The owner separately reports exact-candidate 92-case setup
passing; that is not a final gate or main-delivery receipt. No SQL ordering or
restart condition changes, and no additional worker is needed.

## Objective

Resolve the new mechanism findings from the final schema-first SQL platform
remediation review. These findings do not reopen the completed platform phase.

## Scope

- Make every generated profile annotation resolve through an explicit import or
  alias. Cover datetime, UUID, JSON, and network types.
- Diagnose a missing schema-profile import when a decorator names a configured
  profile. Do not capture unrelated decorators that also end in `.query`.
- Keep explicit user-owned PostgreSQL sequences in live schema evidence. Exclude
  only sequences that PostgreSQL creates as an identity implementation detail.
- Prove reproducible linked native SQL artifacts, not only reproducible Cargo
  check plans.

## Acceptance criteria

- [ ] A generated profile with date, time, timestamp, UUID, JSON, IP, network,
  and MAC fields compiles without an undeclared annotation path.
- [ ] Generated imports and type annotations come from one closed mapping. A
  mutation test rejects an annotation whose import is missing.
- [ ] A configured but unimported profile decorator produces one targeted
  diagnostic with an import correction.
- [ ] An unrelated decorator such as `@cache.query` remains outside SQL
  discovery and does not produce the profile-import diagnostic.
- [ ] PostgreSQL live catalog tests retain explicit `CREATE SEQUENCE` and
  `ALTER SEQUENCE ... OWNED BY` objects.
- [ ] PostgreSQL live catalog tests exclude only implementation-owned identity
  sequences and preserve DDL-versus-introspection parity.
- [ ] Native build qualification links supported SQL artifacts twice from clean,
  locked, offline inputs and compares stable content hashes.
- [ ] Cross-target limitations are explicit. The qualification does not claim
  byte reproducibility for a target that it cannot link locally.
- [ ] Focused compiler, schema-tool, build-qualification, mutation, formatting,
  lint, HIR, and file-size checks pass.
- [ ] One exact-candidate external review and the applicable repository gates
  pass when this issue is selected for implementation.

## Source evidence

- Final implementation: [PR #3645](https://github.com/sifr-lang/sifr/pull/3645).
- The Milestone 18 remediation review returned `SATISFIED` for the four original
  blockers on `c0c6ae255fc605fc58a24d93a15d5a08b8126121` and reported these four
  findings as new follow-up work.
- The archived phase record contains the complete validation, review, gate, and
  merge evidence.


## Coverage registry blocker observed during naming cleanup (2026-09-05)

### SQL registry delivery registration (2026-09-07)

Current delivery owner: **12K-B17 / #3750** continues the complete B16 record
in the existing PR #3751. Its bounded registration and seven named commands
are at the top of `ad-hoc-emitted-rust-excellence.md`. The two new full-mode,
blocking merge suite memberships retain compiler classification and execute
through the existing runner. Combined registry/membership approval is required.
This supersedes the historical blocked handoff below only after qualified merge;
all B16 failed evidence and other SQL follow-up acceptance remain preserved.

Terminal: B16 #3749 is **blocked, not merged**, preserved in draft
[#3751](https://github.com/sifr-lang/sifr/pull/3751), candidate
`a7ea5b8106068ee9394d82dd46e8e95bc1263a36`, main base
`06ea86334b72f49f5aab250a64498ee955ec9331`. All original23 registry diagnostics
are resolved against all37 packages/117 targets, but one exact-SHA readiness
run is3/4PASS: strict registry now identifies missing merge crate-test membership
for `sifr_sql_mysql` and `sifr_sql_sqlite`. Profile19/negative26/taxonomy PASS;
file-size3759 and diff PASS. No review, gate or merge was attempted.

Later owner **12K-B17 / [#3750](https://github.com/sifr-lang/sifr/issues/3750)**,
SQL verification / compiler-verification, owns the two entries in
`verification/profiles/merge.json:44`. Preserve compiler classifications and
require blocking full-mode executed memberships with exact package commands.
The later issue records concrete acceptance commands and distinguishes profile
metadata/verification helpers from workflow files under the zero-gate rule.
No later implementation started. Full terminal/evidence hashes are at the top
of `ad-hoc-emitted-rust-excellence.md`; raw evidence remains under
`/private/tmp/sifr-sql-registry.xaAOLM/evidence/`. B14/B15/original12K counters
and draft #3746 remain unchanged. This worker stops at the preserved handoff.

Item **12K-B16**, issue [#3749](https://github.com/sifr-lang/sifr/issues/3749),
owns the bounded nine-package/thirteen-target registry reconciliation and stale
PostgreSQL `lib` to `rlib` replacement. Its canonical scope, isolated main-based
clone, exact base and three named checks are registered at the top of
`ad-hoc-emitted-rust-excellence.md`. Complete metadata inventory is prepared
from actual main; previous stack-only qualification is not reused as a pass.
Two SQL negative regressions join the existing readiness self-tests. All prior
coverage policy and checks remain intact; no compiler, lockfile or gate changes.
Live-server targets are explicit `test-fixture` inputs to the existing live SQL
adapters, rather than claims of execution in the offline nightly profile.

The B15 recurrence on `4a0a03f430f1ad87b080a6172bc46209082e955a` failed its one
merge gate with the same 23 registry diagnostics (readiness 3/4). Its raw
terminal remains at `/private/tmp/sifr-companion.KIogHV/evidence/terminal.json`,
SHA256 `657424e6ab98d7adaf150062f34f91abbba350d806cad23e3eacb73c9f6ffae7`.
B14/B15 PR #3746 remains unmerged. Their historical review/gate counts are
unchanged by this separate registry delivery. No successor work starts here.

The repository naming cleanup ran `scripts/run_all_tests.sh` once. The gate
failed in coverage-matrix readiness with nine unclassified SQL packages,
unclassified SQL/host-tool test targets, an unclassified PostgreSQL `rlib`,
and a stale PostgreSQL `lib` classification. The naming cleanup changes no
SQL Cargo packages, targets, or coverage classifications.

Examples include `sifr_sql_mysql`, `sifr_sql_mysql_runtime`,
`sifr_sql_postgresql_runtime`, `sifr_sql_sqlite`, `sifr_sql_tool`,
`test:host_tool_cli`, `test:sql_migrations`, and `test:runtime_policies`.
The complete failure list is in `target/naming-cleanup/merge-gate.log` and
`target/verification/areas/coverage-matrix-merge-results.json`.

This issue owns reconciling the coverage registry with the existing SQL
package and target graph. No classification or coverage requirement was
weakened during naming cleanup. The merge gate was not repeated.

The subsequent demo directory follow-up ran its own final merge gate once on
2026-09-05 and reproduced the same SQL coverage classifications failure.
All 264 demo emitted companions passed freshness, along with the file-size,
HIR, Rust interop, and naming checks. No SQL classifications changed.
Evidence: `target/demo-layout/merge-gate.log`.

The abbreviated-label cleanup also ran its final merge gate once on 2026-09-05.
It reproduced the SQL package/target classification failures above. Demo
freshness, Rust interop matrix checks, naming checks, HIR, and file-size checks
passed. No SQL classification or dependency changed. Evidence:
`target/abbreviation-cleanup/merge-gate.log`.

The naming-review remediation ran its final merge gate once on 2026-09-05.
It reproduced the same SQL package/target classification failures after all
264 demo freshness checks and reached guardrails passed. No SQL code or
classification changed. Evidence: `target/review-remediation/merge-gate.log`.

PR [#3692](https://github.com/sifr-lang/sifr/pull/3692) repeated the create-PR
gate before opening the PR. The same coverage classifications blocked it
after all 264 demo freshness checks and reached guardrails passed. No SQL
changes were made. Log: `target/pr-cleanup/create-pr.log`.

The descriptive-demo-variable follow-up ran its final merge gate on
2026-09-05. It reproduced the same SQL coverage-classification failures
after all 264 demo companions passed freshness. No SQL source or coverage
classification changed. Evidence: `target/demo-name-followup/merge-gate.log`.

### Additional test-target lint observation (DX.2, 2026-09-17)

An exploratory all-target Clippy invocation found an existing
`clippy::type_complexity` diagnostic in
`crates/sifr_sql_mysql_runtime/src/codec.rs:115` (the unit-test function-pointer
assertion). That file was unchanged from base
`8fb424984351768116251ed5a5e577373603c533`. No SQL runtime implementation was
changed for this observation. The selected merge profile does not currently
invoke workspace Clippy; Items 1–3 and 7 still name it separately. The SQL
runtime/test owner can factor the assertion's type. Historical evidence:
`/home/yaser5/projects/sifr/dx2-evidence/clippy6.log` (availability not
rechecked during P0).

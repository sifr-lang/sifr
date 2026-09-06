# Python-interop qualification dependencies exposed by Item12B

Status: active; fresh sequential dependency workers authorized on 2026-09-05.
Owners: Python interop verification, codegen naming, project support assembly.

## Evidence and scope boundary

The user approved execution in order 12G, 12H, 12I, 12J, then integration 12K.
One worker owns one item at a time. Each implementation item receives one
exact-SHA Opus review and at most one remediation review. Follow the phase's
file-category gate rules; skip create-PR when merging in-session. No third review.
The integration item has its own explicitly approved integration review and one
merge-profile gate. Item12B's two failed gates and review history remain unchanged.

### Named dependency validation

Run each item's named suite after implementation, from its owned Sifr worktree:

```bash
# Item12G: no dependency on another Python repair to implement the path fix.
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite dependency-versions
# Item12H: execute after Item12G's terminal handoff.
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite binding-authoring
# Item12I: execute after Item12H's terminal handoff.
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite callback-examples
# Item12J: execute after Item12I's terminal handoff.
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite async-declaration-examples --suite async-context-examples
```

For each item, add and name focused regressions for its stated mechanism before
running tests. Run the canonical file-size guardrail. Compiler changes also require
the affected crate's tests, strict Clippy, formatting, HIR guardrail, and one applicable
merge-profile gate. Do not run Sifr gates for runner/docs-only changes if no compiler,
lockfile, fixture, or workflow files change. Do not repeat known-failed gates on an
unchanged candidate. Record incomplete or blocked qualification honestly.

Item12K requires all five suites above, the complete Python-interop area, and
affected Item12B corpus/native qualification with compiler/input provenance before
its integration review and exact-final-candidate merge-profile gate. Preserve every
original acceptance rule. Reuse unchanged-input evidence with explicit attribution.

The one authorized replacement Item12B merge gate ran on
`a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d`, paired with corpus
`8bcbe7ab7939e5c8362c10f61a80e368022cc372`.
It completed 30 Python-interop variants with five blocking failures after the
approved SQL coverage and corpus taxonomy repairs passed.

Evidence root: `/tmp/sifr-item12b.akguMz/`.
Use `merge-replacement-a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d.log`,
`replacement-a319-python-results.json`, `replacement-a319-lane-report.json`,
and the three preserved callback/async example reports listed in Item12B.

Both permitted Item12B reviews returned SATISFIED. The replacement gate failed.
No new compiler/fixture repair, further review, or gate is authorized by that
consumed allowance. These are later items, not an assertion that Item12B is closed.

## Item12G: dependency-checker demo path identity (merged)

Confirmed pre-existing at exact base `b475ebdcd37081aa2860d9c348ace4100b546eff`.
`verification/areas/python_interop/runner/dependency_versions.py:46` constructs
the obsolete `demos/m12_dlpack_demo` path. The real project is
`demos/python_dlpack`. The base and candidate checker share blob
`ee8e02e9df5ad629f761d5bf82ea76f6bd3abb57`; base already contains the renamed
pyproject at blob `7038f54e45d361963a2593a1b3549e59464391bb`.

The dependency-versions variant fails with FileNotFoundError before validation.
Future repair must align authoritative project paths, retain exact dependency
and artifact-hash requirements, and cover computed path references so textual
taxonomy cleanup cannot leave broken runtime paths. No missing-path fallback,
suppression, compatibility directory, or dependency-version change is justified.

### Item12G implementation and focused validation plan

The isolated Item12G branch starts at latest main
`b475ebdcd37081aa2860d9c348ace4100b546eff`. The checker now selects
`demos/python_dlpack` directly. Dependency versions, artifact hashes, lockfiles,
and missing-file failure semantics are unchanged.

Before testing, the focused regression command is registered here:

```bash
uv run --project verification --locked python -m unittest discover -s verification/areas/python_interop/runner -p test_dependency_versions.py -v
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite dependency-versions
python3 scripts/check_file_size_guardrails.py
```

The four focused regressions evaluate the computed paths for every audited
project, observe reads of both authoritative DLPack inputs, reject either missing
demo input, and reject the original concatenated stale-path mutation. The named
suite additionally retains all seven existing version, artifact, ownership, and
service-image negative checks. Runner/test/docs-only changes require no Sifr
create-PR or merge-profile gate under the explicit Item12G user instructions.

### Item12G closure evidence

[PR #3695](https://github.com/sifr-lang/sifr/pull/3695) merged on 2026-09-05.
Reviewed candidate: `1cb24bdd088bddf42077f6e42112e53bba7c3562`.
Merge SHA: `2b114727441f1adc3ed807adc0c41543ddab5b78`.
The three commands above passed on that candidate: 4/4 focused tests,
1/1 dependency-versions variant with all seven original negative mutations,
and the canonical 3,754-file size guardrail. The dependency audit covers two
projects, 19 packages, two locks, and two service images. Other compiled Python
capabilities are explicitly unselected, not qualified by this evidence.

The [one exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3695#issuecomment-5554835685)
returned **SATISFIED**, no blockers. No remediation review or Sifr gates ran.
Evidence root: `/tmp/sifr-item12g.B8fCer/`; validation receipt:
`evidence-1cb24bdd088bddf42077f6e42112e53bba7c3562.md`; review:
`opus-1cb24bdd088bddf42077f6e42112e53bba7c3562.biQjAq/response.md`;
suite report: `sifr/target/verification/areas/python-interop-results.json`.
Blocker: none. This resolves only the dependency-versions failure in the
historical five-failure Item12B report. Its other failures and exhausted
review/gate history remain unchanged.

### Deferred Python verification runner maintenance (not started)

Owner: Python interop verification. These are non-blocking Opus follow-ups,
separate from Items12H–12K and not implemented by Item12G:

- The new focused regression command is recorded but not selected by the area
  manifest. Evaluate continuous discovery/enrollment of standalone runner tests
  through the canonical area mechanism; preserve existing suite semantics.
- The focused test imports its sibling using the registered unittest discovery
  start directory. If broader discovery is adopted, make sibling imports work
  under that selected runner as well. The recorded invocation already passes.

## Later Item12H: project-wide generated-field identity

Binding-authoring fails with eight Rust E0560 diagnostics in generated
`binding_authoring/math_python.rs`: initializers use
`sifr_generated_python_error`, while the imported nominal declares `python_error`.

Suspected root: `generated_rust_canonicalizer/field_name_cleanup.rs:13` derives
its field rename map from declarations in one file. Imported consumers without
the declaration do not receive the same mapping. Relevant producers include
`class_field_emitter.rs:98` and `rust_interop_error_mapping.rs:193`.
Those files and identifier policy are unchanged from the exact base.
No base-runtime reproduction was run; unchanged-source provenance is not
misrepresented as an independent runtime pass/failure.

Future investigation must preserve type/module identity, collisions, imported
consumers, struct literals, patterns, and member access across a project.
Do not fix only PythonError with a name-only special case.

## Later Item12I: macro-defined project support visibility

Three callback examples fail Rust E0425:
`callback/asyncio_roundtrip.sifr`, `callback/reconciliation.sifr`, and
`pubsub/declaration_callback.sifr`. The generated
`SIFR_GENERATED_SIFR_TASK_CANCELLATION` exists but is inaccessible inside the
support module.

`generated_visibility.rs:67` makes ordinary items crate-visible but does not
handle the static declared inside `tokio::task_local!`.
`lib_project_codegen.rs:447` relocates support into a module imported by consumers.
These source files, test-project assembly, and support pruning are unchanged
from base; no separate base runtime test was performed.
Future repair must preserve macro-owned symbol identity, appropriate visibility,
consumer demand, and cancellation semantics in normal and test project modes.
Do not add blanket exports, suppressions, or substitute cancellation behavior.

## Later Item12J: async Python error-channel contract

### Item12J implementation (2026-09-06)

Owned worktree `/private/tmp/sifr-item12j.pT6Xkk/sifr`, branch
`codex/emitted-rust-excellence-item-12j`, freshly fetched base
`4ce05473f58716a611ac190581bf0737ba15331e`. Parent's two intentional dirty
records and retained 12B/12H/12I worktrees are read-only to this worker.

12H and 12I have terminal blocked handoffs, satisfying the execution order.
They are not merged: 12H draft [#3697](https://github.com/sifr-lang/sifr/pull/3697)
retains reviewed `9b52ac20094608c8a31f252db99e49ef7c963384`, record
`b6e6210a97598fb631b929b2d4daf4012b41bb16`; 12I draft
[#3698](https://github.com/sifr-lang/sifr/pull/3698) retains reviewed
`f6e8afd964bb214a44c50271dcb2014ee8e828b4`, record
`19ad69969a672d7b741122ded4dd879f2bdaf9ab`. Both sole gates failed SQL
coverage. Their detailed evidence and deferred 12H-F1–F5 / 12I-F1–F3 remain in
those commits and PRs; integration belongs to 12K, with no budget reset.

Authoritative contracts: `stdlib/_sifr/python.sifr` declares
`PythonError(Error)` with five string fields; architecture error semantics
retain ordinary error inheritance and Result covariance. Python protocol
architecture preserves same-loop async execution, original error replay, and
ordered cleanup/cancellation. The existing examples' `Result[None, Error]`
channels are valid under that inheritance contract and remain unchanged.

Root cause: descriptor data-parent selection intentionally excludes the builtin
Error marker, but class type collection and HIR-to-Type exports reused the
data-parent metadata as complete nominal ancestry. The implementation preserves
semantic error ancestry separately from data storage and uses that ancestry
at each HIR-to-Type reconstruction. It does not add an embedded Error field,
change the five-field Python boundary, or make unrelated nominal errors assignable.

Exact commands registered before test execution:

```bash
cargo test -p sifr_driver async_python_error_channel
cargo build --locked -p sifr
python3 scripts/check_demo_emitted_freshness.py --sifr target/debug/sifr --update
python3 scripts/check_demo_emitted_freshness.py --sifr target/debug/sifr
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite async-declaration-examples --suite async-context-examples
cargo test -p sifr_ir
cargo test -p sifr_lowering
cargo test -p sifr_frontend
cargo test -p sifr_codegen
cargo test -p sifr_driver
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
scripts/run_all_tests.sh --profile merge
```

Focused regressions cover both unchanged original examples, rejection of
unrelated return channels (one/three diagnostics), imported PythonError's exact
identity/shape/ancestry, local and imported transitive error ancestry without
data-parent storage, and rejection of a same-named nominal Error target.
The sixth regression covers imported CSV/configparser classes named Error:
canonical export must not rewrite their builtin ancestor to their own identity.
All six focused regressions pass before the candidate is frozen. The first
attempt lacked fresh-worktree submodules; early regression failures identified
test bridge/constructor setup and the repaired project-export ancestry path.
Those attempts are historical failures, not final candidate evidence.
The merge-profile gate is reserved for the final reviewed SHA, once only;
create-PR is omitted. Outer filtered-suite certification failures are not passes.
No 12K implementation, runner bypass, external repair, or history rewrite is allowed.

Historical pre-12J observation: both fixtures failed SIFR-RESULT-0003 during checking:
`verification/areas/python_interop/fixtures/async_declaration/httpx2_client.sifr`
and `verification/areas/python_interop/fixtures/async_context/aiosqlite_session.sifr`.
Their raised PythonError is incompatible with the declared Result[None, Error]
channel; the latter fixture produces three diagnostics.

At that historical observation, frontend/lowering/type-system, stdlib, and all
Python-interop fixtures/runners were unchanged from the observing candidate's
base. This is input-identity evidence, not a repeated base qualification run.
It does not describe the present12J compiler diff. Determine the authoritative source/error-identity
contract before repairing all affected fixtures or the appropriate compiler
mechanism. Preserve original assertions, async cleanup/cancellation, and
error propagation. Do not broaden accepted errors or suppress diagnostics.

## Item12J-R1 bounded remediation plan (2026-09-06)

Owned worktree `/private/tmp/sifr-item12j-r1.9j9Uhf/sifr`, branch
`codex/emitted-rust-excellence-item-12j-r1`, retaining implementation `f720a342`
and record `60219b0` unchanged in history. Fetched `origin/main` is still
`4ce05473f58716a611ac190581bf0737ba15331e`, the original reviewed base.
The parent's intentional dirty records and every retained worker are read-only.
The 2026-09-06 parent amendment is carried into this worktree's phase record.

Implement only the initial review's 12J-R1 conversion omission. Conversion
demand follows semantic ancestry and canonical nominal identities; project
and test-project support use the project nominal path registry. Existing
consuming inheritance conversions preserve inherited messages without cloning.
No 12I integration or next-item implementation is authorized here.

Register these focused native regressions before running tests, all selected by
the original named `cargo test -p sifr_driver async_python_error_channel` command:

- `async_python_error_channel_native_local_and_transitive_conversions`: emit,
  build and run local root/transitive errors using direct raise and propagation.
- `async_python_error_channel_native_stdlib_nominal_collisions`: emit, build and
  run distinct CSV/configparser Error declarations with original message assertions.
- `async_python_error_channel_native_project_aliases_and_collisions`: emit,
  build and run re-exported aliases, a transitive imported error, two same-named
  project errors, and a local
  nominal named ValueError distinct from the builtin. The existing negative
  regression retains rejection of a same-named nominal Error target.

Run only the exact Item12J command list above, reusing original IR, lowering,
and frontend evidence where their complete crate inputs remain unchanged.
Run affected codegen/driver tests, compiler build, freshness, named async suites,
strict Clippy and formatting/HIR/file-size checks after the complete correction.
Freeze the corrected SHA before the sole remaining remediation Opus review.
One exact-final-candidate merge-profile gate remains; create-PR is omitted.
Known external failures remain owned and honestly failed. If they prevent
independent merge, preserve the corrected reviewed candidate for 12K and stop.

Deferred **12J-F4**, owner nominal error export ancestry: the native test setup
also tried project-imported `class Error(ValueError)` and `class Error(Error)`
as positive source cases. Both remain rejected with SIFR-RESULT-0003 when raised
into builtin Error. R1 changes no lowering/frontend/export inputs relative to
`60219b0`, so this is unchanged-input source-check evidence, not an independent
base runtime run. The retained `focused-corrected.log` and `focused-native.log`
under the R1 evidence root contain the diagnostics. The final native collision
case uses `ValueError(Error)` and retains the original negative Error-target
test. This source ancestry issue is not repaired by the conversion item.

## Item12J-R1 terminal evidence (2026-09-06)

State: **blocked, unapproved, not merged**. Draft
[PR #3699](https://github.com/sifr-lang/sifr/pull/3699) preserves reviewed
implementation `4bc432f3474134b1a1d43202d39fd147893bb014` on the original base
`4ce05473f58716a611ac190581bf0737ba15331e`. History retains original `f720a342`,
record `60219b0`, R1 implementation `3ba19e49a`, then its one-line Clippy fix.
The final record is a separate documentation-only commit. Original/parent
worktrees and indexes remain untouched; only the existing PR branch was
fast-forwarded from this isolated worker's branch.

Final candidate evidence, outside the reviewed tree:
`/private/tmp/sifr-item12j-r1.9j9Uhf/evidence-4bc432f3474134b1a1d43202d39fd147893bb014.md`.
Compiler SHA256:
`12adc00c7d5111550f893a20b1b3c3936ece888a13e3bf14b22e67f2d4e7fe09`.

- Focused command: 9 pass, including all three emission/native build/run groups
  and the transitive imported case. Full driver: 587 pass, 77 existing ignored.
- Build, formatting, HIR, canonical file-size guard (3758 files) and freshness
  of all 264 companions pass. Two companions were producer-regenerated; no
  original fixture, lockfile, workflow, assertion or runtime contract was edited.
- Codegen: 1407 pass / 2 existing 12B list-repeat failures. Strict Clippy retains
  only the 12B/12C `project_stdlib_nominals.rs:45` expect failure. Original
  unchanged-input IR4 and frontend132+7 passes, and lowering1114 pass /2 stale
  TypeVar assertions /1 ignored (#3667), are explicitly reused, not rerun passes.
- Both exact named async suites exit 1 at 12I-owned native task-local E0425.
  HTTP reports one Rust error; context 58, with a retained E0425 tail. Runtime
  output markers are false for both. No native async runtime pass or complete
  area certification is claimed. SHA-keyed JSON archives and logs are retained.
- The [only remaining remediation review](https://github.com/sifr-lang/sifr/pull/3699#issuecomment-5556273003)
  returned **NOT SATISFIED**. Its full response is
  `review-4bc432f3474134b1a1d43202d39fd147893bb014.md` under the evidence root,
  SHA256 `ab616b0a0917bac3c269ece9f24ea9d82f0bb7124f685241ac860af6a34e8b42`.
  It verified the new message-storage mechanism defect and remaining conversion
  omission described in Item12J-M1 below. No third review or post-review code
  repair was attempted.
- Cumulative 12J allowance consumed: one initial review and one remediation
  review, both NOT SATISFIED; zero create-PR gates and zero merge-profile gates.
  There is no approved final candidate to gate or merge. The unused gate is not
  a new review allowance. All 12B/12H/12I histories remain unchanged.

## Later Item12J-M1: error message storage and root-upcast admissibility

Status: recorded only; requires adjudication, not started by R1.
Owners: nominal error representation, error conversion demand, ancestry
admissibility. This is a new second-review mechanism, not a third R1 review.

The sole remediation review found two related blocking consequences at
`crates/sifr_codegen/src/error_refs/conversions.rs:99-130`, using
`preamble/error_conversion.rs:17-23` and the broadened render guard in
`support_plan.rs:204`:

1. **New regression:** `class CodeError(Error): code: int`, raised only into its
   own `Result[None, CodeError]` channel, compiled before R1. Adding an unrelated
   function returning `Result[None, Error]` now demands an invalid
   `From<CodeError> for Error` whose body reads absent `err.message`, producing
   E0609. The reviewer verified native success with preserved compiler
   `36640bf...9b61940` and failure with the exact candidate compiler above.
   `class EmptyError(Error): pass` and `message: int` similarly expose absent or
   non-string storage (E0609/E0308). These classes need not be upcast themselves
   for the regression to occur.
2. **Remaining in-scope omission:** explicitly raising that CodeError into
   `Result[None, Error]` source-checks on both original 12J and R1. Native build
   changes from E0277 to E0609, not to success. The original obligation remains
   unresolved for this representation, so repeated-finding adjudication is
   required in addition to recording the new mechanism.

Later bounded work must establish a shared, typed message-storage/conversion
contract: own string message versus consuming a valid ancestor conversion,
without assuming a `message` field from ancestry alone. Preserve previously
compiling specific-error channels; do not emit invalid unused conversions when
an unrelated Error reference appears. Reconcile root-Error source admissibility
with actual conversion ability instead of accepting backend-invalid programs.
Register emission and native compilation regressions for absent, non-string,
and inherited message storage, inside and outside root-Error channels. No
invented message fallback, fixture weakening or unrelated dependency repair is
authorized by this record. Resolve the contract/adjudication before defining
any later item's review/gate allowance; do not reset 12J's two consumed reviews.

Separate non-blocking **12J-F5**, owner codegen nominal-path mapping: the review
suggests consolidating `render()`'s ancestor-name derivation with the project
nominal-path authority to remove duplicate mapping logic. This is not another
R1 implementation step. Prior 12J-F1–F4 and retained H/I findings remain owned.

## Required next action

Stop the R1 worker after publishing this terminal record. Adjudicate Item12J-M1
and the unresolved conversion obligation before treating Item12J as qualified.
Candidate `4bc432f3474134b1a1d43202d39fd147893bb014` is unapproved and must not be
integrated as-is. No third review, gate, merge, 12K integration, external repair,
or next-item code is performed by this worker. 12K's separately approved
integration allowance follows dependency qualification and does not reset any
12B/H/I/J history.

### Item12J terminal evidence and unresolved review

State: **blocked**, draft [PR #3699](https://github.com/sifr-lang/sifr/pull/3699).
Reviewed implementation `f720a342edd87004975355b478948f7eb5c8b406`; merge SHA:
none. The final record is a separate documentation-only commit after that SHA.
The initial [Opus review](https://github.com/sifr-lang/sifr/pull/3699#issuecomment-5555927728)
returned **NOT SATISFIED**. The user's orchestration checkpoint then requested
terminal handoff because external failure was already established. No remediation
code or review was started; no final-approved candidate exists to gate.
Allowances consumed: one initial review, zero remediation reviews, zero merge
gates, zero create-PR gates. Do not relabel this as a qualified candidate.

[Exact-candidate evidence and changed paths](https://github.com/sifr-lang/sifr/pull/3699#issuecomment-5555929816)
are published outside the reviewed Git tree. Evidence root:
`/private/tmp/sifr-item12j.pT6Xkk/`, receipt
`evidence-f720a342edd87004975355b478948f7eb5c8b406.md`, review
`review-f720a342edd87004975355b478948f7eb5c8b406.md`. Compiler SHA256:
`36640bfbb7c29f7d0019d86ed9539c20311db434c326bf31bada4319b9d61940`.

- Six focused lowering/export regressions pass; IR4, frontend132 unit +7
  integration, driver584 active (77 existing ignored) pass. These focused tests
  do not prove generated conversions for local/imported error classes.
- Lowering1114 pass /2 fail /1 ignored, existing TypeVar message assertions
  [owner #3667 notified](https://github.com/sifr-lang/sifr/issues/3667#issuecomment-5555882691).
  Codegen1407 pass /2 existing12B-owned list-repeat failures. Strict Clippy
  fails at unchanged `project_stdlib_nominals.rs:45` (`expect_used`), owner12B/12C.
- All264 generated companions remain byte-identical. Formatting, HIR and
  canonical file-size guard (3756 files) pass. No fixture/assertion changes.
- The exact two-suite command exits1: both examples pass source checking and
  fail native build at12I-owned inaccessible cancellation task-local E0425.
  HTTP has one reported Rust error; context reports58 errors with retained tail
  naming that task-local and E0425. No runtime output markers were observed;
  original cleanup/cancellation/assertions did not execute. This is neither
  a native pass nor complete-area certification; no bypass flag was used.
- Native report files are `async-declaration-<candidate>.json`,
  `async-context-<candidate>.json`, and `python-results-<candidate>.json` under
  the evidence root; receipt records their SHA256s and log names. Final native
  TMPDIR is private; driver used `RUST_TEST_THREADS=1` without changing selection
  or internal concurrency. Interrupted earlier cache/scheduling attempts are
  explicitly incomplete in the receipt, never passing evidence.

**Unresolved in-scope finding12J-R1 (not implemented):** `support_plan.rs:184–200`
generates conversions only for builtin errors and async PythonError, while the
new ancestry also accepts non-builtin local/imported errors into builtinError.
Opus emitted local DomainError and imported `sifr.csv.Error` examples and
verified E0277, missing `From<T> for Error`. The bounded correction must connect
semantic ancestry to conversion demand and add emission/compilation regressions
for local, project-imported and stdlib errors, preserving nominal identity and
the original runtime contract. The first review remains NOT SATISFIED until
the sole permitted remediation review approves a corrected exact SHA. A new
mechanism defect on that second review must be deferred and stopped, not trigger
a third review. This terminal worker does not implement that continuation.

Separate review follow-ups, not implementation or new blockers:

- **12J-F1**, owner nominal error inheritance: pre-existing mixed data-base plus
  Error-marker ancestry (`MixedError(Payload, Error)`) overwrites the marker and
  still rejects propagation. Review classifies this as unchanged from base;
  no independent base-runtime result is claimed here.
- **12J-F2**, owner nominal export diagnostics: already-diagnosed unresolved
  parent paths can fall back to a noncanonical/nontransitive parent string.
  Evaluate separately; do not add a compatibility fallback in12J.
- **12J-F3**, owner codegen maintainability: the class-field PythonError-contract
  reconstruction does not currently consult ancestry, making that converted
  field a harmless consistency-only change. Cleanup is optional later work.

The review's infrastructure observation remains owned by12I: runtime async
cleanup/cancellation cannot be certified before its visibility repair is
integrated and qualified. Detailed12H-F1–F5 and12I-F1–F3 remain in their retained
record commits/PRs above; none was copied over with stale statuses or discarded.
The known SQL coverage failure remains an external owner and was not rerun
or newly claimed as a12J gate result. This item stops without merging or starting12K.

## Item12J-M1 contract adjudication (2026-09-06)

Status: **needs-new-scope; no implementation, approval, or merge**. The new
bounded M1 scope and full authority audit are preserved in the
[phase handoff](ad-hoc-emitted-rust-excellence.md#item12j-m1-contract-adjudication-handoff-2026-09-06).
Owned checkout `/private/tmp/sifr-item12j-m1.VO82Kk/sifr`, branch
`codex/emitted-rust-excellence-item-12j-m1`, retains `c430ed3331169f06eb148122f681e7d2a457d2ee`
and all reviewed 12J/R1 lineage. Fetched main remains `4ce05473f58716a611ac190581bf0737ba15331e`.
The parent and all prior worker workspaces/evidence remain unchanged.

Architecture requires inherited `message: str` supplied at construction, while
the implementation treats root Error as a special base without embedded
storage and constructs custom errors from only their declared fields. Root
Error itself requires a string. The source/representation authorities do not
define a root message for accepted `CodeError(3)`, `EmptyError()`, or
`message: int` values. Existing specific-error Display behavior is not an
authorized root-conversion policy. The prior native regression and unresolved
root-upcast failures are reused from the sole R1 remediation review; no new
compiler probe or test was run before resolving this contract conflict.

The required explicit choice is between enforcing inherited required string
storage (breaking existing constructors/overrides), defining how existing
Display output supplies a root string (new observable conversion semantics),
or allowing absent-message payloads in a wider root representation (larger
language/runtime scope). No option was implemented. Suppressing only invalid
unused conversions would leave the repeated root-upcast obligation unresolved.

M1 used **zero reviews, zero gates**. Complete implementation, registered
regressions, compiler/native tests, and merge are unreached. Old 12J/R1 remains
NOT SATISFIED, not qualified, with both reviews consumed and no gate. Named
historical external failures retain their owners and statuses; no 12K input
was integrated. Only the two Markdown records change, with documentation diff
and file-size results retained at `/private/tmp/sifr-item12j-m1.VO82Kk/evidence.md`.
Draft PR #3699 remains the linked unapproved predecessor. Next action: explicit
owner/user contract adjudication before resuming M1; this worker stops here.

## Item12J-M1 required-string terminal receipt (2026-09-06)

The user explicitly authorized the required constructor-supplied `message: str`
contract, including breaking message-less calls and integer overrides. The prior
decision blocker above is **resolved**, and the implementation is now source-review
**SATISFIED**, but **not merged / not closed** because external qualification is
still blocked. Historical12J/R1 NOT SATISFIED verdicts and exhausted reviews remain
unchanged; no old allowance was reset.

- Approved implementation: `d726ffc11258c49f0185fd2d49697988cf90972c`, retaining
  record `054a823b9aaafd388ddf1d944f1b7e50fcb95c29` and all original12J/R1 lineage.
- Draft bounded [PR #3700](https://github.com/sifr-lang/sifr/pull/3700), linked to
  preserved predecessor #3699; branch `codex/item12j-m1-required-string`.
- Initial exact-SHA [Opus review](https://github.com/sifr-lang/sifr/pull/3700#issuecomment-5558127599)
  SATISFIED with no blockers. M1 used1initial review,0remediation reviews,0provider
  retries,0create-PR gates,0merge gates; merge SHA none.
- Owned checkout `/private/tmp/sifr-item12j-m1-required.vL5lSI/sifr`; parent and
  previous worker checkouts/indexes/targets stayed read-only. Main remains
  `4ce05473f58716a611ac190581bf0737ba15331e`.
- Receipt, final record SHA,36changed paths, raw logs, and native JSON archives:
  `/private/tmp/sifr-item12j-m1-required.vL5lSI/evidence-d726ffc11258c49f0185fd2d49697988cf90972c.md`.

Typed string storage, required default/custom/inherited construction, declaration
and call rejection, nominal exports, and consuming root projections now agree.
PythonError keeps exactly five fields and five constructor inputs: its native
package regression asserts all fields and root conversion using the existing
probed interpreter/native-link trust, with no ignore or fallback. Local,
inherited, mixed data-parent, imported/aliased, stdlib, collision, project and
test-project native regressions pass. Full driver592active/77existing ignored,
M1focused lowering3, original async-error-channel9, frontend132+7, IR4,
type-system147, locked build,264fresh companions and all named static guards pass.

**12I remains the Python qualification blocker:** both named original async
declaration/context suites pass their source policy but fail native build with
inaccessible `SIFR_GENERATED_SIFR_TASK_CANCELLATION` E0425. HTTP reports1Rust error;
context reports58 with the same retained tail. Both runtime stdout markers and
cleanup/cancellation assertions are **UNREACHED**, not native passes or skips.
Fresh archives are `async-declaration.json`, `async-context.json`, and
`python-results.json` beside `async-native.log`. Compiler SHA256 remained
`166c1d23662db3c0da97b9c921e6f7fee22755b68c49e576906e07a569eec16e` across the run.

Other known owners also block standalone merge:12B's2list-repeat codegen
assertions,12B/12C's unchanged strict-Clippy `expect_used` (now line47), and
TypeVar #3667's2stale lowering assertions. SQL coverage is retained external
history, not newly rerun. None was absorbed or claimed fixed; no merge gate was
spent over failed prerequisites.

The phase terminal handoff records separate follow-ups12J-M1-F1 (compiler
invariant hardening), F2 (unproven reference-only inherited SQL error reachability),
F3 (integration E2E/stdlib executable coverage), and F4 (local Error-shadow
storage documentation). They are not blocking source-review findings and no
follow-up implementation started. Preserve the approved M1 candidate for the
separately owned12K integration after dependency qualification. This worker
stops after the record; it does not integrate12I or begin12K.

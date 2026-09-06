# Python-interop qualification dependencies exposed by Item12B

Status: active; fresh sequential dependency workers authorized on 2026-09-05.

<!-- Historical incoming Item12B record; later 12K dispatch is authoritative. -->
Status: active, blocking Item12B merge; recorded only, not implemented.
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

<!-- Historical incoming Item12B record; later 12K dispatch is authoritative. -->
## Later Item12G: dependency-checker demo path identity

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

<!-- Preserved dependency history; latest 12K dispatch is authoritative. -->

## Item12H: project-wide generated-field identity (implementation)

Owned worktree: `/tmp/sifr-item12h.afJDbk/sifr`; branch:
`codex/emitted-rust-excellence-item-12h`; base:
`4ce05473f58716a611ac190581bf0737ba15331e` (freshly fetched main, including
12G implementation and record merges). Parent and Item12B state are preserved.

The bounded implementation resolves generated fields through a project registry
keyed by Rust module and nominal declaration before identifier cleanup. Binary
and test projects share the registry. Owner-local collisions, import aliases,
re-exports, nested modules, typed receivers, initializers, and patterns use the
same declaration mapping. External fields retain their spelling. Unknown
generated-field receivers fail with a compiler diagnostic rather than a guessed
global replacement. No PythonError-specific naming rule is introduced.

### Item12H terminal handoff (2026-09-06)

**Blocked; reviewed implementation preserved, not merged.**

- PR: [#3697](https://github.com/sifr-lang/sifr/pull/3697), remains draft/open.
  Reviewed/final implementation SHA: `9b52ac20094608c8a31f252db99e49ef7c963384`.
  Merge SHA: none. Branch/worktree ownership above is unchanged.
- [Final Opus remediation review](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555345800):
  **SATISFIED**, no blockers. Exactly one initial and one remediation review
  were used. No further implementation or review is performed after this verdict.
- Final evidence: exact-SHA focused tests 3/3, final-source driver tests 581/581
  active (77 existing ignored), unchanged-codegen canonicalizer tests 115/115,
  and native binding execution (`binding runtime ok`). Formatting, file-size
  and HIR guardrails pass. All 264 demo emissions/freshness checks pass; 21
  generated companions differ from base. Full binding-authoring and strict
  Clippy remain incomplete/failed as recorded below; they are not pass evidence.
- One exact-clean-SHA merge-profile gate failed after 362.20s at
  `coverage_matrix:readiness/coverage_matrix_readiness`: nine unclassified SQL
  packages, 13 unclassified targets, and one stale PostgreSQL library target.
  The three other coverage variants passed; Rust interop passed 10 variants.
  Later Python-area, crate and E2E gate stages were not reached. No create-PR
  gate, second merge gate, or qualification bypass was used.
- This reproduces the existing
  [SQL coverage registry blocker](ad-hoc-schema-first-sql-platform-review-follow-ups.md#coverage-registry-blocker-observed-during-naming-cleanup-2026-09-05).
  **Concrete additional 12K dependency:** SQL compiler/schema-tool verification
  must reconcile the existing package/target classifications and qualify that
  repair. 12H changes none of those inputs. The inherited Item12B/12C compiler
  checks and clean-environment Python bytecode failure remain separate 12K inputs.
- [Published exact-SHA evidence](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555393502).
  Preserved files under `/tmp/sifr-item12h.afJDbk/`:
  `merge-9b52ac20094608c8a31f252db99e49ef7c963384.log` and `.json`,
  `coverage-matrix-9b52ac20094608c8a31f252db99e49ef7c963384.json`,
  `rust-interop-9b52ac20094608c8a31f252db99e49ef7c963384.json`,
  `validation-9b52ac20094608c8a31f252db99e49ef7c963384.md`, and the focused/native
  logs below. The full binding failure report was separately preserved before
  the gate. Do not reuse any failed or incomplete receipt as a pass.
- Stop after the record-only update. No12I/12J code was implemented and no next
  item was started.12K must establish passing integrated evidence before merge;
  this handoff does not reset any exhausted review or gate allowance.

### Item12H validation history

Exact validation commands registered before test execution:

```bash
cargo test -p sifr_codegen project_field_identity
cargo test -p sifr_codegen -p sifr_driver
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite binding-authoring
cargo clippy -p sifr_codegen -p sifr_driver --all-targets -- -D warnings
cargo fmt --check
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile merge
```

The merge-profile command is reserved for the final reviewed SHA, once only.
No create-PR gate runs in this merge session. Focused regressions include
negative external-type/name-collision variants and unresolved-owner rejection.
12I cancellation visibility and 12J error-channel contracts remain out of scope.

The first crate run reached 1,412 passing codegen tests and three failures.
The parsing-diagnostic prefix regression belongs to 12H and is corrected.
Two list-repeat expectations fail upstream of canonicalization in unchanged
`generate_rust_with_metadata`: `test_list_repeat_lowers_without_vec_mul_shape`
and `single_element_list_repeat_uses_std_repeat_not_extend_loop`. Their producer
and tests are outside 12H; Item12K must reconcile them with preserved Item12B
before integration. This is source-path provenance, not an independent base run.
Log: `/tmp/sifr-item12h.afJDbk/crate-tests-repair.log`. No assertion was weakened.
The crate command stopped before driver tests. Follow-up constituent commands:

```bash
cargo test -p sifr_codegen rejects_invalid_assembled_source
cargo test -p sifr_driver
```

Strict affected-crate Clippy reached the unchanged
`crates/sifr_codegen/src/project_stdlib_nominals.rs:45` `expect_used` failure.
This is the builtin-registration blocker already incorporated as Item12C into
preserved Item12B; Item12K owns bringing that repair into the integrated base.
Log: `/tmp/sifr-item12h.afJDbk/clippy.log`. No allowance or duplicate repair was
added by 12H. The failed command is not passing qualification evidence.

After the typed Result/closure repair, binding-authoring passed its native
`binding runtime ok` assertion and the subsequent frozen binding/check
immutability checks, then failed at `binding_authoring.py:362`: bytecode cache
state changed in the initially clean area environment. Remaining assertions
after that line were not reached. Log:
`/tmp/sifr-item12h.afJDbk/binding-authoring-error-flow.log`.
The only observed cache files are `_virtualenv.cpython-314.pyc` and
`_distutils_hack/__init__.cpython-314.pyc`. The unchanged Sifr probes use `-B`
and the unchanged embedded runtime sets `PyConfig.write_bytecode = 0`.
The unchanged PyO3 build-config interpreter launcher lacks `-B`; startup during
native dependency building is a suspected cause, not an independently proven
base reproduction. Owner: Python build/verification, required 12K integration
input. Do not disable the bytecode assertion or count a warmed-environment
rerun as proof of clean-environment immutability. No repair is included in 12H.

The restricted driver run was interrupted after native Cargo probes repeatedly
failed under sandbox restrictions. One exact failed bridge-signature probe
passed with required permissions (1/1); this is diagnostic evidence only.
The affected driver command will use those permissions for final qualification.

Additional focused final-candidate commands (registered before execution):

```bash
cargo test -p sifr_codegen generated_rust_canonicalizer
cargo test -p sifr_driver project_field_identity
cargo build --locked -p sifr
# cwd: target/verification/areas/python_interop/binding-authoring
/tmp/sifr-item12h.afJDbk/sifr/target/debug/sifr run src/main.sifr --frozen
```

The direct native command qualifies the original cross-module field mechanism;
it does not replace the failed full binding-authoring suite. Final mechanism
regressions also cover generic member chains, enum payloads, loop shadowing,
declared method return identities, Result error closures and Err patterns.

### Item12H pre-review qualification receipt

Initial Opus review of `405e3d3c2adcf018044a2f733ac64ec942f01967` returned
NOT SATISFIED: generated Rust bridge modules were assembled after the shared
field pass. The one remediation batch moves their generation into project
assembly, includes root/child bridge module identities in the registry and cache,
and writes finalized files without independent field canonicalization. Record
and error bridges with an underscore field, a same-owner public-name collision,
an imported consumer, a pattern, and native execution are covered together.

Remediation validation registered before execution (no new broad gate allowance):

```bash
cargo test -p sifr_driver project_field_identity
cargo test -p sifr_driver
cargo clippy -p sifr_codegen -p sifr_driver --all-targets -- -D warnings
cargo fmt --check
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
cargo build --locked -p sifr
# cwd: target/verification/areas/python_interop/binding-authoring
/tmp/sifr-item12h.afJDbk/sifr/target/debug/sifr run src/main.sifr --frozen
python3 scripts/check_demo_emitted_freshness.py --sifr target/debug/sifr --update
```

Canonicalizer compiler code is unchanged, so its 115-test evidence is reused.
The previously failed clean-environment binding suite is not rerun warm as a
substitute pass. The single merge-profile gate remains reserved for the exact
final reviewed implementation SHA. The only permitted remediation review follows
this batch. Initial review suggestions about future macro/inference coverage and
map key/value receiver identities are deferred to generated-field maintenance;
they are not implemented as a second mechanism in this batch.

The first remediation refresh exposed listing-only expansion (all pre-existing
bridge carriers were being appended to every demo). That incidental output
expansion is removed before review: the source-listing boundary stays unchanged,
all bridge sources still enter the shared registry before formatting, and only
the field-free native root module declaration is added at materialization.
The registered driver/focused/native commands and companion generator are rerun
for this changed assembly input; the inherited strict-Clippy failure is reused.

Final remediation source inputs pass all 581 active driver tests (77 existing
ignored tests), including the two new bridge materialization/namespace cases;
`driver-remediation-final.log`. The rebuilt CLI passes native binding execution
again (`native-remediation-final.log`, `binding runtime ok`). Formatting, the
3,758-file size guardrail and HIR guardrail pass. Additional interop contract
test files only initialize the new generated-project bridge-module collection;
no async, callback, or other contract behavior is changed. The strict-Clippy
receipt remains `clippy-remediation.log` (inherited Item12C failure, no pass).
The final compiler-owned refresh succeeds for all 264 demos. Relative to the
initial reviewed candidate, only five companions change: `advanced_class_libraries`,
`csv`, `regex_and_filesystem`, `stdlib_expansion`, and
`structured_parsing_serialization`. These remove redundant identifier escaping
from the independent per-file pass and induced reflow; the temporary bridge
listing expansion is absent. There are 21 changed companions overall relative
to the item base. Log: `companion-refresh-remediation-final.log`.

Implementation commit `dba6a8f7075ea071058654c85ed1e46e4d1272fa` passed all
115 canonicalizer tests (including nine focused field regressions), all 579
active driver tests (77 existing ignored tests), and the direct native
cross-module regression (`binding runtime ok`). Logs are keyed by that SHA
under `/tmp/sifr-item12h.afJDbk/`: `canonicalizer-*.log`, `driver-*.log`,
`build-*.log`, and `native-*.log`. The earlier restricted driver run is not
used as final driver evidence.

Compiler-owned companions were regenerated with
`python3 scripts/check_demo_emitted_freshness.py --sifr target/debug/sifr --update`.
All 264 emissions succeeded; 18 companion files changed through that generator.
The changes remove field collision suffixes caused by unrelated nominal types
(Logger, Random, deque, CSV/ZIP carriers). No Sifr demo or reference Rust source
changed. Refresh log: `companion-refresh-dba6a8f7075ea071058654c85ed1e46e4d1272fa.log`.
The only subsequent compiler edit replaces an implicit String clone with
explicit `.clone()` for Clippy; field resolution and generated output are
unchanged. Strict Clippy after that repair reports only the inherited Item12C
failure (`clippy-clone-repair.log`). Exact-final-candidate focused checks and
review follow; no gate has yet run and the full binding suite remains blocked.

### Item12H deferred maintenance (not implemented)

The [initial exact-candidate Opus review](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555203238)
classified these as suggestions, not blocking findings. Owner: generated-Rust
field-resolution maintenance after the current dependency/integration sequence.
They require their own bounded implementation and evidence, not another 12H
review/gate iteration:

- **12H-F1, macro syntax coverage:** non-expression-list macros other than
  `vec![element; count]` are not traversed by field cleanup. Require explicit
  field handling before an emitter adds such field-bearing macro syntax.
- **12H-F2, receiver inference coverage:** casts, async/unsafe/loop expressions,
  and additional iterator adapters need typed coverage before expanding emitted
  receiver forms; unresolved generated-field receivers currently diagnose.
- **12H-F3, map receiver identity:** the general index-element rule selects the
  first generic argument. A map needs its value argument instead; qualify this
  against actual emitted map access and owner-local collision variants before
  changing the mechanism. No runtime base reproduction was claimed by this review.
- **12H-F4, pre-existing bridge layout:** the remediation reviewer flagged the
  bridge module name `mod` as a potential alias of the root `mod.rs` path.
  Qualify reserved-word handling and physical path identity when bridge layout
  is next touched; the item-level module-identity check is not a path registry.
  This was classified as pre-existing, not a new blocking field mechanism defect.
- **12H-F5, single-source API coverage:** review consolidation of the
  single-source canonicalization convenience wrapper now that production uses
  the project entry point. Preserve actual public API requirements and equivalent
  focused coverage if a later owner changes it; no API is removed here.

The duplicate per-file field pass suggestion is resolved as part of the required
bridge correction: materialization now only formats already canonical sources.
The initial review's infrastructure attribution to PyO3 is not stronger evidence
than the suspected-cause provenance above; the clean-environment failure remains
unqualified and externally owned.

### Original Item12H diagnostic provenance

Before this implementation, binding-authoring fails with eight Rust E0560 diagnostics in generated
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

## Item12I: macro-defined project support visibility

### Item12H dependency handoff carried forward (2026-09-06)

Item12H is terminal **blocked, not merged**. Draft
[PR #3697](https://github.com/sifr-lang/sifr/pull/3697) preserves reviewed
implementation `9b52ac20094608c8a31f252db99e49ef7c963384` and final record
`b6e6210a97598fb631b929b2d4daf4012b41bb16`. Initial plus sole remediation
reviews are consumed; final Opus verdict is SATISFIED. Its one merge-profile
gate failed existing SQL coverage classifications (9 packages, 13 targets,
1 stale PostgreSQL target). Focused 3/3, driver 581 active, and canonicalizer
115 passed; full binding-authoring and strict Clippy did not pass.
[Exact evidence](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555393502)
and the complete owner record at that final record SHA remain authoritative,
including deferred suggestions 12H-F1–F5. No 12H implementation is included here.
This terminal handoff permits 12I to execute; 12K owns integrated qualification
with preserved 12B/12C repairs, SQL classifications, and Python build/verification
inputs. No earlier review or gate allowance is reset.

### Item12I implementation and named validation (2026-09-06)

Owned worktree: `/private/tmp/sifr-item12i.0l85Lu/sifr`; branch:
`codex/emitted-rust-excellence-item-12i`; freshly fetched base:
`4ce05473f58716a611ac190581bf0737ba15331e`. Parent and prior workers are
read-only. Scope is the compiler-owned `tokio::task_local!` declaration grammar:
preserve names, attributes, types, and cancellation operations; apply only
crate visibility at the support relocation boundary; discover and prune each
declared symbol using consumer demand in binary and test project assembly.
Unknown macros and nested modules retain their visibility. No blanket exports
or cancellation substitutes are introduced.

Exact commands registered before any test execution:

```bash
cargo test -p sifr_codegen task_local_support
cargo test -p sifr_codegen
uv run --project verification --locked python -m sifr_verify areas run --area python_interop --suite callback-examples
cargo clippy -p sifr_codegen --all-targets -- -D warnings
cargo fmt --check
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
git diff --check
scripts/run_all_tests.sh --profile merge
```

The eight focused regressions cover exact macro identity, multiple declarations,
attributes/types, crate visibility, unknown/nested macro boundaries, transitive
and absent demand, a macro-only owner, both project modes, synchronous absence,
and rejection of invalid compiler-owned declaration syntax. Compiler build for
the named callback suite is setup, not a separate qualification claim. The sole
merge-profile gate is reserved for the exact final reviewed implementation SHA;
no create-PR gate or second merge gate is authorized. Reuse unchanged-input
evidence and preserve failed/incomplete evidence honestly. 12H field identity,
12J error channels, 12B/12C repairs and external qualification inputs stay out
of scope. No 12J or 12K work starts in this session.

### Item12I pre-review qualification receipt

Implementation `2109ac57c9b474ceffd3efea317be5f82c739042` passes all eight
focused regressions (`/private/tmp/sifr-item12i.0l85Lu/focused-final.log`).
The affected codegen suite passes 1,415 tests and fails the two unchanged
list-repeat expectations previously recorded by 12H (`codegen.log`):
`test_list_repeat_lowers_without_vec_mul_shape` and
`single_element_list_repeat_uses_std_repeat_not_extend_loop`. Those tests and
their producers are unchanged from base; this is source-identity attribution,
not a separate base execution. 12K must reconcile the preserved 12B/12C repairs.
No test assertion is weakened. Formatting, file-size (3,756 files), HIR, and
diff checks pass. The initial new-test compile/assertion errors were corrected
before the committed candidate; they are not final failures.

The only checked-in demo companion containing `tokio::task_local!` is
`demos/typed_compiler_boundary/emitted.rs`. Register its bounded compiler-owned
refresh and freshness check before execution:

```bash
target/debug/sifr emit demos/typed_compiler_boundary/main.sifr
```

Capture the successful compiler output, regenerate that companion if different,
and compare its bytes with the candidate emission. This is the affected output
of the shared visibility mechanism; do not hand-edit it or qualify unrelated
demos here. Reuse the unchanged compiler-source test evidence after a generated
companion/record commit. The sole gate remains reserved for the reviewed final SHA.

The first callback suite ran the compiler built from `2109ac57c9b474ceffd3efea317be5f82c739042`
(binary SHA256 `2714fa28a04381ffef42be7ed9eaf5c7adadf7f02f2ef00ca299a011bbb11654`).
All seven native examples passed, including all three original E0425 failures;
the inner report records 14 variants, no failures, and no skips. The exact named
outer command nevertheless exits 1: `verification/areas/python_interop/runner.py:249`
rejects filtered-suite partial compiled certification. This is **not** a passing
whole-command receipt. Owner: Python interop verification; 12K must reconcile
bounded dependency evidence with complete-area promotable certification. No
`--allow-partial-certification` flag, complete-area expansion, or runner repair
is included. Preserved evidence: `callback-examples-2109ac57.json`,
`python-results-2109ac57.json`, and `callback-examples.log` under the private root.

Strict Clippy found two new `nonminimal_bool` suggestions; the parser now uses
equivalent `is_none_or` conditions. All eight focused tests pass after that
correction (`focused-lint-repair.log`). Strict Clippy now fails only at unchanged
`project_stdlib_nominals.rs:45` (`expect_used`, already incorporated as 12C into
preserved 12B); `clippy-repair.log` is not a strict-Clippy pass. No allowance or
duplicate builtin repair is added. The final compiler will be rebuilt and the
named callback suite run on its committed candidate to bind the native evidence
to that SHA; the known outer certification restriction remains an external
qualification blocker. The affected companion is regenerated byte-for-byte from
the compiler; its only diff is task-local crate visibility.

### Original diagnostic evidence

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

### Item12I terminal handoff (2026-09-06)

**Blocked; reviewed implementation preserved, not merged.**

- Draft [PR #3698](https://github.com/sifr-lang/sifr/pull/3698), reviewed/final
  implementation SHA `f6e8afd964bb214a44c50271dcb2014ee8e828b4`; merge SHA none.
  A subsequent record-only commit does not change its implementation inputs.
  Owned branch/worktree/base are recorded above. Parent and retained workers
  were not modified.
- [The one exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3698#issuecomment-5555560780)
  returned SATISFIED, no blocking findings. Zero remediation reviews. Raw
  response: `opus.nQzU0u/response.md`; SHA-keyed copy:
  `review-f6e8afd964bb214a44c50271dcb2014ee8e828b4.md` outside the Git tree.
- Exact final candidate: focused tests 8/8; seven native callback examples
  pass all 14 checks with no failures/skips, including the original three
  E0425 cases, cancellation/cleanup reconciliation, and `close=drained`.
  The named outer command still fails the unchanged filtered-suite partial
  certification restriction. No bypass flag or complete-area expansion.
  Final compiler binary SHA256:
  `2a500a81a5f44098618b4a0ec010008d0158ea77cc2b9ef0c5e0c2e97b09f22d`;
  native report SHA256:
  `df8ca27a1c8ebff8e6b7458aa9119bcb6f21556095b59ee86e43e3ace8949ba7`.
- Formatting, HIR, file-size (3,756 files), and diff checks pass. The candidate
  emission byte-compares equal to the sole refreshed companion. Full codegen
  and strict Clippy retain the externally owned failures recorded above;
  neither command is a pass.
- One exact-clean-SHA merge-profile gate failed (exit 1, 184.65s) at coverage
  readiness: nine unclassified SQL packages, 13 unclassified targets, one
  stale PostgreSQL library classification. Generated-companion freshness and
  preceding guards passed; Rust interop passed 10/10 and the other three
  coverage variants passed. Later Python-area, crate, and E2E stages were
  not reached. No create-PR or second merge gate. Storage was checked:
  246 GiB free, 8.5 GiB private target; no cleanup required.
- SQL compiler/schema-tool verification owns the existing
  [coverage blocker](ad-hoc-schema-first-sql-platform-review-follow-ups.md#coverage-registry-blocker-observed-during-naming-cleanup-2026-09-05).
  Python interop verification owns bounded-suite certification; 12K must
  establish complete-area promotable evidence. Preserved12B/12C compiler
  repairs and12H inputs remain separate integration dependencies. No external
  failure was repaired, waived, or reclassified here.
- Evidence root: `/private/tmp/sifr-item12i.0l85Lu/`. Final receipt:
  `evidence-f6e8afd964bb214a44c50271dcb2014ee8e828b4.md`. Exact-SHA files:
  `focused-f6e8afd964bb214a44c50271dcb2014ee8e828b4.log`,
  `callback-examples-f6e8afd964bb214a44c50271dcb2014ee8e828b4.log` and `.json`,
  `python-results-f6e8afd964bb214a44c50271dcb2014ee8e828b4.json`,
  `merge-f6e8afd964bb214a44c50271dcb2014ee8e828b4.log` and `.json`,
  `coverage-matrix-f6e8afd964bb214a44c50271dcb2014ee8e828b4.json`, and
  `rust-interop-f6e8afd964bb214a44c50271dcb2014ee8e828b4.json`. Reports are
  copied outside target; evidence is published on PR #3698.
- Stop after this record-only update; no12J/12K implementation. 12K must
  qualify the integrated dependencies before an affected merge. Earlier
  exhausted review/gate allowances remain exhausted.

### Item12I deferred maintenance (not implemented)

Opus classified these as suggestions, not blocking findings. Owner:
generated-Rust support/dependency analysis. They are separate future work
if the stated inputs are introduced; no new requirement is added to12I.

- **12I-F1, multi-declaration utility consistency:** strip/partition/single-name
  discovery utilities do not split before `parse_item_name`. Current emitters
  use one declaration per invocation; future grouped emission should normalize
  those boundaries. Current unsupported grouping retains rather than drops.
- **12I-F2, empty macro representation:** splitting an empty `task_local! {}`
  produces no entries. Empty invocations are not emitted today; decide whether
  retaining an empty item is needed before introducing such emission.
- **12I-F3, untrusted Rust diagnostic boundary:** malformed qualified declarations
  use the existing visibility parser's compiler-invariant panic convention.
  Current inputs are compiler-owned; introducing user-authored Rust into this
  discovery path requires a diagnostic boundary.

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

Item12G is merged and complete. Stop its worker after publishing the closure
record. The orchestrator may next assign bounded Item12H to a fresh isolated
worker, then Item12I and Item12J in order. Use the named validation mapping
above and preserve each item's review/gate limits. Item12K's expressly approved
integration allowance follows dependency qualification; it does not reset
Item12B's exhausted history. No Item12H–12K code was written by Item12G.

<!-- Historical incoming Item12B record; later 12K dispatch is authoritative. -->
Assign bounded dependency work with explicit review/validation limits. Relevant
canonical suites are dependency-versions, binding-authoring, callback-examples,
async-declaration-examples, and async-context-examples in the Python interop area.
Qualify all cases affected by each shared mechanism, not only the first example.
Then adjudicate how Item12B's already-approved candidate can obtain required
merge evidence without silently resetting its exhausted review/gate budgets.
No work on these later items was started in this recording change.

<!-- Preserved dependency history; latest 12K dispatch is authoritative. -->
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

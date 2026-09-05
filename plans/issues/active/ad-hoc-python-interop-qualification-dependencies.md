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

Both fixtures fail SIFR-RESULT-0003 during checking:
`verification/areas/python_interop/fixtures/async_declaration/httpx2_client.sifr`
and `verification/areas/python_interop/fixtures/async_context/aiosqlite_session.sifr`.
Their raised PythonError is incompatible with the declared Result[None, Error]
channel; the latter fixture produces three diagnostics.

Frontend/lowering/type-system, stdlib, and all Python-interop fixtures/runners
are unchanged from exact base. This is input-identity evidence, not a repeated
base qualification run. Determine the authoritative source/error-identity
contract before repairing all affected fixtures or the appropriate compiler
mechanism. Preserve original assertions, async cleanup/cancellation, and
error propagation. Do not broaden accepted errors or suppress diagnostics.

## Required next action

Item12G is merged and complete. Stop its worker after publishing the closure
record. The orchestrator may next assign bounded Item12H to a fresh isolated
worker, then Item12I and Item12J in order. Use the named validation mapping
above and preserve each item's review/gate limits. Item12K's expressly approved
integration allowance follows dependency qualification; it does not reset
Item12B's exhausted history. No Item12H–12K code was written by Item12G.

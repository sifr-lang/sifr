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

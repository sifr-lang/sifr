# Python-interop qualification dependencies exposed by Item12B

## B15 delivery recurrence: existing approved H/I/M1 stack is required (2026-09-07)

This is an evidence/ownership update only; no new dependency implementation or
duplicate issue. Final B15 candidate `875a3555a7d1bcd7885ad4151c75a4a7abf17a74`
in OPEN DRAFTPR3746, main base `0b97b3a3f1dd3f93bc724f75e1e40240f15ff942`,
finished its single continuation merge gate naturally exit1 after2833.76s.
The delivered SQL prerequisite now passes integrated readiness4/4. All92graph
preparations,13guards, RustInterop10/core-language5/CPython2 passed.
Python interop completed30variants,26PASS/4blockingFAIL:

- binding-authoring:8E0560, `sifr_generated_python_error` initializer versus
  imported `python_error` declaration. Existing **12H**, [PR3697](https://github.com/sifr-lang/sifr/pull/3697),
  approved source `9b52ac20094608c8a31f252db99e49ef7c963384`, full record
  `b6e6210a97598fb631b929b2d4daf4012b41bb16`, still OPEN DRAFT.
  [Final approval](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555345800),
  [terminal evidence](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555393502).
- callback-examples:3nativecases fail inaccessible cancellation task-local E0425,
  3of14innerchecks failed. Existing **12I**, [PR3698](https://github.com/sifr-lang/sifr/pull/3698),
  approved source `f6e8afd964bb214a44c50271dcb2014ee8e828b4`, record
  `19ad69969a672d7b741122ded4dd879f2bdaf9ab`, OPEN DRAFT.
- async-declaration-examples and async-context-examples: httpx2-client and
  aiosqlite-session SIFR-RESULT-0003, PythonError incompatible with
  Result[None,Error]. Existing correcting **12J-M1**, [PR3700](https://github.com/sifr-lang/sifr/pull/3700),
  approved source `d726ffc11258c49f0185fd2d49697988cf90972c`, record
  `a7e13eb45006eac925417491b89a932af5df2595`, OPEN DRAFT. Original12J/R1
  is not independently approved; only its reviewed correctingM1 lineage applies.

No baseline-runtime replay or new root-cause claim. Referenced naming producer
files are unchanged from actual-main predecessor06ea863. Read-only Git ancestry
verifies all three exact approved sources are already in B13's preserved full
`4eef8a2bb4dbc24fb7c1d1213652be379047f4b1`. B13 owner #3744, PRnone, needs
B14/#3745 capture demand plus its own singleton assertion. B15 now needs the
existing H/I/M1 stack to pass qualification. This is a delivery dependency cycle,
not permission to recreate or independently re-gate those items. Parent owns
assessment of a fresh bounded B13 integration owner carrying approved875a and
the complete B15 terminal record with mergedSQL into full4eef. No such work starts here.

Remaining15areas and full-mode crate/E2E steps are UNREACHED, including stdlib
parity and new SQL crate memberships. The emitted-Rust phase terminal lists all
15areas, exact sources/PRs, historical limits and complete evidence. B15 Opus
remediation SATISFIED but full gateFAILED, so PR3746 is not merged and #3748/#3745
remain open. Cumulative B15 gates2FAILED/0PASS/0RESOURCE; reviews1initial+
1remediation exhausted. B14's1FAILED and original12K4FAILED+1RESOURCE143/0PASS
remain unchanged. No next-item code, cleanup, extra gate or review.

Raw log `/private/tmp/sifr-b15-delivery.NykMhH/evidence/merge.875a3555a7d1bcd7885ad4151c75a4a7abf17a74.log`
SHA256 `34a4bd772f4a459b2f5ffa8cfedc538504149279744ba7e53559a8c6a20e4076`;
supervisor SHA256 `32e6bd96bba04676c8e769bfb4bdbf8aa6bc592ed066f5a418c3f82c6d61e6a8`;
Python result SHA256 `b71c5d790aebbe2ffe07d9d3b3b2b4dc1d1ac3b0b1f3ca614db38b76513b6b9d`.
The final native terminal indexes all artifacts. Parent and predecessors remain read-only.

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

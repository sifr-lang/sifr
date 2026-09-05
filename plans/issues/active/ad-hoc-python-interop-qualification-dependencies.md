# Python-interop qualification dependencies exposed by Item12B

Status: active, blocking Item12B merge; recorded only, not implemented.
Owners: Python interop verification, codegen naming, project support assembly.

## Evidence and scope boundary

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

Assign bounded dependency work with explicit review/validation limits. Relevant
canonical suites are dependency-versions, binding-authoring, callback-examples,
async-declaration-examples, and async-context-examples in the Python interop area.
Qualify all cases affected by each shared mechanism, not only the first example.
Then adjudicate how Item12B's already-approved candidate can obtain required
merge evidence without silently resetting its exhausted review/gate budgets.
No work on these later items was started in this recording change.

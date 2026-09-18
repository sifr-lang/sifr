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

## DX.2 merge-validation prerequisite (2026-09-17, authorized)

DX.2 candidate cec479df631badb5a4abcf475f2c9c7cb13c027a failed nine
Python-interop example variants after passing the preceding lane areas. The
owning DX.2 session is explicitly authorized to diagnose and repair only the
demonstrated interpreter/module-loading cause and import-error masking required
for accurate diagnostics. This is a bounded prerequisite to DX.2 acceptance,
not a new Python feature, public API redesign, or DX.3 work. Existing Item12
historical gate/review allowances and failures are unchanged.

The Kafka callback's native build succeeds, then resolving the embedded
kafka_declaration.poll target reports a missing parent-module attribute.
The identical error is reproduced by the preserved pre-DX.2 compiler artifact
5c7502f7aea3cd887a217c2fb3e289c7a1f2c653 (binary SHA-256
ff932673d79912e0659c630a19623b70d5a4b86e9b745570696d427605a2152b).
Compiler inputs from that artifact to the DX.2 base, Python runtime sources,
and Python verification inputs are unchanged. The target resolver currently
discards import exceptions before attempting parent attributes; diagnosis must
retain the actual import failure before changing module loading.

Evidence root: yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx2-evidence/.
Preserve merge-cec479df6.report.json, the full SHA-keyed merge log,
python-gate-failures/area-results.json, all example reports, and
python-gate-failures/base-kafka-disk-receipt.json. Earlier diagnostic attempts
exposed the ambient uv mismatch and /tmp user quota; they remain failed records.
The successful native base reproduction used uv 0.12.10 and an owned disk-backed
temporary directory with Rust 1.98.1.

### Bounded prerequisite validation plan

Before implementing the repair, register these focused checks:
- Runtime object-operations tests, including new regressions for preserving an
  original declaration-target import failure and retaining valid nested-attribute
  resolution: cargo test -p sifr_runtime --features python --lib python::object_ops_tests.
- A focused initialization/module-loading regression for the demonstrated cause
  once the original import error establishes it; record its exact name here.
- Affected example variants: callback-examples, dataframe-examples,
  buffer-examples, arrow-examples, dlpack-examples, ml, libraries,
  async-declaration-examples, and async-context-examples, via the canonical
  Python-interop area runner.
- Required workspace Clippy, formatting, HIR/file-size checks and a review of the
  final changed inputs, followed by one merge-profile gate on the new final SHA.
  Reuse unchanged identity/rebuild evidence; never rerun an unchanged failed gate.

If diagnosis requires a new Python behavior/API redesign or establishes an
unavailable external service/resource, stop with the precise cause and proposed
scope. This record authorizes no broader Python implementation.

### Demonstrated cause and additional focused regression

The preserved executable's ELF dependencies select only the libpython3.14 SONAME,
with no loader path. Linux loads system libpython 3.14.4, while the captured module
paths belong to uv CPython 3.14.7. The latter builds _ssl into libpython; the former
expects a separate extension in its own stdlib tree. Preserving the original
exception exposes ModuleNotFoundError for _ssl. The exact previously failing
binary succeeds without rebuilding when a diagnostic LD_LIBRARY_PATH selects
the probed uv library directory. See python-gate-failures/libpython-loader-mismatch.json.

The bounded repair emits a generated Cargo build script with the selected Python
library directory as a Unix runtime loader path, preserves it for exported Cargo
projects, removes it on rematerialization without Python, and includes the
selection in native artifact identity. No ambient loader variable is required.
Register focused driver regression:
cargo test -p sifr_driver python_native_loader_materialization
It verifies selected-library cache separation, generated script publication and
removal, and unchanged portable export behavior. The real Kafka/affected example
runs must succeed with LD_LIBRARY_PATH unset and readelf/ldd must show selection
of the probed libpython.

### Bounded prerequisite focused results

The nine affected example variants now pass (zero failures), including Kafka,
dataframe, buffer, Arrow, DLPack, ML, library and both async example families.
The runtime object-operations selection passes nine tests, including preservation
of both an import-time RuntimeError and a missing transitive module alongside
successful nested-attribute resolution. The driver materialization regression
passes. Strict workspace Clippy, formatting, HIR and file-size checks pass.

An actual generated Kafka executable succeeds with LD_LIBRARY_PATH unset.
Its ELF RUNPATH names the selected uv Python library directory, and ldd resolves
the matching library rather than system Python. Evidence is in
python-gate-failures/loader-selection-fixed.json, kafka-loader-fixed stdout/stderr,
python-affected-examples.log, python-qualified/, python-target-errors-tests.log,
python-native-loader-tests.log, clippy-python-loader.log, and
guardrails-python-loader.log under the DX.2 evidence root.
The bounded prerequisite merged with DX.2 in [PR #3842](https://github.com/sifr-lang/sifr/pull/3842),
merge 59be2f046f71e0e3ac626f51e8ffd2614573f8bd, final candidate
c6841ab44a6c0e738045586f32e36c5fa1155a26. Final scoped Opus review is
SATISFIED; all 30 Python merge-area variants also passed on the final inputs.
The user's prospective 2026-09-17 Phase DX policy defers the complete broad gate
to phase end; earlier aggregate failures retain their original outcomes. This
closure does not change Item12's historical records or authorize additional Python
features. Final evidence index: dx2-evidence/evidence-index-c6841ab44.json.

## Pre-existing runtime Clippy follow-up observed during DX.9

A supplemental feature-enabled Clippy invocation at DX.9 exposed existing
runtime Python lints in unchanged Arrow/resource/conversion code (mostly
large Result error variants, plus existing arithmetic/return-style lints).
The exact errors are preserved in the external DX.9 evidence directory,
070123a27-scoped-clippy.jsonl and its summary. The new loader's two local
style findings were corrected; the unrelated runtime API design is not
expanded into DX.9. These existing errors remain for this owner and the
required phase-end gate; this is not a full Clippy pass.

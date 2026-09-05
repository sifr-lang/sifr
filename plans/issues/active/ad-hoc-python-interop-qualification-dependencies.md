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

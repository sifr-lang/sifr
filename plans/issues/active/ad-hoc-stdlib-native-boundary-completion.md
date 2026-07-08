# Ad Hoc Phase: Stdlib Native Boundary Completion

## Status

Planned.

## Objective

Finish the stdlib migration from compiler-native intrinsic dispatch, pasted
preambles, fallback signature tables, and handwritten Rust source injection to
the final sysroot architecture:

```text
user code
  -> stdlib/sifr/*.sifr
  -> stdlib/_sifr/*.sifr
  -> @rust(sifr_stdlib.*)
  -> crates/sifr_stdlib
  -> crates/sifr_runtime, only for reusable substrate
```

The completed state is intentionally strict. Public stdlib behavior is checked
Sifr source plus trusted sysroot Rust interop. The compiler remains responsible
for language semantics, Rust interop bridge glue, panic wrappers, exact-int
conversions, entrypoint machinery, generated test harness mechanics, and
runtime call glue. It does not own stdlib behavior.

This phase is sequential. Each milestone is implemented, validated, reviewed,
merged, and documented before the next milestone starts.

## Design Rules

- Public stdlib source lives under `stdlib/sifr`; private declaration source
  lives under `stdlib/_sifr`. Import is source-origin based: only
  `SysrootPublicStdlib` sources may import `SysrootPrivateDeclaration` sources.
  `_sifr.*` is a naming convention and on-disk layout, not the trust boundary.
- `crates/sifr_stdlib` owns stdlib behavior, public wrapper policy,
  Sifr-facing error taxonomy, CPython-compatibility adaptation, and
  module-specific resource semantics.
- `crates/sifr_runtime` owns reusable substrate only: executors, opaque handle
  storage, poisoning, panic boundaries, exact-int bridge helpers, and low-level
  process, network, TLS, HTTP, Python, and runtime state.
- Declaration source is the signature truth for migrated families. Functions,
  constants, methods, errors, opaque resources, and value classes are declared
  in source, not in fallback compiler tables.
- `internal_docs/stdlib_retained_compiler_intrinsics.toml` is the only
  compiler-native stdlib exception ledger.
- Every compiler-native stdlib surface appears exactly once in the retained-glue
  manifest while it is `retained`, `pilot`, or `closing`; only
  `retained-by-design` rows survive final closure. Row granularity is leaf or
  subfamily level when a `_sifr.*` module mixes migrated and retained leaves.
- Manifest states are `retained`, `pilot`, `closing`, and
  `retained-by-design`. `closing` is temporary; final closure removes those rows
  after guards prove the old compiler-native surface cannot reappear.
- The retained-glue manifest is machine-parseable, schema-versioned, and
  rejects unknown fields after M0 installs the schema validator.
- Generated Cargo for stdlib usage emits only Sifr sysroot crates. Third-party
  crates used by stdlib behavior are transitive implementation details of the
  sysroot crates.
- User package dependencies and explicit Rust interop dependencies remain
  package-owned.
- `StdlibCode.module_rust_code` is allowed only as transport for Rust code
  produced from checked Sifr stdlib source. Handwritten stdlib Rust string
  injection is removed before provenance hardening and is forbidden afterward.
- Direct generated `sifr_runtime::*` calls are allowed only for language,
  bridge, entrypoint, exact-int, test-harness, or retained-by-design runtime
  substrate glue. Stdlib behavior routes through `stdlib/_sifr` declarations
  and `sifr_stdlib`.
- No backward-compatibility shims, fallback paths, duplicate registries, or
  compatibility aliases are introduced. Unsupported or removed surfaces receive
  diagnostics instead of hidden fallback behavior.

## Non-Goals

- No public user syntax for private sysroot declarations.
- No CPython top-level module aliasing for `sifr.*`.
- No new global Cargo registry policy for user dependencies.
- No second migration manifest.
- No blanket runtime facade that re-exports all stdlib behavior from
  `sifr_runtime`.
- No parallel implementation tracks. Work proceeds one milestone at a time.

## Implementation Status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0. Model Split and Raw-Injection Removal | planned |  |
| M1. Manifest Schema and Normal-Path Guards | planned |  |
| M2. Declaration Infrastructure and Provenance | planned |  |
| M3. File and Filesystem Migration | planned |  |
| M4. Random, Time, and Logging | planned |  |
| M5. Simple Sys and Environment | planned |  |
| M6. Async Resource Pilot | planned |  |
| M7. Process Family | planned |  |
| M8. Network and TLS Families | planned |  |
| M9. HTTP Family | planned |  |
| M10. Signal Callback and Subscription Pilot | planned |  |
| M11. Python Interop Adapters | planned |  |
| M12. Task, Signal, Runtime Observability, and Test Helpers | planned |  |
| M13. Final Closure | planned |  |

Evidence cells use this format after each milestone lands:
`PR #<n> · sha=<hex7> · manifest: <id old_state -> new_state>`.

## Affected Inventory

Architecture and planning:

- `internal_docs/architecture.md`
- `internal_docs/sifr_sysroot_and_stdlib_architecture.md`
- `internal_docs/rust_interop_architecture.md`
- `internal_docs/stdlib_retained_compiler_intrinsics.toml`
- `plans/issues/active/**`
- `plans/phases/**`

Compiler manifest, sysroot, and dependency planning:

- `crates/sifr_stdlib_model/**`
- target `crates/sifr_stdlib_manifest/**`
- target `crates/sifr_ipc/**`
- `crates/sifr_driver/src/stdlib/**`
- `crates/sifr_driver/src/build/**`
- `crates/sifr_codegen/src/rust_interop_plan.rs`

Lowering and declaration support:

- `crates/sifr_lowering/src/lower/**`
- `crates/sifr_hir/**`
- `crates/sifr_type_system/**`
- `stdlib/_sifr/*.sifr`
- `stdlib/sifr/*.sifr`

Codegen and retained compiler glue:

- `crates/sifr_codegen/src/intrinsics/**`
- `crates/sifr_codegen/src/preamble/**`
- `crates/sifr_codegen/src/stdlib/**`
- `crates/sifr_codegen/src/project/**`
- `StdlibCode.module_rust_code` producers and consumers

Runtime and stdlib crates:

- `crates/sifr_stdlib/**`
- `crates/sifr_runtime/**`

Validation and guardrails:

- `scripts/check_stdlib_native_intrinsic_allowlist.py`
- `scripts/check_stdlib_migration_closure.py`
- target `scripts/check_stdlib_manifest_schema.py`
- target `scripts/check_stdlib_bootstrap_ordering.py`
- `scripts/check_sysroot_stdlib_resource_certification_gate.py`
- `scripts/check_hir_maintainability_guardrails.py`
- `scripts/check_file_size_guardrails.py`
- `verification/areas/**`
- `verification/runner/e2e/**`

## Milestones

### M0. Model Split and Raw-Injection Removal

Remove large special cases before hardening the final manifest schema.

Tasks:

- Create or rename the compiler manifest crate as `sifr_stdlib_manifest`.
- Move source inventory, private declaration inventory, feature planning,
  sysroot validation, and import policy into `sifr_stdlib_manifest`.
- Move shared IPC frame/schema/transport/request tracking/handshake metadata
  into `sifr_ipc`.
- Move legacy CPython-shaped import suggestion policy out of the manifest crate
  into the frontend or diagnostics boundary.
- Move remaining retained intrinsic signature builders out of the manifest
  crate and into the compiler codegen boundary that consumes them.
- Delete the synthetic `sifr.http_transport` module and its handwritten
  `HTTP_TRANSPORT_HARNESS_RUST` literal from compiler stdlib output.
- Rebuild the equivalent HTTP transport probe as a verification-owned Rust
  fixture that is not exposed via `sifr.*`.
- Update architecture docs with the final stdlib/compiler/runtime boundary and
  the source-origin privacy rule.
- Remove temporary `sifr.http_transport` prose from
  `internal_docs/network_http_architecture.md` after the verification-owned
  fixture replaces it.

Acceptance:

- The compiler manifest crate contains metadata and planning only, not IPC
  protocol code, diagnostics policy, retained intrinsic signatures, or stdlib
  behavior.
- `sifr_ipc` owns shared IPC protocol/frame/schema/request-tracking code.
- Legacy CPython-shaped import suggestion policy lives outside the stdlib
  manifest crate.
- Remaining retained intrinsic signatures live at the codegen boundary that
  consumes them, not in the manifest crate.
- No handwritten Rust stdlib module injection remains in `StdlibCode`.
- HTTP transport verification exists as a verification-owned fixture, not as a
  synthetic `sifr.*` module.
- `_sifr.*` privacy is documented and implemented as source-origin policy, not a
  bespoke import-prefix trust rule.

Validation:

- Focused manifest/sysroot unit tests.
- Focused HTTP verification fixture tests.
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr`

### M1. Manifest Schema and Normal-Path Guards

Make the normal path strong enough that transitional special cases cannot grow.

Tasks:

- Extend the retained-glue manifest schema with migration states, owner or
  issue fields, evidence links, registry entries, preamble entries,
  declaration files, and certification rows.
- Do not add raw-injection fields; handwritten stdlib Rust injection was removed
  in M0.
- Exact-enumerate current prefix dispatchers and delete `prefix_intrinsics` from
  the manifest schema and allowlist guard.
- Split mixed-family rows into leaf or subfamily rows, for example
  `_sifr.collections::counter_defaultdict` rather than `_sifr.collections`.
- Split the mixed IO/logging/random preamble row into leaf rows so no retained
  entry owns unrelated behavior.
- Classify every retained surface as `retained`, `pilot`, `closing`, or
  `retained-by-design`.
- Treat `closing` as a temporary closeout state. Once deletion evidence lands,
  remove the row; Git history and PR links are the audit trail.
- Reject unknown top-level and per-surface manifest fields in
  `scripts/check_stdlib_manifest_schema.py`; the validator owns one explicit
  allowed-field set.
- Add validation that every manifest entry has a valid state and every observed
  compiler-native stdlib exception is owned exactly once.
- Add validation that manifest state transitions are monotonic by comparing the
  current manifest with `main`.
- Migrate `scripts/check_sysroot_stdlib_resource_certification_gate.py` to read
  `certification_rows` from the manifest and delete its hardcoded
  surface-to-matrix table. Each manifest row carries its own certification rows;
  the gate does not derive ownership from `_sifr.*` family prefixes.
- Add deterministic topological stdlib bootstrap ordering.
- Reject cycles and forward references that would make private declarations
  order-dependent or nondeterministic.
- Add a standalone bootstrap-ordering guard script.
- Remove fallback intrinsic signature tables for any surface already marked
  `closing`.
- Ensure codegen, driver, LSP traces, build reports, and cache keys consume one
  `SysrootDependencyPlan` rather than recomputing sysroot features.

Acceptance:

- A reviewer can identify the owner and migration state for every retained
  compiler-native stdlib surface from one manifest.
- No second registry or informal checklist is needed to understand migration
  state or resource certification state.
- `retained-by-design` entries are narrow compiler-language or harness glue,
  not stdlib behavior.
- There are no prefix concepts in the manifest schema.
- Unknown manifest fields are rejected.
- The resource certification gate consumes manifest `certification_rows`
  instead of a parallel table and does not infer certification by prefix.
- Stdlib bootstrap order is deterministic and validated.
- Migrated families cannot keep compiler fallback signatures.

Validation:

- `python3 scripts/check_stdlib_manifest_schema.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_stdlib_migration_closure.py --self-test`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
- Focused driver stdlib bootstrap tests.
- `python3 scripts/check_stdlib_bootstrap_ordering.py`
- Generated Cargo snapshot tests.
- `cargo test -p sifr_driver -- stdlib`
- `cargo test -p sifr_codegen -- rust_interop_plan`
- `scripts/run_all_tests.sh --profile create-pr`

### M2. Declaration Infrastructure and Provenance

Make source declarations expressive enough to replace compiler signature tables,
and lock codegen so new stdlib behavior cannot enter through side channels.

Tasks:

- Add checked declaration support for constants used by private stdlib modules.
- Add opaque resource declarations with nonforgeability and lifecycle metadata.
- Add value class declarations for native-backed values that are not resources.
- Add declaration support for methods, associated constructors, errors, and
  module-level constants.
- Move math and platform constants out of intrinsic constant Rust expressions
  where they are stdlib behavior rather than language facts.
- Add diagnostics for unsupported declaration forms instead of implicit
  fallbacks.
- Add structural provenance to `StdlibCode.module_rust_code` producers.
- Replace the plain string payload with compiled-source provenance:

  ```rust
  struct StdlibRustSource {
      module: String,
      source_path: PathBuf,
      rust: String,
  }
  ```

- Permit compiled checked stdlib source output.
- Normalize `StdlibRustSource.source_path` to the same repo-relative path form
  used by manifest `declaration_files`.
- Reject handwritten stdlib Rust literals outright.
- Guard against new stdlib intrinsic dispatch entries.
- Guard against new stdlib implementation preambles.
- Guard against new direct third-party generated dependencies for stdlib
  behavior.
- Guard against empty private declaration modules once marked `closing`.
- Guard against private declaration targets outside approved sysroot crates.
- Guard against fallback signature registries for `closing` surfaces.
- Guard direct generated `sifr_runtime::*` calls so stdlib behavior cannot enter
  through runtime glue.

Acceptance:

- Functions, constants, methods, errors, opaque resources, and value classes can
  be represented in checked declaration source.
- Private declaration files can express the shape needed by upcoming pilots.
- Migrated constants do not require compiler intrinsic Rust expressions.
- The compiler can still emit language and bridge glue.
- New stdlib behavior cannot enter through codegen strings, preambles,
  intrinsic dispatch, direct generated dependencies, or fallback registries.
- `StdlibCode.module_rust_code` has one producer kind: compiled checked Sifr
  source.

Validation:

- Focused lowering/type-system declaration tests.
- Focused Rust interop declaration tests.
- Representative `sifr emit` snapshots for constants, methods, and opaque
  declarations.
- Guard self-tests for each forbidden path.
- Negative tests for raw stdlib Rust injection.
- Generated Cargo dependency snapshots.
- `scripts/run_all_tests.sh --profile create-pr`

### M3. File and Filesystem Migration

Migrate file handles and the adjacent filesystem/path family end to end.

Tasks:

- Implement the reusable opaque handle substrate needed by file handles in
  `sifr_runtime`.
- Implement file behavior and Sifr-facing errors in `crates/sifr_stdlib`.
- Declare `FileHandle` and related operations in `stdlib/_sifr`.
- Route public `sifr.io` wrappers through private declarations.
- Migrate `builtin_open`, `builtin_open_text`, `open_file`, `file_read`,
  `file_write`, close, byte operations, and related methods.
- Move path, directory, glob, temporary directory, file copy, delete, rename,
  walk, metadata, and text helpers into `sifr_stdlib`.
- Declare filesystem functions and errors in `stdlib/_sifr/fs.sifr`.
- Route `sifr.os`, `sifr.pathlib`, `sifr.glob`, `sifr.shutil`,
  `sifr.tempfile`, and related wrappers through declarations.
- Delete the direct `regex` dependency emitted for `sifr.pathlib`; route
  path-glob regex behavior through `sifr_stdlib` behind the sysroot boundary.
- Prove handle nonforgeability, close poisoning, double-close behavior, and
  panic-boundary conversion.
- Delete migrated file, filesystem, path, and IO registry, preamble, and
  signature entries.
- Keep only explicitly retained compiler-language or bridge glue.
- Mark the relevant manifest entries `pilot` during implementation and delete
  them after migration evidence lands.

Acceptance:

- File handles are source-declared opaque resources.
- Public file and filesystem APIs behave through `sifr_stdlib`, not compiler
  dispatch.
- Generated Cargo emits sysroot crate features only.
- No migrated file/filesystem operation remains in intrinsic dispatch.

Validation:

- Focused file/resource lifecycle tests.
- Focused fs/path tests.
- Representative stdlib demos.
- Generated Cargo feature snapshots.
- Resource certification gate.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M4. Random, Time, and Logging

Migrate stateful but non-handle families after resource declaration patterns are
established.

Tasks:

- Move random module state, distributions, sampling, shuffling, and seeding
  behavior into `sifr_stdlib`.
- Move clock, formatting, parsing, sleeping, and struct-time compatibility
  behavior into `sifr_stdlib`, using runtime substrate where needed.
- Move global logging level behavior into `sifr_stdlib` or classify the exact
  runtime observability substrate as `retained-by-design`.
- Split `_sifr.time` and `_sifr.logging` rows into migrated public behavior and
  retained-by-design substrate rows if either family keeps compiler/runtime
  substrate after public wrappers move.
- Delete the remaining logging/random slices from the mixed
  IO/logging/random preamble after M3 has removed file/filesystem slices.
- Delete `random`, `time`, `logging`, and mixed preamble entries that are no
  longer compiler-owned.

Acceptance:

- Random/time/logging public behavior routes through checked stdlib source and
  sysroot interop.
- Runtime-only substrate is narrow and explicitly classified.
- The remaining random/logging slices of the mixed IO/logging/random preamble
  are decomposed or deleted after M3 has already removed file/filesystem slices.

Validation:

- Deterministic random tests.
- Clock/sleep tests with bounded timing assertions.
- Logging state tests.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M5. Simple Sys and Environment

Migrate process environment and platform surfaces that do not require process
resource handles.

Tasks:

- Move environment get/set/unset/items/keys/values into `sifr_stdlib`.
- Move `get_args`, `getpid`, `which`, platform constants, separators, and
  version strings into source declarations or classify them as
  `retained-by-design` only when they are compiler facts.
- Decide and document whether `sys_exit` is public stdlib behavior or
  language/runtime termination glue.
- Delete migrated environment and sys intrinsic entries.

Acceptance:

- Simple sys/env behavior is not compiler-dispatched.
- Any retained sys entry has a precise compiler-owned reason.
- Platform constants use declaration source where they are stdlib values.

Validation:

- Focused sys/env tests.
- Generated Cargo feature snapshots.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M6. Async Resource Pilot

Prove async resource declarations before migrating async-heavy families.

Tasks:

- Add or harden async resource declaration support.
- Prove cancellation, poisoning, join/drop semantics, blocking boundaries, and
  panic conversion for an intentionally small async resource.
- Update Rust interop certification evidence with executable lifecycle tests.
- Keep the pilot isolated from process, network, TLS, and HTTP migration until
  evidence lands.

Acceptance:

- Async resources have a checked declaration and lifecycle model.
- Cancellation and drop behavior is deterministic and documented.
- Resource certification can distinguish sync and async resource evidence.

Validation:

- Focused async resource tests.
- Rust interop certification gate.
- `scripts/run_all_tests.sh --profile create-pr`

### M7. Process Family

Migrate process execution, child handles, pipes, and async process behavior.

Tasks:

- Implement process behavior and Sifr-facing errors in `sifr_stdlib`.
- Keep low-level spawn, pipe, timeout, and async substrate in `sifr_runtime`.
- Declare child handles, pipe handles, async child handles, and operations in
  `stdlib/_sifr/process.sifr`.
- Route public process wrappers through private declarations.
- Delete process registry files and preambles after migration.
- Prove timeout, kill, terminate, wait, output, pipe close, pipe read/write,
  and async lifecycle semantics.

Acceptance:

- Process behavior is stdlib-owned.
- Runtime owns only substrate.
- No migrated process operation remains in intrinsic dispatch or preambles.

Validation:

- Focused process lifecycle tests.
- Async process tests.
- Resource certification gate.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M8. Network and TLS Families

Migrate TCP and TLS resources after async/process evidence is in place.

Tasks:

- Implement network and TLS behavior in `sifr_stdlib`.
- Keep socket, reactor, TLS engine, certificate, and poisoning substrate in
  `sifr_runtime`.
- Declare listener, stream, split-half, TLS config, and TLS stream resources in
  `stdlib/_sifr`.
- Route public `sifr.net` and `sifr.tls` wrappers through declarations.
- Delete network/TLS registry and preamble entries after migration.
- Prove local/peer address, close, split, shutdown-write, read/write,
  handshake, certificate, and error semantics.

Acceptance:

- Network and TLS public behavior is stdlib-owned.
- Runtime substrate is narrow and reusable.
- No migrated network/TLS operation remains in intrinsic dispatch or preambles.

Validation:

- Focused loopback network tests.
- TLS fixture tests.
- Resource certification gate.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M9. HTTP Family

Migrate HTTP behavior after network/TLS resource evidence exists.

Tasks:

- Implement HTTP behavior in `sifr_stdlib`.
- Keep HTTP client runtime substrate in `sifr_runtime` only where reusable.
- Declare HTTP resources, headers, requests, responses, and protocol operations
  in `stdlib/_sifr/http.sifr`.
- Delete HTTP prefix dispatch, registry entries, and preambles after migration.
- Prove header validation, cookie parsing, HTTP/1, HTTP/2, transport, timeout,
  redirect, body, and error semantics.

Acceptance:

- HTTP behavior is stdlib-owned.
- Test harness behavior is not emitted as stdlib implementation code.
- No HTTP prefix intrinsic remains in active dispatch.

Validation:

- Focused HTTP tests.
- Verification-owned transport fixtures.
- Generated Cargo dependency snapshots.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M10. Signal Callback and Subscription Pilot

Prove callback-shaped interop with the signal subscription surface before
Python closure.

Tasks:

- Add or harden callback/subscription declaration support for signal
  subscriptions.
- Prove callback lifetime, cancellation, thread-safety, panic conversion,
  reentrancy policy, and drop behavior.
- Update resource and callback certification evidence.
- Keep pilot evidence separate from Python migration until complete.

Acceptance:

- Callback and subscription resources have checked declaration semantics.
- Runtime callback substrate is explicitly bounded.
- Certification evidence blocks unsafe callback migration attempts.

Validation:

- Focused callback lifecycle tests.
- Rust interop callback certification gate.
- `scripts/run_all_tests.sh --profile create-pr`

### M11. Python Interop Adapters

Migrate Python adapter stdlib behavior while preserving Python runtime ownership.

Tasks:

- Move Python object wrapper policy and Sifr-facing adapter behavior into
  `sifr_stdlib` where it is public stdlib behavior.
- Keep CPython initialization, GIL/refcount substrate, thread handoff, and raw
  callback machinery in the Python runtime layer.
- Declare Python adapter operations in `stdlib/_sifr/python.sifr`.
- Delete migrated `py_` prefix dispatch and local/threadsafe callback entries.
- Prove callback, resource, zero-copy, blocking/offload, and error semantics.

Acceptance:

- Python stdlib adapter behavior is source-declared and stdlib-owned.
- Runtime owns CPython substrate only.
- No migrated Python adapter operation remains in intrinsic dispatch.

Validation:

- Python interop certification tests.
- Callback/resource gates.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M12. Task, Signal, Runtime Observability, and Test Helpers

Finish the compiler/runtime boundary for surfaces that may stay partly
compiler-owned.

Tasks:

- Reconfirm task, runtime observability, and test-helper surfaces as
  `retained-by-design` only where they are language, bridge, or harness glue.
- Classify any remaining signal public-wrapper behavior after M10 as `closing`
  or `retained-by-design`.
- Move public stdlib wrapper behavior into `sifr_stdlib`.
- Keep language task machinery, generated test harness mechanics, exact-int
  conversions, entrypoint wiring, and shared bridge glue compiler-owned.
- Delete any task/signal/runtime/test intrinsic that is not retained by design.
- Split or remove remaining preambles so by-design glue is not mixed with
  stdlib behavior.

Acceptance:

- Retained-by-design entries are precise and stable.
- Public stdlib behavior is not bundled with compiler harness or bridge glue.
- Mixed preamble files are gone or contain only retained-by-design language
  glue.

Validation:

- Focused task/signal/runtime observability tests.
- Test harness tests.
- Retained-glue and migration closure guards.
- `scripts/run_all_tests.sh --profile create-pr`

### M13. Final Closure

Remove the transitional migration machinery.

Tasks:

- Delete remaining stdlib intrinsic dispatch and registry code that no longer
  represents retained-by-design language glue.
- Delete fallback intrinsic signature tables for all `closing` surfaces.
- Delete stdlib implementation preambles.
- Delete direct generated third-party dependency paths for stdlib behavior.
- Ensure every remaining manifest entry is `retained-by-design`; no `retained`,
  `pilot`, or `closing` rows remain.
- Convert temporary migration guards into permanent no-regression guards.
- Update architecture, roadmap, phases, and issue docs with final status and
  merged PR links.

Acceptance:

- The stdlib implementation boundary matches the final stack exactly.
- The compiler has no stdlib behavior implementation path.
- All remaining compiler-owned entries are intentional language, bridge,
  entrypoint, exact-int, test-harness, or runtime substrate glue.
- The retained-glue manifest is exhaustive and contains only
  `retained-by-design` rows.
- `StdlibCode.module_rust_code` provenance is a single compiled-from-Sifr source
  struct, not an enum with temporary exemption variants.

Validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

## Per-Milestone Closeout Rules

Each milestone must finish with:

- docs updated with status, checklist state, and merged PR links,
- focused tests for the changed compiler/stdlib/runtime behavior,
- guardrail updates for any newly closing or deleted path,
- `scripts/run_all_tests.sh --profile create-pr` before PR,
- review and merge before starting the next milestone.

The final milestone additionally runs the full merge gate with
`scripts/run_all_tests.sh`.

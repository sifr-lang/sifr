# Ad Hoc Phase: Declaration-First Python Interop

## Status

Proposed. Architecture and milestone planning are complete and Opus review pass
3 has no blocking findings. No declaration syntax or implementation described
here is currently supported.

## Objective

Add a declaration-first package-authoring layer over Sifr's existing embedded
Python runtime so package consumers call ordinary typed Sifr APIs instead of
manually operating `py.Object` handles.

The phase must preserve the implemented environment, trust, error, blocking,
sendability, callback, resource, and zero-copy contracts. It solves the root
ergonomic problem through checked declarations, compiler-owned object
lifetimes, and package-local Python bridges rather than weakening the boundary
or adding Python source compatibility.

The durable proposed contract lives in
[`internal_docs/python_interop_declaration_architecture.md`](../../../internal_docs/python_interop_declaration_architecture.md).

## Core Decisions

- `@python(...)` declarations are the normal package-authoring surface.
- `sifr.python` remains an explicit dynamic escape hatch.
- Decorator targets are structured dotted paths with `import-root`, `bridge`,
  and `Self` resolution; string targets are rejected.
- The Sifr declaration signature is authoritative for argument and return
  conversion. Decorators do not repeat conversion types.
- Declaration bodies are ellipsis-only stubs.
- Typed Python objects use sealed compiler-owned opaque handles and automatic
  reference release.
- Ordinary reference drop is distinct from semantic `close`, `aclose`,
  context-manager exit, buffer release, capsule release, and callback shutdown.
- APIs that do not map directly use hermetic package-local Python bridge modules
  under `src/python_bridges/`.
- Static targets and bridge imports infer package import requirements; root
  execution and native-extension trust remain explicit.
- Initial Python declarations have an implicit `blocking_io` effect. There is
  no hidden offload or event loop.
- `.pyi`, `py.typed`, and runtime introspection assist declaration generation
  but never override checked-in Sifr binding contracts.
- Compatibility claims require executable positive and negative evidence.

## Non-Goals

- Python source compatibility or dynamic dot access as the primary API.
- Sifr `Any`, silent `Object` fallback, or implicit deep conversion.
- Automatic environment installation, synchronization, or trust mutation.
- Whole-library automatic wrapper generation.
- Fallible class-constructor syntax in this phase; initial bindings use factory
  functions returning `Result`.
- Hidden async scheduling or coroutine/event-loop integration.
- A general decorator converter pipeline.
- Replacing existing explicit Arrow, DLPack, buffer, callback, or resource
  ownership rules.

## Scope

Compiler and package surfaces expected to change during implementation:

- `crates/sifr_ir/src/` for Python interop declaration metadata;
- `crates/sifr_lowering/src/lower/` for decorator and ownership checking;
- `crates/sifr_codegen/src/` for typed Python wrapper generation;
- `crates/sifr_driver/src/build/` for Python target probes and build plans;
- `crates/sifr_package/src/python/` for inferred requirements, environment,
  trust, and binding package metadata;
- `crates/sifr_runtime/src/python/` for sealed owned handles and generated
  conversion support;
- `crates/sifr_diagnostics/` for active declaration diagnostic families;
- `stdlib/sifr/python*.sifr` for the raw escape hatch over the same ownership
  representation;
- `verification/areas/python_interop/` for executable compatibility evidence;
- CLI/LSP surfaces for `sifr python check`, `doctor`, and symbol-selective
  binding generation after the declaration grammar stabilizes.

## Milestones

### M0. Contract Lock And Verification Scaffold

Tasks:

- Review and accept the declaration-first architecture document.
- Define the supported/future/unsupported compatibility categories.
- Add a machine-readable declaration capability matrix separate from broad
  package inventory.
- Inventory the existing runnable Python examples and classify which assertions
  prove runtime execution versus source or matrix presence only.
- Reserve `SIFR-PYIMP-0001`, `SIFR-PYCALL-0001`, `SIFR-PYCONV-0001`,
  `SIFR-PYRES-0001`, `SIFR-PYZC-0001`, and `SIFR-PYCB-0001` with the meanings
  defined by the architecture.
- Reserve `SIFR-PYTRUST-0005` for a required static import root not authorized
  by the root application, without repurposing existing diagnostic meaning.
- Add stale-draft checks rejecting deprecated prototype syntax such as string
  targets and decorator-level `returns=` converter declarations.
- Lock bridge-version-1 argument passing: regular parameters become positional
  Python arguments, Sifr keyword-only parameters become named Python kwargs,
  defaults are passed explicitly, and variadics/kwargs expansion are rejected.
- Lock the manifest compatibility sequence before implementation: inferred
  requirements coexist with current allow/trust validation until a later
  atomic removal milestone.

Acceptance:

- Reviewers can distinguish implemented raw interop from proposed declaration
  syntax without reading compiler code.
- Every planned capability has a positive and negative evidence owner.
- The matrix cannot label inventory-only evidence as declaration support.
- Current raw behavior and future declaration behavior are not conflated.

Documentation-only validation:

- Markdown link and heading review.
- `git diff --check`.
- Targeted `rg` checks for rejected syntax and contradictory implemented-status
  claims.

### M1. Declaration IR, Direct Calls, And Owned References

Tasks:

- Parse `@python(path)` into structured declaration metadata with source spans.
- Accept ellipsis only as the complete body of eligible Python declarations.
- Resolve static import roots and reject string or dynamic decorator targets.
- Add a `PythonInteropPlan` to generated project metadata.
- Add one compiler-recognized sealed Python foreign-handle kind with no
  Sifr-visible token fields.
- Migrate raw `sifr.python.Object` and generated declaration values to the same
  runtime handle representation; do not retain a second public token model.
- Implement detach-before-decref, attached immediate release, the runtime-owned
  pending-release queue, attach-time draining, and generated epilogue draining.
- Generate direct scalar function/factory wrappers from Sifr signatures.
- Lower regular arguments positionally and keyword-only arguments as kwargs;
  reject `*args`, `**kwargs`, and implicit record expansion.
- Map Python exceptions and conversion failures into structured `PythonError`.
- Infer the `blocking_io` effect and enforce async call-site offload rules.
- Infer import requirements and validate explicit root trust.

Acceptance:

- A direct scalar binding can be checked, built, and run without raw
  `py.Object` operations.
- Ordinary returned Python values cannot leak through success, Python failure,
  conversion failure, or early return.
- Unsupported types and targets fail as Sifr diagnostics before Rust build.
- Direct calls from async code remain rejected unless explicitly offloaded.
- A drop cannot hold the object-store lock while Python decref or `__del__`
  code runs.

Validation:

- Lowering and codegen unit tests for accepted and rejected decorator forms.
- Runtime ownership tests with outstanding-reference assertions.
- Success, Python failure, conversion failure, early return, detached-thread
  drop, and reentrant-callback drop fixtures.
- Executable pure-Python and native-extension fixtures.
- Focused create-PR validation for the touched compiler/runtime packages.

### M2. Opaque Classes, Methods, Attributes, And Items

Tasks:

- Implement `@python.opaque` with `close`, `send`, and type target metadata.
- Implement `Self` method target resolution.
- Implement fallible `@python.attr` descriptor/property access.
- Implement fallible `@python.item` access.
- Enforce borrow/move/use-after-close rules for opaque handles.
- Keep initial construction as explicit factory functions returning `Result`.
- Require factory results to pass `isinstance` against the declared opaque
  Python type, accepting subclasses.
- Record probe results as verified or runtime-checked; reject only targets
  proved absent/incompatible and never claim static proof for an uninspectable
  instance attribute.
- Activate targeted `PYCALL`, `PYCONV`, and `PYRES` diagnostics.

Acceptance:

- A Python-backed object can expose typed methods, attributes, and item access
  without exposing structural handle fields.
- Attribute descriptors that raise return `PythonError`.
- Automatic drop and semantic close are distinct and independently verified.
- Non-send opaque objects cannot cross Sifr task/thread boundaries.

Validation:

- Positive and negative opaque lifecycle fixtures.
- Descriptor/property failure fixtures.
- Method receiver, moved value, double close, and use-after-close tests.
- A fixture that creates an opaque object, fails mid-flow, and proves ordinary
  reference count returns to zero.
- Runnable biip/schwifty or equivalent object-shaped example.

### M3. Typed Containers And Records

Depends on M1's sealed handle representation.

Tasks:

- Add recursively checked list, tuple, `dict[str, T]`, option, and closed-record
  conversion from authoritative Sifr signatures.
- Preserve the canonical record mapping: Sifr records construct Python dicts;
  Python record extraction requires every declared field, tries attribute
  access before string-key access, and ignores extra fields.
- Reject implicit zero-copy, unsupported keys, `Any`, unconstrained generics,
  iterators, generators, and uncontracted callables.

Acceptance:

- Missing record fields and nested conversion failures report exact boundary
  paths; extra Python fields are ignored deliberately.
- Container/record returns are checked copies and never claim zero-copy.
- Unsupported shapes require a future explicit bridge or remain dynamic-only.

Validation:

- Direct conversion matrices with positive and negative execution.
- Nested overflow/type/path failure fixtures.
- Record attribute, item fallback, missing-field, and extra-field fixtures.
- Zero-reference-leak assertions for partial conversion failure.

### M4. Hermetic Package-Local Python Bridges

Depends on M1's sealed handle representation and M3's typed conversion
contract.

Tasks:

- Resolve `bridge.*` to package-owned source under `src/python_bridges/`.
- Derive the runtime namespace
  `__sifr_bridge__.p_<resolved_package_key>.<module_path>`.
- Add `sifr_runtime::python::bridge_loader`, installed before user `main`, as a
  first-position `MetaPathFinder`/loader over an embedded UTF-8 source table.
- Reject reserved-namespace collisions in `sys.modules` and prohibit
  filesystem/`sys.path` fallback for reserved modules.
- Syntax-check and statically inventory ordinary bridge imports; reject dynamic
  import calls in bridge version 1.
- Infer bridge import requirements without inferring execution or native trust.
- Include bridge source, package identity, distribution versions, interpreter
  ABI, and binding contracts in cache fingerprints.
- Include bridge sources and inventory in Sifr package archives and embed the
  resolved graph's bridge table into generated binaries.

Acceptance:

- Two packages may own the same `bridge.identifiers` source path without a
  runtime module collision.
- Bridge deployment does not depend on the source checkout, a writable temp
  directory, or ambient `sys.path` ordering.
- A dependency bridge cannot authorize its own Python or native imports.
- Dynamic bridge imports fail during declaration checking rather than escaping
  static requirement inventory.

Validation:

- Loader ordering, namespace collision, sibling module, and traceback filename
  tests.
- Multi-package same-module-name fixture.
- Package archive/install/run fixture proving embedded bridge resolution.
- Cache invalidation tests for source, package, distribution, ABI, and contract
  drift.
- Static and rejected dynamic import fixtures.

### M5. Ecosystem Example Migration

Tasks:

- Migrate runnable biip/schwifty, dataframe, ML, web, database, cloud, crypto,
  and Redis examples to direct declarations or package-local bridges.
- Keep an intentionally small raw-object example proving the escape hatch.
- Assert zero outstanding ordinary Python references after success and each
  exercised failure path.
- Update capability evidence without promoting inventory-only package rows.

Acceptance:

- Package consumers use no raw handles for migrated binding surfaces.
- The merge profile executes all migrated offline examples.
- Compatibility categories match actual positive and negative evidence.

Validation:

- Migrated dataframe, ML, and library example suites.
- Negative version/shape/conversion cases for each supported declaration kind.
- Outstanding-reference assertions in every migrated executable.

### M6. Environment Discovery And Manifest Authority Migration

Tasks:

- Add uv-compatible project, lock, environment, and interpreter discovery with
  explicit non-standard-layout overrides.
- Establish real project/lock consistency checking instead of readability-only
  metadata checks.
- Keep `[python].requires-imports` for underivable raw/dynamic library needs and
  merge it with compiler-derived declaration/bridge requirements.
- Make `[trust].python` and `[trust].python-native` root-owned authorizations;
  dependency packages cannot self-authorize.
- Remove `[python].allow-imports` atomically from parsing, public/internal docs,
  generated manifests, examples, and every verification fixture.
- Retire `SIFR-PYTRUST-0002` with its old meaning and activate
  `SIFR-PYTRUST-0005` for a required root not authorized by the root.
- Preserve root-only wildcard trust for local control and dependency wildcard
  rejection.

Acceptance:

- A normal uv project needs no repeated default `.venv`, interpreter, project,
  or lock paths.
- No dependency requirement grants execution or native-extension trust.
- No stale docs, fixtures, or diagnostics retain the removed source allowlist.
- Sifr never installs, syncs, or mutates trust during check/build/run.

Validation:

- uv default, override, workspace, missing environment, and stale-lock fixtures.
- Root/dependency trust authority positive and negative tests.
- Manifest stale-draft sweep and diagnostic registry/docs regeneration.
- Migration of all Python interop runner-generated manifests.

### M7. Read-Only Check And Doctor Workflow

Tasks:

- Add `sifr python check` for environment, lock, trust, declaration, bridge,
  probe-confidence, and target validation.
- Add `sifr python doctor` with patch-like suggestions for missing environment,
  requirement, execution-trust, and native-trust entries.
- Reuse the same package/driver plan as normal `sifr check`; do not create a
  second validator.
- Keep both commands read-only and deterministic.

Acceptance:

- Check results match normal build/check diagnostics for the same package.
- Doctor explains root authority and never applies a patch or runs uv.
- Runtime-checked targets are visible without being falsely reported verified.

Validation:

- CLI integration tests and deterministic doctor goldens.
- Cross-command diagnostic parity fixtures.
- Explicit environment/trust non-mutation checks.

### M8. Symbol-Selective Binding Generation

Tasks:

- Add symbol-selective `sifr python bind` scaffold generation from user
  overrides, stub-only packages, `py.typed` packages, approved fallback stubs,
  and safe runtime introspection in that precedence order.
- Reject or emit explicit unresolved markers for `Any`, bare `object`,
  `Callable[..., Any]`, unsupported overloads/generics, dynamic attributes, and
  uninspectable unsupported shapes.
- Record SOABI, distribution version, source-kind precedence, and consumed stub
  hashes as the binding-source fingerprint.
- Add `sifr python bind --check` as a read-only fingerprint drift check.
- Never silently generate `py.Object` as a fallback.

Acceptance:

- Generated declarations are reviewable checked-in Sifr source.
- Unsupported type information cannot produce a falsely typed binding.
- `--check` detects source/version/stub drift without rewriting files.

Validation:

- Stub-only, `py.typed`, partial-stub, C-extension, overload, `Any`, bare
  `object`, and callable fixtures.
- Golden generated declarations and fingerprint drift tests.
- Read-only `--check` verification.

### M9. LSP Binding Authoring Support

Tasks:

- Add completion for decorator kinds, resolved target paths, and policy values.
- Add navigation from a Sifr declaration to local bridge source and available
  Python typing source.
- Surface declaration diagnostics and verified/runtime-checked probe status.
- Reuse compiler/package query results rather than invoking Python from editor
  request handlers ad hoc.

Acceptance:

- Editor results agree with `sifr python check` for the same package snapshot.
- Completion never offers unsupported fallback types as certified bindings.

Validation:

- LSP completion, navigation, diagnostic, cache-invalidation, and cancellation
  fixtures.

### M10. Advanced Protocol Declaration Designs

Tasks:

- Specify declaration metadata for context managers, local/threadsafe
  callbacks, buffers, Arrow capsules, and DLPack tensors independently.
- Preserve existing lifetime, release, one-shot consumption, backpressure,
  shutdown, dtype/device/shape, and no-copy-fallback contracts.
- Reserve syntax only after each positive and negative ownership model is
  accepted.
- Keep Python coroutine/event-loop integration future unless a separate design
  proves cancellation, affinity, and returned-object sendability.

Acceptance:

- Each proposed protocol decorator has an unambiguous ownership state machine,
  error mapping, diagnostic family, and verification owner.
- No protocol is implemented merely as closeout plumbing.

Validation:

- Documentation review and state-machine walkthroughs.
- Positive/negative fixture plans tied to compatibility rows.

### M11. Advanced Protocol Integration And Closeout

Tasks:

- Implement only the accepted M10 protocol declarations and migrate their
  existing raw examples without weakening behavior.
- Keep raw API conveniences over the same sealed handle and conversion model;
  optional scope helpers must not introduce fallback ownership semantics.
- Update public docs so declaration-first use is primary and raw handles are
  documented as intentional advanced/dynamic interop.
- Require compiled Sifr execution for Redis and Postgres round trips, Kafka
  publish/consume, direct SQS send/receive, and SNS-to-SQS delivery; distinguish
  any intentionally client-driven evidence explicitly.
- Run final architecture and implementation review rounds until no actionable
  blockers remain.
- Update phase status, roadmap, compatibility documentation, and merged PR
  evidence.

Acceptance:

- Supported advanced protocols retain explicit copy, ownership, release,
  callback, and shutdown semantics.
- Public examples are declaration-first unless demonstrating raw interop.
- Named live cases invoke actual compiled Sifr binaries.
- Support statements match executable evidence and review has no blockers.

Validation:

- Focused protocol positive/negative suites.
- Named compiled-Sifr live service/callback cases.
- Documentation and diagnostic link validation.
- Authoritative create-PR and merge profiles before implementation closeout.

## Verification Policy

- Create-PR must run at least one real pure-Python declaration example and one
  native-extension declaration example after M1.
- Merge must run the migrated offline dataframe, ML, and library examples.
- Live service evidence must distinguish actual compiled Sifr execution from
  Python-client execution or source-presence checks.
- Every supported declaration capability requires a passing negative fixture
  for its primary failure mode.
- Matrix-only package inventory remains useful discovery evidence but cannot
  certify declaration behavior.
- Outstanding-reference diagnostics must be asserted after success and every
  failure path that creates Python values.

## Review Checklist

- [ ] Current raw interop and proposed declaration syntax are clearly separated.
- [ ] The Sifr signature is the only conversion type declaration.
- [ ] String targets and converter pipelines are rejected.
- [ ] Fixed positional/keyword-only argument mapping is explicit; variadics and
      kwargs expansion are rejected in bridge version 1.
- [ ] Verified and runtime-checked target probe outcomes are distinguished.
- [ ] Automatic reference drop is distinct from semantic resource close.
- [ ] Raw `Object` and typed opaque values use one sealed runtime handle model.
- [ ] Local bridges are hermetic package inputs.
- [ ] The reserved bridge namespace and embedded loader prevent package and
      `sys.path` collisions.
- [ ] Imports are inferred and trust remains explicit.
- [ ] `allow-imports` removal retires its old diagnostic meaning and migrates
      parsing, docs, generated manifests, and fixtures atomically.
- [ ] No build/check path installs or syncs Python packages.
- [ ] Blocking behavior is explicit and no event loop is hidden.
- [ ] Stub generation never silently introduces `Object` or `Any`.
- [ ] Compatibility claims require executable positive and negative evidence.
- [ ] Raw API improvements use the same ownership and conversion contract.

## Planning Review Evidence

- Opus pass 1: approve with changes; identified bridge registration, decorator
  probing, release lifecycle, record mapping, manifest authority, target-root
  resolution, and milestone sizing gaps.
- Opus pass 2: independently confirmed those gaps and added argument passing,
  sealed raw-object migration, diagnostic, and protocol-design sequencing
  requirements.
- Opus pass 3: approved with no blocking findings after the architecture and
  phase revisions; final prose refinements are incorporated in this document.

Review artifacts:

- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-1.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-2.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-3.md`

## Exit Gate

The phase is complete only when all milestones are merged, public and internal
documentation describe the implemented contract rather than the proposal,
compatibility claims match executable evidence, local authoritative validation
passes, review has no unresolved actionable findings, and the phase record
links every merged PR.

# Ad Hoc Phase: Complete Declaration-First Python Interop

## Status

In progress. The phase defines one complete end-state architecture and an
ordered implementation sequence. Opus High pass 5 approved the complete design;
a final independent Fable High audit found no blockers and its eight
non-blocking precision refinements are incorporated. M0 is implemented and
locally validated in [PR #2930](https://github.com/sifr-lang/sifr/pull/2930).
M1 is implemented and locally validated in
[PR #2932](https://github.com/sifr-lang/sifr/pull/2932). M2 is implemented,
locally validated, and Opus-approved in
[PR #2933](https://github.com/sifr-lang/sifr/pull/2933); later milestones are
not yet implemented.
Milestones sequence delivery; they do not create reduced language versions,
temporary public contracts, dual authorities, or alternate lowering paths.

The durable contracts are:

- [`internal_docs/python_interop_declaration_architecture.md`](../../../internal_docs/python_interop_declaration_architecture.md)
- [`internal_docs/python_interop_protocol_architecture.md`](../../../internal_docs/python_interop_protocol_architecture.md)

## Objective

Make existing Python libraries feel like ordinary typed Sifr packages. Package
authors declare direct targets or hermetic Python bridges once; consumers use
normal typed functions, opaque classes, async functions, context managers,
callbacks, and affine data resources without manipulating Python handles.

The complete design preserves Sifr's static error, ownership, blocking,
sendability, cancellation, trust, and no-panic guarantees. It also gives Python
protocols first-class contracts rather than routing unfinished cases through
raw objects.

## End-State Decisions

- `@python(...)` is the synchronous declaration boundary;
  `@python.coroutine(...)` is the genuine coroutine boundary.
- The Sifr signature is the sole conversion type contract. Decorators carry
  only target and protocol facts that types cannot express.
- Decorator targets are structured dotted paths in a dedicated Python target
  namespace. String targets are invalid.
- Declaration bodies are ellipsis-only.
- All Python identity uses one sealed compiler-owned non-send handle. The
  grammar has no configurable `send=` policy.
- Ordinary references release automatically. Semantic close, async close,
  context exit, callback shutdown, buffer release, Arrow release, and DLPack
  consumption keep distinct affine contracts.
- Python construction is exposed through fallible top-level or static factory
  functions; ordinary Sifr construction never hides a `Result`.
- `python.omit` in a declaration default distinguishes omitted arguments from
  explicitly supplied `None` without leaking a wrapper type to consumers.
- Typed positional variadics, typed kwargs, and explicit closed-record kwargs
  expansion are supported and checked.
- Package-local Python bridges under `src/python_bridges/` are hermetic embedded
  package inputs with static import inventory.
- Synchronous declarations are `blocking_io`. Async declarations use one
  application-owned asyncio loop thread with structured bidirectional
  cancellation and deterministic shutdown.
- Callback declarations state lifetime, owner, dispatch, concurrency, and
  shutdown. No callback may escape without a deterministic owner.
- Buffer, Arrow, and DLPack declarations return affine typed resources and never
  copy. Copied values use ordinary copied return types.
- Static declarations and bridge imports infer Python requirements. The root
  application alone authorizes execution and native extensions.
- `[python].allow-imports` is removed atomically. There is no period with two
  requirement authorities.
- Checked-in declarations are authoritative. Stubs and introspection generate
  reviewable scaffolds but never introduce `Any`, bare `object`, or `py.Object`
  automatically.
- `sifr.python` remains the intentional dynamic API over the same sealed handles
  and affine resources, not a generated-declaration degradation path.
- Capability claims require executable positive, negative, cleanup, and
  compiled-binary evidence.

## Non-Goals

- Python source compatibility or arbitrary dynamic attribute syntax in normal
  Sifr code.
- A Sifr `Any` type, untyped declaration generation, or implicit conversion to
  `py.Object`.
- Automatic Python installation, `uv sync`, environment mutation, or trust
  mutation.
- Whole-library binding generation without symbol selection and review.
- Decorator-level converter pipelines or repeated type metadata.
- Hidden offload, ambient event-loop reuse, per-call event loops, or nested
  `asyncio.run`.
- Silent copying, device changes, ownership transfer, callback escape, or
  resource abandonment.
- Static proof of arbitrary Python implementation bodies.

## Scope

Expected implementation surfaces:

- `crates/sifr_ir/src/` for declaration, protocol, ownership, callback, and
  async metadata;
- `crates/sifr_type_system/src/` for compiler-known protocol types,
  `AsyncCallable`, ownership eligibility, and union/affine rules;
- `crates/sifr_lowering/src/lower/` for decorator validation, call-shape
  lowering, non-send checks, affine resources, context protocols, and effects;
- `crates/sifr_codegen/src/` for generated sync/async wrappers, callback
  trampolines, context adapters, and protocol transfers;
- `crates/sifr_driver/src/build/` for environment/target probes and generated
  application plans;
- `crates/sifr_package/src/python/` for inferred requirements, uv discovery,
  trust authority, bridge inventories, and cache identity;
- `crates/sifr_runtime/src/python/` for sealed handles, release queues, the
  asyncio loop, callbacks, contexts, buffers, Arrow, and DLPack;
- `crates/sifr_diagnostics/` for active declaration and protocol diagnostics;
- `stdlib/sifr/python*.sifr` for the typed raw API over the same representations;
- `verification/areas/python_interop/` for deterministic and live executable
  evidence;
- CLI and LSP surfaces for check, doctor, binding generation, completion,
  navigation, and drift reporting.

## Delivery Rule

Internal substrate may land before its public syntax, but no PR may expose a
temporary public grammar or a second runtime representation. Each milestone
ends with one production path for the behavior it activates. When a milestone
replaces an existing authority or representation, it updates every consumer,
fixture, diagnostic, and document in the same merge unit.

M0 reserves `SIFR-PYRES-0002` for syntax recognized from the complete grammar
whose sole production lowering has not yet activated in an intermediate merge.
Activation occurs once: M3 sync calls and call shapes; M4 opaque/method/attr/item
lifecycle; M5 sync contexts; M6 bridge targets; M7 coroutines and async close;
M8 async contexts; M9 callbacks; M10 buffers; M11 Arrow; M12 DLPack. Before its
owner milestone, a form hard-errors with `SIFR-PYRES-0002`; it never uses raw
objects, alternate semantics, or a compatibility implementation.

## Milestones

Implementation progress:

- [x] M0 complete contract lock and evidence model — [PR #2930](https://github.com/sifr-lang/sifr/pull/2930)
- [x] M1 sealed runtime identity and cleanup — [PR #2932](https://github.com/sifr-lang/sifr/pull/2932)
- [x] M2 environment and trust authority cutover — [PR #2933](https://github.com/sifr-lang/sifr/pull/2933)
- [x] M3 synchronous declaration core and complete call shapes — [PR #2934](https://github.com/sifr-lang/sifr/pull/2934)
- [x] M4 recursive conversion and opaque lifecycle — [PR #2935](https://github.com/sifr-lang/sifr/pull/2935)
- [x] M5 synchronous Python context managers — [PR #2942](https://github.com/sifr-lang/sifr/pull/2942)
- [ ] M6 hermetic package-local Python bridges
- [ ] M7 owned asyncio runtime and async declarations
- [ ] M8 async context managers
- [ ] M9 typed callback lifetimes and dispatch
- [ ] M10 typed buffer protocol
- [ ] M11 Arrow C Data Interface
- [ ] M12 DLPack one-shot tensor transfer
- [ ] M13 read-only check and doctor
- [ ] M14 binding and certification authoring
- [ ] M15 LSP declaration authoring
- [ ] M16 raw API ergonomics on shared ownership
- [ ] M17 ecosystem migration and certification

### M0. Complete Contract Lock And Evidence Model

Tasks:

- Accept both architecture documents as the complete target contract.
- Define capability states: declaration-supported, bridge-supported,
  dynamic-only, and unsupported-by-design.
- Add a machine-readable declaration/protocol capability matrix separate from
  package inventory.
- Assign positive, negative, cleanup, cancellation, and live evidence owners to
  every decorator and protocol state transition.
- Lock the complete decorator grammar, policy atoms, target namespace, and
  ellipsis-only body rule.
- Lock positional, keyword-only, `python.omit`, typed `*args`, typed `**kwargs`,
  and explicit record-expansion semantics.
- Lock diagnostic families `PYIMP`, `PYCALL`, `PYCONV`, `PYRES`, `PYASYNC`,
  `PYCTX`, `PYCB`, and `PYZC`, including stable first codes.
- Reserve `SIFR-PYRES-0002` for recognized but not-yet-activated declarations
  during the ordered implementation sequence.
- Lock the single manifest/trust authority and atomic removal of
  `[python].allow-imports`.
- Add stale-design checks rejecting string targets, `send=`, repeated converter
  types, hidden copy policies, and reduced-version terminology.

Acceptance:

- Every public syntax form maps to one HIR/runtime contract.
- Every protocol has an explicit ownership and shutdown state machine.
- No capability is labeled supported from package inventory alone.
- No document describes a smaller language version or an undecided protocol.

Documentation validation:

- Markdown link and heading checks.
- `git diff --check` and file-size guardrails.
- Targeted terminology and rejected-syntax sweeps.

### M1. Sealed Runtime Identity And Cleanup

Tasks:

- Add one compiler-recognized sealed Python foreign-handle representation.
- Move raw `sifr.python.Object`, declared opaque objects, callbacks, and protocol
  resources onto sealed private runtime identities; remove public token fields.
- Implement detach-before-decref, attached immediate release, a runtime-owned
  pending-release queue, attach-time draining, and final epilogue draining.
- Ensure no object-store lock is held while decref, destructor, callback, or
  other Python code can execute.

Acceptance:

- There is one opaque identity model for raw and declared Python state.
- Ordinary object cleanup is automatic on every control-flow exit.
- Drops cannot run Python while holding a Sifr resource-store lock.

Validation:

- Runtime ownership tests for success, failure, early return, detached-thread
  drop, reentrant destructor, and callback reentrancy.
- Repository sweep proving public handle/token fields are gone.

### M2. Environment And Trust Authority Cutover

Tasks:

- Add uv-compatible project, lock, environment, and interpreter discovery with
  explicit non-standard-layout overrides and real lock/project consistency.
- Add the single canonical requirement set with per-root provenance and the
  contribution interface used by declarations and bridge imports; retain
  `[python].requires-imports` only for underivable raw/dynamic library imports.
- Normalize duplicate manual and derived roots without override precedence.
- Remove `[python].allow-imports` atomically from parsing, docs, manifests,
  examples, diagnostics, and verification generation.
- Retire old `SIFR-PYTRUST-0002`, activate `SIFR-PYTRUST-0005` for an
  unauthorized required root, and rebase `SIFR-PYTRUST-0003` on native trust for
  a root that is not required.
- Keep `[trust].python` and `[trust].python-native` root-owned and distinct.

Acceptance:

- There is one canonical requirement authority with visible provenance.
- Dependencies can publish requirements but cannot authorize execution or
  native extensions.
- Sifr never installs, synchronizes, or mutates the Python environment.

Validation:

- uv default, override, workspace, missing-environment, and stale-lock fixtures.
- Root/dependency trust authority and native-trust negative tests.
- Duplicate-root provenance and diagnostic tests.
- Repository sweep proving the old allowlist is gone.

### M3. Synchronous Declaration Core And Complete Call Shapes

Depends on M1 sealed identity and M2 environment/trust authority.

Tasks:

- Parse all synchronous declaration decorators into structured metadata with
  source spans and dedicated target-path resolution.
- Accept ellipsis only as the complete body of eligible declarations.
- Add `PythonInteropDeclaration` HIR and `PythonInteropPlan` build metadata.
- Synthesize interop effects: `blocking_io` for synchronous declarations and
  the async interop effect for coroutine declarations, without bare-name
  annotations or a new suspension-summary variant.
- Generate scalar and opaque sync function/factory wrappers.
- Implement regular positional, keyword-only, explicit defaults,
  `python.omit`, typed positional variadics, typed kwargs, and explicit closed
  record expansion.
- Add parser/HIR productions for call-site `**record` with retained closed-field
  metadata; do not treat it as an existing dictionary-spread feature.
- Validate inspectable target arity, positional-only rules, keyword names, and
  duplicates; require inspectability for record kwargs expansion and mark other
  genuinely uninspectable targets runtime-checked.
- Infer `blocking_io`, requirements, trust, probe inputs, and cache identity.
- Map Python exceptions and conversion failures into structured `PythonError`.

Acceptance:

- Consumers call typed Python-backed functions without raw-object operations.
- Omission and explicit `None` are observably distinct.
- Every supported typed call shape has exactly one lowering.
- Unsupported heterogeneous or data-dependent shapes require an explicit bridge
  and never become dynamic declarations.
- Sync declarations cannot be called directly from async code without explicit
  Sifr offload.

Validation:

- Parser/lowering/codegen matrices for every accepted and rejected call shape.
- Inspectable and uninspectable pure-Python/C-extension targets.
- Library-only deferred target probes and final-application resolution.
- Real pure-Python and native-extension compiled examples.
- Outstanding-reference assertions for every wrapper failure point.

### M4. Recursive Conversion And Opaque Lifecycle

Depends on M1 sealed identity and M3 declaration HIR/wrappers.

Tasks:

- Implement recursively checked scalars, options, lists, tuples,
  `dict[str, T]`, closed records, and nested boundary paths.
- Preserve the canonical record mapping: Sifr records construct Python dicts;
  extraction requires every field, tries attributes before string-key items,
  and ignores extras.
- Implement `@python.opaque(type=..., cleanup=drop | close | async_close |
  context | async_context)` without a send policy.
- Implement `Self` methods, fallible attributes, and item access.
- Enforce factory `isinstance`, borrow/move/poison/use-after-close rules, and
  consuming synchronous semantic close.
- Implement the general linear must-use obligation side table and scope/function
  exit checks for `cleanup=close | async_close | context | async_context`,
  including transfer through moves, returns, aggregates, and control-flow joins.

Acceptance:

- Opaque Python objects expose typed APIs with no structural handle fields.
- Nested conversion failures identify the exact boundary path and release every
  partially constructed value.
- Automatic drop and semantic close are distinct and exact once.
- Python identity remains non-send without package configuration.

Validation:

- Full conversion matrix including overflow, missing fields, extras,
  attribute-then-item lookup, and partial failure cleanup.
- Descriptor/property errors, wrong factory type, moved value, double close,
  poison, and use-after-close fixtures.
- Negative fixture rejecting abandonment of `cleanup=close` and positive
  fixtures transferring the obligation by return/aggregate ownership.
- Runnable biip/schwifty object examples.

### M5. Synchronous Python Context Managers

Depends on M4 opaque lifecycle and conversion.

Implementation waves:

- [x] Declaration types, diagnostics, strict protocol validation, and
  context-only obligations — [PR #2936](https://github.com/sifr-lang/sifr/pull/2936)
- [x] Exception replay, boundary errors, exit APIs, and secondary evidence —
  [PR #2937](https://github.com/sifr-lang/sifr/pull/2937)
- [x] Dedicated Python-context HIR and scoped entered-borrow ownership —
  [PR #2938](https://github.com/sifr-lang/sifr/pull/2938)
- [x] Closure/outcome codegen and the normative exit decision table —
  [PR #2940](https://github.com/sifr-lang/sifr/pull/2940)
- [x] Complete evidence matrices, transaction demo, and milestone closure —
  [PR #2942](https://github.com/sifr-lang/sifr/pull/2942)

Tasks:

- Implement sync context enter/exit, structured `python.ExitCause`, Python
  exception replay capabilities, `SifrBoundaryError`, the normative decision
  table, dedicated Python-context lowering, and secondary cleanup evidence.
- Keep native Sifr `with` on its existing argless drop-style protocol.
- Retain the manager as hidden owner and make opaque entered values
  context-scoped borrows that cannot escape, move, or close independently.

Acceptance:

- Context exit is distinct from automatic drop and runs exactly once.
- Python context lowering honors suppression only for originating Python
  exceptions; ordinary Sifr errors cannot be suppressed by Python truthiness.
- Original Python exception triples replay through nested managers and release
  exactly once.

Validation:

- Context normal/error/early-return/suppression/exit-failure matrices.
- Type-sensitive original-exception replay, nested replay lifetime, ordinary
  Sifr error non-suppression, and replay-token release fixtures.
- Negative fixture rejecting a distinct opaque entered result whose cleanup is
  not `drop`.
- Runnable sync database transaction example.
- Negative fixture rejecting a `cleanup=context` value that is never entered.

### M6. Hermetic Package-Local Python Bridges

Depends on M1 sealed identities and M4 conversion.

Implementation waves:

- [x] Package bridge source and inventory substrate — [PR #2945](https://github.com/sifr-lang/sifr/pull/2945):
  - Discover only package-root `src/python_bridges/**/*.py`, independent of
    custom Sifr source roots; reject misplaced bridge roots, invalid module
    paths, duplicate modules, invalid Python syntax, and dynamic import calls as
    `SIFR-PYIMP-0002`.
  - Build a canonical ordinary `import` / `from ... import ...` inventory,
    classify same-package bridge edges separately from third-party roots, and
    compute stable source and inventory digests.
  - Require every bridge source plus its generated inventory manifest in
    package archives. Keep `bridge.*` declarations gated by
    `SIFR-PYRES-0002` throughout this substrate wave.
- [x] Resolved identity, rewrite, and authority planning — [PR #2947](https://github.com/sifr-lang/sifr/pull/2947):
  - Define and test a deterministic, valid-Python-identifier, collision-resistant
    encoding of resolved Sifr package identity for
    `__sifr_bridge__.p_<resolved_package_key>`.
  - Resolve same-package imports to that package prefix, preserve external
    roots as `PythonRequirementKind::BridgeImport` contributions to the
    canonical requirement set, and keep dependency requirements subject to
    root-owned `SIFR-PYTRUST-0005` authorization.
  - Carry isolated bridge identities and inventories through the selected
    package graph into driver/codegen planning without yet activating public
    `bridge.*` declarations.
- [x] Atomic loader, embedding, and declaration activation — [PR #2949](https://github.com/sifr-lang/sifr/pull/2949):
  - Generate embedded source tables with synthetic package entries and stable
    virtual filenames of the form
    `<__sifr_bridge__.p_<resolved_package_key>.<module_path>>`; propagate each
    filename into Python `co_filename`.
  - Implement the reserved loader in a focused runtime `bridge_loader` module
    through GIL-bound PyO3 APIs, leaving raw C initialization calls isolated to
    the existing unsafe boundary. Install it at `sys.meta_path[0]` immediately
    after CPython configuration and before user `main` or any user import.
  - Reject pre-existing reserved `sys.modules` entries as
    `SIFR-PYIMP-0003`, retain the reserved-name claim even after user
    `sys.meta_path` mutation, and never fall back to filesystem or `sys.path`
    lookup.
  - In the same merge unit, rewrite `bridge.*` call targets to their resolved
    runtime names and lift `SIFR-PYRES-0002`; a distribution literally named
    `bridge` remains reachable only through a non-reserved declared target.
- [ ] Deployment graph and cache closure:
  - Embed every bridge module from every runtime package in the selected target's
    resolved graph, excluding dev-only and otherwise unselected packages; do
    not depend on declaration reachability or a source checkout at runtime.
  - Fingerprint source/inventory digests, resolved package identity, resolved
    distribution versions, interpreter ABI, the binding contract, and typing
    inputs in package, driver, and generated-artifact caches.
  - Prove archive unpack/install/build/run uses only archived bridge inputs and
    the generated binary uses no writable extraction directory.
- [ ] Complete bridge evidence and milestone closure:
  - Cover loader-before-main ordering, first-position and post-mutation reserved
    resolution, collision rejection, sibling import rewriting, deterministic
    traceback filenames, cache invalidation, invalid syntax, rejected dynamic
    imports, misplaced sources, and reserved target ambiguity.
  - Cover two packages owning the same module path and a dependency bridge whose
    third-party import is rejected until the root application authorizes it.
  - Run a compiled biip-backed identifier bridge from an installed archive with
    no source checkout, and add `demos/m6_demo` as the milestone showcase.
  - Activate package-bridge capability evidence and update architecture,
    roadmap, milestone checkboxes, review records, and merged PR links.

Tasks:

- Resolve `bridge.*` under package-owned `src/python_bridges/` source.
- Embed bridge source tables under
  `__sifr_bridge__.p_<resolved_package_key>.<module_path>`.
- Install a first-position `MetaPathFinder`/loader before user code.
- Reject reserved-namespace `sys.modules` collisions and all filesystem or
  `sys.path` resolution for reserved names.
- Rewrite same-package bridge imports under the package namespace.
- Inventory ordinary static imports and reject dynamic import calls in package
  bridges.
- Include source, package identity, distribution versions, interpreter ABI,
  binding contract, and typing inputs in cache fingerprints.
- Include bridge source/inventory in archives and embed only the resolved graph
  in generated binaries.

Acceptance:

- Bridges are reproducible package implementation, not ambient Python files.
- Two packages may own the same bridge module path without collision.
- A dependency bridge cannot authorize its own third-party imports.
- Deployment does not depend on a source checkout, writable temp directory, or
  ambient path ordering.

Validation:

- Loader ordering, collision, sibling import, traceback, and cache tests.
- Multi-package same-name bridge fixture.
- Archive/install/run fixture with no source checkout.
- Static import inventory and rejected dynamic import fixtures.

### M7. Owned Asyncio Runtime And Async Declarations

Depends on M1 sealed identity, M3 declaration effects, and M4 opaque lifecycle.

Tasks:

- Add one generated-application-owned asyncio loop on a dedicated OS thread.
- Start it after CPython/bridge initialization and stop it after registered async
  cleanup and callback shutdown.
- Implement `@python.coroutine(path)` on `async def` for functions, factories, and
  methods using the same conversion and target contracts.
- Convert inputs, invoke, await, and convert outputs on the loop thread.
- Implement structured bidirectional cancellation, terminal-state waiting,
  `CancelledError` mapping, and cancellation suppression behavior.
- Route coroutine ellipsis declarations through the interop `Bodyless` stub path
  so they skip normal body lowering and the `NoSuspend` fake-async gate while
  retaining the async interop effect.
- Route the raw coroutine API through the owned loop and remove per-call
  `asyncio.run`.
- Permit owned opaque results without granting Sifr sendability.
- Implement consuming `cleanup=async_close` and poison-on-cleanup-failure.

Acceptance:

- Python coroutines never block a Sifr executor thread.
- There is one loop per generated application, never one per call.
- Sifr cancellation does not complete before Python `finally` cleanup.
- Runtime shutdown leaves no live asyncio task or loop thread.
- Sync and async declaration kinds cannot silently substitute for each other.

Validation:

- Async success, Python failure, conversion failure, cancellation-before-start,
  cancellation-in-flight, cancellation suppression, and shutdown matrices.
- Async close success/failure/poison/use-after-close fixtures.
- Negative fixture rejecting abandonment of `cleanup=async_close`.
- Concurrent coroutine target tests proving one loop identity.
- Compiled httpx-style async client example.

### M8. Async Context Managers

Depends on M5 context semantics and M7 loop ownership.

Tasks:

- Implement `@python.context.aenter` and `.aexit` on async declarations.
- Classify the concrete async body outcome directly into `python.ExitCause`
  before native cause erasure, replay original `PythonError` triples, and create
  `SifrBoundaryError` for Sifr causes. Native `AsyncExitCause` is not the Python
  classification source.
- Honor suppression only for originating Python exceptions and attach ignored
  truthy decisions or cleanup failures as evidence for unsuppressible causes.
- Mask body cancellation while async exit reaches a terminal state, then resume
  cancellation unless a higher-priority cleanup error wins.
- Integrate async context resources with opaque async close and shutdown order.

Acceptance:

- `async with` executes enter and exit on the owned Python loop.
- Exit occurs exactly once for normal return, Sifr error, Python error, early
  return, and cancellation.
- Cancellation cannot abandon async cleanup.

Validation:

- Async context normal/suppression/error/cancellation/exit-failure matrix.
- Truthy-exit tests proving timeout, cancellation, runtime fault, and ordinary
  Sifr errors remain unsuppressed.
- Negative fixture rejecting a `cleanup=async_context` value never entered by
  `async with`.
- Negative fixture rejecting a distinct async entered opaque result whose
  cleanup is not `drop`.
- Nested sync/async context ordering and secondary-error fixtures.
- Compiled async database/session example.

### M9. Typed Callback Lifetimes And Dispatch

Depends on M4 opaque owners and M7 owned asyncio loop.

Tasks:

- Implement `@python.callback` parameter metadata for `lifetime=call | result |
  Self`, `dispatch=current | foreign | asyncio`, and required concurrency.
- Generate checked callable argument/result conversion and `SifrCallbackError`
  propagation.
- Enforce current-thread non-escaping callbacks and non-send capture rules.
- Implement foreign-thread `Send + Sync` trampolines, serial and parallel
  dispatch, reentrancy rejection, and forbidden Python opaque arguments.
- Implement asyncio-dispatched `AsyncCallable` with bidirectional cancellation.
- Add net-new `AsyncCallable[[...], R]` annotation/type-system support parallel
  to `Callable`; do not treat `Type::AsyncFunction` as already equivalent.
- Require serial/parallel concurrency for foreign and asyncio dispatch; reject
  serial reentrancy before lock or await.
- Aggregate retained callback trampolines into returned or receiver owners.
- Implement open/closing/closed state, unregister-first shutdown, new-entry
  rejection, active-call draining, capture release, and runtime shutdown.
- Reject owner close from within an accepted invocation statically where visible
  and through a runtime reentrancy guard otherwise.

Acceptance:

- No callback can outlive its declared owner.
- Foreign callbacks cannot smuggle non-send Python identity across threads.
- Serial reentrancy fails deterministically instead of deadlocking.
- Owner close drains accepted invocations and rejects later calls.
- Async callbacks block neither executor while awaiting completion.

Validation:

- Current, foreign serial, foreign parallel, and asyncio callback matrices.
- Capture/sendability, wrong argument/result, handler error, Python error,
  reentrancy, concurrent close, callback-after-close, and shutdown fixtures.
- Call-scoped concurrent first-failure, swallowed-callback-error, and
  Python-error-plus-secondary-handler-error fixtures.
- Compiled CFFI, Kafka, and Pub/Sub callback examples.

### M10. Typed Buffer Protocol

Depends on M1 sealed affine resources and M4 conversion.

Tasks:

- Add affine non-send `python.Buffer[T]` with compiler-private identity.
- Implement `@python.buffer(target, access=read | write, layout=any |
  c_contiguous | f_contiguous)`.
- Implement receiver acquisition for `Self` and call-then-acquire for import-root
  or bridge producer targets.
- Validate format, item size, dimensions, shape, strides, suboffsets,
  writability, and layout before return.
- Retain exporter ownership and enforce slice/view lifetimes.
- Require exclusive Sifr borrow for writable buffers.
- Implement exact-once automatic and explicit early `PyBuffer_Release`.
- Expose checked runtime metadata and typed bounded element/slice accessors.

Acceptance:

- A buffer view cannot outlive its exporter or be released while borrowed.
- Writable access cannot alias through Sifr.
- Buffer declarations never copy.
- Every acquired buffer releases exactly once on all exits.

Validation:

- Format/layout/writability positive and negative matrices.
- `Self`, import-root producer, and bridge producer acquisition fixtures.
- Borrow/move/release/use-after-release checks.
- Pointer identity and exact release counters.
- Compiled NumPy-compatible buffer example.

### M11. Arrow C Data Interface

Depends on M1 sealed affine resources and M4 conversion.

Tasks:

- Add affine `python.ArrowArray`, `ArrowSchema`, `ArrowStream`,
  `ArrowDeviceArray`, and `ArrowDeviceStream` resources, with array resources
  owning their required schema/data pair.
- Implement `@python.arrow(target)` with return-kind derivation.
- Implement receiver acquisition for `Self` and call-then-acquire for import-root
  or bridge producer targets.
- Implement `schema=omitted | parameter(name)` and validate the latter as a
  same-declaration keyword-only borrowed `python.ArrowSchema` parameter.
- Validate exact capsule names, non-null payloads, release callbacks, device
  metadata, producer identity, and paired schema/data consistency.
- Require executable no-copy certification tied to the exact producer and
  distribution fingerprint; reject uncertain producers and schema requests.
- Add the package-authored `sifr python certify arrow` fixture/artifact workflow
  and read-only certification recheck, bootstrapping the `sifr python` command
  group.
- Implement owned argument transfer, consumed-state detection, failure cleanup,
  and exact-once release.

Acceptance:

- Arrow declarations have no copy switch and never certify uncertain copying.
- Ownership transfers once and remains moved even if a consumer later fails.
- Unconsumed resources release exactly once.

Validation:

- Array/schema/stream acquisition and transfer matrices.
- `Self`, import-root producer, and bridge producer acquisition fixtures.
- Wrong name, null payload, missing releaser, producer-copy, partial-consumption,
  double-consumption, and use-after-move fixtures.
- Requested-schema omitted/parameter shape, certification, and mismatch fixtures.
- Pointer/release evidence with pandas, PyArrow, and Polars compiled examples.

### M12. DLPack One-Shot Tensor Transfer

Depends on M1 sealed affine resources and M4 conversion.

Tasks:

- Add affine `python.DlpackTensor[T]`.
- Implement `python.DlpackStream`, `@python.dlpack.stream`, and
  `@python.dlpack(target, device=..., stream=none | parameter(name))` with no
  cross-device request.
- Implement receiver acquisition for `Self` and call-then-acquire for import-root
  or bridge producer targets.
- Add `parameter(name)` as a decorator-argument parser/HIR production resolving
  only to a same-declaration keyword-only `python.DlpackStream` parameter.
- Require `device=any` to use that stream parameter and validate its family/id
  against the producer-reported device at runtime before acquisition.
- Pass `copy=False` and supported `max_version`; validate legacy/versioned
  capsule names, copied flags, dtype, lanes, device, dimensions, shape, strides,
  byte offset, deleter state, and synchronization contract.
- Accept legacy capsule names only from producers that accept the complete
  versioned call signature; never retry an old signature without `copy` or
  `max_version`.
- Implement `used_dltensor` marking, one-shot owned transfer, committed moves on
  post-consumption failure, and exact-once deleter cleanup before consumption.
- Rename the producer capsule to its used sentinel at Sifr acquisition before
  assuming deleter responsibility, then create a fresh consumer capsule for any
  later owned transfer.
- On failure before consumer acquisition, rename that fresh capsule to its used
  sentinel before invoking the deleter exactly once.
- Keep copied tensor conversion as an ordinary copied declaration or bridge.

Acceptance:

- DLPack declarations never copy or change device.
- A tensor is consumed at most once and cannot be used after move.
- Stream synchronization is explicit and validated.
- Every unconsumed tensor invokes its deleter exactly once.

Validation:

- CPU/CUDA, dtype, shape, stride, stream, and device matrices.
- `Self`, import-root producer, and bridge producer acquisition fixtures.
- `device=any` matching and mismatched runtime stream-device fixtures.
- Wrong capsule, copied flag, null/non-null deleter, double consume,
  failure-before-consume, failure-after-consume, and shutdown fixtures.
- Pointer/deleter evidence with PyTorch and TensorFlow compiled examples.

### M13. Read-Only Check And Doctor

Depends on the completed compiler/runtime protocol plans from M2 through M12.

Tasks:

- Add read-only `sifr python check` using the same package/driver plan as normal
  check/build for environment, lock, trust, target, protocol, and probe status.
- Add read-only `sifr python doctor` with patch-like suggestions that never
  mutate manifests, trust, or environments.

Acceptance:

- Both commands agree with compiler/build results for the same snapshot.
- Neither command writes, installs, trusts, or runs environment synchronization.

Validation:

- CLI parity, deterministic doctor goldens, and non-mutation checks.
- Library-only deferred-probe and final-application resolution fixtures.

### M14. Binding And Certification Authoring

Depends on M11/M12 certification contracts and M13 shared check plan.

Tasks:

- Add symbol-selective `sifr python bind` from explicit user overrides,
  selected stub packages, `py.typed` packages, configured external stubs, and
  safe introspection in recorded precedence.
- Stop or emit an explicit unresolved marker for `Any`, bare `object`, unknown
  overloads, unsupported generics, uncontracted callables, or dynamic fields.
- Record SOABI, distribution version, source precedence, and consumed typing
  hashes; implement read-only `bind --check` drift reporting.
- Extend the `sifr python certify` surface bootstrapped by M11's Arrow workflow
  to the general protocol-evidence and fingerprint-drift contract.

Acceptance:

- Generated declarations are reviewable checked-in Sifr source.
- No authoring tool silently generates an untyped boundary.
- Certification reruns within-run assertions rather than comparing addresses.

Validation:

- Stub-only, `py.typed`, C-extension, overload, unresolved, and drift fixtures.
- Arrow/DLPack certification artifact creation and read-only recheck fixtures.

### M15. LSP Declaration Authoring

Depends on M13/M14 compiler and authoring queries.

Tasks:

- Add LSP completion, navigation, diagnostics, verified/runtime-checked status,
  protocol policy help, and cache invalidation from compiler queries.

Acceptance:

- LSP results agree with compiler/check results for the same package snapshot.
- Completion never offers an untyped or unsupported declaration as certified.

Validation:

- LSP completion/navigation/diagnostic/cancellation and cache-drift tests.

### M16. Raw API Ergonomics On Shared Ownership

Depends on M1 shared identity, M7 owned loop, and M10-M12 affine resources.

Tasks:

- Improve raw `sifr.python` with sealed automatic ownership, typed generic
  conversion, method-style operations, and typed kwargs over the same runtime.

Acceptance:

- Raw ergonomics improve without a second ownership or conversion model.
- Raw coroutine execution uses the owned loop and no per-call event loop remains.

Validation:

- Raw/declaration representation and cleanup equivalence tests.
- Typed raw conversion/call/kwargs and coroutine-path tests.

### M17. Ecosystem Migration And Certification

Depends on all preceding declaration, protocol, tooling, and raw-runtime
milestones.

Tasks:

- Migrate all runnable biip/schwifty, dataframe, ML, web, database, cloud,
  crypto, Redis, callback, context, and async examples to declarations or
  hermetic bridges.
- Keep one intentionally small raw-object example documenting the dynamic API;
  ordinary examples contain no raw handles or raw protocol plumbing.
- Require actual compiled Sifr binaries for Redis and Postgres round trips,
  Kafka publish/consume, direct SQS send/receive, SNS-to-SQS delivery, async
  HTTP, callback, Arrow, and DLPack certification.
- Replace the existing Python-client live lane rather than counting its results
  as compiled-Sifr evidence. Docker/network hosts own service cases, CPU hosts
  own CPU Arrow and CPU DLPack certification, and labeled CUDA runners own Arrow
  device-interface and CUDA DLPack certification; unsupported hosts report
  structured skips and cannot promote those capability rows.
- Assert zero outstanding ordinary objects, async tasks, contexts, callbacks,
  buffers, Arrow resources, and DLPack tensors after every success and failure
  path.
- Update public/internal docs, architecture status, roadmap, capability matrix,
  phase status, and merged PR evidence.
- Run final architecture and implementation review until no actionable finding
  remains.

Acceptance:

- Declaration-first APIs are the normal documented Python interop experience.
- Every protocol preserves its stated ownership, cancellation, copy, release,
  and shutdown semantics in executable evidence.
- Capability categories match actual evidence.
- Named live cases execute compiled Sifr rather than Python-client substitutes.
- Review has no unresolved actionable findings.

Validation:

- Complete deterministic positive/negative/cleanup/cancellation suites.
- Named compiled-Sifr live service, callback, async, and zero-copy cases.
- Documentation, diagnostics, link, and stale-design sweeps.
- Authoritative create-PR and merge profiles.

## Verification Policy

- A supported capability needs a positive executable, its primary negative
  executable, and every relevant cleanup/cancellation transition.
- Real pure-Python and native-extension declarations run from M3 onward.
- Zero-copy certification requires observable pointer identity or equivalent
  protocol evidence plus exact release/deleter counters.
- Callback certification covers every dispatch/concurrency mode and owner
  shutdown with work in flight.
- Async certification proves one loop identity, terminal cancellation cleanup,
  and no live tasks at shutdown.
- Live evidence distinguishes compiled Sifr execution from Python-client or
  package-presence evidence.
- Inventory remains discovery evidence and cannot certify a capability.
- Every executable that creates Python state asserts zero outstanding resources.

## Review Checklist

- [ ] The architecture defines one complete language, not a reduced release.
- [ ] The Sifr signature is the only conversion type declaration.
- [ ] Targets are structured paths in a dedicated namespace.
- [ ] Omission, defaults, positional arguments, kwargs, and variadics are exact.
- [ ] All Python identity uses one sealed non-send runtime handle.
- [ ] Automatic reference drop is distinct from semantic resource operations.
- [ ] Sync context suppression and cleanup-error precedence are explicit.
- [ ] One owned asyncio loop has bidirectional cancellation and terminal cleanup.
- [ ] Async close and async context exit cannot be abandoned by cancellation.
- [ ] Callback lifetime, owner, dispatch, concurrency, and shutdown are explicit.
- [ ] Buffer borrow/access/layout and exact release are explicit.
- [ ] Arrow and DLPack are affine, one-path, and never copy.
- [ ] Bridges are hermetic embedded package inputs with static imports.
- [ ] There is one requirement/trust authority and no allowlist coexistence.
- [ ] Check, doctor, binding generation, and LSP reuse compiler plans.
- [ ] No tool or compiler path creates an untyped boundary automatically.
- [ ] Capability claims require executable negative and cleanup evidence.
- [ ] Named live cases invoke compiled Sifr binaries.

## Planning Review Evidence

Earlier reviews shaped the declaration-first direction and identified bridge
loading, sealed handles, release queues, record mapping, target probes,
requirement authority, argument passing, diagnostics, and milestone sequencing.
Their artifacts remain useful historical evidence, but any reduced-version or
compatibility-period recommendation in them is superseded by the complete
architecture in this phase.

Review artifacts:

- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-1.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-2.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-opus-review-pass-3.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-fable-review-final.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-opus-high-pass-1.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-opus-high-pass-2.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-opus-high-pass-3.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-opus-high-pass-4.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-opus-high-pass-5.md`
- `plans/reviews/active/ad-hoc-declaration-first-python-interop-complete-fable-high-final.md`

Complete-architecture Opus High pass 1 requested changes. It found undefined
Python exit-cause/suppression semantics, live-exception replay, Arrow evidence
authoring, async effect sealing, record-expansion ambiguity, DLPack stream
provenance, raw async-path duplication, callback concurrency, requirement
provenance, and milestone-sizing gaps. The architecture and phase now resolve
all of those findings directly rather than removing or postponing capabilities.
Complete-architecture pass 2 confirmed every pass-1 finding resolved and found
one async-context replay contradiction plus pointer-assertion, replay-sendability,
grammar-ownership, DLPack-capsule, and milestone-sizing refinements. Dedicated
Python async-context lowering now classifies the concrete body outcome directly,
and all refinements are incorporated.
Complete-architecture pass 3 confirmed those corrections and found a final
front-end issue: three surface spellings used Python hard keywords. The grammar
now uses `@python.coroutine`, `lifetime=result`, and
`stream=parameter(name)`. The pass also prompted explicit general must-use
analysis, the existing interop bodyless-stub mechanism, complete CPU/CUDA
evidence ownership, dependency annotations, old-signature DLPack rejection, and
callback-close reentrancy guards.
Complete-architecture pass 4 found no blockers and approved with two
non-blocking refinements. `device=any` now has an explicit runtime-validated
stream rule, and the async interop effect/`Bodyless` rule explicitly covers
coroutines, async contexts, and asyncio-dispatched callback handlers.
Complete-architecture pass 5 rechecked those refinements and the complete
constraint set, found no actionable issue, and approved the design.
The final independent Fable High audit re-grounded the design against the
repository and protocol specifications, found no blocker, and approved with
eight non-blocking precision refinements. Those refinements now close DLPack
failed-transfer cleanup, entered-object cleanup eligibility, non-`Self`
acquisition, `AsyncCallable` ownership, Arrow requested schemas, call-scoped
callback error precedence, certification milestone ownership, and staged
activation diagnostics.

## Exit Gate

The phase is complete only when every milestone is merged, current architecture
and public docs describe the implemented end state, every capability claim has
matching executable evidence, all resource diagnostics are clean, authoritative
local validation passes, review has no unresolved actionable finding, and this
record links every merged PR.

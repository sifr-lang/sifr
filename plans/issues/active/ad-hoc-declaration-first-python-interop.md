# Ad Hoc Phase: Complete Declaration-First Python Interop

## Status

In progress. The phase defines one complete end-state architecture and an
ordered implementation sequence. Opus High pass 5 approved the complete design;
a final independent Fable High audit found no blockers and its eight
non-blocking precision refinements are incorporated. M0 through M9 and M10
Wave 1 are implemented, locally validated, reviewed, and linked below. M10
Wave 2 is merged; whole-diff review passes 14 through 25 reopened
generic/inherited Rust-trait, reusable affine-closure ownership, keyed sorting,
per-type-parameter bound, specialization, generic-operator, conditional-source,
top-level inference, multi-hop re-export identity, and specialized generic
pattern-capture gaps. Their remediation passes the focused and complete
compiler suites, native-positive/negative coverage, full merge-profile E2E,
maintainability, formatting, and file-size checks plus the authoritative local
create-PR gate in [PR #2988](https://github.com/sifr-lang/sifr/pull/2988).
Pass-25 remediation is complete, whole-diff review pass 26 is satisfied, and
PR #2988 is merged. M10 Wave 3 evidence, demo, and documentation closure is
implemented and locally validated; whole-diff review pass 6 is satisfied and
the authoritative create-PR gate passes, and PR #2989 is merged. The complete
merged M10 implementation is in final closure on PR #2990. Codex milestone
passes 10 through 14 drove exact nominal identity through unions, conversions,
inheritance, handlers, patterns, generics, operators, merged stdlib
declarations, imported-parent semantics, exact try-error carriers,
same-basename Rust bridge records, nested functions, and ordinary unions
sharing carrier enums. Subsequent Fable High passes 2 through 7 found and
closed the remaining error-carrier parity, PythonError identity, exact
catch-all, borrowed-field mutation, nested parameter, and class-method
mut-argument place gaps. Pass 8 was an invalid timer-only artifact and is not
review evidence. Fresh Fable High pass 9 independently rechecked the complete
candidate and returned **SATISFIED** with no blockers; post-commit closure
pass 10 confirmed the frozen PR diff and ledger with the same verdict. The
authoritative merge
gate passes every blocking lane in `2685.23s`, including E2E `674/674`, Python
interop `18/18`, diagnostics `175/175`, and `261` hardening variants with zero
failures; the warm-time notice is non-blocking. Typed
synchronous and asynchronous declarations and context managers run on the
application-owned Python loop with structured cancellation and consuming
cleanup; M9 current-thread, foreign-thread, and asyncio callback execution plus
retained-owner integration are merged, publicly active, and milestone-reviewed.
M11's five Arrow C Data Interface delivery waves are implemented, locally
validated, and milestone-reviewed in
[PR #2991](https://github.com/sifr-lang/sifr/pull/2991). Repeated whole-diff
Fable High review drove ownership, identity, async-move, operator, and
certification remediation through pass 8, which returned **SATISFIED** with no
blockers or majors; frozen-diff closure pass 9 independently confirmed the same
verdict. The fresh authoritative merge gate passes every blocking
lane in `3772.09s`, including E2E `674/674`, Python interop `20/20`, diagnostics
`175/175`, and `261` hardening variants with zero failures; its warm-time and
batch-skew notices are non-blocking advisories. M12's five DLPack delivery waves
are implemented and locally validated in
[PR #2992](https://github.com/sifr-lang/sifr/pull/2992). Repeated whole-diff
Fable High review found and closed shared call-frame cleanup, exact runtime-gate,
ownership-matrix, empty-reconciliation, and CPU stream-policy gaps; pass 4
returned **APPROVED** after independently reproducing the critical fixes, and
final frozen-diff pass 5 independently returned **APPROVED** with no blockers
or majors after fresh lifecycle, compiler-shape, gate, evidence, and
documentation sweeps. Closure pass 6 rechecked the exact committed candidate
and returned **APPROVED** with no blockers. The
fresh authoritative merge gate passes every blocking lane in `3554.66s`,
including E2E `674/674`, Python interop `22/22`, diagnostics `175/175`, and
`261` hardening variants with zero failures. Its warm-time and group-skew notices
are non-blocking advisories. M13's shared read-only driver plan, `python check`
and `python doctor` CLI surfaces, multi-app/final-app resolution, deferred
library reporting, executable non-mutation/parity evidence, demo, and docs are
implemented and in milestone closure. M14 and later milestones are not yet
implemented.
Milestones sequence delivery; they do not create reduced language
versions, temporary public contracts, dual authorities, or alternate lowering
paths.

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
- [x] M6 hermetic package-local Python bridges — [PR #2953](https://github.com/sifr-lang/sifr/pull/2953)
- [x] M7 owned asyncio runtime and async declarations — [PR #2968](https://github.com/sifr-lang/sifr/pull/2968)
- [x] M8 async context managers — [PR #2970](https://github.com/sifr-lang/sifr/pull/2970), [PR #2972](https://github.com/sifr-lang/sifr/pull/2972)
- [x] M9 typed callback lifetimes and dispatch — [PR #2974](https://github.com/sifr-lang/sifr/pull/2974), [PR #2977](https://github.com/sifr-lang/sifr/pull/2977), [PR #2979](https://github.com/sifr-lang/sifr/pull/2979), remediation [PRs #2981](https://github.com/sifr-lang/sifr/pull/2981), [#2982](https://github.com/sifr-lang/sifr/pull/2982), [#2984](https://github.com/sifr-lang/sifr/pull/2984), and [#2985](https://github.com/sifr-lang/sifr/pull/2985)
- [x] M10 typed buffer protocol ([PR #2990](https://github.com/sifr-lang/sifr/pull/2990))
- [x] M11 Arrow C Data Interface ([PR #2991](https://github.com/sifr-lang/sifr/pull/2991))
- [x] M12 DLPack one-shot tensor transfer ([PR #2992](https://github.com/sifr-lang/sifr/pull/2992))
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
- [x] Deployment graph and cache closure — [PR #2951](https://github.com/sifr-lang/sifr/pull/2951):
  - Embed every bridge module from every runtime package in the selected target's
    resolved graph, excluding dev-only and otherwise unselected packages; do
    not depend on declaration reachability or a source checkout at runtime.
  - Fingerprint source/inventory digests, resolved package identity, resolved
    distribution versions, interpreter ABI, the binding contract, and typing
    inputs in package, driver, and generated-artifact caches.
  - Prove archive unpack/install/build/run uses only archived bridge inputs and
    the generated binary uses no writable extraction directory.
- [x] Complete bridge evidence and milestone closure — [PR #2953](https://github.com/sifr-lang/sifr/pull/2953):
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

Implementation waves (one locally validated and reviewed PR per wave):

- [x] Prepare coroutine and async-close frontend contracts behind the existing
  `SIFR-PYRES-0002` gate — [PR #2956](https://github.com/sifr-lang/sifr/pull/2956):
  - Parse and validate the internal contract for bodyless
    `@python.coroutine(path)` `async def` functions, factories, and methods;
    retain `PythonInteropEffect::Async`, call-shape metadata, package/bridge
    target authority, and the ordinary declaration conversion contract without
    making the syntax publicly executable yet.
  - Mark bodyless async interop declarations as `Suspends` in the existing
    suspension summary so they bypass normal body lowering and the `NoSuspend`
    fake-async diagnostic without adding a summary variant or removing their
    ordinary async function identity.
  - Prepare stable diagnostics for sync/async decorator substitution, borrowed
    async results, non-consuming close, and unsupported cleanup shapes. Keep
    `cleanup=async_close` gated until its runtime lifecycle is complete.
- [x] Add the application-owned asyncio runtime and raw submission path —
  [PR #2958](https://github.com/sifr-lang/sifr/pull/2958):
  - Start one loop on one named OS thread after CPython and bridge-loader setup;
    publish a thread-safe submission handle only after loop readiness.
  - Maintain an explicit accepting/running/stopping/stopped state machine and a
    registry keyed by monotonically assigned submission ids. Reject work once
    shutdown starts, and prove initialization failure cannot leave a thread.
  - Wire loop bootstrap only when the resolved target uses an async Python
    declaration or the raw coroutine intrinsic. Replace raw `asyncio.run` with
    owned-loop submission while preserving the raw API's synchronous
    `blocking_io` classification and explicit-offload requirement.
  - Prove repeated and concurrent raw calls use one loop/thread identity.
- [x] Land the cooperative cancellation carrier and direct task paths —
  [PR #2960](https://github.com/sifr-lang/sifr/pull/2960):
  - Replace the direct generated-task abort handle with a cancellation carrier
    for `task.cancel`, cancel-and-join, and timeout. At Python-await entry the
    submission atomically claims that carrier and registers its exact-task
    cancellation hook.
  - A claimed cancellation signals the exact asyncio task and makes the Sifr
    supervisor await the child and Python terminal latch; an unclaimed
    cancellation keeps the existing Tokio-abort behavior. A cancellation racing
    registration either aborts before Python submission or is observed by the
    newly registered submission, never leaving untracked Python work.
  - Prove cancellation-before-registration, claimed terminal waiting,
    unclaimed fallback abort, and timeout without changing the behavior of
    tasks that never enter a Python await.
- [x] Complete cancellation-aware supervisors and ordered shutdown substrate —
  [PR #2962](https://github.com/sifr-lang/sifr/pull/2962):
  - Route scope/group fail-fast, race/select losers, and join-set cancellation
    through the same carrier, preserving current abort behavior for unclaimed
    ordinary Sifr tasks while terminally waiting for claimed Python work.
  - Add fail-fast sibling, race/select loser, join-set, cancellation suppression,
    and terminal-latch ordering tests before typed wrappers depend on them.
  - Define shutdown phases now: stop external admissions; invoke the M9 callback
    shutdown hook (a no-op ordered slot until M9); run registered async cleanup
    while the loop is live; cancel and terminally drain remaining submissions;
    stop the loop; join its thread.
- [x] Generate typed async declaration wrappers behind the public gate —
  [PR #2964](https://github.com/sifr-lang/sifr/pull/2964):
  - Submit compiler-private owned inputs and object-store identities; resolve
    targets, convert arguments, invoke, require an awaitable, await, and convert
    the owned result on the loop thread.
  - Cover functions, opaque factories, borrowed methods, consuming methods,
    positional/keyword/variadic/omitted arguments, recursive values, opaque
    results, package bridges, and Python/conversion/awaitable-shape failures.
  - Keep opaque values non-send in Sifr and freeze borrowed receivers for the
    full await without moving raw Python pointers across threads.
  - Prove two concurrent typed wrappers observe the same loop/thread identity,
    while the syntax remains gated until cancellation and cleanup are complete.
- [x] Complete consuming async-close lifecycle behind the public gate —
  [PR #2966](https://github.com/sifr-lang/sifr/pull/2966):
  - Require a consuming `@python.coroutine(Self.<member>)` close declaration for
    `cleanup=async_close`; transfer exclusive ownership before submission and
    close exactly once.
  - Poison on cleanup failure and reject reuse, duplicate close, or abandonment
    of an `async_close` obligation. Cover success, failure, poison,
    use-after-close, cancellation, and shutdown interaction.
- [x] Atomically activate async declarations and close M7 evidence —
  [PR #2968](https://github.com/sifr-lang/sifr/pull/2968):
  - Lift the `@python.coroutine` and `cleanup=async_close` gates only after the
    owned loop, typed wrappers, cooperative cancellation, terminal shutdown,
    and consuming lifecycle are present in the same production path.
  - Map `CancelledError` to the active Sifr cancellation cause and prove that
    cancellation waits Python `finally`; if Python suppresses cancellation, its
    later return or exception wins normally.
  - Add the full success/failure/conversion/cancellation/suppression/shutdown and
    async-close matrices, a compiled httpx-style client fixture, concurrent
    one-loop identity proof, and `demos/m7_demo` using real binary output.
  - Activate coroutine and async-close capability evidence; update user and
    architecture docs, exit evidence, roadmap, review records, milestone
    checkboxes, and merged PR links.

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

Implementation waves (one locally validated and reviewed PR per wave):

- [x] Land the gated async-context substrate —
  [PR #2970](https://github.com/sifr-lang/sifr/pull/2970):
  - Validate and retain async enter/exit declarations and
    `cleanup=async_context` obligations before emitting the existing public
    reservation; keep invalid shapes on their stable diagnostics.
  - Add the dedicated Python async-with HIR, scoped entered borrows, concrete
    body-outcome classification, original Python replay, unsuppressible Sifr
    boundary evidence, and exact-once semantic close/poison transitions.
  - Add parent/child cancellation claims, biased body cancellation, masked
    terminal exit, exact parent fallback resumption—including enter failure—and
    generated async-main carrier installation.
  - Cover lowering, codegen, runtime, cancellation, conversion, and ownership
    contracts while leaving all three M8 public surfaces reserved.
- [x] Atomically activate async contexts and close M8 evidence —
  [PR #2972](https://github.com/sifr-lang/sifr/pull/2972):
  - Lift only the async enter, async exit, and `cleanup=async_context`
    reservations; retain all M9-M12 gates and M8 diagnostics.
  - Add the compiled offline `aiosqlite` database/session matrix,
    `demos/m8_demo`, unconditional verification ownership, capability evidence,
    and public/internal documentation updates.

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

Delivery waves:

- [x] Wave 1 — gated declaration/type contract, `AsyncCallable`, lifecycle
  substrate, and callback attachment plans ([PR #2974](https://github.com/sifr-lang/sifr/pull/2974);
  merged after focused coverage, frozen-diff review, and both authoritative local gates).
- [x] Wave 2 — gated current/foreign execution and retained-owner integration
  ([PR #2977](https://github.com/sifr-lang/sifr/pull/2977); merged after frozen-diff
  review and both authoritative local gates, including 651/651 merge-profile
  E2E fixtures and 261 hardening variants).
- [x] Wave 3 — asyncio execution, atomic activation, compiled evidence, demo,
  and documentation ([PR #2979](https://github.com/sifr-lang/sifr/pull/2979);
  merged after frozen-diff review and both authoritative local gates, including
  131/131 create-PR and 651/651 merge-profile E2E fixtures, 261 hardening
  variants, and compiled CFFI, Kafka, asyncio, and Pub/Sub evidence through
  `demos/m9_demo`).
- [x] Aggregate-review remediation Wave 1 — attachment-site proof for foreign
  and asyncio handler captures, including non-send and Python-identity
  exclusion, parallel share safety, same-owner rejection, and unproven callable
  rejection ([PR #2981](https://github.com/sifr-lang/sifr/pull/2981);
  [GPT-5.6-Sol High review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-codex-5-6-sol-high-review-pass-1.md)).
- [x] Aggregate-review remediation Wave 2 — per-entry asyncio terminal records,
  exact bidirectional cancellation, asynchronous owner cancel/join, rollback
  drain without executor blocking, and retained loop authority for async
  unregister during shutdown ([PR #2982](https://github.com/sifr-lang/sifr/pull/2982);
  create-PR `131/131`, signature `7c39b8c1dd4fec7c`; merge `651/651`,
  signature `ee5e5d44306f270c`, 261 hardening variants; addresses review findings
  2 and 3 and the active-close portion of finding 4).
- [x] Aggregate-review remediation Wave 3 — cancellation/finalization race
  closure, failed context-entry owner reconciliation, terminal provisional
  receiver rollback, and an emitted Rust `!Send`/`!Sync` opaque-identity
  backstop ([PR #2984](https://github.com/sifr-lang/sifr/pull/2984);
  [GPT-5.6-Sol High review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-remediation-wave3-codex-5-6-sol-high-review-pass-2.md),
  [pass 3 findings](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-remediation-wave3-codex-5-6-sol-high-review-pass-3.md));
  create-PR `131/131`, signature `7c39b8c1dd4fec7c`, with compiled sync and
  asyncio reconciliation fixtures plus same-loop Python cancellation evidence
  that keeps identity-bearing context values on their owning task. Pass 3
  remediation adds unconditional late-cancellation release on failed async
  entry, owner-local retained foreign callback identities, awaited foreign
  drains in async wrappers, nested typed-error-union registration/mapping, and
  compiled typed handler-error plus post-call closure evidence.
- [x] Aggregate-review remediation Wave 4 — add a distinct compiled CFFI
  caller-thread fixture for `dispatch=current`, retain the worker-thread CFFI
  fixture as `dispatch=foreign` evidence, and correct capability ownership and
  verification documentation after the complete merged M9 review exposed the
  certification mismatch
  ([PR #2985](https://github.com/sifr-lang/sifr/pull/2985);
  [GPT-5.6-Sol High complete review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-complete-codex-5-6-sol-high-review-pass-1.md),
  [remediation review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-current-dispatch-remediation-codex-5-6-sol-high-review-pass-1.md));
  focused callback examples pass all seven compiled binaries and the
  authoritative create-PR gate passes `131/131`, signature
  `7c39b8c1dd4fec7c`; merged as `71087cfd948b226d6fba2868d18ebea88f21214a`.
- [x] Milestone review — the complete merged M9 implementation passed the
  [GPT-5.6-Sol High review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m9-complete-codex-5-6-sol-high-review-pass-2.md)
  after all pass-1 blockers were remediated and merged.

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

Delivery waves:

- [x] Wave 1 — keep `@python.buffer` reserved while replacing the legacy
  `uint8`-only store with a closed typed buffer request, complete metadata and
  layout validation, lock-free Python operations, bounded typed access, and
  exact-once sealed-resource release
  ([PR #2987](https://github.com/sifr-lang/sifr/pull/2987);
  [review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave1-codex-5-6-sol-high-review-pass-1.md),
  [pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave1-codex-5-6-sol-high-review-pass-2.md),
  [pass 3](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave1-codex-5-6-sol-high-review-pass-3.md),
  [satisfied pass 4](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave1-codex-5-6-sol-high-review-pass-4.md));
  focused buffer tests pass `19/19`, the complete Python runtime suite passes
  `145/145`, and the authoritative create-PR gate passes Python interop `10/10`
  plus E2E `131/131` with signature `7c39b8c1dd4fec7c`.
- [x] Wave 2 — add the compiler-known affine `python.Buffer[T]` contract,
  decorator validation, `Self` and call-then-acquire lowering/code generation,
  exclusive writable borrowing, early release, and atomic public activation
  ([PR #2988](https://github.com/sifr-lang/sifr/pull/2988)); focused lowering
  contracts pass `17/17`, focused buffer code generation passes `6/6`, permanent
  native top-level, receiver, bridge, and affine-aggregate examples pass `4/4`
  with zero live resources, and the authoritative create-PR gate passes Python
  interop `11/11`, all enforced lane budgets, runtime platform `28/28`, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c`. Codex `gpt-5.6-sol` high/fast
  [review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-1.md)
  requested receiver codegen, recursive affine capability, permanent compiled
  evidence, and atomic activation remediation; all four findings are addressed
  and authoritatively validated. Full-diff
  [review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-2.md)
  found remaining union/aggregate affine synthesis, receiver-convention, and
  tracking issues; remediation round 2 was implemented and focused validation
  passed. Full-diff
  [review pass 3](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-3.md)
  found residual collection capabilities, constructor/walrus/comprehension
  moves, and exporter-level writable aliasing. Those findings were remediated.
  Full-diff
  [review pass 4](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-4.md)
  then found cross-view writable admission, iterator/generator and conditional
  expression moves, dynamic/generic collection capabilities, missing permanent
  coverage, and stale `PYZC` documentation. Those findings are remediated with
  exporter-footprint conflict admission, closed affine iterator and generator
  paths, recursive conditional moves, collection capability checks, permanent
  compiler/runtime regression coverage, and corrected architecture text.
  Full-diff
  [review pass 5](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-5.md)
  cleared those runtime and documentation corrections but found borrowed
  affine escape through owned calls and aggregates, missing `min`/`max` and
  dynamic collection capability checks, and untracked tuple/dict/list-`+=`
  moves. Those compiler paths are now closed through one range-aware affine
  ownership-transfer gate, operation-specific dynamic capability validation,
  complete constructor and augmented-assignment moves, and permanent negative
  coverage. Focused lowering contracts pass `26/26`, focused runtime buffer
  operations pass `15/15`, full code generation passes `810/810`, lowering
  passes `736/736` with one ignored, type system passes `97/97`, and the
  Python-enabled runtime passes `203/203`. After `cargo clean` removed a
  provenance-tainted macOS build tree, a cold and a subsequent warm
  authoritative create-PR facade passed every blocking lane: Python interop
  `11/11`, runtime platform `28/28` with one gated skip, and E2E `131/131` with
  signature `7c39b8c1dd4fec7c`. The warm facade completed in `415.80s`; all lane
  budgets passed, with only the non-blocking warm wall-time advisory. Full-diff
  [review pass 6](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-6.md)
  confirmed the runtime lifecycle and prior exact compiler remediations, then
  found list repetition capability gaps, variadic `min`/`max` ordering gaps,
  affine-list self-`+=`, and stale activation evidence. Those paths are now
  closed by a dedicated list-repetition clone-capability check, real ordering
  validation for variadic `min`/`max`, safe assignment rewrites for cloneable
  list `*=` and self-`+=`, exact affine/dynamic negative contracts, and corrected
  activation status. Focused type capability tests pass `1/1`, buffer lowering
  contracts pass `26/26`, buffer code generation passes `7/7`, full lowering
  passes `736/736` with one ignored, full type-system tests pass `98/98`, and
  full code generation passes `811/811`. The file-size guardrail passes over
  `2601` files. The authoritative create-PR facade passes every blocking lane:
  Python interop `11/11`, runtime platform `28/28` with one gated skip, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c`. The facade completed in
  `423.95s`; all lane budgets passed, with only the non-blocking warm wall-time
  advisory. Full-diff
  [review pass 7](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-7.md)
  confirmed the prior lifecycle, release, concurrency, overlap-safety,
  declaration, bridge, diagnostic, and retention work, then found recursive
  affine collection/storage transfers, indirect self-`+=`, recursive dynamic
  clone capability, generic and borrowed-string `min`/`max`, async-generator
  sendability, exact strided footprint, and evidence-ledger gaps. Those paths
  are now closed with recursive range-aware ownership transfer, move-only
  storage code generation, recursive clone capability, concrete total-order
  validation, owned string result emission, affine async-generator rejection,
  exact logical-item footprint admission, and permanent compiler/runtime/E2E
  regressions. Focused buffer lowering passes `28/28`, buffer code generation
  passes `9/9`, buffer runtime operations pass `23/23`, and the full affected
  suites pass: type system `98/98`, lowering `738/738` with one ignored, code
  generation `813/813`, and Python-enabled runtime `204/204`. The
  borrowed-string `min`/`max` E2E fixture also builds and runs as a native
  release binary. After the requested `cargo clean`, the cold create-PR run
  passed every functional case but exceeded the Python interop warm-lane budget
  while rebuilding callback artifacts. The immediate warm authoritative rerun
  passed every blocking lane: Python interop `11/11`, runtime platform `28/28`
  with one gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c`; its `445.31s` wall time produced only the non-blocking
  warm wall-time advisory. Full-diff
  [review pass 8](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-8.md)
  confirmed the prior release, bridge, sendability, and recursive capability
  work, then found affine membership, chained/unpack ownership, affine match,
  large-contiguous footprint scaling, exact indirect admission, and activation
  evidence gaps. Those paths are now closed by explicit affine operation
  diagnostics, consuming tuple-unpack semantics, a permanent native-negative
  membership fixture, constant-space contiguous admission, exact indirect
  logical-item ranges, linear merged-range overlap checks, and corrected
  architecture and activation evidence. Focused buffer lowering passes `30/30`,
  buffer code generation passes `9/9`, buffer runtime operations pass `18/18`,
  and the complete compile-fail matrix passes `482/482`. Full affected suites
  pass: type system `98/98`, lowering `740/740` with one ignored, code generation
  `813/813`, and Python-enabled runtime `206/206`. The first post-change
  create-PR run passed all functional Python interop cases but exceeded that
  step's warm budget while rebuilding callback and buffer examples. The
  immediate authoritative rerun passed every blocking lane: Python interop
  `11/11` in `103.07s`, runtime platform `28/28` with one gated skip, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c`; all step budgets passed and the
  `800.94s` uncached overall wall time produced only the non-blocking warm
  wall-time advisory. Full-diff
  [review pass 9](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-9.md)
  verified all pass-8 runtime admission remediations, then found that the
  compiler still overclaimed recursive equality, borrowed/star-unpack cloning,
  and nested async-generator capture support. Those accepted-invalid paths are
  now closed by Rust-trait-accurate recursive equality and clone capabilities,
  `None`-only identity operators, borrowed tuple cloning versus owned tuple
  moves, non-clone star-unpack rejection, and yield-aware free-variable capture
  analysis with explicit nested async-generator rejection. Four native-negative
  fixtures and one native-positive borrowed/owned tuple fixture make the
  boundary permanent. After the requested `cargo clean` removed `28.1 GiB`,
  focused type-system tests pass `99/99`, buffer lowering passes `32/32`, buffer
  code generation passes `10/10`, and the complete compile-fail matrix passes
  `486/486`; the borrowed/owned tuple fixture also builds and runs as a native
  release binary. Full affected suites pass: lowering `742/742` with one
  ignored, code generation `814/814`, and Python-enabled runtime `206/206`.
  The first runtime run exposed one pre-existing timing-sensitive async shutdown
  failure; its exact rerun and the immediate complete runtime rerun both passed.
  Workspace Clippy is warning-free, formatting/diff checks pass, and the HIR
  maintainability and `900`-line file-size guardrails pass over `2610` files.
  The cold authoritative create-PR facade passed every blocking lane after the
  clean: Python interop `11/11`, runtime platform `28/28` with one gated skip,
  and E2E `131/131` with signature `7c39b8c1dd4fec7c`. Its `995.68s` wall time
  produced only the expected non-blocking warm-target advisory because all 42
  native E2E groups rebuilt from the empty cache. Full-diff
  [review pass 10](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-10.md)
  then found union equality generation, set/dictionary `Eq + Hash` requirements,
  non-clone chained assignment, unconditional union formatting traits, and
  stale activation evidence. Those accepted-invalid paths are now closed by
  distinct recursive `PartialEq`, `Eq + Hash`, `Debug`, and `Display`
  capabilities; conditional class/newtype/union derives and union formatting;
  hash-aware membership and equality validation; and non-clone chained-
  assignment rejection. The exact native-positive union/class reproduction now
  builds and runs through the Rust backend, including corrected boxing and
  lifetime bounds for callable-field default constructors. Three new native-
  negative fixtures are included in the complete `489/489` compile-fail matrix,
  focused type-system tests pass `100/100`, union code generation passes `4/4`,
  callable-constructor code generation passes `1/1`, buffer lowering passes
  `32/32`, and buffer code generation passes `10/10`. Full affected suites pass:
  type system `100/100`, lowering `742/742` with one ignored, and code generation
  `817/817`. Workspace Clippy is warning-free, formatting and JSON checks pass,
  and the HIR maintainability and `900`-line file-size guardrails pass over
  `2615` files after trait capability analysis was split into its own focused
  module. The authoritative create-PR facade passes every blocking lane: Python
  interop `11/11` in `250.73s`, runtime platform `28/28` with one gated skip,
  and E2E `131/131` with signature `7c39b8c1dd4fec7c`. E2E rebuilt 22 of 42
  groups and completed in `163.68s`; all enforced step budgets passed, while the
  `733.86s` overall wall time produced only non-blocking warm-cache advisories.
  Full-diff
  [review pass 11](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-11.md)
  then found five broader compiler capability defects: a non-optional union
  equality stack overflow; invalid unwrapped union-member membership for lists,
  sets, and dictionaries; child derives that ignored embedded-parent traits;
  specialized generic hash-key overclaims; and error classes whose fields could
  not satisfy Rust's required `Debug` bound. The remediation now guards optional
  union recursion and injects concrete members into non-optional unions for both
  equality operand orders and all three membership containers. Class trait
  planning recursively includes the emitted parent; specialized generic set,
  dictionary, and `hash()` uses are conservatively rejected; and error shapes
  without recursive `Debug` receive `SIFR-CLASS-0006` before code generation.
  The expanded native fixture covers both union equality orders, all membership
  containers, callable-parent inheritance, and `NonSend`-parent inheritance and
  builds through the release Rust backend. Five new negative fixtures expand the
  compile-fail matrix to `494/494`. Focused suites pass: type system `101/101`,
  lowering `742` passed with one ignored, and code generation `817/817`. Workspace
  Clippy is warning-free; formatting, diff, JSON, HIR maintainability, and the
  `900`-line file-size guardrail pass over `2620` files. The authoritative
  create-PR facade also passes every blocking lane: Python interop `11/11`,
  runtime platform `28/28` with one gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. Its `429.98s` wall time produced only
  the non-blocking warm-wall-time advisory; every enforced step budget passed.
  Full-diff
  [review pass 12](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-12.md)
  confirmed the pass-11 repairs, then found that flattened transitive `NonSend`
  ancestry was not consulted by Clone, equality, hash, and debug capability
  queries, and that specialized generic keys still reached set/dictionary
  equality plus dictionary read, write, augmented-write, and delete consumers.
  One shared parent-chain query now closes every transitive `NonSend` trait
  decision, while contextual structural-equality and dictionary hash-key checks
  cover every emitted Rust consumer without rejecting provisional
  `defaultdict[Any, ...]` refinement. Permanent negative fixtures
  `transitive_non_send_equality_rejected`,
  `error_transitive_non_send_field_rejected`,
  `set_specialized_generic_equality_rejected`,
  `dict_specialized_generic_equality_rejected`, and the four
  `dict_specialized_generic_index_{read,write,augassign,delete}_rejected`
  fixtures expand the compile-fail matrix to `502/502`. Full affected suites
  pass: type system `102/102`, lowering `742` passed with one ignored, and code
  generation `817/817`; the native trait-capability fixture builds through the
  release Rust backend. Workspace Clippy is warning-free; formatting, diff, HIR
  maintainability, and the `900`-line file-size guardrail pass over `2628` files.
  The authoritative create-PR facade passes every blocking lane: Python interop
  `11/11` in `104.65s`, runtime platform `28/28` with one gated skip, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c` and `22/42` cache hits after the
  requested `cargo clean`. Its `615.77s` wall time produced only the
  non-blocking warm-wall-time and cache-hit advisories;
  every enforced step budget passed. Full-diff
  [review pass 13](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-13.md)
  confirmed the transitive `NonSend` and specialized-generic hash repairs, then
  found that dictionary projections and formatting consumers still admitted
  unavailable Rust Clone/Display/Debug traits, `dict(iterable)` did not validate
  its inferred key's `Eq + Hash`, and list methods conflated Clone, PartialEq,
  and total Ord. The remediation now gates every affected dictionary clone and
  construction path, `print`, `str`, f-string, and `repr` formatting path, and
  separates list clone, structural-equality, and total-order requirements.
  Eight permanent negative fixtures expand the complete compile-fail matrix to
  `510/510`. Full affected suites pass: type system `102/102`, lowering `742`
  passed with one ignored, and code generation `817/817`. Workspace Clippy,
  formatting, diff, HIR maintainability, and the `900`-line file-size guardrail
  pass over `2636` files. Explicit `None` formatting preserves Python spelling
  without requiring Rust unit `Display`, while compiler-owned task/failure/
  timeout/select wrappers and `JoinItemId` follow their emitted recursive
  `Debug` or bespoke `Display` implementations. The authoritative create-PR
  facade passes every blocking lane: Python interop `11/11` in `104.24s`,
  runtime platform `28/28` with one gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `41/42` cache hits. Its `444.51s` wall time produced
  only the non-blocking warm-target advisory; every enforced step budget
  passed. Full-diff
  [review pass 14](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-14.md)
  confirmed the pass-13 formatting and diagnostic-precedence repairs, then
  found that `sorted()` admitted element and callable-key result types without
  total Rust ordering; generic declarations imposed unconditional bounds while
  capability queries overclaimed generic and inherited formatting traits; and
  affine buffers could escape through reusable lambda/nested-function captures
  or acquire an incoherent walrus alias. The remediation validates the exact
  ordering type at `sorted()`, emits generic declarations and constructors
  without unrelated bounds, attaches proved bounds to conditional trait impls
  and individual methods, and formats inherited classes through their embedded
  parent. Lowering now rejects all three reusable affine escape/alias families
  before HIR. Six permanent negative fixtures expand the complete compile-fail
  matrix to `516/516`, while two native-positive fixtures cover non-clone
  generic storage and conditional generic/inherited formatting. Full affected
  suites pass: type system `102/102`, lowering `744` passed with one ignored,
  and code generation `818/818`; both positive fixtures build and run through
  the native release backend. The requested `cargo clean` removed `39.4 GiB`
  and exposed two stale-cache-hidden regressions: stdlib deduplication discarded
  distinct per-method inherent impl blocks, and signature-only bound inference
  overconstrained valid channel methods while underconstraining `Counter[T]`.
  Inherent impl identity now includes its item names, representation-required
  `Hash + Eq` bounds apply only to stored key parameters, and Clone/ordering
  bounds follow the emitted method body. The corrected generated-code corpus
  and exact create-PR E2E manifest pass, including the structural datetime,
  channel, and collection ownership reproductions. Workspace Clippy is warning-
  free; formatting, diff, HIR maintainability, and the `900`-line file-size
  guardrail pass over `2646` files. The authoritative create-PR facade passes
  every blocking lane: Python interop `11/11` in `104.05s`, runtime platform
  `28/28` with one gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. Its `416.20s` wall time produced
  only the non-blocking warm-wall-time advisory. Full-diff
  [review pass 15](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-15.md)
  confirmed ordering, formatting, affine capture/alias, deduplication, and
  file-decomposition repairs, then found that keyed `sorted()` still cloned
  non-Clone comparator elements and generic method/operator bounds still leaked
  from one type parameter onto unrelated parameters. Keyed sorting now passes
  shared-borrow keys the comparator's existing references, requires Clone only
  for owned keys or preserved-source materialization, and rejects mutable-borrow
  keys. Generic method and binary-operator impl bounds are derived per type
  parameter and propagate transitively through direct `self` method calls;
  equality operations add `PartialEq` only where consumed. Four permanent
  sorted/generic fixtures and two focused codegen/query regressions cover these
  boundaries. The requested `cargo clean` then exposed fifteen additional
  stale-cache-hidden full-suite failures: collection wrapper methods lost
  transitive Clone/PartialEq obligations, later callers retained `Any` instead
  of an inferred top-level return type, and two old parity fixtures violated the
  active structural-equality/typed-empty-collection contracts. Those root
  causes are corrected. Full affected suites pass: code generation `822/822`,
  lowering `745` passed with one ignored, compile-fail `518/518`, and native
  execution for every cold-build representative. The complete merge-profile
  E2E suite passes `657/657` with signature `18e6999f2fd35220` and `154/175`
  cache hits after the cold repair. The authoritative create-PR facade passes
  every blocking lane: Python interop `11/11` in `104.97s`, runtime platform
  `28/28` with one gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `41/42` cache hits. Its `441.13s` wall time produced
  only the non-blocking warm-wall-time advisory. A fresh whole-diff review of
  the complete remediation remains pending.
  Full-diff
  [review pass 16](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-16.md)
  then found six remaining compiler-wide gaps: emitted generic method
  requirements were not checked at concrete specialization sites; recursive
  equality and `*`, `/`, `%`, and unary operator requirements were incomplete;
  generic operator-protocol impl targets were malformed; keyed `sorted()` did
  not validate its parameter or safely materialize conditional sources;
  top-level inferred returns remained source-order dependent; and the evidence
  overstated closure. Lowering now records exact per-method, per-type-parameter
  requirements, closes direct `self` dependencies, exports/imports them with
  module signatures, and rejects unsupported concrete specializations before
  code generation. Codegen recursively derives exact arithmetic, equality,
  ordering, and negation bounds and emits generic protocol impl targets.
  `sorted()` validates the key parameter and preserves conditional branches
  without untracked moves. A diagnostic-neutral module signature prepass makes
  successful inferred returns mutually visible while ordinary body lowering
  remains the reachability-aware diagnostic authority. Five permanent fixtures
  cover transitive and recursive generic requirements, operator protocols,
  keyed and conditional sorting, and forward inferred returns. Full codegen
  passes `824/824`, lowering passes `745` with one ignored, and the complete
  compile-fail corpus passes `520/520`; all three positive fixtures build and
  run through the native backend. The `cargo clean` rebuild and responsibility-
  based splits leave the touched orchestration files at `898` and `892` lines;
  HIR maintainability and the `900`-line file-size guardrail pass over `2659`
  files. The authoritative create-PR gate passes every blocking lane, including
  Python interop `11/11` in `106.69s`, runtime platform `28/28` with one gated
  skip, and E2E `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache
  hits. Its `489.54s` wall time produced only the non-blocking warm-target
  advisory; every enforced step budget passed. Full-diff
  [review pass 17](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-17.md)
  then reproduced six remaining gaps: aliased generic imports lost canonical
  specialization identity; operator consumers skipped conditional impl checks;
  generic ordering ignored the declared `__lt__` body and its `PartialEq`
  supertrait; mixed conditional sorting bypassed branch-local Clone admission;
  union-return inference remained declaration-order dependent; and generic
  negation evidence did not execute negation. The remediation preserves local
  aliases separately from canonical class identity, validates method and
  operator specializations through one requirement map, emits ordering through
  the actual `__lt__` body with exact representation equality bounds, checks
  every preserved sorting branch, performs union-aware fixed-point module
  inference, and supports TypeVar/class negation end-to-end. Six permanent
  negative fixtures plus aliased-import and generic operator runtime evidence
  cover the reproductions. Focused native execution passes for aliased `deque`,
  multi-parameter ordering, and generic negation; full codegen passes `824/824`,
  lowering passes `747` with one ignored, and compile-fail passes `526/526`.
  The requested post-remediation `cargo clean` rebuild confirms the file-size
  guardrail over `2667` files and HIR maintainability. The authoritative
  create-PR gate passes every blocking lane, including Python interop `11/11`,
  runtime platform `28/28` with one gated skip, and cold-cache E2E `131/131`
  with signature `7c39b8c1dd4fec7c` and all `42` groups rebuilt. Its `855.62s`
  wall time produced only the expected non-blocking warm-target advisory; every
  enforced step budget passed. Full-diff
  [review pass 18](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-18.md)
  then found four remaining compiler defects plus overstated evidence:
  transitive helper bounds were absent from operator trait impls; user-project
  generic aliases lost their emitted identity and same-name modules collided;
  module signature inference ignored `match`, `try`, and context statements;
  and valid nested custom class operators were rejected during specialization.
  Operator impls now combine the lowering-owned semantic closure with exact
  helper-impl bounds. User-project imports retain collision-safe local class,
  method-signature, requirement, and generic-template identities while merged
  stdlib imports remain canonical. Module inference covers compound bindings,
  returns, and conservative terminal reachability. Concrete class negation,
  equality, and ordering dunders satisfy generic requirements recursively,
  while binary ownership shapes that cannot meet owned Rust generic traits
  remain conservatively rejected. Permanent native, compile-fail, focused
  lowering/codegen, and two-module project-build regressions cover every
  reproduction. Full code generation passes `825/825`, lowering passes `750`
  with one ignored, and compile-fail passes `527/527`. Workspace Clippy,
  formatting, JSON and diff checks, HIR maintainability, and the `900`-line
  file-size guardrail pass over `2672` files. The authoritative create-PR gate
  passes every blocking lane: Python interop `11/11`, runtime platform `28/28`
  with one gated skip, and E2E `131/131` with signature `7c39b8c1dd4fec7c` and
  `42/42` cache hits. Its `476.95s` wall time produced only the non-blocking
  warm-wall-time advisory. Full-diff
  [review pass 19](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-19.md)
  then found three remaining root-cause gaps: operator helper closure recognized
  only `self` receivers; user-module alias rewriting stopped at imported class
  bodies rather than exported functions and inheritance; and compound inference
  substituted `Unknown` or the context-owner type for class-pattern and
  `with ... as` bindings. Method dependencies now follow every receiver of the
  current class. User-module class aliases are collected across the complete
  import set and recursively rewrite function, constant, class, and transitive
  parent types; frontend exports retain ancestry instead of erasing it.
  Class-pattern inference reads declared field types and context bindings read
  `__enter__` results. The resulting native match-capture regression also fixed
  owned return materialization from borrowed Rust match captures. Permanent
  peer-receiver, separate-statement same-name factory, aliased ancestry,
  class-pattern field, entered-result, and native compound fixtures cover the
  exact reproductions. Full codegen passes `825/825`, lowering passes `752`
  with one ignored, and frontend passes `47/47`; focused project builds and
  native fixtures pass. Workspace Clippy, formatting, JSON/diff checks, HIR
  maintainability, and the `900`-line file-size guardrail pass over `2672`
  files. The authoritative create-PR gate passes every blocking lane: Python
  interop `11/11`, runtime platform `28/28` with one gated skip, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache hits. Its
  `482.54s` wall time produced only the non-blocking warm-wall-time advisory.
  Full-diff
  [review pass 20](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-20.md)
  then found that multi-hop facade re-exports did not localize embedded
  user-class identities and generic class-pattern captures read unspecialized
  declaration fields. Frontend re-exports now localize the complete imported
  function, class, constant, and parent type graph against all class aliases in
  the source module. Both inference and final pattern lowering specialize
  fields from the narrowed subject type, including nested generic patterns;
  project code generation also resolves generic class metadata through facade
  chains. Permanent regressions cover two same-name generic classes re-exported
  through a facade, factory signatures, cross-assignment rejection, three-level
  ancestry, and direct plus nested generic captures. Full codegen passes
  `825/825`, lowering passes `754` with one ignored, and frontend passes `47/47`;
  focused project checks and the native multi-module build pass. Workspace
  Clippy and formatting are clean, HIR/driver maintainability passes, and the
  `900`-line file-size guardrail passes over `2673` files after re-export alias
  collection was split into a focused frontend module. The authoritative
  create-PR gate passes every blocking lane: Python interop `11/11` in
  `111.34s`, runtime platform `28` variants with one gated skip, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache hits. Its
  `532.87s` wall time produced only the non-blocking warm-target advisory. A
  fresh whole-diff review remains pending. Full-diff
  [review pass 21](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-21.md)
  then showed that identity still depended on symbols sharing one facade,
  user-module generic requirements disappeared at the export boundary, and a
  union of two specializations of the same generic class selected its first
  member and emitted invalid Rust. Class types now carry stable declaration
  identity independently of their local emitted spelling; the identity and
  canonical ancestry are preserved through every function, constant, class,
  factory, and re-export path. Frontend exports and re-exports now carry generic
  callable and per-method requirement metadata. Until the backend has distinct
  variants for repeated generic-class specializations, those unions are
  rejected as invalid annotations before HIR/codegen. Permanent tests split
  classes from factories and roots from leaves across separate facades, reject
  a multi-hop unavailable generic method, reject direct and nested repeated-
  specialization patterns, and build the complete split-path positive project
  natively. Full affected suites pass: type system `103/103`, lowering `756`
  with one ignored, frontend `47/47`, codegen `825/825`, and driver `347` with
  `22` ignored. Workspace Clippy, formatting, HIR/driver maintainability, and
  source-size guardrails pass over `2674` files; touched orchestration files
  remain below `900` lines. The clean-build authoritative create-PR gate passes
  every blocking lane: Python interop `11/11` in `106.89s`, runtime platform
  `28` variants with one capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` after rebuilding all `42` fixture groups. Its `911.81s`
  wall time produced only the expected non-blocking warm-target advisory after
  `cargo clean`. Full-diff
  [review pass 22](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-22.md)
  then found four remaining generic and identity gaps: nominal assignability
  ignored concrete specializations, imported-parent ancestry stayed
  path-local, user-module imports did not install exported generic-function
  metadata, and return inference emitted unsubstituted generic constructor
  results. The remediation gives nominal classes explicit invariant type
  arguments, canonicalizes imported-parent ancestry, emits executable parent
  dereference bridges, installs generic callable metadata through user-module
  facades, propagates project codegen signatures, and substitutes generic calls
  during both return inference and concrete annotated-initializer lowering.
  Optional contextual inference binds only the non-`None` payload, and nominal
  class arguments participate in TypeVar discovery. Permanent regressions cover
  every reported direct, inferred, multi-hop, bound, native, and recursive
  shape. Full affected suites pass: type system `103/103`, frontend `47/47`,
  lowering `763` with one ignored, codegen `825/825`, driver `350` with `25`
  ignored, and the native driver lane `25/25`. Workspace Clippy, formatting,
  HIR/driver maintainability, and the `900`-line source-size guardrail pass over
  `2678` files. The authoritative create-PR gate passes every blocking lane in
  `471.02s`: Python interop `11/11`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. Its only advisory is the
  non-blocking warm wall-time target. Full-diff
  [review pass 23](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-23.md)
  then found two native-codegen boundaries still incomplete: contextual
  zero-argument generic specialization did not survive into emitted Rust, and
  child-to-ancestor assignability had executable coercions only for borrowed
  values, not consuming arguments or returns. Explicit class arguments now
  drive generic-return specialization, concrete contextual locals retain their
  Rust type annotation, and compiler-owned `PhantomData` represents fieldless
  generic parameters. Direct move-based `From` bridges plus one generated
  conversion per ancestry edge make consuming arguments, locals, and returns
  executable for direct, transitive, and imported/re-exported inheritance.
  Exact zero-argument generic and owned-upcast projects build and run natively.
  Full affected suites pass: codegen `827/827`, driver `350` with `27` ignored,
  and the native driver lane `27/27`. Workspace Clippy, formatting, HIR/driver
  maintainability, and the `900`-line source-size guardrail pass over `2680`
  files. After `cargo clean`, the authoritative create-PR gate passes every
  blocking lane in `861.81s`: Python interop `11/11`, runtime platform `28`
  variants with one capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` after rebuilding all `42` fixture groups. Its only advisory
  is the expected non-blocking warm-target timing warning. Full-diff
  [review pass 24](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-24.md)
  cleared runtime lifecycle, overlap admission, exact release, and no-panic
  paths, then found four compiler boundaries: nested writable receiver places,
  union/`Result` consuming upcasts, same-basename canonical ancestor selection,
  and phantom derive/auto-trait alignment. Receiver root tracing now covers
  legal nested places while the existing whole-aggregate affine rule rejects
  buffer field/index projections before mutation; union and `Result` payloads
  receive consuming conversions before wrapping; exact canonical ancestry wins
  over only-unambiguous basename fallback; and fieldless generics use
  non-owning `PhantomData<fn() -> T>` with concrete type arguments included in
  Clone, structural-equality, Hash, and Debug capability checks. Permanent
  direct, transitive, imported/re-exported, repeated-basename, non-capable
  generic, and non-send native regressions pass. Full codegen passes `828/828`,
  driver passes `350/350`, and the native generated-project lane passes `28/28`.
  Workspace Clippy and formatting are clean; HIR/driver maintainability and the
  `900`-line source-size guardrail pass over `2680` files. The authoritative
  create-PR gate passes every blocking lane in `537.98s`: Python interop
  `11/11`, runtime platform `28` variants with one capability-gated skip, and
  E2E `131/131` with signature `7c39b8c1dd4fec7c` and `36/42` cache hits. Its
  warm-time and cache-hit notices are non-blocking advisories. Full-diff
  [review pass 25](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-25.md)
  cleared the pass-24 buffer, writable-place, ancestry-selection, and phantom
  corrections, then found that owned coercion is not recursive across existing
  union/`Result` representations and recursive affine, trait, and sendability
  guards still key distinct imported classes by basename. Shared consuming
  coercion now recursively maps existing union, `Option`, and `Result` values
  across calls, locals, and returns; borrowed representation changes require a
  Clone-capable source and mutable borrowed changes are rejected. Recursive
  ownership, Clone, equality, Hash, Debug, bounds, and task-sendability guards
  now use canonical declaration identity plus concrete specialization. Same-
  basename and specialization regressions cover each affected capability. Full
  type-system, lowering, codegen, and driver suites pass `106/106`, `765/765`
  active with one ignored, `829/829`, and `351/351` active with `29` ignored;
  the native generated-project lane passes `29/29`. Workspace Clippy,
  formatting, HIR/driver maintainability, and the `900`-line source-size
  guardrail pass over `2682` files. The authoritative create-PR gate passes all
  blocking lanes in `536.29s`: Python interop `11/11`, runtime platform `28`
  variants with one capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. Its warm-time notice is a
  non-blocking advisory. Whole-diff
  [review pass 26](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave2-codex-5-6-sol-high-review-pass-26.md)
  independently re-grounded both findings, exercised an additional owned
  `AsyncCallable` structural-upcast build and all `18` Python buffer runtime
  tests, and returned **SATISFIED** with no actionable findings.
- [x] Wave 3 — add complete positive/negative/cleanup matrices, compiled
  import-root, bridge, receiver, and NumPy-compatible evidence, demo and public
  documentation, and complete activation evidence
  ([PR #2989](https://github.com/sifr-lang/sifr/pull/2989)). The implementation adds a
  validator-owned declaration evidence matrix, a fifth compiled real-NumPy
  case, `demos/m10_demo`, and active `PYZC` public docs. Focused lowering
  (`34/34`), codegen (`10/10`), runtime buffer matrix (`30/30`), runner self-test, scaffold,
  and five-binary demo checks pass. The authoritative create-PR gate passes
  Python interop `11/11` and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` after a clean build. Whole-diff
  [review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-1.md)
  requested independent pointer/exact-release evidence, a strict evidence
  schema, least-authority generated manifests, and an architecture wording
  repair. [Review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-2.md)
  requested CPython 3.11-compatible exporters, exact owner sets, narrower
  compiled-identity claims, and a consistent phase status. [Review pass
  3](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-3.md)
  verified the native release and packaging paths, then requested a pinned 3.11
  lane, complete native-negative fixture ownership, and current review
  tracking. [Review pass 4](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-4.md)
  then found false-pass paths for zero selected runtime tests and an empty
  compiled-case result. Exact named-test and registered-case validation plus
  adversarial self-tests remediate those findings. [Review pass 5](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-5.md)
  found duplicate runtime observations were collapsed and the primitive runtime
  owner matrix overstated pointer-width evidence. Observation multiplicity,
  missing/duplicate self-tests, exact raw ownership, and C-level round trips for
  every supported primitive remediate both findings. Whole-diff
  [review pass 6](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-wave3-codex-5-6-sol-high-review-pass-6.md)
  re-ran adversarial exactness checks, the pinned CPython 3.11 release tests,
  all eleven primitive-family round trips, the native-format matrix, evidence
  validation, generated least-authority manifest inspection, and integrity
  checks, then returned **SATISFIED** with no actionable findings. The
  authoritative create-PR gate passes every blocking lane in `920.39s`:
  Python interop `12/12` including the five release tests and five compiled
  examples on CPython `3.11.14`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `27/42` cache hits. Warm-time and cache-hit notices are
  non-blocking advisories. PR #2989 is merged.
- [x] Milestone review — complete merged-M10
  [review pass 1](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-1.md)
  found writable `Self` owner aliasing and name-only `PythonError` acceptance.
  Remediation rejects writable receiver acquisition until an exclusive owner
  freeze is representable, adds permanent owner-close negative evidence, and
  shares an exact Python error field-contract predicate across lowering, method
  typing, and code generation. Focused buffer lowering passes `37/37`, the
  complete compile-fail matrix passes `528/528`, the type-system and codegen
  mapping regressions pass, and the Python interop evidence self-test passes.
  Workspace Clippy, formatting, HIR maintainability, and the `900`-line
  file-size guardrail pass over `2690` files. The authoritative create-PR gate
  passes every blocking lane: Python interop `12/12` including CPython `3.11`
  runtime `5/5` and compiled `5/5`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c`. Its `892.55s` wall time and `15/42` cache hit rate produce
  only non-blocking advisories. Full
  [review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-2.md)
  re-ran the complete focused compiler/runtime/native buffer evidence, confirmed
  the pass-1 ownership and error-channel remediations, and found only a stale
  roadmap reference to the already-merged PR #2988. That ledger reference is
  corrected in PR #2990; final re-review is in progress before closing the
  milestone checkbox. Full
  [review pass 3](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-3.md)
  confirmed the compiler remediations and corrected roadmap, then found that
  generic ignore rules omitted checksum-required vendored files from a clean
  checkout, public documentation overstated nominal `PythonError` enforcement,
  and exit evidence still listed active `PYZC` as reserved. PR #2990 now tracks
  all three remediations. Full
  [review pass 4](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-4.md)
  confirmed those fixes and the clean-checkout vendor inventory, then found that
  duplicate canonical fields could still pass the structural `PythonError`
  predicate and reach the generated-Rust duplicate-field assertion. The shared
  predicate now requires exactly five canonical fields, and lowering plus driver
  regressions lock check/compile diagnostic parity. Full
  [review pass 5](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-5.md)
  confirmed that remediation, then found writable producers could return a
  borrowed opaque/Object parameter while leaving its Sifr owner usable. Writable
  producer parameters that can transitively carry Python identity now require
  `own`, with permanent lowering, compile-fail, documentation, and evidence
  coverage before final re-review. Full
  [review pass 6](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-6.md)
  confirmed those ownership fixes, then found that raw Python `Object` still
  used basename matching and that exact `PythonError` validation remained
  buffer-specific. The stdlib export boundary now preserves the originating
  `_sifr.python.Object` identity through public re-exports; lowering, recursive
  ownership analysis, callback typing, and code generation use that canonical
  identity, while same-named user records retain record conversion. The shared
  duplicate-safe five-field error predicate now guards ordinary, callback,
  context, coroutine, and buffer declarations, with lowering, codegen, stdlib,
  and driver check/compile parity regressions. Full
  [review pass 7](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-7.md)
  confirmed those remediations, then reproduced three remaining High-severity
  parity defects: coroutine conversion still selected a local `Object` by
  basename; the canonical stdlib `Object` alias collided with a user record in
  flat generated Rust; and callback error unions could contain distinct
  same-basename members that emitted duplicate variants. Async conversion and
  context classification now use the shared canonical predicates. The sealed
  handle initially emitted as `__SifrPythonObject` while source-level local
  `Object` records retained record conversion, and a native package-build
  regression compiled synchronous and coroutine declarations together.
  Callback lowering rejects duplicate generated variant names before codegen,
  with lowering and driver check/compile parity coverage. The first
  authoritative gate exposed one additional strict-identity mismatch where the
  compiler-special `open()` path replaced an imported canonical
  `TextFileHandle` with an anonymous same-named type. Compiler-special text and
  binary opens now preserve imported handle identities, with a focused lowering
  regression and the previously failing text/i18n fixture compiling and running
  natively. The repeated authoritative create-PR gate passes every blocking
  lane in `629.99s`: Python interop `12/12`, runtime platform `28` variants with
  one capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `41/42` cache hits. The warm-time notice is a
  non-blocking advisory after the requested clean rebuild. Full
  [review pass 8](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-8.md)
  confirmed the prior identity and error-union fixes, then showed that
  `__SifrPythonObject` itself remained a legal user class name and that
  compiler-special `open()` still resolved text and binary handles through
  basename keys. Canonical Python `Object` now renders directly as the fully
  qualified runtime handle and emits no alias into the user's flat Rust
  namespace; the native package regression includes the reviewer's exact
  `class __SifrPythonObject` collision. Text and binary opens now select
  imported handles by canonical `sifr.io` identity across aliases, synthesize
  the same canonical identity when unimported, and reject local same-basename
  shadows. Focused type-system, lowering, codegen, stdlib-bootstrap, native
  package, aliased-open native build, and shadow-rejection regressions pass. The
  repeated authoritative create-PR gate passes every blocking lane in
  `822.00s`: Python interop `12/12`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `42/42` cache hits. Its warm wall-time notice is a
  non-blocking advisory after the requested clean rebuild. Full
  [review pass 9](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-9.md)
  confirmed the pass-8 fixes, then proved two remaining generated-Rust identity
  collisions: local `sifr_runtime` shadowed relative external-crate paths, and
  local `FileHandle`/`TextFileHandle` classes collided with canonical inferred
  `open()` results. Generated runtime and stdlib paths are now absolute across
  Rust IR, imports, bridge types, and assembled stdlib source. Canonical file
  handles use compiler-owned Rust names in declarations, intrinsics, fallback
  preambles, ordinary type rendering, and generic-aware type rendering while
  retaining their nominal Sifr identities. Permanent lowering, renderer,
  stdlib-filter, and native package regressions lock the reviewer's exact local
  shadow cases. Sealed Python `Object` conversion now unwraps and rewraps the
  canonical runtime handle through compiler-private runtime bridge functions
  for synchronous and coroutine declarations. Both exact native collision
  proofs pass. After the requested `cargo clean`, the repeated authoritative
  create-PR gate passes every blocking lane in `1334.68s`: Python interop
  `12/12`, runtime platform `28` variants with one capability-gated skip, and
  E2E `131/131` with signature `7c39b8c1dd4fec7c` and `0/42` cold-cache hits.
  The warm wall-time notice is a non-blocking advisory for this clean rebuild;
  a proactive follow-up proof then showed that the compiler-owned handle names
  themselves were source-spellable. Source-declared `__Sifr*` classes now use
  an injective escaped Rust namespace disjoint from compiler-owned identities,
  and the strengthened native collision package passes with exact local
  `__SifrIoFileHandle` and `__SifrIoTextFileHandle` classes. The follow-up
  authoritative create-PR gate passes every blocking lane in `657.69s`: Python
  interop `12/12`, runtime platform `28` variants with one capability-gated
  skip, and E2E `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache
  hits. Its warm wall-time notice is a non-blocking advisory. Full
  [review pass 10](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-10.md)
  then reproduced three remaining generated-Rust collision families: source
  newtypes, enums, and protocols named with compiler prefixes; local `std`
  classes capturing compiler-owned relative paths; and source classes colliding
  with display-derived union enums or variants. One shared injective source-name
  renderer now covers every nominal definition and reference. Token-aware Rust
  rendering absolutizes compiler-owned standard-library and external-crate
  roots without rewriting strings, character literals, raw strings, or
  comments. Compiler-owned union enums and variants now use exact structural
  identities derived from emitted nominal names, so canonicalized and local
  views of the same Rust payload agree while source aliases remain distinct.
  The native collision package covers regular classes, newtypes, enums,
  protocols, `class std`, source-spellable compiler prefixes, and adversarial
  unions. A clean create-PR E2E run exposed and fixed the canonical/local
  `JsonValue` union split; the focused rerun passes `131/131` with only that
  group rebuilt. After the requested `cargo clean`, the final authoritative
  create-PR gate passes every blocking lane in `605.42s`: Python interop
  `12/12`, runtime platform `28` variants with one capability-gated skip, and
  E2E `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache hits. The
  warm wall-time notice is a non-blocking advisory. Full
  [review pass 11](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-11.md)
  then found that class union keys still omitted canonical identity, several
  synchronous/asynchronous conversion and inheritance paths still emitted raw
  source class names, and the merged-stdlib sealing pass protected only file
  handles rather than every canonical nominal. Remediation and native
  canonical/local collision coverage are complete in PR #2990. Nominal Rust
  identities now flow recursively through HIR types, union keys, constructors,
  operators, handlers, patterns, Python conversions, inheritance, and generated
  runtime helpers; flattened stdlib declarations are sealed by canonical module
  identity while external qualified paths remain untouched. Compiler-generated
  errors render their Sifr source names in user-visible `Debug` output even
  though their Rust identifiers remain collision-safe. Focused regressions,
  the eight previously failing E2E fixtures, and the full E2E suite pass. The
  authoritative create-PR gate passes every blocking lane in `921.00s`: Python
  interop `12/12`, runtime platform `28` variants with one capability-gated
  skip, and E2E `131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache
  hits. Its warm wall-time notice is a non-blocking advisory. Full
  [review pass 12](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-12.md)
  found four remaining exact-identity gaps: global nominal exemptions used
  basenames, inheritance discarded canonical parent types, match/try paths
  compared alias spellings, and generic/operator self shortcuts were
  basename-only. Remediation now carries exact nominal identities and parent
  types through class, `super()`, pattern, handler, generic-method, and
  operator HIR/codegen paths; only exact compiler-global identities bypass
  source-name escaping. Native regressions cover duplicate stdlib `Error`
  classes, aliased stdlib inheritance and handlers, compiler-prefixed generic
  self returns, canonical/local same-name operators, and rejected cross-module
  assignments. Focused type-system, lowering, and codegen suites pass, as do
  Clippy, formatting, maintainability, and file-size guardrails. After the
  requested `cargo clean`, the authoritative create-PR gate passes every
  blocking lane in `1504.56s`: Python interop `12/12`, runtime platform `28`
  variants with one capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `0/42` cold-cache hits. Its warm wall-time notice is a
  non-blocking advisory. Full
  [review pass 13](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-13.md)
  then found that imported parents lost exact fields and capabilities, try
  codegen collapsed the recorded error set to one handler type, and Rust bridge
  records collapsed distinct canonical classes sharing a basename. Imported
  class completion now selects the richest exact-identity surface and preserves
  parent fields, methods, derives, and `Display`. Try bodies use an exact
  discriminated carrier with exhaustive nested-raise discovery, per-member
  conversions, identity-aware handler dispatch, catch-all fallback, and carrier
  inheritance across timeout, task scope, async context, and `finally` paths.
  Rust bridge definition lookup, generated names, schemas, and recursion keys
  now include canonical module and nominal identity. Complete-suite regressions
  additionally restrict generic phantom markers to genuinely unrepresented
  parameters and make Python async contexts inherit the enclosing try carrier.
  Focused native proofs cover inherited formatting, exact TOML error routing,
  same-basename bridge records, file handles, cancellation/process/network/TLS,
  i18n, and Python async-context execution. The full merge-profile E2E suite
  passes `664/664` with signature `76a3c67a1e579374`. Workspace Clippy,
  formatting, HIR maintainability, and the `900`-line guardrail pass over `2705`
  files. The authoritative create-PR gate passes every blocking lane in
  `1024.46s`: Python interop `12/12`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c`. Its warm-time and cache-hit notices are non-blocking
  advisories. Full
  [review pass 14](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-codex-5-6-sol-high-review-pass-14.md)
  found that nested functions were skipped during synthesized try-carrier
  discovery and that marking a structural union as a carrier globally removed
  equality and hash traits required by ordinary uses of the same enum. Carrier
  discovery now traverses nested functions, and union registration records
  ordinary-value use separately from its carrier role. Carrier-only enums keep
  their minimal trait surface, while reused ordinary unions retain their proved
  `PartialEq`, `Eq`, and `Hash` derives. Unit regressions cover both usage modes;
  the native nominal-identity fixture covers nested exact-error dispatch plus
  equality and hash-set use on the shared canonical error union. Codegen passes
  `854/854`, and the authoritative create-PR gate passes every blocking lane in
  `612.79s`: Python interop `12/12`, runtime platform `28` variants with one
  capability-gated skip, and E2E `131/131` with signature
  `7c39b8c1dd4fec7c` and `41/42` cache hits. Its warm wall-time notice is a
  non-blocking advisory. The first full merge validation passed every
  functional lane but exposed approximately 20% extra frontend work from
  scanning every rendered byte against every compiler path root and cloning
  every visited HIR type during stdlib nominal canonicalization. Token-boundary
  root recognition is now a span-based lexical scan of identifiers, allocates
  only when a rewrite is required, and leaves protected Rust strings and
  comments untouched; identifier-only rendering avoids the scan entirely.
  Stdlib HIR types are canonicalized in place with an empty-class fast path.
  The representative performance matrix passes `8/8` after instruction
  count returned from approximately `15.0B` to `12.6B` for the arithmetic
  check. The same run measured current LSP RSS at `72,187,904` bytes, while
  unmodified `main` measured `70,451,200` bytes against the stale `64 MiB`
  ceiling; all LSP query ceilings are therefore recalibrated to `80 MiB`, still
  below the checked-in baseline policy's documented `baseline + 32 MiB`
  headroom for the aggregate case. Type-system `113/113`, IR `3/3`, lowering
  `779/779` plus one ignored test, codegen `856/856`, driver `358/358` plus 32
  ignored tests, and the native nominal-identity fixture pass. The next full
  gate exposed one downstream test-runner manifest gap: absolute runtime bridge
  paths in filtered private-stdlib source no longer requested the direct
  `sifr_runtime` crate because import collection intentionally ignored absolute
  paths. Runtime-crate dependency need is now tracked separately from the need
  for an unqualified `SifrInt` import. Focused collector and multi-module
  metadata regressions pass, and the exact failing `sifr test
  demos/mode_consistency` command builds and runs successfully. Clippy, format,
  HIR/driver maintainability, and the `2705`-file size guardrail pass. A
  ten-run debug-binary A/B under identical package discovery measures this M10
  branch at `1.391s` mean versus `1.563s` for unmodified `main`; a later
  representative run exceeded three absolute latency ceilings while concurrent
  macOS policy/trust services consumed substantial CPU, but retained the branch
  improvement. Fable High milestone
  [review pass 2](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-2.md)
  found declared union error channels and Python-error identity parity gaps;
  [pass 3](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-3.md)
  narrowed the remaining builtin carrier and alias collision cases;
  [pass 4](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-4.md),
  [pass 5](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-5.md),
  and
  [pass 6](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-6.md)
  drove the borrowed-field mutation fix through direct, nested, class-method,
  and field-argument shapes. Fable High
  [pass 7](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-7.md)
  verified all blockers closed and requested the authoritative merge gate plus
  a separate issue for the pre-existing class-field mutating-receiver clone
  defect. That defect and adjacent fail-closed parity gaps are tracked in
  [`ad-hoc-class-field-mutating-receiver-place-semantics.md`](ad-hoc-class-field-mutating-receiver-place-semantics.md).
  Pass 8 is retained as an explicitly invalid timer-only artifact and is not
  treated as review evidence. Fresh Fable High
  [pass 9](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-9.md)
  independently inspected the complete candidate and adversarially probed the
  mutation, identity, carrier, PythonError, and buffer surfaces, then returned
  **SATISFIED** with no blocking findings. Post-commit Fable High
  [pass 10](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-10.md)
  re-inspected the frozen `origin/main...HEAD` diff, confirmed the final commit
  was documentation-only, and returned **SATISFIED** with no blockers. The
  authoritative merge gate passes
  every blocking lane in `2685.23s`: E2E `674/674` with signature
  `1f8b1cadc4f48ec8`, Python interop `18/18`, diagnostics `175/175`, codegen
  `864/864`, lowering `784/784` plus one ignored test, driver `360/360` plus 33
  ignored tests and generated builds `33/33`, file-size guardrails over `2720`
  files, and `261` hardening variants with zero failures. Its warm-time budget
  notice is a non-blocking advisory. [PR #2990](https://github.com/sifr-lang/sifr/pull/2990)
  is the reviewed and validated M10 closure PR.

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

Delivery waves:

- [x] Wave 1 — add the five nominal affine Arrow resource types, HIR contract,
  decorator/schema grammar, exact return-kind derivation, and static ownership
  failure matrices.
- [x] Wave 2 — implement runtime and stdlib acquisition for array/schema,
  stream, device-array, and device-stream capsules with structural C-interface
  validation, paired ownership, rollback, and exact-once release.
- [x] Wave 3 — generate receiver and producer acquisition plus owned consumer
  transfer, consumed-state inspection, and failure cleanup without copy paths.
- [x] Wave 4 — bootstrap `sifr python certify arrow` and read-only
  `sifr python certify --check`, define the package artifact schema, and bind
  exact producer/distribution/schema evidence into build and archive identity.
- [x] Wave 5 — add package, native, CPython, and malformed-capsule evidence,
  the compiled Arrow example, public/durable documentation, and delivery-profile
  wiring.
- [x] Align ordinary class-method move parameters with the language-wide
  borrow-by-default rule; escaping or storing one requires explicit `own`, and
  instance, static, and `super()` calls enforce that transfer exactly once.
- [x] Milestone review — repeated whole-diff Fable High review passes
  [1](../../reviews/active/m11-arrow-full-review-pass-1.md),
  [2](../../reviews/active/m11-arrow-full-review-pass-2.md),
  [3b](../../reviews/active/m11-arrow-full-review-pass-3b.md),
  [4](../../reviews/active/m11-arrow-full-review-pass-4.md),
  [5](../../reviews/active/m11-arrow-full-review-pass-5.md),
  [6](../../reviews/active/m11-arrow-full-review-pass-6.md),
  [7](../../reviews/active/m11-arrow-full-review-pass-7.md), and
  [8](../../reviews/active/m11-arrow-full-review-pass-8.md) drove all blocker
  and major findings to closure. Pass 7 is explicitly superseded because its
  timer placeholder predated the real gate result; pass 8 independently
  rechecked the remediation and returned **SATISFIED** with no blockers or
  majors. Final frozen-diff
  [pass 9](../../reviews/active/m11-arrow-full-review-pass-9.md) verified the
  complete PR and truthful closure ledger, then again returned **SATISFIED**
  with no blockers or majors. The fresh authoritative merge gate passed every blocking lane
  in `3772.09s`: E2E `674/674` with signature `1f8b1cadc4f48ec8`, Python
  interop `20/20`, diagnostics `175/175`, codegen `878/878`, lowering
  `812/812` plus one ignored test, driver `365/365` plus 33 ignored tests and
  generated builds `33/33`, file-size guardrails over `2744` files, and `261`
  hardening variants with zero failures. The warm-time and batching-skew
  notices are non-blocking advisories. [PR #2991](https://github.com/sifr-lang/sifr/pull/2991)
  is the reviewed and validated M11 delivery PR.

Tasks:

- Add affine `python.ArrowArray`, `ArrowSchema`, `ArrowStream`,
  `ArrowDeviceArray`, and `ArrowDeviceStream` resources, with array resources
  owning their required schema/data pair.
- Implement `@python.arrow(target)` with return-kind derivation.
- Implement receiver acquisition for `Self` and call-then-acquire for import-root
  or bridge producer targets.
- Implement `schema=omitted | parameter(name)` and validate the latter as a
  same-declaration keyword-only owned `python.ArrowSchema` parameter. Requested
  schema capsules are consumed by conforming Arrow producers, so the type-level
  contract must expose the one-shot transfer rather than promise a reusable
  borrow.
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

Delivery waves:

- [x] Wave 1 — activate the nominal affine tensor/stream types, decorator and
  `parameter(name)` grammar, device/stream signature contracts, and static
  ownership failure matrices.
- [x] Wave 2 — replace the legacy raw DLPack helper with versioned runtime and
  stdlib acquisition, validation, stream synchronization, one-shot capsule
  ownership, transfer reconciliation, and exact-once deleter cleanup.
- [x] Wave 3 — generate receiver and producer acquisition plus owned consumer
  transfer across direct, method, constructor, callable, and collection paths.
- [x] Wave 4 — add CPU/device, malformed-capsule, failure-transition, package,
  PyTorch/TensorFlow, documentation, and delivery-profile evidence.
- [x] Milestone review — whole-diff Fable High
  [pass 1](../../reviews/active/m12-dlpack-full-review-pass1.md) identified
  shared call-frame cleanup, runtime-gate, and ownership-matrix gaps. Pass 2 is
  explicitly invalid and superseded because it contains progress text rather
  than a verdict. [Pass 3](../../reviews/active/m12-dlpack-full-review-pass3.md)
  found the empty-reconciliation attribute-owner regression and CPU stream
  policy mismatch; both were reproduced and fixed. Independent
  [pass 4](../../reviews/active/m12-dlpack-full-review-pass4.md) returned
  **APPROVED** after re-running the focused runtime, lowering, codegen, full
  E2E, formatting, maintainability, and file-size checks. The fresh
  authoritative merge gate then passed in `3554.66s`: E2E `674/674` with
  signature `1f8b1cadc4f48ec8`, Python interop `22/22` (including all `18`
  exact CPython 3.11 DLPack checks), diagnostics `175/175`, the full crate-test
  matrix, and `261` hardening variants with zero failures. Workspace Clippy
  passes with warnings denied and file-size guardrails cover `2761` files.
  Final frozen-diff
  [pass 5](../../reviews/active/m12-dlpack-full-review-pass5.md) repeated the
  focused tests plus roughly thirty adversarial compiler probes and returned
  **APPROVED** with no blockers or majors. Frozen-candidate
  [pass 6](../../reviews/active/m12-dlpack-full-review-pass6.md) rechecked the
  committed ledger and focused suites and again returned **APPROVED**. Merge is
  recorded before closing
  [PR #2992](https://github.com/sifr-lang/sifr/pull/2992).

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

- [x] Wave 1 — route ordinary package check and read-only inspection through
  one codegen/protocol/target-probe plan with an explicit deferred-library
  policy and structured report.
- [x] Wave 2 — add frozen, non-mutating `sifr python check` and deterministic
  patch-reporting `sifr python doctor`, including all runnable application
  targets and graph/source-content snapshot identity.
- [x] Wave 3 — add library/final-application success and failure parity,
  deterministic JSON/patch, source-digest, and byte-level non-mutation tests;
  add the blocking delivery-profile suite, runnable demo, and public/internal
  documentation.
- [ ] Wave 4 — authoritative validation, repeated full milestone review,
  closure ledger, and merge.

Pre-review validation: the authoritative `create-pr` profile passes every
blocking lane in `1110.82s`, including the new read-only check/doctor suite,
Python interop `17/17`, create-PR E2E `131/131` with signature
`7c39b8c1dd4fec7c`, and hardening `6/6`. The warm wall-time notice is a
non-blocking advisory; every per-step blocking budget passes.

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

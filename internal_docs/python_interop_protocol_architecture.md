# Python Interop Protocol Architecture

## Status And Scope

This document defines the declaration-first Python interop contract. Typed
coroutines, synchronous and asynchronous contexts, every callback dispatch
mode, and typed affine buffer declarations are implemented. Arrow and DLPack
rows remain reserved and must implement the same contract without publishing
reduced substitutes.

The common rules in
[`python_interop_declaration_architecture.md`](./python_interop_declaration_architecture.md)
remain authoritative: the Sifr signature owns types, every boundary is
fallible, Python identity uses one sealed non-send handle, package bridges are
hermetic, and the root application owns environment and trust decisions.

## Design Principles

- Protocol behavior is explicit wherever ownership cannot be derived from the
  Sifr signature.
- Sync Python calls never hide offload. Async Python calls never block a Sifr
  executor thread or create a per-call event loop.
- Cancellation, cleanup, release, and callback shutdown are part of the type and
  ownership contract, not best-effort epilogues.
- Zero-copy declarations never copy. Copying is a different API with a copied
  Sifr return type.
- Python values remain non-send. A threadsafe callback is a generated transfer
  boundary, not permission to send arbitrary Python objects.
- There is one implementation path for each behavior. Unsupported declarations
  are rejected; they do not lower to raw `py.Object` or an alternate path.

## Async Python Calls

### Declaration Surface

Python coroutine targets use `@python.coroutine(path)` on `async def` declarations:

```sifr
@python.opaque(type=httpx.AsyncClient, cleanup=async_close)
class AsyncClient:
    @python.coroutine(Self.get)
    async def get(
        self,
        url: str,
        *,
        timeout: float | None = python.omit,
    ) -> Result[Response, PythonError]: ...

    @python.coroutine(Self.aclose)
    async def aclose(own self) -> Result[None, PythonError]: ...
```

`@python.coroutine` is valid only on `async def`. Ordinary `@python`, attribute,
item, buffer, Arrow, and DLPack declarations are synchronous `blocking_io`
operations. Async context and callback declarations have their own explicit
forms below.

### Event-Loop Ownership

A generated application containing any async Python declaration owns exactly one
CPython asyncio loop on one dedicated OS thread. The loop starts after CPython
initialization and bridge-loader installation, before user `main`, and stops
after all registered Python async work and async cleanup have completed. Sifr
does not use an ambient loop, call `asyncio.run` per operation, nest event loops,
or run Python coroutines on a Sifr Tokio worker.

The loop thread is runtime infrastructure, not hidden scheduling policy:
`@python.coroutine` explicitly selects it. Sync declarations remain sync and
blocking; the compiler never silently changes one declaration kind into the
other.

Generated async wrappers submit one operation to the owned loop. On that thread
the wrapper converts inputs, resolves the target, calls it, verifies that the
result is awaitable, awaits it, converts the result to the declared Sifr type,
and completes the Sifr future. Python exceptions and conversion failures become
`PythonError` with the declaration span and target path.

Every async Python interop declaration carries the async interop effect,
including `@python.coroutine`, `@python.context.aenter`/`.aexit`, and
asyncio-dispatched callback handlers. Its ellipsis body uses the interop
`Bodyless` stub path, matching `@rust` stubs, so normal body lowering and the
`NoSuspend` fake-async gate do not run. This does not add a new
`AsyncSuspensionSummary` variant. Synchronous declarations carry `blocking_io`
through the same interop-effect channel rather than relying on a source-name
workload annotation.

Opaque arguments remain non-send in Sifr. Generated wrappers transfer only
compiler-private object-store ids to the loop thread, then resolve them under
the GIL; they never move a raw Python pointer or grant user-visible sendability.
A borrow of an opaque receiver is frozen across the await, so the value cannot
be moved or closed concurrently. Consuming async close transfers exclusive
ownership to the wrapper before submission.

### Cancellation And Shutdown

Cancellation is bidirectional and structured:

1. Cancelling or dropping the awaiting Sifr operation requests cancellation of
   the exact asyncio task with `loop.call_soon_threadsafe(task.cancel)`.
2. The generated Sifr future does not report cancellation complete until the
   Python task reaches a terminal state and its `finally` blocks and async
   context exits have run.
3. Python `CancelledError` maps to the active Sifr cancellation cause, not to an
   ordinary `PythonError`.
4. If Python suppresses cancellation and returns or raises another exception,
   that terminal result wins and is mapped normally.
5. Runtime shutdown stops accepting new submissions, cancels all registered
   tasks, awaits their terminal states and async resource cleanup, then stops
   and joins the loop thread. It never abandons a live Python task.

An async declaration cannot return a borrowed Python value. Converted Sifr
values are owned. Declared opaque results enter the same sealed foreign-object identity model as
sync results and remain non-send at the Sifr level; the runtime's internal
loop-to-task handoff does not grant user-visible sendability.

The raw coroutine API submits to this same loop and follows the same
cancellation registry. The former per-call `asyncio.run` implementation has
been removed; the runtime retains one coroutine execution path.

## Opaque Lifecycle And Context Managers

### Cleanup Policies

`@python.opaque` has one complete cleanup policy:

- `cleanup=drop`: ordinary reference release is sufficient;
- `cleanup=close`: the class must declare exactly one consuming synchronous
  semantic close method;
- `cleanup=async_close`: the class must declare exactly one consuming async
  close method;
- `cleanup=context`: the class must declare a consuming synchronous context
  exit and values must be consumed by `with`;
- `cleanup=async_context`: the class must declare a consuming asynchronous
  context exit and values must be consumed by `async with`.

The consuming method marks the Sifr handle closing before invoking Python. On
success it becomes closed. On Python failure it becomes poisoned and may only be
queried for diagnostics or consumed by runtime cleanup; it cannot be re-entered.
Automatic reference release always follows semantic cleanup and never replaces
it. Ownership analysis rejects paths that can abandon a value requiring
semantic close.

This requires a net-new linear must-use obligation layered over Sifr's affine
move tracking. Lowering records every value with `cleanup=close | async_close |
context | async_context` in a liveness side table, transfers the obligation
through moves, returns, and owning aggregates, joins it across control flow, and
checks every scope/function exit. A consuming close, completed context exit, or
ownership transfer discharges the local obligation; ordinary drop does not.
Cleanup failure still discharges it after poisoning because re-entry is unsafe.
The mechanism follows the existing JoinSet exit-liveness precedent but is a
general foreign-resource obligation rather than a hard-coded type check.

### Exit Cause And Decision Types

Python context lowering uses compiler-known `python.ExitCause` and
`python.ExitDecision` types. It does not reuse or redefine the native
`AsyncExitCause` protocol type.

```sifr
enum python.ExitCauseKind:
    Normal
    Return
    Break
    Continue
    OrdinaryError
    PythonException
    Timeout
    Cancellation
    RuntimeFault

enum python.ExitDecision:
    Propagate
    Suppress
```

`python.ExitCause` is a sealed value containing the kind plus redacted public
metadata: the Sifr error type name, its already-public display message where
one exists, and timeout/cancellation classification. It never contains an
arbitrary Sifr error payload, captured local, or native backtrace.

A `PythonException` cause also carries a compiler-private replay capability for
the original live `(type, value, traceback)` triple. `PythonError` stores that
capability beside its public structured fields. The capability is an
unforgeable object-store identity, may be resolved only by generated context
exit under the GIL, and is not exposed as a general Python handle. `PythonError`
remains sendable because it carries only this integer store identity and
structured Sifr data, never a `PyObject*`; final release uses the pending-release
queue. Every Python-originating boundary error retains the triple for full
context-manager fidelity, so traceback frames may remain live until that
`PythonError` is dropped. This cost is deliberate and visible rather than
selected inconsistently by call site. The capability stays live through nested
exits while the error propagates and releases when the cause is suppressed,
replaced, or leaves the context chain.

If no context consumes the replay capability, dropping the `PythonError`
releases its retained triple through the same detach-before-decref queue as
ordinary sealed references. The capability can be replayed by multiple nested
exits while borrowed but released exactly once by its final owner.

For non-Python causes, generated exit adapters create `SifrBoundaryError`, a
Python exception with `cause_kind`, `sifr_type`, and redacted `message` fields.
It has no Sifr payload or fabricated Python traceback beyond the adapter frame.

Exit decisions are normative:

| Active cause | Python exit receives | Truthy result |
| --- | --- | --- |
| normal, return, break, continue | `(None, None, None)` | no effect on the control-flow action |
| originating Python exception | original live exception triple | suppresses that Python error and continues after the block |
| ordinary Sifr error | `SifrBoundaryError` | recorded as ignored cleanup evidence; original error propagates |
| timeout | `SifrBoundaryError` | recorded as ignored cleanup evidence; timeout propagates |
| cancellation | `SifrBoundaryError` | recorded as ignored cleanup evidence; cancellation resumes |
| runtime fault | `SifrBoundaryError` | recorded as ignored cleanup evidence; fault remains primary |

If exit itself fails, that cleanup failure follows Sifr's normal primary and
secondary error rules for the active cause; a foreign truthy return can never
swallow Sifr cancellation or an ordinary Sifr error.

### Synchronous Context Managers

Python context managers declare their protocol methods explicitly:

```sifr
@python.opaque(type=database.Transaction, cleanup=context)
class Transaction:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[Transaction, PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(
        own self,
        cause: python.ExitCause,
    ) -> Result[python.ExitDecision, PythonError]: ...
```

The generated enter wrapper calls `__enter__` and converts its result. The exit
wrapper maps normal exit to `(None, None, None)`, an originating Python error to
its preserved exception triple, and a non-Python Sifr error or cancellation to
a generated `SifrBoundaryError` carrying redacted structured cause metadata.
Python's truthy `__exit__` result becomes `python.ExitDecision.Suppress`; false
becomes `python.ExitDecision.Propagate`, subject to the cause table above.

The context manager remains a hidden owner for the full block. When `__enter__`
returns opaque Python identity, the `as` binding is a context-scoped borrow: it
may be used normally inside the block but cannot escape, move, or close the
manager independently. If Python returns distinct opaque identity, the hidden
context owner retains that object and releases it after exit. Non-opaque entered
values are converted to ordinary owned Sifr values. This handles Python's common
`__enter__ -> self` shape without creating two owning handles for one resource.

Declaration checking permits an opaque entered result only when it is the
manager identity itself or its declared class has `cleanup=drop`. A distinct
opaque result requiring `close`, `async_close`, `context`, or `async_context`
cannot be represented as the context-scoped borrow and is rejected with
`SIFR-PYCTX-*` rather than abandoned by generated cleanup.

`with` selects dedicated Python-context lowering only when the manager type is a
`@python.opaque(cleanup=context)` declaration. Native Sifr context managers keep
their existing argless, drop-style `__exit__` protocol and gain no foreign
suppression behavior. The Python manager is consumed by exit exactly once on
normal completion, error propagation, return, break, or continue.
Direct source calls to declared `__exit__` or `__aexit__` are rejected because
`python.ExitCause` is compiler-constructed and exists only in context lowering.

Implementation status: synchronous Python context declarations are active. The
compiler uses dedicated HIR and closure outcomes for normal fallthrough,
return, break, continue, and typed errors; runtime exit adapters preserve exact
Python exception replay and record unsuppressible Sifr-cause cleanup evidence.
The executable evidence matrix is
`verification/areas/python_interop/fixtures/sqlite_context/sync_context_evidence.json`,
with the registered SQLite transaction example providing the live binary gate.

### Asynchronous Context Managers

Async context managers use the owned Python loop:

```sifr
@python.context.aenter(Self.__aenter__)
async def __aenter__(self) -> Result[Session, PythonError]: ...

@python.context.aexit(Self.__aexit__)
async def __aexit__(
    own self,
    cause: python.ExitCause,
) -> Result[python.ExitDecision, PythonError]: ...
```

Both decorators require `async def`. Dedicated Python async-context lowering
constructs `python.ExitCause` directly from the body's concrete control-flow
outcome before native async-context cause erasure. It inspects a terminal
`Err(PythonError)` and borrows its replay capability for `PythonException`, uses
runtime timeout/cancellation/fault signals for those causes, and classifies
other error values as `OrdinaryError`. Native `AsyncExitCause` remains solely
the native `async with` protocol type and is not the classification source for
Python contexts. Cancellation of the body still runs `__aexit__`; cancellation
of `__aexit__` is masked until cleanup reaches a terminal state, after which the
original cancellation always resumes with any exit failure attached as
secondary evidence. Python truthiness never suppresses it.

Implementation status: asynchronous Python context declarations are active.
The compiler selects dedicated Python async-context HIR, runs enter and exit on
the application-owned asyncio loop, preserves original Python exception replay,
and uses parent/child cancellation claims plus a biased body race so cleanup is
terminal before cancellation resumes. Mixed synchronous/asynchronous contexts
preserve lexical LIFO cleanup. The executable evidence ledger is
`verification/areas/python_interop/fixtures/async_context/async_context_evidence.json`;
the blocking compiled proof uses real `aiosqlite` over in-memory SQLite.

## Callback Declarations

### Surface And Ownership

Callback behavior is attached to the parameter whose Python callable is
generated:

```sifr
@python(events.map_rows)
@python.callback(handler, lifetime=call, dispatch=current)
def map_rows(
    rows: list[Row],
    handler: Callable[[Row], Result[Row, HandlerError]],
) -> Result[list[Row], PythonError | HandlerError]: ...
```

Retained callbacks must name the owner that retains their trampoline:

```sifr
@python(events.subscribe)
@python.callback(
    handler,
    lifetime=result,
    dispatch=foreign,
    concurrency=parallel,
)
def subscribe(
    handler: Callable[[Event], Result[Ack, HandlerError]],
) -> Result[Subscription, PythonError | HandlerError]: ...
```

`lifetime=call` keeps the trampoline alive only for the dynamic Python call and
proves it cannot escape according to the package binding contract.
`lifetime=result` transfers the trampoline into the returned opaque owner.
`lifetime=Self` attaches it to the receiver. Retained callbacks require an
opaque owner whose cleanup policy provides deterministic synchronous or
asynchronous shutdown; the compiler rejects any declaration without such an
owner.

Callback parameters use the ordinary declared `Callable` or `AsyncCallable`
signature. Arguments and results use the same checked conversion contract as
function declarations. A Sifr handler error becomes a generated
`SifrCallbackError` Python exception during the callback. Every call-scoped
trampoline records the first handler failure by callback entry sequence across
current, serial, or parallel dispatch. After the Python target returns, the
wrapper returns that `HandlerError` even if Python caught `SifrCallbackError`;
if the target itself raised, `PythonError` is primary and the handler failure is
secondary evidence. A retained callback records its first handler failure on
the owner; later owner operations and semantic shutdown surface that failure
through their declared error channels, and shutdown diagnostics report it if no
operation observes it.

Sifr union error channels are the ordinary normalized union types implemented by
the type system. A declaration such as `Result[T, PythonError | HandlerError]`
is checked and propagated exactly like any other union error channel; callback
lowering does not synthesize an untyped catch-all error.

### Dispatch Modes

- `dispatch=current` invokes a synchronous handler on the thread currently
  executing the Python call. It permits non-send captures and is valid only for
  `lifetime=call`.
- `dispatch=foreign` exposes a synchronous `Send + Sync` trampoline callable
  from arbitrary Python-created threads. Handler captures and converted
  arguments/results must be sendable and thread-safe; Python opaque values are
  forbidden. `concurrency=serial` or `concurrency=parallel` is required.
- `dispatch=asyncio` exposes a Python async callable backed by an async Sifr
  handler. Invocation and result completion are bridged between the owned
  Python loop and the Sifr runtime without blocking either executor. It requires
  `concurrency=serial | parallel`, just like foreign dispatch.

Serial foreign dispatch assigns an entry sequence number, acquires the generated
handler-state lock, and executes on the invoking thread in that acceptance
order. Reentrant invocation is detected before locking and rejected with
`SifrCallbackReentrancyError` rather than deadlocking. Parallel dispatch permits
concurrent invocations and therefore requires immutable `Sync` captures.
Each retained foreign trampoline receives a distinct owner-local runtime
identity when it is created. Declaration parameter positions are not runtime
callback identities: the same receiver method may register more than one
trampoline, and those registrations must never be mistaken for recursive entry
into one callback. A call-scoped foreign trampoline drains synchronously for a
synchronous declaration, while an async declaration awaits the drain so an
accepted foreign invocation cannot block the current-thread executor that must
complete its handler.
Asyncio dispatch maps cancellation in both directions using the same
terminal-state rule as `@python.coroutine`. Serial asyncio dispatch is ordered per
owner; a recursive invocation from the active handler is rejected before it can
await itself. Parallel asyncio dispatch permits interleaved handler futures and
requires sendable, immutable shared captures.

### Shutdown State Machine

A retained callback owner has `open`, `closing`, and `closed` states. Semantic
owner close first unregisters the callback from the Python target, then marks
the trampoline closing, rejects new entries, waits for accepted invocations to
finish, releases captured state, and finally releases the Python callable.
Concurrent close is idempotently joined internally but the affine Sifr owner can
be consumed only once. Callback invocation after closing raises a stable Python
exception and never enters released Sifr state.

Runtime shutdown performs the same sequence for every registered retained
callback before stopping the Python loop or reporting outstanding resources.
Semantic owner close from within one of that owner's accepted callback
invocations is rejected statically where visible and by a runtime reentrancy
guard otherwise, so shutdown never waits for the invocation that initiated it.

Context entry is part of the same ownership boundary. If synchronous or
asynchronous entry fails after a receiver callback owner exists, the wrapper
closes and drains that owner, releases its captures, and attaches retained
handler-failure evidence to the entry error. It marks unregister authority
complete without calling `__exit__` or `__aexit__`, because a failed entry never
established a context. The async error path releases its exact native
cancellation claim unconditionally and decides the returned error from the
terminal `CancellationResume` state. A cancellation request arriving after an
earlier notification observation therefore resumes the parent instead of being
lost behind the Python entry error.

An asyncio receiver callback remains provisional until the Python registration
operation succeeds and transfers it into the receiver owner. On failure or
cancellation, generated code closes that callback's admission gate, waits for
in-progress setup, cancels and joins only its accepted entries, and releases
the target and Python callable. This terminal rollback prevents the emergency
drop path from leaking captured state when Python starts a callback before
rejecting registration.

## Buffer Protocol

Buffer declarations use a typed affine return:

```sifr
@python.buffer(Self, access=read, layout=c_contiguous)
def bytes_view(self) -> Result[python.Buffer[uint8], PythonError]: ...
```

For buffer, Arrow, and DLPack acquisition decorators, `Self` acquires directly
from the opaque receiver. An import-root or `bridge` target is invoked with the
declaration's ordinary non-protocol parameters first; the generated wrapper then
acquires from that returned producer object and owns the temporary producer
until acquisition or failure cleanup completes. No other target interpretation
exists.

The element type comes from `python.Buffer[T]`; `access=read | write` and
`layout=any | c_contiguous | f_contiguous` are protocol facts not expressible in
the ordinary return type. Acquisition validates format, item size, dimensions,
shape, strides, suboffsets, requested writability, and layout before returning.
The buffer retains its exporter owner.

`python.Buffer[T]` is affine and non-send. Drop performs exact-once
`PyBuffer_Release`; explicit `release(own buffer)` provides deterministic early
release. The active surface exposes bounded zero-copy element access and an
explicit checked `copy_slice`; it does not expose a borrowed slice that could
outlive the buffer. Writable buffers require an exclusive Sifr borrow. A
declaration returning `bytes` or a typed collection is a checked copy, not a
buffer declaration.

Compiler capabilities follow the emitted Rust traits rather than assuming that
every non-affine type is reusable. Sequence equality and membership require a
recursive `PartialEq` capability. Set membership and equality require elements
with recursive `Eq + Hash`; dictionary membership and equality require keys
with recursive `Eq + Hash` and equality-capable values. Generated classes,
newtypes, and non-optional union enums derive only the `Debug`, `Clone`,
`PartialEq`, `Eq`, and `Hash` traits proved by their complete shapes, including
the traits of an embedded inheritance parent, and union formatting is emitted
only when every member supports `Display` or `Debug`. Equality and list, set, or
dictionary membership inject a concrete union member into the generated union
representation before invoking Rust equality. Specialized generic classes are
emitted without unrelated declaration-wide bounds: conditional trait
implementations and individual methods carry only their required bounds, and a
concrete specialization is rejected at any consumer whose emitted Rust bound it
cannot satisfy. Generic classes are not admitted as Rust hash keys until their
emitted representation proves `Eq + Hash`. Error-class fields must prove
`Debug` before code generation because `std::error::Error` requires it.
`Any`, dynamic trait objects, callable-bearing classes, affine resources, and
other unsupported Rust representations are rejected before an operation that
would require a missing trait. Source `is` and `is not` are limited to identity
checks against `None`; they are not rewritten into structural equality for
arbitrary resources.

Tuple unpacking clones a borrowed source only when its complete type is
recursively cloneable. An owned tuple source is consumed and destructured by
move. Star unpacking preserves its list source and therefore requires cloneable
elements; affine, `Any`, callable-bearing, and other non-clone element shapes
are rejected. Chained assignment likewise rejects move-only values whose Rust
representation cannot be cloned; it never emits multiple moves from one source.
Async-generator validation includes free-variable captures.
Nested async generators are rejected until their dedicated lazy materialization
path exists, with affine captures receiving the buffer-specific zero-copy
diagnostic.
Reusable lambdas and nested functions cannot capture an affine protocol
resource: a callable could otherwise be invoked more than once with a single
owner. Walrus expressions likewise cannot create an affine alias whose source
and expression result would represent two live owners. Both cases are rejected
during lowering before HIR or Rust block scoping can make ownership incoherent.

Runtime admission compares the physical byte ranges of logical buffer items
across every live view. C- and F-contiguous views use one compressed range, so
large ordinary arrays require constant admission memory. Non-contiguous direct
and indirect views resolve each logical item through `PyBuffer_GetPointer`, then
sort and merge the resulting ranges. This admits physically disjoint slices,
strides, and indirect exporters while rejecting any overlapping pair for which
at least one view is writable.

The public resource exposes read-only `length`, `item_size`, `dimensions`,
`shape`, `strides`, `suboffsets`, `format`, `readonly`, `c_contiguous`, and
`f_contiguous` accessors. Typed element/slice access checks the declared layout
and bounds; multi-dimensional metadata remains runtime data rather than pretending
shape values are static type parameters.

Complete activation evidence is machine-owned by
`verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_evidence.json`.
It locks positive, negative, cleanup, cancellation-disposition, live-source,
and delivery-profile ownership. Its owners resolve to checked-in source or
named tests, and strict mutation tests reject schema, row, owner, cancellation,
live-case, and profile drift. The compiled suite covers import-root, `Self`,
package-bridge, affine-aggregate, and real NumPy ndarray producers. The bridge
fixtures expose data identity and exact `bf_releasebuffer` counts for explicit
and aggregate-drop cleanup; runtime tests independently cover pointer identity,
validation failure, admission conflict, and store-failure rollback. The
runnable typed-buffer example exposes the same five deterministic markers.

## Arrow C Data Interface

Arrow declarations derive capsule kind from their affine return type:

```sifr
@python.arrow(Self)
def arrow_stream(self) -> Result[python.ArrowStream, PythonError]: ...
```

Requested schemas are explicit. `schema=omitted` calls the protocol without a
requested schema. `schema=parameter(name)` requires a same-declaration
keyword-only borrowed `python.ArrowSchema` parameter and passes that capsule to
`__arrow_c_array__` or `__arrow_c_stream__`; certification binds the exact
schema/producer contract. No implicit schema request exists.

The permitted return types are `python.ArrowArray` (owning the required schema
and array capsule pair), `python.ArrowSchema`, `python.ArrowStream`,
`python.ArrowDeviceArray` (owning schema plus device-array capsules), and
`python.ArrowDeviceStream`. Acquisition validates exact standard capsule names,
non-null payloads, required release callbacks, device metadata, producer
identity, and paired schema/data consistency.

The Arrow PyCapsule protocol does not itself prove that a producer avoided
allocation or representation conversion. Therefore an `@python.arrow`
declaration is accepted only when the exact producer target and distribution
fingerprint has executable zero-copy certification recorded in the binding
evidence. A requested schema is accepted only when certification proves the
request is representation-preserving. Without this evidence the author may
declare an ordinary copied value or dynamic object, but cannot claim an Arrow
zero-copy resource. There is no policy that silently accepts uncertain copying.

Certification is package-authored, not a compiler allowlist. The package checks
in a fixture plus `src/python_certifications/<name>.json`, keyed by the fully
qualified Sifr declaration. The artifact records the Python target, protocol
kind, distribution name/version and locked artifact hash, SOABI, schema mode and
schema-contract digest, fixture/source digest, compiler certification version, within-run
producer/consumer pointer-identity assertion results, and exact release counts.
It never records absolute addresses, which are unstable across processes. The
artifact participates in the binding contract digest and package archive.

The authoring flow is explicit and reproducible:

```bash
sifr python certify arrow \
  my_package.dataframe.DataFrame.arrow_stream \
  --fixture verification/python/arrow_stream.sifr
sifr python certify --check
```

The first command executes the compiled fixture in the selected locked
environment and writes or updates the adjacent certification artifact. The
read-only `--check` reruns the fixture, verifies the complete fingerprint, and
fails on environment/source drift, a pointer-identity assertion failure, or a
release-count mismatch. A consumer build
verifies the locked fingerprint; project and package certification lanes rerun
the executable evidence. Any package author can certify a new producer without
changing Sifr itself.

Arrow resources are affine, non-send, retain the producer, and invoke the exact
release callback once. Passing `own python.Arrow*` to another Python declaration
transfers the capsule. The generated wrapper treats it as moved regardless of
call success and inspects the C structure's release callback: a consumer that
moved the data must have set it to null, while an unconsumed structure remains
the wrapper's cleanup responsibility. A copied dataframe or record is declared
as an ordinary copied return and never masquerades as Arrow transfer.

## DLPack

DLPack declarations state device and stream synchronization policy:

```sifr
@python.dlpack.stream(bridge.torch.cuda_stream, device=cuda)
def cuda_stream(device_id: int) -> Result[python.DlpackStream, PythonError]: ...


@python.dlpack(
    Self,
    device=cuda,
    stream=parameter(consumer_stream),
)
def tensor(
    self,
    *,
    consumer_stream: python.DlpackStream,
) -> Result[python.DlpackTensor[float32], PythonError]: ...
```

The element type comes from `python.DlpackTensor[T]`. `device=cpu | cuda | any`
constrains the producer's validated source device; declarations never request a
cross-device move. `stream=none` is valid only for CPU. A non-CPU declaration
uses `stream=parameter(parameter_name)`, and that named keyword-only parameter
must be `python.DlpackStream` for the same device family and id.

`device=any` also requires `stream=parameter(parameter_name)`. Because its
producer device is not statically fixed, lowering validates the stream's device
family and id at runtime against `__dlpack_device__` before calling
`__dlpack__`. A mismatch is a `PythonError`; it never triggers a different
stream, device move, or retry.

`parameter(name)` is a dedicated decorator-argument production, not a literal atom.
`name` resolves only in the same declaration and must name a keyword-only
`python.DlpackStream` parameter. Lowering verifies its device family/id against
the acquired producer and reports mismatches through `SIFR-PYZC-*` or
`SIFR-PYCALL-*`.

`@python.dlpack.stream` converts a consumer-library stream object or normalized
package-bridge result into an affine, non-send `python.DlpackStream` carrying
the device family, device id, and protocol stream token. The generated producer
call passes that exact token to `__dlpack__(stream=...)`. This makes the
consumer's synchronization context explicit rather than guessing a current
stream.

Generated acquisition passes `copy=False` and pins `max_version=(1, 0)`; a newer
supported DLPack contract requires an architecture update, not opportunistic
adoption. It validates legacy or versioned capsule names, copied flags, dtype
code/bits/lanes, device,
dimensions, shape, strides, byte offset, deleter state, and stream contract.

Legacy capsule-name support means a producer accepting the versioned call
signature may legally emit a v0 `dltensor` capsule. A producer whose old
signature rejects `max_version` or `copy` is not directly bindable; authors must
publish an explicit package bridge with the complete signature. Generated code
never catches `TypeError` and retries without those arguments.

`python.DlpackTensor[T]` is affine and non-send. Consumption is one-shot. When
acquisition transfers the managed tensor into Sifr, the source capsule is
immediately renamed from `dltensor` to `used_dltensor` (or between the versioned
equivalents), disabling its Python destructor before Sifr assumes deleter
responsibility. Passing the resulting affine value as an owned argument requires
the new consumer capsule to be renamed from `dltensor` to `used_dltensor` (or the
corresponding versioned names) and permanently moves the Sifr value. If a Python
call fails after consuming the capsule, the move
remains committed. If it fails before consumption, generated cleanup first
renames the unconsumed consumer capsule to the used sentinel so its destructor
becomes a no-op, then invokes a non-null producer deleter exactly once. A
spec-valid null deleter is recorded as no-op release rather than invented
ownership. Drop of an unconsumed tensor follows the same rule.

No DLPack declaration copies or silently changes device. Explicit copied tensor
conversion is a separate ordinary declaration or package bridge with a copied
Sifr return type.

## Raw API Relationship

The raw `sifr.python` API exposes the same sealed object and affine protocol
resources used by declarations. It gains typed generic conversion, method-style
call/attribute/item helpers, typed kwargs construction, and ordinary automatic
drop. It does not expose handle/token fields or require reverse-order manual
object close chains.

A lexical scope helper is unnecessary because Sifr ownership already releases
ordinary objects and affine protocol resources on every exit path. Semantic
close, async close, context exit, callback shutdown, and one-shot transfer
remain distinct compiler-checked operations and are never collapsed into a
generic scope cleanup stack.

## Protocol Diagnostics And Verification

- `SIFR-PYASYNC-*` covers loop ownership, awaitable shape, cancellation, and
  async shutdown declarations.
- `SIFR-PYCTX-*` covers enter/exit shape, exit-cause mapping, suppression, and
  cleanup ordering.
- `SIFR-PYCB-*` covers callback lifetime, owner, dispatch, concurrency,
  capture, error-channel, and shutdown violations.
- `SIFR-PYZC-*` covers buffer, Arrow, and DLPack format, ownership, consumption,
  synchronization, and no-copy violations.

Every protocol requires executable positive and negative evidence for normal
completion, Python failure, conversion failure, Sifr error, cancellation,
early return, double use, use after close/consume, and shutdown with work in
flight. Async tests use the generated application-owned loop. Callback tests
cover current-thread, foreign serial, foreign parallel, reentrancy rejection,
asyncio dispatch, retained-owner close, and callback-after-close. Zero-copy
tests prove pointer identity where observable and assert exact-once release or
deleter counts; a test that only validates values cannot certify zero-copy.

Live certification uses compiled Sifr binaries for dataframe/Arrow, tensor
DLPack, broker callbacks, Redis, Postgres, Kafka, SQS, and SNS-to-SQS cases.
Python-client-only evidence and package inventory never certify a declaration
protocol.

## Primary Protocol References

- [Python asyncio thread-safety and cross-thread submission](https://docs.python.org/3/library/asyncio-dev.html#asyncio-multithreading)
- [Python buffer protocol](https://docs.python.org/3/c-api/buffer.html)
- [Apache Arrow PyCapsule Interface](https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html)
- [Apache Arrow C Data Interface](https://arrow.apache.org/docs/format/CDataInterface.html)
- [DLPack Python specification](https://dmlc.github.io/dlpack/latest/python_spec.html)

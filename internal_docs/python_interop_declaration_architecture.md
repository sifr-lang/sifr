# Declaration-First Python Interop Architecture

Status: production contract. Synchronous declarations, opaque lifecycle,
synchronous contexts, package-local bridges, typed coroutine declarations,
consuming async close and async contexts, typed callbacks, and typed zero-copy
buffer declarations are active. Remaining zero-copy protocol sections stay the
ordered target contract until their implementations activate. The
embedded runtime contract is also documented in
[`python_interop_architecture.md`](./python_interop_architecture.md).

## Problem

The embedded Python runtime already preserves Sifr's important safety
boundaries: one root-selected CPython environment, explicit trust, structured
`PythonError` values, non-send Python objects, explicit blocking effects, and
explicit zero-copy ownership. Its public authoring surface is nevertheless too
low-level for ordinary package use.

Today every caller performs bridge work directly: import a module, allocate
`Object` handles for arguments, build positional and keyword containers, call
attributes by string, convert results, and close every intermediate handle.
Rust interop pays equivalent adaptation cost once in a checked declaration or
package-local bridge, then exposes an ordinary typed Sifr API. Python interop
should use the same package-author/consumer split.

## Goals

- Make typed Python-backed APIs read like ordinary Sifr APIs to consumers.
- Make the Sifr declaration signature the single conversion contract.
- Preserve every existing environment, trust, error, blocking, sendability,
  callback, resource, and zero-copy invariant.
- Release ordinary owned Python references automatically on every exit path.
- Give package authors a deterministic local Python adapter boundary for APIs
  that are not directly bindable.
- Use Python type information to assist authoring without treating it as a
  runtime-enforced contract.
- Certify capabilities with executable positive and negative evidence rather
  than package inventory alone.

## Non-Goals

- Python source compatibility or an implicit `from python import ...` mode.
- A Sifr `Any` type or silent degradation from unsupported types to `py.Object`.
- Automatic installation, `uv sync`, environment mutation, or inferred trust.
- Whole-package automatic binding generation.
- Decorator-level input/output converter pipelines.
- Hidden blocking offload, ambient event-loop reuse, per-call event loops, or
  implicit conversion between synchronous and asynchronous declarations.
- Silent copying for buffers, Arrow, DLPack, arrays, dataframes, or tensors.
- Static proof that arbitrary Python implementation code matches its hints.

## Two-Level User Model

Python interop has two deliberate levels.

1. Declaration-first bindings are the normal package-authoring and consuming
   surface. Package authors expose typed Sifr functions and opaque classes.
2. `sifr.python` remains the explicit dynamic escape hatch. It is appropriate
   for exploration, highly dynamic APIs, and bridge implementation work, but it
   is not the primary application API.

Consumers should not know whether a normal Sifr function is implemented by a
direct Python target or a package-local Python bridge.

## Declaration Syntax

Python interop uses decorators on ellipsis-only Sifr declarations. Targets are
structured dotted paths, never strings.

```sifr
from sifr.python import PythonError

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...
```

The declared parameter and return types are authoritative. The Sifr signature
is the only conversion type contract. The decorator does not repeat types
through `returns=`, `copy=`, or per-argument converter fields.
This prevents decorator metadata and Sifr types from drifting apart.
Ellipsis is public Python interop declaration syntax, not a general Sifr
function body form.

A non-reserved root in `@python(pkg.a.b)` is declared by that decorator use.
Lowering adds `pkg` to the package's required Python imports; no separate root
declaration is needed. A final application build resolves and probes the target
through the root-selected environment and requires root-owned execution trust.
A library-only check without a selected application environment validates the
declaration and records a deferred target probe rather than selecting an
environment on the library's behalf.

Decorator target paths use a dedicated Python interop target namespace; they do
not resolve through ordinary Sifr imports or local bindings. The reserved roots
`bridge` and `Self` always retain their meanings even if a Python distribution
uses one of those names. Such a distribution must be reached through a
package-local bridge with a non-reserved Sifr target.

### Opaque Python Types

Opaque declarations represent Python identity without exposing Python object
layout as a Sifr class.

```sifr
@python.opaque(type=schwifty.BIC, cleanup=drop)
class Bic:
    @python.attr(Self.country_code)
    def country_code(self) -> Result[str, PythonError]: ...

    @python.attr(Self.bank_code)
    def bank_code(self) -> Result[str, PythonError]: ...


@python(schwifty.BIC)
def bic(text: str) -> Result[Bic, PythonError]: ...
```

Fallible top-level or static factory functions represent Python construction.
Ordinary Sifr class construction never gains a hidden `Result` channel.

When a function returns a declared opaque Python class, the generated wrapper
checks that the returned Python value is an instance of the class named by the
opaque declaration, including subclasses. An inspectable mismatch is a target
probe diagnostic; an uninspectable or runtime-only mismatch is a
`PythonError` conversion failure.

Python attribute access remains fallible even when it resembles a field:
descriptors and properties may execute arbitrary Python and raise exceptions.

### Decorator Grammar

The declaration grammar contains only semantics that cannot be derived
from the Sifr signature:

- `@python(path)` for a synchronous function, factory, or method target;
- `@python.coroutine(path)` for a Python awaitable target exposed as `async def`;
- `@python.opaque(type=path, cleanup=...)` for Python identity and semantic
  cleanup;
- `@python.attr(path)` for fallible attribute or property access;
- `@python.item` for fallible `__getitem__` access;
- `@python.context.enter`, `.exit`, `.aenter`, and `.aexit` for Python context
  protocols;
- `@python.callback(...)` for callback lifetime, dispatch, concurrency, and
  ownership that cannot be inferred from `Callable` types;
- `@python.buffer`, `.arrow`, and `.dlpack` for explicit affine protocol
  resources.

Attribute/item mutation has no declaration shorthand. Setter surfaces are
classified dynamic-only or adapted behind a typed package bridge; they are not
silently inferred from getter declarations.

Decorator policy values such as `cleanup=drop`, `cleanup=close`,
`cleanup=async_close`, `cleanup=context`, and `cleanup=async_context` are closed
literal atoms consumed by Python interop lowering. They are not resolved as
ordinary Sifr names.

Allowed target roots are:

- a statically declared Python import root;
- `bridge`, resolving to package-local Python bridge modules;
- `Self`, resolving against the enclosing opaque Python type.

The complete async, context, callback, buffer, Arrow, and DLPack contracts are
defined in
[`python_interop_protocol_architecture.md`](./python_interop_protocol_architecture.md).

`@python.item` is valid only on an opaque method with one key parameter after
the receiver. It maps that parameter to `receiver[key]`; the key and return
types come only from the Sifr signature.

```sifr
@python.opaque(type=collections.UserDict, cleanup=drop)
class StringMap:
    @python.item
    def get(self, key: str) -> Result[str, PythonError]: ...
```

Target probing has three outcomes: `verified`, `runtime-checked`, or rejected.
A target proven absent or of the wrong callable/attribute kind is rejected.
Instance attributes that cannot be proved from stubs, class metadata, or safe
introspection remain `runtime-checked`; the generated wrapper still performs
fallible lookup and output conversion. This status is visible in
`sifr python check` and build reports and is never described as static proof.

The Python opaque grammar intentionally omits Rust-specific `clone`, `borrow`,
and `sync` policies. Python values preserve object identity and shared Python
semantics; the main interpreter and GIL serialize Python interaction, generated
calls borrow the sealed handle, values do not gain an implicit structural clone
operation, and all opaque values are non-send. The grammar has no `send=` knob:
non-send is an invariant of Python identity, not package policy.

## Argument Passing

Declarations support Python's statically typed call shapes:

- regular parameters before `*` are passed positionally to Python, making them
  compatible with Python positional-only and positional-or-keyword targets;
- Sifr keyword-only parameters after `*` are passed as Python kwargs using the
  declared parameter name;
- Sifr default values are evaluated by normal Sifr call semantics and passed
  explicitly to Python;
- the compiler-known declaration default `python.omit` means that an omitted
  Sifr argument is not sent to Python, while explicitly supplied `None` remains
  Python `None`;
- typed `*args: T` expands a homogeneous Sifr sequence into positional Python
  arguments;
- typed `**kwargs: T` expands a `dict[str, T]` into named Python arguments after
  rejecting duplicate names;
- a closed record may be expanded as kwargs only through explicit `**record`
  syntax; this form requires an inspectable target so every field name can be
  checked, and is rejected with `SIFR-PYCALL-*` otherwise.

`**record` is a new call-site grammar production added by this work, not
ordinary dictionary expansion inferred by a wrapper. Its HIR retains the closed
record type, source span, and statically known field set.

`python.omit` is valid only as a default in a Python declaration and is not a
runtime value or member of the parameter type. Call lowering preserves whether
the caller supplied that argument. This keeps omission distinct from every
ordinary value without introducing an `Omittable[T]` wrapper into consumer APIs.
Lowering records a compile-time provided-argument bitset; no runtime sentinel is
passed to Python. Conditional data-driven omission uses explicit typed kwargs or
a package bridge rather than attempting to store `python.omit` in a variable.

An inspectable target probe validates arity and keyword-only names. When a
target is not introspectable, the declaration remains runtime-checked and
Python argument errors return `PythonError`. Heterogeneous or data-dependent
call shapes use a package-local bridge that publishes a stable typed boundary.

## Conversion Contract

Generated wrappers lower declarations to the existing checked Python runtime
operations. Every conversion can fail and every failure returns
`PythonError`; Python exceptions never unwind through Sifr.

Every ordinary, coroutine, callback, context, and protocol declaration error
channel must contain the exact runtime `PythonError` shape: the unique string
fields `message`, `kind`, `exception_type`, `traceback`, and `context`, with no
other fields. This structural contract is checked before code generation so
frontend checking and generated-wrapper compilation cannot disagree.

The direct conversion surface is intentionally closed and recursively typed:

| Sifr type | Sifr to Python | Python to Sifr |
| --- | --- | --- |
| `None`, `bool`, `int`, fixed-width integers, `float`, `str`, `bytes` | Checked scalar construction. | Checked scalar extraction; fixed-width integers reject overflow. |
| `Option[T]` | Python `None` or recursively constructed `T`. | Python `None` or recursively extracted `T`. |
| `list[T]`, `tuple[...]`, `dict[str, T]` | Owned Python container construction. | Checked owned copy of every element or entry. |
| closed record | Plain Python dict keyed by declared field name. | For each required field, attribute lookup first, then string-key item lookup; extra Python fields are ignored. |
| declared opaque Python class | Borrowed handle passed to Python. | Owned sealed handle after an `isinstance` check. |
| `Callable` / `AsyncCallable` | Generated callable only under an explicit callback declaration. | Not inferred from an arbitrary Python callable. |
| `python.Buffer[T]`, Arrow resource types, `python.DlpackTensor[T]` | Affine protocol transfer under the protocol declaration. | Affine protocol acquisition and validation. |
| `py.Object` | Explicit dynamic handle. | Explicit dynamic handle with no additional type claim. |

The raw-object row applies only to the compiler-owned declaration identity
originating at `_sifr.python.Object` and publicly re-exported as
`sifr.python.Object`. Basename equality is not identity: a user record named
`Object` follows the closed-record conversion rules and cannot enter sealed
handle conversion or Python-identity ownership analysis. Generated Rust writes
the compiler-owned handle as the fully qualified
`sifr_runtime::interop::Handle<sifr_runtime::python::ForeignObject>` type and
emits no source-spellable alias into the flat user namespace.

Unsupported unions, unconstrained generics, iterators, generators, arbitrary
mapping keys, callables without a callback contract, and Python `Any` are
rejected at declaration checking. A package-local bridge is the explicit typed
adapter for a Python surface that cannot itself form a stable Sifr contract.

Typed container or record returns imply checked copying. Zero-copy behavior is
never inferred from an ordinary return type and requires the dedicated buffer,
Arrow, or DLPack declaration and affine resource type.

This record rule deliberately preserves the current `py_from_record` and
`py_copy_record_fields` behavior. Package authors use an explicit local bridge
when a target requires a dataclass, Pydantic model, named tuple, or another
object-shaped input rather than a plain dict.

## Ownership And Cleanup

Ordinary Python reference release is a runtime/compiler responsibility, not a
user-facing resource operation.

The declaration layer uses a sealed compiler-owned Python handle. Its generated
Rust representation contains a private reference-counted `ForeignObject`
identity; Sifr source cannot inspect or construct the payload. Package
`@python.opaque` classes and the raw `sifr.python.Object` type lower to this one
runtime representation with different declared surfaces. The raw API has no
structural `_handle`/`_token` fields and no second public token representation.

Dropping the final identity releases its owned Python reference without an
object-store lookup or lock. If the current thread is attached to CPython, the
reference is released immediately. Otherwise it is transferred without decref
into a runtime-owned pending-release queue. Every runtime attach drains that
queue while holding the GIL, and generated program epilogue performs a final
drain before resource diagnostics. Protocol stores remove affine entries and
release their locks before any buffer release, capsule destructor, DLPack
deleter, callback shutdown, or other Python code can run. Queue insertion, lock
recovery, and drain paths must return diagnostics or conservatively retain a
reference; they must never panic in user-triggerable paths.

Normal shutdown does not call `Py_FinalizeEx`, so pending references are never
decref'd after interpreter teardown. Abrupt process termination may bypass the
epilogue but cannot expose a Sifr panic or use-after-free. Reentrant drops from
callbacks follow the same detach-before-decref rule. Cleanup runs on normal
return, error propagation, and early exit.

Cleanup policies have distinct meanings. The complete normative state machines
are in
[`python_interop_protocol_architecture.md`](./python_interop_protocol_architecture.md);
this list is the declaration summary:

- `cleanup=drop`: automatic reference release only;
- `cleanup=close`: ownership checking requires a declared consuming semantic
  `close` operation;
- `cleanup=async_close`: ownership checking requires a declared consuming
  `@python.coroutine` semantic `aclose` operation;
- `cleanup=context`: ownership checking requires a consuming synchronous
  context exit;
- `cleanup=async_context`: ownership checking requires a consuming asynchronous
  context exit;
- protocol resources such as buffers, Arrow capsules, DLPack tensors, and
  callback subscriptions retain their exact release/cancel/shutdown contract.

For example, a synchronous semantic close is an explicitly consuming method:

```sifr
@python.opaque(type=redis.Redis, cleanup=close)
class RedisClient:
    @python(Self.close)
    def close(own self) -> Result[None, PythonError]: ...
```

The raw dynamic API uses the same automatic ordinary drop and affine protocol
resources. It has no generic scope cleanup stack because semantic close, async
close, context exit, callback shutdown, and one-shot transfer are distinct
operations that must retain their individual ownership rules.

Typed coroutine, `cleanup=async_close`, synchronous context, asynchronous
context, and typed callback declarations are active. For
`cleanup=async_context`, exactly one borrowed async enter and one consuming
async exit are required, and the manager must be consumed by `async with`.
Callback declarations enforce call/result/receiver ownership, current/foreign/
asyncio dispatch, serial/parallel concurrency, capture safety, deterministic
drain, and typed failure reconciliation. Typed affine buffer declarations are
active with synchronous acquisition, explicit access/layout policy, non-send
ownership, and exact release. DLPack tensor/stream declarations are active with
explicit device/stream negotiation and one-shot transfer.

## Package-Local Python Bridges

Direct bindings are for Python callables whose public shape maps cleanly to the
supported Sifr conversion contract. Dynamic traversal, complicated overloads,
protocol negotiation, or output normalization belongs in a local bridge.

```text
src/python_bridges/
  identifiers.py
```

```python
import biip

def parse_gtin(text: str) -> dict[str, object]:
    gtin = biip.parse(text).gtin
    return {
        "value": gtin.value,
        "format": gtin.format.value,
        "check_digit": gtin.check_digit,
    }
```

```sifr
class GtinInfo:
    value: str
    format: int
    check_digit: int


@python(bridge.identifiers.parse_gtin)
def parse_gtin(text: str) -> Result[GtinInfo, PythonError]: ...
```

Bridge modules are package inputs, not ambient `sys.path` files. Sifr must:

- syntax-check and digest their source;
- include their imports in package Python requirements;
- embed them into the binary or its declared deployment artifact;
- register them under a reserved package-specific module namespace;
- prevent arbitrary `sys.path` shadowing of `bridge.*` targets;
- include bridge source and resolved distribution versions in build cache keys;
- map bridge import, call, and conversion failures to `PythonError`.

Python bridge code remains dynamic implementation code. Sifr validates the
declared boundary and runtime conversions; it does not claim to statically
prove the bridge body.

Executing a dependency's bridge module is authorized by the root application's
decision to include that Sifr package, just as package-local Rust bridge code is
package implementation. This authority covers the bridge module itself, not its
third-party Python imports or native extensions; those still require the root's
explicit `[trust].python` and `[trust].python-native` authorization.

### Bridge Module Registration

The runtime namespace is
`__sifr_bridge__.p_<resolved_package_key>.<module_path>`, where the package key
is a valid-identifier encoding of the resolved Sifr package identity. The Sifr
source root `bridge.identifiers` is rewritten to that package-specific runtime
name, so two packages may both own `bridge.identifiers` without collision.

`resolved_package_key` is the full lowercase SHA-256 digest of the
domain-separated byte sequence `sifr-python-bridge-package-v1\0` followed by
the canonical resolved `SifrPackageId`. The `p_` prefix makes the runtime
segment a valid Python identifier independently of the digest's first digit;
using the full digest preserves collision resistance without checkout-local
paths or truncation. Resolution walks only the root package and its normal
selected Sifr dependency scopes, so dev-only and otherwise unselected packages
do not contribute bridge identities or requirements.

Package resolution carries a deterministic bridge plan into driver/codegen
metadata. Each plan entry records the isolated
runtime package, inventory/source digests, runtime module names, resolved
same-package import names, and external roots. External roots contribute
`PythonRequirementKind::BridgeImport` provenance to the canonical requirement
set and remain subject to the root application's `SIFR-PYTRUST-0005`
authorization. Package-aware lowering activates a public `bridge.*` target only
when the declaration's owning resolved package has an inventoried bridge. It
replaces the source-level `bridge` segment with that package's full reserved
runtime prefix before wrapper codegen; bridge targets without package bridge
authority remain a `SIFR-PYIMP-0001` error.

Generated runtime metadata contains an embedded UTF-8 source table keyed by
the full runtime module name, including synthetic package entries and stable
virtual filenames for tracebacks. Before user `main` or any bridge import,
`sifr_runtime::python::bridge_loader` installs a first-position CPython
`MetaPathFinder`/loader that claims only the reserved namespace and reads from
that table. Existing `sys.modules` entries in the namespace are rejected as a
`SIFR-PYIMP-0003` setup failure, and the loader claims unknown reserved names
instead of falling back to filesystem or `sys.path` resolution. Runtime target
resolution restores the finder to first position after `sys.meta_path`
mutation. No temporary extraction directory is used.

Bridge source is parsed into a Python AST under the GIL before compilation.
Static `bridge.*` imports are rewritten to the owning package prefix while
relative imports retain normal package semantics. Synthetic namespace package
entries cover every parent path, and `compile` receives the stable virtual
filename `<__sifr_bridge__.p_<resolved_package_key>.<module_path>>`, which is
preserved in `co_filename` and tracebacks.

Sifr package archives include bridge sources and their manifest inventory;
generated binaries embed only the resolved graph's bridge table. Bridge source
digests, package identity, imported distribution versions, interpreter ABI,
and the binding contract participate in cache identity.

The environment probe resolves each required import root to its sorted owning
distribution names and installed versions. Those values are serialized beside
SOABI, extension suffixes, pointer width, implementation version, platform,
and machine in the canonical probe digest. The package build cache consumes
that digest, and the driver carries it into the generated-artifact key.

The interop plan separately fingerprints the versioned Python binding contract,
declaration kind and effect, cleanup and receiver-consumption policy, complete
parameter call shape, and authoritative Sifr parameter and return types. Bridge
package identity, inventory and source digests, runtime names, and classified
imports remain in the same plan fingerprint. Source bytes are accepted for
embedding only after matching their inventoried digest, so the composed package,
driver, and generated-artifact cache identities cover every embedded input.

Bridge source is parsed for ordinary static `import` and `from ... import ...`
requirements. Dynamic import calls are rejected in package bridges because they
cannot participate in hermetic requirement inventory. Root-application raw
interop remains the explicit surface for truly data-dependent imports. This is
an authoring restriction on bridge modules, not a claim that Sifr sandboxes
trusted third-party Python packages after import.

Static imports between bridge modules in the same package are rewritten under
the same `__sifr_bridge__.p_<resolved_package_key>` prefix and resolved from the
embedded source table. They do not become third-party requirements and never
fall back to ambient Python module resolution.

The package-side inventory substrate discovers only
`src/python_bridges/**/*.py`, parses every source, classifies static external
and same-package imports, rejects dynamic import calls and misplaced or invalid
sources with `SIFR-PYIMP-0002`, and emits a deterministic
`__sifr_inventory__.json` required in package archives. Source and inventory
digests are stable build inputs. Public `bridge.*` declarations are active only
through the resolved package mapping, embedded source table, and reserved
loader; there is no ambient distribution fallback for that spelling.

## Environment And Trust

The final application continues to own one uv-created CPython environment.
Sifr verifies it and never installs or synchronizes packages implicitly.

Normal uv layout is convention rather than repeated configuration. When a
Python project is selected, Sifr should discover `pyproject.toml`, `uv.lock`,
the project environment, and its interpreter using uv-compatible defaults,
while preserving explicit overrides for non-standard layouts. Environment
checking must establish lock/project consistency, not merely file readability.

Static `@python` targets and package-local bridge imports contribute required
import roots automatically. Dependency packages publish those requirements;
the root application explicitly authorizes execution and native extensions.
Trust is never inferred from source use.

Manifest authority has one model:

| Key | Declaration-first contract |
| --- | --- |
| `[python].requires-imports` | Retained for raw/dynamic library requirements that cannot be derived; declaration and bridge roots contribute equivalent generated package metadata. |
| `[trust].python` | Retained as root-owned authorization to execute required import roots. Dependency packages cannot authorize themselves. |
| `[trust].python-native` | Retained as separate root-owned native-extension authorization and never inferred from requirements. |

The former Python import allow-list has been removed atomically from parsing,
docs, manifests, diagnostics, and fixtures, so no dual authorities operate.
`SIFR-PYTRUST-0002` is retired with its old
meaning; `SIFR-PYTRUST-0005` covers a required static root not authorized by the
root application. `SIFR-PYTRUST-0003` diagnoses native trust for a root that is
not required. Root-only wildcard trust for local control remains, while
dependency wildcards remain rejected.

Derived declaration/bridge roots and manual raw/dynamic
`[python].requires-imports` entries are normalized into one canonical import-root
set with provenance. Duplicate roots collapse; manual metadata cannot override,
remove, or weaken a derived requirement. Trust and native-trust checks run once
against the canonical set, while diagnostics report every contributing source.

Raw dynamic imports remain bounded by explicit root trust, and non-literal
dynamic imports retain an explicit unsafe annotation plus runtime trust
checking.

Native extension trust remains a separate explicit decision. Authoring tools
may propose exact trust changes from the locked environment but may not apply
them silently.

## Blocking And Async

Every synchronous Python declaration has the `blocking_io` effect automatically,
including function/method calls, attribute access, item access, context methods,
and protocol acquisition. The effect is declaration metadata and is not
repeated as `blocking=True`. Async Sifr code must explicitly offload synchronous
declarations, and non-send Python values may not cross that task boundary.

`@python.coroutine`, async context methods, and asyncio-dispatched callbacks use
the single application-owned Python event-loop thread defined in
[`python_interop_protocol_architecture.md`](./python_interop_protocol_architecture.md).
They are genuinely asynchronous and do not block or offload a synchronous
declaration behind the user's back.

A sync-only opaque Python object cannot be captured into or returned from
`task.spawn_blocking`, because it is non-send. Async code that must use such an
API places the object's entire construction, use, semantic cleanup, and
conversion inside one blocking closure and returns only sendable Sifr values.
Libraries with genuine coroutine APIs should expose `@python.coroutine`
declarations instead.

## Stub-Assisted Authoring

Checked-in Sifr declarations are the binding contract. Python `.pyi` files,
`py.typed` inline annotations, and runtime introspection are authoring inputs,
not proof that runtime values satisfy the hints.

The symbol-selective command generates reviewable declaration scaffolds:

```bash
sifr python bind redis --symbols Redis
sifr python bind pandas --symbols DataFrame,concat
sifr python bind --check
```

Resolution follows an explicit source order recorded in the binding: user
overrides, selected stub-only packages, `py.typed` packages, then an explicitly
configured external stub distribution. Runtime
introspection may confirm target existence and callable shape where available,
but it cannot be required because C extension callables may expose no
signature.

Generation must stop or emit an explicit unresolved marker for `Any`, bare
Python `object`, `Callable[..., Any]`, unknown overloads, unsupported generics,
dynamic attributes, or unsupported conversion types. It must never silently
replace an unsupported type with `py.Object`.

Generated declarations record a binding-source fingerprint containing SOABI,
resolved distribution version, source-kind precedence, and hashes of consumed
stub files. `sifr python bind --check` compares that fingerprint without
rewriting declarations or the environment.

## Compiler And Build Model

Lowering records `PythonInteropDeclaration` metadata beside normal HIR:

- decorator kind and source span;
- structured target path and resolved import/bridge root;
- authoritative Sifr function or class signature;
- effect classification;
- conversion and opaque ownership requirements derived from types;
- async-loop, context, callback, and affine protocol requirements;
- environment, trust, and probe requirements;
- bridge source and binding contract digests.

Codegen returns a `PythonInteropPlan` beside generated Rust. The driver uses the
plan for environment probing, target probing, generated wrappers, dependency
requirements, diagnostics, and cache fingerprints. It must not scan emitted
Rust text or Sifr bodies to recover interop metadata.

Python probing validates what can be established without claiming Rust-like
signature proof:

- the selected interpreter and environment contract;
- import and native trust coverage;
- target module and attribute existence;
- callable versus attribute shape;
- inspectable positional-only and keyword-only compatibility;
- local bridge syntax, registration, and target existence.

Python type hints are not runtime-enforced, and some callables are not
introspectable. Runtime input/output conversion therefore remains the final
typed boundary check.

## Diagnostics

Declaration-first checking activates the reserved diagnostic families where a
failure is statically provable:

- `SIFR-PYIMP-*`: invalid target paths, missing imports, unresolved bridge or
  module targets;
- `SIFR-PYCALL-*`: invalid callable, method, attribute, item, or argument shape;
- `SIFR-PYCONV-*`: unsupported declaration types or conversion contracts;
- `SIFR-PYRES-*`: invalid opaque cleanup policy or resource ownership;
- `SIFR-PYZC-*`: invalid view/ownership declarations and hidden copying;
- `SIFR-PYCB-*`: invalid callback lifetime, capture, threading, or shutdown;
- `SIFR-PYASYNC-*`: invalid loop, awaitable, cancellation, or async cleanup
  contract;
- `SIFR-PYCTX-*`: invalid context enter/exit, suppression, or cause mapping.

Dynamic Python exceptions, values that violate declared output conversion, and
runtime resource failures remain structured `PythonError` results.

The declaration contract reserves the first diagnostic codes with stable meanings:

| Code | Meaning |
| --- | --- |
| `SIFR-PYIMP-0001` | Unresolved or invalid static Python/bridge target. |
| `SIFR-PYCALL-0001` | Unsupported or definitively incompatible callable/attribute/item shape. |
| `SIFR-PYCONV-0001` | Unsupported Sifr/Python declaration conversion type. |
| `SIFR-PYRES-0001` | Invalid opaque close or ownership policy. |
| `SIFR-PYRES-0002` | Recognized declaration-first syntax whose sole production lowering is not active yet. |
| `SIFR-PYZC-0001` | Invalid advanced-data ownership or hidden-copy declaration. |
| `SIFR-PYCB-0001` | Invalid callback lifetime, threading, or shutdown declaration. |
| `SIFR-PYASYNC-0001` | Invalid Python awaitable, cancellation, or loop-ownership declaration. |
| `SIFR-PYCTX-0001` | Invalid Python context-manager entry, exit, or suppression declaration. |

## Verification Contract

The Python interop capability matrix classifies the target contract separately
from current implementation status. Target states are:

- `declaration-supported`;
- `bridge-supported`;
- `dynamic-only`;
- `unsupported-by-design`.

Implementation status is independently `reserved` or `active`. A reserved row
is architectural intent and cannot claim passing evidence. No row is active
merely because a package appears in an inventory matrix. Active capability rows
require all executable positive, negative, cleanup, cancellation, and live
evidence marked as required by that row.
The declaration layer must cover at least:

- direct functions, factories, methods, attributes, and item access;
- positional-only, keyword-only, explicit/default omission, typed variadic, and
  typed kwargs shapes;
- scalar, container, record, opaque, and failing output conversion;
- automatic release on success, conversion failure, Python exception, and
  early return;
- semantic close/context behavior and use-after-close rejection;
- non-send enforcement and explicit blocking offload;
- import, bridge, native trust, environment, version, and stub drift;
- sync/async context managers, Python coroutines, every callback dispatch mode,
  buffers, Arrow, and DLPack ownership.

Create-PR verification includes a small real pure-Python binding and a native
extension binding. Merge verification migrates every surface of the existing
runnable ecosystem examples and asserts zero outstanding ordinary Python and
protocol resources. The existing live lane is replaced: representative async,
service, zero-copy, and callback certification invokes actual compiled Sifr
binaries rather than treating Python-client execution plus Sifr source presence
as equivalent runtime evidence. Docker/network runners own service cases,
ordinary CPU runners own CPU Arrow and CPU DLPack evidence, and labeled CUDA
runners own Arrow device-interface and CUDA DLPack rows; a host without the
required resource cannot promote that row to supported.

# Embedded Python Interop Architecture

This note records the production contract implemented by the embedded CPython interop phase. It complements the public guide in `docs/python-interop.mdx` and the verification surface in `verification/areas/python_interop/`.

The declaration-first package-authoring layer is specified separately in
[`python_interop_declaration_architecture.md`](./python_interop_declaration_architecture.md).
Its synchronous declarations, opaque lifecycle, synchronous contexts,
hermetic package-local bridge targets, application-owned asyncio runtime,
typed coroutine declarations, structured cancellation, and consuming async
close are implemented. Typed async contexts, declaration-first current,
foreign-thread and asyncio callbacks, typed affine buffer declarations, and
certified Arrow C Data Interface declarations are active. Declaration-first
DLPack tensor and stream acquisition, validation, and one-shot transfer are
active as well. Read-only Python plan inspection and deterministic doctor
suggestions are active on the same package/driver path used by compilation.

## Ownership Boundary

Python interop is a separate lane from Sifr's Rust-backed packages and raw C ABI interop. The final root application owns one uv-created CPython virtual environment. Sifr verifies and consumes that environment; it never installs packages, runs `uv sync`, or searches host-global Python as a fallback.

Library packages may declare `[python].requires-imports` for underivable
raw/dynamic imports, but only the root application may override
`[python].venv`, `[python].interpreter`, `[python].pyproject`, or `[python].lock`.
Normal uv layout is discovered from the nearest ancestor containing both
`pyproject.toml` and `uv.lock`, with `.venv` and its platform interpreter.

## Environment Probe

The package layer resolves the package graph, validates Python trust policy,
and builds a `PythonEnvironmentProbeRequest` containing:

- discovered or explicitly overridden venv and interpreter;
- canonical required Python roots with every manual or derived contribution;
- trusted native Python imports;
- discovered or explicitly overridden `pyproject.toml` and `uv.lock` paths.

Before codegen/build, Sifr runs read-only `uv lock --check --offline` validation
and probes the selected interpreter. Validation rejects non-CPython,
free-threaded CPython, prefixes outside the selected venv, missing
site-packages, missing required imports, failed trusted native imports, and
missing or stale project metadata. Sifr never synchronizes or mutates the
environment.

Probe metadata participates in generated artifact cache keys: interpreter path,
CPython version tuple, SOABI, extension suffixes, pointer width, platform,
`libpython` when available, project/lock digests, canonical required imports,
root-owned trust config, and sorted owning distribution names and versions.

## Read-Only Inspection

`sifr python check` resolves packages with frozen Cargo lock semantics, derives
all declaration and bridge requirements, and invokes the production driver
codegen/protocol/target-probe plan without Rust emission or build. Final
applications use the exact environment, uv-lock, trust, certification, and
probe resolver used by normal check/build. Packages with multiple runnable
targets remain applications and every target is checked.

Library-only dependency packages have no authority to select an environment.
The package layer exposes one resolution outcome shared by ordinary check and
Python inspection: `NotRequired`, `Resolved`, or
`DeferredToFinalApplication`. A standalone library root resolves when its
imports are authorized and an environment is explicitly selected or found by
normal uv-project discovery. Missing root trust and/or selection may defer only
when the session has no runnable application; every other trust, selection,
lock, probe, and certification failure remains blocking. Deferred import-root
probes are reported as `deferred`, while embedded bridge targets are
`runtime-checked`. Reports include deterministic package-graph and
source-content digests.

Ordinary package `sifr check` also consumes the same resolution outcome and
executes this full plan, including live read-only target probes when an
environment resolves. Build and run retain strict final-application authority
and never defer.

`sifr python doctor` renders the same report plus stable patch-like suggestions.
Both commands are observational: they never update Cargo/Sifr manifests,
lockfiles, trust policy, certification artifacts, or virtual environments, and
never invoke environment synchronization or installation.

## Runtime Lifecycle

The optional `sifr_runtime/python` feature owns embedded CPython lifecycle through PyO3 embedding APIs plus narrow CPython FFI where PyO3 does not expose the required primitive.

Rules:

- initialize CPython once per process from generated runtime metadata;
- use the main interpreter only;
- reject conflicting reinitialization;
- do not call `Py_FinalizeEx` during normal shutdown;
- acquire the GIL for every CPython API operation that requires it;
- decref owned object handles while holding the GIL;
- track owned Python object, buffer, Arrow, DLPack, and callback resources for deterministic diagnostics.

Generated package binaries initialize Python before user `main` when Python metadata is present. Generated Cargo metadata threads `PYO3_PYTHON` so PyO3 links/configures against the selected interpreter.

Typed Python async contexts reuse the application-owned asyncio loop. Enter,
body execution, exit, cancellation handoff, and cleanup stay on that one loop;
the compiler never creates a nested executor. The entered value may differ
from the manager object, `__aexit__` receives the original body failure, and a
truthy exit result suppresses only Python-originated body failures. Sifr
errors, returns, loop control, and cancellation retain their structured
meaning. Exit and cleanup run exactly once after a successful enter, including
on cancellation and nested-context unwinding, and an exit failure supersedes
the body failure while retaining it as secondary diagnostic context.

Package-local Python bridge modules under `src/python_bridges/` are syntax
checked, inventoried, archived, and embedded under a resolved package-specific
`__sifr_bridge__.p_<resolved_package_key>` namespace. A first-position runtime
loader owns that namespace before user `main`, rejects collisions, rewrites
same-package `bridge.*` imports, preserves stable virtual traceback filenames,
and never falls back to filesystem lookup or extraction. The generated binary
embeds every bridge module in the selected normal dependency graph, independent
of declaration reachability; dev-only and otherwise unselected packages are
excluded. Bridge source/inventory digests, resolved package identity, the
versioned binding contract, authoritative Sifr types, distribution versions,
and interpreter ABI all participate in composed cache identity.

## Object And Error Model

`py.Object` is opaque foreign state, not `Any`, and is non-send by default. Public operations are explicit `sifr.python` helpers for import, attribute/item access, calls, kwargs, conversion, context management, coroutine blocking, callbacks, and zero-copy handles.

Every Python boundary operation is fallible. Python exceptions are captured as `py.PythonError` values with operation context and traceback instead of crossing into Sifr as unwinds or panics. Compiler-emitted setup/trust failures use `SIFR-PYENV-*` and `SIFR-PYTRUST-*`; runtime Python object/call/conversion/resource/zero-copy/callback failures are typed `Result` payloads.

## Blocking And Sendability

Every public Python boundary operation is classified as `@blocking_io`. Async Sifr code must offload Python work through the existing blocking offload primitive. `py.Object`, `py.BufferView`, Arrow handles, DLPack handles, and local callbacks cannot cross task/thread boundaries unless an explicit audited bridge such as `py.threadsafe_callback` owns the transfer constraints.

## Data Interchange

Zero-copy support is explicit:

- `Py_buffer` tracks owner, pointer, length, readonly flag, format, item size, shape, strides, suboffsets, requested flags, and release state.
- Arrow provides five affine nominal resources for schema, array, stream,
  device-array, and device-stream surfaces. It validates exact PyCapsule names,
  structural C metadata, paired schema/data ownership, and release callbacks;
  declared producers additionally require environment-bound executable
  no-copy certification and exact runtime producer identity.
- DLPack declarations negotiate `copy=False` and `max_version=(1, 0)`, accept
  the stable 1.x versioned ABI plus legacy capsules from producers supporting
  that complete call shape, validate dtype/device/shape/stride/stream metadata,
  mark capsules used, and release or transfer each deleter exactly once.
- Array-interface protocols retain owners and validate pointer/dtype/shape/stride/device metadata.

Zero-copy APIs never fall back to copying. The retained raw Arrow API reports
`copy_possible = true` conservatively for every producer; only the
package-authored executable certification artifact may establish a no-copy
declaration for an exact target, kind, producer identity, distribution set,
and schema mode.

## Verification Gates

The canonical local entrypoint is `verification/areas/python_interop/run.sh`.

Important selectors:

- `--group scaffold`: matrix, fixture, and runner contract shape.
- `--group env`: live interpreter probe plus checked-in positive/negative probe fixtures.
- `readonly-check-doctor`: executable CLI parity, source-snapshot,
  deterministic-doctor, deferred-library, final-application, and byte-level
  non-mutation evidence.
- `--tier tier1`, `--tier tier2`, `--tier tier3`, `--tier tier4`: deterministic package certification evidence.
- `--group callbacks`, `--group dataframes`, `--group cloud`, `--group brokers`, and similar group filters: representative contract coverage.
- `--self-test`: runner positive/negative tests, certification-policy invariants, fixture JSON validation, and env-probe smoke.

The explicit live dependency lane is `scripts/run_all_tests.sh --profile
python-interop-live`. It selects only the live Python interop area and requires the
`container-runtime`, `network`, and `platform-specific` resource classes. Offline
profiles must not select live Python interop suites.

The live profile runs two suites. `live-policy` validates the policy boundary
without starting containers. `live-examples` compiles Sifr examples through a
generated package with explicit Python allow/trust metadata and builds one
native binary per case. Testcontainers then owns only container lifecycle and
endpoint discovery for Redis, Postgres, a Kafka-compatible Redpanda broker, and
LocalStack Pub/Sub-style SNS fanout, SNS-to-SQS, and direct SQS delivery. The
compiled binary's hermetic bridge owns every service-client operation. Broker
and cloud deliveries cross a foreign-thread typed Sifr callback and require its
acknowledgement. Docker absence is reported as `structured-skip` only after all
binaries build; with Docker available every binary must execute and produce
`live-passed` plus its resource-zero marker.

Report status values are intentional:

- `passed`: live environment execution ran.
- `matrix-passed`: deterministic matrix or contract evidence ran without live package execution.
- `scaffold`: repository-shape scaffold validation only.
- `policy-passed`: live container-runtime policy was validated without running service containers.
- `live-passed`, `structured-skip`, `live-failed`: reserved for testcontainers-backed live examples.

The declaration capability matrix is also an executable compiled-evidence
ledger. Its schema-2 `compiled_evidence` entries bind callback dispatch,
offline async HTTP, buffer, Arrow, and DLPack capability rows to exact suite
reports, case IDs, source fixtures, markers, and certification-command floors.
Before a suite starts, the outer area runner removes that suite's old target
report. After the selected suites finish, it accepts only a current
`examples-passed` report with zero failures/skips, one matching
`compiled-sifr-declaration` case, an observed exact marker, declared trust
roots, successful certification commands, and the checked-in Sifr source. The
area result records the report SHA-256. Unselected suites remain visibly
unpromoted; NumPy buffer, Arrow, and DLPack certification additionally require
their `resources=zero` marker.

Tier 4 and other host-dependent entries must include explicit `skip_reason` evidence. The full external gate is responsible for live service/host evidence.

## Example Evidence Map

The public examples in `docs/python-interop.mdx` are intentionally backed by checked-in verification evidence:

| Example family | Evidence |
| --- | --- |
| biip / schwifty package calls | `library-examples` runs `simple_import/biip_schwifty_full_example.sifr`; Tier 1a package matrix entries and `simple_import` contract coverage remain inventory evidence. |
| installed package-local biip bridge | `package_bridge_archive/package_bridge_evidence.json` records the archive/unpack/build/run proof; the package bridge showcase runs the compiled fixture after checkout and installed bridge-source removal. |
| FastAPI app construction | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `fastapi_app_contract.json` remains the contract inventory. |
| Pydantic / pydantic-core validation | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `pydantic_models_contract.json` remains the contract inventory. |
| pandas / pyarrow / polars Arrow bridge | `arrow-examples` creates and read-only rechecks exact environment-bound certifications, then compiles and runs `pyarrow_capsule/arrow_declaration_compiled.sifr` against all three producers with zero residual resources. The lower-level dataframe/library examples remain dynamic API evidence. |
| torch / TensorFlow DLPack | `dlpack-examples` compiles and runs declaration-first PyTorch and TensorFlow transfers, checks stable data pointers and zero residual resources, and exercises TensorFlow through an explicit complete-signature package bridge. `ml-examples` retains the lower-level raw PyTorch example. |
| Kafka / CFFI / asyncio / Pub/Sub callbacks | `callback-examples` compiles and runs all four offline examples, including foreign-thread CFFI and Kafka dispatch plus active retained Pub/Sub close/drain. |
| cryptography / CFFI / certifi | `library-examples` runs `cryptography_tls/cryptography_cffi_full_example.sifr`; `cryptography_tls_contract.json` remains the contract inventory. |
| boto3 / botocore cloud clients | `library-examples` runs `aws_sqs/boto3_botocore_full_example.sifr`; live LocalStack SNS/SQS examples cover service-backed delivery. |
| redis / fakeredis / hiredis | `library-examples` runs `redis/redis_fakeredis_full_example.sifr`; live Redis examples cover container-backed service behavior. |
| SQLAlchemy / Alembic / psycopg | `library-examples` runs `sqlalchemy_psycopg/sqlalchemy_psycopg_full_example.sifr`; live Postgres examples cover container-backed service behavior. |
| cloud / AI clients | `aws_sqs`, `aws_sns`, `aws_sns_sqs_subscription`, `pubsub`, `library-examples`, live LocalStack examples, and Tier 1 cloud package matrices. |
| observability / production clients | Tier 1 cloud/observability package matrix entries and deterministic matrix reports. |

## Diagnostic Evidence

Active compiler diagnostics:

- `SIFR-PYENV-0001..0011`: malformed config, multiple venvs, missing root env, probe failure, unsupported implementation, prefix mismatch, missing site-packages, missing declared import, native-load failure, free-threaded CPython, and stale project metadata.
- `SIFR-PYTRUST-0001`, `0003..0005`: dependency requirement wildcard
  rejection, native trust for a root that is not required, dynamic import
  without an explicit trust annotation, and a required root not authorized by
  the root application. `SIFR-PYTRUST-0002` is retired.

Declaration diagnostics are activated with their owning compiler surfaces. `PYIMP`,
`PYCALL`, `PYCONV`, `PYRES`, `PYCTX`, `PYASYNC`, and `PYCB` cover synchronous and
async declarations, opaque values, sync/async contexts, package bridges,
consuming async close, typed current/foreign/asyncio callbacks, and the active
typed buffer, Arrow, and DLPack protocols. `PYZC` covers their declaration,
affine ownership, layout or stream policy, certification where required,
capsule validation, transfer, and release diagnostics.

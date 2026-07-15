# Embedded Python Interop Architecture

This note records the production contract implemented by the embedded CPython interop phase. It complements the public guide in `docs/python-interop.mdx` and the verification surface in `verification/areas/python_interop/`.

The declaration-first package-authoring layer is specified separately in
[`python_interop_declaration_architecture.md`](./python_interop_declaration_architecture.md).
Its synchronous declarations, opaque lifecycle, synchronous contexts,
hermetic package-local bridge targets, application-owned asyncio runtime,
typed coroutine declarations, structured cancellation, and consuming async
close are implemented. Typed async contexts, declaration-first current,
foreign-thread and asyncio callbacks, and typed affine buffer declarations are
active; Arrow and DLPack declarations remain reserved.

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
- Arrow validates exact PyCapsule names and release callbacks for array, stream, and schema surfaces.
- DLPack enforces one-shot capsule consumption, `"used_dltensor"` marking, dtype/device validation, and exact-once deleter release.
- Array-interface protocols retain owners and validate pointer/dtype/shape/stride/device metadata.

Zero-copy APIs never fall back to copying. Copy-capable paths are represented by explicit copy APIs or `copy_possible` contract evidence.

## Verification Gates

The canonical local entrypoint is `verification/areas/python_interop/run.sh`.

Important selectors:

- `--group scaffold`: matrix, fixture, and runner contract shape.
- `--group env`: live interpreter probe plus checked-in positive/negative probe fixtures.
- `--tier tier1`, `--tier tier2`, `--tier tier3`, `--tier tier4`: deterministic package certification evidence.
- `--group callbacks`, `--group dataframes`, `--group cloud`, `--group brokers`, and similar group filters: representative contract coverage.
- `--self-test`: runner positive/negative tests, certification-policy invariants, fixture JSON validation, and env-probe smoke.

The explicit live dependency lane is `scripts/run_all_tests.sh --profile
python-interop-live`. It uses selected-areas-only execution and requires the
`container-runtime`, `network`, and `platform-specific` resource classes. Offline
profiles must not select live Python interop suites.

The live profile runs two suites. `live-policy` validates the policy boundary
without starting containers. `live-examples` compiles Sifr examples through a
generated package with explicit Python allow/trust metadata, then runs
testcontainers-backed Python client examples against Redis, Postgres, a
Kafka-compatible Redpanda broker, and LocalStack Pub/Sub-style SNS fanout, SNS,
and SQS message delivery. Docker absence is reported as `structured-skip` only
after source checks pass; with Docker available, service cases must produce
`live-passed`. Message-broker live cases consume through Python clients; the
checked Sifr fixtures pass the consumed Python object back to a
`threadsafe_callback` handler.

Report status values are intentional:

- `passed`: live environment execution ran.
- `matrix-passed`: deterministic matrix or contract evidence ran without live package execution.
- `scaffold`: repository-shape scaffold validation only.
- `policy-passed`: live container-runtime policy was validated without running service containers.
- `live-passed`, `structured-skip`, `live-failed`: reserved for testcontainers-backed live examples.

Tier 4 and other host-dependent entries must include explicit `skip_reason` evidence. The full external gate is responsible for live service/host evidence.

## Example Evidence Map

The public examples in `docs/python-interop.mdx` are intentionally backed by checked-in verification evidence:

| Example family | Evidence |
| --- | --- |
| biip / schwifty package calls | `library-examples` runs `simple_import/biip_schwifty_full_example.sifr`; Tier 1a package matrix entries and `simple_import` contract coverage remain inventory evidence. |
| installed package-local biip bridge | `package_bridge_archive/package_bridge_evidence.json` records the archive/unpack/build/run proof; the package bridge showcase runs the compiled fixture after checkout and installed bridge-source removal. |
| FastAPI app construction | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `fastapi_app_contract.json` remains the contract inventory. |
| Pydantic / pydantic-core validation | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `pydantic_models_contract.json` remains the contract inventory. |
| pandas / pyarrow / polars Arrow bridge | `dataframe-examples` runs pandas and Polars examples; `library-examples` runs `pyarrow_capsule/pyarrow_full_example.sifr`; Arrow capsule fixtures remain contract inventory. |
| torch / TensorFlow DLPack | `ml-examples` runs `torch_dlpack/torch_full_example.sifr`; TensorFlow remains host-dependent matrix/contract evidence through `tensorflow_dlpack_contract.json`. |
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
consuming async close, and typed current/foreign/asyncio callbacks. `PYZC`
remains reserved until the later zero-copy protocols activate.

# Embedded Python Interop Architecture

This note records the production contract implemented by the embedded CPython interop phase. It complements the public guide in `docs/python-interop.mdx` and the verification surface in `verification/areas/python_interop/`.

## Ownership Boundary

Python interop is a separate lane from Sifr's Rust-backed packages and raw C ABI interop. The final root application owns one uv-created CPython virtual environment. Sifr verifies and consumes that environment; it never installs packages, runs `uv sync`, or searches host-global Python as a fallback.

Library packages may declare `[python].requires-imports`, but only the root application may select `[python].venv`, `[python].interpreter`, `[python].pyproject`, or `[python].lock`.

## Environment Probe

The package layer resolves the package graph, validates Python trust policy, and builds a `PythonEnvironmentRequest` containing:

- root-selected venv and interpreter;
- declared Python imports from root allow-list and dependency requirements;
- trusted native Python imports;
- configured `pyproject.toml` and `uv.lock` paths.

The selected interpreter is probed before codegen/build. Probe validation rejects non-CPython, free-threaded CPython, prefixes outside the selected venv, missing site-packages, missing declared imports, failed trusted native imports, and missing/stale configured metadata.

Probe metadata participates in generated artifact cache keys: interpreter path, CPython version tuple, SOABI, extension suffixes, pointer width, platform, `libpython` when available, project/lock digests, declared imports, and trust config.

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
| FastAPI app construction | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `fastapi_app_contract.json` remains the contract inventory. |
| Pydantic / pydantic-core validation | `library-examples` runs `fastapi_app/fastapi_pydantic_full_example.sifr`; `pydantic_models_contract.json` remains the contract inventory. |
| pandas / pyarrow / polars Arrow bridge | `dataframe-examples` runs pandas and Polars examples; `library-examples` runs `pyarrow_capsule/pyarrow_full_example.sifr`; Arrow capsule fixtures remain contract inventory. |
| torch / TensorFlow DLPack | `ml-examples` runs `torch_dlpack/torch_full_example.sifr`; TensorFlow remains host-dependent matrix/contract evidence through `tensorflow_dlpack_contract.json`. |
| Kafka callbacks | `kafka` and `cffi_callback` contracts plus callback source fixtures. |
| cryptography / CFFI / certifi | `library-examples` runs `cryptography_tls/cryptography_cffi_full_example.sifr`; `cryptography_tls_contract.json` remains the contract inventory. |
| boto3 / botocore cloud clients | `library-examples` runs `aws_sqs/boto3_botocore_full_example.sifr`; live LocalStack SNS/SQS examples cover service-backed delivery. |
| redis / fakeredis / hiredis | `library-examples` runs `redis/redis_fakeredis_full_example.sifr`; live Redis examples cover container-backed service behavior. |
| SQLAlchemy / Alembic / psycopg | `library-examples` runs `sqlalchemy_psycopg/sqlalchemy_psycopg_full_example.sifr`; live Postgres examples cover container-backed service behavior. |
| cloud / AI clients | `aws_sqs`, `aws_sns`, `aws_sns_sqs_subscription`, `pubsub`, `library-examples`, live LocalStack examples, and Tier 1 cloud package matrices. |
| observability / production clients | Tier 1 cloud/observability package matrix entries and deterministic matrix reports. |

## Diagnostic Evidence

Active compiler diagnostics:

- `SIFR-PYENV-0001..0011`: malformed config, multiple venvs, missing root env, probe failure, unsupported implementation, prefix mismatch, missing site-packages, missing declared import, native-load failure, free-threaded CPython, and stale project metadata.
- `SIFR-PYTRUST-0001..0004`: dependency wildcard rejection, allowed-but-untrusted imports, native trust without allow-list, and dynamic import without explicit trust annotation.

Reserved families `SIFR-PYIMP`, `SIFR-PYCALL`, `SIFR-PYCONV`, `SIFR-PYRES`, `SIFR-PYZC`, and `SIFR-PYCB` remain allocated for future compiler-emitted diagnostics if runtime Python error values later need promotion to compiler diagnostics.

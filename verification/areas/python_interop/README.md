# Python Interop Verification

This directory owns the embedded CPython interop verification surface. It is
separate from CPython source-parity checks: source parity compares Sifr language
behavior to CPython, while this area verifies Sifr programs calling packages in a
selected uv-created CPython environment.

The canonical entrypoint is:

```bash
verification/areas/python_interop/run.sh --group scaffold
verification/areas/python_interop/run.sh --group env
python3 verification/areas/python_interop/runner.py --suite readonly-check-doctor
verification/areas/python_interop/run.sh --tier tier1
verification/areas/python_interop/run.sh --tier tier4
verification/areas/python_interop/run.sh --package pandas
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --dataframe-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --buffer-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --dlpack-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --ml-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --library-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --async-declaration-examples
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --async-context-examples
verification/areas/python_interop/run.sh --self-test
scripts/run_all_tests.sh --profile python-interop-live
```

The scaffold group validates the checked-in matrix and fixture surface. Tier,
gate, and package filters emit deterministic package-certification evidence from
the checked-in matrix and contracts. The env group records live interpreter ABI
evidence and validates checked-in positive probe fixtures plus concrete negative
probe/selection cases. The runner must never invoke `uv sync` or install
packages implicitly.

The blocking `readonly-check-doctor` suite proves that `sifr python check` and
normal check accept and reject the same final-application targets, library-only
probes defer explicitly, doctor JSON and patches are byte-deterministic,
source-content snapshot digests change with source bytes, and no inspected
package file or symlink changes on success or failure.

`declaration_capabilities.json` is the separate declaration/protocol capability
ledger. Its `target_state` classifies the intended contract as
`declaration-supported`, `bridge-supported`, `dynamic-only`, or
`unsupported-by-design`; it does not claim implementation. Current availability
is recorded independently as `reserved` or `active`, and a reserved row cannot
claim passing evidence. Each row names its durable activation owner and the status of
positive, negative, cleanup, cancellation, and live evidence. The scaffold and
self-test suites reject missing ownership, unsupported states, duplicate rows,
or a supported claim without the evidence required by that row.

Live dependency examples are intentionally opt-in. The `python-interop-live`
profile uses selected-areas-only execution and runs both:

- `live-policy`: verifies the container-runtime/testcontainers policy without
  starting containers.
- `live-examples`: type-checks Sifr interop source examples through an explicit
  package trust policy, then runs testcontainers-backed Python client examples
  for Redis, Postgres, a Kafka-compatible Redpanda broker, and LocalStack
  Pub/Sub-style SNS fanout, SNS, and SQS message delivery.

The live examples use the area-local locked Python project and never install
packages from the runner itself. If Docker is unavailable, the suite emits
`structured-skip` for the service cases after the Sifr source checks pass. When
Docker is running, those same cases must reach `live-passed` or fail the profile.
The Kafka, Pub/Sub-style, SNS, and SQS live cases produce and consume messages
in Python client code. Their checked Sifr source fixtures pass the consumed
Python object to a Sifr `threadsafe_callback` handler, and the report labels
that portion as a source-checked callback contract rather than a live Sifr binary
invocation.
Declaration-first callback evidence is also compiled offline with
`runner/run.py --callback-examples`: separate real CFFI caller-thread and
worker-thread fixtures for current and foreign dispatch, kafka-python
foreign-thread dispatch, application-owned asyncio dispatch, and a retained
Pub/Sub-style owner with consuming async-close drain.
The policy service aliases map to these concrete cases: `pubsub-compatible`
uses LocalStack SNS fanout to an SQS subscription, `aws-compatible-sns` uses
LocalStack SNS delivery to SQS, `aws-compatible-sqs` uses direct LocalStack SQS,
and `aws-compatible-sns-sqs` names the shared LocalStack topology.
Service-backed examples belong in this profile, not in the offline
create-pr/merge/nightly/release profiles. The area manifest remains offline by
default; live suites must declare their own `network_mode` and resource classes.

## Groups

- `scaffold`: validates matrix files, fixture directories, runner modules, and
  report output.
- `env`: interpreter, venv, ABI, platform, lock/env freshness, and probe rejection fixture coverage.
- `imports`: root imports and native extension load diagnostics.
- `native`: trusted native Python package load/use smoke.
- `async`: Python event-loop/client behavior under Sifr blocking semantics.
- `callbacks`: local/threadsafe helpers plus declaration-first current, foreign, and asyncio callback behavior.
- `buffers`: `Py_buffer` ownership, contiguity, readonly/writable, dtype/format.
- `arrow`: Arrow PyCapsule schema/array/stream and zero-copy-vs-copy diagnostics.
- `dlpack`: one-shot tensor capsules, dtype/device/stride handling.
- `dataframes`: pandas/polars/pyarrow dataframe interop.
- `tensors`: NumPy/torch/tensorflow tensor interop.
- `databases`: SQLAlchemy, psycopg, asyncpg, pymongo, motor, redis.
- `brokers`: confluent-kafka, aiokafka, kafka-python, SQS/SNS, Pub/Sub-style callbacks.
- `cloud`: AWS/Google/OpenAI SDK import and auth surface checks without live credentials in the default gate.
- `web`: FastAPI/Starlette/Django/Sanic import and in-process smoke fixtures.
- `cleanup`: close/context-manager/callback release/leak diagnostics.

## Reports

Runner output is written under `target/verification/areas/python_interop/` by
the area and profile runners. Reports use
deterministic JSON with selected filters, matrix counts, fixture coverage, and
package-certification status so interop evidence can be reviewed before live
package execution gates exist.

Dataframe examples are offline but executable. The `dataframe-examples` case
links a temporary Sifr package to the area-local locked uv environment and runs
real Sifr programs for NumPy, pandas, and Polars. These examples cover array or
dataframe construction, library operations, conversion back into typed Sifr
values, explicit result assertions, and deterministic stdout markers checked by
the runner. They are separate from the dataframes matrix case so reviewers can
distinguish package-certification metadata from compiled Sifr execution
evidence. Runner self-tests validate report aggregation and fixture drift; the
actual Cargo/Sifr/venv execution path is covered by `--dataframe-examples`.

Typed buffer examples are offline, compiled, and blocking in every delivery
profile. The `buffer-examples` suite runs declaration-first binaries for a
`builtins.bytearray` import-root producer, opaque `mmap` `Self` receiver,
package-local bridge producer, affine aggregate automatic cleanup, and a real
writable NumPy `int64` ndarray. Every fixture checks a deterministic marker and
zero live/leaked resources. The bridge fixture additionally checks shared
mutation and post-release exporter resizability, the aggregate fixture checks
that all six retained exporters are resizable after automatic drop, and NumPy
mutation is checked through the retained producer. C-level runtime exporters
independently prove pointer identity and exact release counts. The blocking
`buffer-cpython311` suite reruns all five C-level tests and all five compiled
binaries in a minimal locked CPython 3.11 environment. The complete positive,
negative, cleanup,
cancellation disposition, live-source, and profile ownership matrix is locked
in `fixtures/numpy_buffer/buffer_declaration_evidence.json` and validated by the
runner self-test.

ML examples are offline but executable. The `ml-examples` suite uses the same
temporary-package execution path and runs real Sifr programs for torch and
scikit-learn. The torch example constructs a CPU `float32` tensor, performs
tensor math, converts results back to typed Sifr values, and validates DLPack
metadata/release. The scikit-learn example trains a deterministic decision tree,
predicts labels, copies predictions/classes back into typed Sifr lists, and
checks deterministic stdout markers.

DLPack declaration examples are offline, compiled, and blocking in every
delivery profile. The `dlpack-examples` suite runs real PyTorch and TensorFlow
CPU transfers, verifies stable data pointers, exact device metadata, owned
one-shot consumption, and zero residual resources. PyTorch exercises direct
import-root and `Self` acquisition. TensorFlow exercises an explicit
package-local bridge that adapts its capsule API to the complete versioned
`__dlpack__` call shape without copying.
The companion `dlpack-cpython311` suite runs the exact Python-feature runtime
test inventory, so malformed capsules, copied flags, no-retry behavior,
stream/device mismatches, and exact-once cleanup remain blocking evidence.

Library examples are offline but executable. The `library-examples` suite covers
the remaining non-service, non-host-dependent library contracts that previously
had only matrix or JSON/source-fixture evidence:

- biip GTIN parsing and schwifty BIC validation.
- pyarrow array compute plus Arrow PyCapsule metadata/release.
- FastAPI app construction, Pydantic validation through pydantic-core, and
  Starlette JSON response rendering.
- cryptography Fernet encrypt/decrypt, CFFI parser setup, and certifi CA bundle
  discovery.
- boto3 SQS client calls through botocore Stubber with no AWS credentials or
  network access.
- redis client import, fakeredis in-process round trip, and hiredis RESP
  parsing.
- SQLAlchemy in-memory query execution plus Alembic migration-context
  construction and psycopg conninfo construction.

Service-backed libraries remain in `live-examples`: Redis, Postgres/psycopg,
Kafka, Pub/Sub-style SNS fanout, SNS, and SQS are exercised with testcontainers
when Docker is available. TensorFlow CPU DLPack is covered by the locked
offline `dlpack-examples` suite.

Package certification records include:

- per-package tier, gate, group, native-extension, and host-dependency metadata;
- deterministic pass records for Tier 1, Tier 2, and Tier 3 matrix contracts;
- top-level `matrix-passed` status for matrix-only evidence, distinct from live
  environment/package execution status;
- explicit Tier 4 skip records with `skip_reason` for host-dependent packages;
- aggregate tier, gate, pass, and skip counts.

The exit evidence is recorded in
`reports/python_interop_exit_evidence.md`, including diagnostic families,
validation commands, PR links, and the distinction between live and matrix-only
gates.

Live example reports additionally include:

- `source_checks`: `sifr check` results for the Sifr interop examples, compiled
  through generated package metadata that declares allowed and trusted Python
  roots.
- `cases`: service-backed testcontainers results, each tied to the Sifr source
  it proves.
- `container_runtime`: Docker daemon availability and structured skip reason
  when the local runtime is absent.

Dataframe example reports additionally include:

- `source_checks`: checked-in Sifr example presence for NumPy, pandas, and Polars.
- `cases`: `sifr run` results for each compiled example.
- `stdout_marker`: deterministic per-example output required for a passing case.
- `dependencies`: the Python import roots trusted by at least one temporary package.

ML example reports use the same schema and include:

- `source_checks`: checked-in Sifr example presence for torch and scikit-learn.
- `cases`: `sifr run` results for each compiled example.
- `stdout_marker`: deterministic per-example output required for a passing case.
- `dependencies`: the Python import roots trusted by at least one temporary package.

Library example reports use the same schema and include:

- `source_checks`: checked-in Sifr example presence for pyarrow,
  biip/schwifty, FastAPI/Pydantic/Starlette, cryptography/CFFI/certifi,
  boto3/botocore, redis/fakeredis/hiredis, and SQLAlchemy/Alembic/psycopg.
- `cases`: `sifr run` results for each compiled example.
- `stdout_marker`: deterministic per-example output required for a passing case.
- `dependencies`: the Python import roots trusted by at least one temporary package.

The `async-declaration-examples` suite is offline and compiled. It embeds a
package-local bridge, runs a real `httpx.AsyncClient` against an in-process ASGI
transport, proves typed coroutine functions/factories, recursive conversion,
one loop identity, and consuming async close, and requires a deterministic
binary stdout marker. Cancellation, suppression, failure, conversion, poison,
and shutdown matrices are owned by the focused lowering/codegen/runtime tests
named in `fixtures/async_declaration/async_declaration_evidence.json`.

The `async-context-examples` suite is offline and compiled. It runs a real
`aiosqlite.Connection` subclass over in-memory SQLite and proves one owned-loop
identity, typed enter/value conversion, Python-only suppression, unsuppressible
Sifr truthiness, exact-once exit/close, cancellation-finally-exit ordering,
secondary exit failure, and mixed synchronous/asynchronous LIFO cleanup. Its
positive, negative, cleanup, cancellation, and live owners are recorded in
`fixtures/async_context/async_context_evidence.json`. The suite is a blocking,
unconditional selection in create-PR, merge, nightly, and release profiles.

# Python Interop Verification

This directory owns the embedded CPython interop verification surface. It is
separate from CPython source-parity checks: source parity compares Sifr language
behavior to CPython, while this area verifies Sifr programs calling packages in a
selected uv-created CPython environment.

The canonical entrypoint is:

```bash
verification/areas/python_interop/run.sh --group scaffold
verification/areas/python_interop/run.sh --group env
verification/areas/python_interop/run.sh --tier tier1
verification/areas/python_interop/run.sh --tier tier4
verification/areas/python_interop/run.sh --package pandas
verification/areas/python_interop/run.sh --self-test
scripts/run_all_tests.sh --profile python-interop-live
```

The scaffold group validates the checked-in matrix and fixture surface. Tier,
gate, and package filters emit deterministic package-certification evidence from
the checked-in matrix and contracts. The env group records live interpreter ABI
evidence and validates checked-in positive probe fixtures plus concrete negative
probe/selection cases. The runner must never invoke `uv sync` or install
packages implicitly.

Live dependency examples are intentionally opt-in. The `python-interop-live`
profile uses selected-areas-only execution and runs both:

- `live-policy`: verifies the container-runtime/testcontainers policy without
  starting containers.
- `live-examples`: type-checks Sifr interop source examples through an explicit
  package trust policy, then runs testcontainers-backed Python client examples
  for Redis, Postgres, a Kafka-compatible Redpanda broker, and LocalStack
  SNS/SQS.

The live examples use the area-local locked Python project and never install
packages from the runner itself. If Docker is unavailable, the suite emits
`structured-skip` for the service cases after the Sifr source checks pass. When
Docker is running, those same cases must reach `live-passed` or fail the profile.
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
- `callbacks`: local and threadsafe callback behavior.
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

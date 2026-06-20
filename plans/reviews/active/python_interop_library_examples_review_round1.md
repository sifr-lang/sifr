I have all the context I need. Here is the review.

## Summary

The PR adds an offline `library-examples` suite that compiles and runs seven Sifr programs against the area-local uv environment, wires it into create-pr/merge/release profiles, and updates docs/runner/manifest accordingly. Plumbing (runner integration, profile selection, fixture/source registration, README, architecture, exit-evidence map) is consistent with how the existing `dataframe-examples` and `ml-examples` suites are wired. The Sifr fixtures themselves carry several correctness and depth-of-coverage concerns worth addressing.

## Blocking findings

**1. `aws_sqs/boto3_botocore_full_example.sifr:43` — `copy_dict_str_str(create_result)` very likely fails on injected `ResponseMetadata`.**
botocore client responses are populated by the protocol parser, which injects a `ResponseMetadata` dict (`HTTPStatusCode`, `RequestId`, `HTTPHeaders`) into every client response, including ones served by `Stubber`. The runtime helper in `crates/sifr_runtime/src/python/object_ops.rs:625` (`copy_dict_str`) does a strict `extract::<String>()` on every value with no skipping; a nested dict value will return `Err`, propagate as `PythonError`, exit non-zero, no stdout marker → `example-failed`.

The local-validation note says the suite passed end-to-end; that contradicts what I'd expect from the code. Two reads are possible: (a) the local run did not actually hit the example, or (b) version 1.43.33 of botocore behaves differently than my mental model. Either way, the safer, version-stable fix is to project the single key needed instead of converting the whole dict:

```
queue_url_obj: Object = get_item(create_result, from_str_key)
queue_url: str = to_str(queue_url_obj)
```

This also removes the dependency on `copy_dict_str_str` accepting whatever botocore returns. Please re-run the suite under a clean `target/` and confirm; if it really does pass on this lock, capture the response shape in the case payload so future botocore bumps don't silently flip behavior.

## Non-blocking findings

**2. `fastapi_app/fastapi_pydantic_full_example.sifr:35` uses Pydantic-internal kwargs.**
`TypeAdapter(int, _parent_depth=0, module="builtins")` passes underscore-prefixed and undocumented kwargs. These are not part of the public `<3.0` Pydantic surface and can be renamed/removed in any 2.x point release. If they are needed because Sifr's embedded call frame breaks Pydantic's introspection, document that in a one-line comment; otherwise drop them and call `TypeAdapter(int)`.

**3. `sqlalchemy_psycopg/sqlalchemy_psycopg_full_example.sifr:19` doesn't actually exercise psycopg.**
The engine URL is `sqlite+pysqlite:///:memory:` and psycopg is only touched via `psycopg.conninfo.make_conninfo` for string construction — it never drives a query. The case is named after psycopg, the matrix entry implies psycopg coverage, and the README implies a real interop path; the runnable example is shallow. Either run the engine against psycopg in-memory equivalents available offline (there aren't many) or rename the case to make the limited coverage explicit.

**4. `redis/redis_fakeredis_full_example.sifr:20-30,39-40` smokes `redis` only via `__version__`.**
The round trip uses `fakeredis.FakeRedis`; the real `redis.Redis` client API is never invoked. That makes the case mostly a fakeredis test with a redis version-attribute read tacked on. Either construct a `redis.Redis` instance against the FakeRedis-backed server (fakeredis can patch it) or drop the redis import from the marker phrasing.

**5. None of the new examples use `py.with_context` — manual `close()` chains leak handles on mid-flow failure.**
`except PythonError as e: raise e` re-raises without running any of the close()s that follow `passed = ...`. Outstanding-resource leak diagnostics will fire on any partial failure. The dataframe/ml examples follow the same pattern, so this is consistency-correct, but the production contract documented in `internal_docs/python_interop_architecture.md` and `plans/issues/active/ad-hoc-embedded-python-interop.md` flags `with_context` as the recommended ergonomic surface for cleanup. Worth threading at least one example onto `with_context` so the suite isn't entirely a counter-example for the docs.

**6. `pyproject.toml` adds `alembic>=1.13,<2` but no fixture imports it.**
Alembic appears in `verification/areas/python_interop/packages/brokers.toml` and `fixtures/sqlalchemy_psycopg/sqlalchemy_psycopg_contract.json:5`, but no Sifr code calls it. Dead dep weight in the uv resolution. Either add a fixture that constructs an `alembic.runtime.migration.MigrationContext` (alembic loads cleanly without a real DB) or drop the dep.

**7. `fastapi_pydantic_full_example.sifr` trusts `"builtins"` in `python-native` (via `runner/library_examples.py:26`).**
`builtins` is a CPython core module, not a loadable extension. `prepare_example_package` (`runner/example_packages.py:204-209`) copies `import_roots` into both `allow-imports` and `trust.python-native`. The Sifr probe (`crates/sifr_runtime/src/python/...`) detects native roots from extension-module origins; how it handles `builtins` is worth verifying. If it silently no-ops, fine — but the example shouldn't claim "builtins is a trusted native extension." Either drop `builtins` from `import_roots` (use `pydantic` directly via a different route) or split the runner into `allow_imports` and `native_imports` rather than reusing one list.

**8. Submodule trust roots mixed in with package roots in `runner/library_examples.py:50`.**
`("psycopg", "psycopg.conninfo", "sqlalchemy")` lists both `psycopg` (never imported in the example) and `psycopg.conninfo` (the only psycopg path actually called). `optional_import_root_list` (`crates/sifr_package/src/manifest/sifr_fields.rs:148`) accepts dotted identifiers, so neither is rejected, but the bare `psycopg` root is dead policy state. Drop it, or actually `import_module("psycopg")` for version-attribute parity with the redis case.

**9. `pyarrow_full_example.sifr:43` marker `producer=pyarrow` is hardcoded.**
The assertion is `capsule.producer_module != ""`; the marker pretends the actual value is `pyarrow`. If pyarrow upstream changes `producer_module` to `"pyarrow.lib"`, the assertion passes and the marker still claims `pyarrow`. Either assert equality on the literal you embed or include the value in the print (`producer={capsule.producer_module}`).

**10. create-pr budget pressure (`verification/profiles/create-pr.json:92`).**
Profile budget is `warm_wall_time_minutes: 2`. The prior ML-examples PR landed with a "warm wall-time advisory" — meaning the prior addition already crowded the budget. Library-examples adds seven sequential `cargo run -p sifr -- run` invocations, each spinning up a fresh embedded CPython against the temp package. Worth confirming that this stays inside the create-pr budget on a cold cache; if not, gate `libraries` to merge/release only.

**11. Tier 1a Python interop libraries still without full offline examples.**
The plan explicitly scopes biip/schwifty…SQLAlchemy/psycopg to this PR, so this isn't a missed deliverable, but `httpx`, `requests`, `openai`, and `google-genai` are Tier 1a (per the phase contract) without a runnable offline example. `httpx`/`requests` are tractable offline (instantiate a Session, build a Request, exercise a transport stub). `openai`/`google-genai` need credentials/network and reasonably stay matrix-only. Worth a follow-up bullet on the phase plan rather than blocking this PR.

**12. Library-examples suite is happy-path only.**
The existing JSON contract fixtures cover negative cases for each library, but the runnable layer never demonstrates that error propagation behaves correctly when an upstream library raises. The dataframe/ml suites have the same gap; the contract layer can stay the authoritative negative gate, but at least one runnable per surface (e.g., Stubber assertion mismatch in boto3) would harden the suite against silent regressions.

**13. Cleanup style would scale better with a helper.**
Each example ends with a 12–18 line block of `closed_X: None = close(X)` declarations in reverse construction order. This is easy to misorder during edits and easy to leak (see #5). A `py.with_context`-based helper or a multi-handle scope close would scale better as more examples are added — non-blocking, but worth a follow-up.

## Things that look right

- `verification/areas/python_interop/runner/library_examples.py` mirrors the `dataframe_examples.py` / `ml_examples.py` pattern exactly; report shape and self-test wiring stay consistent.
- `runner/run.py:16,143,156,192-200` and `runner.py:79-83,105` plumbing is symmetric with the existing example suites, including `AREA_PROJECT_COMMANDS` membership for uv-driven execution.
- `verification/profiles/{create-pr,merge,release}.json` all add `libraries` to `python_interop.suites`; manifest case has `expect_exit_code: 0`, and `_run_case` propagates non-zero on either bad exit or missing stdout marker.
- `internal_docs/python_interop_architecture.md` updates the example evidence table for biip/schwifty, FastAPI/Pydantic, pyarrow, cryptography/CFFI/certifi, boto3/botocore, redis/fakeredis/hiredis, and SQLAlchemy/psycopg.
- Trust-list construction in `runner/example_packages.py:191-209` correctly mirrors `import_roots` into both `[python] allow-imports` and `[trust].python`/`python-native`; `optional_import_root_list` accepts dotted identifiers.
- Marker assertions look numerically correct for each fixture (GTIN-13 check digit 8, BIC `DEUTDEFF` → country `DE`/bank `DEUT`, pyarrow sum 10, Fernet roundtrip, sqlite scalar 42).

The boto3 finding is the only one I'd treat as a real blocker pending verification; everything else is depth-of-coverage, hygiene, or future-work flagging.

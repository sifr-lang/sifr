I have enough to deliver the review. No files were modified.

---

## Round 2 review — `codex/python-interop-library-full-examples`

### Blocker resolution

**Round 1 #1 (boto3 `copy_dict_str_str` vs injected `ResponseMetadata`) — resolved.**
`aws_sqs/boto3_botocore_full_example.sifr:44-45` now projects the single key:

```
queue_url_obj: Object = get_item(create_result, "QueueUrl")
queue_url: str = to_str(queue_url_obj)
```

`get_item` lowers to `_sifr.python.py_get_item_str` (`crates/sifr_stdlib/src/python.rs:34-44`), whose key arg is `Str`, so the string literal "QueueUrl" is the right shape. The runtime never iterates ResponseMetadata. Closure order is correct (lines 50-67 close in reverse-allocation order). ✅

### Round 1 non-blocking findings — addressed

- **#2 Pydantic kwargs**: `fastapi_pydantic_full_example.sifr:35` carries the required one-line motive: *"Embedded calls may not provide Pydantic's default two-frame caller context."* ✅
- **#3 SQLAlchemy/psycopg depth**: psycopg is now actually imported (`sqlalchemy_psycopg_full_example.sifr:18`); Alembic's `MigrationContext.configure(connection)` is exercised against the live connection (lines 31-35) and `dialect.name == "sqlite"` is asserted. SQLAlchemy still does the in-memory query, psycopg `__version__` and `conninfo.make_conninfo(...)` round out the surface. ✅
- **#6 Alembic dependency unused**: now used via `MigrationContext.configure`. ✅
- **#7 `builtins` in `python-native`**: `runner/example_packages.py:191-217` and `ExampleCase.native_roots` separate the two lists; `library_examples.py:24-30` keeps `builtins` in `import_roots` only and lists only `pydantic_core` as native. ✅
- **#9 pyarrow marker drift**: assertion now requires `capsule.producer_module == "pyarrow.lib"` (line 36) and the stdout marker carries `producer=pyarrow.lib`. ✅

### Round 1 non-blocking findings — still open

These weren't in the user's fix list, restating as still-open follow-ups:

- **#4** `redis` is still smoked only via `__version__`; the actual round trip is fakeredis-only.
- **#5** No example uses `py.with_context`; `except PythonError as e: raise e` re-raises before the `close()` chain runs, so partial failures still leak handles into the cleanup-diagnostics path.
- **#8 (partial)** `library_examples.py:56` still lists dotted entries `"alembic.runtime.migration"` and `"psycopg.conninfo"` alongside their parents in `import_roots`. `validate_import_policy` extracts only the first dotted segment via `name.split('.').next()` and matches via exact equality (`crates/sifr_runtime/src/python/object_ops.rs:421-453`), so dotted entries are dead policy state. The bare `psycopg` root is no longer dead since psycopg is now imported.
- **#10** create-pr budget pressure: the suite stays in profile; no new measurement was supplied.
- **#11** Tier 1a `httpx`, `requests`, `openai`, `google-genai` still lack runnable offline examples.
- **#12** No runnable failure-mode example exercises error propagation through these libraries.
- **#13** Manual reverse-order close chains remain.

### New non-blocking findings (Round 2)

1. **`pyarrow_full_example.sifr:38` has a dead clause.** `passed = total == 10 and capsule_ok and capsule.producer_module != ""`. `capsule_ok` (line 32-37) already requires `producer_module == "pyarrow.lib"`, which strictly implies non-empty — the trailing `and capsule.producer_module != ""` never narrows. Cosmetic.

2. **Hard-coding `producer_module == "pyarrow.lib"` is a deliberate trade-off worth documenting.** It fixes the marker-vs-assertion mismatch but couples both to a PyArrow Cython compilation detail. A future PyArrow rev that moves the C-Data producer to e.g. `pyarrow._arrow_c_data` fails the example silently w.r.t. intent. Either capture the value in the marker via `f"producer={capsule.producer_module}"` (if interpolation is available here) or relax the assertion to `startswith("pyarrow.")`. Either is reasonable; the version-pinning is acceptable while the lock is pinned.

3. **`sqlalchemy-psycopg` marker token `alembic=sqlite` reads as a misspelling.** It actually carries the dialect name read off `MigrationContext.dialect.name`. Consider `dialect=sqlite` or `alembic_dialect=sqlite` for the next bump — but this is a naming nit, not a correctness issue.

4. **Self-test (`example_packages.py:59-92`) doesn't assert `native_roots` shape.** It checks every case has non-empty `import_roots` (line 90-92) but never inspects `native_roots`. A drift where a future case sets `native_roots=()` while actually loading a native extension would not be caught at self-test time — it would only show up when the example fails at runtime under trust enforcement. Adding a parallel `native_roots_ids` drift check (mirroring the marker/import-roots assertions) would close the gap. Optional.

5. **Style: `schwifty.BIC` is constructed via `call(bic_class, [bic_text], [])` (`biip_schwifty_full_example.sifr:32-34`) while sibling fixtures use the `call_attr(module, "Klass", ...)` shape (FastAPI, fakeredis). Functionally identical via `_sifr.python.py_call`; cosmetic consistency.

### Spot checks

- biip GTIN-13 `7032069804988`: odd-position sum 28, even-position sum 28 → 28·3 + 28 = 112; check digit = `(10 - 112 % 10) % 10 = 8`. Assertion `check_digit == 8` and `gtin_format == 13` (biip's `GtinFormat.GTIN_13 = 13` IntEnum) are correct.
- Marker construction across all seven cases is symmetric with the assertions.
- `manifest.json:165-176`, all three profiles, README §library-examples (lines 105-120), and `internal_docs/python_interop_architecture.md:105-114` consistently document the seven cases.
- `runner.py:79-83,104-109` and `runner/run.py:16,143,156,192-200` plumbing matches `dataframe_examples` / `ml_examples`.

### Verdict

**No remaining blocking findings.** The Round 1 blocker is cleanly resolved. The eight non-blocking observations above are the new surface from Round 2 — none gates the PR, and items 4 and 5 of "Round 1 still open" (`with_context` + happy-path-only) remain the highest-value follow-ups for a future PR.

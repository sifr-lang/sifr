# Ad Hoc Phase: Structured Data and Class-Surface Parity Expansion

Status: completed (started 2026-03-18; completed 2026-03-18 after wave/milestone/phase closure review cycles)
Context: follow-up phase after iterator architecture and iterator-waiver reduction
Execution readiness: implementation-ready after `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
Execution ledger: `issues/ad-hoc-structured-data-and-class-surface-parity-expansion-execution.md`

## Objective

Expand CPython parity for the remaining high-value structured-data, parser, and class-surface gaps in modules that are already `parity-closed` for their shipped subset but still carry explicit waiver debt.

This phase does not reopen these modules from zero. It consumes the next tranche of explicit residual waivers under fixed design decisions so implementation can begin without a separate architecture spike.

Primary module targets:

- `json`
- `configparser`
- `csv`
- `collections`
- `argparse`
- `uuid`
- `datetime`

Secondary polish targets:

- `textwrap`
- `html`

## Source of Truth

This phase must use the following as authoritative references:

- canonical parity inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant closure wave ledgers:
  - `verification/stdlib/wave_psp_b1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_c1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_c2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e2_cpython_traceability.md`
- existing phase docs:
  - `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
- architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/07_stdlib_parity.md`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream families:

- `Lib/test/test_json/`
- `Lib/test/test_configparser.py`
- `Lib/test/test_csv.py`
- `Lib/test/test_collections.py`
- `Lib/test/test_argparse.py`
- `Lib/test/test_uuid.py`
- `Lib/test/test_datetime.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Why This Needs Its Own Phase

The remaining gaps in these modules are now concentrated in:

- richer class/object surfaces,
- constructor and option-matrix expansion,
- structured streaming and parser wrappers,
- bounded configuration surfaces,
- and cleanup of intentionally narrowed class families.

That is a different root-cause family from iterator architecture, runtime/file-object parity, or stateful RNG parity.

## Depends on

- `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - iterator-returning APIs must already be settled so parser and collection surfaces do not build on stale eager behavior
- milestone-7 canonical closure inventory remains the baseline
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Public Surface Contract

### `json`

- Arbitrary callable hooks are out of scope in this phase:
  - `object_hook`
  - `object_pairs_hook`
  - `parse_float`
  - `parse_int`
  - `parse_constant`
  - `default`
- Add typed configuration wrappers only:
  - `JSONEncoder(indent: int | None = None, sort_keys: bool = False, ensure_ascii: bool = True)`
  - `JSONDecoder()`
- `JSONEncoder` owns:
  - `encode(value: JsonValue) -> str`
  - `dump(value: JsonValue, path: str) -> Result[None, IOError]`
  - `dump_handle(value: JsonValue, fh: FileHandle) -> Result[None, IOError]`
- `JSONDecoder` owns:
  - `decode(s: str) -> Result[JsonValue, JSONDecodeError]`
  - `load(path: str) -> Result[JsonValue, Error]`
  - `load_handle(fh: FileHandle) -> Result[JsonValue, Error]`
- Top-level `load`, `loads`, `dump`, and `dumps` stay as the simple typed entry points already shipped; config-driven behavior is exposed through the wrapper classes rather than callback-heavy overloads.

### `datetime`

- This phase targets fixed-offset timezone parity only.
- No IANA timezone database, `zoneinfo`, DST rules, or `fold` semantics.
- `timezone` is the only supported timezone implementation.
- Export `UTC` as the canonical uppercase alias; keep existing lowercase `utc` only as a backward-compat alias if already shipped.
- `datetime` stores an optional fixed-offset `timezone | None`.
- Required surfaces:
  - `now(tz: timezone | None = None) -> datetime`
  - `from_timestamp(ts: float, tz: timezone | None = None) -> Result[datetime, ValueError]`
  - `astimezone(tz: timezone | None = None) -> Result[datetime, ValueError]`
- `tzinfo` is not introduced as an extensible base class in this phase.

### `uuid`

- Add:
  - `uuid3(namespace: UUID, name: str) -> UUID`
  - `uuid5(namespace: UUID, name: str) -> UUID`
  - namespace constants:
    - `NAMESPACE_DNS`
    - `NAMESPACE_URL`
    - `NAMESPACE_OID`
    - `NAMESPACE_X500`
- Keep raw `UUID(...)` strict constructor parity as an `intentional-diff` unless constructor lowering is independently closed before implementation begins.
- Prefer typed generation and parse entry points over dynamic constructor overload matrices.

### `collections`

- Add `Counter(iterable)` and `Counter(mapping)` constructor parity.
- Keep `Counter(**kwargs)` out of scope in this phase.
- Promote `defaultdict` to a real typed class with:
  - `default_factory` field
  - missing-key initialization semantics
  - explicit methods rather than compiler-only special casing
- Do not expand into `namedtuple`, `ChainMap`, or user container families in this phase.

### `csv`

- After phase 2, `csv.reader` and `csv.DictReader` are iterator-returning APIs.
- `csv.writer` and `csv.DictWriter` remain eager output surfaces.
- Add a process-local dialect registry with immutable `Dialect` values.
- No dynamic subclass registration or mutation-heavy registry semantics.

### `argparse`

- Support:
  - `subparsers`
  - `nargs` forms:
    - exact integer
    - `?`
    - `*`
    - `+`
- `type=` accepts:
  - built-in coercers `str`, `int`, `float`, `bool`
  - or callables with signature `str -> Result[T, Error]`
- Help-formatting subclass ecosystems remain out of scope.

### `textwrap` / `html`

- `textwrap.TextWrapper` expands only through explicitly named missing fields already adjacent to the current class surface.
- `html` remains top-level-module scoped; `html.parser` and package-wide expansion are out of scope.

## Permanent Sifr-Safe Diffs

The following are intentionally not part of this phase’s execution target:

- arbitrary dynamic JSON callback hooks,
- `tzinfo` subclass ecosystems and timezone-database behavior,
- `Counter(**kwargs)` constructor parity,
- dynamic CSV dialect subclass registration,
- `argparse` formatter-class ecosystems,
- package-wide `html` expansion.

If these remain unsupported at phase exit, they must be recorded as explicit `intentional-diff` or `unsupported` entries rather than left vague.

## Scope

This phase owns:

- `json` typed wrapper expansion without dynamic callback injection,
- `configparser` interpolation, proxy, and write-back parity expansion,
- `csv` iterator reader lifecycle and dialect registry parity,
- `collections` constructor and object-model parity beyond the current narrowed subset,
- `argparse` advanced parser features within the bounded `nargs` and `type=` model above,
- `uuid` non-v4 generation families and namespace constants,
- `datetime` fixed-offset timezone and richer class-family parity,
- `textwrap` advanced `TextWrapper` option matrices adjacent to the shipped surface,
- `html` top-level-module polish only.

This phase does not own:

- async iterators,
- first-class `bytes` object-model design,
- runtime host abstractions,
- `io` / stream hierarchy expansion,
- `zipfile` / archive file-object expansion,
- `tempfile` object lifecycle wrappers,
- `logging` hierarchy/configuration expansion,
- `time` / `timeit` host-object expansion,
- stateful RNG object families,
- bytes-native crypto runtime expansion.

## Non-goals

- reopening modules that are already correctly governed unless this phase shrinks a documented waiver,
- adding fallback-only helper APIs instead of natural CPython-shaped entry points,
- weakening typed safety to mimic CPython exception-first behavior,
- merging runtime/file-object work into this implementation track.

## Priority Targets

### priority_1: Parser and serialization parity

Modules:

- `json`
- `configparser`
- `csv`

Required closure direction:

- close the typed wrapper and object-model expansion under the fixed JSON config model above,
- close `configparser` interpolation/proxy/write-back families deliberately,
- convert `csv` to the bounded iterator + registry model described above.

### priority_2: Class-heavy compatibility expansion

Modules:

- `collections`
- `argparse`
- `uuid`
- `datetime`

Required closure direction:

- broaden constructor parity for `Counter` and `defaultdict`,
- add `argparse` advanced parser features with deterministic typed behavior,
- close selected `uuid` non-v4 generation and constructor-adjacent gaps,
- expand `datetime` using fixed-offset timezone semantics only.

### priority_3: Text and formatter polish

Modules:

- `textwrap`
- `html`

Required closure direction:

- close the remaining `TextWrapper` option matrix adjacent to the shipped class,
- keep `html` bounded to top-level module parity.

## Waves

### wave_psp_struct_0: Architecture Lock

Scope:

- `json`
- `datetime`
- `uuid`
- `csv`
- `argparse`
- `collections`

Definition of done:

- the fixed public surface contract in this document is reflected in traceability, waivers, and implementation notes,
- no later wave needs to invent callback, timezone, constructor, or registry semantics,
- every permanently deferred family is explicitly classified before implementation proceeds.

### wave_psp_struct_1: Parser and Serialization Surface Expansion

Scope:

- `json`
- `configparser`
- `csv`

Definition of done:

- the next tranche of parser and serializer waivers is closed or sharply reclassified,
- CPython-derived tests cover both positive-path behavior and explicit unsupported hook boundaries,
- no parser surface claimed by this wave remains vaguely “partial”.

### wave_psp_struct_2: Collections and CLI Class-Surface Expansion

Scope:

- `collections`
- `argparse`

Definition of done:

- constructor and parser feature parity expands beyond the current narrowed closure subset,
- remaining dynamic-only families are explicitly waived,
- docs and ledgers reflect the real class/object boundary.

### wave_psp_struct_3: UUID and Datetime Expansion

Scope:

- `uuid`
- `datetime`

Definition of done:

- the approved `uuid` generation and namespace expansion is shipped or explicitly waived,
- the fixed-offset timezone tranche for `datetime` is shipped or explicitly waived,
- typed safety remains the governing adaptation path.

### wave_psp_struct_4: Text-Surface Polish and Governance Closure

Scope:

- `textwrap`
- `html`
- final ledger cleanup for the whole phase

Definition of done:

- the remaining low-risk formatter/class polish surfaces are closed or sharply waived,
- all affected waiver entries are updated,
- no `open` surface remains for this phase’s owned modules.

## CPython Test Porting Targets

- `Lib/test/test_json/`
- `Lib/test/test_configparser.py`
- `Lib/test/test_csv.py`
- `Lib/test/test_collections.py`
- `Lib/test/test_argparse.py`
- `Lib/test/test_uuid.py`
- `Lib/test/test_datetime.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

Each reviewed family must end in:

- `adopted`
- `adapted`
- `waived`

`waived` requires explicit rationale tied to:

- `intentional-diff`
- `unsupported`
- `host-limited`
- `cpython-implementation-detail`

## Quality Contract

- No user-triggerable panic paths are introduced.
- No claimed parity closure may rely on undocumented helper-only APIs.
- Dynamic JSON hooks are not allowed in this phase.
- Fixed-offset timezone semantics are the only approved timezone model in this phase.
- Every wave must update the canonical waiver ledgers before merge.
- No wave is complete if it expands features without shrinking or clarifying the waiver inventory.

## Architecture Lock Validation

Before `wave_psp_struct_1` begins implementation, the phase must add:

- one Sifr demo covering the chosen JSON wrapper model,
- one Sifr demo covering the chosen fixed-offset datetime model,
- one negative-path test for every newly explicit permanent divergence,
- one CPython-family mapping table proving which upstream cases are adopted, adapted, or permanently waived,
- explicit phase test families covering `test_json`, `test_configparser`, `test_csv`, `test_collections`, `test_argparse`, `test_uuid`, `test_datetime`, and `test_textwrap`,
- one compile-time rejection or negative runtime case for every new typed surface that proves the remaining Sifr-safe divergence is explicit rather than accidental.

## Local Validation Commands

- quick gate:
  - `scripts/run_all_tests.sh --profile quick`
- full gate:
  - `scripts/run_all_tests.sh`
- targeted:
  - `cargo test -p sifr -- <test_name>`
  - `cargo run -q -p sifr -- run demos/<structured-parity-demo>.sifr`

## Exit Gate

This phase is complete only when:

- the targeted structured-data and class-surface waivers have been materially reduced,
- every owned module above has updated CPython traceability and waiver accounting,
- the full validation suite is green,
- external review confirms production-grade closure for the documented scope.

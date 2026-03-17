# Ad Hoc Phase: Structured Data and Class-Surface Parity Expansion

Status: open (documented 2026-03-17)
Context: follow-up phase after iterator architecture and iterator-waiver reduction
Execution readiness: planning-ready after `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`

## Objective

Expand CPython parity for the remaining high-value structured-data, parser, and class-surface gaps in modules that are already `parity-closed` for their shipped subset but still carry explicit waiver debt.

This phase is not about reopening all parser and class-heavy modules from zero. It is about consuming the next tranche of explicit residual waivers where the current architecture is already strong enough to support them without fallback-heavy design.

Primary module targets:

- `json`
- `configparser`
- `csv`
- `collections`
- `argparse`
- `uuid`
- `datetime`

Secondary polish targets that fit the same class/object and structured-value pattern:

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

The remaining gaps in these modules are no longer about raw module absence. They are mostly concentrated in:

- richer class/object surfaces,
- constructor and option-matrix expansion,
- structured streaming or wrapper behavior,
- dynamic but still bounded parser features,
- and cleanup of intentionally narrowed class families.

That is a different root-cause family from iterator architecture, runtime/file-object parity, or stateful RNG parity. It should therefore execute as its own phase.

## Depends on

- `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - iterator-returning APIs should already be settled so parser and collection surfaces do not build on stale eager behavior
- milestone-7 canonical closure inventory remains the baseline
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Scope

This phase owns:

- `json` callback-safe parity expansion where dynamic hook support can be modeled safely,
- `configparser` interpolation, proxy, and write-back parity expansion,
- `csv` streaming, dialect-registry, and reader lifecycle parity,
- `collections` constructor and object-model parity beyond the current narrowed subset,
- `argparse` advanced parser features such as `subparsers` and selected `nargs` matrices,
- `uuid` non-v4 generation families and stricter constructor parity where architecture allows,
- `datetime` timezone-aware and richer class-family parity,
- `textwrap` advanced `TextWrapper` option matrices,
- `html` class/package-adjacent parity only where it is still in-scope for top-level `html`.

This phase does not own:

- async iterators,
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
- merging runtime/file-object work into the same implementation track.

## Priority Targets

### priority_1: Structured parser parity

Modules:

- `json`
- `configparser`
- `csv`

Required closure direction:

- reduce `json` callback/format-control waivers only where hooks can be typed and deterministic,
- close `configparser` interpolation/proxy/write-back families deliberately,
- convert `csv` from eager-only reader lifecycle to the richer shipped reader/dialect model that CPython source expects, without sacrificing determinism.

### priority_2: Class-heavy compatibility expansion

Modules:

- `collections`
- `argparse`
- `uuid`
- `datetime`

Required closure direction:

- broaden constructor parity for `Counter`, `defaultdict`, and related collection wrappers,
- add `argparse` advanced parser features with deterministic typed behavior,
- close selected `uuid` non-v4 generation and constructor gaps,
- expand `datetime` timezone-aware and related class-family coverage.

### priority_3: Text and formatter polish

Modules:

- `textwrap`
- `html`

Required closure direction:

- close the remaining `TextWrapper` option matrix that is already adjacent to the shipped surface,
- only expand `html` where the top-level module still has a meaningful parity gap rather than inventing a broader package roadmap here.

## Waves

### wave_psp_struct_1: Parser and Serialization Surface Expansion

Scope:

- `json`
- `configparser`
- `csv`

Definition of done:

- the next tranche of parser and serializer waivers is either closed or reclassified with sharper rationale,
- CPython-derived tests cover both positive-path behavior and unsupported hook boundaries,
- no parser surface claimed by this wave remains vaguely “partial”.

### wave_psp_struct_2: Collections and CLI Class-Surface Expansion

Scope:

- `collections`
- `argparse`

Definition of done:

- constructor and parser feature parity expands beyond the current narrowed closure subset,
- remaining dynamic-only or reflection-heavy families are explicitly waived,
- docs and ledgers reflect the real class/object boundary.

### wave_psp_struct_3: UUID and Datetime Expansion

Scope:

- `uuid`
- `datetime`

Definition of done:

- the approved `uuid` generation and constructor expansion is shipped or explicitly waived,
- the next timezone/class-family tranche for `datetime` is shipped or explicitly waived,
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
- Dynamic hook or callback families are allowed only when their typing and failure semantics are explicit and deterministic.
- Every wave must update the canonical waiver ledgers before merge.
- No wave is complete if it expands features without shrinking or clarifying the waiver inventory.

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

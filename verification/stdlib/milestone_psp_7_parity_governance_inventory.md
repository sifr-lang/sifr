# `milestone_psp_7` Parity Governance Inventory

Status: complete  
Phase: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`  
Execution ledger: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md`

This is the canonical closure inventory for phase 31.5 milestone 7.

## Canonical Builtin Parity Inventory

Terminal state legend for this milestone:
- `parity-closed`: shipped, traceable, and governed (including explicit adapted semantics)
- `intentional-diff`: intentionally divergent from CPython behavior with explicit rationale
- `unsupported`: intentionally not shipped in this phase
- `host-limited`: behavior depends on host/runtime boundaries

| Builtin surface | Terminal state | Evidence | Notes |
| --- | --- | --- | --- |
| `list(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Constructor and iterable forms closed. |
| `tuple(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Fixed-size typed tuple semantics preserved. |
| `dict(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Mapping and iterable-of-pairs entry forms closed. |
| `set(...)` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Core constructor/method object model closed. |
| `str(...)` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Constructor and method parity for shipped surface closed. |
| `ord(...)` / `chr(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Dynamic safety uses typed results where CPython raises. |
| `int(...)` / `float(...)` conversion paths | `intentional-diff` | `internal_docs/architecture.md` | Typed `Result` adaptation is a core Sifr safety contract. |
| `len(...)` / `abs(...)` / `min(...)` / `max(...)` / `sum(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Closed in builtin parity waves with typed-safe semantics. |
| `sorted(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Keyword/binding parity closed for shipped call shapes. |
| `reversed(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Iterator-returning contract is closed; eager materialization is explicit (`list(...)`). |
| `enumerate(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Positional/keyword start forms closed. |
| `zip(...)` / `map(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Base variadic forms close on iterator-returning behavior; strict-mode waivers remain explicit. |
| `range(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Keyword normalization is an intentional adaptation. |
| `any(...)` / `all(...)` | `parity-closed` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Closed for shipped iterable surface. |

## Canonical Core Object-Model Inventory

| Object model | Terminal state | Evidence | Notes |
| --- | --- | --- | --- |
| `list` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Index/pop/mutation and optional-bound forms closed for shipped typed behavior. |
| `dict` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Update/get/pop typed semantics closed with explicit compile-time rejection paths. |
| `set` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Variadic update/intersection/difference_update object model closed. |
| `tuple` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Count/index optional-bound behavior closed with typed-safe miss handling. |
| `str` | `parity-closed` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Split/replace and bound typing closed for shipped surface. |
| `bytes` (custom shipped surface) | `intentional-diff` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Classified custom utility surface; no first-class CPython `bytes` object model claim. |

## Per-Module Closure Inventory (`lib/sifr`)

| Module | Closure wave | Terminal state | Evidence |
| --- | --- | --- | --- |
| `argparse` | `wave_psp_e2` | `parity-closed` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `base64` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `bisect` | `wave_psp_b1` | `parity-closed` | `verification/stdlib/wave_psp_b1_cpython_traceability.md` |
| `bytes` | `wave_psp_a2` | `intentional-diff` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` |
| `calendar` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `collections` | `wave_psp_b1` | `parity-closed` | `verification/stdlib/wave_psp_b1_cpython_traceability.md` |
| `configparser` | `wave_psp_c1` | `parity-closed` | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| `csv` | `wave_psp_c1` | `parity-closed` | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| `datetime` | `wave_psp_e1` | `parity-closed` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| `difflib` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `env` | `wave_psp_d2` | `intentional-diff` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `fnmatch` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `functools` | `wave_psp_b2` | `parity-closed` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `glob` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `graphlib` | `wave_psp_e2` | `parity-closed` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `gzip` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `hashlib` | `wave_psp_e1` | `parity-closed` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| `heapq` | `wave_psp_b1` | `parity-closed` | `verification/stdlib/wave_psp_b1_cpython_traceability.md` |
| `html` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `io` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `ipaddress` | `wave_psp_e2` | `parity-closed` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `itertools` | `wave_psp_b2` | `parity-closed` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `json` | `wave_psp_c1` | `parity-closed` | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| `logging` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `math` | `wave_psp_e1` | `parity-closed` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| `operator` | `wave_psp_b2` | `parity-closed` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `os` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `pathlib` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `platform` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `random` | `wave_psp_b2` | `parity-closed` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `re` | `wave_psp_e1` | `parity-closed` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| `secrets` | `wave_psp_b2` | `host-limited` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `shutil` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `statistics` | `wave_psp_e1` | `parity-closed` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| `string` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `subprocess` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `sys` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `tempfile` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| `test` | `wave_psp_e2` | `intentional-diff` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `textwrap` | `wave_psp_c2` | `parity-closed` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| `time` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `timeit` | `wave_psp_d2` | `host-limited` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| `tomllib` | `wave_psp_c1` | `parity-closed` | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| `uuid` | `wave_psp_e2` | `parity-closed` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `zipfile` | `wave_psp_d1` | `parity-closed` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |

## Canonical CPython Adopt/Adapt/Waive Ledger (By Wave)

| Wave | Canonical source | Summary |
| --- | --- | --- |
| `wave_psp_a1` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | Builtin constructors/call-shape closure with explicit `strict` waivers for `zip`/`map` and tuple dynamic-shape waiver. |
| `wave_psp_a2` | `verification/stdlib/wave_psp_a2_cpython_traceability.md` | Core object-model closure via adapted semantics and explicit `bytes`/`bytearray` unsupported classification. |
| `wave_psp_b1` | `verification/stdlib/wave_psp_b1_cpython_traceability.md` | Collections/bisect/heapq closure with constructor and helper-family unsupported waivers. |
| `wave_psp_b2` | `verification/stdlib/wave_psp_b2_cpython_traceability.md` | Iterator/functional/random closure; ad-hoc iterator architecture phase now closes core lazy protocol surfaces and classifies retained advanced itertools families explicitly. |
| `wave_psp_c1` | `verification/stdlib/wave_psp_c1_cpython_traceability.md` | Structured parser/module closure with callback-hook/interpolation unsupported waivers. |
| `wave_psp_c2` | `verification/stdlib/wave_psp_c2_cpython_traceability.md` | Text/pattern/module closure with explicit advanced-formatting/locale-class waivers. |
| `wave_psp_d1` | `verification/stdlib/wave_psp_d1_cpython_traceability.md` | Filesystem/archive closure with stream hierarchy and advanced archive/class-family unsupported waivers. |
| `wave_psp_d2` | `verification/stdlib/wave_psp_d2_cpython_traceability.md` | Process/runtime/platform closure with explicit async process and interpreter-mutation waivers. |
| `wave_psp_e1` | `verification/stdlib/wave_psp_e1_cpython_traceability.md` | Strong-core module closure with timezone/capture-object/extended-crypto unsupported waivers. |
| `wave_psp_e2` | `verification/stdlib/wave_psp_e2_cpython_traceability.md` | Class-heavy cleanup closure with explicit constructor and incremental-frontier intentional diffs. |

## Waiver Index (`intentional-diff`, `unsupported`, `host-limited`)

Owner baseline for every entry below: phase owner in `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md`  
Canonical issue for revisit tracking: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`

| Surface | Terminal state | Rationale | Revisit rule | Evidence |
| --- | --- | --- | --- | --- |
| Builtin numeric parsing (`int(str)`, `float(str)`) typed-result behavior | `intentional-diff` | Sifr safety contract uses `Result`/`Option` instead of exceptions. | Revisit only with language-contract change approval. | `internal_docs/architecture.md` |
| `bytes` as CPython object-model equivalent | `intentional-diff` | Shipped as custom utility surface, not first-class CPython bytes object parity. | Revisit when first-class bytes type lands. | `verification/stdlib/wave_psp_a2_cpython_traceability.md` |
| Advanced iterator-object/lazy parity families (`itertools` combinators beyond `chain`/`repeat`/`islice`/`count`) | `intentional-diff` | Core iterator architecture is closed; retained combinator families remain explicitly list-backed in this phase. | Revisit only in a future explicitly scoped iterator-expansion phase. | `verification/stdlib/wave_psp_b2_cpython_traceability.md`, `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-execution.md` |
| `functools.partial`/wrapper-family parity | `unsupported` | Requires broader callable-wrapper typing and object runtime support. | Revisit with callable-object typing milestone. | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| Reflective `operator` factories (`attrgetter`, `methodcaller`) | `unsupported` | Static object model intentionally excludes string-driven reflective dispatch. | Revisit only with explicit reflection policy change. | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| Weighted/stateful random families (`choices(weights=...)`, `seed/getstate/setstate`) | `unsupported` | Current randomness layer intentionally avoids deterministic mutable RNG object model. | Revisit with RNG-state architecture milestone. | `verification/stdlib/wave_psp_b2_cpython_traceability.md` |
| `json`/`tomllib` dynamic decode-hook callback matrices | `unsupported` | Typed parsing path intentionally excludes dynamic callback injection. | Revisit with callback-safe typed hook design. | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| `configparser` interpolation/proxy/write-back parity family | `unsupported` | Current closure targets parser/object/error core only. | Revisit with class/proxy expansion milestone. | `verification/stdlib/wave_psp_c1_cpython_traceability.md` |
| Advanced `string`/`textwrap` formatter option matrices | `unsupported` | Complex formatting matrix intentionally reduced in adapted surface. | Revisit with text formatting expansion wave. | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| Advanced `difflib`/`calendar` class families | `unsupported` | Wave c2 closes high-value helpers/classes only. | Revisit with module-family expansion milestone. | `verification/stdlib/wave_psp_c2_cpython_traceability.md` |
| Rich `io` stream hierarchy and in-memory wrappers | `unsupported` | Current closure targets file-handle parity and typed error boundaries. | Revisit with stream-class hierarchy design. | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| Full `pathlib` class specialization and URI/device semantics | `unsupported` | Single portable `Path` class is the current safe closure boundary. | Revisit with platform-specific path-type plan. | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| Recursive `glob` keyword matrix (`recursive`, `include_hidden`, etc.) | `unsupported` | Deterministic non-recursive surface intentionally chosen in this phase. | Revisit with deterministic recursive matching policy. | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| Extended `shutil` / `tempfile` object-wrapper families | `unsupported` | Core helper parity closed; object-wrapper lifecycle matrix deferred. | Revisit with filesystem object-model expansion milestone. | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| Advanced `gzip`/`zipfile` file-object and compression-option families | `unsupported` | Core archive behavior closed for shipped safe surface. | Revisit with archive runtime feature expansion. | `verification/stdlib/wave_psp_d1_cpython_traceability.md` |
| Async `subprocess.Popen` lifecycle and full option matrix | `unsupported` | Current closure intentionally targets sync `CompletedProcess` workflow. | Revisit with async runtime milestone (phase 32+). | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| Mutable CPython interpreter hooks in `sys` | `unsupported` | Deterministic compiler/runtime model excludes interpreter mutation hooks. | Revisit only with explicit runtime-mutability policy change. | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| Full `logging` config hierarchy parity | `host-limited` | Host/runtime integration is intentionally lightweight and synchronous. | Revisit with runtime logging subsystem expansion. | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| Rich `time`/`timeit` object model parity | `host-limited` | Functional timer surface closed; dynamic eval/object families deferred. | Revisit with runtime host-abstraction expansion. | `verification/stdlib/wave_psp_d2_cpython_traceability.md` |
| Timezone/capture-object/extended-crypto families (`datetime`, `re`, `hashlib`) | `unsupported` | Strong shipped core is closed; advanced class-family matrices are explicitly deferred. | Revisit with targeted module-expansion milestones. | `verification/stdlib/wave_psp_e1_cpython_traceability.md` |
| Strict raising direct constructor parity in `ipaddress`/`uuid` (`IPv4Address(...)`, `UUID(...)`) | `intentional-diff` | Factory APIs provide typed validation; direct constructors remain pass-through under current constructor-lowering constraints. | Revisit with constructor-lowering architecture changes. | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `graphlib` multi-node frontier incremental semantics | `intentional-diff` | Current API uses deterministic one-node `get_ready()` progression. | Revisit with broader graph object-model parity expansion. | `verification/stdlib/wave_psp_e2_cpython_traceability.md` |
| `env` and `test` as CPython module-equivalent claims | `intentional-diff` | Both are shipped custom infrastructure surfaces, not one-to-one CPython modules. | Revisit only if module contract is redefined. | `verification/stdlib/wave_psp_d2_cpython_traceability.md`, `verification/stdlib/wave_psp_e2_cpython_traceability.md` |

## Exit-Gate Closure Summary (Milestone 7)

- Canonical inventories are now centralized in this document for builtins, core object models, and all shipped `lib/sifr` modules.
- Every shipped module is assigned a terminal governance state (`parity-closed`, `intentional-diff`, or `host-limited`).
- The per-wave CPython traceability corpus is canonically linked for all ten closure waves (`wave_psp_a1` through `wave_psp_e2`).
- All residual non-parity surfaces are explicit in the waiver index with rationale and revisit rules.
- No `open` parity state is carried in this milestone inventory.

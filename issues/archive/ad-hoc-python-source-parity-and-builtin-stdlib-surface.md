# Ad Hoc Phase: Python Source Parity and CPython Surface Closure

Status: complete
Phase placement: ad hoc interstitial phase between Phase 31 and Phase 32
Phase owner: Codex (GPT-5), tracked in `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md`

## Objective

Drive Sifr to maximal CPython parity for:

- builtins
- core container and string object models
- every existing module currently shipped in `lib/sifr`

This phase is not about adding a large number of new module names. It is about closing the gap between:

- current subset parity from Phase 30
- corpus-driven compatibility fixes from Phase 31
- the product requirement that Python-shaped source should compile naturally for the surfaces Sifr already claims to support

The target is:

1. supported Python-shaped source compiles directly without workaround-first APIs
2. top-level exported surfaces for existing `lib/sifr` modules are brought as close to CPython as possible
3. major class/object-model surfaces are completed where architecture already permits them
4. intentional divergences remain explicit, typed, safety-aligned, and documented

## Source of Truth

This phase must use the following inputs as authoritative references:

- CPython source tree:
  - `/Users/yaseralnajjar/work/sifr/cpython`
- CPython test corpus:
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`
  - C-backed and runtime-adjacent test coverage under:
    - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_capi`
    - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_free_threading`
- Current module-by-module parity audit:
  - [stdlib_gaps_cpython_module_by_module_audit_2026-03-14.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/issues/stdlib_gaps_cpython_module_by_module_audit_2026-03-14.md)
- Existing architectural and parity baseline:
  - [architecture.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/internal_docs/architecture.md)
  - [30_reliability_parity_and_performance_budgets.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/internal_docs/phases/30_reliability_parity_and_performance_budgets.md)
  - [phase30_parity_matrix.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/verification/stdlib/phase30_parity_matrix.md)
  - [31_algorithmic_compatibility_and_leetcode_coverage.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md)

Path note:

- Absolute CPython paths in this document are intentional and workspace-authoritative for this planning cycle.
- Relative references such as `Lib/test/test_list.py` are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

## Why This Needs Its Own Phase

Phase 30 proved approved stdlib subsets. Phase 31 proved that real Python-shaped source still fails because the repo is missing the last compatibility layer:

- constructor-entry parity
- call-shape parity
- optional-argument parity
- class/object-model parity
- structured return-shape parity
- retirement of workaround-only APIs as the primary ergonomic path

The 2026-03-14 CPython audit confirms that the main problem is no longer "stdlib modules do not exist". The dominant missing layers are:

1. builtin constructor parity
2. class/object-model parity
3. optional-argument parity
4. structured return-type parity
5. iterator/lazy object-model parity
6. wrapper cleanup where Sifr-specific helper names still stand in for natural CPython entry surfaces

That is phase-sized work. Continuing to discover it through LeetCode or OSS corpora is the wrong execution model.

## Depends on

- [31_algorithmic_compatibility_and_leetcode_coverage.md](/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md)
- Phase 30 parity matrix and approved subset decisions remain the baseline to expand from rather than restart from zero.
- Phase 27 exit gate is treated as satisfied and its non-regression invariants remain mandatory for every milestone in this phase.

## Recommended Placement

- Depends on: Phase 31 completion
- Recommended execution point: execute as an interstitial ad hoc phase after Phase 31 and before the currently planned Phase 32 track if Python parity remains a near-term product claim
- Rationale: this work reduces downstream churn across algorithmic compatibility, OSS validation, docs, and future ecosystem phases

## Full-Parity Target

The closure target for this phase is broader than "make current demos pass" and narrower than "copy CPython blindly".

For builtins and every existing module in `lib/sifr`, the target state is:

1. all major top-level CPython entry points are implemented where compatible with Sifr's safety model
2. major classes and object-model methods are implemented where the runtime and type system already support them
3. common CPython call shapes, optional arguments, and constructor forms compile directly
4. workaround-only API names stop being the primary documented or ergonomic path
5. every remaining gap is classified as one of:
   - `intentional-diff`
   - `unsupported`
   - `host-limited`
6. no remaining `open` gap is allowed at phase exit without an explicit owner, rationale, and issue

This phase should therefore aim for "full parity as much as possible" under the existing language contract, not just another approved subset.

## Scope Boundary

In scope:

- builtins and builtin helper parity needed for natural Python-shaped source
- core container and string object-model parity
- closure of every module already shipped under `lib/sifr`
- classification and documentation of every remaining non-parity surface

Out of scope:

- adding a large new tier of modules not already shipped in `lib/sifr`
- changing Sifr's core safety contract to mimic CPython failure behavior
- leaving workaround-first APIs as the steady-state answer where direct parity is feasible

## Non-goals

- copying CPython behavior where it violates Sifr safety guarantees
- reintroducing exception-driven control flow
- hiding ownership or mutability semantics behind Python-compatibility shims
- adding fallback or duplicate workaround APIs as the final user-facing answer
- claiming parity for modules not currently shipped in `lib/sifr`
- bundling unrelated new ecosystem expansion into this phase

## Planning Principles

### 1. Root-Cause First, Not Module-First

This phase must execute top-down. Start with the infrastructure defects that cause the same parity gap to recur across many modules, then move outward into module closure.

### 2. Existing Modules Must Be Closed Deliberately

Every module already present under `lib/sifr` must be reviewed against CPython and assigned to an explicit closure wave. No existing shipped module may remain in a vague "partial parity someday" state at phase exit.

### 3. Workaround APIs Are Transitional

If a CPython-shaped source form is architecturally compatible with Sifr, that source form should become the primary path. Helper surfaces like `from_list(...)`, `json_dumps`, `run_command`, `move_file`, and similar compatibility detours should not remain the main answer when natural parity is feasible.

### 4. Divergences Must Stay Explicit

This phase should increase parity without weakening Sifr's contract.

- `int(str)` remains `Result[int, ParseError]`
- `float(str)` remains `Result[float, ParseError]`
- `Result` / `Option` remain the adaptation path where CPython would raise
- compile-time rejection remains preferable to runtime rejection where possible
- no user-triggerable panics are allowed

### 5. CPython Tests Define Hardness, Sifr Defines Adaptation

Each wave must begin by reading the relevant CPython tests for the builtins and modules in scope. The job is not to copy CPython blindly; it is to inherit the same behavioral hardness and then adapt expectations where Sifr intentionally diverges.

- If a CPython test expresses behavior that is compatible with Sifr's model, port it or derive a direct Sifr equivalent.
- If a CPython test depends on exceptions, dynamic typing, runtime mutability, implementation-specific refcounting, or host behavior that Sifr intentionally rejects, keep the hardness but adapt the assertion to Sifr's typed result, compile-time rejection, or `host-limited` classification.
- If a CPython test exercises a surface Sifr will not support, record an explicit waiver rather than silently dropping the case.

## Parity Accounting Model

The phase should not track parity as a vague per-module adjective. Every builtin and every shipped module must be measured across the same review dimensions:

1. top-level entry surfaces
   - functions
   - constructors
   - aliases
2. call-shape parity
   - positional arguments
   - keyword arguments
   - default arguments
   - variadic forms
3. class and object-model parity
   - public classes
   - methods
   - instance behavior
   - iterator behavior
4. exported constants and error types
5. structured return-shape parity
   - tuples
   - objects
   - iterators
   - typed structured values
6. semantic parity and safety adaptation
   - Sifr-safe divergence
   - panic-free behavior
   - compile-time rejection where required

Each tracked surface must end in exactly one state:

- `done`
- `intentional-diff`
- `unsupported`
- `host-limited`
- `open`

`open` is allowed during implementation only. It is not allowed at phase exit.

For this phase, `done` is not a percentage threshold. A surface is `done` only when:

- major top-level exports in scope are implemented or explicitly waived
- major public classes and object-model behavior in scope are implemented or explicitly waived
- common CPython call shapes in scope are implemented or explicitly waived
- local regression coverage exists with traceability to the relevant CPython test families
- no undocumented parity gap remains for that surface

Each upstream CPython test or test family reviewed for this phase must also end in exactly one state:

- `adopted`
- `adapted`
- `waived`

`waived` requires explicit rationale tied to one of:

- `intentional-diff`
- `unsupported`
- `host-limited`
- `cpython-implementation-detail`

The phase should therefore close both surface parity and test-parity accounting.

## CPython Test Harvesting Contract

Every milestone and closure wave must produce a reviewable upstream-test inventory before implementation starts.

For each builtin or module in scope:

1. identify the relevant CPython test files, subpackages, and high-value fixtures
2. classify the upstream cases into:
   - direct-port candidates
   - Sifr-adaptation candidates
   - explicit waivers
3. port or derive a representative Sifr regression corpus that preserves the same hardness for:
   - happy-path behavior
   - boundary conditions
   - error behavior
   - deterministic behavior
4. record the mapping between:
   - CPython test family
   - Sifr parity surface
   - resulting local regression location
   - final adopt/adapt/waive state

Minimum bar per wave:

- at least one positive-path and one negative-path case must be traceable back to the relevant CPython tests for every builtin or module closed in that wave
- parser-heavy or data-heavy modules should import larger upstream fixture families where practical rather than rewriting tiny hand-picked examples
- no wave is complete if it claims parity without an upstream test review matrix

The goal is not literal test-suite duplication. The goal is CPython-grade behavioral hardness with Sifr-native semantics.

## Root-Cause Stack

This is the dependency order that should drive the entire phase.

### root_cause_1: Callable, Constructor, and Signature Lowering

Missing parity repeatedly comes from incomplete callable lowering:

- missing builtin constructors: `list`, `tuple`, `dict`, `ord`, `chr`
- incomplete builtin call shapes: `enumerate(start)`, `zip(*iters)`, `sorted(key=..., reverse=...)`
- incomplete module call shapes: `bisect(..., lo, hi, key)`, `defaultdict(factory[, initial])`, `json.dump/load/dumps`, `tempfile` option matrices
- incomplete keyword/default/variadic handling

This layer must be fixed before broad module closure. Otherwise downstream module work will keep rediscovering the same signature-lowering gap.

### root_cause_2: Iterable, Container-Conversion, and Structured Return Infrastructure

Many parity gaps depend on treating Python iterables and structured returns more naturally:

- `list(iterable)`, `tuple(iterable)`, `dict(iterable-of-pairs)`
- iterable-compatible builtins rather than list-only lowering
- tuple- and object-shaped returns instead of list/string adapters
- lazy-iterator parity for `itertools` and builtin helpers
- structured parse outputs for `json` and `tomllib`

This is the second root layer. Without it, module implementations will remain workaround-heavy even if the names exist.

### root_cause_3: Class, Object-Model, Error, Constant, and Export Infrastructure

The weakest modules in the audit are mostly class-heavy:

- `argparse`
- `io`
- `subprocess`
- `zipfile`
- `configparser`
- `ipaddress`
- `logging`
- `graphlib`

They need a stronger standard-library class surface:

- class construction and method parity
- exported error types
- exported constants
- richer object return types
- class-family / hierarchy support where appropriate

### root_cause_4: Module-Specific Semantic Closure

Only after the three layers above are stabilized should the phase move fully into module-by-module closure sub-waves. At that point the remaining work should mostly be direct semantics, not infrastructure rediscovery.

## Sequencing Note

This phase is strictly sequential, not a parallel track.

- Milestones are completed in numeric order.
- Closure waves start only after their prerequisite root-cause milestones are merged and locally validated.
- No downstream module may claim completion while depending on an upstream `open` parity surface.

## Execution Model

- This phase remains a single sequential phase.
- Work is grouped into dependency-ordered milestones and reviewable closure sub-waves.
- Only one root-cause layer or one closure sub-wave may be in active implementation at a time.
- No later sub-wave starts before the current sub-wave has:
  - implemented the targeted root-cause or module closure
  - completed the CPython test inventory and adopt/adapt/waive matrix for the surfaces in scope
  - added regression coverage
  - updated the parity inventory
  - passed local validation
  - completed review
- No module is declared complete when it merely has "useful subset behavior". A module is complete only when:
  - major CPython top-level exports are implemented or explicitly waived
  - major class/object-model surfaces are implemented or explicitly waived
  - major constructor and call-shape parity is implemented or explicitly waived
  - every remaining gap is classified

## Milestone-to-Wave Mapping

The phase uses milestones for dependency-ordered closure logic and sub-waves for execution grouping. Their mapping is:

| milestone | primary sub-waves | purpose |
| --- | --- | --- |
| `milestone_psp_1` | `wave_psp_a1` | builtin constructor and callable/signature lowering closure |
| `milestone_psp_2` | `wave_psp_a2` | core container and string object-model closure |
| `milestone_psp_3` | `wave_psp_b1`, `wave_psp_b2` | collections, iterators, and functional module closure |
| `milestone_psp_4` | `wave_psp_c1`, `wave_psp_c2` | structured data, text, and parsing module closure |
| `milestone_psp_5` | `wave_psp_d1`, `wave_psp_d2` | runtime, filesystem, process, and platform module closure |
| `milestone_psp_6` | `wave_psp_e1`, `wave_psp_e2` | remaining shipped-module cleanup and strong-but-incomplete module closure |
| `milestone_psp_7` | `wave_psp_a1` through `wave_psp_e2` | parity inventory, waiver governance, and final exit closure across all prior sub-waves |

## Milestones

### milestone_psp_1: Builtin and Signature-Lowering Architecture

Scope:

- close builtin constructor-entry parity for:
  - `list(...)`
  - `tuple(...)`
  - `dict(...)`
  - `set(...)` consistency cleanup
  - `ord(...)`
  - `chr(...)`
- expand builtin helper call-shape parity for:
  - `sorted`
  - `reversed`
  - `enumerate`
  - `zip`
  - `map`
  - `range`
- establish the canonical support rules for:
  - positional arguments
  - keyword arguments
  - default arguments
  - variadic argument surfaces
  - callable aliases imported from stdlib modules
- unify call lowering so parity expansion does not require one-off compat aliases for every affected module

Definition of done:

- missing builtin constructors are implemented or explicitly waived
- builtin helper signatures cover the approved CPython call-shape matrix
- keyword/default/variadic behavior is stable enough to support downstream module closure without special-case hacks

### milestone_psp_2: Core Container and String Object-Model Closure

Scope:

- close the Python-shaped object-model surface for:
  - `list`
  - `dict`
  - `set`
  - `tuple`
  - `str`
- ensure constructor parity and method parity are coherent
- add missing method overloads and optional-argument forms where architecturally compatible
- verify hashability, membership, slicing, indexing, copying, and mutation behavior against CPython intent with Sifr-safe adaptation

Definition of done:

- core builtins no longer require workaround APIs for ordinary Python source
- remaining differences in container or string behavior are explicit intentional divergences rather than incidental gaps

### milestone_psp_3: Collections, Iterator, and Functional Surface Closure

Scope:

- close constructor and object-model parity for:
  - `collections.Counter`
  - `collections.defaultdict`
  - `collections.deque`
- close iterator and helper parity for:
  - `itertools`
  - `functools`
  - `operator`
  - `bisect`
  - `heapq`
  - `random`
  - `secrets`
- prioritize removal of workaround-first APIs and eager/list-only stand-ins where direct parity is feasible

Definition of done:

- Python-shaped constructor and helper surfaces for the collections/iterator family work directly
- remaining iterator laziness gaps or callable-surface gaps are explicitly classified

### milestone_psp_4: Structured Data, Text, and Parsing Surface Closure

Scope:

- close major parity gaps for:
  - `json`
  - `tomllib`
  - `csv`
  - `configparser`
  - `string`
  - `textwrap`
  - `base64`
  - `html`
  - `difflib`
  - `calendar`
- replace string/list adapters with structured return surfaces where parity requires them
- export missing error types and constants
- complete high-value class surfaces such as:
  - `string.Template`
  - `string.Formatter`
  - `textwrap.TextWrapper`

Definition of done:

- configuration/text/data modules no longer stop at helper-only parity where CPython-shaped classes and returns are the natural public surface

### milestone_psp_5: Runtime, Filesystem, Process, and Platform Closure

Scope:

- close major parity gaps for:
  - `io`
  - `os`
  - `sys`
  - `pathlib`
  - `glob`
  - `shutil`
  - `tempfile`
  - `subprocess`
  - `logging`
  - `platform`
  - `time`
  - `timeit`
  - `gzip`
  - `zipfile`
- prioritize:
  - natural CPython entry names
  - class/object-model parity
  - constants/error exports
  - structured return objects
  - option matrices where host/runtime allows
- explicitly classify host-limited or low-level surfaces rather than leaving them as silent omissions

Definition of done:

- wrapper-heavy modules are closed as far as the host/runtime model reasonably allows
- remaining low-level omissions are deliberate, documented, and issue-linked

### milestone_psp_6: Remaining Existing-Module Closure

Scope:

- close remaining gaps for the current shipped modules not fully closed earlier, especially:
  - `argparse`
  - `ipaddress`
  - `uuid`
  - `graphlib`
  - `datetime`
  - `re`
  - `math`
  - `statistics`
  - `hashlib`
- use this milestone as the cleanup wave for modules that are already strong but still not fully parity-aligned

Definition of done:

- every existing shipped module is either:
  - parity-closed for the approved CPython surface
  - explicitly marked as intentional divergence
  - explicitly marked as unsupported or host-limited with owner and rationale

### milestone_psp_7: Parity Governance and Exit Closure

Scope:

- create one canonical parity inventory for:
  - builtins
  - core object models
  - every module in `lib/sifr`
- require per-surface classification:
  - `done`
  - `intentional-diff`
  - `unsupported`
  - `host-limited`
  - `open`
- require linked owner, rationale, issue, and revisit rule for every non-`done` entry
- update docs and public claims so they match the actual parity state at closure

Definition of done:

- no parity gap remains undocumented
- no shipped module remains in an ambiguous "partial parity" state
- the closure inventory is reviewable enough that future corpus work does not rediscover unknown surface gaps

## Closure Sub-Waves

The milestones above define architecture and scope. Execution inside them should follow these sub-waves.

Custom-surface note:

- `bytes`, `env`, and `test` remain in scope because they are shipped in `lib/sifr`.
- They are not treated as ordinary CPython module-parity targets.
- Their closure obligation in this phase is classification cleanup, claim hygiene, and alignment to the correct CPython-adjacent surface where applicable.

### wave_psp_a1: Builtin Constructors and Callable Surface

- builtins:
  - `list`, `tuple`, `dict`, `set`, `str`, `int`, `float`, `bool`, `ord`, `chr`
  - `len`, `abs`, `min`, `max`, `sum`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `any`, `all`
- focus:
  - constructor-entry parity
  - builtin helper call-shape lowering
  - keyword/default/variadic callable parity

### wave_psp_a2: Core Object Models and Builtin Semantics

- object models:
  - `list`
  - `dict`
  - `set`
  - `tuple`
  - `str`
- custom-surface classification tied to core-type parity:
  - `bytes`
- focus:
  - indexing
  - slicing
  - membership
  - mutation
  - hashability
  - method and overload parity

### wave_psp_b1: Collections Objects and Ordered Helpers

- `collections`
- `bisect`
- `heapq`
- focus:
  - `Counter`
  - `defaultdict`
  - `deque`
  - constructor parity
  - object-model parity
  - ordered search and heap helper parity

### wave_psp_b2: Iterators, Functional Helpers, and Randomness

- `itertools`
- `functools`
- `operator`
- `random`
- `secrets`
- focus:
  - iterator families
  - lazy/eager boundary decisions
  - callable-wrapper parity
  - random/secrets helper and state surfaces

### wave_psp_c1: Structured Parsing and Serialization

- `json`
- `tomllib`
- `csv`
- `configparser`
- focus:
  - structured returns
  - parser and decode behavior
  - encoder/decoder or reader/writer object surfaces
  - reuse of upstream fixture corpora

### wave_psp_c2: Text, Pattern, and Formatting Modules

- `string`
- `textwrap`
- `base64`
- `html`
- `fnmatch`
- `difflib`
- `calendar`
- focus:
  - class-heavy text helpers
  - formatting and pattern semantics
  - helper and constant parity

### wave_psp_d1: Filesystem, Paths, and Archive Surfaces

- `io`
- `pathlib`
- `glob`
- `shutil`
- `tempfile`
- `gzip`
- `zipfile`
- focus:
  - files and streams
  - path and archive object models
  - lifecycle and filesystem semantics

### wave_psp_d2: Process, Runtime, and Platform Surfaces

- `os`
- `env`
- `sys`
- `subprocess`
- `logging`
- `platform`
- `time`
- `timeit`
- focus:
  - process and environment behavior
  - runtime metadata and constants
  - logging hierarchy
  - clocks and platform probes

### wave_psp_e1: Strong-But-Incomplete Core Modules

- `datetime`
- `re`
- `math`
- `statistics`
- `hashlib`
- focus:
  - strong existing modules with remaining return-shape, option, and object-model gaps

### wave_psp_e2: Class-Heavy and Custom Cleanup

- `argparse`
- `ipaddress`
- `uuid`
- `graphlib`
- `test`
- focus:
  - remaining class-heavy shipped modules
  - custom-surface closure hygiene
  - final cleanup of explicit waivers and claim boundaries

## CPython Test Inputs By Sub-Wave

The following upstream test families should be the default harvesting inputs for each sub-wave. These are not the only possible references, but they are the minimum concrete starting set.

### wave_psp_a1: Builtin Constructors and Callable Surface

- builtins and core containers:
  - `Lib/test/test_list.py`
  - `Lib/test/test_dict.py`
  - `Lib/test/test_set.py`
  - `Lib/test/test_tuple.py`
  - `Lib/test/test_str.py`
- lower-level/runtime-adjacent references to mine selectively where useful:
  - `Lib/test/test_capi/test_list.py`
  - `Lib/test/test_capi/test_dict.py`
  - `Lib/test/test_capi/test_set.py`
  - `Lib/test/test_capi/test_tuple.py`
- adaptation rule:
  - emphasize constructor shapes, builtin entry behavior, and helper-call signatures; convert exception-oriented expectations into Sifr-safe typed or compile-time outcomes

### wave_psp_a2: Core Object Models and Builtin Semantics

- builtins and core containers:
  - `Lib/test/test_list.py`
  - `Lib/test/test_dict.py`
  - `Lib/test/test_set.py`
  - `Lib/test/test_tuple.py`
  - `Lib/test/test_str.py`
- lower-level/runtime-adjacent references to mine selectively where useful:
  - `Lib/test/test_capi/test_list.py`
  - `Lib/test/test_capi/test_dict.py`
  - `Lib/test/test_capi/test_set.py`
  - `Lib/test/test_capi/test_tuple.py`
  - `Lib/test/test_free_threading/test_list.py`
  - `Lib/test/test_free_threading/test_dict.py`
  - `Lib/test/test_free_threading/test_set.py`
  - `Lib/test/test_free_threading/test_str.py`
- adaptation rule:
  - keep semantic hardness around constructors, slicing, mutation, membership, hashing, and iteration, but convert exception-oriented expectations into Sifr-safe typed or compile-time outcomes

### wave_psp_b1: Collections Objects and Ordered Helpers

- `Lib/test/test_collections.py`
- `Lib/test/test_bisect.py`
- `Lib/test/test_heapq.py`
- adaptation rule:
  - preserve constructor and object behavior for `collections`, plus ordered-helper boundary conditions for `bisect` and `heapq`

### wave_psp_b2: Iterators, Functional Helpers, and Randomness

- `Lib/test/test_itertools.py`
- `Lib/test/test_functools.py`
- `Lib/test/test_operator.py`
- `Lib/test/test_random.py`
- `Lib/test/test_secrets.py`
- concurrency/implementation-adjacent references to mine selectively:
  - `Lib/test/test_free_threading/test_bisect.py`
  - `Lib/test/test_free_threading/test_heapq.py`
  - `Lib/test/test_free_threading/test_functools.py`
  - `Lib/test/test_free_threading/test_itertools.py`
- adaptation rule:
  - preserve callable-shape, iterator, and algorithmic boundary coverage, but do not port CPython-only laziness or mutability edge expectations without checking Sifr ownership and iterator contracts first

### wave_psp_c1: Structured Parsing and Serialization

- `Lib/test/test_json/`
- `Lib/test/test_tomllib/`
- `Lib/test/test_csv.py`
- `Lib/test/test_configparser.py`
- adaptation rule:
  - reuse upstream data corpora and invalid-input fixtures wherever practical, especially for `json` and `tomllib`; adapt exception assertions into typed decode/parse failures and keep fixture coverage broad rather than anecdotal

### wave_psp_c2: Text, Pattern, and Formatting Modules

- `Lib/test/test_string/test_string.py`
- `Lib/test/test_string/test_templatelib.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_html.py`
- `Lib/test/test_fnmatch.py`
- `Lib/test/test_difflib.py`
- `Lib/test/test_calendar.py`
- adaptation rule:
  - preserve text and pattern edge-case hardness, but adapt exception or implementation-detail behavior to Sifr-safe and reviewable equivalents

### wave_psp_d1: Filesystem, Paths, and Archive Surfaces

- `Lib/test/test_io/`
- `Lib/test/test_pathlib/`
- `Lib/test/test_glob.py`
- `Lib/test/test_shutil.py`
- `Lib/test/test_tempfile.py`
- `Lib/test/test_gzip.py`
- `Lib/test/test_zipfile/`
- adaptation rule:
  - preserve filesystem, path, temp, and archive hardness while keeping host-limited behavior explicit

### wave_psp_d2: Process, Runtime, and Platform Surfaces

- `Lib/test/test_os/`
- `Lib/test/test_sys.py`
- `Lib/test/test_subprocess.py`
- `Lib/test/test_logging.py`
- `Lib/test/test_platform.py`
- `Lib/test/test_time.py`
- `Lib/test/test_timeit.py`
- runtime-adjacent references to mine selectively:
  - `Lib/test/test_capi/test_sys.py`
  - `Lib/test/test_capi/test_time.py`
  - `Lib/test/test_free_threading/test_io.py`
- adaptation rule:
  - separate portable semantics from host-specific behavior early; for host-limited APIs, preserve the same boundary hardness while explicitly waiving or constraining platform-dependent cases
- custom-surface guidance:
  - `env` should not claim a standalone CPython module analogue; use `os` and environment-related test families as the behavioral reference set

### wave_psp_e1: Strong-But-Incomplete Core Modules

- `Lib/test/test_datetime.py`
- `Lib/test/test_re.py`
- `Lib/test/test_math.py`
- `Lib/test/test_statistics.py`
- `Lib/test/test_hashlib.py`
- selective concurrency/runtime references:
  - `Lib/test/test_free_threading/test_re.py`
- adaptation rule:
  - keep the same semantic hardness for regex, datetime, and numeric edge behavior, while preserving Sifr's explicit `Result`/`Option`, ownership, and compile-time rejection contracts

### wave_psp_e2: Class-Heavy and Custom Cleanup

- `Lib/test/test_argparse.py`
- `Lib/test/test_ipaddress.py`
- `Lib/test/test_uuid.py`
- `Lib/test/test_graphlib.py`
- selective concurrency/runtime references:
  - `Lib/test/test_free_threading/test_uuid.py`
- adaptation rule:
  - keep the same semantic hardness for parser and class-heavy module behavior, but preserve Sifr's explicit `Result`/`Option`, ownership, and compile-time rejection contracts
- custom-surface guidance:
  - `test` is Sifr infrastructure and should exit this phase as a classified non-CPython parity surface rather than a faux stdlib parity target

## Module Closure Ledger

Every shipped module in `lib/sifr` must terminate in an explicit closure bucket during this phase.

| module | execution wave | closure target |
| --- | --- | --- |
| `argparse` | `wave_psp_e2` | close object-model parity for `ArgumentParser`-style usage or classify the remaining class-heavy surfaces explicitly |
| `base64` | `wave_psp_c2` | close remaining codec family and signature gaps compatible with Sifr bytes/string policy |
| `bisect` | `wave_psp_b1` | close aliases and optional-argument parity including `lo`/`hi`/`key` where supported |
| `bytes` | `wave_psp_a2` | classify as custom surface and align parity target to CPython `bytes` object-model semantics rather than a fake module parity claim |
| `calendar` | `wave_psp_c2` | close constants, helper functions, and class-family gaps or classify them explicitly |
| `collections` | `wave_psp_b1` | close constructor parity, object-model parity, and remaining public exports or classify gaps explicitly |
| `configparser` | `wave_psp_c1` | close parser class/error/constant parity as far as architecture permits |
| `csv` | `wave_psp_c1` | close reader/writer/dialect/constant parity and remove helper-only limitations |
| `datetime` | `wave_psp_e1` | close remaining constructors, constants, return types, and aware/naive semantics where supported |
| `difflib` | `wave_psp_c2` | close class and helper parity beyond the current narrow helper subset |
| `env` | `wave_psp_d2` | classify as custom surface and fold parity accounting into `os`/environment behavior rather than standalone CPython-module parity |
| `fnmatch` | `wave_psp_c2` | close helper and signature parity for the public pattern-matching surface |
| `functools` | `wave_psp_b2` | close high-value functional parity and callable-wrapper behavior rather than leaving `reduce` as a token subset |
| `glob` | `wave_psp_d1` | close recursive/pathname-expansion signatures and helper parity or classify host-limited gaps |
| `graphlib` | `wave_psp_e2` | close `TopologicalSorter` object-model parity and supporting errors/helpers |
| `gzip` | `wave_psp_d1` | close class/error/open-surface parity or classify unsupported archive semantics explicitly |
| `hashlib` | `wave_psp_e1` | close remaining constructor/result/object semantics and classify crypto-host limits explicitly |
| `heapq` | `wave_psp_b1` | close merge/max-heap/signature semantics and mutation/error behavior |
| `html` | `wave_psp_c2` | close top-level parity and classify any remaining sibling-module boundaries explicitly |
| `io` | `wave_psp_d1` | close stream class hierarchy, buffering objects, and open/handle semantics as far as Sifr's runtime supports |
| `ipaddress` | `wave_psp_e2` | close public constructors, classes, and error types for IPv4/IPv6 parity |
| `itertools` | `wave_psp_b2` | close remaining iterator families and lazy object-model parity instead of eager stand-ins |
| `json` | `wave_psp_c1` | close natural `dump`/`load`/`dumps`/`loads`, structured returns, and encoder/decoder parity |
| `logging` | `wave_psp_d2` | close handler/filter/record hierarchy and root-helper parity or classify deliberate limits |
| `math` | `wave_psp_e1` | close remaining return-shape and naming details and explicitly classify safety divergences |
| `operator` | `wave_psp_b2` | close callable object helpers and naming parity for the public operator surface |
| `os` | `wave_psp_d2` | close environment/path/process entry surfaces and retire wrapper-first names where direct parity is feasible |
| `pathlib` | `wave_psp_d1` | close path class-family semantics and platform-specific gaps or classify them explicitly |
| `platform` | `wave_psp_d2` | close uname/platform helper parity and classify platform-probe limitations |
| `random` | `wave_psp_b2` | close seed/state/class-based APIs and remaining distributions or classify deliberate omissions |
| `re` | `wave_psp_e1` | close remaining top-level helpers, flags, and iterator/match parity |
| `secrets` | `wave_psp_b2` | close token/helper parity and classify crypto-host limitations explicitly |
| `shutil` | `wave_psp_d1` | close natural copy/move/archive/error names and broader filesystem helper parity |
| `statistics` | `wave_psp_e1` | close remaining distribution/class/helper gaps and classify numerical policy differences explicitly |
| `string` | `wave_psp_c2` | close `Template`/`Formatter` class parity and remaining constants/helper gaps |
| `subprocess` | `wave_psp_d2` | close process object/error/constant parity and classify host-limited lifecycle semantics explicitly |
| `sys` | `wave_psp_d2` | close interpreter metadata, streams, flags, and runtime config parity where host/runtime allows |
| `tempfile` | `wave_psp_d1` | close temporary object/class helpers and lifecycle semantics or classify host-limited behavior |
| `test` | `wave_psp_e2` | classify as Sifr-specific infrastructure and remove it from ordinary CPython module parity claims |
| `textwrap` | `wave_psp_c2` | close `TextWrapper` class and option parity for helper functions |
| `time` | `wave_psp_d2` | close clock families, structured returns, constants, and ns variants or classify host limits |
| `timeit` | `wave_psp_d2` | close `Timer` object-model parity and helper signatures |
| `tomllib` | `wave_psp_c1` | close structured returns and error export parity instead of string/error adapters |
| `uuid` | `wave_psp_e2` | close constructor overloads, public helpers, and newer UUID family coverage where approved |
| `zipfile` | `wave_psp_d1` | close archive class/error/constant/path parity or classify unsupported archive features explicitly |

## Current Priority Tiers From The Audit

### Highest Priority Weak Modules

These should be treated as explicit closure targets, not best-effort cleanup:

- `argparse`
- `functools`
- `json`
- `io`
- `ipaddress`
- `operator`
- `secrets`
- `subprocess`
- `sys`
- `tempfile`
- `zipfile`

### Strong But Still Incomplete Modules

These should not be ignored just because they are already useful:

- `math`
- `statistics`
- `re`
- `datetime`
- `hashlib`
- `collections`
- `random`

The audit shows these are among the strongest modules today, but they still contain real CPython parity gaps in constructor shapes, object models, optional arguments, or return structures.

## Intentional Divergences That Must Stay Explicit

This phase must not weaken Sifr's contract.

- `int(str)` remains `Result[int, ParseError]`
- `float(str)` remains `Result[float, ParseError]`
- `Result` / `Option` remain the adaptation path where CPython would raise
- compile-time rejection remains preferable to runtime rejection for invalid ownership, mutability, or hashability patterns
- empty or missing collection behavior remains panic-free
- ownership transfer remains explicit
- low-level host/runtime APIs may still be classified as `host-limited` where parity is not safely portable

## Quality Contract

### Entry criteria

- Phase 31 is complete.
- Phase 30 and Phase 31 evidence is available and treated as the starting baseline rather than reopened blindly.
- The module-by-module CPython audit is the current planning baseline.
- Phase 27 non-regression baseline is green at phase start and must remain green through completion.
- Phase 16 local-first validation platform remains the authoritative execution foundation.

### Phase-wide invariants

- No user-triggerable panic paths.
- No data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths.
- Stable diagnostic contract:
  - codes
  - severity
  - spans
  - URLs
  - suggestions
  - schema
- Canonical and lossless `json` diagnostics remain authoritative.
- `human` and `compact` remain renderer views over the same diagnostic model.
- Recovery ordering remains deterministic.
- Exit-code and CLI contract remain stable.
- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
- All implementations must be production-grade compiler and stdlib work: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards.

### Milestone quality checks

- no partial parity claims without explicit classification
- no module is complete while major top-level CPython exports remain unclassified
- no class-heavy module is complete while major public classes remain missing without waiver
- every root-cause fix includes regression coverage
- every wave includes a CPython test inventory and adopt/adapt/waive matrix for the surfaces in scope
- no CPython-derived parity claim is accepted without a traceable local regression or explicit waiver
- every milestone includes at least one positive-path and one negative-path validation case
- constructor parity work must include positive-path and safe-error-path validation
- optional-argument parity decisions must be documented, not left implicit
- parser-heavy modules must prefer upstream fixture/data reuse where practical instead of ad hoc examples only
- validation evidence must be recorded in the execution checklist issue before merge
- no milestone is complete if its outputs are not reviewable and reproducible locally
- parity-governance outputs must be machine-reviewable and deterministic
- any divergence or waiver must be explicit, time-bounded, owner-assigned, and issue-linked
- if a milestone changes an approved Phase 30 behavior, the change must explicitly classify whether it is:
  - parity expansion
  - compatibility cleanup
  - intentional divergence retained
  - prior waiver retired
- modules or builtins with parsing-heavy, numeric-edge, or panic-risk surfaces must reuse the established property/fuzz machinery where applicable

### Validation planning goals

- `milestone_psp_1` (Builtin and Signature-Lowering Architecture): validation goals cover: builtin constructor closure, builtin helper call-shape closure, keyword/default/variadic lowering stability, parity-safe callable lowering for downstream stdlib work, and a CPython-derived core-type test matrix for `wave_psp_a1`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_2` (Core Container and String Object-Model Closure): validation goals cover: constructor/method coherence for `list`, `dict`, `set`, `tuple`, and `str`; major optional-argument forms; safe adaptation where CPython would raise; and ported or adapted coverage from the relevant CPython core-type tests for `wave_psp_a2`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_3` (Collections, Iterator, and Functional Surface Closure): validation goals cover: Python-shaped parity for `collections`, `itertools`, `functools`, `operator`, `bisect`, `heapq`, `random`, and `secrets`; removal of workaround-first entry surfaces; iterator/object-model parity where supported; and a sub-wave-level CPython test harvest for `wave_psp_b1` and `wave_psp_b2`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_4` (Structured Data, Text, and Parsing Surface Closure): validation goals cover: structured return-shape parity, class exports, constant exports, and call-shape closure for `json`, `tomllib`, `csv`, `configparser`, `string`, `textwrap`, `base64`, `html`, `difflib`, and `calendar`; plus reuse of upstream CPython data/fixture corpora where practical across `wave_psp_c1` and `wave_psp_c2`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_5` (Runtime, Filesystem, Process, and Platform Closure): validation goals cover: object-model parity, constants/errors, host-limited classification, and Python-shaped entry names for `io`, `os`, `sys`, `pathlib`, `glob`, `shutil`, `tempfile`, `subprocess`, `logging`, `platform`, `time`, `timeit`, `gzip`, and `zipfile`; plus explicit CPython test adaptation for portable versus host-bound cases across `wave_psp_d1` and `wave_psp_d2`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_6` (Remaining Existing-Module Closure): validation goals cover: cleanup closure for the remaining shipped modules and the strong-but-incomplete modules; explicit closure or waiver for all remaining surface gaps; and final CPython test-family harvest for `wave_psp_e1` and `wave_psp_e2`. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_7` (Parity Governance and Exit Closure): validation goals cover: one canonical parity inventory for builtins, object models, and all existing modules; explicit classification of every non-closed surface; documentation alignment with actual support; and a canonical adopt/adapt/waive ledger for all reviewed CPython test families. Include negative-path goals that catch regressions against these guarantees.
- Exit-gate evidence explicitly demonstrates: builtins and all existing shipped modules have either maximal parity closure or an explicit, reviewable divergence classification.

### Local validation commands

- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Quick local suite:
  - `scripts/run_all_tests.sh --profile quick`
- Full local suite via workspace path:
  - `scripts/run_all_tests.sh`
- Unit tests only:
  - `cargo test -p sifr -- --skip test_e2e_pass`
- E2E pass suite:
  - `scripts/run_e2e_pass.sh`
- Lint and maintainability guardrails:
  - `cargo clippy --workspace -- -D warnings`
  - `cargo fmt --check`
  - `python3 scripts/check_hir_maintainability_guardrails.py`

## Required Policies

The phase must define and keep current:

- builtin and module parity classification policy
- signature-parity policy for positional, keyword, default, and variadic forms
- structured-return policy for tuple/object/iterator/value surfaces
- intentional-divergence policy for Sifr-safe behavior differences
- host-limited classification policy for runtime/platform-dependent surfaces
- workaround retirement policy for compatibility aliases and helper-first APIs
- parity inventory update policy tied to each merged milestone
- demo and validation evidence policy for parity claims

## Required Artifacts

- canonical builtin parity inventory
- canonical core object-model parity inventory
- per-module closure inventory for every shipped `lib/sifr` module
- per-sub-wave CPython test inventory with adopt/adapt/waive classification
- traceability matrix from CPython test families to local Sifr regression locations
- explicit waiver index for every `intentional-diff`, `unsupported`, and `host-limited` surface
- milestone demos covering major user-visible parity expansions
- validation evidence summary for each milestone
- final exit-gate closure summary mapping shipped surfaces to their terminal classification

Current canonical artifact path for milestone 7 governance closure:

- `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`

### Exit criteria

- All milestone definitions of done are satisfied.
- Supported Python-shaped source for builtins and all existing shipped modules compiles naturally without workaround-first APIs.
- Every existing module in `lib/sifr` is in one of these states:
  - parity-closed
  - intentional divergence
  - unsupported
  - host-limited
- Every reviewed CPython test family for the phase is in one of these states:
  - adopted
  - adapted
  - waived
- No existing shipped module remains with undocumented open parity gaps.
- Intentional divergences remain explicit, typed, panic-free, and documented.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate

Python source parity is production-governed for builtins and all existing shipped stdlib modules: supported builtins, constructors, object models, and module entry surfaces compile naturally; major class/object-model surfaces are closed wherever architecture allows; intentional divergences remain explicit and safety-aligned; and the Phase 27 non-regression contract remains green with deterministic, reviewable validation evidence.

## Recommended First Execution Order

1. `wave_psp_a1` builtin constructors and callable surface
2. `wave_psp_a2` core object models and builtin semantics
3. `wave_psp_b1` collections objects and ordered helpers
4. `wave_psp_b2` iterators, functional helpers, and randomness
5. `wave_psp_c1` structured parsing and serialization
6. `wave_psp_c2` text, pattern, and formatting modules
7. `wave_psp_d1` filesystem, paths, and archive surfaces
8. `wave_psp_d2` process, runtime, and platform surfaces
9. `wave_psp_e1` strong-but-incomplete core modules
10. `wave_psp_e2` class-heavy and custom cleanup
11. `milestone_psp_7` parity governance and exit closure

## Why This Is Better Than Continuing Corpus Discovery

This phase converts a recurring compatibility smell into a deliberate closure program.

- Phase 30 proved subset module behavior.
- Phase 31 proved that real Python-shaped source still exposes the missing compatibility layer.
- The 2026-03-14 audit proves the remaining work is broad, concrete, and already localizable module-by-module.
- A top-down, root-cause-first phase will close more parity with less churn than continuing to rediscover the same structural gaps through external corpora.

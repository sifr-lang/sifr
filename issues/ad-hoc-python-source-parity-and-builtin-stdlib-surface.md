# Ad Hoc Phase: Python Source Parity for Builtins and Stdlib Surface

Status: proposed

## Objective

Close the gap between:

- Phase 30's approved stdlib subset parity
- Phase 31's corpus-driven compatibility fixes
- the broader product requirement that supported Python-shaped source should compile naturally in Sifr

This ad hoc phase is about source compatibility, not raw module-count expansion. The target is:

1. support idiomatic Python source for the parts Sifr already claims to support
2. keep safety, ownership, and type-system divergences explicit
3. stop requiring workaround APIs where Python-shaped source should compile directly

## Why This Needs Its Own Phase

Phase 30 closed approved stdlib subsets module by module. That work was necessary and correct, but it explicitly left many constructor, optional-argument, and object-model surfaces out of scope. The Phase 30 parity matrix records many of these as approved subset boundaries rather than bugs.

Phase 31 then rediscovered part of the same problem from the opposite direction under the `stdlib.python_module_surface` bucket. Several high-value fixes landed there already, including:

- native `set()` / `set(iterable)` lowering
- bare `deque(...)` compatibility resolution
- `defaultdict(...)` compatibility lowering for the current factory subset
- `len(deque)` compatibility

That pattern is now clear enough that continuing to discover the remaining gaps through LeetCode or OSS corpora is the wrong execution model. This is broad enough to deserve a dedicated parity phase with its own scope, milestones, and verification rules.

## Recommended Placement

- Depends on: Phase 31 completion
- Recommended execution point: before Phase 32 if Python-parity remains a near-term product claim
- Rationale: this work is mostly about consolidating already-supported language/runtime surfaces into Python-shaped entry points, and it directly reduces future compatibility churn across algorithmic, OSS, and docs tracks

## Non-goals

- Copying CPython blindly where it conflicts with Sifr safety guarantees
- Claiming full CPython parity for every existing stdlib module
- Reintroducing exception-driven control flow
- Hiding ownership transfer or mutability behind Python-compatible syntax
- Adding fallback or duplicate workaround APIs as the primary user-facing surface

## Current Repo-State Review

The current repo is stronger than the older stdlib audits in several areas, but it still has a real parity gap at the Python-shaped source layer.

### 1. Builtin constructors and conversions

Already present in lowering or compatibility surface:

- `set()`, `set(iterable)`
- `str(...)`
- `int(...)`
- `float(...)`
- `bool(...)`
- `range(...)`

Still missing or incomplete:

- `list(...)` builtin constructor parity is missing
- `tuple(...)` builtin constructor parity is missing
- `dict(...)` builtin constructor parity is missing
- `ord(...)` is missing
- `chr(...)` is missing
- `Counter()` / `Counter(iterable)` / `Counter(mapping)` parity is missing; today the natural path is still `from_list(...)` or direct field-shaped construction
- `defaultdict(factory[, initial])` exists only as a narrow compatibility slice for builtin factories `int`, `list`, and `set`

Current implication:

- supported container types exist in the language
- some supported collection classes exist in stdlib
- but the constructor-entry surface is still incomplete and uneven

### 2. Builtin functional helpers

Present today:

- `len`
- `abs`
- `min`
- `max`
- `sum`
- `sorted`
- `reversed`
- `enumerate`
- `zip`
- `map`
- `range`
- `any`
- `all`

Main remaining parity gaps:

- `sorted(...)` is currently one-argument, eager, and list-backed; no `reverse=` or `key=`
- `reversed(...)` is currently list-only
- `enumerate(...)` currently lacks the `start` argument
- `zip(...)` currently supports exactly two iterables
- `map(...)` currently supports one iterable and lowers to eager list output
- builtin helper lowering is still mostly list-centric rather than iterable-centric

Current implication:

- the names exist, which is good
- the Python call shapes and optional-argument surfaces are still not broad enough to count as true parity

### 3. Builtin type object-model parity

#### `list`

Already present:

- indexing
- slicing
- `append`
- `extend`
- `pop`
- `sort`
- `reverse`
- `index`
- `remove`
- `clear`
- `copy`
- `count`

Remaining parity gaps:

- no `list(...)` constructor parity
- `list.pop()` is still zero-arg only in the current lowering
- `list.sort()` remains milestone-limited with no parity for option arguments
- `list.index()` remains narrow compared with CPython's optional bounds

#### `dict`

Already present:

- literal construction
- lookup and membership
- `get`
- `keys`
- `values`
- `items`
- `update`
- `pop`
- `copy`
- `clear`

Remaining parity gaps:

- no `dict(...)` constructor parity
- no Python-shaped constructor support from iterable pairs / mapping-copy inputs
- `dict.pop()` is still narrow compared with CPython's default-value form
- broader object-model helpers such as `setdefault` and `fromkeys` are not yet part of the native parity surface

#### `set`

Already present:

- literal construction
- native `set()` / `set(iterable)`
- membership
- `add`
- `remove`
- `discard`
- `copy`
- algebra and relation helpers
- `pop`
- `clear`

Remaining parity gaps:

- constructor parity is now in much better shape
- the main remaining work is consistency and broader object-model polish, not first-entry access

#### `tuple`

Already present:

- tuple literals
- fixed-shape typing
- destructuring
- indexing

Remaining parity gaps:

- no `tuple(...)` constructor parity
- tuple object-model parity is not tracked as a first-class phase surface yet
- tuple hashability and use as a broad Python-compatibility surface need explicit verification rather than incidental coverage

#### `str`

Already present:

- indexing
- slicing
- `join`
- `split`
- `replace`
- `find`
- `startswith`
- `endswith`
- `strip`
- `lstrip`
- `rstrip`
- `lower`
- `upper`
- `isdigit`
- other case predicates already beyond the reviewer's minimum ask

Remaining parity gaps:

- `ord` / `chr` builtins are still missing
- some optional-argument and richer CPython object-model edges remain untracked

Current implication:

- `str` parity is materially ahead of several other builtins
- `tuple` and constructor-entry parity remain behind
- `list` and `dict` object models are good enough to justify this phase focusing on constructor and call-shape cleanup rather than claiming they are entirely absent

### 4. Collections ergonomics

#### `Counter`

Current state:

- `Counter[T]` class exists
- arithmetic and helper methods exist
- common workflows are validated through `from_list(...)`

Main remaining gap:

- Python-shaped constructor parity is still absent
- users should not have to reach for `from_list(...)` when `Counter(...)` is the natural Python surface

#### `defaultdict`

Current state:

- compatibility lowering exists
- bare `defaultdict(list|set|int)` works

Main remaining gap:

- factory support is restricted to builtin names
- the callable/object-model parity is still slice-limited
- this is still a compatibility shim, not a mature parity surface

#### `deque`

Current state:

- `deque(iterable?, maxlen?)` constructor shape now exists
- bare-call compatibility exists
- `len(deque)` support exists

Main remaining gap:

- broader object-model parity is still partial
- current behavior is enough for common algorithmic use, but not enough to call the type fully Python-parity aligned

### 5. High-value stdlib module cleanup

The repo is not missing a stdlib strategy. It is missing the last layer that makes supported modules feel Pythonic.

#### `math`

- Phase 30 and later work expanded this module substantially
- the remaining issue is less "module missing" and more "subset still governed as approved slice rather than natural parity surface"

#### `collections`

- constructor and object-model parity remain the core problem
- helper functions such as `from_list(...)` should not remain the canonical ergonomic path

#### `heapq`

- subset is present
- helper semantics and safety adaptation still diverge from CPython in ways that need an explicit cleanup decision rather than perpetual drift

#### `random`

- current repo state is materially ahead of older audits: `choice`, `shuffle`, `sample`, `randrange`, and `gauss` already exist
- parity work here is mostly cleanup and optional-argument review, not greenfield module creation

#### `bisect`

- current parity matrix still records missing optional-argument parity (`lo`, `hi`, `key`)

#### `itertools`

- current module remains a valuable but eager, list-backed subset
- major missing layer is lazy iterator object-model parity

#### `functools`

- current surface is effectively `reduce` only
- if `functools` remains an advertised supported module, it still needs a deliberate parity policy instead of accidental minimal presence

#### `operator`

- current surface is narrow and type-specific
- naming still includes compatibility debt such as `mod_val`

## Root-Cause Summary

The main missing layer is no longer "stdlib modules do not exist". The current root causes are:

1. constructor parity is incomplete
2. builtin callable parity is incomplete
3. optional-argument parity is incomplete
4. object-model parity is inconsistent across core types
5. workaround APIs are still the practical path for several supported surfaces

## Phase Policy

This ad hoc phase should use the following policy.

1. If Python syntax is compatible with Sifr's safety model, support it directly.
2. If the repo already has a workaround API for it, that workaround should usually stop being the primary documented path.
3. If Python behavior conflicts with Sifr's guarantees, keep the divergence explicit, typed, and documented.

## Milestones

### milestone_psp_1: Builtin Constructor and Conversion Parity

Scope:

- implement Python-shaped constructor parity for:
  - `list(...)`
  - `tuple(...)`
  - `dict(...)`
  - `set(...)` cleanup and consistency hardening
  - `str(...)`
  - `int(...)`
  - `float(...)`
  - `bool(...)`
  - `ord(...)`
  - `chr(...)`
- define accepted input-shape matrix for each constructor
- keep safety divergences explicit where parse or bounds failure is possible

Definition of done:

- constructor-entry parity exists for the approved matrix
- no supported constructor requires workaround APIs for common Python source forms
- every intentional divergence is documented with rationale and tests

### milestone_psp_2: Builtin Functional Helper Parity

Scope:

- deepen builtin parity for:
  - `len`
  - `abs`
  - `min`
  - `max`
  - `sum`
  - `sorted`
  - `reversed`
  - `enumerate`
  - `zip`
  - `map`
  - `range`
  - `any`
  - `all`
- explicitly decide and implement the supported optional-argument matrix
- broaden list-only helpers into iterable-compatible helpers wherever the safety model allows

Definition of done:

- common Python call shapes compile directly
- optional arguments are either supported or explicitly classified
- builtins do not remain artificially two-argument or list-only when the broader safe shape is already architecturally compatible

### milestone_psp_3: Core Type Object-Model Parity

Scope:

- close parity gaps for the approved object-model surface of:
  - `list`
  - `dict`
  - `set`
  - `tuple`
  - `str`
- prioritize Python-shaped methods over Sifr-specific helper paths
- audit each type for missing method overloads, optional arguments, and constructor consistency

Definition of done:

- supported object-model methods compile from Python-shaped source without workaround naming
- constructor parity and method parity are coherent for each approved type
- remaining unsupported methods are classified explicitly rather than left implicit

### milestone_psp_4: Collections Constructor and Ergonomics Parity

Scope:

- implement natural constructor-entry parity for:
  - `Counter()`
  - `Counter(iterable)`
  - `Counter(mapping)`
  - `defaultdict(factory[, initial])`
  - `deque(iterable?, maxlen?)`
- retire helper-only ergonomics as the primary documented path where parity exists

Definition of done:

- common Python `collections` entry surfaces work directly
- `from_list(...)`-style helpers are no longer required for natural Python source
- remaining callable-factory or ownership-driven divergences are explicit and justified

### milestone_psp_5: Existing-Module Python-Surface Cleanup

Scope:

- audit and clean up the Python-shaped surface for:
  - `math`
  - `collections`
  - `heapq`
  - `random`
  - `bisect`
  - `itertools`
  - `functools`
  - `operator`
- remove the need for workaround names or workaround call shapes where parity is otherwise feasible
- keep this milestone focused on already-existing modules, not new-module expansion

Definition of done:

- supported modules feel Pythonic at the entry surface
- workaround APIs are either retired from primary docs or classified as explicit Sifr extensions
- every claimed supported surface has a concrete parity status

### milestone_psp_6: Parity Governance for Python-Shaped Source

Scope:

- add a canonical inventory for:
  - `done`
  - `open`
  - `intentional-diff`
  - `unsupported`
- classify constructor, builtin, object-model, and module-surface gaps explicitly
- prevent future compatibility work from rediscovering undocumented surface gaps piecemeal

Definition of done:

- parity status is reviewable at the source-surface level, not only the module-subset level
- no unresolved gap remains undocumented
- future corpus-driven work can link to explicit surface classifications instead of reopening ambiguity

## Intentional Divergences That Must Stay Explicit

This phase should not erase Sifr's safety model.

- `int(str)` remains `Result[int, ParseError]`
- `float(str)` remains `Result[float, ParseError]`
- `Option` / `Result` remain the adaptation path where CPython would raise
- compile-time rejection remains preferable to runtime rejection for invalid ownership, mutability, or hashability patterns
- empty or missing collection behavior should remain panic-free
- ownership transfer should remain explicit

## Quality Contract

### Entry criteria

- Phase 31 is complete
- Phase 30 and Phase 31 evidence is available and treated as the starting baseline rather than reopened blindly
- Phase 27 non-regression baseline is green at phase start and must remain green through completion
- Phase 16 local-first validation platform remains the authoritative execution foundation
- this phase must start from the current Phase 30 and Phase 31 parity evidence rather than reopening closed module subsets blindly

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

- no fallback compatibility shims as the final user-facing answer
- no partial parity claims without explicit classification
- no user-triggerable runtime panics
- every root-cause fix includes regression coverage
- constructor parity work must include both positive-path and negative-path validation
- optional-argument parity decisions must be documented, not left implicit
- every milestone must satisfy the scope and definition of done already documented in this file
- every milestone includes at least one positive-path and one negative-path validation case
- validation evidence must be recorded in the execution checklist issue before merge
- no milestone is complete if its outputs are not reviewable and reproducible locally
- parity-governance outputs must be machine-reviewable and deterministic
- any divergence or waiver must be explicit, time-bounded, owner-assigned, and issue-linked
- if a milestone changes existing approved subset behavior from Phase 30, the change must explicitly classify whether it is:
  - parity expansion
  - compatibility cleanup
  - intentional divergence retained
  - prior waiver retired
- modules or builtins with parsing-heavy, numeric-edge, or panic-risk surfaces must reuse the established property/fuzz machinery where applicable rather than relying only on happy-path e2e coverage

### Validation planning goals

- `milestone_psp_1` (Builtin Constructor and Conversion Parity): validation goals cover: Python-shaped constructor parity for `list`, `tuple`, `dict`, `set`, `str`, `int`, `float`, `bool`, `ord`, and `chr`; accepted input-shape matrix for each constructor; explicit safe adaptation for parse/bounds failures. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_2` (Builtin Functional Helper Parity): validation goals cover: common Python call shapes and approved optional-argument surfaces for `len`, `abs`, `min`, `max`, `sum`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `any`, and `all`; iterable-vs-list behavior; explicit classification for unsupported call shapes. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_3` (Core Type Object-Model Parity): validation goals cover: approved object-model surface for `list`, `dict`, `set`, `tuple`, and `str`; constructor/method coherence; explicit classification of unsupported methods and overloads. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_4` (Collections Constructor and Ergonomics Parity): validation goals cover: Python-shaped constructor-entry parity for `Counter`, `defaultdict`, and `deque`; retirement of workaround-only entry surfaces as the primary path; explicit handling of callable-factory and ownership-driven divergences. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_5` (Existing-Module Python-Surface Cleanup): validation goals cover: Python-shaped cleanup for existing modules `math`, `collections`, `heapq`, `random`, `bisect`, `itertools`, `functools`, and `operator`; removal or explicit classification of workaround names and call shapes. Include negative-path goals that catch regressions against these guarantees.
- `milestone_psp_6` (Parity Governance for Python-Shaped Source): validation goals cover: canonical source-surface inventory for `done`, `open`, `intentional-diff`, and `unsupported`; explicit classification of constructor, builtin, object-model, and module-surface gaps; prevention of undocumented rediscovery through future corpora. Include negative-path goals that catch regressions against these guarantees.
- Exit-gate evidence explicitly demonstrates: supported Python-shaped source compiles naturally for the approved scope, intentional divergences remain explicit and safe, and future compatibility work is governed by a canonical source-surface inventory rather than ad hoc rediscovery.

### Local validation commands

- Full local suite:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

### Exit criteria

- All milestone definitions of done are satisfied.
- Supported Python-shaped source for the approved scope compiles naturally without workaround-first APIs.
- Intentional divergences remain explicit, typed, panic-free, and documented.
- Constructor, builtin-helper, object-model, and module-surface gaps are tracked in a canonical parity inventory.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate

Python-shaped source parity is production-governed for the approved scope: supported builtins, constructors, object models, and existing in-scope stdlib entry surfaces compile naturally; intentional divergences remain explicit and safety-aligned; and the Phase 27 non-regression contract remains green with deterministic, reviewable validation evidence.

## Recommended First Execution Order

1. builtin constructors and conversions
2. collections constructor parity
3. builtin functional helper optional-argument parity
4. core type object-model cleanup
5. existing-module Python-surface cleanup
6. parity governance closeout

## Why This Is Better Than Continuing Corpus Discovery

This phase turns a recurring compatibility smell into a deliberate engineering program.

- Phase 30 proved subset module behavior.
- Phase 31 proved that real Python-shaped source still exposes a missing surface layer.
- This ad hoc phase should now close that layer systematically instead of letting future corpora keep rediscovering the same class of gap under different symptoms.

# Ad Hoc Phase: Stdlib Namespace Contract And Compatibility Cleanup

Status: complete

## Objective

Make the stdlib import contract explicit, diagnostic-backed, and internally consistent.

Sifr stdlib modules are permanently public as `sifr.*`. CPython-style bare stdlib names such as `math`, `json`, `os`, `heapq`, and `collections` are not aliases for Sifr stdlib. This phase closes the current inconsistency where some Python-shaped call expressions compile through hidden lowering compatibility, while bare stdlib imports either silently no-op and fail later at the use site or hit generic unsupported-import diagnostics instead of the namespace-contract diagnostic.

This phase is complete when:

- architecture and user docs state the namespace contract;
- bare stdlib import attempts produce targeted suggestions to use `sifr.*`;
- lowering no longer synthesizes hidden `sifr.*` imports for `math.*`, `heapq.*`, `collections.*`, `deque(...)`, or `Counter(...)`;
- `defaultdict(int/list/set)` remains supported only through an explicit `from sifr.collections import defaultdict` binding;
- tests and demos use explicit `sifr.*` imports for stdlib symbols.

This phase does not provide backward compatibility, legacy support, staged deprecation, compatibility warnings, or temporary bridges for Python-shaped stdlib calls. Milestones may be split for review and validation, but once a compatibility surface is touched it must be removed or converted directly to the explicit `sifr.*` contract in that same milestone. There is no intermediate production state where legacy bare stdlib call forms are intentionally preserved.

Execution status, validation evidence, review artifacts, and merged PR links are tracked in [ad-hoc-stdlib-namespace-contract-and-compat-cleanup-execution.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup-execution.md).

## Context

The current stdlib architecture has three tiers:

```text
_sifr.*       compiler intrinsics, internal only
sifr.*        embedded Sifr stdlib wrappers
top-level     user modules and third-party package import roots
```

The first two tiers are embedded and must not resolve to filesystem or package-manager modules. Top-level names are reserved for user code, packages, and any future interop story. The existing resolver already treats `sifr.*` and `_sifr.*` as special import roots, but lowering still contains a hidden call-expression compatibility path in `crates/sifr_lowering/src/lower/compat_imports.rs`.

That path accepts forms such as:

```python
math.fmod(x, y)
heapq.heapify(items)
collections.defaultdict(list)
deque(items)
Counter(items)
```

by synthesizing hidden aliases like `__compat_sifr_math_fmod` and pushing synthetic `HirImport` entries for `sifr.math`, `sifr.heapq`, or `sifr.collections`. This creates a mixed contract: `math.fmod(...)` may compile, while `from math import fmod` does not resolve as a Sifr stdlib import.

Current bare stdlib import diagnostics are inconsistent:

| Mode | `from math import sqrt` today | `import math` today |
|---|---|---|
| Single-file lowering | `SIFR-IMPORT-0002` unknown import target from lowering. | `SIFR-IMPORT-0003` unsupported import form from lowering. |
| Project discovery | `SIFR-IMPORT-0002` unknown import target with workspace tried paths before lowering. | Discovery ignores `Stmt::Import`; lowering later emits `SIFR-IMPORT-0003`. |
| Package discovery | `SIFR-IMPORT-0002` unknown import target during package import resolution. | Package discovery ignores `Stmt::Import`; lowering later emits `SIFR-IMPORT-0003` if the module is compiled. |

## Locked Decisions

1. Sifr stdlib is reached through `sifr.*`; this is a permanent public contract.
2. Bare CPython stdlib names are not aliases for `sifr.*` in any edition, manifest option, warning mode, migration mode, or deprecation track.
3. Sifr is Python-syntax and CPython-behavior-informed, not Python-source-compatible.
4. No `sifr migrate`, codemod, or Python-source conversion tool is part of this phase.
5. `typing` and `enum` remain closed frontend/type-system support imports. They do not generalize to runtime stdlib.
6. `_sifr.*` remains internal and blocked in user code.
7. Existing compatibility behavior is removed atomically at each touched surface. Do not keep `collections.defaultdict(...)`, bare `defaultdict(...)`, bare `deque(...)`, bare `Counter(...)`, `math.*`, `heapq.*`, or `collections.*` working as a transitional production bridge once the call-compatibility milestone lands.
8. The current `__compat_defaultdict_int`, `__compat_defaultdict_list`, and `__compat_defaultdict_set` names are internal typed defaultdict representations, not stdlib namespace aliases. This phase renames them to `__sifr_defaultdict_int`, `__sifr_defaultdict_list`, and `__sifr_defaultdict_set` in the same milestone that removes bare `defaultdict` compatibility, so no legacy `__compat_defaultdict_*` naming remains after the explicit-binding change.
9. Async/task compatibility bookkeeping for explicitly imported `sifr.asyncio`, `sifr.task`, `sifr.sync`, or `sifr.concurrent` surfaces is out of scope unless it depends directly on the removed synthetic stdlib import path. Existing `__compat_sifr_sync_*`, `__compat_sifr_concurrent_*`, and generic async/task defensive codegen checks are intentionally retained for now.

## Non-Goals

- Do not add support for bare `import math` or `from math import sqrt`.
- Do not add transitional support, compatibility warnings, deprecation periods, or legacy fallback behavior for bare stdlib calls.
- Do not add a `--python-compat` mode.
- Do not add a `sifr.toml` setting for Python-compatible imports.
- Do not make `sifr.*` optional, deprecated, or secondary.
- Do not remove the specialized typed `defaultdict` lowering itself.
- Do not broaden this phase into package-manager aliasing, Python interop, or general import-form expansion.

## Namespace Policy

Add the following invariant to `internal_docs/architecture.md` and summarize it in user docs:

| Import root | Owner | Resolution |
|---|---|---|
| `_sifr.*` | Compiler intrinsics | Embedded only; never filesystem or package-manager resolution. |
| `sifr.*` | Sifr standard library | Embedded `sifr_stdlib::STDLIB_SOURCES`; never filesystem or package-manager resolution. |
| top-level | User code and third-party packages | Workspace/package resolution. |

Bare module names that happen to match Sifr stdlib module tails are rejected as bare stdlib import attempts only after normal top-level user/package resolution fails. User or package modules named `math`, `json`, or similar therefore keep priority once they are real import targets.

The bare-stdlib module-tail set is derived from `sifr_stdlib::STDLIB_SOURCES` by stripping the leading `sifr.` from each embedded stdlib module name. `_sifr.*` intrinsic names are not part of this set. Add shared `sifr_stdlib` helpers, e.g. `is_bare_stdlib_tail(module: &str) -> Option<BareStdlibMatch>`, so driver discovery, package discovery, and lowering use the same source of truth.

Matching is exact full-tail first, then leading-root fallback:

1. If `module` is exactly in the tail set, match that full tail and suggest `sifr.<module>`.
2. Otherwise, if the first dotted component is in the tail set, match the full written module as `bare_module`, use the root as `matched_tail`, and suggest `sifr.<root>` with help that no embedded `sifr.<module>` module exists.
3. Otherwise, the import is not a bare stdlib import.

Examples:

- `from math import sqrt` -> `bare_module = "math"`, `suggested_module = "sifr.math"`.
- `from collections.abc import Iterable` when `sifr.collections.abc` is not embedded -> `bare_module = "collections.abc"`, `suggested_module = "sifr.collections"`, with help that `sifr.collections.abc` is unavailable.
- `import collections.abc` follows the same match data, but still reports that Sifr does not support module-object imports.

`typing` and `enum` are closed frontend/type-system support imports. They emit no Rust, have no embedded stdlib source, and are not runtime stdlib aliases.

## Diagnostics Contract

Bare stdlib import attempts should not fall through to generic unknown-name, unknown-import, or unsupported-import messages when the requested root is a known Sifr stdlib module tail and top-level user/package resolution did not find a real module.

Examples:

```python
import math
import math as m
from math import sqrt
from collections import deque
from json import dumps
```

Expected diagnostic shape:

```text
bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'
help: use 'from sifr.math import sqrt'
```

The diagnostic must:

- use diagnostic code `SIFR-IMPORT-0008` with registry constant `IMPORT_BARE_STDLIB`;
- cover both `Stmt::Import` and `Stmt::ImportFrom`;
- for `import math` / `import math as m`, suggest `from sifr.math import <name>` because Sifr does not currently support module-object imports;
- for `import sifr.math` / `import sifr.math as m`, keep the existing unsupported-import-form behavior unless a later import-form phase adds module-object imports;
- for `from math import sqrt`, suggest `from sifr.math import sqrt`;
- for `import collections.abc`, report `bare_module = "collections.abc"` and suggest `sifr.collections.abc` only if that embedded stdlib module exists; otherwise suggest `sifr.collections` and state that no `sifr.collections.abc` module exists;
- preserve existing unresolved-user-module diagnostics for names that do not match stdlib module tails;
- avoid masking real workspace/package modules named `math`, `json`, or similar by attempting top-level resolution before the bare-stdlib diagnostic;
- include machine-readable diagnostic data: `bare_module`, `suggested_module`, and `imported_names` for `from ... import ...` forms;
- have docs under `docs/errors/` and an entry in `internal_docs/diagnostic_codes.md`.

### Diagnostic Transport

M1 extends lowering diagnostic transport so `SIFR-IMPORT-0008` carries the same structured args in single-file lowering as it does in driver discovery:

- Add `args: BTreeMap<String, DiagnosticArg>` to `sifr_ir::HirDiagnostic`.
- Add a lowering helper such as `LowerCtx::error_with_code_args_at`.
- Thread lowering args through frontend rendering in `crates/sifr_frontend/src/query_diagnostics.rs`.
- Existing lowering diagnostics may keep empty args; this phase does not require backfilling structured args for unrelated diagnostics.
- Because `DiagnosticArg` currently has scalar variants only, encode `imported_names` as a stable comma-separated string in declaration order, with aliases rendered as `name as alias`. For `Stmt::Import` diagnostics, use an empty string for `imported_names`.
- `bare_module` is always the exact module text written by the user. `suggested_module` is the embedded `sifr.*` module the diagnostic recommends. Discovery-owned diagnostics may keep existing resolution-scope args such as `resolution_scope` and `tried_paths` in addition to these required args.

### Layer Ownership

`SIFR-IMPORT-0008` ownership is split by import form and compilation layer:

- Project `Stmt::ImportFrom`: `crates/sifr_driver/src/project/discovery.rs` probes workspace candidates first. If resolution is `Unresolved` and the written module matches a bare stdlib tail, discovery reclassifies the failure to a bare-stdlib diagnostic and emits `SIFR-IMPORT-0008` with `bare_module`, `suggested_module`, `imported_names`, and resolution-scope args. Lowering does not run for that unresolved module, so no duplicate diagnostic is emitted.
- Package `Stmt::ImportFrom`: `crates/sifr_driver/src/project/package_discovery.rs` applies the same probe-then-reclassify rule after package/source-map resolution fails.
- Single-file `Stmt::ImportFrom`: lowering owns `SIFR-IMPORT-0008` in the import resolution path because there is no project discovery pass.
- `Stmt::Import` in all modes: lowering owns `SIFR-IMPORT-0008` in `crates/sifr_lowering/src/lower/mod_impl.rs`, replacing the generic unsupported-form diagnostic only when the imported module matches a bare stdlib tail. Project/package discovery intentionally continues to ignore `Stmt::Import` for dependency edges because Sifr has no module-object import support in this phase.

Duplicate prevention rule: only the layer that first observes an unresolved bare stdlib import emits `SIFR-IMPORT-0008`. Discovery-owned `ImportFrom` diagnostics stop project/package processing before lowering. Lowering-owned `Import` diagnostics are not emitted by discovery.

No user-facing bare-stdlib diagnostics are emitted by dependency collectors. `crates/sifr_driver/src/project/compile_order.rs` remains dependency-order-only, keeps collecting only supported `Stmt::ImportFrom` local-module edges, and does not emit `SIFR-IMPORT-0008`. No changes are required in `crates/sifr_frontend/src/query_diagnostics.rs` dependency collection or `crates/sifr_frontend/src/module_signatures.rs` import signatures for `Stmt::Import`; these collectors do not emit user-facing import diagnostics and bare stdlib imports do not create local dependency edges or supported module-object signature entries.

## Defaultdict Contract

`defaultdict(int/list/set)` remains a compiler-recognized typed surface because current codegen relies on specialized alias types for efficient defaulting and mutation. Only `int`, `list`, and `set` factories are recognized; other factory expressions continue to produce the existing unsupported-factory diagnostic. Expanding the factory set is out of scope.

This phase chooses the typed factory-based `defaultdict` surface as the public `sifr.collections.defaultdict` contract. The older integer-default class-style API currently present in `lib/sifr/collections.sifr`, e.g. `defaultdict(0)` with `.ensure(...)` / `.set(...)`, is not preserved as compatibility behavior. Any tests, demos, or LeetCode fixtures using that older API must be rewritten to the typed `defaultdict(int/list/set)` surface or to an ordinary `dict` helper, depending on the use case.

The public spelling becomes explicit:

```python
from sifr.collections import defaultdict

def main():
    groups = defaultdict(list)
```

Aliases must work:

```python
from sifr.collections import defaultdict as dd

def main():
    groups = dd(set)
```

Bare forms must fail:

```python
defaultdict(list)
collections.defaultdict(list)
```

`from sifr.collections import defaultdict as defaultdict` is treated the same as a plain import.

Implementation should record explicit imported special constructors in lowering state rather than treating `defaultdict` as an unconditional builtin name. The compatibility-removal milestone owns all `defaultdict` compatibility removal atomically: the `collections.defaultdict(...)` attribute-call short-circuit, the bare `defaultdict(...)` short-circuit in `compat_imports.rs`, the unconditional builtin recognition in `call_builtins.rs`, the older integer-default `lib/sifr/collections.sifr` class API conflict, and the `__compat_defaultdict_*` internal alias rename.

Class-field inference must follow the same contract. `crates/sifr_lowering/src/lower/class_field_inference.rs` currently infers bare compatibility constructors for `deque`, `Counter`, and bare `defaultdict`. The compatibility-removal milestone removes the bare `deque`/`Counter` inference helper and changes field inference for `defaultdict(...)` to consult the same explicit `sifr.collections.defaultdict` import-binding state used by call lowering. Bare `defaultdict(...)` in class-field initializer inference must not continue to infer a typed mapping after the call itself is rejected.

## Example Corpus Discovery

Full discovery found two first-party example corpora that must be adapted and validated before phase closeout:

- LeetCode audit corpus: `audits/leetcode/src` contains 416 checked-in `.sifr` fixtures.
- Demo corpus: `demos` contains 389 `.sifr` files, including 310 `main.sifr` entrypoints.

LeetCode namespace-impact discovery:

- Must update source: `audits/leetcode/src/0036_valid_sudoku.sifr` uses `collections.defaultdict(set)`.
- Must update source: `audits/leetcode/src/0350_intersection_of_two_arrays_ii.sifr` uses bare `Counter(...)` and bare `defaultdict(...)`.
- Must update source: `audits/leetcode/src/0383_ransom_note.sifr` uses bare `Counter(...)`.
- Must update source: `audits/leetcode/src/0474_ones_and_zeroes.sifr` uses bare `defaultdict(...)`.
- Must update source: `audits/leetcode/src/0621_task_scheduler.sifr` uses bare `Counter(...)`.
- Must update source: `audits/leetcode/src/0767_reorganize_string.sifr` uses bare `Counter(...)`.
- Must update source: `audits/leetcode/src/1189_maximum_number_of_balloons.sifr` uses bare `Counter(...)`.
- Must update source: `audits/leetcode/src/1383_maximum_performance_of_a_team.sifr` uses `heapq.heappop(...)` and `heapq.heappush(...)`.
- Must update source: `audits/leetcode/src/1481_least_number_of_unique_integers_after_k_removals.sifr` uses bare `Counter(...)`.
- Already explicit but must remain green: `audits/leetcode/src/0752_open_the_lock.sifr` imports `deque` from `sifr.collections`.
- Existing explicit `sifr.heapq` fixtures must remain green: `0355_design_twitter`, `0502_ipo`, `0703_kth_largest_element_in_a_stream`, `0743_network_delay_time`, `0778_swim_in_rising_water`, `0973_k_closest_points_to_origin`, `1046_last_stone_weight`, `1631_path_with_minimum_effort`, `1834_single_threaded_cpu`, and `1985_find_the_kth_largest_integer_in_the_array`.
- `audits/leetcode/src/1963_minimum_number_of_swaps_to_make_the_string_balanced.sifr` only mentions `math.ceil` in a comment; update the comment if it reads as an endorsed bare stdlib spelling.

Demo namespace-impact discovery:

- Must update source: `demos/defaultdict/main.sifr` uses `collections.defaultdict(...)` and bare `defaultdict(...)`.
- Must update source: `demos/collections_and_argparse/main.sifr` explicitly imports `defaultdict` but uses the older integer-default `defaultdict(0)` API. Rewrite it to the typed `defaultdict(int)` surface or an ordinary `dict` helper; do not preserve `defaultdict(0)` for this demo.
- Already explicit but must remain green: `demos/generic_stdlib/main.sifr`, `demos/collections/main.sifr`, `demos/ordered_collections/main.sifr`, `demos/stdlib_classes/main.sifr`, and `demos/advanced_class_libraries/main.sifr`.
- Text/comment cleanup only unless validation fails: `demos/core_libraries/main.sifr` prints labels containing `math.*` while calling explicitly imported `sifr.math` functions; comments in `demos/stdlib_classes/main.sifr` and `demos/advanced_class_libraries/main.sifr` mention `collections.*` as module names.
- False positive: `demos/subscript_assignment/main.sifr` defines a local `Counter` class and does not rely on stdlib compatibility.

The final corpus milestone must repeat this discovery after implementation, because new checked-in examples may appear while earlier milestones are landing.

## Milestones

### milestone_stdlib_namespace_1: Policy And Diagnostics

- Add the namespace ownership invariant to `internal_docs/architecture.md`.
- Add `docs/stdlib_imports.md`, linked from the docs index if an index is present, explaining why stdlib imports use `sifr.*`.
- Add shared `sifr_stdlib` bare-stdlib tail helpers derived from `STDLIB_SOURCES`.
- Add `SIFR-IMPORT-0008` / `IMPORT_BARE_STDLIB` for bare stdlib import attempts.
- Extend lowering diagnostic transport so `HirDiagnostic` can carry structured args and frontend rendering preserves them.
- Add project discovery and package discovery reclassification from unresolved import to bare-stdlib import after real top-level resolution fails.
- Add positive and negative diagnostic tests covering at least `import math`, `import math as m`, `from math import sqrt`, `from collections import deque`, `from collections.abc import Iterable`, and a non-stdlib missing import.
- Add project-mode coverage where `from math import sqrt` without `math.sifr` produces `SIFR-IMPORT-0008`, and a paired case with real `math.sifr` proves user modules win.
- Add package-mode coverage for unresolved bare stdlib `ImportFrom`.
- Add single-file lowering coverage for both `Stmt::Import` and `Stmt::ImportFrom`.
- Add CLI verification fixture coverage for human/json/compact output and register required args in `crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs`.

Validation:

- the bare-stdlib diagnostic tests added in this milestone;
- focused `sifr_stdlib` helper tests for exact-tail and root-fallback matching;
- verification fixture update for human/json/compact output.

### milestone_stdlib_namespace_2: Atomic Compatibility Removal

- Remove the lowering path that maps `math.*`, `heapq.*`, non-`defaultdict` `collections.*`, bare `deque(...)`, and bare `Counter(...)` to hidden `sifr.*` imports.
- Remove `collections.defaultdict(...)` and bare `defaultdict(...)` compatibility in the same milestone. Do not keep a transitional helper, compatibility bridge, deprecation path, warning mode, or legacy fallback.
- Add lowering state `explicit_defaultdict_bindings: HashSet<String>` or equivalent, recording local names imported from `sifr.collections.defaultdict`, including aliases.
- Populate the state during import resolution for `from sifr.collections import defaultdict`, `from sifr.collections import defaultdict as dd`, and `from sifr.collections import defaultdict as defaultdict`.
- In call lowering, before unconditional builtin dispatch, route only names present in `explicit_defaultdict_bindings` to `lower_defaultdict_constructor_call`.
- Remove the `collections.defaultdict(...)` and bare `defaultdict(...)` short-circuits from `compat_imports.rs`.
- Remove unconditional builtin lowering for bare `defaultdict` from `call_builtins.rs`.
- Resolve the `lib/sifr/collections.sifr` `defaultdict` name conflict in favor of the typed `defaultdict(int/list/set)` public contract. Remove or rename the older integer-default class-style API so `from sifr.collections import defaultdict` has one public meaning.
- Remove `synthetic_imports` and `synthetic_import_aliases` from lowering if no remaining producer uses them.
- Remove the consuming site in `crates/sifr_lowering/src/lower/mod_impl.rs` that extends final `imports` with `ctx.synthetic_imports`, and verify no readers or writers remain.
- Remove bare `deque` and `Counter` compatibility inference from `crates/sifr_lowering/src/lower/class_field_inference.rs`; class-field inference must use real local/class/import bindings after this milestone.
- Update `class_field_inference.rs` so `defaultdict(...)` field inference only applies when the called local name is in `explicit_defaultdict_bindings`; bare unimported `defaultdict(...)` must not infer a typed mapping.
- Remove codegen canonicalization for `__compat_sifr_math_*`, `__compat_sifr_heapq_*`, and `__compat_sifr_collections_*`.
- Remove or rewrite Rust unit tests in `sifr_codegen` that construct or assert on `__compat_sifr_math_*`, `__compat_sifr_heapq_*`, or `__compat_sifr_collections_*`; those tests lose meaning once the synthetic import path is deleted.
- Leave the generic `is_compat_stdlib_alias` codegen guard in place for retained async/task aliases unless the implementation proves no retained path can reach it.
- Rename typed aliases from `__compat_defaultdict_int/list/set` to `__sifr_defaultdict_int/list/set` across lowering, type rendering, and codegen.
- Add negative coverage for bare `defaultdict(list)`, `collections.defaultdict(list)`, and explicit-import `defaultdict(0)`.
- Update e2e `.sifr` fixtures that rely on `math.*`, `heapq.*`, `collections.*`, bare `deque(...)`, bare `Counter(...)`, bare `defaultdict(...)`, or `collections.defaultdict(...)` to import stdlib symbols explicitly.
- The implementation should grep `math\.`, `heapq\.`, `collections\.`, `deque(`, `Counter(`, and `defaultdict(` under `crates/sifr/tests/e2e/pass/` and `crates/sifr/tests/e2e/fail/`, then update or intentionally classify every hit. Demo and LeetCode adoption is owned by M3.
- Confirm mixed imports such as `from sifr.collections import defaultdict, deque` keep both surfaces working after M2; `deque(...)` must work through the explicit imported class/function path, not through synthetic compatibility.

Validation:

- focused lowering/codegen tests touched by this milestone;
- focused lowering tests for direct `defaultdict` import, alias import, bare rejection, and `collections.defaultdict` rejection;
- codegen tests for `defaultdict(int/list/set)` with explicit imports;
- e2e pass/fail fixtures for explicit and rejected forms;
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_attribute_calls.sifr`;
- affected e2e pass/fail fixtures.

### milestone_stdlib_namespace_3: Corpus Adoption And Closeout

- Add a guardrail test or script check that no production lowering code synthesizes `__compat_sifr_*` stdlib imports.
- Sweep docs, tests, demos, and comments for stale claims that Python-shaped stdlib calls work without imports.
- Repeat namespace-impact discovery across `audits/leetcode/src`, `demos`, `crates/sifr/tests/e2e/pass`, and `crates/sifr/tests/e2e/fail` after M2 lands.
- Update every affected LeetCode `.sifr` fixture to use explicit `sifr.*` imports or project-local helpers only. All checked-in LeetCode `.sifr` fixtures must compile and run under the post-cleanup namespace contract.
- Add or update a validation command for the checked-in LeetCode corpus. Do not rely on `audits/leetcode/run_audit.py` unless it is changed to validate the checked-in corpus without regenerating fixtures from hard-coded external paths.
- Update every affected demo to use explicit `sifr.*` imports, and update stale demo labels/comments that imply bare Python stdlib module calls are supported.
- Add or update a validation command for all runnable demos. Negative-case demos may be excluded only by an explicit, documented exclusion list; all non-negative demo `main.sifr` entrypoints must work.
- Update execution checklist with validation evidence, PR links, and review artifacts.
- Run create-PR validation before implementation PRs and merge-gate validation before phase closeout.

Validation:

- repeated discovery commands show no unclassified `math.*`, `heapq.*`, `collections.*`, bare `deque(...)`, bare `Counter(...)`, bare `defaultdict(...)`, or `collections.defaultdict(...)` uses in `audits/leetcode/src`, `demos`, or e2e fixtures;
- all checked-in LeetCode `.sifr` fixtures in `audits/leetcode/src` compile and run through the final corpus validation command;
- all runnable demos compile and run through the final demo validation command;
- `rg "__compat_sifr_(math|heapq|collections)_" crates/sifr_lowering/src crates/sifr_codegen/src crates/sifr_type_system/src -g '*.rs'` returns no production hits;
- `rg "__compat_defaultdict_|resolve_python_compat_call_alias|resolve_bare_python_compat_call_alias|synthetic_imports|synthetic_import_aliases" crates/sifr_lowering/src crates/sifr_codegen/src crates/sifr_type_system/src -g '*.rs'` returns no production hits;
- `cargo fmt --check`;
- `cargo clippy --workspace -- -D warnings`;
- `python3 scripts/check_file_size_guardrails.py`;
- `scripts/run_all_tests.sh --profile create-pr`;
- `scripts/run_all_tests.sh` before phase closeout.

## Exit Gates

This phase may close only when:

1. `sifr.*` namespace policy is documented in architecture and user docs.
2. Bare stdlib imports receive targeted diagnostics with suggestions.
3. No production lowering path synthesizes hidden `sifr.*` imports from Python-shaped bare module calls.
4. `defaultdict` support requires an explicit `sifr.collections` import or alias, with no transitional support for bare `defaultdict(...)` or `collections.defaultdict(...)`.
5. Checked-in LeetCode fixtures, demos, and e2e fixtures use explicit stdlib imports.
6. All checked-in LeetCode `.sifr` fixtures compile and run.
7. All runnable demos compile and run.
8. Typed defaultdict internals use `__sifr_defaultdict_*`, with no remaining `__compat_defaultdict_*` names.
9. Local validation evidence is recorded in the execution checklist.
10. agent planning and final-readiness reviews have returned `READY` after all blocking findings are addressed.

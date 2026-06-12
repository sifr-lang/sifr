# Review: Stdlib Namespace Contract And Compatibility Cleanup

## 1. Verdict

**READY**

## 2. Loophole check

The phase has no backward-compatibility or legacy-support loopholes for CPython-style bare stdlib calls. Specifically:

- **Objective and contract**: explicitly disclaims backward compatibility, legacy support, staged deprecation, compatibility warnings, and temporary bridges for Python-shaped stdlib calls. The "no intermediate production state" sentence locks this in.
- **Locked Decisions 1, 2, 7**: bare CPython names are not aliases "in any edition, manifest option, warning mode, migration mode, or deprecation track," and compatibility is removed atomically with no transitional production bridge.
- **Non-Goals**: explicitly bars `--python-compat` mode, `sifr.toml` opt-in, transitional support, compatibility warnings, deprecation periods, and legacy fallback.
- **M2 (Atomic Compatibility Removal)**: removes `math.*`, `heapq.*`, non-`defaultdict` `collections.*`, bare `deque(...)`, bare `Counter(...)`, `collections.defaultdict(...)`, and bare `defaultdict(...)` in a single milestone. The class-field inference compatibility paths for `deque`, `Counter`, and bare `defaultdict` are removed in the same milestone, and `__compat_defaultdict_*` is renamed to `__sifr_defaultdict_*` atomically with the binding change — so no `__compat_defaultdict_*` survives the explicit-binding cutover.
- **Planning review pass 2 note**: the previously requested transitional defaultdict helper is explicitly superseded by the no-legacy-support clarification; the prior M2/M3 split is no longer in effect.
- **Retained surfaces (Locked Decision 9)**: `__compat_sifr_sync_*`, `__compat_sifr_concurrent_*`, generic async/task defensive codegen checks, and the `is_compat_stdlib_alias` codegen guard are intentionally retained, but they support **explicitly imported** `sifr.asyncio` / `sifr.task` / `sifr.sync` / `sifr.concurrent` — not CPython-style bare stdlib calls. They do not constitute a bare-stdlib compatibility loophole; they are out-of-scope internal bookkeeping for already-explicit `sifr.*` imports.
- **`import sifr.math` keeps unsupported-import-form behavior**: this is not legacy bare-stdlib preservation; module-object imports are simply unsupported at the language level, and the diagnostic reflects that.
- **M3 guardrails**: ripgrep-driven checks fail the build if any production code still synthesizes `__compat_sifr_(math|heapq|collections)_*`, `__compat_defaultdict_*`, `resolve_python_compat_call_alias`, `resolve_bare_python_compat_call_alias`, `synthetic_imports`, or `synthetic_import_aliases`.

## 3. Confirmation

The phase has no backward-compatibility or legacy-support loopholes for CPython-style bare stdlib calls. All hidden compatibility surfaces (`math.*`, `heapq.*`, `collections.*`, bare `deque`, bare `Counter`, bare `defaultdict`, `collections.defaultdict`, synthetic stdlib imports, class-field inference compat, and `__compat_defaultdict_*` naming) are removed or converted directly to explicit `sifr.*` binding inside a single atomic milestone, with grep-based guardrails enforcing the removal at closeout.

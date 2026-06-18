# Review Round 2: Embedded Python Interop Phase

## 1. Round 1 blocking issues — all resolved

| # | Issue | Status | Where addressed |
|---|---|---|---|
| B1 | `foreign_blocking` vs `@blocking_io` | ✅ | `@blocking_io` throughout (lines 22, 39, 291, 348) |
| B2 | `py.blocking` vs offload primitive | ✅ | "There is no Python-specific `py.blocking` alias" (line 348); milestone_py_5 echoes it (line 738) |
| B3 | `ThreadsafeCallback` Send story | ✅ | Lines 499–510 name it as the audited bridge; specify registry storage, GIL reacquisition, scheduler dispatch, and no exposure of Python refs as Send |
| B4 | Native-extension carve-out | ✅ | Line 534 (boundary text) and line 833 (Quality Contract invariant) |
| B5 | Trust enforcement mechanism | ✅ | HIR static check for static imports + `@trust_python_dynamic` annotation for dynamic + runtime root check (line 92); wildcards rejected at publish/check + package-graph load (line 91); allow-imports vs trust.python vs trust.python-native triad (line 90) |
| B6 | GIL boundary | ✅ | `py.scope` Sifr API (line 263), `PyGilScope` type (line 186), `with_gil` Rust API (line 191), lowering row (line 288) |
| B7 | Coroutine semantics | ✅ | Per-thread loop reuse, no-reentry rule, uvloop honored, blocking return (line 352) |
| B8 | Build/link contract | ✅ | New "Build and Link Contract" section (lines 137–142): probe is build metadata, cache key invalidators enumerated, no host-global fallback, PyO3 embedding not extension mode, dynamic libpython loader requirements recorded |
| B9 | Self-containment vs Phase 27 | ✅ | No phase-number anchor remains; Entry Criteria says "Existing local validation gates remain green" (line 828) |
| B10 | `43.1` numbering | ✅ | Renumbered `PY-1`, marked sequence-independent in `plans/phases/index.md:53` |
| E1 | View Send/Sync | ✅ | All five view types non-`Send` by default; audited-bridge clause (line 394) |
| E2 | Conversion table split | ✅ | Two-column "default Sifr type" / "explicit conversion API" (lines 299–308) |
| E3 | Tier 1 too broad | ✅ | Tier 1a core gate vs Tier 1b ecosystem gate (lines 544, 549) |
| E4 | `enabled=true` redundancy | ✅ | Removed from example |
| E5 | `py.with` lifetime | ✅ | "`entered` cannot escape", generic `T`, `__exit__(exc_type, exc, tb)` receives failure context (line 373) |
| E6 | `run.sh` vs `run.py` canonical | ✅ | "`run.sh` is canonical; delegates to `runner/run.py`" (line 649) |
| E7 | Reserve diagnostic families | ✅ | All eight `SIFR-PY*` families reserved in milestone_py_0 (line 675) |
| E8 | Unix-only interpreter example | ✅ | Comment now says "Sifr resolves from venv per platform" (line 69) |

## 2. No new blocking issues

I went through every revision-touched section looking for new contradictions, weakened guarantees, or under-specified mechanics. None rise to the level of a blocker:

- The new `@trust_python_dynamic` is a genuine unsafe annotation but it preserves runtime trust checking, so the security surface is intact.
- The new `py.scope` + `PyGilScope` does not break the per-call GIL discipline rule because every `py.*` op acquires the GIL idempotently inside a held scope (PyO3's `Python<'py>` token model). No statement in the doc requires that to be re-derived.
- The Tier 1a / Tier 1b split keeps the certification promise intact; nothing in `milestone_py_11` was weakened — both tiers stay inside it.
- "Audited bridge" is now a defined term for `ThreadsafeCallback`. Views still reference "an explicit audited bridge" (line 394) without naming a concrete type, but that is consistent — the bridge for views is a *future* surface, and the doc correctly defaults views to non-`Send`.

## 3. Final polish (no scope expansion)

These are wording inconsistencies that a reviewer will flag but that don't change a single decision:

1. **`call_attr` signature is inconsistent across the doc.** Example at line 244 uses three args (`obj, "method", positional`); core-operations bullet at line 260 uses four (`obj, "method", args, kwargs`); lowering-table row at line 285 uses three again. Pick `call_attr(obj, name, args, kwargs)` everywhere and let the example pass `[]` for kwargs.
2. **`py.call_method` appears once (line 362) and is otherwise undefined.** Either add it to the core operations list and lowering table, or drop the line and rely on `call_attr`.
3. **`PyValue` is used in the Rust API (lines 193–194) but not listed in the module struct block (lines 171–188).** Add `PyValue` to that block (or `enum PyValue { … }`).
4. **`py.zero_copy_as[T]` and `py.copy_as[T]` are missing from the operation lowering table** (lines 281–292) even though they are first-class user-facing operations introduced in the Zero-Copy section. Add two rows.
5. **`PyArrowCapsule` in the Rust API vs three Sifr types (`py.ArrowArray`, `py.ArrowStream`, `py.ArrowSchema`)** — either rename the Rust struct to indicate it is the generic capsule wrapper that the three Sifr types specialize, or expose three Rust types. One sentence of mapping resolves it.
6. **`LocalCallback`'s `Send` status is not stated explicitly.** It is implied by "same-thread/same-stack reentry only" (line 491), but for symmetry with the view-type clause (line 394), add "non-`Send`" to its bullet list.
7. **Conversion-rules row "bytearray, memoryview, buffers"** suggests `py.zero_copy_as[py.BufferView[T]]` as the default explicit API but the Buffer subsection talks about `Py_buffer` ownership and never gives the user-facing call signature. One sentence in the BufferView subsection ("Acquired via `try py.zero_copy_as[py.BufferView[T]](obj)`") would close the loop.
8. **`py.run_coroutine_blocking` argument convention** is shown in the resource-cleanup example (line 364) as `py.run_coroutine_blocking(try py.call_attr(obj, "aclose", []))`. The doubled `try` is parseable but awkward; consider `let coro = try py.call_attr(obj, "aclose", [], []); try py.run_coroutine_blocking(coro)`.

## 4. Verdict

**The document is ready after the small polish in §3.** All round-1 blockers are closed, no new blockers were introduced, and the remaining items are signature/wording consistency fixes that do not touch a single design decision.

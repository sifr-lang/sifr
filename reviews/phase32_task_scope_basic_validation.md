

## Review Findings

### Fixture: `task_scope_basic.sifr`

**Finding 1 — Exact duplicate of `scope_spawn_core.sifr`**

The new fixture:
```sifr
async def worker() -> int:
    await task.sleep(0.0)
    return 41

async def main() -> Result[None, ScopeFailure]:
    async with task.scope() as scope:
        handle = scope.spawn(worker())
        result = await handle.join()
    return None
```

`scope_spawn_core.sifr` (already committed):
```sifr
async def worker() -> int:
    return 41

async def main() -> Result[None, ScopeFailure]:
    async with task.scope() as scope:
        handle = scope.spawn(worker())
        result = await handle.join()
    return None
```

The structure is identical — `scope.spawn` + `await handle.join()` inside `task.scope()`. The only difference is that `scope_spawn_core.sifr`'s worker omits the `await task.sleep(0.0)` yield point. The new fixture adds `await task.sleep(0.0)` but this does not meaningfully change the validated path. Both exercise the same lowering: scope spawn → handle return → join await → normal scope exit.

The milestone positive validation list at line 467 of `32_async_ecosystem.md` explicitly names `task_scope_basic.sifr`, but `scope_spawn_core.sifr` already covers it. The checklist fixture name was missing, not the coverage.

---

### Phase doc update: `32_async_ecosystem.md`

**Finding 2 — "In progress" phrasing is ambiguous**

The line:
```
- In progress task-scope basic validation slice: added the canonical `task_scope_basic.sifr` milestone fixture for the minimal spawn/join scoped-task path.
```

"In progress" is vague for a slice that has already been validated and committed. Other entries in the same section (lines 516–519) use either direct past-tense ("added") or reference a merged PR. The new entry should be consistent with that style, but the bigger issue is Finding 1.

---

### Recommended changes

1. **Withdraw `task_scope_basic.sifr`** — the path is already covered by `scope_spawn_core.sifr`. The checklist entry in the phase doc is the gap; the fix is to add the missing fixture name to the positive validation list, not to duplicate existing coverage. If the intent is to have a fixture whose worker yields (`await task.sleep(0.0)`) rather than returning immediately, rename the fixture to clarify that distinction and ensure it tests a meaningfully different lowering path (child that yields vs. child that returns synchronously).

2. **Update phase doc line 467** — if `task_scope_basic.sifr` is kept, the positive validation list already names it, so the doc entry is self-referential. If it's not kept, remove `task_scope_basic.sifr` from the positive validation list or replace it with `scope_spawn_core.sifr`.

**Changes required.**

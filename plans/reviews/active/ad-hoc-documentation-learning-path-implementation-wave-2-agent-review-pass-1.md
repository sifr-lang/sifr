Now I have everything I need. Here is the review.

---

## Wave 2 Review

### Slice 2 — Core Data Model

**Pass, with one notable gap.**

`values-and-collections.mdx` and `iteration.mdx` are clean, accurate, and well-placed before the Standard Library group in the sidebar. Information architecture is correct — language pages own the model, stdlib/collections.mdx owns Counter/deque, cross-links are in the right direction.

**Gap: fixed-width integer type names are missing** (`values-and-collections.mdx:27`)

> "Use explicit fixed-width integer types when storage layout, binary protocols, FFI, or dtype-sensitive work requires a specific representation."

The acceptance criteria says "at a high level." This sentence explains *when* but not *what*. A reader who needs `i32` or `u64` for a binary protocol has no idea what to type. Add the type names and a link to wherever they're documented. One sentence, not a section.

**Suggestion: no Python callouts in either new page**

The issue plan (Gap #5, content principle #2) calls for `<Note>` callouts on pages where Python developers would infer the wrong thing. Two places where they'd be valuable:
- `values-and-collections.mdx` Dictionaries section: "If you know Python, `dict["missing"]` here returns `None` instead of raising `KeyError`." One sentence, no table needed.
- `iteration.mdx` Mutation While Iterating: "In Python this is undefined behavior; in Sifr it's an ownership constraint the compiler enforces." Not critical, but this is the most surprising difference on the page.

**Suggestion: `iteration.mdx` has no mention of `enumerate`**

It's the most common Python loop pattern after plain `for`. If it's supported, show it. If it's not, a single sentence saying so prevents a common frustration.

---

### Slice 3 — Concurrency Section

**Pass, with one blocker.**

The overall structure is correct: Overview is a concise mental-model entry point, four concept pages stay concept-level, Channels/Parallel/Processes are correctly deferred, the sidebar group sits between Learn Sifr and Standard Library, `sidebarTitle: "Concurrency API"` is in place, and the stdlib reference correctly opens with a back-link to the Overview.

---

#### Blocker: `TaskGroup` API shape conflicts between `stdlib/concurrency.mdx` and the concept page + demo

`structured-tasks.mdx` shows the context-manager form (which matches `demos/structured_concurrency_demo/main.sifr` line 58):

```python
async with task.TaskGroup() as group:
    slow = group.spawn(slow_writes_marker())
    failing = group.spawn(fail_fast())
    failure = await failing
```

`stdlib/concurrency.mdx` shows a completely different form:

```python
group: TaskGroup = TaskGroup()
group.spawn(job_one())
group.spawn(job_two())
await group.join()
```

One of these is wrong. The demo compiles and runs, so the context-manager form is authoritative. The stdlib page's `group.join()` pattern needs to be reconciled. This inconsistency was pre-existing in the stdlib reference, but it's now directly exposed because the concept page and stdlib page sit next to each other in the same sidebar section.

---

#### Suggestion: `task` namespace needs one explanatory note

The demo confirms `task` is available without an import — `task.sleep`, `task.scope`, `task.gather`, `task.select`, `task.TaskGroup` are all used with no `import` statement in the demo. The concept pages correctly match this. But the stdlib reference shows `from sifr.task import TaskGroup`, which implies explicit import. This confuses the picture.

A single note in `async-and-await.mdx` (the first page where `task.sleep` appears) would close this: "The `task` namespace is available without an import in any async Sifr program." Then the stdlib page showing an explicit import can be disambiguated on the stdlib side.

---

#### Suggestion: `ownership-across-tasks.mdx` Mutable Borrows section shows the permitted case only

The section heading promises a constraint, but the example shows only what compiles:

```python
def append_before_await(mut items: list[int]) -> None:
    items.append(2)
    return None          # borrow ends here

async def main() -> ...:
    values: list[int] = [1]
    append_before_await(values)  # fine — borrow is done
    await task.sleep(0.0)        # await is after the borrow
```

The teaching moment is what the compiler *rejects* — e.g., holding a `mut` borrow across an `await` or storing a borrowed reference into a spawned task. Show the rejected shape first, then the fix. The current example is accurate but doesn't actually demonstrate the constraint.

---

#### Suggestion: `async-and-await.mdx` is thin; `blocking_offload` concept is absent

The page is 62 lines. The issue plan calls out `blocking_offload_demo` as source material. The most common asyncio mistake Python developers carry over is blocking the runtime with CPU-heavy work. A single section on "offloading CPU work" — pointing to `sifr.parallel` or `task.spawn_blocking` — would close the gap the issue identified and make this page complete enough to stand alone.

---

#### Minor: `structured-tasks.mdx` `select` snippet has no type annotations

The `selected` binding in the `select` example is untyped, inconsistent with the style of every other snippet on the page.

---

### Summary

| Finding | Severity | File |
|---|---|---|
| `TaskGroup` API mismatch: stdlib reference contradicts demo/concept page | **Blocker** | `stdlib/concurrency.mdx` |
| Fixed-width integer type names not given | Gap (AC) | `values-and-collections.mdx:27` |
| `task` builtin status not stated | Suggestion | `concurrency/async-and-await.mdx` |
| Mutable Borrows example shows permitted case, not the rejected constraint | Suggestion | `concurrency/ownership-across-tasks.mdx` |
| Blocking offload concept absent | Suggestion | `concurrency/async-and-await.mdx` |
| No Python callouts in new language pages | Suggestion | `values-and-collections.mdx`, `iteration.mdx` |
| `enumerate` not mentioned in iteration page | Suggestion | `iteration.mdx` |
| `select` snippet missing type annotation | Minor | `concurrency/structured-tasks.mdx` |

The implementation is otherwise well-structured and accurate. The blocker is in the pre-existing stdlib reference page, not in the new concept pages.

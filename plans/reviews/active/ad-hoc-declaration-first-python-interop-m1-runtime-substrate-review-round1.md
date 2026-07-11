Review of the M1 runtime substrate diff below. I did not modify any files.

## Intent verification

| Intent | Where | Status |
|---|---|---|
| ForeignObject owns Py<PyAny>, queues on unattached drop | `foreign_object.rs:34-46` | ✓ Fast path on `PyGILState_Check`, else push to global queue |
| Every runtime attach drains pending outside the queue lock | `python.rs:210-213`, `foreign_object.rs:48-59` | ✓ `mem::take` releases guard before `drop(pending)`; count updated after |
| `close_object` removes before attach/drop | `object_ops.rs:227-242` | ✓ Store guard scoped in inner block, released before `super::attach(drop(entry))` |
| Poisoned queue mutex recovers without panic | `foreign_object.rs:75-80` | ✓ `Err(poisoned) => poisoned.into_inner()` |
| Generated PythonRuntimeGuard declared before user main locals; drains on scope exit | `python_runtime.rs:202`, `python.rs:81-85` | ✓ Injected as first `let` in `fn main(` / `async fn main(`; drop uses `Python::try_attach(drain_pending_releases)` |
| Tests: detached-thread drop | `python.rs:676` | ✓ |
| Tests: next-attach drain | `python.rs:647` | ✓ |
| Tests: epilogue drain | `python.rs:709` | ✓ |
| Tests: `__del__` reentering Sifr callback which reenters object store | `callback_ops.rs:417-479` | ✓ Uses `threadsafe_callback` + `ReentrantDestructor.__del__`; asserts `live_objects: 0` post-run |
| Object_ops tests moved to sibling | `object_ops_tests.rs` (116L); `object_ops.rs` at 782L | ✓ Under 900-line cap |

## Correctness checks

- **Lock ordering (OBJECT_STORE ↔ GIL):** `close_object` drops store guard before attach; `store_object`/`clone_handle` acquire OBJECT_STORE only inside attach. No thread holds OBJECT_STORE and then requests the GIL, so no AB/BA cycle. Similar clean ordering for `CALLBACK_STORE`.
- **Nested attach reentry:** `Python::try_attach` handles the nested case; `drain_pending_releases` is idempotent (`mem::take` yields an empty queue on subsequent calls in the same GIL scope).
- **Closure-drop-on-attach-fail:** If `Python::try_attach` returns `None`, the closure `|_py| drop(entry)` is dropped rather than called; `entry` still drops → `ForeignObject::drop` queues without GIL. State stays consistent, deferred until the next attach.
- **Object count accounting:** `ForeignObject::new` +1 pairs with either the direct-drop `-1` (GIL fast path) or the drained `update_object_count(-released)`. `create_callback`/`close_callback` also pair a separate ±1 for the callback record. Uses `saturating_add`/`saturating_sub` and `checked_add` — no panic surface.
- **Generated main forms:** Tests cover `fn main() {}`, `fn main() -> Result<...>`, and `async fn main()`. Guard is the first `let`, dropped last, so user locals that hold ForeignObjects drop → queue → guard drain runs after them.
- **No user-triggerable panic paths:** No `unwrap`/`expect` on data-dependent values in non-test code; poisoned PENDING_RELEASES mutex recovers.

## Observations (non-blocking)

- **`python.rs` at 885 lines** (`wc -l`) — 15 lines under the 900-line guardrail. Any further growth here should trigger a decomposition pass.
- **Rare-failure cleanup gaps (not introduced by this diff, but adjacent):**
  - `callback_ops.rs:107-110` — if `update_object_count(-1)` errored (RUNTIME_STATE poisoned) after callback entry removal, the PyCFunction object is never `close_object`'d. In practice RUNTIME_STATE is only poisoned via panic-in-hold, and no code path panics while holding it, so this is theoretical.
  - `callback_ops.rs:214` — if `clone_handle(py, result_handle)` fails, both `arg_handle` (if still open) and `result_handle` (if it happens to be open) are leaked before returning `Err`. Only triggerable by a malformed callback returning an unexpected handle.
- **`initialize_runtime` holds RUNTIME_STATE across `configure_interpreter`'s attach** (`python.rs:180-197`). Safe today (no `super::attach`, no callbacks registered yet, no pending releases), but the invariant "don't call attach while holding RUNTIME_STATE" is worth preserving if init grows.
- **Guard type is Send+Sync** (`PythonRuntimeGuard { _private: () }`), so it survives await points in async main correctly. If it later gains a non-Send field, that assumption breaks.

None of these are actionable for closing this sub-wave.

## Verdict

**SATISFIED** — the sealed runtime substrate implements every intent item, all four required tests are present and correct, the diff respects the 900-line guardrail, and lock/GIL ordering, count accounting, and error-path drops hold up across attached, unattached, nested, poisoned, and shutdown scenarios. Safe to proceed to the sealed stdlib identity cutover.

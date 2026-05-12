

## Review: phase32-async-generator-control-diagnostics

### Verdict: SATISFIED

---

### Diagnostic alignment with Phase 32 docs/design

**Confirmed.** The three diagnostic families are explicitly called out in the phase contract:

| Diagnostic | Spec location | Message matches spec |
|---|---|---|
| `SIFR-STDLIB-0001` for `send`/`throw` | Model doc "Out of Scope": `async generator send() and throw()` | ✅ "AsyncGenerator.{method}() is not supported in v1; consume async generators with async for, anext(), async comprehensions, or aclose()" |
| `SIFR-TYPE-0012` for `yield from` | Model doc "Out of Scope": `async yield from / generator delegation` | ✅ "async yield from is not supported in v1; use async for over the source and yield values explicitly" |
| `SIFR-CALL-0001` for `aclose` wrong arity | Existing method arity diagnostic, no new surface needed | ✅ `aclose()` takes no arguments` |

All three match the documented non-goals from `milestone_async_7b` scope and the model invariants (invariant 22: "async generator `send()`, `throw()`, async `yield from` [...] are deferred").

---

### No regressions in existing aclose/anext behavior

**Confirmed.** `aclose` in `async_generator_methods.rs` is unchanged — it still returns `Awaitable[Result[None, GeneratorCloseError]]` and calls `reject_no_method_args` when arguments are provided. The `send`/`throw` match arm is additive and completely separate. `anext` is untouched across both files.

---

### `yield from` match scoped correctly to async generators

**Confirmed.** The match at `expressions.rs:118` guards on `ctx.current_function_is_async_generator`, which is set only when `func.is_async && function_body_contains_yield()`. This means:

- Ordinary (sync) generators: `current_function_is_async_generator = false` → falls through to the `unsupported expression type` catchall
- `yield from` in a non-async context: falls through to catchall, which is appropriate for now (no existing `yield from` fixture exists in the codebase, so no regression is possible)

This is the right scoping — async generator delegation is deferred, but there's no need to special-case sync generator `yield from` yet.

---

### Fail fixtures are meaningful and not overbroad

**Confirmed.** Each fixture tests exactly one surface:

| Fixture | Error code | Surface tested |
|---|---|---|
| `async_generator_send_not_supported.sifr` | `SIFR-STDLIB-0001` | `agen.send(2)` |
| `async_generator_throw_not_supported.sifr` | `SIFR-STDLIB-0001` | `agen.throw(...)` |
| `async_generator_aclose_wrong_arity.sifr` | `SIFR-CALL-0001` | `agen.aclose(1)` |
| `async_yield_from_not_supported.sifr` | `SIFR-TYPE-0012` | `yield from numbers()` |

No fixture tests cross-product behavior or mixes unrelated surfaces. All four match their respective diagnostic codes. The `async_yield_from_not_supported.sifr` fixture uses `yield from` inside an `async def` body that also contains `yield`, which correctly activates the async generator path.

---

### Review artifacts should not be committed

The `reviews/` directory contains many `.log` and `.md` review artifacts from prior work. These are clearly ephemeral and should not be included in any PR. No action needed on the reviewer side beyond noting this.

---

**SATISFIED**

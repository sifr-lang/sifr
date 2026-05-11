

---

## Phase 32 Async Generator Basic — Review

**Scope:** basic async-generator value surface (HIR lowering through codegen emission, eager materialization, explicit await rejection)

### Design Alignment

| Contract point | Implementation | Status |
|---|---|---|
| `async def` + `yield` → `AsyncGenerator[T, E]` | `async_generator_functions` HashSet tracks these; `infer_function_return_type` branches on `is_async` | ✅ |
| Calling returns `AsyncGenerator[T, E]`, not `Coroutine` | `lower_call` bypasses `coroutine_result_type` for async generator calls | ✅ |
| `await` on async generator rejected | `first_await_range_in_stmts` + explicit diagnostic in `lower_function` | ✅ |
| `GeneratorCloseError` registered as built-in | In `BUILTIN_ERROR_CLASSES` and `BUILTIN_ERROR_CLASSES` (codegen) | ✅ |
| Non-`None` async generator return rejected | Deferred (correct per scope) | ✅ |
| `AsyncGenerator` struct + `new`/`anext`/`aclose` in codegen | `build_async_generator_type_items()` with `new`, async `anext`, async `aclose` | ✅ |
| Codegen emits non-async `fn` | `is_async: func.is_async && !is_async_generator` | ✅ |
| `Never` → `std::convert::Infallible` in codegen | `rust_generator_error_type_with_generics` | ✅ |

### Safety

- **No user-triggerable panics:** generated `anext`/`aclose` use `Result`-returning logic with no `unwrap`/`expect`
- **Generated Rust validity:** binary builds and executes correctly
- **Error class registration:** `GeneratorCloseError` propagates through both HIR builtin registration and codegen `referenced_error_classes` insertion
- **Await rejection bounds:** the explicit diagnostic "await inside async generator bodies requires async generator state-machine lowering and is not supported yet" is clear and correctly gates the entire class of `await` + `yield` violations

### Eager Materialization Limitation

The implementation materializes all yielded values at the call site (`_yields.push(...)` for all yields, then `AsyncGenerator::new(_yields)`) and emits a non-async `fn`. This is the documented temporary state.

The spec's `milestone_async_7b` goal "lazy start on first `anext()` / `async for` / comprehension consumption" is deferred. The explicit await rejection correctly enforces the boundary: code that needs lazy semantics cannot accidentally pass the current path and will receive a clear diagnostic. This is acceptable for the current slice.

### Tests

| Fixture | Type | Validates | Result |
|---|---|---|---|
| `async_generator_basic.sifr` | pass | `async for` over `AsyncGenerator[int, GeneratorCloseError]`, generated `fn numbers()` | ✅ |
| `async_generator_await_rejected.sifr` | fail | `await task.sleep(...)` inside async generator body | ✅ diagnostic fires |
| `async_generator_return_annotation_mismatch.sifr` | fail | sync `Iterator[int]` annotation on async generator | ✅ diagnostic fires |
| `quick_e2e_manifest.json` | manifest | `async_generator_basic` in lane | ✅ |

### Local Validation

```
scripts/run_all_tests.sh --profile quick
  → 60 pass fixtures, 1 fail test suite (34 unit tests), all green
  → report_signature=827a6a7de4a4e667
  → wall_time=106.27s, cache_hit_rate=100%, rebuild_groups=0

cargo fmt --check                → PASS
cargo clippy --workspace         → PASS (no warnings)
python3 .../check_hir_...py      → PASS
cargo run -q -p sifr -- run ...  → EXECUTION SUCCESS (cached)
```

### Codegen Soundness

The generated `AsyncGenerator<T, E>` struct:
- Stores `items: std::vec::IntoIter<T>` (owned, not borrowed)
- Stores `closed: bool` for post-`aclose` exhaustion
- Stores `_err: PhantomData<E>` to bind the error type
- `anext()` is async, returns `Result<Option<T>, E>`, checks `closed` first
- `aclose()` is async, sets `closed = true`, returns `Result<(), GeneratorCloseError>`

The `Never`-to-`Infallible` mapping is semantically correct: when `E = Never`, no error can occur, and Rust's `Infallible` is the correct zero-variant type for that position.

---

**REVIEW_STATUS: SATISFIED**

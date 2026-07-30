## Review — Wave 5, PR #3081 (`2fc5c6f76` vs `f1c34cf9a`)

### Validation I reproduced independently

| Lane | Result |
|---|---|
| `cargo test -p sifr_lowering` | 914 passed, 1 ignored ✅ (matches claim) |
| `cargo test -p sifr_codegen` | 941 passed ✅ |
| `cargo clippy --workspace -- -D warnings` | clean ✅ |
| `cargo fmt --check` | clean ✅ |
| `check_hir_maintainability_guardrails.py` | PASS ✅ |
| `check_file_size_guardrails.py` | PASS (3024 files) ✅ |
| `0036_valid_sudoku` | base: `SIFR-TYPE-0005` at `cell in rows[r]`; head: `no errors found`, builds + runs, no fixture change ✅ |

The stated objective is met. However, differential probing of head against a freshly built base binary found three classes of behavior that base accepted and head breaks. Details below; all probes were run through both binaries with the same source.

---

### F1 — BLOCKING: adopted hints use inference-only types that disagree with lowering, rejecting valid programs

`nested_function_inference::infer_subscript_type` models `list[T][i]` as `T`, while real lowering models a guarded sequence index as `T | None`. Wave 5 now writes that inference type into the *declaration*, and the lowered write is then checked against it.

```python
# x1.sifr
def solve(values: list[int]) -> int:
    d = defaultdict(list)
    d[1].append(values[0])
    return len(d[1])
```
- base: builds, runs, prints `1`
- head: `error[SIFR-TYPE-0002]: list.append() argument type 'int | None' is not compatible with list element type 'int'` at `x1.sifr:6:17`

Same for the set factory (`x2.sifr`, `d[1].add(values[0])`): head → `error[SIFR-TYPE-0008]: set element type conflict: expected 'str', got 'str | None'`. And on the key side (`u2.sifr`, `key = (n, values[0]); squares[key].add("a")`): head → `error[SIFR-TYPE-0008]: defaultdict key type conflict: expected 'tuple[int, int]', got 'tuple[int, int | None]'`; base builds and runs.

`d[k].append(items[i])` is a mainstream algorithmic idiom, so this is a wide surface. Required correction: the adopted hint must be verified against the types real lowering will produce for the very same key/element expressions (the Wave 3 "exact write shape" gate does not help here — both sides of it come from the inference pass), or subscript-derived keys/elements must be excluded from adoption, or the lowered writes must widen the declaration instead of erroring. `crates/sifr_lowering/src/lower/statements/control_flow.rs:393`, `crates/sifr_lowering/src/lower/nested_function_inference/defaultdict_inference.rs:80`.

### F2 — BLOCKING: `defaultdict_type_contains_any` has no `Type::Tuple` arm

`crates/sifr_lowering/src/lower/defaultdict_refinement.rs:34-42` recurses through `List`/`Set`/`Dict` but falls to `_ => false` for `Tuple`, so a tuple key holding `Unknown` is judged fully concrete and adopted.

```python
# u1.sifr — p.x is Expr::Attribute, which infers as Unknown
def solve(p: Point) -> int:
    d = defaultdict(set)
    key = (p.x, 1)
    d[key].add("a")
    return len(d)
```
- base: builds, runs, prints `1`
- head: `error[SIFR-TYPE-0002]: dict indexing requires a key type with generated Rust Eq + Hash traits, unavailable for 'tuple[Unknown, int]'`

Two defects: a valid program is rejected, and the internal `Unknown` type leaks into a user-facing diagnostic. Required correction: add a `Type::Tuple` arm (and audit `Union`) to `defaultdict_type_contains_any`.

### F3 — BLOCKING: partially-unknown inference types escape the return-type completeness gate, leaking a raw rustc error

`refine_defaultdict_subscript` (`defaultdict_inference.rs:122`) returns the alias's unrefined value slot (`set[Unknown]` / `list[Unknown]`) and `infer_defaultdict_call_type` returns the alias itself with `Unknown` slots — where base returned plain `Type::Unknown`. The completeness gate at `state_collection.rs:382` only tests top-level `state.return_type.is_unknown()`, so these pass.

```python
# r1.sifr
def solve() -> int:
    rows = defaultdict(set)
    def peek(k: int):
        return rows[k]
    return len(peek(1))
```
- base: `error[SIFR-TYPE-0004]: function 'peek' return type could not be inferred deterministically`
- head: `sifr check` → **`no errors found`**; `sifr run` → `error[SIFR-BUILD-0005]: cargo build failed: error[E0282]: type annotations needed for 'HashMap<i64, HashSet<_>>'` at generated `src/main.rs:101`

Reproduced identically for `defaultdict(list)` (`t1.sifr`, `HashMap<_, Vec<T>>`) and for returning the dict itself (`t2.sifr`, `HashMap<K, V>`). This violates the "if it compiles, it works / no raw rustc errors" guarantee and is a strict regression in diagnostic quality.

Relatedly, the extracted `type_contains_unknown_or_any` (`type_unification.rs:62`) gained no `Set` or `Alias` arm even though `collapse_literal` and `unify_types` in the same new file both did — so `Set(Unknown)` and the defaultdict aliases are treated as fully known by `FunctionEnv::bind_var`/`bind_call_result`. Note when fixing: adding an `Alias` arm there alone would make `bind_var` unify a fresh inner-scope `defaultdict` declaration with an outer same-named binding, so the shadow isolation I verified in `v1.sifr` must be re-checked after any such change.

Required correction: do not surface structurally incomplete inference types as complete (return `Unknown`/`None` while the slot is unrefined), and/or make the return-type completeness gate recursive.

### F4 — MINOR: phase ledger cites the wrong diagnostic code

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:314` states conflicts "retain deterministic `SIFR-TYPE-0014` (`TYPE_CONTAINER_ELEMENT_CONFLICT`)". `crates/sifr_diagnostics/src/codes/registry.rs:45` defines `TYPE_CONTAINER_ELEMENT_CONFLICT` as **`SIFR-TYPE-0008`**, and that is what the compiler emits. Wave 3's row states 0008 correctly. Correct the code to `SIFR-TYPE-0008`.

The row also omits the new `set.add` element validation entirely, which changes behavior for non-defaultdict code (see note below). Record it.

### F5 — MINOR: test gaps on exactly the failure classes above

- No test covers a hint whose inference type differs from the lowered type (F1 class) — the single most likely real-world break.
- No test covers a tuple key with an unresolved member (F2 class), nor nested-function return inference over a defaultdict subscript (F3 class).
- `sibling_defaultdict_bindings_keep_independent_shapes` asserts only `is_ok()`. I confirmed by probe that the two declarations do get distinct types (`let mut values: HashMap<i64, HashSet<String>>` vs untyped `HashMap::new()`), but given Wave 3 shipped a declaration-local codegen registry for precisely this hazard, the test should pin both declaration types rather than just absence of errors.
- The two conflict negatives assert presence of a message, not cardinality. Cardinality is in fact clean (I measured exactly 1 error for the direct and the looped key-conflict cases), so this is a coverage gap, not a defect.

### F6 — MINOR: responsibility placement and duplication

`safe_defaultdict_hint_names_for_block` was added to `empty_plain_dict_inference.rs:30-50` — a module named for plain-dict inference — and duplicates the census/`retain` body of `safe_hint_names_for_block` verbatim apart from the predicate. Per AGENTS.md's decompose-by-responsibility rule, either move the defaultdict census next to the other defaultdict logic (`defaultdict_refinement.rs`) or extract the shared "one unshadowed direct declaration in this block" census into a neutral helper both call.

---

### Notes (verified, not actionable)

- **`set.add` element validation is a correct tightening, not a regression.** `set[int].add(True)` and `values.add(flag)` (`bool`) were accepted on base — base silently dropped the `set[int]` annotation and emitted `HashSet<bool>` — and are now rejected. Base already rejects the same widening for `list.append` (`SIFR-TYPE-0002`) and for annotated assignment, so head closes an inconsistency. Two nits: the new check uses a different code and message shape than the analogous `list.append` check, and it is undocumented in the ledger (F4).
- Lexical-shadow isolation holds where I probed it: an inner-function local `defaultdict` of a different shape does not leak into the enclosing declaration (`u3.sifr`, runs correctly on both), and sibling if-branch vs function-level declarations get independent generated types (`v1.sifr`).
- Seeded/initialized aliases are correctly excluded — both `order_independent_defaultdict_hint` and `is_unseeded_defaultdict_call` require exactly one positional arg and no keywords.
- Wave 4 interaction is clean: `d[1] += 1; d["x"] += 1` produces exactly one diagnostic on head, identical to base.
- `t3.sifr` (`return rows[1]` from a plain function, `Box<dyn Any>` / `E0277`) fails on base too — pre-existing, excluded.

---

## Verdict: **CHANGES REQUESTED**

Three blocking findings. F1 and F2 reject programs that base compiles and runs correctly; F3 lets `sifr check` pass and then leaks a raw rustc `E0282` at build time, which breaches the project's core guarantee. F1 is the root-cause item: the wave adopts inference-pass types onto declarations without any verification that lowering will compute the same types. F4–F6 should be cleared in the same pass.

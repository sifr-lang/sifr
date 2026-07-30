Probes and the base tree are removed; the working tree is byte-identical to how I found it (`git status` matches the starting snapshot; no worktrees added).

# Wave 3 Review — pass 6 (exact working tree vs `b3f663a17`)

## CHANGES REQUIRED

One blocking finding. Pass 4's blocker is **fully and correctly fixed** — I independently reproduced all five widening families against a freshly built `b3f663a17` baseline (`git archive` to `/tmp`, own `cargo build`) and every one is now deterministically rejected. But the new adoption gate treats a plain-dict subscript **augassign** as invisible, so it silently converts base compile errors into check-clean programs that compile and return wrong counts — the exact shape the wave contract forbids.

---

## Blocking

### 1. Newly adopted plain dicts silently accept missing-key subscript augassign → wrong counts

`crates/sifr_lowering/src/lower/empty_plain_dict_inference.rs:57-69` (census: `Expr::Subscript` targets fall through `_ => {}` at :67) · `crates/sifr_lowering/src/lower/nested_function_inference/state_collection.rs:551-563` (`analyze_stmt` handles `AugAssign` only for `Expr::Name` targets) · `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:126-131` (gate) · `crates/sifr_lowering/src/lower/container_literal_specialization.rs:200-243` (augassign validation, permissive once the dict is concrete)

A subscript augassign is invisible to *both* halves of the new gate: it never reaches `record_dict_write` (only `analyze_assign`'s `Expr::Subscript` arm calls it, `expression_inference.rs:33`), and it never refines `env.vars`. So a block whose *only* plain `Assign` writes agree exactly still passes `binding_hints == exact_dict_write_hints`, the declaration is pinned concrete, and the augassign — previously rejected because the dict was `dict[Any, Any]` — now type-checks against the adopted value type.

The realistic shape is a word tally. Both from my own runs:

```python
def tally(words: list[str]) -> int:
    counts = {}
    for w in words:
        counts[w] += 1
    counts["seed"] = 0
    return len(counts)
```

| | `b3f663a17` | this tree |
|---|---|---|
| `check` | `error[SIFR-TYPE-0005]: unsupported operand type(s) for +: 'Any' and 'int'` | **clean** |
| `build` | — | succeeds |
| `run` on `["a","b","a"]` | — | prints **`1`** |

Emitted body — the missing-key augassign compiles to a silent no-op:

```rust
let mut counts: HashMap<String, i64> = HashMap::from([]);
for w in words.iter().cloned() {
    if let Some(__elem) = counts.get_mut(&w) { *__elem += 1_i64; }   // never taken
}
counts.insert("seed".to_string(), 0_i64);
counts.len() as i64            // 1, not 3
```

Minimal form, same result: `d={}` ; `d[3] += 1` ; `d[1] = 2` ; `return len(d)` → base `SIFR-TYPE-0005`; this tree builds and prints `1` (CPython raises `KeyError`).

I isolated the cause to the new gate on this tree alone, without the baseline: appending a second `d = {}` (which pushes `direct_binding_counts` to 2 and makes the name ineligible) restores `SIFR-TYPE-0005` verbatim. So it is the adoption, not an unrelated drift.

Why this is blocking rather than deferrable to the separately-tracked issue: the augassign-after-concrete-write order (`d={}` ; `d[1]=2` ; `d[3]+=1`) is *already* wrong on base — that is `ad-hoc-dict-missing-key-augassign-semantics.md` and not this wave's job. But this wave's own remediation clause is "preserve ordinary missing-key access and **augassign** semantics" (`plans/issues/active/…-preexisting-failures.md:115`), and the ledger explicitly guards against changes that "can silently produce incorrect counts" (:257). A program base *rejected at check* now compiles and returns a wrong count, which also breaks the "if it compiles, it works" guarantee. Blast-radius expansion of a known wrong-result bug is a change in the unsafe direction.

Suggested fix, consistent with the gate's own design: disqualify any candidate name that appears as a subscript-augassign target anywhere in the block — either by counting `Expr::Subscript` augassign targets in `collect_direct_binding_counts`/`safe_hint_names_for_block`, or by poisoning `FunctionEnv::exact_dict_write_shapes` to `None` for that name from the `AugAssign` arm of `analyze_stmt` (the second is more robust: it also covers augassigns reached only through control flow, and it composes with the existing merge). `0001_two_sum` is unaffected (no augassign). Wave 4's `defaultdict(int)` augassign work is unaffected — that alias never goes through this path (verified byte-identical emit, below). Pin both orders with a lowering test asserting `SIFR-TYPE-0005`.

---

## Verified: pass 4's blocker is fixed, and the whole response landed

All from my own differential runs (baseline built from `b3f663a17`).

| Repro | base `check` | this tree `check` |
|---|---|---|
| `d={}`; `d[1]=4`; `d[2]=2.5` | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `d={}`; `d["a"]=1`; `d["b"]=2.5` | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `d={}`; `for n in nums: d[n]=n`; `d[0]=1.5` | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `d={}`; `d[1]=Derived(…)`; `d[2]=Base(…)` | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `d={}`; `d[1]=Node(5)`; `d[2]=make(flag)` (`Node\|None`) | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `d={}`; `d[1]="a"`; `d[2.5]="b"` | 1× `0002` + `0008` | **exactly 1× `0002` + `0008`** (2 errors total) ✓ |
| `if/else` divergent shapes | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `try/except` divergent shapes | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |
| `while` loop + trailing `str` write | `SIFR-TYPE-0008` | **`SIFR-TYPE-0008`** ✓ |

Mechanism confirmed by reading, not just behavior:

- **Exact shapes recorded** — `FunctionEnv::record_dict_write` (`state_collection.rs:125-135`) stores `Some((key, value))` and collapses to `None` on the first disagreeing write; `exact_dict_write_hints()` (:150-162) exposes only unanimous shapes. Call site `expression_inference.rs:33`, before any unification.
- **Merged through control flow** — `merge_exact_dict_writes` (:137-148) is invoked from `merge_env_types` (`expression_inference.rs:89`), which is the single merge point for all nine `If`/`While`/`For`/`Try`/`Match` sites (`state_collection.rs:582,590,606,619,629,659,669`; `compound_statement_inference.rs:164,180,194,205,242`). Branch envs are clones, so a differing branch write poisons to `None` in both directions.
- **Reset on direct rebinding** — `bind_var` :107 and `bind_call_result` :121 both `remove(name)`.
- **Gate requires exact equality** — `statement_dispatch.rs:128-131` retains a candidate only when `binding_hints.get(name) == exact_dict_write_hints.get(name)`, i.e. only when `unify_types` did not widen. Widening/optional/branch/loop shapes therefore drop out of the safe set and fall back to base's forward specialization plus `SIFR-TYPE-0008`, as the table shows.
- **Both HIR nodes concrete, including nested-function blocks** — `control_flow.rs:381-405` no longer conditions on `!allow_general_hint` (pass 4 §5 asymmetry resolved); `Let.ty`, `DictLiteral.ty`, and `empty_dict_specializations` are set from the same `adopted_hint_ty`. Pinned by `expressions_tests/empty_plain_dict_inference.rs:140-146`.
- **Homogeneous / read-before-write still adopts** — `0001_two_sum` shape emits `let mut prevMap: HashMap<i64, i64>` and runs correctly; the pinned corpus fixture `verification/…/leetcode/src/0001_two_sum.sifr` and `crates/sifr/tests/e2e/pass/empty_plain_dict_write_inference.sifr` both `run` exit 0 natively.
- **Pass 4 §6 / §7 addressed** — the census now carries the reasoning comment (`empty_plain_dict_inference.rs:31-33`) and `empty_collection_literal_kind` is bound once (`statement_dispatch.rs:102`).

## Verified: no regression in the requested regression surfaces

- **Same-name lexical isolation** — `if flag: d={}; d["s"]=1 … else-path d={}; d[7]=8` emits `HashMap<String,i64>` and `HashMap<i64,i64>` and runs correctly; base fails `build` with 2 rustc errors. Strict improvement.
- **Nearest-declaration patching** — I could not construct a mis-patch. Reverse iteration + `pending.remove` (`container_literal_specialization.rs:273-289`) is correct because patches drain after every statement (`statement_dispatch.rs:174,185`), so at most one candidate declaration for a name is live, and reverse order picks the nearest *preceding* `Let` rather than descending into an earlier sibling block. A three-level probe (outer `total`, write inside a doubly-nested `if`, plus a same-named sibling-branch declaration) fails `build` on base and produces correct results (`2/0/1`) here.
- **Declaration-local codegen typing** — `local_binding_registry.rs:12-20` ambiguity drop is function-scoped (`local_binding_types` is saved/cleared/restored at `function_like_lowering.rs:24,34,114`, `class_method_emitter.rs:596,611,742`, `class_emitter.rs:416,424,445`). I probed every consumer class that could lose an entry — optional narrowing (`condition_lowering.rs:161`), nested subscript assignment, `None`-widened collection methods. Generated-Rust error sets are strict *subsets* of base's in every case (`5×E0308 2×E0599` → `3×E0308`; `3×E0282 4×E0308 2×E0599` → `3×E0308`); no new error kind appears anywhere.
- **Plain-dict missing-key read semantics** — `return d[99]` on an adopted dict still yields `int | None` and the identical `SIFR-TYPE-0002` as base. `d[3] = d[3] + 1` still rejected on both (message improves from `Any | None` to `int | None`). Missing-key `del d[k]` behaves identically on both trees.
- **list / set / deque / defaultdict boundaries** — a fixture exercising `[]`+`append`, `set()`+`add`, `deque()`+`append`, and `defaultdict(int)`+`counts[n] += 1` produces **byte-identical `emit` output** on both trees. No `dict` annotation is introduced anywhere in the diff.
- **`d.setdefault(3, 0)` before the first plain write** now checks and runs correctly (returns 2) where base rejected on `Any` key capability — an intended order-independence gain.
- Module-level `table = {}` / `table[1] = "a"` is byte-identical to base (pre-existing `Box<dyn Any>` `E0277`); chained `a = b = {}` and `global` rebinding are byte-identical (both ineligible by construction).

## Independently re-verified clean on this exact tree

| Check | Result |
|---|---|
| `cargo test -p sifr_lowering` | **897 passed, 0 failed, 1 ignored** (matches ledger :307; pass 4's off-by-one is resolved) |
| `cargo test -p sifr_codegen` | **934 passed, 0 failed** |
| focused wave-3 modules | 12/12 lowering + 2/2 codegen |
| `cargo clippy -p sifr_lowering -p sifr_codegen -- -D warnings` | exit 0 (the `--all-targets` warnings that exist are all in files this diff does not touch) |
| `cargo fmt --check` | clean |
| `check_hir_maintainability_guardrails.py` / `check_file_size_guardrails.py` | PASS / PASS (3008 files, limit 900) |
| `git diff --check` vs base | clean |
| native `run`: new e2e fixture, `0001_two_sum` | exit 0, exit 0 |

Touched-file sizes: `statement_dispatch.rs` 890, `scope_and_function_types.rs` 866, `control_flow.rs` 858, `mod_context.rs` 779, `state_collection.rs` 721, `expressions_tests/empty_plain_dict_inference.rs` 152, `empty_plain_dict_inference.rs` 109, `local_binding_registry.rs` 47 — all under cap. Submodule pointers are unchanged vs base.

Per the prompt I did not re-run the full 677-fixture e2e suite; pass 4 ran it on the pre-delta tree and the tests, focused e2e, guardrails, and ~30 differential probes above cover this corrective delta.

---

## Optional suggestions (non-blocking)

1. **Nested-function blocks are outside the new exact-write gate.** `data={}` + a nested `def` + `data[1]=nested()` + `data[2]=2.5` checks clean and fails `build` with `E0308` — **identically on base and this tree**, because `allow_general_hint` alone can adopt a *unified* hint (`statement_dispatch.rs:104-107`) and `inferred_empty_dict_ty` stays `None` there. Not a regression and provably unaffected by this diff, but the wave's "widening falls back to `SIFR-TYPE-0008`" claim holds only in nested-function-free blocks; worth one ledger sentence so the boundary is not later mistaken for a guarantee. `expressions_tests/empty_plain_dict_inference.rs:128-137` already pins the `TYPE_MISMATCH` variant of this path.
2. **`merge_exact_dict_writes` iterates only `source` keys** (`state_collection.rs:138`). Safe today because every branch env is a clone of the parent, so `source ⊇ target`. A one-line comment would keep that invariant explicit if a non-clone merge is ever introduced.
3. **Exact shapes are not `collapse_literal`d** while `binding_hints` are (`expression_inference.rs:834-835`), so a `LiteralInt`/`LiteralStr`-typed key or value silently disables adoption. Conservative and safe, only a missed opportunity.
4. **Ledger continuity:** the Wave 3 row (`plans/issues/active/…-preexisting-failures.md:307`) records passes 1–4 but not pass 5's zero-output timeout. This issue's own precedent (the pass-2 and passes-8-to-10 dispositions) requires naming discarded passes explicitly. Also `plans/reviews/active/…-wave-3-claude-opus-review-pass-6.md` is present and zero bytes; I left it untouched per the no-write constraint.

---

**CHANGES REQUIRED** — finding 1 must be addressed (plus its test) before this wave merges. Everything else in the delta is correct, and the pass-4 blocker with its five confirmed families is genuinely closed.

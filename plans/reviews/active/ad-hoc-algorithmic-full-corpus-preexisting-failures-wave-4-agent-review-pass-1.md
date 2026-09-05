## Review: Wave 4 — `defaultdict(int)` augassign key specialization (PR #3079, `cb2179438` vs `789b35973`)

I inspected the full diff, the consuming patch machinery, the codegen `entry`/`or_insert` path, and Wave 3's gating for the analogous problem. I then built both head and base and ran differential probes.

**Independently verified claims (all hold):** focused lowering 5/5 and focused codegen 2/2 (899 and 934 filtered → totals 904 / 936, matching the stated 903+1 ignored and 936); the new e2e fixture `crates/sifr/tests/e2e/pass/defaultdict_int_augassign_key_refinement.sifr` builds and runs; corpus `0350` and `0621` build and run at head; alias/int-only gating, literal widening, initialized-alias preservation, conflicting-key rejection, and `entry(k).or_insert(0)` preservation all behave as specified in the non-shadowed shape; `float` and `list[int]` keys are rejected with exactly one `SIFR-TYPE-0002` each.

---

## Actionable findings

### A1 — High: the new pending patch is name-keyed with no declaration-safety gate, so it retargets a nested function's shadowing declaration and produces generated-Rust compile errors (regression vs. base)

`crates/sifr_lowering/src/lower/defaultdict_refinement.rs:46` inserts into `ctx.pending_container_specialization_patches` keyed only by the variable's *name*. The consumer, `crates/sifr_lowering/src/lower/container_literal_specialization.rs:281`, walks the enclosing statement list in reverse and **descends into `HirStmt::NestedFunction` bodies** (`container_literal_specialization.rs:348`), claiming the first `Let` with a matching name — which may be a completely different binding — and then overwrites its declared type unconditionally at `container_literal_specialization.rs:289` (`*ty = patch_ty.clone();`) regardless of what the value expression is.

This is precisely the leak class Wave 3 pass 1 identified and fixed for the plain-dict path via `empty_plain_dict_inference::safe_hint_names_for_block` / `nested_block_binds_name` (`crates/sifr_lowering/src/lower/empty_plain_dict_inference.rs:6`, `:71`). Wave 4 bypasses that gate entirely.

**Failure scenario 1 — two independent `defaultdict(int)` bindings, different key types:**

```python
def solve(words: list[str], nums: list[int]) -> int:
    counts = defaultdict(int)
    def helper() -> int:
        counts = defaultdict(int)
        for n in nums:
            counts[n] += 1        # correctly refined to int keys...
        return len(counts)
    for w in words:
        counts[w] += 1            # ...then clobbered to str by the outer patch
    return len(counts) + helper()
```

Head emits `let mut counts: HashMap<String, i64> = HashMap::new();` **inside `helper`** next to `counts.entry(n.clone())`, and leaves the outer declaration as bare `HashMap::new()`. Result: `SIFR-BUILD-0005` / rustc `E0308`. Base (`789b35973`) compiles and runs this program successfully.

**Failure scenario 2 — the shadowing binding is not a container at all:**

```python
def solve(words: list[str]) -> int:
    counts = defaultdict(int)
    def helper() -> int:
        counts = 7
        return counts
    for w in words:
        counts[w] += 1
    return len(counts) + helper()
```

Head emits literally `let mut counts: HashMap<String, i64> = 7_i64;` → two rustc errors (`E0308`, `E0277`). Base compiles and runs.

Both shapes require only the ordinary Python layout of "declare, define helper, then use", so this is reachable, not contrived. Two of the wave's stated requirements are violated here: *"update declaration and constructor-call HIR consistently"* (the real declaration is never patched) and the "if it compiles, it works" guarantee (a previously-working program now fails inside generated Rust).

Root-cause direction: gate the insertion the way Wave 3 does (only when the current lexical block has a single unshadowed binding for the name and no nested block rebinds it), and/or key the patch by `binding_id` rather than name, stop `patch_stmt_container_specialization` from descending into `NestedFunction` bodies that rebind the name, and stop the unconditional `*ty = patch_ty` when the value expression doesn't match the patch shape (`container_literal_specialization.rs:289` should move inside the shape match).

### A2 — Medium: missing negative coverage for the failure class above and for the new gate's boundaries

`crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs` covers variable str/int keys, literal widening, conflicting keys, and the initialized alias. It has no coverage for:

- a nested function (or any block) shadowing the refined name — the A1 bug (`defaultdict_augassign_refinement.rs:78`, end of file);
- `defaultdict(list)` / `defaultdict(set)` remaining untouched by the new path, i.e. the `name != DEFAULTDICT_INT_ALIAS` guard at `defaultdict_refinement.rs:20`;
- unhashable / `float` key rejection on the new path. This matters beyond the generic case: the refinement now makes `key_ty` concrete *before* `validate_subscript_augassign_target` runs, so which of the two `reject_unavailable_hash_key` calls fires (`container_literal_specialization.rs:201` vs `:207`) has flipped. Diagnostic cardinality happens to stay at one, but nothing pins it;
- plain-dict missing-key rejection remaining unexpanded (asserted in the wave description, not tested here).

---

## Non-blocking observations

- **N1** — The unconditional `*ty = patch_ty.clone();` at `container_literal_specialization.rs:289` predates this wave, but Wave 4 is what makes the value-shape-mismatch variant (A1 scenario 2) reachable. Worth fixing at the same time.
- **N2** — Alias `type_args` handling is inconsistent: the new refiner preserves them (`defaultdict_refinement.rs:36`) while the neighbouring `refine_defaultdict_binding_expr` discards them (`defaultdict_refinement.rs:106`, `Vec::new()`). Harmless today (defaultdict aliases carry none) but the divergence invites confusion.
- **N3** — The Any/Unknown guard on the inferred key omits `resolve_alias()` (`defaultdict_refinement.rs:29-32`), unlike the sibling refiners (`empty_collection_refinement.rs:47`, `:145`). `widen_literal` also doesn't resolve aliases. I could not construct a reachable trigger (an alias whose body is `Any`), so this is consistency-only.
- **N4** — `ctx.scope`, flow narrowing, and the pending patch are all mutated at `defaultdict_refinement.rs:39-47` *before* `validate_subscript_augassign_target` can reject the key (e.g. unhashable). No user-visible effect since diagnostics abort the build, but the ordering leaves the context in a state that describes a rejected program.
- **N5** — Generated `counts.entry(item.clone())` for `i64` keys (pinned at `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:22`) is a redundant clone on a `Copy` type. Correct, just noisy output.
- **N6** — Ledger: the Wave 3 table row in `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:312` still reads `review; PR #3077` while the prose added at `:334-336` states it merged at `789b359737` and supersedes the row. The prose reconciles it explicitly, so this is cosmetic, but the row is stale.
- **N7** — File-size and maintainability are fine: `defaultdict_refinement.rs` is 127 lines; no touched file approaches the 900-line cap.

---

CHANGES REQUESTED

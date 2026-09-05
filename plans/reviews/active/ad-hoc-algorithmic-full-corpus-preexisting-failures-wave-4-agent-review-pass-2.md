## Review: Wave 4 — `defaultdict(int)` augassign key specialization (PR #3079, `8acf7ad01` vs `789b35973`)

I read the full diff, the patch/refinement machinery and its callers, then built **both** head and base and ran differential probes plus the repo's own gates.

### Verified claims (all hold)
- `cargo test -p sifr_lowering` → **907 passed, 1 ignored**; `-p sifr_codegen` → **938 passed**; the 9 new lowering and 4 new codegen tests are real and pass.
- Full native e2e suite: **678/678 passed** (`verification/runner/e2e/run_e2e_pass.sh`, 322 s).
- `cargo fmt --check`, `check_hir_maintainability_guardrails.py`, and lib-level clippy on both touched crates are clean; no touched file is near the 900-line cap (largest: `container_literal_specialization.rs` 313).
- All 7 corpus fixtures using `defaultdict(int)` (`0350`, `0383`, `0474`, `0621`, `0767`, `1189`, `1481`) check/build/run at head (`0621`'s only output is a pre-existing overflow **warning**).
- Pass-1's A1/A2 are genuinely fixed: nested `defaultdict` shadow gets its own `HashMap<i64,i64>`, the scalar shadow emits `let counts: i64 = 7_i64`, `defaultdict(list)`/`defaultdict(set)`/seeded aliases emit **byte-identical** Rust at head and base, and the float-key diagnostic is exactly one.
- **The specific traversal question**: direct-only patching still reaches an outer declaration whose evidence is in a loop, an `if` body, or a `try`/`finally` body — each inner block is lowered by its own `lower_stmts`, which applies patches per statement (`statements/statement_dispatch.rs:185`), so the pending patch survives up to the declaring block. Verified by emit: `for`/`if` (`HashMap<String, i64>` emitted) and `try/finally` (head annotates where base did not). Declarations *inside* a nested block are unreachable from outside anyway — Sifr does not hoist branch-local bindings (a plain `if flag: x = 1 else: x = 2; return x` fails to build at head *and* base), so the removed recursion has no legitimate target.
- No regressions found: `emit` output is identical head-vs-base for a nesting battery (loop-local decl, nested-function decl, while-decl, list decl, nested-if decl), for plain-dict/list/scalar shadow programs, and for `defaultdict(list|set|int, seed)`. Missing-key read semantics are preserved (`*counts.entry("missing".to_string()).or_insert(0)`), identical to base. A base-vs-head `check` sweep over the 411-file corpus is still running; the first 132 files are identical.

---

## Actionable finding

### F1 — Medium: the refined declaration is silently dropped when a same-named binding appears later inside the same enclosing compound statement

`refine_defaultdict_int_augassign_key` records the specialization *only* as a deferred, name-keyed pending patch (`crates/sifr_lowering/src/lower/defaultdict_refinement.rs:46-47`). Any new binding of that name clears the pending entry before the enclosing block ever gets to apply it (`statements/patterns_and_assignments.rs:288`, `:468`, `:566-606`). Because patches are applied per statement at each block level, a nested function defined *after* the refining augassign but *inside* the same `if`/`for` body consumes and discards the outer declaration's patch.

Repro (`/tmp/rev/f1.sifr`), head emit:

```rust
fn solve(words: &Vec<String>, nums: &Vec<i64>) -> i64 {
    let mut counts = HashMap::new();            // <-- refinement lost, no annotation
    if ((words.len() as i64) > (0_i64)) {
        for w in words.iter().cloned() { ... counts.entry(w.clone()).or_insert(0) ... }
        let inner = || {
    let mut counts: HashMap<i64, i64> = HashMap::new();   // inner is correct
```

Consequences:
- The outer declaration's HIR type stays `alias<dict[Any, int]>`, inconsistent with both the refined scope type and the constructor-call type — the wave's "patch declaration and constructor-call HIR consistently" goal is not met in this shape.
- Codegen silently reverts to the pre-wave shape (bare `HashMap::new()`), i.e. exactly the state this wave exists to fix; it only survives because the augassign itself pins `K`/`V` for rustc inference.
- The analogous plain-dict path does **not** have this hole: Wave 3's up-front binding inference keeps `HashMap<String, i64>` on the outer declaration in the identical program shape (verified, head == base emit). So the two waves now use different-strength mechanisms for the same problem.

Impact bound honestly: I could not construct a miscompile from it — `f1`, `leak3`, `leak5`, `leak7` all build and run at head, and head is never worse than base on any probe (`leak7` fails on base, passes at head). So this is a correctness/consistency gap and a latent fragility, not a live wrong-code bug.

Coverage gap (the reason this survived pass 1's fix): all three new shadow regressions place the shadowing declaration **before** the refining augassign — `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:84`, and e2e `defaultdict_int_augassign_key_refinement.sifr:18-29` and `:32-40` — which is precisely the ordering that works. Nothing pins the reversed order.

Suggested direction: derive the declaration type the way Wave 3 does (block-scoped binding lookup with a shadow gate) instead of relying solely on the pending map; or, at minimum, add a lowering/codegen regression pinning the reversed order and record the limitation explicitly in the ledger rather than claiming unconditional declaration/constructor consistency.

---

## Non-blocking observations
- **N1** — Pass-1's N2/N3/N5 remain unaddressed (alias `type_args` preserved here but discarded in the sibling `refine_defaultdict_binding_expr:106`; no `resolve_alias()` on the Any/Unknown guard at `defaultdict_refinement.rs:26,30`; redundant `.clone()` on `Copy` keys in `entry(item.clone())`). Also note `refine_defaultdict_binding_expr:100` still takes `index.ty()` un-widened while the new refiner widens — a divergence worth unifying eventually.
- **N2** — `container_literal_specialization.rs:298-303` shadows the outer `name` (the `Let` name) with the alias `name` inside the guard; correct, but the `func == name` comparison reads as if it compares against the binding.
- **N3** — Pre-existing, untouched by this wave, but adjacent and worth logging: a `defaultdict(int)` with no augassign evidence (`c = defaultdict(int); return len(c)`) and `deque()` + `append` both fail identically at head and base with rustc `E0282`/`E0425` under `SIFR-BUILD-0005`.
- **N4** — Pass-1's N6 (stale Wave 3 ledger row) is resolved; the row now reads `merged`.

---

**Verdict: CHANGES REQUESTED** (one actionable finding, F1 — Medium).

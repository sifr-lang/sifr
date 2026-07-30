## Verdict: **APPROVE with findings** — no regressions found; 6 non-blocking findings (1 medium, 5 low)

Every stated requirement holds on exact head `e170b9d53`. All findings are undercoverage/consistency, not regressions; the medium one is a reproducible raw-rustc leak in this wave's exact feature area that pre-dates the change.

### Independently verified

| Requirement | Result |
|---|---|
| Corpus fixtures use `own mut`; PR #43 pinned at `9d71595…` | ✓ gitlink at head = `9d71595347a369ef3a4f8d90a0a01508b591369a`, parent `d50fa7350`, on `origin/main`; diff is exactly 2 files / 2 lines (`own mut l1/l2`, `own mut head`); `third_party/ruff` unchanged from base; submodule dirt is untracked artifacts only |
| Owned recursive destructures bind mutably before `.take()` | ✓ `let Some(mut a), …` + `a.next.take().map(…)` in emitted Rust across let-else, if-let, and or-tuple shapes |
| Shared/mut-borrowed recursive options stay borrowed | ✓ `fn nextNode(node: &mut Option<LinkedNode>) { let Some(node) = node else …; (node.next).as_deref().cloned() }` — non-moving, no `mut`; same for shared borrow, incl. nested functions |
| Non-recursive optional classes gain no mutability | ✓ `let Some(value) = value else` |
| 22 linked-list fixtures + `0617` check/build/run | ✓ 23/23 green with this tree's compiler (0 failures) |
| No fallback/suppression/baseline/helper workaround | ✓ diff is crates + plans + gitlink only; `helpers/list_node.sifr` and both local `nodeNext` copies untouched; no `#[ignore]`, waiver, or baseline edit |
| Local gates | ✓ reran: codegen 954/954, `clippy -p sifr_codegen -D warnings`, `cargo fmt --check`, HIR guardrails PASS, file-size PASS (limit 900), submodule ownership PASS. e2e 679/679 accepted from the report (not re-run) |

Adversarial probes that passed (all in `/tmp`, nothing in the repo touched): if-let narrowing, `and`-conjunction chain, `or`-disjunction tuple let-else, truthiness alias, nested function with `own` recursive param, nested function with shared-borrow recursive param, nested TypeVar compare, several nested Copy-param shapes (emit byte-identical to `own` variants).

---

### Findings

**1. Medium — mutually recursive (SCC) owned optional classes still emit an immutable binding → raw rustc leak**
`crates/sifr_codegen/src/lower_stmt/simple_dispatch_and_bindings.rs:510` (`class_has_recursive_option_field`), reached from `condition_type_and_expr_helpers.rs:33`.
The new predicate only recognizes a class whose field union names *itself*. Boxing, however, is SCC-based (`field_analysis_helpers.rs:128-155`), and the structured path's equivalent (`stmt_support_emitter/condition_lowering.rs:152` `option_binding_requires_mut_for_ir`) consults the SCC-derived `recursive_fields` registry — so the two paths disagree.
Reproducer (`Branch.leaf: Leaf | None`, `Leaf.branch: Branch | None`, both narrowed via `own`):
```
error[E0596]: cannot borrow `b.leaf` as mutable, as `b` is not declared as mutable
  let Some(mut b) = b else {   ← rustc's own suggestion
error[SIFR-BUILD-0005]
```
Pre-existing (before the diff, mut came only from `mutated_vars`, so the same program failed identically), but it is exactly the defect class this wave fixes, and it violates the no-raw-rustc contract.
*Required fix:* make the simple path use the same recursive-class knowledge as `option_binding_requires_mut_for_ir` — thread the emitter's `recursive_fields` (or an SCC-recursive class-name set) into `SimpleStmtBindings` — with a focused codegen test for mutually recursive owned optional narrowing. If deferred instead, record it explicitly in the issue ledger as a newly identified pre-existing defect with an owning wave rather than leaving it undocumented.

**2. Low — two divergent predicates for one decision (`should_force_mutable_binding` is not an adequate recursive-class predicate)**
`condition_type_and_expr_helpers.rs:32-46` vs `stmt_support_emitter/condition_lowering.rs:152-172`. They differ in *precedence* (new: borrow-exclusion before `mutated_vars`; structured: `mutated_vars` first) and in *predicate* (new: `should_force_mutable_binding`, which also fires for `Iterator`, `JoinSet`, `__sifr_defaultdict_*` aliases and `__next__`-protocol classes; structured: recursive-class lookup). Note `should_force_mutable_binding` already exists as two byte-identical copies (`stmt_support_emitter/expr_call_metadata.rs:159`, `simple_dispatch_and_bindings.rs:489`); this adds a third consumer. *Fix:* one shared helper used by both lowering paths.

**3. Low — mutability is type-based, not use-based (over-broad, and the ledger overstates it)**
`mut` is emitted for *every* owned narrowing of a recursive-class option, including read-only ones — e.g. `if let Some(mut t) = t { println!("{}", t.val) }` in a probe's `main`, which `ir_optimize`'s `IfLet` demotion (`mutability_and_clone_rewrites.rs:120`) did not strip. Impact is cosmetic only: `unused_mut` is allowed by `verification/areas/generated_code_quality/generated_code_quality.py:114`. But the ledger line 333 claim — "emits a mutable binding **only for** owned recursive class values **whose child extraction uses `.take()`**" — is not what the code does; there is no use-site analysis. *Fix:* correct the ledger wording (or add the use-site condition).

**4. Low — nested param sets omit the Copy-ownership filter applied everywhere else**
`simple_dispatch_and_bindings.rs:555-566` filters on convention only, while `function_like_lowering.rs:39-48` and `class_method_emitter.rs:638` additionally require `ty.ownership() != OwnershipKind::Copy`. Since the default convention is `borrow()`, a nested `def f(a: int)` now enters `nested_borrowed_params`, which feeds `expr_uses_borrowed_name` (simple-path bail) and the match-arm copy-capture deref path. I found no observable divergence (5 shapes emit identically to their `own` variants), so severity is low. *Fix:* mirror the ownership filter; add a nested-function codegen pin.

**5. Low — new coverage pins only one of five rerouted shapes**
The diff routes if-let (`condition_lowering.rs:113`), the and-chain (`lower_if_not_none_chain`), the or-tuple let-else (`condition_lowering.rs:32`), the truthiness alias (`:126`), and nested-function lowering through `option_binding_pattern`, but the only new positive test is the simple let-else. I confirmed all shapes emit correct `mut` and run correctly, so this is coverage, not a defect. *Fix:* add assertions for `if let Some(mut …)`, `(Some(mut a), Some(mut b))`, and a nested-function `own`-recursive positive. (The three negatives are mutation-sensitive — deleting either borrow-exclusion clause makes them fail — so they are real guards.)

**6. Low — nested-block lowering still evaluates the new predicate blind**
`try_tuple_flow.rs:261` (`try_lower_simple_stmt_block`) and `simple_dispatch_and_bindings.rs:19` (`try_lower_simple_stmt_with_ctx`) still pass `&HashSet::new()` / `&HashMap::new()` for `mut_borrowed_params` and `local_binding_types`, so any narrowing lowered through them cannot see types and forces no `mut`. I could not build a reproducer — both nested-block probes fell back to structured lowering, which is correct — so the fix's coverage inside nested blocks currently depends on that fallback rather than being guaranteed. *Fix:* thread the full bindings through block lowering.

---

### Non-actionable observations

- **Corpus artifact SHA:** the artifact approves `b1170d7`, the ledger/gitlink pin merged `9d71595`. Verified equivalent — identical trees (`74a2d04b…`) and identical parent — so the evidence stands; traceability nit only.
- **Pre-existing raw-rustc leaks surfaced while probing (none caused by this diff, all reproduce with `own` too):** mut-borrow optional-class field mutation emits `as_ref()` → `E0594` (`stmt_support_emitter/condition_lowering.rs:129-141`); `match` inside a nested function emits `compile_error!("structured statement emission missing…")`; moving an owned recursive option inside a loop body escapes ownership checking → `E0382`; generic recursive class fields are not boxed → `E0072`. Worth adding to the ledger's pre-existing surface if not already tracked elsewhere.
- `simple_dispatch_and_bindings.rs` is at 877/900 lines — little headroom for the finding-1 fix; prefer extracting the shared predicate into its own module.
- `sifr check <relative-path>` from inside the corpus `src/` mis-resolves `helpers.*`; absolute paths pass. Harness quirk, not a fixture defect.

I modified no files, branches, commits, PRs, or repository state; all probes ran in `/tmp/w7probe`.

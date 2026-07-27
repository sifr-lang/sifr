Reviewed the full `082988df1` → working-tree diff, re-ran every gate, and re-probed the round-2 blocker plus adjacent shapes with real generated package builds. Review follows (stdout only — no files written).

---

## Independent review — Rust-interop Track A, `certification_1` (round 3)

Baseline `082988df1`; uncommitted working tree as reviewed. No files modified. `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` ignored per scope.

### Re-evaluation of the round-2 blocker

**B3-residual — top-level list containers bypassed recursive conversion — RESOLVED.**

I did not rely on the unit tests. I built and ran real packages against contract-correct bridges (`SIFR_SYSROOT` at the repo root, release builds, asserts live). Every shape below compiles and round-trips in both directions:

| Probe | Shapes | Result |
| --- | --- | --- |
| A | `list[dict[str,str]]`, `list[str]`, `list[bytes]`, `list[tuple[str,int]]`, `dict[str,list[dict[str,str]]]`, `list[str] \| None`, `dict[str,str] \| None`, `list[dict[str,str]] \| None` | `probeA-ok` |
| B (with `sifr_runtime` path dep so `SifrIntBridge` is nameable) | `list[list[int]]`, `dict[str,list[int]]`, `list[dict[str,int]]`, `dict[str,dict[str,list[int]]]`, `list[int] \| None`, `list[int]`, `int \| None` | `probeB-ok` |
| C | `float\|None`, `bytes\|None`, `str\|None`, `list[str\|None]`, `list[int\|None]`, `dict[str,int\|None]`, `Result[dict[str,list[int]], E]`, `Result[list[dict[str,str]], E]` | `probeC-ok` |
| E | `own` variants of `list[dict[str,str]]`, `dict[str,list[int]]`, `list[list[int]]`, `list[int]\|None`, `list[str]`; plus `list[float]`, `dict[str,tuple[str,int]]` | `probeE-ok` |

The two exact E0308 cases round 2 blocked on (`list[dict[str,str]]` → `&[IndexMap<String,String>]`, `list[list[int]]` → `&[Vec<SifrIntBridge>]`) now build clean in both directions.

The dispatch is also correct by construction, not by luck. `composite_conversion_required` (`rust_interop_direct.rs:125`) returns true exactly for `Int`, `List` with a converting element, string-keyed `Dict`, and `Option` with a converting payload. Everything it returns false for — `bool`, `float`, `str`, `bytes`, fixed ints, tuples, classes/enums/handles — has a bridge owned type identical to the Sifr representation (`bridge_type_contract`, and `tuple_item_rust_type` admits scalars only, mapping `int → i64` rather than `SifrIntBridge`), so identity is the right conversion. Borrowed/owned split is right: `value_iter` picks `iter()`/`into_iter()`, the outer `&` is added only for `List`/`Dict` (bridge borrowed types are `&[T]` / `&IndexMap<..>`) and deliberately *not* for `Option` (bridge borrowed type for `Option` is the owned `Option<T>`) — probes A/C/E confirm all three. `bridge_result_expr` recurses into `direct_rust_return_expr`, so `Result`-wrapped composites convert too; the new `_ if composite_conversion_required` arm is correctly ordered before `Type::Result` yet cannot swallow it (`Result` → `_ => false`). Retaining `is_int_list`/`is_optional_str`/`is_optional_int` ahead of the composite arm is behaviour-preserving.

Round-2 optional items also cleared: `_require_root_lock_subset` and the "Cargo.lock is required" rule are hoisted out of the `bridge_type_matrix` branch (`_scenario_checks.py:362-386`). I confirmed the hoist empirically on an unrelated scenario — injecting `memchr 99.99.99` into `ecosystem_cli_certification`'s lock produces `... is not present in root Cargo.lock`, and deleting it produces `... Cargo.lock is required`. The word "ordered" is gone from every bridge doc (`grep -rni ordered` over docs/internal_docs/verification/plans leaves only unrelated hits). The scenario lock's 17 sourced packages are all present in the root lock, `serde_derive 1.0.228` included.

### Independently reproduced gates

- `cargo test -p sifr_codegen rust_interop_direct --lib` → 25 passed, 0 failed.
- `cargo test -p sifr_driver --lib -- --ignored test_build_bridge_type_matrix_positive_cargo_probe` → 1 passed (18.8 s); the test now also asserts `check_package_project(pristine).is_empty()` (round-1 optional #3 closed).
- Full area runner → `variants=10, failures=0, blocking_failures=0`; self-test `cases=90`; `fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=11`; `claims=24`.
- `cargo fmt --all --check`, `git diff --check`, `cargo clippy -p sifr_codegen --lib -- -D warnings`, `check_file_size_guardrails.py` (2853 files, limit 900), `check_hir_maintainability_guardrails.py` → all pass. Sizes: `rust_interop_direct.rs` 873, collections module 260, `_scenario_checks.py` 728.
- Post-item inventory recomputed from the data files: 36/36/36 rows-manifests, 48 passing / 24 planned, categories 17/6/1/12, execution kinds 13/4/10/9, 44 crate aliases, 24 claims — every number in the issue block matches.
- Fixture positive `.sifr` vs scenario `src/main.sifr` differ only in header lines and the verifier name.

---

## Blocking finding

**B5 — composite conversion interpolates the raw parameter name into a verbatim expression, so a Rust-keyword parameter name causes an internal compiler panic on the exact shape this row certifies.**

`sifr_composite_to_bridge_expr` (`rust_interop_direct_collections.rs:5-15`) builds `RustExpr::Verbatim(sifr_value_to_bridge_expr(name, …))` from `param.name` verbatim (`rust_interop_direct.rs:87`). `RustExpr::Verbatim` renders through `render_compiler_path_string` only, which does **not** apply raw-identifier escaping; `RustExpr::Ident` renders through `render_identifier` (`render_expr_and_blocks.rs:593`), which emits `r#type`. Every other arg path builds an `Ident`, so only the new composite path is affected.

Reproduced end-to-end, twice, on the certified scenario:

```
# def dict_kw(type: dict[str, str]) -> dict[str, str]      (bridge: &IndexMap<String,String>)
thread 'main' panicked at crates/sifr_codegen/src/lib_modules_and_codegen.rs:711:5:
codegen IR validation failed (assembled file): compiler-owned verbatim Rust expression is invalid: expected an expression
error[SIFR-INTERNAL-0001]: internal compiler panic during project code generation

# def opt_float(type: float | None) -> float | None        (bridge: Option<f64>)
… identical SIFR-INTERNAL-0001 panic
```

Control: the same keyword names on the paths this round deliberately left alone all build and run green (`def echo(type: str)`, `def echo_list(match: list[str])`, `def echo_opt(move: str | None)` → `probeJ-ok`). So this is specific to the composite route, and it fires on `dict[str, T]`, `list[dict[…]]`, nested exact ints, and non-`str`/`int` `Option` payloads — precisely the set `bridge_type_matrix` is promoted to certify.

Why this blocks rather than carries: `type`, `match`, `ref`, `move`, `box`, `impl`, `mod`, `use`, `fn`, `let`, `loop`, `where`, `unsafe`, `const`, `static`, `struct`, `enum`, `trait`, `pub`, `dyn`, `extern` are all legal Sifr/Python parameter names and Rust keywords; `type` and `match` are ordinary. The outcome is a compiler ICE with a panic on stderr — a direct hit on both "no user-triggerable panics" and "if it compiles, it works" — on a row this PR moves from `future-owned-by-separate-phase`/`planned` to a `supported-through-bridge` stable claim with `passing` executable evidence. That is the identical scope argument round 2 applied to B3-residual: the promotion is what pulls the defect inside the certified contract. (For `dict` shapes the baseline failed too, with E0308 rather than a panic; I did not build the baseline compiler to settle whether `float | None` compiled there, and the verdict does not depend on it.)

Fix (small, local): escape the root name before interpolation — pass `Renderer::render_identifier(&param.name)` into `sifr_composite_to_bridge_expr`, or take a `&RustExpr` and render it, the way `bridge_composite_to_sifr_expr` already does for the return side. The generated binder names (`__sifr_bridge_item_N`, `__sifr_bridge_key_N`, …) are compiler-owned and safe; only the root is user-controlled. Add a unit test asserting `r#type` in the rendered argument, and ideally a keyword-named parameter in the scenario so the cargo probe observes it.

---

## Non-blocking findings

1. **Borrowed composite conversion requires the element type to be `Clone`.** `sifr_value_to_bridge_expr`'s `_ if borrowed => "{value}.clone()"` fallback means a borrowed `dict[str, OpaqueClass]` clones each `Handle<T>`, and `Handle<T>: Clone` only holds when `T: Clone` (`sifr_runtime/src/interop.rs:246`). A non-`Clone` opaque target would surface a raw rustc error from generated glue. Outside this row's certified claim (opaque handles are their own rows), and strictly better than baseline, which cloned the whole map — but worth a bounded note or a `SIFR-RUST-TYPE-0001` for the case.
2. **No executable coverage for exact-`int` payloads.** Carried from round 2 and still true: the scenario crate cannot name `SifrIntBridge` without a `sifr_runtime` dependency, so the `Type::Int` recursion has unit-string coverage only inside the certified probe. It does work — probe B proves it with a path dep — but the certified probe cannot observe it. Worth stating in the fixture provenance.
3. **Async composite arguments are unexercised anywhere in the area.** No scenario example binds an `async` Rust bridge, so `foo(&<temporary>).await` with a converted collection has no executable coverage. The statement-scoped temporary is sound Rust and the types involved are `Send`, so I rate this coverage, not correctness.
4. **Checklist wording understates the fix.** "Fix generated dict-parameter lowering …" now describes recursive list, dict, exact-int, and `Option` conversion in both directions.
5. **Carried, unchanged:** `bridge_type_matrix` is still inserted out of alphabetical order in `stable_support_claims.json` and the generated docs table; negative evidence still binds the synthetic `set[int]` contract test rather than the checked-in `negative/unsupported_container_rejections.sifr`.
6. **Not a finding, for the record:** `cargo clippy -p sifr_codegen --all-targets -- -D warnings` fails with 14 errors (9 `needless_borrow` at `rust_interop_direct.rs:473-795`, rest in unrelated test modules). All are pre-existing — the same 9 `render_expr(&expr)` occurrences exist verbatim at `082988df1` — and `--all-targets` is not the gated invocation.

---

## NOT SATISFIED

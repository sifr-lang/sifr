Review follows (stdout only — no files written; working tree byte-identical to the state I received).

---

## Independent review — Rust-interop Track A, `certification_1` (round 4)

Baseline `082988df1`; uncommitted working tree as reviewed. Read rounds 1–3 and the full diff. `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` and the user-scoped pre-existing demo/algorithm failures ignored per scope.

### Re-evaluation of B5 — RESOLVED

The fix is structurally correct, not just test-satisfying:

- `sifr_composite_to_bridge_expr` (`rust_interop_direct_collections.rs:5-20`) takes `&RustExpr` and calls `render_expr`, so the root goes through `Renderer::render_identifier_or_compiler_path` → `render_identifier`, which emits `r#…` for every escapable keyword. `direct_rust_arg_expr` (`rust_interop_direct.rs:87`) passes its existing `RustExpr::Ident(param.name)`; `hash_map_to_bridge_index_map_expr` builds an `Ident` before delegating. No raw `param.name` reaches the composite `Verbatim` any more.
- `render_compiler_path_string` cannot damage the escape: it only inserts a leading `::` for an identifier immediately followed by `::` that is in `COMPILER_RUST_PATH_ROOTS`; `r#type` is `r` followed by `#`, so it is left alone. Confirmed in real generated output below.
- `is_escape_required_keyword` covers `type`, `match`, `move`, `ref`, `where`, `loop`, `box`, `impl`, `mod`, `use`, `fn`, `let`, `unsafe`, `const`, `static`, `struct`, `enum`, `trait`, `pub`, `dyn`, `extern`, plus reserved words — i.e. every name round 3 enumerated.

I did not rely on the unit test. I built and ran real packages (debug compiler, source-tree sysroot, release output, asserts live):

| Probe | Shapes / names | Result |
| --- | --- | --- |
| Checked-in scenario | `indexmap_list_roundtrip(type: list[dict[str,str]])` against `&[IndexMap<String,String>]` | builds; prints `serde:nested\|bytes:6\|invalid nested payload`, exit 0 |
| Adjacent keywords | `dict_kw(match: dict[str,str])`, `opt_float_kw(move: float\|None)`, `nested_kw(where: dict[str,list[dict[str,str]]])`, `opt_list_kw(loop: list[str]\|None)`, `list_dict_owned_kw(own box: list[dict[str,str]])` | `kw-probe-ok`, exit 0 |

Generated glue for the certified row, verbatim:

```rust
fn indexmap_list_roundtrip(r#type: &Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
    (bridge::types::indexmap_list_roundtrip(&r#type.iter().map(|__sifr_bridge_item_0| …
```

and `r#match`, `r#move`, `r#where`, `r#loop`, `r#box` all render correctly in borrowed-dict, owned-list, `Option<f64>`, `Option<Vec<String>>`, and depth-3 nested positions. No `SIFR-INTERNAL-0001`, no IR-validation failure. B5's exact two repro cases (`dict[str,str]` and `float | None` under the name `type`/`move`) now compile and run.

The escaping is confined to the root, which is the only user-controlled name; binders (`__sifr_bridge_item_N`, `__sifr_bridge_key_N`, `__sifr_value_N`) remain compiler-owned.

### Recursive behaviour re-verified after the refactor

The round-4 change alters only the argument-side root rendering, but since the helper signature changed I re-probed the round-2/3 shape set end to end with a `sifr_runtime` path dep so `SifrIntBridge` is nameable. One package, all shapes in one binary, `deep-probe-ok`, exit 0:

`list[list[int]]` (named `type`), `dict[str,list[int]]`, `list[int] | None` (Some **and** None), owned `list[dict[str,str]]`, `dict[str,dict[str,list[int]]]`, `dict[str,str] | None`, and `Result[dict[str,list[int]], DiagnosticError]` on both the Ok and Err branches. Generated signatures show the expected `to_i64_saturating()` inverse on returns, `SifrIntBridge::from(*…)` on borrowed elements, `into_iter()`/`iter()` chosen by convention, the outer `&` added for `List`/`Dict` and correctly omitted for `Option`, and `bridge_result_expr` recursing into the Ok payload while the `_ if composite_conversion_required` arm still cannot swallow `Type::Result`.

### Gates reproduced independently

- `cargo test -p sifr_codegen rust_interop_direct --lib` → **26 passed**, including `composite_root_identifiers_escape_rust_keywords` (asserts `&r#type.iter()`).
- `cargo test -p sifr_codegen` (whole crate) → 897 passed, 0 failed.
- `cargo test -p sifr_driver --lib -- --ignored test_build_bridge_type_matrix_positive_cargo_probe` → **1 passed** (19.6 s), exercising the keyword-named `list[dict[str,str]]` parameter and the pristine `check_package_project(...).is_empty()` assertion.
- `cargo test -p sifr_driver --lib` → 387 passed, 0 failed, 41 ignored.
- Fixture-matrix self-test → `cases=90`; all five checkers pass (`fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=11`, `rows=36`, `claims=24`).
- Full area runner → `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`.
- `cargo clippy -p sifr_codegen --lib -- -D warnings`, `cargo fmt --all --check`, `git diff --check`, `check_file_size_guardrails.py` (2853 files, limit 900), `check_hir_maintainability_guardrails.py` → pass. Sizes: `rust_interop_direct.rs` 873, collections module 277, `_scenario_checks.py` 728.
- **Root-lock enforcement**: re-diffed all eleven scenario locks by `(name, version, has-source)` against the root lock — every one is a strict subset, `bridge_type_matrix` included. The rule is hoisted out of the per-fixture branch (`_scenario_checks.py:362-386`), and the `memchr 2.8.0 → 2.8.3` plus missing-lockfile mutations are in the self-test.
- **Provenance**: `sifr_driver_generated_builds` is `status: blocking`, `executed_in_merge: true`, and runs `-- --ignored --test-threads=1` in both `create-pr.json` and `merge.json`; `fixture.json` binds the correct file/test/suite/profile, and the README prose matches the structured record.
- **Inventory recomputed from the data files**: 36/36/36 rows-compat-manifests (all `schema_version: 2`), 48 passing / 24 planned, categories 17/6/1/12, execution kinds 13/4/10/9, 44 distinct crate aliases, 24 claims — every number in the `certification_1` post-item block matches exactly.
- **No-panic**: generated glue for the certified scenario contains zero `unwrap()`, `expect(`, `panic!`, or `unwrap_or_else`; the only `assert!`s are the six from the user-authored evidence. `to_i64_saturating` is the non-panicking inverse.
- **Documentation claims**: the checklist now says "recursive `list`, `dict`, exact-`int`, and `Option` lowering in both directions, including escaped user identifiers" — accurate against the implementation I probed. No ordering claim survives anywhere (`grep -rni ordered/insertion` over docs/internal_docs/verification/plans is clean for the bridge). The architecture paragraph's statement that borrowed collection arguments materialise an owned statement-scoped temporary is correct.

---

## Non-blocking findings

1. **The same raw-name interpolation still exists on one untouched path.** `python_object_callback_adapter_expr` (`rust_interop_direct.rs:157-158`, unchanged from `082988df1`, verified) formats `param.name` straight into a `Verbatim`. A keyword-named callback parameter on a Python-callback constructor target would produce the identical `SIFR-INTERNAL-0001`. Pre-existing, outside `bridge_type_matrix`, and not certified by this row — but it is now the only remaining instance of the exact root cause B5 named, so it is the natural follow-up.
2. **Bridge `Result` error mapping is name-whitelisted.** `is_message_error_alias` (`rust_interop_error_mapping.rs:103`) maps a bridge error's `Display` into a Sifr error class only for `DiagnosticError`, `ProcessError`, `NetError`, `TlsError`, `HeaderError`, `HttpError`, `SignalError`; any other single-`message` user error class gets `map_err(|e| e)` and a raw `E0308` (I hit this with a class named `ProbeError`, then confirmed the fixture works only because it is named `DiagnosticError`). Pre-existing, untouched by this diff, and the row's `thiserror` claim is satisfied by the fixture as written — but the certified "display-mapped `thiserror` errors" wording silently depends on the class name.
3. **Pre-existing doc line sits awkwardly beside the new Option certification.** `internal_docs/rust_interop_architecture.md` still says `Option[str]`/`Option[bytes]` "use generated optional borrowed views for borrowed parameters", while `bridge_optional_type` (`rust_interop_bridge_contract.rs:642-656`) reports `Option<inner_owned>` as *both* the borrowed and owned bridge type — which is what the lowering now generates and what my probes compile against. Unchanged text, so out of scope, but a reader of the newly certified row may be misled.
4. **Carried from round 3, still true and still non-blocking:** borrowed composite conversion clones elements, so a borrowed `dict[str, OpaqueClass]` needs `T: Clone` for `Handle<T>`; the certified probe cannot observe exact-`int` payloads (needs a `sifr_runtime` dep — works, verified out of band); no scenario binds an `async` bridge with a converted collection.
5. **Carried cosmetics:** `bridge_type_matrix` is still inserted out of alphabetical order in `stable_support_claims.json` and the generated docs table; the negative evidence still binds the synthetic `set[int]` contract test rather than the checked-in `negative/unsupported_container_rejections.sifr`; `cargo clippy -p sifr_codegen --all-targets -- -D warnings` still fails with the same 14 pre-existing errors present verbatim at `082988df1` (`--all-targets` is not the gated invocation).

None of these blocks: each is either pre-existing and outside the promoted row, or coverage rather than correctness.

---

## SATISFIED

# `implementation pass_clone_2` Index/Slice/Star-Unpack Ownership Traceability

Capability: `issues/ownership-aware-collection-lowering-and-clone-elision.md`

## Scope

`implementation pass_clone_2` applies ownership-aware copy/clone extraction decisions to:

- list/dict safe indexing (`Option` paths)
- non-optional list/dict index extraction in structured/simple lowering
- star-unpack lowering (remove whole-source clone)
- stepped list slicing element extraction (`Copy` element fast path)

## Key implementation points

- shared copy-vs-clone helpers added in:
  - `crates/sifr_codegen/src/helpers.rs`
    - `is_copy_type_for_codegen(...)`
    - `option_projection_method_for_owned_type(...)`
- structured index lowering now picks `copied` vs `cloned` based on element ownership:
  - `crates/sifr_codegen/src/expr_render_helpers.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- simple index/condition lowering now uses copy-aware extraction:
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- simple star-unpack no longer clones whole source container:
  - `crates/sifr_codegen/src/lower_stmt.rs`

## Evidence: clone-elision outcomes

Command:

- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/indexing_and_unpacking_cloning.sifr`

Observed shape:

- list/dict `int` safe indexing uses copy extraction:
  - `__sifr_index_list.get(__sifr_index_norm).copied()`
  - `scores.get("alice").copied()`
- move-element list safe indexing still clones as required:
  - `__sifr_index_list.get(__sifr_index_norm).cloned()`
- star-unpack source clone removed:
  - `let _star_tmp = &nums;`
  - `let _star_tmp = vec![1 as i64, 2 as i64, 3 as i64];`
  - no `let _star_tmp = <source>.clone();`
- stepped `Copy` slice extraction uses copy-out instead of per-element clone:
  - `_result.push(*_el);`
  - no `_result.push(_el.clone());` for the `list[int]` stepped-slice path

## Capability artifacts

- pass fixture: `crates/sifr/tests/e2e/pass/indexing_and_unpacking_cloning.sifr`
- demo: `demos/slice_unpacking/main.sifr`

## Validation snapshot

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/indexing_and_unpacking_cloning.sifr` -> PASS
- `cargo run -q -p sifr -- run demos/slice_unpacking/main.sifr` -> PASS

Residual items intentionally deferred to later implementation passes:

- conservative generic/index extraction hardening (`TypeVar` / `Any`) in `implementation pass_clone_3`
- broader generated-code normalization polish (`.copied().collect()` redundancy) in `implementation pass_clone_3`

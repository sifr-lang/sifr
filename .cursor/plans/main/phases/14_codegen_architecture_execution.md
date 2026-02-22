# Phase 14 Execution Checklist (Codegen Architecture)

This checklist operationalizes `14_codegen_architecture.md` into milestone-by-milestone execution loops.

Loop per milestone: **Work -> Validate -> Demo -> PR -> Review -> Merge -> Mark Done**

---

## Global Guards (apply to every milestone)

- [ ] Keep milestone scope limited to the current definition-of-done
- [ ] Preserve semantic parity: generated Rust must compile and behave the same
- [ ] Run `cargo test -p sifr_codegen` and `cargo clippy -p sifr_codegen -- -D warnings`
- [ ] Run targeted E2E parity checks for changed paths
- [ ] Create/update demo file in `demos/` named after the milestone
- [ ] Open one or multiple PRs for the milestone
- [ ] Review the PR(s), address findings, merge, and update roadmap/progress docs

---

## milestone_rust_ir_types

status: done

- [x] Add `crates/sifr_codegen/src/rust_ir.rs` with core IR node families:
  - [x] `RustFile`, `RustItem`, `RustStmt`, `RustExpr`
  - [x] `RustType`, `RustLiteral`, `RustParam`, `RustMatchArm`, `RustEnumVariant`, `RustTypeParam`, `Visibility`
  - [x] `RawCode(String)` variants at item/stmt/expr/type levels
- [x] Derive `Debug` and `Clone` for all IR types
- [x] Wire module exports in `crates/sifr_codegen/src/lib.rs`:
  - [x] `mod rust_ir;`
  - [x] `pub use rust_ir::*;`
- [x] Add representative IR construction tests in `rust_ir.rs`
- [x] Demo: `demos/milestone_rust_ir_types_demo.rs`
- [x] Open PR for milestone 1
- [x] Review and merge PR for milestone 1
- [x] Mark `milestone_rust_ir_types` status done in phase docs

---

## milestone_rust_ir_renderer

status: done

- [x] Add `crates/sifr_codegen/src/render.rs` with `Renderer`
- [x] Implement full render coverage for all IR variants
- [x] Add `render_items`, `render_stmts`, `render_expr` convenience functions
- [x] Wire exports in `lib.rs` (`mod render; pub use render::*;`)
- [x] Add renderer snapshot/unit tests (including `RawCode` passthrough)
- [x] Demo: `demos/milestone_rust_ir_renderer_demo.rs`
- [x] Open PR(s), review, merge
- [x] Mark done in phase docs

---

## milestone_codegen_preamble_migration

status: done

- [x] Move preamble emission to IR items (`error types`, `FileHandle`, logging, imports)
- [x] Add `sifr_type_to_rust_type(&Type) -> RustType`
- [x] Remove/reduce `is_builtin_error_referenced` string scanning
- [ ] Add differential old-vs-new codegen harness for parity
- [ ] Remove at least 5 clippy suppressions (including `format_push_string`)
- [x] Demo: `demos/milestone_codegen_preamble_migration_demo.sifr`
- [x] Open PR(s), review, merge
- [x] Mark done in phase docs

---

## milestone_codegen_stmt_expr_migration

status: in_progress

- [x] Introduce `context.rs` (`CodegenContext`, `ScopeContext`, `CodegenError`)
- [x] Add `lower_expr.rs`, `lower_stmt.rs`, `lower_item.rs`, `preamble.rs`
- [x] PR217 slice: fix match-literal pattern rendering to avoid expression casts in Rust pattern position (`1 as i64 =>`), add regression test, and validate stmt/expr demo + targeted e2e match run
- [x] PR218 slice: reduce `emit_match`/`emit_match_arm` `expr_to_string` callsite cluster by rendering match subjects directly and routing literal/guard string building through structured helper paths (`try_lower_leaf_expr` + `render_expr` fallback)
- [x] PR219 slice: remove `RustEmitter::pub_mode` temporal-coupling flag by threading explicit module visibility through `emit_module`/`emit_class`/`emit_function` paths and add `generate_rust_multi` visibility regression coverage
- [x] PR220 slice: replace `in_loop_with_else` temporal flag with explicit loop-else context stack (`loop_else_stack`) and add nested-loop regression coverage for `_broke` propagation
- [x] PR221 slice: remove `RustEmitter::test_mode` temporal flag by threading explicit test-context through `emit_module`/`emit_function` and add `generate_rust_test` regression coverage
- [x] PR222 slice: replace sticky `suppress_field_clone` temporal flag with scoped pending suppression (`pending_self_field_clone_suppression`) and add non-sticky clone-suppression regression coverage
- [x] PR223 slice: route match and registry rendering through `render_expr_with_lowered_fallback` helper to remove direct `expr_to_string` callsite cluster (remaining direct usage isolated to helper fallback)
- [x] PR224 slice: make `expr_to_string` a pure emitter fallback (remove duplicate leaf-lowering fast path) so lowered-first behavior is centralized in `render_expr_with_lowered_fallback`
- [x] PR225 slice: remove `expr_to_string` entirely and fold fallback rendering into `render_expr_with_lowered_fallback` so direct `expr_to_string` callsites are zero
- [x] PR226 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `if` statements (no `elif`) when condition/body substatements are lowerable
- [x] PR227 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `while` statements (no `else`) and preserve loop-else break-marker isolation in nested loop contexts
- [x] PR228 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `for` statements (no `else`, non-tuple target) and preserve nested loop-else break-marker isolation
- [x] PR229 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `while ... else ...` using `_broke` marker IR and preserve outer loop-else context for else-body breaks
- [x] PR230 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `for ... else ...` (non-tuple target) using `_broke` marker IR and preserve outer loop-else context for else-body breaks
- [x] PR231 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `if/elif/else` chains (lowerable conditions/bodies), preserving fallback for non-lowerable `elif` conditions
- [x] PR232 slice: extend `lower_stmt` dual-path coverage with structured lowering for safe simple augmented assignments (`-=`, `*=`, `/=`, `%=`) while preserving fallback for special ops (`+=`, `//=`, `**=`)
- [x] PR233 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `raise` statements (`return Err(...)`) when raise value is leaf-lowerable, preserving fallback for non-leaf values
- [x] PR234 slice: extend `lower_stmt` dual-path coverage with structured lowering for simple `assert` without message (`assert!(cond)`) when condition is leaf-lowerable, preserving fallback for message asserts and non-leaf conditions
- [x] PR235 slice: extend assert dual-path with structured IR `RustStmt::Assert` so simple message asserts (`assert!(cond, "{}", msg)`) lower without raw code, while preserving fallback for option-typed or non-leaf message paths
- [x] PR236 slice: extend assert dual-path message lowering for option-typed variable messages by structuring `map_or("None", |_v| format!("{}", _v))`, while preserving fallback for non-leaf option expressions
- [x] PR237 slice: extend simple `AugAssign` dual-path coverage to include `//=`, normalizing to legacy-compatible Rust `/=` emission while preserving fallback for `**=` and `+=`
- [x] PR238 slice: extend `lower_stmt` dual-path with structured lowering for bare `return` (`value: None`) by threading explicit return context so option-return functions emit `return None;` while display-impl contexts preserve fallback
- [x] PR239 slice: extend `lower_stmt` dual-path with conservative structured lowering for `return <leaf>` by threading class/return context guards so option/union/class-sensitive return semantics still fall back safely
- [x] PR240 slice: extend `return <leaf>` dual-path to lower `return None` in option-return context while preserving fallback for non-option unions and other guarded return-shaping contexts
- [x] PR241 slice: extend option-return dual-path to lower `return <leaf-non-option>` as `return Some(<leaf>)` with conservative fallback preserved for non-leaf and class/display-sensitive contexts
- [x] PR242 slice: extend non-option union return dual-path to lower `return <leaf>` as `return <UnionEnum>::<Variant>(<leaf>)` while preserving fallback for non-leaf return values
- [x] PR243 slice: extend simple `AugAssign` dual-path to lower numeric `+=` with structured IR while preserving fallback for string/list-style `+=` emitter-specific semantics
- [x] PR244 slice: extend numeric `AugAssign` dual-path to lower RHS simple name operands (e.g. `x += delta`) while preserving fallback for non-numeric/string-style `+=` cases
- [x] PR245 slice: extend simple `Assign` dual-path to lower RHS simple name operands (e.g. `x = y`) while preserving borrowed-TypeVar clone fallback by refusing structured lowering in borrowed-param cases
- [x] PR246 slice: extend plain `return <expr>` dual-path to lower simple name operands (e.g. `return x`) in non-option/non-union contexts while preserving guarded fallback paths for option/union/class/display-sensitive return shaping
- [x] PR247 slice: extend simple `raise` dual-path to lower simple name operands (e.g. `raise e`) to structured `Err(e)` while preserving fallback for non-leaf raise values
- [x] PR248 slice: extend plain non-option `return <expr>` dual-path to lower option-typed simple name operands via structured `.unwrap()` while preserving fallback for option-return and union-return shaping paths
- [ ] Migrate `emit_*` to `lower_* -> IR -> render` dual path
- [ ] Implement semantic transforms (`elif`, `for/else`, `while/else`)
- [x] Replace all `expr_to_string` call sites with structured lowering
- [ ] Convert >= 80% of stmt/expr arms to structured IR (`RawCode` only for remainder)
- [x] Remove/replace at least 4 temporal coupling flags
- [ ] Add differential corpus parity tests
- [x] Demo: `demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] Open PR(s), review, merge
- [ ] Mark done in phase docs

---

## milestone_codegen_intrinsic_migration

status: done

- [x] PR1 scaffold: add `intrinsics/mod.rs` registry and `intrinsics/math.rs` domain lowerers (initial subset)
- [x] Route `emit_intrinsic_call` through registry-first dispatch with legacy match fallback
- [x] Demo: `demos/milestone_codegen_intrinsic_migration_demo.sifr` (initial registry path validation)
- [x] PR2 slice: add registry dependency metadata plumbing and migrate `json_loads/json_dumps` to registry lowering
- [x] PR3 slice: migrate expanded `sifr.math` intrinsic set into registry lowerers with parity checks
- [x] PR4 slice: migrate `sifr.env` intrinsic handlers into registry lowerers with env parity tests
- [x] PR5 slice: migrate `sifr.os` command/argv intrinsics into registry lowerers with OS parity tests
- [x] PR6 slice: migrate core `sifr.io` file/path intrinsics into registry lowerers with IO parity tests
- [x] PR7 slice: migrate additional `sifr.os` intrinsics (`chdir`, `getpid`, `cpu_count`, `stat_size`) into registry lowerers
- [x] PR8 slice: migrate remaining core `sifr.io` intrinsics (`append_text`, `walk_dir`) into registry lowerers
- [x] PR9 slice: migrate core `sifr.pathlib` intrinsics (`touch`, `resolve_path`, `iterdir`) into registry lowerers
- [x] PR10 slice: migrate `sifr.os.which` into registry lowerers
- [x] PR11 slice: migrate core `sifr.test` assertion intrinsics into registry lowerers
- [x] PR12 slice: migrate `sifr.collections` set intrinsics into registry lowerers
- [x] PR13 slice: migrate `sifr.collections` counter/defaultdict intrinsics into registry lowerers
- [x] PR14 slice: migrate core `sifr.bytes` intrinsics into registry lowerers
- [x] PR15 slice: migrate `sifr.time` intrinsics (`time_now`, `sleep`, `time_format`, `perf_counter`, `monotonic`, `strptime`, `gmtime`, `localtime`) into registry lowerers
- [x] PR16 slice: migrate `sifr.random` intrinsics (`random_int`, `random_float`, `random_choice`, `random_uniform`, `random_shuffle`, `random_sample`, `random_randrange`, `random_gauss`) into registry lowerers
- [x] PR17 slice: migrate `sifr.re` intrinsics (core and flags variants) into registry lowerers
- [x] PR18 slice: migrate `sifr.hash` intrinsics (`sha256`, `md5`) into registry lowerers
- [x] PR19 slice: migrate `sifr.platform` intrinsics (`platform_system`, `platform_arch`, `platform_node`, `platform_release`, `platform_version`, `platform_processor`) into registry lowerers
- [x] PR20 slice: migrate `sifr.uuid` intrinsic (`uuid4`) into registry lowerers
- [x] PR21 slice: migrate `sifr.toml` intrinsic (`toml_parse`) into registry lowerers
- [x] PR22 slice: migrate `sifr.datetime` intrinsics (`datetime_now`, `datetime_now_struct`, `datetime_format`, `datetime_from_timestamp`) into registry lowerers
- [x] PR23 slice: migrate `sifr.sys` intrinsics (`sys_exit`, `sys_version`, `sys_platform`, `sys_maxsize`) into registry lowerers
- [x] PR24 slice: migrate `sifr.subprocess` intrinsics (`subprocess_run`, `subprocess_run_with_input`, `subprocess_run_structured`) into registry lowerers
- [x] PR25 slice: migrate `sifr.html` intrinsics (`html_escape`, `html_unescape`) into registry lowerers
- [x] PR26 slice: migrate `sifr.calendar` intrinsics (`calendar_isleap`, `calendar_weekday`, `calendar_monthrange`) into registry lowerers
- [x] PR27 slice: migrate `sifr.gzip` intrinsics (`gzip_compress`, `gzip_decompress`) into registry lowerers
- [x] PR28 slice: migrate `sifr.zipfile` intrinsics (`zip_create`, `zip_add_file`, `zip_read_file`, `zip_namelist`) into registry lowerers
- [x] PR29 slice: migrate core `sifr.base64` intrinsics (`base64_encode`, `base64_decode`, `base64_encode_opts`, `base64_decode_opts`, `urlsafe_b64encode`, `urlsafe_b64decode`) into registry lowerers
- [x] PR30 slice: migrate remaining `sifr.base64` intrinsics (`b32encode`, `b32decode`, `b32hexencode`, `b32hexdecode`) into registry lowerers
- [x] PR31 slice: migrate `sifr.hashlib` intrinsics (`sha1`, `sha512`, `sha224`, `sha384`, `blake2b`, `blake2s`) into registry lowerers
- [x] PR32 slice: migrate remaining `_sifr.fs` OS-adjacent intrinsics (`disk_usage`, `os_sep`, `os_linesep`, `os_name`) into registry lowerers
- [x] PR33 slice: migrate legacy `_sifr.time` compatibility intrinsics (`time_strptime`, `time_gmtime`, `time_localtime`) into registry lowerers
- [x] PR34 slice: migrate remaining `sifr.pathlib` glob intrinsics (`glob_pattern`, `rglob_pattern`) into registry lowerers
- [x] PR35 slice: add `chrono` dependency metadata for all registry-backed time intrinsics and compatibility aliases
- [x] PR36 slice: add dependency metadata for registry intrinsics using `rand`, `sha2`, and `md5`
- [x] PR37 slice: add `regex` dependency metadata for all registry-backed `sifr.re` intrinsics
- [x] PR38 slice: add supplemental dependency metadata plumbing (`random_gauss` now declares `rand_distr`)
- [x] PR39 slice: add method-registry scaffold and route `str.upper/lower/strip` through registry-first lowering
- [x] PR40 slice: expand method-registry coverage for string methods (`startswith`, `endswith`, `split`)
- [x] PR41 slice: expand method-registry coverage for string methods (`replace`, `find`)
- [x] PR42 slice: expand method-registry coverage for string methods (`lstrip`, `rstrip`, `count`, `join`)
- [x] PR43 slice: expand method-registry coverage for string case/predicate methods (`capitalize`, `title`, `swapcase`, `isdigit`, `isalpha`, `isalnum`, `isspace`, `isupper`, `islower`)
- [x] PR44 slice: expand method-registry coverage for string padding methods (`center`, `ljust`, `rjust`, `zfill`)
- [x] PR45 slice: add list method module and route core list methods (`clear`, `copy`, `reverse`, `sort`, `count`, `contains`, `pop`, `remove`, `index`) through registry-first lowering
- [x] PR46 slice: add dict method module and route core dict methods (`keys`, `values`, `items`, `update`, `clear`, `copy`) through registry-first lowering
- [x] PR47 slice: add set method module and route core/relational set methods (`add`, `remove`, `discard`, `contains`, `clear`, `copy`, `issubset`, `issuperset`, `isdisjoint`, `pop`) through registry-first lowering
- [x] PR48 slice: expand set method module with set-algebra methods (`union`, `intersection`, `difference`, `symmetric_difference`) through registry-first lowering
- [x] PR49 slice: expand dict method module with lookup methods (`contains`, `get`, `pop`) through registry-first lowering
- [x] PR50 slice: expand list method module with mutating methods (`append`, `extend`, `insert`) through registry-first lowering and restore deque-specific fallback routing
- [x] PR51 slice: remove driver bigint string-scanning dependency detection and plumb codegen-reported `required_crates` (`HashSet<String>`) into Cargo dependency generation
- [x] PR52 slice: prune legacy `Type::Str` fallback branches from `emit_method_call` now that string methods are registry-backed
- [x] PR53 slice: prune legacy list/dict/set fallback branches from `emit_method_call` where registry-backed lowering now applies (deque-specific branches retained)
- [x] PR54 slice: extract stdlib Rust filtering/dedup helpers from `lib.rs` into `stdlib_filter.rs` to continue codegen decomposition
- [x] PR55 slice: route deque `_data` methods through method registry (`methods/deque.rs`) and remove deque-specific fallback branches from `emit_method_call`
- [x] PR215 slice: decompose `lib.rs` further by extracting helper/test/entrypoint modules and fix `floor`/`ceil` intrinsic lowering to cast to `i64` so the intrinsic milestone demo compiles and runs
- [x] Add intrinsic registry (`intrinsics/mod.rs`) with metadata + dependency crates
- [x] Split intrinsic lowerers into domain modules (`io`, `math`, `json`, etc.)
- [x] Add method registry and type-specific method modules
- [x] Remove driver string-scanning dependency detection; return `HashSet<String>` dependencies
- [x] Convert all intrinsic/method lowering to structured IR (no `RawCode`) (`scripts/check_codegen_rawcode_gate.sh` passes)
- [x] Reduce `lib.rs` by >= 2000 lines via decomposition (`9805 -> 7795`, delta `-2010` lines)
- [x] Demo: `demos/milestone_codegen_intrinsic_migration_demo.sifr` (final milestone gate)
- [x] Open PR(s), review, merge
- [x] Mark done in phase docs

---

## milestone_codegen_structural_passes

status: done

- [x] PR56 slice: replace legacy stdlib text filtering with structural top-level item DCE in `stdlib_filter.rs` and remove legacy helper trio by name
- [x] PR57 slice: add structured shared-import/infrastructure preamble collection pass in `stdlib_filter.rs` and remove `lib.rs` string-contains scanning for stdlib import flags
- [x] PR58 slice: remove `emitter.output.contains(...)` fallback probes for file-handle/logging globals by using structured emitter/module flags during preamble assembly
- [x] PR59 slice: remove `stdlib_preamble.contains(\"struct FileHandle {\")` probe by propagating a structured `provides_file_handle_struct` flag from stdlib preamble analysis
- [x] PR60 slice: remove 5 clippy suppressions (`map_clone`, `if_same_then_else`, `redundant_closure_for_method_calls`, `iter_next_loop`, `useless_format`) and fix surfaced warnings
- [x] PR61 slice: remove 3 clippy suppressions (`option_map_or_none`, `unnecessary_semicolon`, `redundant_closure`) and fix surfaced warnings
- [x] PR62 slice: remove 3 clippy suppressions (`cloned_instead_of_copied`, `doc_link_with_quotes`, `inefficient_to_string`) and fix surfaced warnings
- [x] PR63 slice: remove `unnecessary_map_or` suppression by converting `map_or(false, ...)` patterns to `is_some_and(...)`
- [x] PR64 slice: remove 2 clippy suppressions (`collapsible_match`, `needless_borrow`) and fix surfaced warnings
- [x] PR65 slice: remove `ref_option` suppression by changing `&Option<T>` parameters to `Option<&T>` and updating call sites
- [x] PR66 slice: remove 3 clippy suppressions (`while_let_on_iterator`, `while_let_loop`, `nonminimal_bool`) with no behavior changes needed
- [x] PR67 slice: remove 2 clippy suppressions (`derivable_impls`, `if_not_else`) by deriving `Default` for `StdlibCode` and simplifying negated conditionals
- [x] PR68 slice: remove `unnecessary_unwrap` suppression by replacing `parent_class.as_ref().unwrap()` with an `Option`-driven inheritance branch
- [x] PR69 slice: remove `assigning_clones` suppression by switching stdlib intrinsic map assignment to `clone_from(...)`
- [x] PR70 slice: remove `wildcard_imports` suppression by replacing `use sifr_hir::*;` with an explicit `sifr_hir::{...}` import list
- [x] PR71 slice: remove `unused_self` suppression by converting `substitute_class_captures_in_guard` to an associated helper and updating call sites
- [x] PR72 slice: remove `explicit_iter_loop` suppression by iterating directly over stdlib signature maps instead of calling `.iter()` in `for` loops
- [x] PR73 slice: remove `type_complexity` suppression by introducing shared type aliases for function signatures and union-analysis tuples
- [x] PR74 slice: remove `format_push_string` suppression by replacing `push_str(&format!(...))` patterns with `write!`/`writeln!` across codegen renderers
- [x] PR75 slice: remove `doc_markdown` suppression by applying clippy doc-comment backtick fixes in `lib.rs`, `stdlib_filter.rs`, and `intrinsics/gzip.rs`
- [x] PR76 slice: remove `uninlined_format_args` suppression by applying clippy inline-format interpolation fixes (`{name}` style) in `lib.rs` and `render.rs`
- [x] PR77 slice: start RawCode-zero gate by lowering int/float leaf expressions and loop-else break marker with structured IR nodes instead of `RawCode`
- [x] PR78 slice: continue RawCode-zero gate by structuring deque method lowerers (`push_back`/`push_front`/`pop_back`/`pop_front`) and fixing cast-parentheses rendering precedence
- [x] PR79 slice: continue RawCode-zero gate by structuring zero-arg list method lowerers (`clear`, `copy`, `reverse`, `sort`, `pop`) as `RustExpr::MethodCall`
- [x] PR80 slice: continue RawCode-zero gate by structuring zero-arg dict/set method lowerers (`clear`, `copy`) as `RustExpr::MethodCall`
- [x] PR81 slice: continue RawCode-zero gate by structuring one-arg by-value method lowerers (`list.append/extend`, `dict.update`, `set.add`) as `RustExpr::MethodCall`
- [x] PR82 slice: continue RawCode-zero gate by structuring one-arg borrowed method lowerers (`list.contains`, `set.remove/discard/contains/issubset/issuperset/isdisjoint`) with `RustExpr::Ref` args
- [x] PR83 slice: continue RawCode-zero gate by structuring dict lookup method lowerers (`dict.contains/get/pop`) as chained `RustExpr::MethodCall` + `RustExpr::Ref` args
- [x] PR84 slice: continue RawCode-zero gate by structuring `list.insert` as `RustExpr::MethodCall` with structured cast to `usize`
- [x] PR85 slice: continue RawCode-zero gate by structuring `list.index` as chained `RustExpr::MethodCall` + closure IR nodes
- [x] PR86 slice: continue RawCode-zero gate by structuring `list.count` as chained iterator/closure IR + cast node and fixing borrowed-arg cast precedence in list/set methods
- [x] PR87 slice: continue RawCode-zero gate by structuring `list.remove` as `RustExpr::Block` + `RustStmt::IfLet` (no raw templated control-flow string)
- [x] PR88 slice: continue RawCode-zero gate by structuring `set.pop` as `RustExpr::Block` with `Let`/`IfLet`/trailing expr nodes
- [x] PR89 slice: continue RawCode-zero gate by structuring `dict.keys`/`dict.values` as chained `RustExpr::MethodCall` nodes (including turbofish collect)
- [x] PR90 slice: continue RawCode-zero gate by structuring `dict.items` as iterator/closure/tuple `RustExpr` nodes instead of raw template string
- [x] PR91 slice: continue RawCode-zero gate by structuring set algebra lowerers (`union`/`intersection`/`difference`/`symmetric_difference`) as chained method-call IR
- [x] PR92 slice: continue RawCode-zero gate by structuring zero-arg string method lowerers (`upper`/`lower`/`strip`/`lstrip`/`rstrip`) as method-call IR chains
- [x] PR93 slice: continue RawCode-zero gate by structuring single-arg string method lowerers (`startswith`/`endswith`/`find`/`count`/`join`) as method-call IR
- [x] PR94 slice: continue RawCode-zero gate by structuring string `split`/`replace` lowerers as closure-based/chained method-call IR
- [x] PR95 slice: continue RawCode-zero gate by structuring string predicate lowerers (`isdigit`/`isalpha`/`isalnum`/`isspace`) as non-empty + `chars().all(...)` IR
- [x] PR96 slice: continue RawCode-zero gate by structuring string case-check lowerers (`isupper`/`islower`) as chained iterator/closure IR
- [x] PR97 slice: continue RawCode-zero gate by structuring string padding lowerers (`ljust`/`rjust`/`zfill`) as `RustExpr::FormatMacro`
- [x] PR98 slice: continue RawCode-zero gate by structuring `string.swapcase` as chained iterator/closure/if-expression IR
- [x] PR99 slice: continue RawCode-zero gate by structuring `string.capitalize` as block + iterator/closure IR
- [x] PR100 slice: continue RawCode-zero gate by structuring `string.title` as split/map/closure+block/join IR
- [x] PR101 slice: continue RawCode-zero gate by structuring `string.center` as block/if/binop/format IR
- [x] PR102 slice: continue RawCode-zero gate by replacing string lowerer expression-slot `RawCode` with structured `Ident` where safe (`join`, `ljust`, `rjust`, `zfill`)
- [x] PR103 slice: continue RawCode-zero gate by replacing additional string expression/type `RawCode` with structured `Ident`/`Named` (`render_borrowed_arg_expr`, `center` width cast, width args)
- [x] PR104 slice: continue RawCode-zero gate by replacing string closure placeholder type `RawCode(\"_\")` with structured `Named(\"_\")`
- [x] PR105 slice: continue RawCode-zero gate by replacing deque by-value arg `RawCode` with structured `Ident` (`append`, `appendleft`)
- [x] PR106 slice: continue RawCode-zero gate by replacing set borrowed/by-value helper `RawCode` with structured `Ident` (`render_borrowed_arg_expr`, `add`)
- [x] PR107 slice: continue RawCode-zero gate by replacing list by-value arg `RawCode` with structured `Ident` (`append`, `extend`)
- [x] PR108 slice: continue RawCode-zero gate by replacing dict helper/by-value `RawCode` with structured `Ident`/`Named` (`render_key_arg_expr`, `items` closure placeholder type, `update`, `get` default)
- [x] PR109 slice: continue RawCode-zero gate by replacing list borrowed/helper and insert cast `RawCode` with structured `Ident`/`Named` (`render_borrowed_arg_expr`, `insert`)
- [x] PR110 slice: continue RawCode-zero gate by replacing list closure placeholder type `RawCode(\"_\")` with structured `Named(\"_\")`
- [x] PR111 slice: continue RawCode-zero gate by replacing list predicate/index compare-right `RawCode` with structured `Ident` (`count`, `remove`, `index`)
- [x] PR112 slice: continue RawCode-zero gate by replacing simple `sys` intrinsic `RawCode` returns with structured IR (`sys.version`, `sys.platform`, `sys.maxsize`)
- [x] PR113 slice: continue RawCode-zero gate by replacing simple `platform` const-return `RawCode` with structured path+call IR (`platform.system`, `platform.arch`, `platform.processor`)
- [x] PR114 slice: continue RawCode-zero gate by replacing `json.dumps` `RawCode` with structured path/call IR (`serde_json::to_string(...).unwrap_or_default()`)
- [x] PR115 slice: continue RawCode-zero gate by replacing simple `os` intrinsic `RawCode` returns with structured IR (`get_args`, `getpid`)
- [x] PR116 slice: continue RawCode-zero gate by replacing `env.keys` `RawCode` with structured path/call/closure IR (`vars_os().map(...).collect()`)
- [x] PR117 slice: continue RawCode-zero gate by replacing `env.values` `RawCode` with structured path/call/closure IR (`vars_os().map(...).collect()`)
- [x] PR118 slice: continue RawCode-zero gate by replacing `env.items` `RawCode` with structured path/call/closure IR (`vars_os().map(...).collect()`)
- [x] PR119 slice: continue RawCode-zero gate by replacing `sys.exit` `RawCode` with structured path/call/cast IR
- [x] PR120 slice: continue RawCode-zero gate by replacing `os_sep` `RawCode` with structured path/call IR (`std::path::MAIN_SEPARATOR.to_string()`)
- [x] PR121 slice: continue RawCode-zero gate by replacing `cpu_count` `RawCode` with structured block/path/call/closure IR
- [x] PR122 slice: continue RawCode-zero gate by replacing `bytes.encode_utf8` `RawCode` with structured path/call/closure IR
- [x] PR123 slice: continue RawCode-zero gate by replacing `stat_size` `RawCode` with structured path/call/closure IR (`std::fs::metadata(...).map(...).map_err(...)`)
- [x] PR124 slice: continue RawCode-zero gate by replacing `os_linesep`/`os_name` `RawCode` with structured `If` + `cfg!` + literal-call IR
- [x] PR125 slice: continue RawCode-zero gate by replacing `assert_true`/`assert_false` `RawCode` with structured macro/unary IR
- [x] PR126 slice: continue RawCode-zero gate by replacing `assert_eq`/`assert_ne` `RawCode` with structured macro IR
- [x] PR127 slice: continue RawCode-zero gate by replacing `assert_gt`/`assert_lt` `RawCode` with structured macro/binop IR
- [x] PR128 slice: continue RawCode-zero gate by replacing `assert_almost_eq` `RawCode` with structured macro/binop-call IR
- [x] PR129 slice: continue RawCode-zero gate by replacing `chdir` `RawCode` with structured path/call/ref IR (`set_current_dir(...).map_err(...)`)
- [x] PR130 slice: continue RawCode-zero gate by replacing `which` `RawCode` with structured path/call/closure IR (`var(\"PATH\").ok().and_then(...)`)
- [x] PR131 slice: continue RawCode-zero gate by replacing `json_loads` `RawCode` with structured path/call/closure/struct-init IR
- [x] PR132 slice: continue RawCode-zero gate by replacing `set_len` `RawCode` with structured cast+method-call IR
- [x] PR133 slice: continue RawCode-zero gate by replacing `new_set`/`set_contains` `RawCode` with structured fn-call/method-call+ref IR
- [x] PR134 slice: continue RawCode-zero gate by replacing `set_from_list` `RawCode` with structured block/let/expr IR
- [x] PR135 slice: continue RawCode-zero gate by replacing `set_add` `RawCode` with structured block/let/if/method-call IR
- [x] PR136 slice: continue RawCode-zero gate by replacing `set_remove` `RawCode` with structured block/retain-closure IR
- [x] PR137 slice: continue RawCode-zero gate by replacing `set_union` `RawCode` with structured block/for/if/method-call IR
- [x] PR138 slice: continue RawCode-zero gate by replacing `set_intersection` `RawCode` with structured block/filter-closure/collect IR
- [x] PR139 slice: continue RawCode-zero gate by replacing `counter_from_list` `RawCode` with structured block/for/aug-assign IR
- [x] PR140 slice: continue RawCode-zero gate by replacing `counter_total` `RawCode` with structured block/let/method-call IR
- [x] PR141 slice: continue RawCode-zero gate by replacing `counter_values` `RawCode` with structured block/let/method-call IR
- [x] PR142 slice: continue RawCode-zero gate by replacing `counter_keys` `RawCode` with structured block/let/method-call IR
- [x] PR143 slice: continue RawCode-zero gate by replacing `counter_increment` `RawCode` with structured block/aug-assign/serialize IR
- [x] PR144 slice: continue RawCode-zero gate by replacing `defaultdict_new` `RawCode` with structured format-macro IR
- [x] PR145 slice: continue RawCode-zero gate by replacing `defaultdict_get` `RawCode` with structured block/let/get-default IR
- [x] PR146 slice: continue RawCode-zero gate by replacing `defaultdict_set` `RawCode` with structured block/insert/macro-call IR
- [x] PR147 slice: continue RawCode-zero gate by replacing `counter_get` `RawCode` with structured block/key-lookup/default IR
- [x] PR148 slice: continue RawCode-zero gate by replacing `counter_items` `RawCode` with structured block/sort/map/format IR
- [x] PR149 slice: continue RawCode-zero gate by replacing `counter_most_common` `RawCode` with structured block/sort/truncate/map/format IR
- [x] PR150 slice: continue RawCode-zero gate by replacing `env_unset` `RawCode` with structured block/if/path-call IR
- [x] PR151 slice: continue RawCode-zero gate by replacing `env_get` `RawCode` with structured block/if/path-call IR
- [x] PR152 slice: continue RawCode-zero gate by replacing `env_set` `RawCode` with structured block/if/path-call IR
- [x] PR153 slice: continue RawCode-zero gate by replacing `time_now` `RawCode` with structured path/fn-call/method-call IR
- [x] PR154 slice: continue RawCode-zero gate by replacing `sleep` `RawCode` with structured path/fn-call IR
- [x] PR155 slice: continue RawCode-zero gate by replacing `gmtime` `RawCode` with structured block/cast/map-closure IR
- [x] PR156 slice: continue RawCode-zero gate by replacing `localtime` `RawCode` with structured block/cast/map-closure IR
- [x] PR157 slice: continue RawCode-zero gate by replacing `time_format` `RawCode` with structured block/let/cast/method-call IR
- [x] PR158 slice: continue RawCode-zero gate by replacing `strptime` `RawCode` with structured path/fn-call/map/map-err IR
- [x] PR159 slice: continue RawCode-zero gate by replacing `time_gmtime` compat `RawCode` with structured block/vec/cast IR
- [x] PR160 slice: continue RawCode-zero gate by replacing `time_localtime` compat `RawCode` with structured block/vec/cast IR
- [x] PR161 slice: continue RawCode-zero gate by replacing `time_strptime` compat `RawCode` with structured parse/map/map-err/typed-result IR
- [x] PR162 slice: continue RawCode-zero gate by replacing `decode_utf8` `RawCode` with structured iterator/map/collect/map-err IR
- [x] PR163 slice: continue RawCode-zero gate by replacing `bytes_to_hex` `RawCode` with structured iterator/map/collect/map IR
- [x] PR164 slice: continue RawCode-zero gate by replacing `gettempdir` `RawCode` with structured path/fn-call/method-call IR
- [x] PR165 slice: continue RawCode-zero gate by replacing `exists` `RawCode` with structured path/fn-call/method-call IR
- [x] PR166 slice: continue RawCode-zero gate by replacing `datetime_now_struct` and `datetime_from_timestamp` `RawCode` with structured block/closure/cast IR
- [x] PR167 slice: continue RawCode-zero gate by replacing intrinsic `RustExpr::RawCode` returns in `io`/`re`/`random` registry lowerers with non-raw IR expression nodes (completed via PR173/PR174/PR175 corrective slices)
- [x] PR168 slice: continue RawCode-zero gate by replacing intrinsic `RustExpr::RawCode` returns in `base64`/`hashlib`/`pathlib` registry lowerers (completed via PR176/PR182/PR183 corrective slices)
- [x] PR169 slice: continue RawCode-zero gate by replacing intrinsic `RustExpr::RawCode` returns in remaining registry modules (`base32`, `bytes`, `calendar`, `gzip`, `os`, `platform`, `subprocess`, `time`, `toml`, `uuid`, `zipfile`) (completed via PR177/PR178/PR179/PR180/PR181/PR184/PR185/PR186/PR187 corrective slices)
- [x] PR173 corrective slice: convert `random` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR174 corrective slice: convert `re` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR175 corrective slice: convert `io` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR176 corrective slice: convert `hashlib` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR177 corrective slice: convert `toml` and `uuid` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR178 corrective slice: convert `gzip` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR179 corrective slice: convert `calendar` intrinsic lowerers (`weekday`, `monthrange`) from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR180 corrective slice: convert `subprocess` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR181 corrective slice: convert `zipfile` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR182 corrective slice: convert `base64` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR183 corrective slice: convert `pathlib` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR184 corrective slice: convert `base32` intrinsic lowerers from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR185 corrective slice: convert `os` intrinsic lowerers (`run_command`, `disk_usage`) from templated expression strings to structured IR node trees (no full-expression `Ident(format!(...))`)
- [x] PR186 corrective slice: convert `bytes` intrinsic lowerers (`encode_utf8`, `bytes_from_hex`) from templated/parenthesized expression strings to structured IR node trees (no `Ident(format!(...))`)
- [x] PR187 corrective slice: convert `time` intrinsic borrowed-arg wrappers (`time_format`, `strptime`, `time_strptime`) from parenthesized expression strings to structured IR refs (no `Ident(format!(...))`)
- [x] PR188 corrective slice: convert `datetime_from_timestamp` argument/literal wrappers from expression strings to structured IR nodes (no `Ident(format!(...))`/string-literal `Ident`)
- [x] PR189 corrective slice: convert `json_loads` input wrapper from parenthesized expression string to structured IR binding/ref (no `Ident(format!(...))`)
- [x] PR190 corrective slice: convert `test` intrinsic wrappers (`assert_false`, `assert_almost_eq`) from parenthesized expression strings to structured IR block bindings
- [x] PR191 corrective slice: convert `collections` set-lowerer wrappers (`set_from_list`/`set_add`/`set_remove`/`set_union`/`set_intersection`) from parenthesized expression strings to structured IR bindings
- [x] PR192 corrective slice: convert `collections` counter-lowerer wrappers (`counter_from_list`/`counter_get`/`counter_most_common`/`counter_total`/`counter_values`/`counter_keys`/`counter_items`/`counter_increment`) from parenthesized expression strings to structured IR bindings
- [x] PR193 corrective slice: convert `collections` defaultdict-lowerer wrappers (`defaultdict_get`/`defaultdict_set`) from parenthesized expression strings to structured IR bindings
- [x] PR194 corrective slice: reduce `preamble` `RawCode` usage by structuring error/logging/static/file-handle read+write+close paths and add gate-accounting test for documented exceptions (<=5)
- [x] PR195 corrective slice: reduce `preamble` `RawCode` usage by structuring `FileHandle.read_bytes` and tightening documented exception method set (`readline`/`readlines`/`write_bytes`)
- [x] PR196 corrective slice: reduce `preamble` `RawCode` usage by structuring `FileHandle.write_bytes` and tightening documented exception method set (`readline`/`readlines`)
- [x] PR197 corrective slice: reduce `preamble` `RawCode` usage by structuring `FileHandle.readline` and tightening documented exception method set (`readlines`)
- [x] PR198 corrective slice: reduce `preamble` `RawCode` usage by structuring `FileHandle.readlines` and removing raw-method exception usage (only tuple enum `RawCode` remains documented)
- [x] PR170 slice: reduce remaining preamble-level `RawCode` usage to preamble-only documented exceptions (target <= 5) and wire docs/tests for gate accounting (completed with exact-count assertion: 1 documented tuple-enum `RawCode` node)
- [x] PR199 corrective slice: tighten preamble RawCode gate-accounting from <=5 to exact documented count (1 tuple-enum node) and confirm no raw file-handle method bodies remain
- [x] PR200 slice: add structural import collection over IR preamble items (`ir_imports` pass) and wire import emission to combine IR-derived needs with existing booleans
- [x] PR201 slice: replace pattern-based stdlib DCE dependency detection with structured identifier traversal (comment/string-safe token scan) in `stdlib_filter` and add regression tests
- [x] PR202 slice: extend stdlib DCE top-level item parsing to include `enum`/`trait`/`static` and `pub`/`pub(crate)` prefixes, with regression test coverage
- [x] PR203 slice: harden stdlib DCE parser for `async`/`const`/`unsafe fn` headers and `static mut` declarations, with focused regression test
- [x] PR204 slice: add `type` alias support to stdlib DCE top-level parsing so aliases join dependency closure (and unused aliases are dropped), with regression test
- [x] PR205 slice: tighten stdlib DCE dependency scanning with context-aware token references so local variable names do not create false-positive item retention, with regression test
- [x] PR206 slice: add `ir_validate` structural pass (duplicate struct fields, empty function body, return-outside-function, RawCode brace balance) and enforce it before rendering preamble/import IR
- [x] PR207 slice: add conservative `ir_optimize` clone-removal pass (trivial literal/ref/copy-cast `.clone()` sites) and run it over preamble/import IR before validation and render
- [x] PR208 slice: refactor stdlib DCE into explicit stdlib-IR traversal (parse -> dependency graph -> transitive closure -> render) while preserving existing filtering semantics and adding impl/struct retention regression coverage
- [x] PR209 slice: derive shared prelude import/file-handle needs from tokenized/parsed stdlib IR content (not comment/string probes), and keep stripping behavior unchanged with regression coverage
- [x] PR210 slice: complete PR171 closeout by switching codegen callsites/tests to explicit `filter_stdlib_ir_to_needed` API name and marking structural IR DCE/import-pass checklist items complete
- [x] PR211 slice: add reproducible binary-size regression script (`scripts/check_codegen_binary_size.sh`) and confirm no increase for structural-pass demo (`b0f8b1e` -> `HEAD`: `523504` -> `523504`, delta `0` bytes)
- [x] PR212 slice: add tuple-enum variant support to structured IR and remove preamble file-handle enum `RawCode` escape hatch; also replace `RegexError.detail` default `RawCode(\"String::new()\")` with structured call IR and tighten preamble RawCode gate test to zero
- [x] PR213 slice: add reproducible RawCode gate script (`scripts/check_codegen_rawcode_gate.sh`) to enforce zero RawCode constructors in intrinsics/methods and run `preamble_rawcode_is_zero` guard test
- [x] PR214 slice: close out structural-pass checklist drift by marking PR167/PR168/PR169 as completed via corrective slices and setting milestone status to done
- [x] PR171 slice: add structural import collection pass from IR tree and replace `filter_rust_code_to_needed` with IR DCE traversal
- [x] PR172 slice: add conservative clone optimization pass and IR validation pass; run binary-size regression check and milestone close-out checklist (completed via PR206/PR207/PR211/PR213)
- [x] Meet `RawCode`-zero gate (target zero; hard max 5 preamble-only documented)
- [x] Add structural import collection pass from IR tree
- [x] Replace `filter_rust_code_to_needed` with IR DCE pass
- [x] Add conservative clone optimization pass
- [x] Add IR validation pass for structural correctness
- [x] Delete legacy string-parser helpers (`parse_rust_blocks`, `extract_top_level_item_name`, `count_braces`)
- [x] Remove at least 20 clippy suppressions from file header
- [x] Confirm generated binary size does not increase
- [x] Demo: `demos/milestone_codegen_structural_passes_demo.sifr`
- [x] Open PR(s), review, merge
- [x] PR216 slice: sync phase-14 architecture/roadmap status markers to reflect actual progress (`rust_ir_types`/`renderer`/`preamble`/`intrinsic`/`structural` done; `stmt_expr` still in progress), leaving phase-complete checkbox open
- [ ] Mark phase 14 done in roadmap

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
- [ ] Migrate `emit_*` to `lower_* -> IR -> render` dual path
- [ ] Implement semantic transforms (`elif`, `for/else`, `while/else`)
- [ ] Replace all `expr_to_string` call sites with structured lowering
- [ ] Convert >= 80% of stmt/expr arms to structured IR (`RawCode` only for remainder)
- [ ] Remove/replace at least 4 temporal coupling flags
- [ ] Add differential corpus parity tests
- [x] Demo: `demos/milestone_codegen_stmt_expr_migration_demo.sifr`
- [ ] Open PR(s), review, merge
- [ ] Mark done in phase docs

---

## milestone_codegen_intrinsic_migration

status: in_progress

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
- [ ] Add intrinsic registry (`intrinsics/mod.rs`) with metadata + dependency crates
- [ ] Split intrinsic lowerers into domain modules (`io`, `math`, `json`, etc.)
- [x] Add method registry and type-specific method modules
- [x] Remove driver string-scanning dependency detection; return `HashSet<String>` dependencies
- [ ] Convert all intrinsic/method lowering to structured IR (no `RawCode`)
- [ ] Reduce `lib.rs` by >= 2000 lines via decomposition
- [ ] Demo: `demos/milestone_codegen_intrinsic_migration_demo.sifr` (final milestone gate)
- [ ] Open PR(s), review, merge
- [ ] Mark done in phase docs

---

## milestone_codegen_structural_passes

status: in_progress

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
- [ ] Meet `RawCode`-zero gate (target zero; hard max 5 preamble-only documented)
- [ ] Add structural import collection pass from IR tree
- [ ] Replace `filter_rust_code_to_needed` with IR DCE pass
- [ ] Add conservative clone optimization pass
- [ ] Add IR validation pass for structural correctness
- [x] Delete legacy string-parser helpers (`parse_rust_blocks`, `extract_top_level_item_name`, `count_braces`)
- [x] Remove at least 20 clippy suppressions from file header
- [ ] Confirm generated binary size does not increase
- [ ] Demo: `demos/milestone_codegen_structural_passes_demo.sifr`
- [ ] Open PR(s), review, merge
- [ ] Mark phase 14 done in roadmap

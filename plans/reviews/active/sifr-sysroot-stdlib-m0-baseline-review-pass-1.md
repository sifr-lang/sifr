Now I have everything needed to produce the review.

## M0 Review — Ad Hoc Sifr Sysroot and Stdlib Toolchain

### Blockers

**B1. Registry is missing `_sifr.uuid` (incomplete native-surface inventory)**
- `internal_docs/stdlib_native_surface_ownership.toml` has **no** `[[surface]]` entry for `_sifr.uuid`, but it is a live native surface: `lib/sifr/uuid.sifr:2` does `from _sifr.uuid import uuid4, uuid3_text, uuid5_text`, and the current implementation lives at `crates/sifr_codegen/src/intrinsics/registry/uuid.rs`. The architecture's own private-module table at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:72` lists `_sifr.uuid` → `uuid.sifr`, so the registry contradicts the architecture row it is supposed to enumerate.
- Violates `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:83-86` (acceptance: "inventories identify **every** native stdlib surface", "the migration registry gives reviewers **one mechanical inventory**").
- Required edit: add a `[[surface]]` row `id = "_sifr.uuid"` with `public_modules = ["sifr.uuid"]`, current owner pointing at `crates/sifr_codegen/src/intrinsics/registry/uuid.rs` + the `rand` dep, final owner `crates/sifr_stdlib uuid feature`, deletion_milestone `M9` (parallel to `_sifr.regex` / `_sifr.html`).

**B2. Registry is missing `_sifr.logging` (incomplete native-surface inventory)**
- No `[[surface]]` entry for `_sifr.logging` either, even though `lib/sifr/logging.sifr:2` imports `from _sifr.logging import set_global_level, get_global_level` and the compiler-side implementation lives at `crates/sifr_codegen/src/intrinsics/registry/logging.rs` (with a `__SIFR_GLOBAL_LOG_LEVEL` lock). The architecture private-module table at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:56` lists `_sifr.logging` → `logging.sifr`.
- Same acceptance bar broken as B1. Note this is *stateful* (global mutex), so its certification_state and `can_move_before_runtime_certification` are not trivial — it belongs near `_sifr.runtime` / `_sifr.signal` ("retained-compiler-language-glue" or "future-owned-by-runtime-resource-certification" with M11c/M12 deletion). Reviewers cannot decide M11/M12 ownership without this row.
- Required edit: add `[[surface]]` `id = "_sifr.logging"` with explicit certification_state and deletion_milestone tied to runtime observability ownership.

### Non-blocking observations

**N1. Call-site family table understates preamble breadth.**
`internal_docs/sifr_sysroot_and_stdlib_architecture.md:86-96` covers preamble for net/tls/url_http and registry for json/encoding/unicode/i18n/python, but `crates/sifr_codegen/src/preamble/` also contains `process_runtime.rs`, `process_async_runtime.rs`, `process_async_child_runtime.rs`, `process_child_pipes.rs`, `task_runtime.rs`, `task_context_runtime.rs`, `task_scope_offload_runtime.rs`, `parallel_runtime.rs`, `cpu_offload_runtime.rs`, `join_set_runtime.rs`, `io_bytes_methods.rs`, `io_logging_random.rs`, and `types_and_errors.rs`. The surface registry captures `_sifr.process` / `_sifr.task` / `_sifr.runtime` (so the migration is *owned*), but a reviewer reading only the architecture table will not see these call sites. Either widen the table or add a sentence pointing at the registry as the complete enumeration.

**N2. Matrix-row enumeration is paraphrased, not id-keyed.** Architecture lines 109-115 list the 11 future-owned matrix rows by description but never enumerates the 14 `supported` + 5 `supported-through-bridge` rows by id. The count is correct (`verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json` has 14+5+1+11 = 31), and the future-owned descriptions map cleanly to ids (`bridge_type_matrix`, `opaque_resource_matrix`, `panic_boundary_wrapper_emission`, `async_runtime_reqwest`, `callbacks_call_scoped`, `callback_subscription_matrix`, `ecosystem_backend_certification`, `ecosystem_cli_certification`, `native_build_script`, `proc_macro_trust`, `cargo_locked_offline`), but listing the row ids would make the M11 stable-support gate mechanically checkable instead of requiring future readers to re-derive the mapping.

**N3. `_sifr.sys` deletion milestone is a slight overload.** Registry sets `deletion_milestone = "M11b"` for `_sifr.sys` (env/argv/exit). The surface text already calls out the split between env queries and exit/argv language glue, so this is consistent with the issue plan but mildly invites confusion because M11b is titled "Process children, pipes, environment, and process state." Not a blocker — the certification_state field already says `mixed-stdlib-leaf-plus-runtime-sensitive`.

**N4. PR Log placeholder.** `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:16` reads "M0 baseline/inventory: pending." That is fine for the review-gate state but should be updated to the PR URL on merge.

### Cross-checks that passed
- Private-module → public-wrapper table at `sifr_sysroot_and_stdlib_architecture.md:42-72` matches the `from _sifr.*` imports under `lib/sifr/*.sifr` (verified with `grep -E "^from _sifr\." lib/sifr/*.sifr`).
- Current-owner descriptions for the M0 baseline table match the actual code: `crates/sifr_stdlib/src/sources.rs` uses `include_str!("../../../lib/sifr/...")`, `crates/sifr_driver/src/stdlib/bootstrap.rs:32` iterates `STDLIB_SOURCES`, `crates/sifr_codegen/src/lib_project_codegen.rs:6,126` calls `generated_cargo_dependencies`, `crates/sifr_stdlib/src/features.rs:779` reads `SIFR_RUNTIME_PATH`, `crates/sifr_stdlib/src/features.rs:810` falls back to `env!("CARGO_MANIFEST_DIR")`, and `crates/sifr_analysis/src/host/*` calls `sifr_driver::stdlib_external_defs()`.
- Generated dependency lists in the architecture's "Current generated dependency group" table match the `StdlibCrate` enum / dependency planner in `crates/sifr_stdlib/src/features.rs` for both the leaf-deps group and the network/runtime/observability group.
- Rust interop matrix counts (14/5/1/11) reproduce from `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`.
- `internal_docs/architecture.md` index entry added; `plans/roadmap.md` row 39.1 added with correct relative paths.
- Registry parses to 28 rows (27 `_sifr.*` + `generated-runtime-integer-glue`); file-size guardrail passes; required fields (`current_owner`, `final_owner`, `reason`, `certification_state`, `migration_blocker`, `can_move_before_runtime_certification`, `deletion_milestone`) are present on every row.

### Verdict

**NOT review-satisfied.** Two native stdlib surfaces (`_sifr.uuid`, `_sifr.logging`) are missing from the migration registry while being present in the architecture's own private-module table and in real code. The registry is explicitly designated as "one mechanical inventory" for the phase; reviewers cannot confidently approve M0 with this gap because M9 (stateless leaves), M11c (signals/runtime/logging state), and M12 (retained allowlist decisions) all depend on those two surfaces being tagged with current owner, final owner, certification state, and deletion milestone. Add the two `[[surface]]` rows (B1 + B2) and the M0 inventory acceptance bar is met. Non-blocking items N1-N4 can land with M0 or in M1 cleanup.

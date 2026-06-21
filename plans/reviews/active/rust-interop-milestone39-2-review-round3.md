# M39.2 Rust Interop — Round 3 Review

## Status of round 2 blockers

| Round 2 finding | Status | Evidence |
|---|---|---|
| **P0 #1 Probe execution against rustc** | ✅ Largely satisfied (with caveats) | `rust_interop_probe.rs:24-84` materializes a temporary crate that path-deps on `cargo_manifest_path` and runs `cargo check --quiet`; resolver queues probes in `rust_interop.rs:231-235` and executes them after resolution at `:435-449`; stderr substrings map to `RUST_RESOLVE_TARGET_ROOT` or `RUST_TYPE_PROBE_FAILURE` with the original decorator span; end-to-end fixture `package_rust_interop_maps_rustc_probe_resolution_failure` (`rust_interop_tests.rs:134-166`) creates a real backend crate and asserts resolution failure surfaces at `/ws/app/sifr/app.sifr`. |
| **P0 #2 Post-execution native link evidence** | ✅ Satisfied | `materialize.rs:176-208` runs Cargo with `--message-format=json-render-diagnostics`, `validate_native_link_evidence` (`:223-257`) parses `build-script-executed.linked_libs`, normalizes `kind=name` to `name`, rejects entries not in the trusted set with `SIFR-RUST-TRUST-0001`. Negative test `native_link_evidence_rejects_untrusted_build_script_output` (`:404-414`). |
| **P1 #3 platform-dependent `bridges` digest** | ✅ Fixed | `graph/digest.rs:370` uses `normalized_path_string` instead of `Path::display()`; `:505` also normalizes `cargo_manifest_path`. |
| **P1 #4 missing fixtures** | ⚠️ Partial — see findings below |
| **P1 #5 emit can't reach Rust interop context** | ❌ Not fixed | Not in round 3 fix list; `entrypoint.rs:191-203` still constructs `RootedEntrypoint::Project` with `rust_interop_context = None`. |

File-size guardrail respected: `rust_interop.rs` 885, `rust_interop_probe.rs` 139, `rust_interop_digest.rs` 60 (split into focused modules).

---

## P1 — Should fix before merging

### 1. Probe only validates *symbol existence*, not signature / async / Send / Sync
`crates/sifr_driver/src/build/rust_interop_probe.rs:93-106`

```rust
RustInteropDecoratorKind::Function | Async | ZeroCopy | View =>
    format!("#![allow(dead_code)]\nfn __sifr_probe() {{ let _ = {rust_path}; }}\n"),
RustInteropDecoratorKind::Opaque =>
    format!("#![allow(dead_code)]\ntype __SifrProbe = {rust_path};\n"),
```

M39.2 scope (`plans/phases/39_rust_interop.md:92`): *"…isolated probe modules for sync functions, async functions, receiver-mode methods, opaque types, **Send/Sync assertions, and Rust item/signature checking**"*.

The current probe only proves the path resolves to *some* item. It does not check:
- async-ness mismatch (sync Rust fn declared `@rust.async` or vice versa),
- arity/parameter/return-type alignment,
- `Send` for `async_boundary` declarations or `Sync` for `view` declarations (`abi_requirements` on `RustBridgeProbe` are recorded but never enforced — `rust_interop.rs:429-430`),
- method receiver mode for `Method`/`SelfMethod` paths (no probe is queued for `Self.<method>` declarations at all — `:231-235` only queues for the unknown-root branch).

`SIFR-RUST-TYPE-0001` will still fail to surface the cases the milestone names. If the deeper assertions are deliberately deferred to M39.5 (which `plans/phases/39_rust_interop.md:130` repeats them), the M39.2 scope language should be tightened to "Rust item existence checking" rather than left as-is.

### 2. rustc/cargo error classification is substring-fragile
`crates/sifr_driver/src/build/rust_interop_probe.rs:68-77`

```rust
let code = if stderr.contains("cannot find") || stderr.contains("failed to resolve")
    || stderr.contains("unresolved") || stderr.contains("not found") { RUST_RESOLVE_… }
    else { RUST_TYPE_… };
```

These substrings collide with type errors that mention identifiers ("cannot find type `Foo` in scope" is resolution; "the type `T` cannot be resolved" is a type-system message). The classification also leaks raw `cargo`/`rustc` paths and line numbers into the diagnostic note (`:82`), which makes snapshots non-deterministic across machines. Either parse `--message-format=json-render-diagnostics` (already used downstream in materialize) or fold this into the same path so the classification keys off rustc error codes (`E0432` for unresolved imports vs. `E0277`/`E0271` for trait bounds).

### 3. Probe is single-process-id collision-prone and not cached per backend
`crates/sifr_driver/src/build/rust_interop_probe.rs:33-44`, `:64`

Temp dir name is `sifr_rust_probe_{pid}_{hash(cargo_package_id:dotted_path)}`. Two declarations targeting the same backend symbol run `cargo check` twice in the same process; declarations targeting different symbols of the same backend each rebuild the probe crate from scratch (no incremental sharing). For real packages with N decorators that's N synchronous `cargo check` invocations on the critical path. Probes also leak the temp dir if the process is killed between `create_dir_all` and the trailing `remove_dir_all`. Group probes by `cargo_package_id` and emit one probe crate that asserts all symbols, or at minimum dedupe `pending_direct_probes` on `(backend.cargo_package_id, path)`.

### 4. Bridge / Self target roots are never probed
`crates/sifr_driver/src/build/rust_interop.rs:180-209`, `rust_interop_probe.rs` (no caller)

`PackageBridge` and `SelfMethod` resolutions do not push a `PendingRustBridgeProbe`. The DoD line "Probe failures map rustc diagnostics" technically scopes to "direct Cargo dependency roots" (scope :86), so this may be intentional — but with bridge files now generated under `src/bridges/`, declared `bridge.foo` targets currently sail through plan-only checks and fail later at the main cargo build with a `BUILD_RUSTC_OR_CARGO_FAILURE` instead of a `SIFR-RUST-RESOLVE-*` against the decorator span. Either probe bridge paths through the package's own crate (path-dep on the materialized binary project's `Cargo.toml`), or call this out as an explicit M39.2 limitation in the scope text.

### 5. Test gaps still open from round 2 P1 #4
`crates/sifr_driver/src/build/rust_interop_tests.rs`

Round 3 closed the probe-resolution and link-evidence gaps. Still missing:

- positive `bridge.*` fixture exercising `validate_unsafe_bridge_files` (`rust_interop.rs:345-363` and `:600-630` are uncovered);
- `Self.<method>` fixture (success + the "Self on free function" failure path at `:196-208`);
- `proc-macro`, `native-links`, and `unsafe-rust-bridges` trust gates *firing* — only `rust-build-scripts`, `rust-no-panic`, `rust-panic-abort`, `build-env` are exercised;
- determinism fixture (two consecutive `apply_package_rust_interop_metadata` calls produce identical `cache_key_fragment`, important given `tool_version` shells out to `cargo`/`rustc` at `rust_interop.rs:827-835`);
- cache-invalidation deltas on bridge-source mutation, trust-policy mutation, backend `package = "..."` rename;
- transitive Cargo dep rejected as a Rust target root (DoD scopes resolution to *direct* Cargo dependency roots; no negative covers a transitive root being mistakenly accepted).

### 6. Native-link evidence walks linked_libs only — other build-script directives skip validation
`crates/sifr_driver/src/build/materialize.rs:223-257`

`build-script-executed` JSON also carries `cfgs`, `env`, and `linked_paths`. Only `linked_libs` is validated. A trusted build script that exports a new compile-time `cfg` or env-var to `rustc` flies under the radar — this is plausibly out of scope for M39.2, but the milestone DoD line says "native link evidence emitted by trusted build scripts is validated before final artifact acceptance", which `linked_paths` (extra `-L` directories) is plausibly part of. Confirm scope or extend to `linked_paths`.

### 7. `entrypoint.rs` still cannot route emit through Rust interop
`crates/sifr_driver/src/build/entrypoint.rs:191-203,:397` (unchanged since round 2)

Same finding as round 2 P1 #5. `sifr emit` against any package source carrying `@rust(...)` still hits `SIFR-RUST-CARGO-0001`. Either route through `PackageProject` or document the limitation.

---

## P2 — Recommended

### 8. `digest_path` still silently swallows I/O errors and follows symlinks
`crates/sifr_driver/src/build/rust_interop_digest.rs:42-55` — round 1 P2 #8 / round 2 P2 #6 unchanged. Transient read failure shrinks the cache key without diagnostic; no extension filter (editor swap files in `src/bridges/` poison the key); recursion follows symlinks via default `read_dir`/`metadata`.

### 9. `fnv1a64` still duplicated
`rust_interop_digest.rs:33-40` vs `crates/sifr_package/src/graph/digest.rs:71-78` — round 1 P3 #13 unchanged.

### 10. `Self`-on-non-method diagnostic still misleading
`rust_interop.rs:196-208` still emits `"unresolved Rust target root \`{root}\`"` for an invalid `Self`. Round 1 P3 #12 unchanged.

### 11. `panic_strategy` still env-only
`rust_interop.rs:707` — same as round 1 P2 #10 / round 2 P2 #10. Profile codegen settings cover the cache-relevant case; either populate from the resolved profile or drop the field.

### 12. New probe test depends on `cargo` on PATH with no skip
`rust_interop_tests.rs:135-166` writes a real backend crate and calls `cargo check` indirectly. In environments where `cargo` is missing or sandboxed (some CI partitions), the test will get `RUST_CARGO_METADATA` instead of `RUST_RESOLVE_TARGET_ROOT` and fail with a confusing assertion. Either gate behind a `cargo --version` probe at the top, or document the test's `cargo` dependency.

---

## Verdict

**Cleared for M39.2** with the caveat that the probe is currently a *symbol-existence* probe, not the "Send/Sync assertions, and Rust item/signature checking" probe the scope text promises. The two round 2 P0 blockers (rustc probe execution at `rust_interop_probe.rs:24-84` and post-execution link evidence at `materialize.rs:223-257`) are both closed by real implementations with real negative fixtures, and the graph-digest cross-platform regression (`graph/digest.rs:370`) is fixed.

Before merging M39.2 to `main`, please either:
1. extend the probe source to assert async-ness / Send / Sync / receiver-mode (P1 #1), **or** tighten the scope text in `plans/phases/39_rust_interop.md:92` to reflect what the probe actually verifies and explicitly defer the deeper assertions to M39.5;
2. add fixtures for `bridge.*`, `Self.method`, `proc-macro`, `unsafe-rust-bridges`, determinism, and cache invalidation (P1 #5);
3. swap the substring-based rustc error classification for `--message-format=json-render-diagnostics`-style parsing (P1 #2) — important because incorrect classification will route real type errors to `SIFR-RUST-RESOLVE-0001`, masking the diagnostic the milestone is built around.

The remaining P1/P2 items (emit context, probe deduplication, digest hardening, fnv1a64 dedupe, panic_strategy noise) are non-blocking but worth queueing for the M39.3/M39.5 follow-ups while these files are still warm.

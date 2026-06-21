I have enough context to write the review. The round 2 fix list addresses round 1's P0 (panic gates) and most P1s, but two explicit M39.2 DoD bullets remain unsatisfied, and round 1 P1 #6 (probe execution) was not in the fix list.

---

# M39.2 Rust Interop — Round 2 Review

## Status of round 1 blockers

| Round 1 finding | Status | Evidence |
|---|---|---|
| P0 panic gates (symbol match) | ✅ Fixed | `rust_interop.rs:308-330` + `panic_policy()` at `:531-541`; negative tests `rust_interop_tests.rs:132-162` |
| P1 #2 `build-env` trust gate | ✅ Fixed | `rust_interop.rs:298-307` + `build_env_trust_entries()`; test at `:165-177` |
| P1 #3 cache: lock digest + profile codegen | ✅ Fixed | `:674` (lock), `:679` + `profile_codegen_settings()` (release profile); asserted in `rust_interop_tests.rs:180-211` |
| P1 #4 target/profile sourcing | ✅ Mostly fixed | `cargo_profile = "release"` matches `materialize.rs:176`; `target_triple()` now reads `SIFR_TARGET` or `rustc -vV`. `panic_strategy` is still env-only but is also covered through `profile_codegen_settings`' `"panic"` key |
| P1 #5 unsafe-rust-bridges path matching | ✅ Fixed | `validate_unsafe_bridge_files` + `collect_unsafe_bridge_files` scan package-local `.rs` files containing `"unsafe"` and emit normalized relative paths (`rust_interop.rs:333-601`) |
| P1 #7 trust dedup + outside loop | ✅ Fixed | `seen_trust_requirements` set in `require_trust()` (`:362-369`); `validate_declaration_trust` / `validate_unsafe_bridge_files` called outside per-path loop (`:149-157`) |
| P2 #9 path separators | ⚠️ Partial | `rust_interop.rs` + `rust_interop_digest.rs` use `normalized_path_string`. But `crates/sifr_package/src/graph/digest.rs:368` still serializes `bridges` with `path.display().to_string()` — package-graph digest still diverges across Windows |
| **P1 #6 probe execution** | ❌ **Not fixed** | No rustc invocation anywhere. `push_probe()` (`rust_interop.rs:395-420`) only emits plan metadata. `SIFR-RUST-TYPE-0001` only fires when `probe_kind()` returns `None` (decorator/owner shape mismatch), which is a plan-time check, not a probe failure. The added test `package_rust_interop_rejects_unrepresentable_probe_owner` exercises this plan-time path |

---

## P0 — Blocking for M39.2 DoD

### 1. Probe execution against rustc is not implemented
`crates/sifr_driver/src/build/rust_interop.rs:395-420`, `crates/sifr_driver/src/build/materialize.rs:170-194`

M39.2 DoD (plans/phases/39_rust_interop.md:97): *"Probe failures map rustc diagnostics to `SIFR-RUST-RESOLVE-*` or `SIFR-RUST-TYPE-*` diagnostics at the original decorator span."* And scope line 92: *"Generate `RustBridgeProbePlan` metadata and isolated probe modules for sync functions, async functions, receiver-mode methods, opaque types, Send/Sync assertions, and Rust item/signature checking."*

Current behavior: only `RustBridgeProbe` metadata is generated; no isolated `.rs` probe module is materialized; no `cargo check`/`rustc` is invoked. `SIFR-RUST-TYPE-0001` therefore cannot surface signature/visibility/asyncness/`Send`/`Sync` mismatches. The new negative test only covers plan-time owner/decorator-kind disagreement, not the rustc diagnostic mapping the milestone requires.

This is the same finding as round 1 P1 #6, and it was not included in the round 2 fix list.

### 2. Post-execution link-evidence validation is missing
`crates/sifr_driver/src/build/materialize.rs:170-194`

M39.2 DoD (plans/phases/39_rust_interop.md:95): *"Known untrusted build scripts and proc macros are rejected before Cargo execution; native link evidence emitted by trusted build scripts is validated before final artifact acceptance."*

Pre-execution is correctly enforced via `validate_backend_trust` (`rust_interop.rs:244-290`). But `run_cargo_build` (`materialize.rs:170-194`) does `Command::new("cargo").args(["build", "--release", "--quiet"]).output()` and only inspects exit status. There is no `--message-format=json` parsing, no capture of `cargo:rustc-link-lib=…` lines emitted by build scripts, and no validation against `trust.native-links` after cargo has run. A trusted build script that emits unexpected link directives is accepted silently.

---

## P1 — Should fix before merging

### 3. `CanonicalRustInteropConfig.bridges` is platform-dependent in package graph digest
`crates/sifr_package/src/graph/digest.rs:366-372`

```rust
bridges: package.manifest.rust.bridges.iter()
    .map(|path| path.display().to_string())
    .collect(),
```

`Path::display()` emits `\` on Windows. This serialization feeds `digest_package_graph`, which is then included in `RustInteropCargoInputs.package_graph_digest` (`rust_interop.rs:664-672`). So the round-2 forward-slash normalization in `rust_interop.rs` is undone here — cross-platform cache keys still diverge for any package declaring `rust.bridges`.

### 4. Missing M39.2 verification fixtures
The new test suite in `rust_interop_tests.rs:23-211` covers untrusted build-script, untrusted no-panic, untrusted panic-abort, untrusted build-env, missing cargo context, unknown root, unrepresentable probe owner, and lock/profile cache-input recording — but does not include:

- **`bridge.*` root resolving to `PackageBridge`** (no positive fixture exercises `validate_unsafe_bridge_files`)
- **`Self.<method>` on a method receiver** (no positive or negative fixture)
- **`proc-macro`, `native-links`, `unsafe-rust-bridges` trust gates firing** (cargo_backend tests parse the manifest fields but no rust_interop fixture asserts the trust diagnostic for these kinds)
- **Determinism**: two consecutive `apply_package_rust_interop_metadata` runs producing identical `cache_key_fragment` (relevant given `tool_version` shells out to `cargo`/`rustc` each call at `rust_interop.rs:798-806`)
- **Cache invalidation**: bridge-source mutation, trust-policy mutation, backend dependency rename (`package = "..."`)
- **Transitive Cargo dependency rejected as a Rust target root** (DoD: "direct Cargo dependency roots")

These align with the round-1 test-gap list; round 2 added the panic and build-env negatives but did not close the rest.

### 5. `emit_project_entrypoint` cannot reach Rust interop context
`crates/sifr_driver/src/build/entrypoint.rs:191-203`, `:397`

`resolve_project_entrypoint_plan` always uses `RootedEntrypoint::Project`, whose handling at `:397` sets `rust_interop_context = None`. Any `sifr emit <file>` against a package with `@rust(...)` decorators therefore hits `RUST_CARGO_METADATA` from `apply_package_rust_interop_metadata` (`rust_interop.rs:61-66`). Either route `emit` through `PackageProject` when the file lives inside a package, or document this M39.2 limitation.

---

## P2 — Recommended

### 6. `digest_path` still silently swallows I/O errors and follows symlinks
`crates/sifr_driver/src/build/rust_interop_digest.rs:42-55`

`collect_digest_entries` still has `if let Ok(bytes) = fs::read(path)` and `let Ok(read_dir) = fs::read_dir(path) else { return; }` — a transiently unreadable file silently shrinks the digest (cache poisoning risk). The function also has no extension filter (editor swap files, `.DS_Store`, generated artifacts under a bridge directory shift the cache key), and `Path::is_file()` follows symlinks via `fs::metadata`, so a symlink loop will infinite-recurse. Bridge file collection in `rust_interop.rs:582-601` does filter to `.rs`, but the cache-input digest does not.

### 7. `fnv1a64` is still duplicated
`crates/sifr_driver/src/build/rust_interop_digest.rs:33-40` vs `crates/sifr_package/src/graph/digest.rs:71-78` — round 1 P3 #13 unchanged. Drift risk; lift to a shared util.

### 8. `Self`-on-non-method diagnostic wording is misleading
`crates/sifr_driver/src/build/rust_interop.rs:190-199` — emits `unresolved Rust target root \`{root}\`` for an invalid `Self` context. The root *is* recognized; the failure is "`Self` is valid only on Rust interop methods". Round 1 P3 #12 unchanged.

### 9. `apply_package_rust_interop_metadata` early-returns when there are no declarations
`crates/sifr_driver/src/build/rust_interop.rs:57-59` — round 1 P3 #14 unchanged. `bridge_sources` and `cargo_inputs` stay empty even when the package's trust policy or bridge layout changes; cache invalidation falls back to `package_graph_digest`. Low risk; flagged for awareness.

### 10. `panic_strategy` only sourced from `SIFR_RUST_PANIC_STRATEGY`
`crates/sifr_driver/src/build/rust_interop.rs:678` — usually unset in real builds. The actual cache-relevant value is already captured via `profile_codegen_settings`' `"panic"` key, so the separate `panic_strategy` field is mostly noise. Either populate it from the resolved profile or drop it.

---

## Verdict

**Not cleared for M39.2.** Round 2 cleanly closes the P0 panic-gate finding and the P1 cache-input / build-env / dedup / unsafe-bridges-path findings from round 1, and the file-size guardrail is respected (`rust_interop.rs` at 856 lines after the digest split).

However, two explicit M39.2 DoD bullets remain unsatisfied:

- **Probe execution** (`plans/phases/39_rust_interop.md:97`) — round 1 P1 #6, not in the round 2 fix list and still absent.
- **Post-execution validation of trusted build-script link output** (`plans/phases/39_rust_interop.md:95`) — not addressed in either round.

These two, plus the platform-dependent `bridges` serialization in `graph/digest.rs:368` (regression boundary of round 1 P2 #9) and the missing positive fixtures for `bridge.*` / `Self.method` resolution, should land before merging M39.2. The remaining P2/P3 items are non-blocking but worth cleaning up while the file is open.

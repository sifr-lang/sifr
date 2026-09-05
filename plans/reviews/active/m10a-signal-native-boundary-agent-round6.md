PR-ready.

Rationale — I traced each focus area against the diff and found the migration internally consistent:

- **Retained intrinsic leakage — clean.** `mod signal` and the three `signal_*` match arms are gone from `intrinsics/registry.rs`; `intrinsic_signal`/`_sifr.signal` are gone from `sifr_retained_intrinsics/lib.rs`; both source files are deleted. The rewritten test `signal_intrinsics_are_owned_by_compiled_stdlib_declarations` positively asserts `lower_intrinsic` returns `None` for all three names, and the closure script adds them to `RETIRED_INTRINSICS`. No residual lowering path remains.

- **Rust interop async Result/error mapping — correct.** `SignalError` is added to `is_message_error_alias`, matching the `ProcessError`/`NetError`/`HttpError` pattern. Its declaration (`class SignalError(Error): message: str`) has the single `message` field the bridge expects and `Error` as parent, so `bridge_error_expr` wraps the Rust `String` error into the struct correctly. The `Result<SifrIntBridge, String>` → `Result[int, SignalError]` shape lines up with the established bridge convention.

- **Signal number behavior — sound.** Rust returns `SifrIntBridge::from(2)`/`(15)`; the sifr layer's `_signal_from_number` maps 2→SIGINT, 15→SIGTERM, else UNKNOWN, and both constants and `sigint()`/`sigterm()` agree on those numbers. Non-unix branches degrade sensibly (terminate → `Err`, shutdown → ctrl_c only).

- **Feature dependency inference — complete across all three touchpoints.** `harness_model.rs` maps `sifr_stdlib::signals::` → `_sifr.signal`; `fixture_dependency_paths.rs` pushes the `signals` feature; `fixture_cargo_toml.rs` adds `sifr.signal`/`_sifr.signal` to the stdlib-dependency predicate; and `Cargo.toml` upgrades `signals` from `[]` to `["dep:sifr_runtime", "dep:tokio"]`, which the `SifrIntBridge` import and `tokio::signal` usage now require. The new `test_infer_dependencies_recognizes_sysroot_signal_references` end-to-end asserts both the module and the `"signals"` feature reach the Cargo manifest.

- **Certification honesty — accurate.** The row flips `retained`→`closing` with the closing removal criteria, drops `registry_files`/`exact_intrinsics`, and keeps `callback_subscription_matrix` while the reason explicitly disclaims broader callback/subscription certification. It does not overclaim.

Non-blocking nit (not a defect): the three `signal_*` entries in `RETIRED_INTRINSICS` are inserted between `sha1_bytes` and `sha224`, out of alphabetical order — harmless in a frozenset, but worth tidying next time the file is touched.

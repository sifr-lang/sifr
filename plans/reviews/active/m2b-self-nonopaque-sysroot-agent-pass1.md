## Review

I traced the new test against the resolver logic at `crates/sifr_driver/src/build/rust_interop.rs:260-360`.

**Diagnostic path exercised** — with `@rust(Self.close, ...)` on a non-opaque private `_sifr.io.FileHandle`:
1. Canonical sysroot guard (line 284-298) is now skipped because `root == "Self"` (this is exactly the M2b bypass being tested).
2. `Self` arm at line 318 enters `RustInteropOwner::Method` matcher; `opaque_contracts` lookup for `_sifr.io.FileHandle` misses because the test's plan supplies only the method declaration, no opaque class declaration.
3. Falls to line 330-347: emits `RUST_RESOLVE_TARGET_ROOT` (`SIFR-RUST-RESOLVE-0001`), message `unresolved Rust target root Self`, child note `Self target roots are valid only on methods for classes declared with @rust.opaque(...)`.

Every assertion matches this exact path:
- `diagnostics[0].code == "SIFR-RUST-RESOLVE-0001"` ✓
- `!message.contains("canonical sysroot crate")` — the message is `unresolved …`, not the canonical-guard string at line 292 ✓
- `children[*].message.contains("@rust.opaque")` — notes are attached as children in `rust_interop_diagnostics.rs:36-38` ✓
- `spans[0].file` equals `private_path("_sifr.io")` — `push_diagnostic` at line 839-848 uses the `module_sources` `display_path`, which `stdlib_interop_many` sets to that same path ✓

**Scope**: test-only, reuses `method_declaration`, `symbol_argument`, `TempSysroot`, `apply_package_rust_interop_metadata`; consistent with sibling tests at lines 314-341 and 453-477. No production code touched.

**Gap coverage**: the prior review's gap ("prove `Self.X` on a non-opaque class gets the `@rust.opaque` diagnostic rather than the canonical-crate diagnostic") is closed precisely — the fixture is a private sysroot `Self.` method, and the test locks in both the negative (no canonical wording) and positive (child mentions `@rust.opaque`) assertions.

**Nit (non-blocking)**: like the two sibling rejection tests, this one only inspects `diagnostics[0]` without asserting `diagnostics.len()`. Style-consistent, so leaving it alone is correct.

File size: 766 lines, under the 900-line cap.

READY

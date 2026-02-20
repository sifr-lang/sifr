# Interoperability

**Why now:** The language is feature-complete (type system, async, web stack). FFI is the "escape hatch" that gives Sifr access to the entire Rust and C ecosystem — the architecture document explicitly calls this the ecosystem strategy. Formalizing FFI as its own phase recognizes its importance: every stdlib module already wraps Rust crates via intrinsics in `stdlib.rs`, and FFI formalizes this pattern for user code. Power users can extend Sifr immediately without waiting for official wrappers.

---

## milestone_ffi: Foreign Function Interface

status: pending

**Goal:** Give Sifr access to the entire Rust and C ecosystem via foreign function interfaces.

**Depends on:** milestone_web_services (the full language and web stack should be complete and stable before opening the FFI boundary)

### Work Items

- Rust FFI: `extern crate` adds Rust crate dependencies to generated `Cargo.toml`; Sifr functions can call Rust functions directly with type mapping
- C FFI: `extern "C"` for calling C functions via Rust's FFI; limited to basic types (int, float, str as `*const c_char`, pointers)
- `unsafe` keyword: required for FFI calls that bypass Sifr's safety guarantees; compiler enforces that all FFI calls are inside `unsafe` blocks
- Safety boundary: the compiler generates safe Sifr wrappers around unsafe FFI calls, validating inputs and converting Rust panics to `Result` at the boundary

### Definition of Done (milestone_ffi)

- `extern crate` adds Rust crate dependencies to generated Cargo.toml
- Rust FFI calls compile and execute correctly
- `unsafe` blocks required and enforced by the compiler
- C FFI via `extern "C"` works for basic function calls
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: ffi_rust_crate, ffi_c_function, unsafe_block
- E2E fail tests: missing_unsafe, ffi_type_mismatch
- Milestone demo in `./demos/milestone_ffi_demo.sifr`

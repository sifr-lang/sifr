// Reference: rust_ir_types
// Reference: compiler-feature-history
// compiler feature demo: Rust IR type definitions.
//
// This feature set introduces the structured Rust IR model in
// `crates/sifr_codegen/src/rust_ir.rs`.
//
// Demo validation command:
//   cargo test -p sifr_codegen rust_ir::tests
//
// What this demonstrates:
// 1) Struct/impl/function IR nodes can be constructed directly.
// 2) Match/closure/raw-code escape-hatch nodes are represented in typed enums.
// 3) The crate compiles and tests execute against the new IR API.

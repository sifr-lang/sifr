// Reference: milestone_rust_ir_types
// Source issue: revised_phase_roadmap_698c629c.md
// Milestone 14.1 demo: Rust IR type definitions.
//
// This milestone introduces the structured Rust IR model in
// `crates/sifr_codegen/src/rust_ir.rs`.
//
// Demo validation command:
//   cargo test -p sifr_codegen rust_ir::tests
//
// What this demonstrates:
// 1) Struct/impl/function IR nodes can be constructed directly.
// 2) Match/closure/raw-code escape-hatch nodes are represented in typed enums.
// 3) The crate compiles and tests execute against the new IR API.

// Reference: milestone_rust_ir_renderer
// Source issue: revised_phase_roadmap_698c629c.md
// Milestone 14.2 demo: Rust IR renderer.
//
// This milestone adds `crates/sifr_codegen/src/render.rs`, which pretty-prints
// structured IR nodes into Rust source strings.
//
// Demo validation commands:
//   cargo test -p sifr_codegen render::tests
//   cargo test -p sifr_codegen rust_ir::tests
//
// The `render::tests` snapshots cover item/stmt/expr rendering and RawCode
// passthrough behavior for incremental migration.

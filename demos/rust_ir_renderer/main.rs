// Reference: rust_ir_renderer
// Reference: compiler-feature-history
// compiler feature demo: Rust IR renderer.
//
// This feature set adds `crates/sifr_codegen/src/render.rs`, which pretty-prints
// structured IR nodes into Rust source strings.
//
// Demo validation commands:
//   cargo test -p sifr_codegen render::tests
//   cargo test -p sifr_codegen rust_ir::tests
//
// The `render::tests` snapshots cover item/stmt/expr rendering and RawCode
// passthrough behavior for incremental migration.

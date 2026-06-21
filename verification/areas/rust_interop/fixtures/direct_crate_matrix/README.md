# direct_crate_matrix

This fixture family tracks direct bindings for representative dependency roots:
`blake3`, `sha2`, `uuid`, and `regex`.

- Positive evidence: `compatible_direct_signatures` passes through the direct
  Cargo dependency probe path for compatible Rust item shapes.
- Negative evidence: `incompatible_direct_signatures` passes by mapping
  incompatible or unsupported public Rust signatures to stable
  `SIFR-RUST-TYPE-*` or `SIFR-RUST-RESOLVE-*` diagnostics.
- Compatibility category: `supported`. Direct bindings are verified only for bridge-compatible signatures; adapters
  still require explicit bridges.

# blocking_diagnostics

Compiler-diagnostic fixture family for Rust interop blocking and
CPU-heavy classification.

- Diagnostic crate rationale: `rusqlite`, `rayon`, and `flate2` supply
  blocking-I/O and CPU-heavy classification examples used only to exercise
  `SIFR-RUST-ASYNC` diagnostics. They are not linked or executed by this
  fixture.
- Positive evidence: `classified_sync_rust_effects` is represented by
  `rust_interop_lowers_blocking_io_effect_for_sync_rust_function`.
- Negative evidence: `classified_async_declarations_rejected` is represented by
  `rust_interop_rejects_blocking_classification_on_async_function`.

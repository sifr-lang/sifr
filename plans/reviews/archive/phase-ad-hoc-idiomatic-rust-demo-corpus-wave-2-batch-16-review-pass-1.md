Review complete. All three files **APPROVE** with no findings requiring changes.

**Summary:**

- `demos/logging_and_timers/idiomatic.rs` — Clean, correct. All Sifr→Rust constructs map properly. `Formatter` is a cosmetic stub (matches Sifr side).

- `demos/config_json_csv/idiomatic.rs` — Correct. `JSONEncoder.indent` is stored but unused; this is benign since no assertions depend on formatted output. All demo assertions pass.

- `demos/collections_and_argparse/idiomatic.rs` — Correct. `add_argument_typed` has an unused `_default` parameter (cosmetic, no assertions use defaults). All demo assertions pass.

The two recurring low-severity notes are intentional simplifications — API surface is mirrored from Sifr but certain features (indentation, default injection) are omitted from the demos. No behavioral discrepancies.

Full review written to `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-16-review-pass-1.md`.

Review complete. Written to `reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-14-review-pass-2.md`.

**Key findings:**

1. **`demos/text_and_bytes/idiomatic.rs` — PASS**: Clean idiomatic implementation with correct UTF-8 encoding/decoding and hex parsing.

2. **`demos/file_streams/idiomatic.rs` — FAIL (Critical)**: The print statement outputs `ad_hoc_runtime_wave0_stream_hierarchy_contract_demo` but the Sifr source expects `ad_hoc_runtime_wave1_io_in_memory_hierarchy_demo`. Additionally, the Rust file's actual content (StringIO/BytesIO/BinaryFileHandle) matches the Sifr `in_memory_streams` demo, not `file_streams`.

3. **`demos/in_memory_streams/idiomatic.rs` — FAIL (Critical)**: Same issue in reverse — the print statement is correct (`ad_hoc_runtime_wave1_io_in_memory_hierarchy_demo`) but the content (simple `fs::write`/`fs::read` text+binary) matches the Sifr `file_streams` demo.

**The Rust implementations appear to be swapped between the two files.** The content of `file_streams/idiomatic.rs` is the wave1 in-memory streams demo, and `in_memory_streams/idiomatic.rs` is the wave0 file streams demo.

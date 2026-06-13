## Review: wave2 batch12 pass1 — `ad-hoc-idiomatic-rust-demo-corpus`

### readonly_bytes/idiomatic.rs

**Verdict: Accept**

Clean and focused. The four operations (index access, iteration, containment check, formatting) map directly to demo-visible behavior. `contains_byte` correctly rejects out-of-range values via `try_from`, and `cleanup` is idempotent. No behavioral issues.

### tempfiles_and_zip/idiomatic.rs

**Verdict: Accept, one observation**

The `mkstemp`/`mkdtemp` retry loop with 64 attempts and `AlreadyExists` forwarding is correct. The `ZipFile` API shape matches the demo lifecycle (create → write → namelist → read).

**Observation (non-blocking):** `ZipFile::namelist` creates an empty zip archive if the file doesn't exist — `let file = File::open(&self.path)?` would naturally error on a missing file, but the `rewrite_zip_with_entry` path always ensures the file exists before any `ZipFile` method is called, so this is never reached. It does add a minor API-shape inconsistency (different behavior than `read` which would panic on a missing file), but it doesn't affect the demo-visible behavior.

### filesystem_and_archives/idiomatic.rs

**Verdict: Accept, one observation**

The `glob_txt`, `gzip_compress`/`gzip_decompress` roundtrip, and `ZipFile` lifecycle all work correctly. The `main` function uses `and` chaining on the final error to ensure cleanup always runs — correct idiom.

**Observation (non-blocking):** `ZipFile::read` uses `read_to_string` on the decompressed entry stream. This works only because the `zip` crate's `FileOptions::default()` uses `Stored` (uncompressed) storage. If the demo intent is general zip handling, this could mislead — but for a demo showing `ZipFile` as a read/write API shape, it's accurate to the current implementation.

### No findings by severity

No behavioral regressions, no incorrect demo parity, no misleading API shapes, no missing edge handling that affects demo-visible behavior. The zip rewrite ceremony in both archive demos is necessary (append to existing zip requires full rewrite), not generated-style scaffolding.

### Final verdict

**Accepted.** All three files are appropriate Rust-first companions for their respective demos. They preserve demo-visible behavior, expose correct idiomatic APIs, and handle edge cases defensively without codegen-shaped ceremony.

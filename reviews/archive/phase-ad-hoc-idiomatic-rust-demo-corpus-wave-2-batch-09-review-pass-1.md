## Review: wave 2 batch 09 pass 1 — batch_09_binary_files_binary_hashing_binary_storage

### Findings by severity

**LOW — self-fulfilling assertion (binary_hashing)**
```rust
const MESSAGE: &str = "ad_hoc_rng_wave2_hashlib_base64_bytes_demo: pass";
assert_eq!(MESSAGE, "ad_hoc_rng_wave2_hashlib_base64_bytes_demo: pass");
```
`idiomatic.rs:22` — asserts a constant equals its own value. Passes trivially, tests nothing. If the demo intent is to verify the message string is preserved, the constant definition alone achieves that without the assertion. If a runtime assertion is desired, compare against a distinct expected value or drop the assertion entirely.

**ACCEPTABLE — weak hash assertion (binary_hashing)**
```rust
assert_eq!(digest.len(), 32);
assert_eq!(hex_string(&digest).len(), 64);
```
`idiomatic.rs:16-17` — only validates lengths, not actual hash output. Acceptable for a demo that focuses on API shape and encoding round-trips rather than cryptographic correctness.

**No issues found in binary_files or binary_storage.**

---

### Accepted/Declined

| File | Status | Rationale |
|------|--------|-----------|
| `binary_files/idiomatic.rs` | **Accepted** | Correct I/O contract, clean assertions, no ceremony |
| `binary_hashing/idiomatic.rs` | **Accepted with note** | Self-fulfilling assertion is inert but harmless; does not cause behavioral regression |
| `binary_storage/idiomatic.rs` | **Accepted** | Comprehensive byte ops coverage, correct edge handling, coherent demo intent |

---

### Final verdict

**Accepted.** The one inert assertion in `binary_hashing` is not a behavioral regression, misleading API shape, or readability failure — it is simply a no-op. No changes required; validation is meaningful as-is.

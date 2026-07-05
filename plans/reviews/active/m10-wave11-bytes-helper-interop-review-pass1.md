## Review: M10 Wave 11 — Bytes Helper Interop

**Scope note:** Reviewing from the described diff without tool access, so findings are at the design/contract level rather than line-verified. Nothing below is a confirmed code defect; they are contract concerns that warrant a look before merge.

### Findings (most significant first)

**1. `bytes_to_hex` returns `Result[str, ParseError]` for an infallible operation — non-blocking**

`encode_utf8` → `Vec<u8>` (infallible) and `bytes_to_hex` → hex string are both total functions: `&[u8]` → lowercase hex has nothing to validate and cannot fail. Yet the public signature is `bytes_to_hex(bytes: bytes) -> Result[str, ParseError]` and the Rust impl returns `Result<String, String>`.

Consequences worth confirming:
- The `Err` arm is **unreachable** — the generated `map_err(... ParseError)` path is dead code. That's tolerable but means every Sifr caller must handle an error that can never occur, which is an ergonomic tax and a slight API-honesty smell.
- Contrast with the *parse* direction (`bytes_from_hex`, `bytes_to_hex_strict`) which genuinely can fail. Having the infallible **format** direction also return `Result` blurs the distinction between the two and invites confusion about which functions actually fault.

Recommendation: either make `bytes_to_hex` return a plain `str` (dropping the Result), or document explicitly why the Result is retained (e.g. CPython API parity / forward-compat with a future validating variant). If kept, a one-line comment on the impl noting the `Err` branch is currently unreachable would prevent a future maintainer "cleaning up" the mapping incorrectly.

**2. `panic=trusted_no_panic` accuracy — looks correct, confirm the impls**

The annotation is sound *if* both impls are the trivial `s.as_bytes().to_vec()` / hex-format bodies described — neither indexes, slices, or unwraps, so no panic path exists. Worth a final glance that `bytes_to_hex`'s impl doesn't use any `.unwrap()`/indexing on the input slice (empty slice, non-ASCII bytes are all fine for hex formatting, so this should hold).

**3. Public/private wrapper — confirm no signature leak and no double-alloc regression**

The stated approach (private `_encode_utf8_impl` / `_bytes_to_hex_impl` taking borrowed `&str`/`&[u8]`, public wrappers taking owned `str`/`bytes`) is the right shape to avoid leaking borrowed interop signatures. Two things to eyeball:
- The public wrapper for `encode_utf8(s: str)` should borrow into the impl, not clone `s` first — otherwise you pay an extra copy before `to_vec()`.
- Ensure the public `bytes_to_hex(data: bytes)` wrapper threads the `Result` through unchanged rather than re-wrapping (which would compound finding #1).

### Areas that look sound from the description

- **Retired registry arms:** removing active arms for `encode_utf8`/`bytes_to_hex` and asserting they now return `None`, while retaining glue for `bytes_to_hex_strict`, `bytes_from_hex`, `bytes_with_size`, `bytes_from_ints`, and `str.encode`/`bytes.decode` via existing encoding paths, is a clean mixed migrated+retained split. The test asserting retired names return `None` *and* retained glue still lowers is the right guard.
- **Cargo feature planning:** enabling `sifr_stdlib` feature `bytes` for `sifr.bytes`/`_sifr.bytes`/`sifr.base64`/`sifr.hashlib`, plus the test asserting no raw third-party bytes dependency leaks for these helpers, is the correct dependency-hygiene check.
- **Adapter policy guard** including `_sifr.bytes` is consistent with the other migrated Wave modules.
- **Docs/ownership registry** moved to "mixed migrated + retained compiler glue" — accurate given the split above, provided the ownership TOML actually enumerates *which* symbols are migrated vs. retained (not just a blanket "mixed" label). Confirm the per-symbol accuracy there.

### Verdict

**PASS WITH NON-BLOCKING NOTES**

The migration structure, feature gating, adapter policy, and test coverage described are consistent and correct. The one substantive concern is the `Result[str, ParseError]` return on the infallible `bytes_to_hex` (finding #1) — resolve or document before merge, but it does not block. Findings #2/#3 are quick confirmations against the actual impls.

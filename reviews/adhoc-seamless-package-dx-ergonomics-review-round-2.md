

---

## Ergonomics Review Round 3: Final Assessment

### 1. Verdict

**READY.**

All five ergonomics items from the initial review are addressed. The phase is implementation-ready.

---

### 2. Findings

**Blockers: None.**

| Item | Status |
|---|---|
| No `[[bin]]` in production schema | ✅ Line 194 + `SIFR-PACKAGE-0711` |
| `[scripts]` structured command aliases | ✅ Lines 243–259, argv-array enforced, namespace collision handled |
| Manifest-less `sifr run path.sifr` | ✅ Lines 261–268, bounded bypass path, `SIFR-PACKAGE-0710` guard |
| Dependency groups | ✅ `[dependencies]`, `[test-dependencies]`, `[dev-dependencies]`, custom `[dependency-groups.<name>.dependencies]`, group composition via `include`, CLI flags |
| `[package.metadata.sifr]` as discovery hook | ✅ Lines 577–582, marked as not a trust anchor, `SIFR-PACKAGE-0703` |

**Non-Blocking Suggestions (not blockers):**

- **N-1:** `[package.metadata.sifr]` rationale is implied but not stated. One-sentence addition explaining the discovery purpose would help.
- **N-2:** `source_kind = "unknown"` could use concrete conditions for test coverage.
- **N-3:** `[scripts]` allow-list creates a maintenance bottleneck for new Sifr commands — monitor after launch.
- **N-4:** `root = "."` edge case undocumented but unlikely to matter in practice.

---

### 3. Required Edits

None.

---

### 4. Long-Term Maintainability Assessment

**Strengths:**

- argv-array scripts directly serve "if it compiles, it works" — no opaque shell strings.
- `SIFR-PACKAGE-0101` as single stable wrapper for all Cargo failures is the most important maintainability decision in the phase. Bounds diagnostic surface area indefinitely.
- Dependency groups projected to Cargo `[dev-dependencies]` for known groups; custom groups stay Sifr-native until Cargo has equivalent schema. Correct split.
- Pure marker lifecycle well-specified with `SIFR-PACKAGE-0501` guard prevents silent Rust-backend proliferation.
- Manifest-less bypass path explicitly bounded — can write targeted tests.

**Residual Risks (not blocking, must be monitored):**

1. **`[package.metadata.sifr]` fallback missing:** If Cargo removes the metadata field, Sifr has no defined fallback. Add search-for-`sifr.toml`-in-root fallback before phase closes, with tests.

2. **Custom groups diverge from Cargo:** If Cargo stabilizes `[dependency-groups]` with different semantics, Sifr must migrate or carry diverging semantics. Documented as known trade-off (line 498) — monitor Cargo roadmap.

3. **`[scripts]` allow-list bottleneck:** Adding `sifr lint` or any future command requires updating every `sifr.toml` with `[scripts]` entries. Consider schema evolution strategy (e.g., `command = ["sifr", "lint", ...]` for arbitrary commands).

4. **Dual compiler entry paths need test coverage:** Package-aware and manifest-less paths require parallel test coverage for every compiler change. Milestone 1 validation covers source layout but not single-file mode. Add `cargo test -p sifr -- manifest_less` coverage in milestone 1.

---

**Conclusion:** The phase is ready for implementation. The five ergonomics items are addressed with sufficient precision. Residual risks are documented as known trade-offs, not blocking gaps. Implementation can proceed.

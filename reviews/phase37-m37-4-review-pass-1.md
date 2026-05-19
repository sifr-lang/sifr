

## Phase 37.4 Code Review

**VERDICT: READY**

### Phase-Contract Coverage

All scope items from `internal_docs/phases/37_package_management.md` are implemented:

| Scope Item | Status |
|---|---|
| CargoCommandPlan APIs (metadata, fetch, build, package, publish, vendor, add, remove, update) | ✅ |
| CargoLockMode Normal/Locked/Offline/Frozen with mutation restrictions | ✅ |
| Offline/frozen Sifr source availability validation (SIFR-PACKAGE-0104) | ✅ |
| Cargo failure mapping (SIFR-PACKAGE-0101, SIFR-PACKAGE-0105 with credential redaction) | ✅ |
| Backend trust policy validation (SIFR-PACKAGE-0301, SIFR-PACKAGE-0305) | ✅ |
| Package build cache input digest API | ✅ |
| Diagnostic registry/docs for active package diagnostics | ✅ |

### Correctness Analysis

**CargoCommandPlan** (`commands.rs`):
- Feature selection uses `stable_csv` for deterministic output
- Add/remove/update omit lock mode (mutation commands shouldn't use locked/offline/frozen)
- All 9 command variants have builder factories

**Credential Redaction** (`errors.rs`):
- Covers `token=`, `Bearer`, `gho_`, `cargo:token`, URL patterns
- `looks_like_credentials_error` catches authentication/credential/unauthorized/403/401 patterns
- Correct dispatch to `SIFR-PACKAGE-0101` vs `SIFR-PACKAGE-0105`

**Trust Policy** (`trust.rs`):
- `SIFR-PACKAGE-0301` fires for backend crates not in `trust.native`
- `SIFR-PACKAGE-0305` fires for trust entries not in direct backend dependencies
- Per-phase-contract: transitive trust is NOT inherited; each package declares trust for its own direct deps

**Cache Digest** (`digest.rs`):
- `fnv1a64` is a standard non-cryptographic hash
- Features and selectors are sorted before hashing for determinism
- All `BTreeMap` uses guarantee stable iteration order
- All canonical structs serialize only the semantically-relevant fields

**Offline Validation** (`lock_modes.rs`):
- Filters to `sifr_metadata.is_some()` packages only
- `is_dir()` returning `false` correctly identifies unmaterialized sources (expected path)
- Diagnostic includes the lock mode for actionable remediation

### Test Coverage

Tests cover all five major components with edge cases:
- Feature reordering → same digest (determinism)
- Lock mutation → digest change
- Offline unavailable source → SIFR-PACKAGE-0104
- Credential redaction → SIFR-PACKAGE-0105, no secrets in output
- Untrusted backend → SIFR-PACKAGE-0301
- Stale trust entry → SIFR-PACKAGE-0305

### Diagnostics Quality

- All 5 new codes registered in `DiagnosticCode` constants
- All 5 codes have `active_entry!` in `DIAGNOSTIC_REGISTRY`
- All 5 docs pages exist (`SIFR-PACKAGE-0101`, `0104`, `0105`, `0301`, `0305`)
- Message templates use `{placeholder}` syntax, not backticks
- Dedupe args subset of declared args
- Reserved codes (0302, 0306-0309) present per phase contract

### Integration Risks

None identified. The implementation is additive (new types/APIs), does not modify prior milestone behavior, and validation gates all passed.

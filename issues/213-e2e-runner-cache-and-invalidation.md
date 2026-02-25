## [Task] Add Persistent Cache with Safe Invalidation

#### Current Situation
- Repeated runs rebuild too much work even when inputs are unchanged.
- No formal cache manifest governs reuse validity.

#### Desired Situation
- Unchanged fixtures/groups reuse valid cached artifacts.
- Any correctness-relevant change invalidates cache safely.
- Cache can be disabled for debugging.

#### Suggested Solution
- Implement cache root under `target/sifr_e2e_cache/`.
- Store manifest keyed by:
  - fixture source hash
  - generated rust hash
  - dependency fingerprint
  - Sifr compiler identity (git commit hash or build/version identifier)
  - `rustc -Vv` full string (version + commit + host)
  - `cargo -V` full string
  - target triple and OS/arch
  - explicit env allowlist:
    - `RUSTFLAGS`
    - `CARGO_ENCODED_RUSTFLAGS`
    - `RUSTC_WRAPPER`
    - `SIFR_E2E_RUNNER_MODE`
    - `SIFR_E2E_NEW_RUNNER`
    - `SIFR_E2E_LEGACY_RUNNER`
- Add `SIFR_E2E_DISABLE_CACHE=1` escape hatch.

#### Implementation Checklist
- Design and serialize cache manifest format.
- Define cache key schema as a versioned contract (`cache_schema_version`).
- Implement read/validate/reuse logic.
- Implement fallback rebuild path on cache mismatch or corruption.
- Add tests for invalidation triggers.

#### Acceptance Criteria
- Warm reruns skip unchanged work correctly.
- Stale artifacts are not reused.
- Cache can be disabled with one env var.
- Any change to a key field above forces cache miss and rebuild.

#### Dependencies
- Depends on Task 212.

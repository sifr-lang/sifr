I've verified each round-1 finding and run the full validation suite. All scaffold checks pass (`fixtures=30 diagnostics=10 crates=44`; `tiers=5 fixtures=30`; stale-draft scan clean; `sifr_verify areas run --area rust_interop` reports 0 failures; `areas check` clean).

## Round 1 closure status

All eight round-1 findings are closed:

1. **opaque_resource_matrix feature pins** — `data/rust_interop_fixture_matrix.json:105-110` now pins reqwest/rusqlite/tokio-postgres/redis exactly per architecture lines 925-928.
2. **`check_fixture_matrix.py` feature validation** — `EXPECTED_FEATURE_POLICIES` (lines 110-124) covers the 10 feature-sensitive crates; `_validate_feature_policies` (lines 210-231) asserts exact match and flags missing `features` blocks.
3. **Stale checks for RustPanicError and python fences** — `RUST_TARGET_DECORATOR` + `_scan_panic_surface` (lines 21, 90-105) flag `@rust(...)` defs that return `Result[...]` without `RustPanicError`, while `SIFR_INTEROP_DECORATOR` (line 22, 51-53) flags Sifr interop decorators inside python fences. Architecture/phase 39 corpus passes (every existing `@rust(...)` example either carries `panic=...` or `RustPanicError`).
4. **`REJECTION_CONTEXT` tightening** — `_is_rejection_context` now checks only `line[:match_start]` and is anchored to a fixed marker list (lines 69-87). The five active occurrences in `architecture.md`, `python-interop.mdx`, and `39_rust_interop.md` all sit behind one of the markers (`no `, `not `, `stale`, `rejected`).
5. **`SIFR-RUST-*` placeholders** — `docs/diagnostics/error-codes.mdx:138-155` contains a RUST accordion with all ten codes.
6. **`prost-build` placement** — moved to `proc_macro_trust` (matrix lines 284-292), matching architecture line 897.
7. **`RustBridgeProbePlan` rename** — `runner/cargo_probe.py:10` adopts the canonical identifier; `BridgeProjectionCheck`/`TrustEvidence`/`NativeLinkEvidence` remain as skeleton names (round 1 marked these "acceptable for placeholders").
8. **Offline-policy documentation** — `verification/areas/rust_interop/README.md:14-17` documents that runtime-observed Redis/Postgres fixtures must use explicit local service config, not silently degrade.

## Remaining gaps against milestone_39_0 DoD

No blocking findings. The three DoD items hold:

- All 30 architecture-required fixture directories and matrix entries are present and tier-assigned exactly once.
- `plans/phases/39_rust_interop.md` and `internal_docs/rust_interop_architecture.md` agree on capabilities, rejected designs, trust schema keys, feature pins, and the 10 diagnostic families.
- No accepted `extern rust`/`dlopen`/legacy `@rust(crate=…)`/`native = [` syntax remains in active docs.

## Non-blocking observations (scaffold quality, deferrable)

- **`runner/report.py:to_jsonable`** is shallow: `asdict()` recurses into the dataclass but does not stringify nested `Path` fields, so `json.dumps(to_jsonable(RustBridgeProbePlan(...)))` would raise. No current caller exercises it (skeleton-only).
- **`_validate_feature_policies`** uses exact equality. Future fixtures that augment a pinned crate (e.g. `tokio-tungstenite` + `rustls-tls-webpki-roots` for TLS coverage, allowed by architecture line 929) will require widening the check; today no such fixture exists.
- **`_is_rejection_context`** still keeps bare `no ` / `not ` markers in the prefix-only check. Lines like `Sifr is not Python; we use extern rust here for X` would still be exempted. Not present in current corpus; not blocking.
- **`_next_fence_language`** returns the full post-``` token (e.g. ` ```python title="…" ` becomes `"python title=\"…\""`), so the `fence_language == "python"` test misses titled fences. Current docs have no `@rust(...)` examples in titled python fences.
- **`_scan_panic_surface`** only flags a defect when `Result[` is in the def. A `def f() -> None` with `@rust(...)` and no `panic=` would not be flagged. Doesn't occur today.

**Bottom line:** Round-1 findings are all closed, scaffold validation is green, and the milestone_39_0 definition of done is satisfied. The five items above are minor scaffold-quality nits that do not block the milestone and have no failing case in the current corpus.

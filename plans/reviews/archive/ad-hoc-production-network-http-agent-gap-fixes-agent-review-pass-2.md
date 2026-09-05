Verification complete. Drafting the pass-2 review.

## Pass-2 verification summary

**Verdict: PASS.** All three pass-1 cleanup suggestions are applied correctly and verified. The pass-1 F1 residual is fully resolved in the `.md` inventory; F3 is fully resolved by the hardened snapshot test; the cookie row is now consistent with the shipped Sifr-owned parser.

### Findings

**No blocking findings.** One borderline same-class residual noted below for traceability only — it is not introduced by this branch.

#### Borderline (non-blocking, pre-existing on `main`)
The sibling JSON inventory `verification/stdlib/network_http_substrate_inventory.json:113` still carries `ProtocolError::UnsupportedExtensionFrame` in the `http2_priority_extensions` outcome cell. It is the only `Error::` reference remaining in the JSON, was not modified by this branch (verified via `git show main:…`), and is the JSON analog of the pass-1 F1 class. Scope is identical: not enumerated by agent, covered by the "and similar variant paths" amendment in spirit, and easy to flip in a small follow-up that also re-derives the JSON inventory from the same source as the `.md`. Calling it out only so future reviewers don't think the sweep missed it — it was never in scope.

### Pass-1 cleanup verification

#### Cleanup 1 — residual variant references in `network_http_substrate_inventory.md` outside the taxonomy
- `grep -n "Error::"` over the file returns only `:184` (the agent amendment itself, which intentionally names the rejected variants).
- All 13 specific cells enumerated in pass-1 F1 are now rewritten to flat-class + deterministic-evidence phrasing:
  - `:51` → "unsupported-extension-frame evidence"
  - `:79-80` → "invalid-name evidence" / "obs-fold rejected"
  - `:94` → "obs-fold evidence"
  - `:97-98` → "conflicting-content-length" / "length-mismatch" / "ambiguous-body-length"
  - `:107` → "trailers-unsupported evidence"
  - `:111-113` → "read-cancellation and `bytes_observed`" / "write-cancellation and `bytes_accepted`" / "stream-reset code and byte-observation"
  - `:124-127` → "ping-flood" / "stream-reset" / "connection-closing" / "malformed-frame kind"
  - `:134-141` → flat class + "size-limit evidence" (URL/Query/Header/Body/TLS rows)
  - `:149,151` → "invalid-port evidence" / "invalid-percent evidence"
- The `Error Taxonomy` table at `:182-195` keeps the 8-class flat list and carries the dated agent amendment. No drift between amendment, taxonomy, and individual rule cells now.

#### Cleanup 2 — cookie dependency row correction in `network_http_dependency_audit.md`
- Row rewritten to:
  - decision: `rejected for this phase; no production crate is emitted`
  - hiding: `Sifr-owned cookie-header parser`
  - typed-error mapping: `invalid cookie syntax maps to HeaderError`
  - lockfile impact: `no production dependency`
- Confirmed against shipped code:
  - `crates/sifr_stdlib/src/features.rs:145-147` — `COOKIE_DEPS: &[]` with explanatory comment "Cookie-header helpers are Sifr-owned string/header validation; no cookie jar".
  - `lib/sifr/http.sifr:181-186` — `parse_cookie_header` / `build_cookie_header` delegate to `http_parse_cookie_header` / `http_build_cookie_header` (intrinsics), not a `cookie` crate.
- Substrate-inventory row at `network_http_substrate_inventory.md:69` already said "Sifr-owned parser; no external cookie crate" — audit and inventory now agree.

#### Cleanup 3 — snapshot-vs-generator structural equality hardening
- `network_http_snapshot_json_matches_generated_dependency_output` (`crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:119-236`) now asserts, for each of the four production snapshots:
  - `production_dependencies` equals normalized generator output (was the only check in pass-1).
  - `required_features` equals an explicit literal list.
  - `must_not_include` equals an explicit literal list.
  - Top-level `status` equals `"closed-audited"`.
- Cross-checked every literal against `verification/stdlib/network_http_dependency_snapshots.json`:
  - `network-runtime-core` (`:17-39`), `tls-runtime` (`:46-69`), `url-header-cookie` (`:76-91`), `http-transport` (`:98-130`) — all three array fields per snapshot match byte-for-byte.
- `cargo test -p sifr_stdlib --test network_http_dependency_snapshots` → **9 passed**.
- Note: the snapshot JSON's per-snapshot schema is `{id, owner_milestones, required_features, production_dependencies, must_not_include}`. The hardening covers every array field that exists; there is no `manifest_codegen_requirements` field at the snapshot level (it lives in `features.rs` per-feature), so the structural check is now complete for this artifact.

### Runtime/test signals re-verified

- `cargo test -p sifr_runtime --features http` → 36 passed (timeouts overflow unit test still included; TLS loopback + H2 + body-limit fixtures still green).
- `cargo test -p sifr_stdlib --test network_http_dependency_snapshots` → 9 passed.
- Working tree diff against `main` is exactly the set of files the pass-1 review described, plus the three cleanup deltas above; nothing unrelated has crept in.

### What changed since pass-1
- `verification/stdlib/network_http_substrate_inventory.md`: ~70 lines of cell rewording across the HTTP type, header/smuggling, body, HTTP/2, size, and URL tables. Amendment text unchanged.
- `verification/stdlib/network_http_dependency_audit.md`: cookie row rewritten to no-production-crate; surrounding rows untouched in this pass.
- `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs`: new test extended by ~120 lines of literal `required_features` / `must_not_include` assertions and a top-level `status` check.

### Suggested follow-ups (none blocking)

1. Sweep the one residual `ProtocolError::UnsupportedExtensionFrame` in `network_http_substrate_inventory.json:113` — same wording fix as the `.md` cell at `:51`. Not in scope for this branch, not enumerated by agent, but the only place an unhelpful reader can still derive a contradicted variant name in the inventory pair.
2. Post-merge: flip the three "in progress" rows in `issues/…-execution.md:451-454` and drop the trailing "remediation in progress" clause in `issues/…-platform-substrate.md:3`; add a ledger row pointing at this pass-2 review artifact (pass-1 review row equivalent).

**PASS.** All six agent blockers and the three pass-1 cleanup items are satisfied; tests pass; remaining residual is pre-existing on main and not a re-FAIL.

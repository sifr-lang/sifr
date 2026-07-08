## Findings

I reviewed the working-tree diff against `main` and the pass‑1 review artifact against the current state.

**Blocker 1 — `evidence_links` required non-empty: FIXED**
- `scripts/check_stdlib_manifest_schema.py:90` now uses `_required_string_list`, which rejects both missing key and empty list (helper at `:148-158` — the `not isinstance(value, list) or not value` guard covers both).
- Self-test coverage added at `:198-205` (pops `evidence_links` and asserts the "must be a non-empty list" failure).
- Manifest content is consistent: every one of the 20 surfaces carries `evidence_links = ["stdlib-native-boundary-completion"]`, including all `retained-by-design` entries.

**Blocker 2 — guardrails.json registration: FIXED**
- `verification/policy/guardrails.json:60-66` registers `stdlib-manifest-schema` with the correct entrypoint, mirroring `stdlib-native-intrinsic-allowlist` (no `--self-test` in the policy inventory; self-test runs from `profile_runner.py:357`, matching precedent).

**Defensive 3 — pipe-chained lowerer match arms: FIXED**
- `scripts/check_stdlib_native_intrinsic_allowlist.py:22` uses the lookahead `(?=\||=>)`, matching `EXACT_INTRINSIC_RE`. Consolidated arms `"a" | "b" =>` in `tls.rs` / `url_http.rs` / `python.rs` will now yield both names.

**Defensive 4 — untracked prefix dispatchers rejected: FIXED**
- `scripts/check_stdlib_native_intrinsic_allowlist.py:24` pins `EXPECTED_PREFIX_DISPATCHERS = {"http_", "py_", "tls_"}`; validation flags both unexpected and stale prefixes at `:126-138`.
- `registry.rs:401,405,422` still shows exactly those three `starts_with` arms — invariant preserved.
- Self-test at `:247-257` confirms an added `s3_` is rejected.

**Create-pr taxonomy — FIXED**
- Manifest self-test at `scripts/check_stdlib_manifest_schema.py:172-190` uses the stable id `stdlib-native-boundary-completion` and neutral removal wording (`"migration lands"`), no active-plan paths or delivery-plan terms.

**Live guard runs (spot-checked in this session):**
- `check_stdlib_manifest_schema.py` main + `--self-test`: PASS (20 surfaces, schema_version=2).
- `check_stdlib_native_intrinsic_allowlist.py` main + `--self-test`: PASS (276 exact intrinsics, 28 registry files, 16 preamble files).
- `check_sysroot_stdlib_resource_certification_gate.py` main + `--self-test`: PASS (11 surfaces, 11 future_runtime_rows).

**Scope note (non-blocking) — certification gate refactor is in-diff but not in the summary**
- `scripts/check_sysroot_stdlib_resource_certification_gate.py:20-33` removes the hardcoded `SURFACE_CERTIFICATION_ROWS` constant and derives surface→row mappings from the retained manifest (`_surface_certification_rows` at `:82-116`). Not mentioned in the pass‑2 summary, but it is a reasonable direct consumer-of-manifest change within the M1a wave.
- Row-ID coverage is equivalent to the prior constants (`opaque_resource_matrix`, `async_runtime_reqwest`, `callback_subscription_matrix`, `callbacks_call_scoped` all still assert), and the manifest carries the same 11 surfaces that map to those rows. Self-test coverage added at `:184-190` for a malformed manifest. No regression identified.

**Observations that are not blockers but flagged for later milestones (from pass‑1 finding 5, still present)**
- `internal_docs/stdlib_retained_compiler_intrinsics.toml`: every retained (non-by-design) row uses the identical `removal_criteria = ["Migrate or classify this retained compiler-native entry before final audit."]`, and every row uses the identical `owner`, `issue`, and `evidence_links = ["stdlib-native-boundary-completion"]`. Schema landing is fine; per-row signal should be populated by subsequent M1 sub-waves.

## Verdict

**READY** — all pass‑1 blockers and defensive suggestions are addressed, create-pr taxonomy is green, and no new blockers surfaced in pass‑2. M1a stands as a self-contained schema/allowlist sub-wave; the certification-gate manifest-derivation is a natural consumer update that does not require additional M1 work.

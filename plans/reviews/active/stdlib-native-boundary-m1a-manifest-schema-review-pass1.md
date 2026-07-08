## Review findings

### Blockers

**1. `evidence_links` is not actually required by the schema validator, contrary to the stated intent** (`require owner/issue/reason/evidence`)
- File: `scripts/check_stdlib_manifest_schema.py:87-98`
- The `owner`, `issue`, `reason` fields are enforced via `_required_text`, but `evidence_links` is only run through `_optional_string_list` (line 90-98), which returns silently when the key is absent AND accepts an empty list.
- Failure scenario I reproduced: a manifest surface entry with `owner`, `issue`, `reason`, `removal_criteria`, `registry_files` but **no** `evidence_links` (or `evidence_links = []`) passes `_validate`. So a new retained row can land with zero provenance and the guard says PASS.
- Fix: add `evidence_links` to the required set, and enforce non-empty (either by a `_required_string_list` helper or by requiring the key + rejecting empty).

**2. New guard script not registered in `verification/policy/guardrails.json`**
- File: `verification/policy/guardrails.json:52-66`
- Precedent from `c5e15e074` (stdlib native intrinsic allowlist) and `ed8b7f0ae` (stdlib migration closure) is that new stdlib guards land in both `profile_runner.py` and this policy inventory. `check_stdlib_manifest_schema.py` is only wired into `profile_runner.py:356-358`; the policy inventory is now out of sync.
- Failure scenario: an operator consulting the policy manifest as the canonical guardrail list misses the new schema gate, and any tooling that snapshots/publishes this inventory loses coverage.
- Fix: append a `{"name": "stdlib-manifest-schema", "entrypoint": "scripts/check_stdlib_manifest_schema.py", ...}` entry.

### Non-blocking findings

**3. Removing `PREFIX_INTRINSIC_RE` also removed the guard against *new* `starts_with("...")` dispatchers appearing in `registry.rs`**
- File: `scripts/check_stdlib_native_intrinsic_allowlist.py:22-27`, cross-ref `crates/sifr_codegen/src/intrinsics/registry.rs:401-425`
- Observation now consists of (a) exact-arm regex in `registry.rs` plus (b) exact match arms inside three hardcoded lowerer files. If a future PR adds e.g. `name if name.starts_with("s3_") => s3::lower_s3_intrinsic(...)` with a new lowerer module, the guard is silent — none of the new intrinsic names are observed and no allowlist entry is required.
- Suggested defensive fix (does not need to block M1a but should be tracked): assert in the guard that the only `starts_with("<x>_")` occurrences in `registry.rs` are the three known ones (`tls_`, `http_`, `py_`), and hard-fail on any new prefix. This keeps the "no new native surface without allowlisting" invariant even if a subsequent lowerer is prefix-shaped.

**4. `LOWERER_MATCH_INTRINSIC_RE` does not handle pipe-chained match arms**
- File: `scripts/check_stdlib_native_intrinsic_allowlist.py:22`
- The regex `r'"([A-Za-z0-9_]+)"\s*=>'` catches only the last name in an arm like `"tls_a" | "tls_b" => lower_x(args)`. No such arms exist today in `tls.rs` / `url_http.rs` / `python.rs`, but if someone consolidates arms (mirroring the `"local_callback" | "threadsafe_callback"` shape already used in `registry.rs:426`), the leading names silently drop out of the observed set.
- Suggested fix: use the same lookahead as `EXACT_INTRINSIC_RE`, i.e. `r'"([A-Za-z0-9_]+)"\s*(?=\||=>)'`.

**5. `removal_criteria` is uniformly boilerplate on every retained row**
- File: `internal_docs/stdlib_retained_compiler_intrinsics.toml`
- Every non-`retained-by-design` row currently uses the identical string `"Migrate or classify this retained compiler-native surface before M13 final closure."`, and every row uses the identical `owner` and `evidence_links`. The schema landing is fine, but the metadata carries no per-row signal today. Subsequent milestones should populate row-specific criteria. Not blocking M1a since the field is now required by schema, but worth noting in the follow-up plan so the fields don't stay as decorative constants.

### Positive observations

- Schema self-test verifies rejection of `prefix_intrinsics`, missing `removal_criteria`, and invalid `state` values (`scripts/check_stdlib_manifest_schema.py:178-193`). Good.
- Exact enumeration of `tls_*`, `http_*`, `py_*` was cross-verified: manifest counts (25 tls / 8 http / 73 python) match the match arms in `tls.rs:298-330`, `url_http.rs:69-80`, and `python.rs:3-84`.
- Guard `PASS` output shows 276 exact intrinsics and 20 surfaces, confirming end-to-end wiring.
- Profile-runner placement (`verification/runner/sifr_verify/profile_runner.py:356-358`) is sensible — schema runs between allowlist and migration closure, matching the logical dependency.

### Untested before PR

- `scripts/run_all_tests.sh --profile create-pr` has not been run yet on this branch per the summary. That is the authoritative gate and must pass before merge.

---

NEEDS_CHANGES — concrete blockers: (1) enforce `evidence_links` in `scripts/check_stdlib_manifest_schema.py` (add to required fields and reject empty), and (2) register `check_stdlib_manifest_schema.py` in `verification/policy/guardrails.json`. Also run the full `scripts/run_all_tests.sh --profile create-pr` before opening the PR.

## Exact-head review round 3 — PR #3071 (certification_10, Proc-Macro and Codegen Trust)

### Scope pinned
- Published head `refs/heads/agent/rust-interop-certification-10` = `4e73e3cddbe6b4ef5875bd2ea697713f4730a866` — exactly as required.
- `refs/heads/main` = `afd25c3920a646fb0eea273c6899010baa7e94b7`; `git merge-base 4e73e3cdd origin/main` = `afd25c392`, so the base is exact and the branch is linear. `gh pr view 3071` reports `headRefOid` `4e73e3cdd…`, base `main`, `MERGEABLE`.
- Two commits: `d0adfa91b` (37 files) + `4e73e3cdd` (doc-only, +6 lines to the issue).
- Excluded per instruction and confirmed excluded: the only PR-file difference between head and the shared worktree is the unstaged `ecosystem_backend_certification` hunk in `rust_interop_compatibility_matrix.json` (`git diff 4e73e3cdd -- <all 37 PR paths>` lists that one file and nothing else). All other ignored items are outside the PR path set.

### 1. PR content is certification_10 only ✓
All 37 paths are certification_10 implementation (`rust_interop.rs`, `trust_validation.rs`, `target_resolution.rs`), tests (`rust_interop_trust_tests.rs`, `package_rust_interop_proc_macro_support.rs`, `package_rust_interop_build_tests.rs`, `rust_interop_tests.rs`), the `proc_macro_trust` fixture/scenario package, the five checkers/scenario modules, docs (`docs/rust-interop.mdx`, both internal docs), data (fixture matrix, compatibility matrix, `stable_support_claims.json`, `fixture.json`), the issue, and the round-1/round-2 review `.md` files. The second commit adds only the create-PR validation paragraph. Only `.md` review artifacts are committed — matching the certification_9 convention (`git ls-tree afd25c392 plans/reviews/active/ | grep certification-9` → four `.md`, zero `.claude.log`), so round 2's "artifacts chore" is resolved by convention, not outstanding.

### 2. Committed matrix promotes only `proc_macro_trust` ✓
`git diff afd25c392 4e73e3cdd -- …compatibility_matrix.json` is a single hunk at `@@ -434,15`: `proc_macro_trust` → `supported`, `future_owner` dropped, both evidence statuses `planned`→`passing`, notes rewritten. The backend hunk is absent from the commit. The committed blob still carries `ecosystem_backend_certification` as `future-owned-by-separate-phase` with `future_owner: plans/issues/active/rust-interop-runtime-ecosystem-certification.md` and both evidence directions `planned`. Future-owned set is exactly `['ecosystem_backend_certification', 'ecosystem_cli_certification', 'cargo_locked_offline']`, matching the corrected `internal_docs/sifr_sysroot_and_stdlib_architecture.md:153-161` text.

Inventory derived from the committed blob: 36 rows; 20 `supported` / 12 bridge / 1 unsupported-by-design / 3 future-owned; kinds 13 `cargo-probe` / 4 `compiler-diagnostic` / 10 `contract-only` / 9 `runtime-observed`; 66 `passing` + 6 `planned` — the issue's expected post-item inventory, exactly.

**Stronger than round 2 could show:** I exported the committed tree (`git archive 4e73e3cdd`) to `/tmp/cert10head` and ran the checkers there, so the excluded hunk was genuinely absent rather than mentally subtracted:
- `check_compatibility_matrix.py` → `rows=36 fixture_rows=36 categories=4`, **exit 0** (round 2's sole area failure was that hunk, now proven).
- `check_tiers.py` → `tiers=5 fixtures=36`; `check_stable_support_claims.py` → `claims=33`; `check_stale_drafts.py` → ok.
- `check_fixture_matrix.py` cannot run in the export (needs the `third_party/ruff` submodule); it was run in the real worktree, where the file content is byte-identical to head → `fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=18`, `--self-test` → `cases=184`.

### 3. Every round-1 remediation is in the committed blobs ✓
1. **Negative fixture type-correctness.** `negative/…pre_execution.sifr:8` declares `-> str`, local is `str`; bridge is `pub fn decode(input: &[u8]) -> String`.
2. **Mutation guard.** `check_fixture_matrix.py:855-876` rejects reintroduction of `-> bytes` ("must match the scenario bridge `str` return"), plus the marker guard at `:730`.
3. **Declaration attribution.** `trust_validation.rs:validate_package_dependency_trust` prefers a declaration whose root segment equals `backend.dependency_name`, falling back to `declarations.first()`; asserted by `package_rust_interop_attributes_package_trust_to_matching_declaration` (`diagnostics.len()==1`, contains `app.hash`, **not** `app.bridge_call`).
4. **User-visible allow-list keys.** `require_trust` renders ``add `{required_entry}` to `[trust].{allowlist_name}` ``; `trust_allowlist_name` maps all seven kinds, and every rendered name matches a real manifest key in `crates/sifr_package/src/manifest/sifr_fields.rs:77-108` (`trust.rust-build-scripts`, `trust.rust-proc-macros`, `trust.native-links`, `trust.build-env`, `trust.unsafe-rust-bridges`, `trust.rust-no-panic`, `trust.rust-panic-abort`).
5. **Direct/local proc-macro coverage.** Both `…_for_direct_root` (`:60`) and `…_for_local_bridge` (`:81`) present; seven tests in the file, matching the issue's corrected "seven focused trust tests".
6. **Marker wording.** `serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed` at 14 committed sites; the only remaining `macro=executed` string is the round-1 artifact quoting the old text.
7. **Decomposition.** `rust_interop.rs` 853, `trust_validation.rs` 168, `_scenario_checks.py` 745, `_scenario_registry.py` 140, `_scenario_proc_macro.py` 403.

### 4. Round 2 still holds at exact head; no regression from the doc-only commit ✓
`4e73e3cdd` touches only `plans/issues/active/…md`, adding six lines. Re-run at head:

| Gate | Result |
|---|---|
| `cargo test -p sifr_driver` | 432 passed, 0 failed, 61 ignored |
| both mandatory ignored tests | 2 passed, 43.10s |
| `check_fixture_matrix.py` / `--self-test` | 36/10/44/60/18 · 184 cases |
| committed-tree compat matrix / tiers / claims / stale drafts | ok · 36/36/4 · 5+36 · 33 · ok |
| `cargo clippy -p sifr_driver --lib -- -D warnings` | clean (full recompile) |
| `cargo fmt --check`, `git diff --check afd25c392 4e73e3cdd` | clean |
| file-size (2982 files, limit 900), `sifr_driver` maintainability, HIR/lowering guardrails | PASS |

### 5. Validation honesty ✓
PR body: "authoritative create-PR profile passed every preceding step, including all 19 Python-interop variants; Rust interop passed 9/10 and stopped only on the unrelated unstaged `ecosystem_backend_certification` promotion with planned evidence. The committed PR matrix keeps that row future-owned." The committed issue note (`4e73e3cdd`) says the same. Both are accurate: the create-PR profile did not fully pass, the stop cause is named, the cause is unstaged, and my independent run of the checker against the committed tree confirms the committed matrix is clean. The claim is stated as a partial pass, not a pass. Round 1 `NOT SATISFIED` / round 2 `SATISFIED` are both disclosed with the finding counts, and every number in the body reproduces.

### 6. Substantive properties re-audited ✓
- **Exact pins:** scenario `Cargo.toml` uses `=0.14.4` (prost, prost-build, prost-types) and `=1.0.228` (serde_derive); `require_root_lock_subset` (`_scenario_lock_checks.py:35-56`) requires every sourced scenario-lock identity in the root lock, and runs for every scenario.
- **Deterministic codegen:** `build.rs` uses `Config::new().compile_fds(descriptor)` on an in-memory `FileDescriptorSet` (no `protoc`), writes only under `OUT_DIR`, and self-checks the schema; the mandatory test compares two fresh `--locked --offline --frozen` target dirs byte-for-byte and asserts the version evidence and `pub struct Probe` / `pub id: u64` tokens.
- **Trust-present negative validity:** `assert_armed_build_time_dependencies_write_sentinels` runs *first* (line 81 of the test), arms both sentinels, requires the build to succeed and both sentinels to exist, then installs the negative source and asserts `check_package_project` returns **no** errors.
- **Independent armed-sentinel rejection:** each of the two removals asserts `errors.len() == 1`, code `RUST_TRUST_MISSING`, presence of the entry, the exact `[trust].<key>`, the kind-specific evidence string, "before Cargo executes this dependency", and that **neither** sentinel file exists.
- **Cache identity:** `package_rust_interop_cache_changes_with_proc_macro_trust_policy` asserts `cache_key_fragment()` differs between empty and `["serde_derive"]` `rust_proc_macros`.
- **Safety:** no new `unsafe` in production or fixture sources; every added `.expect(` in the diff is test-only; `check_fixture_matrix.py:1509-1511` additionally forbids the scenario from granting `unsafe-rust-bridges` trust.
- **Docs/claims/provenance:** `fixture.json` records `profile: merge`, `step: crate_tests`, suite and exact test names for both directions; the stable-claims table entry and `docs/rust-interop.mdx` prose are consistent with the manifest key `rust-build-scripts = ["prost_build"]` (normalized Cargo alias).

### Findings
No actionable findings. Non-blocking, unchanged from round 2:
- **Headroom (nit).** `check_fixture_matrix.py` is 899 lines and `package_rust_interop_build_tests.rs` is exactly 900 — at the guardrail cap. The next addition to either forces a split.
- **Fallback attribution (nit).** The prepass fallback is `declarations.first()`; for a package with several declarations and a never-referenced build-time dependency the diagnostic anchors on the first declaration. Fail-closed, and the message still names the dependency and exact `[trust]` key.
- **Round-3 recording (expected follow-up).** No round-3 link in the issue at this head — the certification_9 precedent records rounds 3/4 in separate doc-only commits after PR head (`1d66d90b0`, `d78fc6bc6`), so this is the next step, not a defect at `4e73e3cdd`.

No files were modified.

VERDICT: SATISFIED

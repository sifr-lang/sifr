# certification_10 (Proc-Macro and Codegen Trust) — Review

Scope confirmed: `HEAD == origin/main == afd25c392`, so the working tree *is* the diff. I excluded `editor_integrations`, `verification/areas/algorithmic_compatibility/corpora/leetcode`, `.cert5probe/`, `.claude/`, the stray `*.webp` files, `plans/phases/43_interoperability.md`, and the `ecosystem_backend_certification` hunk in `rust_interop_compatibility_matrix.json:396-400`. That hunk is textually separate from the `proc_macro_trust` hunk at `:433-444`; the proc_macro_trust promotion is self-contained and valid.

## What I independently confirmed as satisfied

- **Exact pins / locked build (#1).** `examples/proc_macro_trust_package/Cargo.toml:6-9` pins `=0.14.4`/`=1.0.228`; scenario `Cargo.lock:187-292` matches root `Cargo.lock:5535-6776`; `require_root_lock_subset` runs for every scenario (`_scenario_checks.py:403-411`). `serde_derive-822530523e6034d2` and `prost-build-a88b9083dff8257a` appear in the build fingerprints, so both really compile.
- **Derive execution + safe bridge glue (#2).** `GeneratedSchema::sifr_proc_macro_marker()` (`src/bridges/generated.rs:1-11`) only exists if the derive ran, so compilation is the proof; no `unsafe`, and `reject_unsafe_rust` now covers this fixture (`_scenario_checks.py:343-349`).
- **prost-build without protoc, OUT_DIR, determinism, runtime observation (#3).** `build.rs:20-49` builds the descriptor in memory and calls `compile_fds`; output stays under `OUT_DIR`; `src/lib.rs:1-13` compiles it; two fresh `--locked --offline --frozen` target dirs are byte-compared (`package_rust_interop_proc_macro_support.rs:27-52`) and the runtime asserts `id=1404|…`.
- **Package-wide pre-execution validation (#4).** `trust_validation.rs:12-51` called at `rust_interop.rs:172`, before `execute_pending_direct_probes()` at `:185`; `sysroot`-trust handling matches the per-target path (`is_trusted_sysroot_package` ≡ `sysroot_trust_for_package(..).is_some()`). Dedup via `seen_trust_requirements` (`rust_interop.rs:669-676`) plus the fail-fast bail at `:272` prevents duplicate emission.
- **Independent removal + armed sentinels + positive control (#5).** Verified end-to-end: I rebuilt `target/debug/sifr` (the on-disk binary was stale from 18:28) and reproduced both removals producing `SIFR-RUST-TRUST-0001`. Both mandatory ignored tests pass here (2 passed, 37.64s).
- **Cache identity (#6).** `trust_policy_digest` hashes `rust-proc-macros` (`rust_interop_cargo_inputs.rs:411`); asserted by `rust_interop_trust_tests.rs:103-127`.
- **Honest promotion (#7).** Excluding the parallel backend hunk: 36 rows, **20** supported / 12 bridge / 1 unsupported-by-design / 3 future-owned; kinds 13/4/10/9; 66 passing + 6 planned; 33 stable claims — all exactly the contract inventory. Only `proc_macro_trust` is promoted in the matrix, claims, and docs table. `internal_docs/sifr_sysroot_and_stdlib_architecture.md:153-161` was corrected to 20/12/1/3.
- **Gates (#8, #9).** `cargo test -p sifr_driver` → 430 passed / 61 ignored. `check_fixture_matrix.py` → 36/10/44/60/18, self-test 183. Tier, stable-claims (33), stale-draft self-tests pass. `check_compatibility_matrix.py`'s only failure is the excluded backend hunk (the script aggregates all failures and printed exactly one). `cargo fmt --check`, `git diff --check`, HIR guardrail, file-size guardrail (2982 files, limit 900), driver maintainability guardrail, and `cargo clippy -p sifr_driver --lib -D warnings` all clean. No new `unsafe`, panic, or fallback path.

## Findings

### 1. MEDIUM — the negative evidence fixture is not a valid program independent of the trust gate
`verification/areas/rust_interop/fixtures/proc_macro_trust/negative/untrusted_proc_macro_rejected_pre_execution.sifr:8` declares `def decode_without_proc_macro_trust(input: bytes) -> bytes`, but the bridge it targets returns `String` (`examples/proc_macro_trust_package/src/bridges/generated.rs:4`). The checked-in negative evidence therefore fails for a reason unrelated to trust. Reproduced against the checked-in scenario package with **trust fully intact**, using a freshly built compiler:

```
error[SIFR-RUST-TYPE-0001]: Rust bridge probe failed for `main.decode_without_proc_macro_trust`
  = note: expected fn pointer `for<'a> fn(&'a _) -> Vec<u8>`
                found fn item `for<'a> fn(&'a _) -> String {decode}`
```

With the single-token correction to `-> str`, the same fixture behaves as the evidence claims: `no errors found` with trust intact, and exactly `SIFR-RUST-TRUST-0001` naming `serde_derive` once proc-macro trust is removed. So the fix is a one-line change and the property being certified genuinely holds — but as checked in, the artifact carrying `# evidence-status: passing` / `# expected-diagnostic: SIFR-RUST-TRUST-0001` is over-determined and misrepresents why it is rejected. This also breaks the fixture family's own convention: the positive fixture (`positive/trusted_proc_macro.sifr:13`), both package examples, and certification_9's negative fixture all declare signatures that match their bridges. Acceptance #8 requires validators to "reject material drift"; nothing in `_scenario_proc_macro.py` or `check_fixture_matrix.py` checks fixture-level `positive/`/`negative/` sources against the scenario bridge, which is exactly why this passed all gates.

### 2. LOW — trust diagnostics can now be attributed to an unrelated declaration
`trust_validation.rs:22-24,40` picks the *first* declaration per package as the "representative" and stamps every backend trust diagnostic with that declaration's canonical target path and span. Because the pre-pass runs before `resolve_declaration` and `rust_interop.rs:272` then bails on the non-empty diagnostic list, the per-target emission at `rust_interop.rs:404-410` never fires. So for a package with `[bridge.foo, native.hash]` where `native` has a build script, the diagnostic now points at `main.foo` instead of `main.hash` — previously it pointed at the declaration that actually used the dependency. The retained test only passes its `contains("app.hash")` assertion (`rust_interop_trust_tests.rs:28`) because its package has a single declaration.

### 3. LOW — "kind-specific" holds only in the structured payload, not in what the user reads
`require_trust` (`rust_interop.rs:689-701`) interpolates only `{target}`; the kind-bearing `evidence` string lives in `args` and the note is the generic "add `X` to the matching `[trust]` Rust interop allow-list". Verified rendered output for the two removals differs only by the entry name:

```
= note: add `serde_derive` to the matching `[trust]` Rust interop allow-list before Cargo executes this dependency
= note: add `prost_build`  to the matching `[trust]` Rust interop allow-list before Cargo executes this dependency
```

Neither tells the user whether to edit `rust-proc-macros` or `rust-build-scripts`. The milestone's proof asserts kind-specificity against `format!("{diagnostics:#?}")` (`rust_interop_trust_tests.rs:49-55`, `package_rust_interop_proc_macro_support.rs:160,168-173`) — i.e. the debug rendering, not the user-facing text. It is genuinely kind-specific in serialized `args`, so this is a rendering gap rather than a missing property; the rendering code is unchanged from `origin/main`.

### 4. LOW — upstream `serde_derive` is compile-only, and the marker text overstates that
`rust/serde_derive/Cargo.toml:11` declares `serde_derive_upstream`, but no source in the tree references it (grep-confirmed) — unavoidable, since one proc-macro crate cannot invoke another's derive, and the contract does say "wrapper derive macro". However the evidence string `serde_derive=1.0.228;macro=executed` (`rust/serde_derive/src/lib.rs:11,21`) is a hardcoded literal that reads, in the runtime output and the compatibility-matrix notes, as proof that upstream `serde_derive` 1.0.228's macro executed. What executed is the wrapper's own `SifrGenerated`; upstream 1.0.228 is only compiled. The pin is defended by the root-lock subset check and the two-marker count assertion (`_scenario_proc_macro.py:358-365`), so drift is caught — but the marker wording should not claim upstream macro execution.

### 5. LOW — direct-root proc-macro rejection lost its unit test
`package_rust_interop_rejects_untrusted_proc_macro` (direct `native.hash` root, deleted from `rust_interop_tests.rs`) was *replaced* by, not supplemented with, `..._for_local_bridge` (`rust_interop_trust_tests.rs:32-56`). The direct-root proc-macro case now has no unit coverage; the build-script test is the only direct-root trust case left.

### 6. LOW — tracking artifacts
- `plans/reviews/active/rust-interop-certification-10-review-round-1.md` and `.claude.log` are **0 bytes** and untracked, and no review round is linked from the certification_10 section. Certifications 6 and 7 link every round (e.g. issue lines 709, 826).
- Issue line 1162: "the five focused package-wide trust tests **and** proc-macro trust cache-identity test" double-counts — `rust_interop_trust_tests.rs` has 5 tests total (4 trust + 1 cache identity).
- Issue line 1138 remains unchecked, which is correct for the pre-merge state; note the extraction it demands is done for the Python side (`_scenario_checks.py` 891 → 745, new `_scenario_registry.py`/`_scenario_proc_macro.py`), while `crates/sifr_driver/src/build/rust_interop.rs` grew 898 → 899 and now sits one line under the cap.

Finding 1 is the one that materially undermines the milestone's negative-evidence claim and should be fixed before merge; 2–6 are small but actionable.

VERDICT: NOT SATISFIED

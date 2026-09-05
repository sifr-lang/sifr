# Rust-Interop `certification_9` (`native_build_script`) — Review

Scope: current worktree (uncommitted branch `agent/rust-interop-certification-9`). Out-of-scope parallel-agent changes (`editor_integrations`, leetcode corpus, `.cert5probe/`, `.agent/`, stray webp files, `plans/phases/43_interoperability.md`, and the `ecosystem_backend_certification` hunk in the compatibility matrix) were excluded from judgement.

## Independent verification performed

- **Lockfile binding**: all 68 packages in `examples/native_trust_package/Cargo.lock` resolve to versions present in the root `Cargo.lock`; `cc 1.2.63`, `bindgen 0.72.1`, `cxx 1.0.198`, `zstd 0.13.3` are exact-pinned and unique in the root graph.
- **Determinism claim reproduced**: two fresh `cargo build --workspace --release --locked --offline --frozen` runs into separate target dirs produced byte-identical copies of all five evidence artifacts, exactly one copy each, with contents matching the literals asserted in `package_rust_interop_native_build_support.rs:40-63`. Wall time 9s and 14s — no budget risk.
- **Sentinel arming is effective**: applying the test's exact mutation and building with trust present created `rust/zstd/UNTRUSTED_BUILD_SCRIPT_EXECUTED`. The negative test's premise is factually sound.
- **Pre-Cargo ordering confirmed in code**: trust is recorded in `resolve_declaration` → `validate_backend_trust` (`crates/sifr_driver/src/build/rust_interop.rs:482-532`) and diagnostics abort before `execute_pending_direct_probes()` (`rust_interop.rs:176-186`), so `SIFR-RUST-TRUST-0001` genuinely precedes any Cargo dependency execution.
- **Observed native-link envelope on this host** (macOS, `--message-format=json`): `['c++', 'cxxbridge1', 'link-cplusplus', 'sifr_cc_probe', 'zstd']`. `sifr_zstd_probe` and `stdc++` are metadata/portability entries never emitted here — the scenario README and internal docs describe this correctly.
- **Area run reproduced**: `variants=10, blocking_failures=1`; the sole failure is `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence`, i.e. the explicitly out-of-scope hunk. `fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=18`, matrix self-test `cases=165`, claims `32`, all other suites pass.
- **Safety**: no `unsafe` (now enforced for this fixture via `_scenario_checks.py:452-457`), no `unwrap`/`expect`/panic path in build scripts or bridge code, all build-script output confined to `OUT_DIR` and enforced by `_scenario_native_build.py:370-375`.

## Findings

### MEDIUM — 1. The post-build native-link allowlist has no fixture-bound rejection evidence, but the promoted row claims it

`rust_interop_compatibility_matrix.json:431` asserts the row "validates the exact portable native-link envelope after Cargo"; `docs/rust-interop.mdx:202-203` says "post-build evidence outside it is rejected"; `internal_docs/rust_interop_architecture.md:974-976` says "Post-build Cargo messages remain fail-closed". The plan bullet is explicit: "prove the post-build allowlist **accepts only that envelope**".

Nothing in this row's evidence proves the "only" half, or even that link evidence is observed at all:

- `cargo_build_artifacts` (`package_rust_interop_native_build_support.rs:175-206`) discards `output.stdout` and never runs with `--message-format=json`, so no `build-script-executed`/`linked_libs` data is ever inspected.
- The positive test (`:71-86`) asserts only that the generated package checks, builds, and prints — acceptance is implicit. If `should_validate_native_link_evidence` regressed to `false`, or `trusted_native_links` returned everything, or Cargo stopped emitting link evidence, this test stays green.
- The negative test (`:96-113`) covers only the **pre-Cargo direct** path (`backend.links` / `has_build_script`). Transitive names (`zstd` from `zstd-sys`, `cxxbridge1`, `link-cplusplus`, `c++`) are *only* reachable through the post-build check and are untested here.

The repository already has the exact pattern to copy: `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:692-737` removes a transitive link from `sifr.toml` and asserts `build_cached_package_project` fails with `RUST_TRUST_MISSING` + `"untrusted native link evidence"`. Add an analogous sub-case here dropping a transitive-only name (e.g. `"zstd"` or `"cxxbridge1"` — verified above to be emitted, and verified not to trip the direct-`links` pre-Cargo check). That single case converts the row's headline claim from prose to evidence and simultaneously proves the emitted envelope is non-empty and actually inspected.

### MEDIUM — 2. The zstd runtime observation is invariant to the encoder

`positive/trusted_build_script_native_evidence.sifr:29` and `examples/.../src/main.sifr:27` observe compression as `assert len(compressed) > 0`, surfaced as the literal `compressed=nonempty` in the asserted stdout (`package_rust_interop_native_build_support.rs:80`). The pre-change stub (`input.iter().copied().rev()`) satisfies this identically, as would any non-empty transform. The only thing pinning real zstd is the source-token check `zstd_upstream::stream::encode_all` (`_scenario_native_build.py:354`) — a source-shape assertion, which the plan's promotion rules treat as insufficient on its own.

Fix cheaply: assert the zstd frame magic (`0x28 0xB5 0x2F 0xFD`) and/or a decode round-trip back to `b"sifr-rust-interop"`, or an exact compressed length, and surface that in the observed stdout string.

### LOW — 3. Version provenance in artifacts is self-asserted, with no single source of truth

`cc=1.2.63`, `bindgen=0.72.1`, `cxx=1.0.198`, `zstd=0.13.3` are hand-written string literals in the four `build.rs` files, and the expectations in `package_rust_interop_native_build_support.rs:40-63` are equally hand-written. The pins live independently in `Cargo.toml` and `_scenario_native_build.py:93-110`. I confirmed all four currently agree, so this is a drift-hardening gap rather than a present inaccuracy: a future pin bump that updates the manifest and the validator but not the literals would keep every gate green while the artifacts report the old versions. Have `_scenario_native_build.py` assert the evidence literals carry the same pinned versions it already enforces.

### LOW — 4. The build-script negative case does not pin the trust *kind*

`package_rust_interop_native_build_support.rs:163-167` asserts `rendered.contains(required_trust)`, with `required_trust = "zstd"` for the `build_script` case. Both possible diagnostics contain `zstd` (`"build script in Cargo dependency \`zstd\`"` and `"native links \`sifr_zstd_probe\` declared by Cargo dependency \`zstd\`"`), so the assertion cannot distinguish which permission was reported. `errors.len() == 1` limits the blast radius, but for evidence-grade coverage assert the kind-specific evidence text per case.

### LOW — 5. `rust/cc/native/probe.c` is required by the build but unvalidated (and currently untracked)

`_validate_build_sources` (`_scenario_native_build.py:333-375`) pins `.compile("sifr_cc_probe")` but never checks that `rust/cc/native/probe.c` exists or defines `sifr_cc_probe`. The file is still `??` in `git status`. If it is omitted from the commit, every blocking fast gate (area check, self-tests, Clippy, guardrails) stays green and only the `#[ignore]`d merge-profile test fails. Add an existence + symbol token check, and confirm the file is staged.

### LOW — 6. New undeclared host-toolchain requirement

`bindgen 0.72.1` with default features loads `libclang` dynamically at build-script execution time; `cc`, `cxx`, and `zstd-sys` need a working C/C++ toolchain. This is the first place in the repo where bindgen actually *executes* (`sifr_rust_interop_catalog` only compiles it). Nothing in the fixture README, `docs/rust-interop.mdx:195-205`, or `.github/workflows/local-first-validation.yml` (bare `ubuntu-24.04`, no toolchain install step) documents the prerequisite, and the failure mode is an opaque build-script error inside an ignored test. Document the prerequisite alongside the scenario.

### LOW — 7. The public claim is not platform-scoped

`docs/rust-interop.mdx:241` promotes `native_build_script` as `supported` with no target scoping, while the envelope is Apple/GNU-specific (`c++`/`stdc++`; `link-cplusplus` emits nothing on MSVC). The adjacent `advanced_data_runtime_matrix` prose sets the precedent by scoping to "arm64 and x86_64". Scope this claim the same way.

### LOW — 8. Plan bookkeeping is incomplete for the work that is actually done

`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1022-1049` leaves all six `certification_9` checklist items unchecked while line 155 reports "in progress" and the implementation is complete. There is also no `certification_9` "Review and validation notes" section and no "Expected post-item inventory" block (the `certification_8` block at :996 is the established convention; post-`certification_9` values are 36/36/36 rows, 64 passing / 8 planned directions, 19 `supported`, 4 `future-owned-by-separate-phase`, 32 structured claims — the latter two match the sysroot doc update and the observed `claims=32`). `plans/reviews/active/rust-interop-certification-9-review-round-1.md` exists as a 0-byte file and must not be committed empty.

### LOW — 9. `# fixture-trust:` header mirrors only two of three trust lists

`positive/trusted_build_script_native_evidence.sifr:6-7` mirrors `rust-build-scripts` and `native-links` but omits the newly required `rust-no-panic` list from `sifr.toml`. No checker validates these headers against the scenario manifest (`grep fixture-trust` over `verification/**/*.py` returns nothing), so they are unenforced prose in three fixtures.

### LOW — 10. `_scenario_checks.py` is at 891/900 lines

Extracting the native-build validator into `_scenario_native_build.py` was the right decomposition, but the dispatcher still grew from 864 to 891 lines, leaving 9 lines of headroom on a file that every subsequent certification row touches (`certification_10`–`13` remain). The next row should extract the `REQUIRED_SCENARIO_EXAMPLES` / per-fixture dispatch table rather than add to it.

### LOW — 11. No in-test control for the sentinel arming

The negative test proves the mutation was applied (`assert_ne!` at `:144-147`) but never proves the armed script would write the sentinel if Cargo ran — the armed `build.rs` is never compiled. I verified externally that it does write, so the evidence is sound today; the assertion is nevertheless one refactor of `zstd/build.rs` away from becoming vacuous. A one-off control (build once with trust present, observe the sentinel, then rerun untrusted) would close it.

## Non-findings checked and cleared

Hermeticity (explicit `--target-dir`, `--locked --offline --frozen`, `OUT_DIR`-only artifacts enforced by validator + mutation case, in-memory bindgen header, vendored zstd C sources, no network); trust identity correctness (renaming wrapper `links` to `sifr_cc_probe`/`sifr_zstd_probe` is what makes the direct pre-Cargo check meaningful, and the mutation suite pins it); `unsafe-rust-bridges` removal is correct now that the bridge contains no `unsafe`; `rust-no-panic` declarations are honest for all five targets; `assert` in fixture Sifr sources is the repository's sanctioned assertion-first e2e style; `errors.len() == 1` in both negative cases is achievable and correct; provenance records validate against the weakest blocking profile and both `#[ignore]`d tests exist with matching names; mutation coverage (12 cases) hits pins, package identity, `build.rs` declaration, `links` identity, all three trust lists, each crate's execution token, and OUT_DIR containment; `target/` exclusion in `_scenario_files` and `copy_fixture_tree` prevents the stale in-fixture build dir from contaminating checks or copies; the sysroot-doc count update (19 supported / 4 future-owned) is correct for `certification_9` alone.

## Verdict

NOT SATISFIED

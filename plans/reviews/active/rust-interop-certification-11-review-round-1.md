# Verdict: **NOT SATISFIED**

Two hard gate failures and one evidence-binding defect must be fixed before PR. The core architecture is sound and follows the design review closely; the failures are in the mandatory lint gate, in the stability of the certified diagnostic, and in what the negative test actually asserts.

---

## Blockers

### B1 — `cargo clippy --workspace -- -D warnings` fails on new cert-11 code
**Severity: blocker.** Files: `crates/sifr_driver/src/build/cargo_resolution.rs:313`, `crates/sifr_driver/src/build/rust_interop_probe_diagnostics.rs:107`

```
error: very complex type used. Consider factoring parts into `type` definitions
   --> crates/sifr_driver/src/build/cargo_resolution.rs:313:6
    | ) -> Result<BTreeSet<(String, String, String, String)>, Vec<RenderedDiagnostic>> {
    = note: `-D clippy::type-complexity` implied by `-D warnings`

error: this argument is passed by value, but not consumed in the function body
   --> crates/sifr_driver/src/build/rust_interop_probe_diagnostics.rs:107:18
    | diagnostics: Vec<RenderedDiagnostic>,
    = note: `-D clippy::needless-pass-by-value` implied by `-D warnings`
error: could not compile `sifr_driver` (lib) due to 2 previous errors
```

This is the exact command CI runs (`.github/workflows/local-join…/local-first-validation.yml:29`) and AGENTS.md lists it as a required lint. Both sites are new in this change. The issue doc's validation paragraph lists lock-argument, normalization, classification, source-selection, formatting, stable-claim and file-size guards as passing and **omits Clippy** — that omission is the gap.

Remediation: introduce a named type (e.g. `type RegistryIdentity = (String, String, String, String)` or a small struct) for the registry-entry tuple, and take `&[RenderedDiagnostic]` in `probe_resolution_diagnostics`. Then re-run `cargo clippy --workspace -- -D warnings`. Also fix `crates/sifr_driver/src/build/rust_interop_probe.rs:894`, where `use super::cargo_resolution::{prepare_cargo_resolution, CargoResolutionPolicy};` sits **after** the `#[cfg(test)] mod tests` block — that trips `clippy::items_after_test_module` under `--all-targets` and is a plain style defect regardless.

### B2 — the certified `SIFR-RUST-CARGO-0001` classification depends on a lexical scan of the entry file only
**Severity: blocker.** Files: `crates/sifr/src/cli_lock_modes.rs:59-66`, `crates/sifr/src/check_and_package_commands.rs:352-370` and `:400-417`

`entrypoint_declares_rust_interop` decides whether a package-graph Cargo failure is reclassified as `SIFR-RUST-CARGO-0001` by reading the entry file and matching lines that `starts_with("@rust(")` or `("@rust.")`. Reproduced against the fixture scenario itself with the freshly built binary:

*Control* — decorator in `src/main.sifr`, lock removed, `sifr check src/main.sifr --frozen`:
```
error[SIFR-RUST-CARGO-0001]: Rust interop Cargo resolution failed: missing lockfile
```
*Same package, same drift*, decorator moved to `src/bridge.sifr` and imported from `src/main.sifr`:
```
error[SIFR-PACKAGE-0101]: cargo metadata failed: error: cannot create the lock file
  /private/tmp/c11exp2/Cargo.lock because --frozen was passed to prevent this
```

So the promoted `supported` row's stable-diagnostic guarantee holds only when the `@rust` decorator happens to live in the entry module. This directly contradicts the contract clause ("reject … with stable `SIFR-RUST-CARGO-0001` diagnostics") and the matrix note. It also re-introduces exactly the lexical-context pattern that `hardening_4` ("replace lexical rejection context") removed.

Remediation: drive the reclassification from resolved interop state, not source text — e.g. from the package graph / manifest's declared Rust interop (`[rust] direct-crate-bindings`, `[trust]` targets, or the resolved `RustInteropModuleSource` set), which is already available at this boundary. Then add a negative case whose `@rust` declaration lives outside the entry module.

### B3 — the negative test never asserts the diagnostic emitted by the path under test
**Severity: blocker (evidence binding).** File: `crates/sifr/src/cargo_lock_mode_certification_tests.rs:205-279`

`assert_drift_rejected` asserts only `cmd_check(...) == EXIT_USER_DIAGNOSTIC` — i.e. *some* user diagnostic. It then independently spawns its own `cargo metadata`, and calls `crate::cli_lock_modes::rust_interop_cargo_failure_diagnostic(...)` **directly** on that stderr, asserting `diagnostic.code == RUST_CARGO_METADATA` on that fabricated value (lines 247-273). The compatibility-matrix provenance rule "bound test asserts declared diagnostic SIFR-RUST-CARGO-0001" is satisfied by that direct helper call, not by observed CLI output.

The current behaviour *is* correct — I ran the test with `--nocapture` and the real `cmd_check` output is:
```
E SIFR-RUST-CARGO-0001 … missing lockfile
E SIFR-RUST-CARGO-0001 … stale lockfile or feature/source drift
E SIFR-RUST-CARGO-0001 … checksum drift
E SIFR-RUST-CARGO-0001 … stale lockfile or feature/source drift
E SIFR-RUST-CARGO-0001 … stale lockfile or feature/source drift
```
— but nothing in the test would fail if the code changed to `SIFR-PACKAGE-0101`. B2 is precisely that failure mode, present today and undetected.

Remediation: assert on diagnostics actually produced by the command (capture `DiagnosticFormat::Json`/`Compact` output, or route through an API that returns `Vec<RenderedDiagnostic>`), asserting code **and** a case-specific reason. Delete the self-invented cargo-metadata + direct-classifier block; it is not evidence of the compiler's behaviour.

---

## Non-blocking findings (should fix; none of these alone would block merge)

### N1 — the positive test proves nothing about mode propagation
`crates/sifr/src/cargo_lock_mode_certification_tests.rs:131-172` loops the three modes over `cmd_check`/`cmd_build`/`cmd_run` and asserts only `EXIT_SUCCESS`, with messages that read "sifr build must preserve {mode} mode". Success under mode X is not evidence the flag reached any subprocess; if `lock_mode` were dropped in `materialize.rs` or the probe, every assertion would still pass. The contract checklist wording is "proving every package/Cargo subprocess preserves the requested resolution mode", and design review §3.6 called for a `#[doc(hidden)]` cargo-argv sink asserting *every* recorded invocation. That sink was not built.

Code inspection does confirm propagation (`materialize.rs:270-273`, `rust_interop_probe.rs:129-137`), and I confirmed end-to-end that `sifr build --frozen` on the scenario produces a prepared `out/sifr_output/Cargo.lock` with only authoritative registry entries. But the *evidence* is inspection, not the test. Add the argv sink.

### N2 — scenario policy token asserts a code shape that does not exist in the code
`verification/areas/rust_interop/checks/_scenario_cargo_locked.py:15` requires the token `IndexMap::<String, u32>::new()`. The token check (`_scenario_checks.py:345-351`) searches README + `sifr.toml` + `.sifr` + `Cargo.toml` + `.rs` concatenated, and the token is present **only** in `examples/locked_offline_cache/README.md:7` prose. The actual wrapper (`rust/locked_bridge/src/lib.rs:1-10`) builds `IndexMap<usize, u8>` via `.collect()` and never calls `IndexMap::<String, u32>::new()`. So a policy token that is supposed to pin the real crate usage is pinned to a factually wrong README sentence. Same category: the `"CARGO_NET_OFFLINE=true"`, `"cache miss"`, `"cache hit"` tokens and their two mutation cases ("network denial evidence drift", "cache evidence drift") test README prose only.

Also inaccurate: fixture `README.md` and scenario `README.md:19` claim every mutation runs "with `CARGO_NET_OFFLINE=true`". Only the test's secondary `cargo metadata` subprocess sets it (`cargo_lock_mode_certification_tests.rs:251`); `cmd_check` does not — network denial there comes from `--frozen` alone.

Remediation: change the token to text that actually appears in `lib.rs`, and correct both READMEs.

### N3 — authority union lets the sysroot lock override the package lock's pins
`crates/sifr_driver/src/build/entrypoint_resolution.rs:15-44` collects both the nearest package `Cargo.lock` and the sysroot lock as authorities. `cargo_resolution.rs:177-185` seeds the generated resolution from whichever authority is **largest by file size**, and `validate_authoritative_registry_entries` (`:250-272`) accepts any registry entry found in *either*. If a user package pins `X = 1.0` and the sysroot lock carries `X = 2.0`, the seeded/pruned resolution can legitimately land on 2.0 and pass validation with no drift diagnostic. Design review §3.3 stated the intent as "the user's lock stays the sole authority for versions." Recommend preferring the package lock as seed and, where a name appears in both, requiring the package lock's version.

### N4 — drift reasons are not kind-specific
`cargo_lock_failure_reason` (`crates/sifr_package/src/cargo/lock_modes.rs:47-71`) collapses stale-version, source and feature drift into one string. Observed above: 3 of the 5 negative cases all report `stale lockfile or feature/source drift`; the `failed to select a version for the requirement` branch is unreachable for the `StaleVersion` case because Cargo emits `cannot update the lock file` first (that branch is only covered by the synthetic unit test at `lock_modes.rs:142-145`). Design §3.4 required "a distinct `{reason}` per drift kind", and no test asserts any reason. Either order the checks so version drift is distinguishable, or drop the distinct-reason claim from the design record.

### N5 — `run`/`check` still silently downgrade constrained modes in manifestless mode
`cmd_build` now correctly rejects (verified: `sifr build hello.sifr --frozen` → `SIFR-RUST-CARGO-0001: sifr build --frozen requires a package Cargo.lock`). But `cmd_run_with_session` (`diagnostic_rendering_and_run.rs:288-303`) falls through to `cmd_run_file(...)`, and `cmd_check` (`check_and_package_commands.rs:74-79`) to `cmd_check_file(...)`, both of which drop `lock_mode` entirely with no diagnostic. Design §5.1 asked for one documented choice; `docs/cli/build-run.mdx` documents only the build behaviour. This is pre-existing for `run`/`check`, but is now inconsistent with `build` and sits under a "no silent resolver fallback" acceptance clause.

### N6 — maintainability headroom exhausted; design §4 splits mostly not done
Design §4 required splitting six files. Only `cli_lock_modes.rs` (66), `cargo_resolution.rs` (376), `entrypoint_resolution.rs`, `rust_interop_probe_nonce.rs`, `rust_interop_probe_paths.rs` were extracted. Current sizes: `check_and_package_commands.rs` **900** (up from 848), `package_rust_interop_build_tests.rs` **900** (held only by deleting the blank line after `use super::*;`), `entrypoint.rs` 899, `diagnostic_rendering_and_run.rs` 898, `rust_interop_probe.rs` 894, `cli_model_and_entrypoint.rs` 888. The guardrail passes, but two files are at the cap and four within six lines of it — the next touch on any of them forces a split. Per AGENTS.md, split by responsibility now rather than in the follow-up item.

### N7 — prepared-lock store grows without bound
`prepared_lock_path` (`cargo_resolution.rs:216-248`) keys on the generated `Cargo.toml` *content*, which embeds absolute path-dependency paths. Because tests and package builds live under fresh temp roots, keys never repeat across invocations: I observed **101** entries under `$TMPDIR/sifr_generated_artifact_cache/cargo_resolution` with no eviction. Correctness is unaffected (entries are tiny, and both cold and warm paths re-run `validate_authoritative_registry_entries`), and the atomic `cache_prepared_lock` publish via temp + rename is correct. Worth a bounded/GC'd store or a note.

### N8 — `Offline` is treated as lock-immutable
`PreparedCargoResolution::assert_unchanged` rejects any lock change for every non-`Normal` mode, including `Offline`, although `CargoLockMode::is_lock_mutation_disallowed()` returns false for `Offline`. This is stricter than Cargo and is deliberately documented (`docs/package_management.md`, `internal_docs/rust_interop_architecture.md`), so it is fine — but it makes the enum's own predicate misleading at this call site. A one-line comment or a distinct predicate would prevent a future reader "fixing" it.

---

## What I verified as correct

- **CLI parsing / normalization.** `locked && offline ⇒ Frozen` is fixed and shared by all eight commands that carry the flags (`cli_lock_modes.rs:1-15`, `cli_model_and_entrypoint.rs`), with a parse-level test covering all five combinations.
- **Threading.** `PackageEntrypoint.lock_mode` → `package_cargo_resolution_policy` → `RootedEntrypointPlan.cargo_resolution` → probe `PendingRustBridgeProbe.cargo_resolution` and `materialize_*` → final `run_cargo_build` argv (`materialize.rs:270-273`). All four `PackageEntrypoint` construction sites updated (CLI check/run, python_cli, driver tests, diagnostic harness).
- **Prepared-lock mechanism.** Seed → `cargo metadata` (with `--offline` when the mode denies network) → exact `(name, version, source, checksum)` validation against authoritative locks or trusted vendor dirs with matching manifest identity **and** `.cargo-checksum.json` → digest → post-subprocess `assert_unchanged`. Validation runs on both cold and warm paths. Cache publish is atomic (temp + rename, with a benign lost-race branch). Path packages carry no `source` and are correctly excluded.
- **Classification ordering.** `cargo_lock_failure_reason` is the **first** branch of `classify_probe_failure` (`rust_interop_probe_diagnostics.rs:16-24`), ahead of the resolve/type heuristics — the design's mandatory correction is implemented.
- **Fail-closed probe skip.** The `cargo_manifest_path` missing case now errors instead of returning `Ok(())` under constrained modes (`rust_interop_probe.rs:52-61`).
- **Cache isolation.** Probe key distinguishes `normal` from `lock-constrained` (`rust_interop_probe_cache.rs:57-62`); all three constrained modes share one bucket, which is sound because every constrained probe additionally runs at `--frozen` strength (`rust_interop_probe.rs:132-137`) — and this is explicitly documented rather than hidden. Artifact cache key correctly omits the mode so the cold→warm identity requirement is achievable.
- **Package-owned vs sysroot-only source selection.** `uses_sysroot_vendor()` now gates the vendor `--config` replacement so probes match the final build's `CargoVendorMode`. Verified this is the reason the pre-change binary failed the fixture (`vendor` replacing crates-io, `hashbrown 0.17.1` unvendored) and the rebuilt binary succeeds.
- **Fixture and data.** Real exact-pinned `indexmap = { version = "=2.14.0", default-features = false }` genuinely compiled and called; scenario lock is a strict root-lock subset (now enforced on source **and** checksum, `_scenario_lock_checks.py:41-67`); `Cargo.lock` existence/parse still enforced generically at `_scenario_checks.py:405-422`.
- **Gates I ran (read-only).** `check_fixture_matrix.py` → `fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18`; self-test `cases=192`; `check_stable_support_claims.py` → `claims=34`; `check_tiers.py` → `tiers=5 fixtures=36`; `check_stale_drafts.py` ok; `cargo fmt --check` clean; `check_file_size_guardrails.py` PASS; hir/driver/dependency-direction/submodule/sysroot-stdlib guardrails PASS; `check_docs_error_code_links.py` PASS.
- **Both mandatory tests pass at current code.** Negative 0.77 s; positive 85.67 s (reported 66.31 s — machine variance, both fine), including cold miss → warm hit with identical binary path and byte-identical authoritative lock.
- **Counts are exact.** On the backend-reverted baseline I computed 68 passing / 4 planned, 21 `supported`, 12 `supported-through-bridge`, 1 `unsupported-by-design`, 2 `future-owned` — matching the issue doc verbatim.
- **Compatibility checker isolation confirmed.** In an exported tree with only the `ecosystem_backend_certification` hunk reverted to HEAD: `rust interop compatibility matrix ok: rows=36 fixture_rows=36 categories=4` (exit 0). With the hunk present, the single failure is `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence`. So cert-11's own provenance, evidence binding and doc-claim wiring are clean.

## Shared-worktree / commit hygiene

- **Must exclude** the `ecosystem_backend_certification` hunk in `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json` (lines ~396-400) from this commit — it promotes a row whose two evidence directions are still `planned`, and it is the sole reason the live checker fails. Preserve it in the worktree.
- **Must include** `plans/reviews/active/rust-interop-certification-10-review-round-4.md` and `-round-5.md`: both are untracked, and the issue-doc hunk added here links them. Without them the merged doc has dead links.
- `plans/reviews/active/rust-interop-certification-11-review-round-1.md` is currently a 0-byte placeholder — populate it with this review before opening the PR.
- **Exclude, no interference observed:** `editor_integrations` submodule pointer, untracked `verification/areas/algorithmic_compatibility/corpora/leetcode`, `.cert5probe/` (cert-5 scratch `.sifr` files), `.claude/`, `"logo 06.48.53.webp"` / `"docs/logo/logo.webp 08-03-09-514.webp"`, `plans/phases/43_interoperability.md`. None touch cert-11 code, data, fixtures, or docs.
- The deletion of `fixtures/cargo_locked_offline/.gitkeep` is correct and in scope (the directory now holds real files).

## Minimum path to SATISFIED

1. Fix B1 (both Clippy errors + the trailing `use` after `mod tests`); re-run `cargo clippy --workspace -- -D warnings`.
2. Fix B2 — classify from resolved interop state, not an entry-file text scan; add a negative case with the decorator in a non-entry module.
3. Fix B3 — assert the observed `cmd_check` diagnostic code (and a per-case reason) instead of the directly-invoked classifier; delete the synthetic `cargo metadata` block.
4. Then N1 (argv sink) and N2 (token + README accuracy) — both are cheap and both are load-bearing for the honesty of the promoted `supported` claim.
5. Re-run the two mandatory tests, the five rust-interop checkers, fmt, Clippy, and the file-size guardrail; then the create-PR profile.

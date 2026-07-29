## Independent milestone review — `certification_8` (crate-backed advanced data runtime)

Reviewed the full uncommitted tree against `origin/main`, including the untracked `advanced_data_runtime` scenario (11 new source/manifest files), the new driver test module, and the new scenario-policy checker.

### What holds up

- **Counts and promotion are exact.** Computed from the data files: 36 compatibility rows / 36 fixture rows, categories `18 supported / 12 supported-through-bridge / 1 unsupported-by-design / 5 future-owned`, execution kinds `13/4/10/9`, `62 passing / 10 planned`, 60 package examples, 18 scenario examples, 31 stable claims, `runtime_deferrals: []`. Every number matches the plan's expected post-item inventory (`plans/issues/.../rust-interop-runtime-ecosystem-certification.md:934-945`). Only `advanced_data_runtime_matrix` was promoted; `arrow_record_batch`, `tensor_dlpack_bridge`, `advanced_data_matrix` remain `contract-only` in both the matrix and `docs/rust-interop.mdx:222-225`.
- **Pins are exact and offline-safe.** Scenario workspace deps (`examples/advanced_data_runtime/Cargo.toml:9-13`) match `crates/sifr_rust_interop_catalog/Cargo.toml` byte-for-byte, including `candle = { package = "candle-core", "=0.11.0", default-features = false }`. Every one of the 519 external `(name, version)` pairs in the scenario `Cargo.lock` is present verbatim in the root `Cargo.lock` — the scenario lock is a strict subset of the root graph, so nothing needs fetching.
- **No-copy evidence is genuine, not tautological.** Arrow (`record_batch.rs:89-92`), ndarray and Candle (`tensor.rs:102-112`), and the DLPack transfer (`dlpack.rs:61`, checked at `dlpack.rs:33`) all capture the owned `Vec`'s address before the move and compare it after construction, returning `Err` on mismatch. A copying implementation must keep the source alive while copying, so the copy's buffer cannot land at the freed source address — the check cannot false-pass. `layout=c` is soundly derived from `strides() == [3,1]` at rank 2.
- **No panic paths, safe Rust.** No `unwrap`/`expect`/`panic!`/indexing in the three bridge modules; `table_exist` is defused with `unwrap_or(false)` into a diagnostic mismatch (`record_batch.rs:139`). `reject_unsafe_rust` now covers this fixture (`_scenario_checks.py:441`) and is mutation-tested (`_scenario_advanced_data.py:287-305`).
- **The native-link change is not a trust bypass.** `record_declared_native_links` (`trust_validation.rs:11-36`) only *records* entries that already exist in the package manifest's `[trust].native-links`. Enforcement is untouched and still fail-closed: `validate_native_link_evidence` (`materialize.rs:329-360`) rejects any build-script `linked_libs` name absent from the set. Removing the `uses_bridge_root` gate widens trust from "manifest-declared *and* has a package-local bridge target" to "manifest-declared", which is precisely the checklist's intent — the manifest remains the sole authorization.
- All new/touched files are under the 900-line cap; the `assert`-in-fixture style matches the existing `zero_copy_runtime_matrix` precedent.

---

### Blocking findings

**B1 — `blake3_neon` makes the mandatory positive test aarch64-only; it will fail on the x86_64 merge lane.** (severity: high — regression)

`examples/advanced_data_runtime/sifr.toml:30` declares `native-links = ["blake3_neon", "lzma", "onig", "psm_s", "zstd"]`. I confirmed from the fixture's own build output that these five are exactly what this host emits:

```
blake3-…/output        => cargo:rustc-link-lib=static=blake3_neon
liblzma-sys-…/output   => cargo:rustc-link-lib=static=lzma
onig_sys-…/output      => cargo:rustc-link-lib=static=onig
psm-…/output           => cargo:rustc-link-lib=static=psm_s
zstd-sys-…/output      => cargo:rustc-link-lib=static=zstd
```

But `blake3_neon` is emitted only on little-endian aarch64 (`blake3-1.8.5/build.rs:367-371`). On x86_64 with a C compiler present, blake3 instead emits `blake3_sse2_sse41_avx2_assembly` (`build.rs:232`, reached via `build.rs:351`) and `blake3_avx512_assembly` (`build.rs:278`). Neither is in the allowlist, so `validate_native_link_evidence` returns `SIFR-RUST-TRUST-0001` and the build fails before the runtime observation ever runs.

`.github/workflows/local-first-validation.yml:32` runs the `merge` profile on `ubuntu-24.04` (x86_64), and `fixture.json` binds both directions to `profile: merge`. Because PRs run only `create-pr`, this breaks on push to `main`, not in the PR lane. The evidence recorded as `"status": "passing"` for the merge profile is therefore host-specific and not reproducible on the authoritative runner.

Compounding it, the arch-specific list is pinned in three places that all require exact equality — `sifr.toml:30`, `EXPECTED_NATIVE_LINKS` (`_scenario_advanced_data.py:50`, enforced by `_require_exact_trust` with `actual != expected`), and the scenario token at `_scenario_advanced_data.py:33` — so a portable superset cannot be declared without also relaxing the checker. This needs either an arch-conditional allowlist mechanism or a documented, enforced host constraint for this fixture; it cannot be left as-is with `passing` merge-profile evidence.

**B2 — The Polars evidence is not derived from the crossed data, but is presented as part of the exchange claim.** (severity: medium-high — evidence honesty)

`record_batch.rs:109-112` builds the Polars frame from a hardcoded literal:

```rust
let polars_values = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
let series = Series::new("value".into(), polars_values);
let polars = DataFrame::new(polars_values.len(), vec![series.into_column()])
```

Nothing from the Sifr-owned vector or the Arrow array reaches Polars. The "agreement" at `record_batch.rs:140-146` compares a hardcoded name/dtype against a hardcoded name/dtype, and `polars.height() != record_batch.num_rows()` holds only because both literals happen to be length 6. Yet the compatibility notes state the evidence "executes exact-pinned Arrow, DataFusion, **Polars**, ndarray, and CPU-only Candle bridges; observes schema, dtype, … identity" for a capability named *crate-backed … runtime exchange* with `polars` in `required_crates`, and `docs/rust-interop.mdx:182-184` says the batch is "checked against a Polars dataframe with the same field, dtype, and row count" — which a reader will take to mean the crossed data reached Polars.

I accept the underlying constraint: polars uses its own `polars-arrow`, so a true zero-copy arrow-rs handoff would require the C data interface and `unsafe`, which this fixture correctly forbids. The honest fix is cheap: derive the series from the Arrow buffer (`Series::new("value".into(), array.values().to_vec())`) so name/dtype/row agreement is actually observed on the crossed data — the claim never promises Polars is zero-copy. Absent that, the row notes, README, and public doc must state explicitly that the Polars observation is an independently constructed reference schema and that no Sifr→Polars or Arrow→Polars data crossing is claimed.

**B3 — "Consuming close releases exactly one owner" is not distinguished from release during transfer.** (severity: medium — evidence gap)

The checklist requires proving that *consuming close* releases the owner. `main.sifr:66-69` / `positive/…sifr` call `tensor_release_observation()` only *after* `capsule.close()`, observing `tensor-released=1;active=0`. That terminal state is equally consistent with the `OwnerGuard` having been dropped during `transfer_dlpack` — the very failure the one-shot-transfer claim rules out. The implementation does move the guard (`dlpack.rs:62-68`), and the `.take()` token is mutation-covered, but that is a textual guard on the source, not a runtime observation.

One extra observation between `transfer_dlpack` and `capsule.close()` asserting `tensor-released=0;active=1` would close this and make the attribution real. The same gap applies to Arrow: `arrow_release_observation()` is never sampled before `arrow.close()`.

---

### Non-blocking findings

**N1 — the declared DLPack `protocol=` type has no runtime correspondence.** `dlpack.rs:7` defines `pub struct Capsule;`, which is never constructed, referenced, or returned; the runtime type is `DlpackView`. Likewise `sifr_arrow_bridge::schema::RecordBatch` (`lib.rs:3-6`) is a never-used marker. The pre-existing validator only checks the crate-name prefix of these paths (`advanced_data_validation.rs:210-220`, `230-240`), so the negative direction's "schema-root" rejection proves a prefix check, not that the declared type exists or matches the bridge. On the "is *DLPack-style* overstated" question: the fixture README and internal doc hedge appropriately ("models the … DLPack ownership and metadata contract without exposing an unsafe C ABI"), but `docs/rust-interop.mdx:185-187` carries no such caveat while the runtime string advertises `protocol=managed-tensor` — and there is no `DLManagedTensor`, deleter, device_type/device_id, byte_offset, or capsule at all. I would add the same one-clause caveat to the public doc; with it, "DLPack-style" is defensible.

**N2 — "one-shot" is never exercised.** The `"ndarray owner was already transferred"` / `"tensor owner guard was already transferred"` branches (`dlpack.rs:60,64`) are unreachable because Sifr's `own` consumes the handle. That is the right design, but "one-shot" is then a compile-time property with no evidence direction, and those branches are uncovered.

**N3 — always-`Ok` `Result` state fields.** `RecordBatchView.state`, `TensorView.state`, and `DlpackView.state` are `Result<_, String>` but only ever constructed as `Ok` (creation fails before the handle exists). Every `Err(error) => …` arm and the `Ok(DlpackView { state: Ok(_) })` metadata-drift arm (`dlpack.rs:39-42`) is dead. This is a needless error-simulating wrapper of the kind AGENTS.md's "no fallback paths" rule discourages; collapsing to the plain state would remove ~20 lines of unreachable code from three modules.

**N4 — native-link trust is graph-global while recorded per target path.** `record_declared_native_links` is reached per declaration with that declaration's owning package (`rust_interop.rs:238,278`), but `trusted_native_links` (`materialize.rs:289-298`) flattens to a bare name set, discarding `canonical_target_path`. Widening the gate therefore also means a Sifr *dependency* that uses only direct-crate bindings now contributes its manifest's `native-links` to the whole build's allowlist. The manifest is still the authorization and the pattern pre-dates this change for bridge-root dependencies, so this is not a new bypass — but if per-target scoping was intended, the enforcement side needs the path too.

**N5 — scenario lock/root-lock agreement is unverified.** `_scenario_advanced_data.py` validates the `Cargo.toml` pins but nothing asserts that the checked-in scenario `Cargo.lock` exists or that its pins match the root lock. The subset property holds today; a future root-lock bump would silently diverge.

**N6 — cap proximity and import order.** `_scenario_checks.py` is 891 lines and `rust_interop.rs` is 898 — both pass the 900-line guardrail with almost no headroom; the next addition to either forces a refactor. Minor: `_scenario_advanced_data` is imported after `_scenario_async_reqwest` (`_scenario_checks.py:9-16`), breaking the file's otherwise alphabetical block.

**N7 — plan hygiene / unrelated tree state.** All seven `certification_8` checklist boxes remain `[ ]` and the inventory is still labelled "Expected" despite matching actuals; the Implementation Progress row is `in progress`. Separately, the working tree carries items unrelated to this milestone that must not enter the PR: the `editor_integrations` submodule bump (`d7577d49` → `a980835e`), the dirty `algorithmic_compatibility/corpora/leetcode` submodule, untracked `plans/phases/43_interoperability.md` (Phase 43, unrelated), `.cert5probe/`, `.claude/`, and two stray `.webp` files. (`.DS_Store` and `target/` inside the fixture are correctly gitignored.) I have not modified anything.

---

B1 is a hard blocker: the positive direction is recorded as merge-profile `passing` but cannot pass on the x86_64 runner the merge profile actually uses. B2 and B3 are honesty/attribution gaps between what the row and public docs claim and what the runtime actually observes.

NOT SATISFIED

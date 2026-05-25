

Reviewing the "Organization Demo Repositories" section (lines 543-708):

**Check 1: Misleading names**
- `sifr-demo-json` / `demo_json` ✓
- `sifr-demo-http` / `demo_http` ✓
- `sifr-demo-test-support` / `demo_test_support` ✓
- `sifr-demo-app` ✓

Names are non-misleading. User requirement satisfied.

**Check 2: Manifest consistency**

1. **sifr-demo-json**: Cargo manifest complete, sifr.toml complete with `edition = "2026"` and `modules = ["demo_json"]`. ✓

2. **sifr-demo-http**: Cargo manifest complete with Git dep on `sifr-demo-json`, sifr.toml complete with `trust.native = ["reqwest"]`. ✓

3. **Consumer sifr.toml** (lines 651-663): `exports.modules = ["app"]` but **no `__init__.sifr` file is shown** in the directory tree. Earlier in the doc (line 499) states `__init__.sifr` defines explicit package/subpackage re-exports. The consumer's structure should show `sifr/app/__init__.sifr` alongside `sifr/app/main.sifr` — the consumer is the most likely place to need this, and omitting it leaves a reader wondering whether it's required.

4. **`edition = "2024"` in all three Cargo manifests** (lines 573, 601, 638): Cargo edition 2024 is an unreleased/future edition. Cargo stable is still `2021`. This is a minor accuracy issue — the examples will need updating when edition 2024 lands, or they could use `2021` now.

**Check 3: Multi-version alias demo fragmentation**

Lines 676-696 show a second Cargo.toml fragment for the multiple-version alias demo. It is presented as a standalone snippet but it is **not** a complete Cargo.toml — it only shows `[dependencies]` and `[package.metadata.sifr.aliases]`. The reader must infer it appends to the first consumer manifest. This is confusing: the fragment should either be clearly labeled as "additions to the above Cargo.toml" or shown as a unified complete manifest.

**Check 4: No missing dependency declarations**

- `sifr-demo-http` declares `sifr-demo-json` ✓
- `sifr-demo-app` declares `sifr-demo-json` and `sifr-demo-http` (with matching Git tags) ✓
- dev-dependencies on `sifr-demo-test-support` ✓
- Git tag versions (`v0.1.0`, `v0.2.0`) are consistent with the demo's intent to show multiple versions ✓

**Verdict**

The section is functionally sound. The Git-tagged version scheme is intentional for the multiple-versions demo. The only substantive gap is the consumer `__init__.sifr` omission — it should be shown explicitly to match the document's own `__init__.sifr` semantics promise.

**Not ready** — the consumer `sifr.toml` export needs a corresponding `sifr/app/__init__.sifr` file in the directory tree, and the multi-version alias demo Cargo.toml fragment needs to be clearly labeled as additions to a base manifest.

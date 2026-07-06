**Verdict: satisfied for PR 1 as scoped, with follow-ups.** The change is coherent, well-scoped to smoke restoration + supporting cache, and does not touch the migration-closure work or the merged Python interop trim. Local validation is thorough and the numbers demonstrate the intended speedup (411s → 22.6s for the corpus gate).

Key notes:
- The five findings are all cache/scope concerns, not correctness bugs in the smoke gate itself. The most substantive is #1 (missing `sifr_runtime` digest in dev mode); it degrades probe-diagnostic quality but downstream `cargo check` in GCQ still catches type breakage, so the aggregate gate remains sound.
- The `validate_smoke_profile` guard is a good invariant to have — it prevents future edits from silently reintroducing `max_entries`-based smoke selection.
- Smoke correctly drops `clippy` (heavyweight) — consistent with the stated goal of a fast enforcement lane.
- No user-facing CLI changes; both `SIFR_GCQ_ENTRY_IDS` and `SIFR_RUST_BRIDGE_PROBE_CACHE_DIR` are opt-in and default to prior behavior when unset.

Recommend addressing finding #1 in a follow-up PR by adding a `crates/sifr_runtime` source digest to `probe_cache_key` for the workspace-dev fallback path, and adding at least one unit test in `rust_interop_probe_cache.rs` covering cache-key stability across sensitive input changes.

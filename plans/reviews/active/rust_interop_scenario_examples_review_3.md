Review complete — six scenario examples plus the split checker are all wired up. Findings, ordered by severity:

### Substantive

1. **Other fixtures with package/workspace/config claims still lack scenario examples.** You asked me to flag any fixture that still has a package-level config claim without a full example. These four match that description:
   - **`proc_macro_trust`** (`fixtures/proc_macro_trust/positive/*.sifr` advertises `# fixture-trust: rust-proc-macros = ["serde_derive"]` / `rust-build-scripts = ["prost-build"]`) — the claim is a `sifr.toml [trust]` policy that only makes sense at package layout. No `scenario_examples` entry today.
   - **`native_build_script`** — same shape: trust over `cc`/`bindgen`/`cxx`/`zstd` build-scripts/native-links is a package-level claim with no scenario.
   - **`ecosystem_backend_certification`** and **`ecosystem_cli_certification`** — both declare cross-crate `features` policies (`sqlx`, `tracing-subscriber`) that are package-level config. Currently only per-crate package examples.

   The six in scope are coherent; if "completeness" means every package/workspace/config claim is mirrored by a full scenario, these four are the next gap. ([`verification/areas/rust_interop/data/rust_interop_fixture_matrix.json:263-300`](verification/areas/rust_interop/data/rust_interop_fixture_matrix.json))

2. **Heavy helper duplication between `check_fixture_matrix.py` and `_scenario_checks.py`.** The 900-line split copied 8 helpers byte-for-byte: `_rust_bound_declarations`, `_is_rust_decorated_binding`, `_decorated_function_name`, `_decorated_function_return_type`, `_verifier_binds_call`, `_bound_call_prefixes`, `_is_identifier_or_path_char`, `_contains_empty_pass_body` (`check_fixture_matrix.py:425-625` ↔ `_scenario_checks.py:284-350`). Any future rule change has to be made in both — silent divergence will go undetected. Extract to a `_binding_helpers.py` module shared by both checkers.

### Quality / minor

3. **Unused fixture binding — `hex_result` in `local_blake3_bridge/src/main.sifr:17`.** The scenario's verifier declares `hex_result` and never references it, diverging from what a real Sifr package would type-check. Either use it (e.g., wrap the digest into a record like `shared_bridge_crate` does) or drop the binding.

4. **Dead code in `check_fixture_matrix.py`.** `_rust_bound_declaration_names` (`check_fixture_matrix.py:581-582`) has no callers in the repo.

5. **`_require_trust_targets` error message is opaque** (`_scenario_checks.py:271-281`). On missing targets it prints `must declare [trust] {key} targets` without naming the missing target. Include the offending target string so failures are self-debugging.

6. **`_validate_scenario_example_dir` runs `_validate_scenario_manifests` even when `sifr.toml`/`Cargo.toml` are reported missing** (`_scenario_checks.py:106-123`). Not a correctness bug — `_read_toml` silently returns None — but the existence failures hide the rest of the manifest validation until they're fixed. Either return early or have `_read_toml` flag missing files explicitly.

7. **`cargo_locked_offline/Cargo.lock` is pinned at `version = 3` while every other scenario uses `version = 4`** (`fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock:4`). Hand-authored; works under `--locked` today, but inconsistent with the five other lockfiles and will be rewritten on the first `cargo update`. Bump to `version = 4` for consistency.

8. **Token check for `shared_bridge_crate` can be subverted by editing the comment** (`fixtures/shared_bridge_crate/examples/shared_hash_bridge/rust/sifr_shared_hash_bridge/src/lib.rs:18`). The README explicitly says the *comment-only* mention is the allowed boundary and a real `use crate::__sifr_bridge` would not pass. The current scenario validator only does substring search on the concatenated source, so a later regression that turns the comment into a real import would still pass the token check. Strengthen the validator (e.g., reject lines that start with `use crate::__sifr_bridge` after stripping whitespace).

Nothing here blocks the six scenarios — they're internally consistent and pass the checker. Items 1, 2, and 8 are the substantive follow-ups.

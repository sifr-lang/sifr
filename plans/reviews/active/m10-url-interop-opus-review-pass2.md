Now I have enough context to write the pass-2 review.

# VERDICT: PASS

The M10 wave 3 URL stdlib interop migration remains correct after the branch was rebuilt on top of the merged Post-M10 audit. The URL intrinsic ownership has moved cleanly from the compiler into private `_sifr.url` Rust interop declarations backed by `sifr_stdlib::url::*`, the public `sifr.url` surface (`Url`, `UrlQuery`, `UrlError`, parse/build wrappers, query flatten/unflatten, `ParseError` → `UrlError` translation) is preserved, and the new direct-interop rules for `str | None` / `int | None` produce correct `Option<String>` / `Option<SifrIntBridge>` calls at the sysroot crate boundary. Dependency plans and JSON snapshots now advertise `sifr_stdlib = { features = ["url"] }` instead of direct `url` / `percent-encoding`. HTTP/resource surfaces stay owned by their own file and features. The Post-M10 audit merge commits (`e73334d31`, `2fe2f2845`, `c00791a67`) are still on the branch and unchanged, and the audit's guard test now also covers `_sifr.url`.

## Findings

### Blocking
- **None.**

### Non-blocking (low severity / hygiene)

1. **Working tree has unstaged wave-3 hunks that are load-bearing for the review claims.** `git status` shows `MM` on three files: `crates/sifr_stdlib/src/url.rs` (clippy fixes: `url_parts_from_parsed(&parsed)`, explicit `set_query` branches, `Cow::into_owned` as fn-ptr, `is_multiple_of(2)`), `crates/sifr_stdlib/tests/api_behavior.rs` (unwrap → `unwrap_or_else(|err| panic!(...))` for readable failures), and `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs` (adding `_sifr.url` to `COMPLETED_MIGRATED_PRIVATE_DECLARATIONS` at lines 32–35). The last one is the change that makes `completed_private_declarations_follow_adapter_policy_syntax` cover `_sifr.url`; the validation run passed only because the tree includes the unstaged edits. Stage them before opening the PR so what CI validates equals what merges.

2. **Pass-1 non-blocking items still open** — none have shipped fixes on this branch. All remain non-blocking, listed here for continuity:
   - **Dead URL runtime preamble in `crates/sifr_codegen/src/preamble/url_http_runtime.rs`** (`URL_RUNTIME` + `build_url_runtime_items()` at lines 5–313 and 681, plus the `needs_url_runtime` gate at `crates/sifr_codegen/src/lib_modules_and_codegen.rs:448-454`). After wave 3 neither the stdlib preamble nor the intrinsic registry can trigger it; the HTTP runtime in the same file is still live and should be preserved.
   - **`registry/url_http.rs` filename is now HTTP-only** (`crates/sifr_codegen/src/intrinsics/registry/url_http.rs:1`); the doc-comment banner still says "URL and HTTP primitive intrinsic lowerers." Cosmetic; align when HTTP migrates.
   - **Robustness margin in `stdlib/sifr/url.sifr`** (lines 115–122 `_optional_port`, lines 153–161 `_pairs_from_flat`): `i = i + 2` walks the flat list without a `len(flat) % 2 == 0` guard, and `_optional_port` silently swallows non-int strings. Rust side always produces the exact 12-field / chunk-of-2 layout, so this never fires; a defensive `raise UrlError(...)` (parallel to the `len(parts) != 12` check at line 126) would be belt-and-suspenders.
   - **`is_optional_str` / `is_optional_int` silently fall through for any other optional shape** (`crates/sifr_codegen/src/rust_interop_direct.rs:57-81`). `bytes | None`, `float | None`, `list[T] | None`, or a 3-member union produce the raw `value` and will surface as a hard-to-diagnose Rust type error the first time a new migration uses them. A note in `internal_docs/sifr_sysroot_and_stdlib_architecture.md` next to the new "`int | None` / `str | None` boundary" paragraph, and/or a `debug_assert!`-style codegen error, would save the next wave a bisect.

3. **Pass-1 item #5 is resolved.** `_sifr.url` is now in the `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` matrix at `crates/sifr_stdlib_model/src/features_tests.rs:246-247`, alongside `sifr.url`, so a regression that re-attaches direct `url`/`percent-encoding` to the private leaf is caught.

## Validation notes

- The listed focused-cargo commands cover the correct surfaces (intrinsic-ownership guard, direct optional-int codegen unit, adapter-policy guard now including `_sifr.url`, stdlib-model deps matrix, sifr-stdlib url leaf behavior, snapshot regeneration, E2E `network_http_url_query_percent.sifr`, and clippy on `sifr_stdlib --features url`). Fmt + `git diff --check` + file-size guardrail are covered.
- Not covered by the listed set — recommend running before opening the PR:
  1. `scripts/run_all_tests.sh --profile create-pr` (the authoritative gate mandated by `AGENTS.md`; transitively runs the HIR guardrail and the full `test_e2e_pass` suite, which the single fixture does not).
  2. `cargo clippy --workspace --all-targets --locked -- -D warnings` — clippy has been run only for `-p sifr_stdlib --features url`, so the new `is_optional_str`/`is_optional_int` branches and the driver test have not been pedantically linted at workspace scope.
  3. A parity spot-check for a downstream module that transitively pulls `_sifr.url` via HTTP (e.g., a `sifr.http_transport`-shaped fixture), to confirm the URL preamble emission gate at `lib_modules_and_codegen.rs:448-454` really doesn't fire now that no stdlib caller emits `__sifr_url_*` markers.
- Post-M10 audit checkpoint is intact on the branch: commits `e73334d31 Audit stdlib adapter policy adherence`, `5063c0cef Merge pull request #2774`, `c00791a67 Record Post-M10 adapter audit merge`, and `2fe2f2845 Merge pull request #2775` are all present and untouched by the wave-3 diff; the tracker entry for the audit is preserved verbatim.

If items (1) and the recommended validation runs are green, wave 3 is ready to ship.

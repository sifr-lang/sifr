## Review: M2c StdlibRustSource provenance slice

### Findings (all nits — no correctness issues)

- **`StdlibRustSource::contains` shim is a mild abstraction leak** — `crates/sifr_codegen/src/lib_modules_and_codegen.rs:140-145`. The name doesn't hint that it only searches `self.rust`; a reader might expect it to check `module`/`source_path` too. Kept only to preserve existing `private_code.contains(...)` test spellings. Consider migrating tests to `private_code.rust.contains(...)` and dropping the helper in a follow-up so provenance fields are the only "public API" of the type.
- **`lib_modules_and_codegen.rs` is at 899/900 lines** — under cap, but the next M2 slice (raw-string rejection) is very likely to push it over. `StdlibRustSource` is ~15 lines and self-contained; extracting to a tiny sibling module (or into `lib.rs`-adjacent placement) now would remove the guardrail cliff without any cost. Acceptable to defer, but preemptive extraction is cheap.
- **Fallback branch in `canonical_stdlib_source_path` returns an absolute filesystem path** — `crates/sifr_driver/src/stdlib/bootstrap.rs:509-515`. If `sysroot` is `None` or `strip_prefix` fails, it emits e.g. `/Users/.../stdlib/_sifr/platform.sifr`, which is NOT the canonical sysroot-relative form. In current wiring the only caller (`compile_stdlib_uncached`) always passes `Some(sysroot)` and paths are constructed from `stdlib_root`, so this is unreachable. It reads as a silent fallback that could regress the invariant later — either drop the `Option<&ResolvedSysroot>` parameter, or `debug_assert!(false, ...)`/return an explicit error on the miss.

### Review questions

1. **Behavior preservation** — Yes. All prior `String` accesses in `generate_rust_with_stdlib_for_module` now go through `.rust`/`.rust.clone()`; downstream filtering, sync/parallel runtime rewrites, and prelude collection are untouched. Test-facing `contains` call sites still compile via the impl shim.
2. **`source_path` normalization** — Correct. `LoadedStdlibSource.path` is built as `stdlib_root/{sifr,_sifr}/<name>.sifr` (`crates/sifr_stdlib_manifest/src/sources.rs:373, 585-591`), so stripping `paths.stdlib_root` and prepending `stdlib/` yields `stdlib/_sifr/platform.sifr` / `stdlib/sifr/math.sifr` — matching the manifest `declaration_files` shape referenced in the plan. Component-join with `/` is cross-platform safe. Identical output for dev layout (`<root>/stdlib`) and installed layout (`<root>/lib/sifr/stdlib`) because both resolve into the same `stdlib_root`.
3. **`source_sha256` provenance** — Correct. Computed from `source.source` (the same bytes fed to `parse_module_raw` on `bootstrap.rs:58` and `lower_stdlib_source` on `bootstrap.rs:91`). Test uses `include_str!("../../../../stdlib/_sifr/platform.sifr")`, matching what the loader read from disk.
4. **Risk from sha2 / type change / `contains` helper**:
   - `sha2 = 0.10.9` is already a workspace dep (`crates/sifr_sysroot/Cargo.toml:11`), so adding it to `sifr_driver` introduces no new third-party surface. Compile-time only; not linked into generated user binaries — so M2d's "no new direct third-party generated dependencies" guard is not tripped.
   - `module_rust_code` value type change is workspace-internal; all consumers (bootstrap producer, codegen consumer, cache assertion, ~18 test sites) compile against the new shape.
   - `contains` helper — see nit above.
5. **File-size guardrail** — Under cap per the letter of the rule. See nit above about proactive extraction before the next slice.

### Verdict

READY_WITH_NITS

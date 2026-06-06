## M3 Implementation Review — Pass 3

**Result: PASS — no material blockers.**

The post-pass-2 platform-golden unblock is sound. I verified the fixture (`verification/platform/golden/locale_host_limited_formatting.sifr`), the manifest entry, the runtime/codegen lowering paths it exercises, and the updated ledger/traceability evidence. I also re-ran `scripts/run_platform_golden.sh` locally — `3 pass / 2 skip`, with the new fixture passing.

### Verified against M3 contract

- **Explicit locale formatting**: `NumberFormatter(LocaleId("bn"))` and `Collator(LocaleId("en"), "primary")` are object-scoped with explicit locale and strength. The Bangla decimal assertion matches the runtime unit test in `crates/sifr_runtime/src/i18n.rs:200-203`. ✓
- **Host-limited `host_locale`**: Treated as `LocaleId | None`; on `Some` only asserts non-empty `to_string()`, on `None` skips. Nothing in the fixture threads the host value into text I/O, encoding, or any default — the M1 "no implicit locale encoding" invariant is preserved. ✓
- **No implicit text I/O defaults**: The fixture imports only locale formatters and `host_locale` from `sifr.i18n`; it doesn't touch `sifr.io`, `open(...)`, `sifr.encoding`, or any byte/text boundary. ✓
- **No mixed typed-error issue**: `NumberFormatter.format` and `Collator.compare` both lower through `map_string_error(..., "FormatError")` (`crates/sifr_codegen/src/intrinsics/registry/i18n.rs:85-91, 137-148`), matching the `Result[_, FormatError]` signatures in `lib/sifr/i18n.sifr:75, 120`. A single `except FormatError` correctly covers both. ✓
- **Magic stdout / exit**: Prints `platform-golden: pass`; manifest `expected_exit: 0` and `expected_stdout_contains` align. ✓

### Manifest entry

`depends_on: ["milestone_text_i18n_3"]` is correct, `blocked_until: []` correctly reflects M3 implemented, and the command uses `--isolated`. `--isolated` only ignores Sifr config (`crates/sifr/src/cli_model_and_entrypoint.rs:47`), so env-derived `host_locale()` still works — the fixture's `None`-tolerant branch handles both `C.UTF-8`-stripped CI hosts and named-locale dev hosts.

### Docs / validation evidence

- `text_i18n_m3_traceability.md:9` records the host-limited semantic correctly ("never feeds text I/O defaults").
- `text_i18n_substrate_inventory.md:45` still marks `implicit preferred text encoding` as `rejected`; the no-global-state policy (line 109) remains intact.
- Ledger entry (`issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:339, 387`) records `scripts/run_platform_golden.sh` 3 pass / 2 skip and explicitly notes the unblock.
- Both `manifest.json` and `text_i18n_substrate_inventory.json` parse cleanly.

### Non-blocking observation (not gating PASS)

- `must_not_depend_on: ["implicit_text_open_default_encoding"]` in the new entry uses a behavior label, while the platform-contract example (`issues/ad-hoc-production-stdlib-platform-contract.md:173`) and existing entries (`milestone_text_i18n_1`) use milestone IDs. The runner ignores this field entirely (`scripts/run_platform_golden.sh` only checks `blocked_until`), so this is documentation-style drift, not enforcement breakage. Could be tightened in M5 docs cleanup, e.g., to `"milestone_text_i18n_1"` (the milestone that established the no-implicit-default rule), with the rationale captured in the `checks` array.

Unrelated dirty concurrency-runtime files were ignored as instructed.

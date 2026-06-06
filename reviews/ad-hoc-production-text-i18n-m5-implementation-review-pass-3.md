## Code Review: M5 Text/I18n Production Gate (Pass 3)

**Branch:** `text-i18n-m5-production-gate` vs `main`
**Scope:** Pass-2 PASS plus post-pass-2 generated-code-quality remediation: class emitter refactor + new codegen lockdown test, gcq producer-fingerprint cache keying + `inherent_to_string_shadow_display` allowlist, and the full merge-gate validation evidence added to the execution ledger. Unrelated dirty concurrency-runtime files are explicitly out of scope.

---

### 1. Verdict: **PASS**

The pass-2 PASS still stands. The post-pass-2 remediation is mechanical, correctly scoped, and lands cleanly with two full merge-gate runs at 78/78 e2e pass + 34/34 hardening (`execution.md:386,390`).

Key evidence:

- **No `#[allow(...)]` is emitted from class lowering.** `crates/sifr_codegen/src/class_emitter.rs:330-331` only hoists the existing `has_auto_display` computation to sit alongside `has_callable_field`/`has_custom_eq`/`has_custom_str`; `class_emitter.rs:430-432` reuses the same boolean in the existing `else if` arm. No allow-attribute is constructed anywhere in the class-emission path (confirmed by ripgrep for `allow` across `crates/sifr_codegen/src` — zero hits).
- **The lockdown test is correct and minimal.** `classes_and_basics_codegen_tests.rs:83-122` builds an HIR class with an inherent `to_string` method and the same field shape Sifr-generated `Display` covers (`Type::Str`). It asserts (a) no `#[allow(clippy::inherent_to_string_shadow_display)]` in the output, (b) `impl LocaleId` block present, (c) `impl std::fmt::Display for LocaleId` present. That trio pins exactly the decision: inherent `to_string` + auto-Display coexist without an item-local allow.
- **Generated-code clippy invocation now allows the lint at the command line.** `verification/generated_code_quality/generated_code_quality.py:120` inserts `-A clippy::inherent_to_string_shadow_display` into `GENERATED_CLIPPY_ARGS`, alphabetically slotted between `identity_op` and `iter_cloned_collect`. This moves the allow from "would-be in-source attribute" to "external allowlist applied to generated workspaces only", matching the M5 closure stance.
- **Producer fingerprint correctly invalidates stale cached entries when the compiler changes.** `generated_code_quality.py:354-377` walks the nine first-party compiler/runtime crate `src` trees plus their `Cargo.toml`, the workspace `Cargo.lock`/`Cargo.toml`, and the gcq script itself; restricted to `.lock/.py/.rs/.sifr/.toml`; sorted before hashing; `@functools.cache`'d to keep it one-per-process. `entry_cache_key` (`:380-389`) now prefixes the per-entry digest with the producer fingerprint, so codegen edits force a cache-miss without depending on entry source-byte changes. The pass-2 cache-skew failure mode — codegen change leaves cached crate outputs intact — is closed.
- **Validation matrix is complete in the ledger.** `execution.md:373-390` lists every required command: `cargo fmt --check`, `cargo test -p sifr_codegen test_class_to_string_method_does_not_emit_generated_allow`, focused clippy evidence `target/sifr_generated_code_quality/evidence/clippy-1780767555-1547.json`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_stdlib`, `cargo test -p sifr -- stdlib`, `scripts/run_e2e_pass.sh` (78/78 with cache hits 24/24), `scripts/run_all_tests.sh --profile create-pr` (twice, latest 259.84s, 72/72), and `scripts/run_all_tests.sh` (twice, latest 793.37s, 78/78 e2e pass, hardening 34/34). The pass-2 N1 gap (merge gate missing) is now closed.
- **Evidence file is well-formed.** `target/sifr_generated_code_quality/evidence/clippy-1780767555-1547.json` contains 7 records all `status: "passed"`, including `demo-007-text-i18n` against `demos/text_i18n/main.sifr`.

---

### 2. Blocking findings

**None.**

---

### 3. Non-blocking observations

1. **`clippy-1780767555-1547.json` is `demos-required`-only despite the two-group invocation.** The ledger entry at `execution.md:388` records the command as `--group demos-required --group e2e-pass-representative`, but the evidence file only contains 7 records, all from `demos-required` (`verification/generated_code_quality/manifest.json` has 7 demo entries). Root cause: `SIFR_GCQ_MAX_ENTRIES=7` is applied after group filtering in `selected_positive_entries` (`generated_code_quality.py:323-336`), and `demos-required` entries appear first in manifest order, so the 7-entry cap exhausts the budget before any `e2e-pass-representative` row is selected. The ledger phrasing is technically accurate (the command ran), but a reviewer reading the JSON will not see e2e-pass-representative coverage. Either drop `--group e2e-pass-representative` from the command string in the ledger, raise `SIFR_GCQ_MAX_ENTRIES`, or note the cap behavior in the ledger line. Not blocking because the immediately-prior full merge-gate run (line 390) exercised the full manifest via `scripts/run_all_tests.sh`.

2. **Producer fingerprint excludes `verification/generated_code_quality/manifest.json`.** `PRODUCER_FINGERPRINT_INPUTS` (`generated_code_quality.py:25-47`) covers crate sources, `Cargo.lock/toml`, and the gcq script, but not the manifest. If a manifest edit changes an entry's `group`/`evidence_category` without touching the source file, the cache key won't notice and a stale workspace will be reused. The producer-fingerprint name suggests "producer of generated Rust", so manifest classification is out of scope by design — but the omission is worth a one-line comment in the script so a future reader doesn't expect manifest changes to invalidate. Not blocking for M5; pre-existing behavior.

3. **The codegen lockdown test only checks structural emission, not runtime semantic equivalence.** `classes_and_basics_codegen_tests.rs:118-121` proves the absence of the allow attribute and the presence of both impl blocks, but does not assert that `LocaleId { value: "en-US" }.to_string()` calls the inherent method or that `format!("{}", ...)` produces matching output. With the lint suppressed at the command line, an accidental future divergence between inherent `to_string` and `Display` would compile cleanly. The existing M5 demo (`demos/text_i18n/main.sifr:85`) exercises `LocaleId("en-US")` via `NumberFormatter`, so the runtime path is covered by demo + merge gate; an e2e pass fixture for the inherent-to_string-plus-Display pattern would be a strictly stronger lock. Not blocking.

4. **`pass-1.md:48` still carries the stray "ther structural changes…" trailing fragment noted in pass-2 N7.** Cosmetic; pass-2 didn't clean it up and pass-3 doesn't either. Trim before phase close.

5. **Working tree has the four remediation files unstaged**: `crates/sifr_codegen/src/class_emitter.rs`, `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs`, `verification/generated_code_quality/generated_code_quality.py`, and `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md`. Commits `99b8c0225 Close M5 text i18n production gate` and `649b1d9f1 Record M5 text i18n PR` already landed the pass-1/pass-2 closeout; this pass-3 remediation needs a follow-up commit before PR push. Per the review scope, the six dirty/untracked concurrency-runtime files (the two issue docs, `internal_docs/structured_runtime_work_model.md`, and four `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-*.md` files) must remain unstaged.

6. **Ledger row `M5 pass 3: pending after generated-code quality remediation.` (`execution.md:215`) needs to flip to a PASS line once this review file lands.** This file (`reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-3.md`) was a 0-byte placeholder at scope-time and is now populated by this review.

7. **`producer_fingerprint_files()` walks ~9 crate `src` trees on first call per process.** With `@functools.cache` the cost is one digest per process; payload is small (low-MB of Rust/TOML across the compiler crates) and the SHA-256 is truncated to 16 hex chars. No action needed; recording it because the cache-miss path now reads more than the entry's own source.

---

### 4. Re-review required?

**No.** The class-emitter change is a no-op refactor, the lockdown test is structural and green, the gcq script changes are mechanical (one alphabetically-ordered allow flag + a fingerprint-based cache-key extension), and two clean merge-gate runs (716.17s and 793.37s, both 78/78 e2e + 34/34 hardening) cover the substantive risk. The remaining items are ledger bookkeeping (line 215 pending → PASS, optionally amend line 388 to match the actual evidence scope) and a follow-up commit for the four dirty M5 files. They can be verified on the final PR diff without another full pass.

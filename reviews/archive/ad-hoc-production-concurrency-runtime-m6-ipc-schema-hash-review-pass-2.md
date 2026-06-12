I re-reviewed the diff against `origin/main` after the pass-1 advisories.

**Pass-1 advisory #1 — sensitivity test exercises record shape (RESOLVED)**
- `ipc_schema.rs:248-264` (`schema_hash_v1_is_stable_and_sensitive_to_shape`) now mutates `fields[1].ty` of the request record from `Int` to `Float`, then asserts the hash differs. This exercises an actual request-record field type rather than `compatible_version_max` metadata. The golden hex `4733c89fb23a40ecb5f3bcda99fb34da` is still pinned against the unmodified descriptor.

**Pass-1 advisory #2 — stale ledger line counts (RESOLVED)**
- Issue ledger entry `issues/...md:993` now reports `crates/sifr_stdlib/src/ipc_schema.rs 265`, `crates/sifr_stdlib/src/lib.rs 430`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md 243`, and the ledger itself `1964`. `wc -l` on the current tree returns exactly the same numbers (265 / 430 / 243 / 1964). No drift.

**Scope unchanged**
- The diff still touches only `crates/sifr_stdlib/src/ipc_schema.rs` (new, 265 lines), `crates/sifr_stdlib/src/lib.rs` (5-line re-export), and the two doc files. `grep` for `ipc_schema|IpcSchema|schema_hash` across `crates/` returns only those two files — no compiler/codegen/source registrations.
- No frame encode/decode, no process-pipe transport, no payload eligibility enforcement, and no public worker API surface have been added.

**Traceability / ledger framing**
- `verification/.../concurrency_runtime_m6_typed_ipc_design.md:30` describes the addition as "Internal schema descriptor and hash v1" and explicitly notes "Compiler integration and generated schema extraction remain follow-up work."
- `issues/...md:982-995` records this as a discrete schema-hash slice within M6, not as M6 completion. No overclaim of compiler integration.

**Validation**
- `cargo test -p sifr_stdlib ipc_schema -- --nocapture`: 2 passed (`canonical_descriptor_is_stable_and_ordered`, `schema_hash_v1_is_stable_and_sensitive_to_shape`), 0 failed.
- `cargo fmt --check`: PASS (no diff).
- `git diff --check`: PASS (no whitespace issues).
- `python3 scripts/check_file_size_guardrails.py`: PASS (2250 files, limit 900 lines).
- (For local reproduction the `third_party/ruff` submodule needed `git submodule update --init third_party/ruff` first; pass-1 hit the same missing-submodule blocker.)

**PASS** — both pass-1 advisories addressed, scope and traceability framing unchanged, all four targeted validation commands green.

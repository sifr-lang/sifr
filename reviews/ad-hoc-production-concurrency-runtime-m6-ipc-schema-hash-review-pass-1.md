I've reviewed the diff against `origin/main`. The change consists of one new file (`crates/sifr_stdlib/src/ipc_schema.rs`, 262 lines), a re-export in `lib.rs`, and traceability/ledger entries.

**Scope verification**
- `ipc_schema.rs` only declares descriptor types (`IpcSchemaDescriptor`, `IpcSchemaField`, `IpcSchemaVariant`, `IpcSchemaType`) plus canonical-rendering and FNV-1a-128 hash helpers. No frame encode/decode, no pipe transport, no payload eligibility enforcement, no public worker APIs.
- The new items are `pub use`'d at the `sifr_stdlib` crate root, but `grep` shows no compiler/codegen/sources/intrinsic registrations referencing them — nothing user-facing in `.sifr`.

**Descriptor and canonical rendering (`ipc_schema.rs:52-71`, `93-160`)**
- Eligible shapes cover Bool/Int/Float/Str/Bytes/None, Option, Result, List, DictStr, Tuple, Record, Enum (with optional variant payload).
- Canonical rendering deterministically emits protocol schema version, module path, schema name, compatible version range (`min..max`), request/response/error types, record fields (in declared order), enum variants (in declared order), and recurses for nested containers. `push_escaped` escapes structural delimiters and control chars so descriptor content cannot collide with delimiters.

**Schema hash v1 (`ipc_schema.rs:74-91`)**
- FNV-1a-128 implemented inline; constants match the canonical offset basis `0x6c62272e07bb014262b821756295c58d` and prime `0x0000000001000000000000000000013b`. No external crates added.
- Golden test pins `4733c89fb23a40ecb5f3bcda99fb34da`. Sensitivity test mutates the descriptor and asserts the hash differs.

  *Minor advisory:* the sensitivity test mutates `compatible_version_max`, which is metadata, not record/enum shape. It does prove descriptor-sensitivity (the version range is part of the canonical bytes), but a stricter "shape" test would also mutate e.g. a field name/type or variant order. Not a blocker; just worth strengthening when compiler integration lands.

**Traceability / ledger (`issues/...md:982-995`, `verification/...md:30`)**
- Both explicitly note "Compiler integration and generated schema extraction remain follow-up work." No overclaim of M6 completion or compiler wiring.

  *Minor advisory:* the ledger's "Touched file line counts" reports `1950` for the ledger and `242` for the verification doc; actual `wc -l` is `1964` and `243`. Cosmetic, but the numbers are slightly stale.

**File-size guardrail**
- `crates/sifr_stdlib/src/ipc_schema.rs` 262, `lib.rs` 430, verification doc 243 — all under 900. Issue ledger (1964) is outside the guardrail's `crates/scripts/verification/demos` scope. `python3 scripts/check_file_size_guardrails.py` reports PASS.

**Validation note**
- I could not re-run `cargo test -p sifr_stdlib ipc_schema` locally because `third_party/ruff/crates/ruff_text_size/Cargo.toml` is missing (submodule not populated in this worktree). Relying on the user-reported PASS for the targeted test command.

**PASS** — with two minor advisories (sensitivity test exercises version metadata rather than record/enum shape; ledger's quoted line counts are slightly stale vs. current files). Neither is a correctness or scope concern.

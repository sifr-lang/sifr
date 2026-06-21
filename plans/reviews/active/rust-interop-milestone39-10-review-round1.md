I've read the implementation, tests, fixtures, and architecture/phase docs. Findings below, severity-ranked.

## Findings

### High — Scope deviation: `shape=` and `strides=` are absent

`plans/phases/39_rust_interop.md:219` explicitly lists the milestone scope as:

> Add tensor buffer contracts with **dtype, shape, layout, strides, device, and ownership** metadata.

and `internal_docs/rust_interop_architecture.md:667` (the data‑oriented bridges section) repeats:

> tensor buffers with **shape, dtype, layout, strides, device, and ownership** metadata.

The implementation in `crates/sifr_driver/src/build/rust_interop/advanced_data_validation.rs:137` only validates `data | schema | dtype | rank | layout | device | ownership | protocol` — no `shape=`, no `strides=`. A single integer `rank` is not equivalent to a shape vector + strides vector.

The doc is internally inconsistent: the contract‑surface paragraph at `internal_docs/rust_interop_architecture.md:672–674` lists `rank=` instead of `shape=`/`strides=`, so the architecture has been quietly narrowed to match the code, while the data‑oriented bridges paragraph above it and the phase scope still claim the larger contract. The status text on the milestone and the new READMEs are honest about what was implemented, but the milestone scope hasn't been met or formally reduced.

Pick one:
- Add `shape=` (list of non‑negative integers) and `strides=` (list of integers), with consistency checks against `rank`, OR
- Tighten the milestone scope and the data‑oriented bridges paragraph in the architecture to drop `shape`/`strides` and explicitly defer them to ecosystem closeout.

Today the docs read as "everything done"; either the code or the docs needs to move.

### Medium — Negative test coverage is thin for the new contract surface

`rust_interop_advanced_data_contract_tests.rs` has 4 negative tests covering: missing schema, wrong bridge root, DLPack ownership, negative rank. The validator branches that are uncovered:

- `data=arrow_array` accept/reject path (only `arrow_record_batch` is exercised).
- `data=dataframe` accept/reject path (entirely untested).
- "Arrow and dataframe views cannot declare tensor metadata keys" (`advanced_data_validation.rs:213`) — no test feeds e.g. `data=arrow_record_batch, dtype=f32`.
- "tensor and DLPack views cannot declare `schema=`" (`advanced_data_validation.rs:232`) — no test.
- "Arrow and dataframe views use `ownership=borrowed` or `ownership=owned`" (`advanced_data_validation.rs:215`) — no `ownership=transfer` on Arrow case.
- "advanced data view metadata requires `data=`" / "requires `ownership=`" (`advanced_data_validation.rs:188–189`) — no test.
- Invalid `layout=foo`, `device=cuda`, `dtype=bool32`, `data=parquet` — no tests for any of the symbol allow‑lists; only the negative‑integer rank.
- DLPack with `ownership=transfer` but missing `protocol=` (`advanced_data_validation.rs:238`) — no test.

The CLAUDE.md guidance "a feature is not complete until its failure mode is as deliberate as its success path" applies; this is the new contract surface for the milestone and the negative matrix is sparse.

### Medium — Test name overclaims

`package_rust_interop_rejects_tensor_dtype_shape_mismatch_metadata` (`rust_interop_advanced_data_contract_tests.rs:128`) only mutates `rank=2` → `rank=-1`. It tests neither dtype mismatch nor shape mismatch (and shape isn't a concept the validator knows about). Rename to e.g. `rejects_negative_tensor_rank`, or extend the test to cover dtype/layout invalid symbols too.

### Medium — `tensor_dlpack_bridge` "passing" with `Borrow` owner + `ownership=transfer`

The DLPack test (`rust_interop_advanced_data_contract_tests.rs:39–43, 198–207`) declares `ownership=transfer` on a view whose owner parameter is `input: bytes` lowered with `RustBridgeParamConvention::Borrow`. Transfer semantics imply the bridge consumes the buffer, but the validator accepts a borrow‑convention owner. This is exactly the kind of "explicit ownership" inconsistency the milestone is supposed to lock down ("ownership contract is explicit"). Either:
- enforce that `data=dlpack, ownership=transfer` requires an owned/consumed owner parameter, or
- write a comment in the validator stating that owner‑convention consistency is intentionally deferred.

Otherwise the contract surface is laxer than the README ("rejects DLPack handoff unless `ownership=transfer` and `protocol=` are explicit") suggests.

### Low — Schema path is not root‑constrained

`advanced_data_validation.rs:165–168` accepts any `RustInteropValue::TargetPath` for `schema=`. The README states "requires explicit schema identity through `schema=`" and elsewhere claims `sifr_arrow_bridge` is the canonical root, but `schema=anything.foo.Bar` passes the advanced‑data layer. Downstream path resolution will reject unresolved roots, but the diagnostic at that point won't be `SIFR-RUST-ZC-0001` and won't be span‑anchored to the schema key. Minor — consider requiring `schema=` to live under `sifr_arrow_bridge.*` (the same root the validator already pins for `@rust(...)` targets).

### Low — `bf16` missing from dtype allow‑list

`advanced_data_validation.rs:286` allows `bool | i8 | u8 | i16 | u16 | i32 | u32 | i64 | u64 | f16 | f32 | f64`. `bf16` is standard in `candle`/ML tensors and will be needed for the closeout. Easy to add now; otherwise it'll be a follow‑up gotcha.

### Low — `validate_shared_bridge_root` is silent when no `@rust(...)` function decoration exists

`advanced_data_validation.rs:90–102`: if no declaration with `RustInteropDecoratorKind::Function` is found, the function returns without diagnostic. In practice an `@rust.view(...)` without a paired `@rust(...)` is also handled by other passes, but this branch could quietly skip an advanced‑data check on a malformed declaration. Worth a one‑line note or an explicit "missing `@rust(...)`" path.

### Low — Fixture matrix "passing" labels for `advanced_data_matrix`

`verification/areas/rust_interop/data/rust_interop_fixture_matrix.json:254–262` lists `advanced_data_matrix` with `required_crates: ["datafusion", "polars", "ndarray", "candle"]` and `positive_evidence.status: "passing"` under `execution_kind: contract-only`. No test in `rust_interop_advanced_data_contract_tests.rs` references any of those crate names; the contract‑shape tests use synthetic `sifr_arrow_bridge`/`sifr_tensor_bridge` paths. The README disclaims runtime certification, but the matrix entry alone reads as if the listed crates were exercised. Either:
- mark the positive/negative evidence as `contract-only-passing` (or similar new status), or
- expand the README cross‑reference so a fixture‑matrix reader cannot conclude the crates were touched.

The same caveat applies more weakly to `arrow_record_batch` (`required_crates: ["arrow"]`) and `tensor_dlpack_bridge` (`required_crates: ["ndarray"]`).

### Observation (not a bug) — `validate_advanced_data_contracts` runs only after zero‑copy validation passes

`rust_interop.rs:135–142` short‑circuits on any zero‑copy diagnostic before running advanced‑data validation. That means a view that's both shape‑invalid and advanced‑data invalid will only surface the zero‑copy errors first. Fine and consistent with existing async/opaque pattern, but worth keeping in mind when writing diagnostics docs.

## Blocker assessment

**Blocker for merge:** the scope deviation on `shape=`/`strides=` (Finding 1). The milestone advertises a contract that doesn't exist in code. Either implement it or reword the scope/architecture before this lands; today the docs overpromise.

**Strongly recommended before merge:** the test coverage gap (Finding 2) and the test name overclaim (Finding 3). The current 7 tests don't fence the new contract surface enough for a milestone whose only DoD is "validate metadata, ownership, and dtype behavior."

**Nice to fix in this round:** the DLPack owner‑convention inconsistency (Finding 4) and the fixture‑matrix labelling (Finding 8). Both will be hard to retrofit after closeout.

## Another review round?

Yes — round 2 is required. After Finding 1 is resolved (code or doc) and Findings 2–4 are addressed, a follow‑up pass should re‑check: the expanded negative test matrix, any new tensor‑metadata keys, the updated phase/architecture wording, and the fixture‑matrix status fields.

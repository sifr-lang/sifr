All verification is complete. Here is the full pass-4 review.

---

# Review: PR #2992 — M12 DLPack One-Shot Tensor Transfer (full pass 4, post-remediation)

**Branch:** `codex/m12-dlpack-one-shot-transfer` (`69170536d` + `2364c83e4` + remediation `18701ea14`, 99 files, +5,201/−709 vs `main`)
**Scope:** complete `main...HEAD` diff against the M12 requirements (`plans/issues/active/ad-hoc-declaration-first-python-interop.md:1734-1795`), independent re-verification of the pass-3 blocker remediation without assuming it correct, and a fresh sweep across codegen paths, runtime ABI/capsule lifecycle, lowering, ownership, evidence lanes, docs, and maintainability. Pass 2 is an invalid attempt (recorded as such in-repo); pass 1 and pass 3 findings were used as the ledger.

**Executed this pass (not just read):** `cargo test -p sifr_runtime --features python python::dlpack_ops` → 18/18; `sifr_codegen` dlpack 8/8, direct-interop 11/11 (includes the new regression test); `sifr_lowering` dlpack 10/10; full e2e pass suite at HEAD → **674/674, exit 0, signature `1f8b1cadc4f48ec8`** (merge-scale set, superset of create-pr's 131); pass-3's exact blocker reproducers (`/tmp/m12review/demo_copy`, `/tmp/repro_attr_drop.sifr`) rebuilt with a freshly compiled HEAD compiler → both now finish release native builds; pass-1's mixed Arrow+DLPack repro → builds, generated code shows exactly one `drop(__sifr_python_args)`/`(__sifr_python_kwargs)` pair followed by both reconcilers, ordered after the call; HIR maintainability guardrails PASS; file-size guardrails PASS (2,761 files); `cargo fmt --check` clean; `cargo clippy --workspace -- -D warnings` (new finding below); lane report inspection (dlpack-cpython311 `compatibility-passed` on CPython 3.11.14, dlpack-examples both binaries `example-passed`, python interop 16/16 blocking variants with 0 failures, all generated on the remediated tree).

## Pass-3 blocker: independently verified remediated

**B1-NEW (unguarded hoisted drops on `@python.attr` retained-callback owners) — fixed exactly as prescribed and proven at three levels.** `ArgumentGuards::append_reconciliation` now early-returns when both guard lists are empty (`crates/sifr_codegen/src/python_zero_copy_arguments.rs:69-71`). I verified the fix is complete, not just present:
- Structurally: the drop statements exist nowhere else (protocol reconcilers in `python_arrow_codegen.rs`/`python_dlpack_codegen.rs` contain no drops); attribute-kind methods can never carry guards (no parameters), and the `owner_retained_errors`-only branch at `python_interop_direct.rs:816-817` now reaches only the self-contained `append_owner_failure_evidence`, which touches no argument frame. The `consumes_receiver` path returns before reconciliation and enforces zero params, so no path can emit a drop of an undeclared local.
- By test: the new `retained_callback_owner_attribute_does_not_drop_an_absent_argument_frame` unit test asserts both the absence of the frame identifiers and `syn` parse validity.
- End-to-end: the reviewer's exact reproducer builds and the generated accessor (inspected in materialized `sifr_output/src/main.rs`) has no drops while keeping `get_attr`, failure-evidence attachment, and retained-failure reconciliation intact. Callback-only and owner-errors-only paths are restored to pre-remediation (gate-proven) shape.

**CPU stream mapping (pass-3 minor) — fixed with runtime evidence.** `acquire_dlpack_tensor` now sends `stream=None` whenever the producer reports CPU (`dlpack_ops.rs:111-114`), after `validate_device_policy` has already required family/id match — so `device=any` with a validated CPU stream still passes array-API-conforming `stream=None`. Pinned by the extended `cuda_and_any_require_a_matching_explicit_stream` test (CPU exporter observes `seen_stream == -1`, deleter count exact at 4).

## Fresh verification of the core contracts (this pass, independent)

- **Exact-once deleter / one-shot transfer:** re-audited every `finalize` branch (`dlpack_ops/argument.rs:83-150`) — consumed (used-sentinel observed → entry marked released, consumer owns deleter), unconsumed (rename to sentinel → entry drop releases once), rename-failure and clone/cast/name-failure (`relinquish_to_capsule` → capsule destructor is sole owner). Each branch has exactly one deleter owner; pending releases drain under the GIL; the store lock is never held while Python runs. Move commits at `prepare_dlpack_argument` (store entry removed), so post-consumption failure keeps the tensor moved.
- **No-copy/no-retry:** single `__dlpack__(stream, max_version=(1,0), copy=False)` call, call-count pinned; versioned `IS_COPIED` (bit 1) rejected without leaking; major-version gated before reading 1.x fields; ABI struct layouts match the DLPack spec exactly; shape/strides copied into owned Vecs; null data only for empty tensors; element-count overflow checked.
- **Affine ownership:** `PythonDlpackTensor`/`PythonDlpackStream` statically non-Clone/non-Send; `release`/`prepare_argument` consume identity via `take_resource`; `release()` on borrowed params rejected and owning-binding moves flow-marked in lowering; consumers must be plain `own`, non-omittable, rejected on async declarations (`direct_validation.rs`); 216-line lowering contract matrix pins duplicate-return, double-consume, release-then-use, loop-move, constructor, and callable cases.
- **Stream/device semantics:** closed device atoms, mandatory stream policy, non-CPU requires `parameter(name)`, keyword-only immutable-borrow `python.DlpackStream` parameter filtered from call shapes, CUDA token 0 and negative ids rejected, `device=any` validated against producer-reported device before the one permitted call.
- **Blocking evidence:** `dlpack-cpython311` blocking in all four profiles with an exact-18-test-set assertion (self-tested against drift in both directions); `dlpack_evidence.py` enforces owner-file existence, live rows matching the executable case registry, and profile blocking; compiled PyTorch and TensorFlow binaries re-run with runtime-computed pointer-identity and zero-residual assertions; capability matrix rows flipped to `active`/`passing` accurately.

## Minor

- **m1 (new). The branch breaks the documented workspace lint command.** `cargo clippy --workspace -- -D warnings` (AGENTS.md "Linting") fails with 5 pedantic errors, all on lines new in this diff: `crates/sifr_type_system/src/types/definitions.rs:82` and `:84` (`doc_markdown` — "DLPack" needs backticks), `crates/sifr_lowering/src/lower/python_interop/dlpack.rs:191` (`semicolon_if_nothing_returned`), `crates/sifr_codegen/src/python_dlpack_codegen.rs:22` and `crates/sifr_codegen/src/python_zero_copy_arguments.rs:26` (`needless_pass_by_value`). The rest of the workspace is clean with those three lints allowed, so `main` is unaffected. No blocking gate runs workspace clippy (confirmed: no profile lane invokes it; CI mirrors the scripts), and there is zero runtime effect — but these five mechanical fixes should land before merge to keep the documented command green.
- **m2 (carried).** M12 wave checkboxes in the plan doc remain unticked (consistent with tick-at-merge practice; the milestone-review row is honestly open).
- **m3.** Docs don't state that CPU producers now always receive `stream=None` (including under `device=any` with validated CPU stream metadata); nothing contradicts it, but one sentence in `docs/python-interop.mdx:402-405` would close the loop.

## Advisory

- `crates/sifr_codegen/src/lib_runtime_needs.rs` remains at 899/900 lines — next touch must refactor.
- `third_party/ruff` carries a local uncommitted formatting-only line-join in `crates/ruff_python_parser/src/parser/expression.rs` (semantically identical; not in the diff) — revert before merge to keep the submodule pristine at its pin.
- `plans/reviews/active/m12-dlpack-full-review-pass4.md` exists as an empty untracked placeholder; this pass's record should be committed there (this review ran read-only and did not write it).
- Carried theoretical corners unchanged: `PythonDlpackArgument::Drop` ignores `attach` failure (pre-M12 pattern); `device_code` hard-codes `Ok(2)` instead of `DEVICE_CUDA`; no mixed-protocol `Self`-method codegen unit test (proven only by ad-hoc emit and the direct-path tests).

## Summary

Every prior blocker is independently confirmed closed: the pass-1 mixed-protocol double-drop and the pass-3 attr-accessor regression both build end-to-end, with the fix proven structurally, by unit regression test, and by native builds of the exact reproducers. The runtime capsule/deleter lifecycle, no-copy/no-retry contract, ABI validation, affine ownership enforcement, stream/device semantics, and blocking PyTorch/TensorFlow evidence all held up under fresh independent scrutiny, and the full 674-fixture e2e suite passes at HEAD. The only new finding is a five-line clippy pedantic lint regression on new code — a real but mechanical hygiene nit that no blocking gate executes — plus small doc/artifact tidy-ups. No blocker or major correctness, contract, regression, or verification issue remains.

VERDICT: APPROVED

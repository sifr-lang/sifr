All evidence gathered and cross-verified. The merge profile's `sifr_runtime` lanes run only default features and `--features http` — never `--features python` — and no DLPack analogue of the `arrow-cpython311` lane exists, confirming the evidence-execution gap. I can now write the full review.

---

# Review: PR #2992 — M12 DLPack One-Shot Tensor Transfer (pass 1)

**Branch:** `codex/m12-dlpack-one-shot-transfer` (single commit `69170536d` vs `main`, 94 files, ~4,668 insertions)
**Scope reviewed:** complete diff against the M12 requirements in `plans/issues/active/ad-hoc-declaration-first-python-interop.md:1734-1797`, with three parallel deep-review passes (lowering/type-system, codegen call-paths, evidence/docs/demo) plus direct inspection of the runtime ABI and capsule lifecycle. All checks were read-only on the repo; verification builds used out-of-repo copies.

**What was executed to verify (not just read):** `cargo test -p sifr_runtime --features python dlpack` (18/18 pass), `cargo test -p sifr_codegen python_dlpack` (6/6), `cargo test -p sifr_lowering python_dlpack` (6/6), `runner/run.py --self-test` and `--dlpack-examples` (both torch and TensorFlow compiled binaries genuinely recompiled and re-run, stdout markers observed), the `m12_dlpack_demo` compiled and run successfully, ~30 ad-hoc positive/negative `.sifr` programs compiled against the branch binary, `check_hir_maintainability_guardrails.py` (PASS), and file-size checks (all touched hand-maintained files ≤ 899 lines).

## Blocker

**B1. Mixing an owned Arrow argument and an owned DLPack tensor argument in one declaration emits non-compiling Rust (E0382) — reproduced end-to-end.**
`crates/sifr_codegen/src/python_zero_copy_arguments.rs:67-81` calls both `python_arrow_codegen::append_argument_reconciliation` (`python_arrow_codegen.rs:77-78`) and `python_dlpack_codegen::append_argument_reconciliation` (`python_dlpack_codegen.rs:67-68`), and **each** unconditionally emits `::std::mem::drop(__sifr_python_args); ::std::mem::drop(__sifr_python_kwargs);`. Nothing in `direct_validation.rs` forbids the combination, so this lowering-accepted program:

```python
@python(torch.add)
def consume_mixed(own arr: python.ArrowArray, own tensor: python.DlpackTensor[int64]) -> Result[None, PythonError]: ...
```

passes type-check, emits the double drop (verified in `emit` output, lines 1756/1760 of the generated file), and fails the build with `error[E0382]: use of moved value: __sifr_python_args` surfaced to the user as `SIFR-BUILD-0005` wrapping raw rustc errors. This directly violates the "if it compiles, it works" guarantee and the Wave 3 owned-consumer-transfer contract. The same defect exists on the `Self`-receiver method path. Fix direction: hoist the two `drop_value` calls into `ArgumentGuards::append_reconciliation` once and remove them from both protocol-specific reconcilers; add a mixed-protocol codegen test.

## Major

**M1. The runtime DLPack protocol tests cited as negative/cleanup evidence are not executed by any delivery profile.**
`fixtures/torch_dlpack/dlpack_declaration_evidence.json:15-45` claims positive/negative/cleanup coverage owned by `crates/sifr_runtime/src/python/dlpack_ops/declaration_tests.rs` and `abi.rs`, and `declaration_capabilities.json` marks the capability passing — but every profile's runtime lane runs `cargo test -p sifr_runtime` with default features (`verification/profiles/merge.json:55`; only `http` is ever feature-enabled at line 56), which compiles out the `python` feature: `cargo test -p sifr_runtime dlpack` runs **0 tests**. M11 solved exactly this with the `arrow-cpython311` lane (`runner/cpython311_arrow.py` runs `cargo test -p sifr_runtime --features python` against an explicit `python::arrow_ops::…` test list); no `dlpack_ops` analogue exists, and `runner/dlpack_evidence.py` only checks that the owner files *exist*. Consequence: a regression in versioned-capsule validation, copied-flag rejection, no-retry, or exact-once deleter accounting passes every blocking gate, because the examples lane only exercises well-formed torch/TF capsules. The plan's end-state decision "Capability claims require executable positive, negative, cleanup, and compiled-binary evidence" is therefore claimed but not gate-executable. (The 18 tests do pass when run manually.) Fix: add a `dlpack-cpython311`-style blocking lane mirroring the Arrow one.

**M2. The Wave 1 "static ownership failure matrices" for DLPack are not pinned by tests.**
`crates/sifr_lowering/src/lower/python_dlpack_contract_tests.rs` (136 lines) covers decorator grammar, stream-policy, and declaration-shape ownership (`own` required on consumers, `own` rejected on stream params) — but contains zero use-after-move, double-consume, release-then-consume, `consume(t, t)`, loop-repeat-consume, collection/closure-escape, or bridge/`Self` acquisition lowering tests, versus ~19 equivalents in `python_arrow_contract_tests.rs`. The behavior itself is correct today — the lowering review pass dynamically compiled ~20 such negative programs and every one fails closed with real diagnostics (`SIFR-OWN-0004`, `SIFR-PYZC-0001`, `SIFR-PYCONV-0001`) via the shared affine machinery — but the milestone explicitly lists "double consume … use-after-move fixtures" as validation, and the evidence JSON's `negative.covers: ["…","ownership",…]` names this test file as owner. A refactor that drops the DLPack variants from `contains_affine_resource` recursion or breaks the release-receiver move marking would regress with essentially no test standing in its way.

## Minor

**m1. Misleading diagnostic for `stream=` on `@python.dlpack.stream`.** `crates/sifr_lowering/src/lower/python_interop/dlpack.rs:182-192`: the first `stream=` argument on a stream declaration falls through the guarded arm into the duplicate-argument arm and reports ``duplicate DLPack argument `stream` `` (reproduced). Rejection is correct; message is wrong.

**m2. Bare `python.DlpackTensor` (unsubscripted) reports "unknown Python affine resource type".** Via `python_arrow_annotations.rs:14-21` fall-through from `annotations_and_function_lowering.rs:52-56`. The type exists and needs `[T]`; the diagnostic gives no hint. (Parity with pre-existing `python.Buffer` behavior, but worth fixing while activating the type.)

**m3. Theoretical double-release on `finalize`'s early error paths.** `crates/sifr_runtime/src/python/dlpack_ops/argument.rs:92-97`: if `clone_handle`/`cast`/`capsule_name` fail, the entry drops with `released == false` (deleter runs) while the fresh consumer capsule still carries its original name, so its destructor (`argument.rs:135-148`) would release the same managed tensor again. Only reachable via internal invariant breakage (the capsule is compiler-created), not user input — the rename-failure path at `argument.rs:102-119` handles this correctly and the same discipline should apply here.

**m4. TensorFlow became an unconditional locked dependency of the blocking python_interop area.** `verification/areas/python_interop/pyproject.toml` + `packages/data.toml` removed the `host-dependent`/`skip-reason` escape hatch; `uv.lock` carries tensorflow 2.21.0 wheels for cp311–cp313 only while `requires-python = ">=3.11"` is unbounded, so every blocking lane (including create-pr) fails outright on Python ≥ 3.14 hosts.

**m5. Codegen test quality is smoke-level.** `python_dlpack_codegen_tests.rs`: substring + `syn::parse_file` checks only; `assert!(rust.contains("None"))` (line 33) is vacuous; no mixed-protocol case (which is why B1 was invisible), no multi-tensor case, no ordering assertion that `finish()`/`reconcile_dlpack_argument` are emitted after the call and args/kwargs drops.

**m6. Evidence artifact placement asymmetry.** The single evidence JSON covering both frameworks lives at `fixtures/torch_dlpack/dlpack_declaration_evidence.json` (its `live` rows include `tensorflow-declaration`); `fixtures/tensorflow_dlpack/` has only the contract JSON. Content is validated against the executable case registry, but the placement misleads.

## Advisory (non-blocking)

- **a1.** `python_dlpack_codegen.rs:118` — `acquire_from_foreign` silently returns the raw producer expression for a non-tensor/stream ok type. Unreachable today (lowering enforces the return shape at `dlpack.rs:112-126`), but this is a wrong-code fallback where `None`/unreachable would fail loudly.
- **a2.** `abi.rs:124-142` — the versioned capsule's `READ_ONLY` flag (bit 0) is neither validated nor propagated. Harmless today because Sifr never reads/writes tensor payloads, only transfers them; worth recording as a deliberate decision.
- **a3.** `dlpack_ops.rs:332-351` — if `reserve_handle` or `update_object_count(+1)` fails in `store_tensor`, the dropped `TrackedDlpackTensor` still decrements the live-object count it never incremented (diagnostics skew only; both failure paths are effectively unreachable).
- **a4.** `dlpack_ops.rs:224` — CUDA stream token `0` is accepted (the array-API spec disallows it as ambiguous), and the `< 0` rejection would refuse a valid `cudaStream_t` pointer above `i64::MAX`; both theoretical on current hardware, CUDA is not exercised in evidence.
- **a5.** The DLPack element set (`crates/sifr_stdlib/src/python/dlpack.rs:26-35`) has no `float32`/`float16`/`bfloat16` because Sifr has no sized float types — so the most common ML dtype can only fail at runtime dtype validation (which it does, cleanly). Docs don't state the supported element set; worth documenting.
- **a6.** `crates/sifr_codegen/src/lib_runtime_needs.rs` sits at 899 lines — one under the cap; the next touch must refactor.
- **a7.** Plan doc: the M12 wave checkboxes were added but left unticked (consistent with tick-at-merge practice; milestone-review item is honestly open). `plans/reviews/active/m12-dlpack-full-review-pass1.md` exists untracked and empty (0 bytes) — placeholder only, not in the commit. A stray `__pycache__/…pyc` sits under `fixtures/tensorflow_dlpack/python_bridges/`.
- **a8.** The demo is substantive (pointer-identity assertion across the transfer) but not wired to any verification lane, and its `uv.lock` pulls the full CUDA toolkit on Linux (multi-GB) — fine since it's opt-in.

## Verified sound (highlights)

- **DLPack ABI** (`abi.rs`): struct layouts match the spec exactly (legacy and versioned, `IS_COPIED` = bit 1), major-version gating, non-negative shape/stride/offset validation, overflow-checked element counts, null-data allowed only for empty tensors.
- **No-retry guarantee**: single `__dlpack__(stream=…, max_version=(1,0), copy=False)` call; legacy capsule names accepted only because the producer accepted the versioned signature; pinned by a call-counting test (`declaration_tests.rs:89`).
- **Exact-once deleter / one-shot transfer**: producer capsule renamed to its used sentinel before Sifr assumes deleter responsibility; fresh consumer capsule per owned transfer; move committed at `prepare` time (store entry removed) so post-consumption failure keeps the tensor moved; `finalize` either honors the consumer's used-rename or renames-then-releases exactly once, with the rename-failure path correctly relinquishing to the capsule destructor. Releases run under `attach` (GIL held). Verified by release-counter tests including manual post-transfer deleter invocation.
- **Type-system/ownership wiring is complete**: every `PythonArrow`/`PythonBuffer` handling site (affinity, clone/equality denial, union identity/canonicalization, type-var substitution/collection, sendability, IPC rejection, print formatting) has a DLPack counterpart; ~30 compiled negative scenarios all fail closed; `DlpackTensor`/`DlpackStream` are statically non-Clone/non-Send (`static_assertions`).
- **Codegen call-path parity with Arrow is complete** across direct, `Self`-receiver, collection, comprehension, and callable paths; no data-dependent panics in emitted code; async consumers statically rejected.
- **Evidence is executable and host-independent**: the `dlpack-examples` lane recompiles and re-runs both compiled binaries; pointer stability is recomputed at run time inside the binaries (no recorded addresses/paths); the lane is blocking in all four profiles; the runner refactor (`cli_args.py`, `run.py`) is purely additive — no Arrow/buffer/async lane dropped.
- **Docs** (`docs/python-interop.mdx`, `error-codes.mdx`) match the implemented grammar and runtime semantics exactly, including the committed-move and no-retry wording.

## Summary

The core protocol engineering is genuinely strong — the capsule lifecycle, ABI validation, and no-retry/exact-once contracts are correct and well-tested at the unit level, and the compiled PyTorch/TensorFlow evidence is real and reproducible. But one valid user program makes the compiler emit non-compiling Rust (B1, reproduced end-to-end), the milestone's runtime negative/cleanup evidence is claimed against tests no blocking gate executes (M1), and the mandated static-ownership failure matrix is not pinned (M2). B1 requires a code fix plus a regression test; M1/M2 require gate and test additions before M12 can close.

VERDICT: CHANGES REQUIRED

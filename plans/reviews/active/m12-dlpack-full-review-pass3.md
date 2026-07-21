Confirmed end-to-end: `SIFR-BUILD-0005` wrapping `E0425: cannot find value __sifr_python_args` — a valid, type-checking user program produces non-compiling Rust on this branch. This is a new regression introduced by the remediation commit itself. I have everything needed for the final review.

---

# Final Review: PR #2992 — M12 DLPack One-Shot Tensor Transfer (pass 4, post-remediation)

**Branch:** `codex/m12-dlpack-one-shot-transfer` (`69170536d` + remediation `2364c83e4`, 97 files vs `main`)
**Scope:** complete `main...HEAD` diff against the M12 requirements (`plans/issues/active/ad-hoc-declaration-first-python-interop.md:1734-1795`), independent re-verification of every pass-1 finding, and a fresh sweep via three deep passes (codegen call paths; runtime ABI/capsule lifecycle; evidence/lanes/docs) plus direct end-to-end compiles against the branch compiler using the demo's materialized torch venv (copied to `/tmp`, repo untouched).

**Executed evidence (not just read):** mixed Arrow+DLPack owned-consumer declaration → full native build + run (exit 0, exactly one `drop(__sifr_python_args)`/`(__sifr_python_kwargs)` pair, both reconcilers present, ordered after the call); the committed `m12_dlpack_demo` compiled and ran ("zero-copy one-shot transfer succeeded", pointer identity held); double-consume → `SIFR-OWN-0001`; both remediated diagnostics reproduced with correct messages; `cargo test -p sifr_runtime --features python python::dlpack_ops` → 18/18; `sifr_codegen` dlpack 8/8, arrow 9/9, buffer 10/10, direct 10/10, callback 11/11; `uv lock --check` clean for both area projects; HIR guardrails PASS; and the new blocker reproduction below.

## Blocker

**B1-NEW. The remediation's hoisted drops emit non-compiling Rust (E0425) for `@python.attr` methods on opaque classes that hold a retained callback owner — reproduced end-to-end.**
Commit `2364c83e4` fixed the original double-drop by hoisting `drop(__sifr_python_args)` / `drop(__sifr_python_kwargs)` to the top of `ArgumentGuards::append_reconciliation` (`crates/sifr_codegen/src/python_zero_copy_arguments.rs:69-70`) — but left it **unguarded by `is_empty()`**, and both call sites invoke it whenever callbacks *or* zero-copy guards *or* (method path) `owner_retained_errors` are non-empty (`crates/sifr_codegen/src/python_interop_direct.rs:295/329` and `:783/816`). Attribute-kind methods never declare those locals (`python_interop_direct.rs:467-470` skips the `let`s for `PythonInteropDecoratorKind::Attribute`). So an `@python.attr` accessor on a class whose factory attached a `lifetime=result` (or `Self`) callback reaches line 816 with a non-empty `owner_retained_errors` and emits drops of undeclared identifiers.

Reproduction (program type-checks; entirely merged-M9 surface, no DLPack needed): opaque `Manager` with `@python.attr(Self.value)`, produced by a factory carrying `@python.callback(handler, lifetime=result, dispatch=foreign, concurrency=serial)`. `sifr build` fails:

```
error[SIFR-BUILD-0005]: cargo build failed:
error[E0425]: cannot find value `__sifr_python_args` in this scope
1793 |         ::std::mem::drop(__sifr_python_args);
error[E0425]: cannot find value `__sifr_python_kwargs` in this scope
```

(repro kept at `/tmp/m12review/demo_copy/src/main.sifr`; independent duplicate at `/tmp/repro_attr_drop.sifr` with emitted Rust at `/tmp/repro_emit.rs`). This is strictly a regression of `2364c83e4`: at `69170536d` the drops lived inside the protocol reconcilers, which only run with non-empty guards, so this program compiled. It's the same defect class the original blocker was about — a valid program breaking the "if it compiles, it works" guarantee — and it regresses *existing merged M9 functionality*, which is also why the create-PR gate (131/131 e2e) missed it: no fixture combines an attr accessor with a retained-callback owner.

**Fix direction:** early-return from `append_reconciliation` when `self.is_empty()` (attribute methods can never have zero-copy guards, so this fully closes it and restores pre-remediation behavior for callback-only/owner-errors-only paths), plus a regression test — a codegen unit test asserting no drop statements when guards are empty, and ideally a permanent e2e fixture for attr-accessor-on-retained-callback-owner.

## Pass-1 findings: all independently verified remediated

- **B1 (mixed Arrow+DLPack double drop)** — fixed on every guarded path including `Self`-receiver methods (sole reconciler caller is `ArgumentGuards`; consuming-receiver path can't carry guards). Verified by end-to-end native build + run of the exact pass-1 repro, generated-code drop counts, the new `mixed_…` and `multiple_dlpack_…` tests (with real ordering assertions, closing pass-1 m5), and a mixed-on-`Self`-method emit check. Error paths still finalize guards exactly once before `?` applies.
- **M1 (runtime tests not gate-executed)** — the new `dlpack-cpython311` adapter lane is blocking in all four profiles, mirrors `arrow-cpython311` exactly, provisions uv-managed CPython 3.11 with no silent-skip path, and asserts the **exact** 18-test set (drift in either direction fails the lane; self-test validates the parser). The 18 tests pass. `dlpack_evidence.py` now also enforces manifest/profile ownership of both dlpack suites.
- **M2 (ownership matrices unpinned)** — `python_dlpack_contract_tests.rs` (now 216 lines) adds duplicate-return, double-consume, consume-of-consumed-argument, release-then-use, repeat-owned-call, constructor, callable, and loop-move matrices asserting exact codes (`OWN_USE_AFTER_MOVE`, `OWN_MOVED_ACROSS_LOOP`, `PYZC_INVALID_DECLARATION`). I additionally reproduced double-consume live (`SIFR-OWN-0001`).
- **m1/m2 (diagnostics)** — both verified live: ``@python.dlpack.stream … stream=none`` → "does not accept a `stream` argument"; bare `python.DlpackTensor` → "requires exactly 1 element type".
- **m3 (finalize double-release)** — `relinquish_to_capsule` (`argument.rs:136-150`) is sound: it marks the entry released before dropping, the consumer capsule destructor (which skips used-sentinel names) becomes the sole deleter owner, and pending releases drain under the GIL. Every finalize branch now has exactly one deleter owner; lock discipline (no store lock while Python runs) holds. The `counted` flag closes the object-count skew (single construction site, set only after successful increment).
- **m4 (Python ≥3.14 lock break)** — `requires-python = ">=3.11,<3.14"` pinned in `pyproject.toml` and `uv.lock`; `uv lock --check` passes; tensorflow/torch cp311 wheels present for macOS arm64 and Linux.
- **m6 (evidence placement)** — moved to `fixtures/dlpack_declaration_evidence.json`; no stale references remain outside the historical pass-1 record.
- **a2/a3/a4/a5** — READ_ONLY preservation and the closed element set are now documented (`docs/python-interop.mdx:407-418`); CUDA stream token 0 is rejected in normalization before acquisition (with a test); the count-skew is fixed.

## Verified sound (this pass)

- **Runtime lifecycle:** producer capsule renamed to the used sentinel before Sifr assumes deleter responsibility; single `__dlpack__(stream, max_version=(1,0), copy=False)` call with no legacy retry (call-count pinned); move committed at prepare time so post-consumption failure keeps the tensor moved; ABI validation (versioned/legacy names, `IS_COPIED`, major gating before reading 1.x-only fields, dtype/lanes, shape/stride/offset, null-data-only-when-empty) matches the spec; shape/strides copied into owned Vecs so no reads outlive the producer deleter.
- **Type-system/ownership parity is complete:** affinity, clone/equality/print denial, union identity + canonical ordering (the sort-key change also fixes a pre-existing buffer/arrow collision, with a test), type-var substitution/collection, sendability, share-safety, IPC rejection, imported-class identity, error refs, bridge contracts — every `PythonBuffer`/`PythonArrow` site has a DLPack counterpart. `acquire_from_foreign` now returns `Option` so the non-tensor fallback can't silently splice wrong code.
- **Lowering:** closed decorator grammar (explicit device atom set, mandatory stream policy, non-CPU requires `parameter(name)`, keyword-only immutable-borrow stream param filtered from call shapes, `Self` receiver rules), consumers must be plain `own`, not omittable, rejected on async declarations; element set closed to FixedInt/Float/Bool.
- **Evidence/docs:** examples lane genuinely recompiles and runs both torch and TF binaries with pointer-identity + zero-residual-resource assertions computed at run time; TF bridge enforces the full versioned call shape; capability JSON, both architecture docs, error-codes, and README match the implementation; no committed `.pyc`/`__pycache__`; all touched hand-maintained files ≤ 899 lines; HIR guardrails pass.

## Minor

- **plan-doc waves:** M12 Waves 1–4 in `plans/issues/active/ad-hoc-declaration-first-python-interop.md:1740-1749` are implemented but unticked (M11 practice ticked waves in the delivering PR; the milestone-review row correctly stays open).
- **CPU stream forwarding:** `dlpack_stream(&obj, "cpu")` accepts CPU stream metadata and `acquire_dlpack_tensor` then passes an integer `stream=` to `__dlpack__` on a CPU device (`dlpack_ops.rs:111-116`), where array-API semantics require `stream=None`; conforming producers may raise. Reachable only by explicitly acquiring a `"cpu"` stream; suggest rejecting CPU stream metadata or mapping it to `None`.

## Advisory

- `crates/sifr_codegen/src/lib_runtime_needs.rs` is at 899/900 lines — next touch must refactor.
- Add a mixed Arrow+DLPack `Self`-method codegen unit test (currently proven only by ad-hoc emit).
- Pre-existing theoretical corner: `PythonDlpackArgument::Drop` ignores `attach` failure (`argument.rs:39`); a shutdown-then-reinit with an in-flight argument could double-release via the drained pending queue. Predates M12.
- `dlpack_ops.rs:414` hard-codes `Ok(2)` instead of `DEVICE_CUDA`; a poisoned-handle relinquish path leaks (safe direction) rather than releasing.
- Untracked local artifacts: `plans/reviews/active/m12-dlpack-full-review-pass2.md` (invalid attempt) and `pass3.md` (empty) should be deleted or replaced by this pass's record before merge; `third_party/ruff` shows local submodule-content dirt at the pinned commit (not in the diff).

## Summary

The M12 protocol work itself is in excellent shape: every pass-1 finding is genuinely remediated and independently re-verified, the capsule/deleter lifecycle and no-retry/no-copy contracts are sound and now gate-executed, and the compiled PyTorch/TensorFlow and demo evidence is real. But the remediation introduced a new instance of the exact defect class it was fixing: the unguarded hoisted drops make `@python.attr` accessors on retained-callback owners — merged M9 surface — emit non-compiling Rust, reproduced end-to-end as `SIFR-BUILD-0005`/`E0425`. That is a one-line-plus-tests fix (`if self.is_empty() { return; }` in `ArgumentGuards::append_reconciliation`), after which this PR should be re-gated and is otherwise ready.

VERDICT: CHANGES REQUIRED

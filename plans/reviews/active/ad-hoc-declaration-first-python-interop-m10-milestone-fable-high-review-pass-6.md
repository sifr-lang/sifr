The review surface is complete. All the evidence is in — composing the pass-6 review.

---

# M10 Milestone Review — Pass 6 (branch `codex/m10-milestone-closure` + working tree, PR #2990, Fable high, read-only)

**Summary: every pass-2 through pass-5 blocker is verifiably closed, including all four exact pass-5 probes, and every validation claim independently reproduces — but the class-method `mut` parameter remediation stops one argument-shape short, exactly as the pass-4 fix did before it. A field-access argument passed to a class-method `mut` parameter is silently cloned at the call site, so the mutation is discarded: check passes, build passes, wrong answer at runtime, where pre-M10 failed closed. The milestone is not ready to close.**

**Methodology.** I read the milestone issue, prior review passes 2–5, the complete working-tree remediation diff (31 files, ~551 insertions, 2 new source files, 9 new fixtures), and the surrounding implementation (`field_and_stdlib_rewrites.rs`, `class_body_lowering.rs`, `class_method_emitter.rs`, `queries_impl.rs`, `class_field_emitter.rs`, `error_contracts.rs`, `try_error_carrier.rs`, `class_upcasts.rs`, `union_identity.rs`, `source_names.rs`, `python_interop_common.rs`, lowering dispatch/diagnostics, driver tests). The committed `origin/main...HEAD` surface was whole-diff-reviewed in passes 2–5; I re-inspected every area this remediation touches. I rebuilt release and debug compilers from the exact working tree, reran every claimed suite and gate myself, and ran ~20 fresh behavioral probes in `/tmp/m10rev6` with the pre-M10 installed `~/.sifr/bin/sifr` as reference. No repository file was modified (`git status` is unchanged, 47 entries as at session start); the dirty `third_party/ruff` submodule was left untouched. The empty untracked `...pass-6.md` placeholder was not written to.

## Prior-blocker verification — all closed as claimed

**Pass-5 blocker (nested chains and class-method `mut` parameters) — fixed for every reported shape, adversarially rechecked:**

- **Nested `mut`-parameter chains**: `o.inner.items.append(7)` now prints 3 (`nested.sifr`); `extend`/`pop` through nested chains print 3, 2 (`nestedfn2.sifr`); a three-level chain `r.mid.leaf.items.append` prints 3; `insert`/`remove`/field-assign mixes, `sort()`, and nested `set.add` all preserve mutations. The fix is structurally right: `field_receiver_root_name` walks the chain to the base name, suppression arms on the root (`field_and_stdlib_rewrites.rs:33-46`), and the clone condition fires only on the direct-name hop, so exactly one suppression pairs with exactly one clone site (`pending_self_field_clone_suppression` asserted back to 0 in the new unit test).
- **Nested `self` chains**: `self.inner.items.append(7)` prints 3 (`selfnested.sifr`) — the pass-5 pre-existing MEDIUM is fixed too, via the root-name rewrite of `is_self_field_mutating_method_call` (`helpers_impl.rs:653-660`).
- **Class-method `mut` parameters**: lowering now retains the declared mutable convention (`class_body_lowering.rs:427-440`, with the prescribed unit test); `merge` emits `fn merge(&self, other: &mut Crate)` and the caller emits `let mut b; a.merge(&mut b)` — mutation survives (`methodparam.sifr`). `collect_mutated_vars` marks direct name arguments to `mut`-borrow method parameters (`queries_impl.rs:489-502`).
- **Immutable class-method parameters fail closed at check**: `other.items.append(1)` through a non-`mut` method parameter is rejected with SIFR-OWN-0005, with the prescribed lowering unit test.
- **Value semantics preserved**: a field read from a `mut` parameter into a local still clones (copy mutates to 3, original stays 2); a mutating call whose *argument* reads the same chain (`o.inner.items.append(len(o.inner.items))`) is correct.
- **Adjacent shapes fail closed, not silently**: index-rooted receivers (`o.rows[0].items.append`) are rejected at check (unproven index → `None | list[int]`); nested subscript assignment is rejected at check (SIFR-FLOW-0007); loop-iteration mutation and field-arguments-to-free-`mut`-functions fail closed at rustc on both this tree and pre-M10.

**Passes 2–4 blockers — still closed:** same-basename `CsvError | CfgError` union channels dispatch to the correct handler natively; builtin (`ValueError | KeyError`) and mixed (`CsvError | ValueError`) channels build and dispatch; imported same-basename `Error` is not a catch-all (rejected with SIFR-RESULT-0005 when coverage is missing); `handler_is_catch_all` keys on `is_builtin_error_base` identity across all three consumer sites (`try_error_carrier.rs`, `try_handlers.rs`, `try_tuple_flow.rs`); the basename tail-match upcast fallback is deleted with the prescribed negative test; `_sifr.python.PythonError` is off the global-identity list with the `assert_ne!` guard; `PythonError` bridge-field injection is gated on the contract shape plus actual interop declarations at all sites including the constructor paths split into `class_field_emitter.rs`; runtime-python feature detection covers both namespaces; union identity keys are order-insensitive.

## Validation claims — all independently confirmed

`sifr_codegen` 865, `sifr_lowering` 781 (+1 ignored), `sifr_type_system` 114, `sifr_driver` 358 (+33 ignored) all pass; the ignored `test_build_package_keeps_runtime_and_source_python_error_identities_distinct` passes explicitly (18.1s), and all 33 ignored driver tests pass; all nine new fixtures build and run natively (exit 0, assert-based), including `class_method_mut_borrowed_parameter_field_mutation` and `mut_borrowed_parameter_nested_field_mutation`; the four exact pass-5 probes (`nested`, `nestedfn2`, `methodparam`, `selfnested`) produce correct results; `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, both maintainability guardrails, and the file-size guardrail (PASS, 2716 files, 900-line limit) all pass.

## Blocking finding

### 1. HIGH — Field-access arguments to class-method `mut` parameters are silently cloned at the call site; the mutation is discarded (new regression from the pass-5 remediation)

The method-call argument emitter wraps the *already-lowered* argument in `&mut`, but lowering a field access from `self` or a borrowed parameter inserts the read-clone from the pass-3/4/5 machinery, and no suppression exists for `mut`-convention argument positions. Only direct-name arguments (`a.merge(b)`) take the correct path. Three verified silent-loss repros (working-tree compiler; each is ordinary short code):

- **Through `&self`** — `helper.merge(self.stock)` inside `def refill(self)`: check passes, build passes, prints **1** instead of 2. Emitted Rust: `helper.merge(&mut self.stock.clone())`. Pre-M10 fails closed (SIFR-BUILD-0005).
- **Through an explicit `mut self`** — same program with `def refill(mut self)`: **still prints 1**, and the receiver is emitted as `&self` — the declared `mut self` is not honored, because `collect_mutated_vars` only marks `HirExpr::Name` arguments (`queries_impl.rs:493-497`), so a `FieldAccess` argument never propagates mutability to the enclosing receiver. The mutation intent here is fully explicit and still silently lost.
- **Through a `mut` free-function parameter** — `helper.merge(d.stock)` inside `def relay(mut d: Depot)`: prints **1**; emitted `helper.merge(&mut d.stock.clone())`.

The free-function path proves the correct behavior is already achievable: `mutate(self.stock)` with a free `mut` parameter emits `fn refill(&mut self) { mutate(&mut self.stock); }` and prints the correct 2. The defect is specific to the method-call argument emission path activated by this working tree.

**Why blocking:** identical failure class to the pass-4 and pass-5 blockers — check passes, build passes, silent data loss — the most severe class this project defines ("if it compiles, it works"). Pre-M10 every one of these programs failed closed. It ships in the exact working-tree diff under review, in the feature that diff introduces.

**Remediation direction:** in the method-call argument emitter, arguments bound to `mut`-borrow parameter conventions must be lowered as mutable places (no read-clone) — mirroring the free-function call path — and `collect_mutated_vars` must mark the *root* of a field-access argument (reusing its own `expression_root_name`) so callers and receivers gain `mut`/`&mut self` as the free-function path already does. Where the root is not mutably reachable (immutably-borrowed parameter or `&self`), reject at check with SIFR-OWN-0005 rather than relying on rustc. Add native fixtures for: method `mut` arg from a `self` field (both `self` and `mut self` receivers), from a `mut`-parameter field, and a negative check fixture for an immutably-rooted argument. Today no fixture or unit test covers any non-Name argument to a method `mut` parameter, which is again why the fully green suite coexists with the defect.

## Non-blocking findings (fix-forward)

2. **MEDIUM (adjacent to blocker 1; fold into the same fix)** — passing an immutably-borrowed parameter as a direct-name `mut` method argument (`a.merge(c)` where `c` is a non-`mut` parameter of the caller) passes check and fails the native build (E0596). Fail-closed today, but it is the check-side half of the same incomplete caller-mutability analysis.
3. **LOW (pre-existing; both trees fail closed)** — free-function `mut` arguments that are field accesses of a local (`mutate(b.crate_field)` from `main`) and mutation through loop iteration variables (`for row in o.rows: row.items.append`) pass check and fail at rustc (E0596). Same on pre-M10; worth folding into the mutability-analysis follow-up.
4. **LOW** — `crates/sifr_runtime/src/python.rs` remains at exactly the 900-line cap (carried from passes 3–5, still unaddressed).
5. **LOW** — same-basename diagnostic qualification remains partial (carried from passes 4–5): `format_type_name` now qualifies duplicates within one rendered union, but type-checker-formatted paths still print bare basenames.
6. **LOW** — `is_builtin_error_base` remains a pure shape match (carried pass-5 finding 3); an identity-based tag would be more robust.

## Verdict

The milestone's declared remit — nominal identity, union error carriers and aliases, catch-all exact identity, `PythonError` runtime/source collision behavior, check/build parity in the error/identity domain, feature gating, module decomposition — is genuinely in good shape, and every blocker from passes 2–5 is closed with fixtures, unit tests, and reproducible probes. But for the second consecutive pass, the mutation-analysis remediation fixes the reported shapes while leaving an adjacent, user-reachable shape of the same silent-data-loss defect: field-access arguments to the newly activated class-method `mut` parameters clone at the call site and discard the mutation, converting pre-M10 fail-closed builds into wrong answers — including under an explicit `mut self`. That must be fixed with the missing argument-shape regressions, and the authoritative gate rerun, before closure.

VERDICT: NOT SATISFIED

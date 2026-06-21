Reviewing the implementation across all 18 changed files. Here is my findings report.

## Phase 39 milestone_39_9 review — round 1

**Verdict:** No blockers. Contract-only zero-copy/view surface is correctly implemented and scoped. Validation, diagnostics, fixtures, and docs are consistent. The findings below are minor — clarity, dead code, and test-coverage gaps that the team can fold into milestone_39_10 or a follow-up cleanup.

---

### Low — duplicated downstream diagnostic when `@rust.view(...)` key parse fails
`crates/sifr_driver/src/build/rust_interop/zero_copy_validation.rs:84-104`

When `parse_view_contract` fails (e.g. the legacy `mutable=False` key), the loop pushes "unsupported `@rust.view(...)` key", leaves `view = None`, and then the post-loop branch at line 106–116 pushes a second "requires a paired `@rust.view(...)` declaration" diagnostic for the zero-copy decoration even though the user clearly intended to declare a view. The `package_rust_interop_rejects_legacy_mutable_bool_key` test passes because it only checks `diagnostics[0]`, but the second diagnostic is misleading UX. Consider tracking "the user attempted to declare a view but it failed to parse" so the paired-view diagnostic is suppressed in that case.

### Low — dead `view.is_empty()` defensive check
`crates/sifr_driver/src/build/rust_interop/zero_copy_validation.rs:159-169`

`parse_zero_copy_contract` only accepts `view=` as a `RustInteropValue::TargetPath`, and `RustTargetPath::dotted()` produces a non-empty string whenever the parser produced at least one segment. The `if zero_copy.view.is_empty()` branch is unreachable. Either delete it, or document the invariant. (Not a blocker, but it adds noise to an already complex group validator.)

### Low — unused `let _ = (view.send, view.sync);` discard
`crates/sifr_driver/src/build/rust_interop/zero_copy_validation.rs:213`

`view.send`/`view.sync` are consumed by `view_probe_obligations` via an independent parse, so this discard does nothing — it predates a refactor or is a leftover from when the fields were going to be checked here. Remove it.

### Low — fixture matrix advertises crates not exercised by tests
`verification/areas/rust_interop/data/rust_interop_fixture_matrix.json:217-234`

`zero_copy_bytes` and `zero_copy_view_matrix` both keep `required_crates: ["bytes" | "memmap2" | "bytemuck" | "zerocopy"]` while flipping `execution_kind` to `contract-only` and status to `passing`. The unit tests use a synthetic `RustBridgeSignatureContract`, not the listed crates. The two new README files do call out that runtime-observed crate certification is still staged, so this is internally consistent — but a casual reader of the matrix alone may infer those crates have positive evidence. Consider either dropping `required_crates` until the runtime-observed fixture lands, or adding a `notes:`/`coverage_scope:` field to clarify that `passing` here means contract-only.

### Low — diagnostic-stability follow-up: cascade from `@rust.view` parse + paired check
Same root cause as finding 1, but stated for diagnostic-stability concern: SIFR-RUST-ZC-0001 currently can fire twice for the same source span on a single bad `@rust.view(...)`. If any downstream tooling dedupes by `(code, span)` the second occurrence will be lost; if it doesn't, the user sees two messages for one problem. Worth a deliberate decision before the SIFR-RUST-ZC family gets wider adoption.

### Low — missing positive coverage for `lifetime=static`
`crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs`

Every positive/negative case uses `lifetime=owner` (or `lifetime=call`). The milestone explicitly permits `lifetime=static` (and the async-suspension path *only* accepts static). A positive test such as "async + `lifetime=static` view is accepted" would lock the only currently-allowed async-view shape and prevent future regressions. Similarly, no test covers:
- missing required keys on `@rust.zero_copy(...)` (`owner=`, `view=`),
- missing required keys on `@rust.view(...)` (`lifetime=`, `mutability=`, `send=`, `sync=`),
- duplicate `@rust.zero_copy(...)` / `@rust.view(...)` on a single function,
- a bare `@rust.view(...)` *without* `@rust.zero_copy(...)` (which the implementation accepts).

These are easy `.replace(...)`-style additions to the existing harness. Not blocking but recommended before the milestone is signed off.

### Low — `opaque_probe_obligations` special-cases `View` kind
`crates/sifr_driver/src/build/rust_interop/opaque_validation.rs:92-94`

Reading "view kind returns (false,false)" inside the *opaque* validation module is surprising. The actual View send/sync derivation lives in `zero_copy_validation::view_probe_obligations`, then gets `|=`-merged at `rust_interop.rs:563-567`. Consider either moving the early-return into `push_probe` so the dispatching site reads as a single switch over kind, or renaming `opaque_probe_obligations` to something kind-neutral. This is purely a maintainability nit.

### Low — file-size headroom on `rust_interop.rs`
892 / 900 lines after this milestone. The new validation correctly lives in `rust_interop/zero_copy_validation.rs`; the parent file is fine today, but the next validator added inline will trip the guardrail. Worth keeping in mind for milestone_39_10/11.

### Low — behavior change: implicit `abi_requirements.view` no longer becomes `requires_sync`
`crates/sifr_driver/src/build/rust_interop/opaque_validation.rs:92-94` and `rust_interop.rs:563-567`

Prior to this milestone a `View`-kind declaration would propagate `abi_requirements.async_boundary` → `requires_send` and `abi_requirements.view` → `requires_sync`. After this milestone the View probe's send/sync come solely from the explicit `@rust.view(...)` contract. The new validator requires `send=` and `sync=` to be declared, so users cannot accidentally drop the obligation — the change is intentional and consistent — but it's worth a one-line note in `internal_docs/rust_interop_architecture.md` to record the migration, since other contributors will compare git history and wonder why the implicit signal disappeared.

---

### Things explicitly verified as correct

- `SIFR-RUST-ZC-0001` is wired into `DiagnosticCode`, `ACTIVE_DIAGNOSTIC_CODES`, the registry entry, the catalog, baseline-coverage deferral, generated `docs/errors/SIFR-RUST-ZC-0001.md`, and the public/internal diagnostic-codes tables. Family `RUST-ZC` was already reserved in the family table.
- `validate_zero_copy_contracts` runs before `resolve_declaration` and short-circuits on errors, so malformed contracts can't leak into probe planning, trust validation, or rustc probes.
- The View probe metadata is properly union-merged: `(view.send, view.sync)` from `view_probe_obligations` is OR'd into the opaque baseline, and the async-current-thread special case (`requires_send = false` for tokio_current_thread) still applies after the merge.
- The lowering test was updated to the new `@rust.view(owner=…, lifetime=…, mutability=…, send=…, sync=…)` shape (`rust_interop_tests.rs:34`), so the HIR-level fixture matches the driver-level contract.
- Milestone scope claims (paired view, lifetime=call rejection, async non-static rejection, owner mismatch, unknown owner, mutable-from-shared-borrow, copy_fallback, legacy mutable key, Send/Sync probe propagation) each have at least one negative test in `rust_interop_zero_copy_contract_tests.rs`.
- File-size guardrail and HIR maintainability guardrail are not violated; new module placement under `rust_interop/` follows the existing decomposition.
- `RustBridgeProbeKind::View`/`ZeroCopy` and `RustInteropDecoratorKind::View`/`ZeroCopy` are pre-existing — no new IR surface introduced here that would widen the public crate API.

No further review round is required for milestone_39_9 from a correctness, scope, diagnostic-stability, or panic-safety standpoint. The findings above can be folded into a follow-up cleanup PR or the next milestone.

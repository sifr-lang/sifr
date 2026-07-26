## Verification of pass-3 blockers against the live tree

All eight pass-3 blockers and the three smaller edits are materially resolved. Spot-verified in-tree:

| Pass-3 item | Plan response | Tree check |
|---|---|---|
| 1. `BindingId` / retained facts / `self` kind | §3 introduces `BindingId` in `scope.rs` from `define_binding`, retains facts after frame pop, `BindingKind::Receiver`, `field_identity` via `parent_class` walk; capture lands in Item 1 step 1 | `scope.rs:196 define_binding`, `BindingKind{Local,Parameter}` at `:11`, `definitions.rs:147 parent_class` — all as described |
| 2. Ordering vs. flow effects | §2 runs fixed point inside `lower_module_bodies` between classes and functions; effects derived from final HIR in `flow_graph/effects.rs`, inline `record_flow_effect` removed/narrowed | `module_body_lowering.rs:12-20` lowers classes then functions — split point exists; inline sites are `mutating_methods.rs:74,152`, `regular_calls.rs:416,437` |
| 3. Non-class receivers | §2.1 gives per-family rules, an explicit defined `SharedBorrow` default (not ICE), and `OwnedTemporary` for rvalues | consistent; `Helper().bump()` still accepted |
| 4. Untyped IR pass | §6 protected-local set threaded into `remove_unneeded_mutability_in_items` | `mutability_and_clone_rewrites.rs:63` signature change is feasible |
| 5. Protocol/type/operator emitters | anchors added; `body_contains_field_assign_codegen` deleted; fixture pinned to `own mut param: Proto` | anchors exist — but see blocker A |
| 6. Reachable `SIFR-OWN-0014` fixtures | index/slice/field-narrowed moved to the `SIFR-STDLIB-0001` boundary; registry fixture renamed to `unsupported_narrowed_optional_mutating_receiver.sifr` | `SIFR-OWN-0014` still free (0000–0013 taken) |
| 7. `FunctionType` `self` normalization | §1 removes `self` from `params` and kills `self_offset`/conditional-skip | `FunctionType` at `definitions.rs:301` is `Hash`/`Eq`; migration is mechanical |
| 8. Near-cap decomposition | all eight ≥842-line anchors have a named split | measured 869/882/896/882/867/866/842/881 — plan's numbers are exact |

Smaller edits also resolved: `own self`/`own mut self` → `SharedBorrow`/`MutableBorrow` and `own mut self` seeded; invariant lives in a lowering post-pass surfacing `SIFR-INTERNAL-0001` (public at `registry.rs:269`) plus a debug assertion; `SIFR-OWN-0002.mdx` regeneration named.

## Remaining material blockers

**A. No conformance rule between a protocol-declared and a class-inferred receiver convention.** §2 makes body-less protocol methods `SharedBorrow` unless they declare `mut self`, while the seed rules independently give an implementing class method `MutableBorrow` if it mutates a field. §5 then requires both the trait signature and the impl bridge to consume "the convention" without saying whose wins. Reproduced now (`emit`, protocol `def bump(self)`, `Helper.bump` mutates `self.items`):

```rust
pub trait Bumper { fn bump(&self); }
impl Bumper for Helper { fn bump(&self) { Helper::bump(self) } }   // Helper::bump is &mut self
```

Either resolution leaks a raw rustc error through `SIFR-BUILD-0005`: protocol-wins reproduces today's `E0308`, class-wins yields `E0053` signature mismatch. The implementer must choose a third option (diagnose non-conforming receiver mutability at check time, or propagate the protocol declaration as a hard `SharedBorrow` constraint that makes the class body's mutation an error) — that choice is not made, and no fixture in the matrix covers the mismatch, only the matching `mut self` case.

**B. Iteration / `with` / `except` / match-pattern bindings are accepted as mutable roots by the stated rule but have no emission mechanism.** §3 accepts "an owned local binding", and §3's root eligibility tests binding kind + convention. In-tree, `for`, `with`, `except`, and match-pattern targets all go through `Scope::define` (`control_flow.rs:402,828,843`, `statement_dispatch.rs:446,512,619`, `patterns_and_assignments.rs:252,568,598`) and are therefore `BindingKind::Local`, indistinguishable from an owned local. Reproduced now:

```rust
fn run(&self) { for h in self.helpers.clone().iter().cloned() { h.bump(); } }
// error[E0596] leaked via SIFR-BUILD-0005
```

`for h in self.helpers: h.bump()` is the same silent-mutation-loss family the issue exists to close. Under the plan the place emitter accepts `h` and emits `h.bump()` with no clone, but nothing makes the iterator mutable-borrowing, so the `E0596` leak survives Item 2. The plan needs an explicit decision — classify iteration/with/except/pattern element bindings as a distinct non-root kind rejected with `SIFR-OWN-0014`/`0005`, or specify mutable iteration — plus a matrix row. Its non-goals exclude mutable indexing and slicing but say nothing about iteration elements.

## Optional implementation details (not blocking)

- The anchor list names 4 of the 10 files that touch `pending_self_field_clone_suppression`; `operator_rewrites.rs`, `print_calls.rs`, `subscript_augassign_delete.rs`, `string_char_cache.rs`, `lib_emitter_state.rs`, and `expr_render_helpers/tests.rs` are omitted. The §5 "every arm/disarm site" rule plus the repo-search acceptance criterion still cover them.
- `SIFR-INTERNAL-0001` is registered as "Unclassified compiler panic after a panic boundary." owned by `sifr_driver::diagnostics`; reusing it from a lowering post-pass makes the registry `owner` field inaccurate, though the driver already uses it as a general internal-error code.
- The mandated `own mut entity: Protocol` pass fixture cannot observe mutation from the caller (`own mut` moves — confirmed `SIFR-OWN-0001` on post-call read); the protocol must also declare the getter so the fixture can assert through the returned/in-callee value.
- Narrowing `&mut self` → `&self` for read-only field receivers will churn existing emitted-Rust snapshots; the plan relies on the full gate rather than budgeting it.

VERDICT: NOT SATISFIED

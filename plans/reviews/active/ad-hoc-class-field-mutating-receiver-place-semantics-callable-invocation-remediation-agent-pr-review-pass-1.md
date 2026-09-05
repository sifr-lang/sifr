## Methodology

Reviewed the exact pushed head `ac31b0908` (`git diff origin/main...HEAD`, clean tree apart from an empty untracked review-slot file), read the changed and surrounding code (`method_receiver_places.rs`, new `method_receiver_places/footprint.rs`, `indexed_storage.rs`, `Type::Class` definition), mechanically diffed the moved code against `origin/main` to isolate the semantic delta, then ran the in-repo suites plus 15 independent `.sifr` probes compiled/executed with the release compiler (probes written to `/tmp/probe_review`, no repo files modified).

## Semantic delta

The only behavioral change in the whole 384/219-line diff is the `HirExpr::MethodCall` arm of `collect_footprint` (verified by diffing the moved function against main — everything else in `footprint.rs` is a byte-identical move):

- `footprint.rs:66-91` — `callable_field_identity` resolves `object.ty().resolve_alias()` to `Type::Class`, rejects any name present in `methods`, requires the field's `resolve_alias()` to be `Callable(..) | AsyncCallable(..)`, and derives the exact `FieldIdentity` via the pre-existing `resolve_field_identity` (parent-chain walk).
- `footprint.rs:93-108` — `collect_callable_field` pushes `object_place + Field(identity)` when the base extracts as a static place, else falls back to `collect_footprint(object, …)`.
- `footprint.rs:167-177` — args are traversed unconditionally in both branches.

`expression_overlaps` is the single overlap engine used by all four validation sites (`method_receiver_places.rs:219, 227, 240, 270`), so receiver, specialized indexed storage, shared-receiver-vs-mut-arg, and mut-arg-vs-arg paths all inherit the fix uniformly. No alternate emitter of `SIFR-OWN-0002` exists (`ownership_diagnostics.rs:254` is the sole one).

## Probes and results

| # | Shape | Expected | Result |
|---|---|---|---|
| fixture | `self.update(self.callback(2))` | reject, col 28 | `SIFR-OWN-0002` for `self` at 15:28 ✓ |
| fixture | `self.helper.update(self.callback(v))` | accept + mutate original | runs, both asserts pass; negated copy panics, so asserts are load-bearing ✓ |
| p1/p2 | callable field **inherited** from `Base`, disjoint / true overlap | accept / reject | ran natively / `SIFR-OWN-0002` ✓ |
| p3 | `AsyncCallable` field, `await self.cb(v)` | disjoint accept, overlap reject | both correct ✓ |
| p7 | two-level `self.inner.helper.update(self.inner.cb(v))` | accept + mutate | ran, asserts pass ✓ |
| p8 | local var `owner.update(owner.cb(2))` | reject | ✓ |
| p9 | `self.helper.update(self.cb(self.helper.value))` | reject (args still traversed after narrowing) | ✓ |
| p10 | non-place base `self.update(self.pick().cb(2))` | conservative reject | ✓ |
| p11 | `self.helper.update(self.helper.cb(v))` | reject (prefix overlap) | ✓ |
| p14 | mut-arg path `bump(self.helper, self.helper.cb(v))` | reject | ✓ |
| p6/p15 | mut-arg disjoint; `self.items.append(self.cb(v))` | accept + mutate | ran, asserts pass ✓ |
| p12/p13 | callable field shadowing an own/inherited **method** name | must not narrow | both stay conservative (whole `self`), no misclassification ✓ |
| p16b | generic `Box[T]` with callable field, overlap | reject | ✓ |

Suites re-run independently: `sifr_lowering` **943 passed / 1 ignored**; targeted `invoked_callable_field*` **2/2**; full annotated E2E fail suite (`test_e2e_fail`) **pass**; `cargo clippy -p sifr_lowering -- -D warnings` clean; `cargo fmt --check` clean; HIR maintainability **PASS**; `check_file_size_guardrails.py` **PASS (limit 900)**. The split was necessary, not cosmetic: main's `method_receiver_places.rs` was 898 lines (2 under cap); head is 684 + 276.

Manifest: `class_field_callable_invocation_disjoint_sibling` matches the fixture stem and is in the create-pr pass manifest (`create_pr_e2e_manifest.json:15`); fail fixtures are auto-discovered and lane-independent, so the fail case is covered in every lane.

## Findings

**Blocking: none.**

Non-blocking observations (no action required for this PR):

1. `footprint.rs:76` (test-coverage, low) — the `methods` guard that prevents misclassifying real methods as callable fields has no regression test; probes p12/p13 confirm it works today, but deleting the guard would not fail any suite.
2. `footprint.rs:103` (test-coverage, low) — the non-static-base fallback (probe p10) and the mut-arg/indexed-storage propagation of the new narrowing (p14/p15) are behavior-verified but not locked by a fixture or unit test.
3. Out of scope, pre-existing: a lambda assigned to a `Callable`-typed local fails codegen (`def main(): reader: Callable[[int],int] = lambda v: 1` → `SIFR-BUILD-0005` / rustc `E0562`). Untouched by this diff (no codegen change) — flagged only because it surfaced while probing closure-capture aliasing.
4. `cargo clippy -p sifr_lowering --all-targets` reports pre-existing lints in test-support files; zero hits in any file this PR touches, and the project gate is the lib-target run, which is clean.
5. The PR carries no plan/doc update, consistent with this phase's pattern of recording the review artifact in a separate post-review commit (`0b0d7b8fb` for #3090).

## Verdict

The blocking finding is fully and correctly fixed: callable-field invocations now record `object place + exact FieldIdentity`, true overlaps still produce structured `SIFR-OWN-0002` with exact ranges, disjoint siblings are accepted and mutate the original storage, `AsyncCallable`, inherited, and generic identities behave identically, real methods are not misclassified, non-place bases stay conservative with arguments still traversed, and the module split is a verbatim move plus the one intended arm, under the 900-line cap.

**SATISFIED** — no blocking findings; the non-blocking items above are coverage/out-of-scope observations, so no blocking finding is present and the only remaining findings are non-blocking.

I verified the pass-5 findings against the live tree and ran a fresh adversarial scan (probes in `/tmp` only; no repo files modified).

## Pass-5 findings: all three closed

**1. `with`/`except` targets — closed.** §3 (lines 256–259) now classifies only `for`/comprehension element targets and `match` case captures as `EphemeralLocal`, and explicitly states `with` targets, exception-handler targets, tuple-unpack targets, and chained-assignment targets remain stable owned locals with `with open(...) as f: f.write(...)` required to stay accepted. The `SIFR-OWN-0014` check-fail row (line 668) is narrowed to loop/comprehension elements and `match` captures; the pass matrix (line 644) adds positive fixtures for all four stable-local shapes including the `open()`/`write()` idiom; acceptance criteria lines 771–773 restate both halves. The loop-element rejection carries no migration cost — I confirmed `for row in grid: row.append(9)` already fails today with a leaked `E0596` via `SIFR-BUILD-0005`, and a corpus scan of `demos/`, `stdlib/`, and `tests/e2e/pass/` found no real loop-element-rooted mutating receiver (the six `argparse.sifr` hits are plain locals, not loop variables).

**2. Operator dunder receivers — closed for operator traits.** §2 (182–189) makes Rust std operator dunders a fixed registry-sourced contract exempt from body inference, rejects any receiver-mutating dunder body during receiver analysis with `SIFR-PROTO-0006`, and §5 (405–407) pins the bridges to the registry convention *after* that check. `SIFR-PROTO-0006` is fully specified (title/severity/owner/template/args/fixture, lines 595–604) with a matrix row (672) and acceptance criterion (777–778). The in-tree operator dunder set is `__add__/__sub__/__mul__/__truediv__/__mod__/__neg__/__eq__/__lt__` (`operator_protocol_emitters.rs:54-74`) — no in-place `AddAssign`-style dunder exists, so the blanket rejection creates no contradiction. Codes `SIFR-OWN-0014`, `SIFR-PROTO-0005`, `SIFR-PROTO-0006` are all free (`registry.rs:125`, `:160`).

**3. Tuple/chained assignment — closed.** §3 line 257 and acceptance 772–773 name them as stable owned locals.

Also re-verified: all 30 anchors exist, and all ten decomposition line counts still match exactly (869/882/896/882/867/866/842/881/897/895).

## Remaining finding

**1. `__str__`/`__repr__` Display bridges are a fixed-receiver site the plan claims but does not carve out.** §5 (405–407) requires "regular class emission" to consume `HirFunction.receiver` and limits the fixed-convention exemption to "Rust standard operator-protocol bridges"; §2 scopes the exemption to "operator-trait dunders … such as `PartialEq`, `PartialOrd`, and `Add`" sourced from the operator registry. But `__str__` is not in that registry path — `operator_protocol_emitters.rs:74` explicitly skips `__str__`/`__repr__`, and the body is inlined into a `std::fmt::Display` impl by `class_emitter.rs:469,532,742` and `type_emitters.rs:84,242`, whose `fmt(&self, …)` receiver is fixed by the Rust trait. A `__str__` body that mutates a field will be seeded `MutableBorrow` by §2's first seed rule, `self` becomes a proven mutable root (`BindingKind::Receiver` + `MutableBorrow`), place validation accepts it, and the emitter must then either emit `fn fmt(&mut self, …)` into a fixed-signature trait impl (`E0053`) or hard-code `&self` — contradicting the "sole source for Rust `self` signature" acceptance criterion (line 755). Verified today this leaks rustc through `SIFR-BUILD-0005`:

```
error[E0594]: cannot assign to `self.hits`, which is behind a `&` reference
17 |     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
18 |         self.hits += 1_i64;
```

This is the same defect class as pass-5's finding 2, one site over. Fix is narrow: extend the fixed-receiver contract and `SIFR-PROTO-0006` (or a named sibling) to `__str__`/`__repr__` Display bridges, add `crates/sifr_codegen/src/class_emitter.rs` (786 lines, under cap) to the anchors, and add a mutating-`__str__` fail fixture to the check-fail matrix. The synthesized `Drop` impl at `class_emitter.rs:14` is compiler-authored with no HIR provenance and is already covered by the plan's carve-out for compiler-generated patterns.

VERDICT: NOT SATISFIED

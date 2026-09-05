I've completed the independent review: read all six artifacts and verified their claims against the actual compiler, runtime, package manager, stdlib, diagnostics registry, and verification infrastructure. The design is strong and almost all of it survives adversarial checking, but I found two internal-consistency defects that the three agent passes missed, both cheap to fix but both likely to cause divergent implementations of a milestone if locked as written.

# Final Independent Review: Declaration-First Python Interop

## What I verified against the repository (all claims held up)

Before findings, the grounding — every load-bearing factual claim in the proposal checks out:

- **Feasibility precedent is real, not aspirational.** The entire declaration mechanism is a direct analog of the *implemented* Rust interop: `@rust(...)` ellipsis declarations exist in shipped stdlib source (`stdlib/_sifr/*.sifr`), `@rust.opaque(type=..., send=..., close=async_close, ...)` grammar with symbolic policy atoms is specified at `internal_docs/rust_interop_architecture.md:150-167`, the "Ellipsis is public … declaration syntax" sentence is copied verbatim from the Rust precedent (`rust_interop_architecture.md:102`), and the reserved `__sifr_bridge` namespace pattern exists (`rust_interop_architecture.md:297`). Parser/lowering support for keyword-only parameters exists (`crates/sifr_ir/src/hir_nodes.rs:211`, `crates/sifr_lowering/src/lower/default_args.rs:17`).
- **The record-conversion claim is accurate.** "Attribute lookup first, then string-key item lookup, extras ignored" matches the implemented runtime exactly: `crates/sifr_runtime/src/python/object_ops.rs:404` does `object.getattr(*field).or_else(|_| object.get_item(*field))` per declared field.
- **Diagnostic bookkeeping is accurate.** `SIFR-PYIMP/PYCALL/PYCONV/PYRES/PYZC/PYCB` are reserved family bases (`crates/sifr_diagnostics/src/codes/registry/registry_entries/reserved.rs:13-18`); `SIFR-PYTRUST-0002` exists with the "allowed-but-untrusted" meaning being retired (`registry.rs:76`); `SIFR-PYENV-0001..0011` exist (`registry.rs:64-74`).
- **Scope paths are real.** Every crate path in the plan's Scope section exists (`sifr_ir`, `sifr_lowering`, `sifr_codegen`, `sifr_driver/src/build/`, `sifr_package/src/python/`, `sifr_runtime/src/python/`, `sifr_diagnostics`, `stdlib/sifr/python*.sifr`), as do the manifest keys (`crates/sifr_package/src/manifest/sifr_fields.rs:137-143`), the verification selectors (`verification/areas/python_interop/runner.py`), and the `python-interop-live` profile (`scripts/run_all_tests.sh:19`).
- **The raw-API migration burden is correctly characterized.** `stdlib/sifr/python_core.sifr:15-16` really does expose public `_handle`/`_token` fields, and `stdlib/sifr/python.sifr` uses them structurally (e.g. line 884), so M1's sealed-handle unification is correctly identified as a migration, not a rename.
- **Pass-3's refinements were genuinely incorporated**: bridge-to-bridge import rewriting (`python_interop_declaration_architecture.md:339-342`), M3/M4 dependency lines (plan lines 197, 225-226), the closed-policy-atom note (arch 131-133), and the GIL rationale for omitting `sync=` (arch 166-167). The plan's "Planning Review Evidence" section describes the review history accurately.

## Blocking findings

### B1. M5's example-migration scope contradicts the M10/M11 protocol deferral

**Where:** `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, M5 (lines 265-286), specifically tasks at 269-272 and acceptance at 278-281; mirrored in `internal_docs/python_interop_declaration_architecture.md` "Verification Contract" (lines 512-514).

**Failure mode:** M5 says "Migrate runnable biip/schwifty, dataframe, ML, web, database, cloud, crypto, and Redis examples to direct declarations or package-local bridges" and "Keep an intentionally small raw-object example proving the escape hatch," with acceptance "Package consumers use no raw handles for migrated binding surfaces." But the dataframe, ML, and Redis/Kafka examples exist precisely to demonstrate Arrow capsule exchange, DLPack tensor exchange, and `threadsafe_callback` delivery (`internal_docs/python_interop_architecture.md:85-92, 113-115`) — and those surfaces have **no declaration syntax until M10 (design) and M11 (implementation)**. Every reading of M5 as written conflicts with something:

1. Retain raw protocol code inside "migrated" examples → contradicts "intentionally small raw-object example" (singular) and invites M5 PR rejection under the "no raw handles" acceptance line.
2. Expose the protocol handoffs as `py.Object`-typed declaration parameters → consumers hold raw handles, same acceptance conflict.
3. Wrap the protocol work entirely inside Python bridges → silently deletes the Sifr-side zero-copy demonstrations the examples certify, weakening the evidence map without saying so.

Pass 2 blocked on the ancestor of this problem (old M5 routing protocol contracts through declarations); the fix created M10/M11 but left M5's example list and acceptance wording as incomplete-fix residue. M11 even says "migrate **their existing raw examples**" — confirming protocol raw usage is expected to persist through M5–M10, which M5's own wording doesn't permit.

**Correction (two sentences, no redesign):** In M5, add: "Migration in this milestone covers only declarable surfaces — calls, factories, methods, attributes, items, containers, and records. Arrow, DLPack, buffer, context-manager, and callback exchange points in the dataframe, ML, and broker examples intentionally remain raw `sifr.python` usage until M11, and M5's no-raw-handles acceptance applies only to the migrated surfaces." Adjust "Keep an intentionally small raw-object example" to acknowledge these retained protocol surfaces, and qualify the architecture doc's "Merge verification migrates the existing runnable ecosystem examples to declaration-first calls" the same way.

### B2. `close=async_close` is in the v1 grammar but is unsatisfiable (or meaningless) under the v1 async prohibition

**Where:** `internal_docs/python_interop_declaration_architecture.md`, "Minimal Decorator Grammar" (lines 131-133) and "Ownership And Cleanup" (lines 248-257), versus "Blocking And Async" (lines 392-396); plan M2 tasks (line 164) and M2 validation (lines 186-193).

**Failure mode:** The cleanup-policy list defines `close=async_close` as "ownership checking requires a declared consuming semantic `aclose` operation." In the Rust interop precedent, that operation is `async def aclose(own self)` (`rust_interop_architecture.md:163`). But the Python architecture states "There are no `@python.async` declarations in bridge version 1," and a method on an opaque class can only be a declaration. So either:

- `aclose` must be async → no opaque class can legally satisfy `close=async_close` in v1, making the atom dead grammar that M2 ("Implement `@python.opaque` with `close`, `send`, and type target metadata") must nonetheless implement; or
- `aclose` may be a synchronous consuming method → `close=async_close` differs from `close=close` only in the required method name, which is not a distinct ownership semantic.

An M2 implementer must choose, and the two choices produce observably different compilers and diagnostics. M2's validation list has no `aclose` fixture either way, so the divergence would not even be caught.

**Correction:** Pick one and state it in "Ownership And Cleanup": the cleanest is "`close=async_close` is reserved and rejected with `SIFR-PYRES-0001` in bridge version 1; it activates only with the future `@python.async` contract." Alternatively, define `aclose` as a synchronous consuming declaration and say why the distinct atom still matters. Mirror the decision in M2's task/validation lists.

## Non-blocking refinements

1. **`SIFR-PYTRUST-0003`'s meaning references the key being removed.** Its current message is literally "native Python import root '{root}' is trusted without an allow-imports entry … add the native root to `[python].allow-imports`" (`crates/sifr_package/src/python/trust_policy.rs:115-118`). The manifest-authority table (arch lines 360-368) and M6 (plan lines 290-305) handle 0002's retirement and 0005's activation but never say what 0003 means after `allow-imports` is gone. M6's acceptance line "No stale docs, fixtures, or diagnostics retain the removed source allowlist" implicitly forces this, but it should be an explicit M6 task: re-base 0003 on "native trust for a root that is not a required import root." None of the three passes caught this.

2. **There is no way to express "omit this keyword argument," and the doc doesn't say so.** "Argument Passing" (arch lines 170-187) states Sifr defaults are evaluated and passed explicitly, and the conversion table maps `Option[T]`'s `None` to Python `None` (line 199). Many real Python APIs distinguish *omitted* from *`None`* (sentinel-default APIs in pandas, `timeout=None` semantics in HTTP clients). As designed, an author must either replicate the Python default value exactly or route through a bridge — a defensible v1 rule, but currently only derivable by negative inference. Add one sentence: "Bridge version 1 has no omitted-argument semantics; `Option[T]` maps `None` to Python `None`, never to omission. Targets that distinguish the two require a package-local bridge." Also worth a locked fixture in the M0/M3 evidence list.

3. **`blocking_io` coverage of `@python.attr` and `@python.item` is only implied.** "Every initial `@python` call has the `blocking_io` effect" (arch line 387) names one decorator of four. Attribute access can run arbitrary descriptor code and must acquire the GIL, so it is equally blocking. Say "every declaration produced by any Python interop decorator."

4. **Target-path name resolution versus Sifr scope is unstated.** Arch lines 79-86 say a non-reserved root is declared by decorator use, but never say target paths are exempt from ordinary Sifr name resolution. Sifr ships a stdlib `math` module; in `@python(math.sqrt)` inside a file that also does `import math`, the intended reading (decorator paths live in an interop-target namespace, exactly as `@rust` targets do) should be stated, along with the rule that the reserved roots `bridge` and `Self` always win over a genuine Python distribution with the same name.

5. **`send=` admits only one value in v1.** The grammar requires `send=...` on `@python.opaque` while "all initial opaque values are non-send" (arch lines 166-168). State that `send=True` is rejected with `SIFR-PYRES-0001` in v1 (keeping the parameter as explicit documentation), or make the parameter optional-and-fixed. As written, an implementer could reasonably accept `send=True` as a no-op.

6. **No example shows a `close=close` opaque class.** The only opaque examples use `close=drop`. Since `close=close` "requires a declared consuming semantic close operation," show the consuming-receiver declaration shape (the Rust doc's `KafkaConsumer.aclose(own self)` analog) so M2 implementers don't invent the syntax.

7. **Bridge-module execution trust is implicit.** M4 acceptance says a dependency bridge cannot authorize its own *imports*, but nothing states under what authority the bridge module's own top-level code executes. The evident intent (bridge code is package code, trusted by the root's decision to depend on the package — the same model as Rust bridges; only third-party Python roots need `[trust].python`) deserves one sentence in "Environment And Trust," because it is a genuine security-model question a reviewer will ask.

8. **The library-only deferred-probe path has no named fixture.** Arch lines 83-86 define the "deferred target probe" behavior for library checks without a selected environment, but neither M1 nor M7 validation lists a library-only-check fixture. Add one (positive: declaration validates and records a deferred probe; negative: the deferred probe surfaces on final application build).

9. **M1 is the one remaining oversized milestone.** It bundles four separable subsystems: the sealed handle kind plus raw `Object`/stdlib migration (touching most of `stdlib/sifr/python.sifr`), the detach-before-decref/pending-release-queue rework, decorator parsing/IR/`PythonInteropPlan`, and wrapper codegen with effect/trust inference. The exit gate's "links every merged PR" phrasing permits multiple PRs per milestone, but M1 would benefit from an explicit internal PR sequence (handle + queue first, then decorator IR, then wrappers) the way pass 1 forced for the old M3/M4.

## Assessment against the review dimensions

**Consistency (1):** Excellent apart from B1/B2 and refinement 1 — diagnostic tables, manifest-authority table, argument-passing rules, bridge namespace strings, and verification categories match exactly between architecture and plan. **Ergonomics (2):** The signature-authoritative contract, factory-based construction, dict-out/attr-first-in records, and the two-level model are the right calls; refinement 2 is the only real ergonomic hole. **Feasibility (3):** High confidence — this is a port of an implemented architecture, and every claimed hook exists. **Soundness (4):** The detach-before-decref/pending-queue/epilogue-drain design is correct against CPython constraints and the no-panic guarantee; GIL, non-send, callback, and zero-copy invariants are preserved verbatim. **Hermetic packaging/trust (5):** Well specified after pass 2's fixes; refinements 4 and 7 close the last gaps. **Sequencing (6):** M0–M11 is correctly ordered and independently verifiable except for B1's M5/M11 overlap. **Verification (7):** The positive/negative/compiled-binary evidence policy is the strongest part of the proposal; refinements 2 and 8 name the only missing fixtures I found. **Underspecification (8):** B1, B2, and refinements 3-6 are the residual divergence risks.

Both blocking findings are sentence-scale corrections — neither requires redesigning any subsystem — but both sit exactly where the contract is meant to be locked, and both would otherwise surface as implementation-time disputes with divergent resolutions. Everything else is non-blocking.

# Verdict

**REQUEST CHANGES**

I have enough context. Writing the review now — will keep it CLI-only, no file modifications.

---

## Design Review: Sifr Stdlib Rust Interop Bridge Adapter Shape

### Verdict: **PASS with wording refinements**

The two-level model (direct binding for shape-matched signatures, adapter functions for reshaping/error mapping) is the correct decision. Deferring `@rust.via` for M9–M13 is right and consistent with Sifr's safety/probe/trust/cache invariants. Below is the reasoning, then five actionable wording fixes.

### Why the `@rust.via` deferral is elegant and load-bearing

- **Safety.** Bridge adapter functions are ordinary Rust source; they get compiled, borrow-checked, and wrapped by the boundary's existing `catch_unwind`. A single `@rust(...)` target per declaration preserves the invariant that the compiler validates exactly one bridge-compatible Rust signature and one panic policy. Introducing a second `target=` path per declaration would double the surface the "no user-triggerable runtime panics" contract has to reason about.
- **Probe.** Today's probe surface is a single Rust signature-existence assertion. `@rust.via(adapter, target=...)` would need (a) probe of the adapter, (b) probe of the target, and (c) a compatibility rule that the adapter's first parameter type structurally matches the target's `fn` type — with lifetime and generic constraints resolved. Bridge v1 has no facility for the third; deferring keeps the probe machinery within its current invariants.
- **Trust.** Trust rows are canonical Sifr dotted target paths (`rust_interop_architecture.md:847`). `@rust.via` would ask "does trust apply to the adapter, the target, or both?" — a resolution the current `[trust].rust-no-panic`, `unsafe-rust-bridges`, and `rust-panic-abort` shapes cannot answer without extension. Deferring avoids introducing a policy question with no lock.
- **Cache / dependency reporting.** `InteropBuildPlan` metadata and cache-key fingerprints (M8 sysroot interop dep-plan cache fingerprints) are keyed on the single target. Adding `target=...` would require bumping `bridge-version` and versioning the fingerprint schema — exactly the "future bridge-versioned extension" framing the doc already commits to. Consistent.

The doc's constraint that any future callee injection be limited to **non-generic function items or function pointers** is the right minimum. It rules out `impl Fn`, closures, and — crucially — anything that would require the compiler to synthesize monomorphizations of generic Rust callees, which is where lifetime, `Send`, and cache-key nightmares live.

### Actionable findings

**1. `internal_docs/rust_interop_architecture.md:519-522` — the `@rust.via` example self-contradicts the restriction two paragraphs later.**

```
@rust.via(bridge.json.parse_adapter, target=serde_json.from_str)
```

`serde_json::from_str` has signature `fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> serde_json::Result<T>` — it is generic and lifetime-parameterized, so it is exactly the kind of callee the same section (line 535–540) says would remain rejected in the first supported shape.

Recommendation: replace with a monomorphic example, e.g. `target=serde_json.from_str::<JsonValueBridge>` written explicitly, or pick a non-generic Rust item such as `chrono::NaiveDate::parse_from_str`. Note in prose that the target must be non-generic at the callee site.

**2. `internal_docs/rust_interop_architecture.md:108-125` — new "adapter boundary" text is silent on where panics are caught.**

The paragraph describes bridges reshaping inputs/outputs and "mapping ordinary Rust errors into the declared Sifr error channel." A first-time reader will ask "does the bridge catch panics from the backend call?" The answer, from `line 579-596`, is no — the generated wrapper's `catch_unwind` at the declaration boundary catches panics from the bridge *and* the backend call together, and `panic=` policy on the Sifr declaration decides what happens.

Recommendation: add one sentence at the end of the new block, roughly:

> Bridge code does not install its own panic guard. Panics from the bridge and from any Rust function it calls are caught at the generated wrapper according to the declaration's `panic=` policy.

Also replace "map ordinary Rust errors" with "map Result-typed Rust errors" — "ordinary" is imprecise given the bridge contract is specifically about `Result<_, E>` returns.

**3. `internal_docs/rust_interop_architecture.md:542-546` — "private `_sifr` bridge adapter function" conflates two vocabularies.**

`_sifr.*` is the Sifr-side private stdlib module namespace (see `sifr_sysroot_and_stdlib_architecture.md:172-174`). "Bridge" as a target root means the package-local `crate::bridges::*` module. The stdlib case uses neither: the adapter is a **Rust function inside the `sifr_stdlib` sysroot crate**, targeted by direct binding from a `_sifr.*` Sifr declaration through the M8 synthetic package context.

Recommendation: rewrite as

> For migrated stdlib leaves, direct binding is used both for exact-shape `sifr_stdlib` functions and for `sifr_stdlib` adapter functions that own input reshaping, output reshaping, and typed error mapping. In both cases the private `_sifr.*` declaration binds via `@rust(sifr_stdlib.<path>)`; there is no `bridge.*` package-local module for sysroot stdlib.

**4. `internal_docs/rust_interop_architecture.md:195-202` — the root table does not name `sifr_stdlib`.**

The root table lists "Cargo dependency name", `bridge`, "Shared bridge crate name", and `Self`. The stdlib policy addendum introduces a distinct pattern — direct binding to a sysroot crate through the compiler-owned private package context — without a row in this table. Reviewers of future migrations will look here to justify a target and not find the sysroot case.

Recommendation: add a row (or a note under the table) covering the sysroot-owned crate root: "Sysroot-owned crate (`sifr_stdlib`, `sifr_runtime`) — resolved only under the compiler-owned synthetic private `_sifr.*` package context; user packages cannot target these roots directly." Cross-link `sifr_sysroot_and_stdlib_architecture.md` M8.

**5. `internal_docs/sifr_sysroot_and_stdlib_architecture.md:119-144` — policy is complete, but does not tie itself to the `bridge-version = 1` gate.**

The interop doc's future-callee section is explicit that any extension would be `bridge-version`-gated. The stdlib policy addendum should mirror that in one sentence so reviewers know the M9–M13 migrations are stable against a future bridge-version bump.

Recommendation: append to the third paragraph:

> All M9–M13 migrations are committed to `bridge-version = 1` semantics. A future `@rust.via` or similar callee-injection form must be introduced under a bumped `bridge-version` and must not silently rewrite existing sysroot interop declarations.

### Minor polish (not blocking)

- `docs/rust-interop.mdx:104-107` — "Sifr does not currently expose Rust functions as Sifr values." Consider "Rust functions are not Sifr values today, and generated glue does not accept Rust closures, `impl Fn`, or `Box<dyn Fn>` from Sifr source" — mirrors the internal doc's rejection list so external readers can't infer a callback affordance from silence.
- `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:48-53` — the policy summary reads fine; consider a link to the specific section anchor (`#stdlib-rust-interop-adapter-policy`) rather than the whole architecture doc, so future PR reviewers can jump straight to the lock.
- `internal_docs/rust_interop_architecture.md:116-121` — the enumerated list "1. Direct bindings ... 2. Local or shared bridge functions ..." is good, but combining `_sifr.*` → `sifr_stdlib` under "Local or shared" is inaccurate: sysroot-crate adapters are neither package-local nor a general-ecosystem shared bridge (like `sifr_arrow_bridge`). Consider "2. Local bridge functions, shared bridge crates, and sysroot-owned adapter crates (`sifr_stdlib`, `sifr_runtime`) …".

### Consistency across the four files

The four changes are internally consistent: interop doc introduces adapter-boundary language and defers `@rust.via`; stdlib doc encodes the M9–M13 policy from that; public docs mirror the adapter role and the "Rust functions are not Sifr values" statement; the workstream tracker links to the policy. The only cross-file gap is the vocabulary drift around "bridge adapter" vs. "sifr_stdlib adapter" flagged in findings 3–4 — worth resolving before merge, because reviewers of subsequent M10/M11 PRs will lean on these paragraphs as the normative reference.

### Bottom line

The design is elegant: it keeps the interop model at a single-target-per-declaration boundary, keeps trust/probe/cache invariants intact, and defers a real extension (`@rust.via`) rather than smuggling it in as an implicit affordance. Ship the four docs after fixing findings 1–5; the remaining polish is nice-to-have.

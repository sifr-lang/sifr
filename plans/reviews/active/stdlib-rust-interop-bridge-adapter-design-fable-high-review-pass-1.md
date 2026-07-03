The placeholder review file is empty; per your instruction I'm leaving it untouched and writing the review here. I read all five artifacts plus the full working-tree diff (113 added lines across the four docs), and verified each Opus pass-1 finding was applied.

---

## Fable High Design Review: Stdlib Rust Interop Bridge Adapter Shape — Pass 1

### Verdict: **PASS** (no blocking findings)

All five Opus pass-1 findings and the three polish items are correctly applied in the working tree. The four design decisions under review are sound, mutually consistent across the four documents, and — with one contract-coherence caveat below — the cleanest available shape for the stdlib rewrite.

### Why the design is right (independent assessment, not just re-verification of Opus)

**Decision 2 is the load-bearing one, and it is correct for a structural reason the docs only imply.** A `sifr_stdlib` adapter cannot be replaced by a package-local `bridge.*` module even in principle, because the synthetic `_sifr.*` package context has no user-authored source tree: a `src/bridges` module there would be compiler-generated Rust masquerading as "user bridge code," unversioned by the sysroot lockfile, outside `cargo test -p sifr_stdlib`, and outside the sysroot workspace lints. Putting adapters in `sifr_stdlib` makes them ordinary first-party Rust: vendored, clippy-checked, unit-testable, versioned with the toolchain, and covered by the sysroot content digest in the cache key. There is exactly one place adaptation can live, and the design puts it there.

**The `@rust.via` deferral costs nothing for sysroot stdlib.** The stated motivations for callee injection — per-declaration dependency attribution, trust evidence, and diagnostics naming the real backend — are all solved for the sysroot case by other machinery: backend crates are workspace dependencies of `sifr_stdlib` tracked by the sysroot lockfile and `SysrootDependencyPlan`, and trust is the compiler-owned sysroot policy. `@rust.via` only earns its complexity for *user packages* wrapping third-party crates, which is exactly why deferring it behind a bridge-version bump is right rather than merely convenient.

**Decision 4 (no per-declaration converter pipelines) preserves the single-probe invariant.** Every `_sifr.*` declaration validates exactly one bridge-compatible Rust signature; whether the target is exact-shape or an adapter is invisible to the compiler and irrelevant to the contract. That means "direct binding only for bridge-compatible signatures" is satisfied *by construction* for adapters — the adapter's signature is the bridge-compatible one. No second validation surface, no compiler-interpreted conversion metadata, no fallback path.

### Blocking findings

None.

### Non-blocking findings and wording refinements

**1. The normative interop doc has not absorbed the `E: Display` error mapping that the adapter policy depends on.** (`internal_docs/rust_interop_architecture.md:126` and the Bridge Type Contract table at `:415`; contrast `internal_docs/sifr_sysroot_and_stdlib_architecture.md:456-461` and the merged M10 wave 1/2 evidence.)

The new adapter-boundary paragraph says a Rust crate exposing "domain-specific error types" requires a bridge adapter, and the Bridge Type Contract table permits only generated bridge enums/structs in the `Result` error position. But M10 shipped — and the sysroot doc records — a general direct-binding rule where the compiler maps `Result<_, E: Display>` into message-shaped Sifr error classes (and all-string-field subclasses like `RegexError { message, detail }`) at the generated wrapper. This rule is load-bearing: it is *why* `sifr_stdlib` adapters can return their own error types at all, since a sysroot crate — like any shared bridge crate — cannot import package-generated `__sifr_bridge` types, so the final error-type construction must happen in generated glue. The rule is architecturally correct (it is the sysroot instantiation of "generated glue adapts errors to the shared crate's public types outside the shared crate," `rust_interop_architecture.md:517`), and it is a single global typed rule, not a per-declaration pipeline — so decision 4 holds. But today it exists only in a migration-status paragraph of the sysroot doc, while the normative doc's contract table and the new "domain-specific error types → write an adapter" clause contradict it. Recommend: add the `E: Display` / all-string-fields error-position rule to the Bridge Type Contract (including its probe shape, since the probing section shows only exact-type assertions), and narrow the adapter-required clause to "error types whose mapping needs more than display-text shaping." This is the one place where the otherwise-clean contract is currently split across a normative doc and a status paragraph.

**2. The "two-level" taxonomy conflates binding mechanism with adaptation ownership.** (`internal_docs/rust_interop_architecture.md:116-121` vs. `:547-552`.) Level 2 lists "sysroot-owned adapter crates such as `sifr_stdlib`," yet the Future Callee Injection section correctly states that sysroot adapters are reached through *direct binding*. For packages the two axes coincide (adapter ⇒ `bridge.*` root); for sysroot they don't (adapter ⇒ still `@rust(sifr_stdlib.<path>)`). A one-clause clarification that the levels classify *who owns adaptation*, not which decorator root is used, would prevent a future migration reviewer from concluding that an adapter-backed `_sifr.*` declaration is misfiled as a direct binding.

**3. The adapter policy section is silent on the panic surface of sysroot declarations.** (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:119-148`.) Public stdlib errors like `ParseError` and `RegexError` do not include `RustPanicError`, so under the panic-surface policy every migrated `_sifr.*` declaration must be running under an (implicit, compiler-owned) trusted-no-panic policy backed by `sifr_stdlib`'s crate-level no-panic conventions. That is coherent with M8's "sysroot trust requirements satisfied by the compiler-owned sysroot policy," but the adapter policy — the section future waves will cite — should say it in one sentence, because it is precisely the point where "no user-triggerable panics" rests on a trust assertion about first-party adapter code rather than on the `RustPanicError` channel.

**4. Minor: the `@rust.via` illustration uses a path shape the resolution table doesn't define.** (`internal_docs/rust_interop_architecture.md:524-526`.) `target=regex.Regex.new` names an inherent associated function through a Cargo-dependency root; the current path grammar defines free-function dotted paths and `Self.method`, not `Crate.Type.function`. Since the whole section is explicitly future/bridge-versioned this is acceptable as illustration, but a parenthetical "(associated-function target paths would themselves be part of that future design)" would keep the example from implying the path form already resolves.

### Judgment on fallback/compatibility compromises and cleanliness

**The design avoids fallback and compatibility compromises, and I judge it the cleanest architecture for the stdlib rewrite.** Specifically:

- No converter fallbacks: adaptation is ordinary Rust in one owned crate; the compiler validates one signature per declaration and interprets no conversion chains. The `E: Display` mapping is a primary typed path with probe-checked shape, not a fallback.
- No compatibility shims for abandoned designs: `@rust.via` is deferred as a *future bridge-versioned extension* with explicit rejection of function values crossing Sifr, rather than half-shipped; the docs commit M9–M13 to `bridge-version = 1` and forbid retrofitting fallback conversion onto existing sysroot declarations.
- The only transitional mechanism in the workstream — intrinsic-fallback bootstrap for partially migrated modules like `_sifr.crypto` — lives in the migration tracker with an M12 deletion stage, not in the architecture contract, which is the right separation.
- The alternatives (package-local `bridge.*` for the synthetic sysroot package; decorator converter pipelines; shipping `@rust.via` now) are each strictly worse: respectively unowned/untestable glue, a second compiler-interpreted contract surface, and a second target path per declaration with unresolved trust/probe/cache-key semantics.

Finding 1 is the item worth fixing before the next M10 wave lands, since that wave's reviewers will read the Bridge Type Contract as the rule of record and find it forbidding what the wave does. Findings 2–4 are wording hardening. None block the design.

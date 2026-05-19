

Reviewing the plan for the five specifically claimed changes:

1. **`[package.metadata.sifr]` is only a manifest pointer** — Lines 233-262 define it as a discovery hook with `manifest` as the only required key. All semantic data (name, edition, source roots, features, trust) lives in `sifr.toml`. Consistent throughout.

2. **Package classification cases** — Lines 148-153 cover four cases: Sifr source package, Rust-backed Sifr, no-metadata backend crate, and invalid manifest. No gap.

3. **Package name resolution/discovery** — Lines 529-541 define a five-step lookup order, registry behavior, and the explicit compiler-incompatibility-after-Cargo-resolution rule. Coherent.

4. **Compiler-version compatibility after Cargo resolution** — Line 541: "Sifr rejects that graph with a package diagnostic rather than attempting fallback resolution." No contradiction with the non-goals.

5. **Archive validation** — Lines 603-607 specify two approaches: Cargo CLI package dry-run file list when available, or direct `.crate` archive inspection otherwise. `SIFR-PACKAGE-0403` is defined for include/exclude omissions. Consistent.

Checking cross-cutting consistency:
- Aliases live in `Cargo.toml` (lines 258-260) due to Cargo dependency name dependencies; `sifr.toml` may document but not duplicate authority. No contradiction.
- Pure Sifr marker enforcement (lines 88-98) with `SIFR-PACKAGE-0501` is defined and consistent with the non-goal "pure Sifr packages rely on Rust marker crates for behavior."
- Direct dependency scope definition (lines 295-297) is precise; re-exports are handled through `__init__.sifr` semantics (lines 499-500).
- Milestone definitions, diagnostic table, maintainability architecture, guardrails, and exit gate are internally consistent.

No blockers. No contradictions.

**verdict: ready**

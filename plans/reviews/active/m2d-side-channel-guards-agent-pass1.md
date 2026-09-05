READY

Findings (none blocking):

- **Non-unique dedup fall-through is correct but leaves `owners` pointing at the last surface for `retained_direct_dependency_packages`.** Since duplicates are not failed and `allowed[key]` is a set, this is harmless — just noting the diagnostic (if ever surfaced elsewhere) would show the most-recent owner rather than the first.
- **`GENERATED_DEPENDENCY_PACKAGE_RE` assumes `features.rs` writes dep names as `package: "<name>"` literals.** If a dep is ever emitted via `const` or `format!(…)`, it will be silently missed from `observed`. Not a bug in this diff — worth flagging as an assumption baked into the guard.
- **`DIRECT_RUNTIME_ROOT_RE` only matches when the first path segment after `sifr_runtime::` is a bare identifier.** Any dynamic construction (`format!("sifr_runtime::{}::…")`) would evade observation. Again an assumption, not a correctness bug against the current codegen.
- The `interop`/`DEFAULT_MAX_INTEGER_DIGITS` entries under `shared-language-preamble` and the per-surface `net`/`tls`/`python` roots line up with the `direct_runtime_roots` unique-owner rule — no cross-surface ownership conflicts visible in the manifest.
- Manifest schema updates and the `has_owned_surface` extension correctly permit surfaces (e.g. `additional-feature-planning`) that only own dependency packages without registry files.

No concrete correctness bugs in gating behavior for the new fields.

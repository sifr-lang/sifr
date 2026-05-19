# Phase 37 Cargo Docs Alignment Review

Date: 2026-05-19

Scope: verify `internal_docs/phases/37_package_management.md` against the current official Cargo Book / command documentation for package-management behavior referenced by Phase 37.

## Sources Checked

- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo dependency resolver: https://doc.rust-lang.org/cargo/reference/resolver.html
- `cargo metadata`: https://doc.rust-lang.org/cargo/commands/cargo-metadata.html
- `cargo build`: https://doc.rust-lang.org/cargo/commands/cargo-build.html
- `cargo fetch`: https://doc.rust-lang.org/cargo/commands/cargo-fetch.html
- `cargo package`: https://doc.rust-lang.org/cargo/commands/cargo-package.html
- `cargo add`: https://doc.rust-lang.org/cargo/commands/cargo-add.html
- `cargo vendor`: https://doc.rust-lang.org/cargo/commands/cargo-vendor.html
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html

## Findings

- Cargo's current docs use Rust edition `2024` in examples and resolver `"3"` for current virtual-workspace examples. Phase 37 examples previously used older `2021` / resolver `"2"` values from earlier review rounds.
- `cargo metadata` documents source IDs as opaque identifiers. Phase 37 should store and compare source IDs but not parse their internal string format.
- Cargo package selection docs distinguish root default selection / `default-members` from explicit `--workspace`. Phase 37's workspace text already reflected this after the monorepo pass; one `--workspace` sentence was tightened to avoid implying `--all` is a build selector.
- Cargo `[workspace.dependencies]` docs state members must opt in with `workspace = true`; workspace dependencies cannot be optional; features are additive with member dependency features. Phase 37 now states these details.
- `cargo fetch` has manifest/workspace behavior rather than Sifr package-filter behavior. Phase 37 now says Sifr delegates fetch to Cargo and validates the selected Sifr package graph after Cargo materializes sources; it does not attempt to reimplement Cargo fetch selection.
- `cargo package --list` is documented as printing the included files without making a package. Phase 37's package archive validation model remains aligned: prefer Cargo's file list and inspect generated archives when needed.

## Edits Made

- Updated Cargo examples from `edition = "2021"` to `edition = "2024"`.
- Updated the virtual workspace demo from `resolver = "2"` to `resolver = "3"`.
- Added a Cargo metadata source ID opacity rule.
- Clarified `sifr fetch` as Cargo manifest/workspace fetch plus Sifr graph validation, not independent package-filter fetching.
- Clarified `sifr tree --all` is display-only and not a build/check/test selector.
- Added current `[workspace.dependencies]` constraints: no `optional`, additive features, explicit `workspace = true` opt-in.

## Verdict

Ready. The Phase 37 Cargo-facing specifications now match the current Cargo documentation for the covered areas.

# sifr_package Dependency Audit

package-management rules keeps Cargo as the package substrate without linking to Cargo internals. The `sifr_package` crate consumes stable command output and builds Sifr-owned data structures for the compiler.

## Cargo CLI Metadata JSON

- Surface: `cargo metadata --format-version 1`.
- Local toolchain audited for package-management readiness: `cargo 1.98.0`, `rustc 1.98.0`.
- Used by: `crates/sifr_package::cargo::metadata`.
- Fields consumed: `packages[].id`, `name`, `version`, `source`, `manifest_path`, `dependencies`, `targets`, `features`, `metadata.sifr`, `resolve.nodes[].deps[]`, `workspace_members`, `target_directory`, and `workspace_root`.
- Reason: Cargo owns package resolution, source identity, workspace membership, dependency rename identity, selected package roots, and resolved dependency edges. Sifr needs those facts but not Cargo's internal resolver APIs.
- Stability risk: Cargo may add fields, and JSON ordering is not a semantic rules.
- Mitigation: Sifr deserializes only consumed fields, accepts unknown JSON, normalizes packages/dependencies/targets/resolve edges/workspace members, and computes graph digests from normalized Sifr-owned structures.
- Fallback: incompatible metadata or missing required fields map to `SIFR-PACKAGE-0103` instead of falling back to one-off manifest parsing.

## Cargo Command Plans

Sifr models Cargo command invocations as `CargoCommandPlan` values before any driver shell-out. The audited command surfaces are:

| Cargo surface | package-management rules use | Risk control |
| --- | --- | --- |
| `cargo metadata --format-version 1` | graph discovery and package roots | normalized JSON facade, no Cargo internal crate types |
| `cargo fetch` | source materialization before offline/frozen validation | lock-mode arguments are explicit |
| `cargo build` | generated Rust build delegation | lock mode, target, and feature args are modeled |
| `cargo package` | package archive production after Sifr validation | archive validation is Sifr-owned before delegation |
| `cargo publish --dry-run` / `cargo publish` | publish preflight and upload delegation | credentials are never included in Sifr package metadata; Cargo failures are redacted |
| `cargo vendor` | vendored source generation | output path is passed through Cargo, not interpreted as a registry cache |
| `cargo add`, `cargo remove`, `cargo update` | manifest mutations and updates where supported | mutation commands are blocked under frozen/offline lock modes |

## Cargo Integration Crates

No `cargo_metadata` crate and no `cargo` internal crates are linked in package-management rules. If a future rules introduces one, this file must record:

- exact crate version pinned in `Cargo.lock`;
- Cargo CLI version range and `--format-version` validated against it;
- fields consumed by Sifr;
- ordering and compatibility risks;
- fallback behavior when the crate API or Cargo JSON changes.

## Cargo Source Cache Boundary

Sifr does not parse Cargo registry, Git checkout, or source-cache internals. Source ids are opaque Cargo identifiers. Package roots are trusted only after Cargo exposes them through metadata/fetch, then Sifr validates `sifr.toml`, the source root, source-declared exports, markers, trust policy, archive contents, and import boundaries itself.

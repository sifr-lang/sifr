# Generated Rust authority

This crate owns Rust syntax normalization, support dependency pruning, generated
visibility and compiler-owned macro syntax. Its normal dependencies are syntax
libraries only. Sifr HIR, type identities and compiler build services belong to
upstream consumers. Codegen supplies resolved nominal-name replacements and the
canonical builtin-error catalog; this crate does not duplicate either authority.

Private invariants remain unit tests under `tests/unit`. They do not expose APIs
for tests. Frontend-to-generated-program contracts stay with `sifr_codegen`, whose
existing public normalization functions delegate to this authority. Both packages
remain mandatory in every delivery profile that selected the original codegen tests.

```sh
uv run --project verification --locked cargo test --locked -p sifr_generated_rust <filter>
uv run --project verification --locked cargo test --locked -p sifr_codegen -p sifr_generated_rust
```

Production input ownership is `src` plus Cargo manifest/configuration inputs.
Codegen recursively owns this package's production inputs. Editing a private test
must rebuild its test executable without changing the production CLI; editing a
normalization implementation must invalidate all consumers.

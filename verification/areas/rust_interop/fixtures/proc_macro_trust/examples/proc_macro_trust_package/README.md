# fixture: proc_macro_trust
# scenario-example: proc_macro_trust_package

This locked/offline scenario certifies package-level trust for a direct derive
wrapper compiling exact `serde_derive 1.0.229` and a direct build-script
wrapper running exact `prost-build 0.14.4`. Prost generation uses an in-memory
descriptor set, writes only beneath `OUT_DIR`, compiles the generated message,
and needs no `protoc` installation.
The wrapper executes its own `SifrGenerated` derive; its marker labels that
execution separately from compilation of the exact upstream dependency.

`sifr.toml` trusts `serde_derive` as the proc-macro dependency alias and
`prost_build` as Cargo's normalized Rust alias for the `prost-build`
dependency. The generated Sifr package observes both versioned markers. Test
controls arm wrapper-local sentinels to prove each execution path is live
before missing trust is shown to reject the package ahead of Cargo or rustc.

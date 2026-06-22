# fixture: bridge_version_mismatch
# scenario-example: bridge_version_package

This scenario models a package declaring the currently supported Rust interop
bridge schema. The positive evidence accepts `[rust] bridge-version = 1`; the
paired negative evidence changes the same manifest field to an unsupported
future value and must produce `SIFR-RUST-CARGO-0001`.

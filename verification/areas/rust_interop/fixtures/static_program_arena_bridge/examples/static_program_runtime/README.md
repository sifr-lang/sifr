# Static program runtime example

This package uses the unversioned Rust manifest contract. The Sifr compiler runs the package
specializer and emits immutable static bytes for `StaticRecord`. The Rust executor can access the
program only through the compiler-generated `StaticProgramType` implementation.

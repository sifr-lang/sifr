# package-management rules Demo Repositories

These directories are git submodule checkouts for the package-management rules organization demo
repositories listed in the package-management architecture references.

They are intentionally kept under `verification/areas/package_management` rather than
`demos/`: each subrepo is a multi-file Cargo/Sifr package repository fixture,
not a single runnable Sifr language demo.

The guardrail script validates the required package shapes through
`verification/areas/package_management/data/package_demo_repositories.json`.
Run `scripts/clone_subrepos.sh` after a fresh checkout to initialize these
subrepos together with the rest of the Sifr submodules.

Readiness demos use the canonical production layout: Sifr source files live in
`src/`, public APIs are declared by `src/__init__.sifr`, and production
manifests do not use `[exports].modules` or Sifr manifest `[[bin]]` target
tables. Legacy `sifr/<package>/` layouts are allowed only in explicitly named
internal regression fixtures outside these production demo repositories.

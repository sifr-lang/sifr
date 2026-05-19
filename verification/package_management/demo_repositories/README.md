# Phase 37 Demo Repository Templates

These directories are the source templates for the Phase 37 organization demo
repositories listed in `internal_docs/phases/37_package_management.md`.

They are intentionally kept under `verification/package_management` rather than
`demos/`: each template is a multi-file Cargo/Sifr package repository fixture,
not a single runnable Sifr language demo. Publishing these templates to
`sifr-lang/sifr-demo-*` repositories should preserve the directory contents and
tag the package revisions named in the manifests.

The guardrail script validates the required package shapes through
`verification/package_management/phase37_demo_repositories.json`.

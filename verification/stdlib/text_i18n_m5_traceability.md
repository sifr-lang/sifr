# Text/I18n M5 Traceability

Milestone: `milestone_text_i18n_5`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Public docs | `docs/text_i18n.md` covers `sifr.encoding`, `sifr.unicode`, explicit `sifr.io` text I/O, `sifr.i18n`, and intentional Python-shaped differences; `docs/stdlib_imports.md` links it from the stdlib namespace contract. |
| Internal docs | `internal_docs/text_i18n_architecture.md` records valid-text invariants, registry/text-I/O boundaries, Unicode table/segmentation strategy, locale/i18n data, translation catalog parsing, and host-limited behavior. |
| Demos | `demos/text_i18n/main.sifr` covers non-UTF-8 encode/decode, explicit text open, Unicode normalization/property lookup, grapheme/word segmentation, explicit `LocaleId` number formatting, and translation fallback/plurals. |
| Dependency snapshots | `verification/stdlib/text_i18n_dependency_snapshots.json` records encoding, Unicode, i18n, every pairwise combination, and the combined encoding+Unicode+i18n generated Cargo dependencies; `crates/sifr_stdlib/src/features.rs::text_i18n_feature_dependency_snapshots_cover_phase_combinations` locks the snapshots in unit tests. |
| Panic/emitted-code scans | `verification/generated_code_quality/manifest.json` includes `demo-007-text-i18n` and `e2e-051` through `e2e-055` for encoding, Unicode, segmentation, locale, and translation paths. |
| Validation lane manifests | `verification/validation_lanes/create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` include M1-M4 text/i18n fixtures in the e2e sets. |
| Inventory closure | `verification/stdlib/text_i18n_substrate_inventory.md` and `.json` record terminal state, stability, CPython evidence, fixtures, and revisit rules for production APIs and Python-shaped surfaces. |
| Final review | `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-1.md` returned `PASS`; `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-2.md` returned `PASS` after e2e batch harness feature-propagation remediation, with no blockers and no re-review required. |

"""Fixed fresh pair counts registered in issue #4277 before v2 measurement.

The failed v1 capture is pilot evidence only. These counts are immutable during
an invocation; precision planning does not guarantee qualification or power.
"""

DEFAULT_PAIRS = 32

PAIR_COUNTS = {
    "build-project-001-additional-modules": 32,
    "build-project-002-branch-paths": 64,
    "build-project-003-cargo-manifest": 192,
    "build-project-004-dependency-manifest": 64,
    "build-project-005-project-graph": 192,
    "build-single-file-001-break-continue": 64,
    "build-single-file-002-builtin-any-all": 96,
    "build-single-file-003-builtin-enumerate-zip": 96,
    "build-single-file-004-bytes-constructors": 96,
    "build-single-file-005-class-methods": 32,
    "build-single-file-006-collection-len": 64,
    "build-single-file-007-comp-set": 96,
    "build-single-file-008-decimal-conversions": 96,
    "build-single-file-009-dict-get-option": 64,
    "build-single-file-010-generic-identity": 224,
    "check-project-001-imports": 96,
    "check-project-002-mode-consistency": 96,
    "check-project-003-project-build": 128,
    "check-project-004-project-graph": 64,
    "check-project-005-rooted-entrypoint": 64,
    "check-single-file-001-arithmetic": 64,
    "check-single-file-002-break-continue": 96,
    "check-single-file-003-builtin-any-all": 128,
    "check-single-file-004-builtin-enumerate-zip": 128,
    "check-single-file-005-bytes-constructors": 64,
    "check-single-file-006-class-methods": 224,
    "check-single-file-007-collection-len": 32,
    "check-single-file-008-comp-set": 128,
    "check-single-file-009-decimal-conversions": 64,
    "check-single-file-010-dict-get-option": 96,
    "diagnostic-non-regression-001-human-success-exit": 64,
    "diagnostic-non-regression-002-json-diagnostic-schema": 64,
    "diagnostic-non-regression-003-compact-diagnostic-schema": 64,
    "diagnostic-non-regression-004-recovery-limit": 64,
    "diagnostic-non-regression-005-project-error-exit": 64,
    "formatter-corpus-001-project-check": 32,
    "formatter-large-file-001-check": 32,
    "incremental-local-loop-001-unchanged-file-update": 32,
    "incremental-local-loop-002-leaf-module-change": 32,
    "incremental-local-loop-003-imported-module-change": 32,
    "incremental-local-loop-004-public-api-change": 32,
    "incremental-local-loop-005-failure-recovery": 32,
    "interactive-tooling-foundation-001-cold-context-load": 32,
    "interactive-tooling-foundation-002-warm-diagnostics-query": 32,
    "interactive-tooling-foundation-003-unchanged-file-update": 32,
    "interactive-tooling-foundation-004-changed-file-invalidation": 32,
    "interactive-tooling-foundation-005-source-map-lookup": 32,
    "lsp-query-001-request-families": 32,
    "lsp-query-002-cold-start": 32,
    "lsp-query-003-diagnostics": 32,
    "lsp-query-004-workspace-diagnostics": 32,
    "lsp-query-005-completion": 928,
    "lsp-query-006-hover": 32,
    "lsp-query-007-signature-help": 32,
    "lsp-query-008-navigation": 32,
    "lsp-query-009-references": 32,
    "lsp-query-010-rename": 32,
    "lsp-query-011-semantic-tokens": 32,
    "lsp-query-012-inlay-hints": 32,
    "lsp-query-013-selection-range": 32,
    "lsp-query-014-type-hierarchy": 32,
    "lsp-query-015-code-actions": 32,
    "lsp-query-016-formatting": 32,
    "lsp-query-017-generated-rust-preview": 32,
    "lsp-query-018-did-open-diagnostics": 32,
}


def pairs_for_case(case_id: str) -> int:
    """Unlisted manifest cases use the policy minimum, never outcome adaptation."""
    return PAIR_COUNTS.get(case_id, DEFAULT_PAIRS)


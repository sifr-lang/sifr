# SQL incremental analysis and editor architecture

This document records the implemented incremental and editor boundary for
schema-first SQL. The normative SQL design remains in
[`sql_architecture.md`](./sql_architecture.md).

## Ownership

`sifr_frontend::cache_keys` owns the semantic cache identity. An embedded SQL
key contains the complete component request and the normal frontend context.
The request includes these inputs:

- template segments and closed hole types
- fragment identities
- the selected schema profile and dependency-slice fingerprint
- provider identity and component version
- compatibility settings
- component protocol major
- compiler semantic version

The dependency-slice fingerprint is the schema fingerprint in the component
request. A caller must not substitute the complete database fingerprint when a
smaller proven slice exists.

`sifr_frontend::EmbeddedQueryCache` owns bounded process-local value reuse. Its
default capacity is 4,096 entries. It supports pinning and reports each evicted
key. A miss or eviction causes the caller to repeat the same provider operation.
It does not select a fallback result.

`sifr_analysis::SqlIncrementalAnalysisCache` owns the dependency index. Each
query records canonical dependency identities and fingerprints from the
component response. Invalidation compares them with a complete current
dependency map. A changed or removed dependency invalidates each query that
used it. An unrelated change preserves the cached query.

Eviction, capacity, and pinning can change reuse only. They cannot change a
diagnostic, query plan, or generated output.

## Virtual documents and maps

The frontend creates one `SqlEditorDocumentView` for each typed template in the
HIR. It retains the virtual SQL text, hole order, hole types, and source range.

The lowerer records one source-to-virtual mapping for each decoded character.
This mapping keeps escapes, doubled braces, Unicode text, and interpolation
holes lossless. The frontend supports offset and range translation in both
directions. Rename, semantic tokens, and quick fixes use exact range
translation. They do not replace a complete static segment for one SQL token.

The U+FFFC object-replacement character represents one interpolation hole in a
virtual SQL document. The hole keeps its source expression range and its
zero-based parameter index.

## Semantic editor model

`SqlEditorCatalog::from_schema` converts `SchemaIR` relations, columns,
functions, operators, and types into editor symbols. It keeps schema definition
locations and the reversible generated-name map. `with_provider_analysis`
adds result fields, database types, Sifr types, nullability, and exact
cardinality from provider analysis.

The analysis layer routes a SQL position through the same snapshot as normal
Sifr queries. It provides:

- syntax highlighting and scoped completion
- hover, definition, references, and rename
- parameter, result, nullability, and cardinality information
- formatting through the Sifr formatter
- structured quick fixes

Fragment completion filters both relations and their columns. It resolves a
relation alias before it selects column candidates. The generated identifier
reverse codec converts generated profile names to database identities before
symbol lookup.

Provider diagnostics can supply structured fixes with an exact virtual range,
replacement, title, and detail. The editor also supplies safe fixes for known
alias, cast, missing-column, and bounded-collection diagnostics. Migration
analysis can attach the same structured fix for a supported impact change. The
language server converts the virtual range to an exact Sifr source edit.

A wildcard diagnostic identifies the wildcard at its nested SQL expression. It
does not state that the exported result contains the wildcard. When the schema
has a safe column candidate, the editor offers an explicit-column replacement.

## Cancellation and performance

`run_embedded_provider_operations` checks cancellation at these boundaries:

1. before component entry
2. between provider operations
3. before result publication

The caller supplies the cancellation state. This keeps transport cancellation
outside the frontend. A cancelled pipeline does not publish a partial result.
The language server also checks its request token before and after document
analysis and rejects a result after a document-version change.

The SQL editor reserves these blocking latency budgets:

| Budget | Maximum |
| --- | ---: |
| `perf.lsp.sql.completion` | 200 ms |
| `perf.lsp.sql.hover` | 100 ms |
| `perf.lsp.sql.navigation` | 500 ms |
| `perf.lsp.sql.diagnostics` | 250 ms |
| `perf.lsp.sql.format` | 500 ms |

The SQL platform qualification checks the budget names, limits, cache
identity, ownership, features, fixes, source-map surface, and cancellation
checkpoints. Its mutation test removes required facts and must reject each
changed contract.

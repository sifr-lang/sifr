# SQL incremental analysis and editor architecture

This document records the implemented incremental and editor boundary for
schema-first SQL. The normative SQL design remains in
[`sql_architecture.md`](./sql_architecture.md).

## Ownership

`sifr_frontend::cache_keys` owns the semantic cache identity. The analysis host
loads the resolved SQL profiles from the frozen package graph. It resolves one
schema processor and one query processor from the selected provider package.
The editor uses the same profile authority and provider component as the build.

An embedded SQL key contains the query component request and the normal frontend
context. The request includes these inputs:

- template segments and closed hole types
- fragment identities
- the selected schema profile and dependency-slice fingerprint
- provider identity and component version
- compatibility settings
- component protocol major
- compiler semantic version

The request sent to the provider contains the complete current schema artifact.
The base lookup key excludes that artifact and its complete-schema fingerprint.
After analysis, the provider returns each referenced schema object and its
canonical fingerprint. The final cache key includes this dependency slice. The
analysis cache namespaces the dependencies by profile and maps the stable base
lookup to the slice key. It reuses a result only while all recorded object
fingerprints are unchanged. Thus, an unrelated schema change does not invalidate
the query.

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
HIR. SQL features activate only for a template bound to a SQL profile name, such
as the body of `@profile.query`. An ordinary template remains a Sifr value. A
resolved profile adds live schema and provider semantics; a single-file editor
can still provide lexical SQL behavior before package resolution. Each SQL
document retains the virtual SQL text, hole order, hole types, and source range.

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

The analysis layer routes static SQL positions through the same snapshot as
normal Sifr queries. A cursor inside an interpolation uses Sifr completion,
navigation, references, and rename. SQL hover can still show the parameter's
database type, Sifr type, and nullability. The editor provides:

- syntax highlighting and scoped completion
- hover, definition, references, and rename
- parameter, result, nullability, and cardinality information
- formatting through the Sifr formatter
- structured quick fixes

Completion derives the active relation scope and aliases from the typed SQL
document. It filters both relations and their columns. It resolves a relation
alias before it selects column candidates. The generated identifier reverse
codec converts generated profile names to database identities before symbol
lookup.

Provider diagnostics can supply structured fixes with an exact virtual range,
replacement, title, and detail. The editor also supplies safe fixes for known
alias, cast, missing-column, and bounded-collection diagnostics. Migration
analysis can attach the same structured fix for a supported impact change. The
language server converts the virtual range to an exact Sifr source edit.

A wildcard diagnostic identifies the wildcard at its nested SQL expression. It
does not state that the exported result contains the wildcard. When the schema
has a safe column candidate, the editor offers an explicit-column replacement.

## Cancellation and performance

`run_embedded_provider_items` checks cancellation at these boundaries:

1. before component entry
2. between provider operations
3. before result publication

The language-server message pump signals the active request token immediately,
including while a provider component runs. The analysis host passes that token
to the frontend checkpoints. A cancelled pipeline does not publish a partial
result. The language server also checks its request token before and after
document analysis. It rejects a result after a document-version change.

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

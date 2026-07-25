# Integer Model Serialization Boundary Rules

Status: INT-5 rules lock.

This artifact locks the integer boundary rules that later web, schema, ORM,
and generated serde implementations must satisfy. Runtime JSON profile helpers
live in `sifr_runtime::json`; framework layers may wrap that API, but must not
reimplement or weaken the policy.

The planned external `sifr-lang/pydantic-sifr` package is a consumer of this
contract, not a second integer-schema authority. Its compile-time Core Schema
must provide a general compiler-owned `JsonIntegerBoundaryDescriptor`; the
compiler validates the descriptor and owns `SIFR-INT-0009`, while the package
and native core route execution through `sifr_runtime::json`.

## JSON Profiles

All JSON serializers that can see Sifr `int` values must choose exactly one
integer profile at the boundary:

| Profile | JSON representation | Failure behavior |
| --- | --- | --- |
| `json.exact` | canonical base-10 JSON number | no precision loss; consumers must support exact integer numbers |
| `json.web` | JSON number only for JavaScript-safe integers | values outside `[-9007199254740991, 9007199254740991]` return `JsonIntegerRangeError` unless the field explicitly opts into string encoding |
| `json.string_ints` | decimal JSON string for every integer | no range error for integer magnitude; parser digit limits still apply |

Profile rules apply recursively to nested lists, dictionaries, object fields,
and generated model fields. Error paths use `$` as the root, `[index]` for
array elements, and `.field` or `.key` for object members when available.

## OpenAPI And JSON Schema

Schema generation must reflect the selected integer profile and the static
integer range. It must not emit a browser-facing unbounded integer schema for a
Sifr `int`.

| Sifr field | Selected profile | Schema rules |
| --- | --- | --- |
| `int8`, `int16`, `int32`, `uint8`, `uint16`, `uint32` | `json.web` | `type: integer` with exact `minimum` and `maximum`; may be TypeScript `number` |
| `int64`, `uint64` | `json.web` | decimal string by default: `type: string`, `pattern: "^-?[0-9]+$"`, `x-sifr-format: integer-decimal-string`; numeric schema requires an explicit safe-range constraint |
| `int` | `json.web` | decimal string by default or a typed `JsonIntegerRangeError` policy; numeric schema requires a static safe range |
| any integer | `json.string_ints` | decimal string schema with `x-sifr-format: integer-decimal-string` |
| any integer | `json.exact` | `type: integer`, `x-sifr-integer-profile: exact`, and a generated-client warning unless the client target supports exact integer parsing |

If a route or model lacks enough policy to select one of these mappings, schema
generation must fail closed with `SIFR-INT-0009` and include the field path plus
suggested policies (`json.web` string encoding, explicit safe range,
`json.string_ints`, or exact-client support).

## TypeScript Client Mapping

Generated TypeScript clients must make precision visible in the type:

| Schema shape | TypeScript type |
| --- | --- |
| JavaScript-safe integer number | `number` |
| decimal integer string | branded `SifrDecimalIntString` unless the user opts into plain `string` |
| exact-client integer profile | `bigint` only when the target runtime and JSON parser strategy are explicitly configured |

`int64`, `uint64`, and exact `int` response fields under `json.web` default to
decimal strings. A generated TypeScript `number` for those fields is valid only
when the schema also carries a static safe range.

## Generated Serde

Generated `serde::Serialize` and `serde::Deserialize` implementations for Sifr
classes/structs must route integer fields through the selected JSON integer
profile. They must not derive directly to Rust primitive or `SifrInt` serde
behavior in a way that bypasses profile selection.

Required behavior:

- Public/browser-facing derives default to `json.web`.
- Internal derives must declare `json.exact`, `json.web`, or
  `json.string_ints` explicitly.
- Nested collections and nested model fields inherit the containing profile
  unless a more specific field annotation overrides it.
- Serialization failures return `JsonIntegerRangeError` with the model path.
- Decoder digit limits return `JsonLimitError` before allocating unbounded
  integer values.

## SQL And Storage

Storage schemas must choose representation explicitly. Source-level `int` is an
exact scalar type, not a default database width.

| Storage target | Required Sifr rules |
| --- | --- |
| `SMALLINT`, `INTEGER`, `BIGINT` | fixed-width Sifr field or fallible narrowing from `int` with range checks |
| unsigned dialect column | explicit `uint*` field or fallible narrowing from `int` |
| `NUMERIC`/`DECIMAL` integer scale | exact `int` mapping only when precision and scale constraints are checked |
| text or JSON string column | explicit decimal-string policy |
| binary fixed-width format | fixed-width field plus explicit endian/range policy |

ORM and storage layers must not infer `int64` or `BIGINT` from a plain Sifr
`int` annotation.

## Diagnostic Rules

`SIFR-INT-0009` is the compiler/schema diagnostic for JSON or web-safe integer
serialization policy failures. It is emitted when a compile-time, schema, or
generation step would otherwise create an unsafe or ambiguous integer boundary.
Runtime values that violate a selected JSON profile return
`JsonIntegerRangeError`; untrusted integer token budget violations return
`JsonLimitError`.

Diagnostic payloads must include:

- boundary kind: JSON, OpenAPI, TypeScript, serde, SQL, or binary format
- field/path
- selected or missing profile
- static range when known
- suggested policy alternatives

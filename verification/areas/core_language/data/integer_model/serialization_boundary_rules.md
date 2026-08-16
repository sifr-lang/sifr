# Integer Model Serialization Boundary Rules

Status: INT-5 rules lock.

This artifact locks the integer boundary rules that later web, schema, ORM,
and generated serde implementations must satisfy. Runtime JSON profile helpers
live in `sifr_runtime::json`; framework layers may wrap that API, but must not
reimplement or weaken the policy.

The external `sifr-lang/pydantic-sifr` package consumes this contract. It is not
a second integer-schema authority. The compiler validates the general
`JsonIntegerBoundaryDescriptor` before it seals the Core Schema program and
owns `SIFR-INT-0009`. The implemented native consumer builds one
`SerializationPlan` from that sealed schema and its selected profile.
`TypeAdapter::json_schema` passes the same Core Schema and plan profile to
`pydantic_sifr_core::generate_json_schema`. Runtime output continues to route
through `sifr_runtime::json`.

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
| `int64`, `uint64` | `json.web` | numeric schema only with a statically proven safe range; otherwise schema generation fails with `SIFR-INT-0009` |
| `int` | `json.web` | numeric schema only with a statically proven safe range; otherwise schema generation fails with `SIFR-INT-0009` |
| any integer | `json.string_ints` | decimal string schema with `x-sifr-format: integer-decimal-string` |
| any integer | `json.exact` | `type: integer`, `x-sifr-integer-profile: exact`, and a generated-client warning unless the client target supports exact integer parsing |

If a route or model lacks enough policy to select one of these mappings, schema
generation must fail closed with `SIFR-INT-0009` and include the field path plus
suggested policies (`json.web` string encoding, explicit safe range,
`json.string_ints`, or exact-client support).

`pydantic_sifr_core` exposes the compiler-owned code through
`JsonSchemaError::diagnostic_code` when a `json.web` range is insufficient. It
does not emit the top-level compiler diagnostic. The exact profile emits
`x-sifr-generated-client-warning`; each generated-client backend owns turning
that annotation into its actionable warning and selecting an exact integer
parser. The package does not silently choose a client representation.

The following objects are the implemented bounded serialization-mode snapshots.
Object keys are deterministic. An `int32` under `json.web` is:

```json
{
  "maximum": 2147483647,
  "minimum": -2147483648,
  "type": "integer",
  "x-sifr-integer-profile": "web"
}
```

An `int64` under `json.exact` is:

```json
{
  "maximum": 9223372036854775807,
  "minimum": -9223372036854775808,
  "type": "integer",
  "x-sifr-generated-client-warning": "client must use an exact integer JSON parser for this field",
  "x-sifr-integer-profile": "exact"
}
```

An `int32` under `json.string_ints` is:

```json
{
  "pattern": "^-?[0-9]+$",
  "type": "string",
  "x-sifr-format": "integer-decimal-string",
  "x-sifr-integer-profile": "string_ints",
  "x-sifr-maximum": 2147483647,
  "x-sifr-minimum": -2147483648
}
```

An exact `int` constrained to the JavaScript-safe range under `json.web` is:

```json
{
  "maximum": 9007199254740991,
  "minimum": -9007199254740991,
  "type": "integer",
  "x-sifr-integer-profile": "web"
}
```

Without both safe bounds, the last request returns `IntegerPolicy` with
diagnostic code `SIFR-INT-0009`. Selecting `json.string_ints` is the explicit
decimal-string alternative; `json.web` does not fall back to it.

## TypeScript Client Mapping

Generated TypeScript clients must make precision visible in the type:

| Schema shape | TypeScript type |
| --- | --- |
| JavaScript-safe integer number | `number` |
| decimal integer string | branded `SifrDecimalIntString` unless the user opts into plain `string` |
| exact-client integer profile | `bigint` only when the target runtime and JSON parser strategy are explicitly configured |

`int64`, `uint64`, and exact `int` response fields under `json.web` require a
static safe range. A generated TypeScript `number` is valid only when the schema
carries that range. Select `json.string_ints` explicitly for a decimal-string
wire representation.

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

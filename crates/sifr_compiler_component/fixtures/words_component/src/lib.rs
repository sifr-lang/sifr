use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

wit_bindgen::generate!({
    path: "../../wit",
    world: "embedded-language-provider",
});

struct WordsComponent;

impl Guest for WordsComponent {
    fn analyze(request: Vec<u8>) -> Vec<u8> {
        analyze_request(&request).unwrap_or_else(|| b"not-json".to_vec())
    }
}

export!(WordsComponent);

fn analyze_request(request: &[u8]) -> Option<Vec<u8>> {
    let request: Request = serde_json::from_slice(request).ok()?;
    let [
        Part::Static { text, span: source },
        Part::Hole {
            index,
            span: hole_span,
        },
    ] = request.parts.as_slice()
    else {
        return None;
    };
    let [hole] = request.holes.as_slice() else {
        return None;
    };
    if hole.index != *index || source.document != hole_span.document || source.end > hole_span.start
    {
        return None;
    }
    let token_length = u32::try_from(text.trim_end().len()).ok()?;
    let token_span = Span {
        document: source.document.clone(),
        start: source.start,
        end: source.start.checked_add(token_length)?,
    };
    let mut plan = Plan {
        provider_identity: request.component.processor,
        protocol_major: request.protocol_major,
        plan_kind: request.plan_kind,
        schema_identity: None,
        result_type: ClosedType::Record {
            fields: vec![RecordField {
                name: "token".to_string(),
                ty: hole.ty.clone(),
            }],
        },
        operations: vec![Operation::Sequence {
            operations: vec![
                Operation::Literal {
                    value: text.clone(),
                },
                Operation::Hole { index: *index },
            ],
        }],
        runtime: Runtime::NoRuntime,
        dependencies: vec![Dependency {
            identity: "fixture.dictionary".to_string(),
            fingerprint: hex_digest(Sha256::digest(text.as_bytes()).as_slice()),
        }],
        diagnostics: vec![Diagnostic {
            code: "SIFR-FIXTURE-0001".to_string(),
            severity: DiagnosticSeverity::Note,
            lifecycle: DiagnosticLifecycle::Active,
            message: format!("recognized '{}'", text.trim_end()),
            primary: token_span.clone(),
            related: Vec::new(),
        }],
        source_map: vec![SourceMapEntry {
            provider_start: 0,
            provider_end: token_length,
            source: token_span,
        }],
        stable_fingerprint: String::new(),
    };
    let canonical_plan = serde_json::to_vec(&plan).ok()?;
    plan.stable_fingerprint = hex_digest(Sha256::digest(canonical_plan).as_slice());
    serde_json::to_vec(&Response {
        protocol_major: request.protocol_major,
        plan,
    })
    .ok()
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Deserialize)]
struct Request {
    protocol_major: u16,
    component: ComponentIdentity,
    parts: Vec<Part>,
    holes: Vec<Hole>,
    plan_kind: PlanKind,
}

#[derive(Deserialize)]
struct ComponentIdentity {
    processor: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Part {
    Static { text: String, span: Span },
    Hole { index: u32, span: Span },
}

#[derive(Deserialize)]
struct Hole {
    index: u32,
    ty: ClosedType,
}

#[derive(Clone, Serialize, Deserialize)]
struct Span {
    document: String,
    start: u32,
    end: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ClosedType {
    Bool,
    Int,
    Float,
    Str,
    Bytes,
    None,
    Optional { item: Box<Self> },
    Tuple { items: Vec<Self> },
    List { item: Box<Self> },
    Record { fields: Vec<RecordField> },
}

#[derive(Clone, Serialize, Deserialize)]
struct RecordField {
    name: String,
    ty: ClosedType,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PlanKind {
    Expression,
    Statement,
    Fragment,
    Document,
}

#[derive(Serialize)]
struct Response {
    protocol_major: u16,
    plan: Plan,
}

#[derive(Serialize)]
struct Plan {
    provider_identity: String,
    protocol_major: u16,
    plan_kind: PlanKind,
    schema_identity: Option<String>,
    result_type: ClosedType,
    operations: Vec<Operation>,
    runtime: Runtime,
    dependencies: Vec<Dependency>,
    diagnostics: Vec<Diagnostic>,
    source_map: Vec<SourceMapEntry>,
    stable_fingerprint: String,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Operation {
    Literal { value: String },
    Hole { index: u32 },
    Sequence { operations: Vec<Self> },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Runtime {
    NoRuntime,
}

#[derive(Serialize)]
struct Dependency {
    identity: String,
    fingerprint: String,
}

#[derive(Serialize)]
struct Diagnostic {
    code: String,
    severity: DiagnosticSeverity,
    lifecycle: DiagnosticLifecycle,
    message: String,
    primary: Span,
    related: Vec<Span>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum DiagnosticSeverity {
    Note,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum DiagnosticLifecycle {
    Active,
}

#[derive(Serialize)]
struct SourceMapEntry {
    provider_start: u32,
    provider_end: u32,
    source: Span,
}

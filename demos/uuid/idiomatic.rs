use uuid::Uuid as ExternalUuid;

#[derive(Clone, Debug, PartialEq, Eq)]
struct UUID {
    raw: String,
}

impl UUID {
    fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }

    fn canonical(value: ExternalUuid) -> Self {
        Self::new(value.to_string())
    }

    fn hex(&self) -> String {
        self.raw.chars().filter(|&ch| ch != '-').collect()
    }

    fn to_str(&self) -> &str {
        &self.raw
    }

    fn version(&self) -> i64 {
        self.raw
            .chars()
            .nth(14)
            .and_then(|ch| ch.to_digit(16))
            .map(i64::from)
            .unwrap_or(-1)
    }
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValueError {}

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn uuid4() -> String {
    ExternalUuid::new_v4().to_string()
}

fn uuid4_obj() -> UUID {
    UUID::canonical(ExternalUuid::new_v4())
}

fn normalize_uuid_text(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let without_braces = trimmed
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(trimmed);
    let compact: String = without_braces.chars().filter(|&ch| ch != '-').collect();

    if compact.len() != 32 || !compact.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let lower = compact.to_ascii_lowercase();
    Some(format!(
        "{}-{}-{}-{}-{}",
        &lower[0..8],
        &lower[8..12],
        &lower[12..16],
        &lower[16..20],
        &lower[20..32]
    ))
}

fn uuid_from_hex(input: &str) -> Result<UUID, ValueError> {
    let normalized = normalize_uuid_text(input)
        .ok_or_else(|| ValueError::new("badly formed hexadecimal UUID string"))?;
    let parsed = ExternalUuid::parse_str(&normalized)
        .map_err(|_| ValueError::new("badly formed hexadecimal UUID string"))?;
    Ok(UUID::canonical(parsed))
}

fn namespace_dns() -> UUID {
    UUID::canonical(ExternalUuid::NAMESPACE_DNS)
}

fn parse_uuid(value: &UUID) -> Option<ExternalUuid> {
    normalize_uuid_text(&value.raw).and_then(|text| ExternalUuid::parse_str(&text).ok())
}

fn uuid3(namespace: UUID, name: &str) -> UUID {
    let namespace = parse_uuid(&namespace).unwrap_or(ExternalUuid::nil());
    UUID::canonical(ExternalUuid::new_v3(&namespace, name.as_bytes()))
}

fn uuid5(namespace: UUID, name: &str) -> UUID {
    let namespace = parse_uuid(&namespace).unwrap_or(ExternalUuid::nil());
    UUID::canonical(ExternalUuid::new_v5(&namespace, name.as_bytes()))
}

fn is_canonical_shape(value: &str) -> bool {
    value.len() == 36
        && matches!(value.chars().nth(8), Some('-'))
        && matches!(value.chars().nth(13), Some('-'))
        && matches!(value.chars().nth(18), Some('-'))
        && matches!(value.chars().nth(23), Some('-'))
}

fn collect_generated_actual() -> Vec<bool> {
    let id_text = uuid4();
    let object = uuid4_obj();

    vec![
        is_canonical_shape(&id_text),
        matches!(id_text.chars().nth(14), Some('4')),
        is_canonical_shape(object.to_str()) && object.version() == 4,
    ]
}

fn collect_parse_actual() -> Vec<bool> {
    let parsed_ok = uuid_from_hex("550E8400E29B41D4A716446655440000")
        .map(|uuid| uuid.to_str() == "550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or(false);
    let parsed_v1_ok = uuid_from_hex("550e8400-e29b-11d4-a716-446655440000")
        .map(|uuid| uuid.version() == 1)
        .unwrap_or(false);

    vec![parsed_ok, parsed_v1_ok]
}

fn collect_negative_and_class_actual() -> Vec<bool> {
    let invalid_rejected = uuid_from_hex("invalid").is_err();
    let ctor_passthrough = UUID::new("550e8400-e29b-41d4-a716-44665544000z");
    let ctor_curly_ok = uuid_from_hex("{550E8400-E29B-41D4-A716-446655440000}")
        .map(|uuid| uuid.to_str() == "550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or(false);
    let object = uuid4_obj();

    vec![
        invalid_rejected,
        ctor_passthrough.to_str() == "550e8400-e29b-41d4-a716-44665544000z",
        ctor_curly_ok,
        object.hex().len() == 32,
        uuid3(namespace_dns(), "python.org").version() == 3,
        uuid5(namespace_dns(), "python.org").version() == 5,
    ]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_generated_actual());
    actual.extend(collect_parse_actual());
    actual.extend(collect_negative_and_class_actual());

    assert_bool_vector_eq(
        &actual,
        &[
            true, true, true, true, true, true, true, true, true, true, true,
        ],
    );
    println!("uuid uuid parity demo: pass");
}

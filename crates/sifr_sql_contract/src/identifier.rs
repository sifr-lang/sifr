use crate::{SchemaContractError, SchemaContractErrorKind};

const ESCAPE_PREFIX: &str = "_sifr_sql_";

/// Encode one database identifier as a valid, reversible Sifr identifier.
///
/// Ordinary identifiers remain readable. Reserved words, names that contain
/// the path separator, names with a boundary underscore, and names in the
/// reserved generated namespace use a UTF-8 hex escape. Boundary underscores
/// cannot combine with the path separator, so the path mapping stays
/// injective without a collision table or schema-order-dependent suffix.
pub fn encode_generated_identifier(value: &str) -> Result<String, SchemaContractError> {
    if !valid_identifier(value) {
        return Err(invalid_identifier(value));
    }
    if !is_sifr_keyword(value)
        && !value.starts_with('_')
        && !value.ends_with('_')
        && !value.contains("__")
    {
        return Ok(value.to_string());
    }

    let mut encoded = String::with_capacity(ESCAPE_PREFIX.len() + value.len() * 2);
    encoded.push_str(ESCAPE_PREFIX);
    for byte in value.as_bytes() {
        use std::fmt::Write as _;
        write!(encoded, "{byte:02x}").map_err(|_| serialization_error())?;
    }
    Ok(encoded)
}

/// Decode a Sifr identifier emitted by [`encode_generated_identifier`].
pub fn decode_generated_identifier(value: &str) -> Result<String, SchemaContractError> {
    let decoded = if let Some(payload) = value.strip_prefix(ESCAPE_PREFIX) {
        if payload.is_empty() || payload.len() % 2 != 0 {
            return Err(invalid_encoding(value));
        }
        let bytes = payload
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let high = hex_value(pair[0]).ok_or_else(|| invalid_encoding(value))?;
                let low = hex_value(pair[1]).ok_or_else(|| invalid_encoding(value))?;
                Ok((high << 4) | low)
            })
            .collect::<Result<Vec<_>, SchemaContractError>>()?;
        String::from_utf8(bytes).map_err(|_| invalid_encoding(value))?
    } else {
        value.to_string()
    };

    if !valid_identifier(&decoded) || encode_generated_identifier(&decoded).as_deref() != Ok(value)
    {
        return Err(invalid_encoding(value));
    }
    Ok(decoded)
}

/// Encode a logical generated path. Double underscore is reserved as the
/// segment separator, and the segment codec guarantees that it cannot appear
/// inside an encoded segment.
pub fn encode_generated_path(path: &[String]) -> Result<String, SchemaContractError> {
    if path.is_empty() {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidSchema,
            "generated schema type path must not be empty",
        ));
    }
    path.iter()
        .map(|segment| encode_generated_identifier(segment))
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("__"))
}

/// Decode a path emitted by [`encode_generated_path`].
pub fn decode_generated_path(value: &str) -> Result<Vec<String>, SchemaContractError> {
    if value.is_empty() {
        return Err(invalid_encoding(value));
    }
    let decoded = value
        .split("__")
        .map(decode_generated_identifier)
        .collect::<Result<Vec<_>, _>>()?;
    if encode_generated_path(&decoded).as_deref() != Ok(value) {
        return Err(invalid_encoding(value));
    }
    Ok(decoded)
}

pub(crate) fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}

pub(crate) fn is_sifr_keyword(value: &str) -> bool {
    matches!(
        value,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn invalid_identifier(value: &str) -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::InvalidSchema,
        format!("schema identifier '{value}' cannot be emitted as Sifr"),
    )
}

fn invalid_encoding(value: &str) -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::InvalidSchema,
        format!("generated Sifr identifier '{value}' is not canonical"),
    )
}

fn serialization_error() -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::Serialization,
        "cannot encode generated Sifr identifier",
    )
}

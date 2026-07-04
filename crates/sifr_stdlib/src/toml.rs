#[must_use]
pub const fn feature_name() -> &'static str {
    "toml"
}

const TOML_MAX_BYTES: usize = 1024 * 1024;
const TOML_MAX_DEPTH: usize = 128;
const TOML_MAX_TOKENS: usize = 100_000;

pub fn toml_parse_tokens(text: &str) -> Result<Vec<String>, String> {
    reject_too_large(text)?;
    let table = text
        .parse::<toml::Table>()
        .map_err(|err| format!("TOML parse error: {err}"))?;
    let mut tokens = Vec::new();
    encode_value(&toml::Value::Table(table), 0, &mut tokens)?;
    Ok(tokens)
}

fn reject_too_large(text: &str) -> Result<(), String> {
    if text.len() > TOML_MAX_BYTES {
        return Err("TOML document is too large".to_string());
    }
    Ok(())
}

fn encode_value(value: &toml::Value, depth: usize, tokens: &mut Vec<String>) -> Result<(), String> {
    if depth > TOML_MAX_DEPTH {
        return Err("TOML document nesting is too deep".to_string());
    }
    match value {
        toml::Value::String(value) => push_tag_value(tokens, "str", value),
        toml::Value::Integer(value) => push_tag_value(tokens, "int", &value.to_string()),
        toml::Value::Float(value) => push_tag_value(tokens, "float", &value.to_string()),
        toml::Value::Boolean(value) => {
            push_tag_value(tokens, "bool", if *value { "true" } else { "false" })
        }
        toml::Value::Datetime(value) => push_tag_value(tokens, "datetime", &value.to_string()),
        toml::Value::Array(items) => {
            push_tag_value(tokens, "array", &items.len().to_string())?;
            for item in items {
                encode_value(item, depth + 1, tokens)?;
            }
            Ok(())
        }
        toml::Value::Table(items) => {
            push_tag_value(tokens, "table", &items.len().to_string())?;
            for (key, item) in items {
                push_token(tokens, key)?;
                encode_value(item, depth + 1, tokens)?;
            }
            Ok(())
        }
    }
}

fn push_tag_value(tokens: &mut Vec<String>, tag: &str, value: &str) -> Result<(), String> {
    push_token(tokens, tag)?;
    push_token(tokens, value)
}

fn push_token(tokens: &mut Vec<String>, value: &str) -> Result<(), String> {
    if tokens.len() >= TOML_MAX_TOKENS {
        return Err("TOML bridge payload is too large".to_string());
    }
    tokens.push(value.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::toml_parse_tokens;

    #[test]
    fn toml_leaf_encodes_table_event_stream() {
        let Ok(tokens) = toml_parse_tokens("title = \"Demo\"\ncount = 2\nflags = [true, false]\n")
        else {
            panic!("TOML should parse");
        };

        assert_eq!(tokens[0], "table");
        assert!(tokens.windows(2).any(|pair| pair == ["title", "str"]));
        assert!(tokens.windows(2).any(|pair| pair == ["count", "int"]));
        assert!(tokens.windows(2).any(|pair| pair == ["flags", "array"]));
    }

    #[test]
    fn toml_leaf_reports_parse_errors() {
        let Err(err) = toml_parse_tokens("title = ") else {
            panic!("invalid TOML should fail");
        };

        assert!(err.contains("TOML parse error"));
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
struct ParsingError {
    message: String,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ParsingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonValue {
    Int(i64),
    Str(String),
    Object(Vec<(String, JsonValue)>),
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Int(value) => write!(f, "{value}"),
            JsonValue::Str(value) => write!(f, "{}", serde_json::Value::String(value.clone())),
            JsonValue::Object(entries) => {
                let body = entries
                    .iter()
                    .map(|(key, value)| {
                        format!("{}:{value}", serde_json::Value::String(key.clone()))
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "{{{body}}}")
            }
        }
    }
}

fn from_int(value: i64) -> JsonValue {
    JsonValue::Int(value)
}

fn from_str(value: &str) -> JsonValue {
    JsonValue::Str(value.to_string())
}

fn from_object(values: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

#[derive(Debug, Clone)]
struct JSONEncoder {
    indent: usize,
}

impl JSONEncoder {
    fn encode(&self, value: &JsonValue) -> String {
        let _ = self.indent;
        value.to_string()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct JSONDecoder;

impl JSONDecoder {
    fn decode(&self, text: &str) -> Result<JsonValue, Error> {
        let parsed = serde_json::from_str::<serde_json::Value>(text).map_err(|error| Error {
            message: error.to_string(),
        })?;
        convert_json_value(parsed)
    }
}

fn convert_json_value(value: serde_json::Value) -> Result<JsonValue, Error> {
    match value {
        serde_json::Value::String(text) => Ok(JsonValue::Str(text)),
        serde_json::Value::Number(number) => number.as_i64().map(JsonValue::Int).ok_or(Error {
            message: "only integer JSON numbers are supported".to_string(),
        }),
        serde_json::Value::Object(entries) => {
            let mut values = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                values.push((key, convert_json_value(value)?));
            }
            Ok(JsonValue::Object(values))
        }
        _ => Err(Error {
            message: "unsupported JSON value".to_string(),
        }),
    }
}

#[derive(Debug, Clone)]
struct ConfigParser {
    defaults: HashMap<String, String>,
    sections: HashMap<String, HashMap<String, String>>,
}

impl ConfigParser {
    fn new() -> Self {
        Self {
            defaults: HashMap::new(),
            sections: HashMap::new(),
        }
    }

    fn read_string(&mut self, text: &str) -> Result<(), ParsingError> {
        let mut current_section = "DEFAULT".to_string();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(section) = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
            {
                current_section = section.to_string();
                if current_section != "DEFAULT" {
                    self.sections.entry(current_section.clone()).or_default();
                }
                continue;
            }

            let (key, value) = line.split_once('=').ok_or_else(|| ParsingError {
                message: format!("invalid config line: {line}"),
            })?;
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if current_section == "DEFAULT" {
                self.defaults.insert(key, value);
            } else {
                self.sections
                    .entry(current_section.clone())
                    .or_default()
                    .insert(key, value);
            }
        }

        Ok(())
    }

    fn get(&self, section: &str, key: &str) -> String {
        let raw = self
            .sections
            .get(section)
            .and_then(|values| values.get(key))
            .cloned()
            .unwrap_or_default();
        self.interpolate(section, &raw)
    }

    fn interpolate(&self, section: &str, value: &str) -> String {
        let mut rendered = value.to_string();
        while let Some(start) = rendered.find("%(") {
            let Some(end) = rendered[start + 2..].find(")s") else {
                break;
            };
            let end = start + 2 + end;
            let key = &rendered[start + 2..end];
            let replacement = self
                .sections
                .get(section)
                .and_then(|values| values.get(key))
                .or_else(|| self.defaults.get(key))
                .cloned()
                .unwrap_or_default();
            rendered.replace_range(start..end + 2, &replacement);
        }
        rendered
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Dialect {
    delimiter: char,
}

impl Dialect {
    fn new(delimiter: char) -> Self {
        Self { delimiter }
    }
}

#[derive(Debug, Clone, Default)]
struct DialectRegistry {
    values: HashMap<String, Dialect>,
}

impl DialectRegistry {
    fn register(&mut self, name: &str, dialect: Dialect) {
        self.values.insert(name.to_string(), dialect);
    }

    fn get(&self, name: &str) -> Option<Dialect> {
        self.values.get(name).cloned()
    }

    fn unregister(&mut self, name: &str) -> bool {
        self.values.remove(name).is_some()
    }
}

fn dialect_registry() -> DialectRegistry {
    DialectRegistry::default()
}

#[derive(Debug, Clone)]
struct Reader {
    text: String,
    dialect: Dialect,
}

impl Reader {
    fn rows(&self) -> Vec<Vec<String>> {
        self.text
            .lines()
            .map(|line| {
                line.split(self.dialect.delimiter)
                    .map(ToString::to_string)
                    .collect()
            })
            .collect()
    }
}

fn reader(text: &str, dialect: Dialect) -> Reader {
    Reader {
        text: text.to_string(),
        dialect,
    }
}

fn main() {
    let encoder = JSONEncoder { indent: 2 };
    let decoder = JSONDecoder;

    let payload = from_object(vec![
        ("module", from_str("config_json_csv")),
        ("version", from_int(1)),
    ]);
    let encoded = encoder.encode(&payload);
    assert_eq!(encoded, "{\"module\":\"config_json_csv\",\"version\":1}");

    let decoded_ok = decoder
        .decode(&encoded)
        .is_ok_and(|decoded_value| decoded_value.to_string() == encoded);
    assert!(decoded_ok);

    let mut parser = ConfigParser::new();
    parser
        .read_string("[DEFAULT]\nbase=/tmp\n[paths]\ncache=%(base)s/cache\n")
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(parser.get("paths", "cache"), "/tmp/cache");

    let mut registry = dialect_registry();
    registry.register("pipe", Dialect::new('|'));
    let dialect = registry.get("pipe");
    assert!(dialect.is_some());
    if let Some(dialect) = dialect {
        let rows = reader("a|b\n1|2", dialect).rows();
        assert_eq!(
            format!("{rows:?}").replace('"', "\""),
            "[[\"a\", \"b\"], [\"1\", \"2\"]]"
        );
    }
    assert!(registry.unregister("pipe"));
}

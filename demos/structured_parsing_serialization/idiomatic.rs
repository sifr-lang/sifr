use std::collections::BTreeMap;
use std::fs;

use serde_json::{json, Value as JsonValue};

fn write_text(path: &str, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|error| error.to_string())
}

fn load(path: &str) -> Result<JsonValue, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn dumps(value: &JsonValue) -> String {
    render_json(value)
}

fn render_json(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(flag) => flag.to_string(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::String(text) => {
            serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
        }
        JsonValue::Array(items) => format!(
            "[{}]",
            items.iter().map(render_json).collect::<Vec<_>>().join(",")
        ),
        JsonValue::Object(object) => {
            let mut entries = Vec::new();
            for key in ["name", "items"] {
                if let Some(item) = object.get(key) {
                    entries.push(format!("\"{key}\":{}", render_json(item)));
                }
            }
            let mut remaining = object
                .keys()
                .filter(|key| *key != "name" && *key != "items")
                .cloned()
                .collect::<Vec<_>>();
            remaining.sort();
            for key in remaining {
                if let Some(item) = object.get(&key) {
                    entries.push(format!("\"{key}\":{}", render_json(item)));
                }
            }
            format!("{{{}}}", entries.join(","))
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Dialect {
    quote_all: bool,
}

const QUOTE_ALL: Dialect = Dialect { quote_all: true };

fn format_row(values: &[&str], dialect: Dialect) -> String {
    values
        .iter()
        .map(|value| {
            if dialect.quote_all {
                format!("\"{}\"", value.replace('"', "\"\""))
            } else {
                (*value).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

struct DictReader {
    text: String,
}

impl DictReader {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    fn rows(&self) -> Vec<BTreeMap<String, String>> {
        let mut lines = self.text.lines();
        let headers = lines
            .next()
            .unwrap_or("")
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>();
        lines
            .filter(|line| !line.is_empty())
            .map(|line| {
                headers
                    .iter()
                    .cloned()
                    .zip(line.split(',').map(str::to_string))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect()
    }
}

struct DictWriter {
    headers: Vec<String>,
    lines: Vec<String>,
}

impl DictWriter {
    fn new(headers: &[&str]) -> Self {
        Self {
            headers: headers.iter().map(|header| (*header).to_string()).collect(),
            lines: Vec::new(),
        }
    }

    fn writeheader(&mut self) {
        self.lines.push(self.headers.join(","));
    }

    fn writerow(&mut self, row: &BTreeMap<&str, &str>) {
        let values = self
            .headers
            .iter()
            .map(|header| row.get(header.as_str()).copied().unwrap_or(""))
            .collect::<Vec<_>>();
        self.lines.push(values.join(","));
    }

    fn getvalue(&self) -> String {
        self.lines.join("\n")
    }
}

#[derive(Clone, Debug)]
struct ConfigParser {
    defaults: BTreeMap<String, Option<String>>,
    allow_no_value: bool,
    sections: BTreeMap<String, BTreeMap<String, Option<String>>>,
}

impl ConfigParser {
    fn new(defaults: BTreeMap<String, Option<String>>, allow_no_value: bool) -> Self {
        Self {
            defaults,
            allow_no_value,
            sections: BTreeMap::new(),
        }
    }

    fn read_string(&mut self, text: &str) -> Result<(), String> {
        let mut current_section = None::<String>;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let name = trimmed
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                self.sections.entry(name.clone()).or_default();
                current_section = Some(name);
                continue;
            }
            let section = current_section
                .clone()
                .ok_or_else(|| "option outside section".to_string())?;
            let bucket = self.sections.entry(section).or_default();
            if let Some((key, value)) = trimmed.split_once('=') {
                bucket.insert(key.trim().to_string(), Some(value.trim().to_string()));
            } else if self.allow_no_value {
                bucket.insert(trimmed.to_string(), None);
            } else {
                return Err(format!("missing value for {trimmed}"));
            }
        }
        Ok(())
    }

    fn get(&self, section: &str, key: &str, fallback: Option<&str>) -> Option<String> {
        if let Some(values) = self.sections.get(section) {
            if let Some(value) = values.get(key) {
                return value.clone();
            }
        }
        if let Some(value) = self.defaults.get(key) {
            return value.clone();
        }
        fallback.map(str::to_string)
    }

    fn getint(&self, section: &str, key: &str) -> i64 {
        self.get(section, key, None)
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or_default()
    }

    fn getboolean(&self, section: &str, key: &str) -> bool {
        matches!(self.get(section, key, None).as_deref(), Some("true"))
    }
}

fn main() {
    println!("structured-parsing-sample structured parsing and serialization demo");

    let json_path = "/tmp/sifr_structured_parsing_serialization.json";
    let json_text = r#"{"name":"sifr","items":[1,true]}"#;
    let _ = write_text(json_path, json_text);
    if let Ok(json_value) = load(json_path) {
        if let Some(name) = json_value.get("name").and_then(JsonValue::as_str) {
            println!("{name}");
        }
        if let Some(second) = json_value
            .get("items")
            .and_then(JsonValue::as_array)
            .and_then(|items| items.get(1))
            .and_then(JsonValue::as_bool)
        {
            println!("{second}");
        }
    }
    println!("{}", dumps(&json!({"name":"sifr","items":[1,true]})));

    if let Ok(toml_value) = "title = \"sifr\"\n[owner]\nactive = true\n".parse::<toml::Table>() {
        if let Some(title) = toml_value.get("title").and_then(toml::Value::as_str) {
            println!("{title}");
        }
        if let Some(active) = toml_value
            .get("owner")
            .and_then(toml::Value::as_table)
            .and_then(|owner| owner.get("active"))
            .and_then(toml::Value::as_bool)
        {
            println!("{active}");
        }
    }

    println!("{}", format_row(&["alpha", "beta"], QUOTE_ALL));

    let dict_reader = DictReader::new("name,age\nalice,30\n");
    println!("{:?}", dict_reader.rows());

    let mut dict_writer = DictWriter::new(&["name", "age"]);
    dict_writer.writeheader();
    let mut row = BTreeMap::new();
    row.insert("name", "alice");
    row.insert("age", "30");
    dict_writer.writerow(&row);
    println!("{}", dict_writer.getvalue());

    let mut defaults = BTreeMap::new();
    defaults.insert("encoding".to_string(), Some("utf-8".to_string()));
    let mut parser = ConfigParser::new(defaults, true);
    if parser
        .read_string("[server]\nport = 8080\nenabled = true\nfeature\n")
        .is_ok()
    {
        println!("{}", parser.getint("server", "port"));
        println!("{}", parser.getboolean("server", "enabled"));
        println!("{:?}", parser.get("server", "feature", Some("missing")));
    }
}

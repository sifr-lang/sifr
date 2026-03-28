// --- stdlib: sifr.json ---
#[derive(Debug, Clone, PartialEq)]
struct JsonValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    array_items: Box<Vec<JsonValue>>,
    object_items: Box<Vec<(String, JsonValue)>>,
}
impl JsonValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            array_items: Box::new(vec![]),
            object_items: Box::new(vec![]),
        };
    }
    fn is_null(&self) -> bool {
        return self.kind.clone() == "null".to_string();
    }
    fn is_bool(&self) -> bool {
        return self.kind.clone() == "bool".to_string();
    }
    fn is_int(&self) -> bool {
        return self.kind.clone() == "int".to_string();
    }
    fn is_float(&self) -> bool {
        return self.kind.clone() == "float".to_string();
    }
    fn is_str(&self) -> bool {
        return self.kind.clone() == "str".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_object(&self) -> bool {
        return self.kind.clone() == "object".to_string();
    }
    fn as_bool(&self) -> Option<bool> {
        return self.bool_value;
    }
    fn as_int(&self) -> Option<i64> {
        return self.int_value;
    }
    fn as_float(&self) -> Option<f64> {
        return self.float_value;
    }
    fn as_str(&self) -> Option<String> {
        return self.str_value.clone();
    }
    fn as_array(&self) -> Option<Vec<JsonValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<JsonValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_object(&self) -> Option<Vec<(String, JsonValue)>> {
        if !(self.is_object()) {
            return None;
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<JsonValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64)) || (index >= ((self.array_items).as_ref().clone().len() as i64))) {
            return None;
        }
        let value: Option<JsonValue> = {
            let __sifr_index_list = &self.array_items;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        return value;
    }
    fn get(&self, key: &String) -> Option<JsonValue> {
        if !(self.is_object()) {
            return None;
        }
        for (item_key, item_value) in (self.object_items).as_ref().clone().iter().cloned() {
            if item_key == *key {
                return Some(item_value);
            }
        }
        return None;
    }
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (item_key, _item_value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<JsonValue> {
        let mut result: Vec<JsonValue> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (_item_key, item_value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, JsonValue)> {
        if !(self.is_object()) {
            return vec![];
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", {
            let __json_value = self;
            fn __sifr_json_value_to_serde(value: &JsonValue) -> serde_json::Value {
                match value.kind.as_str() {
                    "null" => {
                        return serde_json::Value::Null;
                    }
                    "bool" => {
                        if let Some(v) = value.bool_value {
                            return serde_json::Value::from(v);
                        }
                        return serde_json::Value::Null;
                    }
                    "int" => {
                        if let Some(v) = value.int_value {
                            return serde_json::Value::from(v);
                        }
                        return serde_json::Value::Null;
                    }
                    "float" => {
                        if let Some(v) = value.float_value {
                            return serde_json::Value::from(v);
                        }
                        return serde_json::Value::Null;
                    }
                    "str" => {
                        if let Some(v) = value.str_value.clone() {
                            return serde_json::Value::String(v);
                        }
                        return serde_json::Value::Null;
                    }
                    "array" => {
                        let mut converted = vec![];
                        for item in value.array_items.as_ref().iter().cloned() {
                            converted.push(__sifr_json_value_to_serde(&item));
                        }
                        return serde_json::Value::Array(converted);
                    }
                    "object" => {
                        let mut converted = serde_json::Map::new();
                        for entry in value.object_items.as_ref().iter().cloned() {
                            let entry_key = entry.0;
                            let entry_value = entry.1;
                            converted.insert(entry_key, __sifr_json_value_to_serde(&entry_value));
                        }
                        return serde_json::Value::Object(converted);
                    }
                    _ => {
                        return serde_json::Value::Null;
                    }
                }
            }
            serde_json::to_string(&__sifr_json_value_to_serde(&__json_value))
                .unwrap_or_else(|_err| "null".to_string().to_string())
        });
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            detail: String::new(),
        };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {}

fn main() {
    println!("=== sifr.io ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w: () = std::fs::write(
            &"/tmp/sifr_demo.txt".to_string(),
            "Hello from Sifr!\nLine 2".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("write error: {}", err.message);
    }
    let file_exists: bool = std::path::Path::new(&"/tmp/sifr_demo.txt".to_string()).exists();
    println!("File exists: {}", file_exists);
    let __sifr_try_res: Result<(), IOError> = (|| {
        let content: String =
            std::fs::read_to_string(&"/tmp/sifr_demo.txt".to_string()).map_err(__io_err)?;
        println!("Content: {}", content);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("read error: {}", err.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let lines: Vec<String> = std::fs::read_to_string(&"/tmp/sifr_demo.txt".to_string())
            .map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<String>>())
            .map_err(__io_err)?;
        println!("Line count: {}", lines.len() as i64);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("lines error: {}", err.message);
    }
    println!("=== sifr.json ===");
    let __sifr_try_res: Result<(), JSONDecodeError> = (|| {
        let data: JsonValue = ({
            let __json_input = "{\"language\":\"sifr\",\"version\":1}".to_string();
            fn __sifr_json_value_from_serde(
                value: serde_json::Value,
            ) -> Result<JsonValue, JSONDecodeError> {
                match value {
                    serde_json::Value::Null => {
                        return Ok(JsonValue {
                            kind: "null".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Bool(b) => {
                        return Ok(JsonValue {
                            kind: "bool".to_string().to_string(),
                            bool_value: Some(b),
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            return Ok(JsonValue {
                                kind: "int".to_string().to_string(),
                                bool_value: None,
                                int_value: Some(i),
                                float_value: None,
                                str_value: None,
                                array_items: Box::new(vec![]),
                                object_items: Box::new(vec![]),
                            });
                        }
                        if n.is_u64() {
                            return Err(JSONDecodeError {
                                message: "json integer out of range for sifr int"
                                    .to_string()
                                    .to_string(),
                                line: 0,
                                column: 0,
                            });
                        }
                        if let Some(f) = n.as_f64() {
                            return Ok(JsonValue {
                                kind: "float".to_string().to_string(),
                                bool_value: None,
                                int_value: None,
                                float_value: Some(f),
                                str_value: None,
                                array_items: Box::new(vec![]),
                                object_items: Box::new(vec![]),
                            });
                        }
                        return Err(JSONDecodeError {
                            message: "unsupported json number representation"
                                .to_string()
                                .to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    serde_json::Value::String(s) => {
                        return Ok(JsonValue {
                            kind: "str".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: Some(s),
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Array(items) => {
                        let mut converted = vec![];
                        for item in items {
                            converted.push(__sifr_json_value_from_serde(item)?);
                        }
                        return Ok(JsonValue {
                            kind: "array".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(converted),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Object(entries) => {
                        let mut converted = vec![];
                        for entry in entries {
                            let entry_key = entry.0;
                            let entry_value = entry.1;
                            let converted_value = __sifr_json_value_from_serde(entry_value)?;
                            converted.push((entry_key, converted_value));
                        }
                        return Ok(JsonValue {
                            kind: "object".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(converted),
                        });
                    }
                }
            }
            serde_json::from_str::<serde_json::Value>(__json_input.as_ref())
                .map_err(|e| JSONDecodeError {
                    message: e.to_string(),
                    line: e.line() as i64,
                    column: e.column() as i64,
                })
                .and_then(|parsed| __sifr_json_value_from_serde(parsed))
        })?;
        println!("Parsed JSON: {}", data);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("json error: {}", err.message);
    }
    println!("=== sifr.env ===");
    let _: () = {
        let __k = "SIFR_DEMO".to_string();
        let __v = "active".to_string();
        if !__k.is_empty()
            && (!__k.contains('=')
                && (!__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0)))
        {
            std::env::set_var(__k, __v);
        }
    };
    let val: Option<String> = {
        let __k = "SIFR_DEMO".to_string();
        if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) {
            None
        } else {
            std::env::var(__k).ok()
        }
    };
    if let Some(val) = val {
        println!("SIFR_DEMO = {}", val);
    }
    let missing: Option<String> = {
        let __k = "SIFR_NONEXISTENT".to_string();
        if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) {
            None
        } else {
            std::env::var(__k).ok()
        }
    };
    if missing.is_none() {
        println!("SIFR_NONEXISTENT not set");
    }
    println!("=== sifr.os ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let output: String = ({
            let __cmd = "echo Sifr OS module works".to_string();
            let __output = std::process::Command::new("sh".to_string())
                .arg("-c".to_string())
                .arg(&__cmd)
                .output()
                .map_err(__io_err)?;
            Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
        })?;
        println!("{}", output);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("os error: {}", err.message);
    }
    println!("=== sifr.math ===");
    println!("sqrt(25.0) = {}", (25.0 as f64).sqrt());
    println!("floor(3.7) = {}", (3.7 as f64).floor() as i64);
    println!("ceil(3.2) = {}", (3.2 as f64).ceil() as i64);
    println!("pi = {}", std::f64::consts::PI);
    println!("e = {}", std::f64::consts::E);
    println!("=== Demo complete ===");
}

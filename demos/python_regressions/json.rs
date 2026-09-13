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
                .unwrap_or_else(|_err| "null".to_string())
        });
    }
}

use std::collections::HashMap;

// --- stdlib: sifr.tomllib ---
#[derive(Debug, Clone, PartialEq)]
struct TomlValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    datetime_value: Option<String>,
    array_items: Box<Vec<TomlValue>>,
    table_items: Box<Vec<(String, TomlValue)>>,
}
impl TomlValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
        datetime_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            datetime_value: datetime_value,
            array_items: Box::new(vec![]),
            table_items: Box::new(vec![]),
        };
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
    fn is_datetime(&self) -> bool {
        return self.kind.clone() == "datetime".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_table(&self) -> bool {
        return self.kind.clone() == "table".to_string();
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
    fn as_datetime(&self) -> Option<String> {
        return self.datetime_value.clone();
    }
    fn as_array(&self) -> Option<Vec<TomlValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<TomlValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_table(&self) -> Option<Vec<(String, TomlValue)>> {
        if !(self.is_table()) {
            return None;
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<TomlValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
            return None;
        }
        let value: Option<TomlValue> = {
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
    fn get(&self, key: &String) -> Option<TomlValue> {
        if !(self.is_table()) {
            return None;
        }
        for (item_key, item_value) in (self.table_items).as_ref().clone().iter().cloned()
        {
            if item_key == *key {
                return Some(item_value);
            }
        }
        return None;
    }
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (item_key, _item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<TomlValue> {
        let mut result: Vec<TomlValue> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (_item_key, item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, TomlValue)> {
        if !(self.is_table()) {
            return vec![];
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
fn loads(text: &String) -> Result<TomlValue, TOMLDecodeError> {
    return {
        let __toml_input = &text;
        fn __sifr_toml_value_from_parsed(
            value: toml::Value,
        ) -> Result<TomlValue, TOMLDecodeError> {
            match value {
                toml::Value::Boolean(v) => {
                    return Ok(TomlValue {
                        kind: "bool".to_string().to_string(),
                        bool_value: Some(v),
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Integer(v) => {
                    return Ok(TomlValue {
                        kind: "int".to_string().to_string(),
                        bool_value: None,
                        int_value: Some(v),
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Float(v) => {
                    return Ok(TomlValue {
                        kind: "float".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: Some(v),
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::String(v) => {
                    return Ok(TomlValue {
                        kind: "str".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: Some(v),
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Datetime(v) => {
                    return Ok(TomlValue {
                        kind: "datetime".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: Some(v.to_string()),
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Array(items) => {
                    let mut converted = vec![];
                    for item in items {
                        converted.push(__sifr_toml_value_from_parsed(item)?);
                    }
                    return Ok(TomlValue {
                        kind: "array".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(converted),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Table(items) => {
                    let mut converted = vec![];
                    for entry in items {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        let converted_value = __sifr_toml_value_from_parsed(
                            entry_value,
                        )?;
                        converted.push((entry_key, converted_value));
                    }
                    return Ok(TomlValue {
                        kind: "table".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(converted),
                    });
                }
            }
        }
        __toml_input
            .parse::<toml::Value>()
            .map_err(|e| TOMLDecodeError {
                message: e.to_string(),
                line: 0,
                column: 0,
            })
            .and_then(|parsed| __sifr_toml_value_from_parsed(parsed))
    };
}

// --- stdlib: sifr.csv ---
const QUOTE_ALL: i64 = 1 as i64;
const QUOTE_NONNUMERIC: i64 = 2 as i64;
const QUOTE_NONE: i64 = 3 as i64;
const QUOTE_STRINGS: i64 = 4 as i64;
const QUOTE_NOTNULL: i64 = 5 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dialect {
    delimiter: String,
    quotechar: String,
    escapechar: String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: i64,
}
impl Dialect {
    fn new(
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut resolved_quoting: i64 = quoting;
        _validate_char(&"delimiter".to_string(), &delimiter);
        if quotechar != "".to_string() {
            _validate_char(&"quotechar".to_string(), &quotechar);
        }
        if escapechar != "".to_string() {
            _validate_char(&"escapechar".to_string(), &escapechar);
        }
        if (quotechar == "".to_string()) && (resolved_quoting != QUOTE_NONE) {
            resolved_quoting = QUOTE_NONE;
        }
        return Self {
            delimiter: delimiter,
            quotechar: quotechar,
            escapechar: escapechar,
            doublequote: doublequote,
            skipinitialspace: skipinitialspace,
            lineterminator: lineterminator,
            quoting: resolved_quoting,
        };
    }
}
impl std::fmt::Display for Dialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
            self.delimiter, self.quotechar, self.escapechar, self.doublequote, self
            .skipinitialspace, self.lineterminator, self.quoting
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct writer {
    _rows: Vec<Vec<String>>,
    dialect: Dialect,
}
impl writer {
    fn new(
        dialect: Option<Dialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: Dialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &lineterminator,
            quoting,
        );
        return Self {
            dialect: resolved_dialect,
            _rows: vec![],
        };
    }
    fn writerow(&mut self, row: &Vec<String>) {
        let mut copied: Vec<String> = vec![];
        for value in row.iter().cloned() {
            copied.push(value);
        }
        self._rows.push(copied);
    }
    fn writerows(&mut self, rows: &Vec<Vec<String>>) {
        for row in rows.iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for value in row.iter().cloned() {
                copied.push(format!("{}{}", value, "".to_string()));
            }
            self._rows.push(copied);
        }
    }
    fn getvalue(&self) -> String {
        return format_csv(
            &self._rows.clone(),
            &Some(self.dialect.clone()),
            &",".to_string(),
            &"\"".to_string(),
            &"".to_string(),
            true,
            false,
            &"\n".to_string(),
            0 as i64,
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct DictReader {
    _fieldnames: Vec<String>,
    _rows: Vec<Vec<String>>,
    _pos: i64,
    restkey: String,
    restval: String,
    dialect: Dialect,
}
impl DictReader {
    fn new(
        text: String,
        fieldnames: Option<Vec<String>>,
        restkey: String,
        restval: String,
        dialect: Option<Dialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: Dialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting,
        );
        let all_rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, "".to_string()),
            &format!("{}{}", resolved_dialect.quotechar, "".to_string()),
            &format!("{}{}", resolved_dialect.escapechar, "".to_string()),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        let mut fieldnames_data: Vec<String> = vec![];
        let mut rows_data: Vec<Vec<String>> = vec![];
        if let Some(fieldnames) = fieldnames {
            for field in fieldnames.iter().cloned() {
                fieldnames_data.push(format!("{}{}", field, "".to_string()));
            }
            for row in all_rows.iter().cloned() {
                let mut copied_row: Vec<String> = vec![];
                for value in row.iter().cloned() {
                    copied_row.push(format!("{}{}", value, "".to_string()));
                }
                rows_data.push(copied_row);
            }
        } else {
            for (index, row) in Box::new(
                (all_rows)
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if index == (0 as i64) {
                    for field in row.iter().cloned() {
                        fieldnames_data.push(format!("{}{}", field, "".to_string()));
                    }
                } else {
                    let mut copied_row2: Vec<String> = vec![];
                    for value in row.iter().cloned() {
                        copied_row2.push(format!("{}{}", value, "".to_string()));
                    }
                    rows_data.push(copied_row2);
                }
            }
        }
        return Self {
            dialect: resolved_dialect,
            restkey: restkey,
            restval: restval,
            _pos: 0 as i64,
            _fieldnames: fieldnames_data,
            _rows: rows_data,
        };
    }
    fn fieldnames(&self) -> Vec<String> {
        let mut copied: Vec<String> = vec![];
        for field in self._fieldnames.clone().iter().cloned() {
            copied.push(format!("{}{}", field, "".to_string()));
        }
        return copied;
    }
    fn __next__(&mut self) -> Option<HashMap<String, String>> {
        while self._pos < (self._rows.clone().len() as i64) {
            let row: Option<Vec<String>> = {
                let __sifr_index_list = &self._rows;
                let __sifr_index_i = self._pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            self._pos = self._pos + (1 as i64);
            let Some(row) = row else {
                return None;
            };
            if (row.len() as i64) == (0 as i64) {
                continue;
            }
            return Some(
                _dict_reader_row(
                    &self._fieldnames.clone(),
                    &row,
                    &self.restkey.clone(),
                    &self.restval.clone(),
                ),
            );
        }
        return None;
    }
    fn rows(&self) -> Vec<HashMap<String, String>> {
        let mut result: Vec<HashMap<String, String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            if (row.len() as i64) == (0 as i64) {
                continue;
            }
            result
                .push(
                    _dict_reader_row(
                        &self._fieldnames.clone(),
                        &row,
                        &self.restkey.clone(),
                        &self.restval.clone(),
                    ),
                );
        }
        return result;
    }
}
#[derive(Debug, Clone, PartialEq)]
struct DictWriter {
    fieldnames: Vec<String>,
    restval: String,
    extrasaction: String,
    _writer: writer,
}
impl DictWriter {
    fn new(
        fieldnames: Vec<String>,
        restval: String,
        extrasaction: String,
        dialect: Option<Dialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut fieldnames_data: Vec<String> = vec![];
        for field in fieldnames.iter().cloned() {
            fieldnames_data.push(format!("{}{}", field, "".to_string()));
        }
        let mut action: String = extrasaction.to_lowercase();
        if (action != "raise".to_string()) && (action != "ignore".to_string()) {
            action = "raise".to_string();
        }
        let writer_value: writer = writer::new(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting,
        );
        return Self {
            fieldnames: fieldnames_data,
            restval: restval,
            extrasaction: action,
            _writer: writer_value,
        };
    }
    fn writeheader(&mut self) {
        let mut current_writer: writer = self._writer.clone();
        current_writer.writerow(&self.fieldnames.clone());
        self._writer = current_writer;
    }
    fn writerow(&mut self, row: &HashMap<String, String>) {
        let mut ordered: Vec<String> = vec![];
        for fieldname in self.fieldnames.clone().iter().cloned() {
            if row.contains_key(&fieldname) {
                ordered.push(_dict_value_at(row, &fieldname));
            } else {
                ordered.push(self.restval.clone());
            }
        }
        let mut current_writer: writer = self._writer.clone();
        current_writer.writerow(&ordered);
        self._writer = current_writer;
    }
    fn writerows(&mut self, rows: &Vec<HashMap<String, String>>) {
        let mut current_writer: writer = self._writer.clone();
        for row in rows.iter().cloned() {
            let mut ordered: Vec<String> = vec![];
            for fieldname in self.fieldnames.clone().iter().cloned() {
                if row.contains_key(&fieldname) {
                    ordered.push(_dict_value_at(&row, &fieldname));
                } else {
                    ordered.push(self.restval.clone());
                }
            }
            current_writer.writerow(&ordered);
        }
        self._writer = current_writer;
    }
    fn getvalue(&mut self) -> String {
        return self._writer.clone().getvalue();
    }
}
fn _copy_dialect(dialect: &Dialect) -> Dialect {
    return Dialect::new(
        format!("{}{}", dialect.delimiter, "".to_string()),
        format!("{}{}", dialect.quotechar, "".to_string()),
        format!("{}{}", dialect.escapechar, "".to_string()),
        dialect.doublequote,
        dialect.skipinitialspace,
        format!("{}{}", dialect.lineterminator, "".to_string()),
        dialect.quoting,
    );
}
fn _validate_char(name: &String, value: &String) {
    let _: String = (name).clone();
    let _: String = (value).clone();
}
fn _resolve_dialect(
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> Dialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    return Dialect::new(
        (delimiter).clone(),
        (quotechar).clone(),
        (escapechar).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator).clone(),
        quoting,
    );
}
fn _quotechar_value(dialect: &Dialect) -> String {
    let quotechar: String = format!("{}{}", dialect.quotechar, "".to_string());
    if quotechar == "".to_string() {
        return "\"".to_string();
    }
    return quotechar;
}
fn _append_field(row: &mut Vec<String>, field: String) {
    row.push(format!("{}{}", field, "".to_string()));
}
fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    rows.push(row);
}
fn _char_at(text: &String, index: i64) -> String {
    if ((index < (0 as i64)) || (index >= (text.chars().count() as i64))) {
        return "".to_string();
    }
    let ch: Option<String> = Some({
        let Some(__indexed_char) = text.chars().nth(index as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    });
    let Some(ch) = ch else {
        return "".to_string();
    };
    return ch;
}
fn _list_value_at(values: &Vec<String>, index: i64) -> String {
    if ((index < (0 as i64)) || (index >= (values.len() as i64))) {
        return "".to_string();
    }
    for (current_index, value) in Box::new(
        (values)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if current_index == index {
            return format!("{}{}", value, "".to_string());
        }
    }
    return "".to_string();
}
fn _dict_value_at(values: &HashMap<String, String>, key: &String) -> String {
    for item_key in values.keys().cloned() {
        if item_key != *key {
            continue;
        }
        let value: Option<String> = values.get(&item_key).cloned();
        let Some(value) = value else {
            return "".to_string();
        };
        return format!("{}{}", value, "".to_string());
    }
    return "".to_string();
}
fn _first_char(text: &String) -> String {
    return _char_at(text, 0 as i64);
}
fn _last_char(text: &String) -> String {
    return _char_at(text, (text.chars().count() as i64) - (1 as i64));
}
fn parse_csv(
    text: &String,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Vec<Vec<String>> {
    let resolved: Dialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut rows: Vec<Vec<String>> = vec![];
    let mut row: Vec<String> = vec![];
    let mut field: String = "".to_string();
    let mut in_quotes: bool = false;
    let mut field_started: bool = false;
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_value: String = _char_at(text, i);
        if in_quotes {
            if ((resolved.escapechar != "".to_string())
                && (ch_value == resolved.escapechar))
            {
                if (i + (1 as i64)) < (text.chars().count() as i64) {
                    let escaped_value: String = _char_at(text, i + (1 as i64));
                    field = format!("{}{}", field, escaped_value);
                    i = i + (2 as i64);
                    continue;
                }
                field = format!("{}{}", field, ch_value);
                i = i + (1 as i64);
                continue;
            }
            if ((resolved.quotechar != "".to_string())
                && (ch_value == resolved.quotechar))
            {
                let quotechar: String = _quotechar_value(&resolved);
                if (((resolved.doublequote)
                    && ((i + (1 as i64)) < (text.chars().count() as i64)))
                    && (_char_at(text, i + (1 as i64)) == quotechar.clone()))
                {
                    field = format!("{}{}", field, quotechar);
                    i = i + (2 as i64);
                    continue;
                }
                in_quotes = false;
                i = i + (1 as i64);
                continue;
            }
            field = format!("{}{}", field, ch_value);
            i = i + (1 as i64);
            continue;
        }
        if (((!(field_started)) && (resolved.skipinitialspace))
            && (ch_value == " ".to_string()))
        {
            i = i + (1 as i64);
            continue;
        }
        if ((resolved.escapechar != "".to_string()) && (ch_value == resolved.escapechar))
        {
            if (i + (1 as i64)) < (text.chars().count() as i64) {
                let escaped_plain_value: String = _char_at(text, i + (1 as i64));
                field = format!("{}{}", field, escaped_plain_value);
                field_started = true;
                i = i + (2 as i64);
                continue;
            }
            field = format!("{}{}", field, ch_value);
            field_started = true;
            i = i + (1 as i64);
            continue;
        }
        if ((resolved.quoting != QUOTE_NONE) && (resolved.quotechar != "".to_string())) {
            let quotechar2: String = _quotechar_value(&resolved);
            if ch_value == quotechar2 {
                in_quotes = true;
                field_started = true;
                i = i + (1 as i64);
                continue;
            }
        }
        if ch_value == resolved.delimiter {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i = i + (1 as i64);
            continue;
        }
        if (ch_value == "\n".to_string()) || (ch_value == "\r".to_string()) {
            if (((ch_value == "\r".to_string())
                && ((i + (1 as i64)) < (text.chars().count() as i64)))
                && (_char_at(text, i + (1 as i64)) == "\n".to_string()))
            {
                i = i + (1 as i64);
            }
            if (((row.len() as i64) == (0 as i64)) && (field == "".to_string())) {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i = i + (1 as i64);
            continue;
        }
        field = format!("{}{}", field, ch_value);
        field_started = true;
        i = i + (1 as i64);
    }
    if in_quotes {
        in_quotes = false;
    }
    if (((row.len() as i64) > (0 as i64)) || (field != "".to_string())) {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    return rows;
}
fn _needs_quote(field: &String, dialect: &Dialect) -> bool {
    if dialect.quoting == QUOTE_ALL {
        return true;
    }
    if dialect.quoting == QUOTE_NONNUMERIC {
        return true;
    }
    if dialect.quoting == QUOTE_STRINGS {
        return true;
    }
    if dialect.quoting == QUOTE_NOTNULL {
        return true;
    }
    if dialect.quoting == QUOTE_NONE {
        return false;
    }
    if (field).contains(&(dialect.delimiter)) {
        return true;
    }
    if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
        return true;
    }
    if dialect.quotechar != "".to_string() {
        let quotechar: String = _quotechar_value(dialect);
        if field.contains(&quotechar) {
            return true;
        }
    }
    if (field.chars().count() as i64) > (0 as i64) {
        let first: String = _first_char(field);
        let last: String = _last_char(field);
        if first == " ".to_string() {
            return true;
        }
        if last == " ".to_string() {
            return true;
        }
    }
    return false;
}
fn _quote_field(field: &String, dialect: &Dialect) -> String {
    let quotechar: String = _quotechar_value(dialect);
    let mut escaped: String = format!("{}{}", field, "".to_string());
    if escaped.contains(&quotechar) {
        if dialect.doublequote {
            escaped = escaped
                .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
        } else {
            if dialect.escapechar != "".to_string() {
                let escapechar_value: String = format!(
                    "{}{}", dialect.escapechar, "".to_string()
                );
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", escapechar_value, quotechar));
            } else {
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
            }
        }
    }
    return format!("{}{}{}", quotechar, escaped, quotechar);
}
fn _escape_unquoted_field(field: &String, dialect: &Dialect) -> String {
    let mut result: String = format!("{}{}", field, "".to_string());
    if (result).contains(&(dialect.delimiter)) {
        if dialect.escapechar != "".to_string() {
            result = result
                .replace(
                    &dialect.delimiter,
                    &format!("{}{}", dialect.escapechar, dialect.delimiter),
                );
        }
    }
    if result.contains(&"\n".to_string()) {
        if dialect.escapechar != "".to_string() {
            result = result
                .replace(
                    &"\n".to_string(),
                    &format!("{}{}", dialect.escapechar, "\n".to_string()),
                );
        }
    }
    if result.contains(&"\r".to_string()) {
        if dialect.escapechar != "".to_string() {
            result = result
                .replace(
                    &"\r".to_string(),
                    &format!("{}{}", dialect.escapechar, "\r".to_string()),
                );
        }
    }
    if dialect.quotechar != "".to_string() {
        let quotechar2: String = _quotechar_value(dialect);
        if result.contains(&quotechar2) {
            if dialect.escapechar != "".to_string() {
                result = result
                    .replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar, quotechar2),
                    );
            } else {
                result = result
                    .replace(&quotechar2, &format!("{}{}", quotechar2, quotechar2));
            }
        }
    }
    return result;
}
fn format_row(
    fields: &Vec<String>,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> String {
    let resolved: Dialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut parts: Vec<String> = vec![];
    for field in fields.iter().cloned() {
        if _needs_quote(&field, &resolved) {
            parts.push(_quote_field(&field, &resolved));
        } else {
            parts.push(_escape_unquoted_field(&field, &resolved));
        }
    }
    return parts.join(&resolved.delimiter);
}
fn format_csv(
    rows: &Vec<Vec<String>>,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> String {
    let resolved: Dialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        quoting,
    );
    let mut rendered: Vec<String> = vec![];
    let resolved_delimiter: String = format!("{}{}", resolved.delimiter, "".to_string());
    let resolved_quotechar: String = format!("{}{}", resolved.quotechar, "".to_string());
    let resolved_escapechar: String = format!(
        "{}{}", resolved.escapechar, "".to_string()
    );
    let resolved_lineterminator: String = format!(
        "{}{}", resolved.lineterminator, "".to_string()
    );
    for row in rows.iter().cloned() {
        rendered
            .push(
                format_row(
                    &row,
                    &None,
                    &resolved_delimiter,
                    &resolved_quotechar,
                    &resolved_escapechar,
                    resolved.doublequote,
                    resolved.skipinitialspace,
                    resolved.quoting,
                ),
            );
    }
    return rendered.join(&resolved_lineterminator);
}
fn _dict_reader_row(
    fieldnames: &Vec<String>,
    row: &Vec<String>,
    restkey: &String,
    restval: &String,
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::from([]);
    for (i, key) in Box::new(
        (fieldnames)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if i < (row.len() as i64) {
            {
                let __assign_key = key;
                let __assign_value = _list_value_at(row, i);
                result.insert(__assign_key, __assign_value);
            }
        } else {
            result.insert(key, format!("{}{}", restval, "".to_string()));
        }
    }
    if ((restkey.clone() != "".to_string())
        && ((row.len() as i64) > (fieldnames.len() as i64)))
    {
        let mut extras: Vec<String> = vec![];
        let mut j: i64 = fieldnames.len() as i64;
        while j < (row.len() as i64) {
            extras.push(_list_value_at(row, j));
            j = j + (1 as i64);
        }
        {
            let __assign_key = restkey.clone();
            let __assign_value = format!("{:?}", extras);
            result.insert(__assign_key, __assign_value);
        }
    }
    return result;
}

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
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
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
        for (item_key, item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
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
        for (item_key, _item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<JsonValue> {
        let mut result: Vec<JsonValue> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (_item_key, item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
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
        return write!(
            f, "{}", { let __json_value = self; fn __sifr_json_value_to_serde(value : &
            JsonValue) -> serde_json::Value { match value.kind.as_str() { "null" => {
            return serde_json::Value::Null; }, "bool" => { if let Some(v) = value
            .bool_value { return serde_json::Value::from(v); } return
            serde_json::Value::Null; }, "int" => { if let Some(v) = value.int_value {
            return serde_json::Value::from(v); } return serde_json::Value::Null; },
            "float" => { if let Some(v) = value.float_value { return
            serde_json::Value::from(v); } return serde_json::Value::Null; }, "str" => {
            if let Some(v) = value.str_value.clone() { return
            serde_json::Value::String(v); } return serde_json::Value::Null; }, "array" =>
            { let mut converted = vec![]; for item in value.array_items.as_ref().iter()
            .cloned() { converted.push(__sifr_json_value_to_serde(& item)); } return
            serde_json::Value::Array(converted); }, "object" => { let mut converted =
            serde_json::Map::new(); for entry in value.object_items.as_ref().iter()
            .cloned() { let entry_key = entry.0; let entry_value = entry.1; converted
            .insert(entry_key, __sifr_json_value_to_serde(& entry_value)); } return
            serde_json::Value::Object(converted); }, _ => { return
            serde_json::Value::Null; }, } } serde_json::to_string(&
            __sifr_json_value_to_serde(& __json_value)).unwrap_or_else(| _err | "null"
            .to_string().to_string()) }
        );
    }
}
fn from_bool(value: bool) -> JsonValue {
    let bool_value: Option<bool> = Some(value);
    return JsonValue::new("bool".to_string(), bool_value, None, None, None);
}
fn from_int(value: i64) -> JsonValue {
    let int_value: Option<i64> = Some(value);
    return JsonValue::new("int".to_string(), None, int_value, None, None);
}
fn from_str(value: &String) -> JsonValue {
    let str_value: Option<String> = Some(format!("{}{}", value, "".to_string()));
    return JsonValue::new("str".to_string(), None, None, None, str_value);
}
fn _append_array_item(mut value: JsonValue, item: JsonValue) -> JsonValue {
    value.array_items.push(item);
    return value;
}
fn _append_object_item(
    mut value: JsonValue,
    key: String,
    item_value: JsonValue,
) -> JsonValue {
    value.object_items.push((key, item_value));
    return value;
}
fn from_array(items: &Vec<JsonValue>) -> JsonValue {
    let mut value: JsonValue = JsonValue::new(
        "array".to_string(),
        None,
        None,
        None,
        None,
    );
    for item in items.iter().cloned() {
        value = _append_array_item(value, item);
    }
    return value;
}
fn from_object(items: &Vec<(String, JsonValue)>) -> JsonValue {
    let mut value: JsonValue = JsonValue::new(
        "object".to_string(),
        None,
        None,
        None,
        None,
    );
    for (key, item_value) in items.iter().cloned() {
        value = _append_object_item(value, key, item_value);
    }
    return value;
}
fn _decode_loaded_json(content: &String) -> Result<JsonValue, Error> {
    let __sifr_try_res: Result<Result<JsonValue, Error>, JSONDecodeError> = (|| {
        let value: JsonValue = ({
            let __json_input = content;
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
                            let converted_value = __sifr_json_value_from_serde(
                                entry_value,
                            )?;
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
        return Ok(Ok(value));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(Error::new(e.message));
        }
    }
}
fn load(path: &String) -> Result<JsonValue, Error> {
    let content_result: Result<String, IOError> = std::fs::read_to_string(&path)
        .map_err(__io_err);
    let __sifr_try_res: Result<Result<JsonValue, Error>, IOError> = (|| {
        let content: String = content_result?;
        return Ok(_decode_loaded_json(&content));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(Error::new(e.message));
        }
    }
}
fn dumps(value: &JsonValue) -> String {
    return {
        let __json_value = value;
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
                        converted
                            .insert(entry_key, __sifr_json_value_to_serde(&entry_value));
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
    };
}

// --- stdlib: sifr.configparser ---
fn __const_DEFAULTSECT() -> String {
    return "DEFAULT".to_string().to_string();
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParsingError {
    line: i64,
    message: String,
}
impl ParsingError {
    fn new(line: i64, message: String) -> Self {
        return Self {
            line: line,
            message: message,
        };
    }
}
impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for ParsingError {}
#[derive(Debug, Clone, PartialEq)]
struct SectionProxy {
    name: String,
    _values: HashMap<String, Option<String>>,
}
impl SectionProxy {
    fn new(name: String, values: HashMap<String, Option<String>>) -> Self {
        return Self {
            name: format!("{}{}", name, "".to_string()),
            _values: _copy_values(&values),
        };
    }
    fn has_option(&self, option: &String) -> bool {
        return _has_option_key(&self._values.clone(), &_normalize_option(option));
    }
    fn get(
        &self,
        option: &String,
        fallback: &Option<String>,
        raw: bool,
    ) -> Option<String> {
        let normalized: String = _normalize_option(option);
        if _has_option_key(&self._values.clone(), &normalized) {
            let value: Option<String> = _lookup_option(
                &self._values.clone(),
                &normalized,
            );
            let Some(value) = value else {
                return None;
            };
            if raw {
                return Some(value);
            }
            return Some(_resolve_interpolation(&value, &self._values.clone(), 0 as i64));
        }
        return _copy_optional_str(fallback);
    }
    fn options(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for key in self._values.clone().keys().cloned() {
            names.push(key);
        }
        return names;
    }
    fn items(&self) -> Vec<(String, Option<String>)> {
        let mut pairs: Vec<(String, Option<String>)> = vec![];
        for (key, value) in self
            ._values
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            pairs.push((key, _copy_optional_str(&value)));
        }
        return pairs;
    }
}
#[derive(Debug, Clone, PartialEq)]
struct ConfigParser {
    _defaults: HashMap<String, Option<String>>,
    _sections: HashMap<String, HashMap<String, Option<String>>>,
    strict: bool,
    allow_no_value: bool,
}
impl ConfigParser {
    fn new(
        defaults: Option<HashMap<String, Option<String>>>,
        strict: bool,
        allow_no_value: bool,
    ) -> Self {
        let mut defaults_map: HashMap<String, Option<String>> = HashMap::from([]);
        let sections_map: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
        if let Some(defaults) = defaults {
            for (key, value) in defaults
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                let normalized: String = _normalize_option(&key);
                {
                    let __assign_key = normalized;
                    let __assign_value = _copy_optional_str(&value);
                    defaults_map.insert(__assign_key, __assign_value);
                }
            }
        }
        return Self {
            strict: strict,
            allow_no_value: allow_no_value,
            _defaults: defaults_map,
            _sections: sections_map,
        };
    }
    fn defaults(&self) -> HashMap<String, Option<String>> {
        return _copy_values(&self._defaults.clone());
    }
    fn read_string(&mut self, text: &String) -> Result<(), ParsingError> {
        let mut current_section: String = "".to_string();
        let default_section: String = _default_section();
        for (line_no, raw_line) in Box::new(
            (text
                .split(&"\n".to_string())
                .map(|s| s.to_string())
                .collect::<Vec<String>>())
                .into_iter()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + (1 as i64), __pair.1)),
        ) {
            let line: String = raw_line.trim().to_string();
            if (((line == "".to_string()) || (line.starts_with(&"#".to_string())))
                || (line.starts_with(&";".to_string())))
            {
                continue;
            }
            if ((line.starts_with(&"[".to_string()))
                && (line.ends_with(&"]".to_string())))
            {
                let section_name: String = line
                    .chars()
                    .skip((1 as i64) as usize)
                    .take(
                        (((line.chars().count() as i64) - (1 as i64)) as usize)
                            - ((1 as i64) as usize),
                    )
                    .collect::<String>()
                    .trim()
                    .to_string();
                if section_name == "".to_string() {
                    return Err(
                        ParsingError::new(line_no, "section name is empty".to_string()),
                    );
                }
                if section_name == default_section {
                    current_section = _default_section();
                    continue;
                }
                if ((self.strict)
                    && ((self._sections.clone()).contains_key(&(section_name))))
                {
                    return Err(
                        ParsingError::new(
                            line_no,
                            format!(
                                "{}{}", "duplicate section: ".to_string(), section_name
                            ),
                        ),
                    );
                }
                current_section = format!("{}{}", section_name, "".to_string());
                if !((self._sections.clone()).contains_key(&(section_name))) {
                    self._sections.insert(section_name, HashMap::from([]));
                }
                continue;
            }
            let __sifr_try_res: Result<(), ParsingError> = (|| {
                let parsed_option_pair: (String, Option<String>) = _split_option_line(
                    &line,
                    self.allow_no_value,
                    line_no,
                )?;
                let (option_name, option_value) = parsed_option_pair;
                if (current_section == "".to_string())
                    || (current_section == default_section)
                {
                    self._defaults
                        .insert(option_name, _copy_optional_str(&option_value));
                } else {
                    let section_key: String = format!(
                        "{}{}", current_section, "".to_string()
                    );
                    let mut section_found: bool = false;
                    for (section_name, section_values) in self
                        ._sections
                        .clone()
                        .iter()
                        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                        .collect::<Vec<_>>()
                    {
                        if section_name != section_key {
                            continue;
                        }
                        if ((self.strict)
                            && (_has_option_key(&section_values, &option_name)))
                        {
                            return Err(
                                ParsingError::new(
                                    line_no,
                                    format!(
                                        "{}{}", "duplicate option: ".to_string(), option_name
                                    ),
                                ),
                            );
                        }
                        let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                            &section_values,
                        );
                        {
                            let __assign_key = option_name;
                            let __assign_value = _copy_optional_str(&option_value);
                            updated_section.insert(__assign_key, __assign_value);
                        }
                        self._sections.insert(section_name, updated_section);
                        section_found = true;
                        break;
                    }
                }
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
        return Ok(());
    }
    fn read(&mut self, path: &String) -> Result<Vec<String>, IOError> {
        let __sifr_try_res: Result<Result<Vec<String>, IOError>, IOError> = (|| {
            let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
            let __sifr_try_res: Result<(), ParsingError> = (|| {
                let _: () = self.read_string(&content)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(
                    IOError::new(
                        format!(
                            "{}{}{}{}", "parse error on line ".to_string(), format!("{}",
                            e.line), ": ".to_string(), e.message
                        ),
                    ),
                );
            }
            let loaded_path: String = format!("{}{}", path, "".to_string());
            return Ok(Ok(vec![loaded_path]));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    fn sections(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for section in self._sections.clone().keys().cloned() {
            names.push(section);
        }
        return names;
    }
    fn has_section(&self, section: &String) -> bool {
        return (self._sections.clone()).contains_key((section).as_str());
    }
    fn options(&self, section: &String) -> Vec<String> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut names: Vec<String> = vec![];
        for option in merged.keys().cloned() {
            names.push(option);
        }
        return names;
    }
    fn items(&self, section: &String) -> Vec<(String, Option<String>)> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut items: Vec<(String, Option<String>)> = vec![];
        for (option, value) in merged
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            items.push((option, _copy_optional_str(&value)));
        }
        return items;
    }
    fn _merged_section(&self, section: &String) -> HashMap<String, Option<String>> {
        let mut merged: HashMap<String, Option<String>> = _copy_values(
            &self._defaults.clone(),
        );
        let default_section: String = _default_section();
        if *section == default_section {
            return merged;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            for (option, value) in section_values
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                {
                    let __assign_key = option;
                    let __assign_value = _copy_optional_str(&value);
                    merged.insert(__assign_key, __assign_value);
                }
            }
            return merged;
        }
        return merged;
    }
    fn has_option(&self, section: &String, option: &String) -> bool {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            return (self._defaults.clone()).contains_key(&(normalized));
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            if _has_option_key(&section_values, &normalized) {
                return true;
            }
            return (self._defaults.clone()).contains_key(&(normalized));
        }
        return false;
    }
    fn get(
        &self,
        section: &String,
        option: &String,
        fallback: &Option<String>,
        raw: bool,
    ) -> Option<String> {
        let normalized: String = _normalize_option(option);
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let default_section: String = _default_section();
        if *section == default_section {
            if !(_has_option_key(&merged, &normalized)) {
                return _copy_optional_str(fallback);
            }
            let raw_value: Option<String> = _lookup_option(&merged, &normalized);
            let Some(raw_value) = raw_value else {
                return None;
            };
            if raw {
                return Some(raw_value);
            }
            return Some(_resolve_interpolation(&raw_value, &merged, 0 as i64));
        }
        if !(self.has_section(section)) {
            if _has_option_key(&self._defaults.clone(), &normalized) {
                let default_value: Option<String> = _lookup_option(
                    &self._defaults.clone(),
                    &normalized,
                );
                let Some(default_value) = default_value else {
                    return None;
                };
                if raw {
                    return Some(default_value);
                }
                return Some(_resolve_interpolation(&default_value, &merged, 0 as i64));
            }
            return _copy_optional_str(fallback);
        }
        if !(_has_option_key(&merged, &normalized)) {
            return _copy_optional_str(fallback);
        }
        let raw_value2: Option<String> = _lookup_option(&merged, &normalized);
        let Some(raw_value2) = raw_value2 else {
            return None;
        };
        if raw {
            return Some(raw_value2);
        }
        return Some(_resolve_interpolation(&raw_value2, &merged, 0 as i64));
    }
    fn getint(
        &self,
        section: &String,
        option: &String,
        fallback: Option<i64>,
    ) -> Option<i64> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let __sifr_try_res: Result<Option<i64>, ParseError> = (|| {
            let parsed: i64 = (raw)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(Some(parsed));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return fallback;
            }
        }
    }
    fn getfloat(
        &self,
        section: &String,
        option: &String,
        fallback: Option<f64>,
    ) -> Option<f64> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let __sifr_try_res: Result<Option<f64>, ParseError> = (|| {
            let parsed: f64 = (raw)
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(Some(parsed));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let _e = __sifr_try_err.clone();
                return fallback;
            }
        }
    }
    fn getboolean(
        &self,
        section: &String,
        option: &String,
        fallback: Option<bool>,
    ) -> Option<bool> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let normalized: String = raw.to_lowercase();
        if (((normalized == "1".to_string()) || (normalized == "yes".to_string()))
            || (normalized == "true".to_string())) || (normalized == "on".to_string())
        {
            return Some(true);
        }
        if (((normalized == "0".to_string()) || (normalized == "no".to_string()))
            || (normalized == "false".to_string())) || (normalized == "off".to_string())
        {
            return Some(false);
        }
        return fallback;
    }
    fn set(&mut self, section: &String, option: &String, value: &Option<String>) {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            self._defaults.insert(normalized, _copy_optional_str(value));
            return;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                &section_values,
            );
            {
                let __assign_key = normalized;
                let __assign_value = _copy_optional_str(value);
                updated_section.insert(__assign_key, __assign_value);
            }
            self._sections.insert(section_name, updated_section);
            return;
        }
        if !((self._sections.clone()).contains_key((section).as_str())) {
            self._sections.insert(section.clone(), HashMap::from([]));
        }
        let mut created_section: HashMap<String, Option<String>> = HashMap::from([]);
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            created_section = _copy_values(&section_values);
            break;
        }
        {
            let __assign_key = normalized;
            let __assign_value = _copy_optional_str(value);
            created_section.insert(__assign_key, __assign_value);
        }
        self._sections.insert(section.clone(), created_section);
    }
    fn add_section(&mut self, section: &String) {
        let default_section: String = _default_section();
        if *section == default_section {
            return;
        }
        if (self._sections.clone()).contains_key((section).as_str()) {
            return;
        }
        self._sections.insert(section.clone(), HashMap::from([]));
    }
    fn remove_option(&mut self, section: &String, option: &String) -> bool {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            if (self._defaults.clone()).contains_key(&(normalized)) {
                self._defaults = _without_option(&self._defaults.clone(), &normalized);
                return true;
            }
            return false;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            if _has_option_key(&section_values, &normalized) {
                self._sections
                    .insert(section_name, _without_option(&section_values, &normalized));
                return true;
            }
            return false;
        }
        return false;
    }
    fn remove_section(&mut self, section: &String) -> bool {
        let default_section: String = _default_section();
        if *section == default_section {
            return false;
        }
        if (self._sections.clone()).contains_key((section).as_str()) {
            self._sections = _without_section(&self._sections.clone(), section);
            return true;
        }
        return false;
    }
    fn proxy(&self, section: &String) -> Option<SectionProxy> {
        let default_section: String = _default_section();
        if ((section.clone() != default_section) && (!(self.has_section(section)))) {
            return None;
        }
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        return Some(SectionProxy::new((section).clone(), merged));
    }
    fn to_ini_string(&self) -> String {
        let mut lines: Vec<String> = vec![];
        if (self._defaults.clone().len() as i64) > (0 as i64) {
            lines
                .push(
                    format!(
                        "{}{}", format!("{}{}", "[".to_string(), _default_section()), "]"
                        .to_string()
                    ),
                );
            for (key, value) in self
                ._defaults
                .clone()
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
                    lines.push(key);
                } else {
                    if let Some(value) = value {
                        lines
                            .push(
                                format!(
                                    "{}{}", format!("{}{}", key, " = ".to_string()), value
                                ),
                            );
                    }
                }
            }
            lines.push("".to_string());
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            lines
                .push(
                    format!(
                        "{}{}", format!("{}{}", "[".to_string(), section_name), "]"
                        .to_string()
                    ),
                );
            for (key, value) in section_values
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
                    lines.push(key);
                } else {
                    if let Some(value) = value {
                        lines
                            .push(
                                format!(
                                    "{}{}", format!("{}{}", key, " = ".to_string()), value
                                ),
                            );
                    }
                }
            }
            lines.push("".to_string());
        }
        if (lines.len() as i64) > (0 as i64) {
            let maybe_last: Option<String> = {
                let __sifr_index_list = &lines;
                let __sifr_index_i = (lines.len() as i64) - (1 as i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if ((maybe_last != None) && (maybe_last == Some("".to_string()))) {
                let _: String = {
                    let Some(__sifr_nonempty_pop_value) = lines.pop() else {
                        unreachable!(
                            "compiler-verified non-empty pop should return Some"
                        );
                    };
                    __sifr_nonempty_pop_value
                };
            }
        }
        return lines.join(&"\n".to_string());
    }
    fn write(&self, path: &String) -> Result<(), IOError> {
        let payload: String = self.to_ini_string();
        return std::fs::write(&path, payload.as_bytes()).map(|_| ()).map_err(__io_err);
    }
}
fn _default_section() -> String {
    return format!("{}{}", __const_DEFAULTSECT(), "".to_string());
}
fn _normalize_option(option: &String) -> String {
    return option.to_lowercase().trim().to_string();
}
fn _some_str(value: &String) -> Option<String> {
    return Some(format!("{}{}", value, "".to_string()));
}
fn _copy_optional_str(value: &Option<String>) -> Option<String> {
    if let Some(value) = value.as_ref() {
        return _some_str(value);
    }
    return None;
}
fn _has_option_key(values: &HashMap<String, Option<String>>, key: &String) -> bool {
    for current_key in values.keys().cloned() {
        if current_key == *key {
            return true;
        }
    }
    return false;
}
fn _lookup_option(
    values: &HashMap<String, Option<String>>,
    key: &String,
) -> Option<String> {
    for (current_key, current_value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if current_key == *key {
            return _copy_optional_str(&current_value);
        }
    }
    return None;
}
fn _copy_values(
    values: &HashMap<String, Option<String>>,
) -> HashMap<String, Option<String>> {
    let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
    for (key, value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        {
            let __assign_key = key;
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
}
fn _without_option(
    values: &HashMap<String, Option<String>>,
    removed_key: &String,
) -> HashMap<String, Option<String>> {
    let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
    for (key, value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if key == *removed_key {
            continue;
        }
        {
            let __assign_key = key;
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
}
fn _without_section(
    values: &HashMap<String, HashMap<String, Option<String>>>,
    removed_key: &String,
) -> HashMap<String, HashMap<String, Option<String>>> {
    let mut copied: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
    for (key, section) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if key == *removed_key {
            continue;
        }
        {
            let __assign_key = key;
            let __assign_value = _copy_values(&section);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
}
fn _find_delimiter(line: &String) -> Option<String> {
    if line.contains(&"=".to_string()) {
        return Some("=".to_string());
    }
    if line.contains(&":".to_string()) {
        return Some(":".to_string());
    }
    return None;
}
fn _split_option_line(
    line: &String,
    allow_no_value: bool,
    line_no: i64,
) -> Result<(String, Option<String>), ParsingError> {
    let delimiter: Option<String> = _find_delimiter(line);
    let Some(delimiter) = delimiter else {
        if allow_no_value {
            return Ok((line.trim().to_string(), None));
        }
        return Err(
            ParsingError::new(
                line_no,
                "expected key=value or key:value entry".to_string(),
            ),
        );
    };
    let parts: Vec<String> = if (1 as i64) < 0 {
        line.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>()
    } else {
        line.splitn(((1 as i64) + 1) as usize, &delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    if (parts.len() as i64) != (2 as i64) {
        return Err(ParsingError::new(line_no, "invalid option line".to_string()));
    }
    let raw_key: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let raw_value: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let Some(raw_key) = raw_key else {
        return Err(ParsingError::new(line_no, "option name is missing".to_string()));
    };
    let key: String = _normalize_option(&raw_key);
    if key == "".to_string() {
        return Err(ParsingError::new(line_no, "option name is empty".to_string()));
    }
    let Some(raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    return Ok((key, stripped_value));
}
fn _resolve_interpolation(
    value: &String,
    merged: &HashMap<String, Option<String>>,
    depth: i64,
) -> String {
    if depth >= (8 as i64) {
        return format!("{}{}", value, "".to_string());
    }
    if !value.contains(&"%(".to_string()) {
        return format!("{}{}", value, "".to_string());
    }
    let mut result: String = "".to_string();
    let mut replaced: bool = false;
    let mut i: i64 = 0 as i64;
    while i < (value.chars().count() as i64) {
        let ch: String = _char_at(value, i);
        if (((ch == "%".to_string())
            && ((i + (1 as i64)) < (value.chars().count() as i64)))
            && (_char_at(value, i + (1 as i64)) == "(".to_string()))
        {
            let mut j: i64 = i + (2 as i64);
            let mut key: String = "".to_string();
            let mut matched: bool = false;
            while j < (value.chars().count() as i64) {
                let part: String = _char_at(value, j);
                if (((part == ")".to_string())
                    && ((j + (1 as i64)) < (value.chars().count() as i64)))
                    && (_char_at(value, j + (1 as i64)) == "s".to_string()))
                {
                    matched = true;
                    let normalized_key: String = _normalize_option(&key);
                    let replacement: Option<String> = _lookup_option(
                        merged,
                        &normalized_key,
                    );
                    if replacement.is_none() {
                        result = format!(
                            "{}{}{}{}", result, "%(".to_string(), key, ")s".to_string()
                        );
                    } else {
                        if let Some(replacement) = replacement {
                            replaced = true;
                            result = format!("{}{}", result, replacement);
                        }
                    }
                    i = j + (2 as i64);
                    break;
                }
                key = format!("{}{}", key, part);
                j = j + (1 as i64);
            }
            if matched {
                continue;
            }
        }
        result = format!("{}{}", result, ch);
        i = i + (1 as i64);
    }
    if replaced {
        return _resolve_interpolation(&result, merged, depth + (1 as i64));
    }
    return result;
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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

impl std::error::Error for ParseError {
}

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

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn main() {
    println!("structured-parsing-sample structured parsing and serialization demo");
    let __sifr_try_res: Result<(), IOError> = (|| {
    let json_path: String = "/tmp/sifr_structured_parsing_serialization.json".to_string();
    let _: () = std::fs::write(&json_path, "{\"name\":\"sifr\",\"items\":[1,true]}".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let mut json_value: JsonValue = (load(&json_path)).map_err(|__e| IOError::new(__e.to_string()))?;
    let mut json_items: Option<JsonValue> = json_value.get(&"items".to_string());
    let mut json_name: Option<JsonValue> = json_value.get(&"name".to_string());
    let mut json_second: Option<JsonValue> = None;
    if let Some(json_items) = json_items {
        json_second = json_items.at(1 as i64);
    }
    if let Some(json_name) = json_name {
        println!("{}", (json_name.as_str()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    }
    if let Some(json_second) = json_second {
        println!("{}", (json_second.as_bool()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    }
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
    }
    println!("{}", dumps(&from_object(&vec![("name".to_string(), from_str(&"sifr".to_string())), ("items".to_string(), from_array(&vec![from_int(1 as i64), from_bool(true)]))])));
    let __sifr_try_res: Result<(), TOMLDecodeError> = (|| {
    let mut toml_value: TomlValue = loads(&"title = \"sifr\"\n[owner]\nactive = true\n".to_string())?;
    let mut owner: Option<TomlValue> = toml_value.get(&"owner".to_string());
    let mut title: Option<TomlValue> = toml_value.get(&"title".to_string());
    if let Some(owner) = owner {
        let mut active: Option<TomlValue> = owner.get(&"active".to_string());
        if let Some(title) = title {
            println!("{}", (title.as_str()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
        }
        if let Some(active) = active {
            println!("{}", (active.as_bool()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
        }
    }
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
    }
    let quoted: String = format_row(&vec!["alpha".to_string(), "beta".to_string()], &Some(Dialect::new(",".to_string(), "\"".to_string(), "".to_string(), true, false, "\n".to_string(), QUOTE_ALL)), &",".to_string(), &"\"".to_string(), &"".to_string(), true, false, 0 as i64);
    println!("{}", quoted);
    let mut dict_reader: DictReader = DictReader::new("name,age\nalice,30\n".to_string(), None, "".to_string(), "".to_string(), None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, 0 as i64);
    println!("{}", format!("{:?}", dict_reader.rows()));
    let mut dict_writer: DictWriter = DictWriter::new(vec!["name".to_string(), "age".to_string()], "".to_string(), "raise".to_string(), None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, "\n".to_string(), 0 as i64);
    dict_writer.writeheader();
    dict_writer.writerow(&HashMap::from([("name".to_string(), "alice".to_string()), ("age".to_string(), "30".to_string())]));
    println!("{}", dict_writer.getvalue());
    let mut defaults: HashMap<String, Option<String>> = HashMap::from([]);
    let encoding_value: Option<String> = Some("utf-8".to_string());
    defaults.insert("encoding".to_string(), encoding_value);
    let mut parser: ConfigParser = ConfigParser::new(Some(defaults), false, true);
    let __sifr_try_res: Result<(), ParsingError> = (|| {
    let _: () = parser.read_string(&"[server]\nport = 8080\nenabled = true\nfeature\n".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
        return;
    }
    println!("{}", (parser.getint(&"server".to_string(), &"port".to_string(), None)).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (parser.getboolean(&"server".to_string(), &"enabled".to_string(), None)).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let fallback_value: Option<String> = Some("missing".to_string());
    println!("{}", (parser.get(&"server".to_string(), &"feature".to_string(), &fallback_value, false)).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}

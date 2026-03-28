use std::sync::Mutex;

// --- stdlib: sifr.re ---
const IGNORECASE: i64 = 2 as i64;
const MULTILINE: i64 = 8 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Match {
    _matched: String,
    _start: i64,
    _end: i64,
}
impl Match {
    fn new(matched: String, start: i64, end: i64) -> Self {
        return Self {
            _matched: matched,
            _start: start,
            _end: end,
        };
    }
    fn group(&self) -> String {
        return format!("{}{}", self._matched.clone(), "".to_string());
    }
    fn start(&self) -> i64 {
        return self._start;
    }
    fn end(&self) -> i64 {
        return self._end;
    }
    fn span(&self) -> Vec<i64> {
        let result: Vec<i64> = vec![self._start, self._end];
        return result;
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._matched.clone(), "".to_string());
    }
}
impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Match(_matched={}, _start={}, _end={})",
            self._matched, self._start, self._end
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pattern {
    _pattern: String,
    _flags: i64,
}
impl Pattern {
    fn new(pattern: String, flags: i64) -> Self {
        return Self {
            _pattern: pattern,
            _flags: flags,
        };
    }
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        if self._flags != (0 as i64) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
                Ok(__re.find(&text).map(|m| m.as_str().to_string()))
            };
        }
        return regex::Regex::new(&self._pattern.clone())
            .map(|re| re.find(&text).map(|m| m.as_str().to_string()))
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            });
    }
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        if self._flags != (0 as i64) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
                Ok(__re.is_match(&text))
            };
        }
        return regex::Regex::new(&self._pattern.clone())
            .map(|re| re.is_match(&text))
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            });
    }
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        if self._flags != (0 as i64) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
                Ok(__re.replace_all(&text, &*replacement).to_string())
            };
        }
        return regex::Regex::new(&self._pattern.clone())
            .map(|re| re.replace_all(&text, &*replacement).to_string())
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            });
    }
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        if self._flags != (0 as i64) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
                Ok(__re
                    .find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<String>>())
            };
        }
        return regex::Regex::new(&self._pattern.clone())
            .map(|re| {
                re.find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            });
    }
    fn finditer(&self, text: &String) -> Result<Box<dyn Iterator<Item = Match>>, RegexError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = Match>>, RegexError>,
            RegexError,
        > = (|| {
            let matches: Vec<Match> =
                _finditer_materialize(&self._pattern.clone(), text, self._flags)?;
            return Ok(Ok(_iter_matches(matches)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(RegexError::new(e.message));
            }
        }
    }
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        if self._flags != (0 as i64) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
                Ok(__re
                    .split(&text)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>())
            };
        }
        return regex::Regex::new(&self._pattern.clone())
            .map(|re| {
                re.split(&text)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            });
    }
    fn pattern(&self) -> String {
        return format!("{}{}", self._pattern.clone(), "".to_string());
    }
}
impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Pattern(_pattern={}, _flags={})",
            self._pattern, self._flags
        );
    }
}
fn search_flags(pattern: &String, text: &String, flags: i64) -> Result<Option<String>, RegexError> {
    return {
        let __flags_val = flags;
        let mut __flag_str = String::new();
        if (__flags_val & 2) != 0 {
            __flag_str.push_str("(?i)");
        }
        if (__flags_val & 8) != 0 {
            __flag_str.push_str("(?m)");
        }
        if (__flags_val & 16) != 0 {
            __flag_str.push_str("(?s)");
        }
        if (__flags_val & 64) != 0 {
            __flag_str.push_str("(?x)");
        }
        let __pat = __flag_str + &pattern;
        let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        })?;
        Ok(__re.find(&text).map(|m| m.as_str().to_string()))
    };
}
fn _iter_matches(matches: Vec<Match>) -> Box<dyn Iterator<Item = Match>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<Match> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<Match> = Vec::new();
            let mut i: i64 = 0 as i64;
            while i < (matches.len() as i64) {
                _yields.push(matches[i as usize].clone());
                i = i + (1 as i64);
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}
fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
    if start < (0 as i64) {
        return -(1 as i64);
    }
    if (needle.len() as i64) == (0 as i64) {
        if start <= (text.len() as i64) {
            return start;
        }
        return -(1 as i64);
    }
    let max_start: i64 = (text.chars().count() as i64) - (needle.chars().count() as i64);
    let mut i: i64 = start;
    while i <= max_start {
        if String::from_iter(
            (text)
                .chars()
                .skip((i).max(0) as usize)
                .take(((i + (needle.chars().count() as i64)).max(0) - (i).max(0)).max(0) as usize),
        ) == needle.clone()
        {
            return i;
        }
        i = i + (1 as i64);
    }
    return -(1 as i64);
}
fn _findall_for_finditer(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    if flags != (0 as i64) {
        return {
            let __flags_val = flags;
            let mut __flag_str = String::new();
            if (__flags_val & 2) != 0 {
                __flag_str.push_str("(?i)");
            }
            if (__flags_val & 8) != 0 {
                __flag_str.push_str("(?m)");
            }
            if (__flags_val & 16) != 0 {
                __flag_str.push_str("(?s)");
            }
            if (__flags_val & 64) != 0 {
                __flag_str.push_str("(?x)");
            }
            let __pat = __flag_str + &pattern;
            let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })?;
            Ok(__re
                .find_iter(&text)
                .map(|m| m.as_str().to_string())
                .collect::<Vec<String>>())
        };
    }
    return regex::Regex::new(&pattern)
        .map(|re| {
            re.find_iter(&text)
                .map(|m| m.as_str().to_string())
                .collect::<Vec<String>>()
        })
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn _finditer_materialize(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<Match>, RegexError> {
    let __sifr_try_res: Result<Result<Vec<Match>, RegexError>, RegexError> = (|| {
        let found_items: Vec<String> = _findall_for_finditer(pattern, text, flags)?;
        let mut matches: Vec<Match> = vec![];
        let mut cursor: i64 = 0 as i64;
        for found in found_items.iter().cloned() {
            let mut start: i64 = _find_index_from(text, &found, cursor);
            if start < (0 as i64) {
                start = cursor;
            }
            let found_len: i64 = found.chars().count() as i64;
            let end: i64 = start + found_len;
            matches.push(Match::new(found, start, end));
            if found_len == (0 as i64) {
                cursor = end + (1 as i64);
            } else {
                cursor = end;
            }
        }
        return Ok(Ok(matches));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message));
        }
    }
}
fn compile_flags(pattern: &String, flags: i64) -> Pattern {
    return Pattern::new((pattern).clone(), flags);
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
struct reader {
    _rows: Vec<Vec<String>>,
    _pos: i64,
    dialect: Dialect,
}
impl reader {
    fn new(
        text: String,
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
        let rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, "".to_string()),
            &format!("{}{}", resolved_dialect.quotechar, "".to_string()),
            &format!("{}{}", resolved_dialect.escapechar, "".to_string()),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        return Self {
            dialect: resolved_dialect,
            _rows: rows,
            _pos: 0 as i64,
        };
    }
    fn __next__(&mut self) -> Option<Vec<String>> {
        if self._pos >= (self._rows.clone().len() as i64) {
            return None;
        }
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
        let mut result: Vec<String> = vec![];
        for field in row.iter().cloned() {
            result.push(format!("{}{}", field, "".to_string()));
        }
        return Some(result);
    }
    fn rows(&self) -> Vec<Vec<String>> {
        let mut result: Vec<Vec<String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for field in row.iter().cloned() {
                copied.push(format!("{}{}", field, "".to_string()));
            }
            result.push(copied);
        }
        return result;
    }
    fn line_num(&self) -> i64 {
        return self._pos;
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
            if ((resolved.escapechar != "".to_string()) && (ch_value == resolved.escapechar)) {
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
            if ((resolved.quotechar != "".to_string()) && (ch_value == resolved.quotechar)) {
                let quotechar: String = _quotechar_value(&resolved);
                if (((resolved.doublequote) && ((i + (1 as i64)) < (text.chars().count() as i64)))
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
        if (((!(field_started)) && (resolved.skipinitialspace)) && (ch_value == " ".to_string())) {
            i = i + (1 as i64);
            continue;
        }
        if ((resolved.escapechar != "".to_string()) && (ch_value == resolved.escapechar)) {
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
            escaped = escaped.replace(&quotechar, &format!("{}{}", quotechar, quotechar));
        } else {
            if dialect.escapechar != "".to_string() {
                let escapechar_value: String = format!("{}{}", dialect.escapechar, "".to_string());
                escaped =
                    escaped.replace(&quotechar, &format!("{}{}", escapechar_value, quotechar));
            } else {
                escaped = escaped.replace(&quotechar, &format!("{}{}", quotechar, quotechar));
            }
        }
    }
    return format!("{}{}{}", quotechar, escaped, quotechar);
}
fn _escape_unquoted_field(field: &String, dialect: &Dialect) -> String {
    let mut result: String = format!("{}{}", field, "".to_string());
    if (result).contains(&(dialect.delimiter)) {
        if dialect.escapechar != "".to_string() {
            result = result.replace(
                &dialect.delimiter,
                &format!("{}{}", dialect.escapechar, dialect.delimiter),
            );
        }
    }
    if result.contains(&"\n".to_string()) {
        if dialect.escapechar != "".to_string() {
            result = result.replace(
                &"\n".to_string(),
                &format!("{}{}", dialect.escapechar, "\n".to_string()),
            );
        }
    }
    if result.contains(&"\r".to_string()) {
        if dialect.escapechar != "".to_string() {
            result = result.replace(
                &"\r".to_string(),
                &format!("{}{}", dialect.escapechar, "\r".to_string()),
            );
        }
    }
    if dialect.quotechar != "".to_string() {
        let quotechar2: String = _quotechar_value(dialect);
        if result.contains(&quotechar2) {
            if dialect.escapechar != "".to_string() {
                result = result.replace(
                    &quotechar2,
                    &format!("{}{}", dialect.escapechar, quotechar2),
                );
            } else {
                result = result.replace(&quotechar2, &format!("{}{}", quotechar2, quotechar2));
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
    let resolved_escapechar: String = format!("{}{}", resolved.escapechar, "".to_string());
    let resolved_lineterminator: String = format!("{}{}", resolved.lineterminator, "".to_string());
    for row in rows.iter().cloned() {
        rendered.push(format_row(
            &row,
            &None,
            &resolved_delimiter,
            &resolved_quotechar,
            &resolved_escapechar,
            resolved.doublequote,
            resolved.skipinitialspace,
            resolved.quoting,
        ));
    }
    return rendered.join(&resolved_lineterminator);
}
fn reader_from_path(
    path: &String,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Result<reader, IOError> {
    let __sifr_try_res: Result<Result<reader, IOError>, IOError> = (|| {
        let text: String = std::fs::read_to_string(&path).map_err(__io_err)?;
        return Ok(Ok(reader::new(
            text,
            (dialect).clone(),
            (delimiter).clone(),
            (quotechar).clone(),
            (escapechar).clone(),
            doublequote,
            skipinitialspace,
            quoting,
        )));
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
fn writer_to_path(
    path: &String,
    rows: &Vec<Vec<String>>,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> Result<(), IOError> {
    let payload: String = format_csv(
        rows,
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        quoting,
    );
    return std::fs::write(&path, payload.as_bytes())
        .map(|_| ())
        .map_err(__io_err);
}

// --- stdlib: sifr.pathlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Path {
    _path: String,
}
impl Path {
    fn new(path: String) -> Self {
        return Self { _path: path };
    }
    fn name(&self) -> String {
        return basename(&self._path.clone());
    }
    fn parent(&self) -> Path {
        return Path::new(dirname(&self._path.clone()));
    }
    fn suffix(&self) -> String {
        return extension(&self._path.clone());
    }
    fn stem(&self) -> String {
        return stem(&self._path.clone());
    }
    fn exists(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).exists();
    }
    fn is_file(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).is_file();
    }
    fn is_dir(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).is_dir();
    }
    fn is_absolute(&self) -> bool {
        return is_absolute(&self._path.clone());
    }
    fn read_text(&self) -> Result<String, IOError> {
        return std::fs::read_to_string(&self._path.clone()).map_err(__io_err);
    }
    fn write_text(&self, content: &String) -> Result<(), IOError> {
        return std::fs::write(&self._path.clone(), content.as_bytes())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn mkdir(&self) -> Result<(), IOError> {
        return std::fs::create_dir_all(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn joinpath(&self, child: &String) -> Path {
        return Path::new(join_path(&self._path.clone(), child));
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._path.clone(), "".to_string());
    }
    fn touch(&self) -> Result<(), IOError> {
        return std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn unlink(&self) -> Result<(), IOError> {
        return std::fs::remove_file(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn rmdir(&self) -> Result<(), IOError> {
        return std::fs::remove_dir(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn resolve(&self) -> Result<String, IOError> {
        return std::fs::canonicalize(&self._path.clone())
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err);
    }
    fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _iterdir_to_iter(&self._path.clone());
    }
    fn with_name(&self, name: &String) -> Path {
        let parent: String = dirname(&self._path.clone());
        if parent == "".to_string() {
            return Path::new(format!("{}{}", name, "".to_string()));
        }
        return Path::new(format!(
            "{}{}",
            format!("{}{}", parent, "/".to_string()),
            name
        ));
    }
    fn with_suffix(&self, suffix: &String) -> Path {
        let s: String = stem(&self._path.clone());
        let parent: String = dirname(&self._path.clone());
        if parent == "".to_string() {
            return Path::new(format!("{}{}", s, suffix));
        }
        return Path::new(format!(
            "{}{}",
            format!("{}{}", format!("{}{}", parent, "/".to_string()), s),
            suffix
        ));
    }
    fn glob(&self, pattern: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _glob_to_iter(&self._path.clone(), pattern);
    }
    fn rglob(&self, pattern: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _rglob_to_iter(&self._path.clone(), pattern);
    }
}
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Path(_path={})", self._path);
    }
}
fn join_path(base: &String, child: &String) -> String {
    if (base.len() as i64) == (0 as i64) {
        return format!("{}{}", child, "".to_string());
    }
    let last: Option<String> = {
        let __sifr_index_str = &base;
        let __sifr_index_i = (base.chars().count() as i64) - (1 as i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str
            .chars()
            .nth(__sifr_index_norm)
            .map(|c| c.to_string())
    };
    if let Some(last) = last {
        if last == "/".to_string() {
            return format!("{}{}", base, child);
        }
    }
    return format!("{}{}{}", base, "/".to_string(), child);
}
fn basename(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter((path).chars().skip((i + (1 as i64)).max(0) as usize));
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", path, "".to_string());
}
fn dirname(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter(
                    (path)
                        .chars()
                        .skip(0 as usize)
                        .take(((i).max(0) - 0).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return "".to_string();
}
fn extension(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter((path).chars().skip((i).max(0) as usize));
            }
            if ch == "/".to_string() {
                return "".to_string();
            }
        }
        i = i - (1 as i64);
    }
    return "".to_string();
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let mut i: i64 = (base.chars().count() as i64) - (1 as i64);
    while i > (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = base.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter(
                    (base)
                        .chars()
                        .skip(0 as usize)
                        .take(((i).max(0) - 0).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", base, "".to_string());
}
fn is_absolute(path: &String) -> bool {
    if (path.len() as i64) == (0 as i64) {
        return false;
    }
    if (path.chars().count() as i64) >= (3 as i64) {
        let colon: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 1 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        let sep: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 2 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":".to_string())
                    && ((sep == "/".to_string()) || (sep == "\\".to_string()))
                {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = path.chars().nth((0 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    });
    if let Some(first) = first {
        if (first == "/".to_string()) || (first == "\\".to_string()) {
            return true;
        }
    }
    return false;
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<String> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<String> = Vec::new();
            let mut i: i64 = 0 as i64;
            while i < (entries.len() as i64) {
                _yields.push(entries[i as usize].clone());
                i = i + (1 as i64);
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    return {
        let __entries = std::fs::read_dir(&path).map_err(__io_err)?;
        Ok(__entries
            .filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string()))
            .collect::<Vec<String>>())
    };
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$",
            regex::escape(__pat)
                .replace("\\*", ".*")
                .replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src).map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        match std::fs::read_dir(__dir) {
            Ok(__entries) => {
                for __entry in __entries {
                    if let Ok(__e) = __entry {
                        let __name = __e.file_name().to_string_lossy().to_string().to_string();
                        if !__include_hidden && __name.starts_with(".") {
                            continue;
                        }
                        if __re.is_match(&__name) {
                            __results.push(__e.path().to_string_lossy().to_string());
                        }
                    } else {
                        continue;
                    }
                }
            }
            Err(_) => {
                return Ok(Vec::new());
            }
        }
        __results.sort();
        Ok(__results)
    };
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$",
            regex::escape(__pat)
                .replace("\\*", ".*")
                .replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src).map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        let mut __stack: Vec<String> = vec![__dir.to_string()];
        loop {
            if let Some(__current) = __stack.pop() {
                let __entries_result = std::fs::read_dir(&__current);
                if let Ok(__entries) = __entries_result {
                    for __entry in __entries {
                        if let Ok(__e) = __entry {
                            let __path = __e.path();
                            let __name = __e.file_name().to_string_lossy().to_string().to_string();
                            if !__include_hidden && __name.starts_with(".") {
                                continue;
                            }
                            if __path.is_dir() {
                                __stack.push(__path.to_string_lossy().to_string());
                            }
                            if __re.is_match(&__name) {
                                __results.push(__path.to_string_lossy().to_string());
                            }
                        } else {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            } else {
                break;
            }
        }
        __results.sort();
        Ok(__results)
    };
}
fn _iterdir_to_iter(path: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
            let entries: Vec<String> = _iterdir_list(path)?;
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}
fn _glob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
            let entries: Vec<String> = _glob_list(path, pattern)?;
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}
fn _rglob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
            let entries: Vec<String> = _rglob_list(path, pattern)?;
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}

// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timezone {
    _offset: i64,
}
impl timezone {
    fn new(offset: i64) -> Self {
        return Self { _offset: offset };
    }
    fn offset(&self) -> i64 {
        return self._offset;
    }
    fn iso_suffix(&self) -> String {
        let mut sign: String = "+".to_string();
        if self._offset < (0 as i64) {
            sign = "-".to_string();
        }
        let mut abs_offset: i64 = self._offset;
        if abs_offset < (0 as i64) {
            abs_offset = -abs_offset;
        }
        let h: i64 = abs_offset / (3600 as i64);
        let m: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
        let mut hs: String = format!("{}", h);
        if (hs.len() as i64) < (2 as i64) {
            hs = format!("{}{}", "0".to_string(), hs);
        }
        let mut ms: String = format!("{}", m);
        if (ms.len() as i64) < (2 as i64) {
            ms = format!("{}{}", "0".to_string(), ms);
        }
        return format!("{}{}{}{}", sign, hs, ":".to_string(), ms);
    }
}
impl PartialEq for timezone {
    fn eq(&self, other: &timezone) -> bool {
        return self._offset == other._offset;
    }
}
impl std::fmt::Display for timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self._offset == (0 as i64) {
            return write!(f, "{}", "UTC".to_string());
        }
        return write!(
            f,
            "{}",
            format!("{}{}", "UTC".to_string(), self.iso_suffix())
        );
    }
}
#[derive(Debug, Clone)]
struct datetime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    _tz_offset: Option<i64>,
}
impl datetime {
    fn new(
        year: i64,
        month: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        tz_offset: Option<i64>,
    ) -> Self {
        return Self {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            _tz_offset: tz_offset,
        };
    }
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if (mo.len() as i64) < (2 as i64) {
            mo = format!("{}{}", "0".to_string(), mo);
        }
        let mut d: String = format!("{}", self.day);
        if (d.len() as i64) < (2 as i64) {
            d = format!("{}{}", "0".to_string(), d);
        }
        let mut h: String = format!("{}", self.hour);
        if (h.len() as i64) < (2 as i64) {
            h = format!("{}{}", "0".to_string(), h);
        }
        let mut mi: String = format!("{}", self.minute);
        if (mi.len() as i64) < (2 as i64) {
            mi = format!("{}{}", "0".to_string(), mi);
        }
        let mut s: String = format!("{}", self.second);
        if (s.len() as i64) < (2 as i64) {
            s = format!("{}{}", "0".to_string(), s);
        }
        let base: String = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            y,
            "-".to_string(),
            mo,
            "-".to_string(),
            d,
            "T".to_string(),
            h,
            ":".to_string(),
            mi,
            ":".to_string(),
            s
        );
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            let mut sign: String = "+".to_string();
            let mut abs_offset: i64 = offset;
            if abs_offset < (0 as i64) {
                sign = "-".to_string();
                abs_offset = -abs_offset;
            }
            let h_off: i64 = abs_offset / (3600 as i64);
            let m_off: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
            let mut hs_off: String = format!("{}", h_off);
            if (hs_off.len() as i64) < (2 as i64) {
                hs_off = format!("{}{}", "0".to_string(), hs_off);
            }
            let mut ms_off: String = format!("{}", m_off);
            if (ms_off.len() as i64) < (2 as i64) {
                ms_off = format!("{}{}", "0".to_string(), ms_off);
            }
            return format!("{}{}{}{}{}", base, sign, hs_off, ":".to_string(), ms_off);
        }
        return base;
    }
    fn timestamp(&self) -> i64 {
        let mut days: i64 = 0 as i64;
        if self.year >= (1970 as i64) {
            let mut y: i64 = 1970 as i64;
            while y < self.year {
                days = days + _days_in_year(y);
                y = y + (1 as i64);
            }
        } else {
            let mut y: i64 = 1969 as i64;
            while y >= self.year {
                days = days - _days_in_year(y);
                y = y - (1 as i64);
            }
        }
        let mut m: i64 = 1 as i64;
        while m < self.month {
            days = days + _days_in_month(self.year, m);
            m = m + (1 as i64);
        }
        days = (days + self.day) - (1 as i64);
        let naive_timestamp: i64 = (((days * (86400 as i64)) + (self.hour * (3600 as i64)))
            + (self.minute * (60 as i64)))
            + self.second;
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            return naive_timestamp - offset;
        }
        return naive_timestamp;
    }
    fn astimezone(&self, tz: &Option<timezone>) -> Result<datetime, ValueError> {
        let mut target: timezone = timezone::new(0 as i64);
        if let Some(tz) = tz.as_ref() {
            let __sifr_try_res: Result<(), ValueError> = (|| {
                let tz_text: String = format!("{}", tz);
                let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                target = timezone::new(target_offset);
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(ValueError::new(e.message));
            }
        }
        return from_timestamp(self.timestamp() as f64, &Some(target));
    }
}
impl PartialEq for datetime {
    fn eq(&self, other: &datetime) -> bool {
        let same_tz: bool = self._tz_offset == other._tz_offset;
        return (((((((self.year == other.year) && (self.month == other.month))
            && (self.day == other.day))
            && (self.hour == other.hour))
            && (self.minute == other.minute))
            && (self.second == other.second))
            && (same_tz));
    }
}
impl std::fmt::Display for datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.isoformat());
    }
}
#[derive(Debug, Clone)]
struct time {
    hour: i64,
    minute: i64,
    second: i64,
}
impl time {
    fn new(hour: i64, minute: i64, second: i64) -> Self {
        return Self {
            hour: hour,
            minute: minute,
            second: second,
        };
    }
    fn isoformat(&self) -> String {
        let mut h: String = format!("{}", self.hour);
        if (h.len() as i64) < (2 as i64) {
            h = format!("{}{}", "0".to_string(), h);
        }
        let mut mi: String = format!("{}", self.minute);
        if (mi.len() as i64) < (2 as i64) {
            mi = format!("{}{}", "0".to_string(), mi);
        }
        let mut s: String = format!("{}", self.second);
        if (s.len() as i64) < (2 as i64) {
            s = format!("{}{}", "0".to_string(), s);
        }
        return format!("{}{}{}{}{}", h, ":".to_string(), mi, ":".to_string(), s);
    }
}
impl PartialEq for time {
    fn eq(&self, other: &time) -> bool {
        return (((self.hour == other.hour) && (self.minute == other.minute))
            && (self.second == other.second));
    }
}
impl std::fmt::Display for time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.isoformat());
    }
}
fn _is_leap_year(year: i64) -> bool {
    return (((year % (4 as i64)) == (0 as i64)) && ((year % (100 as i64)) != (0 as i64)))
        || ((year % (400 as i64)) == (0 as i64));
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366 as i64;
    }
    return 365 as i64;
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31 as i64, 28 as i64, 31 as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64, 31 as i64,
        30 as i64, 31 as i64, 30 as i64, 31 as i64,
    ];
    let idx: i64 = month - (1 as i64);
    let d: Option<i64> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if ((month == (2 as i64)) && (_is_leap_year(year))) {
        return 29 as i64;
    }
    if let Some(d) = d {
        return d;
    }
    return 0 as i64;
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _parse_datetime_iso(value: &String) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    if (value.chars().count() as i64) < (19 as i64) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if (((((({
        let Some(__indexed_char) = value.chars().nth((4 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    }) != "-".to_string())
        || (({
            let Some(__indexed_char) = value.chars().nth((7 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "-".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((10 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "T".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((13 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((16 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<Result<(i64, i64, i64, i64, i64, i64), ValueError>, ParseError> =
        (|| {
            let year: i64 = (_substring(value, 0 as i64, 4 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let month: i64 = (_substring(value, 5 as i64, 7 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let day: i64 = (_substring(value, 8 as i64, 10 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let hour: i64 = (_substring(value, 11 as i64, 13 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minute: i64 = (_substring(value, 14 as i64, 16 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let second: i64 = (_substring(value, 17 as i64, 19 as i64))
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(Ok((year, month, day, hour, minute, second)));
            unreachable!("sifr try/except return capture fell through");
        })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
    }
}
fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
    if text.clone() == "UTC".to_string() {
        return Ok(0 as i64);
    }
    if (text.chars().count() as i64) != (9 as i64) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if _substring(text, 0 as i64, 3 as i64) != "UTC".to_string() {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3 as i64, 4 as i64);
    if (sign_value != "+".to_string()) && (sign_value != "-".to_string()) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if ({
        let __sifr_index_str = &text;
        let __sifr_index_i = 6 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str
            .chars()
            .nth(__sifr_index_norm)
            .map(|c| c.to_string())
    }) != Some(":".to_string())
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4 as i64, 6 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7 as i64, 9 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600 as i64)) + (minutes * (60 as i64));
        if sign_value == "-".to_string() {
            offset = -offset;
        }
        return Ok(Ok(offset));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
    }
}
fn _from_timestamp_with_tz(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    let __sifr_try_res: Result<Result<datetime, ValueError>, ValueError> = (|| {
        let whole_seconds: i64 = ts as i64;
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0 as i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = ({
            let __ts = (adjusted_seconds as f64) as i64;
            chrono::DateTime::from_timestamp(__ts, 0)
                .map(|dt| dt.format(&"%Y-%m-%dT%H:%M:%S".to_string()).to_string())
                .ok_or_else(|| ValueError {
                    message: "invalid timestamp".to_string(),
                })
        })?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0 as i64;
        let mut month: i64 = 1 as i64;
        let mut day: i64 = 1 as i64;
        let mut hour: i64 = 0 as i64;
        let mut minute: i64 = 0 as i64;
        let mut second: i64 = 0 as i64;
        if let Some(year_part) = year_part {
            year = year_part;
        }
        if let Some(month_part) = month_part {
            month = month_part;
        }
        if let Some(day_part) = day_part {
            day = day_part;
        }
        if let Some(hour_part) = hour_part {
            hour = hour_part;
        }
        if let Some(minute_part) = minute_part {
            minute = minute_part;
        }
        if let Some(second_part) = second_part {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(Ok(datetime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                Some(tz_offset_value),
            )));
        }
        return Ok(Ok(datetime::new(
            year, month, day, hour, minute, second, None,
        )));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn now(tz: &Option<timezone>) -> datetime {
    let current_epoch: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let __sifr_try_res: Result<datetime, ValueError> = (|| {
        let current: datetime = _from_timestamp_with_tz(current_epoch, tz)?;
        return Ok(current);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<i64> = {
                let __dt = chrono::Local::now();
                vec![
                    chrono::Datelike::year(&__dt) as i64,
                    chrono::Datelike::month(&__dt) as i64,
                    chrono::Datelike::day(&__dt) as i64,
                    chrono::Timelike::hour(&__dt) as i64,
                    chrono::Timelike::minute(&__dt) as i64,
                    chrono::Timelike::second(&__dt) as i64,
                ]
            };
            let mut yr: i64 = 0 as i64;
            let mut mo: i64 = 1 as i64;
            let mut dy: i64 = 1 as i64;
            let mut hr: i64 = 0 as i64;
            let mut mn: i64 = 0 as i64;
            let mut sc: i64 = 0 as i64;
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if i == (0 as i64) {
                    yr = v;
                }
                if i == (1 as i64) {
                    mo = v;
                }
                if i == (2 as i64) {
                    dy = v;
                }
                if i == (3 as i64) {
                    hr = v;
                }
                if i == (4 as i64) {
                    mn = v;
                }
                if i == (5 as i64) {
                    sc = v;
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<datetime, ValueError> = (|| {
                    let parsed_offset: i64 = _timezone_offset_from_text(&format!("{}", tz))?;
                    return Ok(datetime::new(yr, mo, dy, hr, mn, sc, Some(parsed_offset)));
                    unreachable!("sifr try/except return capture fell through");
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return datetime::new(yr, mo, dy, hr, mn, sc, None);
                    }
                }
            }
            return datetime::new(yr, mo, dy, hr, mn, sc, None);
        }
    }
}
fn from_timestamp(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    return _from_timestamp_with_tz(ts, tz);
}

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError {
        message: e.to_string(),
    });
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    return {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError {
                message: e.to_string(),
            })?);
        }
        Ok(result)
    };
}
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
    return {
        let __vals = values;
        let mut __out = Vec::new();
        for __pair in __vals.iter().enumerate() {
            if (*__pair.1 < 0) || (*__pair.1 > 255) {
                return Err(ValueError {
                    message: format!("byte out of range at index {}: {}", __pair.0, *__pair.1),
                });
            }
            __out.push(*__pair.1 as u8);
        }
        Ok(__out)
    };
}
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    return {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
    };
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    return Ok({
        let __s = s;
        __s.as_bytes().to_vec()
    });
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            count = count + (1 as i64);
        }
    }
    return count;
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            return Some(idx);
        }
        idx = idx + (1 as i64);
    }
    return None;
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.len() as i64) {
        let a: Option<i64> = data.get(i as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = prefix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (suffix.len() as i64) {
        let a: Option<i64> = data.get((offset + i) as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = suffix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0 as i64) {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 2 as i64;
    while i <= n {
        result = result * i;
        i = i + (1 as i64);
    }
    return result;
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    while y != (0 as i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    return x;
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0 as i64) {
        return 0 as i64;
    }
    if b == (0 as i64) {
        return 0 as i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    let mut y: i64 = b;
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    return (x / g) * y;
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    if k == (0 as i64) {
        return 1 as i64;
    }
    if k == n {
        return 1 as i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < r {
        result = result * (n - i);
        result = result / (i + (1 as i64));
        i = i + (1 as i64);
    }
    return result;
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < k {
        result = result * (n - i);
        i = i + (1 as i64);
    }
    return result;
}
fn log_base(x: f64, base: f64) -> f64 {
    return (x).ln() / (base).ln();
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0 as f64) {
        return false;
    }
    if abs_tol < (0.0 as f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if (((a).is_nan()) || ((b).is_nan())) {
        return false;
    }
    if (((a).is_infinite()) || ((b).is_infinite())) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0 as f64) {
        a_abs = (0.0 as f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0 as f64) {
        b_abs = (0.0 as f64) - b_abs;
    }
    let mut rel_bound: f64 = rel_tol * (a_abs).max(b_abs);
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    return diff <= rel_bound;
}
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1 as i64;
    for val in data.iter().copied() {
        result = result * val;
    }
    return result;
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __sfrac);
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __frac);
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return f64::NAN;
    };
    return m;
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __sfrac);
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __frac);
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0 as i64;
    };
    return (exp_val).trunc() as i64;
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return f64::NAN;
    };
    return f;
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return f64::NAN;
    };
    return i;
}
fn pow(x: f64, y: f64) -> f64 {
    return (x).powf(y);
}

// --- stdlib: sifr.random ---
#[derive(Debug, Clone)]
struct __SifrRandomModuleState {
    words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
static __SIFR_RANDOM_MODULE_STATE: std::sync::LazyLock<std::sync::Mutex<__SifrRandomModuleState>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(__SifrRandomModuleState {
            words: Vec::new(),
            index: 0,
            gauss_next: None,
        })
    });
const _MT_N: i64 = 624 as i64;
const _MT_M: i64 = 397 as i64;
const _MT_MATRIX_A: i64 = 2567483615 as i64;
const _MT_UPPER_MASK: i64 = 2147483648 as i64;
const _MT_LOWER_MASK: i64 = 2147483647 as i64;
const _MT_F: i64 = 1812433253 as i64;
const _MT_WORD_MASK: i64 = 4294967295 as i64;
#[derive(Debug, Clone, PartialEq)]
struct RandomState {
    version: i64,
    state_words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
impl RandomState {
    fn new(version: i64, state_words: Vec<i64>, index: i64, gauss_next: Option<f64>) -> Self {
        return Self {
            version: version,
            state_words: state_words,
            index: index,
            gauss_next: gauss_next,
        };
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Random {
    _state_words: Vec<i64>,
    _index: i64,
    _gauss_next: Option<f64>,
}
impl Random {
    fn new(seed_value: Option<i64>) -> Self {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        return Self {
            _state_words: _seed_words_from_seed(normalized_seed),
            _index: _MT_N,
            _gauss_next: None,
        };
    }
    fn seed(&mut self, seed_value: Option<i64>) {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        self._state_words = _seed_words_from_seed(normalized_seed);
        self._index = _MT_N;
        self._gauss_next = None;
    }
    fn _twist(&mut self) {
        let mut i: i64 = 0 as i64;
        while i < _MT_N {
            let y: i64 = (_state_word_at(&self._state_words.clone(), i) & _MT_UPPER_MASK)
                + (_state_word_at(&self._state_words.clone(), (i + (1 as i64)) % _MT_N)
                    & _MT_LOWER_MASK);
            let mut x_a: i64 = y >> (1 as i64);
            if (y % (2 as i64)) != (0 as i64) {
                x_a = x_a ^ _MT_MATRIX_A;
            }
            let new_word: i64 =
                _state_word_at(&self._state_words.clone(), (i + _MT_M) % _MT_N) ^ x_a;
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 {
                    (self._state_words.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = self._state_words.get_mut(__idx_norm as usize) {
                        *__elem = new_word & _MT_WORD_MASK;
                    }
                }
            }
            i = i + (1 as i64);
        }
        self._index = 0 as i64;
    }
    fn _next_u32(&mut self) -> i64 {
        if self._index >= _MT_N {
            self._twist();
        }
        let mut y: i64 = _state_word_at(&self._state_words.clone(), self._index);
        self._index = self._index + (1 as i64);
        y = y ^ (y >> (11 as i64));
        y = y ^ ((y << (7 as i64)) & (2636928640 as i64));
        y = y ^ ((y << (15 as i64)) & (4022730752 as i64));
        y = y ^ (y >> (18 as i64));
        return y & _MT_WORD_MASK;
    }
    fn random(&mut self) -> f64 {
        return (self._next_u32() as f64) / (4294967296.0 as f64);
    }
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        return minimum + ((maximum - minimum) * self.random());
    }
    fn randrange(&mut self, start: i64, stop: Option<i64>, step: i64) -> Result<i64, ValueError> {
        if step == (0 as i64) {
            return Err(ValueError::new(
                "randrange: step must not be zero".to_string(),
            ));
        }
        let mut actual_start: i64 = start;
        let mut actual_stop: i64 = start;
        if stop.is_none() {
            actual_start = 0 as i64;
        } else {
            if let Some(stop) = stop {
                actual_stop = stop;
            }
        }
        let width: i64 = actual_stop - actual_start;
        if step > (0 as i64) {
            if width <= (0 as i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if width >= (0 as i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: i64 = width;
        if abs_width < (0 as i64) {
            abs_width = (0 as i64) - abs_width;
        }
        let mut abs_step: i64 = step;
        if abs_step < (0 as i64) {
            abs_step = (0 as i64) - abs_step;
        }
        let count: i64 = ((abs_width + abs_step) - (1 as i64)) / abs_step;
        if count <= (0 as i64) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: i64 = self._next_u32() % count;
        return Ok(actual_start + (pick * step));
    }
    fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        return self.randrange(minimum, Some(maximum + (1 as i64)), 1 as i64);
    }
    fn getrandbits(&mut self, k: i64) -> Result<i64, ValueError> {
        if k < (0 as i64) {
            return Err(ValueError::new(
                "getrandbits: number of bits must be >= 0".to_string(),
            ));
        }
        let mut result: i64 = 0 as i64;
        let mut bits_left: i64 = k;
        while bits_left > (0 as i64) {
            let word: i64 = self._next_u32();
            let mut take: i64 = 32 as i64;
            if bits_left < (32 as i64) {
                take = bits_left;
            }
            let mask: i64 = ((1 as i64) << take) - (1 as i64);
            result = (result << take) | (word & mask);
            bits_left = bits_left - take;
        }
        return Ok(result);
    }
    fn randbytes(&mut self, n: i64) -> Result<Vec<u8>, ValueError> {
        if n < (0 as i64) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 0 as i64;
        while i < n {
            let byte_value: i64 = self._next_u32() & (255 as i64);
            values.push(byte_value);
            i = i + (1 as i64);
        }
        return {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!("byte out of range at index {}: {}", __pair.0, *__pair.1),
                    });
                }
                __out.push(*__pair.1 as u8);
            }
            Ok(__out)
        };
    }
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0 as f64) {
            u1 = 0.000000000001 as f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = (-(2.0 as f64) * (u1).ln()).sqrt();
        let theta: f64 = ((2.0 as f64) * std::f64::consts::PI) * u2;
        let z0: f64 = radius * (theta).cos();
        let z1: f64 = radius * (theta).sin();
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        return mu + (sigma * z0);
    }
    fn getstate(&self) -> RandomState {
        return RandomState::new(
            3 as i64,
            _clone_words(&self._state_words.clone()),
            self._index,
            self._gauss_next,
        );
    }
    fn setstate(&mut self, state: &RandomState) -> Result<(), ValueError> {
        if state.version != (3 as i64) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (state.state_words.len() as i64) != _MT_N {
            return Err(ValueError::new(
                "setstate: state_words must have length 624".to_string(),
            ));
        }
        if ((state.index < (0 as i64)) || (state.index > _MT_N)) {
            return Err(ValueError::new(
                "setstate: index must be in range [0, 624]".to_string(),
            ));
        }
        let mut normalized: Vec<i64> = vec![];
        for word in state.state_words.iter().copied() {
            if (word < (0 as i64)) || (word > _MT_WORD_MASK) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(word & _MT_WORD_MASK);
        }
        self._state_words = normalized;
        self._index = state.index;
        self._gauss_next = state.gauss_next;
        return Ok(());
    }
}
fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
    let value: Option<i64> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(value) = value {
        return value;
    }
    return 0 as i64;
}
fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for word in words.iter().copied() {
        copied.push(word);
    }
    return copied;
}
fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
    if let Some(seed_value) = seed_value {
        return seed_value;
    }
    return (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * (1000000.0 as f64)) as i64;
}
fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
    let mut words: Vec<i64> = vec![];
    words.push(seed_value & _MT_WORD_MASK);
    let mut i: i64 = 1 as i64;
    while i < _MT_N {
        let prev: i64 = _state_word_at(&words, i - (1 as i64));
        let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30 as i64)))) + i) & _MT_WORD_MASK;
        words.push(next_word);
        i = i + (1 as i64);
    }
    return words;
}
fn _build_state_from_module_storage() -> RandomState {
    return RandomState::new(
        3 as i64,
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.words.clone()
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.index
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.gauss_next.clone()
        },
    );
}
fn _store_state_into_module_storage(state: &RandomState) {
    let _set_result: Result<(), ValueError> = {
        let __words = _clone_words(&state.state_words);
        let __index = state.index;
        let __gauss_next = state.gauss_next;
        if (__index < 0) || (__index > 624) {
            Err(ValueError {
                message: "random module state index must be in range [0, 624]".to_string(),
            })
        } else {
            if __words.len() != 624 {
                Err(ValueError {
                    message: "random module state words must have length 624".to_string(),
                })
            } else {
                {
                    let mut __state = __SIFR_RANDOM_MODULE_STATE
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner());
                    __state.words = __words;
                    __state.index = __index;
                    __state.gauss_next = __gauss_next;
                    Ok(())
                }
            }
        }
    };
    let _: Result<(), ValueError> = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = {
        let __state = __SIFR_RANDOM_MODULE_STATE
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        __state.words.clone()
    };
    if (words.len() as i64) == _MT_N {
        return;
    }
    let mut bootstrap: Random = Random::new(Some(5489 as i64));
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> Random {
    _ensure_module_state_initialized();
    let mut r: Random = Random::new(Some(0 as i64));
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r.setstate(&_build_state_from_module_storage());
        let _: Result<(), ValueError> = _set_result;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    return r;
}
fn _sync_module_random(generator: &mut Random) {
    _store_state_into_module_storage(&generator.getstate());
}
fn choice<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
) -> Result<T, ValueError> {
    if (items.len() as i64) == (0 as i64) {
        return Err(ValueError::new(
            "choice: items must not be empty".to_string(),
        ));
    }
    let mut generator: Random = _module_random();
    let index: i64 = generator._next_u32() % (items.len() as i64);
    let picked: Option<T> = {
        let __sifr_index_list = &items;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    _sync_module_random(&mut generator);
    if let Some(picked) = picked {
        return Ok(picked);
    }
    return Err(ValueError::new("choice: index out of range".to_string()));
}

// --- stdlib: sifr.logging ---
#[derive(Debug, Clone)]
struct FileHandle {
    _handle: i64,
    _mode: String,
}
impl FileHandle {
    fn new(_handle: i64, _mode: String) -> Self {
        return Self {
            _handle: _handle,
            _mode: _mode,
        };
    }
    fn read(&self) -> Result<String, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __s = String::new();
                std::io::Read::read_to_string(__r, &mut __s).map_err(__io_err)?;
                return Ok(__s);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, data.as_bytes()).map_err(__io_err)?;
                return Ok(());
            }
            _ => {
                return Err(IOError {
                    message: "file not open for writing".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __line = String::new();
                let __n = std::io::BufRead::read_line(__r, &mut __line).map_err(__io_err)?;
                if __n == 0 {
                    return Ok(None);
                }
                if __line.ends_with('\n') {
                    __line.pop();
                    if __line.ends_with('\r') {
                        __line.pop();
                    }
                }
                return Ok(Some(__line));
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __lines: Vec<String> = Vec::<String>::new();
                let mut __line = String::new();
                loop {
                    __line.clear();
                    let __n = std::io::BufRead::read_line(__r, &mut __line).map_err(__io_err)?;
                    if __n == 0 {
                        break;
                    }
                    let mut __l = __line.clone();
                    if __l.ends_with('\n') {
                        __l.pop();
                        if __l.ends_with('\r') {
                            __l.pop();
                        }
                    }
                    __lines.push(__l);
                }
                return Ok(__lines);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn close(&self) {
        let __hid = self._handle;
        __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner())
            .remove(&__hid);
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                let mut __buf = Vec::<u8>::new();
                std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                return Ok(__buf);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for binary reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                return Ok(());
            }
            _ => {
                return Err(IOError {
                    message: "file not open for binary writing".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn __enter__(&self) -> &Self {
        return self;
    }
    fn __exit__(&self) {
        self.close();
    }
}
const DEBUG: i64 = 10 as i64;
const INFO: i64 = 20 as i64;
const WARNING: i64 = 30 as i64;
const ERROR: i64 = 40 as i64;
const CRITICAL: i64 = 50 as i64;
const NOTSET: i64 = 0 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Formatter {
    _fmt: String,
}
impl Formatter {
    fn new(fmt: String) -> Self {
        return Self { _fmt: fmt };
    }
    fn template(&self) -> String {
        return self._fmt.clone();
    }
    fn format(&self, level: &String, name: &String, msg: &String) -> String {
        let mut result: String = self._fmt.clone();
        result = result.replace(&"%(levelname)s".to_string(), &level);
        result = result.replace(&"%(name)s".to_string(), &name);
        result = result.replace(&"%(message)s".to_string(), &msg);
        return result;
    }
}
impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Formatter(_fmt={})", self._fmt);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StreamHandler {
    _level: i64,
    _formatter: Formatter,
}
impl StreamHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = self._formatter.clone().format(level, name, msg);
        println!("{}", line);
    }
}
impl std::fmt::Display for StreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "StreamHandler(_level={}, _formatter={})",
            self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileHandler {
    _path: String,
    _level: i64,
    _formatter: Formatter,
}
impl FileHandler {
    fn new(path: String, level: i64) -> Self {
        return Self {
            _path: format!("{}{}", path, "".to_string()),
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn path(&self) -> String {
        return self._path.clone();
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = format!(
            "{}{}",
            self._formatter.clone().format(level, name, msg),
            "\n".to_string()
        );
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: FileHandle = (|| {
                let __path = self._path.clone().to_string();
                let __mode = "a".to_string().to_string();
                let __handle_id = __sifr_next_file_handle_id();
                match __mode.as_str() {
                    "r" | "rt" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "w" | "wt" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "a" | "at" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "rb" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "wb" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "ab" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    _ => {
                        return Err(IOError {
                            message: format!("invalid mode: {}", __mode),
                            kind: "Other".to_string(),
                        });
                    }
                }
            })()?;
            let __sifr_try_res: Result<(), IOError> = (|| {
                let _: () = fh.write(&line)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e2 = __sifr_try_err.clone();
                let _: String = e2.message;
            }
            fh.close();
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _: String = e.message;
        }
    }
}
impl std::fmt::Display for FileHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "FileHandler(_path={}, _level={}, _formatter={})",
            self._path, self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NullHandler {
    _level: i64,
    _formatter: Formatter,
}
impl NullHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let _: String = (level).clone();
        let _: String = (name).clone();
        let _: String = (msg).clone();
    }
}
impl std::fmt::Display for NullHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "NullHandler(_level={}, _formatter={})",
            self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Logger {
    _name: String,
    _level: i64,
    _log_path: String,
    _handler_kind: String,
    _handler_path: String,
    _handler_level: i64,
    _handler_fmt: String,
}
impl Logger {
    fn new(name: String, level: i64) -> Self {
        return Self {
            _name: name,
            _level: level,
            _log_path: "".to_string(),
            _handler_kind: "".to_string(),
            _handler_path: "".to_string(),
            _handler_level: NOTSET,
            _handler_fmt: "%(levelname)s:%(name)s:%(message)s".to_string(),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn set_file(&mut self, path: &String) {
        self._log_path = format!("{}{}", path, "".to_string());
    }
    fn add_handler(&mut self, handler: &FileHandler) {
        self._handler_kind = "file".to_string();
        self._handler_path = handler.path();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_stream_handler(&mut self, handler: &StreamHandler) {
        self._handler_kind = "stream".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_null_handler(&mut self, handler: &NullHandler) {
        self._handler_kind = "null".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn clear_handler(&mut self) {
        self._handler_kind = "".to_string();
        self._handler_path = "".to_string();
        self._handler_level = NOTSET;
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn _handler_allows(&self, level_num: i64) -> bool {
        if self._handler_level == NOTSET {
            return true;
        }
        return level_num >= self._handler_level;
    }
    fn _handler_line(&self, level: &String, msg: &String) -> String {
        let mut formatter: Formatter = Formatter::new(self._handler_fmt.clone());
        return formatter.format(level, &self._name.clone(), msg);
    }
    fn _emit(&self, level: &String, level_num: i64, msg: &String) {
        if self._level > level_num {
            return;
        }
        if self._handler_kind.clone() == "null".to_string() {
            return;
        }
        if self._handler_kind.clone() == "stream".to_string() {
            if self._handler_allows(level_num) {
                println!("{}", self._handler_line(level, msg));
            }
            return;
        }
        if self._handler_kind.clone() == "file".to_string() {
            if ((self._handler_allows(level_num)) && (self._handler_path.clone() != "".to_string()))
            {
                let line: String =
                    format!("{}{}", self._handler_line(level, msg), "\n".to_string());
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: FileHandle = (|| {
                        let __path = self._handler_path.clone().to_string();
                        let __mode = "a".to_string().to_string();
                        let __handle_id = __sifr_next_file_handle_id();
                        match __mode.as_str() {
                            "r" | "rt" => {
                                let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "w" | "wt" => {
                                let __f =
                                    std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "a" | "at" => {
                                let __f = std::fs::OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "rb" => {
                                let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "wb" => {
                                let __f =
                                    std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "ab" => {
                                let __f = std::fs::OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            _ => {
                                return Err(IOError {
                                    message: format!("invalid mode: {}", __mode),
                                    kind: "Other".to_string(),
                                });
                            }
                        }
                    })()?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _: () = fh.write(&line)?;
                        return Ok(());
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e2 = __sifr_try_err.clone();
                        let _: String = e2.message;
                    }
                    fh.close();
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _: String = e.message;
                }
            }
            return;
        }
        let line: String = format!(
            "{}{}{}{}{}{}",
            "[".to_string(),
            level,
            "] ".to_string(),
            self._name.clone(),
            ": ".to_string(),
            msg
        );
        println!("{}", line);
        if self._log_path.clone() != "".to_string() {
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: FileHandle = (|| {
                    let __path = self._log_path.clone().to_string();
                    let __mode = "a".to_string().to_string();
                    let __handle_id = __sifr_next_file_handle_id();
                    match __mode.as_str() {
                        "r" | "rt" => {
                            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "w" | "wt" => {
                            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "a" | "at" => {
                            let __f = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "rb" => {
                            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "wb" => {
                            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "ab" => {
                            let __f = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        _ => {
                            return Err(IOError {
                                message: format!("invalid mode: {}", __mode),
                                kind: "Other".to_string(),
                            });
                        }
                    }
                })()?;
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let _: () = fh.write(&format!("{}{}", line, "\n".to_string()))?;
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e2 = __sifr_try_err.clone();
                    let _: String = e2.message;
                }
                fh.close();
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
            }
        }
    }
    fn debug(&self, msg: &String) {
        self._emit(&"DEBUG".to_string(), DEBUG, msg);
    }
    fn info(&self, msg: &String) {
        self._emit(&"INFO".to_string(), INFO, msg);
    }
    fn warning(&self, msg: &String) {
        self._emit(&"WARNING".to_string(), WARNING, msg);
    }
    fn error(&self, msg: &String) {
        self._emit(&"ERROR".to_string(), ERROR, msg);
    }
    fn critical(&self, msg: &String) {
        self._emit(&"CRITICAL".to_string(), CRITICAL, msg);
    }
}
impl std::fmt::Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
            self._name, self._level, self._log_path, self._handler_kind, self
            ._handler_path, self._handler_level, self._handler_fmt
        );
    }
}
fn _level_name_to_num(level: &String) -> i64 {
    if level.clone() == "DEBUG".to_string() {
        return DEBUG;
    }
    if level.clone() == "INFO".to_string() {
        return INFO;
    }
    if level.clone() == "WARNING".to_string() {
        return WARNING;
    }
    if level.clone() == "ERROR".to_string() {
        return ERROR;
    }
    if level.clone() == "CRITICAL".to_string() {
        return CRITICAL;
    }
    return NOTSET;
}
fn basicConfig(level: i64) -> Logger {
    {
        *__SIFR_GLOBAL_LOG_LEVEL
            .lock()
            .unwrap_or_else(|__err| __err.into_inner()) = level;
        ()
    };
    return Logger::new("root".to_string(), level);
}
fn getLogger(name: &String) -> Logger {
    let level: i64 = *__SIFR_GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(|__err| __err.into_inner());
    return Logger::new((name).clone(), level);
}

// --- stdlib: sifr.subprocess ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
    stderr: String,
}
impl CompletedProcess {
    fn new(returncode: i64, stdout: String, stderr: String) -> Self {
        return Self {
            returncode: returncode,
            stdout: stdout,
            stderr: stderr,
        };
    }
}
impl std::fmt::Display for CompletedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "CompletedProcess(returncode={}, stdout={}, stderr={})",
            self.returncode, self.stdout, self.stderr
        );
    }
}
fn run(cmd: &String) -> Result<CompletedProcess, IOError> {
    let __sifr_try_res: Result<Result<CompletedProcess, IOError>, IOError> = (|| {
        let result: Vec<String> = ({
            let __output = std::process::Command::new("sh".to_string())
                .arg("-c".to_string())
                .arg(&cmd)
                .output()
                .map_err(__io_err)?;
            let __stdout = String::from_utf8_lossy(&__output.stdout).to_string();
            let __stderr = String::from_utf8_lossy(&__output.stderr).to_string();
            let __returncode = __output.status.code().unwrap_or(-1).to_string();
            Ok(vec![__stdout, __stderr, __returncode])
        })?;
        let mut stdout: String = "".to_string();
        let mut stderr: String = "".to_string();
        let mut rc_str: String = "".to_string();
        let mut rc: i64 = 0 as i64;
        for (i, value) in Box::new(
            (result)
                .iter()
                .cloned()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
        ) {
            if i == (0 as i64) {
                stdout = format!("{}{}", value, "".to_string());
            }
            if i == (1 as i64) {
                stderr = format!("{}{}", value, "".to_string());
            }
            if i == (2 as i64) {
                rc_str = format!("{}{}", value, "".to_string());
            }
        }
        if rc_str != "".to_string() {
            let __sifr_try_res: Result<(), ParseError> = (|| {
                let parsed: i64 = (rc_str).parse::<i64>().map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
                rc = parsed;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
                rc = -(1 as i64);
            }
        }
        return Ok(Ok(CompletedProcess::new(rc, stdout, stderr)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
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

enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}

static __SIFR_FILE_HANDLES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(1);

fn __sifr_next_file_handle_id() -> i64 {
    return __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn main() {
    let path: String = "/tmp/sifr_demo_remediation.txt".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
        let mut f: FileHandle = (|| {
            let __path = path.to_string();
            let __mode = "w".to_string().to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        let _: () = f.write(&"hello from open()\n".to_string())?;
        let _2: () = f.write(&"second line\n".to_string())?;
        f.close();
        let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "open write ok = ".to_string(),
                format!("{}", (content.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "open write error: ".to_string(), e.message)
        );
    }
    let path2: String = "/tmp/sifr_demo_ctx.txt".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
        {
            let mut __ctx_0 = (|| {
                let __path = path2.to_string();
                let __mode = "w".to_string().to_string();
                let __handle_id = __sifr_next_file_handle_id();
                match __mode.as_str() {
                    "r" | "rt" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "w" | "wt" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "a" | "at" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "rb" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "wb" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "ab" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    _ => {
                        return Err(IOError {
                            message: format!("invalid mode: {}", __mode),
                            kind: "Other".to_string(),
                        });
                    }
                }
            })()?;
            struct __WithGuard0 {
                ctx: FileHandle,
            }
            impl Drop for __WithGuard0 {
                fn drop(&mut self) {
                    self.ctx.__exit__();
                }
            }
            let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
            let fw = __guard_0.ctx.__enter__();
            let _3: () = fw.write(&"context manager works".to_string())?;
        }
        let result: String = std::fs::read_to_string(&path2).map_err(__io_err)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "context manager ok = ".to_string(),
                format!("{}", result == "context manager works".to_string())
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "context manager error: ".to_string(), e.message)
        );
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let mut fr: FileHandle = (|| {
            let __path = path.to_string();
            let __mode = "r".to_string().to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(FileHandle {
                        _handle: __handle_id,
                        _mode: __mode.to_string(),
                    });
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        let content2: String = fr.read()?;
        fr.close();
        println!(
            "{}",
            format!(
                "{}{}",
                "open read ok = ".to_string(),
                format!("{}", (content2.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "open read error: ".to_string(), e.message)
        );
    }
    let mut t: time = time::new(10 as i64, 30 as i64, 45 as i64);
    println!(
        "{}",
        format!("{}{}", "time isoformat = ".to_string(), t.isoformat())
    );
    let t2: time = time::new(10 as i64, 30 as i64, 45 as i64);
    println!(
        "{}",
        format!("{}{}", "time eq = ".to_string(), format!("{}", t == t2))
    );
    let tz: timezone = timezone::new(0 as i64);
    println!(
        "{}",
        format!("{}{}", "timezone utc = ".to_string(), format!("{}", tz))
    );
    let mut dt: datetime = now(&None);
    let iso: String = dt.isoformat();
    println!(
        "{}",
        format!(
            "{}{}",
            "now isoformat ok = ".to_string(),
            format!("{}", (iso.chars().count() as i64) > (0 as i64))
        )
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let result2: CompletedProcess = run(&"echo hello_subprocess".to_string())?;
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess returncode = ".to_string(),
                format!("{}", result2.returncode)
            )
        );
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess stdout ok = ".to_string(),
                format!("{}", (result2.stdout.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "subprocess error: ".to_string(), e.message)
        );
    }
    let mut tmp: Path = Path::new("/tmp".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
        let mut matches_it: Box<dyn Iterator<Item = String>> =
            tmp.glob(&"sifr_demo_*".to_string())?;
        let matches: Vec<String> = matches_it.collect::<Vec<_>>();
        println!(
            "{}",
            format!(
                "{}{}",
                "glob found = ".to_string(),
                format!("{}", (matches.len() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "glob error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), RegexError> = (|| {
        let found: Option<String> =
            search_flags(&"hello".to_string(), &"HELLO WORLD".to_string(), IGNORECASE)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "re ignorecase = ".to_string(),
                format!("{}", found.is_some())
            )
        );
        let mut pat: Pattern = compile_flags(&"^line".to_string(), MULTILINE);
        let found2: Option<String> = pat.search(&"line1\nline2".to_string())?;
        println!(
            "{}",
            format!(
                "{}{}",
                "re multiline = ".to_string(),
                format!("{}", found2.is_some())
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "re error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let cwd: String = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "os getcwd ok = ".to_string(),
                format!("{}", (cwd.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "os getcwd error: ".to_string(), e.message)
        );
    }
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let picked: i64 = choice(&items)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "random choice ok = ".to_string(),
                format!("{}", (picked >= (1 as i64)) && (picked <= (5 as i64)))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "random choice error: ".to_string(), e.message)
        );
    }
    let mut root: Logger = basicConfig(WARNING);
    root.info(&"should not print".to_string());
    root.warning(&"root warning visible".to_string());
    let mut logger2: Logger = getLogger(&"myapp".to_string());
    logger2.info(&"should not print either".to_string());
    logger2.warning(&"myapp warning visible".to_string());
    println!("basicConfig global level ok");
    let mut handler: FileHandler =
        FileHandler::new("/tmp/sifr_demo_fh_log.txt".to_string(), 0 as i64);
    handler.emit(
        &"INFO".to_string(),
        &"demo".to_string(),
        &"file handler test".to_string(),
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let log_content: String =
            std::fs::read_to_string(&"/tmp/sifr_demo_fh_log.txt".to_string()).map_err(__io_err)?;
        println!(
            "{}",
            format!(
                "{}{}",
                "file handler wrote ok = ".to_string(),
                format!("{}", (log_content.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "file handler error: ".to_string(), e.message)
        );
    }
    let csv_path: String = "/tmp/sifr_demo_csv.csv".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _4: () = std::fs::write(
            &csv_path,
            "name,age\nalice,30\nbob,25".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        let mut r: reader = reader_from_path(
            &csv_path,
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &"".to_string(),
            true,
            false,
            0 as i64,
        )?;
        let rows: Vec<Vec<String>> = r.rows();
        println!(
            "{}",
            format!(
                "{}{}",
                "csv reader_from_path rows = ".to_string(),
                format!("{}", rows.len() as i64)
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "csv error: ".to_string(), e.message));
    }
}

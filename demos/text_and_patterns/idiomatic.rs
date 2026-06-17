use std::collections::HashMap;

// --- stdlib: sifr.textwrap ---
#[derive(Debug, Clone, PartialEq)]
struct TextWrapper {
    width: i64,
    initial_indent: String,
    subsequent_indent: String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
    drop_whitespace: bool,
    break_on_hyphens: bool,
    fix_sentence_endings: bool,
    max_lines: Option<i64>,
    placeholder: String,
}
impl TextWrapper {
    fn new(
        width: i64,
        initial_indent: String,
        subsequent_indent: String,
        expand_tabs: bool,
        tabsize: i64,
        replace_whitespace: bool,
        drop_whitespace: bool,
        break_on_hyphens: bool,
        fix_sentence_endings: bool,
        max_lines: Option<i64>,
        placeholder: String,
    ) -> Self {
        let mut safe_tabsize: i64 = tabsize;
        if safe_tabsize <= (0 as i64) {
            safe_tabsize = 1 as i64;
        }
        return Self {
            width: width,
            initial_indent,
            subsequent_indent,
            expand_tabs: expand_tabs,
            tabsize: safe_tabsize,
            replace_whitespace: replace_whitespace,
            drop_whitespace: drop_whitespace,
            break_on_hyphens: break_on_hyphens,
            fix_sentence_endings: fix_sentence_endings,
            max_lines: max_lines,
            placeholder,
        };
    }
    fn wrap(&self, text: &String) -> Vec<String> {
        if self.width <= (0 as i64) {
            return vec![];
        }
        let prepared: String = _prepare_text(
            text,
            self.expand_tabs,
            self.tabsize,
            self.replace_whitespace,
        );
        let mut lines: Vec<String> = _wrap_with_indents(
            &prepared,
            self.width,
            &self.initial_indent,
            &self.subsequent_indent,
            self.break_on_hyphens,
            self.drop_whitespace,
        );
        if self.fix_sentence_endings {
            lines = _apply_sentence_endings_lines(&lines);
        }
        return _apply_max_lines(
            &lines,
            self.width,
            self.max_lines,
            &self.placeholder,
            self.drop_whitespace,
        );
    }
    fn fill(&self, text: &String) -> String {
        if self.width <= (0 as i64) {
            return "".to_string();
        }
        let lines: Vec<String> = self.wrap(text);
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        for line in lines.iter().cloned() {
            if i > (0 as i64) {
                result = format!("{}{}", result, "\n".to_string());
            }
            result = format!("{}{}", result, line);
            i = i + (1 as i64);
        }
        return result;
    }
}
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
    let normalized: String = text
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    if replace_tabs {
        return normalized.replace(&"\t".to_string(), &" ".to_string());
    }
    return normalized;
}
fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
    let mut effective_tabsize: i64 = tabsize;
    if effective_tabsize <= (0 as i64) {
        effective_tabsize = 1 as i64;
    }
    let mut result: String = "".to_string();
    let mut column: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t".to_string() {
                let mut spaces: i64 = effective_tabsize - (column % effective_tabsize);
                if spaces <= (0 as i64) {
                    spaces = effective_tabsize;
                }
                let mut j: i64 = 0 as i64;
                while j < spaces {
                    result = format!("{}{}", result, " ".to_string());
                    j = j + (1 as i64);
                }
                column = column + spaces;
            } else {
                if (ch == "\n".to_string()) || (ch == "\r".to_string()) {
                    result = format!("{}{}", result, ch);
                    column = 0 as i64;
                } else {
                    result = format!("{}{}", result, ch);
                    column = column + (1 as i64);
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = format!("{}{}", text, "".to_string());
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize);
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    return prepared;
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let parts: Vec<String> = word
        .split(&"-".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) <= (1 as i64) {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let mut units: Vec<String> = vec![];
    let mut index: i64 = 0 as i64;
    for part in parts.iter().cloned() {
        let is_last: bool = index == ((parts.len() as i64) - (1 as i64));
        if is_last {
            if (part.chars().count() as i64) > (0 as i64) {
                units.push(part);
            }
        } else {
            if (part.chars().count() as i64) == (0 as i64) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-".to_string()));
            }
        }
        index = index + (1 as i64);
    }
    if (units.len() as i64) == (0 as i64) {
        units.push(format!("{}{}", word, "".to_string()));
    }
    return units;
}
fn _trim_line(line: &String) -> String {
    let mut start: i64 = 0 as i64;
    while ((start < (line.chars().count() as i64))
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = start;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        start = start + (1 as i64);
    }
    let mut end: i64 = line.chars().count() as i64;
    while ((end > start)
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = end - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        end = end - (1 as i64);
    }
    return String::from_iter(
        (line)
            .chars()
            .skip((start).max(0) as usize)
            .take(((end).max(0) - (start).max(0)).max(0) as usize),
    );
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    return format!("{}{}", line, "".to_string());
}
fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
    let available: i64 = total_width - (indent.chars().count() as i64);
    if available <= (0 as i64) {
        return 1 as i64;
    }
    return available;
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &String,
    indent: &String,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(&format!("{}{}", indent, line), drop_whitespace);
    if drop_whitespace {
        if (candidate.chars().count() as i64) > (0 as i64) {
            result.push(candidate);
        }
    } else {
        result.push(candidate);
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: i64,
    initial_indent: &String,
    subsequent_indent: &String,
    break_on_hyphens: bool,
    drop_whitespace: bool,
) -> Vec<String> {
    let words: Vec<String> = text
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: i64 = _effective_content_width(total_width, initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            if (word.chars().count() as i64) == (0 as i64) {
                if drop_whitespace {
                    continue;
                }
                if (current.chars().count() as i64) > (0 as i64) {
                    if ((current.chars().count() as i64) + (1 as i64)) <= current_limit {
                        current = format!("{}{}", current, " ".to_string());
                    }
                }
                continue;
            }
            if (current.chars().count() as i64) == (0 as i64) {
                current = word;
            } else {
                if (((current.chars().count() as i64) + (1 as i64)) + (word.chars().count() as i64))
                    <= current_limit
                {
                    current = format!("{}{}{}", current, " ".to_string(), word);
                } else {
                    if first_line {
                        _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
                        first_line = false;
                        current_limit = _effective_content_width(total_width, subsequent_indent);
                    } else {
                        _push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                }
            }
        }
    }
    if (current.chars().count() as i64) > (0 as i64) {
        if first_line {
            _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
        } else {
            _push_current_line(&mut result, &current, subsequent_indent, drop_whitespace);
        }
    }
    return result;
}
fn _apply_sentence_endings_line(text: &String) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            result = format!("{}{}", result, ch);
            if ((ch == ".".to_string()) || (ch == "!".to_string())) || (ch == "?".to_string()) {
                let mut next_opt: Option<String> = None;
                if (i + (1 as i64)) < (text.chars().count() as i64) {
                    next_opt = {
                        let __sifr_index_str = &text;
                        let __sifr_index_i = i + (1 as i64);
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
                }
                let mut next2_opt: Option<String> = None;
                if (i + (2 as i64)) < (text.chars().count() as i64) {
                    next2_opt = {
                        let __sifr_index_str = &text;
                        let __sifr_index_i = i + (2 as i64);
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
                }
                if ((next_opt != None) && (next_opt == Some(" ".to_string()))) {
                    if ((next2_opt == None) || (next2_opt != Some(" ".to_string()))) {
                        result = format!("{}{}", result, " ".to_string());
                    }
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _apply_sentence_endings_lines(lines: &[String]) -> Vec<String> {
    return lines
        .iter()
        .map(_apply_sentence_endings_line)
        .collect::<Vec<String>>();
}
fn _clone_lines(lines: &[String]) -> Vec<String> {
    return lines.to_vec();
}
fn _apply_max_lines(
    lines: &[String],
    width: i64,
    max_lines: Option<i64>,
    placeholder: &str,
    drop_whitespace: bool,
) -> Vec<String> {
    let Some(max_lines) = max_lines else {
        return _clone_lines(lines);
    };
    let limit: i64 = max_lines;
    if limit <= (0 as i64) {
        return vec![];
    }
    if (lines.len() as i64) <= limit {
        return _clone_lines(lines);
    }
    let mut result: Vec<String> = vec![];
    let mut i: i64 = 0 as i64;
    while i < limit {
        let line_opt: Option<String> = {
            let __sifr_index_list = &lines;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(line_opt) = line_opt {
            result.push(line_opt);
        }
        i = i + (1 as i64);
    }
    if (result.len() as i64) == (0 as i64) {
        return result;
    }
    let mut effective_placeholder: String = placeholder.to_string();
    if width > (0 as i64) {
        if (effective_placeholder.chars().count() as i64) > width {
            effective_placeholder = String::from_iter(
                (effective_placeholder)
                    .chars()
                    .skip((0 as i64).max(0) as usize)
                    .take(((width).max(0) - (0 as i64).max(0)).max(0) as usize),
            );
        }
    }
    let last_index: i64 = (result.len() as i64) - (1 as i64);
    let last_opt: Option<String> = Some(result[last_index as usize].clone());
    if let Some(last_opt) = last_opt {
        let last: String = last_opt;
        let mut base: String = _trim_line(&last);
        let mut available: i64 = width - (effective_placeholder.chars().count() as i64);
        if available < (0 as i64) {
            available = 0 as i64;
        }
        if (base.chars().count() as i64) > available {
            base = _trim_line(
                &base
                    .chars()
                    .skip((0 as i64) as usize)
                    .take((available as usize) - ((0 as i64) as usize))
                    .collect::<String>(),
            );
        }
        if drop_whitespace {
            base = _trim_line(&base);
        }
        {
            let __idx_raw = last_index;
            let __idx_norm = if __idx_raw < 0 {
                (result.len() as i64) + __idx_raw
            } else {
                __idx_raw
            };
            if __idx_norm >= 0 {
                if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                    *__elem = format!("{}{}", base, effective_placeholder);
                }
            }
        }
    }
    return result;
}

// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while pi < (pattern.chars().count() as i64) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(pi as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(pc) = pc {
            if pc == "*".to_string() {
                pi = pi + (1 as i64);
                if pi == (pattern.len() as i64) {
                    return true;
                }
                let mut j: i64 = ni;
                while j <= (name.chars().count() as i64) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j = j + (1 as i64);
                }
                return false;
            } else {
                if pc == "?".to_string() {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                } else {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name.chars().nth(ni as usize) else {
                            unreachable!("compiler-verified string index should be in range");
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                }
            }
        } else {
            return false;
        }
    }
    return ni == (name.chars().count() as i64);
}
fn _translate_literal(ch: &String) -> String {
    if ch.clone() == ".".to_string() {
        return "\\.".to_string();
    }
    if ch.clone() == "^".to_string() {
        return "\\^".to_string();
    }
    if ch.clone() == "$".to_string() {
        return "\\$".to_string();
    }
    if ch.clone() == "+".to_string() {
        return "\\+".to_string();
    }
    if ch.clone() == "(".to_string() {
        return "\\(".to_string();
    }
    if ch.clone() == ")".to_string() {
        return "\\)".to_string();
    }
    if ch.clone() == "{".to_string() {
        return "\\{".to_string();
    }
    if ch.clone() == "}".to_string() {
        return "\\}".to_string();
    }
    if ch.clone() == "[".to_string() {
        return "\\[".to_string();
    }
    if ch.clone() == "]".to_string() {
        return "\\]".to_string();
    }
    if ch.clone() == "|".to_string() {
        return "\\|".to_string();
    }
    if ch.clone() == "\\".to_string() {
        return "\\\\".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn translate(pattern: &String) -> String {
    let mut body: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (pattern.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "*".to_string() {
                body = format!("{}{}", body, ".*".to_string());
            } else {
                if ch == "?".to_string() {
                    body = format!("{}{}", body, ".".to_string());
                } else {
                    body = format!("{}{}", body, _translate_literal(&ch));
                }
            }
        }
        i = i + (1 as i64);
    }
    return format!("{}{}{}", "(?s:".to_string(), body, ")\\z".to_string());
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

// --- stdlib: sifr.base64 ---
fn b64encode(s: &String) -> String {
    return base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &s.as_bytes());
}
fn b64decode(s: &String) -> Result<String, ParseError> {
    return {
        let __bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &s.as_bytes())
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
        String::from_utf8(__bytes).map_err(|e| ParseError {
            message: e.to_string(),
        })
    };
}

// --- stdlib: sifr.difflib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SequenceMatcher {
    _a: String,
    _b: String,
}
impl SequenceMatcher {
    fn new(a: String, b: String) -> Self {
        return Self {
            _a: format!("{}{}", a, "".to_string()),
            _b: format!("{}{}", b, "".to_string()),
        };
    }
    fn set_seq1(&mut self, a: &String) {
        self._a = format!("{}{}", a, "".to_string());
    }
    fn set_seq2(&mut self, b: &String) {
        self._b = format!("{}{}", b, "".to_string());
    }
    fn set_seqs(&mut self, a: &String, b: &String) {
        self._a = format!("{}{}", a, "".to_string());
        self._b = format!("{}{}", b, "".to_string());
    }
    fn ratio(&self) -> f64 {
        return _similarity(&self._a.clone(), &self._b.clone());
    }
    fn get_matching_blocks(&self) -> Vec<(i64, i64, i64)> {
        return _matching_blocks(&self._a.clone(), &self._b.clone());
    }
}
impl std::fmt::Display for SequenceMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "SequenceMatcher(_a={}, _b={})", self._a, self._b);
    }
}
fn _similarity(a: &String, b: &String) -> f64 {
    let total: i64 = (a.chars().count() as i64) + (b.chars().count() as i64);
    if total == (0 as i64) {
        return 1.0 as f64;
    }
    let mut matches: i64 = 0 as i64;
    let blocks: Vec<(i64, i64, i64)> = _matching_blocks(a, b);
    for block in blocks.iter().copied() {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1, __sifr_tuple_unpack_2) = block;
        let _ = __sifr_tuple_unpack_0;
        _ = __sifr_tuple_unpack_1;
        let block_size = __sifr_tuple_unpack_2;
        matches = matches + block_size;
    }
    return (((2 as i64) * matches) as f64) / (total as f64);
}
fn _longest_common_substring_range(
    a: &String,
    b: &String,
    a_start: i64,
    a_end: i64,
    b_start: i64,
    b_end: i64,
) -> (i64, i64, i64) {
    let mut best_i: i64 = 0 as i64;
    let mut best_j: i64 = 0 as i64;
    let mut best_len: i64 = 0 as i64;
    let mut i: i64 = a_start;
    while i < a_end {
        let mut j: i64 = b_start;
        while j < b_end {
            let mut k: i64 = 0 as i64;
            while ((i + k) < a_end) && ((j + k) < b_end) {
                let ai: Option<String> = {
                    let __sifr_index_str = &a;
                    let __sifr_index_i = i + k;
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
                let bj: Option<String> = {
                    let __sifr_index_str = &b;
                    let __sifr_index_i = j + k;
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
                if ai.is_none() || bj.is_none() {
                    k = k + (1 as i64);
                    continue;
                }
                if ai != bj {
                    break;
                }
                k = k + (1 as i64);
            }
            if k > best_len {
                best_len = k;
                best_i = i;
                best_j = j;
            }
            j = j + (1 as i64);
        }
        i = i + (1 as i64);
    }
    return (best_i, best_j, best_len);
}
fn _sort_blocks(blocks: &Vec<(i64, i64, i64)>) -> Vec<(i64, i64, i64)> {
    let mut sorted_blocks: Vec<(i64, i64, i64)> = vec![];
    for block in blocks.iter().copied() {
        let (bl_a, bl_b, _) = block;
        let mut found_insert_at: bool = false;
        let mut insert_at: i64 = 0 as i64;
        let mut i: i64 = 0 as i64;
        for existing in sorted_blocks.iter().copied() {
            if !found_insert_at {
                let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1, __sifr_tuple_unpack_2) =
                    existing;
                let ex_a = __sifr_tuple_unpack_0;
                let ex_b = __sifr_tuple_unpack_1;
                _ = __sifr_tuple_unpack_2;
                if (bl_a < ex_a) || ((bl_a == ex_a) && (bl_b < ex_b)) {
                    insert_at = i;
                    found_insert_at = true;
                }
            }
            i = i + (1 as i64);
        }
        if found_insert_at {
            sorted_blocks.insert(insert_at as usize, block);
        } else {
            sorted_blocks.push(block);
        }
    }
    return sorted_blocks;
}
fn _matching_blocks(a: &String, b: &String) -> Vec<(i64, i64, i64)> {
    let mut pending_a_start: Vec<i64> = vec![0 as i64];
    let mut pending_a_end: Vec<i64> = vec![a.chars().count() as i64];
    let mut pending_b_start: Vec<i64> = vec![0 as i64];
    let mut pending_b_end: Vec<i64> = vec![b.chars().count() as i64];
    let mut unsorted_blocks: Vec<(i64, i64, i64)> = vec![];
    while (pending_a_start.len() as i64) > (0 as i64) {
        let a_start_value: Option<i64> = Some({
            let Some(__sifr_nonempty_pop_value) = pending_a_start.pop() else {
                unreachable!("compiler-verified non-empty pop should return Some");
            };
            __sifr_nonempty_pop_value
        });
        let a_end_value: Option<i64> = pending_a_end.pop();
        let b_start_value: Option<i64> = pending_b_start.pop();
        let b_end_value: Option<i64> = pending_b_end.pop();
        if let Some(a_start_value) = a_start_value {
            if let Some(a_end_value) = a_end_value {
                if let Some(b_start_value) = b_start_value {
                    if let Some(b_end_value) = b_end_value {
                        let (ai, bj, size) = _longest_common_substring_range(
                            a,
                            b,
                            a_start_value,
                            a_end_value,
                            b_start_value,
                            b_end_value,
                        );
                        if size == (0 as i64) {
                            continue;
                        }
                        unsorted_blocks.push((ai, bj, size));
                        let left_a_end: i64 = ai;
                        let left_b_end: i64 = bj;
                        if (a_start_value < left_a_end) && (b_start_value < left_b_end) {
                            pending_a_start.push(a_start_value);
                            pending_a_end.push(left_a_end);
                            pending_b_start.push(b_start_value);
                            pending_b_end.push(left_b_end);
                        }
                        let right_a_start: i64 = ai + size;
                        let right_b_start: i64 = bj + size;
                        if (right_a_start < a_end_value) && (right_b_start < b_end_value) {
                            pending_a_start.push(right_a_start);
                            pending_a_end.push(a_end_value);
                            pending_b_start.push(right_b_start);
                            pending_b_end.push(b_end_value);
                        }
                    }
                }
            }
        }
    }
    let sorted_blocks: Vec<(i64, i64, i64)> = _sort_blocks(&unsorted_blocks);
    let mut merged_blocks: Vec<(i64, i64, i64)> = vec![];
    let mut have_previous: bool = false;
    let mut prev_a: i64 = 0 as i64;
    let mut prev_b: i64 = 0 as i64;
    let mut prev_size: i64 = 0 as i64;
    for block in sorted_blocks.iter().copied() {
        let (bl_a, bl_b, bl_size) = block;
        if !have_previous {
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
            have_previous = true;
            continue;
        }
        if ((prev_a + prev_size) == bl_a) && ((prev_b + prev_size) == bl_b) {
            prev_size = prev_size + bl_size;
        } else {
            merged_blocks.push((prev_a, prev_b, prev_size));
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
        }
    }
    if have_previous {
        merged_blocks.push((prev_a, prev_b, prev_size));
    }
    merged_blocks.push((a.chars().count() as i64, b.chars().count() as i64, 0 as i64));
    return merged_blocks;
}

// --- stdlib: sifr.string ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Template {
    template: String,
}
impl Template {
    fn new(template: String) -> Self {
        return Self {
            template: format!("{}{}", template, "".to_string()),
        };
    }
    fn substitute(&self, mapping: &HashMap<String, String>) -> Result<String, ValueError> {
        return _template_substitute_impl(&self.template.clone(), mapping, false);
    }
    fn safe_substitute(&self, mapping: &HashMap<String, String>) -> String {
        let __sifr_try_res: Result<String, ValueError> = (|| {
            let value: String = _template_substitute_impl(&self.template.clone(), mapping, true)?;
            return Ok(value);
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
                return format!("{}{}", self.template.clone(), "".to_string());
            }
        }
    }
}
impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Template(template={})", self.template);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Formatter {}
impl Formatter {
    fn new() -> Self {
        return Self {};
    }
    fn format(
        &self,
        format_string: &String,
        values: &HashMap<String, String>,
    ) -> Result<String, ValueError> {
        return _formatter_format_impl(format_string, values);
    }
}
fn _is_identifier_start(ch: &String) -> bool {
    return ((ch.clone() == "_".to_string())
        || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())));
}
fn _is_identifier_continue(ch: &String) -> bool {
    return (((ch.clone() == "_".to_string())
        || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())))
        || (!ch.is_empty() && ch.chars().all(|c| c.is_ascii_digit())));
}
fn _mapping_lookup(mapping: &HashMap<String, String>, key: &String) -> Option<String> {
    for (current_key, current_value) in mapping
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if current_key == *key {
            return Some(format!("{}{}", current_value, "".to_string()));
        }
    }
    return None;
}
fn _template_substitute_impl(
    template: &String,
    mapping: &HashMap<String, String>,
    safe: bool,
) -> Result<String, ValueError> {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (template.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = template.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if ch.is_none() {
            i = i + (1 as i64);
            continue;
        }
        let mut ch_value: String = "".to_string();
        if let Some(ch) = ch {
            ch_value = ch;
        }
        if ch_value != "$".to_string() {
            result = format!("{}{}", result, ch_value);
            i = i + (1 as i64);
            continue;
        }
        if (i + (1 as i64)) >= (template.chars().count() as i64) {
            if safe {
                result = format!("{}{}", result, "$".to_string());
                i = i + (1 as i64);
                continue;
            }
            return Err(ValueError::new(
                "invalid template placeholder at end of string".to_string(),
            ));
        }
        let next_ch: Option<String> = {
            let __sifr_index_str = &template;
            let __sifr_index_i = i + (1 as i64);
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
        let mut next_value: String = "".to_string();
        if next_ch.is_none() {
            if safe {
                result = format!("{}{}", result, "$".to_string());
                i = i + (1 as i64);
                continue;
            }
            return Err(ValueError::new("invalid template placeholder".to_string()));
        } else {
            if let Some(next_ch) = next_ch {
                next_value = next_ch;
            }
        }
        if next_value == "$".to_string() {
            result = format!("{}{}", result, "$".to_string());
            i = i + (2 as i64);
            continue;
        }
        if next_value == "{".to_string() {
            let mut j: i64 = i + (2 as i64);
            let mut name: String = "".to_string();
            while j < (template.chars().count() as i64) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = template.chars().nth(j as usize) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char.to_string()
                });
                if part.is_none() {
                    j = j + (1 as i64);
                    continue;
                }
                let mut part_value: String = "".to_string();
                if let Some(part) = part {
                    part_value = part;
                }
                if part_value == "}".to_string() {
                    break;
                }
                name = format!("{}{}", name, part_value);
                j = j + (1 as i64);
            }
            if j >= (template.chars().count() as i64) {
                if safe {
                    result = format!(
                        "{}{}",
                        result,
                        String::from_iter((template).chars().skip((i).max(0) as usize))
                    );
                    return Ok(result);
                }
                return Err(ValueError::new(
                    "invalid template placeholder: missing closing brace".to_string(),
                ));
            }
            if (name.chars().count() as i64) == (0 as i64) {
                if safe {
                    result = format!("{}{}", result, "${}".to_string());
                    i = j + (1 as i64);
                    continue;
                }
                return Err(ValueError::new(
                    "invalid template placeholder: empty name".to_string(),
                ));
            }
            let first_candidate: Option<String> = Some({
                let Some(__indexed_char) = name.chars().nth((0 as i64) as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            });
            let mut first_value: String = "".to_string();
            let mut has_first: bool = false;
            if let Some(first_candidate) = first_candidate {
                has_first = true;
                first_value = first_candidate;
            }
            if ((!(has_first)) || (!(_is_identifier_start(&first_value)))) {
                if safe {
                    result = format!("{}{}{}{}", result, "${".to_string(), name, "}".to_string());
                    i = j + (1 as i64);
                    continue;
                }
                return Err(ValueError::new(format!(
                    "{}{}",
                    "invalid template placeholder: ".to_string(),
                    name
                )));
            }
            let mut valid: bool = true;
            let mut k: i64 = 1 as i64;
            while k < (name.chars().count() as i64) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = name.chars().nth(k as usize) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char.to_string()
                });
                if let Some(part) = part {
                    if !(_is_identifier_continue(&part)) {
                        valid = false;
                        k = name.chars().count() as i64;
                    }
                }
                k = k + (1 as i64);
            }
            if !valid {
                if safe {
                    result = format!("{}{}{}{}", result, "${".to_string(), name, "}".to_string());
                    i = j + (1 as i64);
                    continue;
                }
                return Err(ValueError::new(format!(
                    "{}{}",
                    "invalid template placeholder: ".to_string(),
                    name
                )));
            }
            let mapped_value: Option<String> = _mapping_lookup(mapping, &name);
            let mut mapped_value_text: String = "".to_string();
            if mapped_value.is_none() {
                if safe {
                    result = format!("{}{}{}{}", result, "${".to_string(), name, "}".to_string());
                    i = j + (1 as i64);
                    continue;
                }
                return Err(ValueError::new(format!(
                    "{}{}",
                    "missing template value for key: ".to_string(),
                    name
                )));
            } else {
                if let Some(mapped_value) = mapped_value {
                    mapped_value_text = mapped_value;
                }
            }
            result = format!("{}{}", result, mapped_value_text);
            i = j + (1 as i64);
            continue;
        }
        if !(_is_identifier_start(&next_value)) {
            if safe {
                result = format!("{}{}{}", result, "$".to_string(), next_value);
                i = i + (2 as i64);
                continue;
            }
            return Err(ValueError::new(format!(
                "{}{}",
                "invalid template placeholder near: $".to_string(),
                next_value
            )));
        }
        let mut name2: String = "".to_string();
        let mut j2: i64 = i + (1 as i64);
        while j2 < (template.chars().count() as i64) {
            let part2: Option<String> = Some({
                let Some(__indexed_char) = template.chars().nth(j2 as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            });
            if part2.is_none() {
                j2 = j2 + (1 as i64);
                continue;
            }
            let mut part2_value: String = "".to_string();
            if let Some(part2) = part2 {
                part2_value = part2;
            }
            if !(_is_identifier_continue(&part2_value)) {
                break;
            }
            name2 = format!("{}{}", name2, part2_value);
            j2 = j2 + (1 as i64);
        }
        let mapped_value2: Option<String> = _mapping_lookup(mapping, &name2);
        let mut mapped_value2_text: String = "".to_string();
        if mapped_value2.is_none() {
            if safe {
                result = format!("{}{}{}", result, "$".to_string(), name2);
                i = j2;
                continue;
            }
            return Err(ValueError::new(format!(
                "{}{}",
                "missing template value for key: ".to_string(),
                name2
            )));
        } else {
            if let Some(mapped_value2) = mapped_value2 {
                mapped_value2_text = mapped_value2;
            }
        }
        result = format!("{}{}", result, mapped_value2_text);
        i = j2;
    }
    return Ok(result);
}
fn _formatter_format_impl(
    format_string: &String,
    values: &HashMap<String, String>,
) -> Result<String, ValueError> {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (format_string.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = format_string.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if ch.is_none() {
            i = i + (1 as i64);
            continue;
        }
        let mut ch_value: String = "".to_string();
        if let Some(ch) = ch {
            ch_value = ch;
        }
        if ch_value == "{".to_string() {
            if (i + (1 as i64)) < (format_string.chars().count() as i64) {
                let escaped_next: Option<String> = {
                    let __sifr_index_str = &format_string;
                    let __sifr_index_i = i + (1 as i64);
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
                if ((escaped_next != None) && (escaped_next == Some("{".to_string()))) {
                    result = format!("{}{}", result, "{".to_string());
                    i = i + (2 as i64);
                    continue;
                }
            }
            let mut j: i64 = i + (1 as i64);
            let mut field_name: String = "".to_string();
            while j < (format_string.chars().count() as i64) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = format_string.chars().nth(j as usize) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char.to_string()
                });
                if part.is_none() {
                    j = j + (1 as i64);
                    continue;
                }
                let mut part_value: String = "".to_string();
                if let Some(part) = part {
                    part_value = part;
                }
                if part_value == "}".to_string() {
                    break;
                }
                field_name = format!("{}{}", field_name, part_value);
                j = j + (1 as i64);
            }
            if j >= (format_string.chars().count() as i64) {
                return Err(ValueError::new(
                    "formatter: missing closing brace".to_string(),
                ));
            }
            if (field_name.chars().count() as i64) == (0 as i64) {
                return Err(ValueError::new(
                    "formatter: empty replacement field is not supported".to_string(),
                ));
            }
            let value: Option<String> = _mapping_lookup(values, &field_name);
            let Some(value) = value else {
                return Err(ValueError::new(format!(
                    "{}{}",
                    "formatter: missing value for key: ".to_string(),
                    field_name
                )));
            };
            result = format!("{}{}", result, value);
            i = j + (1 as i64);
            continue;
        }
        if ch_value == "}".to_string() {
            if (i + (1 as i64)) < (format_string.chars().count() as i64) {
                let escaped_next2: Option<String> = {
                    let __sifr_index_str = &format_string;
                    let __sifr_index_i = i + (1 as i64);
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
                if ((escaped_next2 != None) && (escaped_next2 == Some("}".to_string()))) {
                    result = format!("{}{}", result, "}".to_string());
                    i = i + (2 as i64);
                    continue;
                }
            }
            return Err(ValueError::new(
                "formatter: single \'}\' is invalid".to_string(),
            ));
        }
        result = format!("{}{}", result, ch_value);
        i = i + (1 as i64);
    }
    return Ok(result);
}

// --- stdlib: sifr.html ---
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;");
    if quote {
        return escaped;
    }
    return escaped
        .replace(&"&quot;".to_string(), &"\"".to_string())
        .replace(&"&#x27;".to_string(), &"\'".to_string());
}
fn unescape(s: &String) -> String {
    return s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#X27;", "'")
        .replace("&#39;", "'")
        .replace("&#60;", "<")
        .replace("&#x3C;", "<")
        .replace("&#x3c;", "<")
        .replace("&#X3C;", "<")
        .replace("&#X3c;", "<")
        .replace("&#62;", ">")
        .replace("&#x3E;", ">")
        .replace("&#x3e;", ">")
        .replace("&#X3E;", ">")
        .replace("&#X3e;", ">");
}

// --- stdlib: sifr.calendar ---
fn __const_month_name() -> Vec<String> {
    return vec![
        "".to_string(),
        "January".to_string(),
        "February".to_string(),
        "March".to_string(),
        "April".to_string(),
        "May".to_string(),
        "June".to_string(),
        "July".to_string(),
        "August".to_string(),
        "September".to_string(),
        "October".to_string(),
        "November".to_string(),
        "December".to_string(),
    ];
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextCalendar {
    firstweekday: i64,
}
impl TextCalendar {
    fn new(firstweekday: i64) -> Self {
        return Self {
            firstweekday: _normalize_firstweekday(firstweekday),
        };
    }
    fn formatmonthname(&self, year: i64, month: i64, width: i64) -> Result<String, ValueError> {
        let name_lookup: Option<String> = _month_name_lookup(month);
        let mut name: String = "".to_string();
        if let Some(name_lookup) = name_lookup {
            name = name_lookup;
        } else {
            return Err(ValueError::new(
                "calendar: month must be in 1..12".to_string(),
            ));
        }
        let formatted: String = format!("{}{}{}", name, " ".to_string(), format!("{}", year));
        if width <= (0 as i64) {
            return Ok(formatted);
        }
        if (formatted.len() as i64) >= width {
            return Ok(formatted);
        }
        let pad: i64 = width - (formatted.chars().count() as i64);
        let mut left: i64 = pad / (2 as i64);
        let mut right: i64 = pad - left;
        let mut result: String = "".to_string();
        while left > (0 as i64) {
            result = format!("{}{}", result, " ".to_string());
            left = left - (1 as i64);
        }
        result = format!("{}{}", result, formatted);
        while right > (0 as i64) {
            result = format!("{}{}", result, " ".to_string());
            right = right - (1 as i64);
        }
        return Ok(result);
    }
}
impl std::fmt::Display for TextCalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "TextCalendar(firstweekday={})", self.firstweekday);
    }
}
fn _normalize_firstweekday(firstweekday: i64) -> i64 {
    let mut value: i64 = firstweekday % (7 as i64);
    if value < (0 as i64) {
        value = value + (7 as i64);
    }
    return value;
}
fn _month_name_lookup(month: i64) -> Option<String> {
    if (month < (1 as i64)) || (month > (12 as i64)) {
        return None;
    }
    return {
        let __sifr_index_list = &__const_month_name();
        let __sifr_index_i = month;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
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

fn main() {
    let mut template: Template = Template::new("Hello $name, mode=${mode}".to_string());
    let mut rendered_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let rendered: String = template.substitute(&HashMap::from([
            ("name".to_string(), "Sifr".to_string()),
            ("mode".to_string(), "c2".to_string()),
        ]))?;
        rendered_ok = rendered == "Hello Sifr, mode=c2".to_string();
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(rendered_ok);
    let mut formatter: Formatter = Formatter::new();
    let mut rendered_fmt_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let rendered_fmt: String = formatter.format(
            &"Status {label}: {status}".to_string(),
            &HashMap::from([
                ("label".to_string(), "c2".to_string()),
                ("status".to_string(), "ok".to_string()),
            ]),
        )?;
        rendered_fmt_ok = rendered_fmt == "Status c2: ok".to_string();
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(rendered_fmt_ok);
    let mut wrapper: TextWrapper = TextWrapper::new(
        8 as i64,
        "> ".to_string(),
        ".. ".to_string(),
        true,
        8 as i64,
        true,
        true,
        true,
        false,
        None,
        " [...]".to_string(),
    );
    let wrapped: Vec<String> = wrapper.wrap(&"alpha beta gamma".to_string());
    assert!(format!("{:?}", wrapped) == "[\"> alpha\", \".. beta\", \".. gamma\"]".to_string());
    let encoded: String = b64encode(&"hello".to_string());
    let mut decoded_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(&encoded)?;
        decoded_ok = decoded == "hello".to_string();
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(decoded_ok);
    let escaped: String = escape(&"<b>safe</b>".to_string(), true);
    assert!(unescape(&escaped) == "<b>safe</b>".to_string());
    assert!(fnmatch(&"report.txt".to_string(), &"*.txt".to_string()));
    assert!(translate(&"*.txt".to_string()) == "(?s:.*\\.txt)\\z".to_string());
    let ratio: f64 = SequenceMatcher::new("abcd".to_string(), "abed".to_string()).ratio();
    assert!(ratio > (0.4 as f64));
    let mut month_label_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let month_label: String =
            TextCalendar::new(0 as i64).formatmonthname(2024 as i64, 2 as i64, 0 as i64)?;
        month_label_ok = month_label == "February 2024".to_string();
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(month_label_ok);
}

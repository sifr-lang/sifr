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
fn filter(names: &[String], pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name);
        }
    }
    return result;
}

// --- stdlib: sifr.string ---
fn __const_ascii_lowercase() -> String {
    return "abcdefghijklmnopqrstuvwxyz".to_string();
}
fn capwords(s: &String) -> String {
    let normalized: String = s
        .replace(&"\t".to_string(), &" ".to_string())
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    let words: Vec<String> = normalized
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        if (word.chars().count() as i64) > (0 as i64) {
            if !first {
                result = format!("{}{}", result, " ".to_string());
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result = format!("{}{}", result, cap);
        }
    }
    return result;
}

// --- stdlib: sifr.re ---
fn sub(pattern: &String, replacement: &String, text: &String) -> Result<String, RegexError> {
    return regex::Regex::new(&pattern)
        .map(|re| re.replace_all(&text, &*replacement).to_string())
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
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
fn split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    return regex::Regex::new(&pattern)
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

// --- stdlib: sifr.pathlib ---
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

// --- stdlib: sifr.textwrap ---
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
fn _normalize_whitespace(text: &String) -> String {
    return _prepare_text(text, true, 8 as i64, true);
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
fn _wrap_impl(text: &String, width: i64) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    return _wrap_with_indents(
        &normalized,
        width,
        &"".to_string(),
        &"".to_string(),
        true,
        true,
    );
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
fn wrap(text: &String, width: i64) -> Result<Vec<String>, ValueError> {
    if width <= (0 as i64) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    return Ok(_wrap_impl(text, width));
}
fn fill(text: &String, width: i64) -> Result<String, ValueError> {
    if width <= (0 as i64) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = _wrap_impl(text, width);
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    for line in lines.iter().cloned() {
        if i > (0 as i64) {
            result = format!("{}{}", result, "\n".to_string());
        }
        result = format!("{}{}", result, line);
        i = i + (1 as i64);
    }
    return Ok(result);
}

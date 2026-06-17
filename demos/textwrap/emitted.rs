// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
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
fn _has_non_whitespace(text: &String) -> bool {
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch != " ".to_string() {
                if ch != "\t".to_string() {
                    if ch != "\n".to_string() {
                        if ch != "\r".to_string() {
                            if ch != "\u{b}".to_string() {
                                if ch != "\u{c}".to_string() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i = i + (1 as i64);
    }
    return false;
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
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
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
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
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
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
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
                if (((current.chars().count() as i64) + (1 as i64))
                    + (word.chars().count() as i64)) <= current_limit
                {
                    current = format!("{}{}{}", current, " ".to_string(), word);
                } else {
                    if first_line {
                        _push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = _effective_content_width(
                            total_width,
                            subsequent_indent,
                        );
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
            _push_current_line(
                &mut result,
                &current,
                subsequent_indent,
                drop_whitespace,
            );
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
fn dedent(text: &String) -> String {
    let lines: Vec<String> = text
        .split(&"\n".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut min_indent: i64 = 0 as i64;
    let mut have_indent: bool = false;
    for line in lines.iter().cloned() {
        if _has_non_whitespace(&line) {
            let mut spaces: i64 = 0 as i64;
            let mut j: i64 = 0 as i64;
            let mut done: bool = false;
            while j < (line.chars().count() as i64) {
                if !done {
                    let ch: Option<String> = Some({
                        let Some(__indexed_char) = line.chars().nth(j as usize) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(ch) = ch {
                        if ch == " ".to_string() {
                            spaces = spaces + (1 as i64);
                        } else {
                            done = true;
                        }
                    }
                }
                j = j + (1 as i64);
            }
            if !have_indent {
                min_indent = spaces;
                have_indent = true;
            } else {
                if spaces < min_indent {
                    min_indent = spaces;
                }
            }
        }
    }
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for line2 in lines.iter().cloned() {
        if !first {
            result = format!("{}{}", result, "\n".to_string());
        }
        first = false;
        if have_indent {
            if (line2.chars().count() as i64) > min_indent {
                result = format!(
                    "{}{}", result, String::from_iter((line2).chars().skip((min_indent)
                    .max(0) as usize))
                );
            } else {
                result = format!("{}{}", result, line2);
            }
        } else {
            result = format!(
                "{}{}", result, String::from_iter((line2).chars().skip((min_indent)
                .max(0) as usize))
            );
        }
    }
    return result;
}
fn indent(text: &String, prefix: &String) -> String {
    let lines: Vec<String> = text
        .split(&"\n".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for line in lines.iter().cloned() {
        if !first {
            result = format!("{}{}", result, "\n".to_string());
        }
        first = false;
        if _has_non_whitespace(&line) {
            result = format!("{}{}{}", result, prefix, line);
        } else {
            result = format!("{}{}", result, line);
        }
    }
    return result;
}
fn shorten(text: &String, width: i64) -> String {
    let normalized: String = _normalize_whitespace(text);
    let words: Vec<String> = normalized
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    for word in words.iter().cloned() {
        if (word.chars().count() as i64) == (0 as i64) {} else {
            if (result.chars().count() as i64) == (0 as i64) {
                result = word;
            } else {
                if ((((result.chars().count() as i64) + (1 as i64))
                    + (word.chars().count() as i64)) + (4 as i64)) <= width
                {
                    result = format!("{}{}{}", result, " ".to_string(), word);
                } else {
                    return format!("{}{}", result, " [...]".to_string());
                }
            }
        }
    }
    return result;
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

fn collect_wrap_fill_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let lines: Vec<String> = wrap(&"alpha\tbeta\ngamma".to_string(), 10 as i64)?;
    actual.push((format!("{:?}", lines)).as_str() == ("[\"alpha beta\", \"gamma\"]".to_string()).as_str());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let filled: String = fill(&"alpha\tbeta\ngamma".to_string(), 10 as i64)?;
    actual.push(filled == "alpha beta\ngamma".to_string());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    return actual;
}

fn collect_other_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((dedent(&"  x\n  y".to_string())).as_str() == ("x\ny".to_string()).as_str());
    actual.push((indent(&"x\n \ny".to_string(), &">> ".to_string())).as_str() == (">> x\n \n>> y".to_string()).as_str());
    actual.push((shorten(&"alpha beta gamma".to_string(), 16 as i64)).as_str() == ("alpha beta [...]".to_string()).as_str());
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let wrap_empty: Vec<String> = wrap(&"".to_string(), 5 as i64)?;
    actual.push((format!("{:?}", wrap_empty)).as_str() == ("[]".to_string()).as_str());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_wrap_fill_actual());
    append_all(&mut actual, &collect_other_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("textwrap textwrap parity demo: pass");
}

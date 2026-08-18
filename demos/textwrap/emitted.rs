// src/main.rs
// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}

// --- stdlib: sifr.textwrap ---
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
    let normalized: String = text
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('\u{b}', " ")
        .replace('\u{c}', " ");
    if replace_tabs {
        return normalized.replace('\t', " ");
    }
    normalized
}
fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: i64 = tabsize;
    if effective_tabsize <= (0_i64) {
        effective_tabsize = 1_i64;
    }
    let mut result: String = "".to_string();
    let mut column: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t" {
                let mut spaces: i64 = effective_tabsize - (column % effective_tabsize);
                if spaces <= (0_i64) {
                    spaces = effective_tabsize;
                }
                let mut j: i64 = 0_i64;
                while j < spaces {
                    result.push(' ');
                    j += 1_i64;
                }
                column += spaces;
            } else {
                if (ch == "\n") || (ch == "\r") {
                    result.push_str((ch).as_str());
                    column = 0_i64;
                } else {
                    result.push_str((ch).as_str());
                    column += 1_i64;
                }
            }
        }
        i += 1_i64;
    }
    result
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str((text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize);
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn _normalize_whitespace(text: &String) -> String {
    _prepare_text(text, true, 8_i64, true)
}
fn _has_non_whitespace(text: &String) -> bool {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch != " " {
                if ch != "\t" {
                    if ch != "\n" {
                        if ch != "\r" {
                            if ch != "\u{b}" {
                                if ch != "\u{c}" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i += 1_i64;
    }
    false
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if ((parts.len() as i64) <= (1_i64)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let mut units: Vec<String> = vec![];
    let mut index: i64 = 0_i64;
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = (index == ((parts.len() as i64) - (1_i64)));
        if is_last {
            if ((__sifr_chars_part.len() as i64) > (0_i64)) {
                units.push(part.clone());
            }
        } else {
            if ((__sifr_chars_part.len() as i64) == (0_i64)) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-"));
            }
        }
        index += 1_i64;
    }
    if ((units.len() as i64) == (0_i64)) {
        units.push(format!("{}{}", word, ""));
    }
    units
}
fn _trim_line(line: &String) -> String {
    let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: i64 = 0_i64;
    while (start < (__sifr_chars_line.len() as i64))
        && (({
            let Some(__indexed_char) = __sifr_chars_line
                .get(start as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) == " ")
    {
        start += 1_i64;
    }
    let mut end: i64 = __sifr_chars_line.len() as i64;
    while (end > start)
        && (__sifr_chars_line.get((end - (1_i64)) as usize).map(|c| c.to_string())
            == Some(" ".to_string()))
    {
        end -= 1_i64;
    }
    {
        let _slice_src = &__sifr_chars_line;
        let _slice_len_i64 = _slice_src.len() as i64;
        let _slice_start_i64 = if start < 0 {
            (_slice_len_i64 + start).max(0)
        } else {
            start.min(_slice_len_i64)
        };
        let _slice_stop_i64 = if end < 0 {
            (_slice_len_i64 + end).max(0)
        } else {
            end.min(_slice_len_i64)
        };
        String::from_iter(
            _slice_src
                .iter()
                .skip(_slice_start_i64 as usize)
                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                .copied(),
        )
    }
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(line.len() + 0usize);
        __sifr_concat.push_str((line).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _wrap_impl(text: &String, width: i64) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    _wrap_with_indents(&normalized, width, &"".to_string(), &"".to_string(), true, true)
}
fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: i64 = total_width - (__sifr_chars_indent.len() as i64);
    if available <= (0_i64) {
        return 1_i64;
    }
    available
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
    let __sifr_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if ((__sifr_chars_candidate.len() as i64) > (0_i64)) {
            result.push(candidate.clone());
        }
    } else {
        result.push(candidate.clone());
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
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: i64 = _effective_content_width(total_width, initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if ((__sifr_chars_word.len() as i64) == (0_i64)) {
                if drop_whitespace {
                    continue;
                }
                if ((current.chars().count() as i64) > (0_i64)) {
                    if (((current.chars().count() as i64) + (1_i64)) <= current_limit) {
                        current.push(' ');
                    }
                }
                continue;
            }
            if ((current.chars().count() as i64) == (0_i64)) {
                current = word;
            } else {
                if ((((current.chars().count() as i64) + (1_i64))
                    + (__sifr_chars_word.len() as i64)) <= current_limit)
                {
                    current.push(' ');
                    current.push_str((word).as_str());
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
    if ((current.chars().count() as i64) > (0_i64)) {
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
    result
}
fn wrap(text: &String, width: i64) -> Result<Vec<String>, ValueError> {
    if width <= (0_i64) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    Ok(_wrap_impl(text, width))
}
fn fill(text: &String, width: i64) -> Result<String, ValueError> {
    if width <= (0_i64) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = _wrap_impl(text, width);
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    for line in lines.iter().cloned() {
        if i > (0_i64) {
            result.push('\n');
        }
        result.push_str((line).as_str());
        i += 1_i64;
    }
    Ok(result)
}
fn dedent(text: &String) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut min_indent: i64 = 0_i64;
    let mut have_indent: bool = false;
    for line in lines.iter().cloned() {
        let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
        if _has_non_whitespace(&line) {
            let mut spaces: i64 = 0_i64;
            let mut j: i64 = 0_i64;
            let mut done: bool = false;
            while (j < (__sifr_chars_line.len() as i64)) {
                if !done {
                    let ch: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_line
                            .get(j as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(ch) = ch {
                        if ch == " " {
                            spaces += 1_i64;
                        } else {
                            done = true;
                        }
                    }
                }
                j += 1_i64;
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
        let __sifr_chars_line2: Vec<char> = line2.chars().collect::<Vec<char>>();
        if !first {
            result.push('\n');
        }
        first = false;
        if have_indent {
            if ((__sifr_chars_line2.len() as i64) > min_indent) {
                result
                    .push_str(
                        ({
                            let _slice_src = &__sifr_chars_line2;
                            let _slice_len_i64 = _slice_src.len() as i64;
                            let _slice_start_i64 = if min_indent < 0 {
                                (_slice_len_i64 + min_indent).max(0)
                            } else {
                                min_indent.min(_slice_len_i64)
                            };
                            let _slice_stop_i64 = _slice_len_i64;
                            String::from_iter(
                                _slice_src
                                    .iter()
                                    .skip(_slice_start_i64 as usize)
                                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                    .copied(),
                            )
                        })
                            .as_str(),
                    );
            } else {
                result.push_str((line2).as_str());
            }
        } else {
            result
                .push_str(
                    ({
                        let _slice_src = &__sifr_chars_line2;
                        let _slice_len_i64 = _slice_src.len() as i64;
                        let _slice_start_i64 = if min_indent < 0 {
                            (_slice_len_i64 + min_indent).max(0)
                        } else {
                            min_indent.min(_slice_len_i64)
                        };
                        let _slice_stop_i64 = _slice_len_i64;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start_i64 as usize)
                                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                .copied(),
                        )
                    })
                        .as_str(),
                );
        }
    }
    result
}
fn indent(text: &String, prefix: &String) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for line in lines.iter().cloned() {
        if !first {
            result.push('\n');
        }
        first = false;
        if _has_non_whitespace(&line) {
            result.push_str((prefix).as_str());
            result.push_str((line).as_str());
        } else {
            result.push_str((line).as_str());
        }
    }
    result
}
fn shorten(text: &String, width: i64) -> String {
    let normalized: String = _normalize_whitespace(text);
    let words: Vec<String> = normalized
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    for word in words.iter().cloned() {
        let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if ((__sifr_chars_word.len() as i64) == (0_i64)) {} else {
            if ((result.chars().count() as i64) == (0_i64)) {
                result = word;
            } else {
                if (((((result.chars().count() as i64) + (1_i64))
                    + (__sifr_chars_word.len() as i64)) + (4_i64)) <= width)
                {
                    result.push(' ');
                    result.push_str((word).as_str());
                } else {
                    return {
                        let mut __sifr_concat: String = String::with_capacity(
                            result.len() + 6usize,
                        );
                        __sifr_concat.push_str((result).as_str());
                        __sifr_concat.push_str(" [...]");
                        __sifr_concat
                    };
                }
            }
        }
    }
    result
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

fn collect_wrap_fill_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let lines: Vec<String> = wrap(&"alpha\tbeta\ngamma".to_string(), 10_i64)?;
    actual.push((format!("{:?}", lines)).as_str() == ("[\"alpha beta\", \"gamma\"]".to_string()).as_str());
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let filled: String = fill(&"alpha\tbeta\ngamma".to_string(), 10_i64)?;
    actual.push(filled == "alpha beta\ngamma");
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    actual
}

fn collect_other_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((dedent(&"  x\n  y".to_string())).as_str() == ("x\ny".to_string()).as_str());
    actual.push((indent(&"x\n \ny".to_string(), &">> ".to_string())).as_str() == (">> x\n \n>> y".to_string()).as_str());
    actual.push((shorten(&"alpha beta gamma".to_string(), 16_i64)).as_str() == ("alpha beta [...]".to_string()).as_str());
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let wrap_empty: Vec<String> = wrap(&"".to_string(), 5_i64)?;
    actual.push((format!("{:?}", wrap_empty)).as_str() == ("[]".to_string()).as_str());
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    actual
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

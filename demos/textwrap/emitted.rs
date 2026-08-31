// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn _replace_whitespace_chars(text: &str, replace_tabs: bool) -> String {
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
fn _expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: SifrInt = tabsize.clone();
    if &effective_tabsize <= &SifrInt::from_i64(0) {
        effective_tabsize = SifrInt::from_i64(1);
    }
    if (&effective_tabsize == &SifrInt::from_i64(0)) {
        return text.to_owned();
    }
    let mut result: String = "".to_string();
    let mut column: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if (ch == "\t") {
                let mut spaces: SifrInt = &effective_tabsize
                    - &column.floor_mod_known_nonzero(&effective_tabsize);
                if (&spaces <= &SifrInt::from_i64(0)) {
                    spaces = effective_tabsize.clone();
                }
                let mut j: SifrInt = SifrInt::from_i64(0);
                while (&j < &spaces) {
                    result.push(' ');
                    j = &j + &SifrInt::from_i64(1);
                }
                column = &column + &spaces;
            } else {
                if (ch == "\n") || (ch == "\r") {
                    result.push_str(ch.as_str());
                    column = SifrInt::from_i64(0);
                } else {
                    result.push_str(ch.as_str());
                    column = &column + &SifrInt::from_i64(1);
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _prepare_text(
    text: &str,
    expand_tabs: bool,
    tabsize: SifrInt,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str(text);
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize.clone());
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn _normalize_whitespace(text: &str) -> String {
    _prepare_text(text, true, SifrInt::from_i64(8), true)
}
fn _has_non_whitespace(text: &str) -> bool {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch != " ") {
                if (ch != "\t") {
                    if (ch != "\n") {
                        if (ch != "\r") {
                            if (ch != "\u{b}") {
                                if (ch != "\u{c}") {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    false
}
fn _split_word_units(word: &str, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str(word); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (&SifrInt::from(parts.len()) <= &SifrInt::from_i64(1)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str(word); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let mut units: Vec<String> = vec![];
    let mut index: SifrInt = SifrInt::from_i64(0);
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = (&index
            == &(&SifrInt::from(parts.len()) - &SifrInt::from_i64(1)));
        if is_last {
            if (&SifrInt::from(__sifr_chars_part.len()) > &SifrInt::from_i64(0)) {
                units.push(part.to_owned());
            }
        } else {
            if (&SifrInt::from(__sifr_chars_part.len()) == &SifrInt::from_i64(0)) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-"));
            }
        }
        index = &index + &SifrInt::from_i64(1);
    }
    if (&SifrInt::from(units.len()) == &SifrInt::from_i64(0)) {
        units.push(format!("{}{}", word, ""));
    }
    units
}
fn _trim_line(line: &str) -> String {
    let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: SifrInt = SifrInt::from_i64(0);
    while (&start < &SifrInt::from(__sifr_chars_line.len()))
        && ({
            let __sifr_string_index = start.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string())
            .is_some_and(|__sifr_checked_value_2| {
                (__sifr_checked_value_2.clone() == " ")
            })
    {
        start = &start + &SifrInt::from_i64(1);
    }
    let mut end: SifrInt = SifrInt::from(__sifr_chars_line.len());
    while (&end > &start)
        && (({
            let __sifr_string_index = &end - &SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) == Some(" ".to_string()))
    {
        end = &end - &SifrInt::from_i64(1);
    }
    {
        let _slice_src = &__sifr_chars_line;
        let _slice_len = _slice_src.len();
        let _slice_start = start.clamp_slice_bound(_slice_len);
        let _slice_stop = end.clamp_slice_bound(_slice_len);
        String::from_iter(
            _slice_src
                .iter()
                .skip(_slice_start)
                .take(_slice_stop.saturating_sub(_slice_start))
                .copied(),
        )
    }
}
fn _finalize_line(line: &str, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(line.len() + 0usize);
        __sifr_concat.push_str(line);
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _wrap_impl(text: &str, width: SifrInt) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    _wrap_with_indents(
        &normalized,
        width.clone(),
        &"".to_string(),
        &"".to_string(),
        true,
        true,
    )
}
fn _effective_content_width(total_width: SifrInt, indent: &str) -> SifrInt {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: SifrInt = &total_width - &SifrInt::from(__sifr_chars_indent.len());
    if &available <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    available.clone()
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &str,
    indent: &str,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    let __sifr_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if (&SifrInt::from(__sifr_chars_candidate.len()) > &SifrInt::from_i64(0)) {
            result.push(candidate.to_owned());
        }
    } else {
        result.push(candidate.to_owned());
    }
}
fn _wrap_with_indents(
    text: &str,
    total_width: SifrInt,
    initial_indent: &str,
    subsequent_indent: &str,
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
    let mut current_limit: SifrInt = _effective_content_width(
        total_width.clone(),
        initial_indent,
    );
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if (&SifrInt::from(__sifr_chars_word.len()) == &SifrInt::from_i64(0)) {
                if drop_whitespace {
                    continue;
                }
                if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
                    if (&(&SifrInt::from(current.chars().count())
                        + &SifrInt::from_i64(1)) <= &current_limit)
                    {
                        current.push(' ');
                    }
                }
                continue;
            }
            if (&SifrInt::from(current.chars().count()) == &SifrInt::from_i64(0)) {
                current = word;
            } else {
                if (&(&(&SifrInt::from(current.chars().count()) + &SifrInt::from_i64(1))
                    + &SifrInt::from(__sifr_chars_word.len())) <= &current_limit)
                {
                    current.push(' ');
                    current.push_str(word.as_str());
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
                            total_width.clone(),
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
    if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
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
fn wrap(text: &str, width: SifrInt) -> Result<Vec<String>, ValueError> {
    if (&width <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    Ok(_wrap_impl(text, width.clone()))
}
fn fill(text: &str, width: SifrInt) -> Result<String, ValueError> {
    if (&width <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = _wrap_impl(text, width.clone());
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    for line in lines.iter().cloned() {
        if (&i > &SifrInt::from_i64(0)) {
            result.push('\n');
        }
        result.push_str(line.as_str());
        i = &i + &SifrInt::from_i64(1);
    }
    Ok(result)
}
fn dedent(text: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut min_indent: SifrInt = SifrInt::from_i64(0);
    let mut have_indent: bool = false;
    for line in lines.iter().cloned() {
        let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
        if _has_non_whitespace(&line) {
            let mut spaces: SifrInt = SifrInt::from_i64(0);
            let mut j: SifrInt = SifrInt::from_i64(0);
            let mut done: bool = false;
            while (&j < &SifrInt::from(__sifr_chars_line.len())) {
                if !done {
                    let ch: Option<String> = ({
                        let __sifr_string_index = j.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_line.len());
                        __sifr_chars_line.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if let Some(ch) = ch {
                        if (ch == " ") {
                            spaces = &spaces + &SifrInt::from_i64(1);
                        } else {
                            done = true;
                        }
                    }
                }
                j = &j + &SifrInt::from_i64(1);
            }
            if !have_indent {
                min_indent = spaces;
                have_indent = true;
            } else {
                if (&spaces < &min_indent) {
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
            if (&SifrInt::from(__sifr_chars_line2.len()) > &min_indent) {
                result
                    .push_str(
                        ({
                            let _slice_src = &__sifr_chars_line2;
                            let _slice_len = _slice_src.len();
                            let _slice_start = min_indent.clamp_slice_bound(_slice_len);
                            let _slice_stop = _slice_len;
                            String::from_iter(
                                _slice_src
                                    .iter()
                                    .skip(_slice_start)
                                    .take(_slice_stop.saturating_sub(_slice_start))
                                    .copied(),
                            )
                        })
                            .as_str(),
                    );
            } else {
                result.push_str(line2.as_str());
            }
        } else {
            result
                .push_str(
                    ({
                        let _slice_src = &__sifr_chars_line2;
                        let _slice_len = _slice_src.len();
                        let _slice_start = min_indent.clamp_slice_bound(_slice_len);
                        let _slice_stop = _slice_len;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    })
                        .as_str(),
                );
        }
    }
    result
}
fn indent(text: &str, prefix: &str) -> String {
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
            result.push_str(prefix);
            result.push_str(line.as_str());
        } else {
            result.push_str(line.as_str());
        }
    }
    result
}
fn shorten(text: &str, width: SifrInt) -> String {
    let normalized: String = _normalize_whitespace(text);
    let words: Vec<String> = normalized
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    for word in words.iter().cloned() {
        let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_word.len()) == &SifrInt::from_i64(0)) {} else {
            if (&SifrInt::from(result.chars().count()) == &SifrInt::from_i64(0)) {
                result = word;
            } else {
                if (&(&(&(&SifrInt::from(result.chars().count()) + &SifrInt::from_i64(1))
                    + &SifrInt::from(__sifr_chars_word.len())) + &SifrInt::from_i64(4))
                    <= &width)
                {
                    result.push(' ');
                    result.push_str(word.as_str());
                } else {
                    return {
                        let mut __sifr_concat: String = String::with_capacity(
                            result.len() + 6usize,
                        );
                        __sifr_concat.push_str(result.as_str());
                        __sifr_concat.push_str(" [...]");
                        __sifr_concat
                    };
                }
            }
        }
    }
    result
}
fn collect_wrap_fill_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let lines: Vec<String> = wrap(
            &"alpha\tbeta\ngamma".to_string(),
            SifrInt::from_i64(10),
        )?;
        actual
            .push(
                format!("{:?}", lines).as_str()
                    == "[\"alpha beta\", \"gamma\"]".to_string().as_str(),
            );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let filled: String = fill(
            &"alpha\tbeta\ngamma".to_string(),
            SifrInt::from_i64(10),
        )?;
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
    actual.push(dedent(&"  x\n  y".to_string()).as_str() == "x\ny".to_string().as_str());
    actual
        .push(
            indent(&"x\n \ny".to_string(), &">> ".to_string()).as_str()
                == ">> x\n \n>> y".to_string().as_str(),
        );
    actual
        .push(
            shorten(&"alpha beta gamma".to_string(), SifrInt::from_i64(16)).as_str()
                == "alpha beta [...]".to_string().as_str(),
        );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let wrap_empty: Vec<String> = wrap(&"".to_string(), SifrInt::from_i64(5))?;
        actual.push(format!("{:?}", wrap_empty).as_str() == "[]".to_string().as_str());
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push(false);
    }
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
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

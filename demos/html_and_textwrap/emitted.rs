// src/main.rs
// --- stdlib: _sifr.html ---
fn html_escape(s: &String) -> String {
    ::sifr_stdlib::html::html_escape(s)
}
fn html_unescape(s: &String) -> String {
    ::sifr_stdlib::html::html_unescape(s)
}

// --- stdlib: sifr.html ---
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = html_escape(s);
    if quote {
        return escaped;
    }
    escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
}
fn unescape(s: &String) -> String {
    html_unescape(s)
}

// --- stdlib: sifr.textwrap ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
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
impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
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
        let __sifr_field_init_0: i64 = width;
        let __sifr_field_init_1: String = {
            let mut __sifr_concat: String = String::with_capacity(
                initial_indent.len() + 0usize,
            );
            __sifr_concat.push_str((initial_indent).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_2: String = {
            let mut __sifr_concat: String = String::with_capacity(
                subsequent_indent.len() + 0usize,
            );
            __sifr_concat.push_str((subsequent_indent).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_3: bool = expand_tabs;
        let mut safe_tabsize: i64 = tabsize;
        if safe_tabsize <= (0_i64) {
            safe_tabsize = 1_i64;
        }
        let __sifr_field_init_4: i64 = safe_tabsize;
        let __sifr_field_init_5: bool = replace_whitespace;
        let __sifr_field_init_6: bool = drop_whitespace;
        let __sifr_field_init_7: bool = break_on_hyphens;
        let __sifr_field_init_8: bool = fix_sentence_endings;
        let __sifr_field_init_9: Option<i64> = max_lines;
        let __sifr_field_init_10: String = {
            let mut __sifr_concat: String = String::with_capacity(
                placeholder.len() + 0usize,
            );
            __sifr_concat.push_str((placeholder).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self {
            width: __sifr_field_init_0,
            initial_indent: __sifr_field_init_1,
            subsequent_indent: __sifr_field_init_2,
            expand_tabs: __sifr_field_init_3,
            tabsize: __sifr_field_init_4,
            replace_whitespace: __sifr_field_init_5,
            drop_whitespace: __sifr_field_init_6,
            break_on_hyphens: __sifr_field_init_7,
            fix_sentence_endings: __sifr_field_init_8,
            max_lines: __sifr_field_init_9,
            placeholder: __sifr_field_init_10,
        }
    }
}
impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
    fn wrap(&self, text: &String) -> Vec<String> {
        if (self.width <= (0_i64)) {
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
        _apply_max_lines(
            &lines,
            self.width,
            self.max_lines,
            &self.placeholder,
            self.drop_whitespace,
        )
    }
}
impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
    fn fill(&self, text: &String) -> String {
        if (self.width <= (0_i64)) {
            return "".to_string();
        }
        let lines: Vec<String> = self.wrap(text);
        let mut result: String = "".to_string();
        let mut i: i64 = 0_i64;
        for line in lines.iter().cloned() {
            if i > (0_i64) {
                result.push('\n');
            }
            result.push_str((line).as_str());
            i += 1_i64;
        }
        result
    }
}
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
fn _apply_sentence_endings_line(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
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
            result.push_str((ch).as_str());
            if ((ch == ".") || (ch == "!")) || (ch == "?") {
                let mut next_opt: Option<String> = None;
                if ((i + (1_i64)) < (__sifr_chars_text.len() as i64)) {
                    next_opt = Some({
                        let Some(__indexed_char) = __sifr_chars_text
                            .get((i + (1_i64)) as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                }
                let mut next2_opt: Option<String> = None;
                if ((i + (2_i64)) < (__sifr_chars_text.len() as i64)) {
                    next2_opt = Some({
                        let Some(__indexed_char) = __sifr_chars_text
                            .get((i + (2_i64)) as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                }
                if next_opt.is_some() && (next_opt == Some(" ".to_string())) {
                    if next2_opt.is_none() || (next2_opt != Some(" ".to_string())) {
                        result.push(' ');
                    }
                }
            }
        }
        i += 1_i64;
    }
    result
}
fn _apply_sentence_endings_lines(lines: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for line in lines.iter().cloned() {
        result.push(_apply_sentence_endings_line(&line));
    }
    result
}
fn _clone_lines(lines: &Vec<String>) -> Vec<String> {
    let mut copied: Vec<String> = vec![];
    for line in lines.iter().cloned() {
        copied.push(line.clone());
    }
    copied
}
fn _apply_max_lines(
    lines: &Vec<String>,
    width: i64,
    max_lines: Option<i64>,
    placeholder: &String,
    drop_whitespace: bool,
) -> Vec<String> {
    let Some(max_lines) = max_lines else {
        return _clone_lines(lines);
    };
    let limit: i64 = max_lines;
    if limit <= (0_i64) {
        return vec![];
    }
    if ((lines.len() as i64) <= limit) {
        return _clone_lines(lines);
    }
    let mut result: Vec<String> = vec![];
    let mut i: i64 = 0_i64;
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
            result.push(line_opt.clone());
        }
        i += 1_i64;
    }
    if (result.len() as i64) == (0_i64) {
        return result;
    }
    let mut effective_placeholder: String = {
        let mut __sifr_concat: String = String::with_capacity(
            placeholder.len() + 0usize,
        );
        __sifr_concat.push_str((placeholder).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if width > (0_i64) {
        if ((effective_placeholder.chars().count() as i64) > width) {
            effective_placeholder = {
                let _slice_src = &effective_placeholder;
                let _slice_len_i64 = _slice_src.chars().count() as i64;
                let _slice_start_i64 = if (0_i64) < 0 {
                    (_slice_len_i64 + (0_i64)).max(0)
                } else {
                    (0_i64).min(_slice_len_i64)
                };
                let _slice_stop_i64 = if width < 0 {
                    (_slice_len_i64 + width).max(0)
                } else {
                    width.min(_slice_len_i64)
                };
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
                )
            };
        }
    }
    let last_index: i64 = (result.len() as i64) - (1_i64);
    let last_opt: Option<String> = Some(result[last_index as usize].clone());
    if let Some(last_opt) = last_opt {
        let last: String = last_opt;
        let mut base: String = _trim_line(&last);
        let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut available: i64 = width - (effective_placeholder.chars().count() as i64);
        if available < (0_i64) {
            available = 0_i64;
        }
        if ((__sifr_chars_base.len() as i64) > available) {
            base = _trim_line(
                &base
                    .chars()
                    .skip((0_i64) as usize)
                    .take((available as usize) - ((0_i64) as usize))
                    .collect::<String>(),
            );
            __sifr_chars_base = base.chars().collect::<Vec<char>>();
        }
        if drop_whitespace {
            base = _trim_line(&base);
            __sifr_chars_base = base.chars().collect::<Vec<char>>();
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
                    *__elem = {
                        let mut __sifr_concat: String = String::with_capacity(
                            base.len() + effective_placeholder.len(),
                        );
                        __sifr_concat.push_str((base).as_str());
                        __sifr_concat.push_str((effective_placeholder).as_str());
                        __sifr_concat
                    };
                }
            }
        }
    }
    result
}
// --- end stdlib ---

fn main() {
    let wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(8_i64, "".to_string(), "".to_string(), true, 8_i64, true, true, false, false, None, " [...]".to_string());
    let lines: Vec<String> = wrapper.wrap(&"alpha-beta gamma".to_string());
    assert!((format!("{:?}", lines) == "[\"alpha-beta\", \"gamma\"]"));
    let keep_ws: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(10_i64, "".to_string(), "".to_string(), true, 8_i64, true, false, true, false, None, " [...]".to_string());
    assert!((format!("{:?}", keep_ws.wrap(&"a  b".to_string())) == "[\"a  b\"]"));
    let text: String = "<a href=\"x\">\'ok\' & done</a>".to_string();
    let escaped_default: String = escape(&text, true);
    let escaped_no_quote: String = escape(&text, false);
    assert!(escaped_default == "&lt;a href=&quot;x&quot;&gt;&#x27;ok&#x27; &amp; done&lt;/a&gt;");
    assert!(escaped_no_quote == "&lt;a href=\"x\"&gt;\'ok\' &amp; done&lt;/a&gt;");
    assert!((unescape(&escaped_default) == text));
}

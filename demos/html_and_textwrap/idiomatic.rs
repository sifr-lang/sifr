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

fn main() {
    let mut wrapper: TextWrapper = TextWrapper::new(
        8 as i64,
        "".to_string(),
        "".to_string(),
        true,
        8 as i64,
        true,
        true,
        false,
        false,
        None,
        " [...]".to_string(),
    );
    let lines: Vec<String> = wrapper.wrap(&"alpha-beta gamma".to_string());
    assert!(format!("{:?}", lines) == "[\"alpha-beta\", \"gamma\"]".to_string());
    let mut keep_ws: TextWrapper = TextWrapper::new(
        10 as i64,
        "".to_string(),
        "".to_string(),
        true,
        8 as i64,
        true,
        false,
        true,
        false,
        None,
        " [...]".to_string(),
    );
    assert!(format!("{:?}", keep_ws.wrap(&"a  b".to_string())) == "[\"a  b\"]".to_string());
    let text: String = "<a href=\"x\">\'ok\' & done</a>".to_string();
    let escaped_default: String = escape(&text, true);
    let escaped_no_quote: String = escape(&text, false);
    assert!(
        escaped_default
            == "&lt;a href=&quot;x&quot;&gt;&#x27;ok&#x27; &amp; done&lt;/a&gt;".to_string()
    );
    assert!(escaped_no_quote == "&lt;a href=\"x\"&gt;\'ok\' &amp; done&lt;/a&gt;".to_string());
    assert!(unescape(&escaped_default) == text);
}

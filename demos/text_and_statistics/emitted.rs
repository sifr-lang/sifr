// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
        pub width: SifrInt,
        pub initial_indent: String,
        pub subsequent_indent: String,
        pub expand_tabs: bool,
        pub tabsize: SifrInt,
        pub replace_whitespace: bool,
        pub drop_whitespace: bool,
        pub break_on_hyphens: bool,
        pub fix_sentence_endings: bool,
        pub max_lines: Option<SifrInt>,
        pub placeholder: String,
    }
    impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
        pub fn new(
            width: SifrInt,
            initial_indent: String,
            subsequent_indent: String,
            expand_tabs: bool,
            tabsize: SifrInt,
            replace_whitespace: bool,
            drop_whitespace: bool,
            break_on_hyphens: bool,
            fix_sentence_endings: bool,
            max_lines: Option<SifrInt>,
            placeholder: String,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = width.clone();
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
            let mut safe_tabsize: SifrInt = tabsize.clone();
            if &safe_tabsize <= &SifrInt::from_i64(0) {
                safe_tabsize = SifrInt::from_i64(1);
            }
            let __sifr_field_init_4: SifrInt = safe_tabsize.clone();
            let __sifr_field_init_5: bool = replace_whitespace;
            let __sifr_field_init_6: bool = drop_whitespace;
            let __sifr_field_init_7: bool = break_on_hyphens;
            let __sifr_field_init_8: bool = fix_sentence_endings;
            let __sifr_field_init_9: Option<SifrInt> = max_lines.clone();
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
        pub fn wrap(&self, text: &String) -> Vec<String> {
            if (&self.width.clone() <= &SifrInt::from_i64(0)) {
                return vec![];
            }
            let prepared: String = _prepare_text(
                text,
                self.expand_tabs,
                self.tabsize.clone(),
                self.replace_whitespace,
            );
            let mut lines: Vec<String> = _wrap_with_indents(
                &prepared,
                self.width.clone(),
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
                self.width.clone(),
                self.max_lines.clone(),
                &self.placeholder,
                self.drop_whitespace,
            )
        }
    }
    impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
        pub fn fill(&self, text: &String) -> String {
            if (&self.width.clone() <= &SifrInt::from_i64(0)) {
                return "".to_string();
            }
            let lines: Vec<String> = self.wrap(text);
            let mut result: String = "".to_string();
            let mut i: SifrInt = SifrInt::from_i64(0);
            for line in lines.iter().cloned() {
                if (&i > &SifrInt::from_i64(0)) {
                    result.push('\n');
                }
                result.push_str((line).as_str());
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    pub fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
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
    pub fn _expand_tabs_impl(text: &String, tabsize: SifrInt) -> String {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut effective_tabsize: SifrInt = tabsize.clone();
        if &effective_tabsize <= &SifrInt::from_i64(0) {
            effective_tabsize = SifrInt::from_i64(1);
        }
        if (&effective_tabsize == &SifrInt::from_i64(0)) {
            return text.clone();
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
                        result.push_str((ch).as_str());
                        column = SifrInt::from_i64(0);
                    } else {
                        result.push_str((ch).as_str());
                        column = &column + &SifrInt::from_i64(1);
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub fn _prepare_text(
        text: &String,
        expand_tabs: bool,
        tabsize: SifrInt,
        replace_whitespace: bool,
    ) -> String {
        let mut prepared: String = {
            let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
            __sifr_concat.push_str((text).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if expand_tabs {
            prepared = _expand_tabs_impl(&prepared, (tabsize).clone());
        }
        if replace_whitespace {
            prepared = _replace_whitespace_chars(&prepared, true);
        }
        prepared
    }
    pub fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
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
        if (&SifrInt::from(parts.len()) <= &SifrInt::from_i64(1)) {
            return vec![
                { let mut __sifr_concat : String = String::with_capacity(word.len() +
                0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
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
                    units.push(part.clone());
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
    pub fn _trim_line(line: &String) -> String {
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
    pub fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
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
    pub fn _effective_content_width(total_width: SifrInt, indent: &String) -> SifrInt {
        let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
        let available: SifrInt = &total_width - &SifrInt::from(__sifr_chars_indent.len());
        if &available <= &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        available.clone()
    }
    pub fn _push_current_line(
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
            if (&SifrInt::from(__sifr_chars_candidate.len()) > &SifrInt::from_i64(0)) {
                result.push(candidate.clone());
            }
        } else {
            result.push(candidate.clone());
        }
    }
    pub fn _wrap_with_indents(
        text: &String,
        total_width: SifrInt,
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
        let mut current_limit: SifrInt = _effective_content_width(
            (total_width).clone(),
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
                                (total_width).clone(),
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
    pub fn _apply_sentence_endings_line(text: &String) -> String {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut result: String = "".to_string();
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
                result.push_str((ch).as_str());
                if ((ch == ".") || (ch == "!")) || (ch == "?") {
                    let mut next_opt: Option<String> = None;
                    if (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len()))
                    {
                        if let Some(__sifr_checked_value_4) = ({
                            let __sifr_string_index = &i + &SifrInt::from_i64(1);
                            let __sifr_string_index_normalized = __sifr_string_index
                                .normalize_index_or_len(__sifr_chars_text.len());
                            __sifr_chars_text.get(__sifr_string_index_normalized)
                        })
                            .map(|c| c.to_string())
                        {
                            next_opt = ({
                                let __sifr_string_index = &i + &SifrInt::from_i64(1);
                                let __sifr_string_index_normalized = __sifr_string_index
                                    .normalize_index_or_len(__sifr_chars_text.len());
                                __sifr_chars_text.get(__sifr_string_index_normalized)
                            })
                                .map(|c| c.to_string());
                        }
                    }
                    let mut next2_opt: Option<String> = None;
                    if (&(&i + &SifrInt::from_i64(2))
                        < &SifrInt::from(__sifr_chars_text.len()))
                    {
                        next2_opt = ({
                            let __sifr_string_index = &i + &SifrInt::from_i64(2);
                            let __sifr_string_index_normalized = __sifr_string_index
                                .normalize_index_or_len(__sifr_chars_text.len());
                            __sifr_chars_text.get(__sifr_string_index_normalized)
                        })
                            .map(|c| c.to_string());
                    }
                    if (next_opt.is_some()) && (next_opt == Some(" ".to_string())) {
                        if (next2_opt.is_none()) || (next2_opt != Some(" ".to_string())) {
                            result.push(' ');
                        }
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub fn _apply_sentence_endings_lines(lines: &Vec<String>) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        for line in lines.iter().cloned() {
            result.push(_apply_sentence_endings_line(&line));
        }
        result
    }
    pub fn _clone_lines(lines: &Vec<String>) -> Vec<String> {
        let mut copied: Vec<String> = vec![];
        for line in lines.iter().cloned() {
            copied.push(line.clone());
        }
        copied
    }
    pub fn _apply_max_lines(
        lines: &Vec<String>,
        width: SifrInt,
        max_lines: Option<SifrInt>,
        placeholder: &String,
        drop_whitespace: bool,
    ) -> Vec<String> {
        let Some(max_lines) = max_lines.clone() else {
            return _clone_lines(lines);
        };
        let limit: SifrInt = max_lines.clone();
        if &limit <= &SifrInt::from_i64(0) {
            return vec![];
        }
        if (&SifrInt::from(lines.len()) <= &limit) {
            return _clone_lines(lines);
        }
        let mut result: Vec<String> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &limit) {
            let line_opt: Option<String> = {
                let __sifr_checked_read_collection = &lines;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
            };
            if let Some(line_opt) = line_opt {
                result.push(line_opt.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(result.len()) == &SifrInt::from_i64(0) {
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
        if (&width > &SifrInt::from_i64(0)) {
            if (&SifrInt::from(effective_placeholder.chars().count()) > &width) {
                effective_placeholder = {
                    let _slice_src = &effective_placeholder;
                    let _slice_len = _slice_src.chars().count();
                    let _slice_start = SifrInt::from_i64(0).clamp_slice_bound(_slice_len);
                    let _slice_stop = width.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .chars()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start)),
                    )
                };
            }
        }
        let last_index: SifrInt = &SifrInt::from(result.len()) - &SifrInt::from_i64(1);
        let last_opt: Option<String> = {
            let __sifr_checked_read_collection = &result;
            let __sifr_checked_read_index = last_index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(last_opt) = last_opt {
            let last: String = last_opt;
            let mut base: String = _trim_line(&last);
            let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
            let mut available: SifrInt = &width
                - &SifrInt::from(effective_placeholder.chars().count());
            if (&available < &SifrInt::from_i64(0)) {
                available = SifrInt::from_i64(0);
            }
            if (&SifrInt::from(__sifr_chars_base.len()) > &available) {
                base = _trim_line(
                    &base
                        .chars()
                        .skip(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0))))
                        .take(
                            (::sifr_runtime::to_usize_proven(&(available)))
                                - (::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))),
                        )
                        .collect::<String>(),
                );
                __sifr_chars_base = base.chars().collect::<Vec<char>>();
            }
            if drop_whitespace {
                base = _trim_line(&base);
                __sifr_chars_base = base.chars().collect::<Vec<char>>();
            }
            if (&SifrInt::from_i64(0) <= &last_index)
                && (&last_index < &SifrInt::from(result.len()))
            {
                {
                    let __assign_value = {
                        let mut __sifr_concat: String = String::with_capacity(
                            base.len() + effective_placeholder.len(),
                        );
                        __sifr_concat.push_str((base).as_str());
                        __sifr_concat.push_str((effective_placeholder).as_str());
                        __sifr_concat
                    };
                    {
                        let __index_raw = last_index.clone();
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(result.len());
                        if let Some(__elem) = result.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
            }
        }
        result
    }
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etextwrap_x2eTextWrapper;
use ::sifr_runtime::SifrInt;
fn html_escape(s: &String) -> String {
    ::sifr_stdlib::html::html_escape(s)
}
fn html_unescape(s: &String) -> String {
    ::sifr_stdlib::html::html_unescape(s)
}
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = html_escape(s);
    if quote {
        return escaped;
    }
    escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: SifrInt) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while (&y != &SifrInt::from_i64(0)) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd((a).clone(), (b).clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &Vec<SifrInt>) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    message: String,
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("StatisticsError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
fn _float_int(
    value: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let converted: f64 = value
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(converted))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
            }
        }
    }
}
fn median_grouped(
    data: &Vec<f64>,
    interval: f64,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped requires at least one data point".to_string(),
            ),
        );
    }
    if (interval <= (0.0_f64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped: interval must be > 0".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid_index: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
    let midpoint_opt: Option<f64> = {
        let __sifr_checked_read_collection = &sorted_data;
        let __sifr_checked_read_index = mid_index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(midpoint_opt) = midpoint_opt else {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped: index error".to_string(),
            ),
        );
    };
    let midpoint: f64 = midpoint_opt;
    let mut cf: SifrInt = SifrInt::from_i64(0);
    let mut f: SifrInt = SifrInt::from_i64(0);
    for value in sorted_data.iter().copied() {
        if value < midpoint {
            cf = &cf + &SifrInt::from_i64(1);
        } else {
            if value == midpoint {
                f = &f + &SifrInt::from_i64(1);
            }
        }
    }
    if (&f == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped: grouped frequency is zero".to_string(),
            ),
        );
    }
    let lower: f64 = midpoint - (interval / (2.0_f64));
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let n_float: f64 = _float_int((n).clone())?;
        let cf_float: f64 = _float_int((cf).clone())?;
        let f_float: f64 = _float_int((f).clone())?;
        Ok(Ok(lower + (interval * (((n_float / (2.0_f64)) - cf_float) / f_float))))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
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
fn _expand_tabs_impl(text: &String, tabsize: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: SifrInt = tabsize.clone();
    if &effective_tabsize <= &SifrInt::from_i64(0) {
        effective_tabsize = SifrInt::from_i64(1);
    }
    if (&effective_tabsize == &SifrInt::from_i64(0)) {
        return text.clone();
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
                    result.push_str((ch).as_str());
                    column = SifrInt::from_i64(0);
                } else {
                    result.push_str((ch).as_str());
                    column = &column + &SifrInt::from_i64(1);
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: SifrInt,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str((text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, (tabsize).clone());
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
    if (&SifrInt::from(parts.len()) <= &SifrInt::from_i64(1)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
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
                units.push(part.clone());
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
fn _trim_line(line: &String) -> String {
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
fn _effective_content_width(total_width: SifrInt, indent: &String) -> SifrInt {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: SifrInt = &total_width - &SifrInt::from(__sifr_chars_indent.len());
    if &available <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    available.clone()
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
        if (&SifrInt::from(__sifr_chars_candidate.len()) > &SifrInt::from_i64(0)) {
            result.push(candidate.clone());
        }
    } else {
        result.push(candidate.clone());
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: SifrInt,
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
    let mut current_limit: SifrInt = _effective_content_width(
        (total_width).clone(),
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
                            (total_width).clone(),
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
fn _apply_sentence_endings_line(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
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
            result.push_str((ch).as_str());
            if ((ch == ".") || (ch == "!")) || (ch == "?") {
                let mut next_opt: Option<String> = None;
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    if let Some(__sifr_checked_value_4) = ({
                        let __sifr_string_index = &i + &SifrInt::from_i64(1);
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_text.len());
                        __sifr_chars_text.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string())
                    {
                        next_opt = ({
                            let __sifr_string_index = &i + &SifrInt::from_i64(1);
                            let __sifr_string_index_normalized = __sifr_string_index
                                .normalize_index_or_len(__sifr_chars_text.len());
                            __sifr_chars_text.get(__sifr_string_index_normalized)
                        })
                            .map(|c| c.to_string());
                    }
                }
                let mut next2_opt: Option<String> = None;
                if (&(&i + &SifrInt::from_i64(2))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    next2_opt = ({
                        let __sifr_string_index = &i + &SifrInt::from_i64(2);
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_text.len());
                        __sifr_chars_text.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                }
                if (next_opt.is_some()) && (next_opt == Some(" ".to_string())) {
                    if (next2_opt.is_none()) || (next2_opt != Some(" ".to_string())) {
                        result.push(' ');
                    }
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
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
    width: SifrInt,
    max_lines: Option<SifrInt>,
    placeholder: &String,
    drop_whitespace: bool,
) -> Vec<String> {
    let Some(max_lines) = max_lines.clone() else {
        return _clone_lines(lines);
    };
    let limit: SifrInt = max_lines.clone();
    if &limit <= &SifrInt::from_i64(0) {
        return vec![];
    }
    if (&SifrInt::from(lines.len()) <= &limit) {
        return _clone_lines(lines);
    }
    let mut result: Vec<String> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &limit) {
        let line_opt: Option<String> = {
            let __sifr_checked_read_collection = &lines;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(line_opt) = line_opt {
            result.push(line_opt.clone());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    if &SifrInt::from(result.len()) == &SifrInt::from_i64(0) {
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
    if (&width > &SifrInt::from_i64(0)) {
        if (&SifrInt::from(effective_placeholder.chars().count()) > &width) {
            effective_placeholder = {
                let _slice_src = &effective_placeholder;
                let _slice_len = _slice_src.chars().count();
                let _slice_start = SifrInt::from_i64(0).clamp_slice_bound(_slice_len);
                let _slice_stop = width.clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start)),
                )
            };
        }
    }
    let last_index: SifrInt = &SifrInt::from(result.len()) - &SifrInt::from_i64(1);
    let last_opt: Option<String> = {
        let __sifr_checked_read_collection = &result;
        let __sifr_checked_read_index = last_index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(last_opt) = last_opt {
        let last: String = last_opt;
        let mut base: String = _trim_line(&last);
        let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut available: SifrInt = &width
            - &SifrInt::from(effective_placeholder.chars().count());
        if (&available < &SifrInt::from_i64(0)) {
            available = SifrInt::from_i64(0);
        }
        if (&SifrInt::from(__sifr_chars_base.len()) > &available) {
            base = _trim_line(
                &base
                    .chars()
                    .skip(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0))))
                    .take(
                        (::sifr_runtime::to_usize_proven(&(available)))
                            - (::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))),
                    )
                    .collect::<String>(),
            );
            __sifr_chars_base = base.chars().collect::<Vec<char>>();
        }
        if drop_whitespace {
            base = _trim_line(&base);
            __sifr_chars_base = base.chars().collect::<Vec<char>>();
        }
        if (&SifrInt::from_i64(0) <= &last_index)
            && (&last_index < &SifrInt::from(result.len()))
        {
            {
                let __assign_value = {
                    let mut __sifr_concat: String = String::with_capacity(
                        base.len() + effective_placeholder.len(),
                    );
                    __sifr_concat.push_str((base).as_str());
                    __sifr_concat.push_str((effective_placeholder).as_str());
                    __sifr_concat
                };
                {
                    let __index_raw = last_index.clone();
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(result.len());
                    if let Some(__elem) = result.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    result
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatOverflowError {
    message: String,
}
impl FloatOverflowError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatOverflowError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatOverflowError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatPrecisionLossError {
    message: String,
}
impl FloatPrecisionLossError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatPrecisionLossError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatPrecisionLossError {}
impl From<FloatOverflowError> for Error {
    fn from(err: FloatOverflowError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatPrecisionLossError> for Error {
    fn from(err: FloatPrecisionLossError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    let mut grouped: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let grouped_value: f64 = median_grouped(
            &vec![1.0_f64, 2.0_f64, 2.0_f64, 3.0_f64, 4.0_f64],
            1.0_f64,
        )?;
        grouped = grouped_value;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
        assert!(
            (format!("{}", "median_grouped unexpected error") ==
            "rng_text_and_statistics_waiver_reduction_demo: pass")
        );
    }
    assert!(grouped > (2.2_f64));
    assert!(grouped < (2.3_f64));
    let wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        SifrInt::from_i64(12),
        "".to_string(),
        "".to_string(),
        true,
        SifrInt::from_i64(8),
        true,
        true,
        true,
        false,
        Some(SifrInt::from_i64(2)),
        "...".to_string(),
    );
    let wrapped: Vec<String> = wrapper
        .wrap(&"alpha beta gamma delta epsilon".to_string());
    assert!((format!("{:?}", wrapped) == "[\"alpha beta\", \"gamma del...\"]"));
    let sentence_wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        SifrInt::from_i64(40),
        "".to_string(),
        "".to_string(),
        true,
        SifrInt::from_i64(8),
        true,
        true,
        true,
        true,
        None,
        " [...]".to_string(),
    );
    let filled: String = sentence_wrapper.fill(&"Hello. World. Done!".to_string());
    assert!((filled).as_str() == ("Hello.  World.  Done!".to_string()).as_str());
    let escaped: String = escape(&"<a \"x\">".to_string(), false);
    assert!((escaped).as_str() == ("&lt;a \"x\"&gt;".to_string()).as_str());
    assert!(
        (format!("{}", "rng_text_and_statistics_waiver_reduction_demo: pass") ==
        "rng_text_and_statistics_waiver_reduction_demo: pass")
    );
}

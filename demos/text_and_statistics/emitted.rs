// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
        pub width: i64,
        pub initial_indent: String,
        pub subsequent_indent: String,
        pub expand_tabs: bool,
        pub tabsize: i64,
        pub replace_whitespace: bool,
        pub drop_whitespace: bool,
        pub break_on_hyphens: bool,
        pub fix_sentence_endings: bool,
        pub max_lines: Option<i64>,
        pub placeholder: String,
    }
    impl __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper {
        pub fn new(
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
        pub fn wrap(&self, text: &String) -> Vec<String> {
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
        pub fn fill(&self, text: &String) -> String {
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
    pub fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
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
    pub fn _prepare_text(
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
    pub fn _trim_line(line: &String) -> String {
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
    pub fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
        let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
        let available: i64 = total_width - (__sifr_chars_indent.len() as i64);
        if available <= (0_i64) {
            return 1_i64;
        }
        available
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
            if ((__sifr_chars_candidate.len() as i64) > (0_i64)) {
                result.push(candidate.clone());
            }
        } else {
            result.push(candidate.clone());
        }
    }
    pub fn _wrap_with_indents(
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
    pub fn _apply_sentence_endings_line(text: &String) -> String {
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
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etextwrap_x2eTextWrapper;
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
fn floor(x: f64) -> i64 {
    ::sifr_stdlib::math::floor(x).to_i64_saturating()
}
fn ceil(x: f64) -> i64 {
    ::sifr_stdlib::math::ceil(x).to_i64_saturating()
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
fn round_val(x: f64) -> i64 {
    ::sifr_stdlib::math::round_val(x).to_i64_saturating()
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
fn trunc(x: f64) -> i64 {
    ::sifr_stdlib::math::trunc(x).to_i64_saturating()
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
fn isqrt(n: i64) -> i64 {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .to_i64_saturating()
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
fn ldexp(m: f64, e: i64) -> f64 {
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
fn factorial(n: i64) -> i64 {
    if n < (0_i64) {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 2_i64;
    while i <= n {
        result *= i;
        i += 1_i64;
    }
    result
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    while y != (0_i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    x
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0_i64) {
        return 0_i64;
    }
    if b == (0_i64) {
        return 0_i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    let mut y: i64 = b;
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    (x / g) * y
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    if k == (0_i64) {
        return 1_i64;
    }
    if k == n {
        return 1_i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < r {
        result *= n - i;
        result /= i + (1_i64);
        i += 1_i64;
    }
    result
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < k {
        result *= n - i;
        i += 1_i64;
    }
    result
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
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1_i64;
    for val in data.iter().copied() {
        result *= val;
    }
    result
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
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0_i64;
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
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
fn median_grouped(
    data: &Vec<f64>,
    interval: f64,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped requires at least one data point".to_string(),
            ),
        );
    }
    if interval <= (0.0_f64) {
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
    let mid_index: i64 = n / (2_i64);
    let midpoint_opt: Option<f64> = {
        let __sifr_index_list = &sorted_data;
        let __sifr_index_i = mid_index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(midpoint_opt) = midpoint_opt else {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped: index error".to_string(),
            ),
        );
    };
    let midpoint: f64 = midpoint_opt;
    let mut cf: i64 = 0_i64;
    let mut f: i64 = 0_i64;
    for value in sorted_data.iter().copied() {
        if value < midpoint {
            cf += 1_i64;
        } else {
            if value == midpoint {
                f += 1_i64;
            }
        }
    }
    if f == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_grouped: grouped frequency is zero".to_string(),
            ),
        );
    }
    let lower: f64 = midpoint - (interval / (2.0_f64));
    Ok(lower + (interval * ((((n as f64) / (2.0_f64)) - (cf as f64)) / (f as f64))))
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
        12_i64,
        "".to_string(),
        "".to_string(),
        true,
        8_i64,
        true,
        true,
        true,
        false,
        Some(2_i64),
        "...".to_string(),
    );
    let wrapped: Vec<String> = wrapper
        .wrap(&"alpha beta gamma delta epsilon".to_string());
    assert!((format!("{:?}", wrapped) == "[\"alpha beta\", \"gamma del...\"]"));
    let sentence_wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        40_i64,
        "".to_string(),
        "".to_string(),
        true,
        8_i64,
        true,
        true,
        true,
        true,
        None,
        " [...]".to_string(),
    );
    let filled: String = sentence_wrapper.fill(&"Hello. World. Done!".to_string());
    assert!(filled == "Hello.  World.  Done!");
    let escaped: String = escape(&"<a \"x\">".to_string(), false);
    assert!(escaped == "&lt;a \"x\"&gt;");
    assert!(
        (format!("{}", "rng_text_and_statistics_waiver_reduction_demo: pass") ==
        "rng_text_and_statistics_waiver_reduction_demo: pass")
    );
}

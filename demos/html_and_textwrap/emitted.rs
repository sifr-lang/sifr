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
                if &i > &SifrInt::from_i64(0) {
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
        if &effective_tabsize == &SifrInt::from_i64(0) {
            return text.clone();
        }
        let mut result: String = "".to_string();
        let mut column: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from(__sifr_chars_text.len())) {
            let ch_opt: Option<String> = Some({
                let __indexed_char_option = __sifr_chars_text
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            });
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "\t" {
                    let mut spaces: SifrInt = &effective_tabsize
                        - &column.floor_mod_known_nonzero(&effective_tabsize);
                    if &spaces <= &SifrInt::from_i64(0) {
                        spaces = effective_tabsize.clone();
                    }
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while &j < &spaces {
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
            && (({
                let __indexed_char_option = __sifr_chars_line
                    .get(::sifr_runtime::to_usize_proven(&(start)))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            }) == " ")
        {
            start = &start + &SifrInt::from_i64(1);
        }
        let mut end: SifrInt = SifrInt::from(__sifr_chars_line.len());
        while (&end > &start)
            && (__sifr_chars_line
                .get(::sifr_runtime::to_usize_proven(&(&end - &SifrInt::from_i64(1))))
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
            let ch_opt: Option<String> = Some({
                let __indexed_char_option = __sifr_chars_text
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string());
                __indexed_char_option.as_slice()[0_usize].clone()
            });
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                result.push_str((ch).as_str());
                if ((ch == ".") || (ch == "!")) || (ch == "?") {
                    let mut next_opt: Option<String> = None;
                    if (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len()))
                    {
                        next_opt = Some({
                            let __indexed_char_option = __sifr_chars_text
                                .get(
                                    ::sifr_runtime::to_usize_proven(
                                        &(&i + &SifrInt::from_i64(1)),
                                    ),
                                )
                                .map(|c| c.to_string());
                            __indexed_char_option.as_slice()[0_usize].clone()
                        });
                    }
                    let mut next2_opt: Option<String> = None;
                    if (&(&i + &SifrInt::from_i64(2))
                        < &SifrInt::from(__sifr_chars_text.len()))
                    {
                        next2_opt = Some({
                            let __indexed_char_option = __sifr_chars_text
                                .get(
                                    ::sifr_runtime::to_usize_proven(
                                        &(&i + &SifrInt::from_i64(2)),
                                    ),
                                )
                                .map(|c| c.to_string());
                            __indexed_char_option.as_slice()[0_usize].clone()
                        });
                    }
                    if next_opt.is_some() && (next_opt == Some(" ".to_string())) {
                        if next2_opt.is_none() || (next2_opt != Some(" ".to_string())) {
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
        while &i < &limit {
            let line_opt: Option<String> = {
                let __sifr_index_list = &lines;
                let __sifr_index_i = i.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
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
        if &width > &SifrInt::from_i64(0) {
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
        let last_opt: Option<String> = Some(
            result[::sifr_runtime::to_usize_proven(&(last_index))].clone(),
        );
        if let Some(last_opt) = last_opt {
            let last: String = last_opt;
            let mut base: String = _trim_line(&last);
            let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
            let mut available: SifrInt = &width
                - &SifrInt::from(effective_placeholder.chars().count());
            if &available < &SifrInt::from_i64(0) {
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
            {
                let __idx_raw = last_index.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(result.len());
                if let Some(__elem) = result.get_mut(__idx_norm) {
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
fn unescape(s: &String) -> String {
    html_unescape(s)
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
    if &effective_tabsize == &SifrInt::from_i64(0) {
        return text.clone();
    }
    let mut result: String = "".to_string();
    let mut column: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = Some({
            let __indexed_char_option = __sifr_chars_text
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t" {
                let mut spaces: SifrInt = &effective_tabsize
                    - &column.floor_mod_known_nonzero(&effective_tabsize);
                if &spaces <= &SifrInt::from_i64(0) {
                    spaces = effective_tabsize.clone();
                }
                let mut j: SifrInt = SifrInt::from_i64(0);
                while &j < &spaces {
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
        && (({
            let __indexed_char_option = __sifr_chars_line
                .get(::sifr_runtime::to_usize_proven(&(start)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        }) == " ")
    {
        start = &start + &SifrInt::from_i64(1);
    }
    let mut end: SifrInt = SifrInt::from(__sifr_chars_line.len());
    while (&end > &start)
        && (__sifr_chars_line
            .get(::sifr_runtime::to_usize_proven(&(&end - &SifrInt::from_i64(1))))
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
        let ch_opt: Option<String> = Some({
            let __indexed_char_option = __sifr_chars_text
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            result.push_str((ch).as_str());
            if ((ch == ".") || (ch == "!")) || (ch == "?") {
                let mut next_opt: Option<String> = None;
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    next_opt = Some({
                        let __indexed_char_option = __sifr_chars_text
                            .get(
                                ::sifr_runtime::to_usize_proven(
                                    &(&i + &SifrInt::from_i64(1)),
                                ),
                            )
                            .map(|c| c.to_string());
                        __indexed_char_option.as_slice()[0_usize].clone()
                    });
                }
                let mut next2_opt: Option<String> = None;
                if (&(&i + &SifrInt::from_i64(2))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    next2_opt = Some({
                        let __indexed_char_option = __sifr_chars_text
                            .get(
                                ::sifr_runtime::to_usize_proven(
                                    &(&i + &SifrInt::from_i64(2)),
                                ),
                            )
                            .map(|c| c.to_string());
                        __indexed_char_option.as_slice()[0_usize].clone()
                    });
                }
                if next_opt.is_some() && (next_opt == Some(" ".to_string())) {
                    if next2_opt.is_none() || (next2_opt != Some(" ".to_string())) {
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
    while &i < &limit {
        let line_opt: Option<String> = {
            let __sifr_index_list = &lines;
            let __sifr_index_i = i.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
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
    if &width > &SifrInt::from_i64(0) {
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
    let last_opt: Option<String> = Some(
        result[::sifr_runtime::to_usize_proven(&(last_index))].clone(),
    );
    if let Some(last_opt) = last_opt {
        let last: String = last_opt;
        let mut base: String = _trim_line(&last);
        let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut available: SifrInt = &width
            - &SifrInt::from(effective_placeholder.chars().count());
        if &available < &SifrInt::from_i64(0) {
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
        {
            let __idx_raw = last_index.clone();
            let __idx_norm = __idx_raw.normalize_index_or_len(result.len());
            if let Some(__elem) = result.get_mut(__idx_norm) {
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
    result
}
fn main() {
    let wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        SifrInt::from_i64(8),
        "".to_string(),
        "".to_string(),
        true,
        SifrInt::from_i64(8),
        true,
        true,
        false,
        false,
        None,
        " [...]".to_string(),
    );
    let lines: Vec<String> = wrapper.wrap(&"alpha-beta gamma".to_string());
    assert!((format!("{:?}", lines) == "[\"alpha-beta\", \"gamma\"]"));
    let keep_ws: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        SifrInt::from_i64(10),
        "".to_string(),
        "".to_string(),
        true,
        SifrInt::from_i64(8),
        true,
        false,
        true,
        false,
        None,
        " [...]".to_string(),
    );
    assert!((format!("{:?}", keep_ws.wrap(& "a  b".to_string())) == "[\"a  b\"]"));
    let text: String = "<a href=\"x\">\'ok\' & done</a>".to_string();
    let escaped_default: String = escape(&text, true);
    let escaped_no_quote: String = escape(&text, false);
    assert!(
        (escaped_default).as_str() ==
        ("&lt;a href=&quot;x&quot;&gt;&#x27;ok&#x27; &amp; done&lt;/a&gt;".to_string())
        .as_str()
    );
    assert!(
        (escaped_no_quote).as_str() == ("&lt;a href=\"x\"&gt;\'ok\' &amp; done&lt;/a&gt;"
        .to_string()).as_str()
    );
    assert!((unescape(& escaped_default) == text));
}

// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn html_escape(s: &str) -> String {
        ::sifr_stdlib::html::html_escape(s)
    }
    pub(super) fn html_unescape(s: &str) -> String {
        ::sifr_stdlib::html::html_unescape(s)
    }
    pub(super) fn escape(s: &str, quote: bool) -> String {
        let escaped: String = html_escape(s);
        if quote {
            return escaped;
        }
        escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
    }
    pub(super) fn unescape(s: &str) -> String {
        html_unescape(s)
    }
    pub(super) fn sifr_generated_replace_whitespace_chars(
        text: &str,
        replace_tabs: bool,
    ) -> String {
        let normalized: String = text
            .replace(['\r', '\n'], " ")
            .replace(['\u{c}', '\u{b}'], " ");
        if replace_tabs {
            return normalized.replace('\t', " ");
        }
        normalized
    }
    pub(super) fn sifr_generated_expand_tabs_impl(text: &str, tabsize: &SifrInt) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut effective_tabsize: SifrInt = (*tabsize).clone();
        if effective_tabsize <= SifrInt::from_i64(0) {
            effective_tabsize = SifrInt::from_i64(1);
        }
        if effective_tabsize == SifrInt::from_i64(0) {
            return text.to_owned();
        }
        let mut result: String = String::new();
        let mut column: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < sifr_generated_chars_text.len() {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = &i;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_text.len());
                sifr_generated_chars_text
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "\t" {
                    let mut spaces: SifrInt = ::std::ops::Sub::sub(
                        &effective_tabsize,
                        &column.floor_mod_known_nonzero(&effective_tabsize),
                    );
                    if spaces <= SifrInt::from_i64(0) {
                        spaces.clone_from(&effective_tabsize);
                    }
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while j < spaces {
                        result.push(' ');
                        j = ::std::ops::Add::add(&j, &SifrInt::from_i64(1));
                    }
                    column = ::std::ops::Add::add(&column, &spaces);
                } else {
                    let sifr_generated_shared_branch_condition = ch == "\n" || ch == "\r";
                    result.push_str(ch.as_str());
                    if sifr_generated_shared_branch_condition {
                        column = SifrInt::from_i64(0);
                    } else {
                        column = ::std::ops::Add::add(&column, &SifrInt::from_i64(1));
                    }
                }
            }
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn sifr_generated_prepare_text(
        text: &str,
        expand_tabs: bool,
        tabsize: &SifrInt,
        replace_whitespace: bool,
    ) -> String {
        let mut prepared: String = text.to_string();
        if expand_tabs {
            prepared = sifr_generated_expand_tabs_impl(&prepared, tabsize);
        }
        if replace_whitespace {
            prepared = sifr_generated_replace_whitespace_chars(&prepared, true);
        }
        prepared
    }
    pub(super) fn sifr_generated_split_word_units(
        word: &str,
        break_on_hyphens: bool,
    ) -> Vec<String> {
        if !break_on_hyphens {
            return vec![word.to_string()];
        }
        let parts: Vec<String> = word
            .split('-')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        if parts.len() <= SifrInt::from_i64(1) {
            return vec![word.to_string()];
        }
        let mut units: Vec<String> = Vec::new();
        let mut index: SifrInt = SifrInt::from_i64(0);
        for part in parts.iter().cloned() {
            let sifr_generated_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
            let is_last: bool =
                index == ::std::ops::Sub::sub(&SifrInt::from(parts.len()), &SifrInt::from_i64(1));
            if is_last {
                if sifr_generated_chars_part.len() > SifrInt::from_i64(0) {
                    units.push(part);
                }
            } else if sifr_generated_chars_part.len() == SifrInt::from_i64(0) {
                units.push("-".to_string());
            } else {
                units.push(format!("{part}-"));
            }
            index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
        }
        if units.len() == SifrInt::from_i64(0) {
            units.push(word.to_string());
        }
        units
    }
    pub(super) fn sifr_generated_trim_line(line: &str) -> String {
        let sifr_generated_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
        let mut start: SifrInt = SifrInt::from_i64(0);
        while start < sifr_generated_chars_line.len() && {
            let sifr_generated_string_index = &start;
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
            sifr_generated_chars_line
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string())
        .is_some_and(|_checked_value_2| {
            {
                let sifr_generated_string_index = &start;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_line.len());
                sifr_generated_chars_line
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                == Some(Some(' '))
        }) {
            start = ::std::ops::Add::add(&start, &SifrInt::from_i64(1));
        }
        let mut end: SifrInt = SifrInt::from(sifr_generated_chars_line.len());
        while end > start && {
            let sifr_generated_string_index = ::std::ops::Sub::sub(&end, &SifrInt::from_i64(1));
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
            sifr_generated_chars_line
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            == Some(Some(' '))
        {
            end = ::std::ops::Sub::sub(&end, &SifrInt::from_i64(1));
        }
        {
            let sifr_generated_slice_src = &sifr_generated_chars_line;
            let sifr_generated_slice_len = sifr_generated_slice_src.len();
            let sifr_generated_slice_start = start.clamp_slice_bound(sifr_generated_slice_len);
            let sifr_generated_slice_stop = end.clamp_slice_bound(sifr_generated_slice_len);
            String::from_iter(
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied(),
            )
        }
    }
    pub(super) fn sifr_generated_finalize_line(line: &str, drop_whitespace: bool) -> String {
        if drop_whitespace {
            return sifr_generated_trim_line(line);
        }
        line.to_string()
    }
    pub(super) fn sifr_generated_effective_content_width(
        total_width: &SifrInt,
        indent: &str,
    ) -> SifrInt {
        let available: SifrInt =
            ::std::ops::Sub::sub(total_width, &SifrInt::from(indent.chars().count()));
        if available <= SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        available
    }
    pub(super) fn sifr_generated_push_current_line(
        result: &mut Vec<String>,
        line: &str,
        indent: &str,
        drop_whitespace: bool,
    ) {
        let candidate: String =
            sifr_generated_finalize_line(&format!("{indent}{line}"), drop_whitespace);
        if drop_whitespace {
            if candidate.chars().count() > SifrInt::from_i64(0) {
                result.push(candidate);
            }
        } else {
            result.push(candidate);
        }
    }
    pub(super) fn sifr_generated_wrap_with_indents(
        text: &str,
        total_width: &SifrInt,
        initial_indent: &str,
        subsequent_indent: &str,
        break_on_hyphens: bool,
        drop_whitespace: bool,
    ) -> Vec<String> {
        let words: Vec<String> = text
            .split(' ')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let mut result: Vec<String> = Vec::new();
        let mut current: String = String::new();
        let mut sifr_generated_chars_current: Vec<char> = Vec::new();
        let mut first_line: bool = true;
        let mut current_limit: SifrInt =
            sifr_generated_effective_content_width(total_width, initial_indent);
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for raw_word in words.iter() {
            let units: Vec<String> = sifr_generated_split_word_units(raw_word, break_on_hyphens);
            for word in units.iter().cloned() {
                let sifr_generated_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
                if sifr_generated_chars_word.len() == SifrInt::from_i64(0) {
                    if drop_whitespace {
                        continue;
                    }
                    if sifr_generated_chars_current.len() > SifrInt::from_i64(0)
                        && ::std::ops::Add::add(
                            &SifrInt::from(sifr_generated_chars_current.len()),
                            &SifrInt::from_i64(1),
                        ) <= current_limit
                    {
                        current.push(' ');
                        sifr_generated_chars_current.push(' ');
                    }
                    continue;
                }
                if sifr_generated_chars_current.len() == SifrInt::from_i64(0) {
                    current = word;
                    sifr_generated_chars_current = current.chars().collect::<Vec<char>>();
                } else if ::std::ops::Add::add(
                    &::std::ops::Add::add(
                        &SifrInt::from(sifr_generated_chars_current.len()),
                        &SifrInt::from_i64(1),
                    ),
                    &SifrInt::from(sifr_generated_chars_word.len()),
                ) <= current_limit
                {
                    current.push(' ');
                    sifr_generated_chars_current.push(' ');
                    let sifr_generated_string_concat_current_1 = word;
                    current.push_str(sifr_generated_string_concat_current_1.as_str());
                    sifr_generated_chars_current
                        .extend(sifr_generated_string_concat_current_1.as_str().chars());
                } else {
                    if first_line {
                        sifr_generated_push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit =
                            sifr_generated_effective_content_width(total_width, subsequent_indent);
                    } else {
                        sifr_generated_push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                    sifr_generated_chars_current = current.chars().collect::<Vec<char>>();
                }
            }
        }
        if sifr_generated_chars_current.len() > SifrInt::from_i64(0) {
            if first_line {
                sifr_generated_push_current_line(
                    &mut result,
                    &current,
                    initial_indent,
                    drop_whitespace,
                );
            } else {
                sifr_generated_push_current_line(
                    &mut result,
                    &current,
                    subsequent_indent,
                    drop_whitespace,
                );
            }
        }
        result
    }
    pub(super) fn sifr_generated_apply_sentence_endings_line(text: &str) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < sifr_generated_chars_text.len() {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = &i;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_text.len());
                sifr_generated_chars_text
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                result.push_str(ch.as_str());
                if ch == "." || ch == "!" || ch == "?" {
                    let next_opt: Option<String> =
                        if ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                            < sifr_generated_chars_text.len()
                            && let Some(_checked_value_4) = {
                                let sifr_generated_string_index =
                                    ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                                let sifr_generated_string_index_normalized =
                                    sifr_generated_string_index
                                        .normalize_index_or_len(sifr_generated_chars_text.len());
                                sifr_generated_chars_text
                                    .get(sifr_generated_string_index_normalized)
                                    .copied()
                            }
                            .map(|character| character.to_string())
                        {
                            {
                                let sifr_generated_string_index =
                                    ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                                let sifr_generated_string_index_normalized =
                                    sifr_generated_string_index
                                        .normalize_index_or_len(sifr_generated_chars_text.len());
                                sifr_generated_chars_text
                                    .get(sifr_generated_string_index_normalized)
                                    .copied()
                            }
                            .map(|character| character.to_string())
                        } else {
                            None
                        };
                    let next2_opt_value_88541be202984f38: Option<String> =
                        if ::std::ops::Add::add(&i, &SifrInt::from_i64(2))
                            < sifr_generated_chars_text.len()
                        {
                            {
                                let sifr_generated_string_index =
                                    ::std::ops::Add::add(&i, &SifrInt::from_i64(2));
                                let sifr_generated_string_index_normalized =
                                    sifr_generated_string_index
                                        .normalize_index_or_len(sifr_generated_chars_text.len());
                                sifr_generated_chars_text
                                    .get(sifr_generated_string_index_normalized)
                                    .copied()
                            }
                            .map(|character| character.to_string())
                        } else {
                            None
                        };
                    if next_opt.is_some()
                        && next_opt == Some(" ".to_string())
                        && (next2_opt_value_88541be202984f38.is_none()
                            || next2_opt_value_88541be202984f38 != Some(" ".to_string()))
                    {
                        result.push(' ');
                    }
                }
            }
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn sifr_generated_apply_sentence_endings_lines(lines: &[String]) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for line in lines.iter() {
            result.push(sifr_generated_apply_sentence_endings_line(line));
        }
        result
    }
    pub(super) fn sifr_generated_clone_lines(lines: &[String]) -> Vec<String> {
        let mut copied: Vec<String> = Vec::new();
        for line in lines.iter().cloned() {
            copied.push(line);
        }
        copied
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_apply_max_lines(
        lines: &[String],
        width: &SifrInt,
        max_lines: Option<&SifrInt>,
        placeholder: &str,
        drop_whitespace: bool,
    ) -> Vec<String> {
        let max_lines: Option<SifrInt> = max_lines.cloned();
        let Some(max_lines_value_441854f90b4986e9) = max_lines else {
            return sifr_generated_clone_lines(lines);
        };
        let limit: SifrInt = max_lines_value_441854f90b4986e9;
        if limit <= SifrInt::from_i64(0) {
            return Vec::new();
        }
        if lines.len() <= limit {
            return sifr_generated_clone_lines(lines);
        }
        let mut result: Vec<String> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < limit {
            let line_opt: Option<String> = {
                let sifr_generated_checked_read_collection = &lines;
                let sifr_generated_checked_read_index = &i;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(line_opt) = line_opt {
                result.push(line_opt);
            }
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        if result.len() == SifrInt::from_i64(0) {
            return result;
        }
        let mut effective_placeholder: String = placeholder.to_string();
        let mut sifr_generated_chars_effective_placeholder: Vec<char> =
            effective_placeholder.chars().collect::<Vec<char>>();
        if width > &SifrInt::from_i64(0)
            && &SifrInt::from(sifr_generated_chars_effective_placeholder.len()) > width
        {
            effective_placeholder = {
                let sifr_generated_slice_src = &sifr_generated_chars_effective_placeholder;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(0).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = width.clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            };
            sifr_generated_chars_effective_placeholder =
                effective_placeholder.chars().collect::<Vec<char>>();
        }
        let last_index: SifrInt =
            ::std::ops::Sub::sub(&SifrInt::from(result.len()), &SifrInt::from_i64(1));
        let last_opt: Option<String> = {
            let sifr_generated_checked_read_collection = &result;
            let sifr_generated_checked_read_index = &last_index;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if let Some(last_opt) = last_opt {
            let last: String = last_opt;
            let mut base: String = sifr_generated_trim_line(&last);
            let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
            let mut available: SifrInt = ::std::ops::Sub::sub(
                width,
                &SifrInt::from(sifr_generated_chars_effective_placeholder.len()),
            );
            if available < SifrInt::from_i64(0) {
                available = SifrInt::from_i64(0);
            }
            if sifr_generated_chars_base.len() > available {
                base = sifr_generated_trim_line(&{
                    let sifr_generated_slice_src = &sifr_generated_chars_base;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        SifrInt::from_i64(0).clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop =
                        available.clamp_slice_bound(sifr_generated_slice_len);
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied()
                        .collect::<String>()
                });
            }
            if drop_whitespace {
                base = sifr_generated_trim_line(&base);
            }
            if SifrInt::from_i64(0) <= last_index && last_index < result.len() {
                {
                    let sifr_generated_assign_value = {
                        let mut sifr_generated_concat: String = String::with_capacity(
                            base.len().saturating_add(effective_placeholder.len()),
                        );
                        sifr_generated_concat.push_str(base.as_str());
                        sifr_generated_concat.push_str(effective_placeholder.as_str());
                        sifr_generated_concat
                    };
                    {
                        let sifr_generated_index_raw = &last_index;
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(result.len());
                        if let Some(sifr_generated_elem) =
                            result.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            }
        }
        result
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::{
        sifr_generated_apply_max_lines, sifr_generated_apply_sentence_endings_lines,
        sifr_generated_prepare_text, sifr_generated_wrap_with_indents,
    };
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub struct SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper {
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
    impl SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        #[expect(
            clippy::fn_params_excessive_bools,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
        )]
        pub fn new(
            width: &SifrInt,
            initial_indent: String,
            subsequent_indent: String,
            expand_tabs: bool,
            tabsize: &SifrInt,
            replace_whitespace: bool,
            drop_whitespace: bool,
            break_on_hyphens: bool,
            fix_sentence_endings: bool,
            max_lines: Option<&SifrInt>,
            placeholder: String,
        ) -> Self {
            let max_lines: Option<SifrInt> = max_lines.cloned();
            let sifr_generated_field_value_dbdacd932fd1e9bf_7769647468: SifrInt = (*width).clone();
            let sifr_generated_field_value_f1d9debc65d6e532_696e697469616c5f696e64656e74: String =
                initial_indent;
            let sifr_generated_field_value_45b636e6527b24bb_73756273657175656e745f696e64656e74: String = subsequent_indent;
            let sifr_generated_field_value_9fdde0a58b2f170e_657870616e645f74616273: bool =
                expand_tabs;
            let mut safe_tabsize: SifrInt = (*tabsize).clone();
            if safe_tabsize <= SifrInt::from_i64(0) {
                safe_tabsize = SifrInt::from_i64(1);
            }
            let sifr_generated_field_value_0f728cbe37fa9025_74616273697a65: SifrInt = safe_tabsize;
            let sifr_generated_field_value_d659e98074e25261_7265706c6163655f77686974657370616365: bool = replace_whitespace;
            let sifr_generated_field_value_a317a122f9288b94_64726f705f77686974657370616365: bool =
                drop_whitespace;
            let sifr_generated_field_value_acdab20e5253523e_627265616b5f6f6e5f68797068656e73: bool =
                break_on_hyphens;
            let sifr_generated_field_value_116e01dc088ea88b_6669785f73656e74656e63655f656e64696e6773: bool = fix_sentence_endings;
            let sifr_generated_field_value_441854f90b4986e9_6d61785f6c696e6573: Option<SifrInt> =
                max_lines;
            let sifr_generated_field_value_615e79d982d9f0fa_706c616365686f6c646572: String =
                placeholder;
            Self {
                width: sifr_generated_field_value_dbdacd932fd1e9bf_7769647468,
                initial_indent: sifr_generated_field_value_f1d9debc65d6e532_696e697469616c5f696e64656e74,
                subsequent_indent: sifr_generated_field_value_45b636e6527b24bb_73756273657175656e745f696e64656e74,
                expand_tabs: sifr_generated_field_value_9fdde0a58b2f170e_657870616e645f74616273,
                tabsize: sifr_generated_field_value_0f728cbe37fa9025_74616273697a65,
                replace_whitespace: sifr_generated_field_value_d659e98074e25261_7265706c6163655f77686974657370616365,
                drop_whitespace: sifr_generated_field_value_a317a122f9288b94_64726f705f77686974657370616365,
                break_on_hyphens: sifr_generated_field_value_acdab20e5253523e_627265616b5f6f6e5f68797068656e73,
                fix_sentence_endings: sifr_generated_field_value_116e01dc088ea88b_6669785f73656e74656e63655f656e64696e6773,
                max_lines: sifr_generated_field_value_441854f90b4986e9_6d61785f6c696e6573,
                placeholder: sifr_generated_field_value_615e79d982d9f0fa_706c616365686f6c646572,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper {
        #[must_use]
        pub fn wrap(&self, text: &str) -> Vec<String> {
            if self.width <= SifrInt::from_i64(0) {
                return Vec::new();
            }
            let prepared: String = sifr_generated_prepare_text(
                text,
                self.expand_tabs,
                &self.tabsize,
                self.replace_whitespace,
            );
            let mut lines: Vec<String> = sifr_generated_wrap_with_indents(
                &prepared,
                &self.width,
                &self.initial_indent,
                &self.subsequent_indent,
                self.break_on_hyphens,
                self.drop_whitespace,
            );
            if self.fix_sentence_endings {
                lines = sifr_generated_apply_sentence_endings_lines(&lines);
            }
            sifr_generated_apply_max_lines(
                &lines,
                &self.width,
                self.max_lines.as_ref(),
                &self.placeholder,
                self.drop_whitespace,
            )
        }
    }
}
use crate::sifr_generated_generated_support::{escape, unescape};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper;
fn main() {
    let wrapper: SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper =
        SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper::new(
            &SifrInt::from_i64(8),
            String::new(),
            String::new(),
            true,
            &SifrInt::from_i64(8),
            true,
            true,
            false,
            false,
            None,
            " [...]".to_string(),
        );
    let lines: Vec<String> = wrapper.wrap("alpha-beta gamma");
    assert_eq!(format!("{lines:?}"), "[\"alpha-beta\", \"gamma\"]");
    let keep_ws: SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper =
        SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper::new(
            &SifrInt::from_i64(10),
            String::new(),
            String::new(),
            true,
            &SifrInt::from_i64(8),
            true,
            false,
            true,
            false,
            None,
            " [...]".to_string(),
        );
    assert_eq!(format!("{:?}", keep_ws.wrap("a  b")), "[\"a  b\"]");
    let text: String = "<a href=\"x\">\'ok\' & done</a>".to_string();
    let escaped_default: String = escape(&text, true);
    let escaped_no_quote: String = escape(&text, false);
    assert_eq!(
        escaped_default.as_str(),
        "&lt;a href=&quot;x&quot;&gt;&#x27;ok&#x27; &amp; done&lt;/a&gt;"
            .to_string()
            .as_str()
    );
    assert_eq!(
        escaped_no_quote.as_str(),
        "&lt;a href=\"x\"&gt;\'ok\' &amp; done&lt;/a&gt;"
            .to_string()
            .as_str()
    );
    assert_eq!(unescape(&escaped_default), text);
}

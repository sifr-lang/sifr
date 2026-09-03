// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ValueError;
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert_eq!(
            {
                let sifr_generated_condition_list = &actual;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn sifr_generated_replace_whitespace_chars(text: &str, replace_tabs: bool) -> String {
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
fn sifr_generated_expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
    let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: SifrInt = tabsize.clone();
    if &effective_tabsize <= &SifrInt::from_i64(0) {
        effective_tabsize = SifrInt::from_i64(1);
    }
    if &effective_tabsize == &SifrInt::from_i64(0) {
        return text.to_owned();
    }
    let mut result: String = String::new();
    let mut column: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
        let ch_opt: Option<String> = {
            let sifr_generated_string_index = i.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t" {
                let mut spaces: SifrInt =
                    &effective_tabsize - &column.floor_mod_known_nonzero(&effective_tabsize);
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
                let sifr_generated_shared_branch_condition = ch == "\n" || ch == "\r";
                result.push_str(ch.as_str());
                if sifr_generated_shared_branch_condition {
                    column = SifrInt::from_i64(0);
                } else {
                    column = &column + &SifrInt::from_i64(1);
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn sifr_generated_prepare_text(
    text: &str,
    expand_tabs: bool,
    tabsize: SifrInt,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut sifr_generated_concat: String = String::with_capacity(text.len());
        sifr_generated_concat.push_str(text);
        sifr_generated_concat.push_str("");
        sifr_generated_concat
    };
    if expand_tabs {
        prepared = sifr_generated_expand_tabs_impl(&prepared, tabsize.clone());
    }
    if replace_whitespace {
        prepared = sifr_generated_replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn sifr_generated_normalize_whitespace(text: &str) -> String {
    sifr_generated_prepare_text(text, true, SifrInt::from_i64(8), true)
}
fn sifr_generated_has_non_whitespace(text: &str) -> bool {
    let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
        let ch: Option<String> = {
            let sifr_generated_string_index = i.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(ch) = ch
            && ch != " "
            && ch != "\t"
            && ch != "\n"
            && ch != "\r"
            && ch != "\u{b}"
            && ch != "\u{c}"
        {
            return true;
        }
        i = &i + &SifrInt::from_i64(1);
    }
    false
}
fn sifr_generated_split_word_units(word: &str, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![{
            let mut sifr_generated_concat: String = String::with_capacity(word.len());
            sifr_generated_concat.push_str(word);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(::std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if &SifrInt::from(parts.len()) <= &SifrInt::from_i64(1) {
        return vec![{
            let mut sifr_generated_concat: String = String::with_capacity(word.len());
            sifr_generated_concat.push_str(word);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }];
    }
    let mut units: Vec<String> = Vec::new();
    let mut index: SifrInt = SifrInt::from_i64(0);
    for part in parts.iter().cloned() {
        let sifr_generated_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = &index == &(&SifrInt::from(parts.len()) - &SifrInt::from_i64(1));
        if is_last {
            if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(0) {
                units.push(part);
            }
        } else if &SifrInt::from(sifr_generated_chars_part.len()) == &SifrInt::from_i64(0) {
            units.push("-".to_string());
        } else {
            units.push(format!("{part}-"));
        }
        index = &index + &SifrInt::from_i64(1);
    }
    if &SifrInt::from(units.len()) == &SifrInt::from_i64(0) {
        units.push(word.to_string());
    }
    units
}
fn sifr_generated_trim_line(line: &str) -> String {
    let sifr_generated_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: SifrInt = SifrInt::from_i64(0);
    while &start < &SifrInt::from(sifr_generated_chars_line.len()) && {
        let sifr_generated_string_index = start.clone();
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
        sifr_generated_chars_line
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string())
    .is_some_and(|_checked_value_2| {
        ({
            let sifr_generated_string_index = start.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
            sifr_generated_chars_line
                .get(sifr_generated_string_index_normalized)
                .copied()
        } == Some(" ").and_then(|sifr_generated_cmp_s| {
            let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
            let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
            if sifr_generated_cmp_chars.next().is_some() {
                None
            } else {
                sifr_generated_cmp_first
            }
        }))
    }) {
        start = &start + &SifrInt::from_i64(1);
    }
    let mut end: SifrInt = SifrInt::from(sifr_generated_chars_line.len());
    while &end > &start && {
        let sifr_generated_string_index = &end - &SifrInt::from_i64(1);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
        sifr_generated_chars_line
            .get(sifr_generated_string_index_normalized)
            .copied()
    } == Some(" ").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) {
        end = &end - &SifrInt::from_i64(1);
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
fn sifr_generated_finalize_line(line: &str, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return sifr_generated_trim_line(line);
    }
    {
        let mut sifr_generated_concat: String = String::with_capacity(line.len());
        sifr_generated_concat.push_str(line);
        sifr_generated_concat.push_str("");
        sifr_generated_concat
    }
}
fn sifr_generated_wrap_impl(text: &str, width: SifrInt) -> Vec<String> {
    let normalized: String = sifr_generated_normalize_whitespace(text);
    sifr_generated_wrap_with_indents(
        &normalized,
        width.clone(),
        &String::new(),
        &String::new(),
        true,
        true,
    )
}
fn sifr_generated_effective_content_width(total_width: SifrInt, indent: &str) -> SifrInt {
    let sifr_generated_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: SifrInt = &total_width - &SifrInt::from(sifr_generated_chars_indent.len());
    if &available <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    available.clone()
}
fn sifr_generated_push_current_line(
    result: &mut Vec<String>,
    line: &str,
    indent: &str,
    drop_whitespace: bool,
) {
    let candidate: String =
        sifr_generated_finalize_line(&format!("{indent}{line}"), drop_whitespace);
    let sifr_generated_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if &SifrInt::from(sifr_generated_chars_candidate.len()) > &SifrInt::from_i64(0) {
            result.push(candidate);
        }
    } else {
        result.push(candidate);
    }
}
fn sifr_generated_wrap_with_indents(
    text: &str,
    total_width: SifrInt,
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
    let mut sifr_generated_chars_current: Vec<char> = current.chars().collect::<Vec<char>>();
    let mut first_line: bool = true;
    let mut current_limit: SifrInt =
        sifr_generated_effective_content_width(total_width.clone(), initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = sifr_generated_split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let sifr_generated_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if &SifrInt::from(sifr_generated_chars_word.len()) == &SifrInt::from_i64(0) {
                if drop_whitespace {
                    continue;
                }
                if &SifrInt::from(sifr_generated_chars_current.len()) > &SifrInt::from_i64(0)
                    && &(&SifrInt::from(sifr_generated_chars_current.len()) + &SifrInt::from_i64(1))
                        <= &current_limit
                {
                    current.push(' ');
                    sifr_generated_chars_current.push(' ');
                }
                continue;
            }
            if &SifrInt::from(sifr_generated_chars_current.len()) == &SifrInt::from_i64(0) {
                current = word;
                sifr_generated_chars_current = current.chars().collect::<Vec<char>>();
            } else if &(&(&SifrInt::from(sifr_generated_chars_current.len())
                + &SifrInt::from_i64(1))
                + &SifrInt::from(sifr_generated_chars_word.len()))
                <= &current_limit
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
                    current_limit = sifr_generated_effective_content_width(
                        total_width.clone(),
                        subsequent_indent,
                    );
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
    if &SifrInt::from(sifr_generated_chars_current.len()) > &SifrInt::from_i64(0) {
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
fn wrap(text: &str, width: SifrInt) -> Result<Vec<String>, ValueError> {
    if &width <= &SifrInt::from_i64(0) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    Ok(sifr_generated_wrap_impl(text, width.clone()))
}
fn fill(text: &str, width: SifrInt) -> Result<String, ValueError> {
    if &width <= &SifrInt::from_i64(0) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = sifr_generated_wrap_impl(text, width.clone());
    let mut result: String = String::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    for line in lines.iter().cloned() {
        if &i > &SifrInt::from_i64(0) {
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
        .map(::std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let mut min_indent: SifrInt = SifrInt::from_i64(0);
    let mut have_indent: bool = false;
    for line in lines.iter().cloned() {
        let sifr_generated_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
        if sifr_generated_has_non_whitespace(&line) {
            let mut spaces: SifrInt = SifrInt::from_i64(0);
            let mut j: SifrInt = SifrInt::from_i64(0);
            let mut done: bool = false;
            while &j < &SifrInt::from(sifr_generated_chars_line.len()) {
                if !done {
                    let ch: Option<String> = {
                        let sifr_generated_string_index = j.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_line.len());
                        sifr_generated_chars_line
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(ch) = ch {
                        if ch == " " {
                            spaces = &spaces + &SifrInt::from_i64(1);
                        } else {
                            done = true;
                        }
                    }
                }
                j = &j + &SifrInt::from_i64(1);
            }
            if have_indent {
                if &spaces < &min_indent {
                    min_indent = spaces;
                }
            } else {
                min_indent = spaces;
                have_indent = true;
            }
        }
    }
    let mut result: String = String::new();
    let mut first: bool = true;
    for line2 in lines.iter().cloned() {
        let sifr_generated_chars_line2: Vec<char> = line2.chars().collect::<Vec<char>>();
        if !first {
            result.push('\n');
        }
        first = false;
        if have_indent {
            if &SifrInt::from(sifr_generated_chars_line2.len()) > &min_indent {
                result.push_str(
                    {
                        let sifr_generated_slice_src = &sifr_generated_chars_line2;
                        let sifr_generated_slice_len = sifr_generated_slice_src.len();
                        let sifr_generated_slice_start =
                            min_indent.clamp_slice_bound(sifr_generated_slice_len);
                        let sifr_generated_slice_stop = sifr_generated_slice_len;
                        String::from_iter(
                            sifr_generated_slice_src
                                .iter()
                                .skip(sifr_generated_slice_start)
                                .take(
                                    sifr_generated_slice_stop
                                        .saturating_sub(sifr_generated_slice_start),
                                )
                                .copied(),
                        )
                    }
                    .as_str(),
                );
            } else {
                result.push_str(line2.as_str());
            }
        } else {
            result.push_str(
                {
                    let sifr_generated_slice_src = &sifr_generated_chars_line2;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        min_indent.clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                }
                .as_str(),
            );
        }
    }
    result
}
fn indent(text: &str, prefix: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(::std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let mut result: String = String::new();
    let mut first: bool = true;
    for line in lines.iter().cloned() {
        if !first {
            result.push('\n');
        }
        first = false;
        {
            if sifr_generated_has_non_whitespace(&line) {
                result.push_str(prefix);
            }
            result.push_str(line.as_str());
        }
    }
    result
}
fn shorten(text: &str, width: SifrInt) -> String {
    let normalized: String = sifr_generated_normalize_whitespace(text);
    let words: Vec<String> = normalized
        .split(' ')
        .map(::std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let mut result: String = String::new();
    let mut sifr_generated_chars_result: Vec<char> = result.chars().collect::<Vec<char>>();
    for word in words.iter().cloned() {
        let sifr_generated_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_word.len()) == &SifrInt::from_i64(0) {
        } else if &SifrInt::from(sifr_generated_chars_result.len()) == &SifrInt::from_i64(0) {
            result = word;
            sifr_generated_chars_result = result.chars().collect::<Vec<char>>();
        } else if &(&(&(&SifrInt::from(sifr_generated_chars_result.len()) + &SifrInt::from_i64(1))
            + &SifrInt::from(sifr_generated_chars_word.len()))
            + &SifrInt::from_i64(4))
            <= &width
        {
            result.push(' ');
            sifr_generated_chars_result.push(' ');
            let sifr_generated_string_concat_result_1 = word;
            result.push_str(sifr_generated_string_concat_result_1.as_str());
            sifr_generated_chars_result
                .extend(sifr_generated_string_concat_result_1.as_str().chars());
        } else {
            return {
                let mut sifr_generated_concat: String =
                    String::with_capacity(result.len() + 6usize);
                sifr_generated_concat.push_str(result.as_str());
                sifr_generated_concat.push_str(" [...]");
                sifr_generated_concat
            };
        }
    }
    result
}
fn collect_wrap_fill_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let lines: Vec<String> = wrap(&"alpha\tbeta\ngamma".to_string(), SifrInt::from_i64(10))?;
        actual.push(
            format!("{lines:?}").as_str() == "[\"alpha beta\", \"gamma\"]".to_string().as_str(),
        );
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        actual.push(false);
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let filled: String = fill(&"alpha\tbeta\ngamma".to_string(), SifrInt::from_i64(10))?;
        actual.push(filled == "alpha beta\ngamma");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        actual.push(false);
    }
    actual
}
fn collect_other_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![
        dedent(&"  x\n  y".to_string()).as_str() == "x\ny".to_string().as_str(),
        indent(&"x\n \ny".to_string(), &">> ".to_string()).as_str()
            == ">> x\n \n>> y".to_string().as_str(),
        shorten(&"alpha beta gamma".to_string(), SifrInt::from_i64(16)).as_str()
            == "alpha beta [...]".to_string().as_str(),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let wrap_empty: Vec<String> = wrap(&String::new(), SifrInt::from_i64(5))?;
        actual.push(format!("{wrap_empty:?}").as_str() == "[]".to_string().as_str());
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
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
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_wrap_fill_actual());
    append_all(&mut actual, &collect_other_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("textwrap textwrap parity demo: pass");
}

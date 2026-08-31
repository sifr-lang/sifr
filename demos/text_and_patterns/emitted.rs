// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2estring_x2eTemplate {
        pub template: String,
    }
    impl __SifrStdlib_sifr_x2estring_x2eTemplate {
        pub fn new(template: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    template.len() + 0usize,
                );
                __sifr_concat.push_str((template).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            Self {
                template: __sifr_field_init_0,
            }
        }
    }
    impl __SifrStdlib_sifr_x2estring_x2eTemplate {
        pub fn substitute(
            &self,
            mapping: &HashMap<String, String>,
        ) -> Result<String, ValueError> {
            _template_substitute_impl(&self.template, mapping, false)
        }
    }
    impl __SifrStdlib_sifr_x2estring_x2eTemplate {
        pub fn safe_substitute(&self, mapping: &HashMap<String, String>) -> String {
            let __sifr_try_res: Result<String, ValueError> = (|| {
                let value: String = _template_substitute_impl(
                    &self.template,
                    mapping,
                    true,
                )?;
                Ok(value)
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    let _ = e.message.clone();
                    return {
                        let mut __sifr_concat: String = String::with_capacity(
                            0usize + 0usize,
                        );
                        __sifr_concat.push_str((self.template.clone()).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    };
                }
            }
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2estring_x2eTemplate {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Template(template={})", self.template)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2estring_x2eFormatter {}
    impl __SifrStdlib_sifr_x2estring_x2eFormatter {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2estring_x2eFormatter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2estring_x2eFormatter {
        pub fn format(
            &self,
            format_string: &String,
            values: &HashMap<String, String>,
        ) -> Result<String, ValueError> {
            _formatter_format_impl(format_string, values)
        }
    }
    pub fn _is_identifier_start(ch: &String) -> bool {
        (((ch).as_str() == "_") || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())))
    }
    pub fn _is_identifier_continue(ch: &String) -> bool {
        ((((ch).as_str() == "_")
            || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())))
            || (!ch.is_empty() && ch.chars().all(|c| c.is_ascii_digit())))
    }
    pub fn _mapping_lookup(
        mapping: &HashMap<String, String>,
        key: &String,
    ) -> Option<String> {
        for (current_key, current_value) in mapping
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if current_key == *key {
                return Some({
                    let mut __sifr_concat: String = String::with_capacity(
                        current_value.len() + 0usize,
                    );
                    __sifr_concat.push_str((current_value).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                });
            }
        }
        None
    }
    pub fn _template_substitute_impl(
        template: &String,
        mapping: &HashMap<String, String>,
        safe: bool,
    ) -> Result<String, ValueError> {
        let __sifr_chars_template: Vec<char> = template.chars().collect::<Vec<char>>();
        let mut result: String = "".to_string();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from(__sifr_chars_template.len())) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_template.len());
                __sifr_chars_template.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let Some(ch) = ch else {
                i = &i + &SifrInt::from_i64(1);
                continue;
            };
            let mut ch_value: String = "".to_string();
            {
                ch_value = ch;
            }
            if (ch_value != "$") {
                result.push_str((ch_value).as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (&(&i + &SifrInt::from_i64(1)) >= &SifrInt::from(__sifr_chars_template.len()))
            {
                if safe {
                    result.push('$');
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder at end of string".to_string(),
                    ),
                );
            }
            let next_ch: Option<String> = ({
                let __sifr_string_index = &i + &SifrInt::from_i64(1);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_template.len());
                __sifr_chars_template.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let mut next_value: String = "".to_string();
            if (next_ch.is_none()) {
                if safe {
                    result.push('$');
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(ValueError::new("invalid template placeholder".to_string()));
            } else {
                if let Some(next_ch) = next_ch {
                    next_value = next_ch;
                }
            }
            if (next_value == "$") {
                result.push('$');
                i = &i + &SifrInt::from_i64(2);
                continue;
            }
            if (next_value == "{") {
                let mut j: SifrInt = &i + &SifrInt::from_i64(2);
                let mut name: String = "".to_string();
                let mut __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
                while (&j < &SifrInt::from(__sifr_chars_template.len())) {
                    let part: Option<String> = ({
                        let __sifr_string_index = j.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_template.len());
                        __sifr_chars_template.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    let Some(part) = part else {
                        j = &j + &SifrInt::from_i64(1);
                        continue;
                    };
                    let mut part_value: String = "".to_string();
                    {
                        part_value = part;
                    }
                    if (part_value == "}") {
                        break;
                    }
                    let __sifr_string_concat_name_0 = part_value;
                    name.push_str((__sifr_string_concat_name_0).as_str());
                    __sifr_chars_name
                        .extend(((__sifr_string_concat_name_0).as_str()).chars());
                    j = &j + &SifrInt::from_i64(1);
                }
                if (&j >= &SifrInt::from(__sifr_chars_template.len())) {
                    if safe {
                        result
                            .push_str(
                                ({
                                    let _slice_src = &__sifr_chars_template;
                                    let _slice_len = _slice_src.len();
                                    let _slice_start = i.clamp_slice_bound(_slice_len);
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
                        return Ok(result);
                    }
                    return Err(
                        ValueError::new(
                            "invalid template placeholder: missing closing brace".to_string(),
                        ),
                    );
                }
                if (&SifrInt::from(__sifr_chars_name.len()) == &SifrInt::from_i64(0)) {
                    if safe {
                        result.push_str("${}");
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(
                        ValueError::new(
                            "invalid template placeholder: empty name".to_string(),
                        ),
                    );
                }
                let first_candidate: Option<String> = ({
                    let __sifr_string_index = SifrInt::from_i64(0);
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_name.len());
                    __sifr_chars_name.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let mut first_value: String = "".to_string();
                let mut has_first: bool = false;
                if let Some(first_candidate) = first_candidate {
                    has_first = true;
                    first_value = first_candidate;
                }
                if !has_first || !_is_identifier_start(&first_value) {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(
                        ValueError::new({
                            let mut __sifr_concat: String = String::with_capacity(
                                30usize + name.len(),
                            );
                            __sifr_concat.push_str("invalid template placeholder: ");
                            __sifr_concat.push_str((name).as_str());
                            __sifr_concat
                        }),
                    );
                }
                let mut valid: bool = true;
                let mut k: SifrInt = SifrInt::from_i64(1);
                while (&k < &SifrInt::from(__sifr_chars_name.len())) {
                    let part: Option<String> = ({
                        let __sifr_string_index = k.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_name.len());
                        __sifr_chars_name.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if let Some(part) = part {
                        if !_is_identifier_continue(&part) {
                            valid = false;
                            k = SifrInt::from(__sifr_chars_name.len());
                        }
                    }
                    k = &k + &SifrInt::from_i64(1);
                }
                if !valid {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(
                        ValueError::new({
                            let mut __sifr_concat: String = String::with_capacity(
                                30usize + name.len(),
                            );
                            __sifr_concat.push_str("invalid template placeholder: ");
                            __sifr_concat.push_str((name).as_str());
                            __sifr_concat
                        }),
                    );
                }
                let mapped_value: Option<String> = _mapping_lookup(mapping, &name);
                let mut mapped_value_text: String = "".to_string();
                if (mapped_value.is_none()) {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(
                        ValueError::new({
                            let mut __sifr_concat: String = String::with_capacity(
                                32usize + name.len(),
                            );
                            __sifr_concat.push_str("missing template value for key: ");
                            __sifr_concat.push_str((name).as_str());
                            __sifr_concat
                        }),
                    );
                } else {
                    if let Some(mapped_value) = mapped_value {
                        mapped_value_text = mapped_value;
                    }
                }
                result.push_str((mapped_value_text).as_str());
                i = &j + &SifrInt::from_i64(1);
                continue;
            }
            if !_is_identifier_start(&next_value) {
                if safe {
                    result.push('$');
                    result.push_str((next_value).as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            36usize + next_value.len(),
                        );
                        __sifr_concat.push_str("invalid template placeholder near: $");
                        __sifr_concat.push_str((next_value).as_str());
                        __sifr_concat
                    }),
                );
            }
            let mut name2: String = "".to_string();
            let mut j2: SifrInt = &i + &SifrInt::from_i64(1);
            while (&j2 < &SifrInt::from(__sifr_chars_template.len())) {
                let part2: Option<String> = ({
                    let __sifr_string_index = j2.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_template.len());
                    __sifr_chars_template.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let Some(part2) = part2 else {
                    j2 = &j2 + &SifrInt::from_i64(1);
                    continue;
                };
                let mut part2_value: String = "".to_string();
                {
                    part2_value = part2;
                }
                if !_is_identifier_continue(&part2_value) {
                    break;
                }
                name2.push_str((part2_value).as_str());
                j2 = &j2 + &SifrInt::from_i64(1);
            }
            let mapped_value2: Option<String> = _mapping_lookup(mapping, &name2);
            let mut mapped_value2_text: String = "".to_string();
            if (mapped_value2.is_none()) {
                if safe {
                    result.push('$');
                    result.push_str((name2).as_str());
                    i = j2.clone();
                    continue;
                }
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            32usize + name2.len(),
                        );
                        __sifr_concat.push_str("missing template value for key: ");
                        __sifr_concat.push_str((name2).as_str());
                        __sifr_concat
                    }),
                );
            } else {
                if let Some(mapped_value2) = mapped_value2 {
                    mapped_value2_text = mapped_value2;
                }
            }
            result.push_str((mapped_value2_text).as_str());
            i = j2.clone();
        }
        Ok(result)
    }
    pub fn _formatter_format_impl(
        format_string: &String,
        values: &HashMap<String, String>,
    ) -> Result<String, ValueError> {
        let __sifr_chars_format_string: Vec<char> = format_string
            .chars()
            .collect::<Vec<char>>();
        let mut result: String = "".to_string();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from(__sifr_chars_format_string.len())) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_format_string.len());
                __sifr_chars_format_string.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let Some(ch) = ch else {
                i = &i + &SifrInt::from_i64(1);
                continue;
            };
            let mut ch_value: String = "".to_string();
            {
                ch_value = ch;
            }
            if (ch_value == "{") {
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_format_string.len()))
                {
                    let escaped_next: Option<String> = ({
                        let __sifr_string_index = &i + &SifrInt::from_i64(1);
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_format_string.len());
                        __sifr_chars_format_string.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if (escaped_next.is_some()) && (escaped_next == Some("{".to_string())) {
                        result.push('{');
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                }
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                let mut field_name: String = "".to_string();
                let mut __sifr_chars_field_name: Vec<char> = field_name
                    .chars()
                    .collect::<Vec<char>>();
                while (&j < &SifrInt::from(__sifr_chars_format_string.len())) {
                    let part: Option<String> = ({
                        let __sifr_string_index = j.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_format_string.len());
                        __sifr_chars_format_string.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    let Some(part) = part else {
                        j = &j + &SifrInt::from_i64(1);
                        continue;
                    };
                    let mut part_value: String = "".to_string();
                    {
                        part_value = part;
                    }
                    if (part_value == "}") {
                        break;
                    }
                    let __sifr_string_concat_field_name_0 = part_value;
                    field_name.push_str((__sifr_string_concat_field_name_0).as_str());
                    __sifr_chars_field_name
                        .extend(((__sifr_string_concat_field_name_0).as_str()).chars());
                    j = &j + &SifrInt::from_i64(1);
                }
                if (&j >= &SifrInt::from(__sifr_chars_format_string.len())) {
                    return Err(
                        ValueError::new("formatter: missing closing brace".to_string()),
                    );
                }
                if (&SifrInt::from(__sifr_chars_field_name.len()) == &SifrInt::from_i64(0)) {
                    return Err(
                        ValueError::new(
                            "formatter: empty replacement field is not supported".to_string(),
                        ),
                    );
                }
                let value: Option<String> = _mapping_lookup(values, &field_name);
                let Some(value) = value else {
                    return Err(
                        ValueError::new({
                            let mut __sifr_concat: String = String::with_capacity(
                                34usize + field_name.len(),
                            );
                            __sifr_concat.push_str("formatter: missing value for key: ");
                            __sifr_concat.push_str((field_name).as_str());
                            __sifr_concat
                        }),
                    );
                };
                result.push_str((value).as_str());
                i = &j + &SifrInt::from_i64(1);
                continue;
            }
            if (ch_value == "}") {
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_format_string.len()))
                {
                    let escaped_next2: Option<String> = ({
                        let __sifr_string_index = &i + &SifrInt::from_i64(1);
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(__sifr_chars_format_string.len());
                        __sifr_chars_format_string.get(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if (escaped_next2.is_some()) && (escaped_next2 == Some("}".to_string()))
                    {
                        result.push('}');
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                }
                return Err(
                    ValueError::new("formatter: single \'}\' is invalid".to_string()),
                );
            }
            result.push_str((ch_value).as_str());
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result)
    }
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
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
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        pub fn new(message: String) -> Self {
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
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ValueError> for Error {
        fn from(err: ValueError) -> Self {
            Self::new(err.message)
        }
    }
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
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2estring_x2eFormatter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2estring_x2eTemplate;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etextwrap_x2eTextWrapper;

mod __sifr_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            crate::__sifr_project_nominals::FloatOverflowError,
        ),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            crate::__sifr_project_nominals::FloatPrecisionLossError,
        ),
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
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        __SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(crate::__sifr_project_nominals::Error),
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            crate::__sifr_project_nominals::FloatOverflowError,
        ),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            crate::__sifr_project_nominals::FloatPrecisionLossError,
        ),
    }
    impl From<crate::__sifr_project_nominals::Error>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::Error) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::FloatPrecisionLossError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatPrecisionLossError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0;
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn b64encode(s: &String) -> String {
    base64_encode(s)
}
fn b64decode(s: &String) -> Result<String, ParseError> {
    base64_decode(s)
}
fn calendar_isleap(year: SifrInt) -> bool {
    ::sifr_stdlib::calendar::calendar_isleap(
        ::sifr_runtime::interop::SifrIntBridge::from(year),
    )
}
fn calendar_weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    ::sifr_stdlib::calendar::calendar_weekday(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
            ::sifr_runtime::interop::SifrIntBridge::from(day),
        )
        .into_sifr_int()
}
fn calendar_monthrange(year: SifrInt, month: SifrInt) -> Vec<SifrInt> {
    ::sifr_stdlib::calendar::calendar_monthrange(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn __const_month_name() -> Vec<String> {
    vec![
        "".to_string(), "January".to_string(), "February".to_string(), "March"
        .to_string(), "April".to_string(), "May".to_string(), "June".to_string(), "July"
        .to_string(), "August".to_string(), "September".to_string(), "October"
        .to_string(), "November".to_string(), "December".to_string()
    ]
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    firstweekday: SifrInt,
}
impl __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn new(firstweekday: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = _normalize_firstweekday(
            (firstweekday).clone(),
        );
        Self {
            firstweekday: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn formatmonthname(
        &self,
        year: &SifrInt,
        month: &SifrInt,
        width: &SifrInt,
    ) -> Result<String, ValueError> {
        let name_lookup: Option<String> = _month_name_lookup((month.clone()).clone());
        let mut name: String = "".to_string();
        if let Some(name_lookup) = name_lookup {
            name = name_lookup;
        } else {
            return Err(ValueError::new("calendar: month must be in 1..12".to_string()));
        }
        let formatted: String = {
            let mut __sifr_concat: String = String::with_capacity(
                (name.len() + 1usize) + 0usize,
            );
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push(' ');
            __sifr_concat.push_str((format!("{}", year)).as_str());
            __sifr_concat
        };
        if (&width.clone() <= &SifrInt::from_i64(0)) {
            return Ok(formatted);
        }
        if (&SifrInt::from(formatted.chars().count()) >= &width.clone()) {
            return Ok(formatted);
        }
        let pad: SifrInt = width - &SifrInt::from(formatted.chars().count());
        let mut left: SifrInt = pad.floor_div_known_nonzero(&SifrInt::from_i64(2));
        let mut right: SifrInt = &pad - &left;
        let mut result: String = "".to_string();
        while (&left > &SifrInt::from_i64(0)) {
            result.push(' ');
            left = &left - &SifrInt::from_i64(1);
        }
        result.push_str((formatted).as_str());
        while (&right > &SifrInt::from_i64(0)) {
            result.push(' ');
            right = &right - &SifrInt::from_i64(1);
        }
        Ok(result)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextCalendar(firstweekday={})", self.firstweekday)
    }
}
fn _normalize_firstweekday(firstweekday: SifrInt) -> SifrInt {
    let mut value: SifrInt = firstweekday.floor_mod_known_nonzero(&SifrInt::from_i64(7));
    if &value < &SifrInt::from_i64(0) {
        value = &value + &SifrInt::from_i64(7);
    }
    value.clone()
}
fn _month_name_lookup(month: SifrInt) -> Option<String> {
    if (&month < &SifrInt::from_i64(1)) || (&month > &SifrInt::from_i64(12)) {
        return None;
    }
    {
        let __sifr_checked_read_collection = &__const_month_name();
        let __sifr_checked_read_index = month.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    _a: String,
    _b: String,
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn new(a: String, b: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(a.len() + 0usize);
            __sifr_concat.push_str((a).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: String = {
            let mut __sifr_concat: String = String::with_capacity(b.len() + 0usize);
            __sifr_concat.push_str((b).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self {
            _a: __sifr_field_init_0,
            _b: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn set_seq1(&mut self, a: &String) {
        self._a = {
            let mut __sifr_concat: String = String::with_capacity(a.len() + 0usize);
            __sifr_concat.push_str((a).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn set_seq2(&mut self, b: &String) {
        self._b = {
            let mut __sifr_concat: String = String::with_capacity(b.len() + 0usize);
            __sifr_concat.push_str((b).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn set_seqs(&mut self, a: &String, b: &String) {
        self._a = {
            let mut __sifr_concat: String = String::with_capacity(a.len() + 0usize);
            __sifr_concat.push_str((a).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        self._b = {
            let mut __sifr_concat: String = String::with_capacity(b.len() + 0usize);
            __sifr_concat.push_str((b).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn ratio(
        &self,
    ) -> Result<
        f64,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > {
        _similarity(&self._a, &self._b)
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn get_matching_blocks(&self) -> Vec<(SifrInt, SifrInt, SifrInt)> {
        _matching_blocks(&self._a, &self._b)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "SequenceMatcher(_a={}, _b={})", self._a, self._b)
    }
}
fn _similarity(
    a: &String,
    b: &String,
) -> Result<
    f64,
    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
> {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let total: SifrInt = &SifrInt::from(__sifr_chars_a.len())
        + &SifrInt::from(__sifr_chars_b.len());
    if &total == &SifrInt::from_i64(0) {
        return Ok(1.0_f64);
    }
    let mut matches: SifrInt = SifrInt::from_i64(0);
    let blocks: Vec<(SifrInt, SifrInt, SifrInt)> = _matching_blocks(a, b);
    for block in blocks.iter().cloned() {
        let (_, _, block_size) = block;
        matches = &matches + &block_size;
    }
    let __sifr_try_res: Result<
        Result<
            f64,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
        >,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let numerator: f64 = (&SifrInt::from_i64(2) * &matches)
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
        let denominator: f64 = total
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
        Ok(Ok(numerator / denominator))
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
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            FloatOverflowError::new(error.message.clone()),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            FloatPrecisionLossError::new(error.message.clone()),
                        ),
                    );
                }
            }
        }
    }
}
fn _longest_common_substring_range(
    a: &String,
    b: &String,
    a_start: SifrInt,
    a_end: SifrInt,
    b_start: SifrInt,
    b_end: SifrInt,
) -> (SifrInt, SifrInt, SifrInt) {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut best_i: SifrInt = SifrInt::from_i64(0);
    let mut best_j: SifrInt = SifrInt::from_i64(0);
    let mut best_len: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = a_start.clone();
    while (&i < &a_end) {
        let mut j: SifrInt = b_start.clone();
        while (&j < &b_end) {
            let mut k: SifrInt = SifrInt::from_i64(0);
            while (&(&i + &k) < &a_end) && (&(&j + &k) < &b_end) {
                let ai: Option<String> = ({
                    let __sifr_string_index = &i + &k;
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_a.len());
                    __sifr_chars_a.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let bj: Option<String> = ({
                    let __sifr_string_index = &j + &k;
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_b.len());
                    __sifr_chars_b.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let (Some(ai), Some(bj)) = (ai, bj) else {
                    k = &k + &SifrInt::from_i64(1);
                    continue;
                };
                if (ai != bj) {
                    break;
                }
                k = &k + &SifrInt::from_i64(1);
            }
            if (&k > &best_len) {
                best_len = k;
                best_i = i.clone();
                best_j = j.clone();
            }
            j = &j + &SifrInt::from_i64(1);
        }
        i = &i + &SifrInt::from_i64(1);
    }
    (best_i.clone(), best_j.clone(), best_len.clone())
}
fn _sort_blocks(
    blocks: &Vec<(SifrInt, SifrInt, SifrInt)>,
) -> Vec<(SifrInt, SifrInt, SifrInt)> {
    let mut sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = vec![];
    for block in blocks.iter().cloned() {
        let (bl_a, bl_b, _) = block.clone();
        let mut found_insert_at: bool = false;
        let mut insert_at: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        for existing in sorted_blocks.iter().cloned() {
            if !found_insert_at {
                let (ex_a, ex_b, _) = existing;
                let mut comes_before: bool = false;
                if (&bl_a < &ex_a) {
                    comes_before = true;
                } else {
                    if (&bl_a == &ex_a) {
                        if (&bl_b < &ex_b) {
                            comes_before = true;
                        }
                    }
                }
                if comes_before {
                    insert_at = i.clone();
                    found_insert_at = true;
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if found_insert_at {
            sorted_blocks
                .insert(::sifr_runtime::to_usize_proven(&insert_at), block.clone());
        } else {
            sorted_blocks.push(block.clone());
        }
    }
    sorted_blocks
}
fn _matching_blocks(a: &String, b: &String) -> Vec<(SifrInt, SifrInt, SifrInt)> {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut pending_a_start: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
    let mut pending_a_end: Vec<SifrInt> = vec![SifrInt::from(__sifr_chars_a.len())];
    let mut pending_b_start: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
    let mut pending_b_end: Vec<SifrInt> = vec![SifrInt::from(__sifr_chars_b.len())];
    let mut unsorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = vec![];
    while (&SifrInt::from(pending_a_start.len()) > &SifrInt::from_i64(0)) {
        let a_start_value: Option<SifrInt> = Some(
            pending_a_start.remove(pending_a_start.len() - (1_usize)),
        );
        let a_end_value: Option<SifrInt> = pending_a_end.pop();
        let b_start_value: Option<SifrInt> = pending_b_start.pop();
        let b_end_value: Option<SifrInt> = pending_b_end.pop();
        if let Some(a_start_value) = a_start_value.clone() {
            if let Some(a_end_value) = a_end_value.clone() {
                if let Some(b_start_value) = b_start_value.clone() {
                    if let Some(b_end_value) = b_end_value.clone() {
                        let (ai, bj, size) = _longest_common_substring_range(
                            a,
                            b,
                            (a_start_value).clone(),
                            (a_end_value).clone(),
                            (b_start_value).clone(),
                            (b_end_value).clone(),
                        );
                        if (&size == &SifrInt::from_i64(0)) {
                            continue;
                        }
                        unsorted_blocks.push((ai.clone(), bj.clone(), size.clone()));
                        let left_a_end: SifrInt = ai.clone();
                        let left_b_end: SifrInt = bj.clone();
                        if (&a_start_value < &left_a_end)
                            && (&b_start_value < &left_b_end)
                        {
                            pending_a_start.push(a_start_value.clone());
                            pending_a_end.push(left_a_end.clone());
                            pending_b_start.push(b_start_value.clone());
                            pending_b_end.push(left_b_end.clone());
                        }
                        let right_a_start: SifrInt = &ai + &size;
                        let right_b_start: SifrInt = &bj + &size;
                        if (&right_a_start < &a_end_value)
                            && (&right_b_start < &b_end_value)
                        {
                            pending_a_start.push(right_a_start.clone());
                            pending_a_end.push(a_end_value.clone());
                            pending_b_start.push(right_b_start.clone());
                            pending_b_end.push(b_end_value.clone());
                        }
                    }
                }
            }
        }
    }
    let sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = _sort_blocks(&unsorted_blocks);
    let mut merged_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = vec![];
    let mut have_previous: bool = false;
    let mut prev_a: SifrInt = SifrInt::from_i64(0);
    let mut prev_b: SifrInt = SifrInt::from_i64(0);
    let mut prev_size: SifrInt = SifrInt::from_i64(0);
    for block in sorted_blocks.iter().cloned() {
        let (bl_a, bl_b, bl_size) = block.clone();
        if !have_previous {
            prev_a = bl_a.clone();
            prev_b = bl_b.clone();
            prev_size = bl_size.clone();
            have_previous = true;
            continue;
        }
        if (&(&prev_a + &prev_size) == &bl_a) && (&(&prev_b + &prev_size) == &bl_b) {
            prev_size = &prev_size + &bl_size;
        } else {
            merged_blocks.push((prev_a.clone(), prev_b.clone(), prev_size.clone()));
            prev_a = bl_a.clone();
            prev_b = bl_b.clone();
            prev_size = bl_size.clone();
        }
    }
    if have_previous {
        merged_blocks.push((prev_a.clone(), prev_b.clone(), prev_size.clone()));
    }
    merged_blocks
        .push((
            SifrInt::from(a.chars().count()),
            SifrInt::from(b.chars().count()),
            SifrInt::from_i64(0),
        ));
    merged_blocks
}
fn fnmatch(name: &String, pattern: &String) -> bool {
    _match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
}
fn _match(name: &String, mut ni: SifrInt, pattern: &String, mut pi: SifrInt) -> bool {
    while (&pi < &SifrInt::from(pattern.chars().count())) {
        let pc: Option<String> = ({
            let __sifr_string_source = &pattern;
            let __sifr_string_index = pi.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(pc) = pc {
            if (pc == "*") {
                pi = &pi + &SifrInt::from_i64(1);
                if (&pi == &SifrInt::from(pattern.chars().count())) {
                    return true;
                }
                let mut j: SifrInt = ni.clone();
                while (&j <= &SifrInt::from(name.chars().count())) {
                    if _match(name, (j).clone(), pattern, (pi).clone()) {
                        return true;
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                return false;
            } else {
                if (pc == "?") {
                    if (&ni >= &SifrInt::from(name.chars().count())) {
                        return false;
                    }
                    ni = &ni + &SifrInt::from_i64(1);
                    pi = &pi + &SifrInt::from_i64(1);
                } else {
                    if (&ni >= &SifrInt::from(name.chars().count())) {
                        return false;
                    }
                    let nc: Option<String> = ({
                        let __sifr_string_source = &name;
                        let __sifr_string_index = ni.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(
                                __sifr_string_source.chars().count(),
                            );
                        __sifr_string_source.chars().nth(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if let Some(nc) = nc {
                        if (nc != pc) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = &ni + &SifrInt::from_i64(1);
                    pi = &pi + &SifrInt::from_i64(1);
                }
            }
        } else {
            return false;
        }
    }
    (&ni == &SifrInt::from(name.chars().count()))
}
fn _translate_literal(ch: &String) -> String {
    if (ch).as_str() == "." {
        return "\\.".to_string();
    }
    if (ch).as_str() == "^" {
        return "\\^".to_string();
    }
    if (ch).as_str() == "$" {
        return "\\$".to_string();
    }
    if (ch).as_str() == "+" {
        return "\\+".to_string();
    }
    if (ch).as_str() == "(" {
        return "\\(".to_string();
    }
    if (ch).as_str() == ")" {
        return "\\)".to_string();
    }
    if (ch).as_str() == "{" {
        return "\\{".to_string();
    }
    if (ch).as_str() == "}" {
        return "\\}".to_string();
    }
    if (ch).as_str() == "[" {
        return "\\[".to_string();
    }
    if (ch).as_str() == "]" {
        return "\\]".to_string();
    }
    if (ch).as_str() == "|" {
        return "\\|".to_string();
    }
    if (ch).as_str() == "\\" {
        return "\\\\".to_string();
    }
    {
        let mut __sifr_concat: String = String::with_capacity(ch.len() + 0usize);
        __sifr_concat.push_str((ch).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn translate(pattern: &String) -> String {
    let __sifr_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
    let mut body: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_pattern.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_pattern.len());
            __sifr_chars_pattern.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "*") {
                body.push_str(".*");
            } else {
                if (ch == "?") {
                    body.push('.');
                } else {
                    body.push_str((_translate_literal(&ch)).as_str());
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (4usize + body.len()) + 3usize,
        );
        __sifr_concat.push_str("(?s:");
        __sifr_concat.push_str((body).as_str());
        __sifr_concat.push_str(")\\z");
        __sifr_concat
    }
}
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
fn _is_identifier_start(ch: &String) -> bool {
    (((ch).as_str() == "_") || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())))
}
fn _is_identifier_continue(ch: &String) -> bool {
    ((((ch).as_str() == "_")
        || (!ch.is_empty() && ch.chars().all(|c| c.is_alphabetic())))
        || (!ch.is_empty() && ch.chars().all(|c| c.is_ascii_digit())))
}
fn _mapping_lookup(mapping: &HashMap<String, String>, key: &String) -> Option<String> {
    for (current_key, current_value) in mapping
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if current_key == *key {
            return Some({
                let mut __sifr_concat: String = String::with_capacity(
                    current_value.len() + 0usize,
                );
                __sifr_concat.push_str((current_value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            });
        }
    }
    None
}
fn _template_substitute_impl(
    template: &String,
    mapping: &HashMap<String, String>,
    safe: bool,
) -> Result<String, ValueError> {
    let __sifr_chars_template: Vec<char> = template.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_template.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_template.len());
            __sifr_chars_template.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let Some(ch) = ch else {
            i = &i + &SifrInt::from_i64(1);
            continue;
        };
        let mut ch_value: String = "".to_string();
        {
            ch_value = ch;
        }
        if (ch_value != "$") {
            result.push_str((ch_value).as_str());
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (&(&i + &SifrInt::from_i64(1)) >= &SifrInt::from(__sifr_chars_template.len()))
        {
            if safe {
                result.push('$');
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            return Err(
                ValueError::new(
                    "invalid template placeholder at end of string".to_string(),
                ),
            );
        }
        let next_ch: Option<String> = ({
            let __sifr_string_index = &i + &SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_template.len());
            __sifr_chars_template.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let mut next_value: String = "".to_string();
        if (next_ch.is_none()) {
            if safe {
                result.push('$');
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            return Err(ValueError::new("invalid template placeholder".to_string()));
        } else {
            if let Some(next_ch) = next_ch {
                next_value = next_ch;
            }
        }
        if (next_value == "$") {
            result.push('$');
            i = &i + &SifrInt::from_i64(2);
            continue;
        }
        if (next_value == "{") {
            let mut j: SifrInt = &i + &SifrInt::from_i64(2);
            let mut name: String = "".to_string();
            let mut __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
            while (&j < &SifrInt::from(__sifr_chars_template.len())) {
                let part: Option<String> = ({
                    let __sifr_string_index = j.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_template.len());
                    __sifr_chars_template.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let Some(part) = part else {
                    j = &j + &SifrInt::from_i64(1);
                    continue;
                };
                let mut part_value: String = "".to_string();
                {
                    part_value = part;
                }
                if (part_value == "}") {
                    break;
                }
                let __sifr_string_concat_name_0 = part_value;
                name.push_str((__sifr_string_concat_name_0).as_str());
                __sifr_chars_name
                    .extend(((__sifr_string_concat_name_0).as_str()).chars());
                j = &j + &SifrInt::from_i64(1);
            }
            if (&j >= &SifrInt::from(__sifr_chars_template.len())) {
                if safe {
                    result
                        .push_str(
                            ({
                                let _slice_src = &__sifr_chars_template;
                                let _slice_len = _slice_src.len();
                                let _slice_start = i.clamp_slice_bound(_slice_len);
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
                    return Ok(result);
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder: missing closing brace".to_string(),
                    ),
                );
            }
            if (&SifrInt::from(__sifr_chars_name.len()) == &SifrInt::from_i64(0)) {
                if safe {
                    result.push_str("${}");
                    i = &j + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder: empty name".to_string(),
                    ),
                );
            }
            let first_candidate: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(0);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_name.len());
                __sifr_chars_name.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let mut first_value: String = "".to_string();
            let mut has_first: bool = false;
            if let Some(first_candidate) = first_candidate {
                has_first = true;
                first_value = first_candidate;
            }
            if !has_first || !_is_identifier_start(&first_value) {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = &j + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            30usize + name.len(),
                        );
                        __sifr_concat.push_str("invalid template placeholder: ");
                        __sifr_concat.push_str((name).as_str());
                        __sifr_concat
                    }),
                );
            }
            let mut valid: bool = true;
            let mut k: SifrInt = SifrInt::from_i64(1);
            while (&k < &SifrInt::from(__sifr_chars_name.len())) {
                let part: Option<String> = ({
                    let __sifr_string_index = k.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_name.len());
                    __sifr_chars_name.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                if let Some(part) = part {
                    if !_is_identifier_continue(&part) {
                        valid = false;
                        k = SifrInt::from(__sifr_chars_name.len());
                    }
                }
                k = &k + &SifrInt::from_i64(1);
            }
            if !valid {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = &j + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            30usize + name.len(),
                        );
                        __sifr_concat.push_str("invalid template placeholder: ");
                        __sifr_concat.push_str((name).as_str());
                        __sifr_concat
                    }),
                );
            }
            let mapped_value: Option<String> = _mapping_lookup(mapping, &name);
            let mut mapped_value_text: String = "".to_string();
            if (mapped_value.is_none()) {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = &j + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            32usize + name.len(),
                        );
                        __sifr_concat.push_str("missing template value for key: ");
                        __sifr_concat.push_str((name).as_str());
                        __sifr_concat
                    }),
                );
            } else {
                if let Some(mapped_value) = mapped_value {
                    mapped_value_text = mapped_value;
                }
            }
            result.push_str((mapped_value_text).as_str());
            i = &j + &SifrInt::from_i64(1);
            continue;
        }
        if !_is_identifier_start(&next_value) {
            if safe {
                result.push('$');
                result.push_str((next_value).as_str());
                i = &i + &SifrInt::from_i64(2);
                continue;
            }
            return Err(
                ValueError::new({
                    let mut __sifr_concat: String = String::with_capacity(
                        36usize + next_value.len(),
                    );
                    __sifr_concat.push_str("invalid template placeholder near: $");
                    __sifr_concat.push_str((next_value).as_str());
                    __sifr_concat
                }),
            );
        }
        let mut name2: String = "".to_string();
        let mut j2: SifrInt = &i + &SifrInt::from_i64(1);
        while (&j2 < &SifrInt::from(__sifr_chars_template.len())) {
            let part2: Option<String> = ({
                let __sifr_string_index = j2.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_template.len());
                __sifr_chars_template.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let Some(part2) = part2 else {
                j2 = &j2 + &SifrInt::from_i64(1);
                continue;
            };
            let mut part2_value: String = "".to_string();
            {
                part2_value = part2;
            }
            if !_is_identifier_continue(&part2_value) {
                break;
            }
            name2.push_str((part2_value).as_str());
            j2 = &j2 + &SifrInt::from_i64(1);
        }
        let mapped_value2: Option<String> = _mapping_lookup(mapping, &name2);
        let mut mapped_value2_text: String = "".to_string();
        if (mapped_value2.is_none()) {
            if safe {
                result.push('$');
                result.push_str((name2).as_str());
                i = j2.clone();
                continue;
            }
            return Err(
                ValueError::new({
                    let mut __sifr_concat: String = String::with_capacity(
                        32usize + name2.len(),
                    );
                    __sifr_concat.push_str("missing template value for key: ");
                    __sifr_concat.push_str((name2).as_str());
                    __sifr_concat
                }),
            );
        } else {
            if let Some(mapped_value2) = mapped_value2 {
                mapped_value2_text = mapped_value2;
            }
        }
        result.push_str((mapped_value2_text).as_str());
        i = j2.clone();
    }
    Ok(result)
}
fn _formatter_format_impl(
    format_string: &String,
    values: &HashMap<String, String>,
) -> Result<String, ValueError> {
    let __sifr_chars_format_string: Vec<char> = format_string
        .chars()
        .collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_format_string.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_format_string.len());
            __sifr_chars_format_string.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let Some(ch) = ch else {
            i = &i + &SifrInt::from_i64(1);
            continue;
        };
        let mut ch_value: String = "".to_string();
        {
            ch_value = ch;
        }
        if (ch_value == "{") {
            if (&(&i + &SifrInt::from_i64(1))
                < &SifrInt::from(__sifr_chars_format_string.len()))
            {
                let escaped_next: Option<String> = ({
                    let __sifr_string_index = &i + &SifrInt::from_i64(1);
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_format_string.len());
                    __sifr_chars_format_string.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                if (escaped_next.is_some()) && (escaped_next == Some("{".to_string())) {
                    result.push('{');
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
            }
            let mut j: SifrInt = &i + &SifrInt::from_i64(1);
            let mut field_name: String = "".to_string();
            let mut __sifr_chars_field_name: Vec<char> = field_name
                .chars()
                .collect::<Vec<char>>();
            while (&j < &SifrInt::from(__sifr_chars_format_string.len())) {
                let part: Option<String> = ({
                    let __sifr_string_index = j.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_format_string.len());
                    __sifr_chars_format_string.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                let Some(part) = part else {
                    j = &j + &SifrInt::from_i64(1);
                    continue;
                };
                let mut part_value: String = "".to_string();
                {
                    part_value = part;
                }
                if (part_value == "}") {
                    break;
                }
                let __sifr_string_concat_field_name_0 = part_value;
                field_name.push_str((__sifr_string_concat_field_name_0).as_str());
                __sifr_chars_field_name
                    .extend(((__sifr_string_concat_field_name_0).as_str()).chars());
                j = &j + &SifrInt::from_i64(1);
            }
            if (&j >= &SifrInt::from(__sifr_chars_format_string.len())) {
                return Err(
                    ValueError::new("formatter: missing closing brace".to_string()),
                );
            }
            if (&SifrInt::from(__sifr_chars_field_name.len()) == &SifrInt::from_i64(0)) {
                return Err(
                    ValueError::new(
                        "formatter: empty replacement field is not supported".to_string(),
                    ),
                );
            }
            let value: Option<String> = _mapping_lookup(values, &field_name);
            let Some(value) = value else {
                return Err(
                    ValueError::new({
                        let mut __sifr_concat: String = String::with_capacity(
                            34usize + field_name.len(),
                        );
                        __sifr_concat.push_str("formatter: missing value for key: ");
                        __sifr_concat.push_str((field_name).as_str());
                        __sifr_concat
                    }),
                );
            };
            result.push_str((value).as_str());
            i = &j + &SifrInt::from_i64(1);
            continue;
        }
        if (ch_value == "}") {
            if (&(&i + &SifrInt::from_i64(1))
                < &SifrInt::from(__sifr_chars_format_string.len()))
            {
                let escaped_next2: Option<String> = ({
                    let __sifr_string_index = &i + &SifrInt::from_i64(1);
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_format_string.len());
                    __sifr_chars_format_string.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                if (escaped_next2.is_some()) && (escaped_next2 == Some("}".to_string()))
                {
                    result.push('}');
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
            }
            return Err(
                ValueError::new("formatter: single \'}\' is invalid".to_string()),
            );
        }
        result.push_str((ch_value).as_str());
        i = &i + &SifrInt::from_i64(1);
    }
    Ok(result)
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
fn main() {
    let template: __SifrStdlib_sifr_x2estring_x2eTemplate = __SifrStdlib_sifr_x2estring_x2eTemplate::new(
        "Hello $name, mode=${mode}".to_string(),
    );
    let mut rendered_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let rendered: String = template
            .substitute(
                &HashMap::from([
                    ("name".to_string(), "Sifr".to_string()),
                    ("mode".to_string(), "c2".to_string()),
                ]),
            )?;
        rendered_ok = (rendered == "Hello Sifr, mode=c2");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(rendered_ok);
    let formatter: __SifrStdlib_sifr_x2estring_x2eFormatter = __SifrStdlib_sifr_x2estring_x2eFormatter::new();
    let mut rendered_fmt_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let rendered_fmt: String = formatter
            .format(
                &"Status {label}: {status}".to_string(),
                &HashMap::from([
                    ("label".to_string(), "c2".to_string()),
                    ("status".to_string(), "ok".to_string()),
                ]),
            )?;
        rendered_fmt_ok = (rendered_fmt == "Status c2: ok");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(rendered_fmt_ok);
    let wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        SifrInt::from_i64(8),
        "> ".to_string(),
        ".. ".to_string(),
        true,
        SifrInt::from_i64(8),
        true,
        true,
        true,
        false,
        None,
        " [...]".to_string(),
    );
    let wrapped: Vec<String> = wrapper.wrap(&"alpha beta gamma".to_string());
    assert!((format!("{:?}", wrapped) == "[\"> alpha\", \".. beta\", \".. gamma\"]"));
    let encoded: String = b64encode(&"hello".to_string());
    let mut decoded_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(&encoded)?;
        decoded_ok = (decoded == "hello");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(decoded_ok);
    let escaped: String = escape(&"<b>safe</b>".to_string(), true);
    assert!((unescape(& escaped) == "<b>safe</b>"));
    assert!(fnmatch(& "report.txt".to_string(), & "*.txt".to_string()));
    assert!((translate(& "*.txt".to_string()) == "(?s:.*\\.txt)\\z"));
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0,
    > = (|| {
        let ratio: f64 = (__SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher::new(
                "abcd".to_string(),
                "abed".to_string(),
            )
            .ratio())
            .map_err(|__e| match __e {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    )
                }
            })?;
        assert!(ratio > (0.4_f64));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = __sifr_try_variant_error.clone();
                assert!(false);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = Error::new(__sifr_try_variant_error.clone().message);
                assert!(false);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a331_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = Error::new(__sifr_try_variant_error.clone().message);
                assert!(false);
            }
        }
    }
    let mut month_label_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let month_label: String = __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar::new(
                SifrInt::from_i64(0),
            )
            .formatmonthname(
                &SifrInt::from_i64(2024),
                &SifrInt::from_i64(2),
                &SifrInt::from_i64(0),
            )?;
        month_label_ok = (month_label == "February 2024");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(month_label_ok);
}

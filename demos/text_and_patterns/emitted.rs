// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
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
                return Ok(value);
                unreachable!("sifr try/except return capture fell through");
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
        let mut i: i64 = 0_i64;
        while (i < (__sifr_chars_template.len() as i64)) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_template
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            let Some(ch) = ch else {
                i += 1_i64;
                continue;
            };
            let mut ch_value: String = "".to_string();
            if true {
                ch_value = ch;
            }
            if ch_value != "$" {
                result.push_str((ch_value).as_str());
                i += 1_i64;
                continue;
            }
            if ((i + (1_i64)) >= (__sifr_chars_template.len() as i64)) {
                if safe {
                    result.push('$');
                    i += 1_i64;
                    continue;
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder at end of string".to_string(),
                    ),
                );
            }
            let next_ch: Option<String> = __sifr_chars_template
                .get((i + (1_i64)) as usize)
                .map(|c| c.to_string());
            let mut next_value: String = "".to_string();
            if next_ch.is_none() {
                if safe {
                    result.push('$');
                    i += 1_i64;
                    continue;
                }
                return Err(ValueError::new("invalid template placeholder".to_string()));
            } else {
                if let Some(next_ch) = next_ch {
                    next_value = next_ch;
                }
            }
            if next_value == "$" {
                result.push('$');
                i += 2_i64;
                continue;
            }
            if next_value == "{" {
                let mut j: i64 = i + (2_i64);
                let mut name: String = "".to_string();
                let mut __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
                while (j < (__sifr_chars_template.len() as i64)) {
                    let part: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_template
                            .get(j as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    let Some(part) = part else {
                        j += 1_i64;
                        continue;
                    };
                    let mut part_value: String = "".to_string();
                    if true {
                        part_value = part;
                    }
                    if part_value == "}" {
                        break;
                    }
                    let __sifr_string_concat_name_0 = part_value;
                    name.push_str((__sifr_string_concat_name_0).as_str());
                    __sifr_chars_name
                        .extend(((__sifr_string_concat_name_0).as_str()).chars());
                    j += 1_i64;
                }
                if (j >= (__sifr_chars_template.len() as i64)) {
                    if safe {
                        result
                            .push_str(
                                ({
                                    let _slice_src = &__sifr_chars_template;
                                    let _slice_len_i64 = _slice_src.len() as i64;
                                    let _slice_start_i64 = if i < 0 {
                                        (_slice_len_i64 + i).max(0)
                                    } else {
                                        i.min(_slice_len_i64)
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
                        return Ok(result);
                    }
                    return Err(
                        ValueError::new(
                            "invalid template placeholder: missing closing brace".to_string(),
                        ),
                    );
                }
                if ((__sifr_chars_name.len() as i64) == (0_i64)) {
                    if safe {
                        result.push_str("${}");
                        i = j + (1_i64);
                        continue;
                    }
                    return Err(
                        ValueError::new(
                            "invalid template placeholder: empty name".to_string(),
                        ),
                    );
                }
                let first_candidate: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_name
                        .get((0_i64) as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char
                });
                let mut first_value: String = "".to_string();
                let mut has_first: bool = false;
                if let Some(first_candidate) = first_candidate {
                    has_first = true;
                    first_value = first_candidate;
                }
                if !has_first || !(_is_identifier_start(&first_value)) {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = j + (1_i64);
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
                let mut k: i64 = 1_i64;
                while (k < (__sifr_chars_name.len() as i64)) {
                    let part: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_name
                            .get(k as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(part) = part {
                        if !(_is_identifier_continue(&part)) {
                            valid = false;
                            k = __sifr_chars_name.len() as i64;
                        }
                    }
                    k += 1_i64;
                }
                if !valid {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = j + (1_i64);
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
                if mapped_value.is_none() {
                    if safe {
                        result.push_str("${");
                        result.push_str((name).as_str());
                        result.push('}');
                        i = j + (1_i64);
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
                i = j + (1_i64);
                continue;
            }
            if !(_is_identifier_start(&next_value)) {
                if safe {
                    result.push('$');
                    result.push_str((next_value).as_str());
                    i += 2_i64;
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
            let mut j2: i64 = i + (1_i64);
            while (j2 < (__sifr_chars_template.len() as i64)) {
                let part2: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_template
                        .get(j2 as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!("compiler-verified string index should be in range");
                    };
                    __indexed_char
                });
                let Some(part2) = part2 else {
                    j2 += 1_i64;
                    continue;
                };
                let mut part2_value: String = "".to_string();
                if true {
                    part2_value = part2;
                }
                if !(_is_identifier_continue(&part2_value)) {
                    break;
                }
                name2.push_str((part2_value).as_str());
                j2 += 1_i64;
            }
            let mapped_value2: Option<String> = _mapping_lookup(mapping, &name2);
            let mut mapped_value2_text: String = "".to_string();
            if mapped_value2.is_none() {
                if safe {
                    result.push('$');
                    result.push_str((name2).as_str());
                    i = j2;
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
            i = j2;
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
        let mut i: i64 = 0_i64;
        while (i < (__sifr_chars_format_string.len() as i64)) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_format_string
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            let Some(ch) = ch else {
                i += 1_i64;
                continue;
            };
            let mut ch_value: String = "".to_string();
            if true {
                ch_value = ch;
            }
            if ch_value == "{" {
                if ((i + (1_i64)) < (__sifr_chars_format_string.len() as i64)) {
                    let escaped_next: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_format_string
                            .get((i + (1_i64)) as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if escaped_next.is_some() && (escaped_next == Some("{".to_string())) {
                        result.push('{');
                        i += 2_i64;
                        continue;
                    }
                }
                let mut j: i64 = i + (1_i64);
                let mut field_name: String = "".to_string();
                let mut __sifr_chars_field_name: Vec<char> = field_name
                    .chars()
                    .collect::<Vec<char>>();
                while (j < (__sifr_chars_format_string.len() as i64)) {
                    let part: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_format_string
                            .get(j as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    let Some(part) = part else {
                        j += 1_i64;
                        continue;
                    };
                    let mut part_value: String = "".to_string();
                    if true {
                        part_value = part;
                    }
                    if part_value == "}" {
                        break;
                    }
                    let __sifr_string_concat_field_name_0 = part_value;
                    field_name.push_str((__sifr_string_concat_field_name_0).as_str());
                    __sifr_chars_field_name
                        .extend(((__sifr_string_concat_field_name_0).as_str()).chars());
                    j += 1_i64;
                }
                if (j >= (__sifr_chars_format_string.len() as i64)) {
                    return Err(
                        ValueError::new("formatter: missing closing brace".to_string()),
                    );
                }
                if ((__sifr_chars_field_name.len() as i64) == (0_i64)) {
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
                i = j + (1_i64);
                continue;
            }
            if ch_value == "}" {
                if ((i + (1_i64)) < (__sifr_chars_format_string.len() as i64)) {
                    let escaped_next2: Option<String> = Some({
                        let Some(__indexed_char) = __sifr_chars_format_string
                            .get((i + (1_i64)) as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if escaped_next2.is_some() && (escaped_next2 == Some("}".to_string())) {
                        result.push('}');
                        i += 2_i64;
                        continue;
                    }
                }
                return Err(
                    ValueError::new("formatter: single \'}\' is invalid".to_string()),
                );
            }
            result.push_str((ch_value).as_str());
            i += 1_i64;
        }
        Ok(result)
    }
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
}
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2estring_x2eFormatter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2estring_x2eTemplate;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etextwrap_x2eTextWrapper;
use ::std::collections::HashMap;
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
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
    wrapcol: i64,
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
fn calendar_isleap(year: i64) -> bool {
    ::sifr_stdlib::calendar::calendar_isleap(
        ::sifr_runtime::interop::SifrIntBridge::from(year),
    )
}
fn calendar_weekday(year: i64, month: i64, day: i64) -> i64 {
    ::sifr_stdlib::calendar::calendar_weekday(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
            ::sifr_runtime::interop::SifrIntBridge::from(day),
        )
        .to_i64_saturating()
}
fn calendar_monthrange(year: i64, month: i64) -> Vec<i64> {
    ::sifr_stdlib::calendar::calendar_monthrange(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
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
    firstweekday: i64,
}
impl __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn new(firstweekday: i64) -> Self {
        let __sifr_field_init_0: i64 = _normalize_firstweekday(firstweekday);
        Self {
            firstweekday: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn formatmonthname(
        &self,
        year: i64,
        month: i64,
        width: i64,
    ) -> Result<String, ValueError> {
        let name_lookup: Option<String> = _month_name_lookup(month);
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
        if width <= (0_i64) {
            return Ok(formatted);
        }
        if ((formatted.chars().count() as i64) >= width) {
            return Ok(formatted);
        }
        let pad: i64 = width - (formatted.chars().count() as i64);
        let mut left: i64 = pad / (2_i64);
        let mut right: i64 = pad - left;
        let mut result: String = "".to_string();
        while left > (0_i64) {
            result.push(' ');
            left -= 1_i64;
        }
        result.push_str((formatted).as_str());
        while right > (0_i64) {
            result.push(' ');
            right -= 1_i64;
        }
        Ok(result)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextCalendar(firstweekday={})", self.firstweekday)
    }
}
fn _normalize_firstweekday(firstweekday: i64) -> i64 {
    let mut value: i64 = firstweekday % (7_i64);
    if value < (0_i64) {
        value += 7_i64;
    }
    value
}
fn _month_name_lookup(month: i64) -> Option<String> {
    if (month < (1_i64)) || (month > (12_i64)) {
        return None;
    }
    {
        let __sifr_index_list = &__const_month_name();
        let __sifr_index_i = month;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
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
    fn ratio(&self) -> f64 {
        _similarity(&self._a, &self._b)
    }
}
impl __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn get_matching_blocks(&self) -> Vec<(i64, i64, i64)> {
        _matching_blocks(&self._a, &self._b)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "SequenceMatcher(_a={}, _b={})", self._a, self._b)
    }
}
fn _similarity(a: &String, b: &String) -> f64 {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let total: i64 = (__sifr_chars_a.len() as i64) + (__sifr_chars_b.len() as i64);
    if total == (0_i64) {
        return 1.0_f64;
    }
    let mut matches: i64 = 0_i64;
    let blocks: Vec<(i64, i64, i64)> = _matching_blocks(a, b);
    for block in blocks.iter().copied() {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1, __sifr_tuple_unpack_2) = block;
        let _ = __sifr_tuple_unpack_0;
        _ = __sifr_tuple_unpack_1;
        let block_size = __sifr_tuple_unpack_2;
        matches += block_size;
    }
    (((2_i64) * matches) as f64) / (total as f64)
}
fn _longest_common_substring_range(
    a: &String,
    b: &String,
    a_start: i64,
    a_end: i64,
    b_start: i64,
    b_end: i64,
) -> (i64, i64, i64) {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut best_i: i64 = 0_i64;
    let mut best_j: i64 = 0_i64;
    let mut best_len: i64 = 0_i64;
    let mut i: i64 = a_start;
    while i < a_end {
        let mut j: i64 = b_start;
        while j < b_end {
            let mut k: i64 = 0_i64;
            while ((i + k) < a_end) && ((j + k) < b_end) {
                let ai: Option<String> = __sifr_chars_a
                    .get((i + k) as usize)
                    .map(|c| c.to_string());
                let bj: Option<String> = __sifr_chars_b
                    .get((j + k) as usize)
                    .map(|c| c.to_string());
                let (Some(ai), Some(bj)) = (ai, bj) else {
                    k += 1_i64;
                    continue;
                };
                if ai != bj {
                    break;
                }
                k += 1_i64;
            }
            if k > best_len {
                best_len = k;
                best_i = i;
                best_j = j;
            }
            j += 1_i64;
        }
        i += 1_i64;
    }
    (best_i, best_j, best_len)
}
fn _sort_blocks(blocks: &Vec<(i64, i64, i64)>) -> Vec<(i64, i64, i64)> {
    let mut sorted_blocks: Vec<(i64, i64, i64)> = vec![];
    for block in blocks.iter().copied() {
        let (bl_a, bl_b, _) = block;
        let mut found_insert_at: bool = false;
        let mut insert_at: i64 = 0_i64;
        let mut i: i64 = 0_i64;
        for existing in sorted_blocks.iter().copied() {
            if !found_insert_at {
                let (
                    __sifr_tuple_unpack_0,
                    __sifr_tuple_unpack_1,
                    __sifr_tuple_unpack_2,
                ) = existing;
                let ex_a = __sifr_tuple_unpack_0;
                let ex_b = __sifr_tuple_unpack_1;
                _ = __sifr_tuple_unpack_2;
                if (bl_a < ex_a) || ((bl_a == ex_a) && (bl_b < ex_b)) {
                    insert_at = i;
                    found_insert_at = true;
                }
            }
            i += 1_i64;
        }
        if found_insert_at {
            sorted_blocks.insert(insert_at as usize, block);
        } else {
            sorted_blocks.push(block);
        }
    }
    sorted_blocks
}
fn _matching_blocks(a: &String, b: &String) -> Vec<(i64, i64, i64)> {
    let __sifr_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let __sifr_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut pending_a_start: Vec<i64> = vec![0_i64];
    let mut pending_a_end: Vec<i64> = vec![__sifr_chars_a.len() as i64];
    let mut pending_b_start: Vec<i64> = vec![0_i64];
    let mut pending_b_end: Vec<i64> = vec![__sifr_chars_b.len() as i64];
    let mut unsorted_blocks: Vec<(i64, i64, i64)> = vec![];
    while ((pending_a_start.len() as i64) > (0_i64)) {
        let a_start_value: Option<i64> = Some({
            let Some(__sifr_nonempty_pop_value) = pending_a_start.pop() else {
                unreachable!("compiler-verified non-empty pop should return Some");
            };
            __sifr_nonempty_pop_value
        });
        let a_end_value: Option<i64> = pending_a_end.pop();
        let b_start_value: Option<i64> = pending_b_start.pop();
        let b_end_value: Option<i64> = pending_b_end.pop();
        if let Some(a_start_value) = a_start_value {
            if let Some(a_end_value) = a_end_value {
                if let Some(b_start_value) = b_start_value {
                    if let Some(b_end_value) = b_end_value {
                        let (ai, bj, size) = _longest_common_substring_range(
                            a,
                            b,
                            a_start_value,
                            a_end_value,
                            b_start_value,
                            b_end_value,
                        );
                        if size == (0_i64) {
                            continue;
                        }
                        unsorted_blocks.push((ai, bj, size));
                        let left_a_end: i64 = ai;
                        let left_b_end: i64 = bj;
                        if (a_start_value < left_a_end) && (b_start_value < left_b_end) {
                            pending_a_start.push(a_start_value);
                            pending_a_end.push(left_a_end);
                            pending_b_start.push(b_start_value);
                            pending_b_end.push(left_b_end);
                        }
                        let right_a_start: i64 = ai + size;
                        let right_b_start: i64 = bj + size;
                        if (right_a_start < a_end_value) && (right_b_start < b_end_value)
                        {
                            pending_a_start.push(right_a_start);
                            pending_a_end.push(a_end_value);
                            pending_b_start.push(right_b_start);
                            pending_b_end.push(b_end_value);
                        }
                    }
                }
            }
        }
    }
    let sorted_blocks: Vec<(i64, i64, i64)> = _sort_blocks(&unsorted_blocks);
    let mut merged_blocks: Vec<(i64, i64, i64)> = vec![];
    let mut have_previous: bool = false;
    let mut prev_a: i64 = 0_i64;
    let mut prev_b: i64 = 0_i64;
    let mut prev_size: i64 = 0_i64;
    for block in sorted_blocks.iter().copied() {
        let (bl_a, bl_b, bl_size) = block;
        if !have_previous {
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
            have_previous = true;
            continue;
        }
        if ((prev_a + prev_size) == bl_a) && ((prev_b + prev_size) == bl_b) {
            prev_size += bl_size;
        } else {
            merged_blocks.push((prev_a, prev_b, prev_size));
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
        }
    }
    if have_previous {
        merged_blocks.push((prev_a, prev_b, prev_size));
    }
    merged_blocks.push((a.chars().count() as i64, b.chars().count() as i64, 0_i64));
    merged_blocks
}
fn fnmatch(name: &String, pattern: &String) -> bool {
    _match(name, 0_i64, pattern, 0_i64)
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while (pi < (pattern.chars().count() as i64)) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern
                .chars()
                .nth(pi as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(pc) = pc {
            if pc == "*" {
                pi += 1_i64;
                if (pi == (pattern.chars().count() as i64)) {
                    return true;
                }
                let mut j: i64 = ni;
                while (j <= (name.chars().count() as i64)) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j += 1_i64;
                }
                return false;
            } else {
                if pc == "?" {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                } else {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name
                            .chars()
                            .nth(ni as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                }
            }
        } else {
            return false;
        }
    }
    (ni == (name.chars().count() as i64))
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
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_pattern.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_pattern
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "*" {
                body.push_str(".*");
            } else {
                if ch == "?" {
                    body.push('.');
                } else {
                    body.push_str((_translate_literal(&ch)).as_str());
                }
            }
        }
        i += 1_i64;
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
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_template.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_template
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let Some(ch) = ch else {
            i += 1_i64;
            continue;
        };
        let mut ch_value: String = "".to_string();
        if true {
            ch_value = ch;
        }
        if ch_value != "$" {
            result.push_str((ch_value).as_str());
            i += 1_i64;
            continue;
        }
        if ((i + (1_i64)) >= (__sifr_chars_template.len() as i64)) {
            if safe {
                result.push('$');
                i += 1_i64;
                continue;
            }
            return Err(
                ValueError::new(
                    "invalid template placeholder at end of string".to_string(),
                ),
            );
        }
        let next_ch: Option<String> = __sifr_chars_template
            .get((i + (1_i64)) as usize)
            .map(|c| c.to_string());
        let mut next_value: String = "".to_string();
        if next_ch.is_none() {
            if safe {
                result.push('$');
                i += 1_i64;
                continue;
            }
            return Err(ValueError::new("invalid template placeholder".to_string()));
        } else {
            if let Some(next_ch) = next_ch {
                next_value = next_ch;
            }
        }
        if next_value == "$" {
            result.push('$');
            i += 2_i64;
            continue;
        }
        if next_value == "{" {
            let mut j: i64 = i + (2_i64);
            let mut name: String = "".to_string();
            let mut __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
            while (j < (__sifr_chars_template.len() as i64)) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_template
                        .get(j as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                let Some(part) = part else {
                    j += 1_i64;
                    continue;
                };
                let mut part_value: String = "".to_string();
                if true {
                    part_value = part;
                }
                if part_value == "}" {
                    break;
                }
                let __sifr_string_concat_name_0 = part_value;
                name.push_str((__sifr_string_concat_name_0).as_str());
                __sifr_chars_name
                    .extend(((__sifr_string_concat_name_0).as_str()).chars());
                j += 1_i64;
            }
            if (j >= (__sifr_chars_template.len() as i64)) {
                if safe {
                    result
                        .push_str(
                            ({
                                let _slice_src = &__sifr_chars_template;
                                let _slice_len_i64 = _slice_src.len() as i64;
                                let _slice_start_i64 = if i < 0 {
                                    (_slice_len_i64 + i).max(0)
                                } else {
                                    i.min(_slice_len_i64)
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
                    return Ok(result);
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder: missing closing brace".to_string(),
                    ),
                );
            }
            if ((__sifr_chars_name.len() as i64) == (0_i64)) {
                if safe {
                    result.push_str("${}");
                    i = j + (1_i64);
                    continue;
                }
                return Err(
                    ValueError::new(
                        "invalid template placeholder: empty name".to_string(),
                    ),
                );
            }
            let first_candidate: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_name
                    .get((0_i64) as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            let mut first_value: String = "".to_string();
            let mut has_first: bool = false;
            if let Some(first_candidate) = first_candidate {
                has_first = true;
                first_value = first_candidate;
            }
            if !has_first || !(_is_identifier_start(&first_value)) {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = j + (1_i64);
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
            let mut k: i64 = 1_i64;
            while (k < (__sifr_chars_name.len() as i64)) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_name
                        .get(k as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                if let Some(part) = part {
                    if !(_is_identifier_continue(&part)) {
                        valid = false;
                        k = __sifr_chars_name.len() as i64;
                    }
                }
                k += 1_i64;
            }
            if !valid {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = j + (1_i64);
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
            if mapped_value.is_none() {
                if safe {
                    result.push_str("${");
                    result.push_str((name).as_str());
                    result.push('}');
                    i = j + (1_i64);
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
            i = j + (1_i64);
            continue;
        }
        if !(_is_identifier_start(&next_value)) {
            if safe {
                result.push('$');
                result.push_str((next_value).as_str());
                i += 2_i64;
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
        let mut j2: i64 = i + (1_i64);
        while (j2 < (__sifr_chars_template.len() as i64)) {
            let part2: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_template
                    .get(j2 as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            let Some(part2) = part2 else {
                j2 += 1_i64;
                continue;
            };
            let mut part2_value: String = "".to_string();
            if true {
                part2_value = part2;
            }
            if !(_is_identifier_continue(&part2_value)) {
                break;
            }
            name2.push_str((part2_value).as_str());
            j2 += 1_i64;
        }
        let mapped_value2: Option<String> = _mapping_lookup(mapping, &name2);
        let mut mapped_value2_text: String = "".to_string();
        if mapped_value2.is_none() {
            if safe {
                result.push('$');
                result.push_str((name2).as_str());
                i = j2;
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
        i = j2;
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
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_format_string.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_format_string
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let Some(ch) = ch else {
            i += 1_i64;
            continue;
        };
        let mut ch_value: String = "".to_string();
        if true {
            ch_value = ch;
        }
        if ch_value == "{" {
            if ((i + (1_i64)) < (__sifr_chars_format_string.len() as i64)) {
                let escaped_next: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_format_string
                        .get((i + (1_i64)) as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                if escaped_next.is_some() && (escaped_next == Some("{".to_string())) {
                    result.push('{');
                    i += 2_i64;
                    continue;
                }
            }
            let mut j: i64 = i + (1_i64);
            let mut field_name: String = "".to_string();
            let mut __sifr_chars_field_name: Vec<char> = field_name
                .chars()
                .collect::<Vec<char>>();
            while (j < (__sifr_chars_format_string.len() as i64)) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_format_string
                        .get(j as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                let Some(part) = part else {
                    j += 1_i64;
                    continue;
                };
                let mut part_value: String = "".to_string();
                if true {
                    part_value = part;
                }
                if part_value == "}" {
                    break;
                }
                let __sifr_string_concat_field_name_0 = part_value;
                field_name.push_str((__sifr_string_concat_field_name_0).as_str());
                __sifr_chars_field_name
                    .extend(((__sifr_string_concat_field_name_0).as_str()).chars());
                j += 1_i64;
            }
            if (j >= (__sifr_chars_format_string.len() as i64)) {
                return Err(
                    ValueError::new("formatter: missing closing brace".to_string()),
                );
            }
            if ((__sifr_chars_field_name.len() as i64) == (0_i64)) {
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
            i = j + (1_i64);
            continue;
        }
        if ch_value == "}" {
            if ((i + (1_i64)) < (__sifr_chars_format_string.len() as i64)) {
                let escaped_next2: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_format_string
                        .get((i + (1_i64)) as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                if escaped_next2.is_some() && (escaped_next2 == Some("}".to_string())) {
                    result.push('}');
                    i += 2_i64;
                    continue;
                }
            }
            return Err(
                ValueError::new("formatter: single \'}\' is invalid".to_string()),
            );
        }
        result.push_str((ch_value).as_str());
        i += 1_i64;
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
        rendered_ok = rendered == "Hello Sifr, mode=c2";
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
        rendered_fmt_ok = rendered_fmt == "Status c2: ok";
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(rendered_fmt_ok);
    let wrapper: __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper = __SifrStdlib_sifr_x2etextwrap_x2eTextWrapper::new(
        8_i64,
        "> ".to_string(),
        ".. ".to_string(),
        true,
        8_i64,
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
        decoded_ok = decoded == "hello";
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
    let ratio: f64 = __SifrStdlib_sifr_x2edifflib_x2eSequenceMatcher::new(
            "abcd".to_string(),
            "abed".to_string(),
        )
        .ratio();
    assert!(ratio > (0.4_f64));
    let mut month_label_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let month_label: String = __SifrStdlib_sifr_x2ecalendar_x2eTextCalendar::new(
                0_i64,
            )
            .formatmonthname(2024_i64, 2_i64, 0_i64)?;
        month_label_ok = month_label == "February 2024";
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(month_label_ok);
}

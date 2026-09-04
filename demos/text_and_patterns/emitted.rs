// src/main.rs
mod sifr_generated_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    pub use ::std::collections::HashMap;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estringX2eTemplate {
        pub template: String,
    }
    impl SifrGeneratedStdlibSifrX2estringX2eTemplate {
        #[must_use]
        pub fn new(template: String) -> Self {
            let sifr_generated_field_value_33609a5e9eb92f4b_74656d706c617465: String = {
                let mut sifr_generated_concat: String = String::with_capacity(template.len());
                sifr_generated_concat.push_str(template.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                template: sifr_generated_field_value_33609a5e9eb92f4b_74656d706c617465,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2estringX2eTemplate {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn substitute(&self, mapping: &HashMap<String, String>) -> Result<String, ValueError> {
            sifr_generated_template_substitute_impl(&self.template, mapping, false)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2estringX2eTemplate {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Template(template={})", self.template)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estringX2eFormatter {}
    impl SifrGeneratedStdlibSifrX2estringX2eFormatter {
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2estringX2eFormatter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2estringX2eFormatter {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn format(
            &self,
            format_string: &str,
            values: &HashMap<String, String>,
        ) -> Result<String, ValueError> {
            sifr_generated_formatter_format_impl(format_string, values)
        }
    }
    #[must_use]
    pub fn sifr_generated_is_identifier_start(ch: &str) -> bool {
        ch == "_" || !ch.is_empty() && ch.chars().all(char::is_alphabetic)
    }
    #[must_use]
    pub fn sifr_generated_is_identifier_continue(ch: &str) -> bool {
        ch == "_"
            || !ch.is_empty() && ch.chars().all(char::is_alphabetic)
            || !ch.is_empty() && ch.chars().all(|c| c.is_ascii_digit())
    }
    #[must_use]
    pub fn sifr_generated_mapping_lookup(
        mapping: &HashMap<String, String>,
        key: &str,
    ) -> Option<String> {
        for (current_key, current_value) in mapping
            .iter()
            .map(|sifr_generated_kv| (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if current_key == *key {
                return Some({
                    let mut sifr_generated_concat: String =
                        String::with_capacity(current_value.len());
                    sifr_generated_concat.push_str(current_value.as_str());
                    sifr_generated_concat.push_str("");
                    sifr_generated_concat
                });
            }
        }
        None
    }
    ///# Errors
    ///Returns the typed error produced by this operation.
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub fn sifr_generated_template_substitute_impl(
        template: &str,
        mapping: &HashMap<String, String>,
        safe: bool,
    ) -> Result<String, ValueError> {
        let sifr_generated_chars_template: Vec<char> = template.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_template.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_template.len());
                sifr_generated_chars_template
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let Some(ch) = ch else {
                i = &i + &SifrInt::from_i64(1);
                continue;
            };
            let ch_value: String = ch;
            if ch_value != "$" {
                result.push_str(ch_value.as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if &(&i + &SifrInt::from_i64(1)) >= &SifrInt::from(sifr_generated_chars_template.len())
            {
                if safe {
                    result.push('$');
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(ValueError::new(
                    "invalid template placeholder at end of string".to_string(),
                ));
            }
            let next_ch: Option<String> = {
                let sifr_generated_string_index = &i + &SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_template.len());
                sifr_generated_chars_template
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let mut next_value: String = String::new();
            if next_ch.is_none() {
                if safe {
                    result.push('$');
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(ValueError::new("invalid template placeholder".to_string()));
            } else if let Some(next_ch) = next_ch {
                next_value = next_ch;
            }
            if next_value == "$" {
                result.push('$');
                i = &i + &SifrInt::from_i64(2);
                continue;
            }
            if next_value == "{" {
                let mut j: SifrInt = &i + &SifrInt::from_i64(2);
                let mut name: String = String::new();
                let mut sifr_generated_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
                while &j < &SifrInt::from(sifr_generated_chars_template.len()) {
                    let part: Option<String> = {
                        let sifr_generated_string_index = j.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_template.len());
                        sifr_generated_chars_template
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    let Some(part_value_03b1250debc64fd4) = part else {
                        j = &j + &SifrInt::from_i64(1);
                        continue;
                    };
                    let part_value: String = part_value_03b1250debc64fd4;
                    if part_value == "}" {
                        break;
                    }
                    let sifr_generated_string_concat_name_0 = part_value;
                    name.push_str(sifr_generated_string_concat_name_0.as_str());
                    sifr_generated_chars_name
                        .extend(sifr_generated_string_concat_name_0.as_str().chars());
                    j = &j + &SifrInt::from_i64(1);
                }
                if &j >= &SifrInt::from(sifr_generated_chars_template.len()) {
                    if safe {
                        result.push_str(
                            {
                                let sifr_generated_slice_src = &sifr_generated_chars_template;
                                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                                let sifr_generated_slice_start =
                                    i.clamp_slice_bound(sifr_generated_slice_len);
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
                        return Ok(result);
                    }
                    return Err(ValueError::new(
                        "invalid template placeholder: missing closing brace".to_string(),
                    ));
                }
                if &SifrInt::from(sifr_generated_chars_name.len()) == &SifrInt::from_i64(0) {
                    if safe {
                        result.push_str("${}");
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(ValueError::new(
                        "invalid template placeholder: empty name".to_string(),
                    ));
                }
                let first_candidate: Option<String> = {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_name.len());
                    sifr_generated_chars_name
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                let mut first_value: String = String::new();
                let has_first: bool = first_candidate.is_some_and(|first_candidate| {
                    let has_first = true;
                    first_value = first_candidate;
                    has_first
                });
                if !has_first || !sifr_generated_is_identifier_start(&first_value) {
                    if safe {
                        result.push_str("${");
                        result.push_str(name.as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(ValueError::new({
                        let mut sifr_generated_concat: String =
                            String::with_capacity(30usize + name.len());
                        sifr_generated_concat.push_str("invalid template placeholder: ");
                        sifr_generated_concat.push_str(name.as_str());
                        sifr_generated_concat
                    }));
                }
                let mut valid: bool = true;
                let mut k: SifrInt = SifrInt::from_i64(1);
                while &k < &SifrInt::from(sifr_generated_chars_name.len()) {
                    let part: Option<String> = {
                        let sifr_generated_string_index = k.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_name.len());
                        sifr_generated_chars_name
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(part) = part
                        && !sifr_generated_is_identifier_continue(&part)
                    {
                        valid = false;
                        k = SifrInt::from(sifr_generated_chars_name.len());
                    }
                    k = &k + &SifrInt::from_i64(1);
                }
                if !valid {
                    if safe {
                        result.push_str("${");
                        result.push_str(name.as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(ValueError::new({
                        let mut sifr_generated_concat: String =
                            String::with_capacity(30usize + name.len());
                        sifr_generated_concat.push_str("invalid template placeholder: ");
                        sifr_generated_concat.push_str(name.as_str());
                        sifr_generated_concat
                    }));
                }
                let mapped_value: Option<String> = sifr_generated_mapping_lookup(mapping, &name);
                let mut mapped_value_text: String = String::new();
                if mapped_value.is_none() {
                    if safe {
                        result.push_str("${");
                        result.push_str(name.as_str());
                        result.push('}');
                        i = &j + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(ValueError::new({
                        let mut sifr_generated_concat: String =
                            String::with_capacity(32usize + name.len());
                        sifr_generated_concat.push_str("missing template value for key: ");
                        sifr_generated_concat.push_str(name.as_str());
                        sifr_generated_concat
                    }));
                } else if let Some(mapped_value) = mapped_value {
                    mapped_value_text = mapped_value;
                }
                result.push_str(mapped_value_text.as_str());
                i = &j + &SifrInt::from_i64(1);
                continue;
            }
            if !sifr_generated_is_identifier_start(&next_value) {
                if safe {
                    result.push('$');
                    result.push_str(next_value.as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                return Err(ValueError::new({
                    let mut sifr_generated_concat: String =
                        String::with_capacity(36usize + next_value.len());
                    sifr_generated_concat.push_str("invalid template placeholder near: $");
                    sifr_generated_concat.push_str(next_value.as_str());
                    sifr_generated_concat
                }));
            }
            let mut name2_value_afb6e7fff26812dc: String = String::new();
            let mut j2: SifrInt = &i + &SifrInt::from_i64(1);
            while &j2 < &SifrInt::from(sifr_generated_chars_template.len()) {
                let part2_value_0c51dca7a1f9c3d2: Option<String> = {
                    let sifr_generated_string_index = j2.clone();
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_template.len());
                    sifr_generated_chars_template
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                let Some(part2_value_0c51dca7a1f9c3d2) = part2_value_0c51dca7a1f9c3d2 else {
                    j2 = &j2 + &SifrInt::from_i64(1);
                    continue;
                };
                let part2_value: String = part2_value_0c51dca7a1f9c3d2;
                if !sifr_generated_is_identifier_continue(&part2_value) {
                    break;
                }
                name2_value_afb6e7fff26812dc.push_str(part2_value.as_str());
                j2 = &j2 + &SifrInt::from_i64(1);
            }
            let mapped_value2_value_48081efa5265009a: Option<String> =
                sifr_generated_mapping_lookup(mapping, &name2_value_afb6e7fff26812dc);
            let mut mapped_value2_text_value_302fac2b5e0e93a8: String = String::new();
            if mapped_value2_value_48081efa5265009a.is_none() {
                if safe {
                    result.push('$');
                    result.push_str(name2_value_afb6e7fff26812dc.as_str());
                    i = j2.clone();
                    continue;
                }
                return Err(ValueError::new({
                    let mut sifr_generated_concat: String =
                        String::with_capacity(32usize + name2_value_afb6e7fff26812dc.len());
                    sifr_generated_concat.push_str("missing template value for key: ");
                    sifr_generated_concat.push_str(name2_value_afb6e7fff26812dc.as_str());
                    sifr_generated_concat
                }));
            } else if let Some(mapped_value2) = mapped_value2_value_48081efa5265009a {
                mapped_value2_text_value_302fac2b5e0e93a8 = mapped_value2;
            }
            result.push_str(mapped_value2_text_value_302fac2b5e0e93a8.as_str());
            i = j2.clone();
        }
        Ok(result)
    }
    ///# Errors
    ///Returns the typed error produced by this operation.
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub fn sifr_generated_formatter_format_impl(
        format_string: &str,
        values: &HashMap<String, String>,
    ) -> Result<String, ValueError> {
        let sifr_generated_chars_format_string: Vec<char> =
            format_string.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_format_string.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_format_string.len());
                sifr_generated_chars_format_string
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let Some(ch) = ch else {
                i = &i + &SifrInt::from_i64(1);
                continue;
            };
            let ch_value: String = ch;
            if ch_value == "{" {
                if &(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(sifr_generated_chars_format_string.len())
                {
                    let escaped_next: Option<String> = {
                        let sifr_generated_string_index = &i + &SifrInt::from_i64(1);
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_format_string.len());
                        sifr_generated_chars_format_string
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if escaped_next.is_some() && escaped_next == Some("{".to_string()) {
                        result.push('{');
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                }
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                let mut field_name: String = String::new();
                let mut sifr_generated_chars_field_name: Vec<char> =
                    field_name.chars().collect::<Vec<char>>();
                while &j < &SifrInt::from(sifr_generated_chars_format_string.len()) {
                    let part: Option<String> = {
                        let sifr_generated_string_index = j.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_format_string.len());
                        sifr_generated_chars_format_string
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    let Some(part_value_03b1250debc64fd4) = part else {
                        j = &j + &SifrInt::from_i64(1);
                        continue;
                    };
                    let part_value: String = part_value_03b1250debc64fd4;
                    if part_value == "}" {
                        break;
                    }
                    let sifr_generated_string_concat_field_name_0 = part_value;
                    field_name.push_str(sifr_generated_string_concat_field_name_0.as_str());
                    sifr_generated_chars_field_name
                        .extend(sifr_generated_string_concat_field_name_0.as_str().chars());
                    j = &j + &SifrInt::from_i64(1);
                }
                if &j >= &SifrInt::from(sifr_generated_chars_format_string.len()) {
                    return Err(ValueError::new(
                        "formatter: missing closing brace".to_string(),
                    ));
                }
                if &SifrInt::from(sifr_generated_chars_field_name.len()) == &SifrInt::from_i64(0) {
                    return Err(ValueError::new(
                        "formatter: empty replacement field is not supported".to_string(),
                    ));
                }
                let value_value_7ce4fd9430e80cea: Option<String> =
                    sifr_generated_mapping_lookup(values, &field_name);
                let Some(value_value_7ce4fd9430e80cea) = value_value_7ce4fd9430e80cea else {
                    return Err(ValueError::new({
                        let mut sifr_generated_concat: String =
                            String::with_capacity(34usize + field_name.len());
                        sifr_generated_concat.push_str("formatter: missing value for key: ");
                        sifr_generated_concat.push_str(field_name.as_str());
                        sifr_generated_concat
                    }));
                };
                result.push_str(value_value_7ce4fd9430e80cea.as_str());
                i = &j + &SifrInt::from_i64(1);
                continue;
            }
            if ch_value == "}" {
                if &(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(sifr_generated_chars_format_string.len())
                {
                    let escaped_next2_value_afe7a7cfff221700: Option<String> = {
                        let sifr_generated_string_index = &i + &SifrInt::from_i64(1);
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_format_string.len());
                        sifr_generated_chars_format_string
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if escaped_next2_value_afe7a7cfff221700.is_some()
                        && escaped_next2_value_afe7a7cfff221700 == Some("}".to_string())
                    {
                        result.push('}');
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                }
                return Err(ValueError::new(
                    "formatter: single \'}\' is invalid".to_string(),
                ));
            }
            result.push_str(ch_value.as_str());
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "generated Rust preserves this exact typed Sifr source contract"
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
            reason = "generated Rust preserves this exact typed Sifr source contract"
        )]
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
            let sifr_generated_field_value_dbdacd932fd1e9bf_7769647468: SifrInt = width.clone();
            let sifr_generated_field_value_f1d9debc65d6e532_696e697469616c5f696e64656e74: String = {
                let mut sifr_generated_concat: String = String::with_capacity(initial_indent.len());
                sifr_generated_concat.push_str(initial_indent.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_45b636e6527b24bb_73756273657175656e745f696e64656e74: String = {
                let mut sifr_generated_concat: String = String::with_capacity(
                    subsequent_indent.len(),
                );
                sifr_generated_concat.push_str(subsequent_indent.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_9fdde0a58b2f170e_657870616e645f74616273: bool =
                expand_tabs;
            let mut safe_tabsize: SifrInt = tabsize.clone();
            if &safe_tabsize <= &SifrInt::from_i64(0) {
                safe_tabsize = SifrInt::from_i64(1);
            }
            let sifr_generated_field_value_0f728cbe37fa9025_74616273697a65: SifrInt =
                safe_tabsize.clone();
            let sifr_generated_field_value_d659e98074e25261_7265706c6163655f77686974657370616365: bool = replace_whitespace;
            let sifr_generated_field_value_a317a122f9288b94_64726f705f77686974657370616365: bool =
                drop_whitespace;
            let sifr_generated_field_value_acdab20e5253523e_627265616b5f6f6e5f68797068656e73: bool =
                break_on_hyphens;
            let sifr_generated_field_value_116e01dc088ea88b_6669785f73656e74656e63655f656e64696e6773: bool = fix_sentence_endings;
            let sifr_generated_field_value_441854f90b4986e9_6d61785f6c696e6573: Option<SifrInt> =
                max_lines.clone();
            let sifr_generated_field_value_615e79d982d9f0fa_706c616365686f6c646572: String = {
                let mut sifr_generated_concat: String = String::with_capacity(placeholder.len());
                sifr_generated_concat.push_str(placeholder.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
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
            if &self.width.clone() <= &SifrInt::from_i64(0) {
                return Vec::new();
            }
            let prepared: String = sifr_generated_prepare_text(
                text,
                self.expand_tabs,
                self.tabsize.clone(),
                self.replace_whitespace,
            );
            let mut lines: Vec<String> = sifr_generated_wrap_with_indents(
                &prepared,
                self.width.clone(),
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
                self.width.clone(),
                self.max_lines.clone(),
                &self.placeholder,
                self.drop_whitespace,
            )
        }
    }
    #[must_use]
    pub fn sifr_generated_replace_whitespace_chars(text: &str, replace_tabs: bool) -> String {
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
    #[must_use]
    pub fn sifr_generated_expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
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
    #[must_use]
    pub fn sifr_generated_prepare_text(
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
    #[must_use]
    pub fn sifr_generated_split_word_units(word: &str, break_on_hyphens: bool) -> Vec<String> {
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
    #[must_use]
    pub fn sifr_generated_trim_line(line: &str) -> String {
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
            {
                let sifr_generated_string_index = start.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_line.len());
                sifr_generated_chars_line
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                == Some(Some(' '))
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
        }
        .map(Some)
            == Some(Some(' '))
        {
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
    #[must_use]
    pub fn sifr_generated_finalize_line(line: &str, drop_whitespace: bool) -> String {
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
    #[must_use]
    pub fn sifr_generated_effective_content_width(total_width: SifrInt, indent: &str) -> SifrInt {
        let sifr_generated_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
        let available: SifrInt = &total_width - &SifrInt::from(sifr_generated_chars_indent.len());
        if &available <= &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        available.clone()
    }
    pub fn sifr_generated_push_current_line(
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
    #[must_use]
    pub fn sifr_generated_wrap_with_indents(
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
                        && &(&SifrInt::from(sifr_generated_chars_current.len())
                            + &SifrInt::from_i64(1))
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
    #[must_use]
    pub fn sifr_generated_apply_sentence_endings_line(text: &str) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = i.clone();
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
                    let next_opt: Option<String> = if &(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                        && let Some(_checked_value_4) = {
                            let sifr_generated_string_index = &i + &SifrInt::from_i64(1);
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
                            let sifr_generated_string_index = &i + &SifrInt::from_i64(1);
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
                    let next2_opt_value_88541be202984f38: Option<String> = if &(&i
                        + &SifrInt::from_i64(2))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                    {
                        {
                            let sifr_generated_string_index = &i + &SifrInt::from_i64(2);
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
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    #[must_use]
    pub fn sifr_generated_apply_sentence_endings_lines(lines: &[String]) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        for line in lines.iter().cloned() {
            result.push(sifr_generated_apply_sentence_endings_line(&line));
        }
        result
    }
    #[must_use]
    pub fn sifr_generated_clone_lines(lines: &[String]) -> Vec<String> {
        let mut copied: Vec<String> = Vec::new();
        for line in lines.iter().cloned() {
            copied.push(line);
        }
        copied
    }
    #[must_use]
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub fn sifr_generated_apply_max_lines(
        lines: &[String],
        width: SifrInt,
        max_lines: Option<SifrInt>,
        placeholder: &str,
        drop_whitespace: bool,
    ) -> Vec<String> {
        let Some(max_lines) = max_lines.clone() else {
            return sifr_generated_clone_lines(lines);
        };
        let limit: SifrInt = max_lines.clone();
        if &limit <= &SifrInt::from_i64(0) {
            return Vec::new();
        }
        if &SifrInt::from(lines.len()) <= &limit {
            return sifr_generated_clone_lines(lines);
        }
        let mut result: Vec<String> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &limit {
            let line_opt: Option<String> = {
                let sifr_generated_checked_read_collection = &lines;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(line_opt) = line_opt {
                result.push(line_opt);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(result.len()) == &SifrInt::from_i64(0) {
            return result;
        }
        let mut effective_placeholder: String = {
            let mut sifr_generated_concat: String = String::with_capacity(placeholder.len());
            sifr_generated_concat.push_str(placeholder);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let mut sifr_generated_chars_effective_placeholder: Vec<char> =
            effective_placeholder.chars().collect::<Vec<char>>();
        if &width > &SifrInt::from_i64(0)
            && &SifrInt::from(sifr_generated_chars_effective_placeholder.len()) > &width
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
        let last_index: SifrInt = &SifrInt::from(result.len()) - &SifrInt::from_i64(1);
        let last_opt: Option<String> = {
            let sifr_generated_checked_read_collection = &result;
            let sifr_generated_checked_read_index = last_index.clone();
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
            let mut available: SifrInt =
                &width - &SifrInt::from(sifr_generated_chars_effective_placeholder.len());
            if &available < &SifrInt::from_i64(0) {
                available = SifrInt::from_i64(0);
            }
            if &SifrInt::from(sifr_generated_chars_base.len()) > &available {
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
            if &SifrInt::from_i64(0) <= &last_index && &last_index < &SifrInt::from(result.len()) {
                {
                    let sifr_generated_assign_value = {
                        let mut sifr_generated_concat: String =
                            String::with_capacity(base.len() + effective_placeholder.len());
                        sifr_generated_concat.push_str(base.as_str());
                        sifr_generated_concat.push_str(effective_placeholder.as_str());
                        sifr_generated_concat
                    };
                    {
                        let sifr_generated_index_raw = last_index.clone();
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
        #[must_use]
        pub const fn new(message: String) -> Self {
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
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estringX2eFormatter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estringX2eTemplate;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0;
fn base64_encode(s: &str) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s).map_err(|sifr_generated_bridge_error| ParseError {
        message: sifr_generated_bridge_error.to_string(),
    })
}
fn b64encode(s: &str) -> String {
    base64_encode(s)
}
fn b64decode(s: &str) -> Result<String, ParseError> {
    base64_decode(s)
}
fn sifr_generated_const_6d6f6e74685f6e616d65() -> Vec<String> {
    vec![
        String::new(),
        "January".to_string(),
        "February".to_string(),
        "March".to_string(),
        "April".to_string(),
        "May".to_string(),
        "June".to_string(),
        "July".to_string(),
        "August".to_string(),
        "September".to_string(),
        "October".to_string(),
        "November".to_string(),
        "December".to_string(),
    ]
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SifrGeneratedStdlibSifrX2ecalendarX2eTextCalendar {
    firstweekday: SifrInt,
}
impl SifrGeneratedStdlibSifrX2ecalendarX2eTextCalendar {
    fn new(firstweekday: SifrInt) -> Self {
        let sifr_generated_field_value_2daa31ad6c3bfb29_66697273747765656b646179: SifrInt =
            sifr_generated_normalize_firstweekday(firstweekday.clone());
        Self {
            firstweekday: sifr_generated_field_value_2daa31ad6c3bfb29_66697273747765656b646179,
        }
    }
}
impl SifrGeneratedStdlibSifrX2ecalendarX2eTextCalendar {
    fn formatmonthname(
        &self,
        year: &SifrInt,
        month: &SifrInt,
        width: &SifrInt,
    ) -> Result<String, ValueError> {
        let name_lookup: Option<String> = sifr_generated_month_name_lookup(month.clone());
        let name: String = if let Some(name_lookup) = name_lookup {
            name_lookup
        } else {
            return Err(ValueError::new(
                "calendar: month must be in 1..12".to_string(),
            ));
        };
        let formatted: String = {
            let mut sifr_generated_concat: String = String::with_capacity(name.len() + 1usize);
            sifr_generated_concat.push_str(name.as_str());
            sifr_generated_concat.push(' ');
            sifr_generated_concat.push_str(year.to_string().as_str());
            sifr_generated_concat
        };
        if width <= &SifrInt::from_i64(0) {
            return Ok(formatted);
        }
        if &SifrInt::from(formatted.chars().count()) >= width {
            return Ok(formatted);
        }
        let pad: SifrInt = width - &SifrInt::from(formatted.chars().count());
        let mut left: SifrInt = pad.floor_div_known_nonzero(&SifrInt::from_i64(2));
        let mut right: SifrInt = &pad - &left;
        let mut result: String = String::new();
        while &left > &SifrInt::from_i64(0) {
            result.push(' ');
            left = &left - &SifrInt::from_i64(1);
        }
        result.push_str(formatted.as_str());
        while &right > &SifrInt::from_i64(0) {
            result.push(' ');
            right = &right - &SifrInt::from_i64(1);
        }
        Ok(result)
    }
}
impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ecalendarX2eTextCalendar {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextCalendar(firstweekday={})", self.firstweekday)
    }
}
fn sifr_generated_normalize_firstweekday(firstweekday: SifrInt) -> SifrInt {
    let mut value: SifrInt = firstweekday.floor_mod_known_nonzero(&SifrInt::from_i64(7));
    if &value < &SifrInt::from_i64(0) {
        value = &value + &SifrInt::from_i64(7);
    }
    value.clone()
}
fn sifr_generated_month_name_lookup(month: SifrInt) -> Option<String> {
    if &month < &SifrInt::from_i64(1) || &month > &SifrInt::from_i64(12) {
        return None;
    }
    {
        let sifr_generated_checked_read_collection = &sifr_generated_const_6d6f6e74685f6e616d65();
        let sifr_generated_checked_read_index = month.clone();
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SifrGeneratedStdlibSifrX2edifflibX2eSequenceMatcher {
    a: String,
    b: String,
}
impl SifrGeneratedStdlibSifrX2edifflibX2eSequenceMatcher {
    fn new(a: String, b: String) -> Self {
        let sifr_generated_field_value_09534707b5e0a7dd_5f61: String = {
            let mut sifr_generated_concat: String = String::with_capacity(a.len());
            sifr_generated_concat.push_str(a.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let sifr_generated_field_value_09534407b5e0a2c4_5f62: String = {
            let mut sifr_generated_concat: String = String::with_capacity(b.len());
            sifr_generated_concat.push_str(b.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        Self {
            a: sifr_generated_field_value_09534707b5e0a7dd_5f61,
            b: sifr_generated_field_value_09534407b5e0a2c4_5f62,
        }
    }
}
impl SifrGeneratedStdlibSifrX2edifflibX2eSequenceMatcher {
    fn ratio(
        &self,
    ) -> Result<
        f64,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    >{
        sifr_generated_similarity(&self.a, &self.b)
    }
}
impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edifflibX2eSequenceMatcher {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "SequenceMatcher(_a={}, _b={})", self.a, self.b)
    }
}
fn sifr_generated_similarity(
    a: &str,
    b: &str,
) -> Result<
    f64,
    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
>{
    let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let total: SifrInt =
        &SifrInt::from(sifr_generated_chars_a.len()) + &SifrInt::from(sifr_generated_chars_b.len());
    if &total == &SifrInt::from_i64(0) {
        return Ok(1.0_f64);
    }
    let mut matches: SifrInt = SifrInt::from_i64(0);
    let blocks: Vec<(SifrInt, SifrInt, SifrInt)> = sifr_generated_matching_blocks(a, b);
    for block in blocks.iter().cloned() {
        let (_, _, block_size) = block;
        matches = &matches + &block_size;
    }
    let sifr_generated_try_res: Result<
        Result<
            f64,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        >,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    > = (|| {
        let numerator: f64 = (&SifrInt::from_i64(2) * &matches)
            .checked_to_f64()
            .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
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
            .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(numerator / denominator))
    })();
    sifr_generated_try_res
        .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error.clone();
                Err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        FloatOverflowError::new(error.message.clone()),
                    ),
                )
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error.clone();
                Err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        FloatPrecisionLossError::new(error.message.clone()),
                    ),
                )
            }
        })
}
#[expect(
    clippy::many_single_char_names,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn sifr_generated_longest_common_substring_range(
    a: &str,
    b: &str,
    a_start: SifrInt,
    a_end: SifrInt,
    b_start_argument_c9091ffae73223be: SifrInt,
    b_end_argument_847b7941bf1533ab: SifrInt,
) -> (SifrInt, SifrInt, SifrInt) {
    let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut best_i: SifrInt = SifrInt::from_i64(0);
    let mut best_j_value_1e1620cd9a699d86: SifrInt = SifrInt::from_i64(0);
    let mut best_len_value_af9487aca555f05f: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = a_start.clone();
    while &i < &a_end {
        let mut j: SifrInt = b_start_argument_c9091ffae73223be.clone();
        while &j < &b_end_argument_847b7941bf1533ab {
            let mut k: SifrInt = SifrInt::from_i64(0);
            while &(&i + &k) < &a_end && &(&j + &k) < &b_end_argument_847b7941bf1533ab {
                let ai: Option<String> = {
                    let sifr_generated_string_index = &i + &k;
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_a.len());
                    sifr_generated_chars_a
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                let bj: Option<String> = {
                    let sifr_generated_string_index = &j + &k;
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_b.len());
                    sifr_generated_chars_b
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                let (Some(ai), Some(bj)) = (ai, bj) else {
                    k = &k + &SifrInt::from_i64(1);
                    continue;
                };
                if ai != bj {
                    break;
                }
                k = &k + &SifrInt::from_i64(1);
            }
            if &k > &best_len_value_af9487aca555f05f {
                best_len_value_af9487aca555f05f = k;
                best_i = i.clone();
                best_j_value_1e1620cd9a699d86 = j.clone();
            }
            j = &j + &SifrInt::from_i64(1);
        }
        i = &i + &SifrInt::from_i64(1);
    }
    (
        best_i.clone(),
        best_j_value_1e1620cd9a699d86.clone(),
        best_len_value_af9487aca555f05f.clone(),
    )
}
fn sifr_generated_sort_blocks(
    blocks: &[(SifrInt, SifrInt, SifrInt)],
) -> Vec<(SifrInt, SifrInt, SifrInt)> {
    let mut sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
    for block in blocks.iter().cloned() {
        let (bl_a, bl_b_value_c53dd39bc263efba, _) = block.clone();
        let mut found_insert_at: bool = false;
        let mut insert_at: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        for existing in sorted_blocks.iter().cloned() {
            if !found_insert_at {
                let (ex_a, ex_b_value_e8565f608f1d5555, _) = existing;
                let comes_before: bool = if &bl_a < &ex_a
                    || &bl_a == &ex_a && &bl_b_value_c53dd39bc263efba < &ex_b_value_e8565f608f1d5555
                {
                    true
                } else {
                    false
                };
                if comes_before {
                    insert_at = i.clone();
                    found_insert_at = true;
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if found_insert_at {
            sorted_blocks.insert(::sifr_runtime::to_usize_proven(&insert_at), block.clone());
        } else {
            sorted_blocks.push(block.clone());
        }
    }
    sorted_blocks
}
fn sifr_generated_matching_blocks(a: &str, b: &str) -> Vec<(SifrInt, SifrInt, SifrInt)> {
    let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
    let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
    let mut pending_a_start: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
    let mut pending_a_end: Vec<SifrInt> = vec![SifrInt::from(sifr_generated_chars_a.len())];
    let mut pending_b_start_value_5010e609c75d1d22: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
    let mut pending_b_end_value_9589c6af9c1daa47: Vec<SifrInt> =
        vec![SifrInt::from(sifr_generated_chars_b.len())];
    let mut unsorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
    while &SifrInt::from(pending_a_start.len()) > &SifrInt::from_i64(0) {
        let a_start_value: Option<SifrInt> =
            Some(pending_a_start.remove(pending_a_start.len() - 1_usize));
        let a_end_value: Option<SifrInt> = pending_a_end.pop();
        let b_start_value: Option<SifrInt> = pending_b_start_value_5010e609c75d1d22.pop();
        let b_end_value: Option<SifrInt> = pending_b_end_value_9589c6af9c1daa47.pop();
        if let Some(a_start_value) = a_start_value.clone()
            && let Some(a_end_value) = a_end_value.clone()
            && let Some(b_start_value) = b_start_value.clone()
            && let Some(b_end_value) = b_end_value.clone()
        {
            let (ai, bj, size) = sifr_generated_longest_common_substring_range(
                a,
                b,
                a_start_value.clone(),
                a_end_value.clone(),
                b_start_value.clone(),
                b_end_value.clone(),
            );
            if &size == &SifrInt::from_i64(0) {
                continue;
            }
            unsorted_blocks.push((ai.clone(), bj.clone(), size.clone()));
            let left_a_end: SifrInt = ai.clone();
            let left_b_end_value_2d7948a8a27a7433: SifrInt = bj.clone();
            if &a_start_value < &left_a_end && &b_start_value < &left_b_end_value_2d7948a8a27a7433 {
                pending_a_start.push(a_start_value);
                pending_a_end.push(left_a_end);
                pending_b_start_value_5010e609c75d1d22.push(b_start_value);
                pending_b_end_value_9589c6af9c1daa47.push(left_b_end_value_2d7948a8a27a7433);
            }
            let right_a_start: SifrInt = &ai + &size;
            let right_b_start_value_acd2b29c16778c53: SifrInt = &bj + &size;
            if &right_a_start < &a_end_value && &right_b_start_value_acd2b29c16778c53 < &b_end_value
            {
                pending_a_start.push(right_a_start);
                pending_a_end.push(a_end_value);
                pending_b_start_value_5010e609c75d1d22.push(right_b_start_value_acd2b29c16778c53);
                pending_b_end_value_9589c6af9c1daa47.push(b_end_value);
            }
        }
    }
    let sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> =
        sifr_generated_sort_blocks(&unsorted_blocks);
    let mut merged_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
    let mut have_previous: bool = false;
    let mut prev_a: SifrInt = SifrInt::from_i64(0);
    let mut prev_b_value_471dcfb5c284856d: SifrInt = SifrInt::from_i64(0);
    let mut prev_size: SifrInt = SifrInt::from_i64(0);
    for block in sorted_blocks.iter().cloned() {
        let (bl_a, bl_b_value_c53dd39bc263efba, bl_size) = block.clone();
        if !have_previous {
            prev_a = bl_a.clone();
            prev_b_value_471dcfb5c284856d = bl_b_value_c53dd39bc263efba.clone();
            prev_size = bl_size.clone();
            have_previous = true;
            continue;
        }
        if &(&prev_a + &prev_size) == &bl_a
            && &(&prev_b_value_471dcfb5c284856d + &prev_size) == &bl_b_value_c53dd39bc263efba
        {
            prev_size = &prev_size + &bl_size;
        } else {
            merged_blocks.push((
                prev_a.clone(),
                prev_b_value_471dcfb5c284856d.clone(),
                prev_size.clone(),
            ));
            prev_a = bl_a.clone();
            prev_b_value_471dcfb5c284856d = bl_b_value_c53dd39bc263efba.clone();
            prev_size = bl_size.clone();
        }
    }
    if have_previous {
        merged_blocks.push((
            prev_a.clone(),
            prev_b_value_471dcfb5c284856d.clone(),
            prev_size.clone(),
        ));
    }
    merged_blocks.push((
        SifrInt::from(a.chars().count()),
        SifrInt::from(b.chars().count()),
        SifrInt::from_i64(0),
    ));
    merged_blocks
}
fn fnmatch(name: &str, pattern: &str) -> bool {
    sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
}
fn sifr_generated_match(name: &str, mut ni: SifrInt, pattern: &str, mut pi: SifrInt) -> bool {
    while &pi < &SifrInt::from(pattern.chars().count()) {
        let pc: Option<String> = {
            let sifr_generated_string_chars = pattern.chars().collect::<Vec<char>>();
            let sifr_generated_string_index = pi.clone();
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_string_chars.len());
            sifr_generated_string_chars
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(pc) = pc {
            if pc == "*" {
                pi = &pi + &SifrInt::from_i64(1);
                if &pi == &SifrInt::from(pattern.chars().count()) {
                    return true;
                }
                let mut j: SifrInt = ni.clone();
                while &j <= &SifrInt::from(name.chars().count()) {
                    if sifr_generated_match(name, j.clone(), pattern, pi.clone()) {
                        return true;
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                return false;
            }
            if &ni >= &SifrInt::from(name.chars().count()) {
                return false;
            }
            if pc != "?" {
                let nc: Option<String> = {
                    let sifr_generated_string_chars = name.chars().collect::<Vec<char>>();
                    let sifr_generated_string_index = ni.clone();
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_string_chars.len());
                    sifr_generated_string_chars
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                if let Some(nc) = nc {
                    if nc != pc {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            ni = &ni + &SifrInt::from_i64(1);
            pi = &pi + &SifrInt::from_i64(1);
        } else {
            return false;
        }
    }
    &ni == &SifrInt::from(name.chars().count())
}
fn sifr_generated_translate_literal(ch: &str) -> String {
    if ch == "." {
        return "\\.".to_string();
    }
    if ch == "^" {
        return "\\^".to_string();
    }
    if ch == "$" {
        return "\\$".to_string();
    }
    if ch == "+" {
        return "\\+".to_string();
    }
    if ch == "(" {
        return "\\(".to_string();
    }
    if ch == ")" {
        return "\\)".to_string();
    }
    if ch == "{" {
        return "\\{".to_string();
    }
    if ch == "}" {
        return "\\}".to_string();
    }
    if ch == "[" {
        return "\\[".to_string();
    }
    if ch == "]" {
        return "\\]".to_string();
    }
    if ch == "|" {
        return "\\|".to_string();
    }
    if ch == "\\" {
        return "\\\\".to_string();
    }
    {
        let mut sifr_generated_concat: String = String::with_capacity(ch.len());
        sifr_generated_concat.push_str(ch);
        sifr_generated_concat.push_str("");
        sifr_generated_concat
    }
}
fn translate(pattern: &str) -> String {
    let sifr_generated_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
    let mut body: String = String::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(sifr_generated_chars_pattern.len()) {
        let ch: Option<String> = {
            let sifr_generated_string_index = i.clone();
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_pattern.len());
            sifr_generated_chars_pattern
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(ch) = ch {
            if ch == "*" {
                body.push_str(".*");
            } else if ch == "?" {
                body.push('.');
            } else {
                body.push_str(sifr_generated_translate_literal(&ch).as_str());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    {
        let mut sifr_generated_concat: String = String::with_capacity(4usize + body.len() + 3usize);
        sifr_generated_concat.push_str("(?s:");
        sifr_generated_concat.push_str(body.as_str());
        sifr_generated_concat.push_str(")\\z");
        sifr_generated_concat
    }
}
fn html_escape(s: &str) -> String {
    ::sifr_stdlib::html::html_escape(s)
}
fn html_unescape(s: &str) -> String {
    ::sifr_stdlib::html::html_unescape(s)
}
fn escape(s: &str, quote: bool) -> String {
    let escaped: String = html_escape(s);
    if quote {
        return escaped;
    }
    escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
}
fn unescape(s: &str) -> String {
    html_unescape(s)
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let template: SifrGeneratedStdlibSifrX2estringX2eTemplate =
        SifrGeneratedStdlibSifrX2estringX2eTemplate::new("Hello $name, mode=${mode}".to_string());
    let mut rendered_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let rendered: String = template.substitute(&{
            let mut sifr_generated_dict = HashMap::new();
            sifr_generated_dict.insert("name".to_string(), "Sifr".to_string());
            sifr_generated_dict.insert("mode".to_string(), "c2".to_string());
            sifr_generated_dict
        })?;
        rendered_ok = rendered == "Hello Sifr, mode=c2";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(rendered_ok);
    let formatter: SifrGeneratedStdlibSifrX2estringX2eFormatter =
        SifrGeneratedStdlibSifrX2estringX2eFormatter::new();
    let mut rendered_fmt_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let rendered_fmt_value_a80ff414b6eb8e54: String =
            formatter.format(&"Status {label}: {status}".to_string(), &{
                let mut sifr_generated_dict = HashMap::new();
                sifr_generated_dict.insert("label".to_string(), "c2".to_string());
                sifr_generated_dict.insert("status".to_string(), "ok".to_string());
                sifr_generated_dict
            })?;
        rendered_fmt_ok = rendered_fmt_value_a80ff414b6eb8e54 == "Status c2: ok";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(rendered_fmt_ok);
    let wrapper: SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper =
        SifrGeneratedStdlibSifrX2etextwrapX2eTextWrapper::new(
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
    let wrapped_value_2bd4345c4f3b90ce: Vec<String> = wrapper.wrap(&"alpha beta gamma".to_string());
    assert_eq!(
        format!("{wrapped_value_2bd4345c4f3b90ce:?}"),
        "[\"> alpha\", \".. beta\", \".. gamma\"]"
    );
    let encoded: String = b64encode(&"hello".to_string());
    let mut decoded_ok: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(&encoded)?;
        decoded_ok = decoded == "hello";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(decoded_ok);
    let escaped: String = escape(&"<b>safe</b>".to_string(), true);
    assert_eq!(unescape(&escaped), "<b>safe</b>");
    assert!(fnmatch(&"report.txt".to_string(), &"*.txt".to_string()));
    assert_eq!(translate(&"*.txt".to_string()), "(?s:.*\\.txt)\\z");
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0,
    > = (|| {
        let ratio: f64 = SifrGeneratedStdlibSifrX2edifflibX2eSequenceMatcher::new(
                "abcd".to_string(),
                "abed".to_string(),
            )
            .ratio()
            .map_err(|sifr_generated_e| match sifr_generated_e {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })?;
        assert!(ratio > 0.4_f64);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = sifr_generated_try_variant_error.clone();
                assert!(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                assert!(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                assert!(false);
            }
        }
    }
    let mut month_label_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let month_label: String =
            SifrGeneratedStdlibSifrX2ecalendarX2eTextCalendar::new(SifrInt::from_i64(0))
                .formatmonthname(
                    &SifrInt::from_i64(2024),
                    &SifrInt::from_i64(2),
                    &SifrInt::from_i64(0),
                )?;
        month_label_ok = month_label == "February 2024";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(month_label_ok);
}

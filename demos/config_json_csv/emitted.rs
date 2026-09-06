// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        JSONDecodeError, ParseError, SifrGeneratedStdlibSifrX2econfigparserX2eParsingError,
        SifrGeneratedStdlibSifrX2ecsvX2eDialect, SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry,
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn sifr_generated_const_44454641554c5453454354() -> String {
        "DEFAULT".to_string()
    }
    pub(crate) fn sifr_generated_default_section() -> String {
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(sifr_generated_const_44454641554c5453454354().len());
            sifr_generated_concat.push_str(sifr_generated_const_44454641554c5453454354().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_normalize_option(option: &str) -> String {
        option.to_lowercase().trim().to_string()
    }
    pub(crate) fn sifr_generated_copy_optional_str(value: &Option<String>) -> Option<String> {
        value.clone()
    }
    pub(crate) fn sifr_generated_has_option_key(
        values: &HashMap<String, Option<String>>,
        key: &str,
    ) -> bool {
        for current_key in values.keys().cloned() {
            if current_key == *key {
                return true;
            }
        }
        false
    }
    pub(crate) fn sifr_generated_lookup_option(
        values: &HashMap<String, Option<String>>,
        key: &str,
    ) -> Option<String> {
        for (current_key, current_value) in values
            .iter()
            .map(|sifr_generated_kv| (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if current_key == *key {
                return sifr_generated_copy_optional_str(&current_value);
            }
        }
        None
    }
    pub(crate) fn sifr_generated_copy_values(
        values: &HashMap<String, Option<String>>,
    ) -> HashMap<String, Option<String>> {
        let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
        for (key, value) in values
            .iter()
            .map(|sifr_generated_kv| (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone()))
            .collect::<Vec<_>>()
        {
            {
                let sifr_generated_assign_value = sifr_generated_copy_optional_str(&value);
                {
                    let sifr_generated_assign_key = key.to_owned();
                    copied.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        copied
    }
    pub(crate) fn sifr_generated_find_delimiter(line: &str) -> Option<String> {
        if line.contains(&"=".to_string()) {
            return Some("=".to_string());
        }
        if line.contains(&":".to_string()) {
            return Some(":".to_string());
        }
        None
    }
    pub(crate) fn sifr_generated_split_option_line(
        line: &str,
        allow_no_value: bool,
        line_no: SifrInt,
    ) -> Result<(String, Option<String>), SifrGeneratedStdlibSifrX2econfigparserX2eParsingError>
    {
        let delimiter: Option<String> = sifr_generated_find_delimiter(line);
        let Some(delimiter_value_894f6deb0b90819a) = delimiter else {
            if allow_no_value {
                return Ok((line.trim().to_string(), None));
            }
            return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                line_no.clone(),
                "expected key=value or key:value entry".to_string(),
            ));
        };
        let parts: Vec<String> = if &SifrInt::from_i64(1) < &0 {
            line.split(&delimiter_value_894f6deb0b90819a)
                .map(::std::string::ToString::to_string)
                .collect::<Vec<String>>()
        } else {
            line.splitn(
                (SifrInt::from_i64(1) + SifrInt::from_i64(1))
                    .clamp_slice_bound(line.len().saturating_add(1usize)),
                &delimiter_value_894f6deb0b90819a,
            )
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>()
        };
        if &SifrInt::from(parts.len()) != &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                line_no.clone(),
                "invalid option line".to_string(),
            ));
        }
        let raw_key: Option<String> = {
            let sifr_generated_checked_read_collection = &parts;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let raw_value: Option<String> = {
            let sifr_generated_checked_read_collection = &parts;
            let sifr_generated_checked_read_index = SifrInt::from_i64(1);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(raw_key_value_34bc0b643eda6241) = raw_key else {
            return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                line_no.clone(),
                "option name is missing".to_string(),
            ));
        };
        let key: String = sifr_generated_normalize_option(&raw_key_value_34bc0b643eda6241);
        if key.is_empty() {
            return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                line_no.clone(),
                "option name is empty".to_string(),
            ));
        }
        let Some(raw_value) = raw_value else {
            return Ok((key, None));
        };
        let stripped_value: Option<String> = Some(raw_value.trim().to_string());
        Ok((key, stripped_value))
    }
    pub(crate) fn sifr_generated_char_at(text: &str, index: SifrInt) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if &index < &SifrInt::from_i64(0)
            || &index >= &SifrInt::from(sifr_generated_chars_text.len())
        {
            return String::new();
        }
        let ch: Option<String> = {
            let sifr_generated_string_index = index.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        let Some(ch) = ch else {
            return String::new();
        };
        ch
    }
    pub(crate) fn sifr_generated_resolve_interpolation(
        value: &str,
        merged: &HashMap<String, Option<String>>,
        depth: SifrInt,
    ) -> String {
        if &depth >= &SifrInt::from_i64(8) {
            return {
                let mut sifr_generated_concat: String = String::with_capacity(value.len());
                sifr_generated_concat.push_str(value.as_ref());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        if !value.contains(&"%(".to_string()) {
            return {
                let mut sifr_generated_concat: String = String::with_capacity(value.len());
                sifr_generated_concat.push_str(value.as_ref());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        let mut result: String = String::new();
        let mut replaced: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(value.chars().count()) {
            let ch: String = sifr_generated_char_at(value, i.clone());
            if ch == "%"
                && &(&i + &SifrInt::from_i64(1)) < &SifrInt::from(value.chars().count())
                && sifr_generated_char_at(value, &i + &SifrInt::from_i64(1)) == "("
            {
                let mut j: SifrInt = &i + &SifrInt::from_i64(2);
                let mut key: String = String::new();
                let mut matched: bool = false;
                while &j < &SifrInt::from(value.chars().count()) {
                    let part: String = sifr_generated_char_at(value, j.clone());
                    if part == ")"
                        && &(&j + &SifrInt::from_i64(1)) < &SifrInt::from(value.chars().count())
                        && sifr_generated_char_at(value, &j + &SifrInt::from_i64(1)) == "s"
                    {
                        matched = true;
                        let normalized_key: String = sifr_generated_normalize_option(&key);
                        let replacement: Option<String> =
                            sifr_generated_lookup_option(merged, &normalized_key);
                        if replacement.is_none() {
                            result.push_str("%(");
                            result.push_str(key.as_str());
                            result.push_str(")s");
                        } else if let Some(replacement) = replacement {
                            replaced = true;
                            result.push_str(replacement.as_str());
                        }
                        i = &j + &SifrInt::from_i64(2);
                        break;
                    }
                    key.push_str(part.as_str());
                    j = &j + &SifrInt::from_i64(1);
                }
                if matched {
                    continue;
                }
            }
            result.push_str(ch.as_str());
            i = &i + &SifrInt::from_i64(1);
        }
        if replaced {
            return sifr_generated_resolve_interpolation(
                &result,
                merged,
                &depth + &SifrInt::from_i64(1),
            );
        }
        result
    }
    pub(crate) const fn sifr_generated_const_51554f54455f4e4f4e45() -> SifrInt {
        SifrInt::from_i64(3)
    }
    pub(crate) fn excel() -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            ",".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            "\n".to_string(),
            SifrInt::from_i64(0),
        )
    }
    pub(crate) fn excel_tab() -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            "\t".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            "\n".to_string(),
            SifrInt::from_i64(0),
        )
    }
    pub(crate) fn unix_dialect() -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            ",".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            "\n".to_string(),
            SifrInt::from_i64(1),
        )
    }
    pub(crate) fn sifr_generated_copy_dialect(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            dialect.delimiter.clone().to_string(),
            dialect.quotechar.clone().to_string(),
            dialect.escapechar.clone().to_string(),
            dialect.doublequote,
            dialect.skipinitialspace,
            dialect.lineterminator.clone().to_string(),
            dialect.quoting.clone(),
        )
    }
    pub(crate) fn dialect_registry() -> SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry::new()
    }
    pub(crate) fn sifr_generated_validate_char(name: &str, value: &str) {
        let _ = name.to_owned();
        let _ = value.to_owned();
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn sifr_generated_resolve_dialect(
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &str,
        quoting: SifrInt,
    ) -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        let Some(dialect) = dialect.as_ref() else {
            return SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
                delimiter.to_owned(),
                quotechar.to_owned(),
                escapechar.to_owned(),
                doublequote,
                skipinitialspace,
                lineterminator.to_owned(),
                quoting.clone(),
            );
        };
        sifr_generated_copy_dialect(dialect)
    }
    pub(crate) fn sifr_generated_quotechar_value(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let quotechar: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(dialect.quotechar.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if quotechar.as_str() == String::new().as_str() {
            return "\"".to_string();
        }
        quotechar
    }
    pub(crate) fn sifr_generated_append_field(row: &mut Vec<String>, field: String) {
        row.push(field.to_string());
    }
    pub(crate) fn sifr_generated_append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
        rows.push(row);
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn parse_csv(
        text: &str,
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> Vec<Vec<String>> {
        let quotechar = quotechar.to_owned();
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            &quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting.clone(),
        );
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut row: Vec<String> = Vec::new();
        let mut field: String = String::new();
        let mut in_quotes: bool = false;
        let mut field_started: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
            let ch_value: String = sifr_generated_char_at(text, i.clone());
            if in_quotes {
                if !resolved.escapechar.clone().is_empty()
                    && ch_value == resolved.escapechar.clone()
                {
                    if &(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                    {
                        let escaped_value: String =
                            sifr_generated_char_at(text, &i + &SifrInt::from_i64(1));
                        field.push_str(escaped_value.as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    field.push_str(ch_value.as_str());
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                if !resolved.quotechar.clone().is_empty() && ch_value == resolved.quotechar.clone()
                {
                    let quotechar: String = sifr_generated_quotechar_value(&resolved);
                    if resolved.doublequote
                        && &(&i + &SifrInt::from_i64(1))
                            < &SifrInt::from(sifr_generated_chars_text.len())
                        && sifr_generated_char_at(text, &i + &SifrInt::from_i64(1)) == quotechar
                    {
                        field.push_str(quotechar.as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    in_quotes = false;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                field.push_str(ch_value.as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if !field_started && resolved.skipinitialspace && ch_value == " " {
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if !resolved.escapechar.clone().is_empty() && ch_value == resolved.escapechar.clone() {
                if &(&i + &SifrInt::from_i64(1)) < &SifrInt::from(sifr_generated_chars_text.len()) {
                    let escaped_plain_value: String =
                        sifr_generated_char_at(text, &i + &SifrInt::from_i64(1));
                    field.push_str(escaped_plain_value.as_str());
                    field_started = true;
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                field.push_str(ch_value.as_str());
                field_started = true;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if &resolved.quoting.clone() != &sifr_generated_const_51554f54455f4e4f4e45()
                && !resolved.quotechar.clone().is_empty()
            {
                let quotechar2_value_123324c155e57c27: String =
                    sifr_generated_quotechar_value(&resolved);
                if ch_value == quotechar2_value_123324c155e57c27 {
                    in_quotes = true;
                    field_started = true;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
            }
            if ch_value == resolved.delimiter.clone() {
                sifr_generated_append_field(&mut row, field);
                field = String::new();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if ch_value == "\n" || ch_value == "\r" {
                if ch_value == "\r"
                    && &(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                    && sifr_generated_char_at(text, &i + &SifrInt::from_i64(1)) == "\n"
                {
                    i = &i + &SifrInt::from_i64(1);
                }
                if &SifrInt::from(row.len()) == &SifrInt::from_i64(0) && field.is_empty() {
                    sifr_generated_append_row(&mut rows, Vec::new());
                } else {
                    sifr_generated_append_field(&mut row, field);
                    sifr_generated_append_row(&mut rows, row);
                }
                row = Vec::new();
                field = String::new();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            field.push_str(ch_value.as_str());
            field_started = true;
            i = &i + &SifrInt::from_i64(1);
        }
        let _ = in_quotes;
        if &SifrInt::from(row.len()) > &SifrInt::from_i64(0) || !field.is_empty() {
            sifr_generated_append_field(&mut row, field);
            sifr_generated_append_row(&mut rows, row);
        }
        rows
    }
    pub(crate) fn json_load_tokens(text: &str) -> Result<Vec<String>, JSONDecodeError> {
        ::sifr_stdlib::json::json_load_tokens(text).map_err(|sifr_generated_bridge_error| {
            JSONDecodeError {
                message: sifr_generated_bridge_error.message().to_string(),
                line: SifrInt::from(sifr_generated_bridge_error.line()),
                column: SifrInt::from(sifr_generated_bridge_error.column()),
            }
        })
    }
    pub(crate) fn json_dump_tokens(tokens: &[String]) -> String {
        ::sifr_stdlib::json::json_dump_tokens(tokens)
    }
    #[derive(Debug, Clone)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(JSONDecodeError),
        SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(ParseError),
    }
    impl From<JSONDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: JSONDecodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                value,
            )
        }
    }
    impl From<ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: ParseError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn from_int(value: SifrInt) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let int_value: Option<SifrInt> = Some(value.clone());
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
            "int".to_string(),
            None,
            int_value.clone(),
            None,
            None,
        )
    }
    pub(crate) fn from_str(value: &str) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let str_value: Option<String> = Some({
            let mut sifr_generated_concat: String = String::with_capacity(value.len());
            sifr_generated_concat.push_str(value);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        });
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
            "str".to_string(),
            None,
            None,
            None,
            str_value,
        )
    }
    pub(crate) fn sifr_generated_append_object_item(
        mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        key: String,
        item_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        value.object_items.push((key, item_value));
        value
    }
    pub(crate) fn from_object(
        items: &[(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)],
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                "object".to_string(),
                None,
                None,
                None,
                None,
            );
        for (key, item_value) in items.iter().cloned() {
            value = sifr_generated_append_object_item(value, key, item_value);
        }
        value
    }
    pub(crate) fn sifr_generated_json_token_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<String, JSONDecodeError> {
        let value: Option<String> = {
            let sifr_generated_checked_read_collection = &tokens;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return Err(JSONDecodeError::new(
                "JSON bridge payload ended unexpectedly".to_string(),
            ));
        };
        Ok({
            let mut sifr_generated_concat: String =
                String::with_capacity(value_value_7ce4fd9430e80cea.len());
            sifr_generated_concat.push_str(value_value_7ce4fd9430e80cea.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        })
    }
    pub(crate) fn sifr_generated_json_token_int(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<SifrInt, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrInt, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0,
                )?;
            let parsed: SifrInt = SifrInt::parse_decimal(
                    &token_value_26c4b17d50b3c152,
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(JSONDecodeError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_json_token_float(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<f64, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<f64, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0,
                )?;
            let parsed: f64 = token_value_26c4b17d50b3c152
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(JSONDecodeError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid float metadata".to_string(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_json_decode_bool_token(
        value: &str,
    ) -> Result<bool, JSONDecodeError> {
        if value == "true" {
            return Ok(true);
        }
        if value == "false" {
            return Ok(false);
        }
        Err(JSONDecodeError::new(
            "JSON bridge payload has invalid bool metadata".to_string(),
        ))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_json_decode_value_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let tag: String = sifr_generated_json_token_at(tokens, index.clone())?;
            let payload_index: SifrInt = &index + &SifrInt::from_i64(1);
            if tag == "null" {
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "null".to_string(),
                        None,
                        None,
                        None,
                        None,
                    ),
                    payload_index.clone(),
                )));
            }
            if tag == "bool" {
                let bool_token: String =
                    sifr_generated_json_token_at(tokens, payload_index.clone())?;
                let bool_value: bool = sifr_generated_json_decode_bool_token(&bool_token)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "int" {
                let int_value: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "int".to_string(),
                        None,
                        Some(int_value),
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "float" {
                let float_value: f64 =
                    sifr_generated_json_token_float(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "str" {
                let str_value: String =
                    sifr_generated_json_token_at(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "array" {
                let array_count: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                if &array_count < &SifrInt::from_i64(0) {
                    return Err(JSONDecodeError::new(
                        "JSON bridge payload has invalid array length".to_string(),
                    ));
                }
                let mut array_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "array".to_string(),
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &array_count {
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(tokens, next_index.clone())?;
                    array_value.array_items.push(item_result.0.clone());
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((array_value, next_index.clone())));
            }
            if tag == "object" {
                let object_count: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                if &object_count < &SifrInt::from_i64(0) {
                    return Err(JSONDecodeError::new(
                        "JSON bridge payload has invalid object length".to_string(),
                    ));
                }
                let mut object_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "object".to_string(),
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &object_count {
                    let key: String = sifr_generated_json_token_at(tokens, next_index.clone())?;
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(
                            tokens,
                            &next_index + &SifrInt::from_i64(1),
                        )?;
                    object_value.object_items.push((key, item_result.0.clone()));
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((object_value, next_index)));
            }
            Err(JSONDecodeError::new({
                let mut sifr_generated_concat: String = String::with_capacity(43usize + tag.len());
                sifr_generated_concat.push_str("JSON bridge payload has unknown value tag: ");
                sifr_generated_concat.push_str(tag.as_str());
                sifr_generated_concat
            }))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(JSONDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_json_decode_tokens(
        tokens: &[String],
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let decoded: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                sifr_generated_json_decode_value_at(tokens, SifrInt::from_i64(0))?;
            if &decoded.1.clone() != &SifrInt::from(tokens.len()) {
                return Err(JSONDecodeError::new(
                    "JSON bridge payload has trailing data".to_string(),
                ));
            }
            Ok(Ok(decoded.0.clone()))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(JSONDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_json_append_tokens(
        mut tokens: Vec<String>,
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        tokens.push(value.kind.clone().to_string());
        if value.kind.clone() == "bool" {
            let bool_value: Option<bool> = value.bool_value;
            if bool_value.is_none() {
                tokens.push("false".to_string());
            } else if let Some(bool_value) = bool_value {
                tokens.push(bool_value.to_string().to_lowercase());
            }
        } else if value.kind.clone() == "int" {
            let int_value: Option<SifrInt> = value.int_value.clone();
            if int_value.is_none() {
                tokens.push("0".to_string());
            } else if let Some(int_value) = int_value.clone() {
                tokens.push(int_value.to_string());
            }
        } else if value.kind.clone() == "float" {
            let float_value: Option<f64> = value.float_value;
            if float_value.is_none() {
                tokens.push("0.0".to_string());
            } else if let Some(float_value) = float_value {
                tokens.push(float_value.to_string());
            }
        } else if value.kind.clone() == "str" {
            let str_value: Option<String> = value.as_str();
            if str_value.is_none() {
                tokens.push(String::new());
            } else if let Some(str_value) = str_value {
                tokens.push(str_value);
            }
        } else if value.kind.clone() == "array" {
            tokens.push(SifrInt::from(value.array_items.len()).to_string());
            for item in value.array_items.iter().cloned() {
                tokens = sifr_generated_json_append_tokens(tokens, &item);
            }
        } else if value.kind.clone() == "object" {
            tokens.push(SifrInt::from(value.object_items.len()).to_string());
            for (key, item_value) in value.object_items.iter().cloned() {
                tokens.push(key.to_owned());
                tokens = sifr_generated_json_append_tokens(tokens, &item_value);
            }
        }
        tokens
    }
    pub(crate) fn sifr_generated_json_bridge_tokens(
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        let tokens: Vec<String> = Vec::new();
        sifr_generated_json_append_tokens(tokens, value)
    }
    pub(crate) fn sifr_generated_decode_json(
        s: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let tokens: Vec<String> = json_load_tokens(s)?;
            Ok(sifr_generated_json_decode_tokens(&tokens))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn dumps(
        value: &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    ) -> String {
        match value {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                value,
            ) => json_dump_tokens(&sifr_generated_json_bridge_tokens(value)),
        }
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2econfigparserX2eParsingError {
        pub line: SifrInt,
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eParsingError {
        #[must_use]
        pub fn new(line: SifrInt, message: String) -> Self {
            let sifr_generated_field_value_bf4ba5ad694f5907_6c696e65: SifrInt = line.clone();
            let sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765: String = message;
            Self {
                line: sifr_generated_field_value_bf4ba5ad694f5907_6c696e65,
                message: sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765,
            }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2econfigparserX2eParsingError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("ParsingError")
                .field("line", &self.line)
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2econfigparserX2eParsingError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2econfigparserX2eParsingError {}
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        pub defaults: HashMap<String, Option<String>>,
        pub sections: HashMap<String, HashMap<String, Option<String>>>,
        pub strict: bool,
        pub allow_no_value: bool,
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        #[must_use]
        pub fn new(
            defaults: Option<HashMap<String, Option<String>>>,
            strict: bool,
            allow_no_value: bool,
        ) -> Self {
            let mut defaults_map: HashMap<String, Option<String>> = HashMap::from([]);
            let sections_map: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
            if let Some(defaults) = defaults {
                for (key, value) in defaults
                    .iter()
                    .map(|sifr_generated_kv| {
                        (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone())
                    })
                    .collect::<Vec<_>>()
                {
                    let normalized: String = sifr_generated_normalize_option(&key);
                    {
                        let sifr_generated_assign_value = sifr_generated_copy_optional_str(&value);
                        {
                            let sifr_generated_assign_key = normalized.to_owned();
                            defaults_map
                                .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
            let sifr_generated_field_value_7055edd8fab866f4_737472696374: bool = strict;
            let sifr_generated_field_value_b80c2bb7ade68286_616c6c6f775f6e6f5f76616c7565: bool =
                allow_no_value;
            let sifr_generated_field_value_89dfc9ef20d923a0_5f64656661756c7473: HashMap<
                String,
                Option<String>,
            > = defaults_map;
            let sifr_generated_field_value_2b70bd8b78964186_5f73656374696f6e73: HashMap<
                String,
                HashMap<String, Option<String>>,
            > = sections_map;
            Self {
                strict: sifr_generated_field_value_7055edd8fab866f4_737472696374,
                allow_no_value:
                    sifr_generated_field_value_b80c2bb7ade68286_616c6c6f775f6e6f5f76616c7565,
                defaults: sifr_generated_field_value_89dfc9ef20d923a0_5f64656661756c7473,
                sections: sifr_generated_field_value_2b70bd8b78964186_5f73656374696f6e73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        ///# Errors
        ///Returns the typed error produced by this operation.
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn read_string(
            &mut self,
            text: &str,
        ) -> Result<(), SifrGeneratedStdlibSifrX2econfigparserX2eParsingError> {
            let mut current_section: String = String::new();
            let default_section: String = sifr_generated_default_section();
            for (line_no, raw_line) in Box::new(
                text.split('\n')
                    .map(::std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .into_iter()
                    .enumerate()
                    .map(|sifr_generated_pair| {
                        (
                            SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(1),
                            sifr_generated_pair.1,
                        )
                    }),
            ) {
                let line: String = raw_line.trim().to_string();
                if line.is_empty() || line.starts_with("#") || line.starts_with(";") {
                    continue;
                }
                if line.starts_with("[") && line.ends_with("]") {
                    let section_name: String = {
                        let sifr_generated_slice_src = line.chars().collect::<Vec<char>>();
                        let sifr_generated_slice_len = sifr_generated_slice_src.len();
                        let sifr_generated_slice_start =
                            SifrInt::from_i64(1).clamp_slice_bound(sifr_generated_slice_len);
                        let sifr_generated_slice_stop =
                            (&SifrInt::from(sifr_generated_slice_src.len())
                                - &SifrInt::from_i64(1))
                                .clamp_slice_bound(sifr_generated_slice_len);
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied()
                            .collect::<String>()
                    }
                    .trim()
                    .to_string();
                    if section_name.is_empty() {
                        return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                            line_no.clone(),
                            "section name is empty".to_string(),
                        ));
                    }
                    if section_name == default_section {
                        current_section = sifr_generated_default_section();
                        continue;
                    }
                    if self.strict && self.sections.contains_key(&section_name) {
                        return Err(SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                            line_no.clone(),
                            format!("duplicate section: {section_name}"),
                        ));
                    }
                    current_section = {
                        let mut sifr_generated_concat: String =
                            String::with_capacity(section_name.len());
                        sifr_generated_concat.push_str(section_name.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                    if !self.sections.contains_key(&section_name) {
                        {
                            let sifr_generated_assign_value = HashMap::new();
                            {
                                let sifr_generated_assign_key = section_name.to_owned();
                                self.sections
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                    continue;
                }
                let sifr_generated_try_res: Result<
                    (),
                    SifrGeneratedStdlibSifrX2econfigparserX2eParsingError,
                > = (|| {
                    let parsed_option_pair: (String, Option<String>) =
                        sifr_generated_split_option_line(
                            &line,
                            self.allow_no_value,
                            line_no.clone(),
                        )?;
                    let (option_name, option_value) = parsed_option_pair;
                    let _chars_option_name: Vec<char> = option_name.chars().collect::<Vec<char>>();
                    if current_section.is_empty() || current_section == default_section {
                        {
                            let sifr_generated_assign_value =
                                sifr_generated_copy_optional_str(&option_value);
                            {
                                let sifr_generated_assign_key = option_name.to_owned();
                                self.defaults
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let section_key: String = {
                            let mut sifr_generated_concat: String =
                                String::with_capacity(current_section.len());
                            sifr_generated_concat.push_str(current_section.as_str());
                            sifr_generated_concat.push_str("");
                            sifr_generated_concat
                        };
                        for (section_name, section_values) in self
                            .sections
                            .iter()
                            .map(|sifr_generated_kv| {
                                (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone())
                            })
                            .collect::<Vec<_>>()
                        {
                            if section_name != section_key {
                                continue;
                            }
                            if self.strict
                                && sifr_generated_has_option_key(&section_values, &option_name)
                            {
                                return Err(
                                    SifrGeneratedStdlibSifrX2econfigparserX2eParsingError::new(
                                        line_no.clone(),
                                        format!("duplicate option: {option_name}"),
                                    ),
                                );
                            }
                            let mut updated_section: HashMap<String, Option<String>> =
                                sifr_generated_copy_values(&section_values);
                            {
                                let sifr_generated_assign_value =
                                    sifr_generated_copy_optional_str(&option_value);
                                {
                                    let sifr_generated_assign_key = option_name.to_owned();
                                    updated_section.insert(
                                        sifr_generated_assign_key,
                                        sifr_generated_assign_value,
                                    );
                                }
                            }
                            {
                                let sifr_generated_assign_value = updated_section.clone();
                                {
                                    let sifr_generated_assign_key = section_name.to_owned();
                                    self.sections.insert(
                                        sifr_generated_assign_key,
                                        sifr_generated_assign_value,
                                    );
                                }
                            }
                            break;
                        }
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let e = sifr_generated_try_err.clone();
                    return Err(e);
                }
            }
            Ok(())
        }
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        #[must_use]
        pub fn has_section(&self, section: &str) -> bool {
            self.sections.contains_key(section)
        }
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        #[must_use]
        pub fn sifr_generated_merged_section(
            &self,
            section: &str,
        ) -> HashMap<String, Option<String>> {
            let mut merged: HashMap<String, Option<String>> =
                sifr_generated_copy_values(&self.defaults);
            let default_section: String = sifr_generated_default_section();
            if *section == default_section {
                return merged;
            }
            for (section_name, section_values) in self
                .sections
                .iter()
                .map(|sifr_generated_kv| (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                for (option, value) in section_values
                    .iter()
                    .map(|sifr_generated_kv| {
                        (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone())
                    })
                    .collect::<Vec<_>>()
                {
                    {
                        let sifr_generated_assign_value = sifr_generated_copy_optional_str(&value);
                        {
                            let sifr_generated_assign_key = option.to_owned();
                            merged.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
                return merged;
            }
            merged
        }
    }
    impl SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser {
        #[must_use]
        pub fn get(
            &self,
            section: &str,
            option: &str,
            fallback: &Option<String>,
            raw: bool,
        ) -> Option<String> {
            let normalized: String = sifr_generated_normalize_option(option);
            let merged: HashMap<String, Option<String>> =
                self.sifr_generated_merged_section(section);
            let default_section: String = sifr_generated_default_section();
            if *section == default_section {
                if !sifr_generated_has_option_key(&merged, &normalized) {
                    return sifr_generated_copy_optional_str(fallback);
                }
                let raw_value: Option<String> = sifr_generated_lookup_option(&merged, &normalized);
                let raw_value = raw_value?;
                if raw {
                    return Some(raw_value);
                }
                return Some(sifr_generated_resolve_interpolation(
                    &raw_value,
                    &merged,
                    SifrInt::from_i64(0),
                ));
            }
            if !self.has_section(section) {
                if sifr_generated_has_option_key(&self.defaults, &normalized) {
                    let default_value: Option<String> =
                        sifr_generated_lookup_option(&self.defaults, &normalized);
                    let default_value = default_value?;
                    if raw {
                        return Some(default_value);
                    }
                    return Some(sifr_generated_resolve_interpolation(
                        &default_value,
                        &merged,
                        SifrInt::from_i64(0),
                    ));
                }
                return sifr_generated_copy_optional_str(fallback);
            }
            if !sifr_generated_has_option_key(&merged, &normalized) {
                return sifr_generated_copy_optional_str(fallback);
            }
            let raw_value2: Option<String> = sifr_generated_lookup_option(&merged, &normalized);
            let raw_value2_value_7ff8214b5ccf9553 = raw_value2?;
            if raw {
                return Some(raw_value2_value_7ff8214b5ccf9553);
            }
            Some(sifr_generated_resolve_interpolation(
                &raw_value2_value_7ff8214b5ccf9553,
                &merged,
                SifrInt::from_i64(0),
            ))
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2econfigparserX2eRawConfigParser {
        pub configparser: SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser,
    }
    impl ::std::ops::Deref for SifrGeneratedStdlibSifrX2econfigparserX2eRawConfigParser {
        type Target = SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser;
        fn deref(&self) -> &Self::Target {
            &self.configparser
        }
    }
    impl ::std::ops::DerefMut for SifrGeneratedStdlibSifrX2econfigparserX2eRawConfigParser {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.configparser
        }
    }
    impl ::std::convert::From<SifrGeneratedStdlibSifrX2econfigparserX2eRawConfigParser>
        for SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser
    {
        fn from(value: SifrGeneratedStdlibSifrX2econfigparserX2eRawConfigParser) -> Self {
            value.configparser
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        pub delimiter: String,
        pub quotechar: String,
        pub escapechar: String,
        pub doublequote: bool,
        pub skipinitialspace: bool,
        pub lineterminator: String,
        pub quoting: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        #[must_use]
        pub fn new(
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
            quoting: SifrInt,
        ) -> Self {
            let mut resolved_quoting: SifrInt = quoting.clone();
            sifr_generated_validate_char(&"delimiter".to_string(), &delimiter);
            if !quotechar.is_empty() {
                sifr_generated_validate_char(&"quotechar".to_string(), &quotechar);
            }
            if !escapechar.is_empty() {
                sifr_generated_validate_char(&"escapechar".to_string(), &escapechar);
            }
            if quotechar.is_empty()
                && &resolved_quoting != &sifr_generated_const_51554f54455f4e4f4e45()
            {
                resolved_quoting = sifr_generated_const_51554f54455f4e4f4e45().clone();
            }
            let sifr_generated_field_value_894f6deb0b90819a_64656c696d69746572: String = delimiter;
            let sifr_generated_field_value_071afb87ccff598f_71756f746563686172: String = quotechar;
            let sifr_generated_field_value_35712447096491ca_65736361706563686172: String =
                escapechar;
            let sifr_generated_field_value_0c828b579bd5cc5c_646f75626c6571756f7465: bool =
                doublequote;
            let sifr_generated_field_value_aed440ff683599d8_736b6970696e697469616c7370616365: bool =
                skipinitialspace;
            let sifr_generated_field_value_5421666eeec5d0d2_6c696e657465726d696e61746f72: String =
                lineterminator;
            let sifr_generated_field_value_7f757e185a85e280_71756f74696e67: SifrInt =
                resolved_quoting.clone();
            Self {
                delimiter: sifr_generated_field_value_894f6deb0b90819a_64656c696d69746572,
                quotechar: sifr_generated_field_value_071afb87ccff598f_71756f746563686172,
                escapechar: sifr_generated_field_value_35712447096491ca_65736361706563686172,
                doublequote: sifr_generated_field_value_0c828b579bd5cc5c_646f75626c6571756f7465,
                skipinitialspace:
                    sifr_generated_field_value_aed440ff683599d8_736b6970696e697469616c7370616365,
                lineterminator:
                    sifr_generated_field_value_5421666eeec5d0d2_6c696e657465726d696e61746f72,
                quoting: sifr_generated_field_value_7f757e185a85e280_71756f74696e67,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
                self.delimiter,
                self.quotechar,
                self.escapechar,
                self.doublequote,
                self.skipinitialspace,
                self.lineterminator,
                self.quoting
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        pub dialects: HashMap<String, SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        #[must_use]
        pub fn new() -> Self {
            let sifr_generated_field_value_434d24ee0415b059_5f6469616c65637473: HashMap<
                String,
                SifrGeneratedStdlibSifrX2ecsvX2eDialect,
            > = {
                let mut sifr_generated_dict = HashMap::new();
                sifr_generated_dict.insert("excel".to_string(), excel());
                sifr_generated_dict.insert("excel-tab".to_string(), excel_tab());
                sifr_generated_dict.insert("unix".to_string(), unix_dialect());
                sifr_generated_dict
            };
            Self {
                dialects: sifr_generated_field_value_434d24ee0415b059_5f6469616c65637473,
            }
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        pub fn register(&mut self, name: &str, dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect) {
            {
                let sifr_generated_assign_value = sifr_generated_copy_dialect(dialect);
                {
                    let sifr_generated_assign_key = {
                        let mut sifr_generated_concat: String = String::with_capacity(name.len());
                        sifr_generated_concat.push_str(name);
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                    self.dialects
                        .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        #[must_use]
        pub fn unregister(&mut self, name: &str) -> bool {
            if self.dialects.contains_key(name) {
                let _ = self.dialects.remove(name);
                return true;
            }
            false
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry {
        #[must_use]
        pub fn get(&self, name: &str) -> Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect> {
            let _checked_value_2 = self.dialects.get(name)?;
            for (key, value) in self
                .dialects
                .iter()
                .map(|sifr_generated_kv| (sifr_generated_kv.0.clone(), sifr_generated_kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if key != *name {
                    continue;
                }
                return Some(sifr_generated_copy_dialect(&value));
            }
            None
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2ereader {
        pub rows: Vec<Vec<String>>,
        pub pos: SifrInt,
        pub dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ereader {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            text: String,
            dialect: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            quoting: SifrInt,
        ) -> Self {
            let resolved_dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect =
                sifr_generated_resolve_dialect(
                    &dialect,
                    &delimiter,
                    &quotechar,
                    &escapechar,
                    doublequote,
                    skipinitialspace,
                    &"\n".to_string(),
                    quoting.clone(),
                );
            let rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &resolved_dialect.delimiter.clone().to_string(),
                &resolved_dialect.quotechar.clone().to_string(),
                &resolved_dialect.escapechar.clone().to_string(),
                resolved_dialect.doublequote,
                resolved_dialect.skipinitialspace,
                resolved_dialect.quoting.clone(),
            );
            let sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374: SifrGeneratedStdlibSifrX2ecsvX2eDialect = resolved_dialect;
            let sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773: Vec<Vec<String>> = rows;
            let sifr_generated_field_value_e04b9443eebba9b4_5f706f73: SifrInt =
                SifrInt::from_i64(0);
            Self {
                dialect: sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374,
                rows: sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773,
                pos: sifr_generated_field_value_e04b9443eebba9b4_5f706f73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ereader {
        #[must_use]
        pub fn rows(&self) -> Vec<Vec<String>> {
            let mut result: Vec<Vec<String>> = Vec::new();
            for row in self.rows.iter().cloned() {
                let mut copied: Vec<String> = Vec::new();
                for field in row.iter().cloned() {
                    copied.push(field.to_string());
                }
                result.push(copied.to_vec());
            }
            result
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<SifrInt>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub array_items: Box<Vec<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue>>,
        pub object_items: Box<Vec<(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)>>,
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
        ) -> Self {
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = kind;
            let sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565: Option<bool> =
                bool_value;
            let sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565: Option<SifrInt> =
                int_value.clone();
            let sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565: Option<f64> =
                float_value;
            let sifr_generated_field_value_100b36b139835e22_7374725f76616c7565: Option<String> =
                str_value;
            let sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73: Box<
                Vec<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue>,
            > = Box::default();
            let sifr_generated_field_value_4b0f6d30620fe831_6f626a6563745f6974656d73: Box<
                Vec<(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)>,
            > = Box::default();
            Self {
                kind: sifr_generated_field_value_ef9c96d721673243_6b696e64,
                bool_value: sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565,
                int_value: sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565,
                float_value: sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565,
                str_value: sifr_generated_field_value_100b36b139835e22_7374725f76616c7565,
                array_items: sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73,
                object_items: sifr_generated_field_value_4b0f6d30620fe831_6f626a6563745f6974656d73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn is_object(&self) -> bool {
            self.kind.clone() == "object"
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn as_str(&self) -> Option<String> {
            self.str_value.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn get(&self, key: &str) -> Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> {
            if !self.is_object() {
                return None;
            }
            for (item_key, item_value) in self.object_items.iter().cloned() {
                if item_key == *key {
                    return Some(item_value);
                }
            }
            None
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn keys(&self) -> Vec<String> {
            let mut result: Vec<String> = Vec::new();
            if !self.is_object() {
                return result;
            }
            for (item_key, _item_value) in self.object_items.iter().cloned() {
                result.push(item_key.to_owned());
            }
            result
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "{}", dumps(&
                SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(self
                .clone()))
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder {
        pub indent: Option<SifrInt>,
        pub sort_keys: bool,
        pub ensure_ascii: bool,
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder {
        #[must_use]
        pub fn new(indent: Option<SifrInt>, sort_keys: bool, ensure_ascii: bool) -> Self {
            let sifr_generated_field_value_7b4038f28be9d4fb_696e64656e74: Option<SifrInt> =
                indent.clone();
            let sifr_generated_field_value_61679c40b5c28a76_736f72745f6b657973: bool = sort_keys;
            let sifr_generated_field_value_120e4a8f8440c783_656e737572655f6173636969: bool =
                ensure_ascii;
            Self {
                indent: sifr_generated_field_value_7b4038f28be9d4fb_696e64656e74,
                sort_keys: sifr_generated_field_value_61679c40b5c28a76_736f72745f6b657973,
                ensure_ascii: sifr_generated_field_value_120e4a8f8440c783_656e737572655f6173636969,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder {
        #[must_use]
        pub fn encode(&self, value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue) -> String {
            let _: Option<SifrInt> = self.indent.clone();
            let _: bool = self.sort_keys;
            let _: bool = self.ensure_ascii;
            dumps(
                &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                    value.clone(),
                ),
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {}
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn decode(
            &self,
            s: &str,
        ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
            sifr_generated_decode_json(s)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
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
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl JSONDecodeError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self {
                message,
                line: SifrInt::from_i64(0),
                column: SifrInt::from_i64(0),
            }
        }
    }
    impl ::std::fmt::Display for JSONDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JSONDecodeError {}
    impl From<IOError> for Error {
        fn from(err: IOError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<JSONDecodeError> for Error {
        fn from(err: JSONDecodeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<
        crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eParsingError,
    > for Error {
        fn from(
            err: crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eParsingError,
        ) -> Self {
            Self::new(err.message)
        }
    }
}
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::JSONDecodeError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eParsingError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialect;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ereader;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
            crate::sifr_generated_project_nominals::JSONDecodeError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::JSONDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::JSONDecodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0;
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let encoder: SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder =
        SifrGeneratedStdlibSifrX2ejsonX2eJSONEncoder::new(Some(SifrInt::from_i64(2)), false, true);
    let decoder: SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder =
        SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder::new();
    let payload: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue = from_object(&vec![
        (
            "module".to_string(),
            from_str(&"config_json_csv".to_string()),
        ),
        ("version".to_string(), from_int(SifrInt::from_i64(1))),
    ]);
    let encoded_value_8b21351fd8aea299: String = encoder.encode(&payload);
    assert_eq!(
        encoded_value_8b21351fd8aea299.as_str(),
        "{\"module\":\"config_json_csv\",\"version\":1}"
            .to_string()
            .as_str()
    );
    let mut decoded_ok: bool = false;
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0,
    > = (|| {
        let decoded_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue = decoder
            .decode(&encoded_value_8b21351fd8aea299)
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0,
            )?;
        decoded_ok = decoded_value.to_string() == encoded_value_8b21351fd8aea299;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                let _ = e.message.clone().to_string();
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a228X3a5X3aclass15X3aJSONDecodeError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                let _ = e.message.clone().to_string();
            }
        }
    }
    assert!(decoded_ok);
    let mut parser: SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser =
        SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser::new(None, false, false);
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2econfigparserX2eParsingError> =
        (|| {
            (&mut parser).read_string(
                &"[DEFAULT]\nbase=/tmp\n[paths]\ncache=%(base)s/cache\n".to_string(),
            )?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        assert!(false);
    }
    assert_eq!(
        parser.get(&"paths".to_string(), &"cache".to_string(), &None, false),
        Some("/tmp/cache".to_string())
    );
    let mut registry: SifrGeneratedStdlibSifrX2ecsvX2eDialectRegistry = dialect_registry();
    (&mut registry).register(
        &"pipe".to_string(),
        &SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            "|".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            "\n".to_string(),
            SifrInt::from_i64(0),
        ),
    );
    let d: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect> = registry.get(&"pipe".to_string());
    assert!(d.is_some());
    if let Some(d) = d {
        let r: SifrGeneratedStdlibSifrX2ecsvX2ereader = SifrGeneratedStdlibSifrX2ecsvX2ereader::new(
            "a|b\n1|2".to_string(),
            Some(d),
            ",".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            SifrInt::from_i64(0),
        );
        assert_eq!(
            format!("{:?}", r.rows()),
            "[[\"a\", \"b\"], [\"1\", \"2\"]]"
        );
    }
    assert!((&mut registry).unregister(&"pipe".to_string()));
}

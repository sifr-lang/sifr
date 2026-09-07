// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, SifrGeneratedStdlibSifrX2ecsvX2eDialect,
        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError, ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn parse_flag(args: &[String], flag: &str) -> bool {
        for arg in args.iter().cloned() {
            if arg == *flag {
                return true;
            }
        }
        false
    }
    pub(crate) fn sifr_generated_split_inline_option(token: &str) -> (bool, String, String) {
        let sifr_generated_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
        let mut key: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_token.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_token.len());
                sifr_generated_chars_token
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if ch.is_some() && ch == Some("=".to_string()) {
                let mut value: String = String::new();
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &j < &SifrInt::from(sifr_generated_chars_token.len()) {
                    let part: Option<String> = {
                        let sifr_generated_string_index = j.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_token.len());
                        sifr_generated_chars_token
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(part) = part {
                        value.push_str(part.as_str());
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                return (true, key, value);
            }
            if let Some(ch) = ch {
                key.push_str(ch.as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        (
            false,
            {
                let mut sifr_generated_concat: String = String::with_capacity(token.len());
                sifr_generated_concat.push_str(token);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            },
            String::new(),
        )
    }
    pub(crate) fn parse_option(args: &[String], name: &str, default: &str) -> String {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(args.len()) {
            let arg: Option<String> = {
                let sifr_generated_checked_read_collection = &args;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(arg) = arg {
                if arg == *name && &(&i + &SifrInt::from_i64(1)) < &SifrInt::from(args.len()) {
                    let next_val: Option<String> = {
                        let sifr_generated_checked_read_collection = &args;
                        let sifr_generated_checked_read_index = &i + &SifrInt::from_i64(1);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(next_val) = next_val {
                        if next_val == "--" || next_val.starts_with("--") {
                        } else {
                            return {
                                let mut sifr_generated_concat: String =
                                    String::with_capacity(next_val.len());
                                sifr_generated_concat.push_str(next_val.as_str());
                                sifr_generated_concat.push_str("");
                                sifr_generated_concat
                            };
                        }
                    }
                }
                let (inline_has_value, inline_name, inline_value) =
                    sifr_generated_split_inline_option(&arg);
                let _chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
                let _chars_inline_value: Vec<char> = inline_value.chars().collect::<Vec<char>>();
                if inline_has_value && inline_name == *name {
                    return {
                        let mut sifr_generated_concat: String =
                            String::with_capacity(inline_value.len());
                        sifr_generated_concat.push_str(inline_value.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(default.len());
            sifr_generated_concat.push_str(default);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn bisect_left<T: Clone + 'static + PartialOrd>(
        a: &[T],
        x: &T,
        lo: SifrInt,
        hi: Option<SifrInt>,
    ) -> SifrInt {
        let mut left: SifrInt = lo.clone();
        if &left < &SifrInt::from_i64(0) {
            left = SifrInt::from_i64(0);
        }
        let mut right: SifrInt = SifrInt::from(a.len());
        if hi.is_none() {
            right = SifrInt::from(a.len());
        } else if let Some(hi) = hi.clone() {
            if &hi < &SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else if &hi > &SifrInt::from(a.len()) {
                right = SifrInt::from(a.len());
            } else {
                right = hi;
            }
        }
        while &left < &right {
            let mid: SifrInt = (&left + &right).floor_div_known_nonzero(&SifrInt::from_i64(2));
            let val: Option<T> = {
                let sifr_generated_checked_read_collection = &a;
                let sifr_generated_checked_read_index = mid.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(val) = val {
                if val < *x {
                    left = &mid + &SifrInt::from_i64(1);
                } else {
                    right = mid;
                }
            } else {
                left = &mid + &SifrInt::from_i64(1);
            }
        }
        left.clone()
    }
    pub(crate) const fn sifr_generated_const_51554f54455f414c4c() -> SifrInt {
        SifrInt::from_i64(1)
    }
    pub(crate) const fn sifr_generated_const_51554f54455f4e4f4e4e554d45524943() -> SifrInt {
        SifrInt::from_i64(2)
    }
    pub(crate) const fn sifr_generated_const_51554f54455f4e4f4e45() -> SifrInt {
        SifrInt::from_i64(3)
    }
    pub(crate) const fn sifr_generated_const_51554f54455f535452494e4753() -> SifrInt {
        SifrInt::from_i64(4)
    }
    pub(crate) const fn sifr_generated_const_51554f54455f4e4f544e554c4c() -> SifrInt {
        SifrInt::from_i64(5)
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
    pub(crate) fn sifr_generated_first_char(text: &str) -> String {
        sifr_generated_char_at(text, SifrInt::from_i64(0))
    }
    pub(crate) fn sifr_generated_last_char(text: &str) -> String {
        let _chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        sifr_generated_char_at(
            text,
            &SifrInt::from(text.chars().count()) - &SifrInt::from_i64(1),
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn parse_row(
        line: &str,
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> Vec<String> {
        let rows: Vec<Vec<String>> = parse_csv(
            line,
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            quoting.clone(),
        );
        if &SifrInt::from(rows.len()) == &SifrInt::from_i64(0) {
            return Vec::new();
        }
        for (index, row) in Box::new(rows.iter().cloned().enumerate().map(|sifr_generated_pair| {
            (
                SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(0),
                sifr_generated_pair.1,
            )
        })) {
            if &index == &SifrInt::from_i64(0) {
                let mut copied: Vec<String> = Vec::new();
                for field in row.iter().cloned() {
                    copied.push(field.to_string());
                }
                return copied;
            }
        }
        Vec::new()
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
    pub(crate) fn sifr_generated_needs_quote(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> bool {
        let sifr_generated_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
        if &dialect.quoting.clone() == &sifr_generated_const_51554f54455f414c4c() {
            return true;
        }
        if &dialect.quoting.clone() == &sifr_generated_const_51554f54455f4e4f4e4e554d45524943() {
            return true;
        }
        if &dialect.quoting.clone() == &sifr_generated_const_51554f54455f535452494e4753() {
            return true;
        }
        if &dialect.quoting.clone() == &sifr_generated_const_51554f54455f4e4f544e554c4c() {
            return true;
        }
        if &dialect.quoting.clone() == &sifr_generated_const_51554f54455f4e4f4e45() {
            return false;
        }
        if field.contains(dialect.delimiter.clone().as_str()) {
            return true;
        }
        if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
            return true;
        }
        if !dialect.quotechar.clone().is_empty() {
            let quotechar: String = sifr_generated_quotechar_value(dialect);
            if field.contains(&quotechar) {
                return true;
            }
        }
        if &SifrInt::from(sifr_generated_chars_field.len()) > &SifrInt::from_i64(0) {
            let first: String = sifr_generated_first_char(field);
            let last: String = sifr_generated_last_char(field);
            if first == " " {
                return true;
            }
            if last == " " {
                return true;
            }
        }
        false
    }
    pub(crate) fn sifr_generated_quote_field(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let quotechar: String = sifr_generated_quotechar_value(dialect);
        let mut escaped: String = {
            let mut sifr_generated_concat: String = String::with_capacity(field.len());
            sifr_generated_concat.push_str(field);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if escaped.contains(&quotechar) {
            if dialect.doublequote || dialect.escapechar.clone().is_empty() {
                escaped = escaped.replace(&quotechar, &format!("{quotechar}{quotechar}"));
            } else {
                let escapechar_value: String = {
                    let mut sifr_generated_concat: String = String::new();
                    sifr_generated_concat.push_str(dialect.escapechar.clone().as_str());
                    sifr_generated_concat.push_str("");
                    sifr_generated_concat
                };
                escaped = escaped.replace(&quotechar, &format!("{escapechar_value}{quotechar}"));
            }
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(quotechar.len() + escaped.len() + quotechar.len());
            sifr_generated_concat.push_str(quotechar.as_str());
            sifr_generated_concat.push_str(escaped.as_str());
            sifr_generated_concat.push_str(quotechar.as_str());
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_escape_unquoted_field(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let mut result: String = {
            let mut sifr_generated_concat: String = String::with_capacity(field.len());
            sifr_generated_concat.push_str(field);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if result.contains(dialect.delimiter.clone().as_str())
            && !dialect.escapechar.clone().is_empty()
        {
            result = result.replace(
                &dialect.delimiter.clone(),
                &format!(
                    "{}{}",
                    dialect.escapechar.clone(),
                    dialect.delimiter.clone()
                ),
            );
        }
        if result.contains(&"\n".to_string()) && !dialect.escapechar.clone().is_empty() {
            result = result.replace('\n', &format!("{}\n", dialect.escapechar.clone()));
        }
        if result.contains(&"\r".to_string()) && !dialect.escapechar.clone().is_empty() {
            result = result.replace('\r', &format!("{}\r", dialect.escapechar.clone()));
        }
        if !dialect.quotechar.clone().is_empty() {
            let quotechar2: String = sifr_generated_quotechar_value(dialect);
            if result.contains(&quotechar2) {
                if dialect.escapechar.clone().is_empty() {
                    result = result.replace(&quotechar2, &format!("{quotechar2}{quotechar2}"));
                } else {
                    result = result.replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar.clone(), quotechar2),
                    );
                }
            }
        }
        result
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn format_row(
        fields: &[String],
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> String {
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting.clone(),
        );
        let mut parts: Vec<String> = Vec::new();
        for field in fields.iter().cloned() {
            if sifr_generated_needs_quote(&field, &resolved) {
                parts.push(sifr_generated_quote_field(&field, &resolved));
            } else {
                parts.push(sifr_generated_escape_unquoted_field(&field, &resolved));
            }
        }
        parts.join(&resolved.delimiter)
    }
    pub(crate) fn fnmatch(name: &str, pattern: &str) -> bool {
        sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
    }
    pub(crate) fn sifr_generated_match(
        name: &str,
        mut ni: SifrInt,
        pattern: &str,
        mut pi: SifrInt,
    ) -> bool {
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
    pub(crate) fn reduce<T: Clone + 'static, U: Clone + 'static>(
        func: impl Fn(&U, &T) -> U,
        data: &[T],
        initial: &U,
    ) -> U {
        let mut result: U = initial.clone();
        for val in data.iter().cloned() {
            result = func(&result, &val);
        }
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_sift_down<T: Clone + 'static + PartialOrd>(
        data: &mut Vec<T>,
        mut pos: SifrInt,
        n: SifrInt,
    ) {
        let mut done: bool = false;
        while !done {
            let mut smallest: SifrInt = pos.clone();
            let left: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(1);
            let right: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(2);
            if &left < &n {
                let s_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let l_val_value_c583c4339eb822b3: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = left.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(s_val) = s_val
                    && let Some(l_val) = l_val_value_c583c4339eb822b3
                    && l_val < s_val
                {
                    smallest = left;
                }
            }
            if &right < &n {
                let s_val2_value_8b32ab056d206424: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let r_val_value_839f97b21b19be35: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = right.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(s_val2) = s_val2_value_8b32ab056d206424
                    && let Some(r_val) = r_val_value_839f97b21b19be35
                    && r_val < s_val2
                {
                    smallest = right;
                }
            }
            if &smallest == &pos {
                done = true;
            } else {
                let tmp_pos: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = pos.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let tmp_sm_value_cf4d6d82a6cdd887: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(tmp_pos) = tmp_pos
                    && let Some(tmp_sm) = tmp_sm_value_cf4d6d82a6cdd887
                {
                    if &SifrInt::from_i64(0) <= &pos && &pos < &SifrInt::from(data.len()) {
                        {
                            let sifr_generated_assign_value = tmp_sm.clone();
                            {
                                let sifr_generated_index_raw = pos.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(data.len());
                                if let Some(sifr_generated_elem) =
                                    data.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    if &SifrInt::from_i64(0) <= &smallest && &smallest < &SifrInt::from(data.len())
                    {
                        {
                            let sifr_generated_assign_value = tmp_pos.clone();
                            {
                                let sifr_generated_index_raw = smallest.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(data.len());
                                if let Some(sifr_generated_elem) =
                                    data.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                }
                pos = smallest;
            }
        }
    }
    pub(crate) fn sifr_generated_sift_up<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
        mut pos: SifrInt,
    ) {
        let mut done: bool = false;
        while !done {
            if &pos <= &SifrInt::from_i64(0) {
                done = true;
            } else {
                let parent: SifrInt =
                    (&pos - &SifrInt::from_i64(1)).floor_div_known_nonzero(&SifrInt::from_i64(2));
                let p_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &heap;
                    let sifr_generated_checked_read_index = parent.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let c_val_value_6b01c611cd56bc8e: Option<T> = {
                    let sifr_generated_checked_read_collection = &heap;
                    let sifr_generated_checked_read_index = pos.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(p_val) = p_val {
                    if let Some(c_val) = c_val_value_6b01c611cd56bc8e {
                        if c_val < p_val {
                            if &SifrInt::from_i64(0) <= &parent
                                && &parent < &SifrInt::from(heap.len())
                            {
                                {
                                    let sifr_generated_assign_value = c_val.clone();
                                    {
                                        let sifr_generated_index_raw = parent.clone();
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(heap.len());
                                        if let Some(sifr_generated_elem) =
                                            heap.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        }
                                    }
                                }
                            }
                            if &SifrInt::from_i64(0) <= &pos && &pos < &SifrInt::from(heap.len()) {
                                {
                                    let sifr_generated_assign_value = p_val.clone();
                                    {
                                        let sifr_generated_index_raw = pos.clone();
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(heap.len());
                                        if let Some(sifr_generated_elem) =
                                            heap.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        }
                                    }
                                }
                            }
                            pos = parent;
                        } else {
                            done = true;
                        }
                    } else {
                        done = true;
                    }
                } else {
                    done = true;
                }
            }
        }
    }
    pub(crate) fn heapify<T: Clone + 'static + PartialOrd>(data: &mut Vec<T>) {
        "Convert list to a min-heap in-place. O(n) time.".to_string();
        let n: SifrInt = SifrInt::from(data.len());
        let mut i: SifrInt =
            &n.floor_div_known_nonzero(&SifrInt::from_i64(2)) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
            sifr_generated_sift_down(data, i.clone(), n.clone());
            i = &i - &SifrInt::from_i64(1);
        }
    }
    pub(crate) fn heappush<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>, item: &T) {
        "Push item onto the heap in-place. O(log n) time.".to_string();
        heap.push(item.clone());
        let pos: SifrInt = &SifrInt::from(heap.len()) - &SifrInt::from_i64(1);
        sifr_generated_sift_up(heap, pos.clone());
    }
    pub(crate) fn heappop<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>) -> Option<T> {
        "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
            .to_string();
        let n: SifrInt = SifrInt::from(heap.len());
        if &n == &SifrInt::from_i64(0) {
            return None;
        }
        let top: Option<T> = {
            let sifr_generated_checked_read_collection = &heap;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let last: Option<T> = {
            let sifr_generated_checked_read_collection = &heap;
            let sifr_generated_checked_read_index = &n - &SifrInt::from_i64(1);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        heap.remove(heap.len() - 1_usize);
        let n2: SifrInt = SifrInt::from(heap.len());
        if &n2 > &SifrInt::from_i64(0) {
            if let Some(last) = last {
                {
                    let sifr_generated_assign_value = last.clone();
                    {
                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(heap.len());
                        if let Some(sifr_generated_elem) =
                            heap.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            }
            sifr_generated_sift_down(heap, SifrInt::from_i64(0), n2.clone());
        }
        top
    }
    pub(crate) fn nsmallest<T: Clone + 'static + PartialOrd>(n: SifrInt, data: &[T]) -> Vec<T> {
        let mut heap: Vec<T> = data.to_vec();
        heapify(&mut heap);
        let mut result: Vec<T> = Vec::new();
        let mut count: SifrInt = SifrInt::from_i64(0);
        while &count < &n {
            if &SifrInt::from(heap.len()) == &SifrInt::from_i64(0) {
                return result;
            }
            let val: Option<T> = heappop(&mut heap);
            if let Some(val) = val {
                result.push(val);
            }
            count = &count + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) struct SifrGeneratedYielder<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub(crate) struct SifrGeneratedYieldFuture<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) value: Option<T>,
    }
    impl<T> Unpin for SifrGeneratedYieldFuture<T> {}
    impl<T> ::std::future::Future for SifrGeneratedYieldFuture<T> {
        type Output = ();
        fn poll(
            self: ::std::pin::Pin<&mut Self>,
            _: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<()> {
            let state = self.get_mut();
            let Some(value) = state.value.take() else {
                return ::std::task::Poll::Ready(());
            };
            sifr_generated_store_suspended(&state.slot, value);
            ::std::task::Poll::Pending
        }
    }
    impl<T> SifrGeneratedYielder<T> {
        pub(crate) fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
            SifrGeneratedYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub(crate) fn sifr_generated_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub(crate) fn sifr_generated_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub(crate) struct SifrGeneratedGenerator<T> {
        pub(crate) producer:
            Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
        pub(crate) yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) complete: bool,
    }
    impl<T> SifrGeneratedGenerator<T> {
        pub(crate) fn new<
            F: FnOnce(SifrGeneratedYielder<T>) -> Fut + 'static,
            Fut: ::std::future::Future<Output = ()> + 'static,
        >(
            factory: F,
        ) -> Self {
            let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
            let producer = factory(SifrGeneratedYielder {
                slot: ::std::sync::Arc::clone(&yielded),
            });
            Self {
                producer: Some(Box::pin(producer)),
                yielded,
                complete: false,
            }
        }
    }
    impl<T> Iterator for SifrGeneratedGenerator<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.complete {
                return None;
            }
            let completed = {
                let Some(producer) = self.producer.as_mut() else {
                    self.complete = true;
                    return None;
                };
                let mut context = ::std::task::Context::from_waker(::std::task::Waker::noop());
                ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
            };
            let yielded = sifr_generated_take_suspended(&self.yielded);
            if completed {
                self.complete = true;
                self.producer = None;
            }
            yielded
        }
    }
    pub(crate) trait SifrGeneratedAdd: Sized {}
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
    impl SifrGeneratedAdd for f64 {}
    impl SifrGeneratedAdd for String {}
    pub(crate) fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
        let iterables = iterables.to_vec();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for iterable in iterables.iter().cloned() {
                    for item in iterable.iter().cloned() {
                        sifr_generated_yielder.suspend(item.clone()).await;
                    }
                }
            },
        ))
    }
    pub(crate) fn take<T: Clone + 'static>(n: SifrInt, data: &[T]) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        let mut count: SifrInt = SifrInt::from_i64(0);
        for item in data.iter().cloned() {
            if &count >= &n {
                return result;
            }
            result.push(item);
            count = &count + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
    }
    pub(crate) fn token_hex(nbytes: SifrInt) -> String {
        let hex_chars: String = "0123456789abcdef".to_string();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &(&nbytes * &SifrInt::from_i64(2)) {
            let idx: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(15));
            let ch: Option<String> = {
                let sifr_generated_string_chars = hex_chars.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = idx.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                result.push_str(ch.as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
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
    pub(crate) fn sifr_generated_sum(data: &[f64]) -> f64 {
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            total += val;
        }
        total
    }
    pub(crate) fn sifr_generated_float_int(
        value: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let converted: f64 = value
                .clone()
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
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
            Ok(Ok(converted))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_divide_by_int(
        numerator: f64,
        denominator: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let divisor: f64 = sifr_generated_float_int(denominator.clone())?;
            Ok(Ok(numerator / divisor))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let count: SifrInt = SifrInt::from(data.len());
        if &count == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ));
        }
        let total: f64 = sifr_generated_sum(data);
        sifr_generated_divide_by_int(total, count.clone())
    }
    pub(crate) fn sifr_generated_const_61736369695f6c6f77657263617365() -> String {
        "abcdefghijklmnopqrstuvwxyz".to_string()
    }
    pub(crate) fn sifr_generated_const_646967697473() -> String {
        "0123456789".to_string()
    }
    pub(crate) fn sifr_generated_replace_whitespace_chars(
        text: &str,
        replace_tabs: bool,
    ) -> String {
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
    pub(crate) fn sifr_generated_expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
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
    pub(crate) fn sifr_generated_prepare_text(
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
    pub(crate) fn sifr_generated_normalize_whitespace(text: &str) -> String {
        sifr_generated_prepare_text(text, true, SifrInt::from_i64(8), true)
    }
    pub(crate) fn sifr_generated_split_word_units(
        word: &str,
        break_on_hyphens: bool,
    ) -> Vec<String> {
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
    pub(crate) fn sifr_generated_trim_line(line: &str) -> String {
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
    pub(crate) fn sifr_generated_finalize_line(line: &str, drop_whitespace: bool) -> String {
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
    pub(crate) fn sifr_generated_wrap_impl(text: &str, width: SifrInt) -> Vec<String> {
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
    pub(crate) fn sifr_generated_effective_content_width(
        total_width: SifrInt,
        indent: &str,
    ) -> SifrInt {
        let sifr_generated_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
        let available: SifrInt = &total_width - &SifrInt::from(sifr_generated_chars_indent.len());
        if &available <= &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        available.clone()
    }
    pub(crate) fn sifr_generated_push_current_line(
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
    pub(crate) fn sifr_generated_wrap_with_indents(
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
    pub(crate) fn fill(text: &str, width: SifrInt) -> Result<String, ValueError> {
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
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
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
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("StatisticsError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {}
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialect;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError;
pub use sifr_generated_project_nominals::ValueError;
fn add_ints(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    assert_eq!(
        &SifrInt::from(
            sifr_generated_const_61736369695f6c6f77657263617365()
                .chars()
                .count()
        ),
        &SifrInt::from_i64(26)
    );
    assert_eq!(
        &SifrInt::from(sifr_generated_const_646967697473().chars().count()),
        &SifrInt::from_i64(10)
    );
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
        SifrInt::from_i64(40),
        SifrInt::from_i64(50),
    ];
    let pos: SifrInt = bisect_left(&data, &SifrInt::from_i64(30), SifrInt::from_i64(0), None);
    assert_eq!(pos, SifrInt::from_i64(2));
    let nums_r: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let total: SifrInt = reduce(
        |sifr_generated_arg0, sifr_generated_arg1| {
            add_ints(sifr_generated_arg0.clone(), sifr_generated_arg1.clone())
        },
        &nums_r,
        &SifrInt::from_i64(0),
    );
    assert_eq!(total, SifrInt::from_i64(15));
    let token: String = token_hex(SifrInt::from_i64(8));
    let _chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    assert_eq!(
        &SifrInt::from(token.chars().count()),
        &SifrInt::from_i64(16)
    );
    let data2_value_3d7c5557a5771b75: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64];
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let avg: f64 = mean(&data2_value_3d7c5557a5771b75)?;
            assert_eq!(avg, 4.0_f64);
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let se = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize);
            sifr_generated_concat.push_str("statistics error: ");
            sifr_generated_concat.push_str(se.message.clone().as_str());
            sifr_generated_concat
        });
        assert_eq!(
            format!("statistics error: {}", se.message.clone()),
            "stdlib_expansion demo: all checks passed!"
        );
    }
    let mut heap: Vec<SifrInt> = Vec::new();
    heappush(&mut heap, &SifrInt::from_i64(5));
    heappush(&mut heap, &SifrInt::from_i64(1));
    heappush(&mut heap, &SifrInt::from_i64(3));
    let top: Option<SifrInt> = heappop(&mut heap);
    if let Some(top) = top.clone() {
        assert_eq!(top, SifrInt::from_i64(1));
    }
    let small: Vec<SifrInt> = nsmallest(
        SifrInt::from_i64(2),
        &vec![
            SifrInt::from_i64(9),
            SifrInt::from_i64(3),
            SifrInt::from_i64(7),
            SifrInt::from_i64(1),
        ],
    );
    assert_eq!(SifrInt::from(small.len()), SifrInt::from_i64(2));
    let merged: Vec<SifrInt> = chain(&vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        vec![SifrInt::from_i64(3), SifrInt::from_i64(4)],
    ])
    .collect::<Vec<_>>();
    assert_eq!(SifrInt::from(merged.len()), SifrInt::from_i64(4));
    let first3: Vec<SifrInt> = take(
        SifrInt::from_i64(3),
        &vec![
            SifrInt::from_i64(10),
            SifrInt::from_i64(20),
            SifrInt::from_i64(30),
            SifrInt::from_i64(40),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
    );
    assert_eq!(SifrInt::from(first3.len()), SifrInt::from_i64(3));
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let filled: String = fill(&"hello world foo bar".to_string(), SifrInt::from_i64(12))?;
        let _chars_filled: Vec<char> = filled.chars().collect::<Vec<char>>();
        assert!(SifrInt::from(filled.chars().count()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let row: Vec<String> = parse_row(
        &"a,b,c".to_string(),
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &String::new(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    assert_eq!(SifrInt::from(row.len()), SifrInt::from_i64(3));
    let line: String = format_row(
        &vec!["x".to_string(), "y".to_string()],
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &String::new(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    assert_eq!(line, "x,y");
    let args: Vec<String> = vec!["--output".to_string(), "file.txt".to_string()];
    let has_output: bool = parse_flag(&args, &"--output".to_string());
    assert!(has_output);
    let val: String = parse_option(&args, &"--output".to_string(), &"default".to_string());
    assert_eq!(val, "file.txt");
    assert!(fnmatch(&"test.py".to_string(), &"*.py".to_string()));
    assert!(!fnmatch(&"test.py".to_string(), &"*.txt".to_string()));
    println!("stdlib_expansion demo: all checks passed!");
}

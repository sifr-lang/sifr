// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        IOError, SifrGeneratedStdlibSifrX2ecsvX2eDialect, SifrGeneratedStdlibSifrX2ecsvX2ereader,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
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
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn format_csv(
        rows: &[Vec<String>],
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &str,
        quoting: SifrInt,
    ) -> String {
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting.clone(),
        );
        let mut rendered: Vec<String> = Vec::new();
        let resolved_delimiter: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(resolved.delimiter.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let resolved_quotechar: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(resolved.quotechar.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let resolved_escapechar: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(resolved.escapechar.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let resolved_lineterminator: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(resolved.lineterminator.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        for row in rows.iter().cloned() {
            rendered.push(format_row(
                &row,
                &None,
                &resolved_delimiter,
                &resolved_quotechar,
                &resolved_escapechar,
                resolved.doublequote,
                resolved.skipinitialspace,
                resolved.quoting.clone(),
            ));
        }
        rendered.join(&resolved_lineterminator)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn reader_from_path(
        path: &str,
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> Result<SifrGeneratedStdlibSifrX2ecsvX2ereader, IOError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ecsvX2ereader, IOError>,
            IOError,
        > = (|| {
            let text: String = read_text(path)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2ecsvX2ereader::new(
                text,
                dialect.clone(),
                delimiter.to_owned(),
                quotechar.to_owned(),
                escapechar.to_owned(),
                doublequote,
                skipinitialspace,
                quoting.clone(),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn writer_to_path(
        path: &str,
        rows: &[Vec<String>],
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &str,
        quoting: SifrInt,
    ) -> Result<(), IOError> {
        let payload: String = format_csv(
            rows,
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting.clone(),
        );
        write_text(path, &payload)
    }
    pub(crate) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
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
    pub(crate) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match sifr_generated_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
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
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        pub rows: Vec<Vec<String>>,
        pub dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            dialect: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
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
                    &lineterminator,
                    quoting.clone(),
                );
            let sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374: SifrGeneratedStdlibSifrX2ecsvX2eDialect = resolved_dialect;
            let sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773: Vec<Vec<String>> =
                Vec::new();
            Self {
                dialect: sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374,
                rows: sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        pub fn writerow(&mut self, row: &[String]) {
            let mut copied: Vec<String> = Vec::new();
            for value in row.iter().cloned() {
                copied.push(value.to_owned());
            }
            self.rows.push(copied.to_vec());
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        #[must_use]
        pub fn getvalue(&self) -> String {
            format_csv(
                &self.rows,
                &Some(self.dialect.clone()),
                &",".to_string(),
                &"\"".to_string(),
                &String::new(),
                true,
                false,
                &"\n".to_string(),
                SifrInt::from_i64(0),
            )
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialect;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ereader;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ewriter;
fn collect_parse_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let parsed: Vec<String> = parse_row(
        &"a,b,c".to_string(),
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &String::new(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    actual.push(format!("{parsed:?}").as_str() == "[\"a\", \"b\", \"c\"]".to_string().as_str());
    actual.push(
        format_csv(
            &vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ],
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &String::new(),
            true,
            false,
            &"\n".to_string(),
            SifrInt::from_i64(0),
        )
        .as_str()
            == "1,2\n3,4".to_string().as_str(),
    );
    actual
}
fn collect_object_and_file_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let r: SifrGeneratedStdlibSifrX2ecsvX2ereader = SifrGeneratedStdlibSifrX2ecsvX2ereader::new(
        "name,age\nalice,30".to_string(),
        None,
        ",".to_string(),
        "\"".to_string(),
        String::new(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    actual.push(
        format!("{:?}", r.rows()).as_str()
            == "[[\"name\", \"age\"], [\"alice\", \"30\"]]"
                .to_string()
                .as_str(),
    );
    let mut w: SifrGeneratedStdlibSifrX2ecsvX2ewriter = SifrGeneratedStdlibSifrX2ecsvX2ewriter::new(
        None,
        ",".to_string(),
        "\"".to_string(),
        String::new(),
        true,
        false,
        "\n".to_string(),
        SifrInt::from_i64(0),
    );
    (&mut w).writerow(&vec!["alice".to_string(), "30".to_string()]);
    actual.push(w.getvalue().as_str() == "alice,30".to_string().as_str());
    let path: String = "/tmp/sifr_csv_csv_demo.csv".to_string();
    let mut csv_file_ok: bool = false;
    let mut missing_rejected: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        writer_to_path(
            &path,
            &vec![
                vec!["h1".to_string(), "h2".to_string()],
                vec!["v1".to_string(), "v2".to_string()],
            ],
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &String::new(),
            true,
            false,
            &"\n".to_string(),
            SifrInt::from_i64(0),
        )?;
        let rf: SifrGeneratedStdlibSifrX2ecsvX2ereader = reader_from_path(
            &path,
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &String::new(),
            true,
            false,
            SifrInt::from_i64(0),
        )?;
        csv_file_ok = format!("{:?}", rf.rows()) == "[[\"h1\", \"h2\"], [\"v1\", \"v2\"]]";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    actual.push(csv_file_ok);
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _missing: SifrGeneratedStdlibSifrX2ecsvX2ereader = reader_from_path(
            &"/tmp/sifr_csv_csv_demo_missing.csv".to_string(),
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &String::new(),
            true,
            false,
            SifrInt::from_i64(0),
        )?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        missing_rejected = true;
    }
    actual.push(missing_rejected);
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
    append_all(&mut actual, &collect_parse_actual());
    append_all(&mut actual, &collect_object_and_file_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("csv csv parity demo: pass");
}

// src/main.rs
mod sifr_generated_generated_support {
    use crate::{DivisionError, IOError, SifrGeneratedStdlibSifrX2econfigparserX2eParsingError};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn calendar_isleap(year: SifrInt) -> bool {
        ::sifr_stdlib::calendar::calendar_isleap(::sifr_runtime::interop::SifrIntBridge::from(year))
    }
    pub(crate) fn calendar_weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
        ::sifr_stdlib::calendar::calendar_weekday(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
            ::sifr_runtime::interop::SifrIntBridge::from(day),
        )
        .into_sifr_int()
    }
    pub(crate) fn calendar_monthrange(year: SifrInt, month: SifrInt) -> Vec<SifrInt> {
        ::sifr_stdlib::calendar::calendar_monthrange(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
        )
        .into_iter()
        .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
        .collect()
    }
    pub(crate) fn isleap(year: SifrInt) -> bool {
        calendar_isleap(year.clone())
    }
    pub(crate) fn weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
        calendar_weekday(year.clone(), month.clone(), day.clone())
    }
    pub(crate) fn monthrange(year: SifrInt, month: SifrInt) -> Vec<SifrInt> {
        calendar_monthrange(year.clone(), month.clone())
    }
    pub(crate) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
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
    pub(crate) fn sifr_generated_gzip_compress_bytes_impl(data: &str) -> Vec<u8> {
        ::sifr_stdlib::gzip::gzip_compress_bytes(data)
    }
    pub(crate) fn sifr_generated_gzip_decompress_bytes_impl(
        data: &[u8],
    ) -> Result<String, IOError> {
        ::sifr_stdlib::gzip::gzip_decompress_bytes(data).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_create(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_create(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_add_file(zip_path: &str, name: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_read_file(zip_path: &str, name: &str) -> Result<String, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file(zip_path, name).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_namelist(zip_path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::zipfile::zip_namelist(zip_path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn compress(data: &str) -> Vec<u8> {
        sifr_generated_gzip_compress_bytes_impl(data)
    }
    pub(crate) fn decompress(data: &[u8]) -> Result<String, IOError> {
        sifr_generated_gzip_decompress_bytes_impl(data)
    }
    pub(crate) fn html_escape(s: &str) -> String {
        ::sifr_stdlib::html::html_escape(s)
    }
    pub(crate) fn html_unescape(s: &str) -> String {
        ::sifr_stdlib::html::html_unescape(s)
    }
    pub(crate) fn escape(s: &str, quote: bool) -> String {
        let escaped: String = html_escape(s);
        if quote {
            return escaped;
        }
        escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
    }
    pub(crate) fn unescape(s: &str) -> String {
        html_unescape(s)
    }
    pub(crate) fn add(a: SifrInt, b: SifrInt) -> SifrInt {
        &a + &b
    }
    pub(crate) fn sub(a: SifrInt, b: SifrInt) -> SifrInt {
        &a - &b
    }
    pub(crate) fn mul(a: SifrInt, b: SifrInt) -> SifrInt {
        &a * &b
    }
    pub(crate) fn floordiv(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
        {
            let sifr_generated_floor_left: SifrInt = a.clone();
            let sifr_generated_floor_right: SifrInt = b.clone();
            sifr_generated_floor_left
                .checked_floor_div(&sifr_generated_floor_right)
                .ok_or_else(|| DivisionError::new("division by zero".to_string()))
        }
    }
    pub(crate) fn mod_val(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
        {
            let sifr_generated_floor_left: SifrInt = a.clone();
            let sifr_generated_floor_right: SifrInt = b.clone();
            sifr_generated_floor_left
                .checked_floor_mod(&sifr_generated_floor_right)
                .ok_or_else(|| DivisionError::new("division by zero".to_string()))
        }
    }
    pub(crate) fn neg(a: SifrInt) -> SifrInt {
        -&a
    }
    pub(crate) fn lt(a: SifrInt, b: SifrInt) -> bool {
        &a < &b
    }
    pub(crate) fn eq(a: SifrInt, b: SifrInt) -> bool {
        &a == &b
    }
    pub(crate) fn getitem<T: Clone + 'static>(items: &[T], index: SifrInt) -> Option<T> {
        {
            let sifr_generated_checked_read_collection = &items;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }
    }
    pub(crate) fn itemgetter<T: Clone + 'static>(items: &[T], index: SifrInt) -> Option<T> {
        getitem(items, index.clone())
    }
    pub(crate) fn sys_version() -> String {
        ::sifr_stdlib::sys::sys_version()
    }
    pub(crate) fn sys_maxsize() -> SifrInt {
        ::sifr_stdlib::sys::sys_maxsize().into_sifr_int()
    }
    pub(crate) fn version() -> String {
        sys_version()
    }
    pub(crate) fn maxsize() -> SifrInt {
        sys_maxsize()
    }
    pub(crate) fn sifr_generated_zip_read_only_error() -> String {
        "zipfile operation requires write or append mode".to_string()
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
        pub fn has_option(&self, section: &str, option: &str) -> bool {
            let normalized: String = sifr_generated_normalize_option(option);
            let default_section: String = sifr_generated_default_section();
            if *section == default_section {
                return self.defaults.contains_key(&normalized);
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
                if sifr_generated_has_option_key(&section_values, &normalized) {
                    return true;
                }
                return self.defaults.contains_key(&normalized);
            }
            false
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
    pub struct SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        pub path: String,
        pub mode: String,
        pub compression: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        #[must_use]
        pub fn new(path: String, mode: String, compression: SifrInt) -> Self {
            let sifr_generated_field_value_03c52d0debd70676_70617468: String = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_0d3deba2c41dadb2_6d6f6465: String = {
                let mut sifr_generated_concat: String = String::with_capacity(mode.len());
                sifr_generated_concat.push_str(mode.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_fb545b3ab0be00f5_636f6d7072657373696f6e: SifrInt =
                compression.clone();
            Self {
                path: sifr_generated_field_value_03c52d0debd70676_70617468,
                mode: sifr_generated_field_value_0d3deba2c41dadb2_6d6f6465,
                compression: sifr_generated_field_value_fb545b3ab0be00f5_636f6d7072657373696f6e,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        #[must_use]
        pub fn sifr_generated_writable_mode(&self) -> bool {
            self.mode.clone() == "w"
                || self.mode.clone() == "a"
                || self.mode.clone() == "wb"
                || self.mode.clone() == "ab"
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn create(&self) -> Result<(), IOError> {
            zip_create(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write(&self, name: &str, content: &str) -> Result<(), IOError> {
            if !self.sifr_generated_writable_mode() {
                return Err(IOError::new(sifr_generated_zip_read_only_error()));
            }
            zip_add_file(&self.path, name, content)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read(&self, name: &str) -> Result<String, IOError> {
            zip_read_file(&self.path, name)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn namelist(&self) -> Result<Vec<String>, IOError> {
            zip_namelist(&self.path)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "ZipFile(path={}, mode={}, compression={})",
                self.path, self.mode, self.compression
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        #[must_use]
        pub fn new(message: String) -> Self {
            Self {
                message,
                kind: "Other".to_string(),
            }
        }
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DivisionError {
        pub message: String,
    }
    impl DivisionError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DivisionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DivisionError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::DivisionError;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2econfigparserX2eParsingError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ezipfileX2eZipFile;
fn demo_operator() {
    println!("=== operator ===");
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("add(10, 5) = ");
        sifr_generated_concat.push_str(
            add(SifrInt::from_i64(10), SifrInt::from_i64(5))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("sub(10, 5) = ");
        sifr_generated_concat.push_str(
            sub(SifrInt::from_i64(10), SifrInt::from_i64(5))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(12usize);
        sifr_generated_concat.push_str("mul(3, 4) = ");
        sifr_generated_concat.push_str(
            mul(SifrInt::from_i64(3), SifrInt::from_i64(4))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(17usize);
        sifr_generated_concat.push_str("floordiv(7, 2) = ");
        sifr_generated_concat.push_str(
            format!("{:?}", floordiv(SifrInt::from_i64(7), SifrInt::from_i64(2))).as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("mod_val(7, 2) = ");
        sifr_generated_concat.push_str(
            format!("{:?}", mod_val(SifrInt::from_i64(7), SifrInt::from_i64(2))).as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(10usize);
        sifr_generated_concat.push_str("neg(42) = ");
        sifr_generated_concat.push_str(neg(SifrInt::from_i64(42)).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(11usize);
        sifr_generated_concat.push_str("lt(3, 5) = ");
        sifr_generated_concat.push_str(
            lt(SifrInt::from_i64(3), SifrInt::from_i64(5))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(11usize);
        sifr_generated_concat.push_str("eq(5, 5) = ");
        sifr_generated_concat.push_str(
            eq(SifrInt::from_i64(5), SifrInt::from_i64(5))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(25usize);
        sifr_generated_concat.push_str("itemgetter([1,2,3], 1) = ");
        sifr_generated_concat.push_str(
            itemgetter(&items, SifrInt::from_i64(1))
                .map_or_else(
                    || "None".to_string(),
                    |sifr_generated_v| sifr_generated_v.to_string(),
                )
                .as_str(),
        );
        sifr_generated_concat
    });
}
fn demo_calendar() {
    println!("=== calendar ===");
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("isleap(2000) = ");
        sifr_generated_concat.push_str(isleap(SifrInt::from_i64(2000)).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("isleap(1900) = ");
        sifr_generated_concat.push_str(isleap(SifrInt::from_i64(1900)).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("isleap(2024) = ");
        sifr_generated_concat.push_str(isleap(SifrInt::from_i64(2024)).to_string().as_str());
        sifr_generated_concat
    });
    let wd: SifrInt = weekday(
        SifrInt::from_i64(2024),
        SifrInt::from_i64(1),
        SifrInt::from_i64(1),
    );
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(20usize);
        sifr_generated_concat.push_str("weekday(2024,1,1) = ");
        sifr_generated_concat.push_str(wd.to_string().as_str());
        sifr_generated_concat
    });
    let mr: Vec<SifrInt> = monthrange(SifrInt::from_i64(2024), SifrInt::from_i64(2));
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(24usize);
        sifr_generated_concat.push_str("monthrange(2024,2)[1] = ");
        sifr_generated_concat.push_str(
            {
                let sifr_generated_index_list = &mr;
                let sifr_generated_index_i = SifrInt::from_i64(1);
                let sifr_generated_index_norm =
                    sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
                sifr_generated_index_list
                    .get(sifr_generated_index_norm)
                    .cloned()
            }
            .map_or_else(
                || "None".to_string(),
                |sifr_generated_v| sifr_generated_v.to_string(),
            )
            .as_str(),
        );
        sifr_generated_concat
    });
}
fn demo_html() {
    println!("=== html ===");
    let s: String = "<b>Hi & Bye</b>".to_string();
    let esc: String = escape(&s, true);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(26usize + esc.len());
        sifr_generated_concat.push_str("escape(<b>Hi & Bye</b>) = ");
        sifr_generated_concat.push_str(esc.as_str());
        sifr_generated_concat
    });
    let unesc: String = unescape(&esc);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(44usize + unesc.len());
        sifr_generated_concat.push_str("unescape(&lt;b&gt;Hi &amp; Bye&lt;/b&gt;) = ");
        sifr_generated_concat.push_str(unesc.as_str());
        sifr_generated_concat
    });
}
fn demo_sys() {
    println!("=== sys ===");
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(10usize);
        sifr_generated_concat.push_str("version = ");
        sifr_generated_concat.push_str(version().as_str());
        sifr_generated_concat
    });
    let ms: SifrInt = maxsize();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(14usize);
        sifr_generated_concat.push_str("maxsize > 0 = ");
        sifr_generated_concat.push_str((&ms > &SifrInt::from_i64(0)).to_string().as_str());
        sifr_generated_concat
    });
}
fn demo_configparser() {
    println!("=== configparser ===");
    let mut config: SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser =
        SifrGeneratedStdlibSifrX2econfigparserX2eConfigParser::new(None, false, false);
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2econfigparserX2eParsingError> =
        (|| {
            (&mut config)
                .read_string(&"[database]\nhost = db.example.com\nport = 5432\n".to_string())?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", e.message.clone());
        return;
    }
    let host_value: Option<String> =
        config.get(&"database".to_string(), &"host".to_string(), &None, false);
    let port_value: Option<String> =
        config.get(&"database".to_string(), &"port".to_string(), &None, false);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(7usize);
        sifr_generated_concat.push_str("host = ");
        sifr_generated_concat.push_str(
            host_value
                .map_or_else(
                    || "None".to_string(),
                    |sifr_generated_v| sifr_generated_v.to_string(),
                )
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(7usize);
        sifr_generated_concat.push_str("port = ");
        sifr_generated_concat.push_str(
            port_value
                .map_or_else(
                    || "None".to_string(),
                    |sifr_generated_v| sifr_generated_v.to_string(),
                )
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(11usize);
        sifr_generated_concat.push_str("has_host = ");
        sifr_generated_concat.push_str(
            config
                .has_option(&"database".to_string(), &"host".to_string())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(14usize);
        sifr_generated_concat.push_str("has_missing = ");
        sifr_generated_concat.push_str(
            config
                .has_option(&"database".to_string(), &"missing".to_string())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
}
fn demo_gzip() {
    println!("=== gzip ===");
    let data: String = "Sifr stdlib gzip compression!".to_string();
    let compressed: Vec<u8> = compress(&data);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(21usize);
        sifr_generated_concat.push_str("compressed len > 0 = ");
        sifr_generated_concat.push_str(
            (&SifrInt::from(compressed.len()) > &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let decompressed: String = decompress(&compressed)?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(15usize + decompressed.len());
            sifr_generated_concat.push_str("decompressed = ");
            sifr_generated_concat.push_str(decompressed.as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
}
fn demo_zipfile() {
    println!("=== zipfile ===");
    let zf: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
        SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
            "/tmp/sifr_demo_zipfile.zip".to_string(),
            "a".to_string(),
            SifrInt::from_i64(0),
        );
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        zf.create()?;
        println!("zip created = true");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(14usize);
            sifr_generated_concat.push_str("create error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        zf.write(&"demo.txt".to_string(), &"Hello from ZipFile!".to_string())?;
        let content: String = zf.read(&"demo.txt".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(14usize + content.len());
            sifr_generated_concat.push_str("zip content = ");
            sifr_generated_concat.push_str(content.as_str());
            sifr_generated_concat
        });
        let names: Vec<String> = zf.namelist()?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(19usize);
            sifr_generated_concat.push_str("zip namelist len = ");
            sifr_generated_concat.push_str(SifrInt::from(names.len()).to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(11usize);
            sifr_generated_concat.push_str("zip error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        remove_file(&"/tmp/sifr_demo_zipfile.zip".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
    }
}
fn main() {
    demo_operator();
    demo_calendar();
    demo_html();
    demo_sys();
    demo_configparser();
    demo_gzip();
    demo_zipfile();
}

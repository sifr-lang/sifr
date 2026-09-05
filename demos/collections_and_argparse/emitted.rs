// src/main.rs
mod sifr_generated_generated_support {
    use crate::{ParseError, SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec};
    pub(crate) use ::sifr_runtime::SifrInt;
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
    pub(crate) fn sifr_generated_is_digit_string(value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        for ch in value.chars().map(|c| c.to_string()) {
            if ch < "0".to_string() || ch > "9".to_string() {
                return false;
            }
        }
        true
    }
    pub(crate) fn sifr_generated_normalize_nargs(nargs: &str) -> String {
        if nargs.is_empty() {
            return "1".to_string();
        }
        if nargs == "?" || nargs == "*" || nargs == "+" {
            return {
                let mut sifr_generated_concat: String = String::with_capacity(nargs.len());
                sifr_generated_concat.push_str(nargs);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        if sifr_generated_is_digit_string(nargs) {
            let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                let parsed: SifrInt =
                    SifrInt::parse_decimal(&nargs, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                if &parsed > &SifrInt::from_i64(0) {
                    return Ok(Some(parsed.to_string()));
                }
                Ok(None)
            })();
            match sifr_generated_try_res {
                Ok(Some(sifr_generated_ret_val)) => {
                    return sifr_generated_ret_val;
                }
                Ok(None) => {}
                Err(sifr_generated_try_err) => {
                    let _e = sifr_generated_try_err.clone();
                    return "1".to_string();
                }
            }
        }
        "1".to_string()
    }
    pub(crate) fn sifr_generated_nargs_is_multi(nargs: &str) -> bool {
        let normalized: String = sifr_generated_normalize_nargs(nargs);
        if normalized == "*" || normalized == "+" {
            return true;
        }
        if sifr_generated_is_digit_string(&normalized) {
            let sifr_generated_try_res: Result<bool, ParseError> = (|| {
                let parsed: SifrInt =
                    SifrInt::parse_decimal(&normalized, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?;
                Ok(&parsed > &SifrInt::from_i64(1))
            })();
            match sifr_generated_try_res {
                Ok(sifr_generated_ret_val) => {
                    return sifr_generated_ret_val;
                }
                Err(sifr_generated_try_err) => {
                    let _e = sifr_generated_try_err.clone();
                    return false;
                }
            }
        }
        false
    }
    pub(crate) fn sifr_generated_coerce_bool(raw: &str) -> Option<String> {
        let normalized: String = raw.to_lowercase();
        if normalized == "1" || normalized == "true" || normalized == "yes" || normalized == "on" {
            return Some("true".to_string());
        }
        if normalized == "0" || normalized == "false" || normalized == "no" || normalized == "off" {
            return Some("false".to_string());
        }
        None
    }
    pub(crate) fn sifr_generated_copy_token(value: &Option<String>) -> String {
        let Some(value) = value.as_ref() else {
            return String::new();
        };
        {
            let mut sifr_generated_concat: String = String::with_capacity(value.len());
            sifr_generated_concat.push_str(value);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_derive_dest(name: &str) -> String {
        let sifr_generated_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
        if name.starts_with("--") {
            return {
                let sifr_generated_slice_src = &sifr_generated_chars_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(2).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = SifrInt::from(sifr_generated_slice_src.len())
                    .clamp_slice_bound(sifr_generated_slice_len);
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied()
                    .collect::<String>()
            }
            .replace('-', "_");
        }
        if name.starts_with("-") {
            return {
                let sifr_generated_slice_src = &sifr_generated_chars_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(1).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = SifrInt::from(sifr_generated_slice_src.len())
                    .clamp_slice_bound(sifr_generated_slice_len);
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied()
                    .collect::<String>()
            }
            .replace('-', "_");
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(name.len());
            sifr_generated_concat.push_str(name);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_is_option_like_token(
        specs: &[SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec],
        token: &str,
    ) -> bool {
        if token == "--" {
            return true;
        }
        if token.starts_with("--") {
            return true;
        }
        let (inline_has_value, inline_name, inline_value) =
            sifr_generated_split_inline_option(token);
        let _chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
        let _chars_inline_value: Vec<char> = inline_value.chars().collect::<Vec<char>>();
        let _ = inline_value;
        let lookup_name: String = if inline_has_value {
            {
                let mut sifr_generated_concat: String = String::with_capacity(inline_name.len());
                sifr_generated_concat.push_str(inline_name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        } else {
            let mut sifr_generated_concat: String = String::with_capacity(token.len());
            sifr_generated_concat.push_str(token);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        for spec in specs.iter().cloned() {
            if spec.kind.clone() == "positional" {
                continue;
            }
            if spec.name.clone() == lookup_name {
                return true;
            }
        }
        false
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        pub name: String,
        pub dest: String,
        pub kind: String,
        pub default_value: String,
        pub nargs: String,
        pub type_name: String,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        #[must_use]
        pub fn new(
            name: String,
            dest: String,
            kind: String,
            default_value: String,
            nargs: String,
            type_name: String,
        ) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(name.len());
                sifr_generated_concat.push_str(name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_a5eb0667427cce95_64657374: String = {
                let mut sifr_generated_concat: String = String::with_capacity(dest.len());
                sifr_generated_concat.push_str(dest.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = {
                let mut sifr_generated_concat: String = String::with_capacity(kind.len());
                sifr_generated_concat.push_str(kind.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_c029ceb935ca1970_64656661756c745f76616c7565: String = {
                let mut sifr_generated_concat: String = String::with_capacity(default_value.len());
                sifr_generated_concat.push_str(default_value.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_c4fccdff6d365b00_6e61726773: String =
                sifr_generated_normalize_nargs(&nargs);
            let sifr_generated_field_value_c23e4d7df5c6ddd5_747970655f6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(type_name.len());
                sifr_generated_concat.push_str(type_name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
                dest: sifr_generated_field_value_a5eb0667427cce95_64657374,
                kind: sifr_generated_field_value_ef9c96d721673243_6b696e64,
                default_value:
                    sifr_generated_field_value_c029ceb935ca1970_64656661756c745f76616c7565,
                nargs: sifr_generated_field_value_c4fccdff6d365b00_6e61726773,
                type_name: sifr_generated_field_value_c23e4d7df5c6ddd5_747970655f6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "ArgumentSpec(name={}, dest={}, kind={}, default_value={}, nargs={}, type_name={})",
                self.name, self.dest, self.kind, self.default_value, self.nargs, self.type_name
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub str_values: Vec<(String, String)>,
        pub bool_values: Vec<(String, bool)>,
        pub list_values: Vec<(String, Vec<String>)>,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub const fn new() -> Self {
            let sifr_generated_field_value_ad3f1317446e6bf0_5f7374725f76616c756573: Vec<(
                String,
                String,
            )> = Vec::new();
            let sifr_generated_field_value_1179342d80643edd_5f626f6f6c5f76616c756573: Vec<(
                String,
                bool,
            )> = Vec::new();
            let sifr_generated_field_value_9f4a2d21db1be045_5f6c6973745f76616c756573: Vec<(
                String,
                Vec<String>,
            )> = Vec::new();
            Self {
                str_values: sifr_generated_field_value_ad3f1317446e6bf0_5f7374725f76616c756573,
                bool_values: sifr_generated_field_value_1179342d80643edd_5f626f6f6c5f76616c756573,
                list_values: sifr_generated_field_value_9f4a2d21db1be045_5f6c6973745f76616c756573,
            }
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set(&mut self, name: &str, value: &str) {
            let mut updated: Vec<(String, String)> = Vec::new();
            let mut replaced: bool = false;
            for (key, current) in self.str_values.iter().cloned() {
                if key == *name {
                    updated.push((name.to_string(), value.to_string()));
                    replaced = true;
                } else {
                    updated.push((key.to_owned(), current.to_owned()));
                }
            }
            if !replaced {
                updated.push((name.to_string(), value.to_string()));
            }
            self.str_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set_bool(&mut self, name: &str, value: bool) {
            let mut updated: Vec<(String, bool)> = Vec::new();
            let mut replaced: bool = false;
            for (key, current) in self.bool_values.iter().cloned() {
                if key == *name {
                    updated.push((name.to_string(), value));
                    replaced = true;
                } else {
                    updated.push((key.to_owned(), current));
                }
            }
            if !replaced {
                updated.push((name.to_string(), value));
            }
            self.bool_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn set_list(&mut self, name: &str, values: &[String]) {
            let mut copied: Vec<String> = Vec::new();
            for value in values.iter().cloned() {
                copied.push(value.to_string());
            }
            let mut updated: Vec<(String, Vec<String>)> = Vec::new();
            for (key, current) in self.list_values.iter().cloned() {
                if key == *name {
                    continue;
                }
                updated.push((key.to_owned(), current.to_vec()));
            }
            updated.push((name.to_string(), copied.to_vec()));
            self.list_values = updated;
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub fn get(&self, name: &str, default: &str) -> String {
            for (key, value) in self.str_values.iter().cloned() {
                if key == *name {
                    return {
                        let mut sifr_generated_concat: String = String::with_capacity(value.len());
                        sifr_generated_concat.push_str(value.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                }
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(default.len());
                sifr_generated_concat.push_str(default);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub fn get_bool(&self, name: &str, default: bool) -> bool {
            for (key, value) in self.bool_values.iter().cloned() {
                if key == *name {
                    return value;
                }
            }
            for (key2, value2) in self.str_values.iter().cloned() {
                if key2 != *name {
                    continue;
                }
                let normalized: String = value2.to_lowercase();
                if normalized == "1"
                    || normalized == "true"
                    || normalized == "yes"
                    || normalized == "on"
                {
                    return true;
                }
                if normalized == "0"
                    || normalized == "false"
                    || normalized == "no"
                    || normalized == "off"
                {
                    return false;
                }
            }
            default
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        #[must_use]
        pub fn get_list(&self, name: &str) -> Vec<String> {
            for (key, values) in self.list_values.iter().cloned() {
                if key != *name {
                    continue;
                }
                let mut copied: Vec<String> = Vec::new();
                for value in values.iter().cloned() {
                    copied.push(value.to_string());
                }
                return copied;
            }
            Vec::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
        pub fn merge_from(&mut self, other: &SifrGeneratedStdlibSifrX2eargparseX2eNamespace) {
            for (key, value) in other.str_values.iter().cloned() {
                self.set(&key, &value);
            }
            for (key2, value2) in other.bool_values.iter().cloned() {
                self.set_bool(&key2, value2);
            }
            for (key3, values3) in other.list_values.iter().cloned() {
                self.set_list(&key3, &values3);
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub prog: String,
        pub specs: Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
        pub subparsers_dest: String,
        pub subparsers: Vec<(
            String,
            Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
        )>,
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn new(prog: String) -> Self {
            let sifr_generated_field_value_68bfad6e66c74136_5f70726f67: String = {
                let mut sifr_generated_concat: String = String::with_capacity(prog.len());
                sifr_generated_concat.push_str(prog.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_fe08c9a04e4710ae_5f7370656373: Vec<
                SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            > = Vec::new();
            let sifr_generated_field_value_d0dd847dfcb3acd5_5f737562706172736572735f64657374: String = "command"
                .to_string();
            let sifr_generated_field_value_bca9a861c9b63fd8_5f73756270617273657273: Vec<(
                String,
                Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
            )> = Vec::new();
            Self {
                prog: sifr_generated_field_value_68bfad6e66c74136_5f70726f67,
                specs: sifr_generated_field_value_fe08c9a04e4710ae_5f7370656373,
                subparsers_dest:
                    sifr_generated_field_value_d0dd847dfcb3acd5_5f737562706172736572735f64657374,
                subparsers: sifr_generated_field_value_bca9a861c9b63fd8_5f73756270617273657273,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn add_subparsers(&mut self, dest: &str) {
            if !dest.is_empty() {
                self.subparsers_dest = {
                    let mut sifr_generated_concat: String = String::with_capacity(dest.len());
                    sifr_generated_concat.push_str(dest);
                    sifr_generated_concat.push_str("");
                    sifr_generated_concat
                };
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn add_parser(
            &mut self,
            name: &str,
            parser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser,
        ) {
            let mut specs_copy: Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec> = Vec::new();
            for spec in parser.specs.iter().cloned() {
                specs_copy.push(spec.clone());
            }
            self.subparsers
                .push((name.to_string(), specs_copy.to_vec()));
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn sifr_generated_append_spec(
            &mut self,
            name: &str,
            dest: &str,
            action: &str,
            default: &str,
            nargs: &str,
            type_name: &str,
        ) {
            let mut resolved_dest: String = {
                let mut sifr_generated_concat: String = String::with_capacity(dest.len());
                sifr_generated_concat.push_str(dest);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if resolved_dest.is_empty() {
                resolved_dest = sifr_generated_derive_dest(name);
            }
            let mut kind: String = "positional".to_string();
            if name.starts_with("-") {
                if action == "store_true" {
                    kind = "flag".to_string();
                } else {
                    kind = "option".to_string();
                }
            }
            let spec: SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec =
                SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec::new(
                    name.to_owned(),
                    resolved_dest,
                    kind,
                    default.to_owned(),
                    nargs.to_owned(),
                    type_name.to_owned(),
                );
            self.specs.push(spec.clone());
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        pub fn add_argument_typed(
            &mut self,
            name: &str,
            dest: &str,
            action: &str,
            default: &str,
            nargs: &str,
            type_name: &str,
        ) {
            let mut normalized_type: String = {
                let mut sifr_generated_concat: String = String::with_capacity(type_name.len());
                sifr_generated_concat.push_str(type_name);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if normalized_type != "int"
                && normalized_type != "float"
                && normalized_type != "bool"
                && normalized_type != "str"
            {
                normalized_type = "str".to_string();
            }
            self.sifr_generated_append_spec(name, dest, action, default, nargs, &normalized_type);
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_find_subparser(
            &self,
            name: &str,
        ) -> Option<Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>> {
            for (parser_name, parser_specs) in self.subparsers.iter().cloned() {
                if parser_name == *name {
                    return Some(parser_specs);
                }
            }
            None
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_coerce_token(
            &self,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            token: &str,
        ) -> Option<String> {
            if spec.type_name.clone() == "int" {
                let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                    let parsed_int: SifrInt =
                        SifrInt::parse_decimal(&token, ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)
                            .map_err(|e| ParseError {
                                message: e.to_string(),
                            })?;
                    Ok(Some(parsed_int.to_string()))
                })(
                );
                match sifr_generated_try_res {
                    Ok(sifr_generated_ret_val) => {
                        return sifr_generated_ret_val;
                    }
                    Err(sifr_generated_try_err) => {
                        let _e = sifr_generated_try_err.clone();
                        return None;
                    }
                }
            }
            if spec.type_name.clone() == "float" {
                let sifr_generated_try_res: Result<Option<String>, ParseError> = (|| {
                    let parsed_float: f64 = token.parse::<f64>().map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    Ok(Some(parsed_float.to_string()))
                })(
                );
                match sifr_generated_try_res {
                    Ok(sifr_generated_ret_val) => {
                        return sifr_generated_ret_val;
                    }
                    Err(sifr_generated_try_err) => {
                        let _e = sifr_generated_try_err.clone();
                        return None;
                    }
                }
            }
            if spec.type_name.clone() == "bool" {
                return sifr_generated_coerce_bool(token);
            }
            Some({
                let mut sifr_generated_concat: String = String::with_capacity(token.len());
                sifr_generated_concat.push_str(token);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            })
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn sifr_generated_collect_option_values(
            &self,
            args: &[String],
            start: &SifrInt,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, SifrInt) {
            let mut values: Vec<String> = Vec::new();
            let mut i: SifrInt = start.clone();
            if spec.nargs.clone() == "?" {
                if &i >= &SifrInt::from(args.len()) {
                    return (values.to_vec(), i.clone());
                }
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt_value_6bfb1ab9e84751f4) = token_opt else {
                    return (values.to_vec(), &i + &SifrInt::from_i64(1));
                };
                let token_one_value_6bbe1fb9e813ac55: String =
                    sifr_generated_copy_token(&Some(token_opt_value_6bfb1ab9e84751f4.to_owned()));
                if !force_positional
                    && sifr_generated_is_option_like_token(
                        &self.specs,
                        &token_one_value_6bbe1fb9e813ac55,
                    )
                {
                    return (values.to_vec(), i.clone());
                }
                values.push(token_one_value_6bbe1fb9e813ac55.to_owned());
                return (values.to_vec(), &i + &SifrInt::from_i64(1));
            }
            if spec.nargs.clone() == "*" || spec.nargs.clone() == "+" {
                while &i < &SifrInt::from(args.len()) {
                    let token_opt2_value_c3002fe5b12ff372: Option<String> = {
                        let sifr_generated_checked_read_collection = &args;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(token_opt2_value_c3002fe5b12ff372) = token_opt2_value_c3002fe5b12ff372
                    else {
                        i = &i + &SifrInt::from_i64(1);
                        continue;
                    };
                    let token_many: String = sifr_generated_copy_token(&Some(
                        token_opt2_value_c3002fe5b12ff372.to_owned(),
                    ));
                    if !force_positional
                        && sifr_generated_is_option_like_token(&self.specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.to_owned());
                    i = &i + &SifrInt::from_i64(1);
                }
                return (values.to_vec(), i.clone());
            }
            let mut exact: SifrInt = SifrInt::from_i64(1);
            if sifr_generated_is_digit_string(&spec.nargs.clone()) {
                let sifr_generated_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: SifrInt = SifrInt::parse_decimal(
                        &spec.nargs.clone(),
                        ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                    )
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    if &parsed_count > &SifrInt::from_i64(0) {
                        exact = parsed_count.clone();
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let _e = sifr_generated_try_err.clone();
                    exact = SifrInt::from_i64(1);
                }
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &exact && &i < &SifrInt::from(args.len()) {
                let token_opt3_value_c30030e5b12ff525: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt3_value_c30030e5b12ff525) = token_opt3_value_c30030e5b12ff525
                else {
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                };
                let token_exact: String =
                    sifr_generated_copy_token(&Some(token_opt3_value_c30030e5b12ff525.to_owned()));
                if !force_positional
                    && sifr_generated_is_option_like_token(&self.specs, &token_exact)
                {
                    break;
                }
                values.push(token_exact.to_owned());
                i = &i + &SifrInt::from_i64(1);
                count = &count + &SifrInt::from_i64(1);
            }
            (values.to_vec(), i.clone())
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        pub fn sifr_generated_collect_positional_values(
            &self,
            args: &[String],
            start: &SifrInt,
            spec: &SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
            force_positional: bool,
        ) -> (Vec<String>, SifrInt) {
            let mut values: Vec<String> = Vec::new();
            let mut i: SifrInt = start.clone();
            if &i >= &SifrInt::from(args.len()) {
                return (values.to_vec(), i.clone());
            }
            if spec.nargs.clone() == "?" {
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(token_opt) = token_opt {
                    let token_one_value_6bbe1fb9e813ac55: String =
                        sifr_generated_copy_token(&Some(token_opt.to_owned()));
                    if !force_positional
                        && sifr_generated_is_option_like_token(
                            &self.specs,
                            &token_one_value_6bbe1fb9e813ac55,
                        )
                    {
                        return (values.to_vec(), i.clone());
                    }
                    values.push(token_one_value_6bbe1fb9e813ac55.to_owned());
                }
                return (values.to_vec(), &i + &SifrInt::from_i64(1));
            }
            if spec.nargs.clone() == "*" || spec.nargs.clone() == "+" {
                while &i < &SifrInt::from(args.len()) {
                    let token_opt2_value_c3002fe5b12ff372: Option<String> = {
                        let sifr_generated_checked_read_collection = &args;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(token_opt2_value_c3002fe5b12ff372) = token_opt2_value_c3002fe5b12ff372
                    else {
                        i = &i + &SifrInt::from_i64(1);
                        continue;
                    };
                    let token_many: String = sifr_generated_copy_token(&Some(
                        token_opt2_value_c3002fe5b12ff372.to_owned(),
                    ));
                    if !force_positional
                        && sifr_generated_is_option_like_token(&self.specs, &token_many)
                    {
                        break;
                    }
                    values.push(token_many.to_owned());
                    i = &i + &SifrInt::from_i64(1);
                }
                return (values.to_vec(), i.clone());
            }
            let mut exact: SifrInt = SifrInt::from_i64(1);
            if sifr_generated_is_digit_string(&spec.nargs.clone()) {
                let sifr_generated_try_res: Result<(), ParseError> = (|| {
                    let parsed_count: SifrInt = SifrInt::parse_decimal(
                        &spec.nargs.clone(),
                        ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                    )
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                    if &parsed_count > &SifrInt::from_i64(0) {
                        exact = parsed_count.clone();
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let _e = sifr_generated_try_err.clone();
                    exact = SifrInt::from_i64(1);
                }
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &exact && &i < &SifrInt::from(args.len()) {
                let token_opt3_value_c30030e5b12ff525: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(token_opt3) = token_opt3_value_c30030e5b12ff525 {
                    values.push(sifr_generated_copy_token(&Some(token_opt3.to_owned())));
                    count = &count + &SifrInt::from_i64(1);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            (values.to_vec(), i.clone())
        }
    }
    impl SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser {
        #[must_use]
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn parse_args(
            &self,
            args: &[String],
        ) -> SifrGeneratedStdlibSifrX2eargparseX2eNamespace {
            let mut ns: SifrGeneratedStdlibSifrX2eargparseX2eNamespace =
                SifrGeneratedStdlibSifrX2eargparseX2eNamespace::new();
            for spec in self.specs.iter().cloned() {
                if spec.kind.clone() == "flag" {
                    ns.set_bool(&spec.dest.clone(), false);
                } else if sifr_generated_nargs_is_multi(&spec.nargs.clone())
                    || spec.nargs.clone() == "*"
                    || spec.nargs.clone() == "+"
                {
                    ns.set_list(&spec.dest.clone(), &Vec::new());
                } else {
                    ns.set(&spec.dest.clone(), &spec.default_value.clone());
                }
            }
            if &SifrInt::from(self.subparsers.len()) > &SifrInt::from_i64(0)
                && &SifrInt::from(args.len()) > &SifrInt::from_i64(0)
            {
                let first_token: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(first_token) = first_token {
                    let command_name: String =
                        sifr_generated_copy_token(&Some(first_token.to_owned()));
                    let subparser_specs: Option<
                        Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec>,
                    > = self.sifr_generated_find_subparser(&command_name);
                    if let Some(subparser_specs) = subparser_specs {
                        ns.set(&self.subparsers_dest.clone(), &command_name);
                        let mut subparser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser =
                            SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser::new(command_name);
                        subparser.specs = subparser_specs;
                        let child_ns: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = subparser
                            .parse_args(&{
                                let sifr_generated_slice_src = &args;
                                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                                let sifr_generated_slice_start = SifrInt::from_i64(1)
                                    .clamp_slice_bound(sifr_generated_slice_len);
                                let sifr_generated_slice_stop = SifrInt::from(args.len())
                                    .clamp_slice_bound(sifr_generated_slice_len);
                                Vec::from_iter(
                                    sifr_generated_slice_src
                                        .iter()
                                        .skip(sifr_generated_slice_start)
                                        .take(
                                            sifr_generated_slice_stop
                                                .saturating_sub(sifr_generated_slice_start),
                                        )
                                        .cloned(),
                                )
                            });
                        ns.merge_from(&child_ns);
                        return ns;
                    }
                }
            }
            let mut positional_specs: Vec<SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec> =
                Vec::new();
            for spec2 in self.specs.iter().cloned() {
                if spec2.kind.clone() == "positional" {
                    positional_specs.push(spec2.clone());
                }
            }
            let mut i: SifrInt = SifrInt::from_i64(0);
            let mut positional_index: SifrInt = SifrInt::from_i64(0);
            let mut force_positional: bool = false;
            while &i < &SifrInt::from(args.len()) {
                let token_opt: Option<String> = {
                    let sifr_generated_checked_read_collection = &args;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(token_opt_value_6bfb1ab9e84751f4) = token_opt else {
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                };
                let token: String =
                    sifr_generated_copy_token(&Some(token_opt_value_6bfb1ab9e84751f4.to_owned()));
                if token == "--" && !force_positional {
                    force_positional = true;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                if token.starts_with("-") && !force_positional {
                    let (inline_has_value, inline_name, inline_value) =
                        sifr_generated_split_inline_option(&token);
                    let _chars_inline_name: Vec<char> = inline_name.chars().collect::<Vec<char>>();
                    let _chars_inline_value: Vec<char> =
                        inline_value.chars().collect::<Vec<char>>();
                    let lookup_name: String = if inline_has_value {
                        {
                            let mut sifr_generated_concat: String =
                                String::with_capacity(inline_name.len());
                            sifr_generated_concat.push_str(inline_name.as_str());
                            sifr_generated_concat.push_str("");
                            sifr_generated_concat
                        }
                    } else {
                        let mut sifr_generated_concat: String = String::with_capacity(token.len());
                        sifr_generated_concat.push_str(token.as_str());
                        sifr_generated_concat.push_str("");
                        sifr_generated_concat
                    };
                    let mut handled_option: bool = false;
                    for option_spec in self.specs.iter().cloned() {
                        if option_spec.kind.clone() == "positional" {
                            continue;
                        }
                        if option_spec.name.clone() != lookup_name {
                            continue;
                        }
                        handled_option = true;
                        if option_spec.kind.clone() == "flag" {
                            ns.set_bool(&option_spec.dest.clone(), true);
                            i = &i + &SifrInt::from_i64(1);
                            break;
                        }
                        let values: Vec<String> = if inline_has_value {
                            {
                                let values = vec![inline_value.to_owned()];
                                i = &i + &SifrInt::from_i64(1);
                                values
                            }
                        } else {
                            let (sifr_generated_tuple_unpack_0, sifr_generated_tuple_unpack_1) =
                                self.sifr_generated_collect_option_values(
                                    args,
                                    &(&i + &SifrInt::from_i64(1)),
                                    &option_spec,
                                    force_positional,
                                );
                            let values = sifr_generated_tuple_unpack_0;
                            i = sifr_generated_tuple_unpack_1;
                            values
                        };
                        if sifr_generated_nargs_is_multi(&option_spec.nargs.clone())
                            || option_spec.nargs.clone() == "*"
                            || option_spec.nargs.clone() == "+"
                        {
                            let mut converted_values: Vec<String> = Vec::new();
                            for raw in values.iter().cloned() {
                                let coerced: Option<String> =
                                    self.sifr_generated_coerce_token(&option_spec, &raw);
                                let Some(coerced_value_9a594b45880c48d4) = coerced else {
                                    continue;
                                };
                                converted_values.push(sifr_generated_copy_token(&Some(
                                    coerced_value_9a594b45880c48d4.to_owned(),
                                )));
                            }
                            ns.set_list(&option_spec.dest.clone(), &converted_values);
                        } else if &SifrInt::from(values.len()) > &SifrInt::from_i64(0) {
                            let first_value: Option<String> = {
                                let sifr_generated_checked_read_collection = &values;
                                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(first_value) = first_value {
                                let token_value: String =
                                    sifr_generated_copy_token(&Some(first_value.to_owned()));
                                let coerced_first: Option<String> =
                                    self.sifr_generated_coerce_token(&option_spec, &token_value);
                                if let Some(coerced_first) = coerced_first {
                                    let coerced_value: String =
                                        sifr_generated_copy_token(&Some(coerced_first.to_owned()));
                                    ns.set(&option_spec.dest.clone(), &coerced_value);
                                    if option_spec.type_name.clone() == "bool" {
                                        ns.set_bool(
                                            &option_spec.dest.clone(),
                                            coerced_value == "true",
                                        );
                                    }
                                }
                            }
                        }
                        break;
                    }
                    if handled_option {
                        continue;
                    }
                }
                if &positional_index < &SifrInt::from(positional_specs.len()) {
                    let positional_spec_value_f84646974d692a63: Option<
                        SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec,
                    > = {
                        let sifr_generated_checked_read_collection = &positional_specs;
                        let sifr_generated_checked_read_index = positional_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(positional_spec) = positional_spec_value_f84646974d692a63 {
                        let (values2_value_a37f29e9b1875a8b, next_i2) = self
                            .sifr_generated_collect_positional_values(
                                args,
                                &i,
                                &positional_spec,
                                force_positional,
                            );
                        if sifr_generated_nargs_is_multi(&positional_spec.nargs.clone())
                            || positional_spec.nargs.clone() == "*"
                            || positional_spec.nargs.clone() == "+"
                        {
                            let mut converted_values2_value_d5873b4bca1f063e: Vec<String> =
                                Vec::new();
                            for raw2 in values2_value_a37f29e9b1875a8b.iter().cloned() {
                                let coerced2_value_5203cd262cdfded2: Option<String> =
                                    self.sifr_generated_coerce_token(&positional_spec, &raw2);
                                let Some(coerced2_value_5203cd262cdfded2) =
                                    coerced2_value_5203cd262cdfded2
                                else {
                                    continue;
                                };
                                converted_values2_value_d5873b4bca1f063e.push(
                                    sifr_generated_copy_token(&Some(
                                        coerced2_value_5203cd262cdfded2.to_owned(),
                                    )),
                                );
                            }
                            ns.set_list(
                                &positional_spec.dest.clone(),
                                &converted_values2_value_d5873b4bca1f063e,
                            );
                        } else if &SifrInt::from(values2_value_a37f29e9b1875a8b.len())
                            > &SifrInt::from_i64(0)
                        {
                            let first_value2_value_418fe1d187bd6a23: Option<String> = {
                                let sifr_generated_checked_read_collection =
                                    &values2_value_a37f29e9b1875a8b;
                                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(first_value2) = first_value2_value_418fe1d187bd6a23 {
                                let token_value2_value_5399f4e73b5dc95a: String =
                                    sifr_generated_copy_token(&Some(first_value2.to_owned()));
                                let coerced_first2_value_09853aeb8e001655: Option<String> = self
                                    .sifr_generated_coerce_token(
                                        &positional_spec,
                                        &token_value2_value_5399f4e73b5dc95a,
                                    );
                                if let Some(coerced_first2) = coerced_first2_value_09853aeb8e001655
                                {
                                    let coerced_value2_value_8b96ef5a277fa4a4: String =
                                        sifr_generated_copy_token(&Some(coerced_first2.to_owned()));
                                    ns.set(
                                        &positional_spec.dest.clone(),
                                        &coerced_value2_value_8b96ef5a277fa4a4,
                                    );
                                    if positional_spec.type_name.clone() == "bool" {
                                        ns.set_bool(
                                            &positional_spec.dest.clone(),
                                            coerced_value2_value_8b96ef5a277fa4a4 == "true",
                                        );
                                    }
                                }
                            }
                        }
                        i = next_i2.clone();
                        positional_index = &positional_index + &SifrInt::from_i64(1);
                        continue;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            ns
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, SifrInt>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn new(source: Option<HashMap<T, SifrInt>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<SifrInt> = source.get(&key).cloned();
                    if let Some(value) = value.clone() {
                        {
                            let sifr_generated_assign_value = value.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2_value_2127bacf1a4dd308: Option<SifrInt> = counts.get(&item).cloned();
                    if let Some(value2) = value2_value_2127bacf1a4dd308.clone() {
                        {
                            let sifr_generated_assign_value = &value2 + &SifrInt::from_i64(1);
                            {
                                let sifr_generated_assign_key = item.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let sifr_generated_assign_value = SifrInt::from_i64(1);
                        {
                            let sifr_generated_assign_key = item.clone();
                            counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
            let sifr_generated_field_value_c341febe5aae51e5_636f756e7473: HashMap<T, SifrInt> =
                counts;
            Self {
                counts: sifr_generated_field_value_c341febe5aae51e5_636f756e7473,
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn get(&self, key: &T, default: &SifrInt) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            let Some(val) = val.clone() else {
                return default.clone();
            };
            val
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone + PartialOrd>
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        #[must_use]
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(result.len()) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(result.len()) {
                    let left: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = j.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left
                        && let Some(right) = right
                        && right < left
                    {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
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
                    j = &j + &SifrInt::from_i64(1);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Add<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn add(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let total: SifrInt = &a_val + &b_count;
                    if &total > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = total.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            for key2 in Box::new(other.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let already: Option<SifrInt> = new_counts.get(&key2).cloned();
                if already.is_none() {
                    let b_val2: Option<SifrInt> = other.counts.get(&key2).cloned();
                    if let Some(b_val2) = b_val2
                        && &b_val2 > &SifrInt::from_i64(0)
                    {
                        {
                            let sifr_generated_assign_value = b_val2.clone();
                            {
                                let sifr_generated_assign_key = key2.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Sub<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn sub(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let diff: SifrInt = &a_val - &b_count;
                    if &diff > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = diff.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
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
}
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eArgumentSpec;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eargparseX2eNamespace;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
fn main() {
    let counter: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(
            None,
            Some(vec![
                "parse".to_string(),
                "parse".to_string(),
                "emit".to_string(),
            ]),
        );
    assert_eq!(
        &counter.get(&"parse".to_string(), &SifrInt::from_i64(0)),
        &SifrInt::from_i64(2)
    );
    let mut attempts: HashMap<String, SifrInt> = HashMap::new();
    {
        let sifr_generated_elem = attempts
            .entry("collections_and_argparse".to_string())
            .or_insert(SifrInt::from_i64(0));
        *sifr_generated_elem += SifrInt::from_i64(1);
    }
    assert_eq!(
        &*attempts
            .entry("collections_and_argparse".to_string())
            .or_insert(SifrInt::from_i64(0)),
        &SifrInt::from_i64(1)
    );
    let mut parser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser =
        SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser::new("sifr".to_string());
    (&mut parser).add_subparsers(&"cmd".to_string());
    let mut run_parser: SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser =
        SifrGeneratedStdlibSifrX2eargparseX2eArgumentParser::new("run".to_string());
    (&mut run_parser).add_argument_typed(
        &"--strict".to_string(),
        &"strict".to_string(),
        &"store_true".to_string(),
        &String::new(),
        &"1".to_string(),
        &"str".to_string(),
    );
    (&mut run_parser).add_argument_typed(
        &"--level".to_string(),
        &"level".to_string(),
        &"store".to_string(),
        &"0".to_string(),
        &"1".to_string(),
        &"int".to_string(),
    );
    (&mut run_parser).add_argument_typed(
        &"--custom-level".to_string(),
        &"custom_level".to_string(),
        &"store".to_string(),
        &"0".to_string(),
        &"1".to_string(),
        &"int".to_string(),
    );
    (&mut run_parser).add_argument_typed(
        &"targets".to_string(),
        &"targets".to_string(),
        &"store".to_string(),
        &String::new(),
        &"+".to_string(),
        &"str".to_string(),
    );
    (&mut parser).add_parser(&"run".to_string(), run_parser);
    let parsed_value_e06e69d836b17138: SifrGeneratedStdlibSifrX2eargparseX2eNamespace = parser
        .parse_args(&vec![
            "run".to_string(),
            "--strict".to_string(),
            "--level".to_string(),
            "2".to_string(),
            "--custom-level".to_string(),
            "3".to_string(),
            "main.sifr".to_string(),
        ]);
    assert_eq!(
        parsed_value_e06e69d836b17138.get(&"cmd".to_string(), &String::new()),
        "run"
    );
    assert!(parsed_value_e06e69d836b17138.get_bool(&"strict".to_string(), false));
    assert_eq!(
        parsed_value_e06e69d836b17138.get(&"level".to_string(), &String::new()),
        "2"
    );
    assert_eq!(
        parsed_value_e06e69d836b17138.get(&"custom_level".to_string(), &String::new()),
        "3"
    );
    assert_eq!(
        format!(
            "{:?}",
            parsed_value_e06e69d836b17138.get_list(&"targets".to_string())
        ),
        "[\"main.sifr\"]"
    );
}

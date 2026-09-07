// src/main.rs
mod sifr_generated_generated_support {
    use crate::{SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError};
    pub(crate) use ::sifr_runtime::SifrInt;
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
    pub(crate) fn uuid4() -> String {
        ::sifr_stdlib::uuid::uuid4()
    }
    pub(crate) fn uuid3_text(namespace: &str, name: &str) -> String {
        ::sifr_stdlib::uuid::uuid3_text(namespace, name)
    }
    pub(crate) fn uuid5_text(namespace: &str, name: &str) -> String {
        ::sifr_stdlib::uuid::uuid5_text(namespace, name)
    }
    pub(crate) fn sifr_generated_to_lower_hex_char(ch: &str) -> String {
        if ch == "A" {
            return "a".to_string();
        }
        if ch == "B" {
            return "b".to_string();
        }
        if ch == "C" {
            return "c".to_string();
        }
        if ch == "D" {
            return "d".to_string();
        }
        if ch == "E" {
            return "e".to_string();
        }
        if ch == "F" {
            return "f".to_string();
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(ch.len());
            sifr_generated_concat.push_str(ch);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_is_hex_char(ch: &str) -> bool {
        if ch == "0" {
            return true;
        }
        if ch == "1" {
            return true;
        }
        if ch == "2" {
            return true;
        }
        if ch == "3" {
            return true;
        }
        if ch == "4" {
            return true;
        }
        if ch == "5" {
            return true;
        }
        if ch == "6" {
            return true;
        }
        if ch == "7" {
            return true;
        }
        if ch == "8" {
            return true;
        }
        if ch == "9" {
            return true;
        }
        if ch == "a" {
            return true;
        }
        if ch == "b" {
            return true;
        }
        if ch == "c" {
            return true;
        }
        if ch == "d" {
            return true;
        }
        if ch == "e" {
            return true;
        }
        if ch == "f" {
            return true;
        }
        if ch == "A" {
            return true;
        }
        if ch == "B" {
            return true;
        }
        if ch == "C" {
            return true;
        }
        if ch == "D" {
            return true;
        }
        if ch == "E" {
            return true;
        }
        if ch == "F" {
            return true;
        }
        false
    }
    pub(crate) fn sifr_generated_hex_digit_value(ch: &str) -> SifrInt {
        if ch == "0" {
            return SifrInt::from_i64(0);
        }
        if ch == "1" {
            return SifrInt::from_i64(1);
        }
        if ch == "2" {
            return SifrInt::from_i64(2);
        }
        if ch == "3" {
            return SifrInt::from_i64(3);
        }
        if ch == "4" {
            return SifrInt::from_i64(4);
        }
        if ch == "5" {
            return SifrInt::from_i64(5);
        }
        if ch == "6" {
            return SifrInt::from_i64(6);
        }
        if ch == "7" {
            return SifrInt::from_i64(7);
        }
        if ch == "8" {
            return SifrInt::from_i64(8);
        }
        if ch == "9" {
            return SifrInt::from_i64(9);
        }
        if ch == "a" || ch == "A" {
            return SifrInt::from_i64(10);
        }
        if ch == "b" || ch == "B" {
            return SifrInt::from_i64(11);
        }
        if ch == "c" || ch == "C" {
            return SifrInt::from_i64(12);
        }
        if ch == "d" || ch == "D" {
            return SifrInt::from_i64(13);
        }
        if ch == "e" || ch == "E" {
            return SifrInt::from_i64(14);
        }
        if ch == "f" || ch == "F" {
            return SifrInt::from_i64(15);
        }
        -&SifrInt::from_i64(1)
    }
    pub(crate) fn sifr_generated_substring(value: &str, start: SifrInt, end: SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = start.clone();
        while &i < &end {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
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
    pub(crate) fn sifr_generated_starts_with(value: &str, prefix: &str) -> bool {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let sifr_generated_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_value.len())
            < &SifrInt::from(sifr_generated_chars_prefix.len())
        {
            return false;
        }
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_prefix.len()) {
            let left: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let right: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_prefix.len());
                sifr_generated_chars_prefix
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if left != right {
                return false;
            }
            i = &i + &SifrInt::from_i64(1);
        }
        true
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_canonical_uuid_text(
        input_text: &str,
    ) -> Result<String, ValueError> {
        let mut normalized_input: String = {
            let mut sifr_generated_concat: String = String::with_capacity(input_text.len());
            sifr_generated_concat.push_str(input_text);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let mut sifr_generated_chars_normalized_input: Vec<char> =
            if sifr_generated_starts_with(&normalized_input, &"urn:uuid:".to_string()) {
                {
                    normalized_input = sifr_generated_substring(
                        &normalized_input,
                        SifrInt::from_i64(9),
                        SifrInt::from(normalized_input.chars().count()),
                    );
                    normalized_input.chars().collect::<Vec<char>>()
                }
            } else {
                normalized_input.chars().collect::<Vec<char>>()
            };
        if &SifrInt::from(sifr_generated_chars_normalized_input.len()) >= &SifrInt::from_i64(2) {
            let first: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(0);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let last: Option<String> = {
                let sifr_generated_string_index =
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if first == Some("{".to_string()) && last == Some("}".to_string()) {
                normalized_input = sifr_generated_substring(
                    &normalized_input,
                    SifrInt::from_i64(1),
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1),
                );
                sifr_generated_chars_normalized_input =
                    normalized_input.chars().collect::<Vec<char>>();
            }
        }
        let input_len: SifrInt = SifrInt::from(sifr_generated_chars_normalized_input.len());
        let mut hex_only: String = String::new();
        let mut sifr_generated_chars_hex_only: Vec<char> = hex_only.chars().collect::<Vec<char>>();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &input_len {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "-" {
                } else {
                    if !sifr_generated_is_hex_char(&ch) {
                        return Err(ValueError::new("invalid UUID hex string".to_string()));
                    }
                    let sifr_generated_string_concat_hex_only_0 =
                        sifr_generated_to_lower_hex_char(&ch);
                    hex_only.push_str(sifr_generated_string_concat_hex_only_0.as_str());
                    sifr_generated_chars_hex_only
                        .extend(sifr_generated_string_concat_hex_only_0.as_str().chars());
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(sifr_generated_chars_hex_only.len()) != &SifrInt::from_i64(32) {
            return Err(ValueError::new(
                "UUID hex string must be 32 hex characters".to_string(),
            ));
        }
        if &input_len == &SifrInt::from_i64(36) {
            let h1: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(8);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h2: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h3: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(18);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h4: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(23);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if h1 != Some("-".to_string())
                || h2 != Some("-".to_string())
                || h3 != Some("-".to_string())
                || h4 != Some("-".to_string())
            {
                return Err(ValueError::new("invalid UUID hex string".to_string()));
            }
        } else if &input_len != &SifrInt::from_i64(32) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
        let mut canonical: String = String::new();
        let mut j: SifrInt = SifrInt::from_i64(0);
        while &j < &SifrInt::from(sifr_generated_chars_hex_only.len()) {
            if &j == &SifrInt::from_i64(8)
                || &j == &SifrInt::from_i64(12)
                || &j == &SifrInt::from_i64(16)
                || &j == &SifrInt::from_i64(20)
            {
                canonical.push('-');
            }
            let part: Option<String> = {
                let sifr_generated_string_index = j.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_hex_only.len());
                sifr_generated_chars_hex_only
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(part) = part {
                canonical.push_str(part.as_str());
            }
            j = &j + &SifrInt::from_i64(1);
        }
        Ok(canonical)
    }
    pub(crate) fn uuid4_obj() -> SifrGeneratedStdlibSifrX2euuidX2eUUID {
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(uuid4())
    }
    pub(crate) fn uuid_from_hex(
        hex_str: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError>,
            ValueError,
        > = (|| {
            let canonical: String = sifr_generated_canonical_uuid_text(hex_str)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2euuidX2eUUID::new(canonical)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(ValueError::new(e.message.clone()))
        })
    }
    pub(crate) fn uuid3(
        namespace: &SifrGeneratedStdlibSifrX2euuidX2eUUID,
        name: &str,
    ) -> SifrGeneratedStdlibSifrX2euuidX2eUUID {
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(uuid3_text(&namespace.to_str(), name))
    }
    pub(crate) fn uuid5(
        namespace: &SifrGeneratedStdlibSifrX2euuidX2eUUID,
        name: &str,
    ) -> SifrGeneratedStdlibSifrX2euuidX2eUUID {
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(uuid5_text(&namespace.to_str(), name))
    }
    #[expect(
        non_snake_case,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) fn NAMESPACE_DNS() -> SifrGeneratedStdlibSifrX2euuidX2eUUID {
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
        )
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2euuidX2eUUID {
        pub hex: String,
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn new(hex_str: String) -> Self {
            let sifr_generated_field_value_123cb3437a89ad57_5f686578: String = {
                let mut sifr_generated_concat: String = String::with_capacity(hex_str.len());
                sifr_generated_concat.push_str(hex_str.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                hex: sifr_generated_field_value_123cb3437a89ad57_5f686578,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn hex(&self) -> String {
            let mut result: String = String::new();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &SifrInt::from(self.hex.chars().count()) {
                let ch: Option<String> = {
                    let sifr_generated_string_chars = self.hex.chars().collect::<Vec<char>>();
                    let sifr_generated_string_index = i.clone();
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_string_chars.len());
                    sifr_generated_string_chars
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                if let Some(ch) = ch
                    && ch != "-"
                {
                    result.push_str(ch.as_str());
                }
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn to_str(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.hex.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn version(&self) -> SifrInt {
            let marker: Option<String> = {
                let sifr_generated_string_chars = self.hex.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = SifrInt::from_i64(14);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let Some(marker_value_eddcb72b15486e77) = marker else {
                return -&SifrInt::from_i64(1);
            };
            sifr_generated_hex_digit_value(&marker_value_eddcb72b15486e77)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2euuidX2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self.hex)
        }
    }
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
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2euuidX2eUUID;
pub use sifr_generated_project_nominals::ValueError;
fn is_canonical_shape(value: &str) -> bool {
    let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if &SifrInt::from(sifr_generated_chars_value.len()) != &SifrInt::from_i64(36) {
        return false;
    }
    let h1: Option<String> = {
        let sifr_generated_string_index = SifrInt::from_i64(8);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string());
    let h2: Option<String> = {
        let sifr_generated_string_index = SifrInt::from_i64(13);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string());
    let h3: Option<String> = {
        let sifr_generated_string_index = SifrInt::from_i64(18);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string());
    let h4: Option<String> = {
        let sifr_generated_string_index = SifrInt::from_i64(23);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string());
    h1 == Some("-".to_string())
        && h2 == Some("-".to_string())
        && h3 == Some("-".to_string())
        && h4 == Some("-".to_string())
}
fn collect_generated_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let id_text: String = uuid4();
    let sifr_generated_chars_id_text: Vec<char> = id_text.chars().collect::<Vec<char>>();
    actual.push(is_canonical_shape(&id_text));
    actual.push(
        {
            let sifr_generated_string_index = SifrInt::from_i64(14);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_id_text.len());
            sifr_generated_chars_id_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string())
            == Some("4".to_string()),
    );
    let obj: SifrGeneratedStdlibSifrX2euuidX2eUUID = uuid4_obj();
    actual.push(is_canonical_shape(&obj.to_str()) && &obj.version() == &SifrInt::from_i64(4));
    actual
}
fn collect_parse_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut parsed_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"550E8400E29B41D4A716446655440000".to_string())?;
        parsed_ok = parsed.to_str() == "550e8400-e29b-41d4-a716-446655440000";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parsed_v1_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed_v1_value_b1aa999388397088: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"550e8400-e29b-11d4-a716-446655440000".to_string())?;
        parsed_v1_ok = &parsed_v1_value_b1aa999388397088.version() == &SifrInt::from_i64(1);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        parsed_v1_ok = false;
    }
    actual.push(parsed_v1_ok);
    actual
}
fn collect_negative_and_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut invalid_rejected: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _bad: SifrGeneratedStdlibSifrX2euuidX2eUUID = uuid_from_hex(&"invalid".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        invalid_rejected = true;
    }
    actual.push(invalid_rejected);
    let ctor_passthrough: SifrGeneratedStdlibSifrX2euuidX2eUUID =
        SifrGeneratedStdlibSifrX2euuidX2eUUID::new(
            "550e8400-e29b-41d4-a716-44665544000z".to_string(),
        );
    actual.push(
        ctor_passthrough.to_str().as_str()
            == "550e8400-e29b-41d4-a716-44665544000z".to_string().as_str(),
    );
    let mut ctor_curly_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let ctor_curly: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"{550E8400-E29B-41D4-A716-446655440000}".to_string())?;
        ctor_curly_ok = ctor_curly.to_str() == "550e8400-e29b-41d4-a716-446655440000";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        ctor_curly_ok = false;
    }
    actual.push(ctor_curly_ok);
    let obj: SifrGeneratedStdlibSifrX2euuidX2eUUID = uuid4_obj();
    actual.push(&SifrInt::from(obj.hex().chars().count()) == &SifrInt::from_i64(32));
    actual.push(
        &uuid3(&NAMESPACE_DNS(), &"python.org".to_string()).version() == &SifrInt::from_i64(3),
    );
    actual.push(
        &uuid5(&NAMESPACE_DNS(), &"python.org".to_string()).version() == &SifrInt::from_i64(5),
    );
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![
        true, true, true, true, true, true, true, true, true, true, true,
    ];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_generated_actual());
    append_all(&mut actual, &collect_parse_actual());
    append_all(&mut actual, &collect_negative_and_class_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("uuid uuid parity demo: pass");
}

// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    pub fn uuid4() -> String {
        ::sifr_stdlib::uuid::uuid4()
    }
    pub fn uuid3_text(namespace: &String, name: &String) -> String {
        ::sifr_stdlib::uuid::uuid3_text(namespace, name)
    }
    pub fn uuid5_text(namespace: &String, name: &String) -> String {
        ::sifr_stdlib::uuid::uuid5_text(namespace, name)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub _hex: String,
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn new(hex_str: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    hex_str.len() + 0usize,
                );
                __sifr_concat.push_str((hex_str).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            Self { _hex: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn hex(&self) -> String {
            let mut result: String = "".to_string();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &SifrInt::from(self._hex.chars().count())) {
                let ch: Option<String> = Some({
                    let __indexed_char_option = self
                        ._hex
                        .clone()
                        .chars()
                        .nth(::sifr_runtime::to_usize_proven(&(i)))
                        .map(|c| c.to_string());
                    __indexed_char_option.as_slice()[0_usize].clone()
                });
                if let Some(ch) = ch {
                    if ch != "-" {
                        result.push_str((ch).as_str());
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn urn(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(9usize + 0usize);
                __sifr_concat.push_str("urn:uuid:");
                __sifr_concat.push_str((self._hex.clone()).as_str());
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._hex.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2euuid_x2eUUID {
        pub fn version(&self) -> SifrInt {
            let marker: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = SifrInt::from_i64(14);
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_str.chars().count());
                __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
            };
            let Some(marker) = marker else {
                return -&SifrInt::from_i64(1);
            };
            _hex_digit_value(&marker)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2euuid_x2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self._hex)
        }
    }
    pub fn _hex_digit_value(ch: &String) -> SifrInt {
        if (ch).as_str() == "0" {
            return SifrInt::from_i64(0);
        }
        if (ch).as_str() == "1" {
            return SifrInt::from_i64(1);
        }
        if (ch).as_str() == "2" {
            return SifrInt::from_i64(2);
        }
        if (ch).as_str() == "3" {
            return SifrInt::from_i64(3);
        }
        if (ch).as_str() == "4" {
            return SifrInt::from_i64(4);
        }
        if (ch).as_str() == "5" {
            return SifrInt::from_i64(5);
        }
        if (ch).as_str() == "6" {
            return SifrInt::from_i64(6);
        }
        if (ch).as_str() == "7" {
            return SifrInt::from_i64(7);
        }
        if (ch).as_str() == "8" {
            return SifrInt::from_i64(8);
        }
        if (ch).as_str() == "9" {
            return SifrInt::from_i64(9);
        }
        if ((ch).as_str() == "a") || ((ch).as_str() == "A") {
            return SifrInt::from_i64(10);
        }
        if ((ch).as_str() == "b") || ((ch).as_str() == "B") {
            return SifrInt::from_i64(11);
        }
        if ((ch).as_str() == "c") || ((ch).as_str() == "C") {
            return SifrInt::from_i64(12);
        }
        if ((ch).as_str() == "d") || ((ch).as_str() == "D") {
            return SifrInt::from_i64(13);
        }
        if ((ch).as_str() == "e") || ((ch).as_str() == "E") {
            return SifrInt::from_i64(14);
        }
        if ((ch).as_str() == "f") || ((ch).as_str() == "F") {
            return SifrInt::from_i64(15);
        }
        -&SifrInt::from_i64(1)
    }
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
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2euuid_x2eUUID;
use ::sifr_runtime::SifrInt;
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            Some(actual[::sifr_runtime::to_usize_proven(& (i))]) == expected
            .get(::sifr_runtime::to_usize_proven(& (i))).copied()
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn uuid4() -> String {
    ::sifr_stdlib::uuid::uuid4()
}
fn uuid3_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid3_text(namespace, name)
}
fn uuid5_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid5_text(namespace, name)
}
fn _to_lower_hex_char(ch: &String) -> String {
    if (ch).as_str() == "A" {
        return "a".to_string();
    }
    if (ch).as_str() == "B" {
        return "b".to_string();
    }
    if (ch).as_str() == "C" {
        return "c".to_string();
    }
    if (ch).as_str() == "D" {
        return "d".to_string();
    }
    if (ch).as_str() == "E" {
        return "e".to_string();
    }
    if (ch).as_str() == "F" {
        return "f".to_string();
    }
    {
        let mut __sifr_concat: String = String::with_capacity(ch.len() + 0usize);
        __sifr_concat.push_str((ch).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _is_hex_char(ch: &String) -> bool {
    if (ch).as_str() == "0" {
        return true;
    }
    if (ch).as_str() == "1" {
        return true;
    }
    if (ch).as_str() == "2" {
        return true;
    }
    if (ch).as_str() == "3" {
        return true;
    }
    if (ch).as_str() == "4" {
        return true;
    }
    if (ch).as_str() == "5" {
        return true;
    }
    if (ch).as_str() == "6" {
        return true;
    }
    if (ch).as_str() == "7" {
        return true;
    }
    if (ch).as_str() == "8" {
        return true;
    }
    if (ch).as_str() == "9" {
        return true;
    }
    if (ch).as_str() == "a" {
        return true;
    }
    if (ch).as_str() == "b" {
        return true;
    }
    if (ch).as_str() == "c" {
        return true;
    }
    if (ch).as_str() == "d" {
        return true;
    }
    if (ch).as_str() == "e" {
        return true;
    }
    if (ch).as_str() == "f" {
        return true;
    }
    if (ch).as_str() == "A" {
        return true;
    }
    if (ch).as_str() == "B" {
        return true;
    }
    if (ch).as_str() == "C" {
        return true;
    }
    if (ch).as_str() == "D" {
        return true;
    }
    if (ch).as_str() == "E" {
        return true;
    }
    if (ch).as_str() == "F" {
        return true;
    }
    false
}
fn _hex_digit_value(ch: &String) -> SifrInt {
    if (ch).as_str() == "0" {
        return SifrInt::from_i64(0);
    }
    if (ch).as_str() == "1" {
        return SifrInt::from_i64(1);
    }
    if (ch).as_str() == "2" {
        return SifrInt::from_i64(2);
    }
    if (ch).as_str() == "3" {
        return SifrInt::from_i64(3);
    }
    if (ch).as_str() == "4" {
        return SifrInt::from_i64(4);
    }
    if (ch).as_str() == "5" {
        return SifrInt::from_i64(5);
    }
    if (ch).as_str() == "6" {
        return SifrInt::from_i64(6);
    }
    if (ch).as_str() == "7" {
        return SifrInt::from_i64(7);
    }
    if (ch).as_str() == "8" {
        return SifrInt::from_i64(8);
    }
    if (ch).as_str() == "9" {
        return SifrInt::from_i64(9);
    }
    if ((ch).as_str() == "a") || ((ch).as_str() == "A") {
        return SifrInt::from_i64(10);
    }
    if ((ch).as_str() == "b") || ((ch).as_str() == "B") {
        return SifrInt::from_i64(11);
    }
    if ((ch).as_str() == "c") || ((ch).as_str() == "C") {
        return SifrInt::from_i64(12);
    }
    if ((ch).as_str() == "d") || ((ch).as_str() == "D") {
        return SifrInt::from_i64(13);
    }
    if ((ch).as_str() == "e") || ((ch).as_str() == "E") {
        return SifrInt::from_i64(14);
    }
    if ((ch).as_str() == "f") || ((ch).as_str() == "F") {
        return SifrInt::from_i64(15);
    }
    -&SifrInt::from_i64(1)
}
fn _substring(value: &String, start: SifrInt, end: SifrInt) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = start.clone();
    while &i < &end {
        let ch: Option<String> = __sifr_chars_value
            .get(::sifr_runtime::to_usize_proven(&(i.clone())))
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let __sifr_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_value.len())
        < &SifrInt::from(__sifr_chars_prefix.len()))
    {
        return false;
    }
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_prefix.len())) {
        let left: Option<String> = __sifr_chars_value
            .get(::sifr_runtime::to_usize_proven(&(i.clone())))
            .map(|c| c.to_string());
        let right: Option<String> = Some({
            let __indexed_char_option = __sifr_chars_prefix
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if (left != right) {
            return false;
        }
        i = &i + &SifrInt::from_i64(1);
    }
    true
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = {
        let mut __sifr_concat: String = String::with_capacity(input_text.len() + 0usize);
        __sifr_concat.push_str((input_text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            SifrInt::from_i64(9),
            SifrInt::from(normalized_input.chars().count()),
        );
    }
    if (&SifrInt::from(normalized_input.chars().count()) >= &SifrInt::from_i64(2)) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = SifrInt::from_i64(0);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_str.chars().count());
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = SifrInt::from(normalized_input.chars().count())
                - SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_str.chars().count());
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if (first == Some("{".to_string())) && (last == Some("}".to_string())) {
            normalized_input = _substring(
                &normalized_input,
                SifrInt::from_i64(1),
                SifrInt::from(normalized_input.chars().count()) - SifrInt::from_i64(1),
            );
        }
    }
    let input_len: SifrInt = SifrInt::from(normalized_input.chars().count());
    let mut hex_only: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &input_len {
        let ch_opt: Option<String> = Some({
            let __indexed_char_option = normalized_input
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-" {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only.push_str((_to_lower_hex_char(&ch)).as_str());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    if (&SifrInt::from(hex_only.chars().count()) != &SifrInt::from_i64(32)) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if &input_len == &SifrInt::from_i64(36) {
        let h1: Option<String> = Some({
            let __indexed_char_option = normalized_input
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(8))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        let h2: Option<String> = Some({
            let __indexed_char_option = normalized_input
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(13))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        let h3: Option<String> = Some({
            let __indexed_char_option = normalized_input
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(18))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        let h4: Option<String> = Some({
            let __indexed_char_option = normalized_input
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(23))))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if (((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string()))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if &input_len != &SifrInt::from_i64(32) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: SifrInt = SifrInt::from_i64(0);
    while (&j < &SifrInt::from(hex_only.chars().count())) {
        if (((&j == &SifrInt::from_i64(8)) || (&j == &SifrInt::from_i64(12)))
            || (&j == &SifrInt::from_i64(16))) || (&j == &SifrInt::from_i64(20))
        {
            canonical.push('-');
        }
        let part: Option<String> = Some({
            let __indexed_char_option = hex_only
                .chars()
                .nth(::sifr_runtime::to_usize_proven(&(j)))
                .map(|c| c.to_string());
            __indexed_char_option.as_slice()[0_usize].clone()
        });
        if let Some(part) = part {
            canonical.push_str((part).as_str());
        }
        j = &j + &SifrInt::from_i64(1);
    }
    Ok(canonical)
}
fn uuid4_obj() -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid4())
}
fn uuid_from_hex(
    hex_str: &String,
) -> Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError>,
        ValueError,
    > = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        Ok(Ok(__SifrStdlib_sifr_x2euuid_x2eUUID::new(canonical)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message.clone()));
        }
    }
}
fn uuid3(
    namespace: &__SifrStdlib_sifr_x2euuid_x2eUUID,
    name: &String,
) -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid3_text(&namespace.to_str(), name))
}
fn uuid5(
    namespace: &__SifrStdlib_sifr_x2euuid_x2eUUID,
    name: &String,
) -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid5_text(&namespace.to_str(), name))
}
fn NAMESPACE_DNS() -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
    )
}
fn is_canonical_shape(value: &String) -> bool {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_value.len()) != &SifrInt::from_i64(36)) {
        return false;
    }
    let h1: Option<String> = __sifr_chars_value
        .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(8))))
        .map(|c| c.to_string());
    let h2: Option<String> = __sifr_chars_value
        .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(13))))
        .map(|c| c.to_string());
    let h3: Option<String> = __sifr_chars_value
        .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(18))))
        .map(|c| c.to_string());
    let h4: Option<String> = __sifr_chars_value
        .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(23))))
        .map(|c| c.to_string());
    (((((h1 == Some("-".to_string()))) && ((h2 == Some("-".to_string()))))
        && ((h3 == Some("-".to_string())))) && ((h4 == Some("-".to_string()))))
}
fn collect_generated_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let id_text: String = uuid4();
    let __sifr_chars_id_text: Vec<char> = id_text.chars().collect::<Vec<char>>();
    actual.push(is_canonical_shape(&id_text));
    actual
        .push(
            __sifr_chars_id_text
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(14))))
                .map(|c| c.to_string()) == Some("4".to_string()),
        );
    let obj: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid4_obj();
    actual
        .push(
            is_canonical_shape(&obj.to_str())
                && (&obj.version() == &SifrInt::from_i64(4)),
        );
    actual
}
fn collect_parse_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"550E8400E29B41D4A716446655440000".to_string(),
        )?;
        parsed_ok = (parsed.to_str() == "550e8400-e29b-41d4-a716-446655440000");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parsed_v1_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed_v1: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"550e8400-e29b-11d4-a716-446655440000".to_string(),
        )?;
        parsed_v1_ok = (&parsed_v1.version() == &SifrInt::from_i64(1));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parsed_v1_ok = false;
    }
    actual.push(parsed_v1_ok);
    actual
}
fn collect_negative_and_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _bad: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"invalid".to_string(),
        )?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        invalid_rejected = true;
    }
    actual.push(invalid_rejected);
    let ctor_passthrough: __SifrStdlib_sifr_x2euuid_x2eUUID = __SifrStdlib_sifr_x2euuid_x2eUUID::new(
        "550e8400-e29b-41d4-a716-44665544000z".to_string(),
    );
    actual
        .push(
            (ctor_passthrough.to_str()).as_str()
                == ("550e8400-e29b-41d4-a716-44665544000z".to_string()).as_str(),
        );
    let mut ctor_curly_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let ctor_curly: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(
            &"{550E8400-E29B-41D4-A716-446655440000}".to_string(),
        )?;
        ctor_curly_ok = (ctor_curly.to_str() == "550e8400-e29b-41d4-a716-446655440000");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        ctor_curly_ok = false;
    }
    actual.push(ctor_curly_ok);
    let obj: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid4_obj();
    actual.push(&SifrInt::from(obj.hex().chars().count()) == &SifrInt::from_i64(32));
    actual
        .push(
            &uuid3(&NAMESPACE_DNS(), &"python.org".to_string()).version()
                == &SifrInt::from_i64(3),
        );
    actual
        .push(
            &uuid5(&NAMESPACE_DNS(), &"python.org".to_string()).version()
                == &SifrInt::from_i64(5),
        );
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![
        true, true, true, true, true, true, true, true, true, true, true
    ];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_generated_actual());
    append_all(&mut actual, &collect_parse_actual());
    append_all(&mut actual, &collect_negative_and_class_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("uuid uuid parity demo: pass");
}

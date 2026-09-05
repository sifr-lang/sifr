// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        ParseError, SifrGeneratedStdlibSifrX2ehashlibX2eHashObject,
        SifrGeneratedStdlibSifrX2etimeX2estructTime, ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn b32encode(s: &str) -> String {
        ::sifr_stdlib::base64::b32encode(s)
    }
    pub(super) fn b32decode(s: &str) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::b32decode(s).map_err(|sifr_generated_bridge_error| ParseError {
            message: sifr_generated_bridge_error,
        })
    }
    pub(super) fn sha256_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    pub(super) fn md5_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    pub(super) fn sha1_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    pub(super) fn sha224_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    pub(super) fn sha384_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    pub(super) fn sha512_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    pub(super) fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    pub(super) fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
    pub(super) fn disk_usage(path: &str) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(super) fn sifr_generated_build_hash(
        algorithm: &str,
        data: &[u8],
    ) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        let alg: String = algorithm.to_lowercase();
        if alg == "md5" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "md5".to_string(),
                &SifrInt::from_i64(16),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha1" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha1".to_string(),
                &SifrInt::from_i64(20),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha224" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha224".to_string(),
                &SifrInt::from_i64(28),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha256" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha256".to_string(),
                &SifrInt::from_i64(32),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha384" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha384".to_string(),
                &SifrInt::from_i64(48),
                &SifrInt::from_i64(128),
            );
        } else if alg == "sha512" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha512".to_string(),
                &SifrInt::from_i64(64),
                &SifrInt::from_i64(128),
            );
        } else if alg == "blake2b" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2b".to_string(),
                &SifrInt::from_i64(64),
                &SifrInt::from_i64(128),
            );
        } else if alg == "blake2s" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2s".to_string(),
                &SifrInt::from_i64(32),
                &SifrInt::from_i64(64),
            );
        }
        SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
            alg,
            data.to_vec(),
            "unknown".to_string(),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
        )
    }
    pub(super) fn sifr_generated_hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
        if algorithm == "md5" {
            return md5_bytes(data);
        } else if algorithm == "sha1" {
            return sha1_bytes(data);
        } else if algorithm == "sha224" {
            return sha224_bytes(data);
        } else if algorithm == "sha256" {
            return sha256_bytes(data);
        } else if algorithm == "sha384" {
            return sha384_bytes(data);
        } else if algorithm == "sha512" {
            return sha512_bytes(data);
        } else if algorithm == "blake2b" {
            return blake2b_bytes(data);
        } else if algorithm == "blake2s" {
            return blake2s_bytes(data);
        }
        Vec::<u8>::new()
    }
    pub(super) fn sifr_generated_hash_hex(algorithm: &str, data: &[u8]) -> String {
        {
            let sifr_generated_bytes_receiver: &[u8] = &sifr_generated_hash_bytes(algorithm, data);
            let mut sifr_generated_hex =
                String::with_capacity(sifr_generated_bytes_receiver.len().saturating_mul(2_usize));
            for sifr_generated_byte in sifr_generated_bytes_receiver {
                let _ = ::std::fmt::Write::write_fmt(
                    &mut sifr_generated_hex,
                    format_args!("{:02x}", *sifr_generated_byte),
                );
            }
            sifr_generated_hex
        }
    }
    pub(super) fn sha224(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("sha224", data)
    }
    pub(super) fn sha384(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("sha384", data)
    }
    pub(super) fn blake2b(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("blake2b", data)
    }
    pub(super) fn blake2s(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("blake2s", data)
    }
    pub(super) fn erf(x: f64) -> f64 {
        ::sifr_stdlib::math::erf(x)
    }
    pub(super) fn erfc(x: f64) -> f64 {
        ::sifr_stdlib::math::erfc(x)
    }
    pub(super) fn gamma(x: f64) -> f64 {
        ::sifr_stdlib::math::gamma(x)
    }
    pub(super) fn lgamma(x: f64) -> f64 {
        ::sifr_stdlib::math::lgamma(x)
    }
    pub(super) fn frexp(x: f64) -> Vec<f64> {
        ::sifr_stdlib::math::frexp(x)
    }
    pub(super) fn ldexp(m: f64, e: SifrInt) -> f64 {
        ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
    }
    pub(super) fn modf(x: f64) -> Vec<f64> {
        ::sifr_stdlib::math::modf(x)
    }
    pub(super) fn nextafter(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::nextafter(x, y)
    }
    pub(super) fn ulp(x: f64) -> f64 {
        ::sifr_stdlib::math::ulp(x)
    }
    pub(super) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
    }
    pub(super) fn cpu_count() -> SifrInt {
        ::sifr_stdlib::sys::cpu_count().into_sifr_int()
    }
    pub(super) fn platform_system() -> String {
        ::sifr_stdlib::platform::platform_system()
    }
    pub(super) fn platform_arch() -> String {
        ::sifr_stdlib::platform::platform_arch()
    }
    pub(super) fn platform_processor() -> String {
        ::sifr_stdlib::platform::platform_processor()
    }
    pub(super) fn system() -> String {
        platform_system()
    }
    pub(super) fn machine() -> String {
        platform_arch()
    }
    pub(super) fn processor() -> String {
        platform_processor()
    }
    pub(super) fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt).map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error,
        })
    }
    pub(super) fn sifr_generated_gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub(super) fn sifr_generated_localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub(super) fn sifr_generated_is_leap_year(year: &SifrInt) -> bool {
        year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == SifrInt::from_i64(0)
            && year.floor_mod_known_nonzero(&SifrInt::from_i64(100)) != SifrInt::from_i64(0)
            || year.floor_mod_known_nonzero(&SifrInt::from_i64(400)) == SifrInt::from_i64(0)
    }
    pub(super) fn sifr_generated_days_in_year(year: &SifrInt) -> SifrInt {
        if sifr_generated_is_leap_year(year) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub(super) fn sifr_generated_days_in_month(year: &SifrInt, month: &SifrInt) -> SifrInt {
        let month_days: Vec<SifrInt> = vec![
            SifrInt::from_i64(31),
            SifrInt::from_i64(28),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
        ];
        let idx: SifrInt = ::std::ops::Sub::sub(month, &SifrInt::from_i64(1));
        let d: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &month_days;
            let sifr_generated_checked_read_index = &idx;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if month == &SifrInt::from_i64(2) && sifr_generated_is_leap_year(year) {
            return SifrInt::from_i64(29);
        }
        let Some(d) = d else {
            return SifrInt::from_i64(0);
        };
        d
    }
    pub(super) fn sifr_generated_substring(value: &str, start: &SifrInt, end: &SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = (*start).clone();
        while &i < end {
            let ch: Option<String> = {
                let sifr_generated_string_index = &i;
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
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn sifr_generated_digit_value(ch: &str) -> Option<SifrInt> {
        if ch == "0" {
            return Some(SifrInt::from_i64(0));
        }
        if ch == "1" {
            return Some(SifrInt::from_i64(1));
        }
        if ch == "2" {
            return Some(SifrInt::from_i64(2));
        }
        if ch == "3" {
            return Some(SifrInt::from_i64(3));
        }
        if ch == "4" {
            return Some(SifrInt::from_i64(4));
        }
        if ch == "5" {
            return Some(SifrInt::from_i64(5));
        }
        if ch == "6" {
            return Some(SifrInt::from_i64(6));
        }
        if ch == "7" {
            return Some(SifrInt::from_i64(7));
        }
        if ch == "8" {
            return Some(SifrInt::from_i64(8));
        }
        if ch == "9" {
            return Some(SifrInt::from_i64(9));
        }
        None
    }
    pub(super) fn sifr_generated_parse_decimal(text: &str) -> Option<SifrInt> {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if sifr_generated_chars_text.len() == SifrInt::from_i64(0) {
            return None;
        }
        let mut out: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < sifr_generated_chars_text.len() {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = &i;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_text.len());
                sifr_generated_chars_text
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let ch_opt_value_58c5362056f71db8 = ch_opt?;
            let ch: String = ch_opt_value_58c5362056f71db8;
            let digit_opt: Option<SifrInt> = sifr_generated_digit_value(ch.as_str());
            let digit_opt_value_c39685cb2782ed00 = digit_opt?;
            let digit: SifrInt = digit_opt_value_c39685cb2782ed00;
            out = ::std::ops::Add::add(&::std::ops::Mul::mul(&out, &SifrInt::from_i64(10)), &digit);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        Some(out)
    }
    pub(super) fn sifr_generated_int_or_negative_one(value: Option<&SifrInt>) -> SifrInt {
        let value: Option<SifrInt> = value.cloned();
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return ::std::ops::Neg::neg(&SifrInt::from_i64(1));
        };
        value_value_7ce4fd9430e80cea
    }
    pub(super) fn sifr_generated_day_of_year(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> SifrInt {
        let mut yday: SifrInt = SifrInt::from_i64(0);
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < month {
            yday = ::std::ops::Add::add(&yday, &sifr_generated_days_in_month(year, &m));
            m = ::std::ops::Add::add(&m, &SifrInt::from_i64(1));
        }
        ::std::ops::Add::add(&yday, day)
    }
    pub(super) fn sifr_generated_weekday(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> SifrInt {
        let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
        if year >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < year {
                days_since_epoch =
                    ::std::ops::Add::add(&days_since_epoch, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Add::add(&y, &SifrInt::from_i64(1));
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= year {
                days_since_epoch =
                    ::std::ops::Sub::sub(&days_since_epoch, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Sub::sub(&y, &SifrInt::from_i64(1));
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < month {
            days_since_epoch =
                ::std::ops::Add::add(&days_since_epoch, &sifr_generated_days_in_month(year, &m));
            m = ::std::ops::Add::add(&m, &SifrInt::from_i64(1));
        }
        days_since_epoch = ::std::ops::Sub::sub(
            &::std::ops::Add::add(&days_since_epoch, day),
            &SifrInt::from_i64(1),
        );
        let mut wd: SifrInt = ::std::ops::Add::add(&SifrInt::from_i64(3), &days_since_epoch)
            .floor_mod_known_nonzero(&SifrInt::from_i64(7));
        if wd < SifrInt::from_i64(0) {
            wd = ::std::ops::Add::add(&wd, &SifrInt::from_i64(7));
        }
        wd
    }
    pub(super) fn sifr_generated_valid_date(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> bool {
        if year <= &SifrInt::from_i64(0) {
            return false;
        }
        if month < &SifrInt::from_i64(1) || month > &SifrInt::from_i64(12) {
            return false;
        }
        let max_day: SifrInt = sifr_generated_days_in_month(year, month);
        day >= &SifrInt::from_i64(1) && day <= &max_day
    }
    pub(super) fn sifr_generated_invalid_struct_time() -> SifrGeneratedStdlibSifrX2etimeX2estructTime
    {
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
        )
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_to_struct_time(
        rendered: &str,
    ) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let sifr_generated_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        let Some(_checked_value_3) = {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_4) = {
            let sifr_generated_string_index = SifrInt::from_i64(7);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_5) = {
            let sifr_generated_string_index = SifrInt::from_i64(10);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_6) = {
            let sifr_generated_string_index = SifrInt::from_i64(13);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_7) = {
            let sifr_generated_string_index = SifrInt::from_i64(16);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        if {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(7);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(10);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('T'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(16);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
        {
            return sifr_generated_invalid_struct_time();
        }
        let year: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(0),
                &SifrInt::from_i64(4),
            ))
            .as_ref(),
        );
        let month: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(5),
                &SifrInt::from_i64(7),
            ))
            .as_ref(),
        );
        let day: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(8),
                &SifrInt::from_i64(10),
            ))
            .as_ref(),
        );
        let hour: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(11),
                &SifrInt::from_i64(13),
            ))
            .as_ref(),
        );
        let minute: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(14),
                &SifrInt::from_i64(16),
            ))
            .as_ref(),
        );
        let second: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(17),
                &SifrInt::from_i64(19),
            ))
            .as_ref(),
        );
        if year < SifrInt::from_i64(0)
            || month < SifrInt::from_i64(0)
            || day < SifrInt::from_i64(0)
            || hour < SifrInt::from_i64(0)
            || minute < SifrInt::from_i64(0)
            || second < SifrInt::from_i64(0)
        {
            return sifr_generated_invalid_struct_time();
        }
        if !sifr_generated_valid_date(&year, &month, &day) {
            return sifr_generated_invalid_struct_time();
        }
        let wday: SifrInt = sifr_generated_weekday(&year, &month, &day);
        let yday_value_75753d4973d2a3ce: SifrInt = sifr_generated_day_of_year(&year, &month, &day);
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            &year,
            &month,
            &day,
            &hour,
            &minute,
            &second,
            &wday,
            &yday_value_75753d4973d2a3ce,
            &SifrInt::from_i64(0),
        )
    }
    pub(super) fn gmtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_gmtime_intrinsic(epoch);
        sifr_generated_to_struct_time(rendered.as_str())
    }
    pub(super) fn localtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_localtime_intrinsic(epoch);
        sifr_generated_to_struct_time(rendered.as_str())
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::sifr_generated_hash_hex;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        pub algorithm: String,
        pub data: Vec<u8>,
        pub name: String,
        pub digest_size: SifrInt,
        pub block_size: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        #[must_use]
        pub fn new(
            algorithm: String,
            data: Vec<u8>,
            name: String,
            digest_size: &SifrInt,
            block_size: &SifrInt,
        ) -> Self {
            let sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d: String =
                algorithm;
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: Vec<u8> = data;
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            let sifr_generated_field_value_6344303e03c9f7c7_6469676573745f73697a65: SifrInt =
                (*digest_size).clone();
            let sifr_generated_field_value_e190162752f8783e_626c6f636b5f73697a65: SifrInt =
                (*block_size).clone();
            Self {
                algorithm: sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d,
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
                digest_size: sifr_generated_field_value_6344303e03c9f7c7_6469676573745f73697a65,
                block_size: sifr_generated_field_value_e190162752f8783e_626c6f636b5f73697a65,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        #[must_use]
        pub fn hexdigest(&self) -> String {
            sifr_generated_hash_hex(&self.algorithm, &self.data)
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2etimeX2estructTime {
        pub tm_year: SifrInt,
        pub tm_mon: SifrInt,
        pub tm_mday: SifrInt,
        pub tm_hour: SifrInt,
        pub tm_min: SifrInt,
        pub tm_sec: SifrInt,
        pub tm_wday: SifrInt,
        pub tm_yday: SifrInt,
        pub tm_isdst: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            tm_year: &SifrInt,
            tm_mon: &SifrInt,
            tm_mday_argument_a505494cd43c9214: &SifrInt,
            tm_hour: &SifrInt,
            tm_min_argument_103d514d457d4a49: &SifrInt,
            tm_sec: &SifrInt,
            tm_wday_argument_d5143a059ed34c12: &SifrInt,
            tm_yday_argument_6b9a41f3b9220250: &SifrInt,
            tm_isdst: &SifrInt,
        ) -> Self {
            let sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172: SifrInt =
                (*tm_year).clone();
            let sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e: SifrInt =
                (*tm_mon).clone();
            let sifr_generated_field_value_a505494cd43c9214_746d5f6d646179: SifrInt =
                (*tm_mday_argument_a505494cd43c9214).clone();
            let sifr_generated_field_value_129c5b76af381059_746d5f686f7572: SifrInt =
                (*tm_hour).clone();
            let sifr_generated_field_value_103d514d457d4a49_746d5f6d696e: SifrInt =
                (*tm_min_argument_103d514d457d4a49).clone();
            let sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563: SifrInt =
                (*tm_sec).clone();
            let sifr_generated_field_value_d5143a059ed34c12_746d5f77646179: SifrInt =
                (*tm_wday_argument_d5143a059ed34c12).clone();
            let sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179: SifrInt =
                (*tm_yday_argument_6b9a41f3b9220250).clone();
            let sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374: SifrInt =
                (*tm_isdst).clone();
            Self {
                tm_year: sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172,
                tm_mon: sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e,
                tm_mday: sifr_generated_field_value_a505494cd43c9214_746d5f6d646179,
                tm_hour: sifr_generated_field_value_129c5b76af381059_746d5f686f7572,
                tm_min: sifr_generated_field_value_103d514d457d4a49_746d5f6d696e,
                tm_sec: sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563,
                tm_wday: sifr_generated_field_value_d5143a059ed34c12_746d5f77646179,
                tm_yday: sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179,
                tm_isdst: sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn as_tuple(
            &self,
        ) -> (
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
        ) {
            (
                self.tm_year.clone(),
                self.tm_mon.clone(),
                self.tm_mday.clone(),
                self.tm_hour.clone(),
                self.tm_min.clone(),
                self.tm_sec.clone(),
                self.tm_wday.clone(),
                self.tm_yday.clone(),
                self.tm_isdst.clone(),
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn isoformat(&self) -> String {
            let y: String = self.tm_year.to_string();
            let mut mo: String = self.tm_mon.to_string();
            if mo.chars().count() < SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mo.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.tm_mday.to_string();
            if d.chars().count() < SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(d.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.tm_hour.to_string();
            if h.chars().count() < SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(h.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.tm_min.to_string();
            if mi.chars().count() < SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mi.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.tm_sec.to_string();
            if s.chars().count() < SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(s.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    y.len()
                        .saturating_add(1usize)
                        .saturating_add(mo.len())
                        .saturating_add(1usize)
                        .saturating_add(d.len())
                        .saturating_add(1usize)
                        .saturating_add(h.len())
                        .saturating_add(1usize)
                        .saturating_add(mi.len())
                        .saturating_add(1usize)
                        .saturating_add(s.len()),
                );
                sifr_generated_concat.push_str(y.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(mo.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(d.as_str());
                sifr_generated_concat.push('T');
                sifr_generated_concat.push_str(h.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(mi.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(s.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn eq(&self, other: &Self) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use crate::sifr_generated_generated_support::{
    b32decode, b32encode, blake2b, blake2s, cpu_count, disk_usage, erf, erfc, frexp, gamma, getpid,
    gmtime_struct, ldexp, lgamma, localtime_struct, machine, modf, nextafter, processor, sha224,
    sha384, strptime, system, ulp,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ehashlibX2eHashObject;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etimeX2estructTime;
pub use sifr_generated_project_nominals::ValueError;
fn demo_math() {
    println!("=== math new intrinsics ===");
    let e0: f64 = erf(0.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("erf near 0 = ");
        sifr_generated_concat.push_str((e0 < 0.001_f64 && e0 > -0.001_f64).to_string().as_str());
        sifr_generated_concat
    });
    let ec0: f64 = erfc(0.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("erfc near 1 = ");
        sifr_generated_concat.push_str((ec0 > 0.999_f64 && ec0 < 1.001_f64).to_string().as_str());
        sifr_generated_concat
    });
    let g: f64 = gamma(5.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("gamma(5) > 23 = ");
        sifr_generated_concat.push_str((g > 23.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    let lg: f64 = lgamma(5.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("lgamma(5) > 3 = ");
        sifr_generated_concat.push_str((lg > 3.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    let fp: Vec<f64> = frexp(8.0_f64);
    let mantissa: Option<f64> = {
        let sifr_generated_checked_read_collection = &fp;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .copied()
    };
    if let Some(mantissa) = mantissa {
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(22usize.saturating_add(0usize));
            sifr_generated_concat.push_str("frexp(8.0) mantissa = ");
            sifr_generated_concat.push_str(mantissa.to_string().as_str());
            sifr_generated_concat
        });
    }
    let ld: f64 = ldexp(0.5_f64, SifrInt::from_i64(4));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("ldexp(0.5, 4) = ");
        sifr_generated_concat.push_str(ld.to_string().as_str());
        sifr_generated_concat
    });
    let md: Vec<f64> = modf(3.7_f64);
    let frac: Option<f64> = {
        let sifr_generated_checked_read_collection = &md;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .copied()
    };
    if let Some(frac) = frac {
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(21usize.saturating_add(0usize));
            sifr_generated_concat.push_str("modf(3.7) frac > 0 = ");
            sifr_generated_concat.push_str((frac > 0.0_f64).to_string().as_str());
            sifr_generated_concat
        });
    }
    let na: f64 = nextafter(1.0_f64, 2.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(28usize.saturating_add(0usize));
        sifr_generated_concat.push_str("nextafter(1.0, 2.0) > 1.0 = ");
        sifr_generated_concat.push_str((na > 1.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    let u: f64 = ulp(1.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(15usize.saturating_add(0usize));
        sifr_generated_concat.push_str("ulp(1.0) > 0 = ");
        sifr_generated_concat.push_str((u > 0.0_f64).to_string().as_str());
        sifr_generated_concat
    });
}
fn demo_os() {
    println!("=== os new intrinsics ===");
    let pid: SifrInt = getpid();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(10usize.saturating_add(0usize));
        sifr_generated_concat.push_str("pid > 0 = ");
        sifr_generated_concat.push_str((pid > SifrInt::from_i64(0)).to_string().as_str());
        sifr_generated_concat
    });
    let cpus: SifrInt = cpu_count();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(17usize.saturating_add(0usize));
        sifr_generated_concat.push_str("cpu_count >= 1 = ");
        sifr_generated_concat.push_str((cpus >= SifrInt::from_i64(1)).to_string().as_str());
        sifr_generated_concat
    });
}
fn demo_hashlib() {
    println!("=== hashlib new intrinsics ===");
    let data: Vec<u8> = vec![104u8, 101u8, 108u8, 108u8, 111u8];
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("sha224 len = ");
        sifr_generated_concat.push_str(
            SifrInt::from(sha224(&data).hexdigest().chars().count())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("sha384 len = ");
        sifr_generated_concat.push_str(
            SifrInt::from(sha384(&data).hexdigest().chars().count())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("blake2b len = ");
        sifr_generated_concat.push_str(
            SifrInt::from(blake2b(&data).hexdigest().chars().count())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("blake2s len = ");
        sifr_generated_concat.push_str(
            SifrInt::from(blake2s(&data).hexdigest().chars().count())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
}
fn demo_platform() {
    println!("=== platform new intrinsics ===");
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(17usize.saturating_add(0usize));
        sifr_generated_concat.push_str("system len > 0 = ");
        sifr_generated_concat.push_str(
            (system().chars().count() > SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(18usize.saturating_add(0usize));
        sifr_generated_concat.push_str("machine len > 0 = ");
        sifr_generated_concat.push_str(
            (machine().chars().count() > SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("processor len > 0 = ");
        sifr_generated_concat.push_str(
            (processor().chars().count() > SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
}
fn demo_time() {
    println!("=== time new intrinsics ===");
    let gmt: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("gmtime year = ");
        sifr_generated_concat.push_str(
            (gmt.tm_year == SifrInt::from_i64(1970))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let lt: SifrGeneratedStdlibSifrX2etimeX2estructTime = localtime_struct(0.0_f64);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(22usize.saturating_add(0usize));
        sifr_generated_concat.push_str("localtime yday >= 1 = ");
        sifr_generated_concat.push_str((lt.tm_yday >= SifrInt::from_i64(1)).to_string().as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")?;
        let _chars_parsed: Vec<char> = parsed.chars().collect::<Vec<char>>();
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(14usize.saturating_add(0usize));
            sifr_generated_concat.push_str("strptime ok = ");
            sifr_generated_concat.push_str(
                (parsed.chars().count() > SifrInt::from_i64(0))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(16usize.saturating_add(0usize));
            sifr_generated_concat.push_str("strptime error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
}
fn demo_base64() {
    println!("=== base64 new intrinsics ===");
    let encoded: String = b32encode("hello world");
    let _chars_encoded: Vec<char> = encoded.chars().collect::<Vec<char>>();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("b32encode len > 0 = ");
        sifr_generated_concat.push_str(
            (encoded.chars().count() > SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b32decode(encoded.as_str())?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(12usize.saturating_add(decoded.len()));
            sifr_generated_concat.push_str("b32decode = ");
            sifr_generated_concat.push_str(decoded.as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(17usize.saturating_add(0usize));
            sifr_generated_concat.push_str("b32decode error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
}
fn demo_shutil() {
    println!("=== shutil new intrinsics ===");
    let usage: Vec<SifrInt> = disk_usage("/");
    let total: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &usage;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(total) = total {
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(17usize.saturating_add(0usize));
            sifr_generated_concat.push_str("disk_total > 0 = ");
            sifr_generated_concat.push_str((total > SifrInt::from_i64(0)).to_string().as_str());
            sifr_generated_concat
        });
    }
}
fn main() {
    demo_math();
    demo_os();
    demo_hashlib();
    demo_platform();
    demo_time();
    demo_base64();
    demo_shutil();
}

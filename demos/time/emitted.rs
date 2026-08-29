// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &String) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn gmtime(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn _gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn localtime(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn _localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<SifrInt>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn time_localtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2etime_x2estruct__time {
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
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn new(
            tm_year: SifrInt,
            tm_mon: SifrInt,
            tm_mday: SifrInt,
            tm_hour: SifrInt,
            tm_min: SifrInt,
            tm_sec: SifrInt,
            tm_wday: SifrInt,
            tm_yday: SifrInt,
            tm_isdst: SifrInt,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = tm_year.clone();
            let __sifr_field_init_1: SifrInt = tm_mon.clone();
            let __sifr_field_init_2: SifrInt = tm_mday.clone();
            let __sifr_field_init_3: SifrInt = tm_hour.clone();
            let __sifr_field_init_4: SifrInt = tm_min.clone();
            let __sifr_field_init_5: SifrInt = tm_sec.clone();
            let __sifr_field_init_6: SifrInt = tm_wday.clone();
            let __sifr_field_init_7: SifrInt = tm_yday.clone();
            let __sifr_field_init_8: SifrInt = tm_isdst.clone();
            Self {
                tm_year: __sifr_field_init_0,
                tm_mon: __sifr_field_init_1,
                tm_mday: __sifr_field_init_2,
                tm_hour: __sifr_field_init_3,
                tm_min: __sifr_field_init_4,
                tm_sec: __sifr_field_init_5,
                tm_wday: __sifr_field_init_6,
                tm_yday: __sifr_field_init_7,
                tm_isdst: __sifr_field_init_8,
            }
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
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
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.tm_year.clone());
            let mut mo: String = format!("{}", self.tm_mon.clone());
            if (&SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.tm_mday.clone());
            if (&SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.tm_hour.clone());
            if (&SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.tm_min.clone());
            if (&SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.tm_sec.clone());
            if (&SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2)) {
                s = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((s).as_str());
                    __sifr_concat
                };
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    (((((((((y.len() + 1usize) + mo.len()) + 1usize) + d.len()) + 1usize)
                        + h.len()) + 1usize) + mi.len()) + 1usize) + s.len(),
                );
                __sifr_concat.push_str((y).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat.push('T');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((s).as_str());
                __sifr_concat
            }
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn eq(&self, other: &__SifrStdlib_sifr_x2etime_x2estruct__time) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        pub fn new(message: String) -> Self {
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
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
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
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etime_x2estruct__time;

mod __sifr_project_unions {
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        __SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(crate::__sifr_project_nominals::Error),
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            crate::__sifr_project_nominals::FloatOverflowError,
        ),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            crate::__sifr_project_nominals::FloatPrecisionLossError,
        ),
        __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
            crate::__sifr_project_nominals::ValueError,
        ),
    }
    impl From<crate::__sifr_project_nominals::Error>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::Error) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::FloatPrecisionLossError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatPrecisionLossError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::ValueError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::ValueError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0;
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
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<SifrInt>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn time_localtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
    __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(ValueError),
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
fn _is_leap_year(year: SifrInt) -> bool {
    (((&year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
        && (&year.floor_mod_known_nonzero(&SifrInt::from_i64(100))
            != &SifrInt::from_i64(0)))
        || ((&year.floor_mod_known_nonzero(&SifrInt::from_i64(400))
            == &SifrInt::from_i64(0))))
}
fn _days_in_year(year: SifrInt) -> SifrInt {
    if _is_leap_year((year).clone()) {
        return SifrInt::from_i64(366);
    }
    SifrInt::from_i64(365)
}
fn _days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
    let month_days: Vec<SifrInt> = vec![
        SifrInt::from_i64(31), SifrInt::from_i64(28), SifrInt::from_i64(31),
        SifrInt::from_i64(30), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(30), SifrInt::from_i64(31)
    ];
    let idx: SifrInt = &month - &SifrInt::from_i64(1);
    let d: Option<SifrInt> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx.clone();
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    if (&month == &SifrInt::from_i64(2)) && _is_leap_year((year).clone()) {
        return SifrInt::from_i64(29);
    }
    if let Some(d) = d.clone() {
        return d;
    }
    SifrInt::from_i64(0)
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
fn _digit_value(ch: &String) -> Option<SifrInt> {
    if (ch).as_str() == "0" {
        return Some(SifrInt::from_i64(0));
    }
    if (ch).as_str() == "1" {
        return Some(SifrInt::from_i64(1));
    }
    if (ch).as_str() == "2" {
        return Some(SifrInt::from_i64(2));
    }
    if (ch).as_str() == "3" {
        return Some(SifrInt::from_i64(3));
    }
    if (ch).as_str() == "4" {
        return Some(SifrInt::from_i64(4));
    }
    if (ch).as_str() == "5" {
        return Some(SifrInt::from_i64(5));
    }
    if (ch).as_str() == "6" {
        return Some(SifrInt::from_i64(6));
    }
    if (ch).as_str() == "7" {
        return Some(SifrInt::from_i64(7));
    }
    if (ch).as_str() == "8" {
        return Some(SifrInt::from_i64(8));
    }
    if (ch).as_str() == "9" {
        return Some(SifrInt::from_i64(9));
    }
    None
}
fn _parse_decimal(text: &String) -> Option<SifrInt> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_text.len()) == &SifrInt::from_i64(0)) {
        return None;
    }
    let mut out: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let Some(ch_opt) = ch_opt else {
            return None;
        };
        let ch: String = ch_opt;
        let digit_opt: Option<SifrInt> = _digit_value(&ch);
        let Some(digit_opt) = digit_opt.clone() else {
            return None;
        };
        let digit: SifrInt = digit_opt.clone();
        out = &(&out * &SifrInt::from_i64(10)) + &digit;
        i = &i + &SifrInt::from_i64(1);
    }
    Some(out)
}
fn _int_or_negative_one(value: Option<SifrInt>) -> SifrInt {
    let Some(value) = value.clone() else {
        return -&SifrInt::from_i64(1);
    };
    value.clone()
}
fn _day_of_year(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    let mut yday: SifrInt = SifrInt::from_i64(0);
    let mut m: SifrInt = SifrInt::from_i64(1);
    while &m < &month {
        yday = &yday + &_days_in_month((year).clone(), (m).clone());
        m = &m + &SifrInt::from_i64(1);
    }
    &yday + &day
}
fn _weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
    if &year >= &SifrInt::from_i64(1970) {
        let mut y: SifrInt = SifrInt::from_i64(1970);
        while &y < &year {
            days_since_epoch = &days_since_epoch + &_days_in_year((y).clone());
            y = &y + &SifrInt::from_i64(1);
        }
    } else {
        let mut y: SifrInt = SifrInt::from_i64(1969);
        while &y >= &year {
            days_since_epoch = &days_since_epoch - &_days_in_year((y).clone());
            y = &y - &SifrInt::from_i64(1);
        }
    }
    let mut m: SifrInt = SifrInt::from_i64(1);
    while &m < &month {
        days_since_epoch = &days_since_epoch
            + &_days_in_month((year).clone(), (m).clone());
        m = &m + &SifrInt::from_i64(1);
    }
    days_since_epoch = &(&days_since_epoch + &day) - &SifrInt::from_i64(1);
    let mut wd: SifrInt = (&SifrInt::from_i64(3) + &days_since_epoch)
        .floor_mod_known_nonzero(&SifrInt::from_i64(7));
    if &wd < &SifrInt::from_i64(0) {
        wd = &wd + &SifrInt::from_i64(7);
    }
    wd.clone()
}
fn _valid_date(year: SifrInt, month: SifrInt, day: SifrInt) -> bool {
    if &year <= &SifrInt::from_i64(0) {
        return false;
    }
    if (&month < &SifrInt::from_i64(1)) || (&month > &SifrInt::from_i64(12)) {
        return false;
    }
    let max_day: SifrInt = _days_in_month((year).clone(), (month).clone());
    (&day >= &SifrInt::from_i64(1)) && (&day <= &max_day)
}
fn _invalid_struct_time() -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
    )
}
fn _to_struct_time(rendered: &String) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_rendered.len()) < &SifrInt::from_i64(19)) {
        return _invalid_struct_time();
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_rendered
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(4))))
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(7))))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(10))))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(13))))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(16))))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return _invalid_struct_time();
    }
    let year: SifrInt = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, SifrInt::from_i64(0), SifrInt::from_i64(4))),
    );
    let month: SifrInt = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, SifrInt::from_i64(5), SifrInt::from_i64(7))),
    );
    let day: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(8), SifrInt::from_i64(10)),
        ),
    );
    let hour: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(11), SifrInt::from_i64(13)),
        ),
    );
    let minute: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(14), SifrInt::from_i64(16)),
        ),
    );
    let second: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(17), SifrInt::from_i64(19)),
        ),
    );
    if (((((&year < &SifrInt::from_i64(0)) || (&month < &SifrInt::from_i64(0)))
        || (&day < &SifrInt::from_i64(0))) || (&hour < &SifrInt::from_i64(0)))
        || (&minute < &SifrInt::from_i64(0))) || (&second < &SifrInt::from_i64(0))
    {
        return _invalid_struct_time();
    }
    if !(_valid_date((year).clone(), (month).clone(), (day).clone())) {
        return _invalid_struct_time();
    }
    let wday: SifrInt = _weekday((year).clone(), (month).clone(), (day).clone());
    let yday: SifrInt = _day_of_year((year).clone(), (month).clone(), (day).clone());
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        (year).clone(),
        (month).clone(),
        (day).clone(),
        (hour).clone(),
        (minute).clone(),
        (second).clone(),
        (wday).clone(),
        (yday).clone(),
        SifrInt::from_i64(0),
    )
}
fn time() -> f64 {
    time_now()
}
fn strftime(fmt: &String, epoch: f64) -> String {
    time_format(epoch, fmt)
}
fn gmtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _gmtime_intrinsic(epoch);
    _to_struct_time(&rendered)
}
fn localtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _localtime_intrinsic(epoch);
    _to_struct_time(&rendered)
}
fn mktime(
    t: &__SifrStdlib_sifr_x2etime_x2estruct__time,
) -> Result<
    f64,
    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
> {
    if !(_valid_date(t.tm_year.clone(), t.tm_mon.clone(), t.tm_mday.clone())) {
        return Err(
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                ValueError::new("mktime() received an invalid calendar date".to_string()),
            ),
        );
    }
    let mut days: SifrInt = SifrInt::from_i64(0);
    if (&t.tm_year.clone() >= &SifrInt::from_i64(1970)) {
        let mut y: SifrInt = SifrInt::from_i64(1970);
        while (&y < &t.tm_year.clone()) {
            days = &days + &_days_in_year((y).clone());
            y = &y + &SifrInt::from_i64(1);
        }
    } else {
        let mut y: SifrInt = SifrInt::from_i64(1969);
        while (&y >= &t.tm_year.clone()) {
            days = &days - &_days_in_year((y).clone());
            y = &y - &SifrInt::from_i64(1);
        }
    }
    let mut m: SifrInt = SifrInt::from_i64(1);
    while (&m < &t.tm_mon.clone()) {
        days = &days + &_days_in_month(t.tm_year.clone(), (m).clone());
        m = &m + &SifrInt::from_i64(1);
    }
    days = &(&days + &t.tm_mday.clone()) - &SifrInt::from_i64(1);
    let stamp: SifrInt = &(&(&(&days * &SifrInt::from_i64(86400))
        + &(&t.tm_hour.clone() * &SifrInt::from_i64(3600)))
        + &(&t.tm_min.clone() * &SifrInt::from_i64(60))) + &t.tm_sec.clone();
    (stamp
        .clone()
        .checked_to_f64()
        .map_err(|__sifr_float_error| match __sifr_float_error {
            ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    FloatOverflowError::new(
                        "exact integer is outside the finite float range".to_string(),
                    ),
                )
            }
            ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    FloatPrecisionLossError::new(
                        "exact integer cannot be represented without float precision loss"
                            .to_string(),
                    ),
                )
            }
        }))
        .map_err(|__sifr_error_value| match __sifr_error_value {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                __sifr_union_value,
            ) => {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                )
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                __sifr_union_value,
            ) => {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                )
            }
        })
}
fn collect_clock_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(time() > (0.0_f64));
    let perf_before: f64 = perf_counter();
    let mono_before: f64 = monotonic();
    sleep(0.01_f64);
    let perf_after: f64 = perf_counter();
    let mono_after: f64 = monotonic();
    actual.push((perf_after >= perf_before) && (mono_after >= mono_before));
    actual
}
fn collect_format_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            (strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0_f64)).as_str()
                == ("1970-01-01 00:00:00".to_string()).as_str(),
        );
    let gmt: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    actual
        .push(
            (((((&gmt.tm_year.clone() == &SifrInt::from_i64(1970))
                && (&gmt.tm_mon.clone() == &SifrInt::from_i64(1)))
                && (&gmt.tm_mday.clone() == &SifrInt::from_i64(1)))
                && (&gmt.tm_hour.clone() == &SifrInt::from_i64(0)))
                && (&gmt.tm_min.clone() == &SifrInt::from_i64(0)))
                && (&gmt.tm_sec.clone() == &SifrInt::from_i64(0)),
        );
    let local: __SifrStdlib_sifr_x2etime_x2estruct__time = localtime_struct(0.0_f64);
    actual
        .push(
            (&local.tm_year.clone() > &SifrInt::from_i64(0))
                && (&local.tm_yday.clone() >= &SifrInt::from_i64(1)),
        );
    actual
}
fn collect_parse_and_safety_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime(
            &"2024-01-15 10:30:00".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        parsed_ok = parsed == "2024-01-15T10:30:00";
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parse_error_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _bad: String = strptime(
            &"bad".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parse_error_ok = true;
    }
    actual.push(parse_error_ok);
    sleep(-(0.05_f64));
    actual.push(true);
    let epoch_tm: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0,
    > = (|| {
        let epoch_stamp: f64 = (mktime(&epoch_tm))
            .map_err(|__e| match __e {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __sifr_union_value,
                    )
                }
            })?;
        actual.push(epoch_stamp == (0.0_f64));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = __sifr_try_variant_error.clone();
                actual.push(false);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = Error::new(__sifr_try_variant_error.clone().message);
                actual.push(false);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = Error::new(__sifr_try_variant_error.clone().message);
                actual.push(false);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a423_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let _e = Error::new(__sifr_try_variant_error.clone().message);
                actual.push(false);
            }
        }
    }
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_clock_actual());
    append_all(&mut actual, &collect_format_actual());
    append_all(&mut actual, &collect_parse_and_safety_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("time time parity demo: pass");
}
